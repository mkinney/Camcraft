# QA Report — Q03 Turned Pin (SolidCraft v0.46.2 & CamCraft v0.25.0)

**Workspace:** `/home/mikekinney/qa/runs/20261122-q03-turn-shaft`  
**Part:** 70 mm turned pin in 6061 aluminium from Ø32 mm bar (85 mm stick-out): Ø30 × 20 mm head with 1 mm × 45° chamfer at free end (back/chuck side for turning), Ø24 × 30 mm body with O-ring groove (2.5 mm wide × 1.5 mm deep to Ø21, centred 15 mm from head shoulder), M16 × 1.5 external thread (20 mm end comprising 17 mm Ø16 thread + 3 mm wide × Ø13 thread relief groove next to body shoulder).  
**Machine / Post:** `Tormach 15L Slant-PRO` (selected as first preference; available in CamCraft built-in machine catalog, spindle 180–2500 RPM) with matching built-in post `Tormach PathPilot Lathe` (also tested `Fanuc Lathe` for G76 comparison).

---

## Step Summary Table

| Step | What you did | Result | Exact message or numbers |
|---|---|---|---|
| 1. SolidCraft CAD | Created parametric `ShaftProfile` on YZ plane (dof 0), revolved around Z (`TurnShaft`), added 1 mm × 45° `HeadChamfer` at Z=0 and cosmetic `M16x1.5` `solid.thread` on Z=53..70; checked `measure` & `body_topology`; saved `.sc` and exported `.step`. | **OK** | `volume_mm3=31213.9483` (analytical exact `31213.9483`, 0.0000% err), `bbox=[-15,-15,0]..[15,15,70]`, `faces=14` (1 cone, 6 cyl, 7 plane), `closed=true`; `q03-turn-shaft.sc` (25,887 B), `q03-turn-shaft.step` (23,156 B). |
| 2. CamCraft Setup | Imported both `q03-turn-shaft.sc` directly (8,124 facets) and `q03-turn-shaft.step` (8,142 facets); deleted `.sc` copy to keep `shaft_step`; loaded `Tormach 15L Slant-PRO`, set material `aluminium_6061`, added `Setup1` with `Tormach PathPilot Lathe`, moved `shaft_step` to `[0,0,14]` inside Ø32 × 85 mm bar stock (`Z=0..85`, 1 mm face allowance, 14 mm chuck clearance), placed WCS G54 at `[0,0,84]` (finished front face on axis), set `workholding_max_rpm=2500`. | **OK** | Both `.sc` and `.step` imported with identical bounds `[-15,-15,0]..[15,15,70]` and 0 fidelity issues; `stock_bounds=[-16,-16,0]..[16,16,85]`, `shaft_step` at `Z=14..84`, `G54=[0,0,84]`. |
| 3. CamCraft Tools | Added 4 catalog tools: `OD_Turn_R08` (`TN-1600-0R8`), `Groove_2mm` (`GT-0200`), `Thread_60deg` (`TD-0866-60`, then tested `TD-1732-60`), `Parting_3mm` (`GT-0300`). Tested `suggest_cutting_data`. | **WRONG** | `TN-1600-0R8` defaults to `3000 rpm` (> Tormach 15L `2500 rpm` max, later blocks post). All lathe catalog tools omit `tool_material` → `suggest_cutting_data` refused: `"tool OD_Turn_R08 states no tool material (HSS or carbide)"`. Neither `TD-0866-60` nor `TD-1732-60` can cut M16×1.5 (see Problem 7). |
| 4. CamCraft Operations & Generate | Added Face, OD Rough (`stock_to_leave=0.3`), OD Finish, Groove Rough & Finish (initially full model, then split into O-ring `Z=-33.75..-36.25` and Relief `Z=-17..-20`), Thread M16×1.5 (`pitch=1.5, depth=0.92, flank 29.5°, constant_area, 2 spring passes`), Part-Off; also tested `Chamfering` (identical to `OD Finish` chamfer) and CSS parameter probes. | **REFUSED / WRONG** | `TD-0866-60` refused (`reaches 0.750 of the 0.920`); `TD-1732-60` refused (`a 1.732 wide insert does not fit a 1.500 pitch`); workaround `width=1.3` generated. Full-model grooves failed verify (`5.6/5.5 mm rest`); splitting by Z-window cleared them. No CSS parameter exists on ops/tools. |
| 5a. Verify, Simulate & Threading Workaround | Ran `verify`, `render_stock`, and `post`. Tested smooth `shaft_step`, `source: "stock"`, SolidCraft `modeled: true` helical thread (`q03-turn-shaft-modeled-thread.step`), and a revolved-V workaround (`q03-turn-shaft-revolved-v-sharp.step`). | **REFUSED / WRONG** | `render_stock` & stock sim refused ALL turning programs. Smooth `shaft_step` blocked post (`0.9200 mm gouge, 1.5000 mm rest`); `source: "stock"` refused (`nothing to thread`); SolidCraft helical thread refused (`the model's thread is HELICAL... Threading does not read a helical thread yet`). Phase-aligned revolved-V sharp workaround passed `verify` (`ok: true`, 0 findings). |
| 5b. Post & G-Code Audit | Posted `q03-turn-shaft.nc` (`Tormach PathPilot Lathe`, G33, 1858 lines, 9 ops) and `q03-no-thread.nc` (1778 lines); tested `G76` on `Tormach PathPilot Lathe` and `Fanuc Lathe`; saved `q03-turn-shaft.camproj`; audited every operation in the G-code. | **WRONG** | Posted end-to-end with workarounds (`G7` dia mode, `G95` mm/rev, `G97` constant RPM, `G33 K1.5` 15+2 passes to `X14.16`, part-off to `X0` + retract + `M9 M5 G30 M30`). G-code audit uncovered critical bugs: `Op7_Part_Off` cuts `Z-71.5..-68.5` (chops 1.5 mm off finished part), `Op2/Op3` plunge into solid bar at `Z-70.8` feeding back-to-front (Orientation 2) + air-cut at `X44.6`, `Op6` `G33` plunges into solid bar with zero air lead-in, `G76` unpostable due to `top_z` clamping & angle/degression refusals, no CSS `G96`/`G50`. |

