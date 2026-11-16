//! Shared clamped geometry for ribbon and browser dropdown popups (#2065).
//! Provenance: our own layout metrics derived from theme tokens; no external code.

use egui::{Pos2, Rect};

use super::{Tokens, clamp};

/// Horizontal floor (px) so a tiny window never collapses a menu to 0 px.
const MIN_POPUP_W: f32 = 40.0;

/// Validated dropdown geometry and item-row spacing derived once from `Tokens`.
#[derive(Clone, Copy, Debug)]
pub struct DropGeom {
    pub menu_width: f32,
    pub recent_width: f32,
    pub gap: f32,
    pub pad_x: f32,
    pub clip_inset_x: f32,
    pub icon_px: f32,
    pub icon_center_x: f32,
    pub icon_text_x: f32,
    pub box_px: f32,
    pub box_center_x: f32,
}

impl DropGeom {
    #[must_use]
    pub fn from_tokens(t: &Tokens) -> Self {
        let row_h = clamp::range(t.row_height, clamp::CONTROL_H);
        let pad_x = clamp::range(t.menu_pad_x, clamp::MENU_PAD_X);
        let clip_inset_x = pad_x * 0.5;
        let icon_px = (row_h * 0.8).clamp(8.0, row_h.min(24.0));
        let icon_text_x = clip_inset_x + icon_px + pad_x;
        Self {
            menu_width: clamp::range(t.menu_width, clamp::MENU_W),
            recent_width: clamp::range(t.recent_width, clamp::RECENT_W),
            gap: clamp::range(t.menu_gap, clamp::MENU_GAP),
            pad_x,
            clip_inset_x,
            icon_px,
            icon_center_x: clip_inset_x + icon_px * 0.5,
            icon_text_x,
            box_px: (row_h * 0.5).clamp(6.0, 16.0),
            box_center_x: icon_text_x * 0.5,
        }
    }

    /// Fit a preferred popup width inside `avail` wide container so narrow windows stay usable.
    #[must_use]
    pub fn fit_width(&self, want: f32, avail: f32) -> f32 {
        if !avail.is_finite() || avail <= 0.0 {
            want
        } else {
            want.min(avail.max(MIN_POPUP_W))
        }
    }

    /// Shift a popup's left edge inside `bounds` so it does not overflow the right edge.
    #[must_use]
    pub fn fit_left(&self, prefer_left: f32, bounds: Rect, width: f32) -> f32 {
        prefer_left.min(bounds.right() - width).max(bounds.left())
    }

    /// Expand a menu to fit `longest_px` plus symmetric padding, never narrower than `menu_width`.
    #[must_use]
    pub fn machine_width(&self, longest_px: f32) -> f32 {
        (longest_px + 2.0 * self.pad_x).max(self.menu_width)
    }

    /// Anchor `(top_left, width)` for a browser context popup hung under `row_ix`.
    #[must_use]
    pub fn tree_anchor(&self, pane_rect: Rect, bounds: Rect, row_ix: usize, row_h: f32) -> (Pos2, f32) {
        let width = self.fit_width(self.menu_width, bounds.width());
        let left = self.fit_left(pane_rect.left(), bounds, width);
        let top = pane_rect.top() + (row_ix + 1) as f32 * row_h + self.gap;
        (Pos2::new(left, top), width)
    }
}
