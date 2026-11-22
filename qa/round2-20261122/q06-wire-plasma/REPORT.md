# QA Report: Wire EDM (Die & Punch) and Plasma Plate (`20261122-q06-wire-plasma`)

- **SolidCraft**: v0.46.2 (commit `f4990915b4a91c3c82bab0745dd521f95078a2ac`, headless)
- **CamCraft**: v0.25.0 (headless)
- **Run directory**: `/home/mikekinney/qa/runs/20261122-q06-wire-plasma`
- **Deliverables**:
  - CAD: `q06-die.sc`/`.step`/`.dxf` (plus repros `q06-die-straight.step`, `q06-die-lines.dxf`, `q06-die-poly.dxf`), `q06-punch.sc`/`.step`/`.dxf` (plus repro `q06-punch-sharp.step`), `q06-bracket.sc`/`.step`/`.dxf`
  - CAM & NC: `q06-die.camproj`/`.nc` (plus repro `q06-die-step-blocked.camproj`), `q06-punch.camproj`/`.nc`, `q06-bracket.camproj`/`.nc` (plus diagnostics `q06-bracket-default.nc`, `q06-bracket-comp.nc`)

---

## Step Summary Table

| Step | What was done | Result | Exact message or key numbers |
|---|---|---|---|
| 1a | SolidCraft: build straight die plate 80×60×20 mm with 30×20 R3 + Ø8 half-round notch opening; measure; save `.sc`, export `.step` & `.dxf` | OK | `volume_mm3=83651.8585` (analytic `83651.8585`), bbox `[-40,-30,0]..[40,30,20]`, 16 faces, closed; DXF 10 entities (`q06-die-straight.step`, `q06-die.dxf`) |
| 1a2 | SolidCraft: rebuild die opening with 1° outward taper downward from Z=20 (`direction:"negative", taper:"1 deg"`); measure; save `q06-die.sc`/`.step` | OK | `volume_mm3=83302.5759`, 16 faces, closed; bottom opening bounds `[-15.349101,-10.349101,0]..[15.349101,14.349101,0]` (`20*tan(1°)=0.349101 mm` wider at bottom) |
| 1b | SolidCraft: build matching punch 20 mm thick; measure; save `q06-punch.sc`/`.step`/`.dxf` (sharp copy saved as `q06-punch-sharp.step`; later added R0.5 notch fillets as workaround for CamCraft sharp-corner refusal) | OK | Sharp: `volume_mm3=12348.1415` (analytic `12348.1415`), 12 faces, bbox `[-15,-10,0]..[15,14,20]`. With R0.5 notch fillets: `volume_mm3=12349.9365`, 14 faces |
| 1c | SolidCraft: build plasma bracket 200×120×6 mm R10 corners, Ø40 hole, two 12×30 slots, four Ø10 holes; measure; save `q06-bracket.sc`/`.step`/`.dxf` (`layers:"roles"`) | OK | `volume_mm3=130111.0096` (analytic `130111.0096`), bbox `[-100,-60,0]..[100,60,6]`, 23 faces, closed; DXF 21 entities on `OUTLINE`/`CUT` |
| 2a | CamCraft: `load_machine` `Sodick ALC400G` (and checked `AGIECUT Classic 2S (template)`) | WRONG | Loaded, but neither wire machine has `edm.conditions`, `lower_guide_z`/`upper_guide_z`, or `edm.codes`; `Sodick ALC400G` offers `"Generic ISO Wire EDM"` instead of built-in `"Sodick Wire EDM (Mark 21)"` |
| 2b | CamCraft: `import_model` `q06-die-straight.step` (1386 facets), `set_stock` 80×60×20, `add_tool` `EDM-W025` (Ø0.25 wire), `set_wcs` `[0,0,0]` | OK | Stock `[-40,-30,0]..[40,30,20]`, WCS G54 `[0,0,0]`, T1 `EDM-W025` |
| 2c | CamCraft: `add_operation` `wire_contour` on `Sodick ALC400G` | REFUSED | `add_operation: invalid parameters: a Wire Contour names no EDM condition - choose one of machine Sodick ALC400G's conditions (it states none - add edm.conditions to the machine file)` |
| 2d–2e | Workaround: `edit_machine` adding `edm.conditions` (`C001` gap `0.035`, `C002` gap `0.015`); `add_operation` `DieWireContour` (`condition:"C001"`) | OK | Required probing undocumented `edm.conditions[].id` and `gap` keys and undocumented `wire_contour.condition` |
| 2f | CamCraft: `edit_operation` adding `taper_deg: 1` together with `skims: [{condition:"C002"}]` | REFUSED | `edit_operation: invalid parameters: wire contour: 1 skim pass(es) are stated with a 1.000 degree taper - skims are cut on a vertical, untabbed one-profile contour only; remove the skims or that setting` |
| 2g | CamCraft: `edit_operation` setting `taper_deg: 1` without skims | REFUSED | `edit_operation: invalid parameters: a 1.000 degree taper is asked, and machine Sodick ALC400G states no edm.lower_guide_z and edm.upper_guide_z - a tapered wire's guide positions are read from them; set the taper to 0` |
| 2h–2i | Workaround: patched `lower_guide_z:-25, upper_guide_z:45`; stitched 2 opening chains; added 2nd op `DieWireSkim` (`C002`, `taper_deg:1`); generated; `verify`/`post` on straight STEP | REFUSED | `post blocked` by `wire gouge: the kerf of move 1 enters the part by 0.3491 mm near X0.000 Y-10.189 at Z0.001` (`20*tan(1°)`) and `wire slug` holding `2.893 mm²` / `6.868 mm²` of part |
| 2j | CamCraft: `import_dxf` `q06-die.dxf` and select DXF chain while STEP model is present | REFUSED | `generate: DieWireContour - could not generate the toolpath: DieWireContour: invalid machining parameters: selected DXF wire cuts cannot verify a separate part, fixture or machine table in the wire span` |
| 2k–2l | CamCraft: replaced STEP with exact 1° tapered `q06-die.step` (1448 facets); set `capture: true`; `generate` + `verify` + `post` | REFUSED | `capture: true` cleared slug drop blocker, but post STILL blocked by `wire gouge: the kerf of move 2 enters the part by 0.0004 mm near X12.011 Y-10.349 at Z0.001 (118 move(s) gouge; the deepest reading at any height is 0.0046 mm)` (`q06-die-step-blocked.camproj`) |
| 2m | CamCraft: deleted STEP model, selected `q06-die.dxf` chain, `generate` | REFUSED | `generate: DieWireContour - could not generate the toolpath: DieWireContour: invalid machining parameters: the selected DXF chain contains an arc or circle; the generic wire path requires straight segments` |
| 2n | SolidCraft `file.export_view_dxf` (`curves:"lines"`) -> `q06-die-lines.dxf`; CamCraft `import_dxf` | REFUSED | `import_dxf: DXF import: DXF CLASSES: unsupported SECTION` |
| 2o–2p | SolidCraft polyline sketch -> `q06-die-poly.dxf`; CamCraft `import_dxf`, tried `delete_model` / `delete_setup` on old DXF, then `generate` in `Setup2` | REFUSED | `delete_model: 'q06-die (2)' is not a model`; `delete_setup: invalid parameters: the DXF profile 'die_poly' is placed in 'Setup1' - delete it first`; and `generate` refused: `selected DXF wire cuts require a single vertical pass with slug capture and no tabs` |
| 2q | Workaround: reopened `q06-die-step-blocked.camproj`, set `taper_deg: 0.95` (0.05° under-taper to clear 0.0046 mm tessellation chord error); `verify` passed (`ok: true`); tried `post` with Sodick & Generic ISO | REFUSED | Sodick post: `operation DieWireContour: is tapered - the Sodick wire post writes a vertical 2-axis cut only (no G51 / G52 or A words)`. Generic ISO post: refused for missing `edm.codes.{condition,comp_off,taper_on,taper_off,uv}` |
| 2r | Workaround: patched `edm.codes` (`{id}`, `G40`, `G51`, `G50`, `U{u} V{v}`); posted `q06-die.nc` via `Generic ISO Wire EDM`, saved `q06-die.camproj` | OK (workaround) | Posted `q06-die.nc` (274 lines, 2 ops). Math verified: `Y-10.586 U0. V1.161` = exact 0.95° wider at bottom; `G51`/`G50`, `G40`, `C001`/`C002`, `X0. Y0.`, decimal points present |
| 2s | CamCraft punch: cleared doc, imported `q06-punch-sharp.step` (2806 facets), stock 50×40×20, WCS corner `[-25,-20,0]`, `PunchWireContour` (`side:"punch", start:[0,-15], tabs:{count:1,width:2}`) | REFUSED | `wire contour: an inside corner or slot of the profile near X4.000 Y10.000 is tighter than the wire can cut on the punch side - the minimum inside radius is 0.1600 mm (wire radius 0.1250 + spark gap 0.0350), and the kerf would leave 0.0614 mm standing there; round it to at least that radius, or cut with a thinner wire` |
| 2t | Workaround: added R0.5 fillets at notch corners (`q06-punch.step`), discovered `tabs.at` & `tabs.cutoff_condition:"C001"`, generated, verified (`ok:true`), saved `q06-punch.camproj`, posted `q06-punch.nc` (`Sodick Wire EDM (Mark 21)`) | WRONG / OK | Posted `q06-punch.nc` (789 lines, 2 ops: main + tab cut-off). WCS `X25. Y5.`, outside offset `0.160 mm` (`Y9.84`, `X40.16`, `Y30.16`), 2.0 mm tab (`2.320 mm` wire gap) correct; **WRONG**: retraces entire ~100 mm punch backwards with `C001` ON, emits 81 duplicate zero-length `G01` lines and double `M00` |
| 3a–3b | CamCraft plasma: `Tormach 1300PL`, `mild_steel`, `q06-bracket.step` (7912 facets), stock 220×140×6, WCS top front-left `[-110,-70,6]`, `PT-KERF-150`, `BracketPlasma` (`plasma_profile`); probed tabs/microjoints; posted default `q06-bracket-default.nc` | WRONG | No tabs/microjoints parameter exists; negative `overburn:-2` refused generically; default NC (1647 lines) has no `F` word or `M207/M208/M209`; 4 tip-up warnings hidden in `gate.not_judged` with `verify.ok:true, findings:[], warnings:[]` |
| 3c | CamCraft plasma: set discovered undocumented params (`feed:2200, kerf:1.5, pierce_delay:0.6, pierce_height:3.8, cut_height:1.5, lead_in:4, lead_out:2.5`); saved `q06-bracket.camproj`, posted `q06-bracket.nc` (`comp:"computer"`) & `q06-bracket-comp.nc` (`comp:"control"`) | WRONG / OK | Posted `q06-bracket.nc` (1671 lines). `M200/M205`, `M207 P3.8`, `M208 P1.5`, `M209 P0.6`, `G15/G16`, `F1430` small holes / `F2200` large+outer, scrap-side lead-ins/kerf, holes C0–C6 before outer C7 all verified; `comp:"control"` emits valid `G41.1 D1.5`/`G40`. **WRONG**: 508/1671 (30.4%) duplicate consecutive zero-length `G1` lines; no outside tabs |
| 3d | CamCraft plasma DXF test: `import_dxf` `q06-bracket.dxf` (8 closed loops on `CUT`/`OUTLINE`) into `Setup2`, `add_operation` `plasma_profile` | REFUSED | `generate` refused (`a toolpath from the chains of a DXF profile is not generated yet`) and `set_operation_geometry` refused (`picked chains of a profile are not supported yet`) |
| 5 | Full machinist audit of all `.nc` files | OK | Logged all geometric & G-code checks in Problems below |

