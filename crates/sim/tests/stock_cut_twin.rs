//! The f32 cut twin (#2626, GPU stock sim S1) against the f64 field
//! cut: one-sided parity on the same moves, the order-preserving key,
//! the move-order law and the refusals.
//! Provenance: this codebase's own f64 field cut (`SdfField::
//! subtract_batch`, `preview::pack_cut`) as the reference; nothing
//! from vendor binaries.
//!
//! Gate map: covers camcraft_sim::stock_sim::cut_twin and
//! camcraft_sim::stock_sim::cut_twin_work.

use camcraft_sim::cl::{Move, MoveKind};
use camcraft_sim::geometry::{cube, Point3};
use camcraft_sim::stock_sim::preview::{pack_cut, pack_field, LatticeMeta};
use camcraft_sim::stock_sim::{
    from_ordered_bits, ordered_bits, twin_cut, twin_work, SdfField, SdfTool, StockError,
};

fn p(x: f64, y: f64, z: f64) -> Point3 {
    Point3::new(x, y, z)
}

fn feed(from: Point3, to: Point3) -> Move {
    Move { rotary: None, kind: MoveKind::Feed, from, to, feed: Some(500.0), arc: None, axis: None }
}

/// A 16 x 16 x 8 block on a cell that is not a power of two (0.7),
/// so the tile-relative rounding is real; band 4 -> cap 2.8.
fn block() -> (SdfField, LatticeMeta) {
    let f = SdfField::from_mesh(&cube(p(0.0, 0.0, 0.0), p(16.0, 16.0, 8.0)).facets, 0.7, 4).expect("the block builds");
    let meta = pack_field(&f).expect("the block packs").meta;
    (f, meta)
}

/// A level feed, a plunge, two ramps (the flat's golden-section path)
/// and a rapid that must cut nothing.
fn program() -> Vec<Move> {
    let mut rapid = feed(p(-5.0, -5.0, 3.0), p(20.0, 20.0, 3.0));
    rapid.kind = MoveKind::Rapid;
    vec![
        feed(p(2.0, 3.0, 5.0), p(13.0, 4.0, 5.0)),
        feed(p(13.0, 4.0, 5.0), p(6.0, 12.0, 3.5)),
        feed(p(6.0, 12.0, 9.0), p(6.0, 12.0, 4.0)),
        feed(p(1.0, 1.0, 7.0), p(15.0, 14.0, 2.0)),
        rapid,
    ]
}

const BALL: SdfTool = SdfTool::Ball { r: 2.5 };
const FLAT: SdfTool = SdfTool::Flat { r: 3.0, flank: 10.0 };

/// The f64 reference: the field's own cut, packed (f64 -> f32 rounds
/// to nearest, which keeps order, so the one-sided bound stays exact).
fn reference(tool: &SdfTool, moves: &[Move]) -> (Vec<f32>, LatticeMeta) {
    let (mut f, meta) = block();
    f.subtract_batch(moves, tool, 0.01).expect("the f64 cut");
    (pack_cut(&f, &meta), meta)
}

fn parity(tool: SdfTool) {
    let moves = program();
    let (want, meta) = reference(&tool, &moves);
    let got = twin_cut(&moves, &tool, &meta).expect("the twin cuts");
    assert_eq!(got.len(), want.len());
    // A cut below -cap never shows: the mesh layer is at least -cap,
    // and the march reads max(mesh, cut). Both sides clamp there, so a
    // claim that sits just at the cap threshold on one side only
    // compares by its visible value.
    let cap = meta.cap as f32;
    // Lower side: the twin margin is TWIN_ULPS (32) f32 epsilons of the
    // tile scale (reach + r + flank + 8 cells, under 40 mm here); the
    // f32 distance error is inside that margin, so the gap is at most
    // twice it. 96 eps * 64 mm covers that with room. The flat's
    // golden-section path adds 1e-6 of its length (under 20 mm) to the
    // margin: 1e-4 covers twice that.
    let tol = 96.0 * f32::EPSILON * 64.0 + 1e-4;
    let (mut claimed, mut deep) = (0, 0);
    for (n, (g, w)) in got.iter().zip(want.iter()).enumerate() {
        assert!(g.is_finite() && w.is_finite(), "texel {n}: {g} / {w}");
        let (g, w) = (g.max(-cap), w.max(-cap));
        assert!(g <= w, "texel {n}: the twin removes more than the f64 cut: {g} > {w}");
        assert!(g >= w - tol, "texel {n}: the twin lags the f64 cut by more than {tol}: {g} < {w}");
        if w > -cap {
            claimed += 1;
        }
        if w >= 1.0 {
            deep += 1;
        }
    }
    // fixture sanity: the program claims many cells, and many of them
    // lie more than 1 mm inside the swept solid (the ball's 2.5 mm
    // radius never reaches the 2.8 mm cap, so "full" is no test here)
    assert!(claimed > 200 && deep > 200, "{tool:?}: {claimed} claimed and {deep} deep texels");
}

#[test]
fn the_ball_twin_never_removes_more_than_the_f64_cut() {
    parity(BALL);
}

#[test]
fn the_flat_twin_never_removes_more_than_the_f64_cut() {
    parity(FLAT);
}

