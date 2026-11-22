# QA Test Report: q05-4axis (SolidCraft v0.46.2 & CamCraft v0.25.0)

**Date:** 2026-11-22  
**Workspace:** `/home/mikekinney/qa/runs/20261122-q05-4axis`  
**Part:** 100 mm (X) x 40 mm (Y) x 40 mm (Z) aluminium 6061 indexed 4th-axis block on X/A axis (`X: 0..100`, `Y: -20..20`, `Z: -20..20`):
- Top (`+Z`): 30 x 16 mm pocket, 5 mm deep, corners R3, centred at `(50, 0, 20)`
- Front (`-Y`): two Ø6 mm holes, 12 mm deep, 40 mm apart at `X = 30, 70`, `Z = 0`
- Bottom (`-Z`): 10 mm wide x 3 mm deep full-length slot (`X = 0..100`, `Y = -5..5`)
- Back (`+Y`): Ø20 mm x 2 mm deep counterbore at `(50, 20, 0)`
- Right end (`X = 100`): 45° x 2 mm chamfer on all four outer edges

---

## Step Summary Table

| step | what you did | result | exact message or numbers |
|---|---|---|---|
| 1. SolidCraft CAD | Built `Block` (`solid.box`, `solid.chamfer`, `solid.box` cut + `solid.fillet` R3, 2x `solid.hole`, `solid.box` cut bottom slot, `solid.hole` back counterbore); verified via `measure`, `body_topology`, `screenshot`; saved `q05-4axis.sc` and exported `q05-4axis.step`. | OK | `volume_mm3 = 153042.3924533618` (analytical: `153042.3924534`, 0.0000000% diff); `bbox = [0,-20,-20]..[100,20,20]`; 30 faces (23 planes, 7 cylinders); `closed = true`; `q05-4axis.sc` (82,529 B), `q05-4axis.step` (62,581 B), `cad-iso.png`. |
| 2. CamCraft setup | Imported `q05-4axis.step`; loaded `Tormach 770MX + microARC 4`; set material `aluminium_6061`; created `Setup1` with `Tormach PathPilot`; set 44x44x110 mm square bar stock along X (`[-10,-22,-22]..[100,22,22]`, since Ø44 mm round bar < 56.57 mm diagonal of 40x40 block); placed WCS G54 at `[0,0,0]`. | OK | `facets = 2126`, model bounds `[0,-20,-20]..[100,20,20]`; A axis `[1,0,0]`, `nose_to_axis = 269.24 mm`, `axis_to_table = 80.01 mm`; `stock_bounds = [-10,-22,-22]..[100,22,22]`; `WCS G54 = [0,0,0]`. |
| 3. CamCraft tools | Added 5 requested tools (`EM-0600-4FL`, `EM-1000-4FL`, custom 6 mm 90° spot drill, `DR-0600`, `CM-1000-90`) plus workaround `BN-0600` (6 mm ball nose) when rotary ops refused flat end mills. | OK | T1 `6mm Flat EM`, T2 `10mm Flat EM`, T3 `6mm Spot Drill 90deg` (custom `drill`, `tip_angle: 90` — no spot drill in catalog), T4 `6mm Drill`, T5 `10mm 90deg Chamfer Mill`, T6 `6mm Ball Nose`; all auto-assigned `BT30 ER20`. |
| 4a. 3+2 tool orientation & setup indexing | Tried per-operation orientation fields (`edit_operation`), rotated WCS (`set_wcs`), and `add_setup(datum_from: "Setup1")` + `lay_part_on_face` (`X -90°`) for `Front_A90` / `Setup2_A90`. Added `TopPocket` (`Setup1`) and `FrontDrill` (`Setup2_A90`), generated, verified, and posted probe NC files. | WRONG | `edit_operation` has no orientation field; tilted WCS refused by `generate`; `lay_part_on_face` setup (`X -90°`) posted **`G0 A0`** for BOTH `Setup1` and `Setup2_A90` (lines 19 & 139 of `test-probe3.nc`) plus `M0` setup change; `add_setup("Front_A90")` sorted alphabetically before `Setup1`; `verify` skipped stock sim (`exact side-on carry exhausted its ray budget`) yet returned `ok: true`; post emitted `G0 Z47` before `G0 X.. Y..` after tool change. |
| 4b. `detect_holes` & `detect_pockets` on rotated setup | Called `detect_holes` on `FrontDrill` (`Setup1`) and `FrontDrillAligned` (`Front_A90`, turned `X -90°`); called `detect_pockets` with `Front_A90` active. | WRONG | `detect_holes` populated `cross` with upside-down `-Z` Ø20 counterbore alongside 2 Z holes (`undersize: []`); `generate` refused (`this op lists Z holes AND cross holes... (M5.2 v1)`); `edit_operation` refused `cross: []` (`unknown field cross`), permanently bricking the op. `detect_pockets` ignored `part_alignment` and stayed in `"frame": "world"`. |
| 4c. Wrapped / rotary operations | Tried `rotary_parallel` with `6mm Flat EM`, then `rotary_parallel`, `rotary_contour`, `rotary_offset` with `6mm Ball Nose` (`BN-0600`) on `q05-4axis`. | REFUSED | Flat end mill refused (`No ball-nose in Setup2_A90's tools - a Rotary Parallel operation cuts with one`). With `6mm Ball Nose`, all 3 rotary ops refused: `the model's normals share no rotation axis - a turned workpiece is rotary work; finish a freeform with the 5-Axis kinds`. |
| 4d. 5-Axis workaround (`parallel5d`, `multi_axis_rough`) | Followed CamCraft's error suggestion; added `TestParallel5D` (`parallel5d`) and `TestMultiAxisRough` (`multi_axis_rough`) with `6mm Ball Nose` and called `generate`. | STUCK | Both `generate` calls timed out after 3m0s (`MCP tool call to server "camcraft" timed out after 3m0s: context deadline exceeded`), and 3 subsequent `get_job` calls timed out after 3m0s each (>15.5 min total). CamCraft MCP server permanently wedged; unable to complete bottom/back/chamfer ops or save/post final `q05-4axis.camproj` / `q05-4axis.nc`. |
| 5. Final generate / verify / post | Could not reach full-part post because CamCraft MCP server hung in step 4d. Audited probe G-code from `test-probe.nc`, `test-probe2.nc`, `test-probe3.nc`, `test-explain.nc` (2 ops: `TopPocket` + `FrontDrill`). | STUCK | Partial probe G-code posted (156 lines in `test-probe3.nc`), but full program (`q05-4axis.nc`, `q05-4axis.camproj`) not posted due to server hang. |

