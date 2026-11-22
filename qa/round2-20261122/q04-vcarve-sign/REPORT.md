# QA Report — Q04 V-Carve Sign (`q04-vcarve-sign`)

**Programs tested:** SolidCraft v0.46.2 (`solidcraft` MCP, headless), CamCraft v0.25.0 (`camcraft` MCP, headless)  
**Workspace:** `/home/mikekinney/qa/runs/20261122-q04-vcarve-sign`  
**Part spec:** 160 × 70 × 15 mm sign plate (R8 outer corners), centred 140 × 50 × 4 mm deep recessed pocket (R2 inner corners, 10 mm raised border), two 30 mm tall capital letters `"CC"` on the pocket floor for V-carving, two Ø6 mm through mounting holes 10 mm in from each short end on the centre line (`X = ±70, Y = 0`).

> **Machinist note on part geometry:** With plate length `160 mm` (`X ∈ [-80, 80]`) and a centred `140 mm` pocket (`X ∈ [-70, 70]`), holes placed `10 mm` in from each short end (`X = ±70, Y = 0`) lie directly on the pocket wall step (half in the 15 mm raised border, half in the 11 mm pocket floor). We built the exact specified dimensions and verified analytic volume `138454.531 mm³` (each Ø6 hole removes `π·3²·(15+11)/2 = 367.566 mm³`). This geometry also exposed a real `.sc` tessellation bug in SolidCraft.

---

## Step-by-Step Summary

