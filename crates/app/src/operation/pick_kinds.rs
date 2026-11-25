//! Which kinds take a user start / exit pick (#1448, #3324): one capability
//! per registry row, the one accessor that reads the picks of any kind, and
//! the one refusal of a pick that a kind does not take - in the kind's own
//! name. The design is `docs/decisions/1448-start-exit-shared.md`; each
//! family slice of #1448 changes its own arm here when its planner reads a
//! pick.
//!
//! Provenance: our own parameter model over the general machining practice
//! of a chosen entry and exit position; no vendor material.

use crate::path_picks::{PathPicks, PickRole};

use super::kinds::KindInfo;
use super::types::{OpParams, OpType};

/// What a pick means on a kind, and so how the viewport snaps it. Every
/// capability except [`Self::NotTaken`] takes a start pick.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PickCapability {
    /// A point on a closed ring: the ring is re-sequenced to start there
    /// (the 2D contour). Snaps to a wall. Takes a start and an exit.
    Ring,
    /// A point inside the region: the descent place of the first pass
    /// and the end of the last pass. Snaps to a point in the region.
    Region,
    /// The first hole of the order and the last one. Snaps to a hole.
    HoleOrder,
    /// One end of an open chain or a slot, and the other end. Snaps to
    /// an end.
    Ends,
    /// The corner of the first raster row and the end of the last row.
    /// Snaps to a corner.
    Raster,
    /// The wire's threading point (`WireContourOp.picks`), and where the
    /// cut leaves (#3332): an end of an open run, or a point of a closed
    /// profile's path.
    Threading,
    /// The kind does not take a pick; the reason, said by name in the
    /// refusal and on the operation page.
    NotTaken(&'static str),
}

/// The reason of a cutting kind whose family slice of #1448 has not shipped.
pub const PICK_OWED: &str =
    "the planner chooses the entry and the exit of this kind until its slice of #1448 ships";

/// The reason of the probe: it has no path to start.
pub const PICK_NO_PATH: &str = "a probe touches the part and cuts no path";

impl PickCapability {
    /// Does this capability take a pick of `role`?
    #[must_use]
    pub fn takes(self, role: PickRole) -> bool {
        match (self, role) {
            (Self::NotTaken(_), _) => false,
            (_, PickRole::Start | PickRole::Exit) => true,
        }
    }
}

impl KindInfo {
    /// The kind's pick capability. Exhaustive: a new kind fails to compile
    /// until it states one (the registry's own law).
    #[must_use]
    pub fn pick(&self) -> PickCapability {
        match self.ty {
            OpType::Contour => PickCapability::Ring,
            OpType::WireContour => PickCapability::Threading,
            OpType::Probe => PickCapability::NotTaken(PICK_NO_PATH),
            OpType::Face
            | OpType::Pocket
            | OpType::Slot
            | OpType::Trace
            | OpType::Adaptive2D
            | OpType::Offset2D
            | OpType::Engrave
            | OpType::Chamfer2D
            | OpType::Circular
            | OpType::Thread
            | OpType::Drill
            | OpType::Bore
            | OpType::Pocket3D
            | OpType::Finish3D
            | OpType::Deburr
            | OpType::MultiAxisDeburr
            | OpType::Contour5D
            | OpType::Parallel5D
            | OpType::Swarf
            | OpType::AdvancedSwarf
            | OpType::RotaryContour
            | OpType::RotaryParallel
            | OpType::RotaryOffset
            | OpType::MultiAxisRough
            | OpType::Flow5D
            | OpType::MultiAxisContour
            | OpType::Facing
            | OpType::ProfileRough
            | OpType::ProfileFinish
            | OpType::GrooveRough
            | OpType::GrooveFinish
            | OpType::Part
            | OpType::Threading
            | OpType::Chamfering
            | OpType::TaperTurning
            | OpType::MultiPassTurning
            | OpType::FormTurning
            | OpType::BoringTurning
            | OpType::PlasmaProfile
            // #1883: the waterjet's lead picks are owed, as the plasma's.
            | OpType::WaterjetProfile
            | OpType::TrochoidalSlot
            | OpType::SinkerBurn
            | OpType::SinkerSideBurn
            | OpType::SlotUndercut
            | OpType::VCarve => PickCapability::NotTaken(PICK_OWED),
        }
    }
}

