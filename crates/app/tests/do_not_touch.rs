//! #2055: Protection commands, generation and the post gate.
//! Gate map (M4.4): covers app::operation, app::commands::operation::do_not_touch,
//! app::mcp, app::posting, motion::machining2d::pocket, motion::machining3d, sim::verify::avoid.

mod common;
mod do_not_touch_support;

use camcraft_app::commands::SetOperationDoNotTouch;
use camcraft_app::verify::{FixturePart, Warning};
use camcraft_app::world::new_world;
use camcraft_app::{model, operation, posting, project};
use do_not_touch_support::setup_plate_and_tower;
use serde_json::json;

/// Extract active G90 `(x, y, z)` motion waypoints from Fanuc NC text (skipping G28/G53 home lines).
fn nc_waypoints(nc: &str) -> Vec<[f64; 3]> {
    let (mut x, mut y, mut z) = (0.0, 0.0, 50.0);
    let mut armed = false;
    let mut pts = Vec::new();
    for line in nc.lines() {
        let trim = line.trim();
        if trim.contains("G43") || trim.contains("G54") {
            armed = true;
            continue;
        }
        if !armed || trim.contains("G28") || trim.contains("G53") || trim.contains("G91") || trim.starts_with('(') {
            continue;
        }
        let mut saw_coord = false;
        for tok in trim.split_whitespace() {
            if let Some(rest) = tok.strip_prefix('X')
                && let Ok(v) = rest.parse::<f64>()
            {
                x = v;
                saw_coord = true;
            } else if let Some(rest) = tok.strip_prefix('Y')
                && let Ok(v) = rest.parse::<f64>()
            {
                y = v;
                saw_coord = true;
            } else if let Some(rest) = tok.strip_prefix('Z')
                && let Ok(v) = rest.parse::<f64>()
            {
                z = v;
                saw_coord = true;
            }
        }
        if saw_coord {
            pts.push([x, y, z]);
        }
    }
    pts
}

