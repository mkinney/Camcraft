//! The edge a no-pick 2D chamfer rides (#2435): the wall at the corner
//! depth, matched to the edge at the top plane.
//!
//! Provenance: the chamfer mill's cut law (machine-tool handbook
//! geometry, [`super::chamfer`]): the cone's tip stands ON the wall at
//! the corner depth `top - width * tan(angle)`, its generatrix on the
//! face. Where the wall is vertical the section at the top plane and
//! the section at the corner depth are one outline. Where the model
//! already carries a bevel (a modelled chamfer or round), the top
//! plane's outline is the bevel's TOP edge, standing inside the wall
//! by the bevel's width - a tip on it sits inside the part. The wall
//! the cone must touch is the section at the corner depth. Our own law
//! over this repo's sections; no vendor material consulted.

use crate::cl::TiltedFrame;
use crate::geo2d::{Loop, Point2, Region};
use crate::geometry::Point3;

use super::face::GenError;

/// Plan slack of the match between a top edge and its wall, mm: the
/// chord sag two tessellated sections of one rounded corner can
/// disagree by. A loose match only picks which outline to ride; the
/// ride itself is always the wall section, and the program sim holds
/// a 2D chamfer's tip to its wall (#2435) whatever was matched.
const MATCH_SLACK: f64 = 0.05;

/// Below this plan gap the wall IS the top edge (a vertical wall): the
/// top plane's own loop is ridden, unchanged, so every sharp-edged
/// part posts exactly what it did before #2435.
const SAME_EDGE: f64 = 1e-6;

/// The loops a no-pick 2D chamfer of `width` rides: for each loop of
/// `top` (the section at the top plane - the edges to break), the loop
/// of `corner` (the section at the corner depth - the wall the tip
/// touches) that every vertex of the top loop stands within `width` of,
/// in plan (one way only: a wall loop's mitred corner stands farther
/// than `width` from a bevel's top corner).
/// A vertical wall rides the top loop itself. A `corner` loop matched
/// to no top loop is a lower edge (a ledge, the plate under an island)
/// and is left alone: it is not an edge at the top plane.
/// #2596: when any top loop stands over a bevel, only the bevelled loops
/// are ridden; the sharp ones are edges the model leaves sharp.
///
/// # Errors
/// A top loop with no wall within `width` of it under it - a blind
/// pocket shallower than the chamfer's depth, an island whose wall
/// stands lower than that depth - refuses by name at that loop: the
/// tip would ride below the edge's floor, into the material (a part or
/// the stock, whichever the chamfer's source is - the text fits both, #2521).
/// #2887: a top loop bevelled on only part of its length (a modelled
/// chamfer on one side of a plate) refuses by name too: no whole
/// outline of it can be ridden. `lean` is [`wall_lean`]'s answer: the
/// only plan move by which a top loop may stand off its whole wall.
pub fn ride_loops(top: &Region, corner: &Region, width: f64, corner_z: f64, lean: Option<Point2>) -> Result<Region, GenError> {
    if !(width.is_finite() && width > 0.0 && corner_z.is_finite()) {
        return Err(GenError::Invalid("chamfer width must be positive".into()));
    }
    let reach = width + MATCH_SLACK;
    // #2596: the sharp edges (a vertical wall) and the bevelled ones (a
    // modelled chamfer or round) apart.
    let mut sharp: Vec<Loop> = Vec::with_capacity(top.loops().len());
    let mut out: Vec<Loop> = Vec::with_capacity(top.loops().len());
    for t in top.loops() {
        let wall = corner
            .loops()
            .iter()
            .filter(|c| c.is_ccw() == t.is_ccw())
            .map(|c| (c, gap(t.points(), c.points())))
            .filter(|(_, g)| *g <= reach)
            .min_by(|a, b| a.1.total_cmp(&b.1));
        // #2887: a loop bevelled on only part of its length has no whole
        // outline to ride - refused by name, whichever way its
        // tessellation would have read it.
        if let Some((c, _)) = wall
            && let Some((bevel, sharp)) = partly_bevelled(t.points(), c.points(), width, lean)
        {
            return Err(GenError::At {
                msg: format!(
                    "the edge loop near X{:.3} Y{:.3} carries a modelled chamfer on only part of its length - it is sharp near X{:.3} Y{:.3} - a chamfer with no picks would cut the sharp part; pick the edges to break",
                    bevel.x, bevel.y, sharp.x, sharp.y
                ),
                at: Point3::new(bevel.x, bevel.y, corner_z),
            });
        }
        match wall {
            Some((_, g)) if g <= SAME_EDGE => sharp.push(t.clone()),
            Some((c, _)) => out.push(c.clone()),
            None => {
                let at = t.points().first().copied().unwrap_or(Point2::new(0.0, 0.0));
                return Err(GenError::At {
                    msg: format!(
                        "the edge near X{:.3} Y{:.3} has no wall under it down to the chamfer's depth Z{corner_z:.3} - a {width} mm chamfer there would ride into the material under it; lower the width or pick the edges to break",
                        at.x, at.y
                    ),
                    at: Point3::new(at.x, at.y, corner_z),
                });
            }
        }
    }
    // #2596: where the model carries a bevel at the top plane, the model
    // says which edges are chamfered - a sharp edge beside it is sharp
    // by design and is left alone (pick it to break it). A model with
    // no bevel at all breaks every top edge, as before (#303).
    if out.is_empty() {
        out = sharp;
        return Region::new(out).map_err(GenError::Geo);
    }
    let sides = |loops: &[Loop]| loops.iter().fold((0, 0), |(o, h), l| if l.is_ccw() { (o + 1, h) } else { (o, h + 1) });
    let want = sides(&out);
    let at = out.first().and_then(|l| l.points().first().copied()).unwrap_or(Point2::new(0.0, 0.0));
    let got = Region::new(out).map_err(GenError::Geo)?;
    // A bevelled edge whose enclosing edge is sharp (a modelled chamfer
    // on a pocket in a sharp plate) loses its nesting once the sharp
    // edge is left out: the region would read it the other way round,
    // and the ride would travel against its material. Refuse by name.
    if sides(got.loops()) != want {
        return Err(GenError::At {
            msg: format!(
                "the edge near X{:.3} Y{:.3} carries a modelled chamfer, but the edge around it is sharp - a chamfer with no picks cannot ride it alone; pick the edges to break",
                at.x, at.y
            ),
            at: Point3::new(at.x, at.y, corner_z),
        });
    }
    Ok(got)
}

