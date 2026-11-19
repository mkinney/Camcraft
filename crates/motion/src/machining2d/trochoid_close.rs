//! The closing pencil of a wide closed trochoid lane (issue #1587): the
//! arcs that take the last front arc of a lane round a closed centre
//! line up to the bore the lane started from. A front arc bulges
//! forward and the bore's circle bulges back, so where they meet at the
//! centre line two wedges stand at the walls; on a band wider than the
//! tool the wedges are deep enough to wrap the cutter. The pencil is a
//! family of circles through pairs of wall points (the cap pencil of
//! `trochoid_loops`, with the wall points stepped as well as the tips):
//! each arc's wall points and its tip are ahead of the arc before, so on
//! a straight run two arcs never cross (two circles symmetric about the
//! centre line meet at most once on each side). Round a bend the plan
//! is the same and the lane's own checks are the law (`contained`, the
//! crossing law, the measured engagement). The last arc stops short of
//! the bore's own circle. The circle through three points is school
//! geometry; the pencil is ours, no vendor material.

use crate::geo2d::Point2;

use super::trochoid_loops::{Chain, El, MAX_LOOPS, Piece, dist};

/// The last arc's wall points stop this far (mm) behind the bore's.
const WALL_SHORT: f64 = 0.3;

/// The last arc's tip stops this far (mm) behind the bore's circle.
const TIP_SHORT: f64 = 0.15;

/// The pencil's tips share at least this part of the band's half-width.
const TIP_ROOM_PER_B: f64 = 0.25;

/// The pencil's wall points step by this share of the lane's spacing: a
/// backward bulge cuts on the inside of its curve, where the same bite
/// wraps more of the cutter than on a front arc.
const WALL_STEP: f64 = 0.5;

/// An arc flatter than this (mm of bulge) has its tip moved on by this
/// much: a straight line across the band is no arc move.
const MIN_BULGE: f64 = 0.05;

/// The tips advance at least this much (mm) an arc, or no pencil is laid.
const MIN_TIP_STEP: f64 = 0.12;

/// Replace the tail of `pieces` (the front arcs of a closed lane with
/// band half-width `b`, circle radius `rho`, centres `spacing` apart;
/// piece `k` is the arc about station `(k + 1) * spacing`) by the
/// closing pencil. Returns the count of front arcs kept. Nothing changes
/// when the lane is too short for a pencil: it then ends as it did,
/// short of its bore.
pub(super) fn closing_pencil(chain: &Chain, pieces: &mut Vec<Piece>, b: f64, rho: f64, spacing: f64, ccw: bool) -> usize {
    let n = pieces.len();
    let len = chain.len();
    let back = (rho * rho - b * b).max(0.0).sqrt();
    let tip_end = len - b - TIP_SHORT;
    let wall_end = len - WALL_SHORT;
    // The pencil's arcs step their wall points by [`WALL_STEP`] of the
    // spacing and their tips by at least [`MIN_TIP_STEP`]: the tip room
    // that takes (the wall run is `gap` longer than the tip run).
    let wall_step = WALL_STEP * spacing;
    if wall_step <= MIN_TIP_STEP * 1.05 {
        return n;
    }
    let gap = (wall_end - back) - (tip_end - rho);
    let room = (TIP_ROOM_PER_B * b).max(1.05 * MIN_TIP_STEP * gap / (wall_step - MIN_TIP_STEP));
    // The last front arc kept: its tip leaves the pencil that room.
    let Some(keep) = (1..=n).rev().find(|&k| k as f64 * spacing + rho <= tip_end - room) else {
        return n;
    };
    let s_keep = keep as f64 * spacing;
    let (wall0, tip0) = (s_keep + back, s_keep + rho);
    let m = ((wall_end - wall0) / wall_step).ceil();
    if keep < 2 || !(m.is_finite() && m >= 1.0 && m <= MAX_LOOPS as f64) || (tip_end - tip0) / m < MIN_TIP_STEP {
        return n;
    }
    let m = m as usize;
    let side = if ccw { 1.0 } else { -1.0 };
    let mut arcs = Vec::with_capacity(m);
    for j in 1..=m {
        let f = j as f64 / m as f64;
        let w = wall0 + (wall_end - wall0) * f;
        let mut tip = tip0 + (tip_end - tip0) * f;
        if (tip - w).abs() < MIN_BULGE {
            tip += MIN_BULGE;
        }
        // The wall points at station `w` (the arc starts on `-side`).
        let (c, t) = (chain.at(w), chain.dir(w, spacing));
        let nrm = Point2::new(-t.y * side, t.x * side);
        let from = Point2::new(c.x - b * nrm.x, c.y - b * nrm.y);
        let to = Point2::new(c.x + b * nrm.x, c.y + b * nrm.y);
        let Some((center, sense)) = circle_through(from, chain.at(tip), to) else {
            return n;
        };
        arcs.push(Piece { start: from, cut: vec![El::Arc { center, to, ccw: sense }] });
    }
    pieces.truncate(keep);
    pieces.extend(arcs);
    keep
}

/// The centre of the circle through `a`, `m`, `b` and the sense (CCW?)
/// of the arc from `a` through `m` to `b`; `None` when they are in line.
fn circle_through(a: Point2, m: Point2, b: Point2) -> Option<(Point2, bool)> {
    let (ax, ay, bx, by) = (m.x - a.x, m.y - a.y, b.x - a.x, b.y - a.y);
    let cross = ax * by - ay * bx;
    if !(cross.is_finite() && cross.abs() > 1e-9 * (dist(a, m) * dist(a, b)).max(1e-12)) {
        return None;
    }
    let (la, lb) = (ax * ax + ay * ay, bx * bx + by * by);
    let ux = (by * la - ay * lb) / (2.0 * cross);
    let uy = (ax * lb - bx * la) / (2.0 * cross);
    let c = Point2::new(a.x + ux, a.y + uy);
    (c.x.is_finite() && c.y.is_finite()).then_some((c, cross > 0.0))
}
