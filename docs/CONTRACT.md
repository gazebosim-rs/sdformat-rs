# API contract 0.1 (provisional)

`load_str(xml, source_uri, &dyn ResourceResolver) -> Result<Sdf, Diagnostic>`
returns a complete document or the first source-located error. Syntax, typed
validation, include resolution, and semantic graph construction are distinct
implementation stages. Frame graphs, link/frame pose references, joint endpoints
and joint axes are checked at load time. Collision, visual and sensor non-frame
pose references preserve native deferred errors: load_str can succeed and a later
resolve_pose can fail. `Sdf::validate_non_frame_poses()` explicitly resolves all
collision/visual/sensor/inertial poses into the global frame, recursively over
standalone and world models, returning the first error. Scene builders should
call it before creating runtime objects to keep validation atomic. This validates
poses against the loaded frame graph; it does not rebuild that graph after public
DOM edits. The document owns all strings and values; no XML buffer,
resolver or native library lifetime escapes loading. The parser has no mutable
global state and never installs an executor. Models are not instantiated here.

`ResourceResolver::resolve(uri, parent_source_uri)` returns XML and its canonical
URI. The canonical URI detects include cycles and supplies imported source
provenance; implementers must return stable identities for equivalent resources.
Nested includes pass the imported parent's URI. Name, pose and static overrides
apply at the include site. Maximum include and model depth is 64; no IO is implicit.
Directory/model.config selection and URI/cache policy belong to the resolver.

`World` has name, gravity in m/s², owned atmosphere metadata, models, frames, joints, plugins, merged-include provenance and source.
`Model` has raw pose, static flag, canonical-link and placement-frame references, links, joints,
nested models, frames, plugins, scoped name and source.
`Link` has pose, inertial, collisions, visuals, sensors, plugins, scoped name and source.
`Inertial` has mass in kg, six inertia components (ixx, iyy, izz, ixy, ixz, iyz)
in kg·m² and inertial-frame pose. Positive physical inertia is required.
`Collision` has pose and box/sphere/cylinder/plane geometry, with SI dimensions.
`Visual` has name, pose, optional geometry, plugins and source. Missing
or empty geometry preserves native EMPTY as None. Visual names are unique within
a link, independently of collision and sensor names. Owned visual properties and optional material data preserve the metadata described
in docs/MATERIALS.md. PBR declarations are retained; texture IO and renderer effects remain outside
this loader.
`Joint` has kind, parent/child references, pose, optional axis/axis2 and source.
Ball, gearbox and screw are typed kinds. Screw pitch is stored in metres per
revolution; legacy `<thread_pitch>` converts to that unit, and modern
`<screw_thread_pitch>` takes precedence. `Joint::legacy_thread_pitch()` exposes
classic radians/metre with positive left-handed convention; see
[SCREW_PITCH.md](SCREW_PITCH.md). Known joint physics XML is retained
verbatim as `physics_xml`; its engine interpretation and nested schema
validation are outside this loader. Native warning output for a legacy
`<initial_position>` axis child is not reproduced.
Joint, JointKind and Axis implement Clone so consumers can retain owned typed
configuration after dropping the document; clones share no mutable strings.
Universal and revolute2 types retain both axes; native axis2 is also preserved
when present on another supported joint kind.
Five pinned `DOMJoint` copy and move originals retain the final name and both
axis XYZ values through owned Rust move, `Clone`, `clone_from` and replacement.
Native C++ operator identity, moved-from state and mutable Element pointers
are outside those value observations.
`Axis` has normalized xyz, expressed_in, position limits, effort, velocity and
optional owned mimic metadata. In SDF 1.11–1.12, `<mimic>` retains the leader
joint and axis plus multiplier, offset and reference; an omitted axis defaults
to `axis`. Loading checks the leader joint and axis within the model and
rejects a self mimic. `load_str` returns the first native mimic diagnostic for
the three reviewed invalid fixtures. Native multi-error vectors, warning side
effects for unknown mimic content, constraint execution, URDF import,
`ToElement` and general graph mutation remain unported.
Negative effort/velocity sentinels become positive infinity. Axis resolution
rotates into the requested frame without translation. Joint endpoint frames may
be aliases, but must resolve to different bodies; the child cannot be world.
`Sensor` has pose, rate in Hz, always_on, enable_metrics, topic and selected
`SensorConfig::Imu(Box<Imu>)`, `SensorConfig::Contact(ContactSensor)`,
`SensorConfig::ForceTorque(Box<ForceTorque>)` or
`SensorConfig::Magnetometer(Box<Magnetometer>)` or
`SensorConfig::Altimeter(Box<Altimeter>)` or
`SensorConfig::AirPressure(Box<AirPressure>)` or
`SensorConfig::NavSat(Box<NavSat>)` or
`SensorConfig::AirSpeed(Box<AirSpeed>)` or
`SensorConfig::Lidar(Box<Lidar>)` or
`SensorConfig::Camera(Box<Camera>)` settings, or
`SensorConfig::Unconfigured` for seven native-known envelope-only types, or
`SensorConfig::Custom(CustomSensor)` for one owned vendor-extension subtree. Optional `imu()`,
`contact()`, `force_torque()`, `magnetometer()`, `altimeter()`, `air_pressure()` and
`navsat()`, `air_speed()`, `lidar()` and `camera()` accessors expose the selected type; exact selected
child XML and owned source are preserved. Contact collision names and nested topic
remain distinct from envelope topic; see docs/CONTACT_SENSOR.md. The literal
`Sensor.kind` remains the XML spelling. `Sensor::effective_type()` returns the
native `SensorType` classification for a parsed kind (`gps` becomes `NavSat`,
`gpu_ray` becomes `GpuLidar`, and `ray` becomes `Lidar`). The public enum also
maps the native canonical type strings, but it does not permit parsing otherwise
unsupported sensor configurations or replace the literal kind.

The pinned `DOMSensor.Type` value transitions now map through ordinary Rust
`SensorType` assignment and `set_from_type_str`: invalid strings leave the
previous value intact. This does not add native `Sensor::SetType` object-method
identity or synchronize a changed type with parsed literal XML, config or
Element state.

