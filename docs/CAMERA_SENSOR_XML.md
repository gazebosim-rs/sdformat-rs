<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Camera Sensor XML

Canonical camera Sensor output delegates the accepted bounded Camera writer.
Both sensor and camera poses/frames, envelope flags/rate/topics and every admitted
camera field are preserved. Existing advanced optics, signed-zero, numeric and
name guards remain in force. Other camera-family kinds, plugins, extensions,
gz_frame_id and Joint admission stay unsupported. No images or runtime are added.
Unnamed constructed Camera values remain rejected by the existing lossless-name
guard; no default name or calibration flags are silently changed.

The generic Sensor reader deliberately keeps its existing numeric metadata
admission. Eight native Camera schema-rejected inputs (large signed-int image/
anti-aliasing values, HFOV/clip bounds and enabled saving without a path) still
parse as Rust metadata, then return InvalidValue from the existing bounded Camera
output validator. These exact read/output expectations are recorded separately
from native REJECT outcomes; this change does not tighten document parsing.

## Evidence

Thirty-eight actual native inputs cover all 34 existing base-camera cases,
envelope flags, escaped strings, two independent poses, typed edits and the exact
full-document capability-transition input. Thirty-two actual Rust outputs load
in native Sensor: thirty admitted input rows plus authoritative typed-edit and
full-document exports. All 84 fields are projected; non-pose fields compare
exactly and both pose streams keep direct component <=1e-3 bounds. The unchanged
native Camera.ToElement original passes with empty stderr. Original fixture
generation remains in tools/camera_base_xml_reference.py.

```
python3 tools/camera_sensor_xml_reference.py /path/to/pinned/sdformat
cargo test --locked --test camera_sensor_xml --test camera_base_xml_original
```

The standalone Camera and Sensor tests share the same unchanged camera field
projection/bounds helper. Every genuine upstream Camera assertion/fixture is
preserved. No additional upstream-original credit is claimed; Sensor.ToElement
still has distinct plugin/input-origin residue.

## Reviewed capability transition

The old Rust-only test
constructor_values_and_full_document_presence_rules_are_preserved expected a
matching Camera Sensor.to_xml() call to return Unsupported. The coordinator
authorized coherent default admission, and independent review approved this
narrow capability transition before the predicate changed. Its exact full SDF
input and every constructor/name/presence check remain. The replacement compares
all 84 native fields, requires nonempty successful output and checks standalone/
native reload. No opt-in API or source/input special case was introduced.
The explicit disposition and old predicate are in reference/camera-sensor-xml.json.
All genuine upstream Camera assertions and unrelated advanced/plugin/Joint
rejections remain intact; no extra original credit is claimed.

Author checks pass all 472 Rust tests, formatting and Clippy. Native Sensor proof
passes 38 inputs/32 actual exports and unchanged original. The base-camera helper
also replays all 34 cases/26 exchanges, exact fixture and unchanged original.
All eight earlier family helpers replay, with 114 prior exports byte-identical;
all 26 prior base-camera exports are also byte-identical (author evidence).
Final independent review passed 23 Rust checks in fresh source/target, all 38
native inputs, 32 actual outputs and unchanged original with empty stderr. It
verified unchanged projection/bounds bodies, both direct pose bounds, delegated
Camera guards, exact temporary-boundary transition and separate read/output
policy. Prior byte comparisons remain author evidence. Normal strict staged/
hosted qualification remains pending. Accepted ledger stays 184 complete /28 partial.
