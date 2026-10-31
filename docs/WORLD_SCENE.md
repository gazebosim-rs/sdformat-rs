<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Bounded world scene metadata

This private slice extends the existing partial `DOMWorld.Load` mapping
(`test/integration/world_dom.cc:79–171`, pinned native source
`1249805d9fceeff94ca1fe63e26ab9c051cea24e`) with its five missing scene
observations: ambient, background, grid, shadows and origin visual. The complete
original scene subtree is retained unchanged in a root/world projection of
`test/sdf/world_complete.sdf`; the original vector fixture and all earlier
assertions are unchanged. Native full-document and projected getter observations
match exactly. Colors compare all four native f32 bits, strengthening the
original Color equality assertions without changing their literals.

Native World owns Scene by value (`World.cc:65,577`), so `World.scene` is a concrete
owned `Scene`, including when no scene is declared. Defaults are ambient
(0.4,0.4,0.4,1), background (0.7,0.7,0.7,1), with all three flags true. The new
value carries only those colors/flags and optional explicit input Source. Source
None means no declaration appeared in the caller's XML; it does not claim native
Element-null equivalence, because native schema processing can synthesize a
Scene Element. No general Element or FilePath identity is mapped.

Parsing delegates to the unchanged QuickXML cursor and existing color/boolean
conversion. Missing fields receive native defaults. Explicit empty or whitespace
ambient/background/shadows fail, matching the required native schema parameters;
empty grid/origin_visual retain true. Native color prefixes and per-component
NaN-to-zero behavior remain observable through the existing color conversion.
Infinity, out-of-range components and malformed colors remain invalid.

Sky, fog, unknown scene children/attributes and unrelated world GUI/
spherical data remain Unsupported. Duplicate scene or singleton fields retain
the existing Rust InvalidValue policy even where native keeps the first value.
Case-mismatched known world names remain Unsupported. There is no scene writer,
resource loading, rendering, color-space conversion or simulator activation.
Consumers must explicitly support or reject the new metadata before changing
the producer pin; adding the public World field also affects direct struct
literal construction/destructuring. The monorepo pin is unchanged.

Root independently reviewed the initial nine-path source and five prepared Rust
controls in a fresh target. A fresh native replay reproduced all 47 observations,
constructor defaults, full/projected equality and both unchanged originals. Root
approved only six identical old inputs and caller URIs: the old wind-plus-empty-
scene world, the known-schema empty scene, three known-scene corpus rows at
versions 1.5/1.8/1.12, and the document-light empty-scene input with its point
light. These predicates now use the prepared full positives in their existing
tests; the temporary duplicate test is removed. A shared assertion helper retains
all scene values, owned source, world identity/default vectors and exact child
counts. The light case additionally retains the complete native Light projection
and resolved pose. All other schema, unknown, GUI/projector, Sky/Fog, duplicate
and case-mismatch assertions remain unchanged. The old corpus bytes are unchanged.

Author final replay passes 15 affected Rust tests (four Scene, three environment,
five document-light and three unqualified-world tests) and all-target Clippy.
The native helper reproduces 47 complete controls, constructor defaults,
full/projected equality and both unchanged DOMWorld.Load and
DOMScene.Construction originals. The latter is a supplemental native control,
not a new complete Rust port: Sky and constructor Element contracts remain
outside this slice. Native source, installed header, selected library/build and
separate original-binary linkage identities are checked before/after replay with
the same execution environment. Root final source/fold review found no remaining
issue; a fresh independent source and target passed all 15 affected Rust tests.
The earlier independent 47-native/constructor/full-projection/two-original replay
remains unchanged. Author full suite passes 593 Rust and 39 tooling checks.
Ordinary protected staging and exact hosted checks remain pending. World.Load
remains partial with no additional whole-case credit; accepted counts remain
203/27.


The first protected staged hook failed on uninstrumented declaration/aggregate
syntax after all tests and aggregate coverage passed. The failed hook and raw
LLVM report are retained. The separate proof follow-up passed final independent
23 helper/43 tooling/Clippy replay after two scope corrections; actual measured
DA and region authority are unchanged. Retained-data 42/42 diagnostics are not a
staged pass. Full combined protected staging and exact hosted checks remain
pending; no producer code or original/native assertions changed for the proof.
