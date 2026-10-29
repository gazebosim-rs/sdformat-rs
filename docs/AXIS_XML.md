<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Standalone axis XML

`Axis::to_xml()` emits one current-schema `<axis>` with direction/frame,
dynamics, joint limits and stops, and optional mimic parameters. `load_axis_str`
returns owned, unattached metadata. It does not resolve expressed-in frames,
leader joints or leader-axis names. The containing model's normal `load_str`
validation still checks those references and its original version gates.
Standalone `axis2` roots and legacy conversion directives are not supported.
Units stay unchanged: rotational/linear interpretation belongs to the joint.

Output uses quick-xml escaping and the existing normalized-direction rule.
It emits current typed values, including edits, independently of retained raw
XML. Input padding on string attributes follows native Param space/tab/newline
trimming. The writer rejects padding and control characters that would change
through its attribute encoding. Quotes and ampersands roundtrip normally.

Lower/upper limits may be inverted, infinite or NaN. Canonical NaN signs and
signed zero are retained; NaN payload preservation is not promised. Negative
effort and velocity encode the native unbounded sentinel. Dynamics, stop and
mimic scalars must be finite. Decimal subnormals that the existing GNU reader
rejects are explicit writer errors, including direction components that become
subnormal after normalization. Serialization leaves the source object intact.
These are XML admission rules, not a physics policy.

## Recorded normalization difference

The published Rust parser and setter normalize every admitted direction. Their
protected UnitX assertion at input `[1e-6, 0, 0]` is unchanged. At that boundary,
native JointAxis admits the vector using a float zero epsilon, but gz-math's
double-epsilon Normalize leaves its tiny magnitude unchanged. The new standalone
API retains the existing Rust unit-axis contract. The actual native tiny-vector
oracle stays in the corpus; its named case independently checks Rust UnitX and
all other metadata. Sixteen native outcomes match exactly, with this one explicit
XYZ difference. No old assertion or tolerance was changed.

## Native and original evidence

The native-generated `axis_to_element_native.xml` fixture is checked in with its
C++ generator, regeneration command, source pins and library hashes in
`reference/axis-xml.json`. The helper verifies byte-identical native regeneration,
loads that native XML in Rust, and loads actual Rust output in native JointAxis.
The native path uses low-level SDF parsing followed by JointAxis.Load, without
Root/model graph validation, matching the unattached metadata boundary.

```
python3 tools/axis_xml_reference.py /path/to/pinned/sdformat
cargo test --locked --test axis_xml --test axis_xml_original
```

The ToElement original retains XYZ/frame, all dynamics/limit/stop/mimic fields,
its original comparison bounds, and successful typed results for empty errors.
Explicit output group checks preserve child presence. A child process checks
that the Rust operations add nothing to stdout or stderr beyond libtest's
known protocol, rehosting the original empty-console observation. Constructor
Element and general mutable DOM contracts remain separate.

All 409 Rust tests and Clippy pass. Independent source/error-boundary review,
fresh focused Rust tests and final native/XML exchange replay passed. Strict
staged coverage and exact hosted CI remain required before acceptance. Existing native-build evidence
does not complete the reproducible reference image, SDF-02 or any physics backend.
