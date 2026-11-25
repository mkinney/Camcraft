//! The wire contour's exit pick (#3332): an open run ends at the end
//! nearest the pick, a closed profile is left at the pick's foot after
//! its whole lap, a tabbed cut leaves there before the cut-off, and an
//! impossible pick is refused by name. The wire and gap are SYNTHETIC -
//! a plausible size, from no machine's manual (#290 D4, D10).

use crate::cl::Toolpath;
use crate::geo2d::{Loop, Point2, Region};

use super::super::{cutoff_split, wire_contour, wire_contour_exit, TabParams, WireParams, WireSide};

const K: f64 = 0.165;

fn plate() -> Region {
    Region::new(vec![rect(0.0, 0.0, 100.0, 50.0)]).unwrap()
}

fn rect(x0: f64, y0: f64, x1: f64, y1: f64) -> Loop {
    Loop::new(vec![Point2::new(x0, y0), Point2::new(x1, y0), Point2::new(x1, y1), Point2::new(x0, y1)]).unwrap()
}

fn params(side: WireSide, start: Option<(f64, f64)>) -> WireParams {
    WireParams {
        wire_diameter: 0.25,
        gap: 0.04,
        condition: "C110".into(),
        side,
        start: start.map(|(x, y)| Point2::new(x, y)),
        plane_z: -12.0,
        tabs: None,
        stock: Some(plate()),
        clearance: 1.0,
        sharp_corners: false,
    }
}

fn pts(path: &Toolpath) -> Vec<(f64, f64)> {
    let mut v = vec![(path.moves[0].from.x, path.moves[0].from.y)];
    v.extend(path.moves.iter().map(|m| (m.to.x, m.to.y)));
    v
}

fn near(a: (f64, f64), b: (f64, f64)) -> bool {
    (a.0 - b.0).hypot(a.1 - b.1) < 1e-6
}

/// Done means (#3332): an open run ends at the end nearest the exit
/// pick. The wire-centre path is the same line either way, so the kerf
/// stays on the stated side of the drawn order.
#[test]
fn an_open_run_ends_at_the_end_nearest_the_exit_pick() {
    let line = vec![vec![Point2::new(60.0, 0.0), Point2::new(60.0, 50.0)]];
    let p = params(WireSide::Right, None);
    let plain = wire_contour(&[], &line, &p).unwrap();
    // The drawn order already ends at Y50: the same moves, and a note.
    let far = wire_contour_exit(&[], &line, &p, Point2::new(58.0, 49.0)).unwrap();
    assert_eq!(far.moves, plain.moves);
    assert!(far.notes.iter().any(|n| n.contains("nearest the run's last end")), "{:?}", far.notes);
    // A pick at the first end: the same points, walked the other way.
    let back = wire_contour_exit(&[], &line, &p, Point2::new(61.0, 2.0)).unwrap();
    let (mut want, got) = (pts(&plain), pts(&back));
    want.reverse();
    assert_eq!(got, want);
    assert!(got.iter().all(|q| (q.0 - (60.0 + K)).abs() < 1e-9), "the kerf keeps its side: {got:?}");
    assert!(near(got[got.len() - 1], (60.0 + K, -(K + 1.0))), "run out at the first end: {got:?}");
    let said = back.notes.join(" | ");
    assert!(said.contains("nearest the run's first end") && said.contains("against the run's drawn order"), "{said}");
    assert!(said.contains(&format!("threaded outside the stock at X{:.3} Y{:.3}", 60.0 + K, 50.0 + K + 1.0)), "{said}");
}

/// Done means (#3332): an impossible pick is refused by name. A pick
/// equally far from both ends of the run names no end.
#[test]
fn an_exit_pick_that_names_no_end_of_an_open_run_is_refused() {
    let line = vec![vec![Point2::new(60.0, 0.0), Point2::new(60.0, 50.0)]];
    let p = params(WireSide::Left, None);
    let why = wire_contour_exit(&[], &line, &p, Point2::new(40.0, 25.0)).unwrap_err().to_string();
    assert!(why.contains("exit point at X40.000 Y25.000") && why.contains("names no end"), "{why}");
    let why = wire_contour_exit(&[], &line, &p, Point2::new(f64::NAN, 0.0)).unwrap_err().to_string();
    assert!(why.contains("exit point's position is not a finite point"), "{why}");
}

