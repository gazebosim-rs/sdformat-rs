<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# PBR material declarations

`Material.pbr` distinguishes no PBR declaration from an empty declaration.
`PbrMaterial` owns optional metallic and specular workflows and source metadata.
Each workflow retains its kind, texture declarations, roughness, metalness,
glossiness and light-map UV set. Both workflows may coexist. Repeated workflow
kinds replace the earlier one, matching the native Pbr loader.

Maps remain unresolved strings. Parsing performs no texture IO, material
conversion or rendering. Clone preserves owned values after the input and parsed
document are dropped. Exhaustive Material struct literals now need `pbr: None`.
The simulator's producer pin is unchanged.

Native defaults are roughness and metalness 0.5, glossiness zero, empty maps and
UV set zero. Values outside [0,1], NaN and infinities are metadata, not clamped
rendering inputs. The native Param getter substitutes zero on invalid/range-error
scalar conversion; the parser follows that behavior but propagates allocation
and unsupported-platform errors. UV sets use native unsigned numeric prefixes,
boolean spellings and wrapping conversion. Empty UV attributes use zero.

`normal_map_type` preserves the optional XML attribute verbatim. This is source
metadata rather than an implementation of native `NormalMapType()`: the pinned
native PbrWorkflow loader never updates that getter from XML, leaving it Tangent
even for `type="object"`. No rendering behavior is selected by this library.

The unchanged material_pbr.sdf fixture and every Material.PbrDOM assertion are
ported, including metallic, specular and combined cases, names, diffuse colors,
workflow kinds, all maps, scalar values and light-map UV sets. Original color
comparisons retain their component-wise 1e-6 bound; exact f64 bits strengthen
EXPECT_DOUBLE_EQ. Generic Element queries in Material.MaterialPBR, serialization,
URI resolution modes and renderer output remain unqualified. Unexpected workflow
fields/attributes, nested leaf content and repeated scalar/map children remain
explicit errors; native permissive unknown/schema handling is not claimed.

Reproduce the existing-build native evidence with:

```
python3 tools/pbr_reference.py /path/to/pinned/sdformat
cargo test --locked --test pbr --test pbr_original
```

The helper checks the unchanged fixture and SDK headers, runs the original native
test and reproduces thirty supplementary native cases with exact scalar bits and
strings. Its small ignored report records the compile command and loaded-library
hashes. This qualifies the existing build, not a new reproducible reference image.
Independent source review, four Rust tests and all native observations passed.
Strict staged coverage and hosted CI remain required before acceptance. No parsing or rendering performance improvement is claimed.
