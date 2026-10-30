<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->
# World-merge extension metadata

`World::extensions` retains qualified opaque child metadata imported by
world-scope merged models and declared directly in the world. Each record keeps
its lexical tag, decoded attributes, original XML span and resource source.
Merged entries precede direct entries; each group retains its own order and
duplicates. XML namespace declarations are preserved in the raw span but omitted
from the decoded attribute list by the existing tokenizer. They are owned data, not a mutable native Element;
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
candidate preserves qualified imports and direct declarations while omitting
unqualified imports; unqualified direct world children remain Unsupported even
though native retains them with a warning. Existing direct-link merge and world unknown/cardinality rejections
remain. No generic schema, namespace, warning, resolver or serialization parity is
claimed.

`python3 tools/world_merge_extensions_reference.py ../sdformat` replays fourteen
native output/error observations and three focused Rust checks, including the
exact old input, source columns, ordering and proxy attachment. Native source,
installed headers and loaded libraries are hash-qualified. An independent pre-change review replayed the exact prior Unsupported input and
confirmed native success before that predicate was replaced with checks for the
model, proxy frame, resolved pose, source and metadata. Independent final review reproduced all seven affected Rust checks and Clippy;
normal staged/exact hosted gates remain pending. No original-test credit is proposed.


A follow-up candidate adds direct qualified world children. Thirteen pinned
native observations cover ordering, duplicates, lexical prefixes, namespace
declarations, attributes, unqualified unknowns and duplicate known fields in
SDF 1.5, 1.8 and 1.12. An independent pre-change review reproduced the prior exact direct-world input
and its positive before the old assertion was replaced. Final independent
review replayed all 13 direct and 14 prior native observations, nine affected
Rust tests and all-target Clippy. The simulator consumer obligation above remains.

The validation helper admits a qualified extra name only when its caller
explicitly lists it. Independent review covered all callers and passed 29 Rust
controls plus all-target Clippy; the local staged gate must be rerun.