---

## Problems Found

### 1. Built-in wire EDM machines (`Sodick ALC400G`, `AGIECUT Classic 2S (template)`) are unusable out of the box
- **Program**: CamCraft
- **Smallest steps to reproduce**:
  1. `load_machine {"builtin": "Sodick ALC400G"}` (note also: `post_offer` suggests `"Generic ISO Wire EDM"` rather than built-in `"Sodick Wire EDM (Mark 21)"`).
  2. `import_model {"path": "/home/mikekinney/qa/runs/20261122-q06-wire-plasma/q06-die.step"}`
  3. `set_stock {"box": {"dx": 80, "dy": 60, "dz": 20}}`
  4. `add_tool {"catalog": "EDM-W025"}`
  5. `add_operation {"kind": "wire_contour", "name": "DieWireContour", "source": "q06-die", "tool": "EDM-W025"}`
  6. Even after patching `edm.conditions`, setting `taper_deg: 1.0` fails for missing `edm.lower_guide_z`/`edm.upper_guide_z`, and posting with `Generic ISO Wire EDM` fails for missing `edm.codes.{condition,comp_off,taper_on,taper_off,uv}`.
- **Expected**: Built-in wire EDM machines should include sensible default conditions, guide Z heights, and control codes (or `Sodick ALC400G` should offer `"Sodick Wire EDM (Mark 21)"` and work immediately).
- **Actual**:
  - `add_operation: invalid parameters: a Wire Contour names no EDM condition - choose one of machine Sodick ALC400G's conditions (it states none - add edm.conditions to the machine file)`
  - `edit_operation: invalid parameters: a 1.000 degree taper is asked, and machine Sodick ALC400G states no edm.lower_guide_z and edm.upper_guide_z - a tapered wire's guide positions are read from them; set the taper to 0`
  - `post: post blocked - 8 warning(s): machine Sodick ALC400G states no edm.codes.condition - the wire post never invents an EDM code; machine Sodick ALC400G states no edm.codes.comp_off ...`
  - `AGIECUT Classic 2S (template)` likewise has no `edm.conditions`, guide Z heights, or `edm.codes`.
