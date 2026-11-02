<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Prepared translation-only automatic inertia

This producer batch follows accepted identity-pose Main `8df897c`. The one
reviewed native characterization has run; Rust qualification remains pending.
It retains the existing single box/sphere geometry kernel, effective density,
manual-auto flags, physical validator and source/version fields. Collision pose
may have a translation, but its parsed rotation must be numeric identity, its
frame reference empty and its raw pose element without attributes. Degrees,
quaternion and frame-attribute families, multiple collisions, manual overrides
and other geometry remain closed. Existing scalar parsing determines numeric
spellings; no raw-text whitelist, dependency or public API is added.

Parry0.31.1's public mass-properties `world_com` and translation pose constructor
supply the transformed shape COM. Geometry mass and principal inertia continue
to come from its existing public cuboid/ball kernels. A general mass/inertia or
parallel-axis implementation is not added.

## Selected native accumulation is observable

Pinned SDFormat1249805 `Collision.cc` composes collision pose with the geometry
inertial pose. `Link.cc` then adds that collision into an initially zero-mass
`gz::math::Inertiald`. Pinned gz-math1316 `Inertial.hh::operator+=` computes the
weighted center `m*t/m`, then executes both the zero-mass and nonzero-mass shift
passes in their fixed order. Even for one collision, the center can round away
from t. The residual shift can change the tensor. Large finite coordinates can
also overflow a square or product and produce nonfinite aggregate values.
The actual 24-case capture confirmed six nonfinite/invalid Root aggregates
while all direct collision inertials remained finite/valid. All native error
vectors were empty, including those six DOMs; they are not reported as native
error codes. The 18 finite aggregates had the same recorded COM/tensor bits as
their direct collision values. This batch did not observe a finite residual
shift, so that source-derived possibility remains unqualified.

Parry's pure pose transform leaves COM at t. Its addition skips zero-mass
operands, while its iterator sum has a different shifted-matrix construction
and matrix reconstruction. Those public operations do not reproduce this
selected native floating-point behavior. Independent source review approved a
small private single-collision compatibility step on the unchanged Parry
mass/tensor values. It preserves component division and both ordered shift
passes, including native `0*inf` behavior. The adapted code retains gz-math's
2016 OSRF attribution and the owned 2026 Apache-2.0 notice.

A nonfinite aggregate is refused as Unsupported, not returned as a different
valid DOM or mislabeled malformed XML. The existing physical-inertia validator
is still applied afterward. It can remain stricter than native for finite
ill-conditioned moments; no threshold or policy is changed. The differential
requires every observed finite tensor to pass the ordinary manual loader as
an explicit control, then requires a successful translated result. All 18
finite observations are positive cases; an arbitrary error cannot count as a
pass. No translation cutoff is introduced.

## Source tests and pending reference data

The existing captured box `pose_translated` row uses its exact XML and
`auto-box.sdf` URI. The former identity test also constructs a unit translation
for both shapes with `closed-pose.sdf`. Independently prior-approved complete-value helpers now replace only those
same-input rejection branches. All other assertions and every old corpus byte
remain unchanged. Box mass/tensor
and the now-observed unit-translated sphere retain their existing exact
values. No broader frame-relative native original is completed.

Regression source covers density presence, manual policy, imported frame/source
and clone retention, native nonfinite accumulation, and a fail-closed
per-observation comparison. The reference source contains 24 ordinary,
nonexact, density-presence and extreme inputs. It records direct collision
inertia separately from whole-Root accumulation, both Load/Resolve snapshots,
errors, nonfinite/validity flags, raw stdout/stderr, child exits/timeouts,
compiler dependencies and the actual mapped-library closure. The one old box
projection must still equal its immutable captured bytes.

The actual native fixture now contains all 24 observations: 18 finite/valid Root
aggregates and six nonfinite/invalid aggregates, with 24 successful child exits
and no timeout or crash. Direct collision output is finite/valid for every input.
Missing-density stderr is retained on both absent-density inputs, including all
warning occurrences; output is not described as uniformly empty or clean. The
immutable old box projection matches exactly. No additional cases were hunted
or characterization rounds run to obtain a finite residual witness.

The Rust differential still requires every input to have a captured observation.
The six nonfinite aggregate cases require typed Unsupported and no returned DOM;
the 18 finite cases require success, the existing manual-policy control and the
reviewed value comparisons. Exact retained same-input checks are unchanged.
Independent final data/fold/code review passed. The first mandatory hook passed
54 tooling tests and formatting, then failed Clippy compilation because a new
manual-policy control compared `Inertial` values without a `PartialEq` impl.
That control now compares all four fields against the same default value; no
public type or assertion intent changes. Rust runtime and mandatory coverage
remain unrun, and corrected Clippy/full qualification are pending. The failed
attempt is retained separately; no preliminary test or native round follows it.
No arbitrary-f64 parity or full native original is claimed. Fuel,
simulator/consumer pins, compiler/dependencies, original-case counts and
coverage policy are unchanged.

A separate [origin-centered rotation draft](AUTO_ROTATION_POSE.md) is source-only.
It does not change this accepted evidence or qualify broader pose families.
