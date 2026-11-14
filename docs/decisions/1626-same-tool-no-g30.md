## #1626 - Keep an already loaded tool between Tormach sections

**Decided:** remember only the last tool number for which this program
emitted `T.. M6`. Start with no known tool, so the first tool always keeps
its existing G30/M6 sequence. A later section requesting that same tool
skips G30 and M6, while retaining G43 H, the machine-coordinate Z lift
(#1596), and the stated clearance move before X/Y (#1614). A different
tool, including a return to an earlier tool after another was loaded,
still performs the change.

**Why:** #1626's PathPilot Hub run showed a needless dip to the stored
G30 height between sections using the same tool. The stream establishes
which tool the program loaded, but cannot establish the spindle's tool
before the program starts. Keeping the length offset and positioning
sequence preserves the safety work in the parent changes.

**Considered and dropped:** assuming the spindle already contains the
first requested tool; querying controller state or relying on a repeated
M6 being a no-op; suppressing the whole tool preamble. The post has no
controller-state input, a separate G30 still moves even if M6 does
nothing, and G43/lift/clearance remain necessary at the section boundary.
Tormach's [post guidelines](https://knowledgebase.tormach.com/pcnc-1100/pathpilot-post-processor-guidelines)
describe M6 as a tool-change sequence and recommend G30 before changes.
The [770MX tool-change documentation](https://knowledgebase.tormach.com/770mx/pathpilot-tools-and-features-770mx)
describes the automatic/manual behavior, but neither page specifies a
same-tool M6 exception. This change therefore does not depend on one.

**Cost to change later:** the remembered-tool state and its conditional
are local to this post; changing the policy requires updating its stream
pins and any affected Tormach NC goldens. Controller or Hub behavior
remains a separate user exercise.