| Step | What was done | Result | Exact message or key numbers |
|---|---|---|---|
| 1a | SolidCraft: `new_design`, parameters, 160×70×15 plate + R8 outer fillets + 140×50×4 pocket + R2 inner fillets | OK | 1 closed body, bbox `[-80,-35,0]..[80,35,15]`, vol `139189.663 mm³` (expected `139189.663 mm³`) |
| 1b | SolidCraft: cut two Ø6 mm through mounting holes at `X = ±70, Y = 0` (`solid.cylinder` cut) | OK | 1 closed body, vol `138454.531 mm³` (expected `138454.531 mm³`) |
| 1c | SolidCraft: `LettersSketch` on pocket floor (`Z = 11`), `sketch.text` `"CC"` (`height: 30`), `sketch.export_dxf` (`join: true`), `save` `.sc`, `export` `.step` | OK | 2 profiles, 2 DXF `LWPOLYLINE` entities (`q04-vcarve-sign-letters.dxf`), `q04-vcarve-sign.sc` (59,561 B), `q04-vcarve-sign.step` (52,862 B) |
| 1d | SolidCraft: `measure` + `body_topology` (`cylinders` & `all`) + DXF stroke measurement | WRONG | Volume/bbox/cylinders exact (`4×R8, 4×R2, 2×R3`); DXF bounds `[-29.28,-15.43]..[29.83,15.45]`, stroke widths `3.32–4.13 mm`; **however** `body_topology` reports inaccurate & asymmetric areas on stepped hole faces: `F7:side = 239.486 mm²`, `F8:side = 240.456 mm²` (exact `245.044 mm²`) |
| 2a | CamCraft: `import_model` native `q04-vcarve-sign.sc` | REFUSED | `import_model: Solidcraft design refused: machining mesh: face 19 capped at 3.307e0 mm sag, tolerance 5e-3 mm` |
| 2b | CamCraft: `import_model` `q04-vcarve-sign.step` as `sign` | OK | `8876` facets, no `reduced_fidelity`, no warnings |
| 2c | CamCraft: `load_machine` `Tormach 24R` | OK | Mounted `Tormach 24R` (10k–24k RPM, `post_offer: "Tormach PathPilot"`, `default_holder: null`) |
| 2d | CamCraft: `fit_stock` `sides: [2, 2, 2, 2, 0, 1]` | OK | Box `164 × 74 × 16 mm` (`[-82,-37,0]..[82,37,16]`) |
| 2e | CamCraft: `set_wcs` `stock_point: {x:"min", y:"min", z:"max"}` | OK | `G54` origin `[-82.0, -37.0, 16.0]` (stock top, front-left) |
| 2f | CamCraft: `import_dxf` `q04-vcarve-sign-letters.dxf` (AC1015, `join: true`) | REFUSED | `import_dxf: DXF import: DXF CLASSES: unsupported SECTION` |
| 2g | SolidCraft: workaround `sketch.export_dxf` (`join: false`, R12) to `q04-vcarve-sign-letters-r12.dxf` | OK | `894` LINE entities written |
| 2h | CamCraft: `import_dxf` `q04-vcarve-sign-letters-r12.dxf` as `letters` | OK | `894` primitives in mm, `2` closed chains, `0` open/rejected/skipped, origin `[0,0,16]` |
| 3a | CamCraft: `add_tool` `Router up-cut 6 mm` (T1), `Router up-cut 3 mm` (T2), `Router V-bit 60 deg 6 mm` (T3) | WRONG | Tools added, but all catalog wood router tools default to `18000 RPM, 600 mm/min` (`0.0167 mm/tooth`, too slow / burns wood) |
| 3b | CamCraft: `set_work_material` `hard_maple` | REFUSED | `set_work_material: unknown workpiece material 'hard_maple' (one of aluminium_6061, mild_steel, stainless_304, grey_cast_iron, brass, plastics_pom)` |
| 3c | CamCraft: `suggest_cutting_data` on `Face Top` | REFUSED | `suggest_cutting_data: no workpiece material is stated - pick one first` |
| 4a | CamCraft: `add_operation` `Face Top` (T1, `cut_z: top_of_model`, `feed: 2500`, `plunge_feed: 600`, `stepover: 4`) + `detect_pockets` | OK | `Face Top` added; `detect_pockets` found pocket floor `face:0x899db5f7e23b15d3`, `z=11..15`, `min_corner_radius=2.0`, `kind: open_pocket` (notched by Ø6 holes) |
| 4b | CamCraft: `detect_pockets` `create: 0` (`Pocket 6mm`, T1) + `edit_operation` (`feed: 2500, stepdown: 2, stepover: 3`) | OK | `Pocket 6mm` configured (`coolant` defaulted to `flood` despite wood router) |
| 4c | CamCraft: `list_operation_types` (`pocket`, `engrave`, `vcarve`) + probe `rest` on `Pocket 3mm Rest` | WRONG | No `vcarve` op (`no operation kind "vcarve"`); `list_operation_types` omits optional `rest` (`RestDto`) from `pocket` fields |
| 4d | CamCraft: `edit_operation` `Pocket 3mm Rest` (`rest: {previous_tool_diameter: 6.0}`, `feed: 1500, stepdown: 2, stepover: 1.5`) | OK | Accepted `RestDto {previous_tool_diameter: 6.0, previous_corner_radius: 0.0, previous_stock_to_leave: 0.0, allowance: 0.0}` |
| 4e | CamCraft: `add_operation` `V-Carve CC` (`engrave`, T3, `source: "letters"`) + `generate` | REFUSED | `generate: V-Carve CC - could not generate the toolpath: ... a toolpath from the chains of a DXF profile is not generated yet` |
| 4f | CamCraft: `set_operation_geometry` `V-Carve CC` (`picks: [{model:"letters", id:"sketch_layer:0"}]`) | REFUSED | `set_operation_geometry: invalid parameters: 'letters' is a DXF profile - an operation takes the whole profile as its source; picked chains of a profile are not supported yet` |
| 4g | CamCraft: test `trace` on `source: "letters"` + `generate` | REFUSED | Same refusal: `a toolpath from the chains of a DXF profile is not generated yet` |
| 4h | SolidCraft: workaround — `solid.extrude` cut `LettersSketch` `0.5 mm` deep into pocket floor + export `q04-vcarve-sign-with-letters.step` | OK | 1 closed body, 61 faces, vol `138192.723 mm³` (`-261.81 mm³` => `523.6 mm²` letter area) |
| 4i–4k | CamCraft: import `sign_letters`, `V-Carve CC` on letter chains/faces + `generate` + `verify` + `render_stock` | WRONG | `Pocket 3mm Rest` computed `7.5 of 6968.3 mm²` rest material & cleanly cleared all 4 R2 corners (`0` corner rest); **`V-Carve CC` (`engrave`) is NOT V-carving** — traces outlines on-center at constant Z (`gouge.max = 5.499 mm` at `bottom_z = 5.5`) |
| 4l | CamCraft: `add_operation` `Bore Mount Holes` (`bore`, T2, `break_through: 0.5`, `feed: 1500`, `stepdown: 2`) + `detect_holes` | OK | Detected both Ø6 mm through holes at `(-70, 0)` and `(70, 0)`, `entry_z: 15`, `floor_z: 0` |
| 4m | CamCraft: `add_operation` `Contour Cutout` (`contour`, T1, `side: "outside"`, `stepdown: 3`, `tab_count: 4`, `tab_width: 6`, `tab_height: 2`) on outer chain | OK | Picked closed outer chain (`length: 445.904 mm`) |
| 5a–5b | CamCraft: `generate {}` (all 6 ops) | STUCK | Timed out at `3m0s` (`context deadline exceeded`); server finished in ~4m50s (`get_job` unblocked after 1m50s) |
| 5c | CamCraft: `verify {}` + `render_stock` | WRONG | **Critical:** `Contour Cutout` silently stopped at `Z 7.061` (`levels below skipped (6.000, 3.000, 2.000, 0.000)`) with `0` warnings; `V-Carve CC` blocked post (`552` gouges + `rapid_in_stock` + `post_gate` gouge `1.00 mm`) |
| 5d–5e | CamCraft: isolate `Contour Cutout` (`stock_to_leave: 0.02`, `tab_count: 4`) + `generate` | STUCK / WRONG | Timed out at `3m0s` (finished in `5m45s`); false floor shifted `+0.020 mm` to `Z 7.081` at `(72.067, -38.019)` |
| 5f | CamCraft: `V-Carve CC` shallower (`bottom_z: -4.5` => `Z 10.5`) + `generate` + `verify` | WRONG | Still `552` blocking gouges + `2` blocking `rapid_in_stock` |
| 5h | CamCraft: test `insert_text` `"CC"` (`text_cc`) + `engrave` + `generate` | REFUSED | Even built-in `insert_text` fails: `a toolpath from the chains of a DXF profile is not generated yet` |
| 5i–5j | CamCraft: isolate `Contour Cutout` with `tab_count: 0` + `generate` | STUCK / WRONG | Still timed out at `3m0s` (`~4m30s` total) and still falsely stopped at `Z 7.061` |
| 5k | CamCraft: `V-Carve CC` `top_z: top_of_stock`, `bottom_z: -4.2` (0.2 mm deep) + `generate` + `verify` | WRONG | Still `553` blocking gouges + `rapid_in_stock` on final retract move 555 |
| 5l–5m | CamCraft: set `coolant: off` on all 6 ops, reorder `V-Carve CC` last, `Contour Cutout` on top outer chain with `tab_count: 4`, `generate {}` | WRONG | Finished in `2m57s`; `Contour Cutout` STILL falsely stopped at `Z 7.061` skipping levels `(6, 3, 2, 0)` and all 4 tabs |
| 5n | CamCraft: `save_project` `q04-vcarve-sign.camproj` (6 ops) + `post` (`Tormach PathPilot`) | REFUSED | `post: post blocked - gouge: program cuts below the model in 914 cell(s), worst 0.20 mm at (-7.66, 9.59, z 10.80)` |
| 5o | CamCraft: delete `V-Carve CC` (workaround to audit remaining 5 ops), `verify {}`, `post` (`Tormach PathPilot`) | REFUSED | `verify` returned `ok: true` (`0` findings), yet `post` refused: `operation Face Top: asks off coolant, but this post script does not declare coolantModes ... set the operation's coolant to Flood` (×5 ops) |
| 5p | CamCraft: revert `coolant: flood` on 5 ops, `post` `q04-vcarve-sign.nc`, `save_project` `q04-vcarve-sign-posted-5ops.camproj` | OK | Posted 5 ops (without V-Carve) to `q04-vcarve-sign.nc` (`6824` lines) |
| 5q | Machinist G-code audit of `q04-vcarve-sign.nc` | WRONG | `Face Top` (`Z-1`, `F2500`, `S18000`), `Pocket 6mm` (`Z-1/-3/-5`), `Pocket 3mm Rest` (8 corner/notch regions only, `Z-1/-3/-5`, `F1500`), `Bore Mount Holes` (`X12/X152 Y37`, Ø6, `Z-1..-16.5`, `F1500`) verified; **`Contour Cutout` stops at `Z-8.939` (`Z=7.061` world) with 0 tabs**; no `M8`/`M9` emitted despite Flood demand; `Pocket 3mm Rest` bloated to `5973` lines ramping from `Z+1`; `Bore` retracts to `Z25` twice per Z level |

