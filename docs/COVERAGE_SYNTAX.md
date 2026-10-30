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
