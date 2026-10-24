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
assertions each, using gz-math's 1e-6 equality tolerance (quaternion sign is
physically equivalent). The canonical-link port maps C++ pointer/index assertions
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

The next bounded slice adds complete typed IMU-specific configuration and source
mapping. The original DOMLink.Sensors IMU assertion subset passes in Rust on the
exact IMU subtree from the unchanged sensors.sdf. All other sensor families in
that source case remain pending, so the original case is not marked ported.
The complete native DOMLink.Sensors case passes. Differential probes establish
negative noise sigma rejection, case-insensitive gaussian_quantized, and native
preservation of unknown localization/parent-frame strings. Original sensor XML
is retained with typed settings. The full upstream fixture has an undeclared XML
prefix in another sensor, which roxmltree rejects; exact-subtree extraction keeps
the relevant IMU bytes unchanged and does not hide that open syntax compatibility
gap. This slice has 18 Rust tests and six gate tests; coverage independently reviewed by /root is 1193/1201 lines, 2077/2164 regions, and 117/118 changed
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
all four original include scenarios and original 1e-6 pose assertions. Include
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

Placement slice verified production coverage:1300/1314lines and2261/2352regions;
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

Precision slice coverage:1347/1361production lines and2353/2444regions;
changed48/48. Independent review:/root.

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
validation instead of failing XML parsing. Its other26 sensor families remain
pending. DTD remains an explicit Unsupported feature. Original fixture bytes and
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
plugin children pass and496 fail in the original three-element wrapper. Legacy
header declarations remain permissive, as native accepts <?xml?> before the root.
Corpus:53 acceptance cases, four numeric poses, nine scalar-mass observations and
three typed-string observations.36 Rust tests and six tooling tests pass;
coverage1618/1639lines,2772/2883regions,changed261/265. Final independent re-review
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
Unsupported, and1.6 conversion remains unavailable. Both native and Rust pass the
complete NestedModel.NestedFrameOnlyModel pose assertions on the unchanged1.7
fixture.39 Rust tests and six tooling tests pass; full versions/conversion parity
and the overall SDF packets remain in progress.
