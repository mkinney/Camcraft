# QA Test Report: `q01-2d-plate`

- **Programs tested:** SolidCraft v0.46.2 (`f4990915b4a91c3c82bab0745dd521f95078a2ac`, headless MCP), CamCraft v0.25.0 (headless MCP)
- **Workspace:** `/home/mikekinney/qa/runs/20261122-q01-2d-plate`
- **Part:** 120 × 80 × 12 mm 6061-T6 aluminium mounting plate, R6 outer corners, 60 × 40 × 6 mm centred pocket (R4 inner corners) with Ø16 mm full-height centre island, 4 × Ø6.6 mm through holes on 100 × 60 mm pattern, 1 × M6 tapped hole (Ø5.0 × 10 mm blind) in island centre, 0.5 mm × 45° chamfer on outside top edge.
- **Analytic CAD checks:**
  - Plate + R6 corners: `114829.168 mm³` (measured: `114829.16802635079 mm³`)
  - After 60 × 40 × 6 pocket (R4) & Ø16 island: `101717.947 mm³` (measured: `101717.94671058463 mm³`)
  - After 4 × Ø6.6 thru + Ø5.0 × 10 blind hole: `99879.424 mm³` (measured: `99879.42385785082 mm³`)
  - Final with 0.5 × 45° outer top chamfer: `99830.842 mm³` (measured: `99830.84236856432 mm³`), bbox `[-60, -40, 0]..[60, 40, 12]`, 35 faces.

## Step Summary

