<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->
# Collision density metadata

`Collision::density: Option<f64>` retains whether a density was explicitly
supplied. `Collision::DENSITY_DEFAULT` is 1000 kg/m³ and `density()` returns the
explicit value or that native default. Public edits and deep clones preserve the
stored bits. This is metadata; loading does not calculate mass, inertia or forces.

A missing density remains `None`; an empty density element is explicit
`Some(1000.)`. Zero, negative values and signed zero are admitted as in native
metadata. The existing scalar converter supplies finite prefix conversion and
source-located errors. Existing strict duplicate/unknown-attribute/unknown-child
rules apply. NaN, infinity, malformed and subnormal XML values remain rejected
under the established scalar policy. Native accepts nonfinite XML, chooses the
first duplicate and ignores some unknown payloads; these are explicit differences.

The pinned native helper records 14 complete schema/load/getter/emission/reload
observations and ten setter states. A native subnormal setter retains its value
before emission, but native `ToElement` logs a conversion error and emits 1000
with an empty returned error list. This loss is recorded, not imitated. Direct
Rust metadata edits remain lossless; no Collision writer is provided by this
slice. A later writer must reject unrepresentable states explicitly.

Run `python3 tools/collision_density_reference.py ../sdformat` from this checkout
with its own Cargo target. The helper checks source/header/library identities,
replays every full native stdout/stderr observation and runs 15 focused Rust
checks. These cover presence, effective values, original Collision/Link ownership,
clone independence, exact sign bits and retained scalar/structure failures. The
caller URI and literal Collision source position are checked independently.
No unchanged native original is promoted by this capability; accepted counts
remain 201 complete and 27 partial. Independent review reproduced all 14 native profiles, ten setter states and 15
focused Rust checks with no findings. Normal strict/hosted qualification is pending. No physics interpretation, auto-inertia, simulator
consumer pin or Element identity behavior is claimed.
