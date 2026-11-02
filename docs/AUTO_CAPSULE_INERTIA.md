<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Origin-centered automatic capsule inertia

This batch starts at accepted 907c96c. Native capture and independent SDK/data
review are complete. Two mandatory attempts passed Rust/tooling but failed strict
source checks, first for an opaque macro row and then an unmeasured match-arm
header. The named-principal-component correction remains unrun. All 24 actual
native rows remain fixed, with a mandatory count preventing empty comparisons.

It admits exactly one Capsule collision at numeric identity in the existing
attribute-free local automatic-inertia path. The existing parsed Transform is
compared numerically to identity; ordinary equivalent decimal/exponent and zero
spellings use the same parser. Omitted and empty poses preserve their existing
defaults. This adds no raw-text zero grammar. Translated, rotated and mixed
Capsules remain Unsupported. Degree/quaternion/frame attributes, even empty or
explicit default declarations, overrides and multiple collisions stay closed.

## Kernel reuse and numeric domain

Locked Parry 0.31.1 `MassProperties::from_capsule` receives endpoints along Y at
plus/minus half the parsed central length, plus the radius and effective density.
The central length excludes the two hemispherical caps in both native and Parry
semantics. Public extracted moments are reordered X/Z/Y to SDF's Z-axis
convention. The maintained symmetric center enters the existing translation
helper, and the current finite-positive and manual physical validators remain.
Density absence is distinct from an explicit getter default of 1000 kg/m³.
The typed parser's existing schema/default/nonpositive dimension normalization
is reused; no second geometry parser or raw-size rejection is introduced.

Native gz-math detail/Capsule.hh constructs a cylinder and two hemispheres. It
positions their centroids and uses two ordered Inertial additions, then returns
the resulting mass matrix. Parry directly combines cylinder/ball volumes and
unit inertias with extra terms. These are equivalent physical constructions but
do not promise identical floating-point expression order. This initial slice
uses the public kernel first and keeps the existing four-ULP bound; no custom
capsule mass/moment/aggregation provider is added. The first retained Rust run passed the unchanged comparisons; that is precursor
evidence, while overall corrected qualification remains pending. A failure must be retained
and reviewed rather than raising a bound or inventing a formula.

One concrete source-level admission gap is already visible. Native evaluates
`density * 2 / 3 * PI` before applying radius factors to its hemisphere mass.
For a large finite density and tiny radius that prefix can overflow even when
an analytic volume-then-density kernel is finite. After kernel admission the
new Capsule path checks that exact ordered prefix for finiteness. It also uses
the existing public glam determinant convention to refuse nonfinite validation
arithmetic for this newly admitted domain. These are symbolic representability
checks, not density cutoffs or output formulas. Existing box/sphere/cylinder and
manual paths are unchanged. The prefix adaptation preserves the pinned header's
2020 OSRF/Apache attribution in source and NOTICE.

The large-density/tiny-radius control was fixed from source before capture. Its
actual native calculation failed with error 19 and retained a default inertial,
confirming the intermediate-domain gap. The separately declared finite-prefix
validation-overflow input has a finite native result; Rust deliberately bounds
its own new-domain validation arithmetic with Unsupported. No native error or
retained default inertial counts as a successful shape calculation.

## Fixed reference and regression source

The 24 inputs include the original `DOMCapsule.CalculateInertial` dimensions
(radius 0.1, central length 2, density 2710), omitted/empty/equivalent identity
poses, ordinary shapes, defaults and dimension normalization, density presence,
three retained unsupported pose families and bounded numeric extremes. The mixed
Capsule input is derived exactly from the already accepted Cylinder test's
retained-negative XML by replacing only the shape tag. That old negative remains
unchanged and is not proposed for retirement. All prior tests and corpora remain
byte-identical; no original complete or partial count changes.

The accepted observer protocol is reused: both full Root snapshots, auto/saved
flags, direct Collision inertial values, all nine direct tensor entries, all
error vectors, finite/valid flags, warnings, stdout/stderr, exit/timeout outcomes
and actual maps are retained. The source/provenance recipe checks the pinned
GCC11 compiler, selected sources, schemas and installed headers/build libraries.
Expanded compiler inputs are hashed once after compilation and checked unchanged
after all children. A fresh output directory prevents overwriting an attempt.
All 24 children have bounded deadlines. The same capture invokes the unchanged
native original once and retains its XML/stdout/stderr/exit even on failure.
No SDK or image build, failed-input omission, native rerun or performance claim
is part of the recipe.

