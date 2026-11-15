//! #1580: positive side-face overlap below one pitch must not pass as contact.
//! Provenance: our own box stock and rectangular jaw at four grid phases.
//! Gate map (#1580): covers app::fixture_carried (penetration, mesh_penetration).

use camcraft_app::fixture::FixtureBox;
use camcraft_app::fixture_carried::{MESH_VISIT_BUDGET, mesh_penetration, penetration};
use camcraft_app::geometry::{Point3, cube};
use camcraft_app::stock_sim::{Body, Dexel};
use camcraft_sim::fixture_mesh::FixtureMesh;

fn fixture(face: f64, overlap: f64) -> FixtureBox {
    FixtureBox {
        min: Point3::new(face - overlap, -1.0, 2.0),
        max: Point3::new(5.0, 5.0, 8.0),
    }
}

fn body(face: f64) -> Body {
    // The real material ends at `face`; column centres stay at n + 0.5.
    // Body::on_mesh reads the mesh floor and marks columns beyond it empty.
    let dex = Dexel::block(0.0, 0.0, 4.0, 4.0, 10.0, 1.0).expect("stock grid");
    let stock = cube(Point3::new(0.0, 0.0, 0.0), Point3::new(face, 4.0, 10.0));
    Body::on_mesh(dex, &stock.facets)
}

fn mesh_depth(body: &Body, fixture: &FixtureBox) -> Option<f64> {
    let mesh = FixtureMesh::build(&cube(fixture.min, fixture.max).facets).expect("jaw mesh");
    let mut budget = MESH_VISIT_BUDGET;
    mesh_penetration(body, fixture, &mesh, &mut budget)
}

fn assert_sub_pitch_overlap_is_steel(judge: fn(&Body, &FixtureBox) -> Option<f64>) {
    let mut missed = Vec::new();
    for phase in [0.1, 0.3, 0.7, 0.9] {
        let face = 2.0 + phase;
        let body = body(face);
        assert_eq!(judge(&body, &fixture(face, 0.0)), None, "contact at phase {phase}");
        assert_eq!(judge(&body, &fixture(face, -0.05)), None, "clear at phase {phase}");
        // 0.05 mm is five contact tolerances, but only 0.05 of the pitch.
        if judge(&body, &fixture(face, 0.05)).is_none() {
            missed.push(phase);
        }
    }
    assert!(missed.is_empty(), "0.05 mm of steel passed at face phases {missed:?}");
}

#[test]
fn box_refuses_sub_pitch_side_face_overlap_at_four_grid_phases() {
    assert_sub_pitch_overlap_is_steel(penetration);
}

#[test]
fn mesh_refuses_sub_pitch_side_face_overlap_at_four_grid_phases() {
    assert_sub_pitch_overlap_is_steel(mesh_depth);
}
