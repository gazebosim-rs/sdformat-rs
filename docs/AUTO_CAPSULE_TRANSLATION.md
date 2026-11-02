<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Translation-only automatic capsule inertia

This batch starts at accepted a44bf8be. The source-reviewed single native capture
is complete and independent actual-data/SDK review is clear. The two exact old
translations now use the prior-approved complete positive helper. All Rust,
lint and coverage checks remain unrun. The mandatory 24-row assertion still
prevents an empty observation file from qualifying a differential test.

The Capsule match guard now requires numeric-identity rotation rather than the
entire Transform being identity. Attribute-free local translations therefore
reuse public Parry mass, principal moments and world center, followed by the
already accepted ordered single-collision accumulation. The kernel axis mapping,
native density-prefix and public finite-matrix checks, manual physical validator,
nonfinite aggregate refusal and four-ULP comparisons are unchanged.

Rotated and mixed capsules remain Unsupported. Explicit frame, degree and
quaternion attributes, overrides and multiple collisions remain closed. Raw
collision translation/source/density/shape metadata is retained. Manual loading
keeps its original fields; default density presence remains distinct from the
effective 1000 kg/m³ value. No public types, dependencies, compiler, simulator,
consumer pin or Fuel behavior changes.

## Exact pinned behavior and chosen domain

SDFormat Capsule::CalculateInertial installs the centered mass matrix returned by
gz-math Capsule::MassMatrix. Collision.cc applies the caller's local collision
pose before Link.cc adds the result to the zero accumulator. Inertial.hh then
computes mass-weighted center and applies the two center-shift passes. These
operations already exist in the accepted translation helper; this change adds no
capsule formula, aggregation provider or new math framework.

The computed Inertial center must be compared to actual native aggregation,
not assumed bit-equal to the raw collision translation. A center residual can
contribute to the tensor. Invalid/nonfinite aggregate/default/error outcomes
remain explicit diagnostics. New Rust precision remains unqualified until the consolidated mandatory hook. No arbitrary translation cutoff,
roundoff relaxation or custom residual formula is introduced.

Rotation is deliberately a later functional slice. Native Capsule mass properties
come from an ordered cylinder-plus-two-hemisphere construction, while Parry uses
an analytic combination. Origin agreement does not prove near-zero products of
inertia after rotation. This batch leaves that unqualified family closed and
uses the maintained kernel first within the chosen translation domain.

## Fixed native source and regressions

The 24 declared inputs include both exact old translation cases, integer and
binary/decimal/exponent translations, independent fractional coordinates,
ordinary shape scales, schema/default/nonpositive normalization, density
presence, aggregate overflow and inherited kernel/native-prefix/finite-validation
refusals. All inputs are fixed before capture. No arithmetic or candidate search
has been performed to predict a pass.

The accepted observer/driver recipe retains two complete Root snapshots,
automatic/saved flags, direct Collision mass/tensor/pose and all nine direct Moi
entries, error vectors, finite/valid flags, stdout/stderr, deadlines, exits and
actual maps. GCC11, pinned SDK sources/headers/schemas/build libraries and
expanded dependencies are checked with the existing protocol. The exact old
corpus input must reproduce its already captured native stdout/stderr byte for
byte. The unchanged original DOMCapsule.CalculateInertial is invoked once in
the same bounded capture, with all failures retained. No SDK or image build or
native retry is part of this recipe.

Regression source compares direct centered inertia against the accepted origin
control separately from complete native aggregate mass/tensor/center/orientation.
It preserves existing manual-policy errors, metadata/defaults/source ownership,
frame resolution, canonical includes and cloning. The complete same-input helper
also binds exact XML/URI, Model/Link/Collision defaults, density presence, geometry,
native errors/statuses/idempotence and direct raw pose. Comparisons keep the
existing four-ULP policy and its numeric-zero signed-bit limitation.

## Exact two reviewed transitions

All inherited fixtures and all unrelated assertions remain unchanged. Only the
two independently reviewed same-input dispositions change:

- The existing auto_capsule_inertial corpus row `translated`: radius 1, length 2,
  omitted density, pose `1 2 3 0 0 0`, URI `translated.sdf`. Its immutable JSON
  still records Unsupported. A narrowly named helper branch now runs at the
  old Rust error-dispatch point, after preserving its existing native snapshot,
  error/status and warning assertions. Every other row remains unchanged.
- The first pose string in the old capsule pose-closure table: radius 0.1,
  length 2, density 2710, pose `1 0 0 0 0 0`, URI `closed-capsule.sdf`. Only that
  row now calls the same complete helper; the eight-entry table, order, seven
  other negatives and all override/density/multiplicity controls remain intact.

Both prior-approved expectations are folded. Only the duplicate prepared-positive
wrapper is removed; the original calls and new differential corpus use the same helper. The old
rotated/mixed corpus cases and the earlier Cylinder test's mixed Capsule negative
stay refused and unchanged. No upstream original completion/partial counts change.

Independent complete source/protocol review preceded the one actual capture.
Actual SDK/data and exact-input prior-transition review are clear. Final fold
source review precedes the consolidated mandatory hook. No individual-edit tests, lint, helper or native probes run.


## Actual single native capture

The one authorized capture completed in 14.255 seconds without a reserve stop.
All 24 children exited zero with no timeout, the exact old-input stdout/stderr
closure passed, and the unchanged native original completed once with one pass.
The selected compiler, 340 expanded inputs and 11 mapped observer libraries were
retained with stable hashes. This is correctness/reference evidence, not a
performance comparison. Raw result SHA-256 is
`c5fec41ab31aff53149960905db8e939fc04480b7e9765ea0056f30eab5d4d68`.

Root has 22 stored finite/native-valid results and two nonfinite-invalid
translation-overflow aggregates. Two of the 22 are retained defaults following
maximum-density/native-prefix load and direct error 19; every Resolve vector is
empty. The other 22 cases have empty error vectors, including the overflow
aggregates. All 24 direct stored values have finite-valid flags, so those flags
must not turn failed calculations into shape positives. Five warning rows are
retained: the old default-density translation, three dimension fallbacks and
missing density. Both Root snapshots are identical for every declared input.

The integer-translation case has an actual nonzero-coordinate COM residual in Z
relative to the direct collision position. Its tensor and all other observed
zero-error finite-valid tensors have no numeric residual relative to direct Moi in this
fixed corpus; this does not prove a universal equality. The implementation and
assertions still compare full aggregate values independently from centered
controls. No extra candidates, captures, arithmetic searches or bounds changes
were made. Rust behavior remains unrun. Both exact old translations now use the
complete helper after actual-data/PRIOR approval; every other old predicate is retained.
