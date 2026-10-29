<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# IMU configuration and standalone sensor XML

`Imu::to_xml` and `load_imu_str` write/read current-schema IMU configuration.
They retain six full Noise records, enable_orientation, localization, custom RPY
and gravity vectors, and both parent-frame strings. Custom RPY remains a raw
radian vector; gravity is not normalized or rejected for being zero/non-unit.
Parent-frame and localization strings remain metadata, without resolving a graph
or computing an orientation. Existing finite-noise and nonnegative deviation
policies are unchanged. The writer rejects nonfinite/subnormal values and lossy
string padding, controls and doubled localization spaces that cannot roundtrip
through its supported XML boundary. It does not broaden the existing parser's
admission policy for every native input spelling.

`Sensor::to_xml` now supports matching imu and force_torque configurations.
It retains the current envelope, raw quaternion pose/frame, topic/frame_id, rate
and flags. Negative IMU rates still reject, while the existing force/torque rate
policy is unchanged. Plugins, opaque extensions, gz_frame_id, mismatched kind/config
and other families remain Unsupported. Joint XML stays explicitly force_torque-only;
old IMU joint rejection controls are unchanged. The sensor writer's schema tag is
selected only after matching the literal family and typed configuration.

Noise validation/emission is shared with force/torque without changing its seven
scalar fields, diagnostic source or output order. The accepted force/torque XML
is byte-identical after extraction, and its original corpus and native exchange
checks still pass. No sampling, physical sensor, Element facade or backend
behavior is introduced. Standalone IMU values have no retained Element identity;
source locations remain available on parse diagnostics.

## Native evidence and original assertions

The corpus contains 24 actual native outcomes: 21 configurations and three
standalone sensor envelopes, including omitted configuration defaults. It covers
all three Noise types, distinct values on all six axes, raw zero/non-unit/signed-zero
vectors, finite extremes, orientation flags, localization strings and escaped
parent frames. Non-pose fields compare exact scalar bits/strings; sensor poses
retain direct quaternion signs and the original component-wise <=1e-3 bound.

The generated fixture and C++ helper, source/library pins and regeneration command
are recorded in reference/imu-xml.json. Low-level SDF parsing plus Imu/Sensor.Load
observes unattached metadata without model graph validation. The helper checks
byte-identical fixture regeneration, actual native XML loading in Rust, and four
real Rust-to-native exchanges: one configuration and all three envelopes. Both
unchanged native ToElement originals pass with empty stderr.

```
python3 tools/imu_xml_reference.py /path/to/pinned/sdformat
python3 tools/force_torque_xml_reference.py /path/to/pinned/sdformat
cargo test --locked --test imu_xml --test imu_xml_original
```

The two partial ports retain all six full Noise equality observations, zero
gravity, custom UnitZ RPY, parent-frame strings, NED, disabled orientation and
edited gravity output. Exact finite Noise equality strengthens native <=1e-6;
vector checks preserve native direct component-wise <=1e-3. Successful typed
results cover the original successful error results, and a child process checks
empty library console output beyond libtest's fixed protocol/duration. Generic
Element/null identity remains explicitly unported, with no fabricated substitute.

All 433 Rust tests and Clippy pass. Independent review passed with 33 focused
Rust checks in fresh source/target, all 24 native IMU outcomes, four exchanges
and both unchanged originals. The reviewer also replayed all 32 FT outcomes
and six exchanges, comparing all six FT exports byte for byte against accepted
review output. The normal staged hook, strict coverage and exact hosted CI
remain required before acceptance.
