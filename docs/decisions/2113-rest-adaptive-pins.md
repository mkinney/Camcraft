## #2113 - rest Adaptive: pin the raster half of the entry proof and the load on the default grid

**What was decided**

1. **The raster half is pinned at the `Entries` level**, not through the
   planner. In the planner the raster is built from the same polygon
   bound, grown, so the polygon half (grown by a full half margin more)
   refuses everything the raster would: no planner fixture can make the
   raster refuse alone. The unit test
   `the_raster_half_of_the_entry_proof_refuses_on_its_own` hands
   `Entries` a raster holding a 1 mm pillar that the polygon bound does
   not (the case the second proof exists for: a polygon read that lost
   material) and asks a plunge whose rim would stand 0.2 mm inside it.
   Checked by mutation: dropping `self.stock.air(..)` from
   `Entries::proved`, or setting the footprint to 0, turns it red.
2. **A documented load ceiling on the default D/80 measuring grid:
   optimal x 1.2** (`LOAD_CEILING_D80` in the test's `sim.rs`), pinned on
   the climb 3 mm corners and the conventional 3.5 mm corners with 0.3 mm
   left. Readings (optimal 14.6% of D): 14.6% and 16.7% on D/80, 13.1%
   and 14.7% on D/120.
3. **A corner assertion on D/240**: a 26 x 22 pocket with 3.5 mm
   corners, cut conventionally with 0.3 mm left (the worst case above),
   fits the measuring stock's 4096 cells a side (1680 x 1440) and reads
   12.7% of D, held to the planner's own margin (optimal x 1.05).

**Why**

The issue: removing the raster half or zeroing its footprint left every
test green, and the load was pinned only on the planner's own grid.

**Considered and dropped**

- A planner-level fixture for the raster half: impossible while the
  raster is derived from the polygon bound (point 1).
- A ceiling over the neck and island fixtures too: they read lower on
  D/80 (15.0%) and add 30 s of debug time to the default suite.

**Cost to change later**

A planner change that moves a corner reading on D/80 past 17.6% of D
fails `on_the_default_measuring_grid_...`; the ceiling is one constant.
