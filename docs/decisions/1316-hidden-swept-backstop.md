## #1316 - The mid-turn overhang backstop reads swept cells, not 4 x 5 poses

**What was decided.** Law (c3)'s overhang backstop (stock hidden under a
ledge, `Rest::hidden_*` in `crates/sim/src/verify/hidden.rs`) now asks over
the same reachable set the column law reads since #1313. It does not reuse
the column law's 0.001 mm cells: the hidden-stock law answers one plane per
ask and slices the model once per distinct plane, so it gets its own,
coarser cover (`crates/sim/src/verify/turn_hidden.rs`):

- the turn split so the sag is at most 0.05 mm (`BACKSTOP_SAG`, a tenth of
  the hidden law's own 0.5 mm XY spacing), capped at 20 intervals
  (`BACKSTOP_PLANES`); past the cap the cells only grow (more convictions,
  never fewer);
- the tip's chords grouped into at most 4 runs (`STATIONS`), and no more
  than `20 / intervals` of them;
- one ask per (interval, run), over the union of their footprints. A
  sample point counts only where some position of a cell stands the band
  over it outside the flutes' kerf at that tip position (the cells' own
  test). Each ask slices one plane - the lowest bottom any column of its
  cells reaches - and reads every column it asks over the whole range of
  heights the band's bottom takes there (see "The height rule" below).

**Slice count.** At most 20 distinct planes per live band per move - the
old grid's 4 stations x 5 poses - so the slicing budget is unchanged. A
90 degree turn at 150 mm reach hits the cap and overstates by 0.12 mm.

**Why.** The 4 x 5 grid let a holder swing through hidden stock between two
sampled poses with no `SweepHit` (P1: a collision that passes verify). The
new cover is a superset of every pose and tip position of the move.

**Considered and dropped.**
- Reusing the column law's cells (up to 64 intervals x every chord): up to
  64 x chords planes per band per move, several times the old budget.
- A dedicated capsule/hull query in `hidden.rs`: the cells already carry the
  exact swept membership, so the new entry point (`hidden_in_cover`) takes
  that membership as a closure instead of re-deriving the geometry.
- One plane per band per move (the lowest of the whole move): on a long
  ramp it asks the deep end's plane at the shallow end, a false conviction.

**Kept from the old grid.** XY sampling of the hidden region by its own
law (0.5 mm), and the slice budget.

**The height rule (corrected 2026-11-12, #1324 safety review).** This
file first said each ask is read at the lowest plane of its cells and
that this "follows #1313's column law" and only adds convictions. That
was false. The column law is monotone in the plane (a column standing
under the band's lowest bottom stands under every higher one); the
hidden law is not: `hidden_at(h) = shadow_above(h) ∩ footprint ∖
section(h)`. The shadow only shrinks as the plane rises, but the section
comes and goes, so stock hidden in a cavity, a groove or a T-slot gap is
part material at the plane of the floor under it and drops out there.
One ask holds a whole run of the tip's path, so its lowest plane is the
run's deep end: a plunge while turning (the review's counter-probe: A
0 -> 2 degrees, Z 0 -> -40, a cavity at Z 2.64..4.64 that the old grid
convicted) was read at Z -20.37, in the floor, and passed.

The rule now: **the backstop convicts when stock is hidden at any height
the band's bottom takes over a column, not at one plane.**

- `Cell::heights_over` gives, per cell and stretch of the chord that
  stands the band over a point, the closed range from the lowest bottom
  there to the highest any pose of the cell holds it (the end stations'
  higher value plus the cell's sag; the rim's dip only lowers a bottom).
- `hidden_span.rs` answers a range exactly in Z for one column: it walks
  the vertical through the point down the model's facets counting depth
  (+1 at an up-facing facet, -1 at a down-facing one); a stretch at depth
  0 under a crossing is exactly what the plane law calls hidden at each
  of its heights. The stock top and the lowest earlier clearing cut (less
  `z_tol`) cap it, as in the plane law. No sampling in Z, so no slab is
  too thin to see (below 1e-6 mm, the slicer's own plane nudge, two
  faces are one). A vertical that cannot be read (grazing three times
  over, or a walk that does not close) convicts.
- The columns asked: the footprint probes, the lowest plane's boundary
  samples and a 0.5 mm lattice over the ask's box, clipped to the box of
  what part material hangs over from the lowest plane up (the shadow is
  monotone, so that box holds the hidden stock of every higher plane).
- The lowest plane's region reading is kept as it was, beside it.

Why it holds what both earlier readings held: a grid column of #1319's
sampled backstop at (station, pose) has its plane between the lowest and
the highest bottom of the cell holding it, so it is inside the range
asked over every point of its disc; and the #1324 lowest-plane reading
still runs unchanged. The range only widens what is asked.

Considered and dropped for this:
- Also slicing the old grid's 20 planes per band: it restores the grid's
  convictions but is still a fixed set of planes - a cavity between two
  of them passes. It also doubles the slice budget.
- Sampling the range in planes at a bounded spacing: to bound it by the
  thinnest slab that matters (`z_tol`, 0.001 mm) a 40 mm plunge needs
  40 000 slices per ask. Not provably conservative at any affordable
  spacing.
- One range per ask (its lowest plane to its highest bottom) for every
  point: sound, but it asks a point at heights the band never takes over
  it (the deep end's heights at the shallow end) - false convictions.
  The per-point range is as cheap and exact.

Limits that remain: XY is sampled (0.5 mm lattice, widened past 20 000
columns per ask; hidden stock narrower than that, away from the lowest
plane, can be missed). The range is the band's BOTTOM, as law (c)'s
column is asked - stock a band meets with its side, above its bottom, is
the column law's, as before. Cells read chords, not the exact arc: at
most `chord_tol` (0.01 mm) off at an arc's extremum.

**Cost to change later.** The three constants in `turn_hidden.rs`; a finer
budget only costs slices.

**Composed with #1319's refresh (2026-11-12).** #1319 moved the grid
backstop's column to main's exact rim dip lowered by #1303's span floor
(`band_floors`), and put the same floors into the column law's cells. The
backstop's cells now take those floors too (`push_cells_with` threads
them; `turn_hidden` passes `band_floors`), so every cell's lowest bottom
stands no higher than the span-floored exact-dip column the grid read at
that pose. On its own that does not make the whole-turn reading convict
all the grid did (a lower plane is not a superset for the hidden law);
with the height rule above it does: the grid's plane lies inside the
range each point of its disc is asked over. #1319's pin `the_backstop_reads_the_exact_dip_lowered_by_the_span_floor`
was ported from `backstop_columns` to `backstop_asks` with the same
verdicts and the same exact reading (-56.43).

**Known limit (XY sampling).** The backstop reads XY on the hidden law's
0.5 mm lattice (`hidden_in_cover`), clipped to the overhang; a hidden
pocket narrower than that lattice in both X and Y can fall between its
verticals.
