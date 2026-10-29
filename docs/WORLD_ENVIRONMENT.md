<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# World wind and magnetic metadata

World exposes wind_linear_velocity in m/s and magnetic_field in Tesla. These are
metadata in the declared world coordinate system; the loader does not apply wind
forces, synthesize sensor readings or convert spherical coordinates.

Omitted wind, an empty wind group and an empty/whitespace linear_velocity use
[0, 0, 0]. The empty velocity node is still validated for unknown attributes and
children before its default is used. Omitted magnetic_field uses the schema
[5.5645e-6, 22.8758e-6, -42.3884e-6]; an explicit empty magnetic_field is invalid.
All configured components, including signed zero and subnormals, retain their
bits through the existing finite vector reader. Gravity admission is unchanged.
Native constructed/raw World gravity is -9.80665, while schema-backed document
gravity is -9.8; this document API continues the existing -9.8 behavior.

The existing Rust policy rejects vectors with other than three components,
unknown payloads and duplicate singleton elements. Native parsing accepts some
of these with trailing values, warnings or first-value selection. The checked-in
corpus records each native observation and names its Rust error boundary rather
than calling those cases exact parity. Invalid finite/vector errors retain the
caller's declaration location. Scene, GUI and projector payloads remain errors.

The native proof helper compiles against the pinned installed SDK and checks
loaded libraries, source/header identities, 35 real native inputs, constructor/raw
profiles and exact old wind-input Light fields/pose. The projected original fixture
preserves complete wind/gravity/magnetic subtrees and original SDF/World ancestry.
Its nine getter components match the full native world_complete.sdf stream. The
unchanged DOMWorld.Load original passes; only its World/name/vector assertions
are ported here. Audio, spherical coordinates, scene/GUI, pointer identity and the
remaining full original assertions stay unported. No complete original credit.

The exact temporary Rust-only <wind/> rejection in document_lights.rs is replaced
by a native-backed positive with the identical complete input and URI, all nine
World vector values, every existing Light projection field and semantic pose.
Root independently replayed the prepared native/Rust controls and approved this
exact transition before the edit. The separate prepared test is folded into the
existing test; scene/GUI/projector predicates and all other old checks remain.

Author checks pass all 495 Rust and 17 tooling tests, formatting and all-target Clippy,
including eight affected Rust tests, 35 native observations, full/projection
comparison and unchanged original. Root independently replayed the final eight affected Rust controls, all 35 native
cases, raw/constructor profiles, exact full/projection vectors and unchanged
original with fresh source/target/peers; source/error/assertion review is approved.
Normal strict staged coverage and exact hosted CI remain pending. Accepted ledger is 189 complete /24 partial;
candidate ledger is 189 complete /25 partial. The extra partial mapping remains
pending qualification; it adds no complete original credit.
