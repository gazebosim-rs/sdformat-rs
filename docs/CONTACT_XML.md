<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Contact Sensor XML

`ContactSensor::to_xml` and `load_contact_sensor_str` preserve the ordered collision
string list, including duplicates, and the configuration topic. The standalone
reader reuses existing contact-node parsing, defaults and source locations.
This `<contact>` grammar is distinct from physical Surface contact settings.
No collision-name lookup, contact computation or topic fallback is performed.

Canonical contact Sensor output delegates the same configuration writer and
retains current envelope/pose metadata. Plugins, extensions, gz_frame_id and
mismatched families reject. Joint XML remains force_torque-only. Output follows
typed values without changing source provenance or using stale input XML.

## Native text boundary

Normal native XML text and numeric space entities collapse double spaces; single
CDATA preserves them while trimming ends. The writer uses one safe CDATA section
for retained double-space values and ordinary escaped text otherwise. Native
Param only reads the first chunk of split CDATA, so values containing both double
spaces and ]]> return Unsupported instead of losing data. Empty collision lists,
empty/padded strings, controls and invalid XML characters return InvalidValue.
These explicit bounds preserve existing valid CDATA values such as `arm  foot`.

Existing parser behavior, including first-chunk handling of split input CDATA,
remains unchanged. Defaults stay `[__default__]` and literal `__default_topic__`;
consumer fallback and deferred collision references remain separate policies.

## Evidence and scope

Sixteen actual native inputs cover defaults, order/duplicates, normal versus
CDATA spaces, escaped/unicode/delimiter text, deferred references, sensor flags,
pose/frame metadata and typed edits. Seventeen actual Rust outputs are loaded
and inspected in native Sensor/Element, including the authoritative edited value.
String values compare exactly; raw pose components retain direct signs and
<=1e-3 bounds. Provenance and the text/producer observation are recorded in
reference/contact-xml.json.

```
python3 tools/contact_xml_reference.py /path/to/pinned/sdformat
cargo test --locked --test contact_xml --test contact_sensor
```

Native Sensor.ToElement does not implement contact production: the recorded probe
returns a nonnull Element and one ELEMENT_INVALID (code 9). This is a missing
native producer, not a Rust output requirement or a native writer-parity claim.
There is no standalone native Contact Sensor DOM writer. Source audit found
contact only in the Sensor Type loop; retained Sensor ToElement originals use
Magnetometer/plugins. Physical Surface Contact is unrelated. No retained native
failure assertion is changed and no extra upstream-original credit is claimed.
The unchanged Contact corpus/DOMLink.Sensors fixture and assertions remain gates.

Author full suite passes all 476 Rust tests, formatting and Clippy. Existing
Contact85/native Sensors load proof replays unchanged. Ten prior helpers replay
and all 172 prior XML exports remain byte-identical in author checks. Independent final review passed eleven Rust checks in fresh
source/target, sixteen native inputs, seventeen actual outputs, exact native
producer observation and unchanged DOMLink.Sensors. It verified text/error
boundaries, preserved fields/diagnostics and source audit; prior byte comparisons
remain author evidence. Normal strict staged/hosted qualification remains pending.
Accepted ledger stays 184 complete /28 partial.
