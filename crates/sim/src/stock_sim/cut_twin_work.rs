//! The brick work list of the f32 cut twin (#2626, GPU stock sim
//! S1): from a batch of moves, the bricks and cells that each move's
//! window touches, and the CPU run of the kernel over that list into
//! the dense lattice layout of `preview::pack_cut`.
//!
//! Provenance: the swept windows of `field_sweep::window_of` and the
//! brick-major walk of `field_batch::cut_batch`, clipped to the dense
//! lattice of `preview::pack`; nothing from vendor binaries.
//!
//! ## The laws here
//!
//! - **Same windows as the f64 walk.** A chord's window is the f64
//!   sweep's own (`Sweep::window`), clipped to the lattice box (the
//!   lattice lies inside the f64 cut region, `field_batch::region_of`
//!   pads further). The work list is integer data, so it is made in
//!   f64 on the CPU and only the cell kernel runs in f32.
//! - **Refuse by name, never cut another way.** Only the ball and the
//!   flat end mill, upright, on linear moves have a twin. Any other
//!   tool, an arc, a tilted pose, a zero-length move, a non-finite
//!   value or a non-positive tool size refuses before any output.
//!   Rapids touch nothing (RS-274 G0, the f64 law).
//! - **Deterministic order.** Items are sorted brick-major, then by
//!   chord; the max law makes the result independent of that order.

use super::cut_twin::{cell_cut, from_ordered_bits, ordered_bits, TileChord, TwinChord, TwinTool};
use super::field::{brick_of, BRICK};
use super::field_cut::{chord_sweep, SdfTool};
use super::field_solid::{tool_kind, ToolKind};
use super::kernel::StockError;
use super::preview::{pack::MAX_TEXELS, LatticeMeta};

use crate::cl::{Move, MoveKind};

/// Refusal ceiling of one chord's clipped window, in cells: the f64
/// walk's `MAX_CELLS_PER_MOVE`, the same value.
pub const TWIN_MAX_CELLS_PER_CHORD: u64 = 1 << 22;

/// One unit of kernel work: chord `chord` against brick `brick`, on
/// the cells `lo..=hi` (world cell keys, inside the brick and the
/// lattice).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorkItem {
    pub brick: [i32; 3],
    pub chord: u32,
    pub lo: [i32; 3],
    pub hi: [i32; 3],
}

/// The chords of a batch and the brick work list over them.
#[derive(Clone, Debug, PartialEq)]
pub struct TwinWork {
    pub chords: Vec<TwinChord>,
    pub items: Vec<WorkItem>,
}

fn invalid(msg: String) -> StockError {
    StockError::Invalid(format!("f32 cut twin: {msg}"))
}

/// The tool's twin, or its refusal by name.
fn twin_tool(tool: &SdfTool) -> Result<TwinTool, StockError> {
    let name = match tool {
        SdfTool::Ball { .. } | SdfTool::Flat { .. } => None,
        SdfTool::Shanked { .. } => Some("shanked ball"),
        SdfTool::Taper { .. } => Some("taper"),
        SdfTool::Drill { .. } => Some("drill"),
        SdfTool::BullNose { .. } => Some("bull nose"),
        SdfTool::Dovetail { .. } => Some("dovetail"),
        SdfTool::Lathe { .. } => Some("lathe"),
    };
    if let Some(n) = name {
        return Err(invalid(format!(
            "the {n} tool has no f32 twin; only the ball and the flat end mill cut here"
        )));
    }
    // The f64 gate checks the sizes (positive, finite) by name.
    match tool_kind(tool)? {
        ToolKind::Ball(r) => Ok(TwinTool::Ball { r }),
        ToolKind::Flat { r, flank } => Ok(TwinTool::Flat { r, flank }),
        _ => Err(invalid("the tool has no f32 twin".into())),
    }
}

/// The lattice box `[lo, hi]` per axis, or a refusal of a bad meta.
fn lattice_box(meta: &LatticeMeta) -> Result<[(i32, i32); 3], StockError> {
    if !(meta.cell.is_finite() && meta.cell > 0.0 && meta.cap.is_finite() && meta.cap > 0.0) {
        return Err(invalid(format!(
            "the lattice cell {} and cap {} must be positive finite numbers",
            meta.cell, meta.cap
        )));
    }
    let n = (meta.dims[0] as usize)
        .checked_mul(meta.dims[1] as usize)
        .and_then(|n| n.checked_mul(meta.dims[2] as usize));
    if !n.is_some_and(|n| n > 0 && n <= MAX_TEXELS) {
        return Err(invalid(format!("the lattice dims {:?} are empty or over the texel budget", meta.dims)));
    }
    let mut out = [(0, 0); 3];
    for (a, o) in out.iter_mut().enumerate() {
        let hi = i64::from(meta.origin[a]) + i64::from(meta.dims[a]) - 1;
        let hi = i32::try_from(hi).map_err(|_| invalid(format!("the lattice axis {a} leaves the i32 range")))?;
        *o = (meta.origin[a], hi);
    }
    Ok(out)
}

