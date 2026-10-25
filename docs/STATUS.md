# Evidence and open work

Task: first bounded SDF-01 / QLT-01 slice, 2026-10-24. No SDF-01, SDF-02, P1 or
release completion is claimed. The implementation author cannot approve its own
assertion equivalence or exclusions.

Upstream main: `1249805d9fceeff94ca1fe63e26ab9c051cea24e`.
`python3 tools/inventory.py ../sdformat` expanded the actual existing reference
build: 308 CTest targets, 1344 runtime cases, zero enumeration command failures,
1065 product C++ TEST macros, three example macro mentions in bundled gtest
implementation code, and 1337 source/schema/fixture SHA256 entries. All 1344
runtime cases map to an exact source path, line and symbol. One macro-wrapped
platform-specific test name is normalized for the Linux runtime expansion.
Generated test-result wrapper targets have no independent runtime cases. The 1065 product C++ source cases exactly reconcile to 1065 runtime C++ cases;
279 Python cases account for the rest. Non-enumerable target commands remain explicit in inventory.
All cases are pending unless specifically mapped in traceability.json; the
reference build has not been rebuilt in a pinned reproducible image here.

The three original C++ cases below passed using the existing native build:

```
LD_LIBRARY_PATH=../sdformat/build/lib:/home/nkoenig/local/lib ../sdformat/build/bin/INTEGRATION_nested_model --gtest_filter=NestedReference.PoseRelativeTo:NestedReference.PoseRelativeToInWorld --gtest_output=xml:artifacts/reference-nested.xml
LD_LIBRARY_PATH=../sdformat/build/lib:/home/nkoenig/local/lib ../sdformat/build/bin/INTEGRATION_model_dom --gtest_filter=DOMRoot.LoadNestedExplicitCanonicalLink --gtest_output=xml:artifacts/reference-canonical.xml
python3 tools/reference_probe.py ../sdformat
```

The first two fixture ports retain all six existence, resolve-success and pose
assertions each. The initial helper used1e-6 and allowed quaternion sign
equivalence; independent follow-up review corrected this to preserve native
component-wise <=1e-3 Pose3 equality, retaining tighter1e-6 supplementary checks. The canonical-link port maps C++ pointer/index assertions
to owned collections and verifies resolved scoped body identity. These three complete ports were independently reviewed by /root against their
original assertion bodies and committed in 7fa1fc2. Additional tests exercise a valid
nested attachment fixture, typed physics input, graph failures, bounds, includes,
provenance, plugins, and malformed input; these are supplementary tests, not a
claim to port whole upstream families. The native/Rust acceptance corpus
proves canonical-link aliases and joint frame aliases are accepted, while same-body
joint aliases and a world child are rejected. Diagnostic text/count parity is not
claimed: Rust currently returns one classified diagnostic.

