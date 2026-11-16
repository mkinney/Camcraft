## #1914 - RichAuto DSP A11 3-axis router post (`richauto.cps`)

**What was decided.**

- Added clean-room `assets/posts/richauto.cps` and registered `"RichAuto DSP A11"` in `BUILTINS` in `crates/app/src/posting.rs` (`serves = ["mill"]`, `capabilities = 1`, `extension = "nc"`, `setCodePage = "ascii"`).
- Metric-only contract (`programUnits = "mm"`, `endUnits = "note"`): emits `G21 G90 G17 G40 G80` (without `G91.1`, `G94`, `G43`, `G49`, `G53`, `M6`, `%`, `;`, or `O` numbers), refuses `Units: "inch"` by name, and writes a single `(END UNITS NOT WRITTEN - RICHAUTO A11 IS MM ONLY)` note via `note()` when `EndUnits` is `"inch"`.
- Declared `SpindleDwellMs` (`type: "number"`, `min: 0`, `max: 2147483647`, `default: 0`) in `declaredOptions` for #258/#2190 option discovery; runtime `onOpen` rejects non-numeric/out-of-range values by name and emits `G04 P<ceil(ms)>` after `S<rpm> M3` when `> 0`.
- Enforced single-offset `G54` (`workOffsetIndex = true`, `workOffsetMax = 1`), single-tool continuity (allowing repeated same tool without `M6`, refusing tool `< 1`/non-integer, missing tool, or mid-program tool change), incremental `I`/`J`/`K` (`AbsCntr Off`) in `G17`/`G18`/`G19` with >180° arcs split into halves and `G17` restored at section end, expanded plain `G0`/`G1` moves for all drilling cycles (including floor `G04 P` enforcement and `BORE` spindle `M5`/rapid retract to `R`/restart safety), and named refusals for `TAP`/`FLOATINGTAP`, `COMP`, probing, `A`/`B`/`C` rotary moves (one warning per program), `SurfaceSpeed` (`G96`), live tooling, missing/sub-stock `ClearancePlane`, invalid spindle RPM, missing rapid start position, non-positive/unknown feed, and unsupported coolant (`off` silent; `flood` requires `M8` when `machine.coolants` is non-empty, silent when empty).
- Added unit/safety suite `crates/post/tests/script_richauto.rs` + `crates/post/tests/richauto_safety/mod.rs`, recorded goldens under `crates/post/tests/recorded/golden/richauto/`, and conformance goldens under `crates/app/tests/conformance.rs` + `crates/app/tests/golden/nc/{richauto/*.nc,synth_richauto.nc}`.

**Why.**

- RichAuto A11 DSP pendant workflows run standalone metric programs with relative arc centers (`AbsCntr Off`), single-tool manual setups, optional VFD warm-up dwell, and machine-dependent `Read G54` / `Ign G54` behavior. Expanding drilling cycles into verified `G0`/`G1` motion, refusing unsupported multi-tool/4-axis/tapping/comp constructs explicitly by name, and keeping `endUnits = "note"` prevents unsafe machine state or silent drops without breaking metric operation workflows.

**Considered and dropped.**

- Emitting canned cycles (`G81`-`G89`) or `G20` end units -> dropped: controller firmware and `AbsCntr`/`G54` variants make expanded plain moves + `endUnits = "note"` unambiguous and safe on hardware.
- Reading `SpindleDwellMs` without declaring it in `declaredOptions` -> dropped: leaving it undeclared would hide the setting from #258/#2190 UI/CLI/MCP option discovery and resolution.

**Cost to change later.**

- Localized to `assets/posts/richauto.cps`, one `BUILTINS` entry in `crates/app/src/posting.rs`, and the RichAuto golden/test fixtures; supporting multi-tool ATC variants (e.g. A18/A11 ATC) later would be a separate script or option.
