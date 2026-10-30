<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->
# Collision metadata XML

`Collision::to_xml` and `load_collision_str` handle the currently supported
Collision metadata without resolving poses or resources or computing physics.
The standalone reader reuses the document parser and its strict scalar,
cardinality and unknown-payload rules. The writer delegates shape and surface
output to their existing bounded writers. Source positions describe the input
and are not preserved as Element pointer identity in generated XML.

The writer emits actual `Geometry::Empty` as `<empty/>` and retains optional
density presence: `None` omits it and `Some(1000.)` emits it. Signed zero and
negative density remain metadata; no physical admissibility is implied. A
nonfinite or subnormal density returns an explicit error, preserving the stored
bits in the original object. Native's subnormal setter retains its value but
its emitter logs an error and substitutes 1000; Rust does not manufacture that
loss. Pose must satisfy the existing finite unit-quaternion rules and have no
subnormal components. Name and relative-frame attributes must roundtrip through
the existing XML policy. Geometry and Surface impose their own output limits.

The original `DOMCollision.ToElement` setup uses default Empty Geometry, pose
(1,2,3,.1,.2,.3), mask 123 and ODE mu 1.23; it first omits density and then
sets 1234.5. The original reloads the first element again after setting density,
and checks the original Collision's getter; the test retains that exact order
and adds a separate roundtrip of the second element. `ToElementErrorOutput`
uses the same setup and requires successful output/load plus no console text;
the separate Rust case captures actual stdout and stderr. Owned Geometry,
Surface, Contact, Friction and ODE values supply the safe nonnullable access
corresponding to native pointer assertions. This is not pointer identity or an
unrestricted Element implementation.

The native helper verifies pinned source, installed headers and libraries,
regenerates the constructor fixture byte for byte, replays both unchanged
native originals and sends two actual Rust outputs through native loading.
Focused Rust checks cover both original cases, strict errors, density bit and
presence states, delegated shape/surface behavior and native fixture loading.
Independent fresh-source review reproduced six Rust tests, the exact native
fixture, two exchanges and both unchanged originals with no findings. Normal
staged/exact hosted qualification remains pending; no ledger credit is claimed. Run `python3 tools/collision_xml_reference.py ../sdformat`.
