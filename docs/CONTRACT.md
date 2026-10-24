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
`Sensor` supports IMU pose, rate in Hz, always_on, enable_metrics, topic, typed
`Imu` settings and original IMU XML. The six per-axis `Noise` records retain type,
mean, standard deviation, bias mean/deviation, dynamic bias deviation/correlation
time, and precision. Standard deviations are nonnegative; noise types are case
insensitive. IMU orientation enable, localization, custom RPY and gravity direction
with their parent-frame strings preserve native defaults. Localization and parent
frame strings remain unconstrained at load time, matching the native DOM; their
runtime interpretation belongs to the sensor implementation. No noise is sampled
by this parser.
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
SDF features remain pending. Unknown schema features error; opaque plugin XML is the stated exception.
The XML library currently requires namespace prefixes to be declared, whereas
upstream permits undeclared prefixes in extension element names; that compatibility
gap remains open.

XML references are local to the enclosing world or model. Within a model,
__model__ names its implicit frame; world is only special for joint parent.
Within a world, world names the world frame. The parser distinguishes these local
references from its fully scoped internal default edges. A top-level SDF may
contain one model or worlds, and root-model pose relative_to must be empty.
Public resolve_name accepts a fully scoped/default query or a world endpoint;
it does not expand the allowed namespace of XML pose/axis references.
