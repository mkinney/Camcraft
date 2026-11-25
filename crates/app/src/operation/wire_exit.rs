//! The Wire Contour's exit pick read from the operation (#3332, part of
//! #1448): the stored pick as the generator's point, and the by-name
//! refusal of an exit pick beside a cut that does not take one yet. The
//! exit is cut on a vertical one-pass cut of one polygon profile or one
//! open run (`motion::wire_edm::exit`); every other cut is refused, never
//! cut as if the pick were not stated.
//!
//! Provenance: our own parameter law over the shared pick design of
//! #3324; no vendor material.

use crate::geo2d::Point2;
use crate::path_picks::PickRole;

use super::types::WireContourOp;

/// The operation's exit pick in model XY, if one is stated.
#[must_use]
pub fn exit_of(op: &WireContourOp) -> Option<Point2> {
    op.picks.first(PickRole::Exit).map(|p| p.xy)
}

/// An exit pick beside a cut that does not take one yet: a 4-axis cut,
/// a no-core burnout, skim passes (`skims`: the condition states some),
/// a taper, or a drawn line-and-arc profile (`drawn`). `None` when no
/// exit is stated or the cut takes it.
#[must_use]
pub fn exit_refusal(op: &WireContourOp, skims: bool, drawn: bool) -> Option<String> {
    let exit = exit_of(op)?;
    let with = if op.four_axis.is_some() {
        "a 4-axis cut".to_owned()
    } else if op.burnout.is_some() {
        "a no-core burnout".to_owned()
    } else if skims || !op.skims.is_empty() {
        "skim passes".to_owned()
    } else if op.taper_deg != 0.0 {
        format!("a {:.3} degree taper", op.taper_deg)
    } else if drawn {
        "a selected drawing profile".to_owned()
    } else {
        return None;
    };
    Some(format!(
        "wire contour: an exit point (X{:.3} Y{:.3}) is stated with {with} - the exit point is taken on a vertical one-pass cut of a model profile or an open run only; clear the exit point or that setting",
        exit.x, exit.y
    ))
}
