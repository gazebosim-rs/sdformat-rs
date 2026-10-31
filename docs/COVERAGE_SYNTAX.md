<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->
# Changed-line syntax evidence

LLVM DA records remain authoritative for measured lines. Every mapped changed
line is counted first, including uncovered lines; syntax evidence cannot waive it.
File, crate, workspace and region coverage checks remain unchanged. This tooling
only distinguishes a narrow set of unmapped source lines from executable lines;
it does not add coverage, remove a line or region, or change thresholds.

The test-only `coverage_syntax` helper uses the already locked `syn` and
`proc-macro2` versions to parse the actual source and report syntax spans. The
normal all-target test run builds the helper. The verifier requires its unique
Cargo test artifact from the expected source and target directory, then queries
it with exact source, direct ancestor and module identity. The response echoes
those inputs and the helper's compile-time source. Git patch parsing uses raw
bytes and LF line boundaries, and source and ancestor bytes are decoded without
newline normalization; malformed, stale, missing,
duplicate or unexpected output fails closed. The checked source digest must also
remain unchanged throughout verification. No production package or Cargo target
layout exception is added.

The supported unmapped cases are intentionally narrow: comments and punctuation
outside opaque spans, simple `use`/`mod`/bare `impl` headers backed by AST nodes,
a standalone `);` closing an AST call, and a public `Vec<Identifier>` field in a
simple top-level struct. Exact full-line lexical checks apply as well. Macro token
streams, string literals and non-doc attributes are opaque. Conditional, custom
or unknown attributes and unsupported derives fail closed. A `#[derive(Debug)]`
struct field is admitted only with a direct module and ancestor scope check that
rejects possible builtin-name imports/globs, target item macros, ancestor macros,
`macro_use` and opaque ancestor attributes, including entries after the target
module. The field proof accepts `#[default]` only on a variant of an enum with
the proven standard Default derive.

Call endings use a separate proof: they must occur in a function or method body
whose file, module, ancestor and item attributes pass independent conservative
checks. Unrelated derives can add items but cannot rewrite that body, so a glob
import alone does not prevent the call proof. Unknown or conditional attributes,
item macros and opaque token spans fail closed. The query pins its test harness
thread mode.

Rust's built-in derived Debug implementation is generated code. A separate, unaccepted world-extension candidate checks nonempty World Debug
output for the exposed extension value. A pinned-rustc probe observed no LLVM source attribution for that generated
Debug body; classifying the field declaration does not establish Debug coverage.
This limitation is not a coverage exclusion or a claim about its execution.

Controls cover opaque macro/raw-string content, cfg and custom attributes, derive
shadowing, Unicode and CR/CRLF line boundaries, exact source/context/helper
identity, malformed and missing output,
wrong or duplicate helper artifacts, source mutation, and mapped-uncovered
priority. Native code, business parsing and simulator plugin loading are outside
this test-only tool.

A separately reviewed declaration proof recognizes complete, exclusive LF-line
spans of unannotated imports and one-line static items whose initializer is a
zero-argument plain-path call. Static types are restricted recursively to plain
type paths, immutable references, lifetimes and type generic arguments; const
generics, arrays, type macros, closures, blocks and operators are refused. The
item must be at file scope or directly inside an unannotated top-level function,
with independently checked file and direct ancestor module context. Other
nesting, shared-line text, conditional or custom attributes and macro tokens
remain opaque. This proof is independent of Debug name resolution.

Independent review passed 13 helper tests and 34 tooling tests, including real
compiler and LLVM controls: an uncovered function hidden in a static type is
rejected even with a forged syntax hint, and uncovered regions still fail their
normal coverage threshold. A separate actual-source query covered the six
unmapped declaration lines in the world-metadata candidate. Normal staged and
exact hosted checks remain required before accepting this tooling change. No
ordinary executable operator or literal is classified by this proof.

## Plain owned string fields and exclusive else delimiters

The private follow-up extends field evidence only to `pub NAME: String,` and
`pub NAME: Option<String>,`. Syn must identify a public named field with an
unqualified plain type path, exactly one plain `String` argument for `Option`,
and doc-only field attributes. Arrays, references, other option arguments,
nested generics, const arguments, type macros, qualified paths and trailing
source remain outside this rule. Standard source and ancestor scope is still
required. These two shapes permit only the already audited unqualified standard
Debug/Default/Clone/Copy/PartialEq/Eq derives and docs. The existing Vec rule and
its narrower derive predicate are unchanged; every earlier negative remains.

The separate `else_delimiters` evidence identifies a real `syn::ExprIf` whose
then-block closing brace, `else` token and plain else-block opening brace share
one physical LF line in that order. The line must contain only `} else {` and
whitespace. Existing untransformed function/module/ancestor checks apply. Raw
strings, macro tokens, attributes, `else if`, inline bodies and other source on
the delimiter line fail closed. Both the helper and verifier enforce the
exclusive line spelling; the helper response retains exact source, ancestor,
module and helper-source identity. Unknown, stale, missing or malformed evidence
continues to fail.

These are syntax-only line classifications. Actual DA records take priority,
including an uncovered record on a purported syntax line. Per-file
instrumentation, measured changed-line coverage and aggregate line/region
thresholds are unchanged. No runtime expression or new source exclusion is
introduced.

