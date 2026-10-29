<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Complete joint-pose conformance

The existing unchanged `joint_complete.sdf` fixture now has a faithful port of
`DOMJoint.Complete`, separate from the already mapped `DOMJointAxis.Complete`.
The new case preserves successful load, model presence, all nine indexed raw
poses, the named screw joint and classic pitch 20. Translation and quaternion
components keep the native 1e-3 comparison bound and direct signs. The pitch
observation keeps EXPECT_DOUBLE_EQ's four-ULP bound.

For this loaded fixture, native `joint.Element() != nullptr` maps to presence of
the exact `joint` child with explicit name `screw_joint` in the existing immutable
loaded model snapshot. The match uses the name of the same typed joint returned
by the lookup. It tests loaded source identity. No new production API, generic Joint Element facade,
constructor Element contract or lifetime/mutation claim is introduced.

The unchanged native original and Rust port pass. Reproduce them with:

```
LD_LIBRARY_PATH="$upstream/build/lib:$prefix/lib" "$upstream/build/bin/INTEGRATION_joint_dom" --gtest_filter=DOMJoint.Complete
cargo test --locked --test joint_complete_poses_original
```

All 402 Rust tests and Clippy pass. Independent source/assertion review and fresh
Rust/native replay passed. Strict staged verification and exact hosted CI remain
required. This is conformance evidence for existing parsing and
loaded metadata. It adds no joint physics or performance claim; full SDF-02 remains
open.