---

## Problems Found

### Problem 1 — CamCraft: No 3+2 indexed A-axis rotation in G-code (posts `G0 A0` for 90°-rotated setup)
- **Program:** CamCraft (v0.25.0)
- **Smallest steps to reproduce:**
  1. `import_model {"name": "q05-4axis", "path": "/home/mikekinney/qa/runs/20261122-q05-4axis/q05-4axis.step"}`
  2. `load_machine {"builtin": "Tormach 770MX + microARC 4"}`
  3. `add_setup {"name": "Setup1"}`, `set_setup_post {"setup": "Setup1", "post": "Tormach PathPilot"}`, `set_stock {"axis": "x", "box": {"dx": 44, "dy": 44, "dz": 110}}`, `fit_stock {"shape": "box", "sides": [2, 2, 2, 2, 10, 0]}`, `set_wcs {"origin": [0, 0, 0]}`
  4. `add_tool {"catalog": "EM-0600-4FL", "name": "6mm Flat EM"}` and `add_tool {"catalog": "DR-0600", "name": "6mm Drill"}`
  5. Add `TopPocket` in `Setup1` (`add_operation {"kind": "pocket", "name": "TopPocket", "source": "q05-4axis", "tool": "6mm Flat EM"}`, `set_operation_geometry` on `face:0x8337da35d564eb1e`, `bottom_z: {reference: "top_of_model", offset: -5}`).
  6. Try per-op orientation: `edit_operation {"name": "TopPocket", "params": {"tool_orientation": [0, -1, 0]}}` → refused.
  7. Try tilted WCS in a second setup: `add_setup {"name": "Front_A90", "datum_from": "Setup1"}`, `set_wcs {"origin": [0, 0, 0], "rotation": [0.70710678, 0, 0, 0.70710678]}`, add `FrontDrill`, `generate` → refused with instruction to use Part alignment.
  8. Follow CamCraft's instruction: `add_setup {"name": "Setup2_A90", "datum_from": "Setup1"}`, `lay_part_on_face {"setup": "Setup2_A90", "face": {"model": "q05-4axis", "id": "face:0x9a06666bad6a627f"}, "plane": "g17", "place": false}` (returns `"turned X -90°, Y 0°, Z 0°"`), add `FrontDrill` with the two Ø6 front holes (`xy: [30,0]` and `[70,0]`, `entry_z: 20, floor_z: 8`, `top_z: {reference: "top_of_stock", offset: 1}`), `generate {}`, `post {"post": "Tormach PathPilot", "output": "/home/mikekinney/qa/runs/20261122-q05-4axis/test-probe3.nc"}`.