Regression source requires actual complete data and native positive statuses,
empty error vectors and idempotent snapshots before positive comparisons. It
checks full aggregate mass/tensor/pose and direct tensor entries at the unchanged
four-ULP bound, existing manual-policy errors, Model/Link/Collision owned defaults,
geometry/density presence, source URI/line/column, frame resolution and cloning.
Manual loading retains all four Inertial fields and collision metadata. Separate
controls preserve unsupported poses/attributes/overrides/multiplicity and invalid
density diagnostics; canonical includes preserve source and the local origin.
Signed-zero bit equivalence and unobserved version/encoding families are not
claimed. No public types, dependencies, compiler version, general framework,
simulator pin or Fuel behavior changes.

Independent complete source/protocol review preceded the one actual capture.
Independent actual-data/final source review precedes one mandatory Rust hook. There are no preliminary product tests, lint or arithmetic probes.


## Actual single-capture results

The one capture completed in 14.507 seconds with no reserve stop. All 24 children
exited zero without a timeout, and the unchanged original ran once with one pass
and zero failures. The selected GCC11 executable, 340 expanded compiler inputs
and 11 mapped observer libraries were retained with unchanged input hashes. This
is correctness/reference evidence, not a performance comparison. Raw result
SHA-256 is
`58e5d14df8f2c5b38e77e145bdb58ce1cc1dfe625046506f355449c80101ef34`.

All 24 stored Root/direct inertials have finite and native-valid flags and both
Root snapshots are identical. Those flags do not establish strict physical
admission. Maximum density, the native-prefix-overflow case and maximum length
each report load/direct error 19 and retain default values; every Resolve vector
is empty. The other 21 cases have empty error vectors, including the minimum-
radius zero-mass/zero-tensor result and declared unsupported pose/representability
families. The Rust regression explicitly requires the existing Unsupported
classification for those declared refusals and prevents an error default from
reaching a positive comparison. These native observations were reviewed before the first Rust execution; its
actual results and failed overall status are recorded below.

Seven warning rows are retained: three dimension-fallback cases, missing density
and the three nonidentity-pose cases with missing density. No warnings/errors
were dropped, new candidates sought, captures repeated or bounds changed.
All inherited tests/corpora and the old mixed-capsule refusal remain unchanged.


## First mandatory result and source correction

The first ordinary mandatory hook failed after 410.054 seconds without a reserve
stop; no commit or qualification pass occurred. It retained 733 passing Rust
tests in 216 clean summaries, all three new regression functions, 59 tooling
checks, formatting, all-target Clippy and rustdoc. The runtime precursor includes
the actual unchanged four-ULP comparisons and explicit refusals. Owned production
coverage was 9045/9092 lines and 14360/14639 regions; all 22 measured changed lines
were covered. Full-source 8640/8674 and all 34 cold DA lines remain. The final
strict source-classification assertion nevertheless failed.

One authorized pure AST query of the existing source-identical retained helper
identified only the enum alternation inside the multiline `matches!` expression
as opaque among seven absent-DA changed rows. The other six rows were plain
braces and comments, without opaque classification. No product/compiler/LLVM or
native replay was used to diagnose it; request/output/helper/source identities
are retained separately from the failed reports.

The completed correction expresses the same Y-to-Z bit permutation with a plain
match on borrowed Geometry and an explicit `[d0, d2, d1]` array for cylinders and
capsules. Other geometry keeps the original array. It replaces only the mutable
array, opaque macro and swap; it performs no floating arithmetic, allocation,
normalization, guard, policy, test, native-data or bound change. Independent
source review precedes one affected mandatory hook. No preliminary query/test/
lint/native repetition or accepted Main claim follows from the runtime precursor.


## Affected mandatory result and named principal components

The affected plain-match tree f9d9236a failed after 398.046 seconds without a
capacity stop, retaining 733 passing Rust tests in 216 summaries and 59 tooling
checks plus formatting, Clippy and rustdoc. Owned coverage was 9042/9089 lines,
14361/14640 regions, changed executable lines 22/22, full-source 8637/8671 and
34 cold lines. No commit or qualification pass occurred.

The new error was the executable-line assertion after the opaque check passed.
Among ten absent-DA changed rows, nine are plain braces/comments; only the
Geometry arm header is neither. Its array expression on the following line has
actual DA coverage. This identifies the detached header directly from retained
LCOV, the staged diff and the unchanged checker; the earlier different-source
AST query was not reused and no new query or runtime diagnostic was needed.

The final source names the three extracted principal components ix/iy/iz and
returns [ix, iz, iy] for Cylinder/Capsule and [ix, iy, iz] otherwise. Each complete
short arm contains its value expression. This is the same borrowed match and bit
permutation, with no floating operation, allocation, normalization, condition,
public API, numeric bound, test, native data or coverage-policy change. It makes
the axes explicit instead of repeatedly indexing the array. Final independent
source review and one affected mandatory hook remain required; no unchanged
product/helper/native checks ran during this correction.
