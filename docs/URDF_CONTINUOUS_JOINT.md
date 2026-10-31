# Bounded two-link continuous-joint URDF import

`load_urdf_str` additionally admits URDF 1.0 with exactly two distinct ordinary
links and one `continuous` joint. Each link retains the single-link admission
rules: required positive physically realizable inertia, optional local inertial
origin, and at most one primitive visual and collision. Unknown attributes,
children and duplicate singleton nodes are rejected before `urdf-rs` parsing.
No dependency, simulator, Bevy, Fuel or transport API is added.

The joint needs a name and different parent/child endpoints naming the two links.
Declaration order does not select the root: output orders the parent link, joint,
then child link, matching the native converter and its default canonical link.
The optional joint origin uses metres and fixed-axis radians, with zero defaults.
Native continuous joints become SDF `revolute` joints: the joint pose is relative
to the parent, and the child has identity pose relative to the joint. Model/root
poses start at identity. Inertial moments and inertial/visual/collision offsets
stay in their declared link frames. Existing SDF pose/attachment graphs resolve
all queries; no transform, inertia-rotation or joint-reduction algorithm is added.

An omitted axis defaults to positive X in the joint frame. An explicit axis must
have three finite components and a finite norm strictly greater than `1e-6`.
Existing SDF normalization and axis resolution are reused. Native probes show
that an axis of length exactly `1e-6` can remain unnormalized, while overflowing
norms can produce a zero direction. Those cases are explicitly outside this
subset; the shared SDF math policy is unchanged.

An omitted URDF limit preserves unbounded position, effort and velocity. The
adapter records actual limit presence because `urdf-rs` otherwise supplies zero
values. A present limit requires finite `effort` and `velocity`; zero stays zero,
and negative sentinels use the existing SDF unbounded policy. Explicit continuous
`lower`/`upper` attributes are unsupported: native conversion ignores them and
this adapter does not silently discard declarations. Optional dynamics requires
finite damping or friction; the other coefficient defaults to zero. Signed finite
coefficients are retained as metadata. Spring reference/stiffness and stop
stiffness/dissipation retain existing SDF defaults. No limit, damping, friction or
spring law is executed.

One optional Gazebo block may target either link. The existing material/mu rules
and reviewed generated-name guard apply to that selected link only. Material
needs a visual and friction needs a collision. Metadata native conversion would
leave unapplied is rejected. Multiple blocks, joint extensions, mimic,
calibration, safety controllers, fixed/revolute/prismatic/other joints, larger or
disconnected graphs, dummy world links, meshes and arbitrary extensions remain
unsupported. The ordinary SDF loader and URDF resource/include boundary are
unchanged.

`UrdfImport` keeps the same distinct immutable original URDF and generated SDF
snapshots and source rules. Generated Model/Link/Joint Sources point into the
actual generated SDF URI/bytes. Admission errors point at original nodes;
dependency or converted-loader errors retain original robot-root context.
Typed edits and owned joint copies do not rewrite snapshots or rebuild graphs.
The native copy probe checks raw getters only, without claiming mutable Element
aliasing, native moved-from state or graph mutation parity.

The retained `URDF2SDF.ValidContinuousJointedURDF` fixture is byte-identical to its
inline native input. Its complete five observations are successful loading,
Model presence, both named links and the named joint. Supplemental probes cover
raw/resolved link, inertial, visual, collision, joint and axis values, declaration
ordering, defaults, numeric/error boundaries and source ownership. Scalar/name/
geometry observations are exact; quaternion physical equivalence and resolved
coordinates use `1e-12`. The original has no numeric tolerance to replace, and
all existing tolerances remain unchanged. All 31 prior accepted generated XML
snapshots and both older URDF originals remain preserved.

Replay `python3 tools/urdf_continuous_joint_reference.py /path/to/pinned/sdformat`
for 82 actual native controls, copy and four frame-resolution probes, and the
unchanged full original. Provenance checks pinned source/math, installed headers,
build/SDK libraries, transitive linkage, urdfdom headers, exact dependency source
and fixture bytes. Other URDF originals, general articulated import, native Root
method identity and exact whole generated XML formatting remain unqualified.
Consumers must explicitly admit the resulting joint/metadata domain before a pin
migration. This slice changes no consumer pin and makes no performance claim.