- **Expected:** Either per-operation tool orientation or a rotary-indexed setup (`datum_from: "Setup1"` with `part_alignment` rotated -90° about the machine's A axis `[1,0,0]`) should emit `G0 A-90` (or `G0 A90` / `G0 A270` matching microARC 4 right-hand convention) between faces without an `M0` manual re-clamp stop.
- **Actual:**
  - `edit_operation`: `edit_operation: bad params patch: unknown field tool_orientation - a pocket operation has no field "tool_orientation"; see list_operation_types`
  - Tilted WCS `generate`: `generate: FrontDrill - could not generate the toolpath: FrontDrill: invalid machining parameters: setup 'Front_A90': its WCS (G54) has its Z axis tilted 90.000 deg off the machine's Z - the tool always cuts along the machine's Z, so this would machine the part as it lies, not as the WCS shows; turn the part with the setup's Part alignment instead and keep the WCS Z pointing up`
  - Aligned setup `post` (`test-probe3.nc`): emits `(MSG, SETUP CHANGE - NEXT: Setup2_A90)` + `M0` (lines 124–125) and emits **`G0 A0`** for BOTH `Setup1` (line 19) AND `Setup2_A90` (line 139). Even with `between_setups: "continue"` (`test-probe2.nc`), both setups emit `G0 A0` (lines 19 & 48).
- **How bad:** **CRITICAL — posts wrong G-code / catastrophic gouge.** Running this program on a Tormach 770MX + microARC 4 does not rotate the 4th axis at all; `FrontDrill` drills the two Ø6 x 12 mm holes straight down into the top face (`A=0°`) right through the top pocket instead of the front face.
- **Reproducing files:** `q05-4axis.step`, `probe.camproj`, `test-probe.nc`, `test-probe2.nc`, `test-probe3.nc`, `test-explain.nc`.

---

### Problem 2 — CamCraft: `generate` on `parallel5d` / `multi_axis_rough` hangs permanently and wedges the MCP server
- **Program:** CamCraft (v0.25.0)
- **Smallest steps to reproduce:**
  1. From the `probe.camproj` state (`q05-4axis.step`, `Tormach 770MX + microARC 4`, `Setup1` active).
  2. `add_tool {"catalog": "BN-0600", "name": "6mm Ball Nose"}`
  3. `add_operation {"kind": "parallel5d", "name": "TestParallel5D", "source": "q05-4axis", "tool": "6mm Ball Nose"}`
  4. `add_operation {"kind": "multi_axis_rough", "name": "TestMultiAxisRough", "source": "q05-4axis", "tool": "6mm Ball Nose"}`
  5. `generate {"name": "TestParallel5D"}` and `generate {"name": "TestMultiAxisRough"}`
- **Expected:** Either generate a valid 4/5-axis toolpath in seconds or fail fast (e.g., refuse 5-axis op on a 4-axis machine with only an A axis, or enforce a timeout).
- **Actual:** Both `generate` calls failed with `MCP tool call to server "camcraft" timed out after 3m0s: context deadline exceeded`. Three subsequent `get_job {}` calls also timed out after 3m0s each (>15.5 minutes total elapsed). The server became permanently unresponsive, with no `cancel` tool available.
- **How bad:** **CRITICAL — STUCK / permanent server hang & data loss.** Unsaved project edits cannot be saved and the entire CAM session is deadlocked.
- **Reproducing files:** `q05-4axis.step`, `probe.camproj`.

---

### Problem 3 — CamCraft: `detect_holes` on a mill puts bottom-facing holes into lathe `cross` list, permanently bricking the operation
- **Program:** CamCraft (v0.25.0)
- **Smallest steps to reproduce:**
  1. Load `q05-4axis.step` and `Tormach 770MX + microARC 4` (mill, no turret), add `DR-0600` (`6mm Drill`).
  2. Create `Front_A90`, align front face up via `lay_part_on_face {"setup": "Front_A90", "face": {"model": "q05-4axis", "id": "face:0x9a06666bad6a627f"}, "plane": "g17", "place": false}`.
  3. `add_operation {"kind": "drill", "name": "FrontDrillAligned", "source": "q05-4axis", "tool": "6mm Drill"}`
  4. `detect_holes {"name": "FrontDrillAligned"}`
  5. `generate {"name": "FrontDrillAligned"}`
  6. Attempt to clear `cross`: `edit_operation {"name": "FrontDrillAligned", "params": {"cross": []}}`
- **Expected:** On a mill (no lathe turret), `detect_holes` should only populate upright (`+Z`) drillable holes (the two Ø6 front holes) and ignore upside-down (`-Z`) or side holes — or at minimum allow editing/removing `cross`. Also, any kept hole wider than the 6 mm drill should appear in `undersize`.
- **Actual:**
  - `detect_holes` returned `{"holes": 2, "radial_holes": 1, "skipped": [], "undersize": []}`, populating `params.drill.cross` with the bottom-facing (`-Z`) Ø20 counterbore: `{"dir": [0.0, 0.0, 1.0], "entry_d": 0.0, "floor_d": -2.0, "origin": [50.0, -0.0, -20.0], "radius": 10.0, "through": false}` (and reporting `undersize: []` despite Ø20 > Ø6).
  - `generate` refused: `generate: FrontDrillAligned - could not generate the toolpath: FrontDrillAligned: invalid machining parameters: this op lists Z holes AND cross holes - drill either the mill's holes or the turret's, not both (M5.2 v1)` (and in `Setup1` with only `cross` populated, `generate` refused: `cross holes are live-tooling work - the mounted machine is a mill with no turret`).
  - `edit_operation` refused clearing `cross`: `edit_operation: bad params patch: unknown field cross - a drill operation has no field "cross"; see list_operation_types`.
- **How bad:** **HIGH — permanently bricks the operation.** Any `Drill` op where `detect_holes` runs on a multi-sided mill part becomes un-generatable and un-fixable; the user must delete it and manually enter hole coordinates.
- **Reproducing files:** `q05-4axis.step`.

---

### Problem 4 — CamCraft: All Rotary operations refuse flat end mills and refuse prismatic/square 4-axis parts
- **Program:** CamCraft (v0.25.0)
- **Smallest steps to reproduce:**
  1. On `q05-4axis.step` with `Tormach 770MX + microARC 4` and `6mm Flat EM`: `add_operation {"kind": "rotary_parallel", "name": "TestRotaryFlat", "source": "q05-4axis", "tool": "6mm Flat EM"}`.
  2. Add `BN-0600` (`6mm Ball Nose`); add `rotary_parallel`, `rotary_contour`, and `rotary_offset` with `source: "q05-4axis"`, `tool: "6mm Ball Nose"`; call `generate` on each.
- **Expected:** Rotary Parallel / Rotary Contour (or a wrapped 2D Contour option) should allow cleaning the outside of a 40x40x100 mm bar mounted along the machine's A axis (`[1,0,0]`), ideally with a flat end mill or at least with a ball-nose cutter.
- **Actual:**
  - Step 1 refused: `add_operation: invalid parameters: No ball-nose in Setup2_A90's tools - a Rotary Parallel operation cuts with one; add it to the tool library - add 'BN-0600' (6 mm ball-nose) from the catalog?` (`list_operation_types` confirms all rotary/multi-axis ops accept `["BallNose"]` only; `2D Contour` has no wrap option).
  - Step 2 refused for ALL THREE (`rotary_parallel`, `rotary_contour`, `rotary_offset`): `invalid machining parameters: the model's normals share no rotation axis - a turned workpiece is rotary work; finish a freeform with the 5-Axis kinds`.
- **How bad:** **HIGH — blocks wrapped/rotary machining on non-cylindrical 4-axis parts** (and led directly to trying the 5-axis kinds that hung the server in Problem 2).
- **Reproducing files:** `q05-4axis.step`.

---

### Problem 5 — CamCraft: First traverse after tool change drops Z (`G0 Z47`) BEFORE XY positioning (`G0 X.. Y..`)
- **Program:** CamCraft (v0.25.0)
- **Smallest steps to reproduce:**
  1. Post `TopPocket` and `FrontDrill` with `Tormach PathPilot` on `Tormach 770MX + microARC 4` (see `test-probe3.nc` lines 8–23 and 130–142).
- **Expected:** After `G30` / `T# M6` / `G43 H#` / `G53 G0 Z0` / `G0 A0`, the first traverse should move XY (`G0 X.. Y..`) at machine Z home (`G53 Z0`) first, and ONLY THEN descend in Z to clearance height (`G0 Z47`).
- **Actual:** In `test-probe3.nc`:
  - `TopPocket` (lines 18–22): `G53 G0 Z0` → `G0 A0` → **`G0 Z47`** → **`G0 X38.568 Y-4.54`** → `G0 X38.568 Y-4.54 Z47` (redundant Z47 move!).
  - `FrontDrill` (lines 138–142): `G53 G0 Z0` → `G0 A0` → **`G0 Z47`** → **`G0 X30 Y0`** → `G0 X30 Y0 Z23`.
- **How bad:** **HIGH — collision risk.** Descending to `Z47` (47 mm above the A-axis centreline) at the `G30` tool-change XY position and then rapiding horizontally across the table at `Z47` can collide with the microARC 4 rotary headstock, chuck body (which extends >60 mm above the A centreline), or tailstock.
- **Reproducing files:** `test-probe3.nc`, `test-explain.nc`, `test-probe.nc`, `test-probe2.nc`.

---

### Problem 6 — CamCraft: `verify` and `post` skip stock simulation on a 2-op 90° setup carry (`exhausted its ray budget`) yet return `ok: true`
- **Program:** CamCraft (v0.25.0)
- **Smallest steps to reproduce:**
  1. Create `Setup1` (`TopPocket`) and `Setup2_A90` (`FrontDrill`, turned `X -90°` via `lay_part_on_face`, `datum_from: "Setup1"`).
  2. Call `generate {}`, then `verify {}`, then `post {"post": "Tormach PathPilot", "output": "/home/mikekinney/qa/runs/20261122-q05-4axis/test-probe3.nc"}`.
- **Expected:** Stock simulation should reliably verify a trivial 2-op program (one 30x16 pocket + two Ø6 drills), or if verification cannot judge the program, it should clearly warn/flag rather than silently returning `ok: true`.
- **Actual:** Both `verify` and `post` returned `"ok": true` with:
  `"not_judged": "stock sim did not judge this program: exact side-on carry exhausted its ray budget; note: the stock carried from setup 'Setup1' into setup 'Setup2_A90' is turned side-on - its turned faces are read up to one grid cell outside the real stock, never inside it, and a gouge cut before the turn is counted from the setup that cut it"`.
- **How bad:** **MEDIUM — could miss gouges or collisions** while telling the user verification passed (`ok: true`) and allowing G-code to post.
- **Reproducing files:** `q05-4axis.step`, `probe.camproj`, `test-probe3.nc`.

---

### Problem 7 — CamCraft: `add_setup` sorts setups alphabetically instead of creation/program order (no `reorder_setup` tool)
- **Program:** CamCraft (v0.25.0)
- **Smallest steps to reproduce:**
  1. Create `Setup1` with `TopPocket`.
  2. Call `add_setup {"name": "Front_A90", "datum_from": "Setup1"}` and add `FrontDrill` to `Front_A90`.
  3. Call `post {"post": "Tormach PathPilot", "output": "/home/mikekinney/qa/runs/20261122-q05-4axis/test-probe.nc"}`.
- **Expected:** `Front_A90` (created second, with `datum_from: "Setup1"`) should appear and post AFTER `Setup1`, or there should be a `reorder_setup` tool.
- **Actual:** `add_setup` returned `"setups": ["Front_A90", "Setup1"]`, and `test-probe.nc` / `test-probe2.nc` posted `Front_A90` first (line 4) and `Setup1` second (line 38). Only renaming `Front_A90` to `Setup2_A90` put it after `Setup1`.
- **How bad:** **MEDIUM — posts setups in wrong machining order** (can cut secondary/finishing setups before primary roughing setups, crashing tools into uncut stock).
- **Reproducing files:** `probe.camproj`, `test-probe.nc`, `test-probe2.nc`.

---

### Problem 8 — CamCraft: `detect_pockets` ignores the active setup's `part_alignment`
- **Program:** CamCraft (v0.25.0)
- **Smallest steps to reproduce:**
  1. With `Front_A90` active (`lay_part_on_face` turned `X -90°` so the Front face is `+Z`), call `detect_pockets {"model": "q05-4axis"}`.
- **Expected:** Detect pockets/slots in the active setup's aligned frame (consistent with `detect_holes`, which did respect `Front_A90`'s alignment).
- **Actual:** Returned `"frame": "world"` and only the Top-face pocket (`floor_z: 15.0, top_z: 20.0`, which is sideways in `Front_A90`).
- **How bad:** **LOW–MEDIUM — confuses users** and prevents auto-detecting pockets/slots on rotated 4th-axis faces (e.g., bottom slot, back counterbore).
- **Reproducing files:** `q05-4axis.step`, `probe.camproj`.

