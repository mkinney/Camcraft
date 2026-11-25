## #3938 - the short-flute meridian steps out to the full radius

**Decided.** `Tool::meridian()` adds the radial shoulder `(fl, r)` when a
flute ends before the tip reaches the full radius. This applies to the
sharp taper and the drill (the cone arm), and to the ball nose and the
bull nose (the short-flute branch of `tip_arc`). The meridian then ends
where `Tool::profile` ends: at `(flute_len, diameter / 2)`. This is the
rule that #3929 gave the flat and ball taper tips.

**Why.** The issue asks that the meridian and the profile agree at the
flute top for every tool kind. The ball and bull branch has the same
defect as the cone arm, so one rule closes the whole class. The shoulder
is a flat segment. `Meridian::atoms` makes no atom from a flat segment,
so the solid law pairing (`meridian_atoms_match_cutter_atoms_for_each_mill_kind`)
does not change. No production code calls `Tool::meridian()` today, so no
toolpath, posted program or golden changes. The fix stops a later reader
(sub-issues of #3351) from seeing a smaller tool than the profile.

**Threshold.** The shoulder is added when `r - top_r >= MIN_LEN` (1e-6
mm). Under that value the cone or the arc already meets `r` (float dust),
and a shorter segment would be refused as `MeridianError::Short`. A tool
whose flute reaches the full radius keeps the same meridian points, bit
for bit.

**Dropped.**
- Only the sharp taper and the drill (the issue title). Dropped because
  the ball and bull branch keeps the same gap, and "Done means" names
  every kind.
- A change to the solid law (`Cutter::atoms` in `camcraft-sim`) to add a
  disc at the flute top. Dropped: a zero-height shoulder has no volume,
  and the issue is about the meridian against the profile.
- A change to the bull-nose profile, which clamps the corner radius to a
  short flute and so draws a different arc under the flute top than the
  meridian. Out of scope: the two agree at the flute top after this fix.

**Cost to change later.** Low. The rule is two guarded pushes in
`crates/tools/src/meridian.rs`, pinned by
`a_short_flute_meridian_has_the_profile_shoulder`.
