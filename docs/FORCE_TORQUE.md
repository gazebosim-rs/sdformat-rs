# Force/torque and joint sensor metadata

`Joint.sensors` owns sensor envelopes in declaration order. Names are unique per
owner, so the same sensor name can occur on different joints or links. The
supported configuration enum selects IMU, Contact or boxed ForceTorque settings;
`Sensor::force_torque()` borrows the latter without creating a dummy configuration.
Joint and Sensor clones retain independent owned settings, source identifiers and
original configuration XML. No simulator, transport, filesystem or Fuel dependency
is introduced.

ForceTorque retains the measurement frame (parent, child or sensor), direction
(parent to child or child to parent), and three force and three torque Noise
records. Missing configurations use child / child to parent and no-noise defaults.
An explicit configuration has source provenance and its exact original subtree in
Sensor.configuration_xml; effective default insertion does not rewrite that XML.
Ordinary text collapse, CDATA, numeric references and native string trimming use
the existing parameter adapter. Empty optional frame/direction fields use defaults;
unknown or incorrectly cased names reject. Duplicate frame elements retain the
first native value. Other schema singleton duplicates remain explicit errors.

A sensor under a joint defaults its pose to that joint, not the child link. Its
explicit relative_to uses the owning model's frame scope. The measurement frame
selects the orientation of wrench components and does not change the pose's
reference frame; the wrench's point remains the joint origin. Native Root permits
joint IMU/Contact envelopes and link ForceTorque envelopes too. Parsing preserves
those configurations without claiming that a particular runtime supports them.

Unknown non-frame references remain deferred: load_str can succeed while
resolve_pose or validate_non_frame_poses returns UnknownReference. The explicit
validator visits joint sensors as well as link non-frame poses, including nested
and world-contained models. It validates the loaded graph, not arbitrary later
DOM mutations. Normal and merged resource tests preserve actual native model and
joint pose semantics, original XML and canonical imported source identifiers.

Consumers adopting this additive enum variant and Joint field must explicitly
match or reject unsupported sensor kinds and nonempty joint.sensors. The current
simulator pin predates Contact/ForceTorque and does not provide those runtimes.
This metadata does not measure wrenches, execute noise, schedule samples, publish
messages or apply forces. No renderer, mutable Element facade, serializer or
force/torque runtime is added.

The full original DOMJoint.Sensors case is ported from pinned
`test/integration/joint_dom.cc`: root/model/joint and sensor presence, names,
count/type, raw pose, both semantic resolutions, frame/direction and all twelve
Mean/StdDev observations. Native Pose3 equality is component-wise <= 1e-3 with
no quaternion sign substitution; EXPECT_DOUBLE_EQ uses the original four-ULP
bound. The fixture is byte-identical. Independent assertion review approved the complete mapping; mandatory
staged/hosted gates remain required.

The 96-row native corpus covers nine supported source versions, explicit and
omitted configuration/axis groups, six full Noise records, text/reference/control
edges, invalid enums, owner types/order/duplicates, deferred references, and normal
and merged imports. Eight named rows retain existing strict-schema/finite-input
bounds: unknown axis/attribute/namespaced frame, mismatched IMU/Contact groups,
duplicate selected configuration/direction, and explicit infinite update rate.
Those native-accepted inputs remain Rust errors and are not parity passes.
Negative finite ForceTorque update rates are retained like Contact metadata;
the existing negative IMU-rate rejection remains a separately documented policy.
Native diagnostic counts/text/partial DOM on failure and invalid-input callback
ordering are not claimed to match. General prefix/nonfinite scalar limits remain
as documented in SCALAR_CONVERSION.md.

Reproduce from the repository root with the pinned SDK and native build:

```sh
python3 tools/force_torque_reference.py ../sdformat
cargo test --locked --test force_torque
```

The runner freshly compiles the probe, checks every recorded token, binds installed
headers and linked libraries to pinned source/build identities, verifies four actual
embedded schemas and immutable fixture bytes, and reruns unchanged DOMJoint.Sensors,
DOMLink.Sensors and DOMNoise.Load with one executed test and no failures or skips
in each. These are qualified existing native baseline builds, not a fresh complete
upstream build or proof of all ForceTorque/Joint/Noise behavior.
