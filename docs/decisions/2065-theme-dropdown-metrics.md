## #2065 - Theme-driven dropdown menu dimensions

**What was decided:** Added four geometry tokens (`menu_width`, `recent_width`, `menu_gap`, `menu_pad_x`) to `Tokens`, corresponding `clamp` bounds (`MENU_W`, `RECENT_W`, `MENU_GAP`, `MENU_PAD_X`), `theme_files` vocabulary and overlay support, and `DropGeom` (`Tokens::dropdown()`) so all dropdown drawers (`panes.rs`, `layout/mod.rs`, `recent_drop.rs`, `theme_drop.rs`, `tree_menu.rs`, and `fixture_menu.rs`) consume clamped theme geometry and clamp widths/left positions inside narrow viewports.

**Why:** Dropdown widths (`DROP_W`, `RECENT_W`), trigger gap (`DROP_GAP`), and row horizontal padding (`8.0` / `28.0` / `4.0`) were previously hardcoded constants that ignored theme overrides and could overflow narrow strips. Deriving dependent icon/checkbox/clip offsets in `DropGeom` keeps drawer alignment consistent while preserving default dark/light metrics (`200.0`, `300.0`, `2.0`, `8.0`) and `machine_menu_width` row-text expansion.

**What was considered and dropped:**
- Exposing separate theme JSON keys for secondary offsets (`icon_text_x`, `icon_center_x`, `box_center_x`, `clip_inset_x`): dropped to keep the public theme vocabulary small; derived inside `DropGeom` from `menu_pad_x`.
- Updating `assets/themes/theme.json.example`: not applicable on this base (`base-2027` / `upstream/main` has no such file).

**Cost to change later:** Low. Promoting any derived offset to an independent token or tuning `clamp` bounds requires touching only `theme.rs`, `theme/dropdown.rs`, and `theme_files`.
