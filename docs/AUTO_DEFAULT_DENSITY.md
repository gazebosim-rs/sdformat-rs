<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Default density in bounded automatic inertia

A single origin box or sphere with bare inertial `auto=true/1` uses the existing
effective collision density. Omission selects 1000 kg/m³ and keeps
`Collision.density == None`. An empty or whitespace density element selects the
same value while retaining `Some(1000)`. Explicit values retain their presence.
The loader calls the existing public Parry mass-properties kernel and unchanged
manual physical-inertia validator; it adds no formula, dependency or public API.

This extends only the density fallback. Explicit collision pose, multiple
collisions, link density, auto parameters, manual inertial overrides and other
shapes remain Unsupported. Existing normalization, scalar conversion, numeric
Unsupported extremes and manual physical-policy limits remain unchanged.
The resulting Inertial is computed during loading; later metadata edits do not
recalculate it or rebuild semantic graphs.

Pinned native `Collision::CalculateInertial` prefers explicit collision density,
then optional link density, then its default. Only the existing collision/default
branches are represented here. The native corpus has 86 bounded observations:
the exact two old inputs, absent/empty/whitespace/explicit values for box and
sphere in SDF 1.4–1.12, four strict-warning cases and eight invalid densities.
All source, compiler dependency, schema and actual mapped-library inputs are
recorded by `tools/auto_default_density_reference.py`.

Native default warning policy loads omitted density with a warning. Strict
warning policy returns ELEMENT_MISSING while retaining native DOM. Empty density
has element presence and produces neither missing-density warning nor that
strict-policy error. Rust has no warning-policy or partial-DOM-on-error API;
the Rust differential covers the 82 default-policy observations only. The native
replay executes all 86. Raw warnings are retained, not described as clean output.

The two `auto_box_inertial.json` rows named `default_density` and `empty_density`
remain byte-identical, including their historical Unsupported labels. Reviewed
positive branches now use those exact XML strings and `auto-box.sdf` URI, checking
the complete Model/Link/Collision defaults, density presence, source/version,
mass/tensor/pose and frame queries. Independent prior review approved only those
two same-input transitions, conditional on the prepared positives passing; the
focused execution and final narrow fold review then passed. Other old assertions stay
unchanged. Box comparisons keep the existing four-ULP convention; sphere
comparisons reuse the existing normal-forward oracle at density 1000/radius 1.
No retained original tolerance changes. `tools/auto_default_density_oracle.py`
checks the fixed-input intervals using the existing 200/400-digit oracle.

The unchanged full native `DOMLink.ResolveAutoInertialsWithDifferentDensity`
passes. Its translated collisions and link-density cases remain outside this
Rust slice, so that native execution gives no complete or partial Rust-original
credit. This is standalone producer work, with no Fuel or consumer-pin change.

Native characterization and oracle checks pass. The initial native map-validator
key mismatch is retained separately; only the new harness was corrected.
All four new focused tests and selected Clippy passed before folding. The five
affected tests and selected library/two-test Clippy then passed after the fold.
The initial complete hook passed 50 tooling tests, formatting and all-target
Clippy, then failed its instrumented Rust test command. Its temporary failure
log was not retained. A single authorized persistent diagnostic on the unchanged
staged tree compiled successfully and captured `no_density accepted` in the old
sphere corpus test. That diagnostic is failure evidence, not a qualification
retry or reconstruction of the original lost log.

A source audit also found the sphere `empty_density` and manual-inertia
`true_valid_box` expectations depend on the retired density refusal. All three
corpus rows, XML strings and URIs remain unchanged. Independently source-reviewed
positive branches now check the exact radius-0.1 sphere inputs using
`sphere.sdf` and the unit box named `m/l/c` using `manual.sdf`. The shared complete
helper retains each input's names, geometry, defaults, density presence,
source/version, frame queries, clone independence and full inertial pose/tensor.
The manual corpus's automatic box checks native flags `[1, 1]`; it does not fall
through to the unrelated manual-value branch's `[0, 0]` assumptions. Existing
corpus counts and all other branches remain unchanged. The three duplicate
prepared wrappers were removed; the retained corpus tests call the same complete
helper directly. Source review approved these transitions from retained native
evidence; no preliminary runtime pass is claimed.

The radius-0.1 sphere inputs use their own fixed-input 200/400-digit oracle
record, generated with the existing maintained reference tool and the existing
normal-forward operation-count bounds. The radius-1 intervals are not reused.
The new validator checks normal intermediates/reciprocals, both native sphere
projections and outward-rounded intervals without changing a retained tolerance.
The consolidated reference check passed both fixed-input 200/400-digit oracles
and all 40 stored native sphere projections. The mandatory hook then passed all
50 tooling tests and stopped at formatting before Clippy or the Rust suite.
Formatting and an equivalent `if let` helper cleanup form one reviewed correction
batch; the three folded positive paths still await mandatory runtime verification.

The complete retained suite, ordinary strict gates and exact hosted/source
qualification remain pending. No new original-case credit is claimed.