/// #2887: a top loop bevelled on only part of its length. Judged per
/// segment of `top` - both ends and the middle against the wall ring -
/// so an extra collinear vertex, or the lack of one, changes nothing
/// (the vertex-only `gap` reads a one-sided bevel either way round).
/// A segment standing on the wall (within `SAME_EDGE`) is sharp; one
/// standing farther than `MATCH_SLACK` from it is bevelled; between
/// the two is tessellation noise and counts as neither. `Some((bevel,
/// sharp))` - a point on each - when the loop has a bevelled segment
/// AND more than `width` of sharp length: a ride of the wall would
/// cut the sharp length, a ride of the top loop would stand inside
/// the part at the bevel. #2936: a loop that is its wall moved whole by
/// the frame's `lean` (`leans`) carries no bevel at all. #2944: a
/// segment is judged in pieces, cut where a wall vertex stands over it,
/// so a side that runs on along the wall past its corner (an undercut
/// lip) reads as the sharp length plus the overhang it is.
fn partly_bevelled(top: &[Point2], wall: &[Point2], width: f64, lean: Option<Point2>) -> Option<(Point2, Point2)> {
    if wall.is_empty() || lean.is_some_and(|l| leans(top, wall, l)) {
        return None;
    }
    let mut sharp_len: f64 = 0.0;
    let mut sharp_at: Option<Point2> = None;
    let mut bevel_at: Option<Point2> = None;
    for (i, &p) in top.iter().enumerate() {
        for (a, b) in pieces(p, top[(i + 1) % top.len()], wall) {
            let mid = Point2::new((a.x + b.x) * 0.5, (a.y + b.y) * 0.5);
            let d = near(a, wall).max(near(mid, wall)).max(near(b, wall));
            if d <= SAME_EDGE {
                sharp_len += (b.x - a.x).hypot(b.y - a.y);
                sharp_at = sharp_at.or(Some(mid));
            } else if d > MATCH_SLACK {
                bevel_at = bevel_at.or(Some(mid));
            }
        }
    }
    match (bevel_at, sharp_at) {
        (Some(b), Some(s)) if sharp_len > width => Some((b, s)),
        _ => None,
    }
}

