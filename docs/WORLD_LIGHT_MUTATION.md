# Owned World light mutation

`World::add_light_unique` moves a Light into the world's ordered collection only
when no current direct Light has exactly that name. It returns false without
replacing the original on a duplicate. `World::clear_lights` removes that
collection. These follow the existing Link helpers and the pinned native
`World::AddLight` and `World::ClearLights` implementations.

This is owned DOM mutation, not XML admission. Like native mutation, it accepts
an empty, reserved, scoped or space-containing name and compares case
sensitively; it does not run the document parser's name rules or validate poses.
Renaming through an ordinary mutable vector borrow changes the next insertion's
name comparison. Insertions preserve the complete Light value, including its
optional original Source; clearing a World or cloning a Light collection never consults a resolver.
The caller retains a separate value by cloning before insertion.

The complete fixed `DOMWorld.AddLight` observations map to
`tests/world_light_mutation.rs::original_dom_world_add_light`: initial zero
count, insertion true/count one, duplicate false/count one, clear/count zero,
reinsertion true/count one and index-zero presence/name. Native uses a default
World; Rust parses an otherwise empty world as setup. No World-constructor or
C++ pointer identity equivalence follows from that setup. The test keeps the
original OSRF notice. Supplemental controls cover exact-name edge cases,
first-value retention, declaration order, rename/relookup, owned Light collection clone/source
independence and an unchanged loaded frame query. The Light portion of native
`MutableByIndex` motivates the rename control; the whole multi-family original
is not claimed.

`python3 tools/reference_world_light_mutation.py ../sdformat` replays the
supplemental native controls and the unchanged full `DOMWorld.AddLight` original.
The reference record binds the pinned source, installed headers, compiler,
original ELF and selected shared libraries; runtime paths are explicit and
loader-injection variables are cleared.

Loaded semantic graphs remain snapshots. These helpers do not rebuild them,
add runtime lights, serialize a World, or promise mutable Element aliasing.
Renderers and simulators must separately support or reject edited metadata.
No dependency, consumer pin, Fuel behavior or existing parser rule changes.

The native characterization and unchanged full original pass, as do all three
focused Rust controls, all-target Clippy and formatting. The first new-test
compile assumption (World has no Clone API) and invalid synthetic world-name
failure are retained separately; only the new test setup was corrected.
Independent review and ordinary strict staged/hosted gates remain pending for
this private increment.
