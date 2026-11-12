## #982 - Microlift plan: re-cut on fine chords, not a looser acceptance (HELD)

**Status: held work, escalated to effort:max.** The fine retry below
keeps the slot's plan under every perturbation and holds the planner's
bound, but it also rescues the r6.2 neck levels of `rotary_adaptive`,
and those plans measure 20.3 % of D on the dexel stock (the suite's law:
<= 19.8 %) with 3 in-cut breaks past 5 degrees at radius 5.2. So
`rotary_adaptive::the_r62_neck_levels_are_hsm_planned_under_the_load_and_kink_laws`
is red on this branch: the change cannot land until the rotary bands
keep a refusal of their own (or are planned under the dexel law) - see
the issue.

**What was decided.** `lift_returns` keeps its strict acceptance (the
lifted plan's worst reading may not exceed the spiral's or the bound).
When the lifted plan fails it on its worst reading, the plan is built
once more with every piece on its own fine chords - the spiral's, no
coarsening (`lift(.., coarse: false)`) - and that plan is kept if it
holds the same law. Only then is the bare spiral posted. No tolerance
was added.

**Why not the tolerance the issue proposed.** The reading is a count of
front-half rim samples (`Raster::engage`, 360 over 180 degrees), so
every reading is a whole 0.5 degree step; there is no unquantized
reading to compare. On the 60 x 14 slot the bound is 47.5014 degrees.
Swept in release (the opening eroded 0 / 16 / 32 / 62 / 125 / 250 /
510 nm deeper than it dilates back), the spiral read 47.5 every time and
the lifted plan 47.5 every time but at 32 nm, where it read 48.0 - one
sample over the bound, i.e. 15.71 % of D against the 15.38 % bound. A
one-sample tolerance (tried first, `accepts(lifted, spiral, bound, cap)`
with the cap as ceiling) keeps the slot but posts that 48.0 plan as a
named overload, and on main it flips `rotary_adaptive`'s r6.2 neck level
from refused to posted at 48.0 against 47.5 - a plan whose engagement
exceeds the stated bound, which the Prime Directive forbids. Dropped.

**What the 48.0 really was.** Instrumented, the over-bound reading is
one growth piece cut in the spiral's own order (every piece it depends
on already cut), reached by a lifted return. Riding the spiral's own
air onto it instead changes nothing (the ride reads 11-13 degrees, the
piece still 48.0). Turning `coarsen` off removes the over-bound reading
at every perturbation. So the coarse chords (`thin_turn`, 0.01 mm sag,
4 degree turns) of the earlier pieces each read within the bound
themselves but leave a sliver a cell wide that a later piece eats - a
real, measured engagement of one sample more, which a few nanometres of
ring geometry can make or unmake. The fine chords are the spiral's own,
so a plan cut on them leaves what the spiral leaves.

**Effects measured (release, `--include-ignored`).** motion lib 191 + the
new pin pass; `adaptive_hsm` 6/6 and `adaptive_band` 2/2 unchanged (two
`adaptive_hsm` levels try the fine plan, read 48.0 against 47.5 and keep
their verdict: spiral posted). `rotary_adaptive`: the r6.2 bar's neck
levels 5.7 and 5.2, refused for out-of-cut travel and ring-cut since
#55, now read 47.5 on fine chords and are HSM-planned - no planner
overload note - but the dexel stock reads 20.3 % of D on both (past the
suite's 19.8 % law) and 3 in-cut breaks past 5 degrees at 5.2. That pin
is rewritten
(`the_r62_neck_levels_are_hsm_planned_under_the_load_and_kink_laws`)
and asserts the load (dexel-read, <= optimal + 35 %), zero in-cut breaks
past 5 degrees, posted arcs and no refusal for both levels. It is red: the
one fixture verdict that changes on main changes to a plan the dexel
measure condemns, which is why this work is held.

**Cost.** The second lift runs only when the first fails on its worst
reading (two `adaptive_hsm` levels and three rotary levels today); a
fine plan posts more moves than a coarse one.

**Considered and dropped.**
- *One-sample tolerance* (above): accepts a real overload, flips the
  rotary pin to a worse plan.
- *A share tolerance* (e.g. 2 % of the reading): absorbs several samples
  at high readings - more than any noise.
- *Sub-sample readings* (bisecting each arc end on the bitmap): still
  quantized by the grid cell (0.93 degrees at the slot's rim) and costs
  every walk.
- *Fixing `coarsen` to account for what it leaves for the next piece*:
  the right long-term repair (coarse chords everywhere they are safe),
  but a planner algorithm change; the fine retry is contained and keeps
  every decision the law's.

**Pins.** `adaptive::tests::the_slot_keeps_its_lifted_plan_under_sub_micron_perturbations`
(the slot lifted, nothing overloaded, no crossing, under openings eroded
16 nm, 32 nm and 0.25 um deeper - through a `cfg(test)`-only knob,
`rings::OPEN_INSET`) and the rewritten rotary r6.2 pin.
