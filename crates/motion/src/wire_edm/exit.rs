//! The wire contour's exit pick (#3332, part of #1448): where the cut
//! leaves. A pick is a wish - it is projected onto the wire path and
//! said in a note, or refused by name; it is never moved silently.
//!
//! - An OPEN run ends at the end nearest the exit pick. When that is
//!   the run's first end, the wire walks the same wire-centre path the
//!   other way: the kerf stays where the stated side of the DRAWN
//!   order puts it. A pick equally far from both ends names no end and
//!   is refused.
//! - A CLOSED profile is cut one whole lap from the threading point's
//!   lead-in, as without a pick. The wire then travels in its own kerf
//!   (the shorter way round, cutting nothing) to the path's nearest
//!   point to the exit pick, and leads out straight to the threading
//!   point from there. A lead-out that would bring the kerf into the
//!   product is refused. With tabs (#539), the stretch that holds the
//!   exit is cut last and left there, before the tab cut-off
//!   (`tabs::tabbed`); an exit on a tab is refused.
//!
//! Provenance: general wire EDM practice (a slot the wire has burnt is
//! free to travel in; the lead-out returns to the threading point) as
//! the public literature of #290 states it, over plain 2D arc-length
//! geometry. Our own code; no vendor post, control manual or CAM
//! product behaviour was consulted.

use crate::cl::Toolpath;
use crate::geo2d::{Loop, Point2};
use crate::geometry::Point3;
use crate::machining2d::GenError;
use crate::sheet::arclen::ArcRing;

use super::open::EDGE_TOL;
use super::ring::{dist, nearest};
use super::{invalid, WireParams, PATH_TOL};

/// [`super::wire_contour`] with a stated exit pick (module head).
///
/// # Errors
/// Every refusal of [`super::wire_contour`], and the exit's own.
pub fn wire_contour_exit(loops: &[Loop], chains: &[Vec<Point2>], p: &WireParams, exit: Point2) -> Result<Toolpath, GenError> {
    if !(exit.x.is_finite() && exit.y.is_finite()) {
        return Err(invalid("the exit point's position is not a finite point".into()));
    }
    super::cut_pass(loops, chains, p, false, Some(exit))
}

/// The arc length of `q`'s nearest point on the ring `r` reads, and
/// that point.
pub(super) fn foot_at(ring: &[Point2], r: &ArcRing<'_>, q: Point2) -> (f64, Point2) {
    let (seg, foot) = nearest(ring, q);
    let base = r.cum.get(seg).copied().unwrap_or(0.0);
    (base + ring.get(seg).map_or(0.0, |a| dist(*a, foot)), foot)
}

/// The refusal of a lead-out from `foot` (the path's nearest point to
/// the exit pick) that brings the kerf into the product.
pub(super) fn blocked(exit: Point2, foot: Point2, p: &WireParams) -> GenError {
    GenError::invalid_at(
        format!(
            "wire contour: the exit point at X{:.3} Y{:.3} cannot be left from - the straight lead-out from the wire path at X{:.3} Y{:.3} back to the threading point would bring the kerf into the {}; pick an exit that faces the threading point",
            exit.x,
            exit.y,
            foot.x,
            foot.y,
            p.side.word()
        ),
        Point3::new(foot.x, foot.y, p.plane_z),
    )
}

/// How near `given` (the profile's ring) the segment a-b comes.
pub(super) fn reach(given: &[Point2], a: Point2, b: Point2) -> f64 {
    let n = given.len();
    (0..n).map(|j| super::tabs::seg_seg(a, b, given[j], given[(j + 1) % n])).fold(f64::INFINITY, f64::min)
}

/// A closed, untabbed lap `lap` (from the lead-in's foot `from` round
/// to it again) left at the exit pick: the travel in the kerf is added
/// to `lap`, and the note that says it is returned.
pub(super) fn leave(
    ring: &[Point2],
    given: &[Point2],
    (start, from, exit): (Point2, Point2, Point2),
    p: &WireParams,
    reverse: bool,
    lap: &mut Vec<Point2>,
) -> Result<String, GenError> {
    let r = ArcRing::new(ring);
    let (s, _) = foot_at(ring, &r, from);
    let (e, foot) = foot_at(ring, &r, exit);
    if reach(given, foot, start) < p.kerf_offset() - PATH_TOL {
        return Err(blocked(exit, foot, p));
    }
    // The shorter way round: the whole ring is cut, so either is free.
    let ahead = (e - s).rem_euclid(r.len);
    let travel = if ahead <= r.len - ahead { ahead } else { ahead - r.len };
    if travel.is_finite() && travel.abs() > 1e-9 {
        for q in r.walk(s, s + travel) {
            if lap.last().is_none_or(|l| dist(*l, q) > 1e-9) {
                lap.push(q);
            }
        }
    }
    Ok(format!(
        "wire EDM: threaded at X{:.3} Y{:.3}; straight lead-in of {:.3} mm to the path's nearest point, the profile cut {}; the exit point at X{:.3} Y{:.3} - the wire travels {:.3} mm in its own kerf to the path's nearest point to it (X{:.3} Y{:.3}) and leads out {:.3} mm straight to the threading point",
        start.x,
        start.y,
        dist(start, from),
        if reverse { "clockwise" } else { "counter-clockwise" },
        exit.x,
        exit.y,
        travel.abs(),
        foot.x,
        foot.y,
        dist(foot, start)
    ))
}

/// An open run's end choice: `true` when the exit pick is nearest the
/// run's FIRST end, so the cut is walked against the drawn order.
pub(super) fn against(run: &[Point2], exit: Point2, z: f64) -> Result<bool, GenError> {
    let (Some(&a), Some(&b)) = (run.first(), run.last()) else {
        return Ok(false);
    };
    let (da, db) = (dist(exit, a), dist(exit, b));
    if !(da.is_finite() && db.is_finite()) || (da - db).abs() <= EDGE_TOL {
        return Err(GenError::invalid_at(
            format!(
                "wire contour: the exit point at X{:.3} Y{:.3} stands equally far ({da:.3} mm) from both ends of the open run - it names no end; pick it nearer the end the cut must leave at",
                exit.x, exit.y
            ),
            Point3::new(exit.x, exit.y, z),
        ));
    }
    Ok(da < db)
}

/// What an open run says of its exit pick.
pub(super) fn open_note(exit: Point2, back: bool) -> String {
    format!(
        "wire EDM: the exit point at X{:.3} Y{:.3} is nearest the run's {} end - the cut ends there{}",
        exit.x,
        exit.y,
        if back { "first" } else { "last" },
        if back { ", walked against the run's drawn order; the kerf side is still read from the drawn order" } else { "" }
    )
}

#[cfg(test)]
#[path = "exit_tests.rs"]
mod tests;
