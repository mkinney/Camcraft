## #3852 - A 3D Pocket restart plunge starts at the plunge height, and sub-resolution feeds merge

**Decided.**

1. A 3D Pocket pass that restarts after a hop with a straight plunge
   uses `descent::feed_down`, the same call as a level entry. The tool
   goes down by a rapid to the plunge height over the material top
   (`mat_top`: the plane above on the plain ladder, the top of the cut on
   the first level or on a lifted shell). Then it feeds the rest. Before,
   the plunge fed the whole way from the hop plane. `feed_down` also runs
   the non-centre-cutting `plunge_law::guard`. `reenter` already returned
   `None` only where that law permits the plunge, so the guard adds no
   refusal.
2. The finished 3D Pocket path merges each straight feed that moves less
   than 0.001 mm on every axis into its straight neighbour: into the next
   straight feed, else into the previous one. A short move with no
   straight feed beside it (a plunge, an arc, a rapid) stays. The chain of
   moves stays unbroken. The merge runs only when the path has no
   control-compensation spans, because those spans can refer to moves.

**Why.** The catalog case `3d/rough-finish` posted 18 `G01` blocks with
the same X, Y and Z as the block before them. A sub-resolution move
comes from offset-ring vertices that are almost the same point. The
merged line lies within about 1.7 micrometres (one resolution step on
each axis) of the two moves that it replaces. That is far below the
surface tolerance of the 3D Pocket. The restart plunge from the hop
plane fed up to 31 mm at the plunge feed through air that a rapid can
cross.

**Dropped.**

- A filter in the post engine that skips a block whose words do not
  change. The resolution is a property of each post script (metric or
  inch, three or four decimals). A filter there touches every operation
  and every post. The issue names the 3D Pocket only.
- A threshold in `PathBuilder::push`. That changes every generator and
  every golden program.
- A threshold below 0.001 mm (for example 0.0005 mm). A move between the
  two values can still post as a zero-length block.

**Cost to change later.** Small. `short_moves::merge_short_feeds` is
one call at the end of `ladder`. Another generator can call it, or it
can move to `camcraft-cl` if more generators need it.
