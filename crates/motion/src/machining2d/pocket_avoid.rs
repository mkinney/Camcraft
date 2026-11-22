//! 2D Offset / Pocket path planning with per-level "Do not touch" keep-outs (#2055).
//! Provenance: our own per-Z-level region difference over `levels` and `verify::avoid`;
//! no vendor material consulted.

use camcraft_sim::verify::{AVOID_PAD, AvoidSpec};

use crate::cl::{MoveKind, PathBuilder, Toolpath};
use crate::geo2d::Region;
use crate::geometry::Point3;

use super::arcs::{ARC_FIT_TOL, Seg, fit_arcs};
use super::corner_relief::{CornerRelief, capsule_region, plan_corner_reliefs};
use super::corner_relief_spur::inject_relief_spurs;
use super::fillet::{Legal, fillet_corners};
use super::fillet_tally::{FilletTally, corner_arc_radius, fillet_floor};
use super::equal_steps::cut_levels_for;
use super::levels::{Cert, Motion, WalkState, walk_one_level};
use super::pocket::{PocketParams, PocketStrategy, non_sliver_center, plan_passes};
use super::pocket_zigzag::rest_passes;
use super::{GenError, pocket_entered_with_relief};

/// Plan a 2D pocket/offset path whose tool assembly stays clear of `avoid`.
pub fn pocket_avoid(
    region: &Region,
    p: &PocketParams,
    avoid: &AvoidSpec,
) -> Result<Toolpath, GenError> {
    pocket_avoid_with_relief(region, p, avoid, &CornerRelief::default())
}

