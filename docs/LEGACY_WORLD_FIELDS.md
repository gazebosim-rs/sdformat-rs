# Legacy world-vector conversion

SDF 1.4 and 1.5 can declare `gravity` (m/s²) and `magnetic_field` (tesla) inside
its first world physics group. The typed loader projects these into the existing
`World` vectors, following the two moves in pinned `sdf/1.6/1_5.convert`.
Defaults, finite/exact-three vector validation and physical units are unchanged.
The values are world metadata; no physics or magnetometer computation occurs.

This bounded path requires at most one declaration of each vector in that physics
group and no competing declaration of the same vector directly on the world.
A legacy field in a later profile is Unsupported. Native instead appends only the
first profile's first field to the world, lets an earlier direct world field win,
and retains later or duplicate physics fields as unknown XML with warnings. Those
native warning/ignore cases are captured but deliberately not admitted. Unknown
attributes/children, malformed/empty/nonfinite vectors and duplicate singletons
retain the existing strict policies. SDF 1.6–1.12 physics fields are not converted
and remain Unsupported. Native accepts additional vector tokens; the existing
Rust exact-three policy continues to reject them.

`Sdf.version` is the actual input version. `Physics.source` and `Physics.xml` keep
the declaration's original location and exact input subtree, including legacy
fields. They do not describe a synthesized current-schema Element. This is a
semantic load projection, not `convertString`, converted XML serialization,
`Element::OriginalVersion`, mutable Element identity or error-vector parity.
The original snapshots remain unchanged when typed metadata is edited or cloned.
No dependency or resource policy changes.

The two exact inline fixtures from `test/integration/converter.cc` retain the
world/name, physics/name/type, load success and gravity/magnetic value observations
of `ConverterIntegration.ParserStringConverter` and
`ParserStringConverterFrom14`. Rust comparisons retain native component-wise
absolute ≤1e-3 vector equality and add exact bits. Root/world/physics pointer
presence translates only to typed collection presence. Conversion to version 1.6,
root/descendant OriginalVersion and relocated/removed Element observations remain
unported, so both original cases are partial.

`python3 tools/legacy_world_fields_reference.py /path/to/sdformat` checks pinned
native source/header/library hashes, 83 recorded controls, original fixture bytes,
actual DOM and convertString observations, and both unchanged full originals.
`cargo test --locked --test legacy_world_fields` checks the corresponding typed
values and strict refusals, source locations, original XML and ownership. Native
characterization passed before production edits. Five prepared Rust tests passed before prior review. Root approved only the exact
gravity row transition; the historical corpus remains byte-identical, and its
original branch now calls the same exhaustive positive under the same URI.
All 663 retained Rust tests and all-target Clippy pass. The first expected old
rejection failure is retained separately. Independent final review passed with fresh source/target Rust/native replay,
foreign-Git integrity,Clippy and fmt checks. Strict staged and exact hosted
qualification remain pending.

Copyright (C) 2026 Open Source Robotics Foundation. SPDX-License-Identifier: Apache-2.0.


The first ordinary staged gate passed49 tooling checks and the instrumented Rust
suite, then failed closed on two absent-DA source delimiters. Its temporary raw
snapshot was cleaned; its log is retained. The distinct persistent diagnostic
retains actual source/raw/profdata/LLVM/ELF evidence:8611/8652 lines,13609/13879
regions and47/47 measured changed lines against accepted3e9e, with no changed DA0.
The actual unclassified lines had no overlapping LLVM regions. Independent prior
review approved replacing the inline child-slice argument with a named local and
the new matches macro with ordinary exact input-version comparisons. No proof,
coverage rule, assertion or unsupported input changes. The refactored source
requires the ordinary strict retry and exact hosted qualification; earlier663-test
and native evidence remains separately bound to its original source.
