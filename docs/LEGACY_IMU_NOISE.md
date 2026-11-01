<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Legacy IMU noise projection

For SDF 1.4/1.5 direct root/world models, `load_str` maps legacy IMU
`noise/rate` onto all three angular-velocity Noise values and `noise/accel`
onto all three linear-acceleration Noise values. The shared type and each group's
mean, standard deviation, bias mean and bias standard deviation are retained.
An omitted group has zero values of the selected type. Existing Noise kind,
scalar, finite-value and nonnegative-deviation validation is reused. No sampling,
new value types or dependencies are added.

This follows the two direct-model paths in pinned `sdf/1.6/1_5.convert`.
Included resources use their own original XML scope/version before include
overrides. Nested-model/joint legacy noise, standalone fragments without that
document context, 1.6+ legacy groups and mixed legacy/per-axis groups remain
Unsupported. Native warns/ignores some of these or combines old/new data; this
reader explicitly refuses those unqualified behaviors. Missing/empty type,
unknown data, duplicates and invalid numerical values fail explicitly.

The Sdf version and all URI/line/column sources still describe original input.
`Sensor.configuration_xml` retains the exact old `<imu><noise>...` span, not
fabricated converted XML. Sensor envelope/orientation values remain independent.
Owned clones survive independently. Existing Sensor output writes current
per-axis metadata and does not mutate the old input snapshot.

## Exact original and remaining scope

`Converter.IMU_15_to_16`, Converter_TEST.cc:3017–3145, uses the byte-identical
inline SDF fixture retained here. The partial Rust mapping checks world/model/
link/sensor/IMU presence, Gaussian type, all six complete Noise records and the
original decimal text in the immutable input. It does not claim the converted
TinyXML tree's node order, six copies of each exact output text, the converter's
error vector or console silence. The unchanged full C++ test passes separately.
No general converter, mutable Element or complete-case credit is claimed.

Independent prior review approved only the old `legacy_imu_noise` row's exact
XML and `unsupported-conversion` URI transition to an exhaustive positive. The
historical corpus is byte-identical; `version15.rs` now checks every Model, Link,
Sensor/default/source/raw XML observation, all Noise fields and graph resolution
for that row. Every other old rejection and the gravity positive are unchanged.
The earlier old-rejection failure remains separately recorded.

## Evidence

The native corpus has 69 cases covering direct root/world models, both old
versions, all Noise kinds, missing groups, signed zero, original orientation and
envelope values, an imported old resource, mixed/nested/newer-version behavior,
unknown content, duplicates and numerical failures. Exact original and old-row
typed projections and the unchanged full converter original also pass. Native
source, headers, schemas, compiler and selected library hashes are recorded in
`reference/legacy-imu-noise.json`; foreign Git checks use the existing isolated
context and child preload/audit overrides are cleared.

Independent review passed seventeen focused Rust tests, native69 controls, both
exact projections, the unchanged full converter original, hostile-Git replay,
Clippy and formatting in fresh source/target. The prepared positive now runs in
the retained version15 loop rather than as a duplicate test. Final fold review
and ordinary strict staged/hosted qualification remain pending. Public simulator
revisions 5870373 and 1480725 still pin 0437a54; the private headless integration
pins 486b3e0. This producer slice changes neither pin.

```
python3 tools/legacy_imu_noise_reference.py /path/to/pinned/sdformat
cargo test --locked --test legacy_imu_noise --test imu --test imu_xml --test imu_xml_original
```