The parser also accepts `depth`/`depth_camera` and `rgbd`/`rgbd_camera`
spellings with an owned `Camera` configuration selected from `<camera>`;
`kind` retains the literal spelling and `effective_type()` gives
`DepthCamera` or `RgbdCamera`. This is a bounded fixture and value parser,
without depth image processing, full Camera grammar or rendering.
The `segmentation`/`segmentation_camera` and
`boundingbox`/`boundingbox_camera` spellings also select the owned Camera
values. Explicit `<segmentation_type>` and `<box_type>` populate their strings
and presence flags as in native `Camera::Load`; omitted elements retain
defaults with false presence. Native effective type remains separate from the
literal XML spelling. Segmentation and bounding box rendering are not owned.
`thermal`/`thermal_camera` and `wideanglecamera`/`wide_angle_camera` also
select the owned Camera parser. The wide angle fixture retains its FOV,
image/clip, lens function and coefficients, topic and update rate as values;
no image rendering, thermal sampling or general camera grammar is implied.
`logical_camera`, `multicamera`, `rfid`, `rfidtag`, `sonar`,
`wireless_receiver` and `wireless_transmitter` retain name, literal and
effective type, raw pose and metrics with `SensorConfig::Unconfigured`.
These have no owned family payload. A known mismatched configuration child
(such as `<imu>` under RFID) is rejected; generic extension child grammar,
family behavior and native Link count/index/name lookup remain unported.
`custom` retains one lexically named vendor child, its direct text children
and the exact selected raw subtree. `custom()` exposes those values; the
unchanged native fixture includes `my_namespace:custom_sensor` with
`some_param=123`. Arbitrary nested extension grammar, mutable Element
identity, generic child lookup and native Link name/index lookup are unported.
The six per-axis `Noise` records retain type,
mean, standard deviation, bias mean/deviation, dynamic bias deviation/correlation
time, and precision. Standard deviations are nonnegative; noise types are case
insensitive, after native space/tab/LF attribute-edge trimming. Explicit noise
declarations require a nonempty type attribute; omitted noise retains None
constructor defaults. See docs/NOISE_TYPES.md. Magnetometer owns three noise
axes and exact explicit configuration/source XML; omitted configuration has
all-None noise defaults. Altimeter owns vertical position and velocity noise
with the same source and omission behavior. AirPressure owns pressure noise and
reference altitude; omitted configuration defaults both. NavSat owns horizontal
and vertical position/velocity noise, accepts both `navsat` and legacy `gps`
type/child spellings, and retains the literal type and selected source XML.
When both child spellings occur, native `navsat` selection takes precedence.
Omitted configuration and noise groups retain constructor defaults. AirSpeed
owns differential pressure noise and optional explicit source XML, with an
all-default omitted configuration. Lidar retains horizontal and vertical scan
samples, resolution and angles in radians, range, noise and visibility mask.
Literal `ray`, `lidar`, `gpu_ray` and `gpu_lidar` types select the owned Lidar
configuration; a present `<lidar>` child takes precedence over `<ray>`, matching
native selection. Ray rendering and measurement belong to consumers. IMU orientation enable, localization, custom RPY and gravity direction
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
inertial or visual pose using its model scope and owning link. `attached_body` follows the
separate attachment graph to a link, world, or static model implicit-frame sink. Names, unknown
frame references, duplicate semantic frame names and both kinds of cycles are checked.
Non-frame reference failures are returned by resolve_pose or the explicit validator.

Versions 1.8–1.12 share empty upstream conversion scripts in the supported subset.
For 1.7 an empty inertial pose relative_to acts as the default link frame, matching
the upstream 1.7→1.8 remove_empty action. General XML conversion, serialized
attribute removal and round-trip conversion remain pending. Flattened names
that require unflatten conversion return explicit Unsupported.

The 1.6 pose-frame and joint-axis conversions are applied semantically: pose
frame overwrites relative_to (including an empty frame), and axis/axis2
use_parent_model_frame values true/True/TRUE/1 set expressed_in to __model__.
Other values are discarded by the conversion table. Ordinary XML text uses ASCII
whitespace handling (space/tab/LF/CR/VT/FF), while CDATA padding remains
significant. Reference-created empty first text nodes still block later CDATA;
leading literal whitespace is skipped, and child elements block first text.
Pose references
trim exactly space/tab/LF; numeric CR and NBSP remain part of the name. Literal
attribute CR/CRLF normalize before reference expansion. A true flag without xyz
reproduces the native missing-value rejection caused by its synthesized empty xyz.
Effective typed references are converted without rewriting source XML/positions.
Each included resource uses its own version. Original root Sdf.version is retained.
SDF 1.4–1.6 model members share a namespace in declaration order: later links, joints
and frames with cross-kind collisions append _link/_joint/_frame, then a numeric
suffix starting at 0 when needed. Nested models are never renamed. Worlds load
models before applying the same frame renaming, independent of XML order.
Same-kind raw duplicates reject, and reference strings are not rewritten.
Modern shared names reject. Flattened names remain Unsupported; 1.3 and older
conversions remain unavailable. Full schema/version-aware
validation, serialization, rendering effects, mesh resource loading, legacy merge collisions, automatic inertia, URDF, other sensor families and most other
SDF features remain pending. Unknown schema features error; opaque plugin XML is the stated exception.
XML names are lexical, matching upstream TinyXML: extension prefixes need not
be declared. Plugin XML retains original bytes/ranges, including references,
CDATA, whitespace and extension attributes. Standard/numeric references in typed
values are decoded with the XML library lexer; unknown references and bare
ampersands retain legacy literal behavior. DTD declarations remain explicitly
Unsupported, with no implicit external entity loading. Literal NUL is rejected. Numeric-control references follow native decoding; typed
attribute/text views truncate at decoded NUL, while raw plugin XML stays intact.
Only the first logical XML text/CDATA node supplies a typed scalar. Comments
(including legacy double-hyphen content) and leading whitespace are skipped.
Declarations/PIs are accepted in the initial prolog only. The native recursive
bound accepts498 simultaneously open nonempty XML elements; an empty child does
not consume that recursive depth.

XML references are local to the enclosing world or model. Within a model,
__model__ names its implicit frame; world is only special for joint parent.
Within a world, world names the world frame. The parser distinguishes these local
references from its fully scoped internal default edges. A top-level SDF may
contain one model, declared directly or supplied by a nonmerged include, or worlds.
A literal root model's pose relative_to must be empty. An include pose override
retains its raw relative_to text; the native standalone root graph instead uses
its raw pose and placement frame relative to the absolute root. This does not
expand local XML reference rules in other scopes. Mixed world/model roots remain
a known Rust subset rejection, although the native loader accepts them.
Public resolve_name accepts a fully scoped/default query or a world endpoint;
it does not expand the allowed namespace of XML pose/axis references.

A nonempty Model.placement_frame changes the meaning of its raw pose: it locates
that frame, not the implicit model frame. Resolved poses remove the local
model-to-placement offset, including previously resolved descendant placements.
Placement names can reference links, frames, joints, models or __model__ within
that model. Includes retain or override this setting; an include placement_frame
element requires a pose element, even when clearing the setting. The pose graph
is cycle-checked before placements and solved globally after local correction.

