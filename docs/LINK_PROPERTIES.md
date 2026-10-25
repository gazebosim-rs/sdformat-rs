# Owned link properties

Link.properties retains four native boolean values: enable_wind=false,
enable_gravity=true, kinematic=false and self_collide=false by default. The first
three match native Link DOM getters; self_collide is the effective XML parameter
because native Link has no corresponding typed getter. LinkProperties is an owned,
Copy/Clone value. It does not select collision groups or move/create physics bodies.
Existing model/environment flags remain outside this slice.

The parser preserves explicit nondefaults, native empty/default/boolean spelling
and the existing first-text/CDATA rules. Fifty-six native rows cover all four
fields, malformed values, whitespace boundaries and core versions 1.4–1.12.
A simulator consumer must implement or reject unsupported physical flags before
creating bodies. Loading this data does not claim wind forces, per-link gravity,
kinematic control, self-collision or an articulated pendulum simulation.

The unchanged double_pendulum.sdf fixture now loads with its materials, axis
dynamics, wind and kinematic declarations retained. Proposed original ports cover
all assertions in DOMLink.InertialDoublePendulum and DOMJoint.DoublePendulum:
link/model/joint/axis lookups, raw poses, relative strings, parent/child names,
flags, mass/matrix/pose values and repeated base fluid-absence observations. Native
Pose/Vector3 equality keeps component-wise 1e-3 bounds without quaternion sign
relaxation. EXPECT_DOUBLE_EQ uses its four-ULP relation, including signed zero.
The original literal 1.5708 is preserved with a local test-only approximate-constant
lint allowance, rather than substituting a different numerical input.

For this known-valid fixture only, the original base MassMatrix::IsValid true
observation maps to successful mandatory typed inertia validation and its exact
mass/matrix components, as in the earlier reviewed InertialComplete port. There is
no new generic validity algorithm/API claim. This assertion mapping requires the
independent reviewer's explicit approval before the case becomes a complete port.
Both original files' 2018 notices and fixture bytes are retained unchanged.

Joint::axis(index) borrows the first axis at zero; every positive index aliases
the stored second axis, matching native clamping without narrowing usize. The port
exercises the native index lookup path directly; additional tests verify returned
references are the stored objects for both axes, including indices 2 and usize::MAX. Setter and
mutable Element/serialization facades remain pending. Native references bind
source/schema/header/math comparison headers, libraries, fixture and both original
binaries. Strict staged/hosted gates and independent assertion review remain
required; no whole SDF packet is self-accepted.