`python3 tools/verify.py` uses only the exact Rust 1.94.1 LLVM 21.1.8 tools and
repository-local target/artifact directories. Its manifest has one production
crate/profile and no exclusions. JSON production line/region counts, LCOV changed
lines, test output, source digest, toolchain and runner identity are emitted to
artifacts/core/. Every src/*.rs must be instrumented. Gate self-tests reject
exactly 95%, below 95%, empty denominators, absent files/instrumentation/profiles/
crates, duplicate instrumentation, exclusions, stale review receipts, removed
assertions, a failed runner and partial staging. Every invocation reruns tests;
there is no stale-evidence cache. A staging check exports the actual index tree,
checks it independently, and verifies that the index did not change during checks.
The review receipt pins protected test/mapping/policy content and names a different
reviewer. Hosting branch protection remains a repository-admin task.

Dependencies: quick-xml0.38.3 (tokenization and matched elements),
xmlparser0.13.6 (XML names/references/source positions), glam0.30.9
(f64 rigid transformations), petgraph 0.8.1 (cycle detection/topological order).
Exact versions plus transitives are locked. Custom code is SDF graph scoping and
schema/domain validation, not generic XML, quaternion or graph algorithms.
serde_json 1.0.150 is a test-only corpus dependency.

The initial IMU slice added typed IMU-specific configuration and source mapping.
The original DOMLink.Sensors IMU assertion subset passed on the exact IMU subtree
from the unchanged sensors.sdf. Other families remained pending in that slice,
so the original case was not marked ported. The later Contact candidate below
adds one more exact subtree; the case remains partial.
The complete native DOMLink.Sensors case passes. Differential probes establish
negative noise sigma rejection, case-insensitive gaussian_quantized, and native
preservation of unknown localization/parent-frame strings. Original sensor XML
is retained with typed settings. At that stage the full fixture had an undeclared XML
prefix rejected by roxmltree; the later element cursor described below corrected
that syntax limitation. Exact-subtree extraction preserves the relevant original
bytes; unsupported typed sensor families remain explicit. The initial slice had
18 Rust tests and six gate tests; coverage independently reviewed by /root is 1193/1201 lines, 2077/2164 regions, and 117/118 changed
executable lines. No coverage exclusion was introduced.

Next bounded work: model/include placement frames and merge behavior with C++ probes; older conversions and
serialization; complete schema and retained test inventory, fuzzing, and native
parse benchmarks on tiny and large nested inputs. No comparative performance
claim, full upstream pass, full source/build provenance, or release qualification
is supported by this initial slice.

The third corrective slice separates fully scoped default edges from local XML
references. Native/Rust corpus cases verify that model-local poses, attachments,
canonical links and axes cannot reference world or re-prefix their own model;
world-scoped frames/model poses may reference world, while joint parent retains
its special world endpoint. Top-level model pose relative_to and multiple root
models are rejected. Static models preserve upstream's ignored canonical-link
setting. The canonical __model__ self-reference now reports Cycle, matching the
native graph error instead of the earlier generic InvalidValue. This change is
an explicit error-class correction; the malformed document remains rejected.
Relative query arithmetic overflow is also rejected. The corpus now contains
22 native/Rust cases; full retained semantic coverage remains pending.

The fourth bounded slice supports model/include placement frames, including
nested placement corrections, frame/link/joint/model targets and include overrides.
Both original NestedReference.PlacementFrameAttribute and PlacementFrameElement
native cases pass; corresponding Rust ports preserve the unchanged nested fixture,
all four original include scenarios and native component-wise <=1e-3 pose
assertions, with supplementary tighter1e-6 bounds. Include
placement without a pose is rejected after checking the native error contract.
Supplementary tests cover clearing overrides, nested models, source provenance,
relative_to, unresolved frames, and avoiding false intermediate overflow before
placement correction. The native/Rust corpus has 26 acceptance cases and three
shared numeric model-pose observations. There are 26 Rust integration tests.
Placement resolution uses petgraph A* over validated parent edges, descendant
model placements first, then one absolute pose solve. Performance measurements
remain pending; no comparative speed claim follows from this implementation.

Dependency-policy follow-up: exact library dependency requirements can conflict
with consumer-compatible patch selections. Root integration downgraded its glam
lock entry from 0.30.10 to this library's exact 0.30.9. Evaluate compatible semver
requirements with locked repository validation and explicit integration pins;
this slice does not change the established dependency selection.

Placement slice verified production coverage:1300/1314 lines and 2261/2352 regions;
changed executable lines42/42. Independent review:/root. Both canonical coverage
metrics and changed lines strictly exceed95%; no exclusions added.

The fifth precision slice ports FrameSemantics.resolveAgainstOwnFrameIsExact
with its unchanged fixture and every exact scalar pose/axis assertion. The native
case passes. Same-frame queries now return exact identity after validating both
names, and same-parent non-frame poses retain their exact declared transform.
Relative queries use parent edges up to their nearest common ancestor, avoiding
cancellation through absolute world poses. A native/Rust numeric corpus case
confirms that a 0.125-m local offset survives a 1e16-m model origin; the earlier
absolute-product implementation lost that offset. Global/world queries retain
cached absolute transforms. Native corpus:27 acceptance cases and four numeric
pose observations. Rust integration suite:29 tests. No comparative performance
claim is made; ancestry queries allocate two short paths, while global and exact
self queries retain fast paths. XML tokenizer compatibility evaluation is next.

Precision slice coverage:1347/1361production lines and 2353/2444 regions;
changed 48/48. Independent review:/root.

The sixth syntax slice replaces roxmltree after demonstrating its incompatible
rejection of undeclared extension prefixes. The private SDF element cursor uses
quick-xml for tokens, literal matching and duplicate attributes; xmlparser supplies
XML-name validation, reference decoding and source positions. This avoids both
namespace injection and a custom XML lexer. Name validation is necessary because
quick-xml intentionally tokenizes invalid names such as <1bad/>, which native
TinyXML rejects. Raw plugin spans index the unchanged input. Native differential
cases establish acceptance of undeclared prefixes, unknown references, bare
ampersands and opaque numeric-reference quirks; literal NUL and invalid element
names are rejected. Full unchanged sensors.sdf now reaches Unsupported semantic
validation instead of failing XML parsing. At that stage its other 26 sensor
families remained pending; the later Contact candidate reduces that number by one. DTD remains an explicit Unsupported feature. Original fixture bytes and
all earlier assertions remain intact. Corpus:35 native/Rust acceptance cases,
four numeric pose observations;33 Rust tests and six gate tests pass. Cargo
manifest/lock/toolchain files are now included in protected review digests.

Independent XML review by the transport owner caught four mismatches before
acceptance: aggregation across comment/CDATA text boundaries, rejecting native
legacy comments, accepting nested declarations/PIs, and missing native depth
limits. All are corrected with unchanged reviewer inputs and native scalar
observations in the differential corpus. Additional typed numeric-NUL name/file
truncation was checked and preserved. The earlier supplementary strict-comment
negative assumption was replaced with native-backed positive preservation tests;
no upstream assertion was removed. Boundary probes establish495 nested nonempty
plugin children pass and 496 fail in the original three-element wrapper. Legacy
header declarations remain permissive, as native accepts <?xml?> before the root.
Corpus:53 acceptance cases, four numeric poses, nine scalar-mass observations and
three typed-string observations.36 Rust tests and six tooling tests pass;
coverage1618/1639 lines,2772/2883 regions,changed 261/265. Final independent re-review
and staged verification remain required. The dashboard is maintained at
https://gazebosim-rs.github.io/rewrite-progress/ with separate accepted slices and
whole-packet status.

The seventh default/version slice corrects the inherited moving-joint axis
requirement: native accepts omitted axes and returns no Axis object for all four
supported joint kinds. Rust now preserves None. The original supplementary
same-body/missing-axis input remains rejected, now for its native invalid-endpoint
reason; valid distinct-body omitted-axis cases are positively asserted and
native-probed. SDF1.7's unchanged subset is accepted, including the original
frame_only_model resource. Required unflattening of flattened names is explicitly
Unsupported, and 1.6 conversion remains unavailable. Both native and Rust pass the
complete NestedModel.NestedFrameOnlyModel pose assertions on the unchanged1.7
fixture.39 Rust tests and six tooling tests pass; full versions/conversion parity
and the overall SDF packets remain in progress.


The visual/second-axis candidate supports visual poses and optional primitive
geometry with raw plugins, plus Universal/Revolute2 kinds and optional axis2 on all
supported joints. It reuses primitive geometry and axis validators. Two original
native nested-frame cases pass; Rust ports preserve the unchanged1.7fixture,
original parent-relative-to-nested-frame lookup then inversion, and both joint
axis assertions. Both ports were independently reviewed by /root/msgs, including original
component comparisons, and all 45 Rust tests plus76 native differential cases
were independently rerun. The prior
supplementary visual-unsupported example is now tested positively; its negative
assertion checks unsupported material instead. No upstream assertion was weakened.

Native Root load accepts unknown relative_to references on collision/visual/IMU
poses; SemanticPose.Resolve then returns Error26. Rust now preserves that timing.
The exact supplementary collision/world input still asserts UnknownReference,
now at resolution instead of incorrectly asserting Root rejection. Five native
negative cases establish that link/frame poses, joint endpoints and both joint
axes remain eager load errors. Explicit validate_non_frame_poses recursively
resolves supported non-frame poses before scene construction. Native ignores
inertial relative_to (Link.cc TODO); Rust retains and checks it only under explicit
resolution/validation policy, without claiming native ResolveInertial parity.
Native acceptance corpus:76cases, including four deferred error observations.
45 Rust tests and 8tooltests pass; production coverage1677/1698 lines and
2889/3001 regions, changed 107/107. No exclusions added. Independent assertion
and policy review:/root/msgs. Strict staged-tree verification remains required.

Native probes also expose a pending numeric lexical gap: Param.cc accepts scalar
prefixes and true/false coercion through std::stod/stoul, while supported Rust
scalar parsing currently requires an entire finite decimal token. Visibility
flags use Linux unsigned-long parsing then u32 wrapping. Visual transparency is
f32 and native typed-accessor overflow/subnormal errors can log and return0
without Root errors. Rendering properties remain Unsupported until those
contracts receive separate bounded implementation and differential coverage.
No accepted/public candidate revision or whole-packet completion is claimed.

Independent msgs review checked the actual installed gz-math10 headers used by
the native build: Pose3::operator== delegates to Vector3/Quaternion component
equality with <=1e-3 tolerance, without q/-q equivalence. Earlier ledger prose
incorrectly described1e-6 as the default. All affected pose ports now include the
original component-wise assertions, retain the tighter1e-6 checks as supplementary
assertions, and remove the quaternion sign relaxation. Fixture bytes, original
lookup/inversion paths and production arithmetic are unchanged.

The next bounded version slice applies the two rules in the pinned
sdf/1.7/1_6.convert directly to effective typed references. Pose frame overwrites
relative_to even when empty; both joint axes map exactly true/True/TRUE/1 to
__model__. Other strings (including mixed-case tRuE) are removed rather than
parsed as booleans. Native mapping without xyz creates an empty required element
and fails loading; Rust retains that failure. Included resources use their own
version while their URI, line and column remain unchanged. Original root version
is retained. The former supplementary 1.6 rejection now has a positive assertion;
its empty-document error is Missing, and the older-version rejection is retained
at 1.5. No original upstream assertion was changed or newly retired.

The native corpus has 122 acceptance cases, with old pose-reference
observations and legacy axis references/resolved vectors. Rust has 47 integration
tests; 11 tooling tests, fmt, clippy and doc checks pass. Production coverage is
1749/1770 lines and 2984/3094 regions, changed 84/84; no exclusions. Native 1.6
shared link/joint names rename the joint (a -> a_joint); that legacy graph
compatibility and flattened-name unflattening remain explicit Unsupported.
Full versions, serialization and the SDF packets remain incomplete. Independent
review:/root/msgs, including47 Rust tests,11 tooling tests and 122 native cases.
Strict staged verification remains required for this version slice.

Independent review caught two legacy whitespace mismatches before acceptance:
CDATA-padded true must not map like ordinary padded text, and native pose-reference
trim removes only space/tab/LF, preserving NBSP and numeric CR references. The
cursor now retains first-text/CDATA origin; only legacy mapping uses it. The XML
library normalizes literal attribute EOLs before reference expansion, keeping
numeric CR distinct. Native-backed cases cover both axes, missing-xyz behavior,
CDATA, NBSP, numeric references, literal CR and CRLF. Original plugin XML bytes
and source positions remain preserved.

Follow-up probes retain native C isspace's VT/FF characters in ordinary whitespace
collapse and leading-segment recognition. Numeric references create a text node
even if its decoded whitespace later collapses empty, so later CDATA cannot win;
literal whitespace creates no such node. A leading child blocks text lookup while
comments are skipped. Controls cover NBSP/VT/FF, literal/reference forms, CDATA,
child/comment order and both axes, including the missing-xyz acceptance boundary.

A separate legacy-name candidate now implements native SDF1.6 collision handling:
model members load in declaration order; later links, joints and frames append
their type suffix and then numeric suffixes starting at0. Nested models are not
renamed. World models are collected before frame renaming regardless of XML
declaration order. Same-kind raw duplicates reject and reference strings remain
unchanged, including references that consequently fail semantic resolution.
Modern shared names still reject. This replaces the previous shared-name
Unsupported boundary; flattened-name conversion remains pending.

Both original cases DOMJoint.LoadLinkJointSameName16Valid and
DOMWorld.LoadModelFrameSameName pass natively. Complete assertion ports preserve
names, counts, index bounds, raw references/poses and all six resolved poses per
case, including original component-wise <=1e-3 comparisons plus tighter
supplementary checks. Their fixture bytes match upstream. The corpus now has139
native acceptance/query cases, covering order reversals, future suffix collisions,
numeric suffixes, nested models, unchanged joint endpoints, same-kind duplicates
and a modern-version control. All 50 Rust tests,11 tooling tests, fmt, clippy and
doc checks pass. Coverage 1823/1842 lines,3120/3227 regions and changed 104/104.
No dependencies, exclusions or thresholds changed. Independent review by
/root/msgs verified every original assertion and fixture, reran both original
native tests, all 50 Rust tests,11 tooling tests and 139 native observations.
The two mappings are reviewed; strict frozen staged verification remains required.


A separate geometry/schema candidate adds typed capsule, cone, ellipsoid and mesh
metadata without IO or new dependencies. Native probes retain defaults,
nonpositive-dimension fallback, negative/zero mesh scale, unknown optimization,
independent decomposition settings, required/optional empty-value distinctions,
submesh errors, exact string whitespace and Linux unsigned-long-to-u32 semantics.
The native reserved-name predicate now requires both surrounding double
underscores; the former __reserved include rejection remains as a positive
regression, while __reserved__ retains the negative check. Scalar booleans are
ASCII case-insensitive and retain optional defaults for empty elements.

The isolated candidate passes59 Rust tests and 11 tooling tests;87 geometry and 72
named-entity native cases match. Eight original Geometry typed-shape cases and
the original15-input ReservedNames table have pending assertion ports. Enum
matching preserves shape type/presence; XML construction replaces mutable C++
setters. Full integration Shapes remains pending because its unchanged fixture
also includes collision surface and rendering properties. Production coverage
1927/1950 lines,3327/3440 regions, changed 176/178. No exclusions or performance
claim. Independent review and frozen staged verification remain required.


Independent transport review found malformed required scalars accepted as defaults
for NBSP/VT/FF/CR and a mesh vector accepting NBSP as a separator. Required scalar
blank recognition and vector tokenization now use the native six-character C
whitespace set. Leading Unicode scalar whitespace rejects; valid numeric-prefix
values ending in whitespace retain native acceptance. Additional controls show
that optional whitespace-only mesh scale uses its default, while explicit required
blank dimensions reject. Ordinary mesh URI/submesh strings collapse whitespace;
CDATA keeps its native distinct behavior. Boolean controls cover both origins.
The native geometry corpus now includes87 cases; original assertion ports are
unchanged. General non-whitespace floating-point prefixes remain explicitly pending.


Transport independently re-reviewed the corrected isolated implementation and all
new original mappings. It reran59 Rust tests,87 native geometry cases and 72 native
named-entity cases; original8 Geometry value cases and 15 ReservedNames assertions
were checked against pinned source. Integration onto the reviewed legacy-name
slice is separate and requires combined checks and final matching receipt.

Combined integration onto reviewed eb815a1 passes62 Rust tests,11 tooling tests,
all 139 base,87 geometry and 72 named-entity native observations. Production
coverage2001/2022 lines,3463/3573 regions; changed 176/178. The production delta
and all new test bytes are identical to the isolated reviewed candidate; the
native probe keeps its existing declaration-name/body observations unchanged.
Final independent integration receipt and strict frozen staged gate are required.

A focused follow-up to the reviewed geometry snapshot fixes24 additional native
optional-number controls before publication. Ordinary VT/FF/CR references collapse
to empty and receive field defaults; CDATA VT/FF remain nonempty and must fail
numeric parsing. Literal CDATA CR normalizes to LF in the XML library and then
uses defaults. The same distinctions apply to mesh decomposition counts, mesh
scale and mass. A shared borrowed typed-text adapter now serves scalar numbers,
booleans, unsigned counts, mesh strings and optional scale checks. Common already
normalized values borrow their storage; whitespace collapse allocates only when
needed. Original assertion mappings are unchanged.

All 62 Rust tests,11 tooling tests and 139+111+72 native observations pass.
Coverage 1997/2018 lines,3455/3565 regions, changed 24/24. No dependency,
exclusion or threshold change. Independent /root/msgs follow-up review reran
all 62 Rust/11 tooling tests and 322 native observations and approved the exact
source/text-origin contracts. Strict staged gate remains required; general
floating-prefix/nonfinite parity stays pending.


The current Surface slice adds contact masks and ODE/Bullet/torsional friction
metadata to collisions, with optional provenance on each group. No physics engine
or new dependency is introduced. Explicit group presence survives defaults so the
simulator can reject unsupported surfaces atomically. Native30-case field/default/
error corpus matches; one native-accepted numeric-prefix observation remains
explicitly pending with the existing scalar lexical gap. Native empty-mask65535
versus absent-mask255, 16-bit wrapping, allowed negative slip/rolling friction and
required/optional whitespace controls are retained.

Ten original Surface default/set cases pass natively. New Rust assertion ports
preserve every original value and null-Element provenance through optional Source,
using owned records/public fields instead of a C++ object facade. Original2020
notices and component tolerances are retained. All 75 Rust tests,11 tooling tests,
fmt/clippy and 139+111+72+30 native observations pass. Coverage 2146/2170 lines,
3712/3835 regions, changed 137/139. Independent review and strict staged gate remain
pending. Full Shapes still requires heightmaps, polylines and rendering properties;
other Surface/schema/serialization, primitive edge and numeric lexical gaps remain.


Sensor frame metadata candidate preserves canonical frame_id and literal optional
gz_frame_id, including explicit empty legacy values. Thirty observations against
the pinned native FrameId and Element getters cover both 1.6/1.12, absence,
canonical precedence inputs, normal/CDATA whitespace, NBSP, first text and NUL.
Both strings remain metadata; consumer fallback/scoped-name policy stays outside
the parser. Unknown namespace extensions and nested content remain Unsupported.
The 64-test Rust working tree passes fmt/clippy and 2010/2031 line coverage,
3483/3593 regions with 12/12 changed executable lines. No original full DOMSensor
case is retired; no dependency, exclusion or gate policy changed. Independent
review and a matching frozen staged gate are required before publication.


Sensor topic follow-up replaces raw Unicode trim with existing native string
normalization and maps __default__ to empty, as native Sensor::Load does.
Twenty-four observations across 1.6/1.12 cover empty/default/NUL/text boundaries,
ordinary C whitespace, CDATA and NBSP. Unknown topic leaf attributes/children
remain explicitly Unsupported. Working 66 Rust tests, fmt/clippy and coverage
2012/2033 lines,3483/3593 regions with 4/4 changed lines pass. No dependency,
exclusion or original-case retirement changes. Independent review and strict
staged verification remain required before publication.


Canonical DOM selection candidate adds Model.canonical_link_and_relative_name
without changing attachment graphs. Seventeen native controls verify direct-link
precedence, first-nested-only recursion including empty/static models, explicit
scoped links and frame aliases returning None with unchanged names. The existing
LoadNestedExplicitCanonicalLink port additionally checks direct selected-link
pointer identity and local name; its original native case passed again. Integrated 68-test working
coverage is 2041/2062 lines,3524/3634 regions and 27/27 changed executable lines.
Independent review and the exact frozen staged gate are still required. Interface
models/links and flattened names remain outside the supported subset.


Scalar conversion candidate is qualified only for Linux GNU/glibc. Native
Param scalar doubles and Pose tokens use std::stod; the narrow owned C-locale
boundary preserves finite prefix/hex values, signed zero, exact subnormals,
inexact underflow/overflow and conversion failures. Vector streams retain their
distinct path. Scalar boolean coercion is separate from raw Pose tokens. The 63
native observations contain 61 matching finite/error controls and 2 explicitly
retained nonfinite DOM gaps; no full original case is retired.

Working 72 Rust tests and fmt/clippy pass, including an 8-thread errno stress,
unchanged-global-locale check and isolated C fault-injection child proving
allocation failure returns Resource without freeing null and restores errno.
All three C ABI signatures compile-check against installed glibc headers; source,
header/runtime/library/probe/corpus hashes remain recorded. Coverage is 2067/2089
lines,3580/3691 regions with 36/36 changed executable lines. Dependencies libc and
errno retain their licenses. Cargo deny replaces forbid only with one explicit
production FFI exception, independently reviewed together with the test-only
locale snapshot; no additional unsafe permission is inferred. Independent code,
ABI/safety review and matching strict staged gate are required before acceptance.
Surface remains unaccepted separately; parsing performance claims/optimizations
and portable platform qualification remain pending.


Independent /root/msgs review approved the scalar code, ABI, lifecycle/unsafe
scope and assertion bounds, rerunning all 72 Rust checks,63 native observations,
installed-header ABI compilation and failure/concurrency/locale tests. The final
receipt binds the exact metadata snapshot; publication still requires its
matching strict frozen staged gate. Surface acceptance remains separate.


Surface correction candidate now uses the independently reviewed and published
Linux GNU scalar boundary fbdf7f02. The same formerly unsupported 0.6tail input
positively asserts native 0.6. Forty-nine actual native Surface observations cover
all prior 30 cases plus exact/inexact subnormal, rounded-normal, signed-zero and
scalar boolean controls. Numeric coefficient IEEE bits avoid fixture decimal
rounding ambiguity. All 10 original value/assertion ports are unchanged; their
source/default assertions were independently compared by /root/transport, who
also found the now-corrected scalar underflow gap. Final combined review is
pending /root; no Surface acceptance or physics implementation is claimed yet.


The final Surface working tree passes all 85 Rust tests and fmt/clippy checks.
Coverage is 2216/2241 production lines and 3837/3961 regions, with 137/139 changed
executable lines covered. All 49 native observations pass. This is working-tree
evidence; independent review and the matching frozen staged gate remain required.


Independent /root review replayed all 85 Rust checks, 49 native observations and
all 10 mapped native originals, and checked the source, schema, defaults,
provenance and overlap with the earlier transport assertion review. The native
probe exactly matches the recorded corpus. The 10 Surface ports are now reviewed;
total mappings are 30 complete and one partial out of 1344 runtime cases. The
receipt must bind this final metadata and the strict staged gate must pass before
publication. Nonfinite and vector-prefix gaps, unsupported Surface fields and
explicit simulator rejection remain unchanged.


Include loading candidate retains include-level plugins and original provenance.
Model.include records optional overrides, URI, source and exact span; expanded
plugins retain resource-before-include ordering and duplicate names. Optional
plugin names and native attribute trimming are supported. Nested resolver parent
identities and metadata ownership are tested without library IO.

The 61 fresh native observations contain 59 matching parse/metadata cases and
two explicit native-exception extensions preserving the previously tested empty
placement/default-static contracts. Thirty CR/LF pattern controls bind native
left-to-right normalization for URI CDATA and plugin attributes. All 89 Rust
tests, fmt/clippy and working coverage pass: 2279/2296 lines, 3942/4046 regions,
77/77 changed executable lines. No dependency, unsafe or gate exclusions changed.

Both exact native IncludeModelWithPlugin and IncludeElements originals pass.
Only the former parsing/metadata/plugin observations receive a new partial
mapping; ToElement/formatting/path assertions remain pending. IncludeElements
uses an original version 1.5 box resource, so its full Rust mapping remains
pending rather than rewriting that fixture or claiming old-version support.
Independent review and matching strict staged verification remain required.

Independent include review replayed all 89 Rust tests, 61 freshly rebuilt native
observations and both exact native originals successfully. A supplementary
whitespace-only CDATA URI probe identified different invalid-input callback
ordering: native calls its resolver before rejection; this Result loader rejects
before lookup. Accepted-row callback text/provenance equivalence is qualified;
invalid-input resolver side effects are an explicit remaining boundary. Source,
header, resource, original binary and all shared proof hashes were independently
verified by /root/msgs. The exact final metadata receipt and matching strict
staged gate remain required.

The include parsing/plugin mapping is now independently reviewed as partial.
Total mappings remain 30 complete and two partial out of 1344 runtime cases;
no complete case or SDF packet is retired by this increment. Final publication
requires the matching protected receipt, strict staged gate and hosted checks.

The basic primitive correction closes measured Box/Sphere/Cylinder/Plane DOM
fallback and normalization gaps. Thirty-seven fresh native observations match
exact numeric bits, including signed zero, tiny normals, the normalization
threshold, overflowed lengths, absent defaults and explicit required-empty errors.
Previously rejected negative Sphere and zero/large Plane inputs remain explicit
positive regressions, including a visual Sphere. No original assertion was
removed. Four unchanged native Load cases pass; only their supported value
observations are mapped partially because mutable Element, null/wrong-tag,
schema-description and accumulated-error contracts remain unported.

All 91 Rust tests, fmt/clippy and working coverage pass: 2209/2234 production
lines, 3836/3955 regions and 20/20 changed executable lines. No dependencies,
unsafe code, exclusions or quality thresholds changed. Independent review and the
matching strict staged gate remain required before acceptance or publication.

Independent /root/transport_recovery compared the primitive source, normalization
rules and all four original Load value subsets. It reran all 91 Rust checks,
37 fresh native bit observations and four unchanged native Load cases; all pass
and native proof hashes match. These four mappings are reviewed partial ports,
with no new complete case retirement. Its isolated receipt does not authorize a
changed integrated snapshot; overlap review and the strict staged gate remain
required before publication.

The unchanged primitive correction is integrated onto reviewed include commit
8db5770. All 95 Rust tests, fmt/clippy and working coverage pass: 2272/2289 lines,
3941/4040 regions and 20/20 changed executable lines. The fresh 37-case native
proof equals the isolated reviewed proof, including all source/header/binary
hashes. Include behavior/mapping remains intact; total mappings are 30 complete
and six reviewed partials. Exact integration review, receipt and normal staged
gate remain required before publication.

Independent /root/transport_recovery approved the final integrated overlap:
production/assertion changes and all four new files match its isolated review,
and the fresh native proof equals its independently rebuilt proof. All 95
integrated Rust tests pass independently. The exact qualified receipt preserves
that scope; coverage is author-reported and matching staged/hosted gates remain
required. The four new Load mappings remain partial.

The bounded SDF 1.5 model subset now loads the exact original IncludeElements
raw-string input and original box/model resources without rewriting their version
or content. It applies the already supported 1.6-to-current pose, axis and legacy
name rules. Former version-1.5 negative inputs remain explicit positive checks;
empty roots remain rejected as Missing and version 1.4 remains Version.
The two changed 1.5-to-1.6 conversion families remain explicit Unsupported:
physics-contained gravity/magnetic-field moves and legacy IMU noise conversion.
Two fresh native-supported controls record these gaps separately from passing
parity rows, so no complete 1.5 conversion claim is made.

All 97 Rust checks and fmt/clippy pass. Working coverage is 2275/2291 lines,
3947/4045 regions and 16/16 changed executable lines. The native differential
corpus has 201 matching observations, retaining the exact previous 139 prefix
and adding 62 version-1.5 conversion-chain controls. The unchanged native
IncludeElements original passes once with zero failures. The proposed complete
mapping preserves its lookup paths, Option/presence observations, both original
resources and native Pose3 component tolerance. Independent assertion/source
review and matching strict staged verification remain required; no packet is
complete and older/full conversion remains pending.

Independent /root/msgs compared the full original IncludeElements assertion body
and conversion chain and freshly reproduced the native proof, including every
shared hash, all 201 rows, both explicit conversion obligations and the exact
original case. All 97 Rust tests pass independently using the existing default
target; no independent clean build or coverage rerun is claimed. The complete
IncludeElements mapping is reviewed: 31 complete and six partial mappings total.
Exact final receipt and normal strict staged/hosted gates remain required.

The isolated fluid-added-mass candidate retains optional boxed symmetric 6x6
metadata and the unchanged inertial_complete.sdf fixture. All 40 fresh native
observations match presence and exact coefficient bits, including every XML
entry, empty/default distinctions, signed zero, finite negative values,
scalar-prefix/coercion behavior and range/nonfinite errors. All 98 Rust checks,
fmt/clippy and working coverage pass: 2289/2306 lines, 3999/4096 regions and
16/16 changed executable lines. No unsafe, dependencies or exclusions changed.

The exact native DOMLink.InertialComplete case passes once with zero failures.
Its proposed Rust mapping retains mass, all six rigid inertia values, three
translation values, fluid presence and all 36 matrix coefficients. The native
MassMatrix.IsValid observation maps to successful mandatory physical-inertia
validation during loading, also retaining the exact known-valid values. A
reviewer must explicitly judge that API mapping; no complete port is accepted
before review. Consumers must reject any explicit fluid matrix until engine
support; no fluid force or full inertia/serialization claim is made.

Independent /root/transport_recovery approved the isolated fluid source, schema
and complete original assertion mapping, including the fixture-specific IsValid
observation via mandatory load-time validation and retained exact known-valid
mass/inertia values. It independently passed all 98 Rust tests, 40 freshly
rebuilt native bit/presence observations and the exact original case; proof
hashes matched. This does not claim a generic validity algorithm or physics API.

The identical fluid delta is integrated onto version-1.5 commit 9222bd6. All
100 Rust checks and fmt/clippy pass; working coverage is 2292/2308 lines and
4005/4101 regions, with 16/16 changed executable lines. The fresh native proof
is identical to the isolated reviewed proof. Total proposed reviewed mappings
are 32 complete and six partial; final overlap review, exact receipt and strict
staged/hosted checks remain required. Every explicit fluid matrix still needs
consumer rejection until physics support is implemented.

Independent /root/transport_recovery approved the integrated fluid overlap and
final metadata, verifying identical parser/type changes and all five new files
against the isolated review. The fresh native proof matches, and all 100 Rust
checks pass independently. Its qualified receipt retains the fixture-specific
validity mapping and mandatory consumer rejection until physics support. Exact
staged and hosted gates remain required before publication acceptance.


The isolated visual-property slice preserves clonable owned cast-shadow,
visibility-mask and laser-retro metadata, including absent versus explicit zero.
Transparency follows the native double Param's float getter: a 17-digit decimal
round trip and GNU float conversion, with a second-conversion range error keeping
the zero getter default. Direct double-to-float casts fail the retained midpoint
regression. The Linux GNU unsafe boundary adds only strtof_l; no production dependency,
exclusion or unsafe-lint scope changes. Independent code, ABI and safety review
is required before publication.

All 104 Rust checks and fmt/clippy pass. Working coverage is 2344/2360 production
lines and 4089/4185 regions, with 51/51 changed executable lines. The freshly
compiled C++ probe records 143 observations: 141 matching finite/error outcomes
and two explicit native-accepted nonfinite DOM gaps retained under the existing
finite scalar policy. The installed-header ABI check passes. Concurrent errno,
unchanged global locale and isolated second-locale allocation-failure checks
cover ownership and cleanup. Native console warning bytes/side effects, other
rounding environments, mutable Visual setter/serializer APIs and rendering/lidar
effects remain unqualified. No original case is retired by this slice; full
Shapes still needs heightmap/polyline metadata and assertion review.


Independent visual review found a test-artifact isolation defect: both scalar
and visual fault tests wrote shared objects under source/target. Both now use
maintained test-only tempfile 3.27.0 directories, kept alive until the child
completes and removed by RAII. All allocation-failure and errno assertions are
unchanged. Both fault tests pass in a fresh source export with an external Cargo target;
source/target remains absent. All 104 Rust checks, fmt/clippy and 11 tooling
checks pass after correction. Final independent replay and review receipt remain
required; production conversion bytes are unchanged.


Independent /root/msgs approved the corrected visual source, ABI, lifetime and
assertion scope. It freshly reproduced all 143 native observations and the
installed-header ABI proof, with exact shared hashes. All 104 Rust tests passed
independently before the test-path correction with identical production bytes;
both corrected fault tests then passed in a fresh raw export using a new external
target, without source/target creation. Its review explicitly covers the new
strtof_l boundary and dev-only tempfile correction. Final exact receipt and
strict staged/hosted verification remain required before publication.


The next isolated terrain slice adds owned heightmap/polyline metadata and loads
the unchanged full upstream shapes.sdf fixture. The proposed complete
DOMGeometry.Shapes port retains every named lookup, enum/presence, primitive,
mesh, heightmap and polyline assertion, including native Vector3 1e-3, Vector2 1e-6 and scalar
four-ULP comparisons. It is pending independent assertion review; the 32 accepted
complete and six accepted partial mappings do not yet change.

All 108 combined Rust checks and fmt/clippy pass. Working coverage is 2440/2456
production lines and 4289/4389 regions, with 142/142 changed executable lines
relative to c75839f, including the pending visual parent. Fresh native evidence
contains 52 matching terrain observations and one exact Shapes original with
zero failures. Required schema-default insertion, explicit-empty errors, ordered
textures/blends/points, unconstrained finite dimensions and imported resource
identity are retained. No asset resolution, image decoding, collision or rendering
effect is implemented. Consumers must explicitly reject heightmap/polyline
collisions until physical support. Source/assertion/provenance review and the
separate visual ABI/safety review must precede normal staged/hosted publication.


Independent /root/transport_recovery approved the corrected terrain source and
complete Shapes assertion mapping after finding and correcting the original
Vector2 tolerance. All 108 combined Rust checks, 52 fresh native observations
and the exact unchanged original case pass independently; all proof hashes
match. Owned lookups, enum variants/index collections and the canonical mesh
optimization spelling preserve this original case. The visual parent safety
review remains separately attributed to /root/msgs. This unchanged terrain
delta is now integrated on visual commit 0644c0f; final overlap review and exact
staged/hosted gates remain required. No physical terrain or rendering effect is
implemented, and the new Shapes mapping does not complete a plan packet.


On visual parent 0644c0f, all 108 integrated Rust checks and fmt/clippy pass.
Working coverage remains 2440/2456 lines and 4289/4389 regions; the terrain-only
changed-line delta is 91/91. Fresh native proof equals the independently reviewed
isolated proof, including every shared source, fixture, header and binary hash.
Visual tempfile fixes are inherited unchanged; no new production unsafe or
dependencies are added by terrain. The full Shapes assertion mapping is now
reviewed (33 complete and six partial mappings), while final integration receipt
and the normal strict staged/hosted gates remain required before publication.


The isolated model-scope merge slice flattens imported members through native
proxy frames, validates imported graph scope before flattening, and retains owned
merged-include snapshots/source identities. Placement, nested merges/models,
canonical attachment, direct-reference/axis rewrites, plugin order and deferred
non-frame behavior match the bounded native corpus. Literal spaced true words
produce normal nesting, as native does. World merge, three schema-extension
controls and two legacy name collisions remain explicitly Unsupported.

All 112 Rust checks, fmt/clippy and 11 tooling checks pass; working coverage is
2572/2590 production lines, 4534/4639 regions and 152/152 changed executable lines.
Fresh native evidence contains 153 observations: 148 parity rows and five explicit
native-accepted gaps. Both unchanged original native include cases pass, but
their full source resource still has unsupported fields; no case is retired.
The exact old merge=false negative input remains as a positive normal-include
regression, and merge=true with an unresolved URI remains a Resource error.
Independent review and normal exact staged/hosted gates remain required.

Independent review found two false-accept families in the initial candidate.
Placement lookup now uses schema-local scope rules rather than the broader public
query rules. Original resource root poses are validated before include overrides,
so nonempty relative_to values cannot bind to the parent or be hidden by a valid
include pose. Native-backed controls retain all three placement spellings, 12
resource pose/canonical cases, 35 include-override combinations and 35 ordinary
include controls. All 112 Rust tests and the rebuilt 153-row native proof pass;
Clippy and working-tree coverage also pass. The corrected candidate requires
independent re-review and the exact staged/hosted gates before publication.

Independent /root/msgs re-review approved the corrected source and assertions.
All 112 Rust checks passed in a fresh reviewer-owned source/target; the rebuilt
153-row native proof and both exact original native cases matched all shared
hashes. Five explicit gaps remain, and no original mapping is newly retired.
The unchanged candidate is integrated onto public terrain commit c4e5d627;
matching exact receipt, staged gate and hosted CI remain required.

Pure model.config selection is implemented in an isolated candidate on public
terrain commit c4e5d627. The new owned API returns filename/version metadata and
ordered source-located warnings; it has no filesystem, Fuel, dependency or unsafe
addition. Native first-tag fallback and the nonstandard gz-math version parser
are retained. Numeric native exceptions become typed errors, and warning creation
with missing name metadata is safe rather than reproducing native exceptions or
a null dereference. These new API boundaries are explicit in MODEL_CONFIG.md.

All 112 isolated Rust tests pass, including two filename-only partial original
ports; no complete upstream case is retired. Eleven tooling tests, fmt and Clippy
pass. Working coverage is 2568/2584 lines, 4492/4592 regions and 117/117 changed
executable lines. The freshly rebuilt probe matches all 89 native observations,
with safe warning/error boundaries identified separately; two exact native
originals pass. Sources, math headers, libraries, corpus and immutable original
fixtures are bound in reference/results.json. Independent review, integration
overlap and strict staged/hosted checks remain required before publication.

Independent config review reproduced all baseline tests/proof, then identified an
overly broad test-only crash allowance and inherited XML cursor gaps. The allowance
is now limited to two named native warning failures, with exact filename/version/
warning assertions; unrelated native failures fail the corpus test. Eight reviewer
XML controls are retained, including five accepted-native grammar gaps (extra roots,
leading text and DOCTYPE) that Rust explicitly rejects. No production source changed.
The revised corpus has 97 observations and binds libc/libstdc++ plus installed
parser-header identity. Native proof/two originals and all isolated Rust checks pass;
working coverage remains unchanged. Corrected independent review is pending.

Independent /root/msgs approved the corrected config source/assertions/proof and
both filename-only partial mappings. Production/API/Cargo bytes stayed unchanged;
initial full 112 Rust checks and corrected four config checks/97 native observations/
two originals passed independently. The unchanged delta is integrated onto public
merge fd6d597, preserving all prior merge code/proof. Author integrated replay passes
116 Rust checks and exact fresh native proof; coverage is 2700/2718 lines,
4737/4842 regions and 117/117 changed executable lines. Mapping totals are 33
complete and eight partial; no new full case is retired. Final integration receipt
and strict staged/hosted gates remain required.

An isolated SDF 1.4 common-subset candidate applies the native direct-root/world
axis default insertion, preserving nested and per-resource version scope. Actor
renaming, earlier source versions and Element/serializer/conversion interfaces
remain unsupported. The former simple 1.4 negative input is retained as an exact
positive regression; a 1.3 floor check remains.

All 115 isolated Rust checks, 61 native observations and both exact native
missing-flag originals pass. Two filename-independent typed/axis portions are
recorded as partial mappings; converted Element assertions remain pending.
Original 2014 notices are retained. Working coverage passes 2589/2606 lines,
4569/4673 regions and 24/24 changed executable lines; fmt, Clippy and 11 tooling
checks pass. No dependency or unsafe changes. This candidate is on merge commit
fd6d597 and must integrate with published config commit 80b6c74 without losing
that API/proof. Independent review and strict staged/hosted checks remain required.

Independent /root/transport_recovery approved the isolated 1.4 source/assertion
scope, all 115 Rust checks and fresh 61 native observations/two exact originals,
with complete proof equality. Both mappings remain partial. The unchanged delta
is now integrated onto public config 80b6c74; config source/API/Cargo and proof
remain intact. Author integrated replay passes 119 Rust checks, 2717/2734 lines,
4772/4876 regions and 24/24 changed executable lines; fresh native proof exactly
matches the independently verified isolated proof. Final overlap/receipt and
normal strict staged/hosted gates remain required.

## Staged Git context correction

A linked-worktree hook exports repository-local Git environment variables. The
staged gate now clears exactly the keys reported by git rev-parse --local-env-vars
around exported verification and restores the authoritative parent's environment
on success or failure. Nonlocal settings, including GIT_SSH_COMMAND, are preserved.
The exact raw-blob export, index/HEAD guards, coverage and assertion policies stay.

A real commit hook regression uses an absolute worktree Git directory and a
separate intended index whose content differs from the normal index. Its child
creates and commits an independent fixture repository, checks the intended
snapshot, and proves the parent config, normal index and main HEAD are untouched.
The same regression fails with the original verifier because the child inherits
Git locals. A second test covers restoration after a failed child. All 13 tooling
checks pass; source/native assertions and thresholds are unchanged. Independent
review and the normal strict staged/hosted gates remain required.

## World physics profile candidate

World now retains physics profiles and exposes native first-default/first-profile
selection. Physics owns core step/rate/contact fields, unconstrained engine string,
name/default flag and source/raw XML; missing declarations preserve native effective
schema defaults. No solver/backend selection, engine-specific parameters or runtime
policy is implemented. Signed maximum-contact conversion follows measured native
stoi prefix/range behavior without another FFI boundary.

All 122 Rust checks, fresh 89 native observations and the unchanged original
DOMPhysics.DefaultConstruction pass. Its complete value/default/setter assertions
are retained through public owned fields; source/Element/serializer scope remains
explicit. Independent /root approved its complete value/default/setter assertion mapping
and reproduced all 122 Rust tests, 89 native observations, the exact original
(one run, zero failures), complete proof equality and 15 additional edge controls.
The identical source/assertion delta is integrated onto the published hook fix.
Working isolated coverage is
2857/2875 production lines, 4974/5080 regions and 135/135 changed executable lines.
Existing nonfinite-double limits and engine-specific/custom Unsupported gaps are
recorded, with no dependencies, thresholds or coverage exclusions changed. Strict
staged/hosted verification and independent assertion review remain required.

## Material metadata candidate

The isolated candidate adds owned optional Material/MaterialScript metadata to
Visual: RGBA colors, shininess, render order, lighting/double-sided flags, shader,
normal-map string and ordered script URI/name data. No resource IO, renderer or
PBR implementation is added. Native missing/present defaults and first-script-URI
validation are retained. The existing strtof_l boundary is refactored into a raw
float helper for direct native color/render-order conversion; stored-double visual
transparency still uses the prior 17-significant-digit conversion and zero fallback.
This change requires a fresh independent numeric safety review.

All 124 Rust checks, 11 tooling checks, fmt and clippy pass. Fresh native evidence
contains 110 exact-bit/error rows and the unchanged DOMVisual.Material original
(one run, zero failures), with installed-header ABI/source/library hashes. Four
explicit gaps cover PBR, a custom scalar child and existing nonfinite shininess.
Allocation-failure cleanup, errno preservation and concurrent parsing are tested;
existing scalar/visual boundary tests remain. The complete original material.sdf
and its presence/color/float/shader/script assertions are ported, retaining the
original 2018 notice, color 1e-6 bounds and four-ULP EXPECT_FLOAT_EQ relation.
Independent /root/msgs approved the complete original mapping and numeric safety
review, replaying all 124 owned Rust tests plus an independent string-edge test,
110 fresh native rows, the exact original (one run, zero failures) and ABI proof.

Working-tree coverage is 2892/2911 production lines and 5073/5189 regions, with
168/169 changed executable lines. No exclusions, dependencies or gate thresholds
changed. Former empty-material rejection inputs remain as explicit native-positive
default-material regressions. Full native schema, setter/Element/error-vector
contracts, StoreResolvedURIs, renderer/lidar effects, non-GNU platforms, other
rounding modes and performance claims remain pending. Exact frozen staged and
hosted verification are still required; no SDF-01/02 packet is self-accepted.


Independent material review found six real script-name mismatches from the generic
Unicode-trim helper. That field now uses the existing Param text adapter with
explicit missing/default rejection. All eight exact reviewer inputs (six failures
and two matching controls) are appended to the native corpus without changing the
original 102 rows. The numeric ABI/refactor bytes and original fixture assertions
are unchanged. Independent correction review passes; all six mismatches are resolved. The
unchanged material source/assertion delta is integrated separately onto the
reviewed world physics and hook parent; exact overlap/staged/hosted review remains.

The integrated candidate on public world-physics/hook parent 7ed5ad02 passes all
127 owned Rust checks, 13 tooling tests, fmt/clippy and fresh 110 native rows plus
the exact original and ABI check. The entire native proof equals the reviewed
isolated proof. Integrated coverage is 3032/3052 lines, 5275/5393 regions and
168/169 changed executable lines. Existing physics source/mapping/proof and hook
policy remain untouched. Final overlap receipt and mandatory staged/hosted gates
remain required; accepted original mappings now total 35 complete and ten partial.

## Joint axis dynamics candidate

The isolated successor retains damping, friction, spring reference/stiffness and
independent joint-stop stiffness/dissipation on both axes. Native defaults, required
empty errors, negative/signed-zero/prefix/range behavior and owned declaration
source/XML are tested. No runtime joint effect, FFI change, mimic or initial-position
support is added. Two existing manual Axis fixtures gained the new default fields;
their original frame/query assertions and values are unchanged. Initial compilation
failures from these new required struct fields are retained.

All 125 Rust checks, 13 tooling checks, fmt/clippy, fresh 68 native observations and
the exact original DOMJointAxis.Construction (one run, zero failures) pass. Only
twelve dynamics/stop default/setter observations are proposed as a partial mapping;
Element, XYZ/error/graph, bounds/effort/velocity and mimic assertions remain pending.
The complete original case is not retired. Native nonfinite-double and custom-child
gaps remain explicit. No dependency, exclusion or threshold changes are made.

An initial changed-line gate correctly rejected 32/34 coverage. Added native-backed
custom-child rejection cases exercise both new outer validation paths; all original
66 rows are retained. Final working coverage is 2883/2901 production lines,
5022/5127 regions and 34/34 changed executable lines. Independent review and exact
staged/hosted verification remain required; no whole SDF packet is self-accepted.

Independent /root/transport_recovery approved the isolated twelve-value partial
mapping after all 125 Rust checks, 68 fresh native observations and the complete
original (one run, zero failures), with entire native proof equality. Original
frame/query assertions are unchanged after the six initializer additions. The
identical axis source/assertion delta is integrated separately onto public material
01464a42; final overlap/staged/hosted qualification is required.
Integrated author coverage is 3058/3078 lines, 5323/5440 regions and 34/34 changed
executable lines, with 130 Rust and 13 tooling checks passing. This does not expand
the twelve-value partial original mapping; totals remain 35 complete and eleven
partial cases pending the final exact review receipt and publication gates.

## Link properties and unchanged pendulum candidate

The isolated successor retains four link flags with native defaults and adds a
borrowed Joint::axis(index) lookup. No simulator, Fuel, solver or FFI code changes.
The unchanged double_pendulum.sdf fixture now parses, supporting proposed complete
DOMLink.InertialDoublePendulum and DOMJoint.DoublePendulum ports. All original
lookups, pose/vector 1e-3 comparisons without sign relaxation, four-ULP scalar
comparisons, raw relative strings, flags, inertia fields and repeated base fluid
absence remain. The known-valid fixture IsValid mapping explicitly awaits review;
there is no generic mass-matrix validity parity claim or articulated simulation.

All 135 Rust checks, 13 tools, fresh 56 native flag rows and both exact unchanged
native cases (one run, zero failures each) pass. Source/schema/header/math/fixture
and native binary/library hashes are bound. Corrected working coverage is 3079/3099 lines,
5353/5470 regions and 21/21 changed executable lines. Clippy initially objected to
the original 1.5708 literal; a local test-only allowance preserves that exact input
and its original comparison. No old assertion, gate threshold or exclusion changes.
Independent source/assertion review and exact staged/hosted gates remain required.

Independent review caught an unpublished accessor assumption: native Joint::Axis
clamps all positive indices to its second axis. The helper now preserves that
convention in Rust's usize domain; the original absent-axis checks stay unchanged.
The exact reviewer universal-joint XML and an absent-second-axis control are added
after all original 54 rows. Stored-reference identities at 1, 2 and usize::MAX are
asserted. This correction does not change physical flag metadata or the pending
known-fixture validity mapping. Independent correction review remains required.

Independent /root/msgs approved both complete original mappings after fresh 135
Rust checks, 56 native observations and both unchanged originals, with full proof
equality. The known-fixture IsValid observation is explicitly approved through
mandatory validation plus the exact mass 100/identity inertia; general validity
parity and physical effects remain unclaimed. Corrected source/assertions/corpus
are unchanged by this review-metadata update. Normal staged and hosted gates
remain required.


The isolated scoped DOM successor exposes borrowed Model.model_by_name and
link_by_name, retaining explicit relative nested scopes and actual object identity.
The existing canonical traversal is reused. Proposed full original NestedModel
and LoadDoublePendulum ports preserve unchanged fixtures and all original
lookups/count/index/name/pose assertions. The known-valid pendulum's final graph
validation observation is mapped to mandatory successful eager validation only;
independent review must explicitly judge that mapping, with partial disposition
available. No general mutable graph-validation API or runtime behavior is added.

All 138 Rust checks, 13 tools, fmt/clippy, fresh 36 native pointer-path observations
and both exact originals pass. Coverage is 3087/3107 lines, 5373/5490 regions and
10/10 changed executable lines. A test-only lint allowance preserves original
index-zero presence assertions; the initial lint failure is retained. Reference
source/header/library/binary/fixture provenance is bound. Independent review and
normal staged/hosted gates remain pending.

Independent /root/msgs approved both complete original mappings after all 138
Rust checks and fresh 36-row/two-original proof matched in a new private snapshot.
Every original assertion and unchanged fixture was compared. The known valid
pendulum's graph-validity observation is explicitly approved through mandatory
successful validation of both graph types; generic mutable revalidation and
error-vector parity remain unclaimed. Source/assertion/native proof bytes are
unchanged by the review metadata. Normal staged/hosted gates remain required.


The isolated member-DOM successor adds borrowed joint/frame lookup and shared
native member-scope selection. The original 36 model/link observations remain
unchanged; 104 fresh native rows also cover scoped kind-specific references,
implicit-frame absence and exact literal-name fallback after owned field edits.
No parser namespace, semantic graph or runtime behavior is relaxed. The unchanged
MultiNestedModel fixture has a proposed full assertion port, including actual
stored pointer identity and seven specific graph-membership observations. The
latter mapping requires explicit independent judgment, not a generic graph facade.

All 140 Rust checks, 13 tools, fmt/clippy, 104 native rows and the exact original
(one run/zero failures) pass. Coverage is 3099/3119 lines, 5398/5515 regions and
16/16 changed executable lines. An initial new-test enum spelling compilation
failure is retained; its actual UnknownReference contract now passes. Dependencies,
unsafe scope, existing assertions and quality policy are unchanged. Independent
review and exact normal staged/hosted gates remain required.

Independent /root/msgs approved the complete MultiNestedModel port after all 140
Rust checks and fresh 104-row/exact-original native proof matched, including all
hashes and unchanged prior 36-row prefix. Scoped stored-reference identity and
native prefix/fallback precedence were inspected. The seven known-fixture graph
membership observations are explicitly approved through exact attachment lookup;
general mutable membership APIs remain unclaimed. This metadata update changes
no production, assertion, fixture or native input. Normal staged/hosted gates
remain required.


The isolated axis-limit load successor removes the extra ordering rejection and
retains exact finite values. Native controls also exposed incorrect Rust omitted
bounds (+/-1e16 instead of native Root's converted-schema infinities) and silent
explicit-empty acceptance. Both are corrected with existing scalar helpers. Two
obsolete Rust-only negative XMLs remain exact native-positive value regressions;
unrelated assertion bytes are proven unchanged. No limit execution, normalization
or backend joint admissibility is added. The earlier AXIS_DYNAMICS parsed-default
prose is corrected; its twelve-value partial mapping remains unchanged.

All 142 Rust checks, 13 tools, fmt/clippy, 160 native rows and the unchanged full
Construction baseline pass. Two explicit nonfinite-input gaps remain; implicit
unbounded defaults are separately accepted native metadata. Coverage is 3092/3112
lines, 5395/5510 regions and 2/2 changed executable lines. Runtime embedded-schema
bytes and pinned generator framing are verified. Initial old-guard/default and
provenance-setup failures are retained. Independent source/disposition review and
normal staged/hosted gates remain required; no new original retirement.

Independent /root/transport_recovery approved the bounded correction and explicit
obsolete-test disposition after fresh 142 Rust checks, 160 native rows and the
complete unchanged Construction baseline passed with entire proof equality.
Exact old XML/helper wrappers and restored unrelated assertion bytes match the
public parent. Runtime embedded schema framing is verified; implicit unlimited
metadata and explicit nonfinite gaps stay separate. Source/assertions/native inputs
are unchanged by this metadata update. Normal staged/hosted gates remain required.


## Owned joint configuration cloning candidate

Three derived Clone implementations on JointKind, Joint and Axis allow consumers
to retain typed configuration without duplicating the producer DOM. A consumer
regression drops the parsed document, retains both axes/dynamics/source/XML and
mutates a second clone without changing the retained configuration. No loading,
normalization, native assertion, dependency or runtime behavior changes. Author
checks pass 143 Rust and 13 tooling tests with unchanged 3092/3112 production lines
and 5395/5510 regions; there are no changed executable production lines. Independent review passed the authored consumer regression and two additional
controls for fixed joints without axes and retained revolute unbounded values with
independent UTF-8 source cloning. Runtime graph resolution remains Sdf-owned;
Clone retains configuration only. The normal exact staged and hosted gates remain
required before publication.


## Contact sensor configuration candidate

Isolated work adds an owned SensorConfig enum with boxed IMU or Contact settings,
optional typed accessors, exact selected XML and clone/source ownership. Contact
collision names preserve order and duplicates; nested topic remains distinct
from envelope topic. No dummy IMU, runtime scheduling or contact effect is added.
Existing negative IMU-rate rejection remains; native finite negative contact
rates are preserved as metadata. The complete original Sensors case passes
natively; Rust adds only the exact contact subtree assertions to the prior IMU
partial mapping. Other families and whole fixture assertions remain pending.
The native 85-row corpus has 74 supported parity rows and 11 precisely named
policy/schema gaps. Independent source/assertion review passed; exact staged and
hosted gates remain required. No whole SDF packet acceptance is claimed.

Author checks pass 150 integrated Rust tests, 13 tooling tests, formatting and
strict Clippy.
Production coverage is 3160/3181 lines and 5519/5631 regions; changed executable
lines are 77/78, above the strict threshold. The complete native Sensors original
and all 85 differential observations pass. These are working-tree measurements;
independent review is complete and the normal exact staged gate remains pending.

Self-review found and fixed premature IMU configuration evaluation in the new
enum path. Four exact diagnostic controls from a fresh public-parent consumer
preserve envelope name/pose/boolean precedence, including code, source position
and message. The new test failed before correction and passes after restoring
the original evaluation order. The earlier 81 native observations are unchanged;
four additional invalid-envelope controls now also bind their native outcomes.

Contact integration onto public Clone 4ed690b3 retains its three derives, owned
consumer test and contract text. Fresh integrated author replay passes all 150
Rust checks, 13 tools and the complete 85-row/original native proof; every native
proof field equals the corrected isolated result. Coverage remains 3160/3181
lines and 5519/5631 regions, with 77/78 changed executable lines.

Independent /root/transport_fallback approved the exact Contact source and partial
assertions after 150 fresh private Rust checks, 13 tools, formatting/Clippy, all
85 native rows and the full Sensors original. A separate public-parent build
reproduced all four old IMU diagnostics exactly. Source, XML ownership and native
pose tolerance were reviewed; no full case retirement or runtime effect is
claimed. The reviewer also detected only a post-coverage quality-metadata digest
drift; fresh final-byte author checks corrected that binding. Final metadata,
normal exact staged hook and hosted qualification remain required.


## Joint endpoint original assertion candidate

Three exact original world/model-frame endpoint cases are retained through the
existing Root collections, borrowed DOM and semantic attachment/pose queries.
Original fixture bytes and component-wise native pose tolerances are unchanged.
All three focused Rust tests and all three exact native originals pass, with no
skips and pinned source/header/fixture/library/binary evidence. All author checks
and independent assertion/API-translation review passed.
Normal staged/hosted qualification remains pending; no runtime joint support or
whole packet acceptance is claimed.

Fresh integrated author checks pass 153 Rust tests, 13 tooling tests, formatting
and strict Clippy. All three exact native originals pass with zero skips and
unchanged proof. Production coverage remains 3160/3181 lines and 5519/5631
regions; no executable production line changed. The prior three native cases
and inherited Contact/Clone source and evidence remain intact. Independent
assertion/API-translation review passed; normal staged/hosted gates are pending.

Independent /root/transport_fallback passed all 153 Rust checks, 13 tools and
formatting/Clippy in a fresh private source/target, then reran each unchanged
native original with one actual test and zero failures/skips. Complete proof and
all source/header/library/fixture/binary hashes match. Every assertion and the
exact valid-fixture endpoint API translation were approved. Three complete
mappings are added, for 43 complete and 11 partial cases; no general mutable
endpoint facade or full joint/schema/runtime completion is claimed.


## Noise required-type and trimming candidate

Actual native Root controls expose an existing false acceptance of explicit noise
without its required type, and false rejection of ASCII-padded valid types. The
bounded parser correction requires that attribute and applies native three-char
trimming before the existing case-insensitive match, preserving Unicode/control
edges. All 126 native cases and both unchanged original baselines pass. Exact old
malformed supplementary positive XML is retained as a negative; explicit none
preserves all positive data/operations, and reversing one literal change restores
the full old test file. Original IMU fixture/assertions and unrelated malformed
number/error cases remain unchanged. Independent disposition review passed; the normal exact staged/hosted gates
remain required. No new original case retirement is claimed.

Author checks pass 156 Rust tests, 13 tooling tests, formatting/Clippy and doc
compilation; coverage is 3165/3186 lines and 5533/5645 regions, with 3/3 changed
executable lines. The native 126-row probe and both exact original baselines pass,
including runtime embedded-schema bytes and full supplementary disposition checks.
Initial new-test column-type compile failures are retained separately from the two
subsequent actual behavioral red tests. No original case retirement, dependency
change or production unsafe boundary change is introduced.

Independent /root/msgs_review_fallback approved source and the exact obsolete
supplementary-positive disposition after 156 fresh private Rust checks and all
126 native rows/both unchanged originals matched the complete authored proof.
Every old value/operator and unrelated error assertion remains; exact original
IMU fixture bytes are unchanged. Native source/schema trim rules and Unicode
nonwhitespace/control boundaries were reviewed. Author tooling/coverage remain
separately qualified; final metadata, normal staged and hosted checks are required.


## ForceTorque and joint sensor candidate

Owned Joint.sensors and SensorConfig::ForceTorque retain ordered envelopes,
frame/direction and six Noise records with exact raw configuration/source ownership.
Joint-default semantic pose resolution and deferred non-frame validation extend to
joint sensors. Seven focused Rust tests and all 96 freshly compiled native rows
pass, including normal/merged imports; three unchanged native baselines each run
once with zero failures/skips. The full DOMJoint.Sensors assertion mapping has independent approval;
mandatory staged/hosted qualification remains pending.
Eight explicit strict-schema/finite-input differences and consumer/runtime limits
remain documented. No new unsafe code, dependencies, sensor effects or whole
SDF packet acceptance is claimed.

Fresh author checks pass all 163 Rust tests, 13 tooling tests, formatting/Clippy
and doc compilation. Coverage is 3239/3260 production lines and 5665/5780
regions, with 89/89 changed executable lines. All 96 native observations and
three original baselines pass with complete pinned source/schema/header/library
proof. The seven added tests retain old IMU assertions and configuration ordering;
no existing assertion or error policy is removed. The candidate original mapping
has independent assertion approval; normal staged/hosted qualification is pending.

Independent /root/transport_fallback approved the frozen configuration/API and
complete Sensors mapping after 163 fresh private Rust checks, 96 native rows and
three unchanged originals. The full proof and original fixture/source/header/
library/binary identities match. Native pose/double bounds, shared IMU defaults
and all four existing precedence diagnostics were checked. Eight explicit policy
gaps and consumer runtime guards remain. This adds one complete mapping, for
44 complete and 11 partial cases, subject to normal staged/hosted qualification.


## Nested implicit model-frame candidate

The unchanged nested-joint fixture exposed a missing terminal ::__model__ graph
alias. A bounded resolver correction accepts only actual Model prefixes and
preserves strict schema scope boundaries and raw DOM strings. Four complete
joint frame/relative-pose/nested endpoint originals are ported as candidates with
all presence/name/count/index/body/pose observations and native component bounds.
All five focused Rust checks and 150 freshly compiled native observations plus
four unchanged originals pass. One inherited static attachment identity/child endpoint policy
remains an explicit difference; no runtime policy or old assertion is removed.
Full author gates and independent source/assertion review pass; current-parent
integration and mandatory staged/hosted qualification remain pending.

Fresh isolated author checks pass all 161 Rust tests, 13 tools, formatting/Clippy
and doc compilation. Coverage is 3182/3198 lines and 5557/5666 regions, with
11/11 changed executable lines. The complete 150-row proof and four exact native
originals pass. This source still uses the earlier public Noise parent; the later
ForceTorque increment must be preserved by a separately checked integration.
Independent assertion/source review passed; normal staged/hosted gates remain pending.

Independent /root/transport_fallback approved all four complete original mappings,
source behavior and scoped body API translation after 161 fresh private Rust checks,
150 native observations/four unchanged originals and complete proof equality.
A separate pre-fix graph mutant independently reproduces the nested fixture failure.
Static attachment identity/child validation remains explicitly different; no runtime
static policy is chosen. Four additional complete mappings bring the integrated
candidate to 48 complete and 11 partial cases, pending exact staged/hosted gates.

Fresh author integration onto public green ForceTorque 18e6fec0 passes all 168
Rust checks and 13 tools, formatting/Clippy and doc compilation. Coverage is
3256/3272 lines and 5689/5801 regions, with 11/11 changed executable lines. The
entire native 150-row/four-original proof matches isolated evidence. The new graph
implementation and all new fixtures/assertions/probe bytes are preserved; existing
ForceTorque/IMU/Contact source, assertions and native evidence remain intact.
Final independent overlap binding and normal staged/hosted checks are required.


## UTF-8 BOM source-span candidate

Two real public-parent failures expose shifted raw XML ranges after an initial
BOM: corrupted plugin XML and a valid Unicode model name causing a UTF-8 boundary
panic. A bounded tokenizer-origin correction preserves original spans/source
coordinates for plugin/configuration/physics/axis/include data and imported
resources without changing tokenizer input or document grammar. Nine focused
Rust tests pass, including full-source position comparisons across 96 layouts;
87 fresh native rows and three unchanged originals pass within three explicit
existing XML grammar gaps. No new original mapping, dependency, unsafe or runtime
change is introduced. Full author checks and independent review pass; staged/hosted qualification
remains pending. The performance cache is a separate prototype.

Fresh author checks pass all 177 Rust tests, 13 tools, formatting/Clippy and doc
compilation. Coverage is 3259/3275 lines and 5694/5806 regions, with all six
changed executable lines covered. The full 87-row/three-original native proof
passes; 84 rows match and three inherited grammar differences remain explicit.
No original mapping or old assertion changes. Independent review passed; normal
staged/hosted qualification remains required.

Independent /root/transport_fallback approved the ten-file BOM correction after
177 fresh private Rust tests, formatting/Clippy and the exact 87-row/three-original
native proof. A separate pre-fix cursor countercontrol reproduced both real failures;
all reviewed bytes pass. Raw/source contracts and the three existing grammar gaps
were checked. No case retirement or cache code is introduced. Exact main metadata
binding and normal staged/hosted gates remain required.


## Source-position cache candidate

The measured full-prefix source scan is replaced by a private lazy lexical-prefix
cache that reuses the same xmlparser routine and keeps the XML error fallback.
No schema/raw-source/API or runtime behavior changes. Existing source assertions
plus 64 generated nested layouts, 128 config warning positions and the public
Send+Sync contract pass. All 179 Rust tests, 13 tools, formatting/Clippy/doc checks
pass; coverage is 3281/3297 lines and 5732/5844 regions, with 24/24 changed lines.
Fresh general/BOM/ForceTorque/model-frame/noise native replays exactly match all
660 published observations and twelve original executions (nine unique cases),
including existing explicitly qualified gaps. No original mapping is retired.

A fresh isolated-build benchmark checks exact full semantic debug bytes outside
timing, then counts/pose/attachment/checksums on every timed load. Seven alternating
CPU-0 pairs show a retained 1000-link/frame median of 68.815 ms before and 9.835 ms
after (6.997 ratio). The opaque-tail median is 0.9% slower with overlapping ranges;
its counterworkload and earlier eager regression remain in raw reports. Shared
host/frequency/packaging and bounded-input limits are explicit. These measurements
do not accept P2, SDF-02 or native full-loader performance. Independent source and
benchmark review passed; exact staged/hosted qualification remains pending.

Independent /root/transport_fallback approved source/fidelity/benchmark scope after
179 fresh private Rust checks, formatting/Clippy and byte-identical five native
proofs (660 observations/twelve original executions). Cache byte-order/coordinate/
borrow/ownership invariants were inspected; no source finding. Its independent
fresh seven-pair CPU-0 run retained all full-state/input hashes: 51.053 ms before
versus 9.778 ms after for 1000 links (5.22 ratio). Opaque tail was about 1% slower
with overlapping ranges. Author/independent timing differences remain explicit
shared-host limits. No whole-engine/P2/packet performance acceptance is inferred.
Normal exact main binding, staged gate and hosted qualification remain required.

## Static-model attachment identity candidate

Native `STATIC_MODEL` sinks now retain their scoped implicit-frame identity;
static links remain separate link sinks. Dynamic parents use the recursive
canonical link when present, and follow the first nested model only when the
canonical name is empty. This fixes both the existing static child endpoint gap
and the unchanged nested-static upstream fixture without choosing simulator D23.

Fresh author checks pass all 181 Rust tests and 13 tools, formatting/Clippy/doc
compilation and unchanged dependency/layout gates. Coverage is 3281/3297 lines
and 5728/5839 regions, with 15/15 changed executable lines. The 61-row native
corpus and one exact native original pass. Existing 150-row/four-original alias
proof is byte-identical; 17 canonical rows/one original and 201 general rows also
replay. Historical corpus metadata and all prior XML/tokens remain unchanged.

Four Rust-only world attachment values and the old static child negative have
explicit native-backed dispositions; reversing those edits reconstructs all
three public-parent test files byte-for-byte. The new complete original mapping
uses mandatory eager graph construction/validation for one known-valid fixture;
independent assertion/source review explicitly approved it. No runtime/static policy,
new dependency, unsafe scope, mutable graph facade or full packet acceptance is
claimed. Independent source review passed; exact main overlap, normal staged
hook and hosted qualification remain pending. See STATIC_MODEL_ATTACHMENTS.md
for exact scope and replay commands.

Independent /root/transport_fallback approved the exact 17-file source after 181
fresh private Rust checks, formatting/Clippy, byte-identical 61-row/exact-original
native proof, unchanged 150-row/four-original alias proof and all 17 canonical
observations. A public-parent countercontrol reproduces the original static sink
failure. The four world identities, static-child positive disposition, recursive
canonical edge and known-valid eager graph-validation mapping were explicitly
approved. Only raw ldd ASLR addresses differ in the canonical proof. This adds
one complete original mapping (49 complete / 11 partial), pending exact main
binding, mandatory staged verification and hosted qualification.

Fresh main integration preserves the approved executable source/tests/corpus and
native proof exactly. All 181 Rust tests, 13 tools, formatting/Clippy/doc and
layout/dependency checks pass on the final protected metadata. Coverage remains
3281/3297 lines and 5728/5839 regions, with all 15 changed executable lines
covered. Only additive review metadata, the approved original's mapping status
and current review prose differ from the source freeze. Exact independent main
binding, normal protected staged hook and hosted qualification remain required.

## Pose-relative graph original candidate

The unchanged `model_frame_relative_to_joint.sdf` fixture now drives a partial
port of `FrameSemantics.buildPoseRelativeToGraph`. The exact pinned native original
passes one run with zero failures. The isolated Rust test passes all eight
model-root poses, seven poses in their declared relative frames and both invalid
name failures with the original component-wise pose comparison bound. All
182 Rust and 13 tooling checks, formatting, Clippy and docs pass in the isolated
worktree; author direct coverage is 3281/3297 production lines and 5728/5839
regions with no changed production lines. Native graph map/vertex/edge counts and direct build/validate checks, plus exact native
error code/text, have no public equivalent in this typed loader and remain
unported. This does not retire a complete original case or change production
behavior. Independent assertion/source review approved the corrected notice,
all 8+7+2 query mappings and the explicit partial scope after fresh native 1/0
and private 182 Rust/fmt/Clippy replay. Exact metadata binding and protected
staged/hosted gates remain pending.

## Invalid world-frame attachment candidate

The unchanged `world_frame_invalid_attached_to_scope.sdf` fixture now checks the
first diagnostic from `NestedFrameSemantics.InvalidAttachedToScope`. Native Root
loading reports an invalid attachment before its default pose's same missing
reference. The typed graph validator now does that scoped world-frame name check
first and returns the original diagnostic text. The public parent instead
reported a generic unknown local frame; the red assertion log is retained.

The exact native original passes one run with zero failures. Native controls
confirm that `M1` and `M2::M1::L2` remain valid world-frame targets, while
`M9::L1` has the corresponding first attachment error. A pose-only invalid
reference stays distinct and is a documented unmatched diagnostic context.
All 184 isolated Rust and 13 tooling checks, fmt, Clippy and docs pass; author
direct coverage is 3300/3316 production lines, 5746/5857 regions and 19/19
changed executable lines. The native graph-check query booleans, all seven error
entries and native error code are not mapped by the eager typed loader; this is
one partial mapping, not a complete original retirement. Independent review
approved the exact source, first-message mapping and five native rows after
private 184 Rust/fmt/Clippy replay and a public-parent red countercontrol.
Final metadata binding, protected staged gate and hosted CI remain pending.

## Root frame semantics after move candidate

The exact pinned `DOMRoot.FrameSemanticsOnMove` original passes one run with zero
failures. Both inline SDF 1.8 inputs match the native source byte-for-byte. A
Rust test retains its four load, move-construction and move-assignment blocks,
with all original world/frame presence, frame-name, pose-resolution success and
pose-value observations. Owned `Sdf` moves transfer its owned semantic graphs;
independent review approved this one complete original mapping after
replaying both exact input strings, all four observation blocks and a failing
move-assignment countercontrol. No graph facade or production code changes are proposed.

All 185 isolated Rust tests, 13 tooling checks, formatting, Clippy and docs pass;
author direct coverage is 3300/3316 production lines and 5746/5857 regions.
Exact final metadata binding, protected staged gate and hosted CI remain pending.

## Nested model attachment-body original candidate

The unchanged `model_nested_frame_attached_to.sdf` fixture drives the public
attachment-body subset of `NestedFrameSemantics.buildFrameAttachedToGraph_Model`.
The exact pinned native original passes one run with zero failures. A Rust port
checks all 13 original successful body values, including nested model-frame,
link, declared frame and joint targets, plus the invalid-name failure. Public
`resolve_name` translates terminal model-frame aliases before `attached_body`;
returned names are compared relative to the original root model scope.

Explicit native graph construction/validation, graph Count observations and the
native invalid-name error code/text remain unported. This is partial only, not
a full original retirement. All 185 isolated Rust and 13 tooling checks, fmt,
Clippy and docs pass; direct coverage is 3300/3316 production lines and
5746/5857 regions. Independent review approved the exact 13 body values and invalid-name
failure as partial only after fresh native1/0 and private185 Rust replay. Final
public-parent overlap, protected staged gate and hosted CI remain pending.

## Basic model and world frame attachment bodies candidate

Unchanged `model_frame_attached_to.sdf` and `world_frame_attached_to.sdf` fixtures
drive two partial original ports of `FrameSemantics.buildFrameAttachedToGraph_Model`
and `_World`. The exact pinned native originals pass 2/0. Public name resolution
and attached-body queries retain all six root model, six world and three embedded
model body values and both invalid-name failures. Native explicit graph build,
validation, map/vertex/edge/Count observations, graph-check booleans and exact
error code/text remain unported. Independent review approved those public query
assertions only after a private selected2/full188 Rust replay. All 188 author Rust
tests, 13 tooling checks, fmt, Clippy and docs pass; direct coverage is 3300/3316
production lines and 5746/5857 regions with no production delta. Final metadata
binding, normal protected hook and hosted CI remain pending.

## Nested world attachment-body original candidate

The unchanged `world_nested_frame_attached_to.sdf` fixture drives a partial
port of `NestedFrameSemantics.buildFrameAttachedToGraph_World`. The exact pinned
native original passes 1/0. Public queries retain all sixteen world-scope and
six embedded-model body values, one invalid world name and seven genuine
model-scope out-of-scope failures. The native model graph also rejects `world`
as out of scope; the public resolver intentionally uses `world` as an enclosing
world alias, so that graph-only assertion remains unported. Native explicit
graph construction/validation, Count observations, graph-check booleans and
exact native error code/text also remain unported. Independent review approved
this corrected partial mapping after private selected1/full187 Rust replay.
All 189 integrated Rust tests, 13 tooling checks, fmt, Clippy and docs pass;
author direct coverage is 3300/3316 production lines and 5746/5857 regions.
Final public-parent binding, normal protected hook and hosted CI remain pending.