---

## Problems Found

### Problem 1 (SolidCraft) — Cached `.sc` machining mesh fails tessellation tolerance on stepped through-hole cylinder (3.307 mm sag) and reports asymmetric face areas
- **Program:** SolidCraft v0.46.2
- **Repro files:** `q04-vcarve-sign.sc` (refused by CamCraft), `q04-vcarve-sign.step` (imports cleanly)
- **Minimal steps to reproduce:**
  1. `solidcraft:batch`: build 160×70×15 box (`solid.extrude`), cut centred 140×50×4 pocket (`solid.extrude` `operation: "cut"`), cut Ø6 through cylinder straddling the pocket wall at `base: [70, 0, -1], diameter: 6, height: 17, operation: "cut"`, and symmetric cylinder at `[-70, 0, -1]`.
  2. `solidcraft:body_topology {"body": "Body1", "faces": "all"}`
  3. `solidcraft:save {"path": "/home/mikekinney/qa/runs/20261122-q04-vcarve-sign/q04-vcarve-sign.sc"}`
  4. `camcraft:import_model {"name": "sign_sc", "path": "/home/mikekinney/qa/runs/20261122-q04-vcarve-sign/q04-vcarve-sign.sc"}`
- **Expected:** Cached machining mesh meets tolerance (`≤ 5e-3 mm` sag), both symmetric hole cylinder faces have exact area `2π·3·13 = 245.044 mm²`, and CamCraft imports `.sc` cleanly just like `.step`.
- **Actual:**
  - `body_topology` reports `area: 239.4864` on face 19 (`F7:side`) and `area: 240.4565` on face 22 (`F8:side`) — ~2% error and asymmetric despite symmetric geometry.
  - CamCraft refuses the `.sc` file:
    > `import_model: Solidcraft design refused: machining mesh: face 19 capped at 3.307e0 mm sag, tolerance 5e-3 mm`
