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

## Director gate — A (2026-11-02)

Verdict: the design is accepted; the census below is the audit's record.
52 citations verified; 9 checks PASS, 2 NA. Scope clean: all 28 listed files
are scratch under `.temp/brief-639/`; `check --against-git` calls them
"unchanged in the tree" only because `.temp/` is git-ignored. No repo file
was touched. The director's own release run of the ignored set is recorded
under "Calibration outcome" above.

Spot checks from the current bytes, against the report's production sites:
`evaluation_cache/profile/report.rs:341` `.take(32)`,
`lowering/binary/left_chain.rs:31` `args.push(rhs)`,
`associations/key_reshape/traversal.rs:84`
`drop_nested_path_nodes_iterative(nested)`, `associations/mutation.rs:662`
`updated_target = replace_exact_path(...)`, `kernel_data/association_data.rs:958`
`definition.own_value = Some(updated)`, `tree_walkers/count_like.rs:329`
`usize::from(outcome.matched)`, `native_stack_guard.rs:141`
`native_stack_exhausted(current_addr, self.limit)`. All seven match.

Rulings:

- **R-639-1** Classification accepted as reported: 48 certificate, 4
  contract, 5 behaviour, 2 instrument, 1 generator, 1 falsifier; no
  inventory-class test exists among the ignored set (I55 is a generator).
- **R-639-2** DUP01–DUP06 are deliberate mirrors; DUP07–DUP09 are behaviour
  copies. Stage C has **NO ELIGIBLE COLLAPSE** under the leave-behaviour-alone
  rule; the three copies stay. The old handoff's "5 exact-duplicate body
  pairs" is superseded by the re-derived nine groups / 22 members.
- **R-639-3** I03 is green **with its environment**; the env-precondition red
  is not evidence about the gate.
- **R-639-4** I07/D01: no honest localized production mutant exists. The row
  stays an uncalibrated completion certificate; strengthening it is a
  certificate-only fixture in a later brief, never a production change.
- **R-639-5** M08/M12 are conditional (D02): a bare abort is INCONCLUSIVE and
  receives no credit.
- **R-639-6** The Stage-B campaign is authorized to build
  `rum-runtime/test-native-stack-guard-zero` for I54/I56/I57 only, in scratch
  copies; M54 and M54d run there.
- **R-639-7** The director supplies every bound from a measured clean-copy
  shape; a guessed timeout is a stop.
- **R-639-8** All 45 mutant designs run in Stage B; I07 stays open. The
  campaign makes no authoritative change; its product is evidence.
