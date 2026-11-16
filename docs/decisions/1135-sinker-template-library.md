## #1135 - the empty sinker template joins the machine library

**Decided.** `assets/machines/sinker-edm.mch` ships in `posting::MACHINES`
as "Sinker EDM 3-axis (template)". Its `edm` section stays empty. The
sinker post "Sinker EDM (ISO)" is its only post.

**Why.** Every library machine needs a post. The post shipped with #542.
The user fills the codes from the control manual, and the post refuses by
name until then.

**Dropped.** Moving the test fixture (`crates/app/tests/edm/sinker-edm.mch`)
into the assets: about a dozen suites include it by path. The fixture stays
as a copy, and a test pins that it equals the shipped file. The wire
template stays a fixture: no issue asked for it.

**Cost to change.** Rename the picker label in one line of `posting.rs`.

## #1135 - the sinker template gets a ribbon mode row

**Decided.** `MACHINE_MODES` (`crates/ui/src/machine_modes.rs`) gets a
"Sinker EDM" row. The row mounts "Sinker EDM 3-axis (template)". The door
word is "Sinker". The guard test from #1501 stays as it is: every machine
kind in the built-in library has a mode row.

**Why.** This PR is the first to ship a `SinkerEdm` machine, so the guard
failed after the merge of main. The row works like the other rows: the
switch is the undoable `LoadBuiltinMachine`. The template post still
refuses by name until the user fills `edm.codes`
(`sinker_burn::the_shipped_sinker_template_lists_and_refuses_its_unfilled_codes_by_name`).

**Dropped.** Let the guard skip template entries. That makes the guard
weaker, and it hides a shipped kind from the ribbon.

**Cost to change.** Remove one table row and one `short_caption` arm.
