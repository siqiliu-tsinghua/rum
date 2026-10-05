# brief-571 — G4 step A: retire three obsolete matcher adapters

Implementation brief. **Do not run `git` in any form**, including before you
have read this brief. Leave every change uncommitted for the director's gate.

## Goal

brief-570 found that three of G4's sixteen lost-continuation sites live in
adapters nothing in production calls:

- `match_expr` (`crates/rum-runtime/src/runtime/pattern_matching/core.rs:11-21`,
  P1019) and `match_expr_interpreted`
  (`pattern_matching_interpreted.rs:312-323`, P1285) — borrowed-bindings
  wrappers over the owned matchers, marked `#[allow(dead_code)]`, used only by
  one parity test (`runtime/tests/definitions_and_rules_part2.rs:47, 50`, with
  the binding comparison at `:58-60`);
- `normalize_cases_spec` (`kernel_data/rewrite_data/cases/match_q.rs:78-85`,
  P0779) — no caller at all; production `Cases` resolves and compiles in
  separately framed phases.

The right repair for a continuation lost in code nothing calls is to delete
the code. Migrate the parity test to the owned outcomes (`.matched`,
`.bindings`), keeping every case and every binding assertion, then remove the
three functions. Lost continuations **59 -> 56** (three retirements; G4
16 -> 13). This step repairs no live witness; brief-570's red G4 cases stay
red until steps B and C.

## Context pointers

- brief-570's report §2 and `.temp/brief-570/recursion.md`,
  `caller-search.txt`, `inventory-plan.md` (section A),
  `draft-A-retire-obsolete-wrappers.md`.
- `crates/rum-runtime/src/runtime/mod.rs:48-49` (the interpreted wrapper is
  `pub(super)` in a `cfg(test)` module); the owned interpreted suspension test
  in `definitions_and_rules_part2.rs:66-120`, which stays.
- `docs/dev/codex/TEMPLATE.md` (standing duty).

## Scope

**Objective:** the three deletions and the parity test's migration.

**Edit surface:** the three functions named above, in their three files; the
parity test in `crates/rum-runtime/src/runtime/tests/definitions_and_rules_part2.rs`.

**Standing maintenance duty — always in scope** (`TEMPLATE.md`): replay the 68
contract fingerprints and both certificates, reconcile the ownership
inventories from discovery (positions move), update stale evidence positions
in `retention_closure/inventory/obligations.tsv`. None of the three functions
owns an inventory row; **any row disappearing is a STOP**, as is any other
identity, status, disposition or exclusion change. The named-frame count stays
556; no frame-list change.

**Fence:** no change to the matcher, its compiler or codecs, any production
caller, `Expr`, public or `#[doc(hidden)]` APIs, manifests or the lockfile; no
compatibility wrapper in place of the deleted ones. A production, public or
`#[doc(hidden)]` caller found during the work **blocks** the removal — STOP and
report it. No edits to `AGENTS.md`, `TODO.md`, root `PROGRESS.md`, earlier
briefs or docs.

**Disclosure duty:** every changed line, the migrated assertions, every
inventory row change, and a repository name search for the three names that
separates them from unrelated identifiers (AST-visitor `match_expr` methods).

## Deliverable

- The parity test compares owned outcomes, `.matched` and `.bindings`, over
  the same cases with the same expected flags; the owned interpreted
  suspension test is untouched.
- The three functions are gone, with no remaining reference.
- The standing duty replayed and disclosed; the three frozen IDs recorded as
  retired (brief-548's snapshots are not edited).

## Tests and performance

Record the parity test and the owned interpreted suspension test passing
**before** any edit, and again after. brief-570's red G4 fixtures are evidence
for steps B and C — do not add them here as passing or ignored contracts.

**No performance gate, and why:** two of the functions are compiled only for
tests (`cfg(test)` or used only by a test), and the third has no caller, so the
release binary's code does not change. Do not build a release binary or run a
panel for this step.

## Verification (MANDATORY — run to green or evidence)

Serial, through `python3 scripts/run-bounded.py --timeout <s> -- env
CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true
TMPDIR=<repo>/.temp/brief-571/tmp …`. Bounds: focused 150 s;
`cargo test -p rum-runtime`, `ownership_graph` and `local_production_acceptance`
(either feature set) 300 s; `cargo test --workspace --no-fail-fast` 2 400 s.

```
cargo fmt --check
cargo test -p rum-runtime
cargo test -p rum-stdlib --lib ownership_graph
cargo test -p rum-stdlib --lib local_production_acceptance
cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance
cargo test --workspace --no-fail-fast
```

Report native ZeroMQ skips as skips. At most eight focused attempts; stop
after two compiling attempts that reduce neither remaining references nor
failing tests.

## Checkpointing — protect the scene

Scratch under `.temp/brief-571/`; `.temp/brief-571/PROGRESS.md` after the
baseline, the deletion and the verification.

## Hard rules

- **No `git`, in any form.** No network; nothing outside this repository.
- Commands: scoped reads and edits, `rg`/`grep`, `python3`, bounded serial
  Cargo. No abort probes.
- **Testing Policy is a hard line** (`AGENTS.md`). No new `unsafe`.
- Paste real output; say plainly what did not run.

## Report format (append to this brief)

1. Changed lines and the migrated assertions.
2. Verbatim before/after test output, the name search, every inventory row
   change, the broad suites.
3. The three retired IDs (59 -> 56 predicted); open questions.

---
