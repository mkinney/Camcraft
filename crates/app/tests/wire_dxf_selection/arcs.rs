//! A selected DXF chain with arcs (#3937): a die circle and a line-and-arc
//! punch cut as true arcs, verified by the wire laws and posted as G2 / G3
//! with I J through the generic ISO, Sodick and G-EDM wire posts, each to
//! its golden. TEXT and DIMENSION entities no longer block the cut.
//!
//! Regenerate the goldens after an intentional change, then read the diff:
//! `WRITE_GOLDEN=1 cargo test -p camcraft-app --test wire_dxf_selection arcs`

use super::*;

const ISO: &str = "Generic ISO Wire EDM";
const SODICK: &str = "Sodick Wire EDM (Mark 21)";
const GEDM: &str = "G-EDM Wire EDM";
const GEDM_FIXTURE: &str = include_str!("../edm/g-edm-fixture.mch");

/// A radius 5 circle about the drawing's origin, with a TEXT and a
/// DIMENSION entity beside it.
const DIE: &str = "0\nSECTION\n2\nHEADER\n9\n$INSUNITS\n70\n4\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n\
    0\nCIRCLE\n8\noutline\n10\n0\n20\n0\n40\n5\n\
    0\nTEXT\n8\nnotes\n10\n20\n20\n20\n40\n2\n1\nDIE 10\n\
    0\nDIMENSION\n8\nnotes\n10\n-5\n20\n-8\n0\nENDSEC\n0\nEOF\n";

/// A 20 x 20 L about the origin: its corner at (10, -10) rounded by a
/// radius 3 fillet (bulge tan(22.5 degrees)), a sharp inside corner at
/// (0, 0).
const PUNCH: &str = "0\nSECTION\n2\nHEADER\n9\n$INSUNITS\n70\n4\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n\
    0\nLWPOLYLINE\n8\noutline\n90\n7\n70\n1\n10\n-10\n20\n-10\n10\n7\n20\n-10\n42\n0.41421356237309503\n\
    10\n10\n20\n-7\n10\n10\n20\n0\n10\n0\n20\n0\n10\n0\n20\n10\n10\n-10\n20\n10\n0\nENDSEC\n0\nEOF\n";

/// A start pick at a world point (#3339 moved `start` into `picks`).
fn start_at(p: [f64; 2]) -> camcraft_app::path_picks::PathPicks {
    camcraft_app::path_picks::PathPicks {
        start: vec![camcraft_app::path_picks::PathPick::xy(camcraft_app::geo2d::Point2::new(p[0], p[1]))],
        exit: Vec::new(),
    }
}

/// The drawing point (`x`, `y`) in world XY.
fn to_world(world: &World, x: f64, y: f64) -> [f64; 2] {
    let (e, _) = sketch::find(world, "drawing").expect("drawing");
    let pose = sketch::world_pose(world, &e).expect("placed");
    let q = camcraft_app::geometry::transform_point(Point3::new(x, y, 0.0), pose.pos, pose.rot);
    [q.x, q.y]
}

/// `dxf` placed and picked as `document` does, on `machine`, cut on
/// `side` from the drawing point `hole` under `condition`.
fn doc(dxf: &str, machine: &str, condition: &str, side: WireSide, hole: (f64, f64), sharp: bool) -> World {
    let (world, mut h) = document(dxf);
    if machine != FIXTURE {
        let m = machine::parse_mch(machine).expect("machine");
        h.execute(&world, LoadMachine::new(&world, "wire.mch".into(), Some(m)).expect("mount"));
    }
    let op = WireContourOp {
        condition: condition.into(),
        side,
        picks: start_at(to_world(&world, hole.0, hole.1)),
        sharp_corners: sharp,
        ..wire_op()
    };
    h.execute(&world, EditOperationParams::new(&world, "Cut", OpParams::WireContour(op)).expect("edit"));
    world
}

fn generated(world: &World) -> camcraft_app::cl::Toolpath {
    let cmd = GenerateOperation::new(world, "Cut").unwrap_or_else(|e| panic!("generates: {e}"));
    let mut h = History::default();
    h.execute(world, cmd);
    let e = model::find_by_name(world, "Cut").expect("operation");
    let verdict = operation::wire_verify::wire_verdict(world, &e).expect("wire").expect("verifiable");
    assert!(verdict.findings.is_empty() && verdict.captured && verdict.slugs.len() == 1, "{verdict:?}");
    model::toolpath_of(world, &e).expect("cache").path.clone()
}

/// Post through `post` and hold the program to `golden`.
fn golden(world: &World, post: &str, golden: &str) -> String {
    let file = std::env::temp_dir().join(format!("camcraft-dxf-arcs-{}-{}.nc", std::process::id(), golden.replace('/', "-")));
    let nc = posting::post_script(world, &file, posting::builtin_post_text(post).expect("post"))
        .unwrap_or_else(|e| panic!("{post} posts: {e}"))
        .program;
    let _ = std::fs::remove_file(file);
    let path = format!("tests/golden/nc/{golden}");
    if std::env::var_os("WRITE_GOLDEN").is_some() {
        std::fs::write(&path, &nc).expect("write golden");
    }
    let want = std::fs::read_to_string(&path).expect("golden present");
    assert_eq!(nc, want, "{post} drifted from {golden}");
    nc
}

