//! Outline thinning ahead of the growth rings' opening (issue #882):
//! each closed loop of an area thinned by the Douglas-Peucker
//! line reduction (Ramer 1972, "An iterative procedure for the polygonal
//! approximation of plane curves", Comput. Graph. Image Process. 1(3);
//! Douglas & Peucker 1973, "Algorithms for the reduction of the number
//! of points required to represent a digitized line or its caricature",
//! Cartographica 10(2)), run on the loop's two halves between its first
//! vertex and the vertex farthest from it.
//!
//! Why: the growth rings (`grow`) are filleted by a morphological
//! opening every trial step, the opening erodes its area first, and the
//! boolean engine's erosion costs about the square of the vertex count
//! (every vertex of the eroded outline leaves a join the sweep keeps
//! active at once). The walls' vertices ride through every ring, so a
//! finely tessellated outline - a STEP bore at 9 nm of chord sag - took
//! ~1.3 s a trial step. Vertices that move the outline by less than the
//! rings' own arc flattening say nothing about its shape.
//!
//! The bound: every vertex kept is a vertex of the loop, and every
//! vertex dropped lies within `tol` of the chord that replaces its run.
//! The area between a dropped run and its chord lies in the run's convex
//! hull, so every point the simplified area gains or loses is within
//! `tol` of the simplified boundary: the simplified area shrunk by `tol`
//! lies inside the original. The opening erodes that much deeper than
//! it dilates back (`rings::INSET`), so it never leaves the area it
//! opens. Ours; no vendor material consulted.
//!
//! HELD WORK (#882): correct and bounded by its own tests, but not
//! landable yet - see the issue. Any sub-micron change to the rings
//! flips knife-edge decisions elsewhere in the planner (the microlift
//! acceptance in `lift`, the #850 budget, the #845 overload fallback).

use crate::geo2d::{Loop, Point2, Region};

/// Distance from `p` to the segment `a`-`b`.
fn seg_dist(p: Point2, a: Point2, b: Point2) -> f64 {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let dd = dx * dx + dy * dy;
    let t = if dd > 0.0 { (((p.x - a.x) * dx + (p.y - a.y) * dy) / dd).clamp(0.0, 1.0) } else { 0.0 };
    (p.x - a.x - dx * t).hypot(p.y - a.y - dy * t)
}

/// The vertices of the closed loop `pts` to keep (in order): no dropped
/// vertex is farther than `tol` from the chord across its run. A loop
/// that would keep fewer than three is returned whole.
fn thin_loop(pts: &[Point2], tol: f64) -> Vec<Point2> {
    let n = pts.len();
    if n < 4 {
        return pts.to_vec();
    }
    let first = pts[0];
    let from_first = |i: usize| (pts[i].x - first.x).hypot(pts[i].y - first.y);
    let far = (1..n).max_by(|&i, &j| from_first(i).total_cmp(&from_first(j))).unwrap_or(n / 2);
    // The loop as an open run that ends where it starts (index `n` is
    // the first vertex again), split at the far vertex.
    let at = |i: usize| pts[i % n];
    let mut keep = vec![false; n];
    keep[0] = true;
    keep[far] = true;
    // An explicit stack: a run's depth is bounded by its length, never
    // by the call stack.
    let mut runs = vec![(0usize, far), (far, n)];
    while let Some((i, j)) = runs.pop() {
        if j <= i + 1 {
            continue;
        }
        let (a, b) = (at(i), at(j));
        let (mut worst, mut m) = (-1.0f64, i);
        for k in i + 1..j {
            let d = seg_dist(at(k), a, b);
            if d > worst {
                (worst, m) = (d, k);
            }
        }
        if worst > tol {
            keep[m] = true;
            runs.push((i, m));
            runs.push((m, j));
        }
    }
    let out: Vec<Point2> = pts.iter().zip(&keep).filter(|(_, k)| **k).map(|(p, _)| *p).collect();
    if out.len() < 3 { pts.to_vec() } else { out }
}

/// `r` with every loop thinned to within `tol` mm of itself, or `None`
/// where nothing is dropped - or the thinned loops are no region (a loop made to cross itself): the caller keeps
/// `r` as it is, which is always right and only slower.
pub(super) fn within(r: &Region, tol: f64) -> Option<Region> {
    if !(tol.is_finite() && tol > 0.0) {
        return None;
    }
    let before: usize = r.loops().iter().map(|l| l.points().len()).sum();
    let thinned: Vec<Vec<Point2>> = r.loops().iter().map(|l| thin_loop(l.points(), tol)).collect();
    if thinned.iter().map(Vec::len).sum::<usize>() >= before {
        return None;
    }
    let loops: Vec<Loop> = thinned.into_iter().map(Loop::new).collect::<Result<_, _>>().ok()?;
    let out = Region::new(loops).ok()?;
    // The same loops, each still the piece it was: a loop that crossed a
    // neighbour and fused with it is not taken.
    (out.loops().len() == r.loops().len()).then_some(out)
}

#[cfg(test)]
#[path = "simplify_tests.rs"]
mod tests;
