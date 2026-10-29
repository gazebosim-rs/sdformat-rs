<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Joint semantic-pose targets

`Sdf::joint_pose(scoped_joint, target)` queries the loaded graph using a canonical
joint identity such as `world::robot::hinge`. An empty target selects the joint's
declared child frame. A child can name an offset frame attached to a link;
resolving to that frame preserves its offset instead of substituting the attached
body. This matters when assembling body and joint transforms for a backend.

Explicit targets follow native Joint SemanticPose local scope. Model joints
accept local names, nested paths such as `gripper::tip`, and `__model__`. World
joints accept world-local model/frame names and `world`. A model joint does not
implicitly search ancestors or sibling scopes: `world`, global-prefixed names,
root markers and a descendant's ambiguous short name are not fallback lookups.
Use a path relative to the containing scope. The existing `pose_of` API remains
available for already resolved canonical source and target identities.

Missing joint identities and unknown target references report UnknownReference.
A canonical identity that names a non-joint reports InvalidValue. Target errors
retain the joint's source location; unknown joint errors retain the document
source. These Rust categories do not claim native error-vector or message-text
identity. The native invalid-target cases return first code 26.

The query reuses existing graph references and pose math. It does not rebuild
graphs after edits to public DOM fields, matching `pose_of`. It returns the loaded
joint pose, not a live pose following physics steps. No dynamics, finite-value
admission policy, axis interpretation or graph mutation facade changes.

## Evidence

The checked-in corpus contains 46 actual native Joint SemanticPose queries:
29 successes and 17 invalid-target rejections. It covers the existing original
relative-pose fixture, nested scopes, duplicate descendant short names, distinct
child-frame/body targets, world-parent endpoints and world joints. Pose checks
retain direct quaternion signs and component-wise absolute `<=1e-3`, as in the
upstream Pose3 equality assertions. Original LoadJointPoseRelativeTo and all
existing Rust joint-frame assertions remain unchanged; no duplicate original
credit is taken. Local wrong-kind and post-edit graph checks are supplemental
Rust API contracts, not native error claims.

`reference/joint-semantic-pose.json` records source and linked-library identities.
The native helper uses Root loading and actual Joint SemanticPose Resolve. It
replays every corpus query, verifies the original fixture bytes and runs the
unchanged native original plus Rust query/frame tests.

```
python3 tools/joint_semantic_pose_reference.py /path/to/pinned/sdformat
cargo test --locked --test joint_semantic_pose --test joint_frames
```

All 420 Rust tests and Clippy pass. Independent source/scope/error review passed,
with 29 focused checks in fresh reviewer-owned source/target, all 46 native
queries and the unchanged native original. The final token-count assertion
tightening also passed independent replay. The normal staged hook, strict
coverage and exact hosted CI remain required before acceptance.
