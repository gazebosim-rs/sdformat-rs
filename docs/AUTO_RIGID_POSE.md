<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Combined local translation and rotation in automatic primitive inertia

This is an in-development batch on accepted f0d05146. The single actual native
capture and independent data review are complete; Rust/tooling/coverage
qualification remains unrun. It admits simultaneous local translation and rotation in
the existing single-box/sphere automatic-inertia path. It changes only the mixed-
pose guard and its diagnostic wording. There are no new formulas, dependencies,
public fields, compiler requirements, simulator/Fuel edits or consumer adoption.

The collision must still have no pose attributes, including explicit empty
relative_to, degree or rotation_format declarations. Multiple collisions, other
geometry, inertial overrides and all existing numerical-domain/physical guards
remain closed. Density presence and the1000kg/m³ getter fallback remain distinct;
manual auto=false keeps its existing values and collision metadata.

## Existing implementation and pinned source

Collision.cc340–370 sets the collision's local pose on the geometry inertial.
Link.cc670–708 adds it to the zero accumulator. Inertial.hh275–325 obtains the
rotated tensor with Moi before applying both center-shift passes. That order
already exists in the Rust implementation: maintained Parry geometry/admission,
the previously reviewed rotated scalar provider and maintained glam matrix
products, then the previously reviewed single-translation accumulation and
manual physical validation. This change composes those existing paths instead
of maintaining a separate mixed-pose rejection. The helpers themselves are
byte-identical; their original OSRF/Apache attributions remain in NOTICE.

The pose retained on Collision is still the caller's raw typed translation and
quaternion. The computed Inertial uses the aggregate link-frame tensor, computed
center and identity orientation, as in the native accumulator. Imported canonical
source, model/link/collision ownership and loaded frame semantics remain intact.
The existing four-ULP comparisons, manual validator, determinant/domain refusals
and every prior fixture/assertion remain unchanged.

## Declared native characterization

The new24-row source corpus contains12 box and12 sphere inputs. It includes the
exact two held negative inputs, multi-axis/fractional translations and rotations,
an anisotropic box/second sphere radius, omitted/empty/whitespace/explicit default
density, two inherited translation-overflow classes and the already qualified
minimum/maximum-density shape inputs with only unit translation added.

The native observer reuses the existing Root load/direct Collision/Resolve
protocol. It captures both full18-field Root snapshots, automatic/saved flags,
direct14-field collision values, all nine direct Moi fields, validity/finiteness,
all actual error vectors, stdout/stderr and process maps. Each child has the same
10-second deadline. All failures and timeouts are retained and block qualification;
no failed row, invalid/nonfinite DOM or warning is converted to success.

The script checks the unchanged pinned SDFormat/math sources, selected headers,
schemas and same-build libraries, selects the existing GCC11 compiler explicitly with its prior executable hash,
and retains command/input/depfile/DSO witnesses before running children. The
expanded dependency hash dictionary is retained once, compared again after all
children and reused unchanged in the final result. It creates
a new output directory and cannot overwrite a prior native attempt. No SDK or
image build is introduced. During source preparation the observation file was empty and could not qualify
a control. It now contains all 24 actual rows, independently checked against raw
stdout and the recorded SDK/compiler/library provenance.
No Rust test, lint or parser-protocol test has run for this feature.

```
python3 tools/auto_rigid_pose_reference.py /path/to/pinned/sdformat --capture
```

After actual capture, data/source review precedes one consolidated mandatory
verification. A native-valid value outside an inherited kernel/validation domain
keeps its explicit existing diagnostic; no new cutoff or relaxed tolerance is
proposed. The actual direct Moi can be compared against the accepted zero-
translation path; the combined Root tensor must be compared against the complete
native aggregate, not assumed equal to direct Moi when residual center shifts
are nonzero.

## Exact reviewed old expectations

The first pose string in the old auto_rotation_pose.rs closed-family table is
`<pose>1 0 0 0.1 0 0</pose>` for both generated inputs, with URI
`closed-rotation.sdf`. The box is size 2³/density 1240/box_col; the sphere has
radius 0.1/density 2170/sphere_col. Their source XML still comes from the original
pose_identity/explicit_pose rows through the same substitutions as the old test.

Independent source and actual-data review approved replacing only those two
Unsupported expectations with the complete same-input helper. The helper binds
the exact XML/URI and checks all existing Model/Link/Collision defaults, complete
owned/source/frame values, density presence, geometry, native flags, empty error
vectors, idempotent Root snapshots, mass/tensor/center/orientation, non-frame
validation and clone independence at the unchanged four-ULP bound. The old
six-pose table retains every input and order. Every other attribute, override and
multiple-collision negative remains unchanged. The temporary prepared-positive
wrapper is removed; both the original table and the complete corpus call the
same helper. The corpus also keeps its full differential assertions for these
rows. All Rust/tooling/coverage checks on this completed batch remain unrun.

This is not full ResolveAutoInertials, generic frame-relative inertia, native
Element conversion or a simulator physics capability. It adds no original-case
credit, benchmark result or broad phase-completion claim.

The capture preflight identified the established producer SDK recipe as GCC11,
not the separate physics diagnostic GCC13 environment. Before any compile or
probe, the new selector and docs were corrected to the actual prior GCC11
executable and its unchanged recorded hash. No capture outcome or retry is
attributed to that source/preflight correction.

The first released command stopped during driver preflight in0.501s on a missing
`rust_error` dictionary key, before creating an output directory or starting any
compiler/native child. Sphere maximum density uses the earlier actual invalid
Root flags as its inherited refusal qualifier; the other three inherited numeric
rows have explicit Unsupported qualifiers. The amended driver requires either
that explicit prior qualifier or the actual prior invalid Root flags, and protects
the prior native data bytes. This preserves the known refusal rather than treating
a missing key as success. The failed command/log remains separate; no numeric
outcome is attributed to it and the later actual capture remains separately recorded.

## Actual single-capture result

The reviewed continuation completed once in14.003s with no capacity stop, all24
children exiting0 and no timeouts. It used the unchanged GCC11 executable and
pinned local SDFormat/math SDK,340 expanded compiler inputs and11 mapped
libraries. All dependency/protected input hashes were stable across the capture.
This is correctness/reference evidence, not a performance measurement.

All24 direct collision inertials are finite/valid. Root aggregation produced18
finite/valid results,2 finite/invalid results (the maximum-density rows), and4
nonfinite/invalid results (the two translation-overflow classes per shape). Every
load/direct/resolve error vector is empty, including invalid aggregate states;
two default-density warning stderr outputs are retained. No invalid/nonfinite
DOM is treated as a successful Rust result. The18 finite-valid rows contain no
observed center or tensor residual relative to direct values; this does not prove
that arbitrary finite inputs lack such residuals, and the accumulation is kept.

The minimum-density native-valid rows remain inside the inherited explicit Rust
kernel refusal domain. New mixed-pose four-ULP fidelity is still unqualified
until the one Rust mandatory gate. All24 declared inputs remain fixed; no new
candidates or repeated captures were sought. Independent actual-data/SDK review verified every raw observation, compiler
input and mapped library. Final source review of the completed two-case fold
precedes the one mandatory Rust hook; no preliminary verification is added.

Independent actual-data/SDK review: Msgs, report SHA-256
`c523ab38c027a10c2f439aa72922bd3709434c1be4dabbe0184ffd714c31513c`.
It made no Rust/compiler/native replay and granted no Rust qualification.
