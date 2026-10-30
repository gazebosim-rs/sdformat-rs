<!--
Copyright (C) 2026 Open Source Robotics Foundation
SPDX-License-Identifier: Apache-2.0
-->

# Native-valid Empty geometry

`Geometry::Empty` is the default Geometry value and represents known empty SDF
metadata. It has no shape payload; it does not imply a physical point, collider
or zero-size primitive. `Geometry::to_xml` emits `<geometry><empty/></geometry>`.
The shared loader accepts an explicit `<empty/>`, an empty geometry group and
the native missing-Collision geometry default. No density behavior is added.

Visual's existing distinction is retained: an absent or bare geometry group
remains `None`, while an explicit `<empty/>` is `Some(Geometry::Empty)`. Both
forms have native EMPTY getter values; this records the Rust declaration form
without equating Option presence with a native pointer. Native `Geom()` always
addresses an owned Geometry. Input Element origin, pointer identity and mutable
reload/cached hidden shape state remain outside this value API.

The parser reuses existing allowed-child/attribute and shape-cardinality checks.
Unknown attributes/children, duplicate empty tags and mixed empty/other shapes
fail explicitly. Native often tolerates unknown payload or chooses another shape
by priority; those existing strict Rust policies remain named differences.
Text/comments follow the existing known-shape behavior and do not create a new
payload or restriction. Native also discards ignored text on canonical emission.

## Preserved test transitions

Independent prior review first replayed all 38 native profiles and constructor
output using fresh compilation with pinned identities. The exact former Rust-only
negative inputs are genuine native positives. A second independent review ran
four prepared Rust controls and approved only these folds:

- Standalone bare `<geometry/>` keeps `caller.xml` and gains typed-empty/canonical
  output/reload assertions; the unrelated bad-Box error and source checks remain.
- Exact explicit-empty Visual input retains `caller.xml`, complete properties,
  pose, source, material/plugin absence and immutable output/reload checks.
- Missing/bare Collision inputs retain their full model XML and `test.sdf`, with
  model/link/collision counts/names, pose/source/default surface, frame validation
  and geometry roundtrip assertions.
- The existing Visual corpus row keeps its literal input and complete native
  observations; only the temporary Unsupported disposition becomes a full positive.

Every unrelated negative and original assertion remains. This admission adds no
whole-original ledger credit by itself. It enables later Collision serialization
to preserve its genuine EMPTY constructor setup rather than substituting a Box.

## Consumer boundary

No simulator dependency pin or runtime code changes here. Consumers must explicitly
support or reject Empty before adopting the revision. The current scene importer
in `gz-sim-rs/crates/gz-sim-scene/src/lib.rs` supports four physical shapes and has
an unsupported fallback; the coordinator must add explicit Empty handling and
consumer tests before a pin update. Metadata acceptance alone creates no body,
collider, renderer resource or callback. Density remains unsupported.

## Evidence

The native helper records 38 schema/Geometry.Load/Root observations, one exact
constructor fixture and 20 actual Geometry/Visual exchanges. The 18 strict
structure differences are retained as explicit Rust failures. Collision/document
rows export only their Geometry value; no Collision serializer is claimed.
Native missing/bare/explicit Empty all yield Type 0, no shape pointers and an
empty polyline list. Independent fresh source/owned target review passed 33
distinct selected Rust checks and all 17 tooling controls, reproduced the full
new proof and both unchanged Geometry/Visual native pipelines, and approved
source/assertion folds without findings. Reversible static-model successor
metadata preserves its original preimage and negative-mutation test. Author-wide
547 Rust checks also pass. Normal strict staged and exact hosted qualification
remain pending; accepted mappings stay 201 complete/27 partial.

```sh
python3 tools/geometry_empty_reference.py ../sdformat
cargo test --offline --locked --test geometry_empty --test geometry_xml --test visual_xml --test semantics
```