---

## Detailed Problem Reports

### Problem 1 (CamCraft): `Parting` cuts 1.5 mm into the finished part (chopping 70 mm part to 68.5 mm), passes `verify` with 0 warnings, and refuses the offset workaround
- **Program:** CamCraft v0.25.0
- **Severity:** **Critical — gouges/scraps finished part silently; posts wrong G-code.**
- **Smallest steps to reproduce:**
  1. `import_model {"name": "shaft_step", "path": "/home/mikekinney/qa/runs/20261122-q03-turn-shaft/q03-turn-shaft.step"}`
  2. `move_model {"name": "shaft_step", "position": [0, 0, 14]}`
  3. `set_stock {"cylinder": {"diameter": 32, "height": 85}}`
  4. `set_wcs {"origin": [0, 0, 84]}`
  5. `add_tool {"catalog": "GT-0300", "name": "Parting_3mm"}` (`width: 3.0`)
  6. `add_operation {"kind": "part", "name": "Op7_Part_Off", "tool": "Parting_3mm", "source": "shaft_step", "params": {"bottom_z": {"reference": "bottom_of_model", "offset": 0}, "top_z": {"reference": "top_of_model", "offset": 0}, "stepdown": 1, "feed": 60, "plunge_feed": 50}}`
  7. `generate {}`, `verify {}`, `post {"output": ".../q03-turn-shaft.nc", "post": "Tormach PathPilot Lathe"}`
  8. Attempt workaround: `edit_operation {"name": "Op7_Part_Off", "params": {"bottom_z": {"reference": "bottom_of_model", "offset": -1.5}}}` → `generate {}`.
