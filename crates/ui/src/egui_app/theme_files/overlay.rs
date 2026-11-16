//! The overlay of a parsed theme file on its base table (moved unchanged out
//! of `theme_files.rs` for the 400-line law, #2021).
//! Provenance: our own format; every name and clamp is ours.

use std::collections::HashMap;

use egui::Color32;

use super::names;
use crate::egui_app::theme::{Tokens, clamp};

/// Overlay the parsed maps on the base table. The destructuring is the
/// M6.1 A exhaustiveness law, restated for files: a token added to
/// `Tokens` must be named here (pattern AND lookup) or the build
/// fails - the format can never be able to set less than the whole
/// table. Geometry lands clamped, so every reader (draw sites
/// included) sees shell-legal numbers.
pub(super) fn apply_overlay(
    base: Tokens,
    colors: &HashMap<String, Color32>,
    geom: &HashMap<String, f64>,
) -> Tokens {
    let Tokens {
        ink, ink_alt, well, text, text_dim, text_faint, border, accent,
        accent_hover, field_bg, field_hover, field_active, selection_bg,
        selection_stroke, viewport, radius_widget, radius_window, radius_menu,
        item_spacing_x, item_spacing_y, button_padding_x, button_padding_y,
        control_height, row_height, cell_h, cell_w, glyph_px, font_label,
        font_caption, font_row, font_row_suffix, ribbon_ink, ribbon_tab_h,
        ribbon_cell_top_gap, ribbon_caption_gap, ribbon_caption_row_h,
        ribbon_bottom_gap, menu_width, recent_width, menu_gap, menu_pad_x,
    } = base;
    let c = |name: &str, keep: Color32| colors.get(name).copied().unwrap_or(keep);
    let f = |name: &str, keep: f32, win: (f32, f32)| {
        geom.get(name).map(|v| clamp::range(*v as f32, win)).unwrap_or(keep)
    };
    let rad = |name: &str, keep: u8, cap: f32| {
        geom.get(name)
            .map(|v| clamp::range(*v as f32, (0.0, cap)).round() as u8)
            .unwrap_or(keep)
    };
    Tokens {
        ink: c(names::INK, ink),
        ink_alt: c(names::INK_ALT, ink_alt),
        well: c(names::WELL, well),
        text: c(names::TEXT, text),
        text_dim: c(names::TEXT_DIM, text_dim),
        text_faint: c(names::TEXT_FAINT, text_faint),
        border: c(names::BORDER, border),
        accent: c(names::ACCENT, accent),
        accent_hover: c(names::ACCENT_HOVER, accent_hover),
        field_bg: c(names::FIELD_BG, field_bg),
        field_hover: c(names::FIELD_HOVER, field_hover),
        field_active: c(names::FIELD_ACTIVE, field_active),
        selection_bg: c(names::SELECTION_BG, selection_bg),
        selection_stroke: c(names::SELECTION_STROKE, selection_stroke),
        viewport: c(names::VIEWPORT, viewport),
        ribbon_ink: c(names::RIBBON_INK, ribbon_ink),
        radius_widget: rad(names::RADIUS_WIDGET, radius_widget, clamp::RADIUS as f32),
        radius_window: rad(names::RADIUS_WINDOW, radius_window, 24.0),
        radius_menu: rad(names::RADIUS_MENU, radius_menu, 24.0),
        item_spacing_x: f(names::ITEM_SPACING_X, item_spacing_x, clamp::SPACING),
        item_spacing_y: f(names::ITEM_SPACING_Y, item_spacing_y, clamp::SPACING),
        button_padding_x: f(names::BUTTON_PADDING_X, button_padding_x, clamp::SPACING),
        button_padding_y: f(names::BUTTON_PADDING_Y, button_padding_y, clamp::SPACING),
        control_height: f(names::CONTROL_HEIGHT, control_height, clamp::CONTROL_H),
        row_height: f(names::ROW_HEIGHT, row_height, clamp::CONTROL_H),
        cell_h: f(names::CELL_H, cell_h, clamp::CELL),
        cell_w: f(names::CELL_W, cell_w, clamp::CELL),
        glyph_px: f(names::GLYPH_PX, glyph_px, clamp::GLYPH),
        font_label: f(names::FONT_LABEL, font_label, clamp::FONT),
        font_caption: f(names::FONT_CAPTION, font_caption, clamp::FONT),
        font_row: f(names::FONT_ROW, font_row, clamp::FONT),
        font_row_suffix: f(names::FONT_ROW_SUFFIX, font_row_suffix, clamp::FONT),
        ribbon_tab_h: f(names::RIBBON_TAB_H, ribbon_tab_h, clamp::RIBBON_ROW),
        ribbon_cell_top_gap: f(names::RIBBON_CELL_TOP_GAP, ribbon_cell_top_gap, clamp::RIBBON_GAP),
        ribbon_caption_gap: f(names::RIBBON_CAPTION_GAP, ribbon_caption_gap, clamp::RIBBON_GAP),
        ribbon_caption_row_h: f(names::RIBBON_CAPTION_ROW_H, ribbon_caption_row_h, clamp::RIBBON_ROW),
        ribbon_bottom_gap: f(names::RIBBON_BOTTOM_GAP, ribbon_bottom_gap, clamp::RIBBON_GAP),
        menu_width: f(names::MENU_WIDTH, menu_width, clamp::MENU_W),
        recent_width: f(names::RECENT_WIDTH, recent_width, clamp::RECENT_W),
        menu_gap: f(names::MENU_GAP, menu_gap, clamp::MENU_GAP),
        menu_pad_x: f(names::MENU_PAD_X, menu_pad_x, clamp::MENU_PAD_X),
    }
}
