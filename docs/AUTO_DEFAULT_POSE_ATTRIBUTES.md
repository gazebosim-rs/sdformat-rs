<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Explicit default Euler attributes in automatic inertia

This batch starts at accepted `4e4a919b`. Complete source/protocol review and its
narrow Name-lookup correction were clear before the one actual native capture.
All 24 children and unchanged `Param.Pose3d` completed successfully. Native data
and amended source/PRIOR review are clear. The exact sixteen old negatives now
call the approved complete helper, and final fold source review is clear.
The first required hook passed 59 tooling checks, formatting, all-target Clippy
and documentation checks, then failed the new helper's manual Source assumption.
The corrected helper execution is unrun; no complete Rust-suite or coverage
result is available. The test still requires all 24 rows, so empty or incomplete
data cannot qualify.

The collision pose may explicitly state `degrees='false'` or `degrees='0'`,
`rotation_format='euler_rpy'`, or both in either order. The existing pose parser
already recognizes these exact values as its default radian Euler constructor.
Only the automatic-inertia blanket attribute guard changes. Existing numeric
Euler parsing/normalization, mass properties, scalar compatibility, matrix and
translation operations, metadata ownership and physical/domain validation stay
unchanged. Ordinary equivalent decimal and exponent numeric spellings use the
same existing parser. No new format, geometry formula, math framework,
dependency, public type or consumer behavior is introduced.

Pinned SDFormat `1249805d` Param.cc defaults `parseAsDegrees` to false and
`rotationFormat` to `euler_rpy`. Its attribute loop leaves `euler_rpy` unchanged;
false degrees select exactly the same `Pose3d(x,y,z,roll,pitch,yaw)` constructor
as absence. The pose schema supplies these same defaults. Collision.cc then
uses that parsed local pose in the existing inertial calculation and Link.cc
aggregation. This is a constructor-equivalent attribute admission, not a new
rotation or inertia algorithm. Actual whole native outcomes still require the
reviewed capture; source reasoning is not a passing differential result.

Every explicit `frame`/`relative_to` attribute, including an empty reference,
remains outside this automatic-inertia subset. True-degree, quaternion and
unknown name/value families remain closed. Ordinary malformed pose components
and invalid attribute values retain existing parser errors. Rotated/mixed
Capsule remains Unsupported; default Euler attributes cannot bypass that
geometry guard. Density presence/fallbacks, inertia overrides, multiplicity,
nonfinite/kernel/determinant and manual physical guards are unchanged.

## Retained assertion dispositions proposed for prior review

The fixed native inputs enumerate the exact XML, URI, source fixture/case and
old Rust file for sixteen existing negatives:

- Identity box and sphere: explicit default Euler format (two rows),
  `closed-pose.sdf` in `auto_identity_pose.rs`.
- Origin-rotated box and sphere: false degrees and default format (four rows),
  `closed-rotation.sdf` in `auto_rotation_pose.rs`.
- Combined box and sphere: the same two attribute values (four rows),
  `closed-combined.sdf` in `auto_rigid_pose.rs`.
- Combined cylinder: the same two values (two rows), `closed-cylinder.sdf`.
- Origin Capsule: the same two values (two rows), `closed-capsule.sdf`.
- Translated Capsule: the same two values (two rows), `closed-translation.sdf`.

Actual native data and independent PRIOR review approved only these sixteen
exact generated inputs. The six old files now import and call the complete
helper only for those branches. Original tables/order/counts, corpora and every
other predicate remain unchanged. No duplicate prepared wrapper was present to
remove; the new complete corpus retains all 24 checks. No original-case
completion credit is proposed.

Eight additional fixed controls exercise explicit false/zero and paired
attributes in both orders across the four admitted shapes. All 24 inputs are
fixed before any capture. The existing reference observer retains both complete
Root snapshots, direct inertial values, all nine direct Moi entries, flags,
errors, stdout/stderr, child exits/deadlines and actual maps. The selected GCC11,
SDK source/header/schema/build-library and expanded-dependency protocol is
reused; dependencies are compared again after capture. The unchanged native
`Param.Pose3d` case is invoked once in the same capture. Its observation does not
claim to port the whole native Param contract. Every failure stays explicit.

