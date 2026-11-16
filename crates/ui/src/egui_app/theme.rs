//! The theme's single token table (M6.1 A): every colour, radius, spacing
//! and size the shell shows is a named field here - widget path and
//! painted path read the SAME struct, so a retheme is one table and the
//! two render paths cannot drift. The law is adopted from a sibling
//! project's proven design at the user's direction (ROADMAP M6.1); the
//! values are ours - the dark column carries today's `Palette::DARK`
//! numbers verbatim so the snapshots stay meaningful through the change.
//!
//! The drift guard is structural, not a checklist: the const block below
//! destructures EVERY `Tokens` field without a `..` arm, so adding a
//! field without wiring it (or explicitly resting it in the painted
//! path) fails to build - the sibling repo's exhaustiveness pattern,
//! restated for a table that paints as well as widgets.
//! Provenance: our own module (graduated from the spike and the M6.1 A
//! rework), specced from the imgui theme we already ship, the egui
//! Visuals docs, and the sibling architecture at the user's direction;
//! no HSMWorks material.

use egui::{Color32, CornerRadius, Margin, Stroke, Style, Visuals, vec2};

use super::contrast;

mod dropdown;
mod tables;
pub use dropdown::DropGeom;

/// One theme's whole truth: colours, then geometry.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tokens {
    // -- colours: the old Palette, names unchanged (M6.1 B keys on them).
    /// Window/panel ground.
    pub ink: Color32,
    /// Ribbon / browser / header ground (one step above ink).
    pub ink_alt: Color32,
    /// Input fields and empty recesses (one step below ink).
    pub well: Color32,
    pub text: Color32,
    pub text_dim: Color32,
    pub text_faint: Color32,
    pub border: Color32,
    /// The selection/hover accent (steel blue).
    pub accent: Color32,
    pub accent_hover: Color32,
    /// Widget-field ground, by interaction state.
    pub field_bg: Color32,
    pub field_hover: Color32,
    pub field_active: Color32,
    /// Text selection and combo-open highlight.
    pub selection_bg: Color32,
    pub selection_stroke: Color32,
    /// The viewport's 3D ground (the render pass clear colour).
    pub viewport: Color32,
    /// Ink of a plain ribbon glyph, read against `strip_fill` (#2021).
    pub ribbon_ink: Color32,
    // -- geometry: the Phase-6 complaint made themeable - ribbon size,
    // input fields and button size are tokens, not literals.
    /// Corner radius of widget rects and painted cells/rows.
    pub radius_widget: u8,
    /// Corner radius of egui windows.
    pub radius_window: u8,
    /// Corner radius of menus and combo dropdowns.
    pub radius_menu: u8,
    /// egui item spacing, x/y.
    pub item_spacing_x: f32,
    pub item_spacing_y: f32,
    /// egui button padding, x/y.
    pub button_padding_x: f32,
    pub button_padding_y: f32,
    /// Input-field/combo minimum height (egui's `interact_size.y`).
    pub control_height: f32,
    /// Browser tree row height (painted::tree).
    pub row_height: f32,
    /// Ribbon cell height (painted cell box; RIBBON_H follows it).
    pub cell_h: f32,
    /// Ribbon cell width cap (panes' flow clamp; floor is geometry).
    pub cell_w: f32,
    /// Ribbon glyph edge (the PNG and the painted fallback ride it).
    pub glyph_px: f32,
    /// Cell caption size under the ribbon glyph.
    pub font_label: f32,
    /// Group caption size.
    pub font_caption: f32,
    /// Browser row label size.
    pub font_row: f32,
    /// Browser row suffix size.
    pub font_row_suffix: f32,
    /// Ribbon rows (#2021): the tab row, then the gap to the cells.
    pub ribbon_tab_h: f32,
    pub ribbon_cell_top_gap: f32,
    /// Gap from the cell box to the caption row, the row, the bottom gap.
    pub ribbon_caption_gap: f32,
    pub ribbon_caption_row_h: f32,
    pub ribbon_bottom_gap: f32,
    /// Dropdown menu width, recent list width, trigger gap and row horizontal padding (#2065).
    pub menu_width: f32,
    pub recent_width: f32,
    pub menu_gap: f32,
    pub menu_pad_x: f32,
}

