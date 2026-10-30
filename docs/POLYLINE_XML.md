<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Ordered Polyline metadata XML

Polyline::to_xml, load_polyline_str and Geometry XML preserve height, ordered
points and ordered multiple polylines through the existing parser. They do not
close polygons, remove duplicates, triangulate geometry or create collision or
rendering resources. One point and negative/zero heights remain valid metadata.

Height follows the existing finite, non-subnormal scalar policy. Point components
must be finite and may retain subnormals and signed zero. Existing exact-two,
unknown-node/attribute and singleton checks remain. Native scalar infinity and
vector-prefix admission differences are recorded without relaxing Rust policy.

An omitted or empty point node loads a schema zero point. Native default Polyline
emission succeeds with zero points, but subsequent Load inserts one zero point.
The Rust writer rejects empty point vectors and empty Geometry polyline groups
as Unsupported, avoiding silent insertion. Loaded document defaults already have
the point and can be emitted without loss. No constructor/default equivalence or
native pointer-origin behavior is claimed.

The two genuine Polyline originals configure height 1.2 and append exactly points
(1,2), (3,4). Rust public-vector pushes with immediate length-increment and exact
last-point assertions translate those successful normal additions; no insertion
facade or out-of-memory behavior is claimed. All original scalar four-ULP,
component <=1e-3, ordered count/value, nonempty output/reload and actual quiet
assertions are retained. Neither case asserts input Element identity.

The native Geometry originals use an empty default Polyline, unlike these valid
configured originals. Their block remains unported alongside invalid default
Mesh/Heightmap producer error behavior; no Geometry complete credit follows.
Root independently approved exactly two temporary Geometry Unsupported changes
before edits. Their literal inputs and caller URIs remain, with complete native
ordered observations, ancestry/source, immutable output and readback. All
unrelated predicates are retained.

All 522 author Rust tests, 17 tooling tests, formatting and Clippy pass.
Final author native proof passes 19 cases, one configured fixture, two exact
retained-input controls, nine actual Rust/native exchanges, both unchanged
originals and a separate empty producer/load profile. Existing Heightmap 28/15,
Mesh 28/18 and Geometry 48/46 native regressions pass. Independent fresh-source
review reproduced all native proofs and passed 21 selected Rust checks. Both
private stdout/stderr injections failed quiet capture; restored source passed all
21. Every original normal-addition/value/count/Result/quiet predicate and numeric
boundary is reviewed, with no findings. Strict local and hosted qualification
remain pending. Candidate 197 complete/27
partial remains distinct from accepted 195/27.
