<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Primitive geometry XML

Geometry::to_xml and load_geometry_str support Box, Sphere, Cylinder, Capsule,
Cone, Ellipsoid and Plane metadata. The standalone reader delegates to the
existing geometry parser. Valid Mesh metadata is added by [MESH_XML.md](MESH_XML.md);
Valid Heightmap metadata is added by [HEIGHTMAP_XML.md](HEIGHTMAP_XML.md);
Valid ordered Polyline metadata is added by [POLYLINE_XML.md](POLYLINE_XML.md).
Unrepresentable edited output states continue to fail explicitly.
Full document loading keeps its existing complex-shape admission and defaults.
No resource IO, collision creation or physics validation occurs here.

Output requires positive finite dimensions that the corresponding reader can
retain. Scalar dimensions reject subnormals under the existing GNU numeric
reader; vector dimensions retain positive finite subnormals, including the
smallest value. Nonpositive document input still follows existing native fallback
semantics. The writer emits the resulting current value, not original invalid
XML or its native console messages. It rejects invalid edited dimensions.

Plane normals are emitted raw and do not mutate the value. Reload follows the
existing normalization policy: zero/tiny normals through 1e-6 remain unchanged;
ordinary vectors normalize; an overflowed finite length divides to zero. Finite
subnormal components and signed zeros are retained on the qualified paths.
There is no new epsilon or unit-normal admission gate. Edited raw normals are
not promised a bit-identical reload: native SetNormal/getter and subsequent
Element reload profiles are recorded separately, and actual Rust outputs are
loaded by native. No broader numerical or physical validity follows.

Seven native constructor fixtures preserve real defaults. Cylinder/Cone
constructors use radius 0.5, whereas omitted document radius uses 1; Capsule uses
0.5 in both contexts. These distinctions do not change existing Rust defaults.

The original Geometry.ToElement and ToElementErrorOutput cases have ten shape
blocks. Only the seven supported blocks are mapped: type equality, selected
shape presence, every other shape's absence and empty Polyline are represented
by the single Rust enum variant plus native presence masks. Successful nonempty
XML/reload maps output/error predicates; the selected error-output body uses
actual stdout/stderr child capture. Additional numeric and direct native-fixture
checks are stronger controls, not extra original credit.

Mesh/Heightmap/Polyline blocks remain unported. The first native original prints
two required-URI errors for default Mesh/Heightmap producers; the error-output
original explicitly expects those errors while also checking nonnull output.
Both unchanged full native cases pass, with their distinct stderr retained.
This API does not imitate invalid producer partial output, an empty/None geometry
or a pointer-origin facade. Both exact originals stay partial.

All 507 author Rust tests, 17 tooling tests, formatting and Clippy pass. Native
evidence replays 48 inputs, seven exact
fixtures, six Plane setter/reload profiles and 43 actual Rust/native XML exchanges,
including six raw edited normals. Independent fresh-source review passed all
23 relevant Rust checks and the full native helper; all selected original
assertions and normalization/default/error boundaries were checked. Private
stdout/stderr injections each failed the quiet wrapper; restored source passed
all 23 again. Normal strict/hosted qualification remains pending. Accepted counts are 191 complete /25 partial;
candidate 191/27 adds two partial mappings only, with no runtime or performance claim.