#[test]
fn offset2d_do_not_touch_keeps_full_tool_clear_of_protected_face_and_edge() {
    let (dir, mut c, plate_top, tower_top_face, tower_edge, tower_vert) = setup_plate_and_tower("offset2d");
    let added = c.ok("add_operation", json!({
        "kind": "2D Offset",
        "tool": "SteppedMill",
        "params": {
            "stepover": 2.5,
            "stepdown": 7.0,
            "stock_to_leave": 0.5,
            "top_z": { "reference": "fixed", "offset": 24.0 },
            "bottom_z": { "reference": "fixed", "offset": 10.0 },
            "clearance_z": { "reference": "fixed", "offset": 35.0 },
            "retract_z": { "reference": "fixed", "offset": 26.0 }
        }
    }));
    let op_name = added["operation"].as_str().expect("op name").to_owned();
    c.ok("set_operation_geometry", json!({
        "operation": op_name,
        "picks": [{ "model": "plate", "id": plate_top }]
    }));

    // Baseline WITHOUT Do not touch enters right through the center tower box [-6..6].
    c.ok("generate", json!({ "name": op_name }));
    let base_nc = dir.join("baseline.nc");
    let base_err = c.refused("post", json!({ "output": base_nc.display().to_string(), "post": "Fanuc" }));
    assert!(
        base_err.contains("gouge") && base_err.contains("10.00"),
        "unprotected baseline must cut into tower at z=10: {base_err}"
    );

    // Protect tower top face + an edge chain with 1.5 mm clearance (plus 0.5 mm stock_to_leave = 2.0 mm).
    let vertex_err = c.refused("set_operation_do_not_touch", json!({
        "operation": op_name,
        "picks": [{ "model": "tower", "id": tower_vert }]
    }));
    assert!(vertex_err.contains("vertex"), "expected vertex refusal: {vertex_err}");
    let neg_err = c.refused("set_operation_do_not_touch", json!({
        "operation": op_name,
        "clearance": -0.5,
        "picks": [{ "model": "tower", "id": tower_top_face }]
    }));
    assert!(neg_err.contains("finite"), "expected negative clearance refusal: {neg_err}");

    c.ok("set_operation_do_not_touch", json!({
        "operation": op_name,
        "clearance": 1.5,
        "picks": [
            { "model": "tower", "id": tower_top_face },
            { "model": "tower", "id": tower_edge }
        ]
    }));

    let got = c.ok("get_operation_do_not_touch", json!({ "operation": op_name }));
    assert_eq!(got["clearance"], json!(1.5));
    assert_eq!(got["picks"].as_array().map(Vec::len), Some(2));

    let generated = c.ok("generate", json!({ "name": op_name }));
    assert_eq!(generated["generated"][0]["status"], json!("ok"), "{generated}");
    let ver = c.ok("verify", json!({}));
    assert_eq!(ver["ok"], json!(true), "{ver}");
    let nc = dir.join("offset.nc");
    let posted = c.ok("post", json!({ "output": nc.display().to_string(), "post": "Fanuc" }));
    assert!(posted["lines"].as_u64().is_some_and(|n| n > 0));
    let prot_txt = std::fs::read_to_string(&nc).expect("read protected nc");
    let prot_pts = nc_waypoints(&prot_txt);
    // At cut depth z=10, flute r=3 + req 2.0 = 5.0 outside [-6, 6] => at least outside (-10.9, 10.9).
    for p in prot_pts.iter().filter(|p| p[2] < 24.0 - 1e-3) {
        let dx = (p[0].abs() - 6.0).max(0.0);
        let dy = (p[1].abs() - 6.0).max(0.0);
        let dist = dx.hypot(dy);
        assert!(
            dist >= 5.0 - 1e-2,
            "cut waypoint {p:?} came closer ({dist:.3} mm) than 5.0 mm to tower [-6..6]"
        );
    }

    // Tilted (3+2) Offset2D with Do not touch also generates & verifies clean.
    let tilted = c.ok("add_operation", json!({
        "kind": "2D Offset",
        "tool": "SteppedMill",
        "source": "plate",
        "params": {
            "stepover": 2.5,
            "stepdown": 13.0,
            "top_z": { "reference": "fixed", "offset": 28.0 },
            "bottom_z": { "reference": "fixed", "offset": 15.0 },
            "clearance_z": { "reference": "fixed", "offset": 45.0 },
            "retract_z": { "reference": "fixed", "offset": 35.0 },
            "tilt": { "a_deg": 5.0, "b_deg": 0.0, "origin": [0.0, 0.0, 0.0] }
        }
    }))["operation"].as_str().expect("tilted op").to_owned();
    c.ok("set_operation_do_not_touch", json!({
        "operation": tilted,
        "clearance": 1.0,
        "picks": [{ "model": "tower", "id": tower_top_face }]
    }));
    let tgen = c.ok("generate", json!({ "name": tilted }));
    assert_eq!(tgen["generated"][0]["status"], json!("ok"), "{tgen}");
    let tver = c.ok("verify", json!({ "name": tilted }));
    assert_eq!(tver["ok"], json!(true), "{tver}");
    assert!(
        !tver["findings"].as_array().is_some_and(|fs| fs.iter().any(|f| f["kind"] == json!("avoid_hit"))),
        "tilted op must have zero avoid_hit findings: {tver}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pocket3d_do_not_touch_keeps_clear_raises_links_and_blocks_impossible_cut() {
    let (dir, mut c, plate_top, tower_top_face, _tower_edge, _vert) = setup_plate_and_tower("pocket3d");
    let added = c.ok("add_operation", json!({
        "kind": "3D Pocket",
        "tool": "SteppedMill",
        "source": "plate",
        "params": {
            "stepover": 2.8,
            "stepdown": 6.0,
            "stock_to_leave": 0.5,
            "top_z": { "reference": "fixed", "offset": 24.0 },
            "bottom_z": { "reference": "fixed", "offset": 12.0 },
            "clearance_z": { "reference": "fixed", "offset": 35.0 },
            "retract_z": { "reference": "fixed", "offset": 26.0 }
        }
    }));
    let op_name = added["operation"].as_str().expect("op name").to_owned();
    c.ok("set_operation_do_not_touch", json!({
        "operation": op_name,
        "clearance": 1.0,
        "picks": [{ "model": "tower", "id": tower_top_face }]
    }));
    let got = c.ok("get_operation_do_not_touch", json!({ "operation": op_name }));
    assert_eq!(got["picks"].as_array().map(Vec::len), Some(1));
    let geom = c.ok("get_operation_geometry", json!({ "operation": op_name }));
    assert_eq!(geom["do_not_touch"]["clearance"], json!(1.0));
    assert_eq!(geom["do_not_touch"]["picks"].as_array().map(Vec::len), Some(1));

    // Check Waterline, OffsetRings+shell, Adaptive, rest_rough+feed_opt, KeepDown.
    for (patch, label) in [
        (json!({ "strategy": "waterline", "entry": "plunge" }), "waterline"),
        (json!({ "strategy": "offset_rings", "entry": "ramp", "shell": true }), "offset_rings+shell"),
        (json!({ "strategy": "waterline", "entry": "helix", "rest_rough": true, "feed_opt": true }), "rest+feed_opt"),
        (json!({ "strategy": "adaptive", "entry": "helix", "shell": false, "rest_rough": false, "feed_opt": false }), "adaptive"),
    ] {
        c.ok("edit_operation", json!({ "name": op_name, "params": patch }));
        let generated = c.ok("generate", json!({ "name": op_name }));
        assert_eq!(generated["generated"][0]["status"], json!("ok"), "{label}: {generated}");
        let ver = c.ok("verify", json!({ "name": op_name }));
        assert_eq!(ver["ok"], json!(true), "{label}: {ver}");
    }

    // Undo/redo around SetOperationDoNotTouch + GenerateOperation.
    c.ok("undo", json!({}));
    c.ok("undo", json!({}));
    c.ok("set_operation_do_not_touch", json!({
        "operation": op_name,
        "clearance": 2.0,
        "picks": [{ "model": "tower", "id": tower_top_face }]
    }));
    c.ok("undo", json!({}));
    let after_undo = c.ok("get_operation_do_not_touch", json!({ "operation": op_name }));
    assert_eq!(after_undo["clearance"], json!(1.0));
    c.ok("redo", json!({}));
    assert_eq!(c.ok("get_operation_do_not_touch", json!({ "operation": op_name }))["clearance"], json!(2.0));

    // Unsupported kind refusal + overlap with src & Include boundary refusal.
    let pocket_op = c.ok("add_operation", json!({ "kind": "2D Pocket", "tool": "SteppedMill" }))["operation"]
        .as_str()
        .expect("pocket name")
        .to_owned();
    let unsup = c.refused("set_operation_do_not_touch", json!({
        "operation": pocket_op,
        "picks": [{ "model": "tower", "id": tower_top_face }]
    }));
    assert!(unsup.contains("2D Offset") && unsup.contains("3D Pocket"), "unsupported msg: {unsup}");

    let op2 = c.ok("add_operation", json!({ "kind": "2D Offset", "tool": "SteppedMill" }))["operation"]
        .as_str()
        .expect("op2 name")
        .to_owned();
    c.ok("set_operation_geometry", json!({
        "operation": op2,
        "picks": [{ "model": "plate", "id": plate_top }]
    }));
    let msg = c.refused("set_operation_do_not_touch", json!({
        "operation": op2,
        "picks": [{ "model": "plate", "id": plate_top }]
    }));
    assert!(msg.contains("plate"), "refusal must name item: {msg}");

    // Protect tower on op2, then try adding tower_top_face into op2's Include boundary -> refused!
    c.ok("set_operation_do_not_touch", json!({
        "operation": op2,
        "picks": [{ "model": "tower", "id": tower_top_face }]
    }));
    let area_overlap = c.refused("set_operation_area", json!({
        "operation": op2,
        "area": {
            "extra": [{ "mode": "include", "picks": [{ "model": "tower", "id": tower_top_face }] }]
        }
    }));
    assert!(area_overlap.contains("tower"), "expected Include overlap refusal: {area_overlap}");
    let geom_overlap = c.refused("set_operation_geometry", json!({
        "operation": op2,
        "picks": [{ "model": "tower", "id": tower_top_face }]
    }));
    assert!(geom_overlap.contains("tower"), "expected SetOperationGeometry overlap refusal: {geom_overlap}");

    // Covering the entire plate with a protected face refuses generation naming the protected item.
    let op3 = c.ok("add_operation", json!({
        "kind": "2D Offset",
        "tool": "SteppedMill",
        "source": "plate",
        "params": {
            "stepover": 2.5,
            "stepdown": 5.0,
            "top_z": { "reference": "fixed", "offset": 24.0 },
            "bottom_z": { "reference": "fixed", "offset": 12.0 },
            "clearance_z": { "reference": "fixed", "offset": 35.0 },
            "retract_z": { "reference": "fixed", "offset": 26.0 }
        }
    }))["operation"].as_str().expect("op3 name").to_owned();
    c.ok("set_operation_do_not_touch", json!({
        "operation": op3,
        "clearance": 16.0,
        "picks": [{ "model": "plate", "id": plate_top }]
    }));
    let blocked = c.refused("generate", json!({ "name": op3 }));
    assert!(blocked.contains("plate") && blocked.contains("Do not touch"), "expected item name in refusal: {blocked}");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn do_not_touch_post_gate_blocks_violations() {
    let (dir, mut c, plate_top, tower_top_face, tower_edge, _vert) = setup_plate_and_tower("proj_gate");
    let op_name = c.ok("add_operation", json!({
        "kind": "2D Offset",
        "tool": "SteppedMill",
        "params": {
            "stepover": 2.5,
            "stepdown": 7.0,
            "stock_to_leave": 0.5,
            "top_z": { "reference": "fixed", "offset": 24.0 },
            "bottom_z": { "reference": "fixed", "offset": 10.0 },
            "clearance_z": { "reference": "fixed", "offset": 35.0 },
            "retract_z": { "reference": "fixed", "offset": 26.0 }
        }
    }))["operation"].as_str().expect("op name").to_owned();
    c.ok("set_operation_geometry", json!({
        "operation": op_name,
        "picks": [{ "model": "plate", "id": plate_top }]
    }));
    c.ok("set_operation_do_not_touch", json!({
        "operation": op_name,
        "clearance": 1.5,
        "picks": [
            { "model": "tower", "id": tower_top_face },
            { "model": "tower", "id": tower_edge }
        ]
    }));
    c.ok("generate", json!({ "name": op_name }));

    let proj_path = dir.join("protected.camproj");
    c.ok("save_project", json!({ "path": proj_path.display().to_string() }));

    // Moving the protected tower into the cut path stales the op AND is caught by live_avoid_problems,
    // blocking both verify and post without writing NC output.
    c.ok("move_model", json!({ "name": "tower", "position": [22.0, 0.0, 0.0] }));
    let ver_bad = c.ok("verify", json!({}));
    assert_eq!(ver_bad["ok"], json!(false), "{ver_bad}");
    let post_bad = dir.join("blocked.nc");
    let refused = c.refused("post", json!({ "output": post_bad.display().to_string(), "post": "Fanuc" }));
    assert!(!post_bad.exists(), "blocked post must never write NC output: {refused}");

    // Also test a freshly loaded world where a cached toolpath carries a Do not touch violation:
    // both live_avoid_problems and Warning::AvoidHit block posting via verification_report + post_to_file.
    let w = new_world();
    project::load(&w, &proj_path).expect("reload saved project");
    let live_problems = operation::avoid::live_avoid_problems(&w);
    assert!(live_problems.is_empty(), "loaded clean project has no live avoid problems: {live_problems:?}");
    let op_ent = model::find_by_name(&w, &op_name).expect("op entity");
    let mut cache = model::toolpath_of(&w, &op_ent).expect("cached toolpath");
    cache.warnings.push(Warning::AvoidHit { at: 0, item: 1, part: FixturePart::Holder, short_mm: 0.4 });
    op_ent.entity_view(&w).set(cache);
    let gate = posting::verification_report(&w);
    assert!(gate.problems.is_empty(), "clean problems before move: {gate:?}");
    let err = posting::post_setup(&w, &post_bad)
        .expect_err("cached AvoidHit must block post");
    assert!(err.contains("Do not touch") || err.contains("warning"), "expected block message: {err}");
    assert!(!post_bad.exists(), "blocked post must never create NC output");

    // Clean API checks.
    assert!(Warning::AvoidHit { at: 0, item: 1, part: FixturePart::Holder, short_mm: 0.4 }.blocks_post());
    let _ = std::any::type_name::<SetOperationDoNotTouch>();
    let _ = std::fs::remove_dir_all(&dir);
}
