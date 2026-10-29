<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Loaded model elements

`Model::element()` borrows an optional immutable `ModelElement` containing the
loaded model's direct children after include expansion. `children()` preserves
load order and repeated tags. `find_child(tag)` returns the first direct child
whose tag matches exactly, and `has_child(tag)` tests presence without creating
schema defaults. These queries do not allocate. Each child exposes its tag and
optional explicit XML-decoded `name_attribute()`. A nested `<name>` value is not
used as an attribute fallback.

Presence is distinct from a typed boolean value: an omitted `<static>` differs
from a present `<static>false</static>`. Typed changes to the model's name,
static flag or wind do not alter this loaded snapshot. `Clone` shares the
snapshot through `Arc`; typed model values remain independently owned. The
storage records only each model's direct rows, without recursively copying
whole XML subtrees into every ancestor. Query time is linear in that model's
direct child count, and snapshot storage is linear in the expanded direct rows.
No performance measurement or speedup is claimed.

The view records the supported include name/static/pose/plugin expansion and
model-merge filtering. Generated proxy frames and imported allowed children
retain their observed order. Legacy typed link/joint/frame renaming does not
rewrite the loaded name attributes. The view is structural metadata rather than
original lexical XML or a general mutable Element/Param representation.
Nested-value lookup, schema-default construction, generic `Get`, serialization
and native C++ pointer/object-method parity remain outside this API.

The public `Model` struct gains `loaded_element: Option<Arc<ModelElement>>`.
Downstream explicit struct literals must supply `loaded_element: None`; parsed
models receive their snapshot from the loader. Constructed DTOs have no loaded
view. Consumers can borrow the view without retaining the source XML buffer or
resolver. The qualified native Root-drop observation also retained an edited
Model, so it does not establish that an Element was the sole surviving owner.

The `IncludesTest.MergeInclude` Rust port retains the original fixtures and all
previous load, count, reference, pose, axis, plugin and custom-element assertions.
It now checks the remaining loaded-element presence and explicit-name assertions.
Native `IncludesTest.MergeInclude` and `MergeIncludePlacementFrame` both pass.
Independent source and 35-test runtime review passed. The required staged
coverage gate and hosted CI remain pending;
this change does not complete SDF-02 or the general native Element API.
