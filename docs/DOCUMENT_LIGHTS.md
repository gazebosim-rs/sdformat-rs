<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Direct document lights

Full SDF loading retains direct World and Link lights in declaration order.
Names must be unique within each light collection; sharing a name with another
entity category remains allowed. Light source metadata identifies the actual
input declaration. Scene, GUI, wind, projector and other unsupported payloads
remain explicit errors; no rendering or light-include support is claimed.

## Native loader contexts

One shared Light parser has an explicit private context. The standalone API
retains its existing constructor defaults, required direction/range checks,
numeric policies and setters. Document loading follows native schema-backed
semantics: type is required, diffuse defaults to white and specular to
[.1, .1, .1, 1], including empty color nodes. Omitted direction defaults to
[0, 0, -1]; an empty or malformed explicit direction rejects. Empty attenuation
supplies range 10, while an explicitly empty required range or spot value rejects.
Optional empty intensity retains default 1.

The native point getter ignores supplied direction, and only Spot consumes spot
parameters. All supplied known values are still parsed and validated before
irrelevant getter values are ignored; unknown payloads never become defaults.
Existing numeric conversion and finite-vector policies stay unchanged. The five
fields governed by native std::max(0, value) select +0 in document mode; the linear
and constant clamps preserve -0. Standalone behavior is unchanged.

## Pose behavior

The existing frame graph and Rust math library resolve retained poses. Callers
supply the containing scope, host frame and fully scoped target to resolve_pose.
World lights default to their world frame; Link lights default to the Link frame.
A nonzero Link-offset control distinguishes that target from the model frame.
Native local target names map to their containing scope: model-local `world`
remains an unknown local frame, not a shortcut to the global world.

Loading defers unresolved light-pose references, as native loading does.
validate_non_frame_poses now checks these lights under its existing explicit
consumer policy. No new graph algorithm, live mutation binding or Light query
facade is introduced.

## Original fixture and assertions

The LoadLights port retains the SDF 1.8/world/model/link ancestry, four complete
Light subtrees, model pose and two frame subtrees from world_complete.sdf.
Unasserted unsupported siblings are omitted. No direction or other semantic field
is appended. A fresh native helper produces identical complete observation streams
for the full original and this projection: all 33 getters per light, model raw
metadata, eleven original semantic queries and two unknown-target controls.
The unchanged native original also passes against the full original fixture.

The Rust port preserves every original presence, indexed getter, raw value,
and frame/default resolve observation. Pose comparisons retain direct
signed component bounds <=1e-3; EXPECT_DOUBLE_EQ fields retain four ULP.
No unrelated original assertion is retired or extra case credit inferred.

## Missing-type disposition and evidence

Both exact old document-light rejection inputs omit required type. Native Root
rejects them with ATTRIBUTE_MISSING 4 and wrapper errors. They remain negatives;
new explicit-type positives are separate inputs. The precise Rust Missing classification preserves both old XML inputs and
failure assertions, with literal caller URI/line/column checks. Independent narrow
review approved this refinement before replacing the generic Unsupported
predicate; the redundant uncommitted prepared test was folded into the old test.

All 492 Rust tests, formatting and Clippy pass. The reference helper replays
42 complete schema field cases, eleven document
controls, raw-versus-schema direction/range profiles, full/projection equivalence
and the unchanged original. Invalid schema and shared-name controls retain their
actual native stderr; no blanket empty-console or diagnostic-count parity is
claimed. Final independent review passed 41 Rust checks in fresh source/target, all native
cases/profiles/full-projection streams and the unchanged original. It corrected a
missing explicit Spot-type assertion in the direct port; all original predicates,
pose/double bounds and context/error semantics are now reviewed complete. Prior
twelve-family helper replays, 211 unchanged XML exports and configured Spot bytes
remain author evidence. Normal strict staged/hosted qualification remains pending. Candidate 189 complete /24 partial is separate from accepted
188 complete /24 partial.