| step | what you did | result | exact message or numbers |
|---|---|---|---|
| 1a | SolidCraft: created `q01-2d-plate`, parametric sketch + extrude 120×80×12 + R6 corners + 60×40×6 pocket around Ø16 island + R4 inner corners | OK | v0.46.2; Plate+R6 vol=`114829.168 mm³`; Pocket+Island+R4 vol=`101717.947 mm³`; bbox=`[-60,-40,0]..[60,40,12]`; 21 faces |
| 1b | SolidCraft: added 4×Ø6.6 through holes (100×60 pattern), M6 tapped hole (`thread:"M6"`, `depth:10`, `type:"simple"` → auto Ø5.0), and 0.5×45° outer top chamfer; verified via `measure` | OK | Holes vol=`99879.424 mm³`; Final vol=`99830.842 mm³` (exact to 14 digits vs analytic `99830.84236856436 mm³`); bbox=`[-60,-40,0]..[60,40,12]`; 35 faces |
| 1c | SolidCraft: verified all 14 cylindrical surfaces via `body_topology`, checked timeline (`inspect_design`, 0 errors/warnings), saved `.sc` and exported `.step` | OK | 4×R6 outer (Z=0..11.5), 4×R4 pocket (Z=6..12), 1×R8 island (Z=6..12), 4×R3.3 thru at `(±50,±30)`, 1×R2.5×10 at `(0,0)` Z=2..12; saved `q01-2d-plate.sc` (106827 B) & `q01-2d-plate.step` (71870 B) |
| 2a | CamCraft: tested opening/importing `q01-2d-plate.sc` directly (`import_model` and `open_project`) | WRONG | `import_model(.sc)` succeeded (5402 facets, bounds `[-60,-40,0]..[60,40,12]`, note `"the cached final body only"`); `open_project(.sc)` failed with raw error `"open_project: I/O error: stream did not contain valid UTF-8"` |
| 2b | CamCraft: imported `q01-2d-plate.step`, loaded `Tormach 770MX`, set material `aluminium_6061`, fitted stock (+3 mm XY, +1 mm +Z), created `Setup1`, placed WCS G54 at stock top front-left, set post `Tormach PathPilot` | OK | STEP 5412 facets; stock box `126×86×13 mm` (`[-63,-43,0]..[63,43,13]`); WCS G54 origin `[-63,-43,13]`; post=`Tormach PathPilot (built-in)` |
| 3 | CamCraft: inspected `list_tool_catalog` and added 7 metric tools (`EM-1000-4FL`, `EM-0600-4FL`, `SPOT-0800-90`, `DR-0500`, `DR-0660`, `TAP-M6`, `CM-1000-90`) | WRONG | Catalog had T1 `EM-1000-4FL`, T2 `EM-0600-4FL`, T6 `TAP-M6`, T7 `CM-1000-90`, but lacked spot drill, 5.0 mm drill, and 6.6 mm drill (only 2/3/4/6 mm drills exist); custom-defined T3/T4/T5 |
| 4a | CamCraft: created 8 ops (Face, 2D Pocket via `detect_pockets`, 4×Drill ops + `detect_holes`, 2D Contour w/ 4 tabs, 2D Chamfer) | WRONG | `detect_holes` silently added all 5 holes to `Drill_6_6_Thru` (incl. Ø5.0 hole), `Spot_All_5` (full 10–12 mm depth), and `Tap_M6` (incl. 4×Ø6.6 holes) with `skipped: []`, `undersize: []`; `list_features` reported truncated closed-chain lengths (`321.695` vs `389.70 mm` on Edge 9) |
| 4b | CamCraft: manually filtered `holes[]` & depths on all 4 drill ops, set pocket `stepdown: 3.0`, chamfer `feed: 600` | OK | Spot 2 mm deep (`floor_z: 10`), 6.6 mm 4× thru (+1.5 mm breakthrough), 5.0 mm 1× blind 10 mm (`floor_z: 2`), M6 tap 1× 8 mm (`floor_z: 4`) |
| 5a | CamCraft: `generate` all 8 operations | OK | All 8 ops `status: "ok"`; `Drill_6_6_Thru` breakthrough = `3.483 mm` past bottom (`1.983 mm` 118° tip + `1.5 mm`) |
| 5b | CamCraft: `verify` and `render_stock` (`iso`, `top`) | REFUSED | `verify` `ok: false`, `held_by: ["Spot_All_5"]`: 5× `"move N cuts past the finish surface"` (`blocks_post: true`), contradicting `render_stock` (`gouge: {cells: 0, volume: 0.0}`, `rest: 19.67 mm³` at blind-hole cone tip); island untouched |
| 5c | CamCraft: attempted `post` (refused), saved `repro-spot-gouge.camproj`, tested `SPOT-0800-90` diameter `4.5 mm` | REFUSED | `post` refused: `"post: post blocked - 5 warning(s): Spot_All_5: move 1 cuts past the finish surface; ..."`; still flagged when `holes[].radius` was `2.0` |
| 5d | CamCraft: restored `SPOT-0800-90` Ø8.0 and model `holes[].radius` (`3.3`/`2.5`), regenerated, verified; saved `repro-spot-center-gouge.camproj` | REFUSED | 4×Ø6.6 holes passed, center Ø5.0 hole still failed: `"move 11 cuts past the finish surface"` (`blocks_post: true`) despite 90° cone diameter = 4.0 mm at Z=12 |
| 5e | CamCraft: tested Ø4.5 workaround (`verify` passed w/ 5 non-blocking `drill_undersize` warnings), posted & inspected all 2352 lines of G-code | WRONG | Posted `q01-2d-plate.nc` (2352 lines); G-code inspection found: `G0 Z-1` + `R-1` (0 mm clearance) on all drill/tap cycles, `G73` instead of `G83`, wasted `Z-1` air passes in pocket/contour, vertical plunges in helical pocket, `Z10` tab hops, chamfer rubbing on zero-SFM tip |
| 5f | CamCraft: reopened `q01-2d-plate.camproj`, tested `cycle: "peck"` (still `G73`), `cycle: "spot"` w/ Ø8.0 spot drill (`floor_z: 10.5` passed!), and `Edge 12` on chamfer | WRONG | `cycle: "peck"` still emits `G73` (not `G83`); `Spot_All_5` with Ø8.0 90° spot passes at `floor_z: 10.5` (cone radius = 2.5 mm at stock top Z=13) proving false-positive check evaluates cone at Z=13; picking modeled chamfer top `Edge 12` gouged 0.50 mm (caught by `post_gate`) |
| 5g | CamCraft: restored `Edge 9` on chamfer, kept real Ø8.0 `SPOT-0800-90` (`cycle: "spot"`, center `floor_z: 10.5`), regenerated, verified, rendered, posted & saved final files | OK | `verify` `ok: true` (`findings: []`, 0 warnings); `render_stock` `gouge: 0.0 mm³`, `rest: 19.67 mm³` (blind drill tip); posted `q01-2d-plate.nc` (2352 lines, 8 ops) & saved `q01-2d-plate.camproj` |

