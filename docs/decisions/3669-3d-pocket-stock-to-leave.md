## #3669 - Apply stock to leave axially and radially in 3D Pocket

**What was decided.**
Apply `stock_to_leave` in all directions (axially on tops, flanks, and floors as well as radially) in `machining3d`:

1. **Level target probe height (`ladder.rs`):** For 3D Pocket ladders (`run_plain_with` and `run_rest_with`), query `level_target(facets, footprint, z - p.stock_to_leave)` instead of `z`, and keep the `r + p.stock_to_leave` XY erosion. Because any model point above `z - s` enters `material_overhead(z - s)` and keeps `r + s` horizontal distance from the cutter center, and any point at or below `z - s` stays at least `s` below the cutter bottom, every cut point on the tool envelope keeps at least `stock_to_leave` (within arc-fit tolerance) from the model in 3D.
2. **Flat-floor discovery (`floor_levels.rs`):** When `stock_to_leave > 0.0`, admit upward horizontal model faces `t.a.z + p.stock_to_leave` across `[p.bottom_z, p.top_z)` rather than only above the first stepdown plane, and deduplicate levels within `1e-9` mm. This ensures deep flat floors (such as a plate top at `bottom_z`) receive a level at `floor + stock_to_leave` instead of stopping one stepdown above the floor. When `stock_to_leave == 0.0`, keep the `> first` bound unchanged.
3. **Rotary wrap (`pocket3d_air_split`):** Keep `axial_leave = 0.0` in `pocket3d_air_split` so cylindrical wrap-space operations (`RotaryOffset`) keep their existing polar section behavior.

**Why.**
`floor_levels.rs` and `shell.rs` already treat `stock_to_leave` as a uniform radial and axial allowance (`floor + stock_to_leave`, `roof + leave`). The plain and rest ladders probed `level_target` at `z`, which ignored model tops, shallow slopes, and floors in `(z - stock_to_leave, z]`. Probing at `z - stock_to_leave` with `r + stock_to_leave` erosion closes top, flank, and floor allowances in one uniform law with zero extra slice calls per level and preserves the downward superset law.

**Considered and dropped.**
- **Separate wall and floor stock-to-leave fields:** Rejected because `OpParams::Pocket3D` already uses one `stock_to_leave` field across `floor_levels.rs`, `shell.rs`, UI, and scripts; changing the schema is unnecessary for uniform roughing allowance.
- **Shifting `bottom_z` by `stock_to_leave` unconditionally:** Rejected because through-pockets without a model floor at `bottom_z` would stop short and leave a skin at the bottom.
- **Multi-slice Minkowski sampling in `(z - s, z)`:** Rejected because multiple slice unions per level add cost and corner-interpolation complexity, whereas `z - s` plus `r + s` is one slice call and mathematically bounds the 3D distance from below.

**Cost to change later.**
Local to `crates/motion/src/machining3d/ladder.rs`, `entry.rs`, and `floor_levels.rs`. Adding separate axial/radial allowances later only needs passing a separate axial value into `cut_levels` and `ladder`.
