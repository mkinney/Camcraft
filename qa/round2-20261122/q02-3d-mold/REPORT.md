# QA Report: q02-3d-mold (SolidCraft v0.46.2 & CamCraft v0.25.0)

**Workspace:** `/home/mikekinney/qa/runs/20261122-q02-3d-mold`  
**Part:** Aluminium 6061 mould core block, 80 × 60 × 30 mm (18 mm base block with R5 vertical corners, 50 × 36 × 12 mm ellipsoidal dome boss on top with R2 base fillet, 6 mm wide × 3 mm deep R3 ball-bottom runner channel curving around the boss from `[0, 17, 18]` to the `+X` short edge `[40, 0, 18]`).  
**Dome construction tried in SolidCraft:**
1. **`solid.sphere` + non-uniform `solid.scale` + `solid.combine(join)`** — succeeded cleanly with exact analytic volume (`22619.47 mm³` full ellipsoid, `+11309.73 mm³` upper half-ellipsoid, `97323.45 mm³` joined), and accepted the R2 `solid.fillet` (`+81.89 mm³`). Used for `q02-3d-mold.sc` and `q02-3d-mold.step`.
2. **`solid.loft` (diagnostic test)** — 4-section loft (3 ellipses + apex point) was refused (`not supported yet: a point section in a loft of more than two`); a 3-ellipse flat-topped loft succeeded (`107.6 ms`).

---

## Step Summary Table

| Step | What you did | Result | Exact message or numbers |
|---|---|---|---|
| 1a | SolidCraft `new_design`, 9 parameters, `solid.box` (80×60×18 centred at `[0,0,9]`), `solid.fillet` R5 on 4 vertical corners, `measure` | OK | `vol=86013.7167 mm³` (expected `86013.7167 mm³`), `bbox=[-40,-30,0]..[40,30,18]`, 10 faces, `closed=true` |
| 1b | SolidCraft `solid.sphere` (R10 at `[0,0,18]`) + `solid.scale` (`factors=["boss_x/(20 mm)","boss_y/(20 mm)","H_dome/(10 mm)"]`), `measure` | OK | Ellipsoid `vol=22619.4671 mm³` (exact `4/3*pi*25*18*12`), `bbox=[-25,-18,6]..[25,18,30]`, 1 face |
| 1c | SolidCraft `solid.combine` (join `Dome` into `MoldCore`) + `solid.fillet` R2 at `[25,0,18]`, `measure` | OK | After join: `vol=97323.4502 mm³` (exact). After R2 fillet (`29.69 s`): `vol=97405.3377 mm³` (`+81.89 mm³`, expected `~80 mm³`), 12 faces, `closed=true` |
| 1d | SolidCraft `RunnerPath` sketch on `XY` offset `H_base` (7 tangent lines/arcs from `[0,17]` around boss to `[42,0]`) + `solid.pipe` (`diameter="runner_d"`, `operation="cut"`), `measure` | OK | Took `19.52 s`; `vol=96487.4177 mm³` (`-917.92 mm³` vs `~915 mm³` expected), `bbox=[-40,-30,0]..[40,30,30]`, COM `[-0.206,-0.168,10.516]`, 20 faces (`kernel.faces=22`), `closed=true` |
| 1e | SolidCraft `inspect_design`, `screenshot` (iso & top), `save` (`q02-3d-mold.sc`), `export` (`q02-3d-mold.step`) | OK | 0 timeline errors/warnings; `q02-3d-mold.sc` (`397098 B`), `q02-3d-mold.step` (`366048 B`, 22 `ADVANCED_FACE`s) |
| 1f | SolidCraft diagnostics: `check_parameters` (`L="90 mm"`), `solid.loft` (3 ellipses + apex point), inspect STEP `#1828` | OK / REFUSED | `check_parameters`: `vol=107259.14 mm³` (`+10771.73 mm³` exact, `91 s`, 0 errors); `solid.loft` apex refused (`"not supported yet: a point section in a loft of more than two"`); STEP `#1828` verified exact rational 2×2 NURBS ellipsoid |
| 2a | CamCraft `import_model` (`q02-3d-mold.step`, also `facet_budget: 60000000`), `import_model` (`q02-3d-mold.sc`), `load_machine` (`Tormach 770MX`), `set_work_material` (`aluminium_6061`) | REFUSED | STEP refused: `"import_model: corrupt STEP: solid #4056 'MoldCore' could not be tessellated: machining mesh: face Some(10) stands 5.089e-1 mm off its surface, tolerance 5e-3 mm - re-export the model, or import an STL"`; `.sc` refused: `"import_model: Solidcraft design refused: machining mesh: face Some(10) stands 5.089e-1 mm off its surface, tolerance 5e-3 mm"`; machine & material OK |
| 2b | Workaround: SolidCraft `export` (`q02-3d-mold.stl`), CamCraft `import_model` (`MoldCore_STL`), `fit_stock` (`sides=[2,2,2,2,0,2]`), `set_wcs` (top centre) | OK | STL export emitted 3 angle warnings (faces 10, 16, 17); CamCraft imported `28692` facets, `bbox=[-40,-30,0]..[40,30,30]`; stock `84×64×32` (`[-42,-32,0]..[42,32,32]`); WCS G54 at `[0,0,32]` |
| 3 | CamCraft `add_setup` (`Setup1`), `set_setup_post` (`Tormach PathPilot`), `set_work_offset` (`1`), `set_stock_material` (`aluminium_6061`), `add_tool` ×4 | OK | T1 `EM-1200-2FL`, T2 `EM-0600-4FL`, T3 `BN-0600`, T4 `BN-0300` (from `Router ball nose 3 mm` — no Al 3 mm ball or 6 mm bull nose in catalog); all got `BT30 ER20` |
| 4a | CamCraft `add_operation` ×5 (`Op1_AdaptiveRough`, `Op2_AdaptiveRest`, `Op3_Waterline`, `Op4_ScallopDome`, `Op5_PencilRest`) + `apply_cutting_data` ×5 | OK | All 5 RPMs clamped to 10,000 RPM (`Tormach 770MX` max); feeds: Op1/Op2 `2121 mm/min` (with `0.7071` chip-thinning factor), Op3/Op4 `750 mm/min`, Op5 `375 mm/min` |
| 5a | CamCraft `generate` (`Op1_AdaptiveRough`) | REFUSED | `"generate: Op1_AdaptiveRough - could not generate the toolpath: Op1_AdaptiveRough: invalid geometry: loop crosses itself"` |
| 5b | CamCraft `generate` (`Op3_Waterline`, `Op4_ScallopDome`, `Op5_PencilRest`) | STUCK | `"MCP tool call to server \"camcraft\" timed out after 3m0s: context deadline exceeded"` |
| 5c–5e | CamCraft `get_job` ×3 to check server responsiveness (12 min, 15.5 min, 21+ min after 5b) | STUCK | All 3 calls timed out after `3m0s` (`context deadline exceeded`); `camcraft` server permanently hung. Could not run `verify`, `render_stock`, `post`, or `save_project` (`q02-3d-mold.camproj` / `q02-3d-mold.nc` not written). |

