<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Origin-centered automatic-inertia rotation draft

This source batch starts from accepted producer Main `c6c4bb8`. It extends only
its single box/sphere automatic-inertia domain to a collision pose with numeric
zero translation and ordinary attribute-free Euler rotation. It preserves the
existing parser and raw pose/source metadata. Equivalent decimal/exponent Euler
spellings use that parser; no new pose grammar or literal-angle whitelist is
introduced. An existing unrotated translation keeps its prior extraction path.
Combined nonzero translation and rotation remain Unsupported, as do any raw
pose attributes, frame references, inertial overrides and multiple collisions.
No dependency, compiler, simulator pin or Fuel change is included.

## Library and native contract

Pinned SDFormat `1249805` Collision.cc composes the collision pose with its
geometry inertial. Link.cc adds the collision to a zero-mass Inertial accumulator.
Pinned gz-math `1316ed3` Inertial::Moi forms a quaternion rotation matrix, rotates
the tensor into the link frame, and operator+= returns identity inertial
orientation after the existing weighted-COM and two residual-shift passes.
Rotation remains on the collision metadata; the accumulated inertia tensor is
expressed in the link frame.

Existing Parry 0.31.1 geometry and numeric admission remains mandatory for every
input, and all accepted unrotated/translation paths retain its mass and principal
extraction unchanged. After two observed numerical failures, Root approved a
private selected-SDK mass/principal evaluation only for newly rotated box/sphere
values. Public glam `DMat3` multiplication and transpose perform the subsequent
tensor transformation; no matrix multiplication, parallel-axis, general geometry
or general inertia framework is added. A private Euler-to-matrix adapter preserves
the selected native expression order that the inspected library constructors
do not expose. It reads six already-supported finite pose values with the
existing parser, leaving the public typed Pose and its source metadata unchanged.
Frames, alternate encodings and combined translations remain closed.

The specific library gaps are source-proven: glam Euler construction factorizes
products differently; its quaternion normalization uses reciprocal multiplication
and xyzw ordering; its matrix constructor groups diagonal expressions as
`1-(yy+zz)`. Native uses its four ordered Euler expressions, normalizes in wxyz
order using component division, then normalizes again in its matrix constructor
and computes `(1-2*y*y)-2*z*z` and the other pinned expressions. The private
adapter preserves only this bounded operation contract. Existing
`DVec4(w,x,y,z).length()` has the required ordered four-square sum and square root;
its scalar `/=` performs individual divisions. The original 2012 gz-math Apache
attribution is retained. No new dependency or general rotation API is introduced.

The accepted single-collision accumulation and manual physical validator are
unchanged. Existing kernel numeric gaps still return Unsupported; nonfinite
tensor or COM output cannot be returned as a valid DOM. A finite tensor violating
the inherited manual physical policy remains InvalidValue. The finite public
matrix-determinant guard applies only to newly rotated tensors, conservatively
refusing nonfinite validation arithmetic without a density cutoff.

## Actual failure and one explanatory analysis

The first mandatory hook passed 56 tooling tests, formatting, Clippy and
rustdoc, then failed the instrumented all-targets Rust command. Its temporary
staged export discarded the captured failure output. One authorized persistent
same-tree diagnostic ran both full affected targets with `--no-fail-fast`:
identity 4/4 passed; rotation 3/4 passed and its native differential failed.
For the recorded mixed box, Rust `ixy=6.394884621840902e-14` differed from native
`5.684341886080802e-14` under the unchanged four-ULP assertion.

One standard-library explanatory analysis was limited to that failed input.
Pinned Parry stores reciprocal principal inertia, not an inverse square root.
Its computed principal moment and reciprocal round trip are bit-identical to
the native value for this input. The SDK-ordered quaternion reproduces all
four captured quaternion values; the modeled glam path differs in qz by one
ULP. SDK-ordered quaternion/matrix construction with those same Parry moments
reproduces all nine recorded direct-Moi entries. Inverse-to-transpose alone
leaves the modeled failure unchanged. These observations explain this one
failure; they do not establish parity for every remaining input or arbitrary
floating-point values.

The four-ULP bound, every retained helper, corpus and old assertion remain
unchanged. No new roundoff envelope, case exclusion or custom tensor kernel
is substituted. The existing independent 200/400-digit sphere oracle can
explain an exact isotropic zero product; it is not used to weaken this gate.
That first correction was executed only by the retained failed mandatory hook
reported below; the expanded primitive-evaluation batch later reached the strict classifier failure described below.

## Exact remaining primitive rounding gap