The complete helper binds exact XML/URI, the entire owned Model/Link/Collision
value projection against the same accepted attribute-absent input, manual
collision/default inertia, source line/column, raw pose, density presence,
cloning, frame resolution and all native errors/statuses/idempotence/tail fields.
The helper requires empty captured stderr for twenty explicit-density rows.
Four exact cylinder inputs omit density; their warning bytes must equal the
accepted original cylinder capture, and their typed density remains absent. Attribute iteration supplies names, so the guard obtains values through
the existing `pose.attribute(name)` lookup without expanding the XML API.
All fourteen aggregate and seven direct raw-pose scalars use the unchanged
four-ULP policy. Root and direct data are retained independently; no native
error/default is treated as a positive. Further source regressions cover empty
pose defaults, inherited density absence/empty/whitespace/1000, canonical
include ownership and the closed families. These Rust checks are authored and unrun. The density-presence regression handles
the inherited cylinder absence as well as explicit-density shapes, rather than
assuming every base contains a density element.

## Capsule rotation boundary

This slice deliberately does not extend Capsule rotation. Parry's analytic
capsule moments and the pinned native cylinder-plus-hemisphere evaluation have
different scalar expression order. Existing four-ULP centered scalar agreement
does not imply four-ULP agreement for a near-zero off-diagonal tensor component
after cancellation. That needs a separate concrete precision investigation and
reviewed source plan; no native-order provider or cutoff is added here.

Complete independent source/protocol review precedes one bounded native capture.
Actual native data and exact-input PRIOR review precede the final source folds
and ONE consolidated required hook. No per-edit runtime or lint loop is planned.
Simulator, Fuel and accepted producer trees remain untouched.

## Actual single native capture

The reviewed capture completed once in 14.506 seconds, exit zero without a
reserve stop. All 24 child exits are zero, all Root/direct finite-valid flags
are true, every load/direct/Resolve error vector is empty, and both Root
snapshots are identical. Four cylinder rows preserve the existing two default-
density warning messages byte-for-byte; the remaining twenty stderr streams
are empty. The input numbers and XML were not changed to obtain this outcome.

The selected compiler, 76 protected inputs, 340 expanded compiler dependencies
and 11 mapped libraries per observer were retained with the existing protocol.
The unchanged native `Param.Pose3d` completed once with one pass. Raw result
SHA-256 is `0636937b25c16f34dfba471724da6ab6075c9412e9247d0118a718a8a8c09ba7`;
fixture SHA-256 is `efe608f56212950ed939cdc7c55e29327145bc33a8d16309f8f26c17427f22e9`.
This is native reference evidence, not Rust precision, performance or original
completion credit. No capture retry, additional candidate or arithmetic search
ran. The overly broad new empty-stderr expectation and its density-control
assumption were corrected as source after observing the inherited absent-density
inputs, before any Rust verification. Independent actual SDK/data and amended-source review are clear (review
`1b483f15`). The exact sixteen transitions are prior-approved and now folded;
final fold source review and actual capacity release still precede ONE required
Rust gate. No prepared-positive runtime prerequisite or extra verification
round is added.

## First mandatory hook and provenance correction

The ordinary hook on tree `e194f7d3` failed after 316.037 seconds without a
capacity stop or commit. All 59 tooling checks, formatting, all-target Clippy
and documentation checks passed. Instrumented tests stopped at the first
failing target: the exact old origin Capsule table reached the stronger helper,
whose manual/automatic Collision Debug equality compared two different source
locations. `auto='false'` is one character longer than `auto='true'`, so the
manual collision begins at column 76 and the automatic collision at column 75.
All other displayed Collision fields were identical. This is a new helper
provenance expectation bug, not observed native or mass-property disagreement.

The completed correction saves the exact manual XML, compares every non-Source
Collision field individually, and asserts the manual Source URI, line and
column against that manual XML. The independent existing automatic Source
assertion still uses the original XML, and the entire automatic Model versus
attribute-absent control comparison is unchanged. No Source field is dropped
or normalized away. All six Collision fields remain checked, with native
values, old dispositions, numerical bounds and production code unchanged.
All available failed output was inspected; Cargo stopped that target, so a
complete Rust-suite or coverage result is unavailable. Failed output and raw
profiles are retained. Corrected runtime/coverage remain unrun pending narrow
independent source review and ONE affected mandatory hook; no preliminary
test, lint, helper or native round is introduced.
