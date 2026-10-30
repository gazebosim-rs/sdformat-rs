<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->
# Unqualified world extension metadata candidate

This isolated candidate retains genuinely unknown unqualified direct world
children in `World.extensions`, preserving raw XML, decoded ordinary attributes,
source and duplicate order. Existing qualified direct children and qualified
merged metadata retain their order: merged entries come first and direct entries
follow in document order. Unqualified unknowns from a merged model remain
omitted as in the prior boundary. The data is opaque; loading it executes no
plugin, system, rendering or physics behavior. Simulator consumers must support
or reject the metadata explicitly before adopting this dependency.

Native SDFormat retains unknown unqualified children with warnings; this Rust
loader does not expose or emit those warnings. Native also retains misplaced
known elements such as a world `<link>` as warned XML; this candidate rejects
those instead of representing them as supported links or disguising known schema
features. Known element and attribute spellings and conversion path components
from all supported 1.4–1.12 schemas are conservatively reserved case-insensitively,
even when invalid in world context. Existing world typed declarations keep their
normal validation; `scene`, `gui`, `audio`, `link`, `dt` and other known names
remain explicit errors when unsupported. Existing qualified extension behavior
is unchanged. This conservative denial can reject native unknown XML whose name
only differs in case from a schema spelling; no general warning, schema or
conversion parity is claimed.

The generated production name table is derived deterministically from all 538
pinned `.sdf` files and nine conversion files; its source files, SHA-256 values,
generator and asset are bound in `reference/world-known-schema-inventory.json`.
Cargo includes the `data/` asset in its package and rustc dep-info names it.
The verification source/review digests bind it, and a compiler-artifact check
requires the production library's dep-info to name it. Tooling controls reject
mutated asset receipts, wrong or missing library artifacts and missing or fake
dep-info. No production file or region is excluded from coverage.

Twenty-one native observations cover exact old `<unknown/>`, duplicates,
qualified/unqualified ordering with merged imports, UTF-8 and multiline raw
source, known schema elements, a misplaced link and case-different known names.
The native helper qualifies pinned selected source, installed headers, schema
inventory, native build/library identity and actual per-run loaded libraries;
raw stdout/stderr and maps remain in ignored artifacts. This is an installed
native-library replay, not an immutable reference-image claim.

Independent pre-change review approved replacing the exact `<unknown/>` and
three `plain`-child rejections with the native-backed metadata assertions.
Final independent review reproduced the 21 native observations and inventory,
passed eight affected Rust tests, 35 tooling tests and all-target Clippy, and
checked package and compiler dependency binding. Normal staged checks, exact
hosted CI and downstream consumer adoption remain pending. No original
native-test credit, performance or full SDF-02 claim is made.
The first staged attempt rejected the text asset under the source instrumentation
guard. Moving the data asset outside `src/` leaves that guard unchanged; the
revised layout passed independent review. The earlier retry then rejected
unmapped declaration lines; syntax proof 074f77c has since passed exact hosted
run 41059379474. This additive rebase passed independent re-review; a fresh normal staged run
and exact hosted qualification remain required.
