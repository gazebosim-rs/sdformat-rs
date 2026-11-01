<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# World includes of external lights

This private slice allows a world `<include>` to resolve exactly one external
`<sdf><light>` through the injected resource resolver. It reuses the existing
document Light reader, legacy pose conversion and world frame validation. The
complete typed Light remains owned; no renderer, asset loader, system callback,
new dependency or consumer pin is introduced.

Light order follows declaration order among direct and included lights. A name
override changes only the name. A pose override replaces the transform and frame;
an empty pose means identity with no explicit frame. Omitted pose attributes do
not inherit the resource's `relative_to`, degrees or rotation format. The existing
finite-value, name and quaternion rules apply. The Light's source URI and
line/column still identify its declaration in the canonical resource returned by
the resolver, including after overrides. A clone owns its metadata independently.

An unresolved pose reference remains a deferred semantic error, as with direct
world lights. Consumers must call the existing `validate_non_frame_poses` before
instantiating their runtime. Loading has an atomic Result: it never returns the
native parser's partially loaded first Light alongside errors.

Root and model Light includes remain Unsupported. `merge=true` and model-only
include options (`namespace`, `static`, `placement_frame`, `plugin`) also remain
Unsupported for Light resources. Explicit `merge=false` (including `0`) is
accepted. Native ignores several of those options; this
is an explicit bounded domain, not a malformed-native-input equivalence claim.
Empty/multiple/mixed resource roots and unknown elements/attributes fail closed.
Every existing model include path and public value API is preserved.

## Native basis and retained assertions

The source pin is `1249805d9fceeff94ca1fe63e26ab9c051cea24e`.
`src/parser.cc:1765–1953` selects the included kind, applies name and pose
overrides, and limits static/placement and plugin processing to their native
kinds. `IncludesTest.Includes` uses two external Light includes: the first retains
its name and world-relative pose, the second overrides both name and pose. The
exact original include subtrees and exact SDF1.6 `test_light/model.sdf` fixture are
retained. The Rust test preserves the Light count/presence, names, Point type,
shadow flag, both poses/frames and all four attenuation assertions. The original
four-ULP double bound is unchanged. Supplemental full getter projections are
exact except new pose comparisons use the existing resolver's 1e-12 bound.

The original case also asserts actors, models and generic Element APIs. Those
observations are not claimed here. In particular, native
`Element::OriginalVersion` remains recorded in the native stream but is not a new
Light getter; the containing SDF version and original resource bytes remain
unchanged. No mutable Element, include-to-XML or full original completion credit
follows from this bounded port.

The initial 23 native queries include 22 normal process exits and a genuine
native empty-name abort: `<name/>` throws `std::logic_error` from a null string.
That is retained as native failure evidence. Rust keeps its existing typed
Missing rejection and does not copy the crash. Multiple Lights produce a native
error with a retained first Light; the owned loader instead returns only an
error. Native-accepted ignored-option cases stay explicit Unsupported here.

The final focused run passes 35 tests: the four new controls and retained model,
root, merge and semantic/depth tests. All-target Clippy and formatting pass. The
final native replay reproduces all 23 stored observations (22 normal exits and
the separately retained empty-name abort) and the unchanged full native Includes
original passes. This native result does not complete the Rust original mapping.
Independent review, the ordinary strict staged hook and exact hosted/source
qualification remain pending. No simulator or B03 runtime acceptance follows.

The first enum-based refactor overflowed the unchanged 64-level include-depth
test's stack. Exact accepted b246 source passed under the same compiler, flags
and thread settings. Removing an extra recursive wrapper alone did not suffice;
owned Light and Model move/box temporaries also needed to stay off recursively
live frames. Small finishing helpers now run after recursive loading returns,
and the unchanged depth test passes. A private boxed sum type adds a transient
allocation to existing model includes; it keeps the result small. No depth bound,
thread-stack size, optimization setting, existing assertion or tolerance changed.
The ordinary instrumented gate must still exercise that same depth case; a
focused debug run is not an instrumented pass or a performance qualification.