- **Severity:** High — breaks native SolidCraft → CamCraft `.sc` workflow whenever a cylindrical hole intersects a step; forces STEP export workaround.

---

### Problem 2 (CamCraft) — `import_dxf` rejects standard AutoCAD 2000 (AC1015) DXF `CLASSES` section
- **Program:** CamCraft v0.25.0
- **Repro file:** `q04-vcarve-sign-letters.dxf` (workaround file: `q04-vcarve-sign-letters-r12.dxf`)
- **Minimal steps to reproduce:**
  1. In SolidCraft: `execute {"command": "sketch.export_dxf", "params": {"sketch": "LettersSketch", "join": true, "units": "mm", "path": "/home/mikekinney/qa/runs/20261122-q04-vcarve-sign/q04-vcarve-sign-letters.dxf"}}`
  2. In CamCraft (after `set_stock` / `fit_stock`): `import_dxf {"name": "letters", "path": "/home/mikekinney/qa/runs/20261122-q04-vcarve-sign/q04-vcarve-sign-letters.dxf", "units": "mm"}`
- **Expected:** AC1015 DXF with `LWPOLYLINE` entities imports cleanly (skipping or parsing the standard `CLASSES` section).
- **Actual:** Refused:
  > `import_dxf: DXF import: DXF CLASSES: unsupported SECTION`
- **Severity:** Medium — breaks DXF import from SolidCraft's `join: true` mode and most modern CAD AC1015+ DXFs; requires R12 `LINE`/`ARC` export workaround.

---

### Problem 3 (CamCraft) — No wood material in cutting-data system; catalog wood router tools ship with burning-slow 600 mm/min default feed
- **Program:** CamCraft v0.25.0
- **Minimal steps to reproduce:**
  1. `camcraft:list_tool_catalog {}` -> inspect `Router up-cut 6 mm`, `Router up-cut 3 mm`, `Router V-bit 60 deg 6 mm` (`material_group: "wood"`, `rpm: 18000`, `feed: 600.0`, `flutes: 2`).
  2. `camcraft:set_work_material {"material": "hard_maple"}`
  3. `camcraft:suggest_cutting_data {"operation": "Face Top"}`
- **Expected:** Built-in hardwood/MDF/plywood materials matching the catalog's `"wood"` router tools and built-in wood routers (`Tormach 24R`, `Sienci AltMill`), with realistic wood feeds (~2000–4000 mm/min for 6 mm 2-flute at 18,000 RPM).
- **Actual:**
  - `set_work_material` refuses:
    > `set_work_material: unknown workpiece material 'hard_maple' (one of aluminium_6061, mild_steel, stainless_304, grey_cast_iron, brass, plastics_pom)`
  - `suggest_cutting_data` refuses:
    > `suggest_cutting_data: no workpiece material is stated - pick one first`
  - Every wood router tool in the catalog defaults to `600 mm/min` at `18000 RPM` (chipload `0.0167 mm/tooth`), which rubs and burns hard maple/MDF.