---

## Problems Found

### Problem 1 — `detect_holes` silently includes oversize and mismatched holes with no warning or skip
- **Program:** CamCraft (v0.25.0)
- **Severity:** High — could gouge/scrap the part (e.g., drilling an M6 tap hole to Ø6.6 mm or plunging an Ø8.0 spot drill 12 mm through the plate).
- **Minimal reproduction:**
  1. `import_model {"name": "q01-2d-plate", "path": "/home/mikekinney/qa/runs/20261122-q01-2d-plate/q01-2d-plate.step"}`
  2. `add_tool {"name": "DR-0660", "definition": {"kind": "drill", "diameter": 6.6, "flute_len": 35, "shank_d": 6.6, "shank_len": 40, "tip_angle": 118, "rpm": 3200, "feed": 250, "flutes": 2, "corner_r": 0, "holder_d": 44, "holder_len": 35, "nose_radius": 0, "pitch": 0, "width": 0, "material_group": "aluminum", "tool_material": "carbide"}}`
  3. `add_tool {"catalog": "TAP-M6"}`
  4. `add_operation {"kind": "drill", "name": "Drill_6_6_Thru", "source": "q01-2d-plate", "tool": "DR-0660"}`
  5. `add_operation {"kind": "drill", "name": "Tap_M6", "source": "q01-2d-plate", "tool": "TAP-M6"}`
  6. `detect_holes {"name": "Drill_6_6_Thru"}` and `detect_holes {"name": "Tap_M6"}`
- **Expected:** `detect_holes` should either filter holes matching the mounted tool's diameter (or tap drill range for a `Tap` tool) or warn/skip holes where the drill is **oversize** (`Ø6.6` drill on `Ø5.0` hole; `Ø8.0` spot drill on `Ø5.0`/`Ø6.6` holes) or where a tap is **undersize** for the bore (`TAP-M6` on `Ø6.6` through holes).
- **Actual:** Both calls returned `{"holes": 5, "radial_holes": 0, "skipped": [], "status": "stale", "undersize": []}`, silently populating all 5 holes into both operations with zero warnings. (By contrast, `detect_holes` on `DR-0500` did report 4 `undersize` messages for the 6.6 mm holes.)
- **Reproducing files:** `q01-2d-plate.step`

### Problem 2 — False-positive drill gouge on valid conical spot drill blocks posting, contradicting `render_stock`
- **Program:** CamCraft (v0.25.0)
- **Severity:** High — blocks posting of a valid program (`post` refused) and forces a workaround.
- **Minimal reproduction:**
  1. `open_project {"path": "/home/mikekinney/qa/runs/20261122-q01-2d-plate/repro-spot-center-gouge.camproj"}`
  2. `verify {}`
  3. `render_stock {"view": "iso"}`
  4. `post {"output": "/home/mikekinney/qa/runs/20261122-q01-2d-plate/q01-2d-plate.nc", "post": "Tormach PathPilot"}`
