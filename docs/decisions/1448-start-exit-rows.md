## #1448 - typed Start point / Exit point rows on the contour's Lead tab

**What was decided.** The closed 2D contour's user-set start and exit
points (`links.approach.start_point` / `exit_point`, shipped through the
project file and MCP `edit_operation` by #1478 and #1492) get two optional
groups on the operation page's "Lead in / Lead out" tab: "Start point" and
"Exit point". Each is switched on in its group header and holds an X and a
Y row in model coordinates ("Start X", "Start Y", "Exit X", "Exit Y"),
with one visible consequence line: a point the lead law cannot hold is
refused by name, it is never moved. Off writes `None`; on seeds the model
origin for the user to edit. The rows edit the draft params, and Apply
commits them through `EditOperationParams`, so undo and redo restore the
earlier point. Only the 2D Contour page draws the groups: the other kinds
refuse a point by name (`OpParams::named_refusal`), so they get no row
that could only produce a refusal.

**Why.** Before this, the feature had no door in the GUI at all - a user
could set a start or exit only by editing a project file or through MCP.
Typed coordinates are one of the three pick forms the issue names, and the
smallest one that needs no new viewport interaction. The header switch
follows the page's law for an optional feature (Plunge, Engagement), so
"no point" is never a magic coordinate.

**Dropped.**
- Seeding the switched-on point at the ring's current automatic seam: the
  page does not hold the generated path's seam, and a stale seam would
  read as a promise. The origin is a neutral, obvious seed.
- Drawing the rows on every kind, greyed out: a row whose only result is a
  refusal is noise; the refusal still guards a file or MCP edit.
- The viewport pick (click on the wall): it needs its own pick mode and
  marker. It stays owed in #1473 item 2.

**Cost to change later.** UI only: one new file
(`crates/ui/src/egui_app/start_exit_rows.rs`) and one call in
`approach_page.rs`. No project-file, command, MCP or generator change, so
no version bump and no golden changes.