---

## Problems Found

### Problem 1 (CamCraft): STEP and native SolidCraft `.sc` import both refuse a valid non-uniformly scaled sphere (ellipsoid) with a 0.509 mm tessellation error
- **Program:** CamCraft (v0.25.0)
- **Reproducer files:**
  - `/home/mikekinney/qa/runs/20261122-q02-3d-mold/q02-3d-mold.step`
  - `/home/mikekinney/qa/runs/20261122-q02-3d-mold/q02-3d-mold.sc`
- **Smallest steps to reproduce:**
  1. `camcraft:import_model {"path": "/home/mikekinney/qa/runs/20261122-q02-3d-mold/q02-3d-mold.step", "name": "MoldCore_STEP"}`
  2. `camcraft:import_model {"path": "/home/mikekinney/qa/runs/20261122-q02-3d-mold/q02-3d-mold.step", "name": "MoldCore_STEP_hi", "facet_budget": 60000000}`
  3. `camcraft:import_model {"path": "/home/mikekinney/qa/runs/20261122-q02-3d-mold/q02-3d-mold.sc", "name": "MoldCore_SC"}`
- **Expected:** Both the STEP file and SolidCraft `.sc` file import cleanly as B-rep models (face 10 in `q02-3d-mold.step`, `#3699` / `#1828`, is a mathematically exact 9×5 degree-(2,2) rational NURBS ellipsoid with control points `#1801`–`#1827` from `Z=6` to `Z=30` and standard sphere weights `1, sqrt(2)/2, 0.5`).
- **Actual:** Both STEP (even at max `facet_budget: 60000000`) and `.sc` are refused with the identical 0.5089 mm surface deviation on face 10 (`F3:face:0`, the ellipsoid dome):
  - STEP: `"import_model: corrupt STEP: solid #4056 'MoldCore' could not be tessellated: machining mesh: face Some(10) stands 5.089e-1 mm off its surface, tolerance 5e-3 mm - re-export the model, or import an STL"`
  - `.sc`: `"import_model: Solidcraft design refused: machining mesh: face Some(10) stands 5.089e-1 mm off its surface, tolerance 5e-3 mm"`