- **Severity**: High — blocks any user from creating or posting a wire EDM operation using built-in machines without manually authoring undocumented machine JSON patches.
- **Repro files**: `q06-die.step`, `q06-die.camproj`.

### 2. `list_operation_types` omits required and essential optional fields for `wire_contour` and `plasma_profile`
- **Program**: CamCraft
- **Smallest steps to reproduce**:
  1. `list_operation_types {}`
  2. Inspect `wire_contour` (lists only `bottom_z`, `side`, `top_z`) and `plasma_profile` (lists only `bottom_z`, `clearance_z`, `comp`, `lead_in`, `lead_out`, `overburn`, `reversed`, `small_hole_dia`, `small_hole_feed`, `top_z`).
  3. Call `edit_operation {"name": "DieWireContour", "params": {"__probe__": 1}}`.
- **Expected**: `list_operation_types` should document every accepted field and its type/schema (including optional ones) for `wire_contour` (`condition`, `taper_deg`, `start`, `skims`, `tabs.{count,width,at,cutoff_condition}`, `capture`) and `plasma_profile` (`feed`, `kerf`, `pierce_delay`, `pierce_height`, `cut_height`). Similarly, `edit_machine` should document `edm.*` keys instead of pointing to `get_job` where absent keys are not shown.
- **Actual**: `list_operation_types` omits all of those fields (including the *mandatory* `wire_contour.condition`!) and adds only `"note": "Fields absent from the defaults are optional and off; set them in a params patch to turn them on."`, while `edit_operation` fails with `"unknown field ...; see list_operation_types"`. Without trial-and-error probing, a user cannot even set plasma `feed` or `pierce_delay`, or wire `taper_deg`, `start`, `skims`, `tabs`, or `capture`.
- **Severity**: High — confuses users and effectively hides core CAM functionality (including required fields and plasma feed/pierce controls).
- **Repro files**: `q06-die.camproj`, `q06-bracket.camproj`, `q06-bracket-default.nc` (missing `F` and `M207/M208/M209`).