Author checks pass: 16 helper tests, all 39 tooling tests and targeted Clippy.
New compiler-backed negative fields contain real hidden function bodies or type
macros. A separate real module is compiled and run with LLVM instrumentation;
its String/Option fields and exclusive else line receive syntax evidence while
a real unexecuted branch has DA0 and fails even if evidence is forged in any new
category. The earlier real hidden-body and region-failure controls are retained.
An explicit helper query also proves the three actual custom-sensor candidate
lines; this is separate from normal test counts and does not replace staged
verification. Against retained LLVM data and exact unchanged producer source,
the diagnostic now measures 28/28 executable changed lines and proves only the
three previously identified unmapped declarations/delimiter. This is not a
staged-gate pass. Independent coordinator source review and fresh own-target
replay passed all 16 helper tests and all 39 tooling tests, including the real
LLVM DA0 and region controls, with no remaining findings. Full protected staged
and exact hosted qualification remain pending. The public custom-metadata Default/Clone/Debug test belongs to
the separate producer candidate, not this tooling test count.


## Plain declarations, aggregate delimiters and a closed intrinsic constructor

The World Scene producer's first normal staged hook passed all tests and
aggregate coverage but stopped at the syntax guard. Retained LLVM data measures
42/42 changed executable lines; new declarations and aggregate syntax were
unmapped. That failure is retained, with no production rearrangement, threshold
change or automatic source exclusion.

The additional proof reuses the existing plain-type grammar for exclusive public
field declarations and struct/impl headers under known builtin derives and
untransformed scope. Arrays require literal integer lengths, with no const
expression, type macro or executable generic argument. Only exact proved lines
clear literal/derive opacity. Aggregate closing lines `},` and `})` require actual
AST structure and exclusive spelling; they introduce no general expression rule.

A separate source-bound rule covers only an opener-only intrinsic `Ok(Scene {`
line in an ordinary unannotated function. Exact root/types/camera source bytes
are echoed and checked; all four real source paths, including parse, must occur
in the compiled production-library dep-info. Root public globs are exactly
`types::*` and `camera::*`; other globs and reserved explicit bindings are
refused. Explicit reexports of unrelated names do not supply constructor names.
Scene must be the public named-field plain aggregate, with each unique field
supplied exactly once and no update/rest expression. Explicit/imported/local,
parameter and generic shadows are refused; Syn's raw-identifier spelling and
self-import effective binding are handled. Const/static/associated const and
const-generic bodies, types and closures are not traversed as candidate sites.

This is a closed audit of the current crate, not a Rust name resolver or macro
expander. Only the actual standard macro spellings are allowed in context;
opaque arguments refuse attributes, definitions/exports, reserved constructor
tokens and unknown nested macros. Unknown attributes, extra exports, externs,
verbatim syntax and disabled implicit prelude refuse evidence. Positive normal
library compilation is required before querying. An implicit sibling macro
export cannot introduce a root value binding; a used standard macro name from
`crate::*` conflicts with the default prelude (pinned compiler E0659 control).
Explicit or textual macro overrides are refused by supplied context. No claim
is made that the helper independently resolves arbitrary uncompiled source.

Actual compiler/runner controls distinguish intrinsic construction (zero effects)
from normal and raw-identifier callable Ok shadows (one effect each); only the
intrinsic gains proof. Independent review found and corrected two scope gaps:
terminal self imports and inherited item-const traversal. Compiled regressions
cover three grouped/raw self-import shadows and five const families. Neither
finding demonstrated an executable-code omission. Root independently approved
19 initial helper/41 tooling checks; final fresh independent source/target
review passed all 23 helper tests, 43 tooling tests and all-target Clippy after
these fixes. Real LLVM DA0 remains authoritative even with forged prefix proof,
and actual low region coverage still fails. Stale export context and missing or
wrong compiled source dependencies also fail.

Against the unchanged retained first-hook LLVM data, the final proof resolves
the actual pending constructor line and keeps 42/42 changed DA. This is a
diagnostic, not a staged pass. The combined ordinary protected hook and exact
hosted checks remain required. There is no additional native-original credit,
compiler/dependency/producer-pin change or runtime activation.

The spherical-reference candidate exposed nine genuinely uninstrumented source
lines in a normal staged run: two enum derives, two enum headers, three unit
variants, an exclusive wildcard-block prefix and an owned constructor prefix.
The failed staged export was cleaned up; an approved-source ordinary diagnostic
retains its actual profiles and LLVM export under the author's artifacts/core.
All 62 measured changed lines are covered. Aggregate production coverage remains
7802/7829 lines and 12245/12487 regions; syntax evidence does not alter those counts.

The bounded proof now recognizes public, nongeneric, unit-only enum declarations
with the existing audited builtin derives and source/module/ancestor checks.
Discriminants, payload types, conditional/custom attributes and opaque macros
refuse the entire enum proof. Derive/header/variant lines must have exclusive
AST-backed spans; shared-line declarations receive no exemption.

An exclusive `_ => {` line is recognized only as a genuine wildcard match arm
with an unlabeled plain block inside an untransformed function. Guards, other
patterns, attributes, macros and inline body statements are refused. Only that
prefix line receives evidence; its branch body keeps normal LLVM accounting.
The existing closed constructor audit also covers SphericalCoordinates alongside
Scene, validating its actual owned declaration and fields through the same two
public exports. Both names are protected against imports, local bindings and
macro-token injection. Callable Ok, unknown constructors and transformed contexts
remain unproved.

Author checks pass 26 helper tests, all-target Clippy and both real compiled LLVM
controls. A fresh helper query accounts for the exact nine lines against the
retained production export. Genuine unexecuted wildcard and conditional bodies
still have DA0 and fail even with forged syntax hints; real uncovered regions
still fail their original threshold. Independent proof review, a new ordinary
protected staged run and exact hosted success remain required.
