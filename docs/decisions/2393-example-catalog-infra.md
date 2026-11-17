## #2393 - Example catalog infrastructure (`examples/catalog/<family>/<case>/`, `.camscript`, `xtask catalog`)

- **What was decided:**
  1. Organize catalog cases under `examples/catalog/<family>/<case>/` with `model/` (generator + mesh), `build.camscript`, `<case>.camproj`, `<case>.nc`, and `README.md`. Seed with `examples/catalog/2d/first-part/` converted from `examples/first-part`, while keeping the legacy `examples/first-part.*` and `examples/models/first-part.obj` paths untouched so release packaging, installer checks, and existing UI/guide tests keep working.
  2. Bridge `cam.call(tool, args?)` in `crates/app/src/script/` to `crate::mcp::call`, and accept `.camscript` alongside `.js` in `camcraft-cli script`. Every catalog `build.camscript` runs in the Boa JS sandbox over the same `Env` and `History` as MCP and the GUI.
  3. Add `cargo xtask catalog` (`xtask/src/catalog.rs`, `catalog_media.rs`, `catalog_pdf.rs`) to rebuild or `--check` every case via `camcraft-cli`, render off-screen PNGs into an images-only branch (`catalog-screenshots` by default, refusing `main`/`master`/checked-out/code branches and paths inside the repo tree) via `crates/ui/tests/catalog_render.rs` (with an optional `--with-guide` hook into #1902's `guide_images` renderer when present), and optionally build per-case PDFs via `pandoc` + headless Chrome into `target/catalog-pdf/`.
  4. Pin the catalog in `crates/app/tests/example_catalog.rs` (open `<case>.camproj`, force-regenerate all ops, verify clean, post and compare byte-for-byte against `<case>.nc`, and rebuild `build.camscript` end-to-end in a scratch copy) and wire `examples/catalog/` diffs to `app:example_catalog` in `xtask/src/quick.rs`.

- **Why:**
  - Bridging `cam.call` directly to `mcp::call` gives `.camscript` immediate access to all 60+ undoable commands (`set_stock`, `add_tool`, `add_operation`, `edit_operation`, `set_wcs`, `load_machine`, etc.) over one shared `Env` and `History` instead of duplicating command wrappers in `script/api.rs`.
  - Committing rendered PNGs via an isolated temporary index (`GIT_INDEX_FILE` + `hash-object` / `write-tree` / `commit-tree`) updates the images-only branch without ever touching the working tree or risking screenshots landing in the product branch.
  - Keeping `examples/catalog/2d/first-part/first-part.nc` as the output posted by current `main` (including `G21`, decimal-pointed Fanuc words, per-pocket helical entries, and `G99` cycles) avoids mixing legacy v59 `.nc` text with modern generator output.

- **Considered and dropped:**
  - Defining a custom line-based verb parser separate from Boa JS `cam`: creates a second scripting dialect to maintain and diverges from `camcraft-cli script` / `assets/scripts/cam.d.ts`.
  - Moving or deleting `examples/first-part.camproj`: would break `.github/workflows/release.yml`, `scripts/installer.ps1`, `xtask/src/release_tests.rs`, `crates/ui/tests/gpu_render.rs`, and `site/content/guide/automate/cli.md`.
  - Putting `camcraft-app` as a dependency of `xtask`: violates workspace layering and slows `xtask` builds; invoking the built `camcraft-cli` binary keeps `xtask` dependency-light and tests the real shipped CLI.

- **Cost to change later:** Low — `cam.call` is a single table entry in `VERBS`, `examples/catalog/` discovery is filesystem-driven, and `xtask catalog` subcommands/flags are self-contained in `xtask/src/catalog*.rs`.