### 3. `wire_contour` refuses skim passes on tapered (or tabbed) cuts
- **Program**: CamCraft
- **Smallest steps to reproduce**:
  1. Open `/home/mikekinney/qa/runs/20261122-q06-wire-plasma/q06-die.camproj` (or any `wire_contour` with `taper_deg: 1.0`).
  2. `edit_operation {"name": "DieWireContour", "params": {"skims": [{"condition": "C002"}], "taper_deg": 1.0}}`
- **Expected**: Tapered die openings (the canonical wire EDM application) should support rough + skim passes in one operation without cutting/re-threading the wire between passes, and allow leaving rough stock for skims.
- **Actual**: Refused with:
  `edit_operation: invalid parameters: wire contour: 1 skim pass(es) are stated with a 1.000 degree taper - skims are cut on a vertical, untabbed one-profile contour only; remove the skims or that setting`
  (Same refusal with tabs: `wire contour: 1 skim pass(es) are stated with tabs - skims are cut on a vertical, untabbed one-profile contour only; remove the skims or that setting`).
  Working around it with a second `wire_contour` op forces `(CUT THE WIRE) M00` + `(THREAD THE WIRE AT THE START HOLE) M00` between rough and skim passes.
- **Severity**: Medium-High — breaks standard die-making wire EDM workflow and produces unnecessary wire-cut/rethread cycles in workaround NC.
- **Repro files**: `q06-die.camproj`, `q06-die.nc`.

### 4. `wire_contour` refuses tabs on tapered cuts
- **Program**: CamCraft
- **Smallest steps to reproduce**:
  1. Open `/home/mikekinney/qa/runs/20261122-q06-wire-plasma/q06-die-step-blocked.camproj`.
  2. `edit_operation {"name": "DieWireContour", "params": {"taper_deg": 1.0, "tabs": {"count": 1, "width": 1.0}}}`
- **Expected**: Tapered die cuts should allow hold-back tabs (glue stops) to hold the slug until cut-off.
- **Actual**: `edit_operation: invalid parameters: wire contour: tabs can't be combined with a taper yet (a 1.000 degree taper is asked) - remove the tabs or set the taper to 0`
- **Severity**: Medium — forces slug capture instead of glue-stop tabs on all tapered cuts.
- **Repro files**: `q06-die-step-blocked.camproj`.

### 5. Straight-opening die STEP with CAM 1° taper is blocked by wire gouge & slug post gate
- **Program**: CamCraft
- **Smallest steps to reproduce**:
  1. Load `Sodick ALC400G` (with `edm.conditions`, `lower_guide_z:-25`, `upper_guide_z:45`), `import_model` `q06-die-straight.step`, `set_stock` 80×60×20, `add_tool` `EDM-W025`.
  2. Add `wire_contour` with `side:"die", condition:"C001", taper_deg:1.0, start:[0,0], capture:true`, pick bottom/top opening chains, `generate`, `verify` / `post`.
- **Expected**: Either allow a user to program CAM relief taper onto a straight CAD opening (standard wire EDM practice) with a reviewable notice, or clearly document that CAD must carry the draft angle.
- **Actual**: Post blocked:
  `DieWireContour: wire gouge: the kerf of move 1 enters the part by 0.3491 mm near X0.000 Y-10.189 at Z0.001 (120 move(s) gouge; the deepest reading at any height is 0.3493 mm)`
  `DieWireContour: wire slug: the piece freed near X0.000 Y2.000 at Z0.001 holds 2.893 mm² of the part - a die keeps the material outside the cut, but here the part drops`
- **Severity**: Medium — prevents standard CAM-driven draft/relief tapering on straight die drawings.
- **Repro files**: `q06-die-straight.step`.

### 6. Exact 1° tapered die STEP + matching CAM 1° taper is STILL blocked at post gate by 0.0004–0.0046 mm STEP tessellation chord error
- **Program**: CamCraft
- **Smallest steps to reproduce**:
  1. `open_project {"path": "/home/mikekinney/qa/runs/20261122-q06-wire-plasma/q06-die-step-blocked.camproj"}` (which uses `q06-die.step`, modeled with exact 1° outward draft downward, and `wire_contour` with `taper_deg: 1.0`, `capture: true`).
  2. `verify {}` or `post {"output": "..."}`
- **Expected**: An exact 1° tapered wire cut on an exact 1° tapered STEP model should pass verification and post without gouge errors.
- **Actual**: Post is blocked:
  `DieWireContour: wire gouge: the kerf of move 2 enters the part by 0.0004 mm near X12.011 Y-10.349 at Z0.001 (118 move(s) gouge; the deepest reading at any height is 0.0046 mm)`
  CamCraft's STEP mesher discretizes the top R4 arc into 48 edges, the bottom R4 arc into 42 edges, and each R3 90° corner into 16 segments (sagitta `3.3491*(1-cos(2.8125°)) = 0.0040 mm`), while the wire gouge gate rejects gouges down to `0.0001 mm` (even `taper_deg: 0.98` failed with `0.0001..0.0010 mm`). Posting only succeeded after falsifying `taper_deg` down to `0.95°` (wrong part geometry!).
