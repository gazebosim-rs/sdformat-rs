<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Bounded automatic sphere inertia

A bare inertial auto=true/1 supports one origin sphere collision with positive
effective density, alongside the existing box domain. Explicit-density
qualification is described here; AUTO_DEFAULT_DENSITY.md covers fallback. The loader reuses
Parry 0.31.1's public `MassProperties::from_ball`, `mass` and `principal_inertia`.
No dependency, public type, handwritten mass formula or runtime is added. Units
are metres, kg/m³, kg and kg·m². Pose is the link origin, products of inertia are
zero and fluid added mass is absent. Original version and source positions stay
unchanged in SDF 1.4–1.12, including imported resources and frame placement.

The existing sphere parser supplies normalized geometry: omitted radius is 1,
and nonpositive XML radius normalizes to 1 with a native warning. Native Root
computes inertia from that normalized sphere. Empty radius stays Missing.
This differs from the native mutable Sphere setter: setting radius -2 makes
CalculateInertial return nullopt. The loader does not claim setter semantics.

The subsequent default/empty-density extension is described in
AUTO_DEFAULT_DENSITY.md. AUTO_IDENTITY_POSE.md describes the prepared local
identity-pose extension. Nonidentity or attributed collision poses, multiple collisions,
other shapes, link density, auto parameters and manual inertial overrides remain
Unsupported. Numeric extraction must be finite and positive and pass the
unchanged manual physical-inertia validator. Native-valid density DBL_MIN with
radius 0.5 loses public inertia through reciprocal overflow; density DBL_MIN
with radius 3e102 overflows Parry's intermediate volume despite finite native
mass/inertia. These return Unsupported, not malformed-native-input errors.
The retained manual determinant policy separately rejects radius 1e-30 although
native accepts it. There is no arbitrary radius or density cutoff.

## Original assertions

`DOMSphere.CalculateInertial` uses density 2170 and radius 0.1 with exact EXPECT_EQ
mass, three diagonal moments and pose. All those values remain bit-exact in the
new derived loader fixture: mass bits 4621306650028725639 and diagonal bits
4585400661777136886. No four-ULP allowance is substituted. The full unchanged
native original passes, including the invalid setter and material mutation.
Only its positive inertial value observations map to Rust loading. The mutable
Sphere/Material API, negative setter and nullopt behavior are not ported by this
slice. The original has no XML fixture; the new XML is explicitly derived.

The accepted box corpus remains byte-identical. Independent prior review
approved only its sphere row's rejection-to-positive transition using the
exhaustive same-input/auto-box.sdf helper. Every other retained assertion,
corpus, tolerance and policy stays unchanged.

## New differential cases

The sphere operation order differs between maintained implementations. A
five-ULP discrepancy occurs at density 7917.05362631725, radius 0.4926696955895169.
It is retained as a successful new case. The box sample's four-ULP observation
is not a sphere contract, and does not justify rejecting this geometry.

Let u=2^-53 and γ(n)=nu/(1-nu). Relative error is measured against mathematical
mass 4πρr³/3 and diagonal inertia 2mr²/5 for the exact binary64 inputs.
These formulas occur only in the independent reference tool, not the loader.
For round-to-nearest normal forward intermediates and normal positive extracted
outputs, the new cases use the following conservative operation-count bounds:

| Implementation | Mass | Diagonal inertia |
| --- | --- | --- |
| Pinned native | γ(5) | γ(9) |
| Public Parry, normal stored reciprocal | γ(8) | γ(11) |
| Public Parry, subnormal stored reciprocal | γ(11) | γ(14) |

Each component chooses its own reciprocal class. The native–Parry absolute
comparison uses exact mathematical value times γ(sum of the two counts).
Stored test interval endpoints and absolute bounds round outward to binary64;
the extra endpoint rounding is explicit. Zero products, pose and retained
original values remain exact. The normal-intermediate qualification applies to
these new numerical test claims; it is not a production admission cutoff.

The pinned native library's disassembly evaluates a folded 4π/3 coefficient,
one libm pow(r,3), volume multiplication and density multiplication. Radius²
is a multiplication, followed by two inertia multiplications and the rounded
0.4 constant. The fixed folded coefficient's relative error is 0.988u and the
0.4 constant's is 0.5u. Counting one, two, one, one units for native mass gives
γ(5); four additional contributions give γ(9). For normal pow results we use
2u, conservatively above the documented 0.54-ULP error in
[glibc 2.39 pow](https://raw.githubusercontent.com/bminor/glibc/glibc-2.39/sysdeps/ieee754/dbl-64/e_pow.c).
The actual selected Ubuntu libm binary/version is separately recorded; no claim
is made that an installed binary is byte-identical to upstream source.

Parry evaluates π*r*r*r*4/3, then density. Rounded π, three radius products,
division by 3 and density multiplication give γ(6); multiplication by 4 is exact
in this qualified regime. The unit inertia r*r*2/5 contributes two more units,
and multiplication by mass gives γ(9). Multiplication by 2 is exact. Storage
and public extraction each take a reciprocal, contributing two additional
normal divisions. A subnormal stored reciprocal has at most 4u relative error:
for any finite positive f64, its exact reciprocal exceeds 2^-1024, while half
one subnormal ULP is 2^-1075. The normal extraction adds at mostu, giving five
instead of two contributions. No inverse-square-root reconstruction is used.

An independent mpmath 1.2.1 oracle evaluates exact binary inputs at 200 and 400
decimal digits and checks stable rounded reference values. The 1,006-row
operation-trace evaluation matches public Parry bit-for-bit. Its 842 qualified
rows have maximum native errors 3.34u/3.70u, Parry 3.68u/3.63u and differential
5.16u/6.05u, within the derived bounds. This is sampled build qualification,
not a guarantee for unexamined non-normal regimes. The checked-in 88-case DOM
corpus includes 56 oracle cases; 55 are admitted, while the retained manual
physical policy rejects the tiny determinant case. The optional reference tool
rechecks the independent bounds without adding a production dependency.

This producer slice does not enable simulator automatic inertia: Scene's
existing producer guard rejects explicit collision density. Consumer review is
required before that guard or a dependency pin changes. No complete automatic
inertia, mutable Element, separate ResolveAutoInertials or simulator credit is
claimed. Independent msgs_performance_owner fresh source/own target passed
seven focused Rust tests, native88 plus the unchanged original, all-target
Clippy/fmt and a fresh 1,006-row public/native trace. Independent 400-digit
replay verified all842 qualified cases and56 stored oracle intervals. All592
old test/tool inputs and dependency/license inputs were unchanged before the
single prior-approved fold. Final fold review and ordinary strict/hosted gates
remain pending.

AUTO_TRANSLATION_POSE.md describes the subsequent Rust-unverified local-translation
batch. The existing sphere kernel and all prior roundoff/original
assertions remain unchanged; new aggregate observations are not yet qualified.

A separate [origin-centered rotation draft](AUTO_ROTATION_POSE.md) is source-only.
It does not change this accepted evidence or qualify broader pose families.