- **Expected:** `SPOT-0800-90` (`diameter: 8.0`, `tip_angle: 90.0`) spotting `(0, 0)` from `entry_z: 12.0` to `floor_z: 10.0` (2.0 mm deep) cuts a Ø4.0 mm cone at the model top `Z=12.0` — strictly inside the Ø5.0 mm bore (`radius: 2.5`). Both `verify` and `render_stock` should pass cleanly.
- **Actual:**
  - `render_stock` confirms zero gouge: `"gouge": {"cells": 0, "max": 0.0, "max_at": null, "volume": 0.0}`.
  - Yet `verify` reports `"ok": false`, `"held_by": ["Spot_All_5"]`, `"findings": [{"blocks_post": true, "kind": "gouge", "message": "move 11 cuts past the finish surface", "move": 11, "op": "Spot_All_5"}]` (both with `cycle: "auto"` and `cycle: "spot"`).
  - `post` refuses (in `repro-spot-gouge.camproj` with all 5 holes): `"post: post blocked - 5 warning(s): Spot_All_5: move 1 cuts past the finish surface; Spot_All_5: move 6 cuts past the finish surface; Spot_All_5: move 11 cuts past the finish surface; Spot_All_5: move 16 cuts past the finish surface; Spot_All_5: move 21 cuts past the finish surface"`.
  - Raising `floor_z` on `(0,0)` from `10.0` to `10.5` (where the 90° cone radius equals `2.5 mm` at `Z=13.0`, the **stock top** 1 mm above the already-faced model top!) clears the error — showing the drill gouge check appears to evaluate the cone radius at stock top (`Z=13`) instead of model top (`Z=12`).
- **Reproducing files:** `repro-spot-center-gouge.camproj`, `repro-spot-gouge.camproj`

### Problem 3 — Drilling and rigid-tapping cycles rapid to the workpiece surface (`G0 Z-1`) with zero R-plane clearance (`R-1`)
- **Program:** CamCraft (v0.25.0)
- **Severity:** High — collision / tool breakage risk (especially for `G84` rigid tapping, where 0.0 mm approach distance above the top surface prevents spindle/Z synchronization and risks snapping the tap or stripping top threads).
- **Minimal reproduction:**
  1. `open_project {"path": "/home/mikekinney/qa/runs/20261122-q01-2d-plate/q01-2d-plate.camproj"}`
  2. `post {"output": "/home/mikekinney/qa/runs/20261122-q01-2d-plate/q01-2d-plate.nc", "post": "Tormach PathPilot"}`
  3. Inspect lines 1970–2074 of `q01-2d-plate.nc`.
- **Expected:** Drills and taps should rapid to a safe R-plane above the part top (e.g., `R+1` to `R+3` above `Z-1`, i.e., `Z0`..`Z+2` in WCS, especially `>= 3 mm` for `G84` tapping) and `drill` should provide a `retract_z` / `r_plane` parameter.
- **Actual:** Every hole in all 4 drilling/tapping ops rapids directly onto the machined top surface `Z-1` and sets `R-1`:
  - `Spot_All_5` (lines 1971–1973): `G0 X13 Y13 Z-1` / `G99` / `G81 X13 Y13 Z-3 R-1 F200`
  - `Drill_6_6_Thru` (lines 2014–2016): `G0 X13 Y13 Z-1` / `G99` / `G73 X13 Y13 Z-16.483 R-1 Q3 F250`
  - `Drill_5_0_Blind` (lines 2052–2054): `G0 X63 Y43 Z-1` / `G99` / `G73 X63 Y43 Z-11 R-1 Q3 F250`
  - `Tap_M6` (lines 2071–2073): `G0 X63 Y43 Z-1` / `G99` / `G84 X63 Y43 Z-9 R-1 F600`
  - `list_operation_types {"section": "Drilling"}` confirms `drill` has only `clearance_z` and `top_z`, with no `retract_z` / `r_plane` field.
- **Reproducing files:** `q01-2d-plate.camproj`, `q01-2d-plate.nc` (lines 1970–2074)

