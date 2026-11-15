## #1580 - Escalate finer carried-side contact detection

**Decided:** Retain the targeted reproduction on
`mkinney:issue/1580-carried-side-contact` and escalate from `effort:high` to
`effort:max`. This branch holds reproduction work, not a production fix.

**Why:** The box scan reads column centres. The mesh check between columns
reads only the material that four columns share. Neither reading identifies
the stock boundary inside a cell. `Body::on_mesh` discards continuous history,
so another fixture-mesh query cannot recover that boundary. General finer
contact detection needs a new boundary-intersection or refinement algorithm.
The effort table assigns new geometry and collision algorithms to `effort:max`.

**Dropped:** Expanding each column into its cell can refuse valid face
contact. Increasing fixture-mesh sampling does not restore the missing stock
boundary. The permitted bound-only tests passed unchanged in the earlier
attempt, so that path does not supply the required red-before-fix pin.

**Cost to change later:** The next worker must define the stock boundary
representation, contact tolerance and read budget. It must cover side faces
and steps, preserve valid contact, and prove the fix against the reproducer.
No production behaviour changes in this handoff.
