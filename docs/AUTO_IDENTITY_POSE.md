<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Prepared automatic inertia with a local identity pose

This source-only candidate builds on accepted density Main `e196b687`, whose
complete local gate, exact hosted workflow and independent official audit passed.
It retains the current single origin box/sphere kernel and manual
physical-inertia validator. The
collision's already-parsed Pose must be numeric identity and have an empty frame
reference; any explicit pose attributes remain outside the qualified domain.
The default Euler parser determines values, so ordinary equivalent decimal and
exponent spellings are not a new grammar. There is no raw-text six-zero whitelist,
new mass formula, transform algorithm, dependency or public API.

This permits an explicit local identity pose while preserving its typed collision
value and Source. The exact old input strings still contain their pose element.
Collision has no public HasPose flag; this candidate does not invent one or claim
generic mutable Element identity. Signed-zero bit equivalence and noncanonical
quaternion semantics are not native-qualified by this source-only work.

Nonidentity translation/rotation, degrees/quaternion/frame attributes, multiple
collisions and inertial overrides stay Unsupported. Existing numeric limits,
scalar parsing and all old tolerances remain unchanged. The inherited
effective-density accessor remains unchanged: omitted density
keeps `None` and uses 1000 kg/m³; empty/whitespace density keeps `Some(1000)`.
The same existing kernel and validator compute inertia. Coupling regressions
compare each local identity input with its otherwise identical omitted-pose
control for both shapes and all four density-presence forms. No inference or
recalculation after metadata edits is introduced.

Pinned native `Collision.cc` composes an in-link collision pose with geometry
inertia. Existing native goldens already accept exactly the two retained inputs:
box `pose_identity` in `auto_box_inertial.json` with URI `auto-box.sdf`, and sphere
`explicit_pose` in `auto_sphere_inertial.json` with URI `sphere.sdf`. The box keeps
exact mass 9920 and its tensor; the sphere keeps the exact original density
2170/radius 0.1 mass and tensor. The shared complete helper checks owned
defaults, source/version, pose, physical values and frame observations. The
original corpus bytes remain unchanged. Independent source review approved
replacing only these two historical rejection branches with the exact same-input
complete helpers, preserving every other branch and corpus count. Those folds
are applied and only the two duplicate prepared wrappers are removed. The
complete source batch is reviewed before consolidated mandatory verification;
there is no extra prepared-positive runtime prerequisite.

Regression source also covers numeric-equivalent/default Euler identity values,
closed pose families, density-fallback coupling, preserved multiple/override
refusals, cloning and imported source/frame behavior. This does not complete the native frame-relative
CalculateInertialPoseNotRelativeToLink original or enable simulator admission.

No standalone native probes or preliminary feature-test/lint rounds have run.
The completed source batch and independent review preceded the required hook;
corrections are batched and reviewed before further mandatory verification. Density acceptance is complete; this identity batch still
requires final
independent source review and mandatory qualification under the coordinated
compiler/capacity window. Fuel and all consumer/compiler pins are untouched.

The first complete mandatory hook passed all 50 tooling tests and formatting,
then stopped at Clippy because the shared identity helper loaded the existing
sphere helper file a second time in one test crate. The correction reuses the
already-loaded module through a test-local alias, without an allow attribute,
assertion change, numeric change or production change. Rust-runtime and coverage
qualification remain pending; the failed attempt is retained separately.

The later Rust-unverified translation batch is AUTO_TRANSLATION_POSE.md.
It retains identity rotation and the attribute/frame closures. Only the independently prior-approved unit-translation refusals now use
stronger same-input positive helpers;
this does not rewrite the accepted identity qualification.
