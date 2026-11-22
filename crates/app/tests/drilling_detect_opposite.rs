//! Provenance: original closed D50 drilled-bar regression for #2601.
//! Gate map: covers camcraft_app::operation::material, camcraft_app::operation::generate.

mod cross_support;
mod punched_bar;

use cross_support::*;

#[test]
fn mill_detection_preserves_opposite_blind_radial_holes_and_refuses_generation() {
    let expected: Vec<_> = [-1.0, 1.0]
        .into_iter()
        .map(|side| CrossHole {
            origin: Point3::new(side * 25.0, 0.0, 0.0),
            dir: Point3::new(-side, 0.0, 0.0),
            radius: 2.5,
            entry_d: 0.0,
            floor_d: -8.0,
            through: false,
        })
        .collect();
    let world = new_world();
    let mut history = History::default();
    let cmd = LoadMachine::new(&world, "bench.mch".into(), Some(machine("mill")));
    history.execute(&world, cmd.expect("machine"));
    add_tool(&world, &mut history, "DR-0400");
    history.execute(
        &world,
        ImportModel::from_mesh(
            "bar",
            punched_bar::punched_bar(25.0, -40.0, 40.0, &expected),
        ),
    );
    let source = SrcRef::Entity("bar".into());
    let detected = operation::detect_holes(&world, "D", &cross_params(vec![], None), None, &source)
        .expect("the mill must report radial holes instead of dropping them");
    let OpParams::Drill { holes, cross, .. } = &detected.params else {
        panic!("Drill")
    };
    assert!(holes.is_empty(), "{holes:?}");
    assert_eq!(cross.len(), 2, "{cross:?}");
    for hole in expected {
        assert!(cross.contains(&hole), "missing {hole:?}: {cross:?}");
    }
    let op = Operation {
        tool: "DR-0400".into(),
        src: source,
        params: detected.params,
        stamp: None,
        setup: None,
        area: Default::default(),
        coolant: Default::default(),
        after: Default::default(),
        suppressed: false,
    };
    let error = operation::generate(&world, &op, "").expect_err("a plain mill has no turret");
    assert!(error.to_string().contains("live-tooling"), "{error}");
}