- **Expected:** For a 3.0 mm wide grooving/parting blade with `ORIENTATION 2` (left edge at programmed Z, right edge at `Z + 3.0` — verified against `Op5_Groove_Finish` and `Op4d_Relief_Finish` where programmed Z is the blade's left edge), parting off at `Z = -70.0` must place the blade's **right** edge at `Z = -70.0` (blade center at `Z = -71.5`, programmed left edge `Z-73.0`). At minimum, `verify` must flag a 1.5 mm gouge into the part, and `bottom_z: bottom_of_model - 1.5` (still 12.5 mm inside the Ø32 × 85 bar stock) must be accepted.
- **Actual:** `Op7_Part_Off` places the blade **center** at world `Z = 14.0` (WCS `Z = -70.0`), and the post subtracts `width/2 = 1.5 mm`, emitting `G0 Z-71.5` / `G1 X0 Z-71.5 F0.025` (`q03-turn-shaft.nc` lines 1821–1852). With a 3 mm blade in Orientation 2, the blade spans `Z = -71.5` to `Z = -68.5`, removing 1.5 mm from the finished 70 mm part and obliterating the 1 mm head chamfer (`Z = -70..-69`). `verify` reports `warnings: []`. Trying to offset `bottom_z` by `-1.5` is refused with:
  > `generate: Op7_Part_Off - could not generate the toolpath: Op7_Part_Off: nothing to machine: nothing to part - the parting plane stands off the end of the bar`
- **Reproduction files:** [q03-turn-shaft.camproj](file:///home/mikekinney/qa/runs/20261122-q03-turn-shaft/q03-turn-shaft.camproj), [q03-turn-shaft.nc](file:///home/mikekinney/qa/runs/20261122-q03-turn-shaft/q03-turn-shaft.nc#L1813-L1858), [q03-turn-shaft.step](file:///home/mikekinney/qa/runs/20261122-q03-turn-shaft/q03-turn-shaft.step).

---

### Problem 2 (CamCraft): `Profile Rough` & `Profile Finish` use Orientation 2 feeding back-to-front (`Z-70.8 → Z-0.8`), plunging into solid bar stock behind the part and into narrow grooves
- **Program:** CamCraft v0.25.0
- **Severity:** **Critical — tool crash / gouge on a real lathe; wrong G-code.**
- **Smallest steps to reproduce:**
  1. Setup as in Problem 1 with `OD_Turn_R08` (`TN-1600-0R8`, `nose_radius: 0.8`, `rpm: 2500`).
  2. `add_operation {"kind": "profile_rough", "name": "Op2_OD_Rough", "tool": "OD_Turn_R08", "source": "shaft_step", "params": {"stock_to_leave": 0.3, "stepdown": 1, "feed": 250, "plunge_feed": 120}}`
  3. `add_operation {"kind": "profile_finish", "name": "Op3_OD_Finish", "tool": "OD_Turn_R08", "source": "shaft_step", "params": {"feed": 150, "plunge_feed": 100}}`
  4. `generate {}`, `post {"output": ".../q03-no-thread.nc", "post": "Tormach PathPilot Lathe"}`.
- **Expected:** Standard right-hand OD turning (Orientation 3) approaches from the front (`+Z`, `Z ≈ +2`), turns in `-Z` toward the chuck (Ø16 → Ø24 → Ø30), does not plunge past `Z = -70` into solid bar stock, and bridges over narrow grooves (2.5 mm O-ring groove, 3 mm thread relief) that a standard OD turning insert cannot enter without trailing-flank collision.
- **Actual:** Both operations post `(T0101 OD_Turn_R08 - TURNING INSERT ORIENTATION 2)`, rapid directly to `G0 Z-70.8` (0.8 mm behind the part back face into uncut Ø32 bar stock), plunge radially into solid bar (`G1 X29.512 Z-70.8` in Rough, `G1 X28.662 Z-70.8` in Finish), and feed **backwards** in `+Z` toward `Z-0.8`, plunging into both the 2.5 mm O-ring groove (`X21`, lines 1428–1442) and the 3 mm Ø13 thread relief (`X13.6` in Rough, `X13` in Finish, lines 1491–1505). On a real Tormach 15L with a CNMG/DNMG insert, this crashes the insert/holder into the bar stock and groove walls. Furthermore, the tool center stops at `Z = -70.0`, leaving the lower half of the 1 mm × 45° head chamfer (`R = 14.0..14.565`, `Z = -70.0..-69.43`) uncut.
- **Reproduction files:** [q03-turn-shaft.camproj](file:///home/mikekinney/qa/runs/20261122-q03-turn-shaft/q03-turn-shaft.camproj), [q03-turn-shaft.nc](file:///home/mikekinney/qa/runs/20261122-q03-turn-shaft/q03-turn-shaft.nc#L27-L1552).

---

### Problem 3 (CamCraft): `Profile Rough` air-cuts up to `X44.6` above Ø32 bar stock and retracts at plunge feed; `Groove Rough` ignores turned OD and air-pecks from `X31`
- **Program:** CamCraft v0.25.0
- **Severity:** **Moderate — excessive air cutting, slow retracts, unexpected tool excursion beyond stock.**
- **Smallest steps to reproduce:** Same as Problem 2, plus `Op4_Groove_Rough` (`Z=-33.75..-36.25`) and `Op4c_Relief_Rough` (`Z=-17..-20`) after `Op2_OD_Rough` and `Op3_OD_Finish`.
- **Expected:** `Profile Rough` passes should clip to the Ø32 bar stock (`X ≤ 32`) with rapid (`G0`) retracts; `Groove Rough` after OD finishing should start near the finished local OD (`X24` for O-ring groove, `X16` for thread relief).
- **Actual:** `Op2_OD_Rough` offsets the entire profile outward by 7 × 1 mm radial stepdowns to `X44.6` (12.6 mm dia above the Ø32 bar stock; lines 34–176), air-cutting for 6 of 8 passes, and retracts radially at slow plunge feed `G1 X44.7 Z-0.8 F0.048` (line 177, 1321). `Op4_Groove_Rough` air-pecks from `X31` down to `X24` (8 air pecks per Z station), and `Op4c_Relief_Rough` air-pecks from `X31` down to `X16` (15 air pecks per Z station), retracting all the way to `X34` on every 0.5 mm radial peck.
- **Reproduction files:** [q03-turn-shaft.nc](file:///home/mikekinney/qa/runs/20261122-q03-turn-shaft/q03-turn-shaft.nc#L34-L210), [q03-turn-shaft.nc](file:///home/mikekinney/qa/runs/20261122-q03-turn-shaft/q03-turn-shaft.nc#L1553-L1715).

---

### Problem 4 (CamCraft): `Threading` refuses both smooth major-OD CAD models (cosmetic thread) AND SolidCraft's modeled helical thread, requiring an artificial non-helical revolved-V groove model
- **Program:** CamCraft v0.25.0
- **Severity:** **High — breaks standard CAD-to-CAM threading workflow; blocks posting.**
- **Smallest steps to reproduce:**
  - *Case A (smooth bar stock):* `add_operation {"kind": "threading", "name": "Test_Thread_Stock", "tool": "Thread_60deg", "source": "stock", "params": {"pitch": 1.5, "depth": 0.92, "infeed": 0.2, "top_z": {"reference": "top_of_model", "offset": 0}, "bottom_z": {"reference": "top_of_model", "offset": -18.5}}}` → `generate {}`.
  - *Case B (smooth Ø16 cylinder + Ø13 relief in `q03-turn-shaft.step`):* Same op with `source: "shaft_step"` → `generate {}` → `verify {}` → `post`.
  - *Case C (SolidCraft `solid.thread {modeled: true}` helical thread in `q03-turn-shaft-modeled-thread.step`):* `import_model` → `move_model [0,0,14]` → same op with `source: "shaft_threaded"` → `generate {}`.
- **Expected:** `Threading` should machine an M16 × 1.5 thread onto a standard Ø16 major-diameter cylinder (industry-standard CAD representation, and SolidCraft's default cosmetic thread) without flagging its own 0.92 mm thread depth as a gouge — OR at minimum accept SolidCraft's own `modeled: true` helical thread.
- **Actual:**
  - Case A refused: `"Test_Thread_Stock: invalid machining parameters: nothing to thread - no valley 0.920 down from the bar's surface stands in the window: threading cuts a MODELED thread (a revolved V section); a smooth bar has none to find"`
  - Case B generates (only because the Ø13 relief sits in the window) then blocks post with a false gouge equal to the thread depth: `"post: post blocked - 1 warning(s): Op6_Thread_M16x1_5: finish envelope near move 402: 0.9200 mm gouge, 1.5000 mm rest"` (persists even with `bottom_z: -17.0`).
  - Case C refused: `"Op6_Thread_M16x1_5: invalid machining parameters: the model's thread is HELICAL (about a 1.50 pitch, right-hand helix about the bar's axis) - Threading does not read a helical thread yet; it reads a modelled thread as a revolved V section. Model the thread as revolved V grooves at its pitch (no helix) for this operation"`
  - Even when manually modeling a non-helical revolved-V groove workaround, a 0.5 mm Z-phase mismatch triggers `"0.0540 mm gouge"` (`blocks_post: true`), and adding a standard ISO 68-1 `0.18 mm` root flat triggers `"0.2338 mm rest"` (`blocks_post: true`). Only a phase-aligned, sharp-root revolved-V dummy model (`q03-turn-shaft-revolved-v-sharp.step`) passed `verify`.
- **Reproduction files:** [q03-turn-shaft.step](file:///home/mikekinney/qa/runs/20261122-q03-turn-shaft/q03-turn-shaft.step), [q03-turn-shaft-modeled-thread.step](file:///home/mikekinney/qa/runs/20261122-q03-turn-shaft/q03-turn-shaft-modeled-thread.step), [q03-turn-shaft-revolved-v.step](file:///home/mikekinney/qa/runs/20261122-q03-turn-shaft/q03-turn-shaft-revolved-v.step), [q03-turn-shaft-revolved-v-sharp.step](file:///home/mikekinney/qa/runs/20261122-q03-turn-shaft/q03-turn-shaft-revolved-v-sharp.step), [q03-v-test.camproj](file:///home/mikekinney/qa/runs/20261122-q03-turn-shaft/q03-v-test.camproj).

---

### Problem 5 (CamCraft): `Threading` clamps `top_z` to `top_of_model`, causing `G33` to plunge straight into solid material (zero air lead-in) and permanently blocking `G76`
- **Program:** CamCraft v0.25.0
- **Severity:** **Critical — plunges threading tool into solid material (ruins lead-in / chips insert); catch-22 makes `G76` unpostable.**
- **Smallest steps to reproduce:**
  1. On `Op6_Thread_M16x1_5` (source `shaft_revolved_v`, `pitch=1.5, depth=0.92`), set `top_z: {"reference": "top_of_model", "offset": 2.0}` to give 2 mm air lead-in in front of `Z=0`, then `generate {}` → `verify {}`.
  2. With `top_z: {"reference": "top_of_model", "offset": -1.0}`, `set_thread_infeed` to `method: "radial", schedule: "constant_depth", min_depth: 0.2`, `set_post_options {"threading": "g76", "setup": "Setup1"}`, `generate {}`, then `post` to `Tormach PathPilot Lathe` and `Fanuc Lathe`.
  3. Switch back to `threading: "g32"` and read posted `q03-turn-shaft.nc` lines 1740–1810.
- **Expected:** `top_z` above `top_of_model` (`Z = +2.0` to `+4.5`, 2–3 pitches) should start each threading pass in free air so Z-axis synchronization/acceleration finishes before entering the workpiece at `Z = 0`, enabling both safe `G33`/`G32` and valid `G76`.
- **Actual:** `top_z = +2.0` is clamped back to `top_of_model` (`Z = 0.0`), breaking phase alignment (`0.0540 mm gouge`). With `top_z = -1.0`:
  - PathPilot `G33` posts `G0 X34 Z-0.593` → `G1 X15.6 Z-0.593 F0.1125` → `G33 Z-15.593 K1.5` (down to `G1 X14.16 Z-1`), plunging the threading insert directly into solid Ø16 bar at `Z-0.593..-1.0` with zero air lead-in.
  - Both `Tormach PathPilot Lathe` and `Fanuc Lathe` refuse `G76`:
    > `post: post blocked - 1 warning(s): G76 threading cycle refused - the start point is above material (G76 moves the insert to pass depth at rapid traverse) - start the thread in free air in front of the end face or in a relief groove wider than the insert, regenerate the operation, or choose G33 passes`
- **Reproduction files:** [q03-turn-shaft.camproj](file:///home/mikekinney/qa/runs/20261122-q03-turn-shaft/q03-turn-shaft.camproj), [q03-turn-shaft.nc](file:///home/mikekinney/qa/runs/20261122-q03-turn-shaft/q03-turn-shaft.nc#L1733-L1812).

---

### Problem 6 (CamCraft): `G76` post-processor bugs — PathPilot refuses `constant_area` & `flank` infeed; Fanuc refuses CamCraft's own default `flank_angle: 29.5`
- **Program:** CamCraft v0.25.0
- **Severity:** **High — blocks posting standard `G76` threading cycles.**
- **Smallest steps to reproduce:**
  1. On verified `Op6_Thread_M16x1_5` with `set_thread_infeed {"name": "Op6_Thread_M16x1_5", "infeed_plan": {"method": "flank", "flank_angle": 29.5, "schedule": "constant_area", "min_depth": 0.05, "spring_passes": 2}}` (note `29.5` is the tool's own default from `get_thread_infeed`).
  2. `set_post_options {"setup": "Setup1", "threading": "g76"}`.
  3. `post {"output": ".../q03-turn-shaft-g76.nc", "post": "Tormach PathPilot Lathe"}` and `post {"output": ".../q03-turn-shaft-fanuc.nc", "post": "Fanuc Lathe"}`.
  4. Change to `schedule: "constant_depth", min_depth: 0.2, flank_angle: 30.0`, regenerate, and `post` to `Tormach PathPilot Lathe`.
- **Expected:** PathPilot (`LinuxCNC` G76) supports both `R2.0` (constant-area degression) and `Q` (compound flank infeed angle); Fanuc G76 should accept CamCraft's own default 29.5° modified-flank infeed on a 60° metric thread (emitting `60` or `29`/`30` as appropriate rather than invalid `59`).
- **Actual:**
  - PathPilot `constant_area` refused: `"G76 threading cycle refused - the passes are not equal rungs (minimum depth differs from the first) - only LinuxCNC's constant-depth degression (R1) states the record's ladder exactly"`
  - PathPilot `flank` (`30°`) refused: `"G76 threading cycle refused - infeed angle 60 - a compound infeed (Q) is not posted on this control yet (radial only, Q0)"`
  - Fanuc refused CamCraft's own default `flank_angle: 29.5` (doubled internally to `59.0`): `"G76 threading cycle refused - infeed angle 59 is not a G76 tool angle (0, 29, 30, 55, 60, 80)"`
- **Reproduction files:** [q03-turn-shaft.camproj](file:///home/mikekinney/qa/runs/20261122-q03-turn-shaft/q03-turn-shaft.camproj).

---

### Problem 7 (CamCraft): Neither built-in 60° threading catalog tool can cut M16 × 1.5, and the `width > pitch` check is geometrically wrong for a 60° V-insert
- **Program:** CamCraft v0.25.0
- **Severity:** **High — refuses valid standard M16 × 1.5 thread with every catalog threading tool.**
- **Smallest steps to reproduce:**
  1. `add_tool {"catalog": "TD-0866-60", "name": "Thread_60deg"}` → `add_operation {"kind": "threading", "tool": "Thread_60deg", "source": "shaft_step", "params": {"pitch": 1.5, "depth": 0.92, "infeed": 0.2}}` → `generate {}`.
  2. `edit_tool {"name": "Thread_60deg", "patch": {"width": 1.732}}` (matching catalog `TD-1732-60`) → `generate {}`.
- **Expected:** A standard 60° partial-profile threading insert (`TD-1732-60`, V-height 1.500 mm) easily cuts an M16 × 1.5 thread (`depth = 0.92 mm`): at `depth = 0.92 mm`, its 60° V width at the workpiece surface is only `2 × 0.92 × tan(30°) = 1.062 mm < 1.500 mm` pitch. Everything above `0.92 mm` is outside the bar in open air and cannot touch the adjacent thread turn.
- **Actual:**
  - `TD-0866-60` (`width: 0.866`) refused: `"Op6_Thread_M16x1_5: invalid machining parameters: a 0.866 wide insert at 60 degrees reaches 0.750 of the 0.920 this thread is deep - a full-form insert must stand at or above the crest, or the crests never get cut; single-flank passes are not this kernel's to speak"`
  - `TD-1732-60` (`width: 1.732`) refused: `"Op6_Thread_M16x1_5: invalid machining parameters: a 1.732 wide insert does not fit a 1.500 pitch - the V is wider than the groove it was asked to cut; its flanks would stand in the neighbouring turn. Grind narrower steel for a single-point thread"`
  - Workaround required manually editing `width` to `1.3 mm` (within the narrow artificial band `1.063..1.500 mm`).
- **Reproduction files:** [q03-turn-shaft.step](file:///home/mikekinney/qa/runs/20261122-q03-turn-shaft/q03-turn-shaft.step), [q03-turn-shaft.camproj](file:///home/mikekinney/qa/runs/20261122-q03-turn-shaft/q03-turn-shaft.camproj).

---

### Problem 8 (CamCraft): Constant Surface Speed (`G96`) with max RPM clamp (`G50` / `G96 D`) cannot be enabled via MCP tools or tool/operation fields
- **Program:** CamCraft v0.25.0
- **Severity:** **High — prevents posting standard lathe CSS (`G96` + `G50`/`D`); forces constant RPM (`G97`) across Ø32 down to X0.**
- **Smallest steps to reproduce:**
  1. `list_operation_types {"section": "Turning"}` (lists no CSS/surface-speed field).
  2. Attempt `edit_operation {"name": "Op2_OD_Rough", "params": {"surface_speed": 200}}` (also tested `css`, `cutting_speed`, `spindle_mode`, `constant_surface_speed`, `css_m_min`, `sfm`, `css_speed`).
  3. Attempt `edit_tool {"name": "OD_Turn_R08", "patch": {"surface_speed": 200}}` (also tested `css`, `constant_surface_speed`, `css_m_min`, `sfm`, `cutting_speed`, `css_speed`).
  4. `get_post_options {"setup": "Setup1"}` (`script_schema: []`).
- **Expected:** `set_workholding_max_rpm` explicitly documents: *"Constant surface speed clamps to the lowest of the machine's, the tool's and this limit (Fanuc G50 S, Tormach G96 D)"*. Facing, OD turning, grooving, and parting should be able to enable CSS (`G96`) with a max RPM clamp (`G50` / `G96 D`).
- **Actual:** Every probed parameter is refused (`unknown field ...`), `list_operation_types` omits all optional fields with only a generic note (`"Fields absent from the defaults are optional and off; set them in a params patch to turn them on"`), and every operation in `q03-turn-shaft.nc` posts `G97 S2500 M3` / `G97 S2000 M3` constant RPM (no `G96`, no `G50`, no `D` clamp).
- **Reproduction files:** [q03-turn-shaft.nc](file:///home/mikekinney/qa/runs/20261122-q03-turn-shaft/q03-turn-shaft.nc).

---

### Problem 9 (CamCraft): Built-in lathe tool catalog defects — `TN-1600-0R8` exceeds Tormach 15L max RPM (3000 > 2500), and all lathe catalog tools omit `tool_material`, breaking `suggest_cutting_data`
- **Program:** CamCraft v0.25.0
- **Severity:** **Moderate — blocks posting & breaks feeds/speeds advisor out of the box.**
- **Smallest steps to reproduce:**
  1. `load_machine {"builtin": "Tormach 15L Slant-PRO"}` (`spindle_max: 2500`), `set_work_material {"material": "aluminium_6061"}`.
  2. `add_tool {"catalog": "TN-1600-0R8", "name": "OD_Turn_R08"}` (`rpm: 3000`, no `tool_material`).
  3. `add_operation {"kind": "profile_rough", "name": "Op2_OD_Rough", "tool": "OD_Turn_R08", "source": "shaft_step"}`.
  4. `suggest_cutting_data {"operation": "Op2_OD_Rough"}` and `generate {}` → `verify {}`.
- **Expected:** Built-in catalog lathe tools should carry `"tool_material": "carbide"` (just as all milling catalog tools do) so `suggest_cutting_data` works immediately; `suggest_cutting_data` should work without manual patching, and turning operation defaults should not default to milling feeds (`feed: 1200, plunge_feed: 300`).
- **Actual:**
  - `suggest_cutting_data` refuses: `"suggest_cutting_data: tool OD_Turn_R08 states no tool material (HSS or carbide)"` (after patching `tool_material: "carbide"`, it succeeds with `cutting_speed: 325, rpm: 2500 (clamped from 3233), feed: 562.5`).
  - `verify` blocks post on `Tormach 15L Slant-PRO`: `"machine Tormach 15L Slant-PRO: 3 limit problem(s): Op1_Face: tool OD_Turn_R08 runs 3000 rpm, above the machine's 2500; ..."`
- **Reproduction files:** [q03-turn-shaft.camproj](file:///home/mikekinney/qa/runs/20261122-q03-turn-shaft/q03-turn-shaft.camproj).

---

### Problem 10 (CamCraft): `Groove Rough` / `Groove Finish` with default full-model Z bounds fail verification (`5.6 / 5.5 mm rest`) and block posting
- **Program:** CamCraft v0.25.0
- **Severity:** **Moderate — blocks posting with confusing error; prevents machining multiple grooves in one operation.**
- **Smallest steps to reproduce:**
  1. `add_operation {"kind": "groove_rough", "name": "Op4_Groove_Rough", "tool": "Groove_2mm", "source": "shaft_step", "params": {"stock_to_leave": 0.1, "stepdown": 0.5, "feed": 90, "plunge_feed": 60}}` (leaving default `top_z: top_of_model`, `bottom_z: bottom_of_model`).
  2. `add_operation {"kind": "groove_finish", "name": "Op5_Groove_Finish", "tool": "Groove_2mm", "source": "shaft_step", "params": {"feed": 60, "plunge_feed": 50}}`.
  3. `generate {}` → `verify {}`.
- **Expected:** Either groove operations automatically detect and machine all OD grooves (O-ring + thread relief) within `[top_z, bottom_z]` and only judge those groove zones, OR documentation clearly states that each groove requires explicit tight `[top_z, bottom_z]` bounds.
- **Actual:** `verify` blocks posting with:
  > `Op4_Groove_Rough: finish envelope near move 308: 0.0000 mm gouge, 5.6000 mm rest`  
  > `Op5_Groove_Finish: finish envelope near move 76: 0.0000 mm gouge, 5.5000 mm rest`  
  Splitting into separate operations narrowed to `Z=-33.75..-36.25` (O-ring) and `Z=-17..-20` (thread relief) cleared all warnings and posted exact groove widths (2.5 mm and 3.0 mm) and depths (Ø21 and Ø13).
- **Reproduction files:** [q03-turn-shaft.step](file:///home/mikekinney/qa/runs/20261122-q03-turn-shaft/q03-turn-shaft.step), [q03-turn-shaft.camproj](file:///home/mikekinney/qa/runs/20261122-q03-turn-shaft/q03-turn-shaft.camproj).

---

### Problem 11 (CamCraft): Stock simulation and `render_stock` are completely unavailable for turning programs
- **Program:** CamCraft v0.25.0
- **Severity:** **Moderate — no visual or volumetric stock verification for any lathe program.**
- **Smallest steps to reproduce:**
  1. With `Op6_Thread_M16x1_5` present: `generate {}` → `render_stock {"view": "iso", "toolpath": true}`.
  2. With `Op6_Thread_M16x1_5` deleted (only Face, OD Rough/Finish, Grooves, Part-Off): `generate {}` → `render_stock {"view": "iso", "toolpath": true}`.
- **Expected:** `render_stock` and program-level stock simulation should simulate and render turned bar stock (at least for non-threading turning operations, and ideally showing rest/gouge visually).
- **Actual:**
  - With threading: `"render_stock: no stock simulation to render: Op6_Thread_M16x1_5: the lathe stock sim does not cut threads yet (helical passes) - not simulated; the thread op's verdict still stands"`
  - Without threading: `"render_stock: no stock simulation to render: Op1_Face: a turning program is cut by the field eye about its fitted spindle, but the column verdict reads Z heights from above (a parting kerf reads as a plunge) - not judged here; the revolved verdict at generation stands"`
- **Reproduction files:** [q03-turn-shaft.camproj](file:///home/mikekinney/qa/runs/20261122-q03-turn-shaft/q03-turn-shaft.camproj).

---

## Confusing Tool Names, Defaults, and UX Notes (even where working)
1. **Undocumented optional fields in `list_operation_types`:** Returns `"note": "Fields absent from the defaults are optional and off; set them in a params patch to turn them on."` without listing what those optional field names or types are (making features like CSS impossible to discover via MCP).
2. **Milling defaults on Turning operations:** Every Turning operation in `list_operation_types` defaults to `feed: 1200.0, plunge_feed: 300.0` (mm/min, typical of milling) rather than realistic lathe feeds, and `set_post_options` names the single-pass threading mode `"g32"` even though `Tormach PathPilot Lathe` emits LinuxCNC `G33`.
3. **Confusing turning tool `diameter` / `flute_len` / `shank_d` fields:** Catalog lathe inserts (`TN-1600-0R8`, `GT-0200`, `TD-0866-60`) carry milling-style `diameter: 16.0, flute_len: 8.0` alongside `nose_radius` / `width`, and threading insert `width` (`0.866`, `1.732`) is not documented as "full 60° V top-width" vs pitch.
4. **No built-in lathe tool holders:** `Tormach 15L Slant-PRO` has `default_holder: null`, so every turning operation logs `"holder not judged - the turning tool carries no modelled body, so only its cutting footprint was judged against the stock"`.
5. **SolidCraft `.sc` direct import worked well** (`8,124` facets, identical geometry/bounds to `.step`), with a clear note that it imports the cached final body only.

---

## G-Code Verification Checklist (`q03-turn-shaft.nc`, 1858 lines)
- **Diameter vs radius mode:** `G7` (Diameter mode on LinuxCNC/PathPilot) in header line 2 (`G7 G18 G21 G90 G91.1 G40 G95`). All X coordinates are diameters (`X32` stock, `X30` head, `X24` body, `X21` O-ring groove, `X16` thread major, `X14.16` thread minor, `X13` thread relief, `X0` part-off, `X-1.6` face past center).
- **Feeds:** `G95` (mm/rev) — CamCraft accurately converts mm/min op feeds to mm/rev (`F = feed / rpm`: Face `200/2500=0.08`, Rough `250/2500=0.1`, Finish `150/2500=0.06`, Groove Rough plunge `60/2500=0.024`, Groove Finish plunge `50/2500=0.02`, Part-off plunge `50/2000=0.025`).
- **CSS (`G96`) & max RPM clamp (`G50` / `G96 D`):** **MISSING** — every op uses `G97` constant RPM (`S2500`, `S2000`, `S800`).
- **Grooves:** O-ring groove `Op5_Groove_Finish` plunges at `Z-36.25` and `Z-35.75` with 2 mm Ori-2 blade to `X21` (`F0.02`) → exact 2.50 mm wide, 1.50 mm radial depth (`Ø21`). Thread relief `Op4d_Relief_Finish` plunges at `Z-20` and `Z-19` with 2 mm Ori-2 blade to `X13` (`F0.02`) → exact 3.00 mm wide, `Ø13`.
- **Thread (`Op6_Thread_M16x1_5`):** Emits `G33 ... K1.5` (exact pitch 1.5 mm), `G97 S800`, 15 constant-area flank-shifted cutting passes + 2 spring passes down to `X14.16` (exact `0.92 mm` radial depth) — **BUT** starts at `Z-0.593..-1.0` plunging straight into solid Ø16 bar with zero air lead-in, and required a dummy revolved-V CAD model; `G76` refused.
- **Part-off (`Op7_Part_Off`):** Reaches axis `X0` (`G1 X0 Z-71.5 F0.025`), retracts cleanly (`G0 X34 Z-71.5`, `G0 X34 Z24.5`), stops coolant & spindle (`M9`, `M5`, `G30`, `M30`) — **BUT** `Z-71.5` with a 3 mm Ori-2 blade cuts `Z-71.5..-68.5`, chopping 1.5 mm off the finished part.

RESULT: 11 problems (11 CamCraft, 0 SolidCraft); end-to-end: posted
