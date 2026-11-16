## #1468 - Lathe chucks: fit to the stock in the dialog, inside grip, stepped jaws

**What was decided**

- This PR is the first slice of #1468: the dialog's "Fit to the stock",
  the inside grip and stepped jaws, and the camcraft-mcp skill row. The
  independent 4-jaw offsets, the chuck rpm limit, the chuck on a rotary
  axis and the viewport screenshots stay on the issue (`Refs`, not
  `Fixes`).
- The grip direction and the step are figures of `JawChuck`
  (`grip`, `step_height`, `step_depth`), not new accessory kinds. The
  three new fields read as outside, 0 and 0 from a project saved before
  them (`serde(default)`), so no file-version change is needed and the
  default chuck's mesh is the #1442 mesh to the triangle.
- An inside jaw's gripping face is the arc of the bore it opens into,
  drawn as a polygon with its flats on the grip circle (circumscribed), in
  steps of at most 1/256 turn. The drawn steel is never less than a jaw
  whose face is ground to the bore, and stands into the work by at most
  0.0075 % of the grip radius, which is under the fixture clash depth
  (0.05 mm) for any grip radius below about 660 mm.
- A stepped jaw is two shells that touch on the step's top: the step
  (reaching the step depth past the gripping face, under the work) and
  the plain jaw above it. The step is given by both figures or by
  neither; one alone is refused by name.
- A fit to the stock seats the stock on the step (the chuck's face goes
  the step height below the stock's underside).
- A fit to the stock with an inside grip is refused by name: the stock is
  a box or a solid cylinder and does not state a bore. The dialog unticks
  and disables "Fit to the stock" for an inside grip and says why.
- The dialog opens "Fit to the stock" ticked when the setup's stock reads
  as round, the way parallels open fitted when the setup has a vise.

**Why**

Workholding steel is read by the lathe holder law and blocks the post, so
the envelope must hold the real jaw wherever it stands. A flat face at the
grip radius would put an inside jaw's corners 1 to 2 mm into the work for
common sizes and raise a clash warning on every inside grip; a flat face
through the corners' contact points would draw less steel than a jaw
ground to the bore. The circumscribed arc is on the safe side of both.

**Considered and dropped**

- New accessory kinds for inside-grip chucks: it would double the kinds
  for one figure and split the resize path.
- Reading the bore for an inside-grip fit from the part's mesh: it needs a
  bore finder on the part and a rule for which bore; left for a later
  issue if it is wanted.
- One L-shaped shell for a stepped jaw: two touching shells follow the
  rule the chuck module already has (shells touch, never overlap) and
  need no new triangulation.

**Cost to change later**

The three fields are in saved projects once this ships; renaming them
needs a serde alias. The arc step is one constant.
