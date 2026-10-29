<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Light input origin and quiet output

Light::source records the supplied URI and exact line/column of a successfully
loaded explicit declaration. Constructed values have None. Cloning, typed edits
and immutable XML emission preserve origin; loading emitted XML records the new
input. The writer remains unchanged and does not use provenance to choose output.

This is explicit declaration metadata. It is not a native Element pointer or an
identity, ownership or mutation interface. Direct document Light schema admission is documented separately in
DOCUMENT_LIGHTS.md; origin metadata does not imply generic Element semantics.

## Exact original mapping

The two native Light ToElement originals assert constructed Element null after
emission and loaded Element nonnull. Independent narrow review approved the translation before old-test edits. It
checks constructed
SourceNone after successful emission (no fabricated input declaration), then
binds loaded Some to the literal supplied light-roundtrip URI and root line 1,
column 1 (successful explicit input declaration presence). It does not assert
pointer equality or generalize this mapping to omitted schema groups.

All existing name/type/pose/frame/shadow/color/attenuation/direction/spot/intensity
and edited-shadow predicates remain. Both the direct quaternion component bound
and the prior dot-product bound are retained. The error-output case uses the existing child-process quiet helper to verify
actual empty library stdout/stderr alongside the unchanged value checks.

Three prepared Rust controls cover exact standalone source locations, edit/clone/
immutable emission, constructed/default output and complete typed reload equality
with an independently literal origin, plus retained document-admission limits.
The native lifecycle probe observes constructor-null before/after emission and
loaded FilePath/line retained through edit/emission; both unchanged native
originals pass with empty stderr. Independent narrow review passed three prepared controls/four unchanged Rust
ports and the pinned native lifecycle/both originals before the original-test
additions. Author formatting, Clippy and all 485 Rust tests pass. All twelve prior
family helpers and 211 unchanged XML exports pass; configured Spot bytes also
match accepted Light output and a freshly compiled old native peer loads them.
Those regression comparisons remain author evidence. Final independent review passed seven Rust checks in fresh source/target,
pinned native lifecycle and both genuine originals with empty stderr. Private
stdout/stderr injections each failed the actual quiet wrapper; restored source
matches the author and quiet replay passes. Both exact mappings are reviewed
complete. Normal strict staged/hosted qualification remains pending: candidate 188
complete /24 partial is separate from accepted 186 complete /26 partial.