/// The painted path's clamps: geometry a theme file supplies can make
/// the shell ugly, never impossible - each value is clamped to the
/// window it must live in at the draw site (M6.1 law: a 400-point
/// radius cannot blow out a checkbox, a 0-height cell cannot eat the
/// viewport).
pub mod clamp {
    /// Widget/cell corner radius.
    pub const RADIUS: u8 = 12;
    /// Item spacing and button padding (px per axis): a window, so a
    /// file can space the shell out or in, never apart.
    pub const SPACING: (f32, f32) = (0.0, 32.0);
    /// Input-field height, browser row height (px).
    pub const CONTROL_H: (f32, f32) = (12.0, 64.0);
    /// Ribbon cell height/width (px).
    pub const CELL: (f32, f32) = (24.0, 96.0);
    /// Ribbon glyph edge (px).
    pub const GLYPH: (f32, f32) = (8.0, 64.0);
    /// Caption/row font sizes (px).
    pub const FONT: (f32, f32) = (6.0, 32.0);
    /// Ribbon tab row and caption row heights (px).
    pub const RIBBON_ROW: (f32, f32) = (12.0, 48.0);
    /// Ribbon gaps between rows (px).
    pub const RIBBON_GAP: (f32, f32) = (0.0, 24.0);
    /// Dropdown menu default width (px).
    pub const MENU_W: (f32, f32) = (100.0, 480.0);
    /// Recent-files dropdown width (px).
    pub const RECENT_W: (f32, f32) = (120.0, 640.0);
    /// Gap between a trigger button and its dropdown (px).
    pub const MENU_GAP: (f32, f32) = (0.0, 24.0);
    /// Horizontal text inset inside dropdown rows (px).
    pub const MENU_PAD_X: (f32, f32) = (0.0, 32.0);

    #[must_use]
    pub fn range(v: f32, lo_hi: (f32, f32)) -> f32 {
        if v.is_finite() { v.clamp(lo_hi.0, lo_hi.1) } else { lo_hi.0 }
    }
}

/// The ribbon strip's height: the tab row, its gap, the cell box, the
/// caption gap, the caption row and the bottom gap, all from the theme
/// (88 at the built-in tables). A theme that grows a row grows the ribbon.
#[must_use]
pub const fn ribbon_h(t: &Tokens) -> f32 {
    t.ribbon_tab_h
        + t.ribbon_cell_top_gap
        + t.cell_h
        + t.ribbon_caption_gap
        + t.ribbon_caption_row_h
        + t.ribbon_bottom_gap
}

impl Tokens {
    /// Clamped dropdown widths, trigger gap and item horizontal inset.
    #[must_use]
    pub fn dropdown(&self) -> DropGeom {
        DropGeom::from_tokens(self)
    }

    /// The panel ground behind the ribbon/browser strips (painted.rs).
    #[must_use]
    pub const fn strip_fill(&self) -> Color32 {
        self.ink_alt
    }

