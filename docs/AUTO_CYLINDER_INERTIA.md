<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Single-cylinder automatic inertia

This batch extends accepted f4d2e19's single automatic primitive path with
Cylinder. The independently source-reviewed native capture completed once;
independent actual-data/SDK review is clear. Final fold/source review is pending.
Rust tests, lint and coverage for this feature are unrun. The observation file now contains all 24 actual rows;
its required row count still prevents an empty differential corpus from passing.

The existing typed Cylinder dimensions feed maintained Parry 0.31.1
`MassProperties::from_cylinder(density, length / 2, radius)`. Parry's symmetry axis
is Y; the extracted principal moments are reordered X/Z/Y to SDF's Z-axis
convention. Its local center is unchanged. Existing reciprocal-property admission,
finite-positive checks, numeric diagnostics and manual physical validation remain
in force. Parsed fallback dimensions are used without a second raw-size parser:
missing schema dimensions and nonpositive dimensions retain the already supported
Cylinder normalization. Density absence remains distinct from explicit default
1000 kg/m³ metadata. Manual inertial loading is unchanged.

The same attribute-free local origin, translation, rotation and combined poses
use the accepted constructor, matrix and single-collision accumulation helpers.
No frame-relative, degree or quaternion-attribute family is admitted. Inertial
overrides, multiple collisions, other unsupported geometry and every old numeric
limit remain closed. This adds no public types, dependencies, compiler change,
general inertia framework, simulator/Fuel changes or consumer adoption.

## Maintained kernel and scalar compatibility boundary

Pinned SDFormat Cylinder.cc calls the existing gz-math Cylinder mass-matrix
provider. `Cylinder::MassMatrix` delegates to `MassMatrix3::SetFromCylinderZ`.
That provider orders volume as `((PI * radius) * radius) * length`, then multiplies
by density. Its transverse moment is `(mass / 12) * (3 * radius² + length²)`;
the axial moment is `(mass / 2) * radius²`. Squaring follows the existing pinned
integer-power-two expression. The cylinder's native local tensor is Z-aligned.

Parry orders volume as `(((half_height * radius) * radius) * PI) * 2`, computes
unit moments before multiplying by mass, and stores reciprocal principal values.
Its public extraction therefore does not promise the native scalar operation
order or the original low bits. This matters when rotated tensor products nearly
cancel, as demonstrated for the already accepted box/sphere path. The native scalar and tensor values are now captured; Rust cylinder parity
remains unqualified until the one mandatory hook.

Only newly rotated cylinders extend the existing private SDK-ordered scalar
provider. Parry still supplies admission first; existing unrotated box/sphere
paths and scalar arms remain unchanged. The existing glam matrix multiplication
and transpose, determinant guard, upper-triangle projection and two center-shift
passes then operate without a new formula. Unrotated cylinders use public Parry
extraction and the exact axis permutation. No comparison bound is increased:
public loaded values must meet the existing four-ULP policy. A new private test
also requires exact scalar bits against every actual finite-valid direct native
record; it does not treat invalid native output as a scalar reference.

The adapted scalar expressions preserve gz-math's 2015 OSRF/Apache attribution
in source and NOTICE. This is a narrow compatibility branch, not an independent
primitive library or an automatic-inertia API.

## Fixed reference and regression source

The declared 24 inputs include the original native Cylinder inertial dimensions
(radius 0.1, length 2, density 2170), the exact retained negative, Z-axis checks,
translation/rotation/combined poses, ordinary shape scales, density presence,
XML/default/fallback dimensions, equivalent decimal spelling and explicit numeric
and translation-overflow controls. The dimensions from the original test are
supplemental loader data: they do not port its setter/nullopt/Element contract.

