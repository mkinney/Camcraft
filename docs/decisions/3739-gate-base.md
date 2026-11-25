## #3739 - Quick and heavy gates warn when the local base branch is stale

**Decided:** `cargo xtask gate quick` and `gate heavy` print a warning for
each remote copy (`upstream/<base>`, `origin/<base>`) that has commits the
local `<base>` lacks. The warning gives the count and names
`--base <remote>/<base>`. The gate still diffs against the base that the
caller gave. The check reads the refs of the last `git fetch`.

**Why:** a worktree agent never pulls the main checkout (#1201), so the
local `main` can be thousands of commits behind. The quick gate then sees
thousands of changed files and falls back to batch. A warning shows the
cause at once and names the fix. It changes no plan, so no gate result
moves.

**Dropped:**
- Change the default base to `origin/main`. In a fork, `origin/main` is the
  fork's main, which syncs only at cut tags, so it is also stale. The right
  remote is not the same in each checkout.
- Run `git fetch` in the gate. The gate then needs the network, and a fetch
  while another gate runs in the same checkout is a side effect.
- Stop the gate when the base is stale. A caller can have a reason for an
  old base. A warning is enough.

**Cost to change later:** small. The check is one function in
`xtask/src/stale_base.rs`, called from `changed_since`.