    /// Build the egui Visuals from the tokens: every default-egui colour
    /// and corner the shell shows is named here, so the widget path and
    /// the painted path cannot disagree about what the theme is.
    #[must_use]
    pub fn visuals(&self) -> Visuals {
        // The destructuring arm: every COLOUR is named here and nowhere
        // else; a new colour field without a wiring line is a build
        // error, not default-egui grey. (Geometry routes through
        // `style()`; the const guard below covers those names.)
        let Tokens {
            ink, ink_alt, well, text, text_dim, text_faint, border, accent,
            accent_hover, field_bg, field_hover, field_active, selection_bg,
            selection_stroke, viewport: _, ribbon_ink: _,
            radius_widget, radius_window, radius_menu,
            item_spacing_x: _, item_spacing_y: _,
            button_padding_x: _, button_padding_y: _,
            control_height: _, row_height: _, cell_h: _, cell_w: _,
            glyph_px: _, font_label: _, font_caption: _,
            font_row: _, font_row_suffix: _, ribbon_tab_h: _,
            ribbon_cell_top_gap: _, ribbon_caption_gap: _,
            ribbon_caption_row_h: _, ribbon_bottom_gap: _,
            menu_width: _, recent_width: _, menu_gap: _, menu_pad_x: _,
        } = *self;
        let mut v = Visuals::dark();
        v.panel_fill = ink;
        v.window_fill = ink_alt;
        v.faint_bg_color = well;
        v.extreme_bg_color = well;
        v.code_bg_color = well;
        v.override_text_color = Some(text);
        v.weak_text_color = Some(text_dim);
        v.hyperlink_color = accent_hover;
        v.window_corner_radius = CornerRadius::same(radius_window);
        v.menu_corner_radius = CornerRadius::same(radius_menu);
        v.selection = egui::style::Selection {
            bg_fill: selection_bg,
            stroke: Stroke::new(v.selection.stroke.width, selection_stroke),
        };
        let stroke_1px = Stroke::new(1.0, border);
        let r = CornerRadius::same(radius_widget);
        v.widgets.noninteractive.bg_fill = ink_alt;
        v.widgets.noninteractive.weak_bg_fill = Color32::TRANSPARENT;
        v.widgets.noninteractive.bg_stroke = stroke_1px;
        v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, text);
        v.widgets.noninteractive.corner_radius = r;
        v.widgets.inactive.bg_fill = field_bg;
        v.widgets.inactive.weak_bg_fill = ink_alt;
        v.widgets.inactive.bg_stroke = stroke_1px;
        v.widgets.inactive.fg_stroke = Stroke::new(1.0, text);
        v.widgets.inactive.corner_radius = r;
        v.widgets.hovered.bg_fill = field_hover;
        // #698: light hovers field_hover, not the accent (egui's bare title
        // glyphs took white-on-accent ink), and its title bars wear ink_alt.
        let light = super::ground_ink::light_ground(ink);
        let hover = if light { field_hover } else { accent };
        v.widgets.hovered.weak_bg_fill = hover;
        v.widgets.hovered.bg_stroke = Stroke::new(1.0, accent_hover);
        // Glyphs ride the hover fill (M6.1 D): WHITE on steel blue, black on pale.
        v.widgets.hovered.fg_stroke = Stroke::new(1.0, contrast::on_fill(hover));
        v.widgets.hovered.corner_radius = r;
        v.widgets.active.bg_fill = field_active;
        v.widgets.active.weak_bg_fill = accent_hover;
        v.widgets.active.bg_stroke = Stroke::new(1.0, accent_hover);
        // The active (pressed) fill is the HOVER accent, one step
        // lighter - and there white only reads 3.8:1 on dark's blue.
        // The derivation picks black (5.5:1): pressed chrome is
        // momentary, readable chrome is the point. The first tables
        // hand-typed white here; the arithmetic now overrules the
        // habit, which is exactly what M6.1 D exists to do.
        v.widgets.active.fg_stroke = Stroke::new(1.0, contrast::on_fill(accent_hover));
        v.widgets.active.corner_radius = r;
        v.widgets.open.bg_fill = field_bg;
        if light {
            (v.widgets.open.weak_bg_fill, v.widgets.open.fg_stroke) = (ink_alt, Stroke::new(1.0, text));
        }
        v.widgets.open.bg_stroke = stroke_1px;
        v.widgets.open.corner_radius = r;
        for exp in [
            &mut v.widgets.noninteractive.expansion,
            &mut v.widgets.inactive.expansion,
            &mut v.widgets.hovered.expansion,
            &mut v.widgets.active.expansion,
            &mut v.widgets.open.expansion,
        ] {
            *exp = 0.0; // flat chrome: hovered widgets must not bleed past their bounds
        }
        let _ = text_faint; // read by the painted path's disabled ink, not Visuals
        v
    }

    /// The style spacing the table owns: item gaps, button padding, the
    /// field height. Defaults equal egui's own numbers, so wiring the
    /// tokens in changes no pixels until a theme file overrides them.
    #[must_use]
    pub fn style(&self, base: &Style) -> Style {
        let mut s = base.clone();
        // Clamp at the seam (the M6.1 law's other half): the const guard
        // proves the built-in tables are in range; a file-supplied
        // overlay is checked here, where it can no longer hurt.
        s.spacing.item_spacing =
            vec2(clamp::range(self.item_spacing_x, clamp::SPACING), clamp::range(self.item_spacing_y, clamp::SPACING));
        s.spacing.button_padding =
            vec2(clamp::range(self.button_padding_x, clamp::SPACING), clamp::range(self.button_padding_y, clamp::SPACING));
        s.spacing.interact_size =
            vec2(s.spacing.interact_size.x, clamp::range(self.control_height, clamp::CONTROL_H));
        s.spacing.window_margin = Margin::same(self.item_spacing_y.max(6.0) as i8);
        s
    }
}

