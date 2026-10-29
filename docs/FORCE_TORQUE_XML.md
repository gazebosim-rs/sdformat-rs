<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Force/torque XML

`ForceTorque::to_xml` and `load_force_torque_str` write and read current-schema
`<force_torque>` configuration without a sensor or frame graph. Frame and measure
direction retain their stored meanings; all three force and three torque noise
records include type, mean, deviation, bias, dynamic bias and precision fields.
Forces remain in newtons and torques in newton metres. No noise is sampled and no
wrench is computed. The existing finite-noise and nonnegative standard-deviation
reader policies are unchanged. The writer rejects nonfinite values, negative
standard deviations and decimal subnormals that cannot pass that reader.

`Sensor::to_xml` supports the force_torque family with a matching typed config.
Standalone IMU output is also supported; see [IMU_XML.md](IMU_XML.md).
It emits current name, raw quaternion pose/frame, frame_id, topic, update rate,
always_on, visualize and enable_metrics, followed by the typed configuration.
Typed edits are authoritative; configuration_xml and source locations retain
input provenance and are not used as stale output templates. Serialization does
not mutate the source object. Loading emitted XML creates new input provenance;
owned-object equality including Source is not promised across a read/write cycle.

Joint XML now reads and writes force_torque sensors in declaration order, using
the same bounded sensor writer. Duplicate sensor names reject. Other joint sensor families,
plugins, opaque legacy extensions, gz_frame_id and joint physics XML remain
Unsupported; existing unsupported IMU joint inputs still reject. No payload is silently
discarded. The full document loader keeps its existing sensor support and graph
validation. Standalone loading retains unresolved pose references, with no
runtime/backend admission implied.

Strings must survive XML and native Param canonicalization: padding, controls,
non-XML characters and lossy doubled text spaces are writer errors. The literal
manual topic value __default__ is rejected because the reader treats it as its
empty-topic sentinel. Poses use existing finite unit-quaternion validation and
explicit quat_xyzw output, retaining the quaternion sign. Decimal subnormal pose
components and rates reject. Negative finite force/torque sensor rates retain
the existing unthrottled metadata meaning; the separate IMU rate policy is intact.
Native Sensor.ToElement omits some envelope flags; this writer preserves the
existing Rust owned flags, verified by actual native loading of emitted XML.

## Native and original evidence

The corpus records 32 actual native outcomes: 27 configuration cases including
three invalid frame/direction rejections, three sensor envelopes and two joints
with one/two sensors. It covers all frame/direction variants, all noise types,
signed zero and finite extremes, distinct values on every axis, edited envelopes
and unresolved pose frames. Non-pose projections match exact scalar bits. Sensor
poses retain direct quaternion signs and component-wise absolute <=1e-3 bounds.

The original native-generated fixture is checked in with its C++ generator and
regeneration command, source pins and linked-library identities in
reference/force-torque-xml.json. The helper verifies byte-identical regeneration,
native XML loading in Rust and six actual Rust-to-native exchanges covering
configuration, all three sensor envelopes and both joint sensor collections. Native
loading uses low-level SDF read plus ForceTorque/Sensor/Joint.Load, without a
model graph. Both unchanged native ToElement originals pass with empty stderr.

```
python3 tools/force_torque_xml_reference.py /path/to/pinned/sdformat
cargo test --locked --test force_torque_xml --test force_torque_xml_original
```

Both original ports retain the six full Noise equality observations,
frame/direction, edited-direction output, successful error results and empty
console observation. Exact Noise equality on these finite values is stronger
than native Noise's component-wise <=1e-6 comparison. A child process admits
only libtest's fixed protocol/duration on stdout and requires empty stderr.
The approved constructor-null translation checks that immutable emission does
not fabricate a loaded input declaration (source=None); see
[XML_ASSERTION_MAPPINGS.md](XML_ASSERTION_MAPPINGS.md). This is not a general
Source/Element identity mapping.

All 427 Rust tests and Clippy pass. Independent source/assertion/boundary review
passed, with 32 focused Rust checks in fresh reviewer source/target, all 32 native
outcomes, fixture regeneration, six real exchanges and both unchanged originals
with empty stderr. The normal staged hook, strict coverage and exact hosted CI
remain required before acceptance.