- **Severity:** High — completely blocks native `.sc` and STEP B-rep workflows on parts containing ellipsoidal / scaled-sphere NURBS surfaces, forcing lossy STL workarounds (which lose exact face/edge geometry and then triggered Problems 2 and 3). If the tolerance check had not caught it, a 0.509 mm surface error would gouge or leave 0.5 mm error on a mould surface.

---

### Problem 2 (CamCraft): 3D Adaptive roughing fails with `invalid geometry: loop crosses itself` on the imported STL
- **Program:** CamCraft (v0.25.0)
- **Reproducer files:** `/home/mikekinney/qa/runs/20261122-q02-3d-mold/q02-3d-mold.stl`
- **Smallest steps to reproduce:**
  1. `camcraft:import_model {"path": "/home/mikekinney/qa/runs/20261122-q02-3d-mold/q02-3d-mold.stl", "name": "MoldCore_STL"}`
  2. `camcraft:fit_stock {"sides": [2, 2, 2, 2, 0, 2]}`
  3. `camcraft:add_tool {"catalog": "EM-1200-2FL"}`
  4. `camcraft:add_operation {"kind": "pocket3d", "name": "Op1_AdaptiveRough", "source": "MoldCore_STL", "tool": "EM-1200-2FL", "params": {"strategy": "adaptive", "stock_to_leave": 0.3, "stepdown": 3, "top_z": {"reference": "top_of_stock", "offset": 0}, "bottom_z": {"reference": "bottom_of_model", "offset": 0}}}`
  5. `camcraft:generate {"name": "Op1_AdaptiveRough"}`
- **Expected:** Generates 3D adaptive roughing toolpath around the dome, runner, and outside perimeter leaving 0.3 mm stock.
- **Actual:** Refused with:
  `"generate: Op1_AdaptiveRough - could not generate the toolpath: Op1_AdaptiveRough: invalid geometry: loop crosses itself"`
- **Severity:** High — blocks 3D roughing of the mould core with no diagnostic information (no Z height, no offending triangle/slice loop, no suggestion to repair or change stepdown).

---

### Problem 3 (CamCraft): `generate` on 3D Finish (`finish3d`) permanently hangs the `camcraft` MCP server
- **Program:** CamCraft (v0.25.0)
- **Reproducer files:** `/home/mikekinney/qa/runs/20261122-q02-3d-mold/q02-3d-mold.stl`
- **Smallest steps to reproduce:**
  1. Import `q02-3d-mold.stl` (`28692` facets), `fit_stock {"sides":[2,2,2,2,0,2]}`, `load_machine {"builtin":"Tormach 770MX"}`, `add_tool {"catalog":"BN-0600"}`, `add_tool {"catalog":"Router ball nose 3 mm","name":"BN-0300"}`.
  2. `camcraft:add_operation {"kind":"finish3d","name":"Op3_Waterline","source":"MoldCore_STL","tool":"BN-0600","params":{"pattern":"waterline","top_z":{"reference":"top_of_model","offset":0},"bottom_z":{"reference":"bottom_of_model","offset":0}}}`
  3. (Also queued: `Op4_ScallopDome` with `pattern:"scallop"` and `Op5_PencilRest` with `pattern:"pencil"`.)
  4. `camcraft:generate {"name":"Op3_Waterline"}` (plus `Op4_ScallopDome`, `Op5_PencilRest`).
  5. `camcraft:get_job {"summary": true}` (tested 3 times over 21+ minutes).
- **Expected:** Toolpaths generate in reasonable time (or return an error/progress/cancellation cleanly) and the MCP server remains responsive.
- **Actual:** Every call timed out after 3 minutes with:
  `"MCP tool call to server \"camcraft\" timed out after 3m0s: context deadline exceeded"`
  The `camcraft` MCP server remained completely wedged for >21 minutes (06:20:05 to 06:41:09), blocking even trivial read-only `get_job` calls.
- **Severity:** Critical (hang / data loss) — permanently freezes the CAM backend with no cancellation mechanism; all unsaved job setup, tools, and operations in the session were lost and `verify`, `render_stock`, `save_project`, and `post` could not be reached.

---

