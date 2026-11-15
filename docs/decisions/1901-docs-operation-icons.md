## #1901 - Show app UI and operation icons in the user guide

**What was decided.**
1. Added a Zola shortcode `site/templates/shortcodes/icon.html` (`{{ icon(name="...") }}`) that resolves `icons/<name>.png` through `get_url` so sub-path deployments under `/Camcraft/` resolve cleanly, sets explicit dimensions and `alt`/`aria-hidden`, and applies the `ui-icon` class.
2. Copied the 92 UI icon PNGs listed in `Icon::ALL` (`crates/ui/src/egui_app/icons.rs`) and `assets/ui/LICENSES.md` (`LICENSES.txt`) into `site/static/icons/`, and linked `icons/LICENSES.txt` from the `site/templates/base.html` footer.
3. Styled `img.ui-icon` in `site/static/style.css` and applied `filter: invert(0.92)` inside `@media (prefers-color-scheme: dark)` because the app PNGs are black ink on alpha transparency.
4. Started all 44 operation sections across all seven `operations-*.md` pages with `{{ icon(name="op-...") }}` matching `op_icon()` in `crates/ui/src/egui_app/op_icons.rs`, and added inline toolbar, browser tree, status badge, and settings-tab icons across the user guide (`first-run.md`, `workflows.md`, `operations/_index.md`, `setups.md`, `tools.md`, `workholding.md`, `machines.md`, `posting.md`).
5. Added parity and link tests in `crates/ui/src/egui_app/tests/op_icon_table.rs` (`docs_site_icons_match_app_table_and_operations`) and `site/tests/test_guide_icons.py` checking byte-for-byte PNG parity against `Icon::ALL`, no stale PNGs, notice parity, dark-mode CSS styling, every `OpType` section icon, and no unknown or stale `{{ icon(name=...) }}` names.

**Why.**
- `.github/workflows/release.yml` runs `(cd site && /tmp/zola build ...)` with no asset copy step, and symlinks inside `site/static/` break on Windows checkouts with `core.symlinks=false`. Committing `site/static/icons/*.png` alongside byte-for-byte parity tests keeps Windows, CI, and the app UI in sync and fails `cargo test` loudly if an app icon is renamed or changed.
- Leaving `assets/ui/cube.png` out of `site/static/icons/` avoids copying an unused packaging image that is not part of `Icon::ALL`.

**What was considered and dropped.**
- Symlinking `site/static/icons` to `../../assets/ui`: fails on Windows (`core.symlinks=false`).
- Modifying `.github/workflows/release.yml` to copy icons at build time: out of scope for docs changes and breaks standalone local `zola build` / `site/tests/`.
- Inlining SVG sources directly instead of PNGs: loses parity with the rasterized UI PNGs tested by `op_icon_table` and duplicates SVG path rewriting for dark mode.

**What it would cost to change later.**
Low: removing `site/static/icons/`, `site/templates/shortcodes/icon.html`, and the shortcode calls reverts the site if a future build step copies `assets/ui/` directly. Updating an icon only requires syncing `site/static/icons/<name>.png` (which `docs_site_icons_match_app_table_and_operations` enforces).