- **Severity:** Medium — renders `suggest_cutting_data` / `apply_cutting_data` unusable for wood; using catalog feeds unedited would char wood and prematurely dull carbide router bits.

---

### Problem 4 (CamCraft) — `list_operation_types` omits optional parameters such as 2D Pocket's `rest` (`RestDto`)
- **Program:** CamCraft v0.25.0
- **Minimal steps to reproduce:**
  1. `camcraft:list_operation_types {"kind": "pocket"}`
  2. `camcraft:edit_operation {"name": "Pocket 3mm Rest", "params": {"rest": true}}`
  3. `camcraft:edit_operation {"name": "Pocket 3mm Rest", "params": {"rest": {}}}`
- **Expected:** `list_operation_types` lists all supported fields (including optional fields like `rest` and its subfields `previous_tool_diameter`, `previous_corner_radius`, `previous_stock_to_leave`, `allowance`).
- **Actual:** `list_operation_types` only prints `"note": "Fields absent from the defaults are optional and off; set them in a params patch to turn them on."` without naming `rest`. Discovering it required triggering serde errors:
  > `edit_operation: bad params patch: invalid type: boolean `true`, expected struct RestDto`  
  > `edit_operation: bad params patch: missing field `previous_tool_diameter``
- **Severity:** Low–Medium — confuses users and automation; hides working features (2D Pocket rest machining worked accurately once discovered).

---

### Problem 5 (CamCraft) — No V-Carve operation; `engrave` with a Taper V-bit traces outlines on-center at constant Z instead of variable-Z medial-axis V-carving
- **Program:** CamCraft v0.25.0
- **Repro files:** `q04-vcarve-sign.camproj`, `q04-vcarve-sign-with-letters.step`
- **Minimal steps to reproduce:**
  1. `camcraft:list_operation_types {"kind": "vcarve"}`
  2. `camcraft:add_operation {"kind": "engrave", "name": "V-Carve CC", "tool": "Router V-bit 60 deg 6 mm", "params": {"top_z": {"reference": "top_of_model", "offset": -4}, "bottom_z": {"reference": "top_of_model", "offset": -9.5}, "stepdown": 2}}`
  3. Pick closed `"CC"` letter chains/faces on `sign_letters`, run `generate {"name": "V-Carve CC"}` and `render_stock {"view": "top", "toolpath": true}`.
- **Expected:** A V-Carve operation driving the 60° V-bit along the letter medial axis with variable Z depth `d = (w/2) / tan(30°)`: ~`2.88 mm` below pocket floor (`Z = 8.12`) on `3.32 mm` arcs and ~`3.58 mm` (`Z = 7.42`) on the `4.13 mm` spine, staying completely inside the closed letter outlines.
- **Actual:**
  - `list_operation_types {"kind": "vcarve"}`:
    > `list_operation_types: no operation kind "vcarve" - call list_operation_types without "kind" for the list`
  - `engrave` traces both inner and outer letter outlines on-center at constant Z stepdowns all the way to `bottom_z` (`Z = 5.5 mm`, `gouge.max = 5.499 mm`, `gouge.volume = 1292 mm³`), carving a 6 mm wide trench outside the letter boundaries.
- **Severity:** High — core V-carving functionality is missing; attempting to use `engrave` with a V-bit produces wrong geometry and massive gouges.

---

### Problem 6 (CamCraft) — DXF profiles and CamCraft's own `insert_text` cannot generate toolpaths in `engrave` or `trace`, and `set_operation_geometry` refuses DXF picks documented in its schema
- **Program:** CamCraft v0.25.0
- **Repro file:** `q04-vcarve-sign-letters-r12.dxf`
- **Minimal steps to reproduce:**
  1. `camcraft:import_dxf {"name": "letters", "path": "/home/mikekinney/qa/runs/20261122-q04-vcarve-sign/q04-vcarve-sign-letters-r12.dxf", "units": "mm"}` (or `camcraft:insert_text {"name": "text_cc", "text": "CC", "height": 30, "position": [60, 22]}`)
  2. `camcraft:add_operation {"kind": "engrave", "name": "V-Carve CC", "source": "letters", "tool": "Router V-bit 60 deg 6 mm"}` (also tested with `kind: "trace"` and `source: "text_cc"`).
  3. `camcraft:generate {"name": "V-Carve CC"}`
  4. `camcraft:set_operation_geometry {"operation": "V-Carve CC", "picks": [{"model": "letters", "id": "sketch_layer:0"}]}`
