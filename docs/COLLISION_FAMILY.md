<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->
# Collision metadata original cases

The five already mapped ownership cases now start with `Geometry::Empty` and
an empty name, as the pinned `Collision_TEST.cc` constructors do. The earlier
Rust helper used a Box although those originals only asserted pose. That Box
was a stronger different setup, not the original construction. Existing pose
assertions, signed component tolerances and supplemental ownership checks remain.
A separate supplemental test preserves replacement and copy assignment into the
previously configured Box/name/pose destination.
Rust's required `Source` is synthetic for these unattached values; it does not
represent native Element pointer identity. The original-default destination in
assignment cases now has a real default pose and Empty geometry.

The accepted standalone writer covers the complete `DOMCollision.ToElement`
and `DOMCollision.ToElementErrorOutput` setups: default Empty geometry, name,
pose, contact mask 123 and ODE mu 1.23. It retains the original's omitted and
explicit density sequence, including reloading the first element again and
checking the original object's getter. Successful XML/loads map nonnull and
empty-error predicates; owned values provide safe nonnullable geometry, surface,
contact, friction and ODE access. The quiet-output case captures child stdout
and stderr. A second-output density roundtrip is supplemental. Two case mappings
are candidates for 203 complete/27 partial; normal staged and exact hosted
qualification remain necessary.

The remaining Collision family is not complete:

- `DOMcollision.Construction` also requires a native-null Element, an
  auto-inertia parameters Element with pointer identity and unattached semantic
  pose resolution behavior; a required Rust `Source` is not that pointer.
- `DOMcollision.SetGeometry` starts with native-null Element and sets type BOX
  on a Geometry with no Box payload. Native stores type and optional shape
  separately; Rust's `Geometry::Box` always includes a size, so inventing one
  would change the setup. `SetSurface` also asserts native-null Element on a
  default-constructed Collision, which this bounded API cannot represent.
- Five `CalculateInertial` originals depend on automatic inertia, error/failure
  policies and Element/pose behavior outside this metadata writer. They are not
  promoted or approximated here.

The five unchanged ownership originals and two unchanged writer originals pass
on the pinned native test binary. The accepted writer's separate native helper
records the constructor fixture, two actual Rust-to-native exchanges and
original-test binary/library identities. Run `python3 tools/collision_family_reference.py
../sdformat` to replay the full family evidence. Independent review replayed all 13 Rust checks, seven unchanged native originals
and the family helper without findings. Normal staged and exact hosted gates
remain required before accepting the candidate mappings.
Broader SDF-02 version/schema conversion and merge/include behavior remain open;
this family reconciliation does not replace those priorities.
