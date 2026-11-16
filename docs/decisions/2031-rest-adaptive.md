## #2031 - 2D rest Adaptive: plan engagement from proved remaining stock

**What was decided**

1. **A generator of its own, in the motion crate.**
   `camcraft_motion::machining2d::rest_adaptive(region, params, stock)`
   (`crates/motion/src/machining2d/adaptive/rest/`). It takes the pocket,
   the pocket's own parameters and a `RestStock`. The operation's
   parameter door (`rest2d::rest_strategy`) still refuses Adaptive by
   name: the operation only has an entered previous tool, which is no
   proof. Lifting that refusal is the work of the task that derives the
   stock from the earlier operations (#2033): it builds a `RestStock`
   and calls this generator.

2. **The stock contract.** A `RestStock` is a list of proved floors. A
   floor at depth `z` claims: inside the pocket, at and above `z`,
   material stands only inside `remaining` or `uncertain`. Below `z` it
   says nothing. For a cut depth, every floor at least that deep speaks,
   and the bound is the intersection of what they allow. There is no
   constructor from a tool diameter.

3. **Three guards, each with a pinning test that fails without it**
   (`adaptive/rest/tests.rs` runs the planner with one guard off):
   - stock: `uncertain` stands as material;
   - depth: a cut depth below every proved floor is refused by name
     before anything is planned;
   - entry: a plunge needs the whole footprint, plus a margin (0.1 mm or
     four grid cells), proved clear by the stock as it was given. What
     this operation's own passes cleared is never credited to a plunge:
     a column cleared at this depth says nothing about the depths above.
     The proof is asked twice, of the polygon and of the raster.

4. **Conservative raster.** The stock bound is grown by a cell's half
   diagonal (plus two offset tolerances) before it is sampled at cell
   centers, so every cell the bound touches stands
   (`adaptive/raster/conservative.rs`). Every load reading stands on this
   raster.

5. **A film is safety material, not a target.** A bound read large keeps
   a thin band along every wall the earlier tool rode. Residue thinner
   than `RestStock::film()` (default 0.02 mm, `rest2d`'s film) gets no
   pass planned for it. It still stands for every load reading and no
   plunge lands on it. Without this the planner laps every wall.

6. **The passes: the opened room.** The room is the legal center area
   minus the material grown by the tool radius less the step. It is
   opened (eroded, then dilated) by a disc whose radius falls phase by
   phase, by at most 15% and at most a step and a half, down to an eighth
   of the tool radius, then zero. A pass is the room's boundary where it
   bites. In a corner this gives arcs tangent to both walls; across a
   neck it gives half loops (trochoidal motion); a piece of material that
   touches no wall is not opened and is cut by closed rings.
   The first design rode the wall into the corner's wedge with a fixed
   0.5 r blend. It was dropped on measurement: a wedge 0.03 mm thick
   already reads 25 degrees of contact, and the second pass of a corner
   read past the bound at every step of the ladder.

7. **The load is proved or the pass is not cut.** Each candidate feed
   (lead and link included) is walked on the raster before it is kept.
   The bound is the optimal load plus the plain Adaptive's 5% planning
   margin (`PLAN_TOL`). There is no overload fallback (the plain Adaptive
   cuts up to twice the load and says so; the issue asks for a refusal).
   What the last, sharp phase cannot cut is left standing; if that is
   more than 2% of the rest material in reach, the operation is refused
   and the refusal names the load and the place. Under 2% it is said in a
   note.

8. **No crossing by construction, then checked.** The verifier reads a
   feed-feed crossing by exact signs. Passes that ride a wall ride it on
   one fixed vertex list (`canon.rs`), room vertices within 0.05 mm of a
   wall are snapped to it, and a pass in an area cut before is stretched
   along the walls to where the earlier passes of that area landed and
   left. The way in, the way out and the link between are then retraced
   chord for chord. Each candidate is still asked the crossing law
   against the depth's earlier feeds; a crossing candidate is not kept.

9. **Chords, no arc fit.** The posted moves are the chords that were
   measured. The plain Adaptive fits arcs and gives 5% of margin for it;
   here the measured path is the posted path.

10. **The entry is always a plunge** into proved air. The `entry` family
    and the pocket's links describe entries into material, and are not
    read.

**Why**

The issue's contract: separate legal-center and remaining-material
regions per depth, cleared space derived from verified stock, engagement
initialised from the remaining material, the whole footprint and approach
proved before an entry, named refusals, a conservative raster, a bounded
budget.

**Known limits (each a named refusal, pinned by a test)**

- A pocket corner the cutter's center has to turn in on a tight arc: the
  last sliver wraps a quarter of the cutter however thin it is. With a
  4 mm tool a center arc of 1.0 mm is cut and one of 0.7 mm is refused
  ("cannot hold the optimal load at (x, y)"); a square corner always is.
  Follow-up #2108 (leave corner rest with a note, or cut it over the load
  by name - the maintainer's call).
- A neck under about 2.5 tool diameters wide. The 10 mm neck of the test
  is cut by a 4 mm tool; the 6 mm one is refused (#2108).
- One entry per area and depth is the aim; a neck takes more (37 per
  depth on the test's 10 mm neck), because a half loop whose way back
  would cross an earlier feed is reached by a retract. Follow-up #2109
  (air feed and retract budget).

**Readings** (debug build, 4 mm tool after a 12 mm tool, two depths)

| Fixture | Passes a depth | Plunges a depth | Worst planned load | Measured (D/120 grid) | Cut / air feed / rapid (mm) | Planning time |
|---|---|---|---|---|---|---|
| 60 x 40, 3 mm corners | 40 | 4 | 14.4% | 13.1% of D | 224 / 967 / 360 | 3.6 s |
| 70 x 50 with a 14 x 14 island | 36 | 4 | 13.4% | 13.5% | 250 / 955 / 450 | 4.9 s |
| two bays, 10 mm neck | 128 | 37 | 15.4% | 15.1% | 954 / 3445 / 1337 | 19 s |

Optimal load 14.6% of D. The measuring stock is on the planner's grid
(D/120, the heavy lane's): on the default D/80 grid the same paths read
up to 17.4% in a concave corner, because the yardstick's rim stands a
cell and a half outside the cutter and takes in more wall there.

The air feed is high (about four times the cut length): every pass of an
area rides the wall in from the area's first landing and out to its first
exit, and returns on one chord. That is the price of retracing instead of
crossing (point 8). The plain Adaptive holds its plans to 1.5 times; this
one is not held to that budget yet (#2109).

**Considered and dropped**

- Reusing the plain Adaptive's `open_rounds`: it assumes its own bore,
  re-enters wherever a cut starts, cuts over the load up to a cap and
  laps the whole wall.
- A stay-down return on Dubins links: fresh links a few microns from an
  earlier feed cross it under the exact-sign law.
- Planning all areas phase by phase: one retract per pass (31 a depth on
  the corner fixture, against 4).

**Cost to change later**

The stock contract (`RestStock`, `StockFloor`) is the public surface; the
planner behind it can be replaced. Changing what a floor claims changes
#2033's producer with it.