### Problem 4 — `cycle: "peck"` posts `G73` (chip-break) instead of `G83` (full-retract peck) with no way to emit `G83`
- **Program:** CamCraft (v0.25.0)
- **Severity:** Medium — posts wrong/unexpected canned cycle (`G73` partial retract instead of `G83` full-retract peck, risking chip packing in gummy 6061 aluminium).
- **Minimal reproduction:**
  1. `open_project {"path": "/home/mikekinney/qa/runs/20261122-q01-2d-plate/q01-2d-plate.camproj"}`
  2. `edit_operation {"name": "Drill_6_6_Thru", "params": {"cycle": "peck", "peck": 3.0}}`
  3. `generate {}` and `post {"output": "/home/mikekinney/qa/runs/20261122-q01-2d-plate/q01-2d-plate.nc", "post": "Tormach PathPilot"}`
  4. Inspect lines 2016–2034 and 2054 of `q01-2d-plate.nc`.
- **Expected:** Full-retract peck drilling (`G83 X... Y... Z-16.483 R... Q3 F250`), or distinct `chip_break` (`G73`) vs `deep_peck` (`G83`) cycle variants.
- **Actual:** Both `cycle: "auto"` and `cycle: "peck"` with `peck: 3.0` emit `G73 X13 Y13 Z-16.483 R-1 Q3 F250` (and `G73 X63 Y43 Z-11 R-1 Q3 F250`). Probing an invalid cycle confirmed the full enum is `auto`, `spot`, `drill`, `peck`, `tap`, `bore`, `back_bore`, `floating_tap`, `ream`, `fine_bore` — there is no separate deep-peck (`G83`) variant.
- **Reproducing files:** `q01-2d-plate.camproj`, `q01-2d-plate.nc` (lines 2016–2034, 2054)

### Problem 5 — `2D Chamfer` has no `tip_offset` (cuts on zero-SFM tip point) and plunges 26.5 mm from clearance Z directly onto the part edge
- **Program:** CamCraft (v0.25.0)
- **Severity:** Medium — poor machining practice / tool chipping & poor surface finish (zero cutting speed at the sharp tip) and wasted feed time plunging through air.
- **Minimal reproduction:**
  1. `open_project {"path": "/home/mikekinney/qa/runs/20261122-q01-2d-plate/q01-2d-plate.camproj"}`
  2. `post {"output": "/home/mikekinney/qa/runs/20261122-q01-2d-plate/q01-2d-plate.nc", "post": "Tormach PathPilot"}`
  3. Inspect lines 2212–2348 of `q01-2d-plate.nc`.
- **Expected:** `2D Chamfer` should support a `tip_offset` (e.g., 1.0–2.0 mm outward & deeper so the 90° flutes cut above the dead point), a safe feed/retract height above `top_z`, and a tangential lead-in/lead-out.
- **Actual:** `Chamfer_Outside_Top` plunges straight from `Z25` to `Z-1.5` at `F300` directly on the edge (`G0 X117.267 Y3.006 Z25` → `G1 X117.267 Y3.006 Z-1.5 F300`), then traces `X=3..123, Y=3..83` at `Z-1.5` — running the `r=0` sharp tip point along the bottom corner of the 0.5 mm chamfer with zero lead-in/lead-out.
- **Reproducing files:** `q01-2d-plate.camproj`, `q01-2d-plate.nc` (lines 2212–2348)

### Problem 6 — Toolpath defects in `2D Pocket` and `2D Contour` (zero-depth `Z-1` air passes, vertical plunges ignoring `entry: "helix"`, duplicate `G1` moves, full-height `Z10` tab hops)
- **Program:** CamCraft (v0.25.0)
- **Severity:** Medium — inefficient G-code and vertical plunges into solid aluminium despite `entry: "helix"` (risk of end-mill breakage on non-centre-cutting tools or hard materials).
- **Minimal reproduction:**
  1. `open_project {"path": "/home/mikekinney/qa/runs/20261122-q01-2d-plate/q01-2d-plate.camproj"}`
  2. `post {"output": "/home/mikekinney/qa/runs/20261122-q01-2d-plate/q01-2d-plate.nc", "post": "Tormach PathPilot"}`
  3. Inspect `Pocket_Center` (lines 61–1956) and `Contour_Outside` (lines 2089–2198) in `q01-2d-plate.nc`.