/// [`pocket_avoid`] with optional inside-corner reliefs (#2400).
pub fn pocket_avoid_with_relief(
    region: &Region,
    p: &PocketParams,
    avoid: &AvoidSpec,
    relief: &CornerRelief,
) -> Result<Toolpath, GenError> {
    if avoid.is_empty() {
        return pocket_entered_with_relief(region, p, &[], relief);
    }
    p.validate()?;
    relief.validate()?;
    if region.is_empty() {
        return Err(GenError::Empty("empty pocket region".into()));
    }
    let off = p.tool_diameter * 0.5 + p.stock_to_leave;
    let base_center = non_sliver_center(region, off)?;
    if base_center.is_empty() {
        return Err(GenError::Invalid("pocket is narrower than the tool".into()));
    }
    let relief_plan = plan_corner_reliefs(region, &base_center, off, relief)?;

    let strategy = match p.strategy {
        PocketStrategy::Auto => {
            let outer = region
                .loops()
                .iter()
                .max_by(|a, b| a.area().abs().total_cmp(&b.area().abs()))
                .map(|l| l.points())
                .unwrap_or(&[]);
            p.strategy.resolve(outer, p.stepover, p.tool_diameter, ARC_FIT_TOL)
        }
        s => s,
    };
    let zs = cut_levels_for(p.top_z, &p.links.approach, p.bottom_z, p.stepdown)?;
    let mut b = PathBuilder::approached(Point3::new(0.0, 0.0, p.clearance_z));
    let mut ws = WalkState::new(p.links.approach.entry_top(p.top_z));
    let mut prev_center: Option<Region> = None;
    let mut blocked_levels: Vec<f64> = Vec::new();
    let mut notes: Vec<String> = Vec::new();
    let mut tally = FilletTally::default();

    for &z in &zs {
        let ko = avoid
            .keepout_region(z, z, AVOID_PAD)
            .map_err(GenError::Invalid)?;
        let level_center = if ko.is_empty() {
            base_center.clone()
        } else {
            for r in &relief_plan.reliefs {
                if let Ok(cap) = capsule_region(r.start, r.end, r.radius)
                    && !cap.intersect(&ko).map(|i| i.is_empty()).unwrap_or(false)
                {
                    return Err(GenError::Invalid(format!(
                        "corner relief at ({:.3}, {:.3}) enters protected 'Do not touch' zone ({})",
                        r.corner.x,
                        r.corner.y,
                        avoid.names_list()
                    )));
                }
            }
            base_center.difference(&ko)?
        };
        if level_center.is_empty() {
            blocked_levels.push(z);
            continue;
        }
        // Check overhead protected obstacles along descent/plunge from top/retract plane.
        let hop = if ws.last_cut_z.is_none()
            || matches!(p.links.level, crate::links::Link::Clearance)
            || matches!(p.links.pass, crate::links::Link::Clearance)
        {
            p.clearance_z
        } else {
            p.retract_z.unwrap_or(p.clearance_z)
        };
        let top_above = hop.max(z);
        let ko_entry = avoid
            .keepout_region(z, top_above, AVOID_PAD)
            .map_err(GenError::Invalid)?;
        let level_center = if ko_entry.is_empty() {
            level_center
        } else {
            let sub = level_center.difference(&ko_entry)?;
            if sub.is_empty() {
                blocked_levels.push(z);
                continue;
            }
            sub
        };
        let (passes, actual, room, joins) = if strategy == PocketStrategy::Adaptive {
            let what = format!("level at Z {z:.2}");
            let (ps, anotes, act, rm, js) =
                super::pocket_hsm::adaptive_passes(&level_center, p, &what, true, None)?;
            notes.extend(anotes);
            (ps, act, rm, js)
        } else if matches!(strategy, PocketStrategy::Zigzag | PocketStrategy::Auto) {
            // #3129: rows alone leave a cusp between row ends on each wall,
            // island and keep-out edge; the level closes with its rings.
            (rest_passes(&level_center, p.tiling_step(), p.climb)?, strategy, None, Vec::new())
        } else {
            (plan_passes(&level_center, p.tiling_step(), p.climb, strategy)?, strategy, None, Vec::new())
        };
        let fit = match actual {
            PocketStrategy::Waterline | PocketStrategy::OffsetRings => Some(ARC_FIT_TOL),
            PocketStrategy::Zigzag | PocketStrategy::Auto | PocketStrategy::Adaptive => None,
        };
        let mut passes = if p.links.approach.arc_xy && actual != PocketStrategy::Adaptive {
            super::descent_xy::reseat_first(passes)
        } else {
            passes
        };
        if passes.iter().all(|ps| ps.len() < 2) {
            blocked_levels.push(z);
            continue;
        }
        let mut runs: Option<Vec<Vec<Seg>>> = if p.blend_turns && actual != PocketStrategy::Adaptive {
            let legal = Legal { region: &level_center, inside: true, air_left: None };
            let rho = corner_arc_radius(p.blend_radius, p.tool_diameter).map_err(GenError::Invalid)?;
            let r = passes
                .iter()
                .map(|pass| {
                    let Some((first, rest)) = pass.split_first() else {
                        return Vec::new();
                    };
                    let walk = match fit {
                        Some(tol) => fit_arcs(pass, tol),
                        None => vec![Seg::Line(rest.to_vec())],
                    };
                    let (walk, t) = fillet_corners(&walk, *first, rho, ARC_FIT_TOL, &legal);
                    tally.add(t);
                    walk
                })
                .collect();
            Some(r)
        } else {
            fit.map(|tol| passes.iter().map(|ps| fit_arcs(ps, tol)).collect())
        };
        if !relief_plan.reliefs.is_empty() {
            inject_relief_spurs(&mut passes, &mut runs, &level_center, &relief_plan.reliefs);
        }
        let hr = room.as_ref().map_or(p.helix_radius, |rm| p.helix_radius.min(rm.helix_r));
        let level_motion = Motion::new(p.clearance_z, p.retract_z, p.feed, p.plunge_feed)
            .with_entry(p.entry, hr)
            .with_arc_radius(p.tool_diameter)
            .with_links(p.links, hr)
            .with_top(p.links.approach.entry_top(p.top_z))
            .with_xy_arc(p.links.approach.arc_xy)
            .with_microlift(joins)
            .with_solid_restart();
        let cert_entry = room.as_ref().map(|rm| Cert::Region(&rm.tess));
        let cert_links = Cert::Region(&level_center);
        walk_one_level(
            &mut b,
            z,
            &passes,
            runs.as_deref(),
            &level_motion,
            Some(cert_entry.as_ref().unwrap_or(&cert_links)),
            Some(&cert_links),
            prev_center.as_ref(),
            &mut ws,
        );
        prev_center = Some(level_center);
    }

    if ws.last_cut_z.is_none() {
        return Err(GenError::Invalid(format!(
            "cannot cut clear of protected 'Do not touch' {}: no safe cutting room remains",
            avoid.names_list()
        )));
    }
    for n in notes {
        b.note(n);
    }
    for n in tally.notes("pass", fillet_floor(ARC_FIT_TOL)) {
        b.note(n);
    }
    let end_motion = Motion::new(p.clearance_z, p.retract_z, p.feed, p.plunge_feed)
        .with_entry(p.entry, p.helix_radius)
        .with_arc_radius(p.tool_diameter)
        .with_links(p.links, p.helix_radius)
        .with_top(p.links.approach.entry_top(p.top_z));
    ws.finish(&mut b, &end_motion);
    let cur = b.at();
    b.push(MoveKind::Rapid, Point3::new(cur.x, cur.y, p.clearance_z), None);
    if !blocked_levels.is_empty() {
        b.note(format!(
            "Do not touch ({}) blocked {} cut level(s); remaining levels stayed clear",
            avoid.names_list(),
            blocked_levels.len()
        ));
    }
    let mut path = b.finish();
    let _ = camcraft_sim::verify::elevate_rapid_links(&mut path, avoid, None, AVOID_PAD);
    if let Some(why) = crate::approach::refusal(&path) {
        return Err(GenError::Invalid(why).with_ghost(&path));
    }
    if !path.all_finite() {
        return Err(GenError::Invalid("path left the finite horizon".into()));
    }
    Ok(path)
}