impl OpParams {
    fn picks_ref(&self) -> Option<&PathPicks> {
        match self {
            OpParams::Face { picks, .. }
            | OpParams::Contour { picks, .. }
            | OpParams::Pocket { picks, .. }
            | OpParams::Adaptive2D { picks, .. }
            | OpParams::Offset2D { picks, .. }
            | OpParams::Engrave { picks, .. }
            | OpParams::VCarve { picks, .. }
            | OpParams::Chamfer2D { picks, .. }
            | OpParams::Circular { picks, .. }
            | OpParams::Slot { picks, .. }
            | OpParams::TrochoidalSlot { picks, .. }
            | OpParams::Trace { picks, .. }
            | OpParams::Drill { picks, .. }
            | OpParams::Pocket3D { picks, .. }
            | OpParams::Finish3D { picks, .. }
            | OpParams::Deburr { picks, .. }
            | OpParams::MultiAxisDeburr { picks, .. }
            | OpParams::Contour5D { picks, .. }
            | OpParams::Parallel5D { picks, .. }
            | OpParams::Swarf { picks, .. }
            | OpParams::AdvancedSwarf { picks, .. }
            | OpParams::RotaryContour { picks, .. }
            | OpParams::RotaryParallel { picks, .. }
            | OpParams::RotaryOffset { picks, .. }
            | OpParams::MultiAxisRough { picks, .. }
            | OpParams::Flow5D { picks, .. }
            | OpParams::MultiAxisContour { picks, .. }
            | OpParams::Facing { picks, .. }
            | OpParams::ProfileRough { picks, .. }
            | OpParams::ProfileFinish { picks, .. }
            | OpParams::GrooveRough { picks, .. }
            | OpParams::GrooveFinish { picks, .. }
            | OpParams::Part { picks, .. }
            | OpParams::Threading { picks, .. }
            | OpParams::Chamfering { picks, .. }
            | OpParams::TaperTurning { picks, .. }
            | OpParams::MultiPassTurning { picks, .. }
            | OpParams::FormTurning { picks, .. }
            | OpParams::BoringTurning { picks, .. }
            | OpParams::Bore { picks, .. }
            | OpParams::Thread { picks, .. } => Some(picks),
            OpParams::PlasmaProfile(p) => Some(&p.picks),
            OpParams::WaterjetProfile(p) => Some(&p.picks),
            OpParams::WireContour(p) => Some(&p.picks),
            OpParams::SinkerBurn(p) => Some(&p.picks),
            OpParams::SinkerSideBurn(p) => Some(&p.burn.picks),
            OpParams::SlotUndercut(p) => Some(&p.picks),
            OpParams::Probe { .. } => None,
        }
    }

    /// Mutable access to the operation-level picks for any path kind;
    /// `None` only for `Probe`.
    pub fn path_picks_mut(&mut self) -> Option<&mut PathPicks> {
        match self {
            OpParams::Face { picks, .. }
            | OpParams::Contour { picks, .. }
            | OpParams::Pocket { picks, .. }
            | OpParams::Adaptive2D { picks, .. }
            | OpParams::Offset2D { picks, .. }
            | OpParams::Engrave { picks, .. }
            | OpParams::VCarve { picks, .. }
            | OpParams::Chamfer2D { picks, .. }
            | OpParams::Circular { picks, .. }
            | OpParams::Slot { picks, .. }
            | OpParams::TrochoidalSlot { picks, .. }
            | OpParams::Trace { picks, .. }
            | OpParams::Drill { picks, .. }
            | OpParams::Pocket3D { picks, .. }
            | OpParams::Finish3D { picks, .. }
            | OpParams::Deburr { picks, .. }
            | OpParams::MultiAxisDeburr { picks, .. }
            | OpParams::Contour5D { picks, .. }
            | OpParams::Parallel5D { picks, .. }
            | OpParams::Swarf { picks, .. }
            | OpParams::AdvancedSwarf { picks, .. }
            | OpParams::RotaryContour { picks, .. }
            | OpParams::RotaryParallel { picks, .. }
            | OpParams::RotaryOffset { picks, .. }
            | OpParams::MultiAxisRough { picks, .. }
            | OpParams::Flow5D { picks, .. }
            | OpParams::MultiAxisContour { picks, .. }
            | OpParams::Facing { picks, .. }
            | OpParams::ProfileRough { picks, .. }
            | OpParams::ProfileFinish { picks, .. }
            | OpParams::GrooveRough { picks, .. }
            | OpParams::GrooveFinish { picks, .. }
            | OpParams::Part { picks, .. }
            | OpParams::Threading { picks, .. }
            | OpParams::Chamfering { picks, .. }
            | OpParams::TaperTurning { picks, .. }
            | OpParams::MultiPassTurning { picks, .. }
            | OpParams::FormTurning { picks, .. }
            | OpParams::BoringTurning { picks, .. }
            | OpParams::Bore { picks, .. }
            | OpParams::Thread { picks, .. } => Some(picks),
            OpParams::PlasmaProfile(p) => Some(&mut p.picks),
            OpParams::WaterjetProfile(p) => Some(&mut p.picks),
            OpParams::WireContour(p) => Some(&mut p.picks),
            OpParams::SinkerBurn(p) => Some(&mut p.picks),
            OpParams::SinkerSideBurn(p) => Some(&mut p.burn.picks),
            OpParams::SlotUndercut(p) => Some(&mut p.picks),
            OpParams::Probe { .. } => None,
        }
    }

    /// The start and exit picks of this operation, for every kind, read from
    /// the operation-level `picks` field (#3339). `Probe` answers empty lists.
    #[must_use]
    pub fn path_picks(&self) -> PathPicks {
        self.picks_ref().cloned().unwrap_or_default()
    }

    /// The by-name refusal of a pick that this kind does not take (#1448):
    /// it names the kind, the pick and the reason. A tilted (3+2) contour
    /// keeps its own refusal. `None` when every pick is taken.
    pub(super) fn pick_refusal(&self) -> Option<String> {
        let picks = self.path_picks();
        if let Some(why) = picks.refusal() {
            return Some(why);
        }
        if picks.is_empty() {
            return None;
        }
        let info = self.kind().info();
        let cap = info.pick();
        for role in [PickRole::Start, PickRole::Exit] {
            if picks.of(role).is_empty() {
                continue;
            }
            let what = role.word();
            if matches!(self, OpParams::Contour { tilt: Some(_), .. }) {
                return Some(format!("a {what} point is not taken on a tilted (3+2) contour yet - clear it or the tilt"));
            }
            if !cap.takes(role) {
                let why = match cap {
                    PickCapability::NotTaken(why) => why,
                    _ => "this kind takes a start point only",
                };
                return Some(format!("{}: a {what} point is not taken - {why}; clear it on this operation", info.label));
            }
        }
        None
    }
}