fn arc_blocks(nc: &str) -> Vec<&str> {
    nc.lines().filter(|l| l.starts_with("G2 ") || l.starts_with("G3 ") || l.starts_with("G02") || l.starts_with("G03")).collect()
}

#[test]
fn die_circle_cuts_two_half_arcs_from_its_centre_and_posts_g3_with_ij() {
    let world = doc(DIE, FIXTURE, "C110", WireSide::Die, (0.0, 0.0), false);
    let path = generated(&world);
    assert_eq!(path.arc_count(), 2, "{:?}", path.moves);
    assert!(path.notes.iter().any(|n| n.contains("towards +X")), "{:?}", path.notes);
    assert!(path.notes.iter().any(|n| n.contains("2 TEXT, MTEXT or DIMENSION entities")), "{:?}", path.notes);
    // The arcs stand on the kerf circle about the drawn centre.
    let c = to_world(&world, 0.0, 0.0);
    for m in path.moves.iter().filter(|m| m.arc.is_some()) {
        let a = m.arc.expect("arc");
        assert!((a.center.x - c[0]).abs() < 1e-9 && (a.center.y - c[1]).abs() < 1e-9, "{a:?}");
        let r = (m.to.x - c[0]).hypot(m.to.y - c[1]);
        assert!((r - (5.0 - 0.165)).abs() < 1e-9, "{r}");
    }
    let nc = golden(&world, ISO, "wire-iso/wire_dxf_die_circle.nc");
    let arcs = arc_blocks(&nc);
    assert_eq!(arcs.len(), 2, "{nc}");
    // Each half arc turns counter-clockwise about the centre: I = -4.835 or +4.835.
    assert!(arcs.iter().all(|l| l.starts_with("G3") && (l.contains("I-4.835") || l.contains("I4.835"))), "{arcs:?}");
    let nc = golden(&world, SODICK, "sodick-wire/wire_dxf_die_circle.nc");
    assert_eq!(arc_blocks(&nc).len(), 2, "{nc}");
    let gedm = doc(DIE, GEDM_FIXTURE, "R1", WireSide::Die, (0.0, 0.0), false);
    generated(&gedm);
    let nc = golden(&gedm, GEDM, "g-edm/wire_dxf_die_circle.nc");
    assert_eq!(arc_blocks(&nc).len(), 2, "{nc}");
}

#[test]
fn line_and_arc_punch_pivots_its_corners_and_posts_its_goldens() {
    let world = doc(PUNCH, FIXTURE, "C110", WireSide::Punch, (20.0, 0.0), true);
    let path = generated(&world);
    // The fillet, and a pivot about each of the four sharp outside corners.
    assert_eq!(path.arc_count(), 5, "{:?}", path.moves);
    assert!(path.notes.iter().any(|n| n.contains("sharp inside corners stated")), "{:?}", path.notes);
    for (post, file) in [(ISO, "wire-iso/wire_dxf_punch_fillet.nc"), (SODICK, "sodick-wire/wire_dxf_punch_fillet.nc")] {
        let nc = golden(&world, post, file);
        assert_eq!(arc_blocks(&nc).len(), 5, "{nc}");
    }
    let gedm = doc(PUNCH, GEDM_FIXTURE, "R1", WireSide::Punch, (20.0, 0.0), true);
    generated(&gedm);
    let nc = golden(&gedm, GEDM, "g-edm/wire_dxf_punch_fillet.nc");
    assert_eq!(arc_blocks(&nc).len(), 5, "{nc}");
}

#[test]
fn arcs_stay_refused_for_taper_and_a_hatch_still_blocks_the_cut() {
    let world = doc(DIE, FIXTURE, "C110", WireSide::Die, (0.0, 0.0), false);
    let tapered = WireContourOp { picks: start_at(to_world(&world, 0.0, 0.0)), taper_deg: 1.0, ..wire_op() };
    let mut h = History::default();
    h.execute(&world, EditOperationParams::new(&world, "Cut", OpParams::WireContour(tapered)).expect("taper"));
    let why = refusal(&world);
    assert!(why.contains("single vertical pass"), "{why}");
    let hatch = DIE.replace("0\nENDSEC\n0\nEOF", "0\nHATCH\n8\nnotes\n0\nENDSEC\n0\nEOF");
    let world = doc(&hatch, FIXTURE, "C110", WireSide::Die, (0.0, 0.0), false);
    let why = refusal(&world);
    assert!(why.contains("skipped 1 HATCH entity"), "{why}");
}
