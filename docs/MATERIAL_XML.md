<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Lossless Material/PBR metadata XML

Material::to_xml and load_material_str preserve the current typed colors,
shininess, render order, lighting/double-sided flags, script URI order/name,
shader/normal map and Metal/Specular PBR workflows. They reuse the document
reader without resolving script/texture assets or choosing a renderer.

Output preserves current typed values, rather than the original XML spelling or
warnings from an already-normalized input. Native converts a subnormal roughness
input to zero with a diagnostic; emission of the resulting zero is clean. Native
Material::ToElement omits shininess and Metal environment_map. Rust emits those
valid schema fields instead of silently losing current metadata. Native producer
byte parity is not claimed.

Colors retain float32 values within [0,1], including signed zero; edited nonfinite
colors reject because the reader would clamp or reject them. Render order uses
its existing float32 metadata domain. Standard decimal text is checked through
the unchanged audited float converter and must retain its exact bits. This covers
positive/negative zero, normal endpoints, infinities and canonical signed NaNs;
subnormals and noncanonical NaN payloads reject if they cannot survive the reader.
Shininess retains the existing finite double policy. PBR scalar metadata retains
its existing infinities/canonical signed NaNs and out-of-range finite values;
subnormal or noncanonical-payload edited values reject. No physical normalization
or numeric-boundary change is introduced. The existing signed-NaN decimal helper
is moved unchanged from Axis XML to the shared Param module.

Both PBR slots are supported, including an empty PBR group. Workflow kind must
match its slot. Inactive-kind fields must retain their exact existing defaults:
Metal cannot carry specular/glossiness maps or nondefault glossiness; Specular
cannot carry Metal maps or nondefault roughness/metalness. Signed-zero distinctions
are checked where omission would drop them. These states return Unsupported
rather than silently discarding fields.

The raw optional normal-map type attribute remains separate from native's
NormalMapType getter, which the pinned loader leaves at Tangent. Absent, explicit
empty, object/unknown strings and ordinary spaces retain literal input/reload
oracles. Forbidden XML/control characters in raw attributes reject. A light-map
UV index is retained even with an empty map. Optional empty maps are otherwise
omitted without changing typed values.

The shared text emitter preserves internal repeated spaces/tabs/line feeds with
one CDATA section and escapes ordinary markup. Outer ASCII whitespace, forbidden
characters and text requiring split CDATA reject. Scripts preserve all declared
URIs even though native's getter exposes only the first. Missing/default first URI
or script name, empty edited URI entries, reserved base-shader normal-map sentinel
and missing maps for normal-map shaders reject according to existing reader rules.
Shader aliases remain reader behavior; output uses canonical spellings.

## Exact native original disposition

The unchanged Material.ToElement and ToElementErrorOutput cases both pass natively.
Their constructor setup is preserved in the new packet, including unasserted
inactive workflow values: Metal has specular_map/glossiness_map="gloss"; Specular
has roughness_map="rough", metalness_map="metalness", roughness=0.1, metalness=1.
Native serialization drops these fields. Constructor-control tests retain every
value, compare complete native pre-emission getter bits/strings and assert the
Rust writer's explicit Unsupported result. No setup value is omitted to make a
success assertion pass.

Actual native-produced XML has a representable loaded state. Separate controls
verify its complete current getter values and quiet Rust output. These controls
are not translations of the original constructor-success predicate. No full or
partial original mapping is promoted by this slice; the same-input success gap
and native pointer/origin behavior remain open.

All 528 author Rust tests, 17 tooling tests, formatting and Clippy pass. New proof
passes 53 native inputs, three exact generated fixtures, 44 actual Rust/native
exchanges and both unchanged native originals. Regression evidence passes the
existing 110 Material rows/direct-float ABI/DOMVisual.Material, 30 PBR cases/
Material.PbrDOM and 17 Axis cases/exact fixture/actual exchange/unchanged original.
Independent fresh-source review passed 22 selected Rust controls and reproduced
all native proofs above. Constructor retention/rejection, numeric/text/raw fields
and the byte-identical shared Axis formatter are reviewed with no findings.
Private stdout/stderr injections each failed quiet capture; restored source passed
all 22. Strict staged coverage and exact hosted qualification remain pending. Accepted mappings remain 197/27; no
asset, rendering, physics or performance acceptance follows.