Same-frame pose queries return exact identity after checking that the frame
exists. Non-frame poses relative to their exact target return the declared
transform without a floating-point round trip. Other relative queries compose
only the paths to their nearest common ancestor, retaining local-coordinate
precision even when the shared world/model origin is large. Global/world results
remain f64 absolute coordinates and have the representational limits of f64.

Joint.axis=None preserves the native DOM when an axis element is omitted,
including revolute/prismatic/continuous joints. An explicitly present empty axis
has the schema default XYZ. Consumers must validate backend requirements before
constructing a joint; the parser does not invent a missing axis.

Native inertial loading is a distinct compatibility boundary: upstream Link.cc
loads but ignores inertial relative_to (its inertia-frame handling is marked TODO),
and Link::ResolveInertial composes the raw inertial pose with the link pose. Rust
retains the original reference and offers explicit resolve_pose semantics for it;
validate_non_frame_poses checks it as a consumer policy. This does not claim native
Link::ResolveInertial parity for nonempty inertial references.


Geometry metadata includes box, sphere, cylinder, plane, capsule, cone, ellipsoid
and mesh. New capsule/cone dimensions and ellipsoid radii use native fallback
values for nonpositive inputs. Mesh scale may be zero or negative. Mesh retains
URI, scale, submesh/centering, optimization enum, optional convex-decomposition
counts, and its original source position; loading performs no resource IO.
Unknown optimization strings retain native NONE. Decomposition metadata exists
independently of optimization mode. Missing mesh URI defaults to __default__;
an explicitly empty required URI or missing/default submesh name rejects.

Decomposition counts match Linux Param.cc stoul followed by u32 casting,
including numeric prefixes, initial0x base selection, boolean coercion,
negative wrapping and u64 overflow rejection. Scalar/pose prefix and range behavior is qualified on Linux GNU as described
below; general vector prefix and nonfinite parity remain pending. Scalar booleans use native ASCII case-insensitive
true/false and empty optional defaults. Required new primitive numeric fields
reject explicit empty values, while absent fields receive schema defaults.

Named entities reserve world and names of at least four bytes surrounded by
double underscores. Prefix-only names and the two/three-underscore names are
valid; include name overrides use the same rule. Scoped/flattened-name conversion
and other name lexical contracts retain their documented limitations.

Required scalar blank recognition and vector splitting use native C whitespace,
not Unicode whitespace. Mesh URI/submesh text collapses ordinary C whitespace;
CDATA retains internal whitespace before the native space/tab/LF string trim.
Boolean parsing likewise keeps ordinary and CDATA whitespace rules distinct.

Optional numeric fields distinguish XML text origin. Ordinary whitespace-only
references collapse to empty and use defaults. CDATA VT/FF do not collapse and
therefore fail numeric parsing; XML-normalized CDATA CR becomes LF and trims
empty. This applies to optional mass, mesh scale and decomposition counts.


Collision.surface retains native contact/friction metadata independently of any
physics backend. Surface, Contact, Friction and each engine-specific friction
group carry optional Source provenance: absent groups have None; parsed groups
retain URI/line/column. Surface.source distinguishes an absent surface from an
explicit empty surface even when numeric defaults agree. Consumers must reject
unsupported explicit surfaces rather than silently ignoring their parameters.

Contact masks are u16. Missing collide_bitmask uses the native DOM default255;
an explicitly empty element uses the schema default65535. An absent category mask
is None, meaning the collision mask is the effective category. Native unsigned
conversion and final16-bit wrapping are retained.

Friction always has ODE defaults; optional Bullet and torsional groups preserve
their presence. ODE mu/mu2, Bullet friction/friction2 and torsional coefficient/
radii follow nonnegative schema constraints. Slip and rolling friction retain
negative values where the schema permits them. fdir1 values remain unnormalized.
Unknown bounce/contact/compliance/engine properties remain Unsupported. No
friction/contact simulation, full surface schema or full Shapes parity is claimed.
General vector-prefix/nonfinite gaps remain documented separately.


Sensor.frame_id is canonical SDF frame metadata. Sensor.gz_frame_id preserves the
literal legacy element as Option<String>, including Some("") for an explicitly
empty element. Both values use native string whitespace/text-origin rules and
are retained together even when the canonical value is nonempty. These strings
are metadata, not semantic graph references. Only the exact unprefixed frame_id
and gz_frame_id leaf names are accepted; other extensions remain Unsupported.

The consumer chooses a nonempty canonical value first, then a present legacy
value even if empty, then its effective sensor name. The simulator may override
that name with the scoped sensor identity before applying fallback. This parser
does not choose a runtime name or apply consumer fallback. These common fields
apply to supported IMU, Contact, ForceTorque, Magnetometer, Altimeter,
AirPressure, NavSat, AirSpeed, Lidar and Camera variants. No
other sensor-family completion or full DOMSensor original-case retirement is
claimed.


Sensor.topic uses the same native string text-origin handling as frame metadata:
ordinary C whitespace collapses, CDATA internal whitespace and NBSP survive,
and typed NUL values truncate. Native Sensor::Load additionally maps the literal
__default__ sentinel to an empty topic. This is DOM metadata normalization;
runtime transport topic repair/qualification remains consumer-owned. Topic is a
supported scalar leaf; nested content and extension attributes are Unsupported.


Model.canonical_link_and_relative_name returns the native DOM-selected link and
its name relative to that model. Explicit canonical_link names are looked up as
links, without attachment-body alias expansion; a nonempty explicit frame name
can therefore return None with the original name. Otherwise the first direct
link wins; if no direct link exists, selection recurses into only the first
nested model and prefixes its name when the result name is nonempty. It does not
skip an empty first nested model or inspect static status. An empty subtree
returns (None, ""). Interface model/link branches and flattened names remain
outside the supported parser subset. This helper is independent of semantic
attachment resolution: a static model is its own `<model scope>::__model__` sink, while a dynamic model follows the recursive canonical
link when its name is nonempty, or the first nested model otherwise.


On Linux GNU/glibc, scalar floating-point fields and individual pose tokens use
an owned C locale and strtod_l to preserve native numeric prefixes, hexadecimal
forms, exact subnormals, invalid conversion and ERANGE. Scalar Param true/false
coercion remains separate; pose tokens do not coerce booleans. Vector streams
retain their distinct Rust path because native vector underflow behavior differs.
Pose empty/default recognition preserves native ordinary/CDATA text-origin rules.
Nonfinite semantic values remain rejected, an explicitly retained native DOM gap.
Other platforms return Unsupported for explicit scalar/pose-token conversion;
portable parity is not claimed. The boundary is safe to call concurrently, owns
and frees each locale, restores caller errno, and leaves global locale untouched.
Locale allocation failure is Resource; numeric errors are InvalidValue. See the
reviewed ABI/lifecycle notes and failure tests in SCALAR_CONVERSION.md. No parsing
performance improvement or unmeasured cached-locale optimization is claimed.


