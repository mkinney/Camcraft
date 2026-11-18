## #2403 - Shared user-friendly names for picked geometry

**What was decided:**
- Add a deterministic, purely functional `FeatureNames` lookup table (`crates/geometry/src/geometry/feature/names.rs` + `patch_shape.rs`) keyed by `(NameKind, u64)` (`Face`, `Patch`, `Chain`, `Vertex`) and computed over a model's or stock's `Features` and `&[Triangle]`:
  - Planar/free faces are numbered `"Face 1"`, `"Face 2"`, ... in deterministic spatial centroid order (`(centroid.z, centroid.y, centroid.x, id)`).
  - Welded smooth patches are classified via normal-covariance axis fit and Kasa circle fit into concave bores (`"Bore 1"`, ...), convex cylinders (`"Cylinder 1"`, ...), or general curved surfaces (`"Curved face 1"`, ...).
  - Edge chains, splits/runs, and singleton edges all map to `"Edge 1"`, `"Edge 2"`, ... sharing one continuous ordinal sequence (chains first by `(mid.z, mid.y, mid.x, len, id)`, then edge runs within each chain, then singleton edges not already assigned).
  - Feature vertices map to `"Vertex 1"`, `"Vertex 2"`, ... sorted by `(z, y, x, id)`.
- Cache `Arc<FeatureNames>` on each model and stock entity in a derived ECS component `FeatureNamesMemo { rev, tris: Weak<[Triangle]>, table }` (`crates/app/src/schema.rs`, `crates/app/src/pick/names.rs`), rebuilt only on `Revision` or `Weak<[Triangle]>` mismatch, with `name_builds()` counter for frame-cost verification.
- Keep `GeoSel` internal 64-bit hashes (`Face(id)`, `Chain(id)`, `Patch(id)`, `Vertex(id)`) as the stable reference in commands, undo history, `.camproj` persistence, and MCP `"id"` inputs/outputs. Route every UI readout, status line, operation geometry chip, refusal message, and MCP view through shared helpers `camcraft_app::pick::{geo_label, feature_name, feature_name_or_kind, label_in_table}`.
- Add a user-friendly `"name"` field beside `"id"` in MCP feature lists (`list_features`, `detect_pockets`) and pick responses (`set_operation_geometry`, `get_wcs_align`, `locate_part`, `get_selection`, and box/area/boundary pick results).
- Remove `{id:#x}` from `FeatureSel::Display` so even uncached/missing picks format as plain kinds (`"face"`, `"edge"`, `"curved face"`, `"vertex"`).

**Why:**
- Internal feature IDs are 64-bit structural hashes (displayed as `"welded face 0x885acc..."`), which are unreadable to users and differed between pages (e.g. `short_id` hex chips vs WCS status lines vs operation refusals).
- Centralizing the ordinals in `camcraft-geometry` with an `app`-side `Revision`+`Weak<[Triangle]>` memo ensures every UI page, refusal, and MCP tool shows the exact same friendly name per part or stock with zero per-frame mesh scanning.
- Keeping internal IDs in storage and wire inputs avoids breaking existing `.camproj` projects, undo commands, and geometric hash stability when nearby features are edited.

**What was considered and dropped:**
- Persisting names in `.camproj` or replacing `FeatureSel` hash variants with ordinal strings: rejected because model edits or re-imports would invalidate stored references cleanly handled by structural hashes, and `.camproj` schema compatibility would break.
- Generating names on the fly per UI widget: rejected because names would diverge across pages (`WCS` vs `Operation` chips vs `Locate` vs refusals) and violate the frame-cost budget (#2238).
- Putting `PatchShape` / naming helpers in `crates/app`: rejected because cylindrical/curved patch geometry and deterministic feature enumeration depend only on `geometry::feature` types (`Features`, `Triangle`).

**What it would cost to change later:**
- The naming scheme (`"Face N"`, `"Bore N"`, `"Cylinder N"`, etc.) and sort order live in one place (`crates/geometry/src/geometry/feature/names.rs` and `patch_shape.rs`); no serialized project state depends on the ordinals, so adjusting labels or classification thresholds only updates UI/MCP string expectations.
