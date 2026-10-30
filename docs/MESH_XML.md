<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Mesh metadata XML

`Mesh::to_xml`, `load_mesh_str`, and the Mesh arm of the Geometry XML APIs
preserve URI, scale, submesh name/centering, optimization and optional convex
decomposition counts. They reuse the existing mesh reader without resolving a
URI, decoding an asset, or creating rendering or physics resources. Full document
admission and source locations remain unchanged.

Omitted URI follows the existing document default `__default__`; an explicit
empty URI fails. Native default-constructed Mesh emission instead returns a
nonnull Element and error 34 before loading that default URI. Rust does not
imitate this invalid partial producer. Missing and explicit-empty input, valid
loaded defaults and invalid edited values remain distinct.

Scale accepts all finite components, including negative values, signed zero and
finite subnormals. There is no new physical dimension policy. Optimization uses
the existing None/convex-hull/convex-decomposition mapping; decomposition counts
retain the existing unsigned parser behavior, including zero and u32::MAX.
Decomposition is independent of the optimization attribute.

XML escaping preserves markup and literal `]]>` text. A single CDATA section
preserves internal repeated spaces, tabs and line feeds through native Param;
outer ASCII whitespace and forbidden XML characters are rejected. Repeated
whitespace combined with `]]>` is explicitly Unsupported because native Param
truncates split CDATA. Unicode nonbreaking spaces remain valid. An empty submesh
with centering enabled, or the reserved submesh name `__default__`, is rejected
rather than silently dropped. Existing stricter exact-three, unknown-payload and
duplicate-node reader rules retain their recorded native differences.

The two exact Mesh ToElement originals check valid configured metadata, returned
nonnull output, successful reload, enum/string optimization, vector components
within 1e-3 and decomposition presence/counts. The error-output original also
checks no decomposition, empty errors and actual quiet output. Result of nonempty
XML plus successful reload maps returned output/error assertions. Neither case
asserts input Element identity. The Rust ports preserve every field predicate and
the quiet case captures both stdout and stderr in a child process.

Root independently approved replacement of exactly three temporary Rust-only
Geometry Unsupported predicates before they were edited. The same corpus XML,
full collision document, omitted-URI XML and caller URIs remain; complete native
Mesh observations, full-document ancestry and source, immutable output and
readback replace those predicates. Heightmap/Polyline and unrelated failures are
unchanged. Both Geometry originals remain partial: their invalid default Mesh
producer error assertions are still unimplemented. This valid Mesh capability
adds no Geometry complete credit.

Author checks pass all 512 Rust tests and all-target Clippy. The new helper passes
28 native inputs, two byte-identical configured fixtures, three exact old-input
controls, 18 actual Rust/native exchanges and both unchanged Mesh originals with
empty stderr. Existing Geometry proof passes 48 inputs and now 44 actual exports;
Contact regression passes 16 inputs, 17 actual exchanges, the producer limitation
and unchanged DOMLink.Sensors after extraction of the shared text emitter.
Independent fresh-source review passed 30 relevant Rust checks and all the native
proofs above. Every original predicate, including successful literal optimization
string parsing, was audited. Private stdout/stderr injections each failed quiet
capture; restored source passed all 30. Shared text extraction preserves accepted
Contact output because its unchanged validator rejects tabs/line feeds. Strict
local/hosted gates remain pending. Candidate
193 complete/27 partial remains separate from accepted 191/27. No asset, runtime,
performance or pointer-identity claim follows.

Replay with `CARGO_TARGET_DIR` set to an owned target:

```
python3 tools/mesh_xml_reference.py ../sdformat
python3 tools/geometry_xml_reference.py ../sdformat
python3 tools/contact_xml_reference.py ../sdformat
```