- **Expected:** Operations whose source is an imported DXF profile or `insert_text` profile generate toolpaths from those chains, and `set_operation_geometry` accepts `sketch_chain` / `sketch_layer` picks as documented in its own tool schema.
- **Actual:**
  - `generate` refuses on `engrave` (DXF), `trace` (DXF), and `engrave` (`insert_text`):
    > `generate: V-Carve CC - could not generate the toolpath: V-Carve CC: invalid machining parameters: this operation is checked against the stock 'Stock' and the chains of the DXF profile 'letters' (2 closed, 0 open) - there is no solid model; a toolpath from the chains of a DXF profile is not generated yet`
  - `set_operation_geometry` refuses:
    > `set_operation_geometry: invalid parameters: 'letters' is a DXF profile - an operation takes the whole profile as its source; picked chains of a profile are not supported yet`
- **Severity:** High — `import_dxf` and `insert_text` are unusable for engraving/tracing; forces cutting text into a 3D solid model as a workaround.

---

### Problem 7 (CamCraft) — `engrave` on a solid model is blocked at `verify` and `post` by `gouge` and `rapid_in_stock` even at 0.2 mm depth
- **Program:** CamCraft v0.25.0
- **Repro files:** `q04-vcarve-sign.camproj`, `q04-vcarve-sign-with-letters.step`
- **Minimal steps to reproduce:**
  1. Open `q04-vcarve-sign.camproj` (where `V-Carve CC` has `top_z: top_of_stock`, `bottom_z: top_of_model - 4.2` [0.2 mm into pocket floor, inside `sign_letters` 0.5 mm cutout], `stepdown: 6.0`).
  2. `camcraft:verify {}`
  3. `camcraft:post {"post": "Tormach PathPilot", "output": "/home/mikekinney/qa/runs/20261122-q04-vcarve-sign/q04-vcarve-sign.nc"}`
- **Expected:** Shallow engraving (`0.2 mm` <= `3.0 mm` mark budget) passes verification (or does not trigger `rapid_in_stock` when retracting vertically out of an already-pocketed floor) and posts cleanly.
- **Actual:**
  - `verify` flags 553 blocking gouges (`move 2 cuts past the finish surface` .. `move 554`) and blocks final vertical retract:
    > `rapid move 555 travels through stock (below the clearance plane, or at it below the stock top)`
  - `post` refuses:
    > `post: post blocked - gouge: program cuts below the model in 914 cell(s), worst 0.20 mm at (-7.66, 9.59, z 10.80)`
- **Severity:** High — makes `engrave` unpostable end-to-end on solid models; user must delete the engraving operation to post the job.

---

### Problem 8 (CamCraft) — CRITICAL: `2D Contour` falsely detects a solid floor at Z = 7.061 mm outside the part and silently skips bottom levels and all 4 holding tabs
- **Program:** CamCraft v0.25.0
- **Repro files:** `q04-vcarve-sign.camproj`, `q04-vcarve-sign-posted-5ops.camproj`, `q04-vcarve-sign.nc` (lines 6751–6824)
- **Minimal steps to reproduce:**
  1. Import `q04-vcarve-sign.step`, fit stock `[2,2,2,2,0,1]`, add `Router up-cut 6 mm`.
  2. `camcraft:add_operation {"kind": "contour", "name": "Contour Cutout", "source": "sign", "tool": "Router up-cut 6 mm", "params": {"side": "outside", "top_z": {"reference": "top_of_model", "offset": 0}, "bottom_z": {"reference": "bottom_of_model", "offset": 0}, "stepdown": 3, "tab_count": 4, "tab_height": 2, "tab_width": 6}}`
  3. `camcraft:set_operation_geometry {"operation": "Contour Cutout", "picks": [{"model": "sign", "id": "chain:0x7205c39f7fa5aec6"}]}` (also reproduces on bottom chain `chain:0x68271d2095f17a12`, with `tab_count: 0`, and with `stock_to_leave: 0.02`).
  4. `camcraft:generate {"name": "Contour Cutout"}` -> `camcraft:verify {}` -> `camcraft:post`