Model.include is optional owned provenance for models loaded through an include.
It stores the URI, optional name/pose/static/placement overrides, include-only
plugins, parent Source and exact original XML span. A direct model has None.
Model.source remains the resolved resource URI; nested resolver calls receive
that canonical parent identity. The boxed optional record keeps direct-model
storage small. Its values are snapshots; mutating expanded model plugins does
not rewrite the original include metadata.

Included plugins append after resource plugins, preserving duplicate names and
opaque XML. Plugin names are optional and default empty, as native plugin.sdf
specifies; names and required filenames use native attribute string trimming.
Resolver requests use native XML text: ordinary whitespace collapses, while
CDATA edges remain significant. Include.uri matches the parsed native string
value, which trims CDATA edges; the raw span remains available independently.
CR/LF and LF/CR pairs normalize in one left-to-right pass before reference
expansion, matching TinyXML rather than XML1.0-only EOL normalization.

An explicit empty include placement clears imported placement when pose is
present; empty static defaults false. These prior Rust contracts are retained
safe extensions: the pinned native include path throws on null XML text for
these two inputs. Empty CDATA forms are valid natively and produce the same
values. No exception byte/outcome parity is claimed for those extension rows.
Merge, old-version and unknown include/schema limits remain explicit errors.
The exact include span is source preservation, not a converted/expanded XML
serializer or ToElement implementation. No resource IO or consumer dependency
is added by this metadata API.

Resolver callback text/order equivalence is qualified for accepted include rows.
Rejected inputs can fail before lookup in this Result API; for example a
whitespace-only CDATA URI is rejected before calling the resolver, while the
pinned native loader can call its callback before rejecting it. Invalid-input
lookup side effects and diagnostic ordering are not claimed as native parity.

Basic Box/Sphere/Cylinder/Plane parsing applies the native DOM fallback for
nonpositive dimensions. Any nonpositive Box or Plane size component resets the
whole size to ones; Sphere radius resets to one, while Cylinder radius and length
reset independently to 0.5 and one. Omitted children retain schema defaults,
including Cylinder's XML radius of one; explicitly empty required values fail.
Plane normalization follows native gz-math: lengths at most 1e-6 leave the raw
normal unchanged, including zero; an overflowed length divides finite components
to signed zero. This is DOM data, not a physics capability or valid-normal claim.
Consumers must validate supported physical geometry before instantiation.

The SDF 1.5 common model subset uses the existing 1.6-to-current semantic
conversion chain, including pose frame, axis expressed-in and legacy shared-name
rules. It loads original 1.5 box resources through the injected resolver without
rewriting fixtures and retains the root Sdf.version and original resource source positions.
The changed 1.5-to-1.6 actions remain explicit boundaries: physics-contained world
gravity/magnetic-field moves and legacy IMU noise redistribution are Unsupported,
as are unimplemented schema fields at newer versions. No full 1.5 conversion or
serialization claim is made. Empty 1.5 roots now report Missing after version
acceptance; 1.3 roots remain Version errors.

Inertial.fluid_added_mass is optional boxed symmetric 6x6 data in native
[x,y,z,p,q,r] order about the link origin, independently of the inertial pose.
An absent element is None; an explicitly empty element is Some(zero matrix).
The 21 named upper-triangle entries default to zero and are mirrored. Finite
negative/indefinite values remain permitted native DOM metadata; no extra
positive-definiteness or hydrodynamic calculation is imposed. Physics consumers
must reject Some until their engine supports it. Existing rigid-body mass/inertia
validation is unchanged; this metadata API introduces no math dependency.


Visual.properties is an owned clonable VisualProperties record. It preserves
cast_shadows (default true), float32 transparency (default zero), visibility_flags
(default u32::MAX) and laser_retro (None if absent, Some even for explicit zero).
The optional laser value matches native HasLaserRetro for loaded XML; no generic
mutable C++ setter facade is claimed. Finite negative/outside-unit transparency
and negative laser values remain native DOM metadata, without extra clamping.

Transparency follows native double storage, 17-digit formatting and float getter
conversion, including zero fallback when the second conversion is out of range.
The Linux GNU/default-rounding qualification and audited strtof_l extension are
specified in SCALAR_CONVERSION.md. Native nonfinite DOM values remain an explicit
unsupported semantic boundary. These properties are metadata; no render/lidar
effect or image/mesh IO is implemented by parsing. Consumers may retain them as
scene metadata before rendering/lidar support; they must not claim those effects.


Geometry additionally retains boxed Heightmap and ordered Vec<Polyline> metadata.
Heightmap owns URI, size, position, paging and sampling, ordered textures/blends
and resource Source. Polyline owns height, ordered 2D points and Source. These
records are clonable and independent of Bevy, filesystem and image decoding. URI
strings remain unresolved, matching native default StoreResolvedURIs(false); the
include ResourceResolver is not called for image/texture assets. Source identity
continues to identify the imported document.

Native schema-inserted defaults are retained: missing heightmap URI and texture
strings become __default__, omitted required numeric values use schema defaults,
and a polyline without points contains one zero point. Explicit empty required
string/scalar children reject; empty vector children keep schema defaults. No
positive-height, closed-polygon, texture/blend count or sampling-power constraint
is invented. Metadata acceptance implements no collision, terrain, rendering or
lidar effects. Physics consumers must reject heightmap/polyline shapes until the
backend supports them. Full conversion, generic vector-stream parity, Element
mutation and serialization remain pending.


Model-scope merge includes flatten imported members through the native generated
proxy frame, retaining Model.merged_includes provenance snapshots. Imported scope
validation occurs before flattening. See MERGE_INCLUDES.md for exact reference,
boolean-attribute, deferred-error and unsupported-schema/legacy boundaries, plus the world-scope rules.
No new IO or simulator dependency is introduced.

Pure model.config selection is available through select_model_config, returning
owned filename/version metadata and ordered source-located warnings. Selection
retains native first-tag fallback when no supported version improves on 0.0; it
does not guarantee that the chosen file can be parsed. Filesystem lookup, path
policy, deprecated manifest fallback and actual resource loading remain caller
responsibilities. See MODEL_CONFIG.md for numeric/XML and safe-error boundaries.

The common loader subset accepts SDF 1.4 with the native direct-root/world joint
axis default insertion scope. Nested models retain their existing default, and
included documents use their own source version/scope. Actors, earlier versions
and Element/serializer/conversion facades remain unsupported; see VERSION_14.md.

World physics profiles are owned metadata, with native effective defaults and
first-default/first-profile selection; see docs/PHYSICS.md. Parsing does not choose
a runtime engine or timestep. Explicit engine-specific parameter blocks remain
Unsupported. Consumers must validate configuration they use rather than assume
that native DOM acceptance makes zero/negative values a valid runtime timestep.