The observer reuses the accepted Root load, direct Collision inertial and second
Resolve protocol. It retains both complete Root snapshots, auto/saved flags,
all direct mass/principal/pose values, all nine direct tensor fields, validity and
finiteness flags, error vectors, stderr warnings, exit statuses and actual maps.
Each of the fixed children has a 10-second deadline. No failed child, warning,
invalid/nonfinite DOM or empty fixture is fabricated into a successful result.
The same capture command ran the unchanged native
`DOMCylinder.CalculateInertial` original once: one pass, zero failures, stdout,
stderr, XML and exit status retained. This is reference evidence, not a complete
Rust original-case claim.

The existing compiler/source/header/schema/DSO provenance recipe is reused with
GCC11's pinned executable identity. Cylinder sources and both public/detail math
headers join the protected inputs. Expanded compiler dependencies are captured
once after compile and checked unchanged after all children; that same dictionary
is recorded. A new output directory cannot overwrite prior outcomes. No SDK or
image rebuild is introduced. Native library maps describe the observer children;
the unchanged original additionally has an explicit loader-resolution witness.

Complete regression source covers native aggregate and direct-value comparison,
manual-policy refusal, density/geometry/pose/source ownership, closed encodings
and overrides, canonical include provenance, frame resolution and clone ownership.
The new same-input helper checks the whole Model/Link/Collision defaults and
owned/source fields, full native errors and idempotent Root snapshots, native
mass/tensor/COM/orientation and the loaded graph. No arbitrary error counts as a
positive. Existing native-valid values outside kernel representability remain
explicit refusals; invalid/nonfinite aggregation remains unsupported.

## Exact reviewed retained assertion disposition

The accepted `auto_rigid_pose.rs` final Cylinder rejection contains this exact
input: one model `m`, link `l`, collision `c`, radius 1, length 2, omitted density,
local pose `1 2 3 0.1 0.2 0.3`, URI `cylinder.sdf`. Independent source and actual
native-data review approved replacing only that assertion with the complete
same-input helper. Its XML/URI are unchanged. Every other old predicate, fixture,
bound and table remains unchanged. Only the temporary prepared-positive wrapper
was removed; the full corpus also uses the helper for this same row while keeping
all its differential assertions. The helper still requires every complete
zero-error/idempotent native value and the unchanged four-ULP bound.

No original complete/partial ledger counts change in this source batch. Source
and protocol review preceded the single bounded actual native capture. Data and
final source review precede one consolidated mandatory Rust hook. No preliminary
test/lint,
arithmetic search, capture retries or comparative performance claim is included.


## Actual single-capture outcomes

The one authorized capture completed in 14.257 seconds with no reserve stop,
all 24 children exiting zero, no timeout and the unchanged original passing.
The selected GCC11, 340 expanded compiler inputs and 11 mapped observer libraries
were retained and checked unchanged. This is correctness evidence, not a runtime
performance comparison. Raw result SHA-256 is
`8e29ac410733fad327520dd1d072f7f5f26daba1eb66235b70d41292647cb13b`.

Raw Root status counts are 20 finite-valid, two nonfinite-invalid translation
aggregates and two finite-invalid density extremes. Of the 20 finite-valid
stored values, the minimum/maximum-radius cases are retained default inertials
following native calculation failure: both report load error 19 and direct
Collision error 19. Their later Resolve vectors are empty. They are not successful
shape calculations, despite finite-valid direct stored defaults. The other 22
cases have empty load/direct/resolve error vectors. Five stderr warning rows are
retained: the two missing-density cases and three dimension-fallback cases.
The exact old negative is a genuine finite-valid, zero-error result, with two
identical complete Root snapshots and the expected default-density warning.

The regressions explicitly check those two error vectors before requiring Rust's
existing Unsupported diagnostic. All six declared numeric refusal controls keep
their exact inputs and bounds. No capture was repeated, candidate searched,
invalid/default DOM reclassified as valid shape data, or Rust result assumed.
Independent actual-data/SDK/PRIOR review is clear. The old rejection now calls the
reviewed stronger same-input helper; final narrow source review precedes the one
mandatory Rust hook. No Rust execution is claimed.
