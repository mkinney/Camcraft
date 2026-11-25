//! #3332 (part of #1448): the Wire Contour's start and exit picks through
//! the commands. A file with the old `start` key opens to the same path,
//! byte for byte. An exit pick ends an open run at the nearest end, and
//! undo and redo restore the path. A closed cut leaves at the pick, and
//! a pick that cannot be cut is refused by name.
//!
//! Gate map (#3332): covers app::operation (pick_kinds capability,
//! wire_exit, wire_op::generate_wire, wire_verify), app::commands
//! (operation edit and generate, undo and redo), app::project
//! (wire_edm_dto legacy `start` load).
//! Provenance: our own tests over our own synthetic fixture (the
//! invented wire-iso machine of #533, #290 D10).

use flecs_ecs::prelude::World;

use camcraft_app::cl::Toolpath;
use camcraft_app::commands::{CreateOperation, EditOperationParams, GenerateOperation, History, ImportModel, LoadMachine, SetStock};
use camcraft_app::geo2d::Point2;
use camcraft_app::geometry::{feature, Mesh, Point3, Triangle};
use camcraft_app::machine::parse_mch;
use camcraft_app::model;
use camcraft_app::operation::{self, FeatureSel, GeoSel, Height, OpParams, OpStatus, SrcRef, WireContourOp, WireSide};
use camcraft_app::path_picks::{PathPick, PathPicks};
use camcraft_app::project;
use camcraft_app::schema::Location;
use camcraft_app::stock::StockShape;
use camcraft_app::world::new_world;

const FIXTURE: &str = include_str!("edm/wire-iso-fixture.mch");
/// Wire radius 0.125 + gap 0.04.
const K: f64 = 0.165;

/// The CCW outline `pts` extruded from z = 0 to 12.
fn prism(pts: &[(f64, f64)]) -> Mesh {
    let p = |(x, y): (f64, f64), z: f64| Point3::new(x, y, z);
    let mut f = Vec::new();
    for i in 0..pts.len() {
        let (a, b) = (pts[i], pts[(i + 1) % pts.len()]);
        f.push(Triangle::new(Point3::new(0.0, 0.0, 12.0), p(a, 12.0), p(b, 12.0)));
        f.push(Triangle::new(Point3::new(0.0, 0.0, 0.0), p(b, 0.0), p(a, 0.0)));
        f.push(Triangle::new(p(a, 0.0), p(b, 0.0), p(b, 12.0)));
        f.push(Triangle::new(p(a, 0.0), p(b, 12.0), p(a, 12.0)));
    }
    Mesh::from_facets(f)
}

fn pick(xy: [f64; 2]) -> Vec<PathPick> {
    vec![PathPick::xy(Point2::new(xy[0], xy[1]))]
}

fn wire_op(side: WireSide, start: Option<[f64; 2]>) -> WireContourOp {
    WireContourOp {
        top_z: Height::fixed(12.0),
        bottom_z: Height::fixed(0.0),
        condition: "C110".into(),
        side,
        picks: PathPicks { start: start.map(pick).unwrap_or_default(), exit: Vec::new() },
        tabs: None,
        capture: false,
        both_pieces_restrained: false,
        stock_centered_in_xy_travel: false,
        taper_deg: 0.0,
        four_axis: None,
        burnout: None,
        clearance: operation::DEFAULT_CLEARANCE,
        skims: Vec::new(),
        sharp_corners: false,
    }
}

fn with_exit(op: &WireContourOp, exit: [f64; 2]) -> WireContourOp {
    WireContourOp { picks: PathPicks { start: op.picks.start.clone(), exit: pick(exit) }, ..op.clone() }
}

fn status(world: &World, name: &str) -> OpStatus {
    operation::status_of(world, &model::find_by_name(world, name).expect("op"))
}

fn params(world: &World, name: &str) -> WireContourOp {
    let op = model::find_by_name(world, name).and_then(|e| model::operation_of(world, &e)).expect("op");
    let OpParams::WireContour(w) = op.params else { panic!("not a wire contour") };
    w
}

fn path(world: &World, name: &str) -> Toolpath {
    let e = model::find_by_name(world, name).expect("op");
    model::toolpath_of(world, &e).expect("generated").path
}

fn pts(p: &Toolpath) -> Vec<(f64, f64)> {
    let mut v = vec![(p.moves[0].from.x, p.moves[0].from.y)];
    v.extend(p.moves.iter().map(|m| (m.to.x, m.to.y)));
    v
}

fn near(a: (f64, f64), b: (f64, f64)) -> bool {
    (a.0 - b.0).hypot(a.1 - b.1) < 1e-6
}

fn edit(world: &World, h: &mut History, name: &str, op: WireContourOp) {
    h.execute(world, EditOperationParams::new(world, name, OpParams::WireContour(op)).expect("edit"));
}

fn generate(world: &World, h: &mut History, name: &str) {
    h.execute(world, GenerateOperation::new(world, name).expect("generates"));
}

fn refusal(world: &World, name: &str) -> String {
    GenerateOperation::new(world, name).err().expect("refused").to_string()
}