/// The exhaustiveness guard, compiled: every field is named in this
/// arm, so the table can never gain a member that no code path reads.
/// Geometry gets its sanity window here (the same clamps the draw sites
/// re-apply to file-supplied values); colours have no range to check.
/// Both built-in columns wear the check (M6.1 C).
const fn geom_legal(t: &Tokens) -> bool {
    let Tokens {
        ink: _, ink_alt: _, well: _, text: _, text_dim: _, text_faint: _,
        border: _, accent: _, accent_hover: _, field_bg: _, field_hover: _,
        field_active: _, selection_bg: _, selection_stroke: _, viewport: _,
        ribbon_ink: _,
        radius_widget, radius_window, radius_menu,
        item_spacing_x, item_spacing_y, button_padding_x, button_padding_y,
        control_height, row_height, cell_h, cell_w, glyph_px,
        font_label, font_caption, font_row, font_row_suffix,
        ribbon_tab_h, ribbon_cell_top_gap, ribbon_caption_gap,
        ribbon_caption_row_h, ribbon_bottom_gap,
        menu_width, recent_width, menu_gap, menu_pad_x,
    } = *t;
    radius_widget <= clamp::RADIUS
        && radius_window <= 24
        && radius_menu <= 24
        && item_spacing_x >= 0.0
        && item_spacing_y >= 0.0
        && button_padding_x >= 0.0
        && button_padding_y >= 0.0
        && control_height >= clamp::CONTROL_H.0
        && control_height <= clamp::CONTROL_H.1
        && row_height >= clamp::CONTROL_H.0
        && row_height <= clamp::CONTROL_H.1
        && cell_h >= clamp::CELL.0
        && cell_h <= clamp::CELL.1
        && cell_w >= clamp::CELL.0
        && cell_w <= clamp::CELL.1
        && glyph_px >= clamp::GLYPH.0
        && glyph_px <= clamp::GLYPH.1
        && font_label >= clamp::FONT.0
        && font_label <= clamp::FONT.1
        && font_caption >= clamp::FONT.0
        && font_caption <= clamp::FONT.1
        && font_row >= clamp::FONT.0
        && font_row <= clamp::FONT.1
        && font_row_suffix >= clamp::FONT.0
        && font_row_suffix <= clamp::FONT.1
        && ribbon_tab_h >= clamp::RIBBON_ROW.0
        && ribbon_tab_h <= clamp::RIBBON_ROW.1
        && ribbon_caption_row_h >= clamp::RIBBON_ROW.0
        && ribbon_caption_row_h <= clamp::RIBBON_ROW.1
        && ribbon_cell_top_gap >= clamp::RIBBON_GAP.0
        && ribbon_cell_top_gap <= clamp::RIBBON_GAP.1
        && ribbon_caption_gap >= clamp::RIBBON_GAP.0
        && ribbon_caption_gap <= clamp::RIBBON_GAP.1
        && ribbon_bottom_gap >= clamp::RIBBON_GAP.0
        && ribbon_bottom_gap <= clamp::RIBBON_GAP.1
        && menu_width >= clamp::MENU_W.0 && menu_width <= clamp::MENU_W.1
        && recent_width >= clamp::RECENT_W.0 && recent_width <= clamp::RECENT_W.1
        && menu_gap >= clamp::MENU_GAP.0 && menu_gap <= clamp::MENU_GAP.1
        && menu_pad_x >= clamp::MENU_PAD_X.0 && menu_pad_x <= clamp::MENU_PAD_X.1
}

const _: () = assert!(geom_legal(&Tokens::DARK));
const _: () = assert!(geom_legal(&Tokens::LIGHT));

/// The live theme in force, one cell per process. Every consumer reads
/// it through `current()`; the ONLY writer is `set_current`, and the
/// only caller of that is the shell's per-frame theme sync (M6.1 C) -
/// no pane, command, or system pokes a table in. The default is DARK,
/// so a process that never syncs (the kittest frames, the headless
/// proof) renders exactly the pre-C shell.
static LIVE: std::sync::OnceLock<std::sync::RwLock<Tokens>> = std::sync::OnceLock::new();

fn live() -> &'static std::sync::RwLock<Tokens> {
    LIVE.get_or_init(|| std::sync::RwLock::new(Tokens::DARK))
}

/// The theme in force. A poisoned cell still answers (Tokens is Copy,
/// so a panicked writer left either the old or the new table - both
/// legal); the read never crashes the frame.
#[must_use]
pub fn current() -> Tokens {
    *live().read().unwrap_or_else(|p| p.into_inner())
}

/// Install a token table as the live theme WITHOUT touching any
/// context - the half of the swap the sync chooses not to take (a
/// refused file theme keeps the previous table live; only the status
/// line says so). `apply` is the paired install; nothing else writes.
pub fn set_current(t: &Tokens) {
    *live().write().unwrap_or_else(|p| p.into_inner()) = *t;
}

/// Install a token table into a context: visuals and spacing together,
/// so no theme can half-apply (Prime Directive, chrome edition). BOTH of
/// egui's theme slots get the table (#506): egui follows the OS
/// appearance, and writing only the slot active at startup (dark, before
/// the first input names the system theme) left a light-mode Mac on
/// egui's stock light visuals while the painted tree drew this table's
/// ink on it. The document's theme is the only theme; the OS never picks.
pub fn apply_to(ctx: &egui::Context, t: &Tokens) {
    let visuals = t.visuals();
    for slot in [egui::Theme::Dark, egui::Theme::Light] {
        let mut style = t.style(&ctx.style_of(slot));
        style.visuals = visuals.clone();
        ctx.set_style_of(slot, style);
    }
}

/// Install the live theme. Called per Context (windowed); the switcher
/// re-calls it on change (M6.1 C).
pub fn apply(ctx: &egui::Context) {
    apply_to(ctx, &current());
}
