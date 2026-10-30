<!--
Copyright (C) 2026 Open Source Robotics Foundation
SPDX-License-Identifier: Apache-2.0
-->

# Visual metadata XML

`Visual::to_xml` and `load_visual_str` preserve the currently typed Visual
metadata without loading resources, executing plugins or creating a renderer.
The standalone reader delegates to the existing document Visual parser. Pose
references remain unresolved until a consumer supplies a model/link context.

The writer emits name, quaternion pose/reference, cast shadows, float32
transparency, visibility flags, optional laser retro, supported Geometry and
losslessly representable Material/PBR metadata. It delegates geometry/material
output to their accepted writers; their explicit rejection policies remain.
Missing or bare geometry retains the existing `None` representation and valid
`<geometry/>` output. Explicit `Geometry::Empty` emits a canonical `<empty/>`
child and reloads as `Some(Empty)`. Both are native EMPTY values. The native
producer's explicit empty form is now admitted through the shared parser;
[GEOMETRY_EMPTY.md](GEOMETRY_EMPTY.md) records the exact capability transition.
No producer-byte or general pointer equivalence is claimed.

Transparency follows the existing stored-double-to-float32 conversion, including
native's default-zero result when a textual value overflows or underflows the
float getter. Writer values must be finite and return exactly the same stored
bits. A manually stored float32 subnormal is therefore rejected. Retro presence
is preserved independently of its value, including `Some(0)` and signed zero;
nonfinite/subnormal scalar output is rejected. Pose uses the existing finite unit
quaternion policy and rejects subnormal components that native pose parsing
cannot retain. Names/frame attributes must roundtrip through existing XML rules.

Plugin order and opaque lexical XML are retained unchanged. Nonempty raw XML
must be exactly one plugin element whose trimmed name/filename agree with the
owned fields. Mismatches fail explicitly rather than dropping payload or silently
rewriting it. An empty raw XML string permits constructing an empty plugin from
its owned attributes. XML namespace handling remains the existing lexical parser
contract; no additional namespace validation is introduced.

## Original assertion mapping

The genuine `DOMVisual.ToElement` setup is retained: name, cast shadows,
transparency 0.2f, pose (1,2,3,.1,.2,.3), flags 1234, retro presence/value 1.2,
default EMPTY Geometry, default Material, and one named plugin/filename.
`Result::Ok(nonempty XML)` maps returned output presence. Full reload preserves
all original field predicates: direct signed pose components <=1e-3, exact
transparency/retro values, material presence and plugin count/name/file.

Native `Visual::Geom()` always returns the address of an owned Geometry value.
Safe access to the owned Rust geometry field supplies the nonnullable-object
guarantee. The separate `geometry.is_none()` assertion checks the existing EMPTY
value, confirmed by actual native EMPTY readback; it is not an Option-to-pointer
presence equivalence or a declaration-pointer identity, mutation or ownership
mapping. Independent exact-original review approved this precise mapping.
Accepted
credit remains 197 complete/27 partial until strict/hosted checks pass; the
reviewed candidate has 198 complete/27 partial.

## Evidence

Author checks pass six new Rust tests, 28 native input observations, one exact
native constructor XML fixture, 25 actual Rust-to-native exchanges and the
unchanged original. Pose projections use the original direct signed 1e-3 bound;
other projected values compare exact bits/strings. Shape/material payloads also
retain complete typed roundtrip checks through existing delegated writers.
Native input warnings for float32 range fallback are recorded; valid emitted
output has empty native stderr. Unknown payload and missing name remain explicit reader errors. Native `<empty>`
is admitted by the separately reviewed Empty capability. Independent fresh source/owned target
review passed 30 selected Rust controls
and reproduced the full new native proof, plus unchanged Geometry 48/46 and
Material 53/three fixtures/44 exchanges/originals. No findings remain. Normal
strict staged and exact hosted qualification are pending.

Replay with the pinned SDK:

```sh
python3 tools/visual_xml_reference.py ../sdformat
cargo test --offline --locked --test visual_xml
```
