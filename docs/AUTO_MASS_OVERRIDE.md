<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Origin single-primitive automatic mass override

This batch starts at accepted `1594ac36`. Complete source/protocol review
preceded one actual capture attempt. All 24 children ran: 23 exited zero; the
minimum-subnormal scalar produced the observer's controlled exit 2 without a
usable required model/single-link DOM. The initial all-zero-child assertion
stopped packaging before the selected original invocation, so the first capture
remains a failed harness result. Its outputs and original driver are preserved.
The reviewed retained-output finish has now completed: it did not compile or
rerun any observer, and the previously uninvoked original passed once. All 24
observations are retained, including the load-only refusal. Rust, lint, helper,
arithmetic searches and coverage remain unrun. Independent actual SDK/data and
PRIOR review cleared only the six declared old inputs, now routed through the
same complete helper. Final fold source review is pending.

The selected domain is one local, origin-centered, numeric-identity Box or Sphere
collision with automatic inertia and only an optional mass child. The collision
can use the already admitted explicit default-radian attributes. Model/link
world poses are independent of this local geometry restriction. Missing mass
uses the original path. Empty or whitespace mass uses the existing schema scalar
default of 1; otherwise the existing scalar conversion supplies the finite value.
Mass must be positive. Requested mass is retained exactly as parsed, rather than
roundtripped through reciprocal storage. Density presence and geometry metadata
remain unchanged.

## Exact pinned semantics and maintained API

Pinned SDFormat `1249805d` Link.cc673–703 tests actual mass-child presence, not a
comparison with the default value. Presence disables its default-collision-density
warning. It still calculates every collision and adds each inertial first. Then
it divides each accumulated diagonal/off-diagonal moment by accumulated mass,
multiplies that unit-mass result by the requested mass, and keeps accumulated
COM pose. Root.cc invokes this resolution during ordinary loading and default
SAVE_CALCULATION prevents a second call from changing the saved result.

Link's automatic load path does not read user inertia or inertial pose; native
warnings describe overwriting those fields. This slice does not adopt them.
Inertial density, fluid added mass, custom parameters and multiple collisions
remain outside the selected automatic subset. Native scaling does not validate
its new result or add a new error at that assignment; zero, negative, nonfinite
and failed-geometry-default outcomes must remain observable in the capture and
must not be called supported positives just because errors are empty.

Locked Parry 0.31.1 supplies `MassProperties::set_mass(M, true)`. It preserves COM
and inertia frame while adjusting angular inertia with mass. It stores inverse
mass and inverse principal moments; internally inverse moments are multiplied
by `(1/M) * previous_mass`, then public extraction reciprocates them. These
operations are mathematically equivalent to native scaling but not the identical
floating-point expression order. This is a real API precision boundary, not a
claim of exact bits or proven four-ULP agreement. The unchanged four-ULP helper
must qualify the complete fixed corpus; a mismatch will be reported before any
custom compatibility math, cutoff or tolerance change. No custom primitive or
scaling formula is implemented here.

The existing geometry/kernel, aggregate finite and manual physical domain must
pass before adjustment. This prevents an override from rescuing an already
unsupported base representation. Adjusted moments must stay finite and positive,
then pass the same manual validator. Reciprocal overflow/underflow can therefore
exclude native-valid extremes; that explicit Unsupported domain is recorded,
without an arbitrary mass or density threshold. Existing invalid/nonfinite and
ERANGE scalar conversion errors remain InvalidValue. No-mass numerical paths
are unchanged; the shared diagnostic wording now mentions compatible options
rather than claiming that every override is forbidden.

## Fixed native evidence and regression source

The 24 predeclared cases include six exact retained inputs, ordinary unit and
anisotropic boxes/spheres, fractional dimensions/masses/densities, default density,
empty/whitespace mass defaults, wide mass scales, zero/negative/nonfinite and
scalar-underflow values, representability overflow, and a base density overflow
that cannot be rescued by a target mass. No candidate search or new formula is
used to choose passing examples.

The existing GCC11/SDK/header/schema/build-library/dependency/maps/deadline
protocol is reused. The observer keeps both complete Root snapshots and all
flags/errors, direct unscaled Collision state and all nine Moi entries. It
records mass presence and explicitly supplies direct `warnIfDensityIsUnset =
!massSpecified`, matching Link's context for this subset, where Link density and
custom parameters are absent. Thus an independent direct call cannot fabricate
a Root default-density warning. Every raw stderr/error/default/nonfinite outcome
is retained. The two old corpus Root stdout projections and stderr must exactly
match existing native recordings; their XML and URIs are unchanged.

The unchanged original
`DOMLink.ResolveAutoInertialsWithMassAndDefaultDensity` was selected for the
capture protocol and actually ran once during retained finishing, after the
first attempt stopped before its invocation. It includes multiple collisions beyond this Rust subset, so its
native pass cannot complete the Rust original or change ledger counts. Source
also documents `ResolveAutoInertialsWithMass`, which includes inertial pose/inertia
overrides; those payloads remain refused here.

The complete helper checks owned Model/Link/Collision defaults and metadata,
manual mass/default tensor, exact requested mass, both actual XML-specific Source
locations, native aggregate and direct values at unchanged four-ULP, flags/errors,
idempotence, frame resolution and cloning. Additional source regressions cover
density absence/empty/whitespace/1000, equivalent numeric identity poses, canonical
includes, positive-context restrictions, malformed scalar and mass attributes,
nonpositive density, overrides and multiplicity. These checks remain unrun.
Negative reference rows are not treated as arbitrary successful errors; only
explicit declared typed refusals qualify. Actual native outcome assertions were
reviewed after retained finishing before the six old dispositions; Rust numerical
qualification remains pending.

