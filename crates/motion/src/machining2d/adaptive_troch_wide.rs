//! Wide straight lanes for `adaptive_troch` (#1587): in an open area the
//! loop band follows the free width, widest lane first. The widest point
//! is the largest inscribed circle (the grassfire skeleton's maximum,
//! Blum 1967; Held 1991, "On the Computational Geometry of Pocket
//! Machining"); the lane through it is a scanline of the area eroded by
//! the band. The law is ours:
//!
//! - a lane is ONE straight segment, so its front arcs are translates of
//!   one another and never cross, whatever the band (`trochoid_loops`);
//! - the segment is a row of the free area eroded by the band and the
//!   wall margin, so every loop lies in the free area by construction,
//!   and no two lanes share cutter-centre ground;
//! - the step is still the measured-engagement ladder's (`plan_lane_safe`)
//!   and the finished path is still checked move by move (`adaptive_troch`).
//!
//! A band no wider than [`NARROW_PER_R`] of the tool radius is left to the
//! narrow passes of `adaptive_troch_lanes`. No vendor material.

use crate::geo2d::{Point2, Region};

use super::adaptive::widest_point;
use super::adaptive_troch::AdaptiveTrochParams;
use super::adaptive_troch_lanes::plan_lane_safe;
use super::adaptive_troch_slice::{candidate_angles, layout_confined, rotate_pt, rotate_region};
use super::face::GenError;
use super::pocket::PocketParams;
use super::pocket_hsm::components;
use super::trochoid_slot::LaneLayout;
use super::waterscan::scan_row;

/// The loops stay this far (mm) inside the free area's walls.
const WALL_MARGIN: f64 = 0.01;

/// The gap (mm) between the cutter-centre bands of two lanes.
const LANE_GAP: f64 = 0.25;

/// A band at most this share of the tool radius is no wide lane: the
/// narrow passes cut it (their own bound).
pub(super) const NARROW_PER_R: f64 = 0.90;

/// The band is tried at these shares of the room at the widest point,
/// widest first. The first is under 1: the widest point is a grid
/// estimate, and a row through it must lie in the eroded area.
const BAND_SHARES: [f64; 4] = [0.9, 0.75, 0.6, 0.45];

/// Wide lanes at most for one connected piece (never loop forever).
const MAX_WIDE_LANES: usize = 24;

/// A lane candidate: the area its loops sweep, its band and its line.
type Candidate = (f64, f64, [Point2; 2]);

/// Lay the widest straight lanes of `comp` (see the module law), widest
/// first, until no band wider than the narrow passes' fits. An asked loop
/// width is the user's own bound: nothing is laid here.
#[allow(clippy::too_many_arguments)]
pub(super) fn fill_wide(
    comp: &Region,
    center: &Region,
    reach: &Region,
    p: &PocketParams,
    t: &AdaptiveTrochParams,
    layouts: &mut Vec<LaneLayout>,
    used_center: &mut Region,
    cleared: &mut Region,
) -> Result<(), GenError> {
    if t.loop_width.is_some() {
        return Ok(());
    }
    let r = 0.5 * p.tool_diameter;
    for _ in 0..MAX_WIDE_LANES {
        let avail = if used_center.is_empty() { comp.clone() } else { comp.difference(&used_center.offset(LANE_GAP)?)? };
        if avail.is_empty() {
            break;
        }
        let uncut = reach.difference(cleared)?;
        let mut placed = false;
        for sub in components(&avail)? {
            let mut cands = candidates(&sub, r)?;
            cands.sort_by(|a, b| b.0.total_cmp(&a.0));
            for (_, b, seg) in cands.into_iter().take(3) {
                let chain = vec![seg[0], seg[1]];
                let swept = Region::sweep(std::slice::from_ref(&chain), r + b)?;
                // A lane over ground the lanes before it already cut is no gain.
                if swept.intersect(&uncut)?.area() < 0.5 * swept.area() {
                    continue;
                }
                let Ok(lay) = plan_lane_safe(&chain, b, None, None, b > r, false, p, t) else {
                    continue;
                };
                if !layout_confined(&lay, center, false, p)? {
                    continue;
                }
                *used_center = used_center.union(&Region::sweep(std::slice::from_ref(&chain), lay.b)?)?;
                *cleared = cleared.union(&swept)?;
                layouts.push(lay);
                placed = true;
                break;
            }
        }
        if !placed {
            break;
        }
    }
    Ok(())
}

/// The straight lanes `sub` holds through its widest point: for each band
/// share and each direction, the longest row of `sub` eroded by the band.
fn candidates(sub: &Region, r: f64) -> Result<Vec<Candidate>, GenError> {
    let Some((q, widest)) = widest_point(sub) else {
        return Ok(Vec::new());
    };
    let room = widest - WALL_MARGIN;
    let mut out = Vec::new();
    for theta in candidate_angles(sub) {
        let rot = rotate_region(sub, -theta)?;
        let v = rotate_pt(q, -theta).y;
        for share in BAND_SHARES {
            let b = room * share;
            if !(b.is_finite() && b > NARROW_PER_R * r) {
                break;
            }
            let eroded = rot.offset(-(b + WALL_MARGIN))?;
            let Some((u0, u1)) = scan_row(&eroded, v, true).into_iter().max_by(|a, c| (a.1 - a.0).total_cmp(&(c.1 - c.0))) else {
                continue;
            };
            let len = u1 - u0;
            // A lane shorter than its own band is a bore, not a lane.
            if !(len.is_finite() && len >= b) {
                continue;
            }
            let trim = 0.02_f64.min(0.1 * len);
            let seg = [rotate_pt(Point2::new(u0 + trim, v), theta), rotate_pt(Point2::new(u1 - trim, v), theta)];
            let reach = r + b;
            out.push((2.0 * reach * len + std::f64::consts::PI * reach * reach, b, seg));
        }
    }
    Ok(out)
}