Visual.material now retains optional owned basic material/script/shader metadata;
see MATERIALS.md. Colors follow native float-token range/default rules, render
order follows direct std::stof, and render/lidar/texture effects remain consumer
work. PBR metallic and specular declarations are retained as owned metadata; see
[PBR_MATERIALS.md](PBR_MATERIALS.md). Unknown extensions remain Unsupported. No resource resolver call or
asset IO occurs while loading this metadata. Scalar/vector numeric policies remain
distinct; the existing finite-double shininess boundary remains explicit.

Axis.dynamics and joint-stop stiffness/dissipation retain native metadata for both
axes, independently of runtime articulation support; see AXIS_DYNAMICS.md. Optional
source/XML records distinguish declaration presence. Native negative/default
values are retained; their runtime interpretation belongs to the consumer.

Link.properties owns the native wind/gravity/kinematic values and self_collide XML
parameter; see LINK_PROPERTIES.md. Parsing these flags does not implement physical
effects. Simulator builders must implement/reject unsupported nondefaults before
spawning. Joint::axis(index) borrows the first axis at zero and the second axis for every
positive index; this does not add mutable Element or serialization facades.

Model.model_by_name and link_by_name return references to stored DOM objects using
explicit names relative to that model. Descendant models require each enclosing
scope; unqualified names do not trigger recursive search. No leading global-root,
current-model prefix or semantic frame-alias lookup is added. This exposes the
existing canonical-link traversal without changing canonical-link selection.
Literal double-colon declaration names, interface models, mutable DOM facades and
model-level graph revalidation after arbitrary field edits remain outside this
subset. See MODEL_DOM.md for the original-fixture mapping.

Model.joint_by_name and frame_by_name borrow stored members through the same
explicit model scopes as link_by_name. Frame lookup includes declared frames
only, not implicit link/joint/model graph vertices. All three member methods
preserve native exact-name fallback when a model prefix is missing, including
literal-colon names assigned in memory. If the prefix resolves, only that nested
model is searched even if the direct parent contains a matching literal name.
Parsed declaration names still reserve double colons; edited DOM names do not
refresh the separately stored semantic graphs. See MEMBER_DOM.md.

Axis lower/upper limits preserve finite native values without requiring lower<=upper.
Native Root-loaded omitted bounds are negative/positive infinity after conversion
to the current schema, including supported older source versions. An omitted or
empty limit group uses these defaults; an explicitly empty bound child rejects.
Explicit `inf`, `infinity`, `nan` and signed forms in the four joint-axis
limit fields follow the pinned native DOM values; negative effort and velocity
still read as positive infinity. This local path leaves the general finite
scalar converter unchanged. Arbitrary numeric token prefixes remain outside
the qualified matrix. Parser acceptance does not execute, normalize or
approve limits for any backend; consumers must handle their own admissibility.
See AXIS_LIMITS.md for native controls and obsolete Rust-only rejection disposition.


Joint.sensors owns ordered supported sensor envelopes with names unique per owner.
SensorConfig::ForceTorque stores frame/direction and six Noise records; optional
force_torque() borrows it. Defaults, explicit source/raw XML and joint-default
semantic poses follow the bounded native controls in FORCE_TORQUE.md. The explicit
non-frame validator includes joint sensors; load-time deferred pose failures stay
deferred. Consumers must match or reject this enum variant and nonempty joint
sensors until runtime support. No measurement, noise execution or publication is
implemented here.


Terminal nested ::__model__ references map to an actual Model graph vertex while
retaining raw DOM strings. Link/Frame/Joint/missing prefixes and intermediate or
repeated implicit aliases do not qualify. Strict schema scope boundaries remain
unchanged. See MODEL_FRAME_REFERENCES.md for native observations, original joint
assertion candidates. The later static-model attachment correction closes the
previously recorded static child endpoint difference; historical native corpus
metadata remains unchanged.


An initial UTF-8 BOM is accounted for in original XML spans and tokenizer error
positions. Preserved plugin/configuration/physics/axis/include XML and imported
resources retain their exact original bytes. Source coordinates keep existing
LF-row/Unicode-scalar-column semantics over the original input. Repeated or
intermediate BOM document text remains a named grammar gap; see UTF8_BOM.md.

## Camera value boundary

The owned `Camera` value object retains native construction defaults and direct
field mutation for image, clipping, distortion, pose, lens, projection and
visibility metadata. `CameraPixelFormat` currently covers only the two formats
exercised by the retained construction original. A typed `SensorConfig::Camera`
parser retains the exact camera fixture fragment with owned source and XML. The
pinned fixed-fixture `DOMLink.Sensors` original maps all 27 sensor observations
and ordinary index/name lookups through the owned Link sensor vector. Native
Link method identity and mutable Element semantics are outside that mapping.
Remaining native camera grammar (`anti_aliasing`, `depth_camera`, segmentation
and box types, other pixel formats), `ToElement`, image capture and rendering
remain pending. An explicit `<camera>` requires
`<image>` and `<clip>`; only the first missing-element diagnostic is reported
by the Rust loader, while native `Camera::Load` returns both.

## Mutable Sensor payload values

`SensorConfig` owns one selected payload, which callers may mutably borrow and
then read through the typed `Sensor` accessors. The pinned
`DOMSensor.MutableSensors` original observes eight payload field updates and
reads; those values map through normal Rust enum borrowing. This is a value
contract, not a requirement to reproduce C++ mutable pointer or setter method
names. Generic in-memory Sensor mutation and serialization have separate
behavioral acceptance requirements. The native original also installs an IMU
payload while its type tag remains ForceTorque; that observed independence is
retained in the Rust case.

## Sensor owned copy and move values

The five pinned `DOMSensor` constructor and assignment originals observe final
owned values. Rust move, `Clone`, `clone_from` and `mem::replace` retain the
observed type, full raw pose, relative frame, frame ID, Magnetometer noise and
constructor update rate. The native cases do not inspect moved-from objects.
C++ operator method identity and Element pointer identity are outside this
value mapping; parsed Sensor behavior and serialization have separate gates.

## Sensor plugins

Owned `Sensor.plugins` keeps repeated plugins in declaration or insertion
order. The parser preserves each plugin name, filename and original XML span.
Cloning a `Plugin` before adding it captures the value, so changing the source
value does not alter the earlier entry. Native `DOMSensor.Plugins` observes
vector order and clear; `Sensor::ToElement` and broader plugin grammar remain
separate acceptance work.

## Model, World and Visual plugin values