/// The chords of `moves` and the brick work list that their windows
/// touch inside `meta`'s lattice. Validates the whole batch first.
pub fn twin_work(moves: &[Move], tool: &SdfTool, meta: &LatticeMeta) -> Result<TwinWork, StockError> {
    let tt = twin_tool(tool)?;
    let bx = lattice_box(meta)?;
    let kind = tool_kind(tool)?;
    let up = [0.0, 0.0, 1.0];
    let mut chords = Vec::new();
    let mut items = Vec::new();
    for m in moves {
        if !m.is_finite() {
            return Err(invalid("the move has a non-finite endpoint".into()));
        }
        if m.kind == MoveKind::Rapid {
            continue;
        }
        if m.arc.is_some() {
            return Err(invalid("an arc move has no twin; only linear moves cut here".into()));
        }
        if let Some(a) = m.axis
            && !(a[0] == 0.0 && a[1] == 0.0 && a[2].is_finite() && a[2] > 0.0)
        {
            return Err(invalid(format!("the tool axis {a:?} is not upright; only +Z cuts here")));
        }
        if m.from == m.to {
            return Err(invalid("a zero-length move has no swept solid".into()));
        }
        let sw = chord_sweep(&kind, up, m.from, m.to)?;
        let (a, b) = match tt {
            TwinTool::Ball { r } => ([m.from.x, m.from.y, m.from.z + r], [m.to.x, m.to.y, m.to.z + r]),
            TwinTool::Flat { .. } => ([m.from.x, m.from.y, m.from.z], [m.to.x, m.to.y, m.to.z]),
        };
        let n = u32::try_from(chords.len()).map_err(|_| invalid("too many chords in one batch".into()))?;
        chords.push(TwinChord { a, b, tool: tt });
        let w = sw.window(meta.cell, meta.cap, up);
        let c: [(i32, i32); 3] = std::array::from_fn(|i| (w[i].0.max(bx[i].0), w[i].1.min(bx[i].1)));
        if c.iter().any(|s| s.0 > s.1) {
            continue; // the whole reach lies outside the lattice
        }
        let cells = c
            .iter()
            .fold(1u64, |acc, s| acc.saturating_mul((i64::from(s.1) - i64::from(s.0) + 1) as u64));
        if cells > TWIN_MAX_CELLS_PER_CHORD {
            return Err(invalid(format!(
                "a chord's window spans more than {TWIN_MAX_CELLS_PER_CHORD} cells at cell size {}; segment the move",
                meta.cell
            )));
        }
        let (blo, bhi) = (brick_of([c[0].0, c[1].0, c[2].0]), brick_of([c[0].1, c[1].1, c[2].1]));
        for kz in blo[2]..=bhi[2] {
            for ky in blo[1]..=bhi[1] {
                for kx in blo[0]..=bhi[0] {
                    let bk = [kx, ky, kz];
                    let lo: [i32; 3] = std::array::from_fn(|i| bk[i].saturating_mul(BRICK).max(c[i].0));
                    let hi: [i32; 3] =
                        std::array::from_fn(|i| bk[i].saturating_mul(BRICK).saturating_add(BRICK - 1).min(c[i].1));
                    items.push(WorkItem { brick: bk, chord: n, lo, hi });
                }
            }
        }
    }
    items.sort_by_key(|w| (w.brick, w.chord));
    Ok(TwinWork { chords, items })
}

/// Run the f32 kernel over a work list: the cut layer in the dense
/// layout of `pack_cut` over `meta` (z-major, then y, then x), with
/// the same `-4 cap` floor where no chord claims a cell.
pub fn run_twin(work: &TwinWork, meta: &LatticeMeta) -> Result<Vec<f32>, StockError> {
    let bx = lattice_box(meta)?;
    let dims = [meta.dims[0] as usize, meta.dims[1] as usize, meta.dims[2] as usize];
    let floor = ordered_bits((-4.0 * meta.cap) as f32);
    let mut keys = vec![floor; dims[0] * dims[1] * dims[2]];
    let (cell32, cap32) = (meta.cell as f32, meta.cap as f32);
    for it in &work.items {
        let Some(ch) = work.chords.get(it.chord as usize) else {
            return Err(invalid(format!("work item names chord {} of {}", it.chord, work.chords.len())));
        };
        let b0 = [
            it.brick[0].saturating_mul(BRICK),
            it.brick[1].saturating_mul(BRICK),
            it.brick[2].saturating_mul(BRICK),
        ];
        let origin = [f64::from(b0[0]) * meta.cell, f64::from(b0[1]) * meta.cell, f64::from(b0[2]) * meta.cell];
        let tc = TileChord::of(ch, origin, meta.cell);
        for k in it.lo[2]..=it.hi[2] {
            for j in it.lo[1]..=it.hi[1] {
                for i in it.lo[0]..=it.hi[0] {
                    let key = [i, j, k];
                    let outside = || invalid(format!("work item cell {key:?} lies outside its brick or the lattice"));
                    let mut q = [0.0f32; 3];
                    let mut at = [0usize; 3];
                    for a in 0..3 {
                        // the local index must be in 0..BRICK, the
                        // lattice offset in 0..dims: a hand-made list
                        // refuses, never writes a wrapped slot
                        let local = key[a].saturating_sub(b0[a]);
                        let off = usize::try_from(i64::from(key[a]) - i64::from(bx[a].0)).map_err(|_| outside())?;
                        if !(0..BRICK).contains(&local) || off >= dims[a] {
                            return Err(outside());
                        }
                        q[a] = (local as f32 + 0.5) * cell32;
                        at[a] = off;
                    }
                    let Some(v) = cell_cut(&tc, q, cap32) else {
                        continue;
                    };
                    let slot = (at[2] * dims[1] + at[1]) * dims[0] + at[0];
                    let Some(dst) = keys.get_mut(slot) else {
                        return Err(outside());
                    };
                    *dst = (*dst).max(ordered_bits(v));
                }
            }
        }
    }
    Ok(keys.into_iter().map(from_ordered_bits).collect())
}

/// [`twin_work`] then [`run_twin`]: the f32 cut layer of a batch.
pub fn twin_cut(moves: &[Move], tool: &SdfTool, meta: &LatticeMeta) -> Result<Vec<f32>, StockError> {
    run_twin(&twin_work(moves, tool, meta)?, meta)
}