/// The wire-iso fixture mounted; a 60 x 50 x 12 part at the left end of
/// a 100 x 50 x 12 bar, cut off along its x = 60 edge by "Cutoff1" (an
/// open run), and a punch "Punch1" around the same part, threaded at
/// X-10 Y25. Both are generated.
fn doc() -> (World, History) {
    let world = new_world();
    let mut h = History::default();
    let m = parse_mch(FIXTURE).expect("the fixture parses");
    h.execute(&world, LoadMachine::new(&world, "wire-iso.mch".into(), Some(m)).expect("mount"));
    h.execute(
        &world,
        camcraft_app::commands::AddTool::new(&world, "EDM-W025", camcraft_app::wire_tools::wire_tool(0.25)).expect("wire"),
    );
    h.execute(&world, ImportModel::from_mesh("part", prism(&[(0.0, 0.0), (60.0, 0.0), (60.0, 50.0), (0.0, 50.0)])));
    let loc = Location { pos: Point3::new(50.0, 25.0, 0.0), ..Location::default() };
    h.execute(&world, SetStock::new(&world, StockShape::Box { dx: 100.0, dy: 50.0, dz: 12.0 }, loc).expect("stock"));
    let e = model::find_by_name(&world, "part").expect("model");
    let f = model::features_of(&world, &e).expect("features");
    let (chain, up) = feature::chains(&f)
        .into_iter()
        .filter(|c| !c.closed)
        .find_map(|c| {
            let p = feature::chain_points(&f, &c);
            let on = p.len() >= 2 && p.iter().all(|q| (q[0] - 60.0).abs() < 1e-9 && (q[2] - 12.0).abs() < 1e-9);
            on.then(|| (c.id, p[p.len() - 1][1] > p[0][1]))
        })
        .expect("the part's x = 60 top edge is its own open chain");
    // Travelling +Y the offcut (x > 60) is on the right.
    let side = if up { WireSide::Right } else { WireSide::Left };
    let src = SrcRef::Selection(vec![GeoSel { entity: "part".into(), feature: FeatureSel::Chain(chain) }]);
    let c = CreateOperation::new(&world, "Cutoff1", "EDM-W025", src, OpParams::WireContour(wire_op(side, None))).expect("open op");
    h.execute(&world, c);
    generate(&world, &mut h, "Cutoff1");
    let punch = OpParams::WireContour(wire_op(WireSide::Punch, Some([-10.0, 25.0])));
    let c = CreateOperation::new(&world, "Punch1", "EDM-W025", SrcRef::Entity("part".into()), punch).expect("punch op");
    h.execute(&world, c);
    generate(&world, &mut h, "Punch1");
    (world, h)
}

/// Rewrites every punch wire contour in `v` to the file form before
/// #3339: no `picks` key, the threading point under `start`.
fn to_old_start_key(v: &mut serde_json::Value, changed: &mut usize) {
    match v {
        serde_json::Value::Object(m) => {
            if let Some(serde_json::Value::Object(w)) = m.get_mut("wire_contour")
                && w.get("side").and_then(serde_json::Value::as_str) == Some("punch")
            {
                let picks = w.remove("picks").expect("the saved punch states its picks");
                w.insert("start".into(), picks["start"][0].clone());
                *changed += 1;
            }
            m.values_mut().for_each(|c| to_old_start_key(c, changed));
        }
        serde_json::Value::Array(a) => a.iter_mut().for_each(|c| to_old_start_key(c, changed)),
        _ => {}
    }
}

/// Done means (#3332): a file with the old `start` key opens with the
/// same threading point, reads Ok against its saved path, and generates
/// the same path again, byte for byte.
#[test]
fn a_file_with_the_old_start_key_opens_to_the_same_path_byte_for_byte() {
    let (world, _h) = doc();
    let before = camcraft_app::cl::text::to_text(&path(&world, "Punch1")).expect("cl text");
    let file = std::env::temp_dir().join(format!("camcraft-3332-old-start-{}.ccam", std::process::id()));
    project::save(&world, &file).expect("save");
    let mut v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&file).expect("read")).expect("json");
    let mut changed = 0;
    to_old_start_key(&mut v, &mut changed);
    assert_eq!(changed, 1, "one punch is rewritten to the old key");
    let text = serde_json::to_string_pretty(&v).expect("text");
    assert!(text.contains("\"start\": [") && !text.contains("\"picks\""), "the file holds the old key only");
    std::fs::write(&file, text).expect("write");
    let back = new_world();
    let loaded = project::load(&back, &file);
    let _ = std::fs::remove_file(&file);
    loaded.expect("load");
    assert_eq!(params(&back, "Punch1"), wire_op(WireSide::Punch, Some([-10.0, 25.0])), "the old key is the start pick");
    assert_eq!(status(&back, "Punch1"), OpStatus::Ok, "the saved path is still this cut's");
    let mut h = History::default();
    generate(&back, &mut h, "Punch1");
    let after = camcraft_app::cl::text::to_text(&path(&back, "Punch1")).expect("cl text");
    assert_eq!(after, before, "the old key generates the same path, byte for byte");
}

