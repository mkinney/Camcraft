//! #2611: the stock the eye draws during playback shows the removal.
//! The UI's playback path is replayed headless: the transport asks the
//! worker for positions one after the other, each published frame is
//! composed into one resident cut layer the way the GPU eye does it
//! (a whole layer replaces, a patch over the resident frame writes its
//! runs, any other patch repacks the frame whole), and the drawn
//! material is counted as `max(mesh, cut) < 0` texels. No pixels.
//! Provenance: our own fixtures over the M7.1 field engine; nothing
//! from vendor binaries.
//!
//! Gate map (M4.4): covers app::stock_play, app::stock_play_worker, app::stock_play_worker_precalc, app::stock_view.

use std::time::{Duration, Instant};

use flecs_ecs::prelude::World;

use camcraft_app::commands::{AddTool, CreateOperation, GenerateOperation, History, ImportModel, SetStock};
use camcraft_app::geometry::{cube, Point3, IDENTITY_Q};
use camcraft_app::operation::{Height, OpParams, SrcRef};
use camcraft_app::schema::Location;
use camcraft_app::stock::StockShape;
use camcraft_app::stock_play::{FieldPlay, PlaySeed, Precalc, PRECALC_BUDGET};
use camcraft_app::stock_play_worker::{PlayFrame, PlayWorker};
use camcraft_app::stock_sim::preview::pack_cut;
use camcraft_app::stock_view::{field_view_with, CutLayer, PackedBase};
use camcraft_app::tools::catalog;
use camcraft_app::world::new_world;

/// A staged face over a `side` x `side` block (the `stock_play_precalc`
/// fixture).
fn faced_doc(side: f64, stepover: f64, stepdown: Option<f64>) -> World {
    let world = new_world();
    let mut history = History::default();
    let half = side / 2.0;
    let loc = Location { pos: Point3::ZERO, rot: IDENTITY_Q };
    let cmd = SetStock::new(&world, StockShape::Box { dx: side, dy: side, dz: 20.0 }, loc);
    history.execute(&world, cmd.expect("box stock"));
    let tool = catalog().into_iter().find(|(n, _)| *n == "EM-0400-2FL").map(|(_, t)| t).expect("catalog entry");
    history.execute(&world, AddTool::new(&world, "EM-0400-2FL", tool).expect("catalog tool"));
    let (lo, hi) = (Point3::new(-half, -half, 0.0), Point3::new(half, half, 14.0));
    history.execute(&world, ImportModel::from_mesh("part", cube(lo, hi)));
    let params = OpParams::Face {
        holes: Default::default(),
        tilt: None,
        stepover,
        clearance_z: Height::fixed(25.0),
        cut_z: Height::fixed(14.0),
        top_z: stepdown.map(|_| Height::fixed(19.0)),
        stepdown,
        feed: 1200.0,
        plunge_feed: 300.0,
        climb: true,
        blend_turns: false,
    };
    let cmd = CreateOperation::new(&world, "Face1", "EM-0400-2FL", SrcRef::Stock, params);
    history.execute(&world, cmd.expect("create face"));
    history.execute(&world, GenerateOperation::new(&world, "Face1").expect("generate face"));
    world
}

/// Texels drawn as material: `max(mesh, cut) < 0`.
fn solid(mesh: &[f32], cut: &[f32]) -> usize {
    mesh.iter().zip(cut).filter(|(m, c)| m.max(**c) < 0.0).count()
}

/// The eye's resident state: the frame number it holds and its cut layer.
struct Resident {
    seq: u64,
    cut: Vec<f32>,
}

/// Take `frame` into the resident layer as `eye.rs` `rewrite_cut` /
/// `upload` do.
fn compose(res: &mut Option<Resident>, frame: &PlayFrame) {
    let packed = frame.view.packed.as_ref().expect("the worker packs its frames for the eye");
    let whole = |p: &camcraft_app::stock_view::Packed| match &p.cut {
        CutLayer::Full(c) => c.as_ref().clone(),
        CutLayer::Patch { .. } => pack_cut(frame.view.stock.field().expect("a field stock"), &p.base.meta),
    };
    match res {
        Some(r) if packed.patches(r.seq) => {
            let CutLayer::Patch { patch, .. } = &packed.cut else { unreachable!("patches() is only true for a patch") };
            patch.apply(&mut r.cut).expect("the patch fits the layer");
            r.seq = packed.seq;
        }
        _ => *res = Some(Resident { seq: packed.seq, cut: whole(packed) }),
    }
}

