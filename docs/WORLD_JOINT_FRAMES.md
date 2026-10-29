<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# World-joint frame conformance

The unchanged `world_joint_child_frame.sdf` fixture now has a faithful Rust port
of `DOMJoint.LoadWorldJointChildFrame`. This verifies existing parser and graph
behavior; no production API or physics capability changes.

Both world joints resolve their parent/child references to the expected model
links. The child-frame alias retains its pose offset when used as a joint's
default pose frame. Nine semantic queries cover world-relative model, joint and
frame poses and joint poses relative to child-frame and model-link targets.

Every original model/joint/frame presence, name, count, out-of-range, raw pose,
empty relative-to, endpoint and resolved-pose assertion is retained. Native
Pose3 comparisons keep their component-wise 1e-3 bound and direct quaternion
signs. Rust uses the existing owned vectors and public semantic queries; no
generic Element or parallel DOM layer is introduced. Endpoint names returned
fully scoped by Rust are compared in the original's world-relative scope.

Reproduce the reference in the pinned native build:

```
LD_LIBRARY_PATH="$upstream/build/lib:$prefix/lib" \
  "$upstream/build/bin/INTEGRATION_joint_dom" \
  --gtest_filter=DOMJoint.LoadWorldJointChildFrame
cargo test --locked --test world_joint_frames_original
```

The existing native original and the Rust port pass. All 401 Rust tests and
Clippy pass. Independent source, fixture and fresh Rust/native review passed.
Strict staged verification and exact hosted CI remain required before acceptance.
General articulation, physics support for world joints and full SDF-02 remain
separate requirements. No performance result is claimed.
