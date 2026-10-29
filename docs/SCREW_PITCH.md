<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Screw-joint pitch units

`Joint.screw_thread_pitch` stores metres per revolution. Positive values denote
right-handed screws. XML `<screw_thread_pitch>` takes precedence over the legacy
`<thread_pitch>` declaration; absent pitch defaults to one metre per revolution.
These load rules were already implemented. `Joint::legacy_thread_pitch()` now
exposes the corresponding Gazebo-classic value: radians per metre, positive for
left-handed screws, computed as `-TAU / screw_thread_pitch`.

The conversion uses ordinary f64 division, matching native Joint::ThreadPitch.
Zero and signed zero produce signed infinities; extreme finite pitch may overflow
the reciprocal. This is metadata and unit conversion, not a new screw constraint
or a physics-admission policy. Clones retain independent owned configuration;
the classic observation always reflects the current authoritative pitch field.

The unchanged five-joint `joint_screw_thread_pitch.sdf` fixture ports every
DOMJoint.ScrewThreadPitch predicate: load/model presence, both link names, all
five joint names, counts, both unit systems, defaults and modern-field precedence.
The original four-ULP double comparisons and 1e-3 near bounds are retained.
Eighteen supplementary native cases check exact binary64 bits for signed values,
zero, signed zero, empty declarations and extreme finite input.

```
python3 tools/screw_pitch_reference.py /path/to/pinned/sdformat
cargo test --locked --test screw_pitch --test screw_pitch_original
```

The native helper checks the fixture and SDK header and records its compile
command and loaded-library hashes in a small ignored report. It qualifies the
existing native build on Linux GNU, not a new reproducible reference image.
Independent source review, three Rust tests and all native observations passed.
Strict staged verification and exact hosted CI remain required before acceptance. General joint construction, mutable Element, serialization,
screw physics and full SDF-02 remain outside this slice.