These owned DOM values retain plugins in declaration or insertion order.
Cloning a `Plugin` before insertion preserves the original entry when the
source value changes, and clearing the vector removes all entries. The three
pinned native Model, World and Visual plugin originals observe those values.
Their C++ method names, `ToElement` serialization and broader plugin grammar
are separate contracts.

## Standalone Sensor metrics loading

`Sensor::load_element_str` loads an owned standalone `<sensor>` element without
a frame graph. For the pinned `DOMSensor.EnableMetrics` element with
`type='none'`, native `Sensor::Load` updates `enable_metrics` and then reports
an invalid type. The Rust method likewise retains the value change and returns
`InvalidValue`; full-document loading continues to reject type `none`. This
maps the original's observed false, true and false values. It does not imply
exact native error-vector parity, complete `Sensor::Load` behavior, Element
identity, serialization or general sensor grammar.

## Plugin owned copy and assignment values

The pinned `DOMPlugin` copy constructor, copy assignment, move assignment and
copy assignment after move originals observe name and filename values. Rust
`Clone`, `clone_from`, owned move and swap retain those source and destination
values. The native originals do not inspect moved-from state. Element
contents, `ToElement`, C++ method identity and general plugin parsing remain
separate acceptance work.


## Joint axis owned copy and move values

The five pinned `DOMJointAxis` copy and move originals observe XYZ values.
Existing XML axis validation plus ordinary Rust `Clone`, `clone_from`, owned
move and swap retain the original source/destination X/Y/Z values. Native
mutable `SetXyz`, default/Element/`ToElement`, frame graph and moved-from
state require separate evidence.


## Joint axis direction mutation

`Axis::try_set_xyz` accepts a finite direction, normalizes it, and preserves
the previous XYZ when validation fails. A zero-length vector within native
`SetXyz`'s default float epsilon returns `AxisDirectionError::ZeroLength`, whose
message matches the pinned original first error. Nonfinite vectors return
`NonFinite`. This owned value operation does not imply complete native
`SetXyz` error-vector parity, Element identity,
`ToElement` or frame-graph behavior.

## Joint axis XML direction threshold

The XML loader rejects a joint-axis XYZ vector whose length is at or below
the native default float epsilon promoted to `f64` (`f64::from(1e-6_f32)`).
Accepted directions are normalized. The pinned native 165-row axis corpus
checks the exact boundary, including decimal `1e-6`, while preserving all 160
earlier observations. This is a bounded `JointAxis::Load` correction; the native
full error vector, Element behavior and frame graph remain separate.

## Joint axis expressed-in frames

The fixed `DOMJointAxis.XyzExpressedIn` fixture loads through ordinary model,
link and joint lookups. `pose_of` and `resolve_axis` retain its raw and resolved
pose and direction observations. The native omitted axis target is the joint
child link, named explicitly when calling the owned Rust resolver. Invalid
`expressed_in` names return the native first typed code 38 and exact message;
the native second code 26 and complete error vector remain unported. This does
not imply general graph editing or native method identity.


## Joint sensor owned mutation

For an owned `Joint`, a caller can mutably borrow a `Sensor` from the ordered
`sensors` vector by index or by a name search, rename it, then find its new
name. The two pinned DOMJoint mutable originals establish those fixed value
observations after one unique insertion. The Rust empty/count checks are
supplemental. Native default Sensor construction, pointer identity, native
method identity, duplicate insertion policy and broader sensor grammar are
separate acceptance work.


## Model owned collection mutation

`Model::add_joint_unique` and `add_model_unique` consume an owned value and
return whether a direct member name was new. `clear_joints` and `clear_models`
empty those collections. The two pinned DOMModel add originals establish
fixed count, duplicate rejection, clear, reinsert and index-name values.
These mutations do not update an already built semantic frame graph. Native
default-object state, C++ method/pointer identity, arbitrary scoped-name or
cross-kind uniqueness, Element serialization and graph editing remain separate.


## Model owned member rename

`Model::add_link_unique` and `add_frame_unique` add direct members with
unique names and return whether insertion occurred. Alongside joint and
nested-model insertions, ordinary mutable vector borrows can rename those
owned members. The pinned DOMModel mutable originals observe index/name
lookup before and after rename; `link_by_name`, `joint_by_name`,
`model_by_name` and `frame_by_name` reflect current member names. An
already built semantic frame graph is unchanged by these edits. Native
mutable pointer or method identity, generic scoped/cross-kind uniqueness
and Element serialization are separate acceptance work.

`Model::clear_frames` removes owned frame members. Together with
`add_frame_unique`, mutable `frames` borrowing and `frame_by_name`, it
preserves the fixed add, duplicate, clear, reinsert and rename/relookup
observations in `DOMModel.AddModifyFrame`. Edits do not update an already
built semantic frame graph. C++ pointer or method identity and arbitrary
scoped name policy remain separate acceptance work.


## Link owned sensor collection

`Link::add_sensor_unique` accepts a sensor only when no direct sensor has its
name, and `clear_sensors` removes the owned sensor collection. The fixed
`DOMLink.AddSensor` original observes count, insertion, duplicate rejection,
clear, reinsert and index-name results. The Rust test uses a valid parsed
typed IMU sensor as its owned value. Native untyped default Sensor state,
C++ pointer or method identity, arbitrary name policy and broader sensor
grammar remain separate acceptance work.

## Link owned collision and visual collections

`Link::add_collision_unique` and `Link::add_visual_unique` accept a value only
when no direct collection member has its name. `clear_collisions` and
`clear_visuals` remove those owned values. The fixed `DOMLink.AddCollision`
and `DOMLink.AddVisual` originals observe count, insertion, duplicate
rejection, clear, reinsert and index-name results. Rust parses valid collision
and visual values as setup; native originals use default objects and do not
load XML. Untested default state, C++ pointer or method identity, Element
serialization and broader geometry grammar remain separate acceptance work.

## Link owned light collection

`Light` owns its direct name and six spot/attenuation scalars; `Link.lights` starts empty.
`Link::add_light_unique` admits a light only when no direct light has its
name, and `clear_lights` removes the collection. The fixed `DOMLink.AddLight`
original observes count, insertion, duplicate rejection, clear, reinsert
and index-name results. Native uses a default Light with a set name; Rust
initializes that name directly and parses only the enclosing Link as setup.
Light XML, pose, configuration beyond the six scalars, nonfinite scalar parity,
Element, rendering and C++ pointer or method identity remain separate work.
Public `Light` gained scalar fields and no longer implements `Eq`; consumers
using exhaustive struct literals or `Eq` bounds must adapt.

## Link owned mutable name collections

