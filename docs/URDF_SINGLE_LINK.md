# Bounded single-link URDF import

The same explicit API also admits the separately bounded
[two-link continuous-joint subset](URDF_CONTINUOUS_JOINT.md). This document
describes the preserved single-link branch.

`load_urdf_str(xml, source_uri) -> Result<UrdfImport, Diagnostic>` imports URDF
1.0 with exactly one ordinary link, required positive physically realizable
inertia and zero or one visual/collision each. Sphere, box and cylinder geometry
have finite positive dimensions. Optional xyz/rpy origins retain metres and
fixed-axis radians. The root model and root link start at identity; inertial
moments stay in their declared inertial frame. No tensor rotation, joint
reduction, root-pose inference or new frame algorithm is introduced.

The adapter uses [urdf-rs 0.10.0](https://docs.rs/urdf-rs/0.10.0/urdf_rs/) for
owned core parsing. Its published source is Apache-2.0 and the exact package and
transitive checksums are locked. Existing quick-xml/XML facilities provide the
admission tree and safe escaping; the existing SDF loader supplies physical
inertia, geometry and frame validation. A custom general URDF/XML parser would
duplicate that maintained work. No native SDK, simulator, Bevy or Fuel dependency
is added. Third-party dependencies retain their own licenses.

Admission is mandatory before `urdf_rs::read_from_string`: its reader removes
unknown/Gazebo nodes, and its vector reader filters malformed numeric tokens.
Every admitted node rejects unknown attributes/children and duplicate singleton
fields. Native-compatible whole numeric attributes and space-separated vectors
are validated with std before dependency parsing. Scalar leading ASCII whitespace
is accepted; trailing scalar whitespace, malformed vector tokens and nonfinite
values fail. Literal XML newline normalization and character-reference boundaries
are retained. Core URDF attributes retain the observed extreme-underflow-to-zero behavior;
nonzero subnormal converted scalars retain the existing SDF range-error policy. No float parsing algorithm is implemented here.

Only a single `<gazebo reference="the_link">` block is admitted, with material,
mu1 and mu2 children. Material becomes a script name and the native legacy
`file://media/materials/scripts/gazebo.material` URI. No script is opened or
activated. Friction values use the existing finite GNU numeric boundary, retaining
native std::stod prefix behavior, and must be nonnegative. Omitted coefficients
default to one. Material requires a visual, friction requires a collision, and
an absent or different reference is an error. Unknown or unapplied declarations
are never silently discarded. Native nonfinite coefficients are explicitly
outside the existing finite Surface policy; native scalar exceptions become
source-located errors. A visual receiving material, or collision receiving mu1/mu2, must have a native
generated name without the exact `_fixed_joint_lump__` token. Native conversion
otherwise leaves that metadata unapplied. The importer returns a source-located
Unsupported error instead of applying metadata that native conversion would
ignore. Names alone, or a name on the other view when it receives no such
metadata, remain admitted.
No friction or damping law is implemented.

Names follow the observed native converter: choose the explicit nonempty name
or the link name, then append `_visual` or `_collision`. If that generated name
does not start with the link name, it gains the native `_fixed_joint_lump__`
prefix even without a joint. Signed scalar zero is
printed as zero, matching native conversion. Other generated XML formatting is
not a native serialization contract. Quaternion physical equivalence is checked
within 1e-12, including gimbal and very large finite input angles; scalar values,
geometry, names and units are compared exactly in the qualified native controls.

Outside that continuous-joint subset, joints and multiple links are unsupported.
World dummy links, root origins, multiple visuals or
collisions, URDF versions beyond 1.0, meshes/capsules, inline/global URDF materials,
textures, arbitrary Gazebo/SDF extensions, plugins and xacro are unsupported.
Native zero/negative dimension fallback is also outside this positive-only
subset. The existing `load_str` remains an SDF loader and URDF resource includes
remain unqualified; the new explicit API does not broaden either path.

`UrdfImport.sdf` is the ordinary owned SDF view. `original_source()` and
`original_xml()` describe the unchanged original URDF input, including its actual
robot-root position. `converted_source()` and `converted_xml()` describe the
actual generated SDF 1.12 bytes, under `generated:urdf-to-sdf:<source_uri>`. Every
converted Model/child Source and immutable loaded-element snapshot refers to
that generated view. Typed edits do not rewrite either immutable XML snapshot.
Admission errors point to actual original nodes. Dependency parsing and converted
SDF validation errors are attributed to the original robot root with conversion
context; they do not invent a precise original field location. No mutable native
Element, native FilePath/OriginalVersion or exact whole-XML parity is claimed.

The two candidate original mappings retain the unchanged inline URDF inputs.
ValidBasicURDF checks successful import, Model presence and the named link.
ParseWhitespace checks all thirteen original selected-tree presence observations
and the exact three GetText results: Gazebo/Orange, 100 and 1000. These checks
read the real public converted snapshot; they do not assert whole XML equality.
Other URDF originals, including OutputPrecision's exact generated pose text and
the remaining articulated/fixed-joint reduction cases, remain unported. Existing URDF
extension ledger rows still describe their previous native-projection-only scope.

Replay `python3 tools/urdf_single_link_reference.py /path/to/pinned/sdformat`
for 84 actual native controls and both unchanged full originals. Provenance
checks clean pinned sdformat/math source, installed headers, SDK/build library
hashes, native urdfdom headers/transitive linkage, imported fixture bytes and
the exact Rust dependency source/lock. No performance claim is made. Consumer
adoption must explicitly handle the new import result and generated provenance;
this slice changes no simulator pin or runtime behavior.

The named-target regression corpus captures twenty-nine actual native inputs and
Rust projections from accepted revision `74fa759`. An outside visual name loses
native material while old Rust applied Gazebo/Orange. An outside collision name
leaves native mu1/mu2 at one while old Rust applied 100 and 1000. The correction
rejects only these unapplied target combinations. Empty/default names, matching
byte prefixes (including `linkage` for link `link`), suffix-completed link
prefixes, case-sensitive tokens, decoded tokens, tokens completed by the suffix,
tokens in the link name and unrelated view names are qualified. Rejection also
preserves an unapplied declaration when its numeric value equals the default. All 84 prior controls/assertions stay intact. All 31 accepted generated
XML snapshots captured from `74fa759` remain byte-identical.
Replay `python3 tools/urdf_named_extensions_reference.py /path/to/pinned/sdformat`
for the twenty-nine native regressions and both unchanged full originals. The
native helper is the unchanged single-link DOM probe. This fix adds no original
case credit, dependencies, mutable Element semantics or runtime behavior.