- **Expected:** Outside contour cuts down to `bottom_of_model` (`Z = 0` world / `G54 Z-16`) at stepdowns `Z-1, Z-4, Z-7, Z-10, Z-13, Z-16`, stepping up to `Z-14` across 4 holding tabs (6 mm wide × 2 mm tall).
- **Actual:**
  - `generate` / `verify` logs only a non-blocking note and returns `status: "ok"`, `warnings: []`, `gate.ok: true`:
    > `depth: the ring at (72.067, -37.999) ends at its floor Z 7.061 - the part is solid below it; levels below skipped (6.000, 3.000, 2.000, 0.000)`
  - Increasing `stock_to_leave` by `+0.02 mm` shifted the bogus floor by exactly `+0.020 mm` (`Z 7.081` at `(72.067, -38.019)`).
  - Posted G-code (`q04-vcarve-sign.nc` lines 6764–6821) stops at `Z-8.939` (`Z = 7.061` world), leaving `7.061 mm` of solid wood uncut around the entire sign and emitting **0 tabs**.
- **Severity:** **Critical** — posts silently wrong/incomplete G-code with `verify ok: true` and 0 warnings; fails to cut out the part or create holding tabs.

---

### Problem 9 (CamCraft) — `2D Contour` generation hangs for 3–5.75 minutes on a simple 160×70 R8 contour, timing out MCP calls
- **Program:** CamCraft v0.25.0
- **Repro file:** `q04-vcarve-sign.camproj`
- **Minimal steps to reproduce:**
  1. Run `camcraft:generate {"name": "Contour Cutout"}` (with either `tab_count: 4` or `tab_count: 0`).
- **Expected:** Simple 2D contour on a 132-edge rounded rectangle generates in < 5 seconds (like `Pocket 6mm` and `Pocket 3mm Rest`).
- **Actual:** Repeatedly timed out the 3-minute MCP deadline:
  > `MCP tool call to server "camcraft" timed out after 3m0s: context deadline exceeded`
  - Observed wall times across 4 runs: `~4m50s`, `5m45s`, `~4m30s`, and `2m57s`. Subsequent `get_job` calls blocked until the server finished.
- **Severity:** High — causes MCP timeouts / apparent application freezes on basic 2D contours.

---

### Problem 10 (CamCraft) — `Tormach PathPilot` post blocks `coolant: off` and demands `coolant: flood` on a wood router, after `verify` reported `ok: true`, and emits no coolant codes anyway
- **Program:** CamCraft v0.25.0
- **Repro file:** `q04-vcarve-sign-posted-5ops.camproj`
- **Minimal steps to reproduce:**
  1. Load `Tormach 24R` wood router, set `set_operation_coolant {"name": "<op>", "coolant": "off"}` on all 5 operations.
  2. `camcraft:verify {}` -> returns `ok: true, findings: [], gate: {ok: true}`.
  3. `camcraft:post {"post": "Tormach PathPilot", "output": "/home/mikekinney/qa/runs/20261122-q04-vcarve-sign/q04-vcarve-sign.nc"}`
- **Expected:** `coolant: off` posts cleanly on a wood router (and any post-script constraint should be caught by `verify`).
- **Actual:** `post` blocks with:
  > `post: post blocked - 5 warning(s): operation Face Top: asks off coolant, but this post script does not declare coolantModes (it does not read PARAM Coolant) - set the operation's coolant to Flood or use a post that reads it; ...`
  - After setting coolant back to `flood`, the posted `q04-vcarve-sign.nc` contains zero `M8`/`M9` coolant commands.
- **Severity:** Medium — forces wood-router users to set `Flood` coolant just to get G-code out; contradicts `verify ok: true`.

---

### Problem 11 (CamCraft) — Severe G-code bloat and wasted retracts in `Pocket 3mm Rest` and `Bore Mount Holes` (plus air passes at `Z-1`)
- **Program:** CamCraft v0.25.0
- **Repro file:** `q04-vcarve-sign.nc` (lines 570–6543 for `Pocket 3mm Rest`; lines 6544–6750 for `Bore Mount Holes`; lines 63–120 & 6764–6776 for `Z-1` air passes)
- **Minimal steps to reproduce:**
  1. Post `q04-vcarve-sign-posted-5ops.camproj` to `q04-vcarve-sign.nc` and inspect operations `Pocket 3mm Rest` and `Bore Mount Holes`.
