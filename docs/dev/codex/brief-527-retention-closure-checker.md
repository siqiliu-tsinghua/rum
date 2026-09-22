# brief-527 — D1: the restricted-retention checker and its six falsifiers

Implementation brief. Build the checker, its falsifiers, and the resolved
option-frontier report. **Claim zero production coverage.** Change no
production source file.

## Goal

brief-526 settled the mechanism: a required source test implementing a small
type-and-effect system over an *admitted subset* of Rust, in which every
construct not explicitly modelled is an error. Build that checker, prove each
of its six rejections by watching it fail and then pass, and produce the one
number the line is missing — the **resolved** call/effect closure of the
option region, which no name-based graph can supply.

When you are done: the checker exists, its seven required tests run inside the
ordinary `cargo test` gate with no feature flag and no `#[ignore]`, six
falsifier fixtures are compiled Rust whose rejections are pinned by kind and
span, the accepted fixture exercises the shapes D2 will need, and the frontier
report states every resolved body and every rejected edge of
`merged_option_exprs_for_symbol` / `evaluate_runtime_option_expr`.

**No production region is promoted to Covered by this brief.** The promoted
inventory ships empty.

## Context pointers

- Read `AGENTS.md` first (operational notes, Testing Policy hard lines).
- `docs/dev/codex/brief-526-closure-mechanism-design.md` from line 254 — the
  design you are implementing. §3.3 is the admitted language, §3.5 the
  trusted-leaf rules, §3.6 the six shapes, §3.7 the falsifier protocol.
  Follow it; where you must depart from it, say so in the report.
- `crates/rum-stdlib/src/tests/ownership_graph/` — the closest precedent for
  source discovery, module reconciliation and canonical identities
  (`source.rs`, `resolve.rs`, `model.rs`, `source_items.rs`). **Read its
  module doc (`mod.rs:1-18`) before reusing anything**: it says in its own
  words that an unknown inventory is *discovery, not safety*, and that
  matching an unknown is not a waiver. Do not import that habit into a
  checker whose whole job is refusing unknowns.
- `crates/rum-derive/src/lib.rs:73,113` and
  `crates/rum-runtime/src/runtime/tests/frame_root_trace.rs:73` — compiler
  obligation plus independent manual-impl inventory.
- `crates/rum-stdlib/src/catalog/memory_growth_contract.rs:668` — installed
  source reconciliation.
- The pilot's own code, which you analyse but must not modify:
  `crates/rum-runtime/src/runtime/options/core.rs:87-108` (C01, both loops
  and the public wrapper) and `:165-179` (C02, the raw `self.evaluate`).
- Dev-dependencies are already present and sufficient:
  `crates/rum-stdlib/Cargo.toml:25-29` gives `syn` (`full`, `visit`),
  `quote`, and `proc-macro2` with **`span-locations`**. Adding a dependency
  is out of scope.

## Scope

**Files you MAY create**, all under
`crates/rum-stdlib/src/tests/retention_closure/`:

- `mod.rs`, `model.rs`, `source.rs`, `resolve.rs`, `types.rs`, `effects.rs`,
  `regions.rs`, `leaves.rs`, `fixtures.rs`, `mutations.rs`;
- `inventory/obligations.tsv`, `inventory/promoted.tsv`,
  `inventory/leaves.tsv`;
- `fixtures/bare_vec_loop.rs`, `fixtures/aliased_collect.rs`,
  `fixtures/formatted_string.rs`, `fixtures/map_entry_callback.rs`,
  `fixtures/extracted_allocator.rs`, `fixtures/unleased_callback_result.rs`,
  `fixtures/accepted.rs`.

Keep discovery, resolution, typing, effects, regions and fixtures in separate
modules. Do not copy the ownership checker wholesale into one file.

**Files you MAY modify:**

- `crates/rum-stdlib/src/tests.rs` — register the new module, nothing else.
- `docs/testing-strategy.md` — record the new required gate and what it does
  and does not guarantee. Two paragraphs at most, and no weakening of
  anything already there.

