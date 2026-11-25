## #3671 - 3D Pocket: inter-level rapid stops at feed height above prior floor

**What was decided.** The descent logic in `camcraft-motion::machining3d::descent`
distinguishes level transitions from pass restarts via `Ask.restart`:
- On level transitions (`restart = false`): the descent starting plane `from(lvl, a, ramp)`
  adds `ap.plunge_height` above `mat_top(lvl, a)`, capped at `lvl.tz`. Rapids stop
  at feed height above the prior cut floor, and the full descent is cut at feed rate.
- On pass restarts (`restart = true`): the descent baseline remains `a.prev_z` (`mat_top`),
  preventing ramp depth inflation. Rapids stop at `rapid_z = (zf + ap.plunge_height).min(lvl.tz)`
  and feed down vertically to `zf` before starting the helix or ramp moves (`push_helix`,
  `push_zig3`, and the diagonal ramp arm).

**Why.** When 3D Pocket descends between levels, rapiding straight down to the floor
of the level above with zero clearance creates collision risks with chips or tool
length tolerances. Stopping the rapid at feed height above the floor and feeding
the remainder eliminates collision hazards. On pass restarts, elevating the ramp
baseline doubled the descent depth and caused false chord crossing detections with
previous level entries; stopping the rapid at feed height with a vertical feed to
`mat_top` provides rapid clearance while keeping the ramp geometry intact.

**Considered and dropped.**
- Elevating `mat_top` baseline on pass restarts: dropped because doubling ramp `dz`
  caused candidate ramp moves to overlap previous level entries and trigger false
  collision refusals.
- Setting a hardcoded 2.0 mm margin: dropped in favor of reading the operation's
  configured `plunge_height` (feed height).
- Modifying caller sites without a `restart` flag: dropped because distinguishing
  level transitions from intra-level pass restarts requires explicit caller context.

**Cost to change later.** Small: logic is localized to `descent.rs` and `ladder.rs`.