- **Severity**: Critical — makes it impossible to post an exact tapered wire EDM cut on a curved STEP model without deliberately posting the wrong taper angle (`0.95°` instead of `1.0°`).
- **Repro files**: `q06-die.step`, `q06-die-step-blocked.camproj`, `q06-die.camproj`.

### 7. `Sodick Wire EDM (Mark 21)` post refuses all tapered cuts
- **Program**: CamCraft
- **Smallest steps to reproduce**:
  1. `open_project {"path": "/home/mikekinney/qa/runs/20261122-q06-wire-plasma/q06-die.camproj"}`
  2. `post {"output": "/home/mikekinney/qa/runs/20261122-q06-wire-plasma/test.nc", "post": "Sodick Wire EDM (Mark 21)"}`
- **Expected**: The Sodick post for the 4-axis `Sodick ALC400G` (which advertises U/V axes and `max_taper_deg: 25.0`) should post tapered wire EDM G-code.
- **Actual**: Refused:
  `post: post blocked - 2 warning(s): operation DieWireContour: is tapered - the Sodick wire post writes a vertical 2-axis cut only (no G51 / G52 or A words); operation DieWireSkim: is tapered - the Sodick wire post writes a vertical 2-axis cut only (no G51 / G52 or A words)`
- **Severity**: High — cannot post tapered cuts to the Sodick post even though CamCraft ships a 4-axis Sodick ALC400G machine.
- **Repro files**: `q06-die.camproj`.

### 8. DXF `wire_contour` rejects arcs/circles, rejects taper/skims/tabs, and rejects setups with a STEP model or >1 DXF
- **Program**: CamCraft
- **Smallest steps to reproduce**:
  1. On a wire EDM setup with `q06-die.dxf` (`import_dxf`), assign `sketch_chain:...` via `set_operation_geometry` and call `generate {}`.
  2. With a straight-segment DXF (`q06-die-poly.dxf`) and `taper_deg: 1.0`, call `generate {}`.
  3. With a STEP model also present in the setup, call `generate {}`.
- **Expected**: DXF wire EDM should support standard R12 `ARC`/`CIRCLE` contours and tapered/skim/tabbed cuts.
- **Actual**:
  - Arcs in DXF: `generate: DieWireContour - could not generate the toolpath: DieWireContour: invalid machining parameters: the selected DXF chain contains an arc or circle; the generic wire path requires straight segments`
  - Taper on polyline DXF: `generate: DieWireContour - could not generate the toolpath: DieWireContour: invalid machining parameters: selected DXF wire cuts require a single vertical pass with slug capture and no tabs`
  - STEP model present: `selected DXF wire cuts cannot verify a separate part, fixture or machine table in the wire span`
  - Two DXFs in setup: `selected DXF wire cuts can check only one placed drawing in the setup`
  - Without explicit `set_operation_geometry`: `a toolpath from the chains of a DXF profile is not generated yet`
- **Severity**: High — severely limits DXF-based wire EDM (almost all real dies have corner radii/arcs, taper, skims, or tabs).
- **Repro files**: `q06-die.dxf`, `q06-die-poly.dxf`.

### 9. CamCraft `import_dxf` refuses AutoCAD `CLASSES` section written by SolidCraft `file.export_view_dxf`
- **Program**: CamCraft
- **Smallest steps to reproduce**:
  1. In SolidCraft, export `q06-punch.sc` via `file.export_view_dxf` to `/home/mikekinney/qa/runs/20261122-q06-wire-plasma/q06-die-lines.dxf`.
  2. In CamCraft (with stock set), call `import_dxf {"path": "/home/mikekinney/qa/runs/20261122-q06-wire-plasma/q06-die-lines.dxf"}`.
- **Expected**: CamCraft's DXF importer should skip standard non-geometry sections like `CLASSES` and import the `ENTITIES`.
- **Actual**: `import_dxf: DXF import: DXF CLASSES: unsupported SECTION`
- **Severity**: Medium — breaks SolidCraft-to-CamCraft DXF workflow whenever `file.export_view_dxf` is used (and any AutoCAD 2000+ DXF with a `CLASSES` section).
- **Repro files**: `q06-die-lines.dxf`.

### 10. No MCP tool to create a new project or delete a placed DXF profile (traps setups permanently)
- **Program**: CamCraft
- **Smallest steps to reproduce**:
  1. `import_dxf {"path": "/home/mikekinney/qa/runs/20261122-q06-wire-plasma/q06-die.dxf", "name": "q06-die (2)"}`
  2. `delete_model {"name": "q06-die (2)"}`
  3. Add `Setup2`, move/delete all ops from `Setup1`, then `delete_setup {"name": "Setup1"}`.
