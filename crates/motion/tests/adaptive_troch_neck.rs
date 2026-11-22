//! Adaptive trochoid shape checks from the project tests for issues #1509 and #2990.
//! Gate map: covers camcraft_motion::machining2d (adaptive_troch, adaptive_troch_lanes, adaptive_troch_clean), camcraft_motion::quality.

#[path = "common/adaptive_troch.rs"]
mod fixtures;

use fixtures::*;
use camcraft_motion::machining2d::PocketParams;
use camcraft_motion::geo2d::Region;

#[test]
fn triangle_and_wide_to_narrow_neck_are_cleared() {
    let p = PocketParams { bottom_z: -3.0, ..params() };
    let Ok(tri) = Region::new(vec![poly(&[(0.0, 0.0), (60.0, 0.0), (0.0, 60.0)])]) else { panic!("triangle") };
    check_cleared("triangle 60x60 D6", &tri, &p);
    let Ok(neck) = Region::new(vec![poly(&[
        (0.0, 0.0),
        (30.0, 0.0),
        (30.0, 10.0),
        (48.0, 10.0),
        (48.0, 20.0),
        (30.0, 20.0),
        (30.0, 30.0),
        (0.0, 30.0),
    ])]) else {
        panic!("neck")
    };
    check_cleared("wide-to-narrow neck D6", &neck, &p);
}

#[test]
#[ignore = "heavy lane: release engagement and rest-area measurement"]
fn l_shaped_pocket_is_cleared_under_engagement_limit() {
    let p = PocketParams { tool_diameter: 10.0, bottom_z: -3.0, ..params() };
    let Ok(l) = Region::new(vec![poly(&[(0.0, 0.0), (60.0, 0.0), (60.0, 30.0), (30.0, 30.0), (30.0, 60.0), (0.0, 60.0)])]) else {
        panic!("l shape");
    };
    check_cleared("L-shape D10", &l, &p);
    check_cleared("L-shape D6", &l, &PocketParams { tool_diameter: 6.0, ..p });
}