The corrected mandatory hook again passed 56 tooling tests, formatting, Clippy
and rustdoc, then failed the unchanged four-ULP native differential on the final
anisotropic mixed-box case: `iyz=4.6967674437578495` versus native
`4.696767443757793`. Its failure output was retained by the verifier amendment.
No new diagnostic or native capture was run.

One authorized source/standard-library arithmetic analysis covered all nine
entries of this declared input. Mass was 7440 on both sides. Native principals
were `(8060,6200,3100)`; Parry extracted values differed by `(2,0,1)` ULP, within
the accepted scalar budget. SDK-ordered rotation with native principals reproduced
all nine actual direct-Moi bits. The same rotation with Parry principals produced
entry differences `(2,4,1;4,0,64;3,208,1)` ULP and reproduced the actual failure.
Using values before reciprocal extraction still left 64 ULP in the tested entry:
the unit-inertia multiplication order had already rounded away required bits.
This isolates a scalar-provider gap, rather than another rotation/order defect.
The native lower/upper products also differ; neither symmetrization nor choosing
the other triangle is a valid repair.

The public kernel's principal/reconstruction or mass-rescaling APIs cannot
recover these native-ordered values after rounding. Root explicitly expanded
the source scope based on this evidence. Only inside the new rotation branch,
private evaluation follows pinned `MassMatrix3::SetFromBox` / `SetFromSphere`:
box volume is the ordered full-size product, mass is density times volume, and
each principal uses `mass/12` times the corresponding ordered squared-size sum.
Sphere uses the folded `(4/3)*PI` coefficient, standard `powf(radius,3)`, density
multiplication, and `0.4*mass*(radius*radius)`. The selected native disassembly
already recorded in AUTO_SPHERE_INERTIA.md uses one libm cubic power and a square
multiplication; this does not introduce a power algorithm. No new arithmetic
case search or alternate-kernel tuning was performed.

These new mass/principal values must be finite and positive, then pass the
unchanged finite determinant, aggregate and manual physical guards. Parry's
existing conservative numeric refusal runs first. There is no broader numeric,
shape, pose, override or multiple-collision admission. The private adaptation
retains the original 2015 OSRF/Apache notice. A private regression compares mass and all three raw principals to the actual
direct values for every one of the 24 captured inputs. Another source-derived
negative control requires Parry admission first, then explicit Unsupported when
the SDK's ordered size-square or volume intermediate overflows. Those two new
Rust inputs are guard contracts, not extra native observations or case exclusions. Existing regression controls still consume all nine Moi
values and the unchanged upper triangle/four-ULP comparison. All native inputs,
goldens, old helpers and retained tolerances remain byte-identical. This complete
expanded source/regression batch subsequently completed the instrumented all-targets command; its later strict classification failure is recorded below.

The initial verifier correction only emits already-captured failing instrumented stdout
and stderr to the caller's streams before re-raising the same exception. It
retains the existing tests.log and commands. Its existing regression now checks
both emitted streams alongside every old log, exception, status and command
assertion; temporary staged cleanup can no longer erase the sole diagnostic.
That initial output-only change did not alter thresholds, classification, commands or exception behavior.

## Source regression and reference batch

Four integration controls, two private raw-scalar/overflow regressions and the folded
historical controls are prepared: complete same-input historical positives;
density-presence and manual-default retention; genuine combined-transform,
attribute and override refusals; resource/clone/source retention; and the
reviewed native differential with explicit inherited numeric/manual refusals.
The initial empty native observation fixture deliberately prevented qualification; it is now populated by the one authorized capture.

The 24 prepared native inputs contain the exact two held historical roll inputs,
ordinary roll/pitch/yaw/mixed cases, half turns, small and large finite Euler
angles, missing/empty density, existing kernel extremes and anisotropic boxes.
The observer retains all direct collision and two aggregate snapshots as raw
u64 values, every error vector, finiteness/validity, warnings, child exit/timeout
status and actual mapped-library identities. It additionally records the direct
Collision::Moi 3x3 matrix, separating principal moments and tensor transformation.
The one native capture completed all 24 children with exit zero and no timeout
or crash. Every direct collision matrix is finite and valid. Every Root aggregate
is finite, but only 22 are native-valid: `box_maximum` and `sphere_maximum` are
native-invalid despite empty native error vectors. Both absent-density inputs
retain their warnings. The isotropic sphere historical roll has a real recorded
`iyz = -4.336808689942018e-19`; it is not erased or treated as exact zero.
All raw values, including nine direct-Moi entries, remain preserved. The first
mandatory attempt and one persistent diagnostic are recorded above. Corrected
numerical coverage qualification is incomplete; no extra native capture or numerical search has occurred.

