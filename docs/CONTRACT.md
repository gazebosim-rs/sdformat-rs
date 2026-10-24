# API contract 0.1 (provisional)

`load_str(xml, source_uri, &dyn ResourceResolver) -> Result<Sdf, Diagnostic>`
returns a complete document or the first source-located error. Syntax, typed
validation, include resolution, and semantic graph construction are distinct
implementation stages. The document owns all strings and values; no XML buffer,
resolver or native library lifetime escapes loading. The parser has no mutable
global state and never installs an executor. Models are not instantiated here.

`ResourceResolver::resolve(uri, parent_source_uri)` returns XML and its canonical
URI. The canonical URI detects include cycles and supplies imported source
provenance; implementers must return stable identities for equivalent resources.
Nested includes pass the imported parent's URI. Name, pose and static overrides
apply at the include site. Maximum include and model depth is 64; no IO is implicit.
Directory/model.config selection and URI/cache policy belong to the resolver.

`World` has name, gravity in m/s², models, frames, plugins and source.
`Model` has raw pose, static flag, canonical-link reference, links, joints,
nested models, frames, plugins, scoped name and source.
`Link` has pose, inertial, collisions, sensors, plugins, scoped name and source.
`Inertial` has mass in kg, six inertia components (ixx, iyy, izz, ixy, ixz, iyz)
in kg·m² and inertial-frame pose. Positive physical inertia is required.
`Collision` has pose and box/sphere/cylinder/plane geometry, with SI dimensions.
`Joint` has kind, parent/child references, pose, optional axis and source.
`Axis` has normalized xyz, expressed_in, position limits, effort and velocity.
Negative effort/velocity sentinels become positive infinity. Axis resolution
rotates into the requested frame without translation. Joint endpoint frames may
be aliases, but must resolve to different bodies; the child cannot be world.
`Sensor` currently supports only an IMU envelope: pose, rate in Hz, always_on,
topic, and original IMU-specific XML. Typed/schema-validated IMU settings remain
pending; this is not a fully validated sensor contract yet.
`Plugin` retains exact original XML including extension attributes/content.

All transforms use f64 metres and unit quaternions [x,y,z,w]. Raw poses retain
relative_to. Euler poses are fixed-axis roll/pitch/yaw (Rz*Ry*Rx); degree and
xyzw quaternion input are supported. Parsing normalizes nonzero quaternions.
Publicly constructed transforms passed to resolve_pose are checked for finite
values and unit norm (squared-norm tolerance 1e-12). Arithmetic overflow is an
error. Resolved transforms do not silently reinterpret body or rendering axes.

`Sdf::pose_of(name,target)` takes fully scoped semantic frame names. The global
root is `::`; a world `default`, model `robot`, link `base` is named
`default::robot::base`. `resolve_name(scope,reference)` applies SDF's world and
__model__ references. `resolve_pose` resolves a non-frame collision, sensor or
inertial pose using its model scope and owning link. `attached_body` follows the
separate attachment graph to a link or static world attachment. Names, unknown
references, duplicate semantic frame names and both kinds of cycles are checked.

Versions 1.8–1.12 share empty upstream conversion scripts in the supported subset.
Older unflatten/rename conversions are unsupported. Full schema/version-aware
validation, serialization, visual/material data, mesh/resources, merge and
placement frames, automatic inertia, URDF, other sensor families and most other
SDF features remain pending. Unknown schema features error; opaque plugin XML
and currently opaque IMU-specific XML are the stated exceptions.