#[test]
fn the_ordered_key_keeps_the_order_of_negative_zero_and_positive() {
    let vals = [
        f32::NEG_INFINITY,
        -3.0e38,
        -2.5,
        -1.0e-30,
        -f32::from_bits(1), // the smallest negative subnormal
        -0.0,
        0.0,
        f32::from_bits(1),
        1.0e-30,
        1.0,
        2.5,
        3.0e38,
        f32::INFINITY,
    ];
    for w in vals.windows(2) {
        assert!(ordered_bits(w[0]) < ordered_bits(w[1]), "{} vs {}", w[0], w[1]);
    }
    for v in vals {
        assert_eq!(from_ordered_bits(ordered_bits(v)).to_bits(), v.to_bits(), "round trip of {v}");
    }
}

#[test]
fn the_batch_answers_bit_for_bit_in_any_move_order() {
    let (_, meta) = block();
    for tool in [BALL, FLAT] {
        let moves = program();
        let base = twin_cut(&moves, &tool, &meta).expect("the twin cuts");
        let mut rev = moves.clone();
        rev.reverse();
        let mut rot = moves.clone();
        rot.rotate_left(2);
        for other in [rev, rot] {
            let got = twin_cut(&other, &tool, &meta).expect("the twin cuts");
            let same = base.iter().zip(got.iter()).all(|(a, b)| a.to_bits() == b.to_bits());
            assert!(same, "{tool:?}: the move order changed the result");
        }
    }
}

#[test]
fn the_work_list_covers_every_claimed_cell_inside_its_bricks() {
    let (_, meta) = block();
    let mut moves = program();
    moves.push(feed(p(200.0, 200.0, 5.0), p(210.0, 200.0, 5.0))); // outside the lattice
    let work = twin_work(&moves, &FLAT, &meta).expect("the work list");
    assert_eq!(work.chords.len(), 5, "four feeds and the far feed; the rapid has no chord");
    assert!(work.items.iter().all(|w| w.chord != 4), "the far feed touches no brick");
    assert!(work.items.windows(2).all(|w| (w[0].brick, w[0].chord) < (w[1].brick, w[1].chord)));
    let hi = |a: usize| meta.origin[a] + meta.dims[a] as i32 - 1;
    for w in &work.items {
        for a in 0..3 {
            let b0 = w.brick[a] * 8;
            assert!(b0 <= w.lo[a] && w.lo[a] <= w.hi[a] && w.hi[a] <= b0 + 7, "{w:?}");
            assert!(meta.origin[a] <= w.lo[a] && w.hi[a] <= hi(a), "{w:?}");
        }
    }
    let cut = twin_cut(&moves, &FLAT, &meta).expect("the twin cuts");
    let floor = (-4.0 * meta.cap) as f32;
    let [dx, dy, _] = meta.dims.map(|d| d as usize);
    for (n, v) in cut.iter().enumerate() {
        if *v == floor {
            continue;
        }
        let key = [
            meta.origin[0] + (n % dx) as i32,
            meta.origin[1] + (n / dx % dy) as i32,
            meta.origin[2] + (n / (dx * dy)) as i32,
        ];
        let covered = work.items.iter().any(|w| (0..3).all(|a| w.lo[a] <= key[a] && key[a] <= w.hi[a]));
        assert!(covered, "claimed cell {key:?} is in no work item");
    }
}

fn refusal(moves: &[Move], tool: &SdfTool, needle: &str) {
    let (_, meta) = block();
    match twin_cut(moves, tool, &meta) {
        Err(StockError::Invalid(msg)) => assert!(msg.contains(needle), "{msg:?} does not name {needle:?}"),
        other => panic!("expected a refusal naming {needle:?}, got {other:?}"),
    }
}

#[test]
fn degenerate_input_refuses_with_a_clean_error() {
    let ok = [feed(p(2.0, 2.0, 5.0), p(9.0, 2.0, 5.0))];
    refusal(&[feed(p(4.0, 4.0, 5.0), p(4.0, 4.0, 5.0))], &BALL, "zero-length");
    refusal(&ok, &SdfTool::Ball { r: 0.0 }, "radius");
    refusal(&ok, &SdfTool::Flat { r: 0.0, flank: 10.0 }, "radius");
    refusal(&ok, &SdfTool::Ball { r: f64::NAN }, "radius");
    refusal(&ok, &SdfTool::Flat { r: 3.0, flank: f64::NAN }, "flank");
    refusal(&[feed(p(f64::NAN, 2.0, 5.0), p(9.0, 2.0, 5.0))], &FLAT, "non-finite");
    let mut tilted = ok[0];
    tilted.axis = Some([0.0, 0.6, 0.8]);
    refusal(&[tilted], &BALL, "upright");
}

#[test]
fn every_other_tool_refuses_by_name() {
    let ok = [feed(p(2.0, 2.0, 5.0), p(9.0, 2.0, 5.0))];
    refusal(&ok, &SdfTool::Shanked { r: 2.0, shank: 10.0 }, "shanked ball");
    refusal(&ok, &SdfTool::Taper { r: 2.0, cot: 1.0 }, "taper");
    refusal(&ok, &SdfTool::Drill { r: 2.0, cot: 0.6, flank: 10.0 }, "drill");
    refusal(&ok, &SdfTool::BullNose { r: 3.0, corner: 1.0, flank: 10.0 }, "bull nose");
}