- **R-639-9** D03 (I31's no-op tail) is a real certificate weakness — the
  named transformation is not forced — and is queued for a certificate-only
  brief after the campaign. D04/D05 stay open obligations with their costs in
  `TODO.md`; they are not dispatched now.

## Report record (codex, DONE)

The record below is `python3 scripts/report.py render 639`, unedited.
## report (codex, DONE)

Design complete: all 61 ignored tests and nine duplicate groups inventoried, 45 production-mutant designs traced to observers, open obligations costed, and four TEMPLATE-shaped follow-up briefs drafted. I03 is green with its required environment. Six duplicate groups remain deliberate mirrors; three behaviour copies remain protected. No git, network, cargo/tests, source edits or inventory changes; only .temp/brief-639 scratch. No source change required a standing replay or acceptance suite. Mutants are unrun designs, not calibrated evidence.

| File | Kind | Lines | Why |
| --- | --- | --- | --- |
| `.temp/brief-639/PROGRESS.md` | added | scratch artifact | Brief-639-only survey, design, source-hash or report-validation evidence; no authoritative source edit. |
| `.temp/brief-639/assembly-1.log` | added | scratch artifact | Brief-639-only survey, design, source-hash or report-validation evidence; no authoritative source edit. |
| `.temp/brief-639/assembly-2.log` | added | scratch artifact | Brief-639-only survey, design, source-hash or report-validation evidence; no authoritative source edit. |
| `.temp/brief-639/assembly-3.log` | added | scratch artifact | Brief-639-only survey, design, source-hash or report-validation evidence; no authoritative source edit. |
| `.temp/brief-639/assembly-4.log` | added | scratch artifact | Brief-639-only survey, design, source-hash or report-validation evidence; no authoritative source edit. |
| `.temp/brief-639/audit_data.py` | added | scratch artifact | Superseded scratch working data retained for attempt history; report.json is the reviewed deliverable. |
| `.temp/brief-639/build_report.py` | added | scratch artifact | Brief-639-only survey, design, source-hash or report-validation evidence; no authoritative source edit. |
| `.temp/brief-639/coverage-final.log` | added | scratch artifact | Brief-639-only survey, design, source-hash or report-validation evidence; no authoritative source edit. |
| `.temp/brief-639/coverage.log` | added | scratch artifact | Brief-639-only survey, design, source-hash or report-validation evidence; no authoritative source edit. |
| `.temp/brief-639/duplicates.json` | added | scratch artifact | Superseded scratch working data retained for attempt history; report.json is the reviewed deliverable. |
| `.temp/brief-639/ignored.json` | added | scratch artifact | Superseded scratch working data retained for attempt history; report.json is the reviewed deliverable. |
| `.temp/brief-639/logs/survey-2.log` | added | scratch artifact | Brief-639-only survey, design, source-hash or report-validation evidence; no authoritative source edit. |
| `.temp/brief-639/logs/survey.log` | added | scratch artifact | Brief-639-only survey, design, source-hash or report-validation evidence; no authoritative source edit. |
| `.temp/brief-639/report-check-1.log` | added | scratch artifact | Brief-639-only survey, design, source-hash or report-validation evidence; no authoritative source edit. |
| `.temp/brief-639/report.json` | added | scratch artifact | Brief-639-only survey, design, source-hash or report-validation evidence; no authoritative source edit. |
| `.temp/brief-639/retained-snapshot-check.log` | added | scratch artifact | Brief-639-only survey, design, source-hash or report-validation evidence; no authoritative source edit. |
| `.temp/brief-639/review-index.json` | added | scratch artifact | Brief-639-only survey, design, source-hash or report-validation evidence; no authoritative source edit. |
| `.temp/brief-639/source-hashes.json` | added | scratch artifact | Brief-639-only survey, design, source-hash or report-validation evidence; no authoritative source edit. |
| `.temp/brief-639/source-snapshot-complete.json` | added | scratch artifact | Brief-639-only survey, design, source-hash or report-validation evidence; no authoritative source edit. |
| `.temp/brief-639/source-snapshot.json` | added | scratch artifact | Brief-639-only survey, design, source-hash or report-validation evidence; no authoritative source edit. |
| `.temp/brief-639/survey-complete.json` | added | scratch artifact | Brief-639-only survey, design, source-hash or report-validation evidence; no authoritative source edit. |
| `.temp/brief-639/survey-complete.log` | added | scratch artifact | Brief-639-only survey, design, source-hash or report-validation evidence; no authoritative source edit. |
| `.temp/brief-639/survey.json` | added | scratch artifact | Brief-639-only survey, design, source-hash or report-validation evidence; no authoritative source edit. |
| `.temp/brief-639/survey.log` | added | scratch artifact | Brief-639-only survey, design, source-hash or report-validation evidence; no authoritative source edit. |
| `.temp/brief-639/survey.py` | added | scratch artifact | Brief-639-only survey, design, source-hash or report-validation evidence; no authoritative source edit. |
| `.temp/brief-639/tests.json` | added | scratch artifact | Superseded scratch working data retained for attempt history; report.json is the reviewed deliverable. |
| `.temp/brief-639/verify_survey.py` | added | scratch artifact | Brief-639-only survey, design, source-hash or report-validation evidence; no authoritative source edit. |
| `.temp/brief-639/report-check-2.log` | added | scratch artifact | Brief-639-only report validation evidence; no authoritative source edit. |

| Run | Try | Exit | Tests | Log |
| --- | ---: | ---: | --- | --- |
| survey-early-1 | 1 | 1 | - | `.temp/brief-639/logs/survey.log` |
| survey-early-2 | 2 | 0 | - | `.temp/brief-639/logs/survey-2.log` |
| survey-rederive | 3 | 0 | - | `.temp/brief-639/survey.log` |
| survey-complete | 4 | 0 | - | `.temp/brief-639/survey-complete.log` |
| coverage-1 | 1 | 0 | - | `.temp/brief-639/coverage.log` |
| coverage-2 | 2 | 0 | - | `.temp/brief-639/coverage-final.log` |
| retained-snapshot | 1 | 0 | - | `.temp/brief-639/retained-snapshot-check.log` |
| assembly-1 | 1 | 0 | - | `.temp/brief-639/assembly-1.log` |
| assembly-2 | 2 | 0 | - | `.temp/brief-639/assembly-2.log` |
| assembly-3 | 3 | 0 | - | `.temp/brief-639/assembly-3.log` |
| assembly-4 | 4 | 0 | - | `.temp/brief-639/assembly-4.log` |
| report-check-1 | 1 | 0 | - | `.temp/brief-639/report-check-1.log` |
| report-check-2 | 2 | 0 | - | `.temp/brief-639/report-check-2.log` |

| Check | Bar | Result | Verdict | Note |
| --- | --- | --- | --- | --- |
| inventory | Account for all ignored tests and classify from code | 61 rows: 48 certificate, 4 contract, 5 behaviour, 2 instrument, 1 generator, 1 falsifier; 0 inventory | PASS | Every required class maps to a mutation design or explicit D01; no candidate is claimed calibrated. |
| duplicates | Rederive every normalized duplicate body and resolve bindings | 9 groups; 6 deliberate mirrors, 3 protected behaviour copies | PASS | No edits authorized; shared-form proposals and D07 explain why no collapse is performed. |
| profile-env | Report historical I03 with its environment | Designed precondition red in first director log; env-enabled rerun passed | PASS | Neither a defect nor an inventory-ordering artifact; no rerun here. |
| mutant-calibration | Design only; never substitute a predicted red for an observed red | 45 designs; I07 open; drop evidence and broader observer claims remain explicit obligations | PASS | The design is complete; Stage B calibration and D obligations require later director dispatch. |
| standing-replay | Replay after covered source edits before first acceptance | No covered source edit; no acceptance suite; no replay executed | NA | No replay verdict is invented. Copy and future implementation plans preserve the mandatory ordering and forbid inventory weakening. |
| acceptance | No source change means no test gate | No worker-run tests, guards, instrument drivers, falsifiers or calibration | NA | Director logs are historical evidence in a separate section, never attributed to this worker. |
| source-snapshot-1 | Raw ignore set reconciles and surveyed Rust bytes remain unchanged | Independent reconciliation and source hashes pass | PASS | This is a filesystem hash comparison, not git status; it covers all surveyed Rust files. |
| source-snapshot-2 | Raw ignore set reconciles and surveyed Rust bytes remain unchanged | Independent reconciliation and source hashes pass | PASS | This is a filesystem hash comparison, not git status; it covers all surveyed Rust files. |
| survey-parser-history | Preserve failed survey attempts and resolve macro-template gaps | Early scratch parser assertion and later 12-template gap resolved; final census reconciles | PASS | The earlier nonzero exit is a scratch-tool defect, not a Rust test regression; all attempts are retained and quoted. |
| retained-source-hashes | Retained broad source/document/script snapshot unchanged | 2584 entries checked; zero mismatches | PASS | Independent of the 1519-Rust-file census; does not use git. |
| report-format | Validate rum-report/1 without accessing git | check: OK, 50 citations verified (report.json) | PASS | Final summary also validates the report after this receipt is recorded. No --against-git invocation. |

### Scope, method and classification

Design-only execution of brief 639: no git command, network, cargo/test invocation, source/test/doc/inventory edit or write outside .temp/brief-639. Scratch files are listed in changes, not production modifications. Workspace members are listed at `Cargo.toml:3`. The scanner traversed their Rust files, masked literals/comments for balanced bodies, and resolved each source-level #[test] site. No compiled macro-expansion census is claimed. Counts and reconciliation are in coverage.log and coverage-final.log. Source hash checks compare filesystem bytes, not inferred git cleanliness.

Retained survey history is included in full: logs/survey.log contains an early assertion failure on the macro template at crates/rum-stdlib/tests/suspension_oracle/fixtures/definition_application.rs:34; logs/survey-2.log records its successful macro-aware repair. A subsequent extraction revision in survey.log again left 12 macro templates unresolved; survey-complete.log explicitly resolves all of them. These are scratch-parser attempts, not Rust test runs or production regressions. tests.json, ignored.json, duplicates.json and audit_data.py are superseded working data, not the final reviewed table. The authoritative deliverable is this report; review-index.json supports its reconciliation. The earlier broad source-hashes.json was also rechecked: 2584 retained source/document/script file hashes match (retained-snapshot-check.log:2).

Classification totals: 48 certificate, 4 contract, 5 behaviour, 2 instrument, 1 generator, 1 falsifier, 0 inventory (review-index.json; coverage.log). Certificates explicitly probe deployment shape/stack/completion; contracts pin report or forced-resource refusal. Behaviour stress tests remain behaviour even when ignored. Instruments need external drivers. I55 writes a review copy and installs no acceptance marks (`crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs:25`). I61 is an intentional private-copy entry, not a test expected to pass when invoked naked (`crates/rum-stdlib/tests/typed_slot_census.rs:232`). No row needs an unsettled classification policy.

Source references in the mutant table use test:N relative to each listed row file and production:N relative to that mutant's production file. In prose, a subsequent :N continues the immediately named full path. brief:N means docs/dev/codex/brief-639-test-audit-inventory.md:N. All claims are limited to their listed observer; candidate red predictions are not run evidence. No standing inventory replay or acceptance suite was applicable because covered source was not edited; there are no replay verdict lines to quote and none are fabricated. Future ordering and replay rules are explicit in plan/brief-B/brief-C.

Read-only evidence verdicts:

```text
SURVEY files=1519 test_functions=3816 ignored=61 duplicate_groups=9 unresolved=0
NORMALIZATION whitespace_removed_groups=9 whitespace_runs_collapsed_groups=9
TEMPLATES macro_test_templates=12 concrete_named_test_functions=3804
```

`.temp/brief-639/logs/survey.log`

```text
AssertionError: ('crates/rum-stdlib/tests/suspension_oracle/fixtures/definition_application.rs', 34)
```

`.temp/brief-639/logs/survey-2.log`

```text
PASS ignored census: 61 test functions; 61 live ignore attributes; every attribute linked to one test.
PASS duplicate census: 3816 syntactic test functions in 1519 Rust files (including 12 macro-template functions); 9 groups / 22 members after deleting whitespace; 9 groups after collapsing whitespace.
PASS source snapshot: 2584 non-hidden source/document/script files hashed; no repository content changed by survey.
```

`.temp/brief-639/coverage.log`

```text
IGNORE_RECONCILIATION PASS raw=61 extracted=61 reported=61 missing=0 extra=0
SOURCE_SNAPSHOT PASS rust_files=1519 modified=0 added=0 removed=0 original_and_complete_snapshots_match=1
CLAIM_COVERAGE PASS rows=61 required=52 mutant_or_open=52 open_without_local_mutant=I07 candidate_designs=45
DUPLICATE_RECONCILIATION PASS groups=9 both_normalizations_match=1 identical_string_literals=1 unresolved=0
CLASSIFICATION PASS certificate=48 contract=4 inventory=0 behaviour=5 instrument=2 generator=1 falsifier=1
```

`.temp/brief-639/coverage-final.log`

```text
IGNORE_RECONCILIATION PASS raw=61 extracted=61 reported=61 missing=0 extra=0
SOURCE_SNAPSHOT PASS rust_files=1519 modified=0 added=0 removed=0 original_and_complete_snapshots_match=1
CLAIM_COVERAGE PASS rows=61 required=52 mutant_or_open=52 open_without_local_mutant=I07 candidate_designs=45
DUPLICATE_RECONCILIATION PASS groups=9 both_normalizations_match=1 identical_string_literals=1 unresolved=0
CLASSIFICATION PASS certificate=48 contract=4 inventory=0 behaviour=5 instrument=2 generator=1 falsifier=1
```

```text
SNAPSHOT_FORMAT dict
RETAINED_SNAPSHOT checked=2584 mismatches=0
```

### All 61 ignored tests

Complete ignored-test census. Attribute locations and literal reasons come from the source survey; function locations distinguish the attribute from the test. `none (#[ignore])` means the source supplies no reason, not that a reason was inferred. All execution in this report is historical director evidence; no tests were run by this worker. Profile row I03 is green **with its environment**, never unqualified green.

| ID / class | Function and source | Ignore reason as written | Required mode / feature / environment | One-line claim and assertion sites | Director evidence / candidate |
| --- | --- | --- | --- | --- | --- |
| I01 / **behaviour** | `cancel_rewrite_loop_stability_10_cycles`; fn `crates/rum-host/tests/stress.rs:126`; ignore `crates/rum-host/tests/stress.rs:125` | none (`#[ignore]`) | no required profile/extra feature/env in the test; explicit ignored selection; director used release. | Ten cancel/recover cycles terminate the rewrite request and permit a fresh evaluation. `crates/rum-host/tests/stress.rs:136`, `crates/rum-host/tests/stress.rs:32`, `crates/rum-host/tests/stress.rs:33`, `crates/rum-host/tests/stress.rs:41`, `crates/rum-host/tests/stress.rs:142`, `crates/rum-host/tests/stress.rs:26` | director green `.temp/director/audit-638/ignored-release.log:203`; mutant not required for this class |
| I02 / **behaviour** | `battered_session_recovers_after_repeated_recoverable_limit_trips`; fn `crates/rum-host/tests/stress.rs:293`; ignore `crates/rum-host/tests/stress.rs:292` | none (`#[ignore]`) | no required profile/extra feature/env in the test; explicit ignored selection; director used release. | Three rounds of six recoverable limit trips preserve a usable session on the declared 8 MiB stack. `crates/rum-host/tests/stress.rs:15`, `crates/rum-host/tests/stress.rs:295`, `crates/rum-host/tests/stress.rs:307`, `crates/rum-host/tests/stress.rs:278`, `crates/rum-host/tests/stress.rs:279`, `crates/rum-host/tests/stress.rs:310`, `crates/rum-host/tests/stress.rs:320`, `crates/rum-host/tests/stress.rs:330`, `crates/rum-host/tests/stress.rs:340`, `crates/rum-host/tests/stress.rs:350`, `crates/rum-host/tests/stress.rs:359`, `crates/rum-host/tests/stress.rs:368` | director green `.temp/director/audit-638/ignored-release.log:202`; mutant not required for this class |
| I03 / **contract** | `runtime::evaluation_cache::tests::call_stability_profile_reports_per_source_head_outcomes`; fn `crates/rum-runtime/src/runtime/evaluation_cache/tests.rs:530`; ignore `crates/rum-runtime/src/runtime/evaluation_cache/tests.rs:529` | `release-only profiling gate; run with RUM_PROFILE_EVALUATION_CACHE=1` | release; no special feature; RUM_PROFILE_EVALUATION_CACHE=1; child marker managed by the test (tests.rs:537). | With profiling enabled, a successful child request emits the expected source/head outcome row. `crates/rum-runtime/src/runtime/evaluation_cache/tests.rs:531`, `crates/rum-runtime/src/runtime/evaluation_cache/tests.rs:545`, `crates/rum-runtime/src/runtime/evaluation_cache/tests.rs:566`, `crates/rum-runtime/src/runtime/evaluation_cache/tests.rs:578`, `crates/rum-runtime/src/runtime/evaluation_cache/tests.rs:583` | GREEN WITH ITS ENVIRONMENT: `.temp/director/audit-638/profile-gate-release.log:27` (designed env-precondition red: `.temp/director/audit-638/ignored-release.log:252`, `.temp/director/audit-638/ignored-release.log:261`).; M03 |
| I04 / **instrument** | `runtime::profiling::tests::context_scaling::logical_context_instruction_probe`; fn `crates/rum-runtime/src/runtime/profiling/tests/context_scaling.rs:47`; ignore `crates/rum-runtime/src/runtime/profiling/tests/context_scaling.rs:46` | `run explicitly with perf stat --delay=-1 and acknowledged control FIFOs` | driver-selected build (no test-enforced release cfg); no special feature; LP02_PROBE_SIZE/REPEATS/OPERATION, LP02_PERF_CONTROL/ACK; perf --delay=-1; forbidden in this audit. | The selected logical-context operation completes with acknowledged perf control and a balanced, valid epoch. `crates/rum-runtime/src/runtime/profiling/tests/context_scaling.rs:35`, `crates/rum-runtime/src/runtime/profiling/tests/context_scaling.rs:104`, `crates/rum-runtime/src/runtime/profiling/tests/context_scaling.rs:105`, `crates/rum-runtime/src/runtime/profiling/tests/context_scaling.rs:107` | excluded from director calibration; not run here; mutant not required for this class |
| I05 / **instrument** | `runtime::profiling::tests::engine_scaling::logical_engine_instruction_probe`; fn `crates/rum-runtime/src/runtime/profiling/tests/engine_scaling.rs:12`; ignore `crates/rum-runtime/src/runtime/profiling/tests/engine_scaling.rs:11` | `run explicitly under perf stat; sizes and repetitions supplied by the driver` | driver-selected build (no test-enforced release cfg); no special feature; LP02_PROBE_SIZE/REPEATS/AXIS/OPERATION; external perf; forbidden in this audit. | The selected engine operation completes and closes a valid logical epoch for an external instruction measurement. `crates/rum-runtime/src/runtime/profiling/tests/engine_scaling.rs:51`, `crates/rum-runtime/src/runtime/profiling/tests/engine_scaling.rs:93`, `crates/rum-runtime/src/runtime/profiling/tests/engine_scaling.rs:105`, `crates/rum-runtime/src/runtime/profiling/tests/engine_scaling.rs:120` | excluded from director calibration; not run here; mutant not required for this class |
| I06 / **certificate** | `runtime::tests::left_chain_lowering::left_chain_two_mib_full_round_trip_accepts_and_rejects_without_abort`; fn `crates/rum-runtime/src/runtime/tests/left_chain_lowering.rs:320`; ignore `crates/rum-runtime/src/runtime/tests/left_chain_lowering.rs:319` | `release-only deployment-stack probe; run explicitly with --release` | release, explicit ignored selection; no special feature/env; declared 2 MiB worker. | On 2 MiB, the 4096-operator additive chain lowers with the expected arity, formats, and drops; 4097 and 20000 operators are rejected with the left-chain diagnostic. `crates/rum-runtime/src/runtime/tests/left_chain_lowering.rs:327`, `crates/rum-runtime/src/runtime/tests/left_chain_lowering.rs:434`, `crates/rum-runtime/src/runtime/tests/left_chain_lowering.rs:435`, `crates/rum-runtime/src/runtime/tests/left_chain_lowering.rs:329`, `crates/rum-runtime/src/runtime/tests/left_chain_lowering.rs:344`, `crates/rum-runtime/src/runtime/tests/left_chain_lowering.rs:355`, `crates/rum-runtime/src/runtime/tests/left_chain_lowering.rs:357`, `crates/rum-runtime/src/runtime/tests/left_chain_lowering.rs:360`, `crates/rum-runtime/src/runtime/tests/left_chain_lowering.rs:381` | director green `.temp/director/audit-638/ignored-release.log:253`; M06, M06b |
| I07 / **certificate** | `runtime::tests::left_chain_lowering::left_chain_two_mib_nested_expr_drops_naturally_at_the_boundary`; fn `crates/rum-runtime/src/runtime/tests/left_chain_lowering.rs:568`; ignore `crates/rum-runtime/src/runtime/tests/left_chain_lowering.rs:567` | `release-only deployment-stack probe; run explicitly with --release` | release, explicit ignored selection; no special feature/env; declared 2 MiB worker. | For seven boundary-sized nested-operator fixtures, parse, lower, nonempty format and natural Expr drop finish on 2 MiB. `crates/rum-runtime/src/runtime/tests/left_chain_lowering.rs:569`, `crates/rum-runtime/src/runtime/tests/left_chain_lowering.rs:573`, `crates/rum-runtime/src/runtime/tests/left_chain_lowering.rs:575`, `crates/rum-runtime/src/runtime/tests/left_chain_lowering.rs:577`, `crates/rum-runtime/src/runtime/tests/left_chain_lowering.rs:579`, `crates/rum-runtime/src/runtime/tests/left_chain_lowering.rs:381`, `crates/rum-runtime/src/runtime/tests/left_chain_lowering.rs:382` | director green `.temp/director/audit-638/ignored-release.log:254`; D01: no honest localized drop mutant identified |
| I08 / **certificate** | `associations::basic::lookup::tests::lookup_outcome_drop_is_bounded_at_physical_height_4097_on_two_mib`; fn `crates/rum-stdlib/src/associations/basic/lookup.rs:849`; ignore `crates/rum-stdlib/src/associations/basic/lookup.rs:848` | `release-only deployment-stack drop certificate; run explicitly with --release` | release, explicit ignored selection; no special feature/env; declared 2 MiB worker. | Dropping a uniquely built LookupOutcome list chain of physical height 4097 completes on 2 MiB. `crates/rum-stdlib/src/associations/basic/lookup.rs:854`, `crates/rum-stdlib/src/associations/basic/lookup.rs:859`, `crates/rum-stdlib/src/associations/basic/lookup.rs:863` | director green `.temp/director/audit-638/ignored-release.log:274`; M08 |
| I09 / **certificate** | `tests::association_key_reshape_depth::association_key_flatten_completes_at_physical_height_4097_on_two_mib`; fn `crates/rum-stdlib/src/tests/association_key_reshape_depth.rs:259`; ignore `crates/rum-stdlib/src/tests/association_key_reshape_depth.rs:258` | `release-only deployment-stack probe; run explicitly with --release` | release, explicit ignored selection; no special feature/env; declared 2 MiB worker. | KeyFlatten preserves the exact 2048-key path and delayed terminal value at checked physical height 4097 on 2 MiB. `crates/rum-stdlib/src/tests/association_key_reshape_depth.rs:265`, `crates/rum-stdlib/src/tests/association_key_reshape_depth.rs:95`, `crates/rum-stdlib/src/tests/association_key_reshape_depth.rs:96`, `crates/rum-stdlib/src/tests/association_key_reshape_depth.rs:269` | director green `.temp/director/audit-638/ignored-release.log:296`; M09 |
| I10 / **certificate** | `tests::association_key_reshape_depth::association_key_deflatten_completes_at_physical_height_4097_on_two_mib`; fn `crates/rum-stdlib/src/tests/association_key_reshape_depth.rs:275`; ignore `crates/rum-stdlib/src/tests/association_key_reshape_depth.rs:274` | `release-only deployment-stack probe; run explicitly with --release` | release, explicit ignored selection; no special feature/env; declared 2 MiB worker. | KeyDeflatten reconstructs the checked 4097-height association chain with exact keys and the delayed sentinel on 2 MiB. `crates/rum-stdlib/src/tests/association_key_reshape_depth.rs:283`, `crates/rum-stdlib/src/tests/association_key_reshape_depth.rs:95`, `crates/rum-stdlib/src/tests/association_key_reshape_depth.rs:284`, `crates/rum-stdlib/src/tests/association_key_reshape_depth.rs:108`, `crates/rum-stdlib/src/tests/association_key_reshape_depth.rs:114`, `crates/rum-stdlib/src/tests/association_key_reshape_depth.rs:115`, `crates/rum-stdlib/src/tests/association_key_reshape_depth.rs:122` | director green `.temp/director/audit-638/ignored-release.log:278`; M10 |
| I11 / **certificate** | `tests::association_key_reshape_depth::association_key_deflatten_flatten_round_trip_is_exact_at_physical_height_4097`; fn `crates/rum-stdlib/src/tests/association_key_reshape_depth.rs:290`; ignore `crates/rum-stdlib/src/tests/association_key_reshape_depth.rs:289` | `release-only deployment-stack probe; run explicitly with --release` | release, explicit ignored selection; no special feature/env; declared 2 MiB worker. | Deflatten then Flatten preserves both paths and the delayed terminal exactly at checked height 4097 on 2 MiB. `crates/rum-stdlib/src/tests/association_key_reshape_depth.rs:304`, `crates/rum-stdlib/src/tests/association_key_reshape_depth.rs:95`, `crates/rum-stdlib/src/tests/association_key_reshape_depth.rs:306` | director green `.temp/director/audit-638/ignored-release.log:295`; M09, M10 |
| I12 / **certificate** | `tests::association_key_reshape_depth::association_key_deflatten_abandons_a_deep_partial_tree_without_growing_the_stack`; fn `crates/rum-stdlib/src/tests/association_key_reshape_depth.rs:325`; ignore `crates/rum-stdlib/src/tests/association_key_reshape_depth.rs:324` | `release-only deployment-stack probe; run explicitly with --release` | release, explicit ignored selection; no special feature/env; declared 2 MiB worker. | An invalid empty path abandons the preceding 60000-key partial deflatten tree and returns the original inert call on 2 MiB. `crates/rum-stdlib/src/tests/association_key_reshape_depth.rs:330`, `crates/rum-stdlib/src/tests/association_key_reshape_depth.rs:341`, `crates/rum-stdlib/src/tests/association_key_reshape_depth.rs:347`, `crates/rum-stdlib/src/tests/association_key_reshape_depth.rs:17` | director green `.temp/director/audit-638/ignored-release.log:285`; M12 |
| I13 / **certificate** | `tests::association_key_take_drop_depth::key_take_completes_at_physical_height_4097_on_two_mib`; fn `crates/rum-stdlib/src/tests/association_key_take_drop_depth.rs:216`; ignore `crates/rum-stdlib/src/tests/association_key_take_drop_depth.rs:215` | `release-only deployment-stack probe; run explicitly with --release` | release, explicit ignored selection; no special feature/env; declared 2 MiB worker. | KeyTake preserves all deep list wrappers and selects only the delayed take entry at checked height 4097 on 2 MiB. `crates/rum-stdlib/src/tests/association_key_take_drop_depth.rs:225`, `crates/rum-stdlib/src/tests/association_key_take_drop_depth.rs:93`, `crates/rum-stdlib/src/tests/association_key_take_drop_depth.rs:94`, `crates/rum-stdlib/src/tests/association_key_take_drop_depth.rs:230`, `crates/rum-stdlib/src/tests/association_key_take_drop_depth.rs:113` | director green `.temp/director/audit-638/ignored-release.log:277`; M13 |
| I14 / **certificate** | `tests::association_key_take_drop_depth::key_drop_completes_at_physical_height_4097_on_two_mib`; fn `crates/rum-stdlib/src/tests/association_key_take_drop_depth.rs:236`; ignore `crates/rum-stdlib/src/tests/association_key_take_drop_depth.rs:235` | `release-only deployment-stack probe; run explicitly with --release` | release, explicit ignored selection; no special feature/env; declared 2 MiB worker. | KeyDrop preserves all deep list wrappers and retains only the delayed keep entry at checked height 4097 on 2 MiB. `crates/rum-stdlib/src/tests/association_key_take_drop_depth.rs:249`, `crates/rum-stdlib/src/tests/association_key_take_drop_depth.rs:93`, `crates/rum-stdlib/src/tests/association_key_take_drop_depth.rs:94`, `crates/rum-stdlib/src/tests/association_key_take_drop_depth.rs:254`, `crates/rum-stdlib/src/tests/association_key_take_drop_depth.rs:113` | director green `.temp/director/audit-638/ignored-release.log:276`; M13 |
| I15 / **certificate** | `tests::association_keys_depth::keys_completes_at_physical_height_4097_on_two_mib`; fn `crates/rum-stdlib/src/tests/association_keys_depth.rs:206`; ignore `crates/rum-stdlib/src/tests/association_keys_depth.rs:205` | `release-only deployment-stack probe; run explicitly with --release` | release, explicit ignored selection; no special feature/env; declared 2 MiB worker. | Keys preserves the checked deep list shape, key order and requested wrapper on 2 MiB without messages. `crates/rum-stdlib/src/tests/association_keys_depth.rs:216`, `crates/rum-stdlib/src/tests/association_keys_depth.rs:53`, `crates/rum-stdlib/src/tests/association_keys_depth.rs:55`, `crates/rum-stdlib/src/tests/association_keys_depth.rs:224`, `crates/rum-stdlib/src/tests/association_keys_depth.rs:90`, `crates/rum-stdlib/src/tests/association_keys_depth.rs:97` | director green `.temp/director/audit-638/ignored-release.log:275`; M15 |
| I16 / **certificate** | `tests::association_lookup_depth::lookup_hit_completes_at_physical_height_4097_on_two_mib`; fn `crates/rum-stdlib/src/tests/association_lookup_depth.rs:200`; ignore `crates/rum-stdlib/src/tests/association_lookup_depth.rs:199` | `release-only deployment-stack probe; run explicitly with --release` | release, explicit ignored selection; no special feature/env; declared 2 MiB worker. | Lookup resolves the hit through the deep key-spec shape and preserves its requested wrapper on 2 MiB without messages. `crates/rum-stdlib/src/tests/association_lookup_depth.rs:204`, `crates/rum-stdlib/src/tests/association_lookup_depth.rs:61`, `crates/rum-stdlib/src/tests/association_lookup_depth.rs:216`, `crates/rum-stdlib/src/tests/association_lookup_depth.rs:102`, `crates/rum-stdlib/src/tests/association_lookup_depth.rs:113`, `crates/rum-stdlib/src/tests/association_lookup_depth.rs:117` | director green `.temp/director/audit-638/ignored-release.log:280`; M16 |
| I17 / **certificate** | `tests::association_lookup_depth::lookup_miss_and_resume_drop_complete_at_physical_height_4097_on_two_mib`; fn `crates/rum-stdlib/src/tests/association_lookup_depth.rs:225`; ignore `crates/rum-stdlib/src/tests/association_lookup_depth.rs:224` | `release-only deployment-stack probe; run explicitly with --release` | release, explicit ignored selection; no special feature/env; declared 2 MiB worker. | Deep Lookup returns the missing result, suspends for the default, resumes with the supplied string, and completes cleanup on 2 MiB. `crates/rum-stdlib/src/tests/association_lookup_depth.rs:237`, `crates/rum-stdlib/src/tests/association_lookup_depth.rs:253`, `crates/rum-stdlib/src/tests/association_lookup_depth.rs:256`, `crates/rum-stdlib/src/tests/association_lookup_depth.rs:264`, `crates/rum-stdlib/src/tests/association_lookup_depth.rs:265`, `crates/rum-stdlib/src/tests/association_lookup_depth.rs:268`, `crates/rum-stdlib/src/tests/association_lookup_depth.rs:102`, `crates/rum-stdlib/src/tests/association_lookup_depth.rs:113`, `crates/rum-stdlib/src/tests/association_lookup_depth.rs:117` | director green `.temp/director/audit-638/ignored-release.log:281`; M08, M17a, M17b |
| I18 / **certificate** | `tests::association_values_depth::values_completes_at_physical_height_4097_on_two_mib`; fn `crates/rum-stdlib/src/tests/association_values_depth.rs:206`; ignore `crates/rum-stdlib/src/tests/association_values_depth.rs:205` | `release-only deployment-stack probe; run explicitly with --release` | release, explicit ignored selection; no special feature/env; declared 2 MiB worker. | Values preserves the checked deep list shape, value order and requested wrapper on 2 MiB without messages. `crates/rum-stdlib/src/tests/association_values_depth.rs:216`, `crates/rum-stdlib/src/tests/association_values_depth.rs:53`, `crates/rum-stdlib/src/tests/association_values_depth.rs:55`, `crates/rum-stdlib/src/tests/association_values_depth.rs:224`, `crates/rum-stdlib/src/tests/association_values_depth.rs:90`, `crates/rum-stdlib/src/tests/association_values_depth.rs:97` | director green `.temp/director/audit-638/ignored-release.log:279`; M18 |
| I19 / **certificate** | `tests::delete_missing_depth::delete_missing_recursive_completes_at_physical_height_4097_on_two_mib`; fn `crates/rum-stdlib/src/tests/delete_missing_depth.rs:177`; ignore `crates/rum-stdlib/src/tests/delete_missing_depth.rs:176` | `release-only deployment-stack probe; run explicitly with --release` | release, explicit ignored selection; no special feature/env; declared 2 MiB worker. | Recursive DeleteMissing preserves the deep wrappers and matches a shallow expected centre at checked height 4097 without messages on 2 MiB. `crates/rum-stdlib/src/tests/delete_missing_depth.rs:187`, `crates/rum-stdlib/src/tests/delete_missing_depth.rs:96`, `crates/rum-stdlib/src/tests/delete_missing_depth.rs:102`, `crates/rum-stdlib/src/tests/delete_missing_depth.rs:188`, `crates/rum-stdlib/src/tests/delete_missing_depth.rs:194`, `crates/rum-stdlib/src/tests/delete_missing_depth.rs:195`, `crates/rum-stdlib/src/tests/delete_missing_depth.rs:114`, `crates/rum-stdlib/src/tests/delete_missing_depth.rs:120`, `crates/rum-stdlib/src/tests/delete_missing_depth.rs:121`, `crates/rum-stdlib/src/tests/delete_missing_depth.rs:127` | director green `.temp/director/audit-638/ignored-release.log:283`; M19 |
| I20 / **certificate** | `tests::delete_missing_depth::delete_missing_recursive_armed_completes_at_physical_height_4097_on_two_mib`; fn `crates/rum-stdlib/src/tests/delete_missing_depth.rs:201`; ignore `crates/rum-stdlib/src/tests/delete_missing_depth.rs:200` | `release-only deployment-stack probe; run explicitly with --release` | release, explicit ignored selection; no special feature/env; declared 2 MiB worker. | MemoryConstrained recursive DeleteMissing preserves the deep wrappers and matches its shallow delayed-value centre without messages on 2 MiB. `crates/rum-stdlib/src/tests/delete_missing_depth.rs:208`, `crates/rum-stdlib/src/tests/delete_missing_depth.rs:218`, `crates/rum-stdlib/src/tests/delete_missing_depth.rs:219`, `crates/rum-stdlib/src/tests/delete_missing_depth.rs:114`, `crates/rum-stdlib/src/tests/delete_missing_depth.rs:120`, `crates/rum-stdlib/src/tests/delete_missing_depth.rs:121`, `crates/rum-stdlib/src/tests/delete_missing_depth.rs:127` | director green `.temp/director/audit-638/ignored-release.log:325`; M19 |
| I21 / **certificate** | `tests::delete_missing_depth::delete_missing_scan_completes_at_physical_height_4097_on_two_mib`; fn `crates/rum-stdlib/src/tests/delete_missing_depth.rs:225`; ignore `crates/rum-stdlib/src/tests/delete_missing_depth.rs:224` | `release-only deployment-stack probe; run explicitly with --release` | release, explicit ignored selection; no special feature/env; declared 2 MiB worker. | DeleteMissing scan All removes the deep branch containing Missing and returns exactly the kept scan sentinel on 2 MiB. `crates/rum-stdlib/src/tests/delete_missing_depth.rs:235`, `crates/rum-stdlib/src/tests/delete_missing_depth.rs:236` | director green `.temp/director/audit-638/ignored-release.log:282`; M21 |
| I22 / **certificate** | `tests::delete_missing_target_certificates::delete_missing_target_scan_zero_completes_at_physical_height_4097_on_two_mib`; fn `crates/rum-stdlib/src/tests/delete_missing_target_certificates.rs:138`; ignore `crates/rum-stdlib/src/tests/delete_missing_target_certificates.rs:137` | `release-only deployment-stack probe; run explicitly with --release` | release, explicit ignored selection; no special feature/env; declared 2 MiB worker. | Target DeleteMissing with scan 0 removes the target Missing entry and preserves the exact sibling witnesses at checked height 4097 on 2 MiB. `crates/rum-stdlib/src/tests/delete_missing_target_certificates.rs:142`, `crates/rum-stdlib/src/tests/delete_missing_target_certificates.rs:96`, `crates/rum-stdlib/src/tests/delete_missing_target_certificates.rs:97`, `crates/rum-stdlib/src/tests/delete_missing_target_certificates.rs:151`, `crates/rum-stdlib/src/tests/delete_missing_target_certificates.rs:152`, `crates/rum-stdlib/src/tests/delete_missing_target_certificates.rs:111`, `crates/rum-stdlib/src/tests/delete_missing_target_certificates.rs:116`, `crates/rum-stdlib/src/tests/delete_missing_target_certificates.rs:126` | director green `.temp/director/audit-638/ignored-release.log:286`; M22 |
| I23 / **certificate** | `tests::delete_missing_target_certificates::delete_missing_target_scan_all_completes_at_physical_height_4097_on_two_mib`; fn `crates/rum-stdlib/src/tests/delete_missing_target_certificates.rs:158`; ignore `crates/rum-stdlib/src/tests/delete_missing_target_certificates.rs:157` | `release-only deployment-stack probe; run explicitly with --release` | release, explicit ignored selection; no special feature/env; declared 2 MiB worker. | Target DeleteMissing with scan All removes the branch containing the unique nested Missing and preserves the exact sibling witnesses at checked height 4097 on 2 MiB. `crates/rum-stdlib/src/tests/delete_missing_target_certificates.rs:162`, `crates/rum-stdlib/src/tests/delete_missing_target_certificates.rs:96`, `crates/rum-stdlib/src/tests/delete_missing_target_certificates.rs:97`, `crates/rum-stdlib/src/tests/delete_missing_target_certificates.rs:168`, `crates/rum-stdlib/src/tests/delete_missing_target_certificates.rs:169`, `crates/rum-stdlib/src/tests/delete_missing_target_certificates.rs:111`, `crates/rum-stdlib/src/tests/delete_missing_target_certificates.rs:116`, `crates/rum-stdlib/src/tests/delete_missing_target_certificates.rs:126` | director green `.temp/director/audit-638/ignored-release.log:284`; M23 |
| I24 / **certificate** | `tests::delete_missing_target_certificates::delete_missing_target_list_clears_64000_siblings_on_two_mib`; fn `crates/rum-stdlib/src/tests/delete_missing_target_certificates.rs:175`; ignore `crates/rum-stdlib/src/tests/delete_missing_target_certificates.rs:174` | `release-only width probe; run explicitly with --release` | release, explicit ignored selection; no special feature/env; declared 2 MiB worker. | Target DeleteMissing clears a 64000-element list down to the exact first, middle and last survivors on 2 MiB. `crates/rum-stdlib/src/tests/delete_missing_target_certificates.rs:178`, `crates/rum-stdlib/src/tests/delete_missing_target_certificates.rs:196` | director green `.temp/director/audit-638/ignored-release.log:292`; M24 |
| I25 / **certificate** | `tests::delete_missing_target_certificates::delete_missing_target_association_clears_64000_siblings_on_two_mib`; fn `crates/rum-stdlib/src/tests/delete_missing_target_certificates.rs:209`; ignore `crates/rum-stdlib/src/tests/delete_missing_target_certificates.rs:208` | `release-only width probe; run explicitly with --release` | release, explicit ignored selection; no special feature/env; declared 2 MiB worker. | Target DeleteMissing clears a 64000-entry association down to the exact ordered survivor rules on 2 MiB. `crates/rum-stdlib/src/tests/delete_missing_target_certificates.rs:212`, `crates/rum-stdlib/src/tests/delete_missing_target_certificates.rs:240` | director green `.temp/director/audit-638/ignored-release.log:293`; M25 |
| I26 / **certificate** | `tests::held_association_exact_path_depth::associate_to_part_target_completes_at_physical_height_4097_on_two_mib`; fn `crates/rum-stdlib/src/tests/held_association_exact_path_depth.rs:482`; ignore `crates/rum-stdlib/src/tests/held_association_exact_path_depth.rs:481` | `release-only deployment-stack probe; run explicitly with --release` | release, explicit ignored selection; no special feature/env; declared 2 MiB worker. | AssociateTo through a deep Part path returns and stores the updated owner, with unchanged siblings and exact new target on 2 MiB. `crates/rum-stdlib/src/tests/held_association_exact_path_depth.rs:491`, `crates/rum-stdlib/src/tests/held_association_exact_path_depth.rs:494`, `crates/rum-stdlib/src/tests/held_association_exact_path_depth.rs:516`, `crates/rum-stdlib/src/tests/held_association_exact_path_depth.rs:521`, `crates/rum-stdlib/src/tests/held_association_exact_path_depth.rs:129`, `crates/rum-stdlib/src/tests/held_association_exact_path_depth.rs:135`, `crates/rum-stdlib/src/tests/held_association_exact_path_depth.rs:140`, `crates/rum-stdlib/src/tests/held_association_exact_path_depth.rs:145`, `crates/rum-stdlib/src/tests/held_association_exact_path_depth.rs:152` | director green `.temp/director/audit-638/ignored-release.log:288`; M26 |
| I27 / **certificate** | `tests::held_association_exact_path_depth::key_drop_from_part_target_completes_at_physical_height_4097_on_two_mib`; fn `crates/rum-stdlib/src/tests/held_association_exact_path_depth.rs:531`; ignore `crates/rum-stdlib/src/tests/held_association_exact_path_depth.rs:530` | `release-only deployment-stack probe; run explicitly with --release` | release, explicit ignored selection; no special feature/env; declared 2 MiB worker. | KeyDropFrom through the deep explicit-Key Part path returns the empty part and stores the rebuilt owner with unchanged siblings on 2 MiB. `crates/rum-stdlib/src/tests/held_association_exact_path_depth.rs:540`, `crates/rum-stdlib/src/tests/held_association_exact_path_depth.rs:559`, `crates/rum-stdlib/src/tests/held_association_exact_path_depth.rs:560`, `crates/rum-stdlib/src/tests/held_association_exact_path_depth.rs:129`, `crates/rum-stdlib/src/tests/held_association_exact_path_depth.rs:135`, `crates/rum-stdlib/src/tests/held_association_exact_path_depth.rs:140`, `crates/rum-stdlib/src/tests/held_association_exact_path_depth.rs:145`, `crates/rum-stdlib/src/tests/held_association_exact_path_depth.rs:152` | director green `.temp/director/audit-638/ignored-release.log:289`; M26 |
| I28 / **certificate** | `tests::iterative_traversal_depth::matchq_completes_at_physical_height_20001_on_two_mib`; fn `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:177`; ignore `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:176` | `release-only deployment-stack probe; run explicitly with --release` | release, explicit ignored selection; no special feature/env; declared 2 MiB worker. | MatchQ of the height-20001 candidate against the root IntegerQ predicate completes as False on 2 MiB. `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:179`, `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:181` | director green `.temp/director/audit-638/ignored-release.log:299`; M28 |
| I29 / **certificate** | `tests::iterative_traversal_depth::construction_reaches_physical_height_20001_on_two_mib`; fn `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:186`; ignore `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:185` | `release-only deployment-stack probe; run explicitly with --release` | release, explicit ignored selection; no special feature/env; declared 2 MiB worker. | The generated candidate is reported by Depth as 20001 on 2 MiB. `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:188`, `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:190` | director green `.temp/director/audit-638/ignored-release.log:294`; M29 |
| I30 / **certificate** | `tests::iterative_traversal_depth::tostring_completes_at_physical_height_4097_on_two_mib`; fn `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:195`; ignore `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:194` | `release-only deployment-stack probe; run explicitly with --release` | release, explicit ignored selection; no special feature/env; declared 2 MiB worker. | ToString of the deep singleton-list candidate has length 8193 on 2 MiB. `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:197`, `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:199` | director green `.temp/director/audit-638/ignored-release.log:309`; M30 |
| I31 / **certificate** | `tests::iterative_traversal_depth::take_multispec_completes_at_physical_height_4097_on_two_mib`; fn `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:204`; ignore `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:203` | `release-only deployment-stack probe; run explicitly with --release` | release, explicit ignored selection; no special feature/env; declared 2 MiB worker. | Multi-spec Take on the height-4097 singleton-list chain returns the same full nested shape on 2 MiB. `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:208`, `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:211` | director green `.temp/director/audit-638/ignored-release.log:308`; M31 |
| I32 / **certificate** | `tests::iterative_traversal_depth::replacepart_deep_path_completes_at_physical_height_4097_on_two_mib`; fn `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:219`; ignore `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:218` | `release-only deployment-stack probe; run explicitly with --release` | release, explicit ignored selection; no special feature/env; declared 2 MiB worker. | ReplacePart changes the deepest singleton-list leaf from 1 to 9 and preserves every wrapper on 2 MiB. `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:222`, `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:225` | director green `.temp/director/audit-638/ignored-release.log:306`; M32 |
| I33 / **certificate** | `tests::iterative_traversal_depth::map_completes_at_physical_height_4097_on_two_mib`; fn `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:233`; ignore `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:232` | `release-only deployment-stack probe; run explicitly with --release` | release, explicit ignored selection; no special feature/env; declared 2 MiB worker. | Map at leaf level wraps the deepest 1 in deepMapProbe while preserving every list wrapper on 2 MiB. `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:235`, `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:237` | director green `.temp/director/audit-638/ignored-release.log:298`; M33 |
| I34 / **certificate** | `tests::iterative_traversal_depth::association_map_completes_at_physical_height_4097_on_two_mib`; fn `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:245`; ignore `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:244` | `release-only deployment-stack probe; run explicitly with --release` | release, explicit ignored selection; no special feature/env; declared 2 MiB worker. | Map wraps the checked deep association spine at every selected level with ff and preserves its exact rendered structure on 2 MiB. `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:252`, `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:260`, `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:88`, `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:89` | director green `.temp/director/audit-638/ignored-release.log:290`; M34 |
| I35 / **certificate** | `tests::iterative_traversal_depth::association_assignment_read_completes_at_physical_height_4097_on_two_mib`; fn `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:268`; ignore `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:267` | `release-only deployment-stack probe; run explicitly with --release` | release, explicit ignored selection; no special feature/env; declared 2 MiB worker. | Deep association Part assignment stores 8 at the unique target and the later read returns 8 on 2 MiB. `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:281`, `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:286`, `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:114`, `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:115`, `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:298` | director green `.temp/director/audit-638/ignored-release.log:291`; M35 |
| I36 / **certificate** | `tests::iterative_traversal_depth::merge_nested_completes_at_physical_height_4097_on_two_mib`; fn `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:306`; ignore `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:305` | `release-only deployment-stack probe; run explicitly with --release` | release, explicit ignored selection; no special feature/env; declared 2 MiB worker. | MergeNested combines the unique deep values 17 and 25 into 42 while preserving the exact root and inner siblings on 2 MiB. `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:340`, `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:341`, `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:125`, `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:126`, `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:130`, `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:136`, `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:141`, `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:144`, `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:344` | director green `.temp/director/audit-638/ignored-release.log:301`; M36 |
| I37 / **certificate** | `tests::iterative_traversal_depth::apply_completes_at_physical_height_4097_on_two_mib`; fn `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:349`; ignore `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:348` | `release-only deployment-stack probe; run explicitly with --release` | release, explicit ignored selection; no special feature/env; declared 2 MiB worker. | Apply at level -2 replaces the deepest list head with deepApplyProbe and preserves outer wrappers on 2 MiB. `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:351`, `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:353` | director green `.temp/director/audit-638/ignored-release.log:287`; M37 |
| I38 / **certificate** | `tests::iterative_traversal_depth::replaceall_completes_at_physical_height_4097_on_two_mib`; fn `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:361`; ignore `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:360` | `release-only deployment-stack probe; run explicitly with --release` | release, explicit ignored selection; no special feature/env; declared 2 MiB worker. | ReplaceAll changes the deepest 1 to 2 and preserves the entire nested-list shape on 2 MiB. `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:363`, `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:365` | director green `.temp/director/audit-638/ignored-release.log:319`; M38 |
| I39 / **certificate** | `tests::iterative_traversal_depth::replace_completes_at_physical_height_4097_on_two_mib`; fn `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:373`; ignore `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:372` | `release-only deployment-stack probe; run explicitly with --release` | release, explicit ignored selection; no special feature/env; declared 2 MiB worker. | Replace over levels 0 through Infinity changes the deepest 1 to 2 with the same outer shape on 2 MiB. `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:375`, `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:377` | director green `.temp/director/audit-638/ignored-release.log:321`; M39 |
| I40 / **certificate** | `tests::iterative_traversal_depth::with_substitution_completes_at_physical_height_4097_on_two_mib`; fn `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:385`; ignore `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:384` | `release-only deployment-stack probe; run explicitly with --release` | release, explicit ignored selection; no special feature/env; declared 2 MiB worker. | With substitutes 7 for the bound symbol in the deep body and preserves its entire nested shape on 2 MiB. `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:403` | director green `.temp/director/audit-638/ignored-release.log:310`; M40 |
| I41 / **certificate** | `tests::iterative_traversal_depth::rule_rhs_substitution_completes_at_physical_height_4097_on_two_mib`; fn `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:411`; ignore `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:410` | `release-only deployment-stack probe; run explicitly with --release` | release, explicit ignored selection; no special feature/env; declared 2 MiB worker. | A delayed rule substitutes its matched 7 into the deep RHS template and preserves its entire nested shape on 2 MiB. `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:430` | director green `.temp/director/audit-638/ignored-release.log:307`; M41 |
| I42 / **certificate** | `tests::iterative_traversal_depth::replace_active_spine_returns_in_band_resource_error_on_two_mib`; fn `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:438`; ignore `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:437` | `release-only active-spine safety probe; run explicitly with --release` | release, explicit ignored selection; no special feature/env; declared 2 MiB worker. | Active-head Replace returns the StackLimit termination with General::reslim and leaves the session usable on 2 MiB. `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:449`, `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:455`, `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:465` | director green `.temp/director/audit-638/ignored-release.log:320`; M42 |
| I43 / **certificate** | `tests::iterative_traversal_depth::deletecases_completes_at_physical_height_4097_on_two_mib`; fn `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:470`; ignore `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:469` | `release-only deployment-stack probe; run explicitly with --release` | release, explicit ignored selection; no special feature/env; declared 2 MiB worker. | DeleteCases removes the unique deepest 1 while retaining all outer list wrappers on 2 MiB. `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:472`, `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:474` | director green `.temp/director/audit-638/ignored-release.log:303`; M43 |
| I44 / **certificate** | `tests::iterative_traversal_depth::deletecases_active_spine_returns_in_band_resource_error_on_two_mib`; fn `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:482`; ignore `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:481` | `release-only active-spine safety probe; run explicitly with --release` | release, explicit ignored selection; no special feature/env; declared 2 MiB worker. | Active-head DeleteCases returns the StackLimit termination with General::reslim and leaves the session usable on 2 MiB. `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:493`, `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:499`, `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:509` | director green `.temp/director/audit-638/ignored-release.log:300`; M42 |
| I45 / **certificate** | `tests::iterative_traversal_depth::d_nested_sin_completes_at_physical_height_4097_on_two_mib`; fn `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:514`; ignore `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:513` | `release-only active-spine safety probe; run explicitly with --release` | release, explicit ignored selection; no special feature/env; declared 2 MiB worker. | D of the checked 4096-deep Sin chain completes with result Length 4096, no messages and a usable session on 2 MiB. `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:523`, `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:535`, `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:536`, `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:537`, `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:547` | director green `.temp/director/audit-638/ignored-release.log:324`; M45 |
| I46 / **certificate** | `tests::iterative_traversal_depth::cases_completes_at_physical_height_4097_on_two_mib`; fn `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:660`; ignore `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:659` | `release-only deployment-stack probe; run explicitly with --release` | release, explicit ignored selection; no special feature/env; declared 2 MiB worker. | Cases over the deep candidate for absent z completes with result Length 0 on 2 MiB. `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:662`, `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:664` | director green `.temp/director/audit-638/ignored-release.log:318`; M46 |
| I47 / **certificate** | `tests::iterative_traversal_depth::level_completes_at_physical_height_4097_on_two_mib`; fn `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:669`; ignore `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:668` | `release-only deployment-stack probe; run explicitly with --release` | release, explicit ignored selection; no special feature/env; declared 2 MiB worker. | Level at Infinity returns 4096 parts from the deep candidate on 2 MiB. `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:671`, `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:673` | director green `.temp/director/audit-638/ignored-release.log:297`; M47 |
| I48 / **certificate** | `tests::iterative_traversal_depth::memberq_completes_at_physical_height_4097_on_two_mib`; fn `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:678`; ignore `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:677` | `release-only deployment-stack probe; run explicitly with --release` | release, explicit ignored selection; no special feature/env; declared 2 MiB worker. | MemberQ over the deep candidate for absent z completes as False on 2 MiB. `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:680`, `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:682` | director green `.temp/director/audit-638/ignored-release.log:305`; M48 |
| I49 / **certificate** | `tests::iterative_traversal_depth::count_completes_at_physical_height_4097_on_two_mib`; fn `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:687`; ignore `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:686` | `release-only deployment-stack probe; run explicitly with --release` | release, explicit ignored selection; no special feature/env; declared 2 MiB worker. | Count over the deep candidate for absent z completes as 0 on 2 MiB. `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:689`, `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:691` | director green `.temp/director/audit-638/ignored-release.log:302`; M48 |
| I50 / **certificate** | `tests::iterative_traversal_depth::position_completes_at_physical_height_4097_on_two_mib`; fn `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:696`; ignore `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:695` | `release-only deployment-stack probe; run explicitly with --release` | release, explicit ignored selection; no special feature/env; declared 2 MiB worker. | Position over the deep candidate for absent z completes with result Length 0 on 2 MiB. `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:698`, `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:700` | director green `.temp/director/audit-638/ignored-release.log:312`; M50 |
| I51 / **certificate** | `tests::iterative_traversal_depth::freeq_completes_at_physical_height_4097_on_two_mib`; fn `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:705`; ignore `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:704` | `release-only deployment-stack probe; run explicitly with --release` | release, explicit ignored selection; no special feature/env; declared 2 MiB worker. | FreeQ over the deep candidate for absent z completes as True on 2 MiB. `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:707`, `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:708` | director green `.temp/director/audit-638/ignored-release.log:304`; M48 |
| I52 / **certificate** | `tests::map_indexed_association_depth::map_indexed_association_completes_at_physical_height_4097_on_two_mib`; fn `crates/rum-stdlib/src/tests/map_indexed_association_depth.rs:181`; ignore `crates/rum-stdlib/src/tests/map_indexed_association_depth.rs:180` | `release-only deployment-stack probe; run explicitly with --release` | release, explicit ignored selection; no special feature/env; declared 2 MiB worker. | MapIndexed preserves the checked association chain and wraps its leaf with the exact 2048-component position on 2 MiB without messages. `crates/rum-stdlib/src/tests/map_indexed_association_depth.rs:187`, `crates/rum-stdlib/src/tests/map_indexed_association_depth.rs:122`, `crates/rum-stdlib/src/tests/map_indexed_association_depth.rs:198`, `crates/rum-stdlib/src/tests/map_indexed_association_depth.rs:106`, `crates/rum-stdlib/src/tests/map_indexed_association_depth.rs:112`, `crates/rum-stdlib/src/tests/map_indexed_association_depth.rs:204` | director green `.temp/director/audit-638/ignored-release.log:311`; M52 |
| I53 / **certificate** | `tests::map_indexed_association_depth::map_indexed_association_heads_refuses_in_band_rather_than_terminating`; fn `crates/rum-stdlib/src/tests/map_indexed_association_depth.rs:210`; ignore `crates/rum-stdlib/src/tests/map_indexed_association_depth.rs:209` | `release-only deployment-stack probe; run explicitly with --release` | release, explicit ignored selection; no special feature/env; declared 2 MiB worker. | Evaluating the checked deep head-bearing MapIndexed request returns Err(StackLimitExceeded) on 2 MiB. `crates/rum-stdlib/src/tests/map_indexed_association_depth.rs:211`, `crates/rum-stdlib/src/tests/map_indexed_association_depth.rs:237`, `crates/rum-stdlib/src/tests/map_indexed_association_depth.rs:239`, `crates/rum-stdlib/src/tests/map_indexed_association_depth.rs:245` | director green `.temp/director/audit-638/ignored-release.log:313`; M53 |
| I54 / **contract** | `tests::merge_native_stack_guard::merge_zero_native_stack_boundary_fails_in_band_and_session_recovers`; fn `crates/rum-stdlib/src/tests/merge_native_stack_guard.rs:65`; ignore `crates/rum-stdlib/src/tests/merge_native_stack_guard.rs:64` | `requires rum-runtime/test-native-stack-guard-zero` | debug or release; --features rum-runtime/test-native-stack-guard-zero; explicit ignored selection; forbidden in this audit. | With the zero-stack-guard feature, Merge returns StackLimit with the exact zero-byte diagnostic and the session recovers. `crates/rum-stdlib/src/tests/merge_native_stack_guard.rs:71`, `crates/rum-stdlib/src/tests/merge_native_stack_guard.rs:75`, `crates/rum-stdlib/src/tests/merge_native_stack_guard.rs:78`, `crates/rum-stdlib/src/tests/merge_native_stack_guard.rs:84` | excluded from director calibration; not run here; M54, M54d |
| I55 / **generator** | `tests::retention_closure::local::contract_tests::regenerate_local_contract_fingerprints`; fn `crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs:8`; ignore `crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs:7` | `writes a review copy; does not discharge contracts or install marks` | no required profile/extra feature/env in the test; explicit ignored selection; director used release. | The generator reads the local contracts and writes a review copy of their fingerprints without installing or discharging marks. `crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs:10`, `crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs:25`, `crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs:28` | director green `.temp/director/audit-638/ignored-release.log:316`; mutant not required for this class |
| I56 / **contract** | `tests::scan_and_keypath_set_native_stack_guard::scan_zero_native_stack_boundary_covers_each_edge_without_callbacks`; fn `crates/rum-stdlib/src/tests/scan_and_keypath_set_native_stack_guard.rs:149`; ignore `crates/rum-stdlib/src/tests/scan_and_keypath_set_native_stack_guard.rs:148` | `requires rum-runtime/test-native-stack-guard-zero` | debug or release; --features rum-runtime/test-native-stack-guard-zero; explicit ignored selection; forbidden in this audit. | With the zero guard, Scan refuses association, head and argument descents before callbacks and leaves the session usable. `crates/rum-stdlib/src/tests/scan_and_keypath_set_native_stack_guard.rs:150`, `crates/rum-stdlib/src/tests/scan_and_keypath_set_native_stack_guard.rs:176`, `crates/rum-stdlib/src/tests/scan_and_keypath_set_native_stack_guard.rs:26`, `crates/rum-stdlib/src/tests/scan_and_keypath_set_native_stack_guard.rs:30`, `crates/rum-stdlib/src/tests/scan_and_keypath_set_native_stack_guard.rs:177`, `crates/rum-stdlib/src/tests/scan_and_keypath_set_native_stack_guard.rs:185` | excluded from director calibration; not run here; M54, M54d |
| I57 / **contract** | `tests::scan_and_keypath_set_native_stack_guard::keypath_set_zero_native_stack_boundary_preserves_both_lvalue_roots`; fn `crates/rum-stdlib/src/tests/scan_and_keypath_set_native_stack_guard.rs:197`; ignore `crates/rum-stdlib/src/tests/scan_and_keypath_set_native_stack_guard.rs:196` | `requires rum-runtime/test-native-stack-guard-zero` | debug or release; --features rum-runtime/test-native-stack-guard-zero; explicit ignored selection; forbidden in this audit. | With the zero guard, bare-key and explicit-Key assignments refuse without kval, preserve the ownvalue and leave the session usable. `crates/rum-stdlib/src/tests/scan_and_keypath_set_native_stack_guard.rs:205`, `crates/rum-stdlib/src/tests/scan_and_keypath_set_native_stack_guard.rs:220`, `crates/rum-stdlib/src/tests/scan_and_keypath_set_native_stack_guard.rs:26`, `crates/rum-stdlib/src/tests/scan_and_keypath_set_native_stack_guard.rs:30`, `crates/rum-stdlib/src/tests/scan_and_keypath_set_native_stack_guard.rs:221`, `crates/rum-stdlib/src/tests/scan_and_keypath_set_native_stack_guard.rs:228`, `crates/rum-stdlib/src/tests/scan_and_keypath_set_native_stack_guard.rs:236` | excluded from director calibration; not run here; M54, M54d |
| I58 / **behaviour** | `tests::stress_corpus::rule_based_differentiator_handles_heavy_generated_polynomial`; fn `crates/rum-stdlib/src/tests/stress_corpus.rs:166`; ignore `crates/rum-stdlib/src/tests/stress_corpus.rs:165` | none (`#[ignore]`) | no required profile/extra feature/env in the test; explicit ignored selection; director used release. | The rule-based differentiator returns the exact expected ten-term heavy polynomial result on its declared deep stack. `crates/rum-stdlib/src/tests/stress_corpus.rs:175`, `crates/rum-stdlib/src/tests/stress_corpus.rs:178` | director green `.temp/director/audit-638/ignored-release.log:315`; mutant not required for this class |
| I59 / **behaviour** | `tests::stress_corpus::peano_arithmetic_heavy_multiplication`; fn `crates/rum-stdlib/src/tests/stress_corpus.rs:216`; ignore `crates/rum-stdlib/src/tests/stress_corpus.rs:215` | none (`#[ignore]`) | no required profile/extra feature/env in the test; explicit ignored selection; director used release. | The heavy Peano multiplication returns 1560 for 30 times 52 on its declared deep stack. `crates/rum-stdlib/src/tests/stress_corpus.rs:223`, `crates/rum-stdlib/src/tests/stress_corpus.rs:227` | director green `.temp/director/audit-638/ignored-release.log:314`; mutant not required for this class |
| I60 / **behaviour** | `tests::stress_corpus::tokenize_with_ini_grammar_handles_heavy_generated_config`; fn `crates/rum-stdlib/src/tests/stress_corpus.rs:371`; ignore `crates/rum-stdlib/src/tests/stress_corpus.rs:370` | none (`#[ignore]`) | no required profile/extra feature/env in the test; explicit ignored selection; director used release. | The generated heavy INI input yields the expected token count, token-class counts and first/last source spans. `crates/rum-stdlib/src/tests/stress_corpus.rs:372`, `crates/rum-stdlib/src/tests/stress_corpus.rs:380`, `crates/rum-stdlib/src/tests/stress_corpus.rs:319`, `crates/rum-stdlib/src/tests/stress_corpus.rs:322`, `crates/rum-stdlib/src/tests/stress_corpus.rs:327`, `crates/rum-stdlib/src/tests/stress_corpus.rs:330`, `crates/rum-stdlib/src/tests/stress_corpus.rs:348` | director green `.temp/director/audit-638/ignored-release.log:317`; mutant not required for this class |
| I61 / **falsifier** | `forced_red_probe`; fn `crates/rum-stdlib/tests/typed_slot_census.rs:231`; ignore `crates/rum-stdlib/tests/typed_slot_census.rs:230` | `invoked explicitly by both bounded scratch-copy falsifiers` | parent test binary profile/features; RUM_TYPED_SLOT_FORCED_RED_TREE must name a private copy (typed_slot_census.rs:37,236); forbidden in this audit. | The private scratch entry requires SCRATCH_ENV, discovers that copy, and requires the census baseline, intentionally failing for its parent-created mutation. `crates/rum-stdlib/tests/typed_slot_census.rs:232`, `crates/rum-stdlib/tests/typed_slot_census.rs:236`, `crates/rum-stdlib/tests/typed_slot_census.rs:240`, `crates/rum-stdlib/tests/typed_slot_census.rs:241` | excluded from director calibration; not run here; mutant not required for this class |

### Production mutant designs and observation paths

**Design candidates only: none has been edited, compiled or run. No predicted red below is an observed red.** `test:N` resolves to each listed inventory row's exact source file; `production:N` resolves to the production file in that mutant row. Equivalent classes share the stated assertion claim, not merely a similar test name. Distinct ancillary claims stay separate. I07 is explicitly open in D01; drop cases M08/M12 are conditional and receive no credit from an unexplained abort. These are scoped proposals, not claims that all mutation paths have been dynamically reached.

| Mutant / rows | Production site and one-line change | Changed value → assertion → why it is not swallowed | Limits / equivalence |
| --- | --- | --- | --- |
| M03 / I03 | `crates/rum-runtime/src/runtime/evaluation_cache/profile/report.rs:341` — Change the source/head row iterator cap from .take(32) to .take(0). | The child still evaluates to profileLeaf[1] and exits successfully (test:545,566), but no source/head row reaches stderr; the filtered row vector fails the exact row comparison (test:578,583). The parent compares captured raw stderr, so later guest evaluation cannot reconstruct a suppressed report row. | This falsifies report-row preservation, not the correctness of every internal profiling counter. Env precondition red at test:531 is not this mutant. |
| M06 / I06 | `crates/rum-runtime/src/runtime/lowering/binary/left_chain.rs:31` — Remove the args.push(rhs) in append_additive for an already accumulated Plus chain. | The first accepted additive fixture lowers with too few Plus arguments; test:327 calls the shape assertion at test:435 expecting 4097. No evaluator runs between lowering and that raw Expr arity assertion. Formatter output cannot repair the earlier raw check. | Separate rejection-boundary coverage is M06b; this mutation concerns accepted flatten/arity, not all parser diagnostics. |
| M06b / I06 | `crates/rum-syntax/src/parser/front.rs:872` — Change the left-chain limit comparison from *links >= MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS to *links > MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS. | The 4096-link accepted case is unchanged, but the first rejected fixture with 4097 links now parses. Its explicit should-be-rejected panic at test:344 fires after dropping the parsed syntax; the thread join propagates failure at test:382. No evaluator can turn successful parsing into the expected diagnostic. | Different claim from M06. Compile failure, drop failure before test:344, or a timeout would not count. |
| M08 / I08, I17 | `crates/rum-stdlib/src/associations/basic/lookup.rs:33` — Replace drop_lookup_outcomes_iterative(std::mem::take(items)) with drop(std::mem::take(items)). | Restores recursive destruction of the uniquely owned List chain before test:859 returns; I08 observes completion through joined.is_ok at test:863. I17 also drops its prepared deep outcome after resume (test:264,268), but only a causally located destructor failure would cover that aspect. | UNCALIBRATED STACK MUTANT / D02: it may survive at 4097 in release. A process abort that never reaches the join assertion is not an assertion red; a generic signal, timeout or unrelated panic receives no credit. Never weaken or enlarge the test inside this campaign. |
| M09 / I09, I11 | `crates/rum-stdlib/src/associations/key_reshape/traversal.rs:479` — In iterative flatten, set the emitted AssociationEntry delayed field to false instead of entry.delayed. | The deepest RuleDelayed becomes Rule. I09 compares the returned flattened Expr directly at test:269; I11 compares the exact flattened round-trip result with its literal original at test:306. Both call builtins directly, so no downstream session evaluation can restore delayedness. | One preservation claim and one mutation cover both tests; keep their separate one-way and round-trip fixtures. |
| M10 / I10, I11 | `crates/rum-stdlib/src/associations/key_reshape/traversal.rs:211` — In nested_path_node_for_missing_suffix, initialize the terminal Leaf with delayed: false instead of delayed. | The constructed deepest leaf is immediate. I10 descends the raw chain at test:284 and fails delayed equality at test:115; I11 flatten faithfully carries the wrong flag to the literal equality at test:306. Shape and key checks still precede this observation; the direct calls do not reevaluate or repair RuleDelayed. | Independent of M09: one mutates deflatten, the other flatten. The round-trip test is expected to catch either isolated mutation, never a compensating pair. |
| M12 / I12 | `crates/rum-stdlib/src/associations/key_reshape/traversal.rs:84` — On the empty-path error branch, replace drop_nested_path_nodes_iterative(nested) with drop(nested). | The valid 60000-key first entry leaves a partial tree; the following empty path enters this cleanup before the builtin can return its original inert call. The normal observer is equality at test:347 and thread join at test:17; recursive teardown can prevent those observers from returning. | UNCALIBRATED STACK MUTANT / D02: test comment at test:319 records earlier recursive-drop behaviour, not a run in this audit. Require causal evidence in an isolated child; a bare abort is inconclusive under the wrong-reason rule. |
| M13 / I13, I14 | `crates/rum-stdlib/src/associations/basic/key_take_drop.rs:255` — Call keyed_take_drop_terminal_result with !keep_selected in the iterative tail. | The leaf association retains the complementary entry set while outer lists rebuild normally. I13 and I14 reach exact leaf equality in test:113 through test:230 and test:254 respectively. These direct calls compare raw literal expected rules; no later evaluation reselects keys. | One keep-selected polarity claim, two opposite modes; retain both tests. |
| M15 / I15 | `crates/rum-stdlib/src/associations/basic/keys.rs:115` — Pass None as the wrapper to keys_terminal_result in the iterative tail. | Both terminal keys become bare strings instead of wrapped calls. The raw list descent reaches test:97 through test:224; expected order and wrapper are fixed. The direct builtin result is not reevaluated. | Separate production site; exact target test must be green in the same unmodified scratch baseline first. |
| M16 / I16 | `crates/rum-stdlib/src/associations/basic/lookup.rs:163` — Push value directly instead of wrap_keyed_result(value, wrapper) for an owned iterative Hit. | The deepest hit becomes bare 474 instead of the requested wrapper call. I16 test:216 reaches the structural leaf checks at test:102,113,117. The wrapper is undefined guest data, so evaluating the output cannot invent the missing head. | Separate production site; exact target test must be green in the same unmodified scratch baseline first. |
| M17a / I17 | `crates/rum-stdlib/src/associations/basic/lookup.rs:170` — For the owned iterative missing fallback, use Expr::string("mutant-missing") instead of missing_key_absent_expr(key). | The first no-default deep request returns a string leaf instead of Missing[KeyAbsent,...]. Test:237 reaches the fixed leaf/shape observer at test:102,113,117 before the resume case. No default exists on this first path to replace the wrong leaf. | Separate production site; exact target test must be green in the same unmodified scratch baseline first. |
| M17b / I17 | `crates/rum-stdlib/src/associations/basic/lookup.rs:259` — For the borrowed iterative Missing branch, replace default.cloned().unwrap_or_else(...) with missing_key_absent_expr(key.clone()). | The owned no-default case and input prompt remain intact; resume supplies the string at test:264 but the borrowed conversion discards it. Test:268 reaches the literal string/shape assertion at test:102,113,117 with a Missing expression. No later default application occurs after conversion. | Miss, resume and drop are different claims: M17a, M17b and conditional M08 are required separately. |
| M18 / I18 | `crates/rum-stdlib/src/associations/basic/values.rs:115` — Pass None as the wrapper to values_terminal_result in the iterative tail. | Terminal values lose the requested wrapper, failing the literal raw leaf comparison at test:97 via test:224; the direct result is never reevaluated. | Same defect shape as M15, different production owner; do not claim one mutation covers both. |
| M19 / I19, I20 | `crates/rum-stdlib/src/associations/missing/traversal/deletion.rs:236` — Change only the iterative association fast Missing-value exclusion condition to false. | The deep centre retains an immediate Missing entry: the later VisitKeyed path leaves that original entry when it has no rebuilt child. Both comparisons at test:127 fail via I19 test:195 and I20 test:219. Their expected centres use the shallow front at test:188 and test:208, outside this tail-only edit. Successful output is not run through DeleteMissing a second time. | One immediate-Missing exclusion claim, unarmed and memory-armed modes. Mutating the shared shallow predicate could corrupt both expected and actual values and be swallowed; do not substitute that mutant. |
| M21 / I21 | `crates/rum-stdlib/src/associations/missing/traversal/deletion.rs:376` — Return false instead of true when the iterative contains-Missing scan finds Missing. | The enclosing deep branch is retained alongside the kept sentinel, so test:236 receives two branches instead of the exact one-element expected list. Normal result evaluation does not rerun the DeleteMissing scan. | Separate production site; exact target test must be green in the same unmodified scratch baseline first. |
| M22 / I22 | `crates/rum-stdlib/src/associations/missing/target.rs:467` — Inside the iterative is_missing_expr branch, append entry to kept before its existing index increment and continue. | The target Missing rule survives and is not reconsidered by the later scan predicate because the branch still continues. Raw target descent at test:152 reaches exact leaf equality at test:126 with the extra rule. Witness/order checks are unchanged; direct invocation prevents repair by a second DeleteMissing. | Separate production site; exact target test must be green in the same unmodified scratch baseline first. |
| M23 / I23 | `crates/rum-stdlib/src/associations/missing/target.rs:474` — For an immediate entry at level 1 in the iterative target tail, replace !contains_missing_within(&entry.value, scan) with true. | The rule containing the unique nested Missing is kept, leaving an extra target entry. Test:169 reaches exact leaf equality at test:126. This value is only rendered through preserved associations and is not scanned again. | M22 covers direct Missing exclusion; M23 covers scan All inside a non-Missing value. They cannot collapse. |
| M24 / I24 | `crates/rum-stdlib/src/associations/missing/target.rs:167` — Replace the level-1 owned-list keep predicate, including both conjuncts, with true. | All 64000 items remain instead of the literal three survivors; test:196 compares the whole returned list. The wide case stays on this owned front, so mutating only the heap tail would be unreachable and receive no credit. | Separate production site; exact target test must be green in the same unmodified scratch baseline first. |
| M25 / I25 | `crates/rum-stdlib/src/associations/missing/target.rs:246` — In the owned-association Missing branch, push entry into cursor.kept before the existing increment and continue. | The wide association keeps its Missing-valued entries; test:240 fails its exact survivor equality. Continuing bypasses later exclusion, and no output evaluation repeats the deletion. | Different cursor owner from M24; same conceptual deletion requirement is not enough to collapse physical sites. |
| M26 / I26, I27 | `crates/rum-stdlib/src/associations/mutation.rs:662` — Omit the replace_exact_path assignment in the loop that rebuilds updated_target from replacements. | The replacement is computed but updated_target remains current and is stored at production:669. I26 returns that stale owner via production:672 and fails target equality test:152 through test:516 (also readback test:521). I27 returns selected_result via production:673, so its returned-empty-part check test:559 MAY PASS; the independent ownvalue readback test:560 reaches test:152 and fails on the old target. No later write repairs the stale stored owner. | One deep-owner reconstruction claim covers two APIs only through their distinct observers; do not credit KeyDropFrom from its return value alone. |
| M28 / I28 | `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:87` — Negate outcome.matched when converting the general-pattern match outcome to a boolean. | The PatternTest route produces True for the noninteger list root, so test:181 differs from False. The boolean is returned directly; ToString merely renders it. | Narrow predicate-outcome claim only. This root predicate does not require visiting the deepest leaf (D04). |
| M29 / I29 | `crates/rum-stdlib/src/structure_ops/mod.rs:117` — Change the atom structural-depth base case from 1 to 2. | The 20000 wrappers add to a base of 2 and Depth reports 20002 instead of 20001 at test:190; rendering preserves that integer. | This catches depth computation, not an independent proof that the construction reached its advertised raw shape (D04). |
| M30 / I30 | `crates/rum-runtime/src/runtime/kernel_data/text_rendering.rs:110` — Change the List closing delimiter passed to schedule_delimited from "}" to "". | The 4096 closing braces vanish, reducing the rendered string length from 8193 to 4097 at test:199. StringLength reads the returned string; it does not parse it or repair delimiters. | Length coverage only; equal-length wrong text would survive (D04). |
| M31 / I31 | `crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs:501` — Replace completed.push(Expr::call(head, args)) with completed.push(args.into_iter().next()?) in the iterative FinishCall arm. | Each singleton FinishCall loses its enclosing list; the bounded front cannot restore the missing tail wrappers, so the complete expected string at test:211 differs. The Option return type permits ?, and nonempty FinishCall work supplies the child. | This reconstruction corruption should be visible, but the no-op tail mutant at production:320 returning Some(expr.clone()) is predicted to SURVIVE this fixture (D03). A red M31 alone cannot certify selection occurred. |
| M32 / I32 | `crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/plan_application.rs:111` — In Immediate replacement, return expr instead of rhs (bind the unused rhs as _rhs). | The deepest original 1 is returned and the path frames rebuild around it; test:225 expects the literal 9. Ordinary evaluation/rendering of that list cannot apply a discarded ReplacePart plan again. | Separate production site; exact target test must be green in the same unmodified scratch baseline first. |
| M33 / I33 | `crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:115` — In the selected atom arm, return expr instead of Expr::call(operator.clone(), vec![expr]). | The deepest 1 remains bare while parent lists rebuild; test:237 expects deepMapProbe[1]. That symbol has no definition that could manufacture the wrapper during rendering. | Separate production site; exact target test must be green in the same unmodified scratch baseline first. |
| M34 / I34 | `crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:238` — When association children changed, use original_expr instead of render_association of the transformed values. | The association rebuild drops child ff wrappers while selected ancestor wrapping still runs; test:260 compares the entire fixed rendered shape and sees the missing descendants. Undefined ff does not recreate the lost children during evaluation. | Separate association rebuild claim from the atom callback in M33. |
| M35 / I35 | `crates/rum-runtime/src/runtime/kernel_data/association_data.rs:958` — Omit definition.own_value = Some(updated) in finish_association_set_part_lvalue, leaving the existing immediate ownvalue intact. | The assignment still returns rhs 8, so test:281 can pass. Reading b at test:286 observes the old deepest sentinel; test:115 requires the unique 8 and fails (later read equality test:298 also differs). No store follows the omitted commit. | Return-only mutations or assertions are inadequate for this persistence claim. |
| M36 / I36 | `crates/rum-stdlib/src/associations/resumable/aggregate/merge_nested.rs:166` — Use system_expr("First") instead of child_combiner for the iterative leaf-group evaluation. | The unique values [17,25] combine to 17 instead of 42. Persistent group completion propagates that value unchanged; test:341 reaches the explicit deepest numeric assertion at test:144. Literal sentinels and expected 42 are not computed by MergeNested. | The no-suspension deep leaf-combiner claim is covered; this ignored test is not a suspension lifecycle test. |
| M37 / I37 | `crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:194` — Use system_expr("List") instead of operator.clone() when replacing a selected call head. | The deepest head remains List, so test:353 receives braces where its literal deepApplyProbe head is required. No guest definition or later Apply redoes the discarded head replacement. | Separate production site; exact target test must be green in the same unmodified scratch baseline first. |
| M38 / I38 | `crates/rum-runtime/src/runtime/definitions/application/replace_all.rs:245` — Force the changed branch false when choosing rebuilt arguments versus original_args. | The leaf rewrite computes 2 but the frame chooses the old arguments containing 1. Test:365 compares the entire expected string containing 2. The final evaluation in runtime/kernel_data/rewrite_data/replace/all.rs:81 evaluates an ordinary List; it does not apply the removed rule again. | Specifically exercises loss during rebuild, not only rule matching. |
| M39 / I39 | `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once/traversal.rs:132` — In the Some(replaced) outcome, store expr instead of replaced in the returned traversal result. | The successful match still reports change but carries old leaf 1 through rebuilding; test:377 expects 2. The completed Replace result is evaluated without rerunning its original rule. | Separate production site; exact target test must be green in the same unmodified scratch baseline first. |
| M40 / I40 | `crates/rum-runtime/src/runtime/substitution/lexical_substitution/iterative.rs:504` — In the Symbol arm, push None instead of its substitution lookup result. | The deep lexical q remains q; the outer body still rebuilds, then test:403 expects literal 7. q has no independent ownvalue, and leaving With does not perform a second lexical substitution. | Separate production site; exact target test must be green in the same unmodified scratch baseline first. |
| M41 / I41 | `crates/rum-runtime/src/runtime/substitution/binding_substitution/iterative.rs:317` — On a named binding hit, push None instead of Some(value.clone()). | The deep rule RHS retains the pattern symbol q rather than its matched 7; test:430 compares the full literal output. There is no global q ownvalue that could swallow the lost local binding. | Separate production site; exact target test must be green in the same unmodified scratch baseline first. |
| M42 / I42, I44 | `crates/rum-runtime/src/runtime/session/session_helpers.rs:430` — Map EvalError::StackLimitExceeded to EvalTerminationReason::RecursionLimit instead of StackLimit. | The General::reslim message and later 1+1 recovery still pass, but final values become TerminatedEvaluation["RecursionLimit"] at I42 test:465 and I44 test:509. These compare the final public result string, with no later termination remapping. | One public termination-kind propagation claim; this mutation does not establish which native-stack guard fired. |
| M43 / I43 | `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete/traversal.rs:35` — Replace the outcome.matched deletion condition with false. | The unique deepest 1 is retained. Parent lists rebuild and the exact rendered empty-leaf expectation at test:474 fails. Normal evaluation cannot delete a bare 1 without DeleteCases running again. | Separate production site; exact target test must be green in the same unmodified scratch baseline first. |
| M45 / I45 | `crates/rum-stdlib/src/calculus/differentiation/iterative.rs:120` — Push derivative instead of combine_chain_rule_term(session, partial, derivative)? for the scheduled chain-rule term. | The iterative Sin tail discards its Cos factors and becomes 1; the bounded front still contributes its own factors (crates/rum-stdlib/src/calculus/differentiation.rs:13,43,173), leaving a short product rather than Length 4096 at test:535 (also test:547). Fixture Depth at test:523 is unaffected; later algebra cannot restore the missing tail factors. | The test only observes Length, so wrong factors preserving arity remain a separate D04 obligation. |
| M46 / I46 | `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect.rs:407` — Replace outcome.matched.then_some(expr) with (!outcome.matched).then_some(expr). | Nonmatching visited nodes become collected values; Length is positive instead of 0 at test:664. List construction and Length preserve that nonempty result. | An empty-result shortcut could still survive the original fixture; a red predicate mutant does not prove complete traversal (D04). |
| M47 / I47 | `crates/rum-runtime/src/runtime/tree_walkers/level.rs:36` — Omit parts.push(node.expr) in the iterative Level fallback. | The deep traversal takes the heap fallback but returns no selected nodes; test:673 sees 0 instead of 4096. Length reads the materialized returned list and cannot synthesize omitted nodes. | Count proves emission quantity, not the identity/order of every part (D04). |
| M48 / I48, I49, I51 | `crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:329` — Accumulate usize::from(!outcome.matched) instead of usize::from(outcome.matched) in the iterative worker. | Absent z produces a positive count; the unchanged finish mapping at production:31 yields MemberQ True, Count positive and FreeQ False. I48 test:682, I49 test:691 and I51 test:708 compare the opposite literals. No caller re-matches the result. | One absent-pattern count-to-boolean claim across three API projections. Do not collapse the Cases or Position owners into this mutant; a no-traversal shortcut still needs D04. |
| M50 / I50 | `crates/rum-runtime/src/runtime/tree_walkers/position.rs:211` — Negate outcome.matched in the position-recording condition. | Positions for nonmatching visited nodes make the output nonempty, so its Length fails the literal 0 at test:700. No later step filters the returned paths against z. | An always-empty position result can survive the existing absent-pattern fixture (D04). |
| M52 / I52 | `crates/rum-stdlib/src/functional_ops/apply_map/traversal/map_indexed.rs:218` — Pass &[] instead of &path to position_expr_from_path at the selected-node wrap. | The leaf is still wrapped, but its position argument is an empty list instead of 2048 ones. Raw descendant checks at test:198 reach the literal terminal comparison at test:204; guest evaluation does not reconstruct discarded positions. | Separate production site; exact target test must be green in the same unmodified scratch baseline first. |
| M53 / I53 | `crates/rum-runtime/src/runtime/eval_core/mod.rs:105` — Return EvalError::RecursionLimitExceeded with resource_limits.recursion_limit on the native-stack-exhaustion branch instead of StackLimitExceeded. | The same eager evaluation detects exhaustion but carries the wrong error variant to session.evaluate at test:239. The direct matches! assertion at test:245 rejects it without public termination-string conversion. | I53 test comment:211 attributes refusal to eager evaluation of a head-bearing shape, independently of the walker conversion. This calibrates in-band error type only, not head traversal or exact walker reachability (D05). A compile error or earlier setup panic would not count. |
| M54 / I54, I56, I57 | `crates/rum-runtime/src/runtime/native_stack_guard.rs:141` — Replace native_stack_exhausted(current_addr, self.limit) with false inside NativeStackGuard::poll_native_stack_limit. | On these modest 12-deep fixtures the zero-feature guard no longer records exhaustion: Merge returns data (I54 test:71); Scan collection proceeds through apply_map/mod.rs:129 to callbacks at :132, failing I56 test:176 -> test:26 (and zero-callback test:177); keypath assignment completes instead of refusing, failing I57 test:220 -> test:26 (and preserved-ownvalue test:228). Each request is far below the separate ordinary evaluator byte budget; success is not remapped into StackLimit. | One shared refusal-before-effects mechanism, separate family/edge fixtures. FORBIDDEN TO RUN under brief 639; a later director brief must explicitly authorize the zero-feature build. M54d isolates diagnostic propagation if caller/edge reachability remains uncertain. |
| M54d / I54, I56, I57 | `crates/rum-runtime/src/runtime/native_stack_guard.rs:186` — Report limit: guard.limit.saturating_add(1) instead of guard.limit in StackLimitExceeded. | The zero-budget refusal and cleanup are unchanged, but the error text says 1 byte rather than 0. I54 exact diagnostic predicate test:75,78 and the shared I56/I57 helper test:30 fail. The request error mapper uses the supplied limit and does not replace it with the feature configuration. | One diagnostic-limit propagation claim, not proof of all guarded traversal edges. Forbidden feature tests remain unrun. |

Equivalence decisions: M09/I09+I11 = flatten delayedness; M10/I10+I11 = deflatten delayedness; M13/I13+I14 = key-selection polarity; M19/I19+I20 = iterative immediate-Missing exclusion; M26/I26+I27 = stored owner reconstruction; M42/I42+I44 = public termination-kind propagation; M48/I48+I49+I51 = absent-pattern count projection; M54/I54+I56+I57 = shared zero-guard refusal; M54d/the same rows = diagnostic limit propagation. M08 can cover I08/I17 only if the very same destructor failure is demonstrated in both. M17a/M17b, M22/M23, M24/M25, M33/M34 and M15/M18 are not one mutant class. No inventory-class test is in this ignored census; the generator I55 is not an inventory acceptance test.

### All nine normalized duplicate-body groups

Rederived from all source-level #[test] bodies, including macro templates; comments/literals are masked only to balance braces, while comparison uses the original body with whitespace normalized. Both removal of whitespace and collapse of whitespace runs yield the same groups. Attributes and function names are deliberately not part of a body comparison. Compiler macro expansions are not separately enumerated; all source attribute sites resolved. Evidence: `.temp/brief-639/survey-complete.log:1`, `.temp/brief-639/survey-complete.log:2`, `.temp/brief-639/survey-complete.log:3`.

| Group | Every member | Judgment / binding evidence | Shared form / action |
| --- | --- | --- | --- |
| DUP01 | `rule_lhs_missing_root_falsifier` `crates/rum-runtime/src/runtime/definitions/rule_lhs/tests.rs:249`<br>`matcher_pattern_missing_root_falsifier` `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution/tests.rs:303` | **deliberate mirror**. Bodies both clear resolved_head in a local root_fixture then call local assert_roots. The fixtures have different state types and the expected-panic messages differ: `crates/rum-runtime/src/runtime/definitions/rule_lhs/tests.rs:199`, `crates/rum-runtime/src/runtime/definitions/rule_lhs/tests.rs:217`, `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution/tests.rs:251`, `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution/tests.rs:263`. | Keep both concrete tests and local observers. A generic shared fixture would hide the state-owner boundary; no collapse. Their identical bodies do not establish a common production trace implementation; see D08. |
| DUP02 | `cross_file_recursion_falsifier` `crates/rum-stdlib/src/tests/association_depth_contract/destructuring_contract.rs:53`<br>`cross_file_recursion_falsifier` `crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs:88` | **deliberate mirror**. The synthetic a.rs/b.rs cycle is identical, but each local assert_acyclic checks a separately owned source set and names a different combined-contract failure: `crates/rum-stdlib/src/tests/association_depth_contract/destructuring_contract.rs:16` and `crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs:11`. | Retain separate tests and expected-panic messages. At most share a constant synthetic two-file fixture if a future non-behaviour cleanup demonstrates value; do not merge the two source inventories or observers. |
| DUP03 | `cancel_direct` `crates/rum-stdlib/tests/default_fallback_continuations/lifecycle.rs:115`<br>`cancel_at_first_target` `crates/rum-stdlib/tests/destructuring_continuations/lifecycle.rs:78`<br>`cancel_first` `crates/rum-stdlib/tests/option_reader_continuations/lifecycle.rs:48` | **deliberate mirror**. Identical cancel(0, false) wrappers resolve to three distinct local cancellation helpers. Default/fallback helper `crates/rum-stdlib/tests/default_fallback_continuations/lifecycle.rs:29` asserts logs/prints/rejection; destructuring helper `crates/rum-stdlib/tests/destructuring_continuations/lifecycle.rs:4` also asserts partial writes at :53; option-reader helper `crates/rum-stdlib/tests/option_reader_continuations/lifecycle.rs:3` asserts its distinct CountHeads fixture at :35–39. These are behaviour tests. | Keep the existing local helper form and three named family tests. |
| DUP04 | `cancel_fallback` `crates/rum-stdlib/tests/default_fallback_continuations/lifecycle.rs:119`<br>`cancel_at_second_target` `crates/rum-stdlib/tests/destructuring_continuations/lifecycle.rs:82`<br>`cancel_second` `crates/rum-stdlib/tests/option_reader_continuations/lifecycle.rs:52` | **deliberate mirror**. Identical cancel(1, false) wrappers resolve to three distinct local cancellation helpers. Default/fallback helper `crates/rum-stdlib/tests/default_fallback_continuations/lifecycle.rs:29` asserts logs/prints/rejection; destructuring helper `crates/rum-stdlib/tests/destructuring_continuations/lifecycle.rs:4` also asserts partial writes at :53; option-reader helper `crates/rum-stdlib/tests/option_reader_continuations/lifecycle.rs:3` asserts its distinct CountHeads fixture at :35–39. These are behaviour tests. | Keep the existing local helper form and three named family tests. |
| DUP05 | `abandon_direct_without_answer` `crates/rum-stdlib/tests/default_fallback_continuations/lifecycle.rs:123`<br>`abandon_first_target_without_answer` `crates/rum-stdlib/tests/destructuring_continuations/lifecycle.rs:86`<br>`abandon_first_without_answer` `crates/rum-stdlib/tests/option_reader_continuations/lifecycle.rs:56` | **deliberate mirror**. Identical cancel(0, true) wrappers resolve to three distinct local abandonment helpers. Default/fallback helper `crates/rum-stdlib/tests/default_fallback_continuations/lifecycle.rs:29` asserts logs/prints/rejection; destructuring helper `crates/rum-stdlib/tests/destructuring_continuations/lifecycle.rs:4` also asserts partial writes at :53; option-reader helper `crates/rum-stdlib/tests/option_reader_continuations/lifecycle.rs:3` asserts its distinct CountHeads fixture at :35–39. These are behaviour tests. | Keep the existing local helper form and three named family tests. |
| DUP06 | `abandon_fallback_without_answer` `crates/rum-stdlib/tests/default_fallback_continuations/lifecycle.rs:127`<br>`abandon_second_target_without_answer` `crates/rum-stdlib/tests/destructuring_continuations/lifecycle.rs:90`<br>`abandon_second_without_answer` `crates/rum-stdlib/tests/option_reader_continuations/lifecycle.rs:60` | **deliberate mirror**. Identical cancel(1, true) wrappers resolve to three distinct local abandonment helpers. Default/fallback helper `crates/rum-stdlib/tests/default_fallback_continuations/lifecycle.rs:29` asserts logs/prints/rejection; destructuring helper `crates/rum-stdlib/tests/destructuring_continuations/lifecycle.rs:4` also asserts partial writes at :53; option-reader helper `crates/rum-stdlib/tests/option_reader_continuations/lifecycle.rs:3` asserts its distinct CountHeads fixture at :35–39. These are behaviour tests. | Keep the existing local helper form and three named family tests. |
| DUP07 | `arrayfilter_unread_default` `crates/rum-stdlib/tests/option_reader_continuations/boolean.rs:284`<br>`oracle_arrayfilter_unread_default` `crates/rum-stdlib/tests/option_reader_continuations/rule.rs:44` | **copy**. The same ArrayFilter unread-default source and empty input use the same super::support::compare observer in boolean.rs and rule.rs. | Candidate shared form: one named arrayfilter_unread_default case owned by rule.rs, or one shared fixture function with unchanged compare call. Behaviour protection means no current edit or test deletion. |
| DUP08 | `testreport_unread_default` `crates/rum-stdlib/tests/option_reader_continuations/boolean.rs:294`<br>`oracle_testreport_unread_default` `crates/rum-stdlib/tests/option_reader_continuations/phases.rs:294` | **copy**. The same TestReport unread-default source and empty input use the same super::support::compare observer in boolean.rs and phases.rs. | Candidate shared form: one named testreport_unread_default case owned by phases.rs, or one shared fixture function. Behaviour protection means no current edit or test deletion. |
| DUP09 | `names_early_invalid_unread_later` `crates/rum-stdlib/tests/option_reader_continuations/boolean.rs:304`<br>`oracle_names_early_invalid_unread_later` `crates/rum-stdlib/tests/option_reader_continuations/phases.rs:304` | **copy**. The same Names early-invalid/unread-later source and empty input use the same super::support::compare observer in boolean.rs and phases.rs. | Candidate shared form: one named names_early_invalid_unread_later case owned by phases.rs, or one shared fixture function. Behaviour protection means no current edit or test deletion. |

### Judgment of the two existing falsifier patterns

The existing typed-slot examples are useful for mutation isolation and exact diagnostic matching, but two distinct mechanisms must not be conflated. `forced_red(remove_guard)` copies discovered files to PrivateScratch, erases the selected frame/guard span, launches its prebuilt test executable through `run-bounded.py`, sets SCRATCH_ENV and selects `forced_red_probe` (`crates/rum-stdlib/tests/typed_slot_census.rs:141`, `:145`, `:188`, `:196`, `:200`, `:201`). Its parent demands both a failing status and the expected census markers (`:216`, `:221`). The ignored entry checks its private path and unwraps baseline discovery (`:232`, `:236`, `:240`, `:241`). This is a useful wrong-reason rejection pattern; it does not compile the mutated source.

`forced_red_witness(kind)` instead copies into a deterministic per-kind directory and calls the source scanner and `s5::transitions` directly (`crates/rum-stdlib/tests/typed_slot_census.rs:259`, `:263`, `:290`, `:291`); it does **not** route through SCRATCH_ENV/forced_red_probe. It changes the binding/insertion at `:273`, `:278`, `:284` and requires exactly `M0245: witnessed typed insertion absent` at `:292`. Its exact observation is valuable, but the deterministic directory is not the per-run private-copy layout requested here, and parsing success does not establish Rust compilation. Neither family was run. Stage B must add its own compiling cargo invocation per isolated production mutant, not call these excluded drivers or edit their assertions.

The two root-trace falsifiers mutate test fixtures, not production: rule-LHS clears resolved_head at `crates/rum-runtime/src/runtime/definitions/rule_lhs/tests.rs:251`; held-pattern clears it at `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution/tests.rs:305`. Fixed expected roots and expected-panic text test that each local comparison notices the missing sentinel (`:217`/`:248` and `:263`/`:302`). The positive tests use distinct state types, destinations and pending variants (`crates/rum-runtime/src/runtime/definitions/rule_lhs/tests.rs:220`, `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution/tests.rs:270`). Keep these deliberate mirrors, while recognizing that neither supplied falsifier demonstrates that a production trace omission turns the positive certificate red. D08 names that separate cost. No panic added elsewhere and no weakened expected vector would count for the requested production-mutant bar.

### Open obligations and their costs

These are limitations of certificate evidence and costs for a director decision, not production defects and not authorization to change tests. Every row has a one-line classified claim; none required an unresolved classification policy choice. A completed design can carry these open implementation obligations (brief:109).

| ID / affected rows | Obligation and evidence | Disposition / cost |
| --- | --- | --- |
| D01 / I07 | Natural Expr destruction has no isolated explicit drain to remove. The observed drop is `crates/rum-runtime/src/runtime/tests/left_chain_lowering.rs:579`; SharedExpr and SharedArgs are plain Arc wrappers at `crates/rum-core/src/expr.rs:574` and `crates/rum-core/src/expr.rs:638`, with Expr fields at `crates/rum-core/src/expr.rs:750`. A parser/lowerer failure, an inserted panic, or extra recursion elsewhere is not a natural-drop falsifier. The test does not assert the lowered raw shape before drop. | Open: identify a small safe ownership/drop production mutant or explicitly retain this as an uncalibrated completion probe. Cost: bounded child isolation and a raw shape/unique-ownership witness in a separately authorized certificate-only follow-up; replacing the Arc representation or adding a destructor solely to create a red is a disproportionate core ownership rewrite and is prohibited. |
| D02 / I08, I12, I17 | A destructor mutation may abort before its Rust assertion or may survive the deployment depth. M08 and M12 name the actual cleanup calls at `crates/rum-stdlib/src/associations/basic/lookup.rs:33` and `crates/rum-stdlib/src/associations/key_reshape/traversal.rs:84`. The I08 observer is `crates/rum-stdlib/src/associations/basic/lookup.rs:863`; I12 return equality is `crates/rum-stdlib/src/tests/association_key_reshape_depth.rs:347`. The 60000-depth comment at `crates/rum-stdlib/src/tests/association_key_reshape_depth.rs:319` is source rationale, not fresh evidence. | Open: one isolated baseline/mutant build and exact test per drop site; preserve child output and establish that failure is in this destructor, not setup, formatter or another drop. Cost: potentially two release builds per site and a later child-observer design. If only a signal/timeout is obtained, report INCONCLUSIVE and do not credit it. No arbitrary panic or test-depth change is allowed in Stage B. |
| D03 / I31 | The singleton Take fixture cannot force selection or distinguish the identity shortcut. The source requests {1} at every level at `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:208` and expects the unchanged nested singleton shape at `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:211`. Replacing the production tail handoff at `crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs:320` with Some(expr.clone()) is statically predicted to survive. M31 only checks that a damaged reconstruction is observed. | Open: a separately authorized certificate fixture with a selected unique deep sentinel and an excluded sibling at the transition into the tail; literal expected shape must differ from input. Cost: one fixture/helper change, one no-op falsifier calibration, inventory replay and workspace acceptance after that source change; no production repair. |
| D04 / I06, I28, I29, I30, I45, I46, I47, I48, I49, I50, I51 | Output-only, count-only or absent-pattern observers do not establish every stronger algorithmic claim. Accepted left-chain arity does not falsify rejection diagnostics (`crates/rum-runtime/src/runtime/tests/left_chain_lowering.rs:435`, `crates/rum-runtime/src/runtime/tests/left_chain_lowering.rs:344`); M06b separately targets rejection. Root MatchQ is not deep pattern descent (`crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:179`); Depth is its own shape observer (`crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:188`); ToString only checks length (`crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:199`); D only checks Length (`crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:535`); absent z gives empty/false/zero/true at `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:664`, `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:682`, `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:691`, `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:700`, `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:708`; Level checks quantity at `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:673`. | Open: retain the one-line narrow claims in this report; if broader certificates are wanted, add positive deepest sentinels, literal paths/parts/factors and independently inspected raw shapes in a separate certificate-only brief. Cost: several fixture assertions and isolated mutant builds, plus standing replays/workspace after source changes; not an evaluator rewrite and not a demand to modify ordinary behaviour tests. |
| D05 / I53 | The MapIndexed heads refusal does not isolate or establish completing head traversal. The reason is explicit at `crates/rum-stdlib/src/tests/map_indexed_association_depth.rs:211`; the direct error check is `crates/rum-stdlib/src/tests/map_indexed_association_depth.rs:245`. M53 tests the actual eager-evaluation error path. The existing comment points to separate forced-front coverage and does not turn this request into a walker witness. | Open: keep the refusal certificate narrowly described, or design a direct raw-call/forced-front certificate with independent head-position observation under a separately scoped test brief. Cost: a bounded fixture and an observer at the walker boundary; bypassing or changing production evaluation order to make this test prove more is forbidden. |
| D06 / I54, I56, I57 | Feature-gated guard calibration is excluded by the current hard rule. The exact exclusion is `docs/dev/codex/brief-639-test-audit-inventory.md:118`; required feature strings are `crates/rum-stdlib/src/tests/merge_native_stack_guard.rs:64`, `crates/rum-stdlib/src/tests/scan_and_keypath_set_native_stack_guard.rs:148`, `crates/rum-stdlib/src/tests/scan_and_keypath_set_native_stack_guard.rs:196`. M54 and M54d are designs only. | Open: director must explicitly authorize these isolated feature builds in a later brief or carry the obligation forward. Cost: a separate feature build plus baseline and mutated exact runs for each edge; absent permission, stop these rows without executing them. |
| D07 / cross-cutting | No currently eligible copy collapse survives the leave-behaviour-alone rule. DUP07–DUP09 call the same compare helper with identical source/input fixtures at `crates/rum-stdlib/tests/option_reader_continuations/boolean.rs:284`, `crates/rum-stdlib/tests/option_reader_continuations/rule.rs:44`, `crates/rum-stdlib/tests/option_reader_continuations/phases.rs:294`. They are behaviour copies; the brief protects behaviour at `docs/dev/codex/brief-639-test-audit-inventory.md:25`. | Resolved for this dispatch: leave all copies and mirrors intact. Cost of a future explicitly authorized cleanup would be consolidating the three fixtures into their owning rule/phases tests or named shared cases, preserving each observable assertion, followed by standing replays and workspace acceptance. No cleanup permission is inferred here. |
| D08 / cross-cutting | Existing fixture-removal falsifiers do not calibrate production trace omission. The rule-LHS falsifier clears its fixture field at `crates/rum-runtime/src/runtime/definitions/rule_lhs/tests.rs:251` and the held-pattern falsifier at `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution/tests.rs:305`. Their production trace paths and positive observers remain distinct. | Open for a future expansion beyond the ignored census: omit one production root trace while leaving the positive fixture intact, then require that positive certificate to fail at its exact roots assertion. Cost: a safely compiling scratch derive/trace mutation for each state owner, not merging the two tests or editing expected roots. |
| D09 / cross-cutting | Director calibration does not supply per-mutant clean-copy build bounds. Existing release build/test costs are `.temp/director/audit-638/ignored-release.log:36`, `.temp/director/audit-638/ignored-release.log:327` and `.temp/director/audit-638/profile-gate-release.log:23`. They are prior director receipts, not a timing run here; the standalone child timeout at `crates/rum-stdlib/tests/typed_slot_census.rs:198` is for a prebuilt binary, not cargo build in a clean copy. | Open before Stage B launch: director supplies measured copy/build/test bounds and offline dependency availability for that shape. Cost: one authorized clean-copy calibration and its retained logs if no matching receipt exists. Never guess a timeout, repeat the forbidden 55-test calibration, or call a timeout a killed mutant. |

### Stages A through D, with stops

A. Inventory / evidence freeze. This report is the completed design-stage inventory: preserve the 61 source records, exact ignore reasons, resolved assertion helpers, all duplicate members and the supplied director logs. Check the raw ignore-attribute set independently against the extracted test set and compare source hashes before/after the survey. No cargo, instrument, generator or falsifier runs are needed for this stage (`docs/dev/codex/TEMPLATE.md:232`; brief:69). Stop an individual row if its claim cannot be stated; stop the stage on a classification policy conflict, unaccounted attribute, unstable source snapshot or unresolved duplicate binding. None of those conditions was found after resolving the macro templates; the first survey's 12 unresolved template sites remain recorded in its log, not concealed.

B. Production-mutant campaign, only after a new director dispatch. Use the `mutants` and `obligations` sections as the work list. Each mutant ID gets one unique bounded workspace copy under `.temp/brief-639/mutants/<ID>/<attempt>/workspace`, with its own target, temporary directory and writable offline Cargo cache under that attempt. The directory is audit scratch, never a worktree or candidate to promote. Copy ordinary file bytes (no hardlinks or symlinks back to the authoritative tree), excluding `.git`, existing `.temp`, target and credential/config directories. Record the selected source hash and exact old/new snippet. Bound copy/build/test/cleanup processes through `scripts/run-bounded.py`; validate containment before every write or removal. Retain logs, manifest and checksum receipts even if the copied workspace is removed. Stop if any source path points outside this copy or dependencies require network access.

Run one cargo command at a time, with CARGO_BUILD_JOBS=8 and RUST_TEST_THREADS=8; no CPU-heavy work overlaps. The concurrency evidence and serial-red rule are `docs/dev/codex/TEMPLATE.md:193`. Before any launch the director must supply bounds from a measured equivalent copy/build/test shape (D09); neither the 504.68 s group run (`.temp/director/audit-638/ignored-release.log:327`) nor the prebuilt-probe 900 s bound (`crates/rum-stdlib/tests/typed_slot_census.rs:198`) is a measured clean-build timeout. If unavailable, stop at setup instead of inventing a value. A timeout, OOM, compile error, zero selected tests, unrelated panic or missing environment is INCONCLUSIVE, never a killed mutant. Follow the template's once-only one-job OOM retry only after a confirmed oom_kill and record both attempts; a semantic failure must reproduce with --test-threads=1 before it is credited.

Within each copy, first compile and run each exact target unchanged with that row's profile/feature/env and capture its green receipt. Do not rerun the director's 55-test selection. Then apply exactly one listed production mutation; no test/assertion changes, arbitrary panics or compensating mutations. Compile through `cargo test --offline --locked --release -p <row's package> --lib -- <fully-qualified-name> --exact --ignored --nocapture --test-threads=8`, wrapped by run-bounded with the supplied bound. I03 additionally sets RUM_PROFILE_EVALUATION_CACHE=1 and unsets the child marker so the parent executes. Every listed target gets its own cargo test invocation; one mutant may serve multiple exact targets only for the equivalence claim recorded in its row. A predicted red is credited only after the exact assertion/source line and changed value match the observation path, all intervening steps are inspected, and the serial retry repeats it. Preserve the complete cargo output and quote the actual failing test, assertion text and test-result verdict verbatim. A successful test means SURVIVED, not success for the audit. No-mutant I07 stays open; M08/M12 follow D02 and cannot be credited from an unexplained abort. The profile env-precondition red never counts as M03.

Standing maintenance applies inside a copy whenever its covered source changes. Before the first target/acceptance run after an edit, replay the applicable existing generators and acceptance witnesses, with every output redirected to paths inside that copy; do not hand-edit inventories. Follow `docs/dev/codex/TEMPLATE.md:88`: local-contract fingerprints, promoted certificate cells from local_production_acceptance in default and no-default-features modes, ownership positions/discovery, LP01 producer/binding/loss records, typed-slot receipts and rejection census as applicable. Record every derived movement and quote every replay verdict. If a replay would execute an excluded instrument/falsifier/guard, escape the copy, approve an intentionally corrupted contract, change an existing row's identity/status/disposition/exclusion, lose an existing row, introduce UnknownHostPayload, or require a rejection-census ruling, stop that mutant and return it to the director. Do not update an expected baseline to bless the mutation. A stale fingerprint/certificate/position/receipt-only red is an ORDERING ARTIFACT: replay, then retry the same exact target; it is neither a production regression nor a successful falsification. A genuine contract rejection during replay is recorded separately and does not substitute for the promised target assertion. This negative-control stage has no acceptance candidate to merge; full workspace acceptance applies to any later authoritative source/test edit, after its replay. No full workspace run is proposed on a deliberately damaged copy merely to obtain a red.

Keep I04/I05 instruments, I61 and its SCRATCH_ENV parents, I55 generator as a target, all behaviour rows and the I54/I56/I57 feature guards outside the executable campaign. Guard designs remain D06 unless a later director brief explicitly changes that exclusion. Stage B stops each failing path for the wrong reason, any swallowed/unreachable predicted red, a scope violation or missing bound. Report killed, survived, inconclusive, excluded and open separately. Two successive attempts with no progress stop further tuning; compiler-error reduction is progress, never a killed mutant (`docs/dev/codex/TEMPLATE.md:178`). No patch moves from scratch into production.

C. Duplicate disposition / conditional collapse. Preserve DUP01–DUP06 as deliberate mirrors; their same-text bodies bind different state types, source sets or lifecycle helpers. DUP07–DUP09 are copies, but their behaviour assertions are explicitly protected (`docs/dev/codex/brief-639-test-audit-inventory.md:25`, `:104`). Therefore the present authorized action for Stage C is to record the proposed shared forms and perform no edits. Stop with NO ELIGIBLE COLLAPSE rather than treating body equality as permission to delete coverage. Only a new explicit director scope may authorize the three owning fixture files; such a follow-up must keep the compare observer and exact source/input expectations, demonstrate equivalent family coverage, replay standing inventories before its first acceptance suite, then run focused tests and workspace. No production, semantics, public API, diagnostics or hot paths may change for deduplication.

D. Open obligations and costs. Carry D01–D09 with their named observer, missing evidence, scope owner and cost. D01 is the current example of a certificate claim with no honest localized production mutant identified; D02 needs causal drop evidence; D03–D05 concern transformation/observer adequacy; D06 is an explicit execution exclusion; D07 is already resolved as leave-behaviour-alone; D08 judges the supplied falsifier model; D09 blocks unbounded campaign setup. The output is a director decision, not an evaluator fix. Stop before representation rewrites, new unsafe, test weakening, production changes to create an audit green, or changed assertion contracts. Any authorized future certificate-strengthening work is a separate source-change stage with replay, workspace acceptance and unchanged production semantics. Hot-path changes, if separately requested for another goal, would additionally require the project's base-vs-new microbench gate; none is justified by this audit (`AGENTS.md:117`).

### Draft Stage A brief in TEMPLATE shape

# Draft follow-up A — freeze and review the audit inventory

**Shape: DIAGNOSTIC / DESIGN. Scope: NONE. Director rewrites and assigns the final brief number before dispatch.**

## Goal

Produce an auditable source snapshot and complete ignored/duplicate tables, accounting for every source attribute and giving every certificate/contract/inventory a precise assertion and either a production-mutant design or an explicit open obligation. Reuse the accepted brief-639 receipts when unchanged; do not repeat a reference stage after another stage has consumed it (`docs/dev/codex/TEMPLATE.md:224`).

## Context pointers

- `AGENTS.md`, `docs/testing-strategy.md`, `docs/dev/codex/TEMPLATE.md`.
- `.temp/brief-639/report.json` sections inventory, mutants, duplicates, obligations; `.temp/brief-639/survey-complete.json` is the source extraction.
- Existing director calibration `.temp/director/audit-638/ignored-release.log:252` and `.temp/director/audit-638/profile-gate-release.log:27`: I03 is green with its environment.

## Scope

Files you MAY modify: NONE in the authoritative tree. Scratch only `.temp/brief-639/stage-a/`. Do not edit crates, docs, TODO, inventories, Cargo.lock, tests or any existing director scratch. All source reads use the current bytes without git. No cargo command is authorized or needed. Standing replay is N/A without source edits.

## Deliverable

- The source-level ignored table with stable ID, fn/attribute path:line, literal reason, mode/feature/env, claim/assertions, class, calibration provenance and candidate/open ID.
- Exact whitespace-normalized duplicate membership, resolved local helper/type bindings, judgment and allowed action; preserve all behaviour tests.
- Hash receipt, raw attribute reconciliation, unresolved macro-template list (must be empty after explicit accounting), source snapshot check and row-by-row review notes.
- Frozen mutation manifest proposal: `{id, rows, path, line, old_snippet, replacement, package, target_kind, exact_tests, profile, features, env, assertion_sites, expected_changed_value, no_swallow_path, exclusion, obligation}`. This is scratch JSON, not a new Rust API.

## rum / Rust pitfalls (read twice)

Do not classify by ignore reason alone. Resolve #[test] macro templates without pretending to enumerate compiler expansions. Compare bodies only after balanced Rust literal/comment handling; attributes such as should_panic and lexical bindings still matter to the duplicate judgment. A heavy behaviour test and a review-copy generator are not broken inventories. A certificate's output may be unchanged by a no-op transformation: Take I31 is the concrete example (`crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:208`).

## Verification (MANDATORY)

Compare the raw ignore-site set with the table set, check all assertion paths exist, rederive both whitespace-normalization groupings, verify source checksums, then validate rum-report/1. Quote every survey/check verdict verbatim. No suites, profiling, performance gates or calibration reruns. Independent reads may be batched; no background work is required.

## Stop conditions

STOP for a row whose claim cannot fit one line, and continue the rest. STOP the stage for an unsettled classification policy, incomplete source parsing, unexplained membership/count drift or source bytes changing during the audit. Do not fill gaps with guesses.

## Checkpointing — protect the scene

Keep PROGRESS.md, extraction script, manifests, snapshots and raw logs under stage-a; log source counts, resolved rows and remaining obligations. Report review progress before handoff. The sole expected result is evidence, not a source patch.
## Hard rules

No git in any form; no network; the director owns repository state. Use only the named scratch subtree. Production and existing behaviour tests stay unchanged. No third-party engine oracle or compatibility claim. Keep generated rum sources suffixed .rum. No unsafe, no default-path diagnostics cost, and no production change to make the audit pass. Never hand-edit or weaken a standing inventory; replay covered inventories before the first acceptance suite and quote their verdict lines. An inventory-only stale-cell red is an ordering artifact, not a regression. Explicitly excluded instruments, SCRATCH_ENV falsifiers and feature guards remain excluded unless the director's new dispatch says otherwise. These rules derive from `docs/dev/codex/brief-639-test-audit-inventory.md:115`, `docs/dev/codex/TEMPLATE.md:88`, and AGENTS.md.

## Report format

Write rum-report/1 under the assigned scratch subtree; include all attempts and verbatim observed verdicts. Use `python3 scripts/report.py check <report-path>` without --against-git; final response is exactly that report's `summary` output. No generated artifact is installed into the authoritative tree.


### Draft Stage B brief in TEMPLATE shape

# Draft follow-up B — isolated production-mutant calibration

**Shape: DIAGNOSTIC, negative controls in scratch copies. No authoritative source change. Director must supply measured bounds and approve the executable subset before dispatch.**

## Goal

For every enabled candidate in the accepted manifest, observe a green unchanged exact test followed by a compiled production mutant that fails the named observer, or report survived/inconclusive/open with its causal explanation. Do not equate an arbitrary red with proof. The production tree remains byte-for-byte untouched by this worker.

## Context pointers

- `AGENTS.md`, `docs/testing-strategy.md`, `docs/dev/codex/TEMPLATE.md:88` and `:193`.
- Brief-639 report sections mutants, obligations and falsifier-assessment; production and assertion path:line are the dispatch contract.
- `crates/rum-stdlib/tests/typed_slot_census.rs:196`, `:221`, `:259`, `:292` show bounded/exact-marker ideas, but their drivers are excluded and do not compile mutated source.

## Scope

Authoritative files you MAY modify: NONE. For each mutant, only the listed production snippet in `.temp/brief-639/mutants/<ID>/<attempt>/workspace/` may differ from its copied source. Tests, helpers, expectations and dependencies remain identical; derived inventories may move only through faithful existing replay within that copy, never to endorse the mutation. Runtime/source representation, unsafe, feature defaults and production interfaces cannot be redesigned. No instrument, SCRATCH_ENV falsifier, behaviour stress test, generator-as-target or zero-feature guard executes. M54/M54d are excluded pending a separately explicit director ruling; D01 has no executable mutant. Scope must be rewritten before any exception.

## Deliverable

- One owned copy namespace per mutant, independent bytes, offline cache/target/tmp inside it, with a manifest and original snippet hash.
- For every exact test: green baseline log, candidate compile/run log, serial red retry where applicable, exact failing assertion and changed value, complete verdict lines, and killed/survived/inconclusive/excluded/open classification.
- Replay receipts in copy before the first target run after editing; list every moved derived cell and any ordering artifact. Keep test hashes identical before/after.
- Recheck authoritative source hashes after each mutant. Retain evidence outside the copy; safely remove the copied workspace or mark it clearly AUDIT SCRATCH — NOT FOR PROMOTION.

## rum / Rust pitfalls (read twice)

The mutation interface is the frozen JSON manifest from Stage A. The scratch driver exposes `run_mutant(spec, limits) -> receipt` only in a scratch Python script: `limits` requires copy/build/test/cleanup seconds plus receipt paths; no default guessed timeout is permitted. `receipt` records original/mutated hashes, selected-test count, compile status, exit/signal, logs, assertion location, changed value, replay receipts and classification. Use subprocess argv arrays, not shell interpolation of source strings. Copy bytes, never hardlink or follow an external symlink. Unset profile-child marker for M03. Tests with expected results computed by production code need a tail-only mutation (M19); later reevaluation can swallow a lost replacement (inspect M38). Returned KeyDropFrom data can still be correct when the stored owner is stale (M26). Drop aborts need causal evidence and cannot be credited just because the command failed (D02).

## Verification (MANDATORY)

Director supplies measured bounded-copy/build/test receipts; D09 is a setup stop if absent. Run one cargo command at a time, jobs=8, test threads=8; no overlapping heavy workloads. In each copy use its checked-in toolchain, `cargo test --offline --locked --release -p <rum-runtime or rum-stdlib> --lib -- <exact-test-name> --exact --ignored --nocapture --test-threads=8`, as an argv child of `python3 scripts/run-bounded.py --timeout <supplied-seconds> --`. RUM_PROFILE_EVALUATION_CACHE=1 is required only for M03; preserve other row-specific settings. Compile/run the unchanged target before applying the one production mutation. After mutation replay all covered standing inventories inside the copy before the target test; stop if the replay would cross scope, invoke an excluded driver, or change semantic inventory classifications. Retry a stale-derived-cell-only red after replay and label the first result ORDERING ARTIFACT. Do not count it. Run each equivalent target separately. Reproduce a claimed assertion red at --test-threads=1 and quote the actual failure and final test-result line, never predicted text. Report compile errors, OOM, timeout, process signal without causal proof, zero test selection and wrong assertions separately. No production acceptance suite is needed because there is no authoritative source edit; any later implementation dispatch that edits source must replay then run workspace acceptance.

## Stop conditions

Stop a row on an unrelated panic/compile failure/timeout; no mutant credit. Stop on a swallowed predicted red and explain the surviving value path. Stop an entire attempt on scope leakage, modified tests, unknown source hashes, absent offline dependencies, missing measured bound or an inventory policy change. After two consecutive attempts without progress, stop tuning that row. Do not repair production or assertions to make the audit pass.

## Checkpointing — protect the scene

Keep a manifest receipt for every phase. A resumed run skips completed baselines and attempts; never rerun a consumed calibration. PROGRESS records counts of compiled candidates, correct observer reds, survived, inconclusive, excluded and open; failed attempts stay in the report. Keep all generated .rum inputs and all logs under the mutant namespace.
## Hard rules

No git in any form; no network; the director owns repository state. Use only the named scratch subtree. Production and existing behaviour tests stay unchanged. No third-party engine oracle or compatibility claim. Keep generated rum sources suffixed .rum. No unsafe, no default-path diagnostics cost, and no production change to make the audit pass. Never hand-edit or weaken a standing inventory; replay covered inventories before the first acceptance suite and quote their verdict lines. An inventory-only stale-cell red is an ordering artifact, not a regression. Explicitly excluded instruments, SCRATCH_ENV falsifiers and feature guards remain excluded unless the director's new dispatch says otherwise. These rules derive from `docs/dev/codex/brief-639-test-audit-inventory.md:115`, `docs/dev/codex/TEMPLATE.md:88`, and AGENTS.md.

## Report format

Write rum-report/1 under the assigned scratch subtree; include all attempts and verbatim observed verdicts. Use `python3 scripts/report.py check <report-path>` without --against-git; final response is exactly that report's `summary` output. No generated artifact is installed into the authoritative tree.


### Draft Stage C brief in TEMPLATE shape

# Draft follow-up C — duplicate disposition and eligibility gate

**Shape: DIAGNOSTIC / conditional implementation. Current eligible edit set: EMPTY. Director must rewrite scope if a later cleanup is explicitly authorized.**

## Goal

Preserve every distinct contract/lifecycle observer while identifying whether any true copy can be consolidated under the leave-behaviour-alone rule. The current census has six deliberate-mirror groups and three protected behaviour-copy groups (`.temp/brief-639/survey-complete.log:1`; report duplicates section). The expected current result is NO ELIGIBLE COLLAPSE, with proposed shared forms recorded.

## Context pointers

- `AGENTS.md`, `docs/testing-strategy.md`, `docs/dev/codex/TEMPLATE.md`.
- Brief-639 report duplicates and D07; `docs/dev/codex/brief-639-test-audit-inventory.md:25`, `:98`.
- Root observer owners: `crates/rum-runtime/src/runtime/definitions/rule_lhs/tests.rs:217` and `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution/tests.rs:263`.
- Copy owners: `crates/rum-stdlib/tests/option_reader_continuations/boolean.rs:284`, `crates/rum-stdlib/tests/option_reader_continuations/rule.rs:44`, `crates/rum-stdlib/tests/option_reader_continuations/phases.rs:294`.

## Scope

Files you MAY modify: NONE. Scratch only `.temp/brief-639/stage-c/`. No production, no tests, no fixtures, no inventory or doc edits under this version. Preserve all DUP01–DUP06 members. Preserve DUP07–DUP09 because they are behaviour tests. A future dispatch must explicitly list any allowed owning files and resolve behaviour protection before implementation; body equality does not override it.

## Deliverable

For every group, record the lexical helper/type bindings, protected assertions and eligible action. Proposed future shared interface, if explicitly authorized: three `fn <case_name>()` fixture functions that each invoke the existing `compare(label, source, &[], &[])` once (`crates/rum-stdlib/tests/option_reader_continuations/support.rs:70`), or one canonical owning test per case in rule.rs/phases.rs. Do not make a generic observer abstraction, change source strings, change input responses, merge contract inventories, or claim removal of duplicate one-line wrappers as proof of semantic equivalence. Record which test names would remain, and why coverage is unchanged, before an authorized edit.

## rum / Rust pitfalls (read twice)

Same body plus different local cancel/root helpers is a deliberate mirror, not duplicate behaviour. A new helper under crates also expires the standing fingerprints; moving line numbers expires position/receipt cells. No hand-edited expected rows. No deletion of a falsifier merely because its fixture is small. Preserve expected-panic messages and independent owner types.

## Verification (MANDATORY)

For the present scope, verify the nine membership groups and record NO ELIGIBLE COLLAPSE; no cargo. If a later rewritten brief authorizes a concrete test-only consolidation, its sequence must be edit → all applicable standing replays → focused option-reader integration suite → cargo fmt --check → cargo test --workspace, with measured bounds, one cargo at a time, jobs/threads=8, serial reproduction of red, verbatim verdicts and no pre-replay acceptance run. Inventory-only stale-cell failures are ordering artifacts. A test-only fixture cleanup has no hot-path performance claim; production edits are forbidden.

## Stop conditions

Stop implementation when the eligible set is empty (the current expected disposition), a copy is a protected behaviour test, lexical bindings differ, a canonical assertion would be lost, or inventory identity/status/disposition would change. Return evidence instead of asking the worker to infer a policy exception.

## Checkpointing — protect the scene

Keep candidate shared forms, exact test-name mapping and scope disposition in stage-c's rum-report/1. No source patch accompanies the current result.
## Hard rules

No git in any form; no network; the director owns repository state. Use only the named scratch subtree. Production and existing behaviour tests stay unchanged. No third-party engine oracle or compatibility claim. Keep generated rum sources suffixed .rum. No unsafe, no default-path diagnostics cost, and no production change to make the audit pass. Never hand-edit or weaken a standing inventory; replay covered inventories before the first acceptance suite and quote their verdict lines. An inventory-only stale-cell red is an ordering artifact, not a regression. Explicitly excluded instruments, SCRATCH_ENV falsifiers and feature guards remain excluded unless the director's new dispatch says otherwise. These rules derive from `docs/dev/codex/brief-639-test-audit-inventory.md:115`, `docs/dev/codex/TEMPLATE.md:88`, and AGENTS.md.

## Report format

Write rum-report/1 under the assigned scratch subtree; include all attempts and verbatim observed verdicts. Use `python3 scripts/report.py check <report-path>` without --against-git; final response is exactly that report's `summary` output. No generated artifact is installed into the authoritative tree.


### Draft Stage D brief in TEMPLATE shape

# Draft follow-up D — adjudicate unreachable and under-observed certificate claims

**Shape: DESIGN / OBLIGATION REVIEW. Scope: NONE.**

## Goal

Give each open certificate obligation an honest narrow claim, an uncredited mutant status, the smallest additional evidence needed and a named cost. A disproportionate production rewrite is an explicit unresolved obligation, never a reason to change production so that the audit passes (`docs/dev/codex/brief-639-test-audit-inventory.md:109`, `:138`).

## Context pointers

- `AGENTS.md`, `docs/testing-strategy.md`, `docs/dev/codex/TEMPLATE.md`.
- Brief-639 report obligations D01–D09 and Stage B receipts if dispatched.
- Natural drop: `crates/rum-runtime/src/runtime/tests/left_chain_lowering.rs:579`, `crates/rum-core/src/expr.rs:574`, `:638`, `:750`.
- Identity Take: `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:208`, `crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs:320`.
- Heads refusal: `crates/rum-stdlib/src/tests/map_indexed_association_depth.rs:211`, `:245`.

## Scope

Files you MAY modify: NONE. Scratch only `.temp/brief-639/stage-d/`. No feature guard or instrument run, no test edit, no production mutation in the authoritative tree, no core representation change. This stage cannot silently turn an open design into an implementation request.

## Deliverable

- Obligation ledger `{id, rows, actual_claim, missing_observer_or_mutant, evidence, least_scope, cost, decision, next_brief}`.
- D01: assess whether a small safe production drop mutant exists; otherwise keep the natural-drop probe uncalibrated and identify the core ownership rewrite as disproportionate.
- D02: account for stack-abort/survival evidence from bounded children, without accepting unrelated signals.
- D03/D04: propose only certificate fixtures with changed deep sentinels, excluded siblings, exact result paths/factors or raw shape observations; keep ordinary behaviour tests alone.
- D05: separate eager-evaluation refusal from a completing head traversal witness.
- D06/D09: record the director's exclusion and bound decisions; D07 remains no eligible cleanup; D08 identifies production-trace omission coverage beyond fixture corruption.

## rum / Rust pitfalls (read twice)

An ignored green completion probe is not universal proof of bounded destruction. A wrong-value mutant does not show that an identity fixture forces transformation. No-match outputs and lengths omit identities/order. An arbitrary panic, weakened assertion or malformed Rust mutant cannot replace causal evidence. No new unsafe or permanent diagnostic branch is permitted to manufacture a falsifier.

## Verification (MANDATORY)

Verify every ledger entry against the cited current source and retained Stage B logs; quote actual reds and verdicts exactly when they exist, label unrun predictions as such. Do not invent measurements or run the director's calibration again. Validate the report without --against-git. No acceptance suite or inventory replay applies with no source changes. Any approved certificate-strengthening implementation must be a new scoped brief with calibrated falsifier, measured bounds, standing replay before first acceptance, and workspace suite; no production semantic fix is bundled.

## Stop conditions

Stop any attempted implementation, production/core rewrite, assertion weakening or policy reinterpretation. Stop a row where no proportionate mutant or observer can be specified and carry it as OPEN with cost. Completion of this review means the obligations are explicit, not that all certificates have acquired evidence they do not yet have.

## Checkpointing — protect the scene

Keep each decision and its evidence in stage-d's report and PROGRESS. Reuse prior accepted receipts; no reference-stage reruns. The director owns disposition and any next implementation brief.
## Hard rules

No git in any form; no network; the director owns repository state. Use only the named scratch subtree. Production and existing behaviour tests stay unchanged. No third-party engine oracle or compatibility claim. Keep generated rum sources suffixed .rum. No unsafe, no default-path diagnostics cost, and no production change to make the audit pass. Never hand-edit or weaken a standing inventory; replay covered inventories before the first acceptance suite and quote their verdict lines. An inventory-only stale-cell red is an ordering artifact, not a regression. Explicitly excluded instruments, SCRATCH_ENV falsifiers and feature guards remain excluded unless the director's new dispatch says otherwise. These rules derive from `docs/dev/codex/brief-639-test-audit-inventory.md:115`, `docs/dev/codex/TEMPLATE.md:88`, and AGENTS.md.

## Report format

Write rum-report/1 under the assigned scratch subtree; include all attempts and verbatim observed verdicts. Use `python3 scripts/report.py check <report-path>` without --against-git; final response is exactly that report's `summary` output. No generated artifact is installed into the authoritative tree.


### Supplied calibration verdicts, quoted verbatim

These are the director's already supplied logs, not commands executed in brief 639. The selected release calibration executed 55 distinct ignored tests: 54 passed immediately, and I03 failed only its required environment precondition before passing with RUM_PROFILE_EVALUATION_CACHE=1. Six excluded rows are I04/I05/I54/I56/I57/I61. This precondition red is not a production defect and not an inventory-ordering artifact. Generator I55 ran successfully, which demonstrates only review-copy generation, not installed fingerprint acceptance. Source: `.temp/director/audit-638/ignored-release.log:202`, `.temp/director/audit-638/ignored-release.log:252`, `.temp/director/audit-638/ignored-release.log:261`, `.temp/director/audit-638/ignored-release.log:316`, `.temp/director/audit-638/profile-gate-release.log:27`.

Verbatim verdict lines below include every test result / individual test outcome / memory-scope receipt present in these two existing logs. Timings and memory are quoted historical resource data, not a new performance measurement or a bound for clean scratch builds.

`.temp/director/audit-638/ignored-release.log` (line citations precede each exact line):

`.temp/director/audit-638/ignored-release.log:41`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 16 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:47`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 28 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:53`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:59`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:65`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:71`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:77`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:83`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:89`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 90 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:95`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:101`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:107`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:113`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 17 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:119`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:125`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:131`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:137`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:143`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 16 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:149`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:155`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:161`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:167`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:173`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:179`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:185`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:191`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:197`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:202`
```text
test battered_session_recovers_after_repeated_recoverable_limit_trips ... ok
```

`.temp/director/audit-638/ignored-release.log:203`
```text
test cancel_rewrite_loop_stability_10_cycles ... ok
```

`.temp/director/audit-638/ignored-release.log:205`
```text
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.27s
```

`.temp/director/audit-638/ignored-release.log:211`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 13 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:217`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:223`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 152 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:229`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:235`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 48 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:241`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:247`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:252`
```text
test runtime::evaluation_cache::tests::call_stability_profile_reports_per_source_head_outcomes ... FAILED
```

`.temp/director/audit-638/ignored-release.log:253`
```text
test runtime::tests::left_chain_lowering::left_chain_two_mib_full_round_trip_accepts_and_rejects_without_abort ... ok
```

`.temp/director/audit-638/ignored-release.log:254`
```text
test runtime::tests::left_chain_lowering::left_chain_two_mib_nested_expr_drops_naturally_at_the_boundary ... ok
```

`.temp/director/audit-638/ignored-release.log:261`
```text
this profiling gate must be run with RUM_PROFILE_EVALUATION_CACHE=1
```

`.temp/director/audit-638/ignored-release.log:268`
```text
test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 1243 filtered out; finished in 0.10s
```

`.temp/director/audit-638/ignored-release.log:274`
```text
test associations::basic::lookup::tests::lookup_outcome_drop_is_bounded_at_physical_height_4097_on_two_mib ... ok
```

`.temp/director/audit-638/ignored-release.log:275`
```text
test tests::association_keys_depth::keys_completes_at_physical_height_4097_on_two_mib ... ok
```

`.temp/director/audit-638/ignored-release.log:276`
```text
test tests::association_key_take_drop_depth::key_drop_completes_at_physical_height_4097_on_two_mib ... ok
```

`.temp/director/audit-638/ignored-release.log:277`
```text
test tests::association_key_take_drop_depth::key_take_completes_at_physical_height_4097_on_two_mib ... ok
```

`.temp/director/audit-638/ignored-release.log:278`
```text
test tests::association_key_reshape_depth::association_key_deflatten_completes_at_physical_height_4097_on_two_mib ... ok
```

`.temp/director/audit-638/ignored-release.log:279`
```text
test tests::association_values_depth::values_completes_at_physical_height_4097_on_two_mib ... ok
```

`.temp/director/audit-638/ignored-release.log:280`
```text
test tests::association_lookup_depth::lookup_hit_completes_at_physical_height_4097_on_two_mib ... ok
```

`.temp/director/audit-638/ignored-release.log:281`
```text
test tests::association_lookup_depth::lookup_miss_and_resume_drop_complete_at_physical_height_4097_on_two_mib ... ok
```

`.temp/director/audit-638/ignored-release.log:282`
```text
test tests::delete_missing_depth::delete_missing_scan_completes_at_physical_height_4097_on_two_mib ... ok
```

`.temp/director/audit-638/ignored-release.log:283`
```text
test tests::delete_missing_depth::delete_missing_recursive_completes_at_physical_height_4097_on_two_mib ... ok
```

`.temp/director/audit-638/ignored-release.log:284`
```text
test tests::delete_missing_target_certificates::delete_missing_target_scan_all_completes_at_physical_height_4097_on_two_mib ... ok
```

`.temp/director/audit-638/ignored-release.log:285`
```text
test tests::association_key_reshape_depth::association_key_deflatten_abandons_a_deep_partial_tree_without_growing_the_stack ... ok
```

`.temp/director/audit-638/ignored-release.log:286`
```text
test tests::delete_missing_target_certificates::delete_missing_target_scan_zero_completes_at_physical_height_4097_on_two_mib ... ok
```

`.temp/director/audit-638/ignored-release.log:287`
```text
test tests::iterative_traversal_depth::apply_completes_at_physical_height_4097_on_two_mib ... ok
```

`.temp/director/audit-638/ignored-release.log:288`
```text
test tests::held_association_exact_path_depth::associate_to_part_target_completes_at_physical_height_4097_on_two_mib ... ok
```

`.temp/director/audit-638/ignored-release.log:289`
```text
test tests::held_association_exact_path_depth::key_drop_from_part_target_completes_at_physical_height_4097_on_two_mib ... ok
```

`.temp/director/audit-638/ignored-release.log:290`
```text
test tests::iterative_traversal_depth::association_map_completes_at_physical_height_4097_on_two_mib ... ok
```

`.temp/director/audit-638/ignored-release.log:291`
```text
test tests::iterative_traversal_depth::association_assignment_read_completes_at_physical_height_4097_on_two_mib ... ok
```

`.temp/director/audit-638/ignored-release.log:292`
```text
test tests::delete_missing_target_certificates::delete_missing_target_list_clears_64000_siblings_on_two_mib ... ok
```

`.temp/director/audit-638/ignored-release.log:293`
```text
test tests::delete_missing_target_certificates::delete_missing_target_association_clears_64000_siblings_on_two_mib ... ok
```

`.temp/director/audit-638/ignored-release.log:294`
```text
test tests::iterative_traversal_depth::construction_reaches_physical_height_20001_on_two_mib ... ok
```

`.temp/director/audit-638/ignored-release.log:295`
```text
test tests::association_key_reshape_depth::association_key_deflatten_flatten_round_trip_is_exact_at_physical_height_4097 ... ok
```

`.temp/director/audit-638/ignored-release.log:296`
```text
test tests::association_key_reshape_depth::association_key_flatten_completes_at_physical_height_4097_on_two_mib ... ok
```

`.temp/director/audit-638/ignored-release.log:297`
```text
test tests::iterative_traversal_depth::level_completes_at_physical_height_4097_on_two_mib ... ok
```

`.temp/director/audit-638/ignored-release.log:298`
```text
test tests::iterative_traversal_depth::map_completes_at_physical_height_4097_on_two_mib ... ok
```

`.temp/director/audit-638/ignored-release.log:299`
```text
test tests::iterative_traversal_depth::matchq_completes_at_physical_height_20001_on_two_mib ... ok
```

`.temp/director/audit-638/ignored-release.log:300`
```text
test tests::iterative_traversal_depth::deletecases_active_spine_returns_in_band_resource_error_on_two_mib ... ok
```

`.temp/director/audit-638/ignored-release.log:301`
```text
test tests::iterative_traversal_depth::merge_nested_completes_at_physical_height_4097_on_two_mib ... ok
```

`.temp/director/audit-638/ignored-release.log:302`
```text
test tests::iterative_traversal_depth::count_completes_at_physical_height_4097_on_two_mib ... ok
```

`.temp/director/audit-638/ignored-release.log:303`
```text
test tests::iterative_traversal_depth::deletecases_completes_at_physical_height_4097_on_two_mib ... ok
```

`.temp/director/audit-638/ignored-release.log:304`
```text
test tests::iterative_traversal_depth::freeq_completes_at_physical_height_4097_on_two_mib ... ok
```

`.temp/director/audit-638/ignored-release.log:305`
```text
test tests::iterative_traversal_depth::memberq_completes_at_physical_height_4097_on_two_mib ... ok
```

`.temp/director/audit-638/ignored-release.log:306`
```text
test tests::iterative_traversal_depth::replacepart_deep_path_completes_at_physical_height_4097_on_two_mib ... ok
```

`.temp/director/audit-638/ignored-release.log:307`
```text
test tests::iterative_traversal_depth::rule_rhs_substitution_completes_at_physical_height_4097_on_two_mib ... ok
```

`.temp/director/audit-638/ignored-release.log:308`
```text
test tests::iterative_traversal_depth::take_multispec_completes_at_physical_height_4097_on_two_mib ... ok
```

`.temp/director/audit-638/ignored-release.log:309`
```text
test tests::iterative_traversal_depth::tostring_completes_at_physical_height_4097_on_two_mib ... ok
```

`.temp/director/audit-638/ignored-release.log:310`
```text
test tests::iterative_traversal_depth::with_substitution_completes_at_physical_height_4097_on_two_mib ... ok
```

`.temp/director/audit-638/ignored-release.log:311`
```text
test tests::map_indexed_association_depth::map_indexed_association_completes_at_physical_height_4097_on_two_mib ... ok
```

`.temp/director/audit-638/ignored-release.log:312`
```text
test tests::iterative_traversal_depth::position_completes_at_physical_height_4097_on_two_mib ... ok
```

`.temp/director/audit-638/ignored-release.log:313`
```text
test tests::map_indexed_association_depth::map_indexed_association_heads_refuses_in_band_rather_than_terminating ... ok
```

`.temp/director/audit-638/ignored-release.log:314`
```text
test tests::stress_corpus::peano_arithmetic_heavy_multiplication ... ok
```

`.temp/director/audit-638/ignored-release.log:315`
```text
test tests::stress_corpus::rule_based_differentiator_handles_heavy_generated_polynomial ... ok
```

`.temp/director/audit-638/ignored-release.log:316`
```text
test tests::retention_closure::local::contract_tests::regenerate_local_contract_fingerprints ... ok
```

`.temp/director/audit-638/ignored-release.log:317`
```text
test tests::stress_corpus::tokenize_with_ini_grammar_handles_heavy_generated_config ... ok
```

`.temp/director/audit-638/ignored-release.log:318`
```text
test tests::iterative_traversal_depth::cases_completes_at_physical_height_4097_on_two_mib ... ok
```

`.temp/director/audit-638/ignored-release.log:319`
```text
test tests::iterative_traversal_depth::replaceall_completes_at_physical_height_4097_on_two_mib ... ok
```

`.temp/director/audit-638/ignored-release.log:320`
```text
test tests::iterative_traversal_depth::replace_active_spine_returns_in_band_resource_error_on_two_mib ... ok
```

`.temp/director/audit-638/ignored-release.log:321`
```text
test tests::iterative_traversal_depth::replace_completes_at_physical_height_4097_on_two_mib ... ok
```

`.temp/director/audit-638/ignored-release.log:324`
```text
test tests::iterative_traversal_depth::d_nested_sin_completes_at_physical_height_4097_on_two_mib ... ok
```

`.temp/director/audit-638/ignored-release.log:325`
```text
test tests::delete_missing_depth::delete_missing_recursive_armed_completes_at_physical_height_4097_on_two_mib ... ok
```

`.temp/director/audit-638/ignored-release.log:327`
```text
test result: ok. 50 passed; 0 failed; 0 ignored; 0 measured; 1118 filtered out; finished in 504.68s
```

`.temp/director/audit-638/ignored-release.log:333`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:339`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 21 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:345`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 16 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:351`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 47 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:357`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 69 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:363`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 21 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:369`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 100 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:375`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 71 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:381`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 40 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:387`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 51 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:393`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 499 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:399`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:405`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:411`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 193 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:417`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:423`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:429`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:435`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:441`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:447`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:453`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 22 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:459`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:465`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:471`
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

`.temp/director/audit-638/ignored-release.log:475`
```text
# memory-scope: unit=audit638-release max=15032385536 peak=3873386496 oom=0 oom_kill=0 rc=101
```

`.temp/director/audit-638/profile-gate-release.log` (line citations precede each exact line):

`.temp/director/audit-638/profile-gate-release.log:27`
```text
test runtime::evaluation_cache::tests::call_stability_profile_reports_per_source_head_outcomes ... ok
```

`.temp/director/audit-638/profile-gate-release.log:29`
```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1245 filtered out; finished in 0.00s
```

`.temp/director/audit-638/profile-gate-release.log:31`
```text
# memory-scope: unit=audit639-profile max=15032385536 peak=1972015104 oom=0 oom_kill=0 rc=0
```
