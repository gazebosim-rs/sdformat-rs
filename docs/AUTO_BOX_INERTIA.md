<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Bounded automatic box inertia

A bare `inertial auto="true"` or `auto="1"` computes owned mass and diagonal
inertia for exactly one box collision with an explicit, nonempty positive
density and an omitted collision pose. The existing boolean parser also keeps
its case/padding behavior. SDF1.4–1.12 retains its original input version and
source positions. Units are metres, kg/m³, kg and kg·m². Inertial pose is the
link origin, products of inertia are zero, and fluid added mass is absent.
No automatic-state getter, mutable Element or separate resolution API is added.

The existing geometry loader remains authoritative: omitted box size is (1,1,1),
and a nonpositive size normalizes to (1,1,1), matching native Box::Load. The
full native Root computes automatic inertia from that normalized size. Empty
size remains an error. Native normalization warnings are retained in the
reference corpus but not reproduced as Rust console output.

Default collision density, an empty density, any explicit collision pose,
multiple collisions, shapes outside the box or bounded sphere domain, link-level density, auto parameters and
manual inertial overrides remain Unsupported. Manual false/default behavior
and every old assertion/corpus remain unchanged, including the old valid
unit-box refusal whose density is omitted. There is no collision aggregation,
parallel-axis calculation, mesh IO or runtime physics operation.

The implementation calls maintained `parry3d-f64 =0.31.1` with default features
disabled and only `required-features` enabled. Public cuboid mass properties
use half extents and store reciprocals. No local mass/inertia formula is added.
Finite positive extracted results pass the unchanged manual physical-inertia
validator. Nonfinite or nonpositive kernel results return Unsupported: native
valid unit boxes with density DBL_MIN or DBL_MAX demonstrate representational
loss (zero extracted inertia or infinite extracted mass). They are not labeled
malformed native inputs. The existing manual physical-policy check separately
rejects determinant underflow for a 1e-30 m box; native accepts that box. This
preexisting stricter policy is preserved rather than advertised as native error
parity.

The byte-identical `DOMCollision.CorrectBoxCollisionCalculateInertial` fixture
retains explicit density 1240, box dimensions (2,2,2), mass 9920 and all inertia
bits exactly. Supplemental native comparisons use the native four-ULP
EXPECT_DOUBLE_EQ convention for positive mass/diagonal values; zero and pose
bits stay exact. Root.Element identity and a separately invoked
ResolveAutoInertials/error-vector contract remain unported; this is partial
loader/value coverage, not completion of that original or automatic inertia.

The dependency evaluation compiled on unchanged Rust 1.94.1 with 32 registry
packages (24 added version identities, all old lock identities retained).
Its isolated debug0/jobs2 build took 9.04 seconds and allocated about 123 MiB;
this is build-cost evidence, not a runtime performance comparison. All 32
crate archives/checksums and 1,684 extracted source files were checked. License
metadata is retained in `reference/auto-box-kernel.json`; Parry is Apache-2.0.
Parry and rstar omit root license texts from their published archives, so exact
upstream license-text URLs/digests are recorded separately. The published Parry
archive reports a dirty VCS snapshot; its registry archive checksum and actual
source files, rather than a clean upstream commit claim, bind the evaluation.

A 2,067-row public-kernel/native-math evaluation found the retained box exact;
all 1,916 rows with positive finite outputs on both sides were within 4 ULP
(maximum mass 1/diagonal 3). The actual Rust loader admitted 1,792 positive-input rows, all native-valid and
within 3 ULP, plus two nonpositive-size inputs through existing normalization. These are
sampled correctness observations, not a proof for every floating-point input.
The checked-in 53-row native DOM corpus covers versions, normalized geometry,
values, strict scope and numeric refusals. Six focused and all 676 retained Rust tests pass, as does the unchanged full
C++ box original. Independent msgs_performance_owner fresh accepted1b source
and own target passed six focused Rust tests, all-target Clippy, formatting,
53 native controls, the unchanged full original and a hostile-Git replay.
Archive/source/license and unchanged old-input checks pass. Public extraction
is bit-identical for all 2,067 rows on minimal Rust 1.94.1, std/enhanced
Rust 1.97.1 and the actual simulator math feature graph; the reviewer also
replayed the actual loader admissions. Root independently inspected the
bounded source/dependency/manual-validator scope. The mapping remains partial;
ordinary strict staged and exact hosted/source gates remain pending.

This is standalone producer work. Current Scene admission explicitly refuses
Some(collision.density), so every newly admitted automatic box is rejected
there. Computed Inertial and retained density require consumer review before
that guard or a dependency pin changes. This slice changes neither.

The subsequent bounded sphere extension is documented in AUTO_SPHERE_INERTIA.md;
its new numerical comparisons do not change any retained box tolerance.
