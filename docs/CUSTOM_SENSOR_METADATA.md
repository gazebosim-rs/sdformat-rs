<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Custom sensor metadata

The unchanged simulator `test/worlds/environmental_sensor.sdf.in` at
`71c60db31fddd5c98d2aa689e9a2cccb3548a0a3` has three `type="custom"`
sensors with unbound `gz:type` attributes. Native sdformat at
`1249805d9fceeff94ca1fe63e26ab9c051cea24e` loads them, preserving humidity,
2D wind and 3D wind subtypes and their direct configuration children. Native
prints unknown-child-copy warnings. The existing QuickXML lexical cursor
already accepts the unbound prefix; Python's namespace-aware audit rejection
was unrelated to the Rust tokenizer.

Before this change, SDF Main `06c23a3` rejected the complete fixture at
`gz:type`. The simulator still consumes producer `575d4d9`, which rejects the
complete world earlier at its unsupported light; an unchanged exact humidity
sensor fragment also rejects `gz:type`. This metadata change does not update
that pin. A future consumer must explicitly support or reject all admitted
custom data before adopting it.

`CustomSensor.custom_type` preserves absent versus explicitly empty `gz:type`
and its native string value, including native space/tab/LF edge trimming after
XML decoding. No subtype spelling or field-name whitelist is imposed.
`CustomSensor.xml` owns the exact original sensor span, retaining attribute
spelling, whitespace and nested XML without namespace injection or prefix
rewriting. `Sensor.extensions` retains every opaque direct child, including
unqualified names, duplicate names, unknown attributes, nested children,
comments and CDATA, in input order with source URI/line/column. This is original
XML, not native converted or serialized byte output.

The existing first vendor-child `element_name`, direct `parameters` accessor
and `configuration_xml` are preserved. That first child also appears in the
complete opaque-child sequence. Known envelope values still populate their
typed fields. Existing known-family mismatch checks, duplicate typed singleton
errors, malformed XML checks and unsupported attributes other than the
explicitly admitted `gz:type` remain. Native accepts a custom sensor's `<imu>`
child, but Rust retains its existing Unsupported boundary. Other sensor
families do not gain `gz:type` or generic child admission. Custom output through
`Sensor::to_xml` remains Unsupported; no metadata is silently discarded by an
emitter.

This is metadata only: no environmental sampling, preload CSV handling,
transport topics, simulator systems, rendering or full native environmental
CanPreload original is ported or credited.

The checked native corpus includes the complete fixture and ten boundary
observations: absent/empty/whitespace/decoded/arbitrary subtype, nested and
repeated arbitrary children with and without subtype, known payload strict
difference and malformed XML. `tools/custom_sensor_reference.py <upstream>`
recompiles the checked helper and compares complete stdout/stderr, exit status,
fixture/source/header/library hashes. The native helper uses the same explicit
library environment for execution and linkage audit. Rust matrix controls
compare full sensor envelopes and subtype values, while separate checks retain
exact raw spans, ordered extensions and source positions. Existing full Link
sensor and custom-fragment original assertions are unchanged.

Before the additive ownership/proof tests, author checks passed all 585 Rust
checks, including seven new controls;
all-target Clippy and all 36 tooling checks passed.
Dynamic field-name membership uses `BTreeSet` in validation and extension
selection; metadata output still follows source order and preserves duplicates.
The 4,096-distinct-field control checks every ordered value and source position,
plus 32 interleaved repeated fields, without a wall-clock threshold. Full raw
sensor XML and child metadata intentionally duplicate bytes; this slice makes no
memory-improvement claim.

Independent coordinator review found and verified the field-membership fix.
Fresh independent source/target replay passed all seven new Rust controls and
eleven native observations, with no remaining findings. An additional public
Default/Clone/Debug ownership test passed author focused replay and independent
source review. The first staged hook failed closed on uninstrumented declaration
and delimiter syntax; the separately reviewed proof extension passed fresh
independent 16 helper/39 tooling tests, preserving actual DA and region authority.
The full combined protected normal staged gate and exact hosted CI remain required. No original-test ledger changes or runtime/performance claim is made.