### Problem 4 (CamCraft): Built-in tool catalog lacks aluminum 3 mm ball nose & 6 mm bull nose, and specifies dangerous oversized 8 mm shanks on 6 mm aluminum tools
- **Program:** CamCraft (v0.25.0)
- **Reproducer files:** None required (built-in catalog).
- **Smallest steps to reproduce:**
  1. `camcraft:list_tool_catalog {}`
- **Expected:** Standard 3D mould tooling for aluminum (e.g., `BN-0300` 3 mm ball nose, 6 mm bull nose), and 6 mm tools with 6 mm shanks (not wider than the cutter diameter unless clearly marked as stepped-shank short-reach tools).
- **Actual:**
  - No aluminum 3 mm ball nose exists; the only 3 mm ball nose is `"Router ball nose 3 mm"` (`material_group: "wood"`, `rpm: 18000` — exceeding the Tormach 770MX 10,000 RPM limit — and `shank_len: 13.0`, giving only 25 mm total projection).
  - No 6 mm bull nose exists (only `BB-1000-R1`, 10 mm).
  - Both 6 mm aluminum cutters (`EM-0600-4FL` and `BN-0600`) have `diameter: 6.0`, `flute_len: 15.0`, **`shank_d: 8.0`**.
- **Severity:** Medium-High (collision risk / user confusion) — an 8 mm shank on a 6 mm cutter with 15 mm flute length will collide/rub against any vertical wall deeper than 15 mm (this part's outer wall is 18 mm deep and total Z depth is 30 mm). Note: `apply_cutting_data` did properly clamp the 3 mm wood router bit's 18,000 RPM down to 10,000 RPM for the 770MX, which is good.

---

### Problem 5 (CamCraft): `list_operation_types` omits all enum options and optional 3D finish parameters, and `finish3d` disallows non-ball tools
- **Program:** CamCraft (v0.25.0)
- **Reproducer files:** None required (built-in schema).
- **Smallest steps to reproduce:**
  1. `camcraft:list_operation_types {"section": "Mill3D"}`
- **Expected:** Clear enum lists for `pocket3d.strategy`, `pocket3d.method`, `pocket3d.entry`, `finish3d.pattern`, and `finish3d.gap_link`; explicit parameter docs for finishing `stepover`, `stepdown`, `scallop_height`, `stock_to_leave`, and rest-machining controls; and `finish3d` (especially waterline/contour finishing) accepting `BullNose` and `EndMill` tools in addition to `BallNose`.
- **Actual:**
  - All string enum parameters are listed only as `"type": "string"` with a single default (e.g. `strategy: "auto"`, `method: "spiral"`, `pattern: "parallel"`), giving no indication that `"adaptive"`, `"waterline"`, `"scallop"`, or `"pencil"` exist or what exact strings are valid.
  - `finish3d` lists zero stepover/stepdown/scallop/rest fields, only `"note": "Fields absent from the defaults are optional and off; set them in a params patch to turn them on."` without naming any optional fields.
  - `finish3d.tool_kinds` is strictly `["BallNose"]`, refusing flat or bull-nose end mills for waterline wall finishing.
- **Severity:** Medium (confuses users / forces guessing parameter names and prevents standard bull-nose waterline finishing).

---

### Problem 6 (SolidCraft): `solid.pipe` boolean cut across an ellipsoid R2 fillet leaves a `0.000664 mm²` sliver face and poor STL mesh triangles (1.24°), and scaled-sphere face 10 also warns on STL export
- **Program:** SolidCraft (v0.46.2)
- **Reproducer files:**
  - `/home/mikekinney/qa/runs/20261122-q02-3d-mold/q02-3d-mold.sc`
  - `/home/mikekinney/qa/runs/20261122-q02-3d-mold/q02-3d-mold.stl`
- **Smallest steps to reproduce:**
  1. Open `/home/mikekinney/qa/runs/20261122-q02-3d-mold/q02-3d-mold.sc`
  2. `solidcraft:body_topology {"body": "MoldCore", "faces": "all"}` (inspect face `index: 17`, `F8:side#[F6:blend(F1:+z|F3:face:0)]`, `area: 0.0006644478463010834`, and edges `4` & `29` of length `0.174 mm`)
  3. `solidcraft:export {"path": "/home/mikekinney/qa/runs/20261122-q02-3d-mold/q02-3d-mold.stl"}`
- **Expected:** Clean boolean trim with no sub-0.001 mm² sliver faces and clean STL tessellation without mesh-quality warnings.
- **Actual:** `export` succeeded (`1434684 B`) but emitted three warnings:
  - `"MoldCore: face 10: smallest angle 20.00°, short of the 25° asked (stopped by: nothing stopped it)"`
  - `"MoldCore: face 16: smallest angle 14.88°, short of the 25° asked (stopped by: Narrow)"`
  - `"MoldCore: face 17: smallest angle 1.24°, short of the 25° asked (stopped by: Narrow, Corner)"`
  Also note `measure` reported `faces: 20` while `kernel.faces: 22` (matching the 22 faces in `body_topology` and `q02-3d-mold.step`).
- **Severity:** Medium — sliver face + 1.24° triangles degrade downstream CAM STL slicing (likely contributing to CamCraft Problem 2), and face 10 (`stopped by: nothing stopped it`) indicates tessellator difficulty on the scaled-sphere ellipsoid.

---

### Problem 7 (SolidCraft): `solid.loft` rejects an apex point section with more than two sections, undocumented in `describe_command`
- **Program:** SolidCraft (v0.46.2)
- **Reproducer files:** Reproducible from an empty design (steps below).
- **Smallest steps to reproduce:**
  1. `solidcraft:new_design {"name": "test-loft"}`
  2. Create 3 elliptical sketches `S0` (Z=18, 25×18), `S1` (Z=24, 21.65×15.59), `S2` (Z=28.5, 12.1×8.71) and a point sketch `S3` (Z=30, `point: [0,0]` -> `p1`).
  3. `solidcraft:execute {"command": "solid.loft", "params": {"name": "LoftDome", "operation": "join", "sections": [{"sketch": "S0"}, {"sketch": "S1"}, {"sketch": "S2"}, {"sketch": "S3", "point": "p1"}]}}`
- **Expected:** Either lofts smoothly through the 3 elliptical sections to the apex point (forming a smooth closed dome), or `describe_command("solid.loft")` documents the 2-section limit for apex points.
- **Actual:** Refused with:
  `"not supported yet: a point section in a loft of more than two"`
  (`describe_command("solid.loft")` says `sections: [{sketch, profiles?} | {sketch, point: id} (an apex)] (2…50, in order; matching hole counts, no holes at an apex)` with no mention of the 2-section cap.)
- **Severity:** Low (confuses user trying to build a smooth multi-section lofted dome to a pole).

---

## Confusing Tool Names, Parameters & UX Notes (Even When They Worked)

1. **CamCraft `pocket3d` has both `method` (`"spiral"`) and `strategy` (`"adaptive"` / `"auto"`):** It is unclear how `method` interacts with `strategy` (setting `strategy: "adaptive"` left `method: "spiral"` in `get_job`).
2. **CamCraft `set_work_offset` vs `add_setup`:** Calling `set_wcs` works on a fresh document with an implicit G54 setup, but calling `set_work_offset {"index": 1}` is refused (`"set_work_offset: no setup is declared - add_setup first (a document without setups posts at work offset 1)"`), and `load_machine` does not automatically assign the machine's recommended post (`Tormach PathPilot`) — it leaves `posts_with: "Fanuc (built-in)"` until you manually `add_setup` and `set_setup_post`.
3. **CamCraft `pocket3d` default `top_z` is `top_of_model`, not `top_of_stock`:** For a part with +2 mm top stock allowance, leaving the default `top_z` would start cutting 2 mm below the top of the raw stock (risking a heavy first pass or plunge into stock).
4. **CamCraft `add_operation` after `set_stock_material`:** Built-in catalog tools lack `cutting_data` rows, so newly added operations still start at hardcoded `feed: 1200.0` until `apply_cutting_data` is explicitly called on each operation.
5. **SolidCraft `solid.scale` unitless expression requirement:** Writing `"boss_x / 20"` is refused (because `boss_x` has mm units); users must write `"boss_x / (20 mm)"`. The error message is helpful (`expected a unit-less number, got a length`), though easy to trip over.
6. **SolidCraft `sketch.line` uses `points: [[x0,y0],[x1,y1]]`** while `sketch.arc.center_point` uses `start: [x,y], end: [x,y]` — inconsistent endpoint parameter naming across sketch curve creators.
7. **SolidCraft feature rebuild latency on NURBS fillet & pipe:** `BossBaseFillet` (R2 around ellipsoid) took `29.7 s`, `RunnerChannel` (`solid.pipe` cut) took `19.5 s`, and `check_parameters` (`L="90 mm"` + restore) took `91 s`.

---

RESULT: 7 problems (5 CamCraft, 2 SolidCraft); end-to-end: not posted
