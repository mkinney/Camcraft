## #689 - STEP meshes no coarser than half the gouge verdict

**Chosen:** the STEP importer's chord target is
`min(extent * 1e-3, MAX_CHORD)`, where `MAX_CHORD = DEV_TOL / 2` = 0.005 mm.
Before, it was `extent * 1e-3` alone. The program sim judges the program
against the imported facets. On a concave wall a chord stands inside the
true surface by its sagitta, so a bore that cut the true Ø25 hole cut
about 0.022 mm of mesh "material" on Milling-test.step. That is past the
verdict's 0.01 mm, and the column was charged the wall's full 20 mm
height. With half the verdict spent on the mesh, the other half is left
for the cut: a nominal bore passes and a bore 0.05 mm oversize is still a
gouge.

**Why this and not a looser verdict:** granting the gouge check the
model's chord tolerance as extra slack would forgive real gouges up to
`extent / 1000`. That is 0.06 mm on this 60 mm part and 0.3 mm on a 300 mm
one. The issue's own "Done means" (0.05 mm oversize still refused) rules
it out, and so does the Prime Directive. Making the mesh as fine as the
verdict claims keeps the verdict honest.

**Cost:** curved faces get more facets. Milling-test.step went from 448 to
952 facets (9 ms to 16 ms import); sample_ap214.step stayed at 1550. On a
large curved part the chord edge at 0.005 mm is still about 2 mm on a
100 mm radius. The existing facet budget (refinement stops at budget / 8,
with the 30 M cap) still bounds any model, and the importer logs when it
stops short of the target.

**Not covered:** STL and OBJ imports come pre-tessellated, so their
fidelity is whatever the file has. A coarse STL can still read a nominal
finish on a concave wall as a gouge. That would need the verdict to know
the file's chord error, which isn't recorded anywhere.
