## #1896 - Hold the endpoint reproduction for effort:max

**Decided:** Escalate under the work-issue skill to `effort:max` and
preserve the failing endpoint pin for the next worker. No fix was selected
or implemented. The earlier recommendation to extend each shallow flank
pass with a final partial helical turn was not implemented or verified.

**Why:** The FANUC Series 0i-F lathe operator's manual B-64604EN-1,
section 4.2.7, Fig. 4.2.7 (a), and the Siemens SINUMERIK ISO Turning
programming manual A5E52211627B, Fig. 4-25, both state that every G76 pass
runs to D. The endpoint pin reproduces the shorter laid ends. A proper
partial-turn extension requires generalizing Fanuc `threadHalfTurn` and
Tormach `threadHalfTurn`, while preserving the record's lead and spindle
hand. It also requires reviewing the threading verifier's `thread_law`
parity assumptions. This is broader than an endpoint edit.

**Considered and dropped:** Moving only the existing last half-turn's
axial endpoint is unsafe because it changes that arc's lead. Changing the
lead of the whole pass would change the thread. A kernel-only partial-turn
extension was not accepted without accounting for the posts and verifier.

**Cost to change later:** The next worker must resolve the post and
verifier assumptions, then pin the extension's endpoint, lead and hand
before making a production change. The held pin records the reproduction,
not a completed fix.
