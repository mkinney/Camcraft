//! Provenance: original closed-solid fixture from elementary drilled-block geometry, for #2601.

use super::tests::{circle, circle_cw, ring_walls};
use super::{recognize, recognize_cross};
use crate::geometry::{Point3, Triangle};

/// Reduced form of the reported D50 bar: a closed square prism with
/// opposite D5 holes, each 8 mm deep, across a 50 mm material span.
/// The two floor discs leave 34 mm of solid material between the holes.
fn opposite_blind_block() -> Vec<Triangle> {
    let n = 32;
    let inner = circle(0.0, 0.0, 2.5, n);
    let outer: Vec<_> = inner
        .iter()
        .map(|&(x, y)| {
            let scale = 5.0 / x.abs().max(y.abs());
            (x * scale, y * scale)
        })
        .collect();
    let mut mesh = ring_walls(&outer, -25.0, 25.0);
    let bore = circle_cw(0.0, 0.0, 2.5, n);
    mesh.extend(ring_walls(&bore, 17.0, 25.0));
    mesh.extend(ring_walls(&bore, -25.0, -17.0));
    let at = |p: (f64, f64), z| Point3::new(p.0, p.1, z);
    for i in 0..n {
        let j = (i + 1) % n;
        for (mouth, floor, reverse) in [(25.0, 17.0, false), (-25.0, -17.0, true)] {
            let triangles = [
                [
                    at(inner[i], mouth),
                    at(outer[i], mouth),
                    at(outer[j], mouth),
                ],
                [
                    at(inner[i], mouth),
                    at(outer[j], mouth),
                    at(inner[j], mouth),
                ],
                [
                    Point3::new(0.0, 0.0, floor),
                    at(inner[i], floor),
                    at(inner[j], floor),
                ],
            ];
            for [a, b, c] in triangles {
                mesh.push(if reverse {
                    Triangle::new(a, c, b)
                } else {
                    Triangle::new(a, b, c)
                });
            }
        }
    }
    mesh
}

fn radial_block() -> Vec<Triangle> {
    // Proper rotation: old Z becomes X; the outward winding stays intact.
    let rotate = |p: Point3| Point3::new(p.z, p.x, p.y);
    opposite_blind_block()
        .into_iter()
        .map(|t| Triangle::new(rotate(t.a), rotate(t.b), rotate(t.c)))
        .collect()
}

#[test]
fn opposite_blind_z_walls_keep_their_separate_depths() {
    let found = recognize(&opposite_blind_block());
    assert_eq!(
        found.len(),
        2,
        "separate floors must not create a through hole: {found:?}"
    );
    for hole in found {
        assert_eq!(hole.entry_z - hole.floor_z, 8.0, "{hole:?}");
        assert_eq!(hole.radius, 2.5);
    }
}

#[test]
fn opposite_blind_radial_walls_keep_their_separate_depths() {
    let found = recognize_cross(&radial_block());
    assert_eq!(
        found.len(),
        2,
        "separate floors must not create a through hole: {found:?}"
    );
    for hole in found {
        assert_eq!(hole.floor_d, -8.0, "{hole:?}");
        assert_eq!(hole.radius, 2.5);
    }
}

#[test]
fn opposite_blind_radial_holes_enter_from_their_own_mouths() {
    let found = recognize_cross(&radial_block());
    for side in [-1.0, 1.0] {
        let origin = Point3::new(side * 25.0, 0.0, 0.0);
        let hole = found.iter().find(|h| h.origin == origin);
        assert!(hole.is_some(), "missing the mouth at {origin:?}: {found:?}");
        let hole = hole.unwrap();
        assert_eq!(hole.dir, Point3::new(-side, 0.0, 0.0), "{hole:?}");
        assert!(!hole.through, "the floor makes this hole blind: {hole:?}");
        assert_eq!(hole.entry_d, 0.0);
        assert_eq!(hole.floor_d, -8.0);
    }
}