- **Expected:**
  - First cutting level should start at `top_z - stepdown` (`Z-4`), not waste a full pass at `top_z` (`Z-1`, 0 mm depth).
  - `entry: "helix"` should helically ramp every entry into uncut material (and not rapid down to `Z-4` touching the floor at line 1313) without duplicate zero-length `G1` lines.
  - Contour tab hops should step up to tab top (`Z-12`), cross the tab, and step back to `Z-13` rather than retracting 23 mm to `Z10` and plunging 23 mm through air.
- **Actual:**
  - Both `Pocket_Center` (lines 65–666) and `Contour_Outside` (lines 2093–2105) execute full passes at `Z-1` (0 mm depth).
  - `Pocket_Center` plunges straight vertically 3 mm into solid material at line 682 (`G1 X63.637 Y32.01 Z-4 F300`) and line 1327 (`G1 X63.637 Y32.01 Z-7 F300`) despite `entry: "helix"`, rapids to `Z-4` touching the cut floor at line 1313, and emits dozens of consecutive duplicate `G1` lines (e.g., lines 196–197, 252–253, 841–842).
  - `Contour_Outside` tab hops (lines 2174–2176, 2182–2184, 2190–2192) feed up 23 mm to `Z10`, rapid 13 mm across, and plunge 23 mm at `F300` back to `Z-13`. (Note: physical tab geometry is accurate — 3.0 mm wide and 1.0 mm tall.)
- **Reproducing files:** `q01-2d-plate.camproj`, `q01-2d-plate.nc` (lines 65–1328, 2093–2198)

### Problem 7 — `list_features` reports truncated `length` for closed edge chains (omits closing segment)
- **Program:** CamCraft (v0.25.0)
- **Severity:** Low — confuses users/agents selecting edge chains by perimeter length (actual toolpaths still close the loop properly).
- **Minimal reproduction:**
  1. `import_model {"name": "q01-2d-plate", "path": "/home/mikekinney/qa/runs/20261122-q01-2d-plate/q01-2d-plate.step"}`
  2. `list_features {"model": "q01-2d-plate", "kinds": ["chains"], "compact": false}`
- **Expected:** Closed chain `length` should include all segments of the closed loop (e.g., `Edge 9` at `[-60,-40,11.5]..[60,40,11.5]` perimeter ≈ `389.70 mm`; `Edge 12` at `[-59.5,-39.5,12]..[59.5,39.5,12]` ≈ `386.56 mm`; `Edge 8` pocket loop `[-30,-20,6]..[30,20,6]` ≈ `193.13 mm`).
- **Actual:** `list_features` omits the closing edge between `end` and `start`:
  - `Edge 9` (`chain:0x60e930a9e55bf92d`, `closed: true`, `start: [60,-34,11.5]`, `end: [60,34,11.5]`): reports `length: 321.695` (short by `68.0 mm`).
  - `Edge 12` (`chain:0x27cd60db24bf8cca`, `closed: true`, `start: [59.5,-34,12]`, `end: [59.5,34,12]`): reports `length: 318.554` (short by `68.0 mm`).
  - `Edge 8` (`chain:0x4debf4c1532fe7b7`, `closed: true`, `start: [26,-20,6]`, `end: [-26,-20,6]`): reports `length: 141.127` (short by `52.0 mm`).
- **Reproducing files:** `q01-2d-plate.step`

### Problem 8 — Built-in tool catalog lacks spot drills and standard M6 drills (5.0 mm tap drill, 6.6 mm clearance drill)
- **Program:** CamCraft (v0.25.0)
- **Severity:** Low — usability gap requiring manual `add_tool` custom definitions for basic metric hole-making.
- **Minimal reproduction:**
  1. `list_tool_catalog {}`
