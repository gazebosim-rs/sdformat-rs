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

`World` has name, gravity in m/s², models, frames, plugins and source.
`Model` has raw pose, static flag, canonical-link and placement-frame references, links, joints,
nested models, frames, plugins, scoped name and source.
`Link` has pose, inertial, collisions, visuals, sensors, plugins, scoped name and source.
`Inertial` has mass in kg, six inertia components (ixx, iyy, izz, ixy, ixz, iyz)
in kg·m² and inertial-frame pose. Positive physical inertia is required.
`Collision` has pose and box/sphere/cylinder/plane geometry, with SI dimensions.
`Visual` has name, pose, optional geometry, plugins and source. Missing
or empty geometry preserves native EMPTY as None. Visual names are unique within
a link, independently of collision and sensor names. Rendering properties,
and material data remain explicit Unsupported.
`Joint` has kind, parent/child references, pose, optional axis/axis2 and source.
Universal and revolute2 types retain both axes; native axis2 is also preserved
when present on another supported joint kind.
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
inertial or visual pose using its model scope and owning link. `attached_body` follows the
separate attachment graph to a link or static world attachment. Names, unknown
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
SDF 1.5/1.6 model members share a namespace in declaration order: later links, joints
and frames with cross-kind collisions append _link/_joint/_frame, then a numeric
suffix starting at0 when needed. Nested models are never renamed. Worlds load
models before applying the same frame renaming, independent of XML order.
Same-kind raw duplicates reject, and reference strings are not rewritten.
Modern shared names reject. Flattened names remain Unsupported; 1.4 and older
conversions remain unavailable. Full schema/version-aware
validation, serialization, visual rendering properties/material data, mesh resource loading, merge, automatic inertia, URDF, other sensor families and most other
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
contain one model or worlds, and root-model pose relative_to must be empty.
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
does not choose a runtime name or apply consumer fallback. The sensor remains
an IMU-only supported envelope; no additional sensor-family completion or full
DOMSensor original-case retirement is claimed by these metadata assertions.


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
attachment resolution, which can attach a static canonical subtree to world.


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
acceptance; 1.4 roots remain Version errors.