---

## Confusing Tool Names, Schemas, and Parameters (Even When Working)

1. **No spot drill in `list_tool_catalog`:** Catalog includes 118° twist drills (`DR-*`) and 90° chamfer mills (`CM-*`, `kind: "taper"`), which `Drill` refuses (`tool_kinds: ["Drill", "Tap", "Bore"]`). Users must hand-author a custom `kind: "drill"` definition with `tip_angle: 90` to spot-drill.
2. **`fit_stock` `sides` schema vs. `axis: "x"`:** `fit_stock.json` documents `sides` as `"[-X, +X, -Y, +Y, -Z, +Z]"`, but when `axis: "x"`, the allowances are applied in the **stock's own rotated frame**, not world XYZ: `sides: [10, 0, 2, 2, 2, 2]` produced `{dx: 50, dy: 44, dz: 104}` instead of `{dx: 44, dy: 44, dz: 110}`; we had to pass `[2, 2, 2, 2, 10, 0]` to add 10 mm on world `-X`.
3. **`set_stock` `box: {dx, dy, dz}` with `axis: "x"`:** `dz` becomes the X-axis bar length (110 mm), while `add_setup(datum_from)` later re-stored the copied stock as `{dx: 110, dy: 44, dz: 44}` with identity rotation.
4. **Misleading `list_operation_types` note:** `"Fields absent from the defaults are optional and off; set them in a params patch to turn them on."` gives no list of optional fields, and internal fields like `cross` on `Drill` are hidden from `list_operation_types` and rejected by `edit_operation`.
5. **Loading a machine does not switch the document post:** `load_machine {"builtin": "Tormach 770MX + microARC 4"}` still left the document post as `Fanuc (built-in)` (`set_post_options {}` showed `"post": "Fanuc (built-in)"`), requiring `add_setup` + `set_setup_post` or passing `post` on every `post` call.
6. **Misleading error phrasing on `add_operation`:** Refusing `6mm Flat EM` for `rotary_parallel` said `"No ball-nose in Setup2_A90's tools"` even though the tool library is document-wide, not per-setup.
7. **Built-in machine name format:** `load_machine` requires `"Tormach 770MX + microARC 4"` (no slug alias like `tormach-770mx-microarc4`).

---

RESULT: 8 problems (8 CamCraft, 0 SolidCraft); end-to-end: not posted