Native IsNearPositive tests nonnegative minors and determinant with positive
comparisons, so NaN arithmetic cannot satisfy them. The retained Rust manual
validator has different overflow behavior. Root reviewed a bounded correction
only for the newly rotated tensor: its public glam matrix determinant must be
finite before unchanged manual validation. Otherwise the loader returns
Unsupported for numerical validation representability. This is conservative:
it can refuse native-valid extreme inputs, without a raw density cutoff or a
custom determinant. Accepted unrotated and manual paths remain unchanged.

The proposed old-test transition is only the exact `<pose>0 0 0 0.1 0 0</pose>`
entry for both shapes in `auto_identity_pose.rs`, with its original generated
XML and `closed-pose.sdf` URI. The prepared helper asserts complete loaded model,
link, collision, density, tensor, pose, source, graph and clone observations.
Independent source PRIOR review approved exactly those two branches. Actual
captured native data confirms that both are finite and valid with empty errors;
they now call the complete helper. Only their import and two branch insertions
change the original file; its table, corpora, counts and all other assertions
remain unchanged. The duplicate prepared wrapper is removed, while the native
differential uses that helper for its same two rows. Final data/fold review and
qualification are pending; no new original-case completion is claimed. Existing identity, translation, default
density, primitive-value and unsupported-domain tests remain retained.

The source-reviewed, authorized native command ran once:
`python3 tools/auto_rotation_pose_reference.py /path/to/pinned/sdformat --capture`.
Actual data, both numerical failures/arithmetic records, expanded private primitive-value compatibility, and exact folds received independent review before the third mandatory hook. No preliminary test/lint loop or additional native search was used.


## Test-module classification and failure retention

The expanded mandatory hook completed 56 tooling tests, formatting, all-target
Clippy, rustdoc and the instrumented all-target Rust command, then both LLVM
exports, strict report and full-source checks. It failed at the strict changed
classifier (`changed source is opaque syntax`), after 393 seconds without a
capacity stop. Its ordinary temporary export was removed, so exact Rust counts,
DA records and current coverage totals are unavailable; no numerical or coverage
acceptance is claimed from that command flow. Both earlier numerical failures
and their one diagnostic remain separate.

One authorized query of the retained source-identical syntax helper confirms
that the new `#[cfg(test)]` and `#[path = "../tests/private/auto_rotation.rs"]`
lines are opaque and their `mod automatic_rotation_tests;` line has no
declaration evidence. Without the discarded DA records this does not identify
the first pending line visited by the failed checker. The approved source repair
recognizes only three exclusive top-level private external-test-module
declaration rows: exact `cfg(test)`, the bounded private-test path, and `mod`.
It does not clear production attribute opacity or waive mapped DA0/regions.
Malformed, public, inline, extra/feature attributes, escaping paths and shared
executable lines remain outside the proof.

On an actual staged verification exception, the verifier now attempts to retain
only existing test output, coverage JSON/LCOV, merged profiles and raw profiles
in a unique failed-evidence directory before temporary cleanup. Compiler targets
and snapshots are not copied; prior evidence is not overwritten. Copy failure
only warns, and the original verification exception is re-raised unchanged.
The new classifier/protocol and retention regressions are source-complete and
unrun pending independent review and the required gate. No private raw-scalar
test, production line, bound, golden or old assertion is removed or excluded.


The next mandatory hook passed all 709 Rust tests (211 test summaries, no
failures, ignores or filters) and 59 tooling tests, formatting, all-target Clippy
and rustdoc. Its retained strict production report contains 37 files,
8937/8983 lines and 14150/14425 regions; the exact changed DA subset is 92/92.
It still failed the required absent-DA syntax check, so these are precursor
results, not feature acceptance. Failed-report retention worked; reports, test
output and profiles survive independently of the removed compiler snapshot.

Every retained absent-DA changed row was inventoried. Beyond comments, blanks
and braces, the rows are the import at parse.rs:8, call closing at :4845, and
the three test declarations at :4895–4897. The first narrow proof admitted the
declaration rows but left both visitors treating the same known attributes as
transformations, which disabled existing import/call-closing evidence. The
completed source correction computes that exact proof first and excludes only
its two standalone attribute spans from `Audit`/`CallAudit` attribute visitation.
It never clears `opaque`, skips a module body, or exempts another attribute.
The tuple-declaration audit receives the same independently proven spans; nested
format-argument audits receive no such spans. New controls preserve existing
import/call-closing proof only with the exact safe declaration; malformed,
public/inline/extra attributes and adjacent custom production attributes still
invalidate those scopes. All numerical/provider/helper/native/fold bytes are
unchanged. This complete amended source batch is unrun pending independent
review and the one affected mandatory hook.
