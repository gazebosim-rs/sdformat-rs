# Contact sensor configuration

`Sensor.config` selects `SensorConfig::Imu(Box<Imu>)` or
`SensorConfig::Contact(ContactSensor)`. The optional `imu()` and `contact()`
accessors return borrowed settings of the selected type. Contact sensors carry
no dummy IMU. Sensor cloning owns the configuration, source and exact selected
`configuration_xml`; no XML-buffer or resolver lifetime escapes loading.
Existing IMU assertions use `imu().unwrap()` after this additive-family API
change, retaining every original expected value, comparison and error case.

The pinned native SDF DOM has no typed Contact class: `Sensor::Load` sets the
CONTACT enum and retains its Element. Native gz-sim's Contact system reads
`contact/collision` and `contact/topic` from that Element. The producer exposes
those effective configuration values without implementing the contact system.
An omitted or empty contact group has one `__default__` collision and topic
`__default_topic__`. Ordered repeated collision names, including duplicates, are
preserved. References remain unresolved; missing collisions are a consumer
concern. `source` is None only for an omitted group. Raw XML remains empty for
an omitted group and otherwise retains the exact original child bytes.

The envelope `Sensor.topic` and nested `ContactSensor.topic` are distinct.
The parser does not apply the contact system's scoped-name topic fallback.
Ordinary text uses the existing native first-text/whitespace/reference handling;
CDATA retains its distinct internal whitespace. Explicit empty collision/topic
values reject. Other sensor families, mixed or unknown configuration children,
unknown attributes, duplicate singleton groups/topics and namespace extensions
stay explicit errors in the supported schema subset. Native SDF may retain or
ignore some such fields; those measured gaps are named individually in the
corpus, rather than silently broadening extension handling.

Negative finite contact update rates remain native metadata: the native contact
system warns and treats them as unthrottled. This producer runs no scheduler.
The prior negative IMU-rate rejection remains unchanged and is a separately
qualified policy difference from native DOM loading. Nonfinite input-rate gaps
also remain explicit. No contact publication, physics query, sensor scheduling,
consumer collision lookup, setter/Element API, serialization, renderer or IO
behavior is supplied here. A simulator consumer must match Contact explicitly
and reject it until a runtime implementation exists.

`tests/contact_sensor.json` binds 85 actual native Root observations: 74 rows
match the supported load/value/error subset, and 11 named native-accepted rows
assert explicit Rust policy/schema errors. `tools/contact_sensor_reference.py`
recompiles the native helper, verifies all outputs and the complete unchanged
DOMLink.Sensors original, and binds SDF/math/consumer sources, installed headers,
linked libraries and fixture bytes. The Rust original contact subtree port
retains the contact name/type/presence, raw/resolved poses and metrics assertions
from link_dom.cc, with native component-wise 1e-3 pose comparisons. The earlier
IMU subset remains intact. Other 25 sensor families and whole model/link
assertions remain pending; this is a partial mapping, not a retired full case.

Self-review found and fixed premature IMU configuration evaluation in the new
enum path. Four exact diagnostic controls from a fresh public-parent consumer
preserve envelope name/pose/boolean precedence, including code, source position
and message. The new test failed before correction and passes after restoring
the original evaluation order. The earlier 81 native observations are unchanged;
four additional invalid-envelope controls now also bind their native outcomes.
