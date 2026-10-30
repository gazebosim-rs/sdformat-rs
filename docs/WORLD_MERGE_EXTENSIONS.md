<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->
# World-merge extension metadata

`World::extensions` retains qualified opaque child metadata imported by
world-scope merged models. Each record keeps its lexical tag, decoded attributes,
original XML span and imported resource source. Imported entries retain order
within and across includes. They are owned data, not a mutable native Element;
no expanded-world XML writer or Element identity is implied. Native Element
serialization can normalize or lose lexical mixed content; the original imported
XML span is retained, and producer-byte equality is not claimed. Adding this public
field requires external `World` struct literals and exhaustive destructuring to
account for it. Current simulator Scene loading does not consume World
extensions; it must explicitly support or reject them before adopting a pin
containing this capability. No simulator dependency pin is changed.

Pinned native observations cover the prior exact rejection input, two ordered
imports with repeated qualified names across SDF 1.5–1.12, absent extensions,
unqualified imported children, direct world extensions, nested merge ordering and multiline UTF-8 metadata. Native copies qualified
imports into the world Element and omits imported unqualified unknowns. This
candidate preserves qualified imports and omits unqualified imports; direct
world extension declarations remain explicitly Unsupported even though native
retains them. Existing direct-link merge and world unknown/cardinality rejections
remain. No generic schema, namespace, warning, resolver or serialization parity is
claimed.

`python3 tools/world_merge_extensions_reference.py ../sdformat` replays fourteen
native output/error observations and three focused Rust checks, including the
exact old input, source columns, ordering and proxy attachment. Native source,
installed headers and loaded libraries are hash-qualified. An independent pre-change review replayed the exact prior Unsupported input and
confirmed native success before that predicate was replaced with checks for the
model, proxy frame, resolved pose, source and metadata. Independent final review reproduced all seven affected Rust checks and Clippy;
normal staged/exact hosted gates remain pending. No original-test credit is proposed.
