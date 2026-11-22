# QA Report — Q07 Router MDF Shelf Bracket (SolidCraft v0.46.2 & CamCraft v0.25.0)

**Workspace:** `/home/mikekinney/qa/runs/20261122-q07-router-mdf`  
**Programs tested:** SolidCraft `0.46.2` (commit `f4990915b4a91c3c82bab0745dd521f95078a2ac`, headless) and CamCraft `0.25.0` (headless)  
**Files produced:**
- [q07-router.sc](file:///home/mikekinney/qa/runs/20261122-q07-router-mdf/q07-router.sc) (148,694 B — SolidCraft native design)
- [q07-router.step](file:///home/mikekinney/qa/runs/20261122-q07-router-mdf/q07-router.step) (72,215 B — STEP export)
- [q07-router-iso.png](file:///home/mikekinney/qa/runs/20261122-q07-router-mdf/q07-router-iso.png) (355,111 B — SolidCraft model render)
- [q07-router.camproj](file:///home/mikekinney/qa/runs/20261122-q07-router-mdf/q07-router.camproj) (5,655,285 B — verified 4-op CamCraft project)
- [q07-router.nc](file:///home/mikekinney/qa/runs/20261122-q07-router-mdf/q07-router.nc) (14,301 B, 526 lines — posted `grblHAL` G-code)
- [STEPS.log](file:///home/mikekinney/qa/runs/20261122-q07-router-mdf/STEPS.log) (step-by-step execution log)

---

## Step Summary Table

| Step | What you did | Result | Exact message or numbers |
|---|---|---|---|
| 1a | SolidCraft: `new_design` + 14 params + 250×180×18 box − 150×100 cutout + 5× R5 outside fillets + 1× R25 inside fillet | OK | `vol=541931.4165 mm³` (analytic `541931.4165 mm³`), `bbox=[0,0,0]-[250,180,18]`, `bodies=1`, 0 warnings |
| 1b | SolidCraft: 3 mm top outside perimeter round-over (`solid.fillet` on `[125,0,18]`) | OK | Single tangent-chain pick succeeded; `vol=540319.9789 mm³` (analytic `540319.88 mm³`), `bbox=[0,0,0]-[250,180,18]` |
| 1c | SolidCraft: Parametric dado sketch (`dof=0`) + extrude cut (18.5×6 mm, 40 mm from end at X=210) + Ø35×12 blind bore at `(50,50)` + 3× Ø8 through holes at `(30,140),(70,140),(160,40)` | OK | `vol=517251.7522 mm³` (analytic `517251.75 mm³`), `bbox=[0,0,0]-[250,180,18]`, `bodies=1` |
| 1d | SolidCraft: `measure`, `body_topology`, `inspect_design`, `screenshot`, `save`, `export` | OK | `closed=true`, 37 faces, 0 timeline errors/warnings, all bore/hole radii & extents verified; saved `q07-router.sc` (148,694 B) and `q07-router.step` (72,215 B) |
| 2a | CamCraft: `import_model`, `load_machine(Sienci AltMill 4x4)`, `set_work_material('mdf')` & `('wood')` | WRONG | Imported `Bracket` (15,236 facets, `[0,0,0]-[250,180,18]`); AltMill 4x4: `kind="mill"`, `post="grblHAL"`, RPM `1200-24000`, `coolants=[M8 Flood, M7 Mist]`; `set_work_material` refused `'mdf'` and `'wood'`: `set_work_material: unknown workpiece material 'mdf' (one of aluminium_6061, mild_steel, stainless_304, grey_cast_iron, brass, plastics_pom)` |
| 2b | CamCraft: Checked `Tormach 24R` then reloaded `Sienci AltMill 4x4` | OK | `Tormach 24R`: `kind="mill"`, `post_offer="Tormach PathPilot"`, RPM `10000-24000`, `spindle_power=1.5`, `units="inch"`, no `coolants` |
| 2c | CamCraft: `add_setup(RouterSetup)`, `set_setup_post(grblHAL)`, `set_sheet_job(260×190×18, top, front_left, allowance=0.5, clamp_clearance=10)` | OK | `stock_bounds=[-130,-95,0]-[130,95,18]`, `wcs_origin=[-130,-95,18]`, misaligned with `Bracket` at `[0,0,0]-[250,180,18]` |
| 2d | CamCraft: `move_model(Bracket, [-125,-90,0])` + re-call `set_sheet_job` | WRONG | `move_model` moved `Stock` with `Bracket`, and re-calling `set_sheet_job` kept the stale offset: `stock_bounds=[-255,-185,0]-[5,5,18]`, `wcs_origin=[-255,-185,18]` while `Bracket=[-125,-90,0]-[125,90,18]` |
| 2e | CamCraft: Workaround `fit_stock(sides=[5,5,5,5,0,0])` + `set_sheet_job` | OK | Aligned `stock_bounds=[-130,-95,0]-[130,95,18]` around `Bracket=[-125,-90,0]-[125,90,18]`, `wcs_origin=[-130,-95,18]` |
| 3a | CamCraft: `add_tool` from catalog: `Router up-cut 6 mm` (T1), `Router up-cut 3 mm` (T2), `Router ball nose 6 mm` (T3) | OK | Added T1/T2/T3; no Forstner/Ø35 plunge bit or round-over bit in catalog; all 18 wood catalog tools default to `feed=600` at `rpm=18000` (`0.017 mm/tooth`) |
| 3b | CamCraft: `detect_pockets(create=1)` (`Dado`) + `suggest_cutting_data(Dado)` (no material) | WRONG | `Dado` created with `coolant="flood"` on AltMill router; `suggest_cutting_data` refused: `suggest_cutting_data: no workpiece material is stated - pick one first` |
| 3c | CamCraft: Workaround `set_work_material(plastics_pom)` + `suggest_cutting_data` / `apply_cutting_data` | OK | `Router up-cut 6 mm` & `Router ball nose 6 mm`: `Vc=350 m/min`, `RPM=18568`, `fz=0.0525 mm`, `feed=1950 mm/min` |
| 4a | CamCraft: Added `HingeCupBore` (`Circular`), `ThroughHoles` (`Bore`) + `detect_holes`, `TopRoundOver` (`3D Finish`, `parallel`, curved patch), `ProfileCutout` (`2D Contour`, 4 tabs, `ramp`) | WRONG | `detect_holes` on `ThroughHoles` (`Bore`, D6 tool, max coreless Ø18 mm) returned `holes: 4, skipped: []` — failed to skip the Ø35 mm (`radius: 17.5`) blind bore |
| 5a | CamCraft: `generate({})` on all 5 ops | STUCK | `MCP tool call to server "camcraft" timed out after 3m0s: context deadline exceeded` (actually completed in ~6 min) |
| 5b | CamCraft: `get_job`, `verify`, `render_stock` (top & iso) | WRONG | `verify ok=false`: `HingeCupBore` (`Circular`) had 3 `rapid_in_stock` moves + clamp-plane violation at `Z-12.000`; `ProfileCutout` had 89 XY `crossing` warnings, contoured holes & tabbed Ø8 holes, refused `ramp` on `Side::Outside`; `TopRoundOver` (`3D Finish` default `tol=0.05`) rastered flat top & gouged `0.024 mm` blocking post; `Dado` `ramp` fell back to axial dive |
| 5c | CamCraft: Workarounds — deleted `TopRoundOver` & `HingeCupBore`, replaced Ø35 with `HingeCupPocket` (`detect_pockets` index 0, helical 2D Pocket), removed Ø35 from `ThroughHoles`, set `Dado` `entry="helix"`, picked outer bottom chain `chain:0xedf0d9948eba4aa4` on `ProfileCutout`, disabled coolant on all ops; `generate`, `verify`, `render_stock` | OK | Generated in 3 s; `verify ok=true`, `findings=[]`, `gouge=0`, 4 tabs visible in `render_stock`; `ProfileCutout` still refuses `ramp` and `helix` on `Side::Outside` |
| 5d | CamCraft: `save_project(q07-router.camproj)` & `post(grblHAL -> q07-router.nc)` | OK | Saved `q07-router.camproj` (5.65 MB); posted `q07-router.nc` (526 lines, 4 ops) |
| 5e | Machinist G-code inspection of `q07-router.nc` | WRONG | Units `G21`, no `M8`/`M7`, `S18568`, `F1950`/`F600`, max depth `Z-18.3` (0.3 mm into spoilboard), exact part/dado/bore/hole/tab geometry; **BUT** found: (1) full 18 mm vertical plunge into uncut MDF at `(X246.667, Y90.812)` on `ProfileCutout` tabbed pass (line 458), (2) `Bore` is non-helical double-plunge/double-retract, (3) 3 redundant `M5`/`G53 G0 Z0`/`T1 M6` tool changes for the same `T1`, (4) `G0` rapid to exact previous floor Z (`Z-3`/`Z-4`/`Z-8`) in `Dado` and `HingeCupPocket` |
| 5f–5g | CamCraft: Tested `TopRoundOver` (`3D Finish`, `pattern="waterline"`, `tolerance=0.01`, `top_z=0`, `bottom_z=-3.1`) | STUCK | `generate(TopRoundOver)` timed out at 3m0s, and subsequent `get_job` also timed out at 3m0s (server blocked >6 min total). Clean verified 4-op `.camproj` and `.nc` already saved on disk |

---

## Problems Found

### Problem 1 — CamCraft: No wood or MDF workpiece material in `set_work_material` / `set_stock_material`
- **Program:** CamCraft
- **Repro steps:**
  1. `set_work_material {"material": "mdf"}`
  2. `set_work_material {"material": "wood"}`
- **Expected:** MDF / plywood / hardwood / softwood materials are available so `suggest_cutting_data` and `apply_cutting_data` can produce wood/MDF speeds & feeds matching the 18 `"material_group": "wood"` cutters in `list_tool_catalog`.
- **Actual:** Both calls are refused with exact error:
  > `set_work_material: unknown workpiece material 'mdf' (one of aluminium_6061, mild_steel, stainless_304, grey_cast_iron, brass, plastics_pom)`
  And without a material set, `suggest_cutting_data {"operation": "Dado"}` fails with:
  > `suggest_cutting_data: no workpiece material is stated - pick one first`
- **Severity:** Medium — blocks router/woodworking users from using CamCraft's feeds-and-speeds engine; workaround requires choosing `plastics_pom` or entering feeds manually.
- **Repro files:** [q07-router.camproj](file:///home/mikekinney/qa/runs/20261122-q07-router-mdf/q07-router.camproj)

### Problem 2 — CamCraft: Built-in wood router catalog feeds are dangerously slow for MDF/wood
- **Program:** CamCraft
- **Repro steps:**
  1. `list_tool_catalog {}` and inspect all 18 `"material_group": "wood"` entries (e.g., `Router up-cut 6 mm`, `Router up-cut 1/4 in`, `Router surfacing 25 mm`).
- **Expected:** Sensible router feeds for wood/MDF (e.g., ~2000–5000 mm/min for a 6 mm 2-flute at 18,000 RPM, chipload ~0.06–0.14 mm/tooth).
- **Actual:** Every single one of the 18 wood router tools defaults to `feed: 600.0` mm/min at `rpm: 18000.0` (`12000.0` for 25 mm surfacing) — a chipload of `0.0167 mm/tooth` (`0.00065 in/tooth`).
- **Severity:** Medium-High — rubbing at 600 mm/min @ 18,000 RPM in MDF causes severe friction, burned edges, dulled carbide, and fire risk on a wood router.
- **Repro files:** [q07-router.camproj](file:///home/mikekinney/qa/runs/20261122-q07-router-mdf/q07-router.camproj)

### Problem 3 — CamCraft: Built-in wood router `Sienci AltMill 4x4` defines M8 Flood coolant and all new ops default to `coolant: "flood"`
- **Program:** CamCraft
- **Repro steps:**
  1. `load_machine {"builtin": "Sienci AltMill 4x4"}`
  2. `add_operation {"kind": "contour", "tool": "Router up-cut 6 mm", "source": "Bracket"}` (or `detect_pockets` with `create: 1`)
  3. `get_job {}`
- **Expected:** A wood router (`Sienci AltMill 4x4`) has no flood coolant (compare `Tormach 24R`, which omits `coolants` and has `spindle_min: 10000`), and operations on a dry router default to `coolant: "off"` so `M8` is not emitted into G-code.
- **Actual:** `Sienci AltMill 4x4` has `"kind": "mill"`, `"spindle_min": 1200.0`, and `"coolants": [{"mCode": "M8", "name": "Flood"}, {"mCode": "M7", "name": "Mist"}]`, and every created operation defaults to `"coolant": "flood"`. Unless the user manually runs `set_operation_coolant {"coolant": "off"}` on every operation, CamCraft posts `M8` on a dry wood router.
- **Severity:** High — posts wrong G-code (`M8` flood coolant on an MDF router job, which can trigger an unexpected relay/auxiliary output or soak MDF if mist/flood is connected).
- **Repro files:** [q07-router.camproj](file:///home/mikekinney/qa/runs/20261122-q07-router-mdf/q07-router.camproj)

### Problem 4 — CamCraft: `set_sheet_job` ignores model bounds, and `move_model` moves stock with model so they stay misaligned
- **Program:** CamCraft
- **Repro steps:**
  1. `import_model {"path": "/home/mikekinney/qa/runs/20261122-q07-router-mdf/q07-router.step", "name": "Bracket"}` (bounds `[0,0,0]` to `[250,180,18]`)
  2. `set_sheet_job {"length": 260, "width": 190, "thickness": 18, "z_zero": "top", "xy_zero": "front_left", "allowance": 0.5, "clamp_clearance": 10}`
  3. `move_model {"name": "Bracket", "position": [-125, -90, 0]}`
  4. `set_sheet_job {"length": 260, "width": 190, "thickness": 18, "z_zero": "top", "xy_zero": "front_left", "allowance": 0.5, "clamp_clearance": 10}`
  5. `get_job {}`
- **Expected:** Either `set_sheet_job` centers the sheet on existing models (or places `[0,0,0]-[260,190,18]` when the part starts at the origin), or re-running `set_sheet_job` resets the stock origin instead of compounding `move_model`'s translation.
- **Actual:** Step 2 creates stock at `[-130,-95,0]-[130,95,18]`, leaving half the part outside the sheet. Step 3 moves `Stock` along with `Bracket` by `[-125,-90,0]`. Step 4 keeps that stale stock translation and puts `stock_bounds` at `[-255,-185,0]-[5,5,18]` and `wcs_origin` at `[-255,-185,18]` while `Bracket` is at `[-125,-90,0]-[125,90,18]`. Workaround: `fit_stock {"sides": [5,5,5,5,0,0]}` followed by `set_sheet_job`.
- **Severity:** High — silently shifts WCS and stock off the part; if unnoticed, cuts in the wrong place or crashes into clamps/fixtures.
- **Repro files:** [q07-router.step](file:///home/mikekinney/qa/runs/20261122-q07-router-mdf/q07-router.step)

### Problem 5 — CamCraft: `detect_holes` on `Bore` fails to skip Ø35 mm hole exceeding 3×D coreless limit
- **Program:** CamCraft
- **Repro steps:**
  1. Import `q07-router.step` (contains three Ø8 mm holes and one Ø35 mm blind bore).
  2. Add tool `Router up-cut 6 mm` (`diameter: 6.0`).
  3. `add_operation {"kind": "bore", "name": "ThroughHoles", "source": "Bracket", "tool": "Router up-cut 6 mm"}`
  4. `detect_holes {"name": "ThroughHoles"}`
- **Expected:** Per `detect_holes` documentation (`"A Bore operation leaves out holes wider than its tool's ring clears without a core (3 x tool diameter + 2 x stock to leave) and names each in skipped"`), with `D = 6 mm` (`3×D = 18 mm`), the Ø35 mm hole (`radius = 17.5 mm`) must be skipped and listed in `skipped`, leaving 3 holes.
- **Actual:** Returned `{"holes": 4, "operation": "ThroughHoles", "radial_holes": 0, "skipped": [], "status": "stale", "undersize": []}` — including `{"xy": [-75.0, -40.0], "radius": 17.5, "floor_z": 6.0, "through": false}` (likely comparing `radius <= 3*D`, i.e., `17.5 <= 18.0`, instead of `diameter <= 3*D`, `35.0 <= 18.0`), and `generate` succeeded with `warnings: []`.
- **Severity:** High — leaves an uncut Ø23 mm MDF core inside the blind bore (or detaches a flying slug in a through hole).
- **Repro files:** [q07-router.step](file:///home/mikekinney/qa/runs/20261122-q07-router-mdf/q07-router.step)

### Problem 6 — CamCraft: `Circular` operation rapids through stock at cut depth (`Z-12.000`), blocking post
- **Program:** CamCraft
- **Repro steps:**
  1. On `RouterSetup` with `sheet_job` (`clamp_clearance: 10`), `Bracket` at `[-125,-90,0]-[125,90,18]`, `Router up-cut 6 mm`:
  2. `add_operation {"kind": "circular", "name": "HingeCupBore", "source": "Bracket", "tool": "Router up-cut 6 mm", "params": {"center": [-75,-40], "radius": 17.5, "top_z": {"reference": "top_of_model", "offset": 0}, "bottom_z": {"reference": "top_of_model", "offset": -12}, "entry": "helix", "helix_radius": 2.5, "stepdown": 4, "stepover": 3, "feed": 1950, "plunge_feed": 600}}`
  3. `generate {"name": "HingeCupBore"}` then `verify {"name": "HingeCupBore"}`
- **Expected:** Between stepdowns, `Circular` either retracts to `clearance_z` (`Z+25`) before rapiding or uses feed moves; zero `rapid_in_stock` warnings.
- **Actual:** `verify` blocks posting with 3 `rapid_in_stock` findings and a post-gate refusal:
  > `rapid move 25 travels through stock (below the clearance plane, or at it below the stock top)` (same for moves 45 and 65)  
  > `HingeCupBore: rapids across at Z-12.000, below setup 'RouterSetup''s clamp plane Z10.000 (sheet top plus 10.000 mm clamp clearance) - raise the operation's clearance and retract heights`
  Note: `Circular` has no `retract_z` parameter and its `clearance_z` was already `top_of_stock + 25`. Workaround: replace `Circular` with `2D Pocket` (`detect_pockets` index 0).
- **Severity:** Critical — rapid move at bottom of pocket (`Z-12`) would crash/gouge the cutter through stock or clamps; blocks posting; error text mentions a non-existent retract parameter.
- **Repro files:** [q07-router.step](file:///home/mikekinney/qa/runs/20261122-q07-router-mdf/q07-router.step)

### Problem 7 — CamCraft: `2D Contour` on filleted solid model creates 89 XY crossings & tabs inside Ø8 holes; and refuses `ramp`/`helix` on `Side::Outside`
- **Program:** CamCraft
- **Repro steps:**
  1. `add_operation {"kind": "contour", "name": "ProfileCutout", "source": "Bracket", "tool": "Router up-cut 6 mm", "params": {"side": "outside", "entry": "ramp", "stepdown": 4.5, "tab_count": 4, "tab_height": 3, "tab_width": 10, "top_z": {"reference": "top_of_model", "offset": 0}, "bottom_z": {"reference": "bottom_of_model", "offset": -0.3}}}`
  2. `generate {"name": "ProfileCutout"}` and `verify {"name": "ProfileCutout"}`
  3. Workaround: `set_operation_geometry {"operation": "ProfileCutout", "picks": [{"model": "Bracket", "id": "chain:0xedf0d9948eba4aa4"}]}` and try `entry: "ramp"` and `entry: "helix"`.
- **Expected:** `side: "outside"` contours only the outer perimeter without self-crossings, and `entry: "ramp"` ramps smoothly down along the outside contour wall (standard CNC router practice to avoid vertical plunging).
- **Actual:**
  - On `source: "Bracket"`, `ProfileCutout` generated **89 blocking `crossing` warnings** (`"feed moves 7 and 519 cross in XY at shared depth"` etc.), contoured all 4 interior holes, and tried to place 4×10 mm tabs on Ø8 mm holes (`"tabs: 4 tabs of 10.000 mm - with the tool diameter and the leads they fill the whole loop; the tabbed levels cut nothing on this loop"`).
  - Even on the picked bottom outer chain (`chain:0xedf0d9948eba4aa4`), `2D Contour` refuses both `ramp` and `helix`:
    > `entry: the ramp needs bounded air only a bore (Side::Inside) has - grazing dive at the lead-in, unchanged`  
    > `entry: the helix needs bounded air only a bore (Side::Inside) has - grazing dive at the lead-in, unchanged`
- **Severity:** High — blocks posting on filleted solids unless manual chain-picking is used, and makes ramped entry impossible on outside sheet cutouts.
- **Repro files:** [q07-router.camproj](file:///home/mikekinney/qa/runs/20261122-q07-router-mdf/q07-router.camproj)

### Problem 8 — CamCraft: CRITICAL — `2D Contour` tabbed pass shifts lead-in into uncut solid MDF and plunges full 18 mm depth in one move (missed by `verify`)
- **Program:** CamCraft
- **Repro steps:**
  1. Open [q07-router.camproj](file:///home/mikekinney/qa/runs/20261122-q07-router-mdf/q07-router.camproj), `verify {}`, and `post {"post": "grblHAL", "output": ".../q07-router.nc"}`.
  2. Inspect `ProfileCutout` in [q07-router.nc](file:///home/mikekinney/qa/runs/20261122-q07-router-mdf/q07-router.nc#L379-L460).
- **Expected:** Every Z pass of `ProfileCutout` (`stepdown: 4.5`) plunges at most 4.5 mm into uncut material (starting in the already-cleared kerf from upper passes).
- **Actual:** Passes 1–4 (`Z-4.5`, `Z-9`, `Z-13.5`, `Z-15.3`) all lead in at `(X258.066, Y2.09)` (bottom-right edge). On pass 5 (`Z-18`, the first tabbed pass, lines 456–458), CamCraft moves the lead-in start point to `(X246.667, Y90.812)` — in **solid, never-previously-cut 18 mm thick MDF** (above the `Y88` kerf) — and executes a **full 18 mm vertical plunge**:
  ```gcode
  G0 X246.667 Y90.812 Z25
  G0 X246.667 Y90.812 Z2
  G1 X246.667 Y90.812 Z-18 F600
  ```
  `verify {}` reported `ok: true` with `findings: []` and `gouge: 0`!
- **Severity:** Critical — posts dangerous G-code: plunging a Ø6 mm router bit 18 mm straight down into solid MDF at `F600` can snap the cutter, stall the spindle, rip the sheet from hold-downs, or ignite MDF dust.
- **Repro files:** [q07-router.camproj](file:///home/mikekinney/qa/runs/20261122-q07-router-mdf/q07-router.camproj), [q07-router.nc](file:///home/mikekinney/qa/runs/20261122-q07-router-mdf/q07-router.nc#L456-L460)

### Problem 9 — CamCraft: `Bore` operation (`ThroughHoles`) generates non-helical double-plunge / double-retract toolpath
- **Program:** CamCraft
- **Repro steps:**
  1. Open [q07-router.camproj](file:///home/mikekinney/qa/runs/20261122-q07-router-mdf/q07-router.camproj) and inspect `ThroughHoles` in [q07-router.nc](file:///home/mikekinney/qa/runs/20261122-q07-router-mdf/q07-router.nc#L55-L288).
- **Expected:** Smooth helical interpolation (`G3` with simultaneous Z descent) down to `Z-18.3` with one retract per hole.
- **Actual:** For every single 3 mm stepdown on every hole (lines 58–67, repeated 21 times across 3 holes), `Bore` plunges vertically at the center (`G1 Z-3 F600`), feeds 1 mm radially, retracts all the way to `Z25`, rapids back down to `Z2`, plunges vertically a *second* time at the wall (`G1 Z-3 F600`), cuts a flat `G3` circle, and retracts to `Z25` again (14 full retracts and 14 vertical plunges per Ø8 mm hole).
- **Severity:** Medium-High — posts inefficient, non-helical G-code that repeatedly plunges vertically with a router end mill instead of helically interpolating.
- **Repro files:** [q07-router.camproj](file:///home/mikekinney/qa/runs/20261122-q07-router-mdf/q07-router.camproj), [q07-router.nc](file:///home/mikekinney/qa/runs/20261122-q07-router-mdf/q07-router.nc#L55-L133)

### Problem 10 — CamCraft: `grblHAL` post emits redundant `M5` / `G53 G0 Z0` / `T1 M6` tool changes between consecutive operations using the same tool
- **Program:** CamCraft
- **Repro steps:**
  1. Post [q07-router.camproj](file:///home/mikekinney/qa/runs/20261122-q07-router-mdf/q07-router.camproj) (all 4 ops use `T1 Router up-cut 6 mm`) with `post: "grblHAL"`.
  2. Inspect lines 48–54, 288–294, and 371–377 of [q07-router.nc](file:///home/mikekinney/qa/runs/20261122-q07-router-mdf/q07-router.nc).
- **Expected:** When consecutive operations in the same setup use the same tool (`T1`), no redundant `M5` / `G53 G0 Z0` / `T1 M6` tool change is emitted.
- **Actual:** Between every operation (`Dado` → `ThroughHoles` → `HingeCupPocket` → `ProfileCutout`), the post emits `M5`, `G53 G0 Z0`, `T1 M6`, `S18568 M3` — forcing 3 unnecessary manual tool-change pauses and spindle restarts for the same `T1`.
- **Severity:** Medium — posts bad G-code for manual-tool-change grblHAL routers, halting the program 3 extra times mid-job.
- **Repro files:** [q07-router.camproj](file:///home/mikekinney/qa/runs/20261122-q07-router-mdf/q07-router.camproj), [q07-router.nc](file:///home/mikekinney/qa/runs/20261122-q07-router-mdf/q07-router.nc#L48-L55)

### Problem 11 — CamCraft: `3D Finish` (`TopRoundOver`) hangs (>3–6 min), ignores patch area, and self-gouges at default tolerance
- **Program:** CamCraft
- **Repro steps:**
  1. Case A (`parallel`): Add `finish3d` with `Router ball nose 6 mm`, default `tolerance: 0.05`, `pattern: "parallel"`, `top_z: top_of_model + 0`, `bottom_z: top_of_model - 3.2`; pick `patch:0xd96cba0fcbb117b7` ("Curved face 1 of 'Bracket'") via `set_operation_geometry`; run `generate {}` then `verify {}`.
  2. Case B (`waterline`): Add `finish3d` with `Router ball nose 6 mm`, `pattern: "waterline"`, `tolerance: 0.01`, `top_z: top_of_model + 0`, `bottom_z: top_of_model - 3.1`, `source: "Bracket"`; run `generate {"name": "TopRoundOver"}`.
- **Expected:** Generates in a few seconds, restricts cuts to the picked curved fillet region, and passes CamCraft's own gouge check at its default `tolerance`.
- **Actual:**
  - Case A (`parallel`): Timed out the 3m0s MCP call (`context deadline exceeded`), took ~6 minutes total, rastered across the entire flat top of the 250×180 part despite only the curved fillet patch being picked, AND failed `verify` by gouging 2 cells by `0.024 mm`:
    > `gouge: program cuts below the model in 2 cell(s), worst 0.02 mm at (123.69, -13.81, z 17.32)`
    (Default `finish3d` `tolerance: 0.05` exceeds the stock verifier's `0.01` tolerance!)
  - Case B (`waterline`, `tol=0.01`): Timed out `generate` after 3m0s AND timed out the following `get_job` after another 3m0s (>6 min total, blocking the entire server).
- **Severity:** High — hangs/blocks the CAM server on a simple 250×180 mm part, wastes machine time scanning unpicked flat faces, and fails its own post gate at default settings.
- **Repro files:** [q07-router.step](file:///home/mikekinney/qa/runs/20261122-q07-router-mdf/q07-router.step), [q07-router.camproj](file:///home/mikekinney/qa/runs/20261122-q07-router-mdf/q07-router.camproj)

### Problem 12 — CamCraft: `Slot` `entry: "ramp"` falls back to axial dive on open 80×18.5 mm slot; plus zero-clearance `G0` rapid to previous floor Z on stepdowns
- **Program:** CamCraft
- **Repro steps:**
  1. `detect_pockets {"model": "Bracket", "create": 1, "tool": "Router up-cut 6 mm", "name": "Dado"}`
  2. `edit_operation {"name": "Dado", "params": {"entry": "ramp", "stepdown": 3}}`
  3. `generate {"name": "Dado"}` and `verify {"name": "Dado"}`
  4. Switch `Dado` to `entry: "helix"` and inspect [q07-router.nc](file:///home/mikekinney/qa/runs/20261122-q07-router-mdf/q07-router.nc#L31-L33) and `HingeCupPocket` [q07-router.nc](file:///home/mikekinney/qa/runs/20261122-q07-router-mdf/q07-router.nc#L323-L325).
- **Expected:** An 80 mm long × 18.5 mm wide open slot easily accommodates a ramp entry, and multi-pass `G0` rapids stop at a small safety clearance (e.g., 0.5–2 mm) above the previous cut floor before engaging feed/helix.
- **Actual:** `entry: "ramp"` logged:
  > `entry: the ramp's chord is blocked - axial dive on the floor instead at (79.200, -10.500)`
  And with `entry: "helix"`, subsequent passes emit `G0 X209.2 Y84.5 Z-3` (line 32 of `Dado`) and `G0 X55.085 Y52.503 Z-4` / `Z-8` (lines 324, 348 of `HingeCupPocket`) — rapiding at `G0` straight onto the cut floor with 0.000 mm clearance.
- **Severity:** Low-Medium — forces axial plunge when ramp is requested; zero-clearance rapid to cut floor can mark MDF or chip carbide if the sheet lifts slightly.
- **Repro files:** [q07-router.camproj](file:///home/mikekinney/qa/runs/20261122-q07-router-mdf/q07-router.camproj), [q07-router.nc](file:///home/mikekinney/qa/runs/20261122-q07-router-mdf/q07-router.nc#L31-L34)

---

## Confusing Tool Names, Parameters, or Outputs (Non-Blocking Observations)

- **SolidCraft — `describe_command` parameter name (`id` vs `command`):** `execute` takes `command`, while `describe_command` takes `id` (`describe_command: missing required argument 'id'`). Minor inconsistency.
- **SolidCraft — `body_topology` through-hole cylinder origin:** Through holes created by `solid.hole` report `origin.z = -600.1677...` with `extent = [600.1677..., 618.1677...]` (mathematically Z `0..18`, accurate, just surprising when reading raw `origin`).
- **CamCraft — Router machine kind & post activation:** Both `Sienci AltMill 4x4` and `Tormach 24R` report `"kind": "mill"` (no `"router"` kind), and `load_machine` only offers the machine's post (`post_offer: "grblHAL"`) while keeping `setups[0].posts_with: "Fanuc (built-in)"` until you create a named setup (`add_setup`) and call `set_setup_post`.
- **CamCraft — Missing woodworking tool types:** Built-in catalog has no Forstner/hinge-cup plunge bit and no corner-rounding / round-over profile cutter.
- **CamCraft — `list_operation_types` omits enum variants & stepover for `finish3d`:** `finish3d.pattern` is typed only as `"string"` with default `"parallel"` (the 18 allowed pattern names like `waterline`, `scallop`, `contour` are only revealed by passing an invalid string), and `finish3d` exposes no explicit `stepover` parameter.
- **CamCraft — `list_features` chain length discrepancy:** Closed outer bottom chain `chain:0xedf0d9948eba4aa4` reported `closed: true`, `length: 768.535 mm` instead of the true perimeter `838.540 mm` (missing the 70 mm segment between `start: [-25,15,0]` and `end: [-25,85,0]`), although `2D Contour` still machined the full 838.54 mm closed loop.
- **CamCraft — `tab_height` reference plane:** `tab_height: 3.0` is measured upward from `bottom_z` (`-18.3` with 0.3 mm spoilboard allowance), placing the tab top at `Z-15.3` — leaving `2.7 mm` of MDF above the spoilboard rather than `3.0 mm`.
- **CamCraft — `Circular` post-gate error message:** Advises `"raise the operation's clearance and retract heights"`, even though `Circular` has no `retract_z` parameter.

---

RESULT: 12 problems (12 CamCraft, 0 SolidCraft); end-to-end: posted
