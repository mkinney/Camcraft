## #3369 - a form mill kind with roundover, cove, ogee, V-groove and bead presets

Plan step D2 of the tool library redesign (#3663), stacked on D1 (#3929).

**Decided: the outline lives on `Tool` (`Tool::form`), not only in the
drawn cutter (`ToolCutter`).** The issue text puts the outline in the
`ToolDto::cutter` field. But `Tool::meridian`, `Tool::profile` (the band
stack that the collision checks sweep), `is_sane` and the starter catalog
see only `Tool`. `ToolCutter` is a document component, and the catalog
and the tools crate cannot read it. So `Tool` gets one field,
`form: Option<FormSpec>` (`crates/tools/src/form_mill.rs`): the points
`[z, r, arc]`, the pilot length and the preset that wrote them. The
drawn cutter stays what it is for every mill (a drawn neck or shank);
for a form mill it reads the form through `Tool::profile`, so the two
agree. Dropped: the outline in `ToolCutter` only (the catalog, the
meridian and the sanity law could not read it, and a catalog form mill
would land without its shape).

**Decided: the outline owns the diameter and the flute length.** A form
mill's `diameter` is twice the outline's widest radius and its
`flute_len` is the outline's top. `is_sane` refuses a disagreement by
name and states the right numbers. The editor (`ToolUi::to_tool`) and
MCP `add_tool` / `edit_tool` call `Tool::settle_form`, which writes the
points from the preset and the two numbers from the points. The editor
shows the two numbers and does not take them. A file load does not
settle: a file that disagrees is refused, not repaired. Dropped: a
separate stated diameter (two sources for one number).

**Decided: with a preset, the stored points are exactly what the preset
writes.** The file keeps both the preset and the points, so a reader
needs no preset code to draw the form, and `is_sane` refuses points that
disagree with their preset. A catalog entry states only the preset; the
reader fills the points (`form_mill::fill`) and keeps the stated
diameter, so `is_sane` checks the catalog numbers. A preset can be
dropped (`preset: null`) to keep a drawn outline.

**Decided: no file version step.** Only the `form_mill` kind states a
`form`, and each older build (D1's included) refuses the unknown kind
word by name. It cannot read a form mill as another tool. The
`.camproj` stays at its version and the `.camtools` library does not
step (a test pins that it stays under `TIP_FORM_VERSION`). Dropped: a
`.camtools` version 7 and a `.camproj` version 80 (no reader needs them).

**Decided: the presets and their numbers.** Our own geometry from the
terms of the public router bit catalogues: roundover (concave quarter
arc), cove (convex quarter arc), ogee (concave arc, flat step, convex
arc), V-groove (cone with an optional tangent tip ball, no pilot), bead
(a flat step, the quirk, then a concave quarter arc). Each preset with a
pilot starts its cut at the pilot radius. The pilot segments (the tip
face and the bearing side, up to `pilot_len`) are flagged non-cutting in
the meridian. Dropped: a pilot radius apart from the cutter's start
radius (no catalogue bit that this slice covers needs it).

**Decided: no undercut in this slice.** The radius must not narrow
upward: a line that narrows, a convex arc past its widest point or a
concave arc before its narrowest point is refused by name with #896.

**Decided: every operation refuses a form mill by name.** The form mill
is in the `CONE` family (2D Chamfer, Deburr and the multi-axis cone
kinds), so `no_orphan_tool_kinds` holds. `tool_gate` refuses it at the
creation door and at the generation and post door, and the 2D Chamfer
refusal names the edge form work (#896). No refusal suggests a form mill
(`kinds_phrase` drops it beside a kind that cuts), and no operation
offers one. The operation stamp hashes the whole outline and the pilot.

**Decided: the editor.** The kind is in the type list; a "Form preset"
list (a drawn outline or one of the five presets), the preset's number
rows and the pilot length show on the Tool tab (`tool_form_mill_row.rs`,
called from `kind_rows()`). A drawn outline is edited through MCP
`edit_tool`; the editor shows it and does not draw point by point. The
shape window's Draw cutter view names the form in place of a tip number.
Dropped: a point table for the form in the editor (later work, with
#896).

**Catalog:** six bits, a roundover, a cove and a 90-degree V-groove, in
mm (R6 on a 13 mm bearing, 25 mm cut; 12 mm V) and inch (1/4 in radius on
a 1/2 in bearing, 1 in cut; 1/2 in V), the common sizes of public router
bit catalogues. The inch bits state their true inch numbers: a 3/16 in
pilot (4.7625 mm) and a 11/16 in flute length (17.4625 mm). Dropped:
rounded values (4.8 mm, 17.5 mm) chosen so that floats add up exactly. A
check that adds lengths compares them with float dust as its slack
(`AGREE`, 1e-6 mm, in the sanity law; 1e-9 mm in the holder stack test),
not with exact float equality.

**Cost to change later:** the undercut forms (#896) widen
`form_mill::refusal`; a generator that reads the outline removes the form
mill from `tool_gate`'s refusal.
