## #1861 - Sketch chaining, outer-versus-hole nesting and open-chain safety

**What was decided.** Add `camcraft-geometry::sketch_chains`: one call,
`chain(&[LayeredPrimitive], ChainOptions) -> Result<SketchChains, ChainError>`,
that turns the DXF reader's lines, arcs and circles (#1860) into closed loops
and open chains. No new dependency, no file I/O, no application state.
Primitives and the join tolerance are in millimetres; the caller scales a
document by its units first.

- **Joining.** Curve ends within the join tolerance (default 0.01 mm, allowed
  1e-5 mm to 10 mm) are clustered with a uniform grid and union-find. Ends
  join only on the same layer, because the layer name is part of a chain's
  identity. A gap wider than the tolerance is never bridged: the chain stays
  open and reports both free ends. A bridged gap is reported as `max_gap`.
- **Curves stay analytic.** A chain segment is the input primitive turned to
  the walk direction (`reversed`, with the arc's start angle and sweep sign
  rewritten). Nothing is flattened, snapped or healed, so a later operation
  can still post G2/G3 from centre, radius and sweep sign.
- **Canonical order.** A closed loop is walked counterclockwise (by its exact
  signed area, arcs included) from its lowest joint in (x, y). An open chain
  starts at its lower end. The result does not depend on the order or the
  direction the primitives were drawn in.
- **`ChainId`.** The 128-bit FNV-1a hash of a version tag, the layer name, the
  closed flag and the curves in canonical order, each quantized to 1e-6 mm.
  The minimum tolerance is above that quantum, so two joints of one chain
  never share a quantized position and the canonical start is unique. The
  value for a reference slot is pinned in a test, and checked there against a
  DXF polyline of the same shape, because a project file may store it (#1862).
- **Refusals are per chain and named.** `Err` is only for input nothing can be
  read from: a bad tolerance, a non-finite, out-of-range or invalid primitive,
  or a drawing over the limits. Everything else comes back in the result:
  `closed`, `open`, and `rejected` (layer, input indices, `ChainError`):
  `DuplicatePrimitive`, `DegenerateSegment`, `Branch`, `SelfCrossing`,
  `ZeroArea`, `DuplicateChain`. `SketchChains::strict()` turns the first
  rejection or ambiguous nesting into an `Err` for callers that want all or
  nothing. Every input primitive ends in exactly one chain or one rejection.
- **Branches are a diagnostic, not a traversal.** Three or more ends at one
  joint reject the whole connected component with `Branch { at, degree }`.
- **Self-crossing.** A closed loop is tested exactly, line against arc against
  circle in closed form, over a watertight copy whose joint gaps carry straight
  bridges. Contact within the tolerance of a joint between the two elements
  that meet there is the joint itself; any other contact, touching included,
  is `SelfCrossing { at }`.
- **Nesting.** Depth is counted by an exact crossing-number test against the
  arcs themselves. Even depth is `Outer`, odd is `Hole`, with the innermost
  container as `parent`: the even-odd rule of `geo2d::Region`. Nesting is read
  over every closed chain passed in, across layers; a caller that wants the
  nesting of one layer passes only that layer's primitives.
- **Crossing loops get no reading.** Two closed chains that cross or touch
  keep their geometry and identity, but their `nesting` is
  `Err(NestingAmbiguous)`, and so is that of every loop inside one of them.
- **Limits.** Coordinates below 1e7 mm, at most 2,000,000 primitives, and a
  budget of 200,000,000 pair tests per call. Over the budget the call returns
  `TooComplex`; it does not hang.

**Why.** A wrong outer-or-hole reading puts the cutter on the wrong side of a
contour, so every doubtful case is refused or left without a reading instead
of guessed. Partial results follow the Prime Directive's "prefer recoverable
states": a stray scribble on one layer must not block the import of a clean
part on another, and the rejection says which primitives and why.

**Sources.** Fixed-radius neighbour search on a grid (Bentley, Stanat and
Williams 1977); union-find (Tarjan 1975); FNV-1a (IETF draft-eastlake-fnv);
sweep and prune with the widest-spread axis (Cohen, Lin, Manocha and Ponamgi
1995; Ericson 2005); segment, line-circle and circle-circle contacts (O'Rourke
1998; Schneider and Eberly 2003); crossing-number containment (Shimrat 1962;
Haines 1994); area by Green's theorem. Each submodule names its own. No vendor
code, file or output was consulted.

**Considered and dropped.**
- *Reuse `boundary_chains::group_runs`.* It joins point lists in pick order on
  fixed quantized keys; it has no caller tolerance and no curve metadata (the
  earlier attempt's finding on the issue).
- *Nest with `geo2d::Region` / Clipper directly.* `Region::new` unions and
  renormalizes its loops, so a result loop cannot be traced back to a
  `ChainId`, and it needs flattened arcs, where chord error can flip a
  near-tangent containment. The exact test uses the same even-odd rule; a
  test compares its reading with `Region` over flattened copies.
- *Deterministic traversal through a branch* (leftmost turn). It would pick
  one of several loops for the user without saying so.
- *Snap joined ends together.* It would move arc ends off their circles and
  change centre or radius.
- *`std` hashing for the id.* Its output is not stable across Rust releases.
- *One global error for any bad chain.* See "Why".
- *A plain x sweep for the crossing search.* A tall outline with finely
  divided vertical sides kept every side open at once and ran into the budget
  (pinned by a test).

**Known limits.**
- Two curves that meet at a tangent cusp (the path turns back on itself along
  a shared tangent) can read as `SelfCrossing` when coordinate rounding makes
  them cross a little way from the joint. Tangent continuations (fillets,
  S-curves) are not affected and are pinned by a test.
- Open chains are not tested for self-crossing: an engraved script letter may
  cross itself.
- There is no flattening helper; the operations that need polygons (#1867)
  choose their own chord tolerance.

**Cost to change later.** The id recipe is the expensive part: changing the
quantum, the canonical order or the fed bytes changes every stored `ChainId`.
The version tag in the hash (`camcraft-sketch-chain-v1`) exists so that such a
change is a new, non-aliasing id space. Everything else is internal to the
module.