**Everything else is read-only**, explicitly including **every production
`.rs` file in every crate**, all `Cargo.toml` files, and `Cargo.lock`. If you
conclude a production change is required to make the checker work, **STOP and
report it**. D1 is not that implementation, and a production edit here would
invalidate the "no A/B required" basis of this brief.

## Deliverable

1. **The checker.** Canonical item identities, full module reconciliation
   across the production module tree including non-test `cfg` alternatives,
   the admitted-syntax specification as code rather than prose, and an
   ownership/effect fact for every expression in every accepted fixture.
   Every unmodelled construct is an error — no permissive mode, no "visit the
   children and assume the parent is harmless" default, no unknown bucket.

2. **Six falsifiers, each watched red then green.** Each fixture is real
   Rust that **compiles**, so that an unrelated syntax or import error cannot
   impersonate the intended rejection. Each rejection is asserted by
   structured kind (`RET001`/`RET002`/`RET003`/`RET004`), canonical target
   where one applies, file and span, and the witness path — never by
   `is_err()` and never by "it did not compile". For each of the six, the
   report must show: the run where it is red with the pinned diagnostic, the
   run where the valid map is restored and it is green, and a third run where
   **that specific check is disabled in the checker** and its permanent
   negative test fails. Save complete stdout/stderr to a file before quoting.

3. **A non-trivial accepted fixture.** `fixtures/accepted.rs` must exercise
   the shapes D2 actually needs — an accumulator held across a loop under a
   live lease, a callback result adopted before retention, and a transfer of
   an owned payload to an outgoing destination — expressed in the admitted
   surface. **A checker that accepts nothing is trivially fail-closed and
   tells us nothing about whether D2 is possible.** If one of those shapes
   cannot be expressed in the admitted surface, that is a finding: report it
   with what the surface would need, and do not widen the surface to make it
   pass.

4. **The trusted-leaf machinery, and almost nothing in it.**
   `inventory/leaves.tsv` may contain **only** entries exercised by a
   fixture, each with the full record §3.5 requires — identity, signature,
   allowed argument classes, allocation/callback/capture effects, symbolic
   bound, result ownership, exceptional and `Drop` effects, and a dependency
   fingerprint. A trusted entry that no fixture exercises is a trusted entry
   nobody has checked; the required test must reject one. No module, path
   prefix, trait or method-name entry, ever.

5. **The option-frontier report** — this is the number the line is waiting
   on. Taking C01/C02 as candidate roots, list every body the resolver
   *resolves*, every edge it *rejects* with the reason, and the boundary
   records you propose. Give the resolved closure size next to brief-526's
   name-spelling candidate count (2 633 functions across 349 files) so the
   difference is visible. This becomes D2's scope; D2 cannot be written
   without it.

6. **Zero production coverage, stated.** `inventory/promoted.tsv` ships
   empty, and `inventory/obligations.tsv` records the N obligations as
   Pending with no Covered row.

## Required tests

These run in the ordinary gate — no feature flag, no `#[ignore]`, no
permissive mode:

- `tests::retention_closure::production_obligations_are_exact`
- `tests::retention_closure::restricted_surface_rejects_six_shapes`
- `tests::retention_closure::extracted_helpers_enter_closure`
- `tests::retention_closure::unresolved_edges_are_errors`
- `tests::retention_closure::coverage_cannot_shrink`
- `tests::retention_closure::trusted_items_have_exact_bounds`
- `tests::retention_closure::implicit_operations_are_classified`
- `tests::retention_closure::accepted_fixture_exercises_d2_shapes`

`coverage_cannot_shrink` must fail when a Covered root is removed,
downgraded to Pending, redirected, or has an unmarked helper extracted out of
it. `trusted_items_have_exact_bounds` must fail on a stale fingerprint, a
wildcard, and an unexercised entry.

## rum / Rust pitfalls (read twice)