`ParticleEmitter` and `Projector` currently own only direct names. Their Link
vectors start empty, and `add_particle_emitter_unique` and
`add_projector_unique` reject duplicate direct names. The fixed native
`DOMLink.MutableByIndex` and `DOMLink.MutableByName` originals observe Add
success and mutable lookup, rename, and relookup across six owned families:
Visual, Collision, Light, Sensor, ParticleEmitter and Projector. Rust uses
vector borrowing and current-name search; parsed Visual, Collision, typed
IMU Sensor and the Link are test setup, while the remaining names are built
directly. Native default family configuration, XML grammar for Light,
ParticleEmitter and Projector, Element, C++ pointer/method identity and
general mutation policy remain separate acceptance work.

## Light scalar setter bounds

The fixed `DOMLight.SpotLightNegativeValues` and `DOMLight.AttenuationClamp`
originals observe negative spot falloff and angles clamp to zero, negative
linear and constant attenuation clamp to zero, values 20 clamp to one, and
negative quadratic attenuation clamps to zero. Owned setters preserve those
values in order. Native originals use direct setters and no XML input. Light
XML, pose, nonfinite scalar parity, Element, rendering and other configuration
remain separate acceptance work.

## Light owned default values

`Light` now owns native default point type, raw pose and relative frame, on and
visualize flags, shadow flag, float color channels, attenuation range, direction
and intensity alongside its scalar clamp fields. These values map the fixed
`DOMLight.DefaultConstruction` observations with direct owned assignments and
bounded scalar setters. The case remains partial: native `Element() == nullptr`
and the two standalone `SemanticPose.Resolve` error observations have no
corresponding owned API assertion. Cloning the `Pose` observes its raw and
relative values; it does not resolve a graph. Light XML, Element identity,
rendering, nonfinite inputs and broader frame semantics remain open.

## Light owned copy and move values

The five fixed `DOMLight` copy, assignment and move originals observe final
Light values. Rust `Clone`, `clone_from`, ownership transfer and `replace`
preserve the same name, type, raw pose, relative frame, flags, colors,
attenuation, direction, spot values and intensity. The swap-like final case
observes both resulting names. C++ operator or pointer identity, moved-from
state, Element, Light XML, rendering and general aliasing remain outside these
fixed value tests. No production behavior changes in this packet.

## Bounded standalone Light XML

`Light::to_xml` emits one SDF 1.12 `<light>` element, and `load_light_str`
reads that standalone element into an owned Light. Successful emission is
validated against the owned reader's supported names, pose, finite colors,
attenuation and spot ranges so accepted values do not get clamped or rejected
on re-read. The fixed `DOMLight.ToElement` and `ToElementErrorOutput` value
roundtrips map through this pair, including a shadow flag update. A generated
element also loads in the pinned native SDF 1.12 world library. The Rust XML
uses quaternion pose text and explicit light-on/visualize fields; its bytes and
DOM Element identity differ from native `ToElement`. Native Element pointer
observations and redirected console-buffer-empty behavior are pending. This
API does not yet connect Light parsing to `Link` or `World` loading, and does
not claim the full Light grammar, rendering, or native error-vector parity.

## Bounded standalone Frame XML

`Frame::to_xml` emits one SDF 1.12 `<frame>` element, and `load_frame_str`
reads that standalone element into an owned Frame. Emission validates the pose
and attribute strings before writing escaped name, optional attached_to and
quaternion pose values. The fixed `DOMFrame.ToElement` and
`ToElementErrorOutput` tests map their with/without-attached_to value results
through this pair. Both emitted variants also load within a pinned native
SDF 1.12 model whose link targets make the frame graph valid. The native
originals compare the source name to itself, so a Rust loaded-name check is
supplemental. Native Element pointer and redirected console-buffer observations
remain pending. The standalone reader does not build a frame graph, and the
Rust XML bytes, full document grammar, attachment resolution and native error
vectors are outside this bounded API claim.

`Frame::default()` supplies the fixed standalone empty name, attachment and
identity pose values. `Frame::semantic_relative_to()` returns an explicit pose
relative_to when present, otherwise the attachment, matching the native
`SemanticPose.RelativeTo` fallback. It returns a borrowed value and does not
construct a graph-backed SemanticPose. The native standalone resolver failures,
separate SemanticPose raw-pose observations and Element pointer remain outside
this bounded value API.


Five pinned `DOMPhysics` copy and move originals preserve their fixed final name
observations through owned `Physics` Clone, clone_from, move and replacement.
The native case named MoveConstructor invokes copy construction, retained as a
clone in its Rust port. These observations do not qualify C++ constructor/operator
identity, moved-from state, Element or physics-engine runtime.


Bounded Light/Frame XML output rejects nonzero subnormal pose components and
Light scalar/color components that their readers would reject at numeric range
conversion. Zero, signed zero and minimum-normal values remain supported. Light
direction retains its separate vector conversion. This tightens emission checks;
it does not alter the audited scalar loader or qualify additional native XML,
Element, console, graph or original-test semantics.


`Physics::to_xml()` emits bounded world profile metadata that the existing world
loader can read. It preserves the six fixed values in the two Physics ToElement
originals; update-rate metadata is supplemental. Both originals remain partial
because Element and console observations are unported. See docs/PHYSICS.md for
text/numeric limits; this API does not configure an engine.


`World.atmosphere: Atmosphere` owns Adiabatic configuration, temperature in Kelvin,
temperature gradient in K/m and pressure in Pa, plus optional original source/XML.
Implicit defaults are always present. Adding this public field requires exhaustive
World struct literals to supply a value, usually Atmosphere::default(). No current
Root consumer literal required adjustment. The field carries metadata; runtime
atmospheric sampling remains separate. See docs/ATMOSPHERE.md for native input
bounds, six explicit gaps and the scoped seven-original mapping.


## Owned geometry cloning

Geometry and Mesh implement Clone and clone_from for their owned metadata,
including nested URI/source strings, texture/blend records and polyline points.
The copies survive dropping the original and retain independent mutable data.
Cloning does not load a resource, alter parser/graph semantics, create a default
Geometry variant or provide native Shape/runtime methods. Four fixed Box size
ownership originals are complete; MoveConstructor remains partial for native
Shape volume/size observations. Native Vector3 equality's component-wise <=1e-3
bound is retained in tests, with tighter exact-bit checks as supplemental evidence.
General C++ constructor/operator/storage and moved-from identity are unqualified.

## Owned Visual cloning

`Visual` implements Clone for its owned pose, optional geometry/material, visual
properties, plugins and source provenance. Clones retain independent nested
strings, vectors and boxed metadata after the source is dropped. Cloning performs
no resource IO or rendering. Five fixed native copy/move originals observe names,
shadows, transparency, pose, relative frame, material ambient and visibility flags.
Native positive f32 four-ULP transparency, component-wise pose <=1e-3 without
quaternion sign relaxation, and ambient RGBA <=1e-6 bounds are retained; exact
retention checks are supplemental. C++ constructor/operator/storage identity,
moved-from state, Element and geometry-pointer APIs remain unqualified.

