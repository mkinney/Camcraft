## #2018 - One Weldon pick for each taper, the bore from the tool's shank

**Decided.** The holder picker lists one "<taper> Weldon" row for each
taper (BT30, BT40, CAT40, TTS) instead of one row for each bore. The
name "BT40 Weldon" is also accepted by `SetToolBody::library`, by
`AddTool`'s stated holder and by the MCP `set_tool_holder`/`add_tool`
`library`/`holder` keys: it resolves (`holder_library::resolve`) to the
library's existing per-bore entry whose bore takes the tool's shank,
and from then on is exactly that entry (outline, body diameter, fit
check, collision bands, and the name `AddTool` records). A shank no
standard bore of the taper takes is refused with the fit check's own
`LibraryError::Shank` text, listing the taper's bores.

**Why.** The maintainer: "a weldon is a weldon, and whatever tool will
fit it". A real side-lock holder still has one fixed bore and a body
diameter that goes with it, and the body diameter is a collision shape,
so the data keeps one entry for each bore and only the pick groups them.
Resolving to an existing entry means the collision and simulation shape
cannot change (pinned band for band, for every bore of every taper).

**Order.** The issue lists #2016 (holders reference a library entry)
first. This change does not need it: the grouped pick resolves to a
concrete entry name at command time, which is what #2016's reference
will then store. Done ahead of #2016 so the picker change is not held.

**Dropped.** One Weldon shape that fits each shank (no such holder; a
wrong body diameter changes the collision check). Removing the per-bore
names (old project files, machine `default_holder` keys and MCP callers
name them; they keep working). Listing the grouped row in
`list_holder_library` (it lists the data, which is unchanged; the schema
text of `library` says a taper's Weldon pick is accepted).

**Cost to change.** Small: one lookup module
(`crates/tools/src/holder_library_weldon.rs`) and the picker rows.
