## #3332 - Wire Contour takes an exit pick

**What was decided.**

1. **The capability.** `PickCapability::Threading` takes a start pick and an
   exit pick. The start pick is the threading point, as before (#3871).
2. **An open run.** The cut ends at the end nearest the exit pick. When that
   is the first end of the run, the wire walks the same wire-centre path in
   the other direction. The kerf side is still read from the drawn order of
   the run, so the kerf does not move. A note on the path says this.
3. **A closed profile.** The lead-in and the whole lap are the same as
   without a pick. The wire then travels in its own kerf to the nearest point
   of the path to the exit pick. It takes the shorter way round and cuts
   nothing. It leads out straight from that point to the threading point.
   The path still starts and ends at the threading point, so the post, the
   pass split and the wire verify read it without a change.
4. **Tabs.** The stretch between tabs that holds the exit is cut last. The
   main cut leaves that stretch at the exit, before the tab cut-off. The
   cut-off moves do not change.
5. **Refusals, each by name.**
   - An exit pick that is equally far from both ends of an open run.
   - A lead-out that would bring the kerf into the product.
   - An exit pick that meets the path on a tab.
   - An exit pick together with a taper, skim passes, a no-core burnout, a
     4-axis cut or a selected drawing profile.
6. **No new parameter on `WireParams`.** The exit pick is an argument of the
   new entry `wire_edm::wire_contour_exit`. A cut without an exit pick goes
   through the same code as before, so each shipped wire golden is
   byte-identical.

**Why.**

- The issue asks for an exit at the nearest end of an open run, and for an
  exit on a closed profile "before the tab or the cut-off".
- The post and the wire verify read the passes between the visits of the
  threading point. A path that ends at a different point breaks that law.
  Thus the lead-out goes back to the threading point.
- A slot that the wire has cut is free to travel in. Thus the travel to the
  exit after the lap cuts nothing and cannot gouge. The kerf law still
  certifies each move.
- The kerf side of an open run is stated against the drawn order. If the
  side followed the travel direction, an exit pick would move the kerf to
  the kept side. That is a wrong cut. Thus the side stays with the drawn
  order.
- A taper, skims, a burnout, a 4-axis cut and a drawn line-and-arc profile
  each have their own generator. None of them reads an exit. A refusal by
  name is safer than a pick that is ignored.

**Considered and dropped.**

- A path that ends at the exit pick, away from the threading point. Dropped:
  it breaks the pass split of the post and the threading law of the verify.
- An exit that stops the lap early. Dropped: the profile is then not cut
  through, and the slug law changes.
- A start pick that chooses the first end of an open run. Dropped: the issue
  does not ask for it. An open run still states no start hole.
- An `exit` field on `WireParams`. Dropped: it changes 16 struct literals in
  tests and conflicts with PR #3617 at each one.
- A stack on PR #3617 (it changes `cut_pass` and `wire_open` too). Dropped:
  its branch does not contain #3871, so the shared picks do not exist there,
  and it conflicts with `main`. The PR that merges second refreshes.
- An Exit point row on the Wire Contour page and a viewport pick. Not in
  this PR: the issue asks for the MCP door only. The pick is set through
  `edit_operation` or the project file.

**Cost to change later.**

Low. The exit logic is in one module (`crates/motion/src/wire_edm/exit.rs`).
The refusals for the other cuts are in one function
(`crates/app/src/operation/wire_exit.rs`). To add the exit to a taper or to
skims, remove its arm there and pass the exit to that generator.