- **Expected**: There should be a `new_project` tool (like SolidCraft's `new_design`) and a way to delete a placed DXF profile (`delete_profile` or via `delete_model`).
- **Actual**:
  - `delete_model: 'q06-die (2)' is not a model`
  - `delete_setup: invalid parameters: the DXF profile 'die_poly' is placed in 'Setup1' - delete it first`
  - No `new_project` or `delete_profile` MCP tool exists, so once a DXF profile is imported it can never be removed (except by `undo` if immediately undone) and leaks into subsequent projects saved in the same session (note `q06-die (2)` persisted into `q06-bracket.camproj`).
- **Severity**: Medium — causes project contamination and permanent setup lock-in during multi-part sessions.
- **Repro files**: `q06-bracket.camproj` (contains orphaned `q06-die (2)` profile).

### 11. `wire_contour` hard-refuses sharp concave corners on outside punch cuts with misleading advice
- **Program**: CamCraft
- **Smallest steps to reproduce**:
  1. Load `Sodick ALC400G` (with `C001` gap `0.035`), `import_model` `q06-punch-sharp.step`, `set_stock` 50×40×20, `add_tool` `EDM-W025`.
  2. `add_operation {"kind": "wire_contour", "name": "PunchWireContour", "source": "q06-punch", "tool": "EDM-W025", "params": {"side": "punch", "condition": "C001", "start": [0, -15]}}`
  3. `generate {}`
- **Expected**: Warn that sharp concave corners will retain the wire+gap radius (`0.160 mm`) and generate the path, rather than hard-refusing (or at least don't suggest `"cut with a thinner wire"` for a radius-0 corner, since no wire has radius 0).
- **Actual**: `generate: PunchWireContour - could not generate the toolpath: PunchWireContour: invalid machining parameters: wire contour: an inside corner or slot of the profile near X4.000 Y10.000 is tighter than the wire can cut on the punch side - the minimum inside radius is 0.1600 mm (wire radius 0.1250 + spark gap 0.0350), and the kerf would leave 0.0614 mm standing there; round it to at least that radius, or cut with a thinner wire`
- **Severity**: Medium — blocks wire EDM on any punch with sharp internal corners unless the user modifies the CAD model.
- **Repro files**: `q06-punch-sharp.step`.

### 12. Tabbed punch wire EDM retraces entire ~100 mm perimeter backwards with rough power `C001` still active, plus double `M00` and unnecessary cut/rethread
- **Program**: CamCraft
- **Smallest steps to reproduce**:
  1. `open_project {"path": "/home/mikekinney/qa/runs/20261122-q06-wire-plasma/q06-punch.camproj"}`
  2. `post {"output": "/home/mikekinney/qa/runs/20261122-q06-wire-plasma/q06-punch.nc"}`
  3. Inspect lines 390–789 of `q06-punch.nc`.
- **Expected**: After reaching the tab at `X23.84 Y9.84`, the wire should lead out to the start hole `(25, 5)` (or retract with generator off / flush-only), pause once (`M00`) for slug securing, then sever the 2 mm tab without cutting/rethreading at the exact same start hole.
- **Actual**: At `X23.84 Y9.84` (line 392), the wire reverses and retraces all ~360 moves backwards around the entire ~100 mm punch perimeter back to `X26.32 Y9.84` (lines 393–765) with rough cutting condition `C001` still active (no power-off code), then cuts a 0.16 mm stub `X26.16 -> X26.32`, returns to `X25. Y5.`, outputs `(CUT THE WIRE) / M00 / M00` (double `M00` at lines 770–771!), and re-threads at the exact same `X25. Y5.` for the tab cut-off.
- **Severity**: High — posts questionable/bad wire EDM G-code: a 100 mm reverse retrace under active rough generator `C001` risks secondary discharge / overcut on the finished punch walls, doubles cycle time, and adds unnecessary cut/rethread + duplicate `M00`.
- **Repro files**: `q06-punch.camproj`, `q06-punch.nc`.

### 13. `edit_operation` silently ignores unknown fields inside `wire_contour.skims[]`
- **Program**: CamCraft
- **Smallest steps to reproduce**:
  1. On any vertical `wire_contour` op, call `edit_operation {"name": "DieWireContour", "params": {"skims": [{"condition": "C002", "offset": {"__x": 1}, "__x": 1}]}}`.
- **Expected**: Reject unknown keys inside `skims[]` with an error (consistent with top-level unknown-field rejection).
- **Actual**: Succeeds (`status: "stale"`) and silently drops `offset` and `__x`.
- **Severity**: Low-Medium — could silently discard a user's intended skim offset/allowance without warning, resulting in wrong part size.
- **Repro files**: `q06-die.camproj`.

### 14. `plasma_profile` has no tabs (microjoints) support
- **Program**: CamCraft
- **Smallest steps to reproduce**:
  1. `open_project {"path": "/home/mikekinney/qa/runs/20261122-q06-wire-plasma/q06-bracket.camproj"}`
  2. Try `edit_operation` with `tabs`, `tab_count`, `tab_width`, `microjoints`, `microjoint`, `bridges`, or `overburn: -2`.
- **Expected**: `plasma_profile` should support tabs / microjoints on outer profiles (as `contour` and `wire_contour` do) to prevent finished parts from tipping up or falling into the water table.
- **Actual**: All tab/microjoint field names are rejected as `unknown field`, and `overburn: -2` is rejected with the vague error `"edit_operation: invalid parameters: operation parameters are not usable for a Plasma Profile operation"`.
- **Severity**: Medium-High — missing core plasma CAM feature; parts drop or tip up into the torch on outer cut completion.
- **Repro files**: `q06-bracket.camproj`, `q06-bracket.nc`.

### 15. Posted NC contains massive numbers of duplicate consecutive zero-length `G1`/`G01` moves (30.4% of plasma file, 10.3% of punch file)
- **Program**: CamCraft
- **Smallest steps to reproduce**:
  1. Post `q06-bracket.camproj` -> `q06-bracket.nc` (1671 lines) and `q06-punch.camproj` -> `q06-punch.nc` (789 lines).
  2. Run `uniq -d q06-bracket.nc | wc -l` and `uniq -d q06-punch.nc | wc -l`.
- **Expected**: Post-processed G-code should deduplicate zero-length moves after coordinate rounding and fit clean arcs rather than emitting hundreds of repeated identical `G1` lines.
- **Actual**: `q06-bracket.nc` has **508** consecutive duplicate lines out of 1671 (30.4% — including triple repeats such as `G1 X19.551 Y130.74` three times in a row at lines 1653–1655); `q06-punch.nc` has **81** consecutive duplicate lines out of 789 (10.3%, e.g., `G01 X37.549 Y9.889` twice at lines 26–27).
- **Severity**: Medium — bloats NC programs and on plasma controllers can cause motion-planner deceleration/stutter, dross/dwell divots, and THC voltage spikes.
- **Repro files**: `q06-bracket.nc`, `q06-bracket-default.nc`, `q06-punch.nc`.

### 16. Plasma verifier hides 4 tip-up warnings inside `gate.not_judged` while reporting `ok: true`, `findings: []`, `warnings: []`, and default cut order causes 4 tip-up rapid crossings
- **Program**: CamCraft
- **Smallest steps to reproduce**:
  1. `open_project {"path": "/home/mikekinney/qa/runs/20261122-q06-wire-plasma/q06-bracket.camproj"}`
  2. `verify {}`
- **Expected**: (a) Tip-up warnings should be reported in `findings[]` and `operations[0].warnings[]`, not buried inside a `gate.not_judged` prose string while `findings: []` and `warnings: []`; (b) contour ordering on a simple 7-hole bracket should avoid rapiding straight across the Ø40 hole `C2` twice (`C3→C4` and `C4→C5`) and across `C1`/`C6`.
- **Actual**: `verify` returns `"ok": true, "findings": [], "operations": [{"warnings": []}]`, while `"gate": {"not_judged": "... 4 warning(s) - BracketPlasma: plasma tip-up: the rapid into contour C2 crosses the slug of hole C1, already cut free near (81.84, -36.43) mm; BracketPlasma: plasma tip-up: the rapid into contour C4 crosses the slug of hole C2, already cut free near (0.00, 0.00) mm; BracketPlasma: plasma tip-up: the rapid into contour C5 crosses the slug of hole C2, already cut free near (4.17, 13.01) mm; BracketPlasma: plasma tip-up: the rapid into contour C7 crosses the slug of hole C6, already cut free near (79.39, 38.01) mm"}`.
- **Severity**: Medium-High — could cause torch-to-tipped-slug collision on the plasma table because warnings are invisible in standard `findings`/`warnings` fields and the path planner doesn't route around cut holes.
- **Repro files**: `q06-bracket.camproj`, `q06-bracket.nc`.

### 17. DXF `plasma_profile` cannot be generated at all despite being advertised by `add_operation`
- **Program**: CamCraft
- **Smallest steps to reproduce**:
  1. Set up `Tormach 1300PL`, stock 220×140×6, tool `PT-KERF-150`, `import_dxf {"path": "/home/mikekinney/qa/runs/20261122-q06-wire-plasma/q06-bracket.dxf", "name": "bracket_dxf"}`.
  2. `add_operation {"kind": "plasma_profile", "name": "BracketDxfPlasma", "source": "bracket_dxf", "tool": "PT-KERF-150"}`
  3. `generate {"name": "BracketDxfPlasma"}`
  4. `set_operation_geometry {"operation": "BracketDxfPlasma", "picks": [{"model": "bracket_dxf", "id": "sketch_layer:CUT"}]}`
- **Expected**: Because `add_operation` explicitly documents DXF profile support for `"2D milling, plasma and wire EDM kinds"`, `plasma_profile` on a valid DXF (`8 closed, 0 open` on `CUT`/`OUTLINE`) should generate cleanly.
- **Actual**: Catch-22 refusal:
  - `generate`: `this operation is checked against the stock 'Setup2/Stock' and the chains of the DXF profile 'bracket_dxf' (8 closed, 0 open) - there is no solid model; a toolpath from the chains of a DXF profile is not generated yet`
  - `set_operation_geometry`: `'bracket_dxf' is a DXF profile - an operation takes the whole profile as its source; picked chains of a profile are not supported yet`
- **Severity**: Medium — breaks DXF-driven plasma cutting completely despite schema advertising support.
- **Repro files**: `q06-bracket.dxf`.

---

## Confusing Tool Names, Parameters, or Behaviors (Even When They Worked)

1. **CamCraft `wire_contour.start` coordinate frame**: `start: [x, y]` is interpreted in **world** coordinates rather than **WCS** coordinates (e.g., on `q06-punch`, WCS origin is `[-25, -20, 0]`, `start: [0, -15]` posted as `G00 X25. Y5.`). This is undocumented and surprising when a machinist sets WCS at the plate corner.
2. **CamCraft `wire_contour.capture`**: The error message says `"state that the slug is captured"`, but the actual boolean field name is simply `capture` (not `slug_captured` or `slug`), and it is absent from `list_operation_types`.
3. **CamCraft `plasma_profile` defaults omit `F` and `M207`/`M208`/`M209`**: Leaving the hidden `feed`, `pierce_height`, `cut_height`, and `pierce_delay` fields unset causes PathPilot Plasma to post zero `F` words (relying silently on controller AutoFS even though `PT-KERF-150` defines `feed: 2500.0` and `small_hole_feed: 0.65`). Once set explicitly, PathPilot Plasma correctly outputs `M207 P3.8`, `M208 P1.5`, `M209 P0.6`, `F1430` (65% small-hole feed), and `F2200`.
4. **CamCraft `Tormach 1300PL` machine units vs document units**: `get_job` shows `machine.definition.units: "inch"` while document `units: "mm"`; posting still correctly emitted `G21` with mm coordinates and mm `M207`/`M208` heights, but the mismatch is confusing.
5. **CamCraft `Generic ISO Wire EDM` vs `Sodick Wire EDM (Mark 21)` formatting**: `Generic ISO Wire EDM` uses `%`, `G0`/`G1`, `M30`, while `Sodick Wire EDM (Mark 21)` omits `%`, uses `G00`/`G01`, `M02`. Both emit decimal points on integer coordinates (`X0. Y0.`, `X25. Y5.`).
6. **SolidCraft `sketch.line` vs `sketch.rectangle.two_point` / `sketch.slot.overall` parameter names**: `sketch.line` takes `points: [[x0,y0], [x1,y1], ...]` while `sketch.rectangle.two_point` and `sketch.slot.overall` take `p0` and `p1`. Minor API inconsistency; error message (`points must be a list`) made it easy to fix.

---

## G-Code Verification Summary (Step 5)

- **Wire Die (`q06-die.nc`, Generic ISO Wire EDM, 274 lines)**:
  - Start hole: `G0 X0. Y0.` (decimal points present).
  - Taper words & sign: `G51` / `G50` with 4-axis `U{u} V{v}` (`G1 X0. Y-10.586 U0. V1.161`, `G1 X15.586 Y-7.013 U-1.161 V0.026`). Verified mathematically for guide span `Z = -25..+45` (70 mm) and workpiece `Z = 0..20`: at `Z=20` wire center is `Y = -9.840` (`0.160` inside `Y = -10.000`), and at `Z=0` wire center is `Y = -10.172` (wider at the bottom by `20*tan(0.95°) = 0.332 mm`). Sign is **correct** (wider at bottom), though angle had to be set to `0.95°` instead of `1.0°` due to Problem 6.
  - Offset side: die/inside (`0.1600 mm` rough `C001`, `0.1400 mm` skim `C002`), computer-compensated (`G40`).
  - Skim count: 1 skim pass (`DieWireSkim`, `C002`) via second-op workaround due to Problem 3.
- **Wire Punch (`q06-punch.nc`, Sodick Wire EDM Mark 21, 789 lines)**:
  - Start hole: `G40 G00 X25. Y5.` (exact WCS transform of world `[0, -15]` from corner WCS `[-25, -20, 0]`, decimal points present).
  - Offset side: punch/outside (`0.1600 mm` outside `Y=10` -> `Y9.84`, outside `X=40` -> `X40.16`, outside `Y=30` -> `Y30.16`), computer-compensated (`G40`).
  - Hold-back tab: 2.0 mm tab at `X25. Y10.` -> wire gap `X23.84` to `X26.16` (`2.320 mm = 2.0 + 0.32 kerf`), severed by second op `PunchWireContour tab cut-off`. Flawed by full reverse retrace under `C001`, double `M00`, and 81 duplicate `G01` lines (Problems 12 & 15).
- **Plasma Bracket (`q06-bracket.nc`, Tormach PathPilot Plasma, 1671 lines; `q06-bracket-comp.nc`, 1463 lines)**:
  - Torch on/off & pierce: `M200`, `M207 P3.8` (pierce height), `M208 P1.5` (cut height), `M209 P0.6` (pierce delay), `G15` (IHS), `G16` (pierce/cut), `M205` (torch off), `G30` (retract).
  - Cut order: all 7 inner cutouts (`C0, C1, C5, C6` Ø10 holes, `C2` Ø40 hole, `C3, C4` 12×30 slots) cut **before** outer profile `C7`.
  - Lead-ins/lead-outs & kerf side: all hole/slot lead-ins pierce inside the scrap slug (Ø10 holes & slots pierce at exact center; Ø40 `C2` arcs in/out inside hole); outer `C7` arcs in/out outside at `Y > 130.75`. Computer kerf `0.75 mm` verified on every feature (`C2` Ø38.48 + 1.5 = Ø40; `C3/C4` width 10.50 + 1.5 = 12; `C7` `Y130.75` = 130 + 0.75); control comp (`q06-bracket-comp.nc`) emits correct `G41.1 D1.5` / `G40` (CCW holes + CW outer = left is scrap on both).
  - Tabs: none (unsupported, Problem 14). Duplicate `G1` lines: 508 (Problem 15).

RESULT: 17 problems (17 CamCraft, 0 SolidCraft); end-to-end: posted
