## #2057 - Gate: one job budget for all lanes of a wave

**Decided:**

- One budget for each gate run: `budget::DEFAULT = 6`, override `CAMCRAFT_GATE_JOBS` (a whole number of 1 or more). A bad value prints a note and keeps 6.
- The executor sets `-j` and `--test-threads=` of each step when it runs the step (`budget::scale`). The plan builders (`plan.rs`, `quick.rs`, `single.rs`) still write `-j 6` and `--test-threads=6` as before.
- A wave of N lanes gives each lane `budget / N` (minimum 1). A wave has at most `budget / 2` lanes (minimum 1). `lanes::waves` takes this limit as a parameter; `LIMIT = 4` is gone.
- A step alone (each quick, batch and test step, and each release step that runs alone) gets the full budget.
- A value that a step sets on purpose (not the plan default `6`) only goes down to the lane's jobs, never up. Example: `--test-threads=1` stays 1.
- The gate prints the budget line at the start and `-- wave K: N lane(s) x -j M` before each wave, also in `--dry-run`.

**Why:** the rewrite at run time keeps the plan builders unchanged. Open PR #1952 (#1941, one step per test binary) changes those builders on many lines. This change thus does not conflict with it, and each step that #1941 adds gets the budget too. The quick and batch tiers run their steps one at a time, so the budget applies to them as one step at a time.

**Dropped:**

- A budget parameter in each plan builder: it touches the same lines as #1952 and all plan-string tests.
- An automatic core count (`std::thread::available_parallelism`): the machine is shared with review and worker builds, and the gate cannot see them. The maintainer set the budget from the shared `-j 6` rule; the skills tell each build to keep the sum at or below the core count.

**Cost to change later:** small. `budget.rs` holds the rule; a different split (for example by step weight) changes `per_lane` and its tests.
