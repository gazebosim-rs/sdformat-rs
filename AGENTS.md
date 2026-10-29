# Implementation guidance

Read README.md and the linked canonical plan, this repository's contract, task
packets, and test policy. Confirmed owner decisions override proposed defaults.
The semantic loader and strict quality gates are implemented. Production changes
must pass the checked-in staged-tree gate before acceptance.

Keep XML parsing, schema validation/conversion, frame resolution, and resource resolution separate. Accept an injected resource resolver. Do not depend on Bevy, the simulator, or Fuel. Preserve SDF semantics and imported resource provenance, not just XML syntax.

- Pin the upstream main SHA, toolchain, dependencies, and fixtures. Reconcile the
  central source inventory with actual expanded runtime tests for this repository.
- Preserve behavior, formats, protocols, and all retained upstream test semantics.
  Faithful Rust ports are allowed. Keep assertions, edge cases, tolerances, and
  fixture hashes; verify against C++ reference runs and record the mapping.
- Reuse maintained Rust libraries before writing general-purpose algorithms.
  Record exact missing semantics before adding custom code.
- Require strictly greater than 95% line and region coverage for each production
  crate and the repository. Generated code exclusions must be explicit and reviewed.
  Missing tests, unavailable runners, and empty denominators are not passes.
- Install pre-commit checks and required CI gates using the canonical policy.
  Never bypass hooks, weaken tests, or remove hard code from coverage to pass.
- Keep unsafe code narrow and audited. Document ownership, threading, and error
  behavior. Do not let panics or native exceptions cross FFI boundaries.
- Write concise idiomatic code and human comments. Explain invariants and reasons,
  not obvious statements. Avoid repetitive wrappers and speculative abstractions.
- Measure performance against the pinned C++ behavior at equal fidelity. Report
  evidence and limitations; a Rust rewrite is not inherently a speedup.
- Work in bounded reviewable tasks. Commit only owned changes. A different reviewer
  checks semantics, assertions, exclusions, and performance evidence.
- Record exact check commands/results and unrun work. Keep this repo's test ledger
  and integration contract current; do not silently break downstream consumers.

Do not apply the monorepo's internal crate layout here. Do not edit the active Fuel
rewrite or another repository without a coordinated assignment.

## Public progress reporting

Keep the [rewrite dashboard](https://gazebosim-rs.github.io/rewrite-progress/)
current when a slice, test result, review state or real blocker changes. Follow
its [status contract](https://github.com/gazebosim-rs/rewrite-progress/blob/main/status-contract.md).
Prepare ignored `artifacts/sdformat-rs-status.json`; when the sibling dashboard
checkout is available, validate and publish with its `scripts/update-status.py`
for `sdformat-rs` only. Update UTC timestamps. Report working-tree and pushed-commit
evidence separately; link only verified public revisions. Accepted slices do not
complete whole plan packets until every acceptance criterion is met. Never publish
credentials, local private paths, raw environment variables or private user data.

## Licensing

Owned code, tools and documentation use Apache-2.0; keep the root LICENSE and
Cargo package declaration consistent. Preserve original copyright/license notices
on imported code and fixtures. Do not relabel third-party dependencies or replace
their original notices with this repository's license.

The copyright holder for owned new code is Open Source Robotics Foundation,
matching Gazebo main. New Rust, Python and shell source carries
`Copyright (C) YEAR Open Source Robotics Foundation` (YEAR is the file creation
year; current new sources use 2026) and `SPDX-License-Identifier: Apache-2.0`.
Keep original imported notices and years unchanged; this rule does not relabel
third-party dependencies. Maintain NOTICE.


## Audited scalar conversion

Cargo denies unsafe code. The only production allow-scoped exception is the
Linux GNU numeric boundary in src/number.rs; its ABI, ownership and platform
limits are recorded in docs/SCALAR_CONVERSION.md. Preserve the independent
code/safety review requirement for changes to that boundary. No process or
thread locale mutation, escaped pointers, approximate range errors or unreviewed
platform fallback is permitted. The test-only locale snapshot has its own narrow
allow scope; it does not authorize production exceptions. Linux GNU integration
tests require a C compiler for isolated allocation-failure injection. Measure
native/full-parser fidelity and conversion costs before optimizing locale reuse
or making performance claims.

Staged verification clears Git's reported repository-local environment only while
checking the exported tree, then restores the authoritative parent context even
on failure. Preserve the real linked-worktree/alternate-index hook regression;
fixture Git commands must not change the parent repository's index or config.
Use separate reviewer-owned source and Cargo target directories for independent
replays; a shared target can retain artifacts from another source snapshot.

## Owner guidance (2026-10-27)

These answers from the owner apply to every agent, Claude or ChatGPT.

- Keep the real gates: tests, required coverage, this repository's pre-commit
  hook, hosted CI, real C++ interoperability and preserved upstream test intent.
  Do not add process on top of them: no frozen proposal files, hash-bound review
  receipts, CPU/disk slot reservations, versioned reruns of the same check, or
  archiving every failed build. Keep `artifacts/` small.
- Have someone other than the author review each change: give the reviewer the
  code and the claims to verify, and have it re-check each fix.
- Ask the owner only for real product or dependency decisions and record the
  answer. List non-regenerable files for the owner before deleting them.
- Write status and reports in plain sentences that another agent can act on.
- The simulator coordinator's current state and cross-repository lessons are in
  gz-sim-rs `docs/HANDOFF.md` and the end of gz-sim-rs `AGENTS.md`.