fn wait_precalc(w: &PlayWorker) -> Precalc {
    let t0 = Instant::now();
    loop {
        let p = w.precalc();
        if !matches!(p, Precalc::Running { .. }) || t0.elapsed() > Duration::from_secs(120) {
            return p;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// Play the program as the transport does - one position after the
/// other, every frame the worker publishes taken and composed - and
/// return the drawn material at each landed position with the frame.
fn play_through(w: &PlayWorker, positions: &[(usize, f64)], res: &mut Option<Resident>) -> Vec<(usize, PlayFrame)> {
    let mut landed = Vec::new();
    for &(k, f) in positions {
        w.request_at(k, f);
        let (mut seen, t0) = (w.published(), Instant::now());
        loop {
            assert!(t0.elapsed() < Duration::from_secs(120), "no frame at move {k}");
            if let Some(fr) = w.take() {
                let fr = fr.expect("the worker cuts the face program");
                compose(res, &fr);
                if fr.at == k && fr.frac.to_bits() == f.to_bits() {
                    landed.push((k, fr));
                    break;
                }
            }
            seen = w.wait_published(seen, Duration::from_millis(100));
        }
    }
    landed
}

fn check_playback(world: &World, cells: f64) {
    let seed = || PlaySeed::gather(world, false, cells).expect("seed");
    let w = PlayWorker::spawn_precalc(seed(), PRECALC_BUDGET).expect("worker");
    assert!(matches!(wait_precalc(&w), Precalc::Ready { .. }), "the calculation finishes: {:?}", w.precalc());
    let total = w.index().total();
    let uncut = FieldPlay::from_seed(seed()).expect("play").view().stock;
    let (ub, ucut) = PackedBase::of(&uncut, None).expect("packs");
    let full = field_view_with(world, cells).expect("deviate view");
    let (fb, fcut) = PackedBase::of(&full.stock, None).expect("packs");
    let (start, finished) = (solid(&ub.mesh, &ucut), solid(&fb.mesh, &fcut));
    assert!(start > finished, "the face removes material: uncut {start}, finished {finished} texels");
    // The transport from the start to the end, a few moves a frame, the
    // move in flight half drawn on the way.
    let step = (total / 40).max(1);
    let mut positions: Vec<(usize, f64)> = (0..total).step_by(step).map(|k| (k, if k % 2 == 0 { 0.0 } else { 0.5 })).collect();
    positions.push((total, 0.0));
    let mut res = None;
    let landed = play_through(&w, &positions, &mut res);
    let r = res.as_ref().expect("frames were drawn");
    let (_, last) = landed.last().expect("the end landed");
    let base = &last.view.packed.as_ref().expect("packed").base;
    let drawn = solid(&base.mesh, &r.cut);
    assert!(drawn < start, "the drawn stock at the end shows a removal: {drawn} texels drawn, uncut {start}");
    assert_eq!(drawn, finished, "the drawn stock at the end is the finished stock");
    // Played again from the start: at each landed position the drawn
    // layer is that frame's own stock, and the removal grows.
    let mut res = None;
    let mut prev = start;
    for &(k, f) in &positions {
        play_through(&w, &[(k, f)], &mut res);
        let r = res.as_ref().expect("a frame was drawn");
        let drawn = solid(&base.mesh, &r.cut);
        assert!(drawn <= prev, "move {k}+{f}: the drawn material only goes down ({drawn} after {prev})");
        prev = drawn;
    }
    assert_eq!(prev, finished, "the replay ends at the finished stock");
}

/// The small face at about 1 mm cells.
#[test]
fn the_drawn_stock_shows_the_removal_as_the_program_plays() {
    check_playback(&faced_doc(12.0, 1.5, Some(2.5)), 12.0 * 12.0 * 20.0);
}

/// A longer program at the viewport's own display budget.
#[test]
fn the_drawn_stock_shows_the_removal_at_the_display_budget() {
    check_playback(&faced_doc(24.0, 0.8, Some(2.5)), camcraft_app::stock_view::DEFAULT_TEXELS);
}
