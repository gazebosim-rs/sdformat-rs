# Evidence and open work

Task: first bounded SDF-01 / QLT-01 slice, 2026-10-24. No SDF-01, SDF-02, P1 or
release completion is claimed. The implementation author cannot approve its own
assertion equivalence or exclusions.

Upstream main: `1249805d9fceeff94ca1fe63e26ab9c051cea24e`.
`python3 tools/inventory.py ../sdformat` expanded the actual existing reference
build: 308 CTest targets, 1344 runtime cases, zero enumeration command failures,
1068 static C++ test macros, and 1337 source/schema/fixture SHA256 entries.
Generated test-result wrapper targets have no independent runtime cases. Static
macros and runtime counts differ because fixtures/parameters and Python cases
expand differently. Non-enumerable target commands remain explicit in inventory.
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
to owned collections and verifies resolved scoped body identity. Independent
assertion review remains recorded separately. Additional tests exercise a valid
nested attachment fixture, typed physics input, graph failures, bounds, includes,
provenance, plugins, and malformed input; these are supplementary tests, not a
claim to port whole upstream families. The five-case native/Rust acceptance corpus
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

Dependencies: roxmltree 0.20.0 (read-only namespace-aware XML syntax), glam 0.30.9
(f64 rigid transformations), petgraph 0.8.1 (cycle detection/topological order).
Exact versions plus transitives are locked. Custom code is SDF graph scoping and
schema/domain validation, not generic XML, quaternion or graph algorithms.
serde_json 1.0.150 is a test-only corpus dependency.

Next bounded work: validate and type IMU-specific configuration; model/include
placement frames and merge behavior with C++ probes; older conversions and
serialization; complete schema and retained test inventory, fuzzing, and native
parse benchmarks on tiny and large nested inputs. No comparative performance
claim, full upstream pass, full source/build provenance, or release qualification
is supported by this initial slice.
