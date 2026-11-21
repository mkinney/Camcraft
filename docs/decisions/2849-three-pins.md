## #2849 - Two of the three planner pins that moved on the solidcore STEP mesh

The finer solidcore mesh moved three toolpath pins, and each pin has its
own cause. This PR fixes two causes at the planner. The two pins pass
with no change to their asserts. The third pin keeps its marker.

**1. `axis_reseat` keeps the start only on an exact axis edge.**
- Decided: the ring start is kept when its first edge is exactly on X or
  Y, or when no edge is nearer an axis than the first edge. Before, the
  start was kept for each first edge within `axis_tol` (1 degree).
- Why: `entry_arc::true_arc` posts a G18/G19 only for an exact axis edge.
  A first edge 0.93 degrees off X held the start, and the arc entry was
  posted as chords. The tolerance now only selects the candidate edges.
- Dropped: a snap of the near-axis edge onto the axis. That moves the
  ring geometry and needs its own proof against the part.

**2. The horizontal arc entry reads its tangent from the closing edge of
the first ring (`descent_xy::closing_dir`).**
- Decided: when the first pass is a ring that returns to its start, the
  arc tangent is the direction of the closing edge, from the last vertex
  at least 1 micron before the start. Other passes keep the old tangent.
- Why: the old tangent came from the fitted arc of the walk. It was a
  fraction of a micron off the ring polyline that returns to the start.
  The ring behind the start then crossed the proof polylines of the arc
  on both sides, at each radius. On the closing edge, that edge is on the
  tangent line, and the ring curves to one side only. The proof (fan and
  hull against the area and the passes) does not change.
- Dropped: a change to `entry_arc::pts_dir` or `seg_dir`. Those also feed
  the vertical arc entries, and the defect is not there.

**3. The fig8 ring order (test 3) is not changed here.**
- Tried: in `waterline_passes`, the chain step to the other ring family
  at the same offset (outer to hole) was tried last. Test 3 passed: the
  part face was cut last.
- Dropped because: `corner_arc_radius::the_crossing_law_keeps_the_arc_turns_and_crosses_nowhere`
  went red (OffsetRings, r 1 mm). The chain then ends with a short
  diagonal chord into the sharp corner of the outer wall ring, and one
  corner arc is kept sharp ("would cross another feed").
- The two pins want opposite wall orders on a holed area. The rings do
  not know which wall is the finished wall. The maintainer decides the
  law (for example, a flag from the outside containment that names the
  part face as the last wall).

Cost to change later: each rule is local to one function and has its
own unit pin.
