# Brief 639 — the test audit: inventory, falsifiers, duplicates (design only)

**Shape: DESIGN-ONLY. `Scope: NONE` — no source change, no test change.** The
deliverable is a reviewable plan plus drafted implementation briefs, not code.

Dispatched by the director; the director rewrites whatever comes back into
the implementation briefs and dispatches them one at a time.

## Goal

When this brief is done there is one table that accounts for **every
`#[ignore]` test in the workspace (61 today)** and every **exact-duplicate
test body**, and a plan that says, test by test, what the test claims, how it
is supposed to fail, and what work (if any) the claim still needs. Nothing is
changed.

The standing bar from `CLAUDE.md` and the incident log:

- **A check nobody has watched fail is not known to work.** Every
  certificate, contract and inventory test must have a mutant that turns it
  red, and that mutant must be traced through to the observer — a mutant
  whose predicted red is swallowed downstream proves nothing.
- **A certificate must force the transformation it names.** A test named for
  a rebuild must exercise the rebuild.
- **Leave behaviour tests alone.** A `#[ignore]` that exists because a test
  is heavy, slow, or an instrument driven by a script is not an audit
  finding.

## Context pointers

- `AGENTS.md`, `docs/testing-strategy.md`, `docs/dev/codex/TEMPLATE.md`
  (the testing policy: rum's own documented behaviour plus rum-vs-rum A/B).
- The 61 sites: `grep -rn '#\[ignore' --include='*.rs' crates/`. Families
  today, by the attribute's own reason string:
  - 42 `release-only deployment-stack probe; run explicitly with --release`
  - 3 `release-only active-spine safety probe; run explicitly with --release`
  - 2 `release-only width probe; run explicitly with --release`
  - 1 `release-only deployment-stack drop certificate; run explicitly with
    --release`
  - 3 `requires rum-runtime/test-native-stack-guard-zero`
  - 1 `release-only profiling gate; run with RUM_PROFILE_EVALUATION_CACHE=1`
  - 2 instrument drivers (`perf stat`, control FIFOs)
  - 1 `writes a review copy; does not discharge contracts or install marks`
  - 1 `invoked explicitly by both bounded scratch-copy falsifiers`
  - 5 with no reason at all (2 in `crates/rum-host/tests/stress.rs`, 3 in
    `crates/rum-stdlib/src/tests/stress_corpus.rs`)
- Two existing falsifier patterns to imitate and to judge:
  - `crates/rum-stdlib/tests/typed_slot_census.rs`'s
    `forced_red_witness` family — a scratch-copy mutation driven through a
    `SCRATCH_ENV` path, with `forced_red_probe` as the `#[ignore]`d entry.
  - `crates/rum-runtime/src/runtime/definitions/rule_lhs/tests.rs:249`
    `rule_lhs_missing_root_falsifier` and
    `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution/tests.rs:303`
    `matcher_pattern_missing_root_falsifier` — `#[should_panic]` mutants of a
    root-tracing certificate. Their bodies are byte-identical.
- The director's own calibration run of the release-mode `#[ignore]` set is
  at `.temp/director/audit-638/ignored-release.log` (read it; do not rerun
  it). It excludes the two instrument drivers, the three feature-gated
  guard tests, and `forced_red_probe`.
- **Calibration outcome (director, 2026-11-02).** 55 of the 61 were actually
  executed in release: `rum-stdlib --lib` 50 in 504.68 s,
  `rum-host --test stress` 2, `rum-runtime --lib` 3. **54 green. One red**,
  `runtime::evaluation_cache::tests::call_stability_profile_reports_per_source_head_outcomes`
  at `crates/rum-runtime/src/runtime/evaluation_cache/tests.rs:531`, panics
  with `this profiling gate must be run with RUM_PROFILE_EVALUATION_CACHE=1`
  — its own designed precondition, not a defect; rerun with that variable it
  passes in 0.00 s (`.temp/director/audit-638/profile-gate-release.log`,
  rc=0, peak 2.0 GB). Peak of the 55-test run 3.87 GB of 14; oom 0. The
  tree was clean after the run, including the `#[ignore]`d
  `regenerate_local_contract_fingerprints`, which did run. Treat "green"
  for that one row as "green with its environment", and say so wherever the
  inventory reports it.

## Scope

- Files you MAY modify: **NONE.** No source, no test, no inventory, no doc
  edit. This is a survey.
- You MAY write scratch under `.temp/brief-639/` only.
- **MUST NOT touch**: anything under `crates/`, `docs/`, `TODO.md`,
  `.temp/` outside `brief-639`, or any git state. No `git add`, `commit`,
  `checkout`, `stash` or `branch`. No network.

## Deliverables (all in the report, nothing in the tree)

1. **The table, all 61 rows.** `path:line`, test name, the ignore reason as
   written, the mode and feature it needs, **what it asserts** (the claim in
   one line, with the assertion site), and a classification:
   `certificate` / `contract` / `inventory` / `behaviour` / `instrument` /
   `generator` / `falsifier`. A row that cannot be classified from the code
   is an open question, not a guess.
2. **For every `certificate`, `contract` and `inventory` row: the mutant.**
   Name the production code to change (`path:line`, the condition or call),
   the mutation (one line), and **the observation path**: how that mutation
   reaches the test's assertion — which value changes, at which assertion
   line, and why nothing downstream swallows it. Say which mutants collapse
   into one class (one mutant that reds several tests is fine *if* the
   claim is the same claim).
3. **The duplicate table.** Every group of test functions whose bodies are
   identical after whitespace normalisation (the director found nine groups
   among `#[test]` functions; rederive, do not trust that count), with a
   judgment per group: **deliberate mirror** (cross-reader or cross-contract
   copies that must stay separate) or **copy** (collapse into one shared
   helper or one parameterised test), and the shared form you propose.
   Behaviour tests are left alone.
4. **The plan.** Staged, each stage with its own stop: (A) the inventory
   above; (B) the mutant campaign in scratch copies — one bounded scratch
   copy per mutant, `cargo test` per mutant, the red quoted verbatim, and
   the production tree untouched; (C) the duplicate collapse; (D) anything
   the inventory shows is *not* reachable by a mutant (a certificate whose
   claim cannot be broken without a disproportionate rewrite) as an open
   obligation with its cost named.
5. **The drafted implementation briefs**, in `TEMPLATE.md` shape, one per
   stage, ready for the director to rewrite.

## Hard rules

- No source change, no test change, no git command, no network.
- Do not run the instrument drivers (`logical_context_instruction_probe`,
  `logical_engine_instruction_probe`), the bounded scratch-copy falsifiers
  that need `SCRATCH_ENV`, or the feature-gated guard tests.
- Every claim carries `path:line`. A number without a citation is an open
  question.
- A scratch mutation must live in its own bounded copy under
  `.temp/brief-639/`, must never touch the repo tree, and must be removed or
  left clearly outside the repository after the run.
- Do not weaken any assertion to make a mutant red; the mutant changes
  production code, never the test.
- Report in `rum-report/1`; quote every verdict line verbatim.
- Stop and report if a classification needs a policy decision the brief does
  not settle.

## Stop conditions

- Any test whose claim cannot be stated in one line: STOP for that row, list
  it as an open question, and continue with the rest.
- Any mutant that reds a test for the wrong reason (a panic elsewhere, a
  compile error, a timeout): report it, do not count it.
- The plan must not propose editing production code to make an audit pass.