## Owned Collision cloning

`Collision` implements Clone for its owned pose, geometry, surface and source
provenance. Nested strings, Mesh metadata and optional surface group provenance
remain independent after the source is dropped. Cloning performs no resource IO
or physical execution. Five fixed native copy/move originals observe only final
pose values, including two final translations after swapping through a move.
Native signed quaternion and translation components use absolute <=1e-3 without
quaternion sign relaxation; exact owned retention checks are supplemental. Native
default EMPTY geometry, mutable Element, C++ constructor/operator/storage identity
and moved-from APIs remain unqualified.

## Owned Link and Inertial cloning

`Link` and `Inertial` implement Clone for owned metadata. Nested strings, vectors,
boxed fluid-added-mass values, collision/visual/sensor records and provenance
remain independent after the source is dropped. Raw scoped names and pose-frame
references retain their values; cloning does not rebuild or rebind a document's
frame graphs, load resources or execute physics.

Five fixed native Link originals observe only final names. The native case named
MoveConstructor actually invokes a copy; its Rust test retains that operation.
Default constructor values, mutable Element, C++ constructor/operator/storage
identity and moved-from APIs remain unqualified. Inertial ownership checks are
supplemental and do not claim an additional native original or inertia solver.

## Owned Model namespace metadata and cloning

`Model.raw_namespace` is optional owned metadata: None differs from Some("").
`namespace()` returns an owned resolved value, replacing every literal `{name}`
with the current public model name without recursively expanding inserted text.
Direct name edits therefore update the next namespace observation. Namespace
metadata does not change `scoped_name`, frame references or resource lookup.
The parser retains model `namespace` attributes as optional raw metadata. Native
space, tab and newline trimming applies; an empty XML value is absent, unlike an
explicitly empty value assigned through the owned API. Direct namespace cases
were checked across SDF 1.4–1.12. A nonempty include `<namespace>` overrides the
resource's value; empty, self-closing or whitespace-only overrides leave it
unchanged. Text and CDATA use the retained native parameter rules. Include name
overrides determine `{name}` resolution. Child models do not inherit namespace;
flattening a merged include leaves its parent's namespace unchanged. Original
include XML and canonical resource provenance remain available. These values do
not change frame scopes, pose references, resource lookup or simulator topics.
Root-level model includes use the same injected resolver, imported model metadata
and raw include provenance. Root merge includes remain unsupported after resource
resolution and validation. Native OriginalVersion access through Rust Model,
actor/light/URDF includes and serialization remain unqualified.
Native child SemanticPose lookup of __root__ or ::__root__ reports error 26 in the
qualified standalone cases. Global child positions checked through the Rust query
facade are compositions of independently observed model and child-local poses;
they do not claim successful native child-root target lookup.

`Model` and `Frame` implement Clone, retaining owned nested DOM values and
provenance after independent edits and drop. Cloning does not rebuild or rebind
frame graphs. Five fixed Model copy/move originals retain all final name and
optional namespace presence/value observations. Construction namespace checks,
placeholder/rename cases and nested lifetime checks are supplemental; whole
Construction, extra Frame originals, mutable Element, C++ operator/storage or
moved-from identity and physical runtime remain unqualified.

## World merged includes and joint metadata

`World.joints` owns global joint values; `World.merged_includes` owns ordered raw
include snapshots, imported model Source and generated proxy names. These new
public fields require updates to exhaustive `World` struct literals. A merged
world include imports nested models, frames, joints and plugins; imported direct
links return `ErrorCode::MergeIncludeUnsupported` (native first category 33).

The proxy preserves placement-adjusted pose and canonical-body attachment.
Imported scopes validate before flattening; world-joint endpoints, axes and poses
resolve in world scope, with pose defaulting to the child. World-joint sensor
poses remain deferred to `validate_non_frame_poses`. Loading returns the first
diagnostic atomically; native later error vectors and partial DOM are not exposed.
See MERGE_INCLUDES.md for provenance and remaining boundaries. No physics
constraints, resource search policy or downstream producer pins change.


## Unattached axis XML

`load_axis_str` parses current-schema standalone `<axis>` metadata; `Axis::to_xml`
emits its direction/frame, dynamics, limits/stops and optional mimic values.
Unattached metadata does not resolve frame or mimic references. Model loading
keeps its graph checks and existing version gates. The bounded writer uses native
unbounded sentinels and rejects values its XML encoding cannot preserve. The
published normalized-direction policy remains in force, including its explicit
1e-6 native difference. See [AXIS_XML.md](AXIS_XML.md) for the precise contract.


## Unattached joint XML

`load_joint_str` and `Joint::to_xml` read and write bounded current-schema joint
metadata, including raw quaternion pose/frame, endpoints, both axes and modern
screw pitch. Local validity checks are distinct from containing-model graph
validation. Unsupported sensor families/opaque extensions and retained physics
payloads, axis2-only values and
non-screw nondefault pitch return explicit errors. See [JOINT_XML.md](JOINT_XML.md).


## Joint semantic-pose queries

`Sdf::joint_pose` resolves a canonical loaded joint identity to a target in its
containing model/world scope. Empty target selects the declared child frame;
explicit targets use strict native local scoping. It shares the loaded graph
snapshot and math with `pose_of`. See [JOINT_SEMANTIC_POSE.md](JOINT_SEMANTIC_POSE.md).


## Force/torque configuration and sensor XML

`load_force_torque_str` and ForceTorque::to_xml preserve frame/direction and six
complete noise records. Sensor::to_xml and joint XML support bounded force_torque
sensors and their current envelope fields; unsupported families and retained
plugin/extension/physics payloads return explicit errors. Existing document
parsing, numeric admission and graph validation remain unchanged.
See [FORCE_TORQUE_XML.md](FORCE_TORQUE_XML.md).


## IMU XML

Imu::to_xml/load_imu_str preserve all six Noise records, orientation settings
and raw custom/gravity vectors and parent-frame strings. Matching standalone
Sensor output supports imu without changing finite/noise/negative-rate policies.
Joint XML remains force_torque-only; generic Element and runtime behavior remain
unqualified. See [IMU_XML.md](IMU_XML.md).


## Base camera XML

Camera::to_xml/load_camera_str preserve the bounded Camera.ToElement surface
and reject nondefault advanced retained optics instead of dropping fields. Native
pixel aliases, anti-aliasing and parsed presence/name/save defaults are supported.
Constructor values and old assertions remain unchanged; Camera Sensor/Joint output
and rendering stay unqualified. See [CAMERA_BASE_XML.md](CAMERA_BASE_XML.md).