1. **`syn` is syntax, not resolution.** A method receiver's type, a trait
   fallback and a macro's expansion cannot be guessed. Anything you cannot
   resolve is an error; that is the design, not a gap to paper over.
2. **A fixture's rejection only counts if the fixture compiles.** Build the
   compile-pass control first.
3. **`compile_fail` is the wrong instrument here.** Ordinary Rust compiles a
   bare `Vec`; these six are required-test failures with pinned diagnostics.
4. **Discovery's "the inventory matches" is not success.** The precedent's
   own module doc says so.
5. **An unknown `cfg` arm, macro, or local function value is an error**, not
   a skipped body.
6. **Do not reorganize production control flow to make analysis easier.**
   Production is read-only in this brief; if the admitted surface cannot
   express something production does, that is the frontier report's content.
7. **Span locations are available** (`proc-macro2` `span-locations`), so a
   diagnostic without a real file and span is a choice, not a limitation.

## Verification (MANDATORY — you run it; iterate to green)

Timeouts below are derived from director measurements on this host,
warm tree, immediately before dispatch: `cargo test -p rum-stdlib` completed
**green in 36 s**, `cargo test --workspace` **green in 79 s**. The tree is
therefore green at your starting point, and any red you see is yours. The
bounds below carry margin for a cold build; do not raise them silently — if
one is hit, report it with the elapsed time.

```
python3 scripts/run-bounded.py --timeout 600 -- cargo fmt --check
python3 scripts/run-bounded.py --timeout 900 -- cargo test -p rum-stdlib retention_closure -- --nocapture
python3 scripts/run-bounded.py --timeout 900 -- cargo test -p rum-stdlib
python3 scripts/run-bounded.py --timeout 900 -- cargo test -p rum-stdlib --no-default-features retention_closure
python3 scripts/run-bounded.py --timeout 2400 -- cargo test --workspace
```

Paste the verbatim test counts. Then the falsifier transcripts: for each of
the six shapes, the red run with its pinned diagnostic, the restored green
run, and the disabled-check run whose permanent negative test fails.

**Microbench A/B: not applicable.** No production code changes, so no hot
path moves. Say that in the report rather than performing the ceremony — and
if you find yourself needing a production edit, that statement stops being
true and you must STOP instead.

## Hard rules

- **Do not run `git` in any form.** Not `status`, not `diff`, not `log`.
  Nothing in this brief needs it, and the last three briefs each ran it after
  being told not to. Running it is a violation to report, not a bootstrap
  convenience.
- Do NOT commit, branch, push, or touch git config. Leave everything in the
  working tree; the director gates and commits.
- Do NOT modify any file outside Scope — in particular no production `.rs`,
  no `Cargo.toml`, no `Cargo.lock`. If you believe you must, STOP and report.
- No new dependency. No toolchain change. The pin is stable 1.85.0.
- **No new `unsafe`.**
- No permissive unknown mode, no `#[ignore]`, no feature gate on a required
  test, no environment variable that turns the check off.
- Do not resolve a hard case by widening the trusted set, and do not add a
  trusted entry to make a fixture pass.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only; no vendor comparison, oracle harness, compatibility matrix
  or third-party engine output.
- Scratch output goes under `target/brief-527/`, not into the repository.
- **Stop-loss, keyed on progress.** The measure is the unresolved-edge count
  in the frontier report and the number of the eight required tests passing.
  If an attempt does not move either, STOP and report with the two numbers.
  Do not treat "the checker rejected something again" as a failure — a
  converging restrictive checker rejects constantly by design.

## Report format (your final message)

1. What changed — per file, with line ranges and a one-line why.
2. Verification — each command with its verbatim output, the eight test
   results, and the eighteen falsifier transcripts (six shapes × red,
   restored green, disabled-check red).
3. The option-frontier report: resolved bodies, rejected edges with reasons,
   proposed boundaries, and the resolved closure size beside the 2 633
   name-spelling candidate count.
4. Stop-loss / open questions — including anything the admitted surface
   cannot express that D2 will need.
