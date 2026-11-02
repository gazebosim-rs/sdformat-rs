<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Radian Euler pose construction

This source continuation starts at accepted producer `a549b745`. Ordinary
`euler_rpy` poses in radians now use the same private selected-native quaternion
construction already reviewed for automatic-inertia rotation. There is no new
pose grammar, public type, field, default, dependency or compiler change.
Degree and `quat_xyzw` parsing keep their existing implementations.

## Confirmed source gap

Pinned SDFormat `1249805` Param.cc parses a radian pose into the six-argument
gz-math Pose3 constructor. Pinned math `1316ed3` Pose3.hh:97 invokes
Quaternion(roll,pitch,yaw); Quaternion.hh:350–367 evaluates its four ordered
Euler products and calls Normalize. That normalization uses a left-associated
wxyz square sum, square root and individual component division.

The former parser returned glam 0.30.9 `DQuat::from_euler` directly. Its
factorized Euler implementation returns `from_array` without normalization.
The public contract describes unit transforms, rather than preserving raw glam
bits. Its existing squared-norm tolerance of 1e-12 still admits those near-unit
values: this is a selected-native constructor fidelity gap, not evidence that
all previous transforms violated their numeric invariant.

Appending glam `normalize` alone would retain different raw products, xyzw sum
order and reciprocal multiplication. The source correction instead extracts
the already reviewed raw Euler products and first normalization into one private
helper. It uses existing `DVec4` ordered length and component division, then
reorders the result into existing `DQuat` storage. The automatic-inertia matrix
adapter reorders it back and retains its separate second normalization and all
previous matrix expressions/products. No general rotation framework or new
primitive formula is introduced. Original 2012 OSRF/Apache attribution remains.

## Existing evidence and new controls

The unchanged 24-input rotation capture already records the direct collision
inertial quaternion after `collisionPose * identityGeometryPose`. The selected
Pose3 and Quaternion multiplication sources preserve every finite nonzero
quaternion component through that operation. Zero signs are not established by
this composition. New constructor controls therefore compare all nonzero raw
pose components bit-for-bit, and zeros numerically only. No synthetic native
golden, new capture, candidate search or signed-zero bit claim is used.

Those exact XML inputs are loaded with only `auto=true` changed to `auto=false`,
separating pose construction from unrelated automatic-inertia admission. The
same angles, URI and complete captured raw quaternion remain bound. A second
control uses the captured mixed quaternion across model, link, frame, inertial,
collision, sensor, joint and light poses, retaining translation, source identity,
relative frame strings, clone values and existing self-frame/default-frame
queries. Other controls cover equivalent numeric spellings and explicit radian
attributes, omitted/empty/identity values, deferred unknown references, malformed
inputs, exact old degree/quaternion library paths and imported source identity.
All prior tests, fixtures, native rows, graph bounds and automatic-inertia
four-ULP/raw-scalar assertions remain unchanged.

## Limits and qualification

The initial source batch was independently reviewed before its first mandatory
hook. That hook stopped during Clippy compilation as recorded below; corrected
runtime, Clippy and coverage remain unqualified. No native probe or arithmetic
experiment was run. Independent correction review must finish before the
affected consolidated mandatory verification. No old assertion transition or coverage exception is
proposed; original complete/partial case counts remain unchanged.

Degree conversion has a separate known expression-order difference: Rust uses
an existing precomputed PI/180 factor; native GZ_DTOR evaluates angle*PI/180.
This batch intentionally preserves that branch and explicit quaternion parsing,
without claiming additional native-bit qualification for either. Publicly
constructed Transform validation and graph multiplication/inversion stay
unchanged. Raw constructor fidelity alone does not establish Eigen round-trip,
DART joint-frame, B03 trajectory, whole-parser or arbitrary-platform parity.
The retained B03 tick-zero quaternion observation is motivation for this source
review, not a raw-parser golden or a proven cause of later contact divergence.
There is no simulator dependency adoption, Fuel or performance change.


The first mandatory hook passed59 tooling tests and formatting, then Clippy
compilation found one E0308 in the new member-source assertion: loaded Light
source is `Option<Source>`, unlike the other member sources. No runtime, rustdoc
or coverage phase ran. The source correction requires `Some` with an explicit
loaded-light expectation before the same URI/line/column checks; it does not
flatten away a missing source or change production/public types. All other new
source uses were inspected and no additional compiler finding was reported.
The complete correction requires independent review before the affected
mandatory hook; no preliminary check or native round was run.
