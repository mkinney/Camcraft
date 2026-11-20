## #2873 - Hold the Plasma Profile catalog case for DXF support

**What was decided:** Hold the catalog case until #1870 provides Plasma Profile generation from DXF sketches. Keep the original DXF generator, fixture, build script, and pinning tests on the held branch.

**Why:** The case requires a DXF source. The current generator gives a clear refusal: "a toolpath from the chains of a DXF profile is not generated yet". No toolpath is generated. Issue #1870 tracks this missing support.

**Considered and dropped:** A solid mesh substitute would not meet the case requirements. Implementing the generator here would add work covered by the existing high-effort issue to this medium-effort catalog issue.

**Cost to change later:** After #1870 is complete, run the build script to create `profile.camproj` and `profile.nc`. Complete the case README, run the pinning tests and mutation checks, and run the required gates before opening a PR.
