<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Surface metadata XML

Surface::to_xml and load_surface_str roundtrip the supported contact masks and
ODE/Bullet/torsional friction data. The standalone reader reuses the document
surface parser; its finite, nonnegative, unknown-payload and singleton policies
are unchanged. It creates no collision, frame graph or physics engine. Bullet
fields remain file-format metadata even though the simulator drops that backend.

Output always includes the current collision mask and ODE values. It writes the
category mask and Bullet/torsional groups only when present. Optional group
presence and every numeric field survive reload. Source metadata is immutable
input-origin data and is not emitted or treated as native pointer identity.

Finite scalar values must survive the existing GNU numeric reader; subnormals
and nonfinite scalars are output errors. Nonnegative fields retain their existing
schema bounds. Slip and rolling friction keep supported negative values; fdir1
vectors remain unnormalized, including finite subnormal components and signed
zeros. The native corpus distinguishes reader policy differences: native scalar
NaN/Inf, ignored bounce/contact extras, unknown payloads, trailing vector tokens
and duplicate singleton selection are not claimed as Rust admission parity.
No new surface fields are admitted or silently dropped.

The two exact native Surface.ToElement originals require nonnull returned output
and field reload; the error-output case also requires no errors/console text.
Successful nonempty XML and successful reload translate those output assertions.
All scalar four-ULP checks and direct component <=1e-3 vectors are retained, with
actual stdout/stderr child capture. Neither original asserts input Element
identity, so no origin equivalence or generic DOM layer is needed.

The helper regenerates two native fixtures byte for byte, replays 34 native
inputs and sends 17 actual Rust outputs through the native loader. These cover
all supported groups/masks, finite endpoints, signed zero, subnormal directions,
negative supported slips, malformed fields and explicit existing policy limits.
Both unchanged native originals pass with empty stderr. Independent fresh-source review passed all 19 relevant Rust checks and the full
native helper; every original predicate and output/quiet mapping was checked.
Private stdout/stderr injections each failed the quiet wrapper, then restored
source passed all 19 again. All 501 author Rust tests, 17 tooling tests, formatting and
Clippy pass. Normal strict staged/hosted qualification remains pending: candidate
191 complete /25 partial versus accepted 189/25. No contact simulation or
performance claim.
