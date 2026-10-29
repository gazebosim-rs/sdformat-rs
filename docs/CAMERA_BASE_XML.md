<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Base camera XML

`Camera::to_xml` and `load_camera_str` cover the base Camera.ToElement surface:
identity, raw pose/frame, horizontal FOV, image size/two typed formats/anti-aliasing,
near/far clip, visibility, saving path, camera-info/trigger topics and triggering.
Basic camera noise type/mean/deviation and default-valued calibration presence
also roundtrip. No images are rendered or saved; paths remain strings.

Nondefault depth, segmentation, bounding-box, distortion, lens/calibration values
and unsupported image-noise bias/dynamic/precision fields return Unsupported.
Both calibration-presence flags are retained with default-valued blocks.
Omitted default-zero metadata must have canonical +0 bits; signed-zero distortion,
lens.c3, hidden calibration skew/tx/ty and advanced noise fields reject. Fields
that are emitted retain their actual zero signs, verified in native loading. The
constructor's published values remain unchanged. Matching Camera Sensor/Joint
output stays Unsupported in this slice, preserving existing payload boundaries.
Generic Element/null identity, advanced optics and runtime behavior remain open.

The base boundary requires a nonempty, lossless name; the native-loaded omitted
name is __default__. Strings reject padding, controls and doubled text spaces
that native Param canonicalization would change. Finite values and unit quaternions
use the existing checks; unreadable decimal subnormals reject. Image dimensions
and anti-aliasing must fit the native signed-int schema domain. HFOV is within
[0.001,6.283186], near>=0 and far>=0.1. No near/far ordering rule is added.
An enabled save needs a path; an owned disabled save with a retained nonempty path
rejects instead of silently discarding it. Pose output uses quat_xyzw and keeps
the quaternion sign; frame references remain unresolved metadata.

## Native parser corrections

The shared node parser now reads anti_aliasing and the native canonical RGB_INT8
and L_INT8 names, alongside the previously supported R8G8B8/L8 aliases. Parsed
intrinsic/projection presence follows actual XML children. Omitted/empty camera
names use native __default__; disabled saving leaves the native path value empty.
The path node is validated even when saving is disabled, so unknown attributes
or nested payload still reject through standalone and full-document loading.
These correct observed native-loading differences without changing old assertions
or public constructor values. Full-document parsing keeps its existing generic
numeric policy; the new base API applies its declared schema/output bounds.

A present lens without cutoff uses the exact schema literal 1.5707, while an
omitted lens retains the constructor's PI/2. Raw native controls cover both.
SCHEMA_LENS_CUTOFF has a local approx_constant lint allowance because substituting
PI/2 would change observed values. The base emitter preserves both observed defaults
explicitly and rejects other cutoff values. No math approximation is introduced.

## Evidence

Thirty-four actual native cases contain 26 accepted states and eight rejected
schema states. They cover both pixel aliases, signed-int/range edges, zero size,
inverted clipping, name/save semantics, all three camera-noise types, raw mixed
poses, escaped strings and calibration-presence combinations. All 68 retained
fields are projected, including advanced defaults; non-pose fields compare exactly.
Pose components retain direct-sign <=1e-3 equality. Native-generated fixture bytes,
C++ source/library pins and regeneration command are in reference/camera-base-xml.json.

```
python3 tools/camera_base_xml_reference.py /path/to/pinned/sdformat
cargo test --locked --test camera_base_xml --test camera_base_xml_original
```

The helper regenerates the fixture byte for byte, loads native XML in Rust and
loads 26 actual Rust outputs in native Camera.Load. It uses low-level SDF read
without a model graph and reruns unchanged DOMCamera.ToElement with empty stderr.
The partial original port keeps all supported value/index-free assertions and
edited near-clip output, with original positive finite double comparisons within
four ULPs. Generic Element/null identity remains unported; source=None is only
supplemental owned provenance. No original assertion is retired.

All 441 Rust tests and Clippy pass. Independent source/parser/assertion review
passed 22 focused repository tests plus both private regressions, all 34 native
outcomes and 26 actual exchanges. It rechecked canonical omitted zeros, emitted
signed zeros and path validation through both flags/loader routes. Normal strict
staged coverage and exact hosted CI remain required before acceptance.
