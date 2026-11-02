<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Loaded joint semantic-pose evaluation order

This implementation is in development on accepted c94da539. Its first mandatory
hook completed the runtime and numeric checks, then failed syntax classification;
the amended message-binding source and full qualification remain unrun. It changes only the evaluator behind
existing `Sdf::joint_pose`; no public type, dependency, parser, simulator pin,
physics behavior or Fuel work changes. All existing assertions and corpora stay
unchanged. No original case or broad phase completion is claimed.

The pinned `Joint::SemanticPose` passes the joint's name and containing scoped
graph into `SemanticPose::Resolve`. Model::SetPoseRelativeToGraph gives model
members a ChildModelScope, whose source is that model's `::__model__` vertex;
world-owned joints use the world source. FindSourceVertex stops at this source,
not the global/world origin above a containing model. For different vertices,
resolvePose accumulates each entire frame-to-scope chain, then evaluates
`target.Inverse() * joint`. It does not cancel a shared ancestor. Same-vertex
resolution remains exact identity.

The private evaluator follows those edges from the queried vertex toward the
scope and prepends each edge to its accumulator. This preserves the native
association, including an initial identity accumulator. Existing storage merges
a model entity and its implicit frame; the evaluator accounts for the native
identity alias edge when crossing a nested model. An explicit nested-model
entity target starts before that alias, while `nested::__model__` starts after
it. Every intermediate incoming edge also uses its loaded Reference variant and
spelling: default Absolute references enter an implicit model frame, while an
explicit Scoped model name enters the model entity. Raw spelling is retained
throughout the replay; coalesced node kind alone cannot choose an alias edge.
Default targets still mean the declared child frame, not its attached body.

All public target names continue through the existing local lookup. Loaded
pose-parent references are also local schema lookups, so their ancestry reaches
the containing scope; no previously valid public target is rejected. An internal
disconnected-scope guard returns source-located InvalidValue instead of an
ancestry panic. The supplemental private control exercises that guard without
changing public graph admission. Graph queries still use the loaded snapshot
after mutable DTO edits. General pose_of, resolve_pose, axis resolution, cached
world poses, placement-frame adjustment and their small-offset shortcuts are
unchanged.

## Missing maintained-library operations

Locked glam 0.30.9 DQuat multiplication uses the same component expressions and
association as pinned Quaternion multiplication; DVec3 cross matches the native
Vector3 cross expression. These maintained operations are reused. Their unit-
quaternion vector operator and inverse do not implement the SDK's ordered
norm-aware sandwich. A small private boundary therefore implements only:

- Quaternion inverse with the native left-associated w,x,y,z squared norm and
  per-component divisions. Loaded graph rotations are validated unit values, so
  the native near-zero invalid-quaternion fallback is outside these inputs.
- Pose multiplication translation using `q * (pure_vector * q.Inverse())`.
- Pose inverse translation using the SDK's separate two-cross vector operator,
  with its explicit multiply and addition association.

No normalization is added after composition. The existing finite/unit validation
and typed refusal for nonfinite relative results remain. This is not a general
rotation or graph framework. The adapted operations retain 2012 OSRF Apache-2.0
attribution in code and NOTICE.

## Existing actual native evidence

The four golden rows are extracted from the already completed public-plugin
capture, not a new probe. Its exact B03 XML has SHA256
24e85ff944b5efe2d758fdda543e392f34d59f26297a0a87fafbb41d5a3b16aa.
Native result 5ecdb469 and congruence d5d82b4f bind immutable cb921, compiler,
686 inputs, 37 mapped files before/after and the pinned SDK. The independent
actual audit is 9f305d8e. All101 native body/joint samples are numerically equal
to the saved actual Server's16362 values. That is native observer/construction
evidence, not a producer runtime result.

Every actual named joint Resolve(defaultChild) returned an empty error vector,
identity quaternion and translation
`[0, -4.440892098500626e-16, -2.220446049250313e-16]`. The retained twelve raw
semantic rows and their seven-component bit projections are checked in with the
unchanged complete XML. Nonzero components are exact bit assertions; zero values
have numeric comparisons only, with signed-zero fidelity explicitly unqualified.
The other regressions cover nested/world scope, model aliases, owned sources,
loaded-snapshot edits, zero/negative-quaternion metadata, URDF queries, malformed
input and typed query errors. Synthetic semantic cases are not native goldens.

Actual paired native/current-Rust captures establish small initial translation
differences and later trajectory divergence; they do not prove this missing
operation causes it or that this producer correction fixes B03. No consumer pin
or physics tolerance changes. All original joint-pose <=1e-3 corpus bounds,
URDF/frame tests and general-graph assertions remain untouched. Independent source review passed for the first complete batch and the incoming-
reference alias correction. The new message-binding correction requires its final
independent source recheck and one affected ordinary mandatory hook.

## First mandatory result and completed source correction

The first normal hook stopped after394.046s without a capacity stop or commit.
It retained723 passing instrumented Rust tests/213 summaries,59 tooling checks,
fmt/all-target Clippy/rustdoc, and37 production file summaries with9017/9065
lines and14308/14587 regions. Changed executable DA coverage was77/78, above
the unchanged95% threshold; graph.rs503's genuinely cold `)?;` remains in that
denominator. This is precursor evidence, not whole-hook acceptance.

The final assertion was changed-source opaque syntax. One Root-authorized pure
query of the already retained source-identical AST helper identified only the
diagnostic string argument at old graph.rs523 as opaque among the pending graph
rows. Existing import, closing, delimiter and exact private-test declaration
proofs were present; comments/blank rows need no new proof. No helper or coverage
policy changes were made.

The source-only correction binds that identical static message before the lazy
closure and passes the local into the same error constructor. There is no success-
path allocation; error code, source, message and laziness are unchanged. All
numeric operations, data, tests, native observations and tolerances remain exact.
The failed tree/raw reports and pure query remain separately retained; no checks
have run on this amended source.
