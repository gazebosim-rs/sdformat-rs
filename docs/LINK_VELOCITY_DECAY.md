# Link velocity-decay XML metadata

`Link.velocity_decay` is optional owned `VelocityDecayMetadata` containing
linear/angular f64 scalars, actual declaration Source and exact input XML. The
pinned native Link DOM has no velocity-decay getter or stored typed field; the
reference reads its retained Element. This API describes that XML metadata. It
does not apply damping or define a decay equation or unit conversion. The new
public Link field affects exhaustive literals and destructuring; consumers must
explicitly support or reject it before adopting the producer.

An omitted group is None. An explicit empty group is Some with zero values; an
omitted or empty linear/angular child also defaults to zero. Native Root probes
confirm this behavior across SDF 1.4–1.12, including 1.4 whose schema declares the
group and children required. The API does not claim native child Element presence:
the corpus records those flags separately from the effective scalar comparison.
Native Link construction has no Element and no decay group. No Rust constructor
or default facade is added.

Finite negative values and values greater than one are retained without clamps.
Scalar conversion reuses the existing native-compatible numeric boundary:
true/false, numeric prefixes, CDATA and mixed-text behavior follow that boundary.
NaN, infinity, overflow, underflow and malformed values return InvalidValue under
the loader's existing finite-value policy. Native accepts nonfinite values; this
stricter boundary is explicit. Duplicate groups or scalars return InvalidValue;
unknown attributes, child elements and case variants return Unsupported. World
scope remains Unsupported. Model-scope velocity_decay remains an existing opaque
model extension and does not populate Link metadata.

The XML and Source are snapshots and do not change after typed edits. Ordinary
Link Clone copies owned data independently. Native Link copying shares its
mutable Element pointer: the reference mutation changes both native views. Rust
owned cloning does not claim that aliasing behavior.

The partial `SDFParser.UrdfGazeboExtensionURDFTest` mapping retains only the
velocity-decay observations in its original six-link fixture: link1 is 0.1/0.2,
link2 is 0.1/0.1, and the other tested links have no group. Original double-equality
intent is kept within four ULP, with exact native scalar-bit comparisons as an
additional check. The unchanged full C++ original runs successfully. Rust reads
a minimal SDF projection produced from the native-converted original URDF, with
exact Element subtrees and link order. Rust URDF conversion, other extension
assertions, mutable Element operations and the complete original case remain
unported. The unchanged bad_syntax_double fixture additionally checks the actual
malformed linear scalar's source location; no Parser.SyntaxErrorInValues credit
is claimed.

Replay `python3 tools/link_velocity_decay_reference.py /path/to/pinned/sdformat`
for 57 native controls, constructor/copy behavior, full original/projection
equality and the unchanged original test. Provenance pins the SDK headers and
libraries, schemas, parser and Link source. The inspected pinned DART and MuJoCo
SDF loaders and simulator Physics system do not read this field; their source
hashes are recorded. This bounded inspection does not qualify a runtime damping
implementation. No simulator pin, dependency or physics engine changes here.

The first normal staged gate found the aggregate prefix at parse.rs:2110 absent
from LLVM's DA records. The retained ordinary diagnostic and bounded constructor
proof distinguish this syntax from executable field expressions. The extension
reserves only VelocityDecayMetadata at the existing audited binding, macro and
export edges. Shadowed constructors, custom macros and runtime calls remain
unproven; DA0, uncovered regions and absent instrumentation are never excluded.