## Six reviewed prior dispositions

Only these exact old generated inputs are approved for complete positive helpers:

- `auto_box_inertial.json` `mass_override`, unchanged XML and `auto-box.sdf`.
- `auto_sphere_inertial.json` `manual_override`, unchanged XML and `sphere.sdf`.
- `auto_identity_pose.rs` mass=1 override for its origin box and sphere,
  unchanged `override.sdf` (two inputs).
- `auto_default_pose_attributes.rs` mass=1 override only for its two origin
  box/sphere identity-format inputs, unchanged `override.sdf` (two inputs).

Independent actual SDK/data/PRIOR review approved these six exact inputs. The
four old Rust files now import and use the same complete helper only for those
branches; original arrays/order/counts, corpus bytes and all other assertions
remain unchanged. Rotated/mixed/transformed mass overrides, Cylinder/Capsule,
no-collision manual-domain tests and every other old negative remain refused.
There is no prepared-runtime-before-fold prerequisite: independent actual
reference data and Source/PRIOR review approved exactly these six transitions
before one consolidated mandatory hook. No simulator pin, public API, dependency, Fuel,
original completion/partial count or performance claim changes.

Complete source review found the initial boolean Geometry match would trigger
Clippy's match-like-matches rule. The completed source uses an ordinary
value-returning match that reads the same mass node with the same scalar method
for Box/Sphere and returns Unsupported otherwise. It adds no macro opacity or
policy exception and changes no domain, arithmetic or diagnostics. No lint,
helper query or product run was used to make this source correction.

## Controlled range refusal and retained-output finish

The first attempt ended in 13.506 seconds without a reserve stop. Its exact
minimum-subnormal input is unchanged: native stdout is `LOAD 9 9 9 9 9 30`,
stderr reports Param.cc out-of-range for `5e-324`, and the program returns its
reviewed required-model/single-link failure code 2. It is not an abort or timeout.
The observer did not reach its map write or Root/Collision observations for that
row. No numeric state, status, direct-warning flag or actual map is invented.
The other 23 children exited zero and have complete raw snapshots and maps;
the two old Root stdout/stderr projections matched their captured predecessors.

The initial driver source was copied before the packaging correction and its
hash matches the actual protected input. The new classifier accepts only that
exact case, mass text, exit, stdout and stderr, requiring the map to be absent.
Every other nonzero child, signal, timeout, changed diagnostic or missing normal
map remains a failure. Normal observations and the controlled load refusal
remain all 24 rows, with the original exit codes retained.

`--capture --finish-retained` reads only existing observer outputs, compiler
records and maps; it never compiles or reruns an observer child. It requires the
old driver witness, unchanged other protected/compiler/library inputs, no prior
result, empty fixture and no previous original command. It invokes only the
selected unchanged original once, which demonstrably was not reached before.
All preexisting raw files are hashed before and after; first-capture source and
finish/classifier source are recorded separately. The first failed outer result
is not relabeled. Complete correction Source review cleared the retained finish;
it completed after Root release without any observer execution or recompilation. No new input/candidate, numerical
policy, production code, original expectation or four-ULP bound changes.

Outcome assertions precede Rust refusal dispatch: native zero mass has a finite,
valid zero matrix but is refused by Rust's positive-mass policy; negative mass is
finite-invalid; NaN, infinity, maximum mass and the base-density-overflow aggregate
are nonfinite-invalid. Minimum normal mass is native finite-valid but remains
explicitly Unsupported in the public reciprocal kernel domain. Base-density
overflow retains load/direct error 19, while the other complete rows have empty
error vectors; Resolve vectors are empty. All complete direct statuses are
finite-valid, so they cannot erase failed-calculation diagnostics. The load-only
row separately requires exactly its recorded errors and no fabricated state.
Actual-data/SDK and old-six PRIOR review still precede any original assertion
transition or one mandatory Rust hook.

The retained finish exited zero in 11.756 seconds without a reserve stop. All 81
preexisting capture files remained byte-identical; the old driver witness and
new finish/classifier hashes remain separate. Its pending original completed
once with one pass. There are 72 protected inputs, 340 expanded compiler inputs
and 11 observer libraries, with actual maps for the 23 complete observations;
no map is claimed for the load-only refusal. Result SHA-256 is
`2f39c9fdf1d99cbcc09d5a1879493c6eb6852fd3a37729c0198e3d7865444fce`,
and fixture SHA-256 is
`cfb52fc148f391c9d1ec035185ccf55062d420eb8d11751e6af67ef86604a13d`.
Native finite-valid Root flags occur in 18 complete rows, including zero and
minimum-normal masses that remain Rust refusals. One negative row is finite
but invalid, and four complete aggregates are nonfinite-invalid. Every direct
stored flag is finite-valid, including the error19 case. These observations
are not Rust numerical qualification or original completion credit. Independent
actual SDK/data and old-six PRIOR review are clear (review `dba9dbf6`). Only the
six approved calls are folded; final fold Source review still precedes the one
mandatory Rust hook. No duplicate prepared wrapper was present to remove; all
24 differential observations and independent complete-helper calls remain.
