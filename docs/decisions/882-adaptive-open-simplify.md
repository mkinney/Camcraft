## #882 - Thinning the outline before the growth rings' opening: held, not landed

**What was decided.** The fix is written and bounded (`adaptive/simplify.rs`,
`rings::open`), but it is **held on the branch, not proposed for merge**.
It cuts the user's STEP bore from 115 s to 2.2 s. It also changes which
plan four other fixtures get, and two of those changes are worse output.
The planner has to be made robust to a sub-micron change of its rings
first; that is three other pieces of work, named below.

**What the fix is.** `rings::open` thins its area's outline with
Douglas-Peucker at `SIMPLIFY` = `ARC_TOL` (0.25 um), then erodes
`INSET` = 0.51 um deeper than it dilates back. Every dropped vertex is
within the tolerance of the chord that replaces it, so the thinned area
shrunk by the tolerance lies inside the original; the extra erosion also
covers the erosion's own arc flattening at reflex corners. The opening
therefore never leaves the area it opens. Measured on a keyhole with an
island (6000- and 3000-chord arcs): no boundary sample outside the area,
0.75 um off the plain opening on walls and square corners, 2.5 um where
two walls converge on a neck the opening closes.

**Where the time really goes.** Not where #882 says. The bore's outline
reaches the planner with 287 and 445 vertices, not thousands. The ~4,700
vertices a trial are made by the planner itself: every offset and every
opening adds join vertices to a smooth ring, and `piece` only merges
those closer than 10 um. Thinning the area once at its source
(`slit::open_islands`) leaves every coarse fixture bit-identical and the
bore at 115 s. The thinning has to be at the opening, every trial.

**Measured** (release, this Mac, upstream/main f490458a; the whole test
`app pocket3d_hsm::the_users_bore_roughs_adaptive_level_by_level_and_verifies_clean`):

| thinning at the opening | bore test | 60 x 14 slot | island 60 x 40 | rotary pins |
|---|---|---|---|---|
| none (main) | 115.6 s | HSM, 15.7 % load | refused by #850 budget, rings | pass |
| 16 nm (`ARC_TOL` / 16) | 12.2 s | **falls to Offset rings** | refused, rings | 2 fail |
| 62 nm (`ARC_TOL` / 4) | 4.2 s | **falls to Offset rings** | **posted at 94.6 % load** | 1 fails |
| 0.25 um | 2.2 s | **falls to Offset rings** | **posted at 94.6 % load** | 1 fails |
| 0.25 um + 0.51 um inset (the held fix) | 2.2 s | **posted at 73.7 % load** | refused after 136 revolutions | 2 fail |
| 0.51 um inset alone, no thinning | - | **falls to Offset rings** | refused, rings | 1 fails |
| 0.25 um, on PR #953 | 2.0 s | **falls to Offset rings** | refused, rings | 1 fails |

The slot and channel spend 3-33 % fewer growth vertices in every thinned
row (slot 252 k -> 170-244 k).

**Why it is held.** A 16 nm change of the rings is enough to change the
slot's strategy, so no tolerance is small enough to be neutral. Three
decisions in the planner sit on a knife edge:

1. *The microlift acceptance* (`lift.rs`, `lift_returns`): the lifted
   plan is kept only if its worst reading is not above the spiral's own.
   Both are read on a raster in half-degree steps. On main the slot's
   spiral reads 48.0 and the lifted plan 48.0, so it is kept; perturbed,
   the spiral reads 47.5, the lifted plan is dropped, the unlifted spiral
   fails the travel law and the pocket falls to Offset rings.
2. *The #850 geometry budget* refuses the island by vertex count. With
   fewer vertices a trial the island is no longer refused, and its plan
   is posted with "no step held the load": 94.6 % of D against a 14.6 %
   optimal load, with crossings and rapids in stock in the verifier. That
   is #845; main is protected from it only by the budget. PR #953 removes
   this case in the one run measured, but not the slot's or the rotary's.
3. *The rotary pins* (`rotary_adaptive`) name which developed levels are
   refused for travel; they flip level by level under every variant, and
   one variant cuts a level by rings without a note naming it.

Landing the fix now would trade a slow bore for an overloaded cut or a
lost HSM plan on parts that work today. The Prime Directive decides it.

**Considered and dropped.**
- *Thin only fine outlines* (drop at least half the vertices, else leave
  the area untouched): the planner's own rings pass that gate, so the
  slot and island change anyway (slot posted at 75.6 % load).
- *Thin the growth area once at its source*: bit-identical fixtures, no
  speed-up (see above).
- *A smaller tolerance*: 16 nm still flips the slot, and gives 12 s.
- *Re-pin the fixtures to the new outcomes*: two of them are worse
  output, not a neutral change.
- *Recalibrate `GEOM_MAX` to keep refusing the island*: it would hide
  #845 behind a second number instead of fixing it.

**What has to land first.** #845 (PR #953), a tolerance on the microlift
acceptance, and a named note for every rotary level cut by rings. After
those, re-measure the table above and take the 0.25 um + inset row.

**Also found.** The bore test is red on upstream/main f490458a on this
Mac before any change: two levels (Z 20 and Z 10) fall to rings for
"crosses itself", where the test pins one. Its pinned counts were not
reached, so they could not be re-measured here.

**Cost to change later.** `open` is one function; the held fix is two
files and 30 lines in `rings.rs`, with four unit tests that pin the
bound and the cost on their own fixtures.