/// Done means (#3332): an exit pick ends an open run at the nearest
/// end, and undo and redo restore the previous path.
#[test]
fn an_exit_pick_ends_an_open_run_at_the_nearest_end_and_undo_and_redo_restore_the_path() {
    let (world, mut h) = doc();
    let p0 = params(&world, "Cutoff1");
    let drawn = path(&world, "Cutoff1");
    let a = pts(&drawn);
    let (first, last) = (a[0], a[a.len() - 1]);
    // A pick at the end the drawn order starts from: the cut ends there.
    edit(&world, &mut h, "Cutoff1", with_exit(&p0, [first.0 + 1.0, first.1]));
    assert_ne!(status(&world, "Cutoff1"), OpStatus::Ok, "an exit pick marks the old path stale");
    generate(&world, &mut h, "Cutoff1");
    let picked = path(&world, "Cutoff1");
    let b = pts(&picked);
    assert!(near(b[0], last) && near(b[b.len() - 1], first), "the cut runs to the picked end: {b:?}");
    assert!(b.iter().all(|q| (q.0 - a[0].0).abs() < 1e-9), "the kerf keeps its side: {b:?}");
    assert!(picked.notes.iter().any(|n| n.contains("exit point") && n.contains("first end")), "{:?}", picked.notes);
    assert_eq!(status(&world, "Cutoff1"), OpStatus::Ok);
    // Undo the generate and the edit: the previous pick and the previous path.
    assert!(h.undo(&world) && h.undo(&world));
    assert_eq!(params(&world, "Cutoff1"), p0);
    assert_eq!(path(&world, "Cutoff1").moves, drawn.moves, "undo restores the previous path");
    assert_eq!(status(&world, "Cutoff1"), OpStatus::Ok);
    assert!(h.redo(&world) && h.redo(&world));
    assert_eq!(params(&world, "Cutoff1"), with_exit(&p0, [first.0 + 1.0, first.1]));
    assert_eq!(path(&world, "Cutoff1").moves, picked.moves, "redo restores the picked path");
    // A pick at the other end: the drawn order already ends there.
    edit(&world, &mut h, "Cutoff1", with_exit(&p0, [last.0 - 1.0, last.1]));
    generate(&world, &mut h, "Cutoff1");
    assert_eq!(path(&world, "Cutoff1").moves, drawn.moves);
    // Done means: an impossible pick is refused by name.
    edit(&world, &mut h, "Cutoff1", with_exit(&p0, [first.0 - 20.0, 0.5 * (first.1 + last.1)]));
    let why = refusal(&world, "Cutoff1");
    assert!(why.contains("exit point") && why.contains("names no end"), "{why}");
}

/// A closed cut leaves at the exit pick after its whole lap, and the
/// wire verify reads the same findings. A pick whose lead-out would cut
/// the product, and a pick beside a taper, are refused by name.
#[test]
fn a_closed_cut_leaves_at_the_exit_pick_and_an_impossible_pick_is_refused_by_name() {
    let (world, mut h) = doc();
    let p0 = params(&world, "Punch1");
    let plain = path(&world, "Punch1");
    let e = model::find_by_name(&world, "Punch1").expect("op");
    let verdict = |w: &World| format!("{:?}", operation::wire_verify::wire_verdict(w, &e).expect("a wire cut").expect("judged").findings);
    let judged = verdict(&world);
    edit(&world, &mut h, "Punch1", with_exit(&p0, [-5.0, 40.0]));
    generate(&world, &mut h, "Punch1");
    let picked = path(&world, "Punch1");
    let (a, b) = (pts(&plain), pts(&picked));
    assert_eq!(b[..a.len() - 1], a[..a.len() - 1], "the lead-in and the lap are unchanged");
    assert!(near(b[b.len() - 2], (-K, 40.0)) && near(b[b.len() - 1], (-10.0, 25.0)), "{b:?}");
    assert!(picked.notes.iter().any(|n| n.contains("exit point at X-5.000 Y40.000")), "{:?}", picked.notes);
    assert_eq!(status(&world, "Punch1"), OpStatus::Ok);
    assert_eq!(verdict(&world), judged, "the wire verify reads the same findings");
    // The far side: the lead-out would cross the punch.
    edit(&world, &mut h, "Punch1", with_exit(&p0, [65.0, 25.0]));
    let why = refusal(&world, "Punch1");
    assert!(why.contains("exit point at X65.000 Y25.000") && why.contains("into the punch"), "{why}");
    // A taper beside an exit pick is not cut as something else.
    edit(&world, &mut h, "Punch1", WireContourOp { taper_deg: 1.0, ..with_exit(&p0, [-5.0, 40.0]) });
    let why = refusal(&world, "Punch1");
    assert!(why.contains("exit point") && why.contains("a 1.000 degree taper"), "{why}");
    // The law of the threading point stays: a closed cut without one is refused.
    edit(&world, &mut h, "Punch1", WireContourOp { picks: PathPicks { start: Vec::new(), exit: pick([-5.0, 40.0]) }, ..p0.clone() });
    let why = refusal(&world, "Punch1");
    assert!(why.contains("no start hole is stated"), "{why}");
}