/// A closed profile is cut one whole lap exactly as without a pick;
/// the wire then travels in its kerf to the pick's foot and leads out
/// to the threading point from there.
#[test]
fn a_closed_profile_is_left_at_the_exit_pick_after_its_whole_lap() {
    let loops = [rect(-20.0, -10.0, 20.0, 10.0)];
    let p = params(WireSide::Punch, Some((-30.0, 0.0)));
    let plain = wire_contour(&loops, &[], &p).unwrap();
    let path = wire_contour_exit(&loops, &[], &p, Point2::new(-25.0, 8.0)).unwrap();
    let (a, b) = (pts(&plain), pts(&path));
    // The lead-in and the lap are the plain cut's, point for point.
    assert_eq!(b[..a.len() - 1], a[..a.len() - 1]);
    assert_eq!(b.len(), a.len() + 1);
    assert!(near(b[b.len() - 2], (-20.0 - K, 8.0)), "left at the pick's foot: {b:?}");
    assert!(near(b[b.len() - 1], (-30.0, 0.0)), "the lead-out ends at the threading point: {b:?}");
    let said = path.notes.join(" | ");
    assert!(said.contains("exit point at X-25.000 Y8.000") && said.contains("travels 8.000 mm in its own kerf"), "{said}");
    assert!(!said.contains("back along the lead-in"), "{said}");
    // A die: the same law from a start hole in the slug.
    let die = WireParams { sharp_corners: true, ..params(WireSide::Die, Some((0.0, 0.0))) };
    let path = wire_contour_exit(&loops, &[], &die, Point2::new(15.0, 9.0)).unwrap();
    let v = pts(&path);
    assert!(near(v[v.len() - 2], (15.0, 10.0 - K)) && near(v[v.len() - 1], (0.0, 0.0)), "{v:?}");
    // A pick whose foot is the lead-in's own: the plain cut's moves.
    let same = wire_contour_exit(&loops, &[], &p, Point2::new(-22.0, 0.0)).unwrap();
    assert_eq!(same.moves, plain.moves);
}

/// Done means (#3332): an impossible pick is refused by name. The
/// lead-out from the far side of a punch would cut through the punch.
#[test]
fn an_exit_pick_whose_lead_out_would_cut_the_product_is_refused() {
    let loops = [rect(-20.0, -10.0, 20.0, 10.0)];
    let p = params(WireSide::Punch, Some((-30.0, 0.0)));
    let why = wire_contour_exit(&loops, &[], &p, Point2::new(0.0, 15.0)).unwrap_err().to_string();
    assert!(why.contains("exit point at X0.000 Y15.000") && why.contains("would bring the kerf into the punch"), "{why}");
    // The law without an exit stays: a closed profile needs its threading point.
    let why = wire_contour_exit(&loops, &[], &params(WireSide::Punch, None), Point2::new(-25.0, 8.0)).unwrap_err().to_string();
    assert!(why.contains("no start hole is stated"), "{why}");
}

/// With tabs, the stretch that holds the exit is cut last and left at
/// the pick's foot, before the tab cut-off; the cut-off is unchanged.
/// An exit on a tab is refused by name.
#[test]
fn a_tabbed_cut_leaves_at_the_exit_pick_before_the_cut_off() {
    let loops = [rect(-20.0, -10.0, 20.0, 10.0)];
    let at = vec![Point2::new(0.0, 10.0), Point2::new(0.0, -10.0)];
    let tabs = TabParams { count: Some(2), width: Some(2.0), at, cutoff: Some(("C110".into(), K)) };
    let p = WireParams { tabs: Some(tabs), sharp_corners: true, ..params(WireSide::Die, Some((0.0, 0.0))) };
    let start = Point2::new(0.0, 0.0);
    let plain = wire_contour(&loops, &[], &p).unwrap();
    let path = wire_contour_exit(&loops, &[], &p, Point2::new(19.0, 5.0)).unwrap();
    let (m0, m1) = (cutoff_split(&plain, start, 2).unwrap(), cutoff_split(&path, start, 2).unwrap());
    let ends = |path: &Toolpath, from: usize| path.moves[from..].iter().map(|m| (m.to.x, m.to.y)).collect::<Vec<_>>();
    assert_eq!(ends(&plain, m0), ends(&path, m1), "the cut-off is the plain cut's");
    // The main cut's last leave: the pick's foot, then the threading point.
    let (foot, home) = (path.moves[m1 - 2].to, path.moves[m1 - 1].to);
    assert!(near((foot.x, foot.y), (20.0 - K, 5.0)) && near((home.x, home.y), (0.0, 0.0)), "{foot:?} {home:?}");
    assert!(path.notes.iter().any(|n| n.contains("exit point at X19.000 Y5.000") && n.contains("before the tab cut-off")), "{:?}", path.notes);
    let why = wire_contour_exit(&loops, &[], &p, Point2::new(0.3, 9.0)).unwrap_err().to_string();
    assert!(why.contains("exit point at X0.300 Y9.000") && why.contains("on a tab"), "{why}");
}