/// #2944: the segment `a`-`b` cut at the foot of each vertex of `wall`
/// that falls strictly inside it, in order from `a` to `b`.
fn pieces(a: Point2, b: Point2, wall: &[Point2]) -> Vec<(Point2, Point2)> {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let l2 = dx * dx + dy * dy;
    let mut ts: Vec<f64> = if l2 > 1e-24 {
        wall.iter().map(|w| ((w.x - a.x) * dx + (w.y - a.y) * dy) / l2).filter(|t| *t > 1e-9 && *t < 1.0 - 1e-9).collect()
    } else {
        Vec::new()
    };
    ts.sort_by(f64::total_cmp);
    let at = |t: f64| Point2::new(a.x + t * dx, a.y + t * dy);
    let mut out = Vec::with_capacity(ts.len() + 1);
    let mut from = a;
    for t in ts {
        out.push((from, at(t)));
        from = at(t);
    }
    out.push((from, b));
    out
}

/// Plan slack of the lean test, mm: the sections stand on a 1e-6 mm
/// grid, so two copies of one outline disagree by about that much.
/// Far under `MATCH_SLACK`: no bevelled segment hides in it.
const LEAN_SLACK: f64 = 1e-4;

/// #2936: `top` is the whole of `wall` moved by `lean` - the two
/// sections of a prism whose upright walls lean in the operation's
/// frame (a box stock under a tilted frame). No segment of such a loop
/// is a modelled chamfer: the edge is the same edge all round, and the
/// ride is the wall at the corner depth. Each segment of each ring -
/// both ends and the middle - must stand on the other ring. A loop
/// bevelled on part of its length is no moved copy of its wall: its
/// bevelled side moved and its sharp sides did not. #2944: the move
/// is the frame's own, never one read off the sections - a bevel on
/// one side and an equal undercut lip on the other also move the whole
/// outline, but no upright frame leans a wall.
fn leans(top: &[Point2], wall: &[Point2], lean: Point2) -> bool {
    let on = |from: &[Point2], to: &[Point2], dx: f64, dy: f64| {
        from.iter().enumerate().all(|(i, &a)| {
            let b = from[(i + 1) % from.len()];
            let mid = Point2::new((a.x + b.x) * 0.5, (a.y + b.y) * 0.5);
            [a, mid, b].iter().all(|p| near(Point2::new(p.x + dx, p.y + dy), to) <= LEAN_SLACK)
        })
    };
    !top.is_empty() && on(top, wall, -lean.x, -lean.y) && on(wall, top, lean.x, lean.y)
}

/// #2944: the plan move, from the corner plane up to the top plane
/// `depth` above it, of a wall that stands upright in the world, as the
/// operation's frame reads it. Zero with no frame or an upright one.
/// `None` when the tool axis lies in the world's XY (an upright wall has
/// no section there to move): no lean then excuses a moved loop.
#[must_use]
pub fn wall_lean(frame: Option<&TiltedFrame>, depth: f64) -> Option<Point2> {
    let Some(f) = frame else {
        return Some(Point2::new(0.0, 0.0));
    };
    // The world's +Z in the frame is (u.z, v.z, axis.z): climbing
    // `depth` along the axis climbs `depth / axis.z` along the wall.
    let k = depth / f.axis[2];
    let lean = Point2::new(f.u[2] * k, f.v[2] * k);
    (f.axis[2].abs() > 1e-9 && lean.x.is_finite() && lean.y.is_finite()).then_some(lean)
}

/// The farthest any vertex of `from` stands from the closed polyline
/// `to`, in plan (the one-sided Hausdorff gap over the vertices).
/// `+inf` when either side has no points.
fn gap(from: &[Point2], to: &[Point2]) -> f64 {
    if from.is_empty() || to.is_empty() {
        return f64::INFINITY;
    }
    from.iter()
        .map(|&p| near(p, to))
        .fold(0.0, f64::max)
}

/// Plan distance from `p` to the closed polyline `ring`.
fn near(p: Point2, ring: &[Point2]) -> f64 {
    let n = ring.len();
    (0..n)
        .map(|i| seg(p, ring[i], ring[(i + 1) % n]))
        .fold(f64::INFINITY, f64::min)
}

fn seg(p: Point2, a: Point2, b: Point2) -> f64 {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let l2 = dx * dx + dy * dy;
    let t = if l2 > 1e-24 { (((p.x - a.x) * dx + (p.y - a.y) * dy) / l2).clamp(0.0, 1.0) } else { 0.0 };
    let (qx, qy) = (a.x + t * dx - p.x, a.y + t * dy - p.y);
    (qx * qx + qy * qy).sqrt()
}

#[cfg(test)]
#[path = "chamfer_ride_tests.rs"]
mod tests;