- **Expected:** A catalog that includes `TAP-M6` should also include a spot drill, a 5.0 mm tap drill, and a 6.6 mm M6 clearance drill.
- **Actual:** Catalog only contains four twist drills (`DR-0200`, `DR-0300`, `DR-0400`, `DR-0600`) and zero spot drills (`CM-*-90` are `kind: "taper"`, which `Drill` ops do not accept).
- **Reproducing files:** `q01-2d-plate.camproj` (contains custom `SPOT-0800-90`, `DR-0500`, `DR-0660`)

### Problem 9 — `open_project` on a SolidCraft `.sc` file returns raw UTF-8 I/O error
- **Program:** CamCraft (v0.25.0)
- **Severity:** Low — confuses users trying to open a SolidCraft file directly instead of using `import_model`.
- **Minimal reproduction:**
  1. `open_project {"path": "/home/mikekinney/qa/runs/20261122-q01-2d-plate/q01-2d-plate.sc"}`
- **Expected:** A clear message explaining `.sc` is a SolidCraft model file (not a `.camproj` project) and pointing the user to `import_model`. (Note: `import_model` DOES support `.sc` directly and worked cleanly!)
- **Actual:** Fails with raw error: `"open_project: I/O error: stream did not contain valid UTF-8"`.
- **Reproducing files:** `q01-2d-plate.sc`

---

## Confusing Tool Names, Defaults & Parameters (Worked, but Worth Noting)

- **SolidCraft:**
  - `describe_command` takes parameter `id`, whereas `execute` takes `command` (easy to mix up).
  - `body_topology {"faces": "cylinders"}` reported the 4 patterned through holes with arbitrary parametric origin `z = -278.44084` and `extent = [278.44084, 290.44084]` (geometrically exact `Z=0..12`, just surprising to read).
- **CamCraft:**
  - `Face` defaults `cut_z` to `{"reference": "top_of_stock", "offset": 0.0}`, which skims air at 0 mm depth unless changed to `top_of_model`.
  - `list_operation_types` lists enum fields like `drill.cycle`, `pocket.strategy`, `contour.entry`, `contour.side`, and `chamfer2_d.side` simply as `"type": "string"` without enumerating valid values (we only learned `cycle: "spot"` existed by sending an invalid string to read the error message).
  - `add_operation` seeds milling ops with a hard-coded `feed: 1200.0` instead of the catalog tool's `feed` unless `cutting_data` rows exist; meanwhile `suggest_cutting_data` refuses drills and taps (`"suggest_cutting_data: the cutting-data table does not speak for a drill"` / `"a tap"`).
  - `2D Chamfer` on a CAD model that already models the chamfer requires selecting the **lower** outer edge of the chamfer (`Edge 9` at `Z=11.5`, outer dimensions `[-60,60]×[-40,40]`) rather than the top edge (`Edge 12` at `Z=12.0`, `[-59.5,59.5]×[-39.5,39.5]`); selecting `Edge 12` gouges 0.50 mm around the perimeter (which `post_gate` did catch and block cleanly).
  - `detect_pockets` listed the Ø5.0 blind hole as pocket `index: 0` (area 19.60 mm²) before the actual 60×40 pocket at `index: 1`.
  - `get_post_options` takes `setup` (not `post`).
  - **What worked particularly well:** SolidCraft's parametric modeling, exact B-rep volumes, and `.sc`/STEP export; CamCraft's direct `.sc` import, `Tormach 770MX` machine definition (including belt-range comments in G-code), `detect_pockets` recognising the pocket with its Ø16 island and keeping the island standing cleanly, `Drill_6_6_Thru` tip + breakthrough calculation (`3.483 mm`), `Tap_M6` pitch/feed math (`S600` × `1.0 mm` = `F600`), exact `Contour_Outside` arc/XY offsets (`G2 R11`) and physical tab sizes (`3.0 × 1.0 mm`), `post_gate` catching the `Edge 12` chamfer gouge, and `.camproj` save/reopen fidelity.

RESULT: 9 problems (9 CamCraft, 0 SolidCraft); end-to-end: posted
