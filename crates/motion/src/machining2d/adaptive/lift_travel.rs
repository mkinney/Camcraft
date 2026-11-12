//! A plan's travel (M11.4 C round 4): the passes walked in order on the
//! planner's own stock, every segment read cut or floor air by the
//! front-half contact angle (`raster`), plus the lifted returns' legs
//! and the retract hops - what the pocket's travel law weighs. Split
//! from `lift` (the 400-line law). Ours; no vendor material consulted.

use crate::geo2d::Point2;

use super::front::dist;
use super::lift_link::{Return, legs};
use super::raster::Raster;

/// A plan's travel, per level (XY mm): in cut, in air at the floor and
/// lifted; and the moves it posts.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Travel {
    pub cut: f64,
    pub air: f64,
    pub lift: f64,
    pub moves: usize,
    /// Passes reached by the ladder's retract hop rather than a microlift
    /// return (the entry's own first pass aside).
    pub hops: usize,
}

/// The travel of `passes` (reached by `joins`) walked in order from
/// `ras0`, read by `(r, rim, step, bite)`: a segment reading over the
/// bite is cut, the rest floor air.
pub(super) fn travel(ras0: &Raster, passes: &[Vec<Point2>], joins: &[Option<Return>], (r, rim, step, bite): (f64, f64, f64, f64)) -> Travel {
    let mut ras = ras0.clone();
    let mut t = Travel::default();
    for (k, pass) in passes.iter().enumerate() {
        if let (Some(Some(ret)), Some(from)) = (joins.get(k), k.checked_sub(1).and_then(|j| passes[j].last())) {
            let lg = legs(*from, ret);
            t.lift += lg.iter().map(|l| dist(l.0, l.1)).sum::<f64>();
            t.moves += lg.len();
        } else if k > 0 {
            t.hops += 1;
        }
        for (w, m) in pass.windows(2).zip(ras.walk_marks(pass, (r, rim, step))) {
            if m > bite {
                t.cut += dist(w[0], w[1]);
            } else {
                t.air += dist(w[0], w[1]);
            }
        }
        t.moves += pass.len().saturating_sub(1);
    }
    t
}
