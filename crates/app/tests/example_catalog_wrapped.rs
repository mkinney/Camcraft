//! Provenance: original catalog assertions for the wrapped A-axis example (#2869).
//! Gate map: covers app::project, app::setup, app::model, app::operation.

use std::{fs, path::Path};

use camcraft_app::operation::OpParams;
use camcraft_app::{model, project, setup, world};

#[test]
fn wrapped_case_keeps_three_rotary_operations_and_posts_a_axis_moves() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/catalog/multiaxis/wrapped");
    let w = world::new_world();
    let notes = project::load_noted(&w, &dir.join("wrapped.camproj"))
        .expect("the wrapped catalog project must load");
    assert!(notes.is_empty(), "unexpected load notes: {notes:?}");
    let mut kinds = Vec::new();
    for (_, entity) in setup::operation_order(&w) {
        let operation = model::operation_of(&w, &entity).expect("saved operation");
        kinds.push(match operation.params {
            OpParams::RotaryContour { .. } => "Contour",
            OpParams::RotaryParallel { .. } => "Parallel",
            OpParams::RotaryOffset { .. } => "Offset",
            other => panic!("unexpected operation in wrapped example: {other:?}"),
        });
    }
    kinds.sort_unstable();
    assert_eq!(kinds, ["Contour", "Offset", "Parallel"]);

    let nc = fs::read_to_string(dir.join("wrapped.nc")).expect("read wrapped.nc");
    let sections: Vec<_> = nc.split("(OPERATION: ").skip(1).collect();
    assert_eq!(sections.len(), 3, "each rotary operation must appear in the program");
    for section in sections {
        let mut angles: Vec<f64> = section.lines()
            .filter(|line| line.starts_with("G00 ") || line.starts_with("G01 "))
            .flat_map(str::split_whitespace)
            .filter_map(|word| word.strip_prefix('A')?.parse::<f64>().ok())
            .collect();
        assert!(angles.iter().all(|a| a.is_finite()), "A angles must be finite");
        angles.sort_by(f64::total_cmp);
        angles.dedup();
        assert!(angles.len() > 1, "each rotary operation must move the A axis: {section}");
    }
}
