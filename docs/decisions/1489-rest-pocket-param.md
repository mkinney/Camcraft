## #1489 - 2D rest machining on the Pocket operation: parameter, edit, file, MCP

**What was decided.** The 2D Pocket gets one optional field, `rest:
Option<RestParams>` (the #1446 generator's own struct: previous tool
diameter, its corner radius, the stock it left on the walls, the rest
allowance). `None` is off: the default, every earlier file, and the plain
pocket's output byte for byte (the stamp writes the group only when it is
on, byte `0xD7`). The project file key is `rest` (an object; absent is
off). MCP reaches it through `edit_operation` (a merge patch; `"rest":
null` turns it off), with a camcraft-mcp skill row. Generation calls
`machining2d::rest_pocket` when it is on; its "trusts the entered previous
tool" note reaches the path's notes.

**Refusals at the edit, in the generator's words.** `RestParams` gained
`validate_alone` (the checks that need no tool, plus "the previous tool
diameter must be positive") and `rest_strategy` (Zigzag and Adaptive
refused) is public, so the parameter door (`named_refusal`, `is_sane`)
and the tool door (`rest_gate`: create, edit, re-tool, retype) say the
same text the generator would. A previous tool that is not larger is
refused when the operation is made, edited or given a new tool - not only
at Generate. Pre-drilled entry beside rest machining is refused by name
(`REST_PREDRILLED`): `rest_pocket` does not read the holes, and a silently
dropped choice is not allowed. Dropped: letting the rest pass ignore the
holes with a note (a choice the user made would do nothing).

**The dialog group is not in this PR.** "Done means" names the file, the
undo, MCP, the refusals and the goldens, not the page; the page needs
before/after screenshots and the UI gate. The page keeps the group as it
is (the field rides the page's Apply unchanged). Follow-up: #1815.

**The stock-simulation reading, as measured** (`rest_pocket.rs`,
`read_against_the_simulated_roughing_pass`): after the 12 mm roughing
pass is simulated, no rapid of the rest pass meets stock; every plunge's
axis goes down on floor already cleared, and its rim meets no more than
the one level it descends (2 of the 4 corner entries per level touch the
corner's rest material at the rim); every ring cuts its corner, but the
inner leg of each ring crosses cleared floor (measured 6.7 of 14.8 to
15.3 mm per ring). The plain pocket's first level at the top of the part
runs over no stock, as it always has. Entry from cleared space is #1490's;
the inner legs are #1816. Both are feeds, so they cost time,
not safety.

**No new dependency between crates.**