- **Expected:** `Pocket 3mm Rest` ramps from each level's start Z (not `Z+1` above stock on every deeper level) in concise passes; `Bore` cuts helically or feeds directly from plunge into the circular cut without retracting to `Z25` twice per depth level; operations starting at `top_of_model` (`Z-1`, already faced) don't run full air passes at `Z-1`.
- **Actual:**
  - `Pocket 3mm Rest` (only `7.5 mm²` of corner/notch rest material) produces **5,973 lines of G-code**, starting secondary corner ramps from `Z+1` (2 mm above faced top) down to `Z-3` and `Z-5` in ~0.06 mm Z oscillations (~470 lines per corner per depth).
  - `Bore Mount Holes` plunges at center `(12, 37)` to `Z`, feeds radially to `(13.5, 37)`, retracts all the way to `G0 Z25`, rapids back down and re-plunges to `Z` at `(13.5, 37)`, cuts one flat `G3` circle, and retracts to `Z25` again — 36 unnecessary full-clearance retracts for 2 holes.
  - `Pocket 6mm` and `Contour Cutout` each run a full cutting pass at `Z-1` (already faced by `Face Top`).
- **Severity:** Low–Medium — safe (no gouge), but wastes substantial machine cycle time and generates excessive plunge/retract cycles in wood.

---

## Confusing Tool Names, Parameters, and UX Notes

1. **`insert_text` / `move_dxf_profile` `position` reference:** Documented as `[x, y] ... from the stock origin`, which for a centered box stock is world `[0, 0]` (stock *center*), NOT the WCS G54 origin `[-82, -37]` (front-left). Passing G54 coordinates `[60, 22]` places text near the right edge of the stock instead of inside the pocket.
2. **`set_operation_geometry` schema vs. implementation:** Schema explicitly lists `sketch_chain:<hex>`, `sketch_layer:<layer>`, and per-chain `side`/`reverse` overrides for DXF profiles, yet calling it with a DXF profile pick returns `picked chains of a profile are not supported yet`.
3. **`list_operation_types` hidden optional fields:** Prints `"Fields absent from the defaults are optional and off; set them in a params patch to turn them on."` without documenting what optional fields exist (`rest`, etc.).
4. **`Face` default `cut_z`:** Defaults to `{offset: 0.0, reference: "top_of_stock"}`, which cuts 0 mm air unless the user remembers to switch the reference to `top_of_model`.
5. **`load_machine` vs. active setup post:** Mounting `Tormach 24R` reports `post_offer: "Tormach PathPilot"`, yet `get_job` shows `posts_with: "Fanuc (built-in)"` unless you explicitly pass `post` on every `post` call or create a declared setup (`add_setup` + `set_setup_post`).
6. **Default `coolant: "flood"` on wood routers:** Every newly added operation defaults to flood coolant even when a dry CNC wood router (`Tormach 24R`) is mounted.
7. **`Bore` vs. helical boring:** `Bore` has no helical-ramp mode; it executes flat plunge + radial move + full retract + re-plunge + flat circle at each `stepdown`.

---

## Reproduction Files in `/home/mikekinney/qa/runs/20261122-q04-vcarve-sign`

- `q04-vcarve-sign.sc` — SolidCraft native design (reproduces Problem 1: refused by CamCraft `import_model` with 3.307 mm sag on face 19)
- `q04-vcarve-sign.step` — SolidCraft STEP export (imports cleanly into CamCraft)
- `q04-vcarve-sign-with-letters.step` — SolidCraft STEP export with `"CC"` extruded 0.5 mm into pocket floor (workaround for Problem 6)
- `q04-vcarve-sign-letters.dxf` — SolidCraft AC1015 DXF (`join: true`; reproduces Problem 2)
- `q04-vcarve-sign-letters-r12.dxf` — SolidCraft R12 DXF (`join: false`; imports cleanly, reproduces Problem 6)
- `q04-vcarve-sign.camproj` — Full 6-op CamCraft project including `V-Carve CC` (reproduces Problems 5, 7, 8, 9)
- `q04-vcarve-sign-posted-5ops.camproj` — 5-op CamCraft project without `V-Carve CC` (reproduces Problems 8, 10, 11)
- `q04-vcarve-sign.nc` — Posted 5-op Tormach PathPilot G-code (`6824` lines; evidences Problems 8, 10, 11 and verifies `Pocket 3mm Rest` corner-only moves and `Bore` hole coordinates)
- `STEPS.log` — Chronological step log of all tool calls and outcomes

RESULT: 11 problems (10 CamCraft, 1 SolidCraft); end-to-end: not posted
