<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Legacy Sensor output

Sensor::to_xml admits gps, ray and gpu_ray through the existing NavSat/Lidar
writers. The output retains the declared raw type attribute and emits canonical
navsat/lidar configuration. Sensor::kind and effective_type remain distinct:
native TypeStr classifies these aliases as navsat, lidar and gpu_lidar. Parsing,
public values and input provenance do not change. Native producers may canonicalize
the attribute; producer byte parity is not claimed.

All prior configuration checks, including advanced Lidar Noise limits, remain.
Legacy standalone configuration roots, mismatched families, extensions/plugins,
gz_frame_id and Joint non-force-torque payloads still reject. This is metadata
serialization, with no navigation measurement, ray generation or physics claim.

## Exact capability transitions

The existing NavSat and Lidar tests rejected output for three exact bare Sensor
inputs: gps named legacy, ray named s and gpu_ray named s. These were temporary
Rust-only writer limits, not native assertions. Before editing them, independent
review verified the unchanged inputs against native Load/Element, including raw
attribute and effective classification, every 50-field NavSat or 38-field Lidar
projection and direct component pose bounds. Only those Unsupported predicates
were replaced with complete positive output/reload checks. Existing kind and all
unrelated rejection assertions remain. No additional original-case credit is given.

## Evidence

Author formatting, Clippy and all 479 Rust tests pass. Sixteen actual native
inputs and twenty-two actual Rust output exchanges cover raw/canonical defaults,
legacy/canonical configuration payloads, complete envelopes and typed edits.
The unchanged DOMSensor.Type original also passes. Pose components use the
retained direct <=1e-3 bound; other projected fields compare exactly.

Independent narrow transition review passed four prepared tests and that native
replay before predicate edits. The redundant uncommitted prepared test was folded
into the existing tests. Final independent review passed fourteen Rust checks in fresh source/target,
all sixteen native inputs, twenty-two outputs and unchanged DOMSensor.Type.
Only normal strict staged/hosted qualification remains pending; the accepted ledger remains 184 complete and 28 partial.

Author regression replay passes all eleven prior family helpers. All 189 prior
actual XML exports remain byte-identical; this comparison is author evidence.
