# brief-592 — E1 drain 6 of 8: `debug`, `symbols`, `contexts` and `scoping` (77 owed entries)

Implementation brief, **test-only**. **Do not run `git` in any form**,
including before you have read this brief. Leave every change uncommitted for
the director's gate.

## Goal

brief-586 built the catalog-wide suspension oracle
(`crates/rum-stdlib/tests/suspension_oracle/`) and left **500** declared
guest-effect entries in an exact owed ledger (`inventory/owed.tsv`). Drains 1-5
(briefs 587-591) took nine families; twenty-seven declared roles that do not
exist were removed at the director's gate (owed 500 -> 186). **This one drains
77 entries**: exactly the IDs listed in
`.temp/brief-586/followups/draft-E1-{06-debug,21-symbols,04-contexts,15-scoping}.md`
— the bodies, options, filters, fallbacks and combiners of the timing,
resource, trace, monitor, test and `Reap`/`Sow` builtins; `Check`, `Quiet`,
`Message`'s template, the attribute lists of `SetAttributes` and
`ClearAttributes`, `OptionValue`'s delayed and scoped defaults, `SetOptions`,
`Unset`; the file forms and dependencies of `Get`, `Needs`, `BeginPackage`,
`DeclarePackage`, the options of `DumpSave` and `Names`, `Once`; and the
bodies, initializers and requests of `Block`, `Module`, `With`,
`DefinePackage`, `Dialog`, `Evaluate`, `ReleaseHold`,
`RestoreSealedReferences`, `Input` and `InputString`.

Each entry leaves the ledger **only in the same change that attaches and runs
its complete descriptor**: every required variant (`Middle`, `Twice`, and the
`Decline`, `Error`, `AlreadyResumed`, `Nested` variants its effect row
records), lifecycle at the first and second pause for every distinct
continuation owner (completion, cancellation, reset, a pending request
rejecting a new one), and a fresh request after.

Those drafts are the raw material; **this brief governs** where they differ.

## A failing descriptor is a finding, not a test to fix

If a correct descriptor shows the resumed path differing from the queued one —
a lost continuation the census did not see — **do not weaken the descriptor,
do not repair production code, and do not commit a failing test.** Keep the
entry owed, write the witness (source, answers, both observations) to
`.temp/brief-592/defects/<entry>.md`, and report it. It becomes a repair group
of this branch. **A role that does not exist** (an `Ordinary` builtin's option or operand
evaluated by shared preparation, a positional argument declared as an option,
option syntax admitted only literally) is reported with source evidence and
kept owed; the director removes it at the gate — briefs 587-591 found
twenty-seven, most of them `optionSyntax` on `Ordinary` builtins, where a
computed option expression is evaluated by shared preparation before the
handler sees it (a delayed rule's rhs *is* read locally: that is
`explicitOption`).
Check the registration (`Ordinary` or held) and the argument partition first.
**A wrong policy is not a reason to keep an entry owed**: the oracle's equality
does not depend on whether the entry is `Frame` or `Tail`, so attach the
descriptor, and report the correct policy for the director to set.

## Context pointers

- briefs 586-591 and their reports; the four drafts above; the descriptors
  under `fixtures/*_roles/` and `fixtures/rules/` as the model.
- `crates/rum-stdlib/tests/suspension_oracle/{manifest,descriptors,catalog}.rs`,
  `support/` (including `support/files.rs`, the in-memory file provider),
  `fixtures/`, `inventory/{catalog,effects,shared-effects,owed,historical-labels}.tsv`.
- The owning functions named in the drafts; the guide (`evaluation.md`,
  `scoping-and-contexts.md`, `embedding.md`) and the generated reference are
  the contract.
- The existing `sow_trace_continuations.rs` (Sow, Trace filters) and
  `guest_loading_resume.rs` (guest loading is resumable by ruling) cases and
  the historical labels, reusable as descriptors.
- Three cautions for this family. Timing results carry a nondeterministic
  duration: declare that leaf as normalized and compare everything else
  exactly (never a wall-time equality). File forms use the in-memory provider
  with an explicit capability profile, never the real filesystem.
  `Input`/`InputString`/`Dialog` are themselves the suspending builtins: their
  descriptors pause in the request and resume with the answer, at a middle and
  a second request.

## Scope

**Edit surface:** the `suspension_oracle` test tree; the existing
continuation suites only to share descriptors (test names and specialized
assertions stay).

**Standing duty — always in scope** (`TEMPLATE.md`; test edits under `crates/`
expire the fingerprints): fingerprints, both certificates, ownership
reconciliation, every row disclosed. **STOP** on any existing row change or
disappearance, any new `UnknownHostPayload` or `self_reaching` row, any new
row in untouched code.

**Fence:** no production `.rs`, manifest disposition, `lib.rs`, Cargo
manifest, lockfile or dependency change; no other family's entries; no edits
to `AGENTS.md`, `TODO.md`, root `PROGRESS.md`, earlier briefs, `scripts/` or
docs.

**Disclosure duty:** every entry drained, every entry kept owed and why, every
defect witness, every changed test name, every inventory row.

## Deliverable

Role-specific descriptor factories with explicit expected values, logs and
state projections and exact input kind, prompt and answer plans. A callback,
guard, option or held-target phase is exercised by its own role — ordinary
argument preparation never stands in. Loops run at least five operations and
pause in the middle and at a second position. Shared factories across the
family are welcome where they demonstrably exercise each entry's role.

## rum / Rust pitfalls (read twice)

1. Never make a pre-fix or resumed-only value the expected answer; expected
   values come from the documented contract and the queued path.
2. An eager-only or single-pause fixture proves nothing.
3. Support modules under 10k characters, scenario modules under 15k.
4. Stop-loss counters: entries drained, variants passing, lifecycle
   obligations passing, defects recorded. Stop after two consecutive compiling
   attempts that move none; fewer compiler errors is progress.

## Verification (MANDATORY — run to green or evidence)

Serial, through `python3 scripts/run-bounded.py --timeout <s> -- env
CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true
TMPDIR=<repo>/.temp/brief-592/tmp …`. Bounds: the oracle target 900 s per
build; runtime, ownership and retention 300 s; `cargo test --workspace
--no-fail-fast` 3 000 s.

```
cargo fmt --check
scripts/check-suspension-oracle.sh 900
cargo test -p rum-runtime
cargo test -p rum-stdlib --lib ownership_graph
cargo test -p rum-stdlib --lib local_production_acceptance
cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance
cargo test --workspace --no-fail-fast
```

The oracle's `CATALOG` lines must show `owed` reduced by exactly the drained
count in both builds. Report native ZeroMQ skips as skips.

## Checkpointing — protect the scene

Everything under `.temp/brief-592/`; `PROGRESS.md` after each milestone with
the counters and the exact next action.

## Hard rules

- **No `git`, in any form.** No network; nothing outside this repository
  (except the legacy certificate output `.temp/brief-546/certificates/`).
- Commands: scoped reads and edits, `rg`/`grep`, `python3`, bounded serial
  Cargo, `scripts/check-suspension-oracle.sh`. No probe that aborts a process.
- **Testing Policy is a hard line** (`AGENTS.md`): rum against rum only. No
  new `unsafe`.
- Paste real output; say plainly what did not run or did not pass.

## Report format (append to this brief)

1. Files changed; the no-production-edit audit.
2. Entries drained and kept owed (with reasons); owed before and after, per
   build; variants and lifecycle obligations added.
3. Defect witnesses, if any; the broad suites verbatim; every inventory row
   change.
4. Stop-loss counters, open questions.

---

## Execution report — brief-592

### 1. Files changed and fence audit

Test-only drain: **58 of 77 entries**, attached through **75 descriptors**. The other 19 remain owed with individual witnesses. No git command or network request was used. No production implementation, manifest disposition, `lib.rs`, Cargo manifest, lockfile, dependency, script, AGENTS, TODO, root PROGRESS or earlier brief was changed. The source hash audit starts from the scene captured before edits; it preserves pre-existing changes without relying on git.

Changed files (repository-relative):

| File | Purpose |
|---|---|
| `crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv` | Replay all 68 generated fingerprint cells; contract facts unchanged. |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv` | Replay only C01/C02 hash=true/hash=false certificate cells. |
| `crates/rum-stdlib/tests/suspension_oracle/descriptors.rs` | Duration paths, in-memory files and owner-pause selection on fixtures. |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/debug_roles/audit.rs` | Execute every retained variant and owner lifecycle; print raw observations on failure. |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/debug_roles/bodies.rs` | Timing/resource/monitor/reap/sow and generated echo callback roles. |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/debug_roles/contexts.rs` | In-memory file forms, five dependencies, Once caching, and context/package projections. |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/debug_roles/falsifiers.rs` | Missing-pause/lost-log red controls and duration normalization guards. |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/debug_roles/lifecycle.rs` | First/second owner pauses, cancellation/reset/pending rejection, cleanup and fresh requests. |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/debug_roles/mod.rs` | Wire the grouped test modules. |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/debug_roles/options.rs` | Role-specific local option reads and held option syntax. |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/debug_roles/phases.rs` | Seventeen additional retained owners/stages under the same catalog entries. |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/debug_roles/reports.rs` | TestReport local default/explicit option reads in all three stages. |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/debug_roles/scoping.rs` | Scope bodies/initializers, package registration, dialogs and input answer/request phases. |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/debug_roles/support.rs` | Five-operation plans, exact prompts/answers, errors, logs and state projections. |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/debug_roles/symbols.rs` | Check/Quiet, delayed templates/options, held attribute and Unset/SetOptions phases. |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/debug_roles/trace.rs` | Trace bodies and filter alternatives with literal trace expectations. |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/mod.rs` | Wire the grouped test modules. |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/primary.rs` | Attach the new descriptor registry. |
| `crates/rum-stdlib/tests/suspension_oracle/inventory/effects.tsv` | Only descriptor attachment cells change, for the 58 drained entries. |
| `crates/rum-stdlib/tests/suspension_oracle/inventory/owed.tsv` | Remove exactly those same 58 rows. |
| `crates/rum-stdlib/tests/suspension_oracle/support/files.rs` | In-memory provider and explicit capability profile; no real filesystem fixtures. |
| `crates/rum-stdlib/tests/suspension_oracle/support/lifecycle.rs` | First/second owner pauses, cancellation/reset/pending rejection, cleanup and fresh requests. |
| `crates/rum-stdlib/tests/suspension_oracle/support/mod.rs` | Wire the grouped test modules. |
| `crates/rum-stdlib/tests/suspension_oracle/support/normalize.rs` | Validate and normalize only explicitly declared exact numeric duration leaves. |
| `docs/dev/codex/brief-592-oracle-drain-debug.md` | Append this report. |

No existing test name was changed or removed. Three tests were added:

- `oracle::fixtures::debug_roles::audit::complete_role_descriptors`
- `oracle::fixtures::debug_roles::falsifiers::every_role_rejects_missing_pause_and_lost_log`
- `oracle::fixtures::debug_roles::falsifiers::duration_normalization_preserves_value_and_rejects_non_numeric_leaves`

The temporary diagnostic test `oracle::fixtures::debug_roles::snapshot_probe::restore_boundary_witness` was moved to [restore-boundary-probe.rs](../../../.temp/brief-592/restore-boundary-probe.rs); it is not a retained test. Existing continuation-suite source files are byte-identical; all 209 historical oracle labels remain. All added support/scenario modules are under 10k/15k characters respectively. The four families share a `debug_roles/` directory with separate modules for their scenarios. No hot-path change was made, so no performance A/B gate applies. [logs/final-audit.txt](../../../.temp/brief-592/logs/final-audit.txt)

### 2. Drained and kept-owed entries

Both builds started at **186 owed / 328 descriptors / 1,944 lifecycle obligations**. Both end at **128 owed / 403 descriptors / 2,394 lifecycle obligations**: exactly 58 drained, with **375 required variants and 450 lifecycle obligations added per build** (750 variants and 900 obligations across the two builds). Completion and a fresh request are checked for every variant. Each descriptor also has strict missing-pause and lost-log falsifiers, weak controls and a passing good control: 75 descriptor pairs per build.

Every retained descriptor has `Middle,Twice,Error,AlreadyResumed,Nested`; none of the drained rows requires Decline. The two Decline-bearing test roles remain owed. First/second lifecycle pauses are selected within the parsed-answer/body owner when an earlier request precedes it. Five operations are used; RepeatedTiming preserves its documented three samples, each with five operations. Only its elapsed-time leaf (and Timing/AbsoluteTiming’s corresponding leaf) is normalized. Trace structure, typed values, error/termination, messages, prints, logs and state projections remain exact.

`Unevaluated` is used deliberately for `SetAttributes`, `ClearAttributes`, `SetOptions`, and ordinary runtime option values: shared preparation releases the raw operand to the handler, whose explicit local evaluation then owns the pause. This is distinct from an absent optionSyntax role that only parses already-resolved rules. Runtime option readers keep RuleDelayed held and evaluate ordinary values; the fixtures store unresolved ordinary defaults before defining their value reads. Trace’s default boolean reader instead evaluates its delayed default. Generated EchoFunction application is reported as a Tail policy finding.

Every drained entry (this also enumerates every changed effects attachment and removed owed row):

| Family | Entry | Attached descriptor IDs |
|---|---|---|
| contexts | `BeginPackage.dependency` | `BeginPackage.dependency` |
| contexts | `BeginPackage.fileForms` | `BeginPackage.fileForms` |
| contexts | `DeclarePackage.fileForms` | `DeclarePackage.fileForms` |
| contexts | `DumpSave.defaultOption` | `DumpSave.defaultOption` |
| contexts | `DumpSave.explicitOption` | `DumpSave.explicitOption` |
| contexts | `Get.fileForms` | `Get.fileForms` |
| contexts | `Names.defaultOption` | `Names.defaultOption`, `Names.defaultOption.aliases`, `Names.defaultOption.ignoreCase` |
| contexts | `Names.explicitOption` | `Names.explicitOption`, `Names.explicitOption.aliases`, `Names.explicitOption.ignoreCase` |
| contexts | `Needs.fileForms` | `Needs.fileForms` |
| contexts | `Once.body` | `Once.body` |
| debug | `AbsoluteTiming.body` | `AbsoluteTiming.body` |
| debug | `EchoFunction.callback` | `EchoFunction.callback` |
| debug | `MemoryConstrained.body` | `MemoryConstrained.body` |
| debug | `MemoryConstrained.fallback` | `MemoryConstrained.fallback` |
| debug | `Monitor.body` | `Monitor.body` |
| debug | `Monitor.defaultOption` | `Monitor.defaultOption`, `Monitor.defaultOption.step` |
| debug | `Monitor.explicitOption` | `Monitor.explicitOption`, `Monitor.explicitOption.step` |
| debug | `Monitor.optionSyntax` | `Monitor.optionSyntax` |
| debug | `Reap.body` | `Reap.body` |
| debug | `Reap.combiner` | `Reap.combiner`, `Reap.combiner.groups` |
| debug | `RepeatedTiming.body` | `RepeatedTiming.body` |
| debug | `Sow.recipientPattern` | `Sow.recipientPattern`, `Sow.recipientPattern.scopes` |
| debug | `TestReport.defaultOption` | `TestReport.defaultOption`, `TestReport.defaultOption.memory`, `TestReport.defaultOption.time` |
| debug | `TestReport.explicitOption` | `TestReport.explicitOption`, `TestReport.explicitOption.memory`, `TestReport.explicitOption.time` |
| debug | `TimeConstrained.body` | `TimeConstrained.body` |
| debug | `TimeConstrained.fallback` | `TimeConstrained.fallback` |
| debug | `Timing.body` | `Timing.body` |
| debug | `Trace.body` | `Trace.body` |
| debug | `Trace.defaultOption` | `Trace.defaultOption` |
| debug | `Trace.filter` | `Trace.filter` |
| debug | `TraceDetail.body` | `TraceDetail.body` |
| debug | `TraceDetail.defaultOption` | `TraceDetail.defaultOption` |
| debug | `TraceDetail.filter` | `TraceDetail.filter` |
| debug | `TracePrint.body` | `TracePrint.body` |
| debug | `TracePrint.defaultOption` | `TracePrint.defaultOption` |
| debug | `TracePrint.filter` | `TracePrint.filter` |
| scoping | `Block.body` | `Block.body`, `Block.body.single` |
| scoping | `Block.initializers` | `Block.initializers`, `Block.initializers.single` |
| scoping | `DefinePackage.body` | `DefinePackage.body` |
| scoping | `Dialog.body` | `Dialog.body` |
| scoping | `Evaluate.body` | `Evaluate.body` |
| scoping | `Input.parsedAnswer` | `Input.parsedAnswer` |
| scoping | `Input.request` | `Input.request` |
| scoping | `InputString.request` | `InputString.request` |
| scoping | `Module.body` | `Module.body`, `Module.body.afterInitializer` |
| scoping | `Module.initializers` | `Module.initializers` |
| scoping | `ReleaseHold.body` | `ReleaseHold.body` |
| scoping | `With.body` | `With.body` |
| scoping | `With.initializers` | `With.initializers` |
| symbols | `Check.body` | `Check.body` |
| symbols | `ClearAttributes.attributes` | `ClearAttributes.attributes` |
| symbols | `Message.template` | `Message.template` |
| symbols | `OptionValue.delayedValue` | `OptionValue.delayedValue`, `OptionValue.delayedValue.list` |
| symbols | `OptionValue.scopedDefault` | `OptionValue.scopedDefault`, `OptionValue.scopedDefault.list` |
| symbols | `Quiet.body` | `Quiet.body` |
| symbols | `SetAttributes.attributes` | `SetAttributes.attributes` |
| symbols | `SetOptions.optionSyntax` | `SetOptions.optionSyntax` |
| symbols | `Unset.target` | `Unset.target` |

Drained by family: contexts **10**, debug **26**, scoping **13**, symbols **9**. The other 109 pre-existing owed entries are unchanged. Full executed mapping: [completed-descriptors.tsv](../../../.temp/brief-592/completed-descriptors.tsv).

Every requested entry kept owed:

| Entry | Reason | Witness |
|---|---|---|
| `DefinePackage.bindings` | NoLocalGuestEffect: The held declaration and binding scan is structural; it collects target symbols and rewrites the body. Guest evaluation starts only in resume_package_install, covered by DefinePackage.body. | [source / observations](../../../.temp/brief-592/defects/DefinePackage.bindings.md) |
| `DumpSave.optionSyntax` | NoLocalGuestEffect: DumpSave holds its option argument, but compile_assigned_options_from_resolved accepts literal rule/list syntax and does not evaluate a computed option expression. CompressionLevel value reads are separate covered roles. | [source / observations](../../../.temp/brief-592/defects/DumpSave.optionSyntax.md) |
| `Names.optionSyntax` | NoLocalGuestEffect: Names is Ordinary with no hold attribute. Shared preparation owns a computed option expression; compile_names_options parses an already-resolved literal rule/list. Local value reads are separately exercised via Unevaluated literal rules. | [source / observations](../../../.temp/brief-592/defects/Names.optionSyntax.md) |
| `Reap.pattern` | NoLocalGuestEffect: Reap holds only its body. Shared preparation resolves the pattern argument, then normalize_reap_patterns only partitions it. Recipient matching is performed by Sow, covered by Sow.recipientPattern. | [source / observations](../../../.temp/brief-592/defects/Reap.pattern.md) |
| `TestCreate.expectedOutput` | NoLocalGuestEffect: TestCreate constructs a symbolic test specification; Input/ExpectedOutput are held data and SameTest is stored, never invoked here. The corresponding evaluation belongs to TestReport, whose defects are recorded separately. | [source / observations](../../../.temp/brief-592/defects/TestCreate.expectedOutput.md) |
| `TestCreate.input` | NoLocalGuestEffect: TestCreate constructs a symbolic test specification; Input/ExpectedOutput are held data and SameTest is stored, never invoked here. The corresponding evaluation belongs to TestReport, whose defects are recorded separately. | [source / observations](../../../.temp/brief-592/defects/TestCreate.input.md) |
| `TestCreate.sameTest` | NoLocalGuestEffect: TestCreate constructs a symbolic test specification; Input/ExpectedOutput are held data and SameTest is stored, never invoked here. The corresponding evaluation belongs to TestReport, whose defects are recorded separately. | [source / observations](../../../.temp/brief-592/defects/TestCreate.sameTest.md) |
| `TestReport.optionSyntax` | NoLocalGuestEffect: TestReport is unheld Ordinary. Its argument partition and parse_test_report_options parse already-resolved rules; shared preparation owns computed syntax. Local immediate value reads are separately covered. | [source / observations](../../../.temp/brief-592/defects/TestReport.optionSyntax.md) |
| `Trace.explicitOption` | NoLocalGuestEffect: This HoldAll builtin reads its default Heads through system_option_bool, then accepts only literal Heads -> True\|False (or the literal default form). The explicit option is not locally evaluated; parse_heads_option[_with_default] is structural. | [source / observations](../../../.temp/brief-592/defects/Trace.explicitOption.md) |
| `Trace.optionSyntax` | NoLocalGuestEffect: This HoldAll builtin reads its default Heads through system_option_bool, then accepts only literal Heads -> True\|False (or the literal default form). The explicit option is not locally evaluated; parse_heads_option[_with_default] is structural. | [source / observations](../../../.temp/brief-592/defects/Trace.optionSyntax.md) |
| `TraceDetail.explicitOption` | NoLocalGuestEffect: This HoldAll builtin reads its default Heads through system_option_bool, then accepts only literal Heads -> True\|False (or the literal default form). The explicit option is not locally evaluated; parse_heads_option[_with_default] is structural. | [source / observations](../../../.temp/brief-592/defects/TraceDetail.explicitOption.md) |
| `TraceDetail.optionSyntax` | NoLocalGuestEffect: This HoldAll builtin reads its default Heads through system_option_bool, then accepts only literal Heads -> True\|False (or the literal default form). The explicit option is not locally evaluated; parse_heads_option[_with_default] is structural. | [source / observations](../../../.temp/brief-592/defects/TraceDetail.optionSyntax.md) |
| `TracePrint.explicitOption` | NoLocalGuestEffect: This HoldAll builtin reads its default Heads through system_option_bool, then accepts only literal Heads -> True\|False (or the literal default form). The explicit option is not locally evaluated; parse_heads_option[_with_default] is structural. | [source / observations](../../../.temp/brief-592/defects/TracePrint.explicitOption.md) |
| `TracePrint.optionSyntax` | NoLocalGuestEffect: This HoldAll builtin reads its default Heads through system_option_bool, then accepts only literal Heads -> True\|False (or the literal default form). The explicit option is not locally evaluated; parse_heads_option[_with_default] is structural. | [source / observations](../../../.temp/brief-592/defects/TracePrint.optionSyntax.md) |
| `EvaluationData.body` | Defect: Queued/resumed discrepancy; see exact witness. | [source / observations](../../../.temp/brief-592/defects/EvaluationData.body.md) |
| `TestReport.input` | Defect: Queued/resumed discrepancy; see exact witness. | [source / observations](../../../.temp/brief-592/defects/TestReport.input.md) |
| `TestReport.expectedOutput` | Defect: Queued/resumed discrepancy; see exact witness. | [source / observations](../../../.temp/brief-592/defects/TestReport.expectedOutput.md) |
| `TestReport.sameTest` | Defect: Queued/resumed discrepancy; see exact witness. | [source / observations](../../../.temp/brief-592/defects/TestReport.sameTest.md) |
| `RestoreSealedReferences.body` | DocumentedBoundary: Only snapshot restore admits the role; that public request is non-resumable. | [source / observations](../../../.temp/brief-592/defects/RestoreSealedReferences.body.md) |

Fourteen rows have no local guest phase; only the director may remove those declarations. `RestoreSealedReferences.body` is a separate documented snapshot boundary: public snapshot restore is not resumable through the single-source input API, and host restore retires input as a terminal failure. It stays owed for a disposition decision, not a production repair.

Policy findings (no disposition/policy cell edited): `With.body` is Tail after lexical substitution; `EchoFunction.callback` is a returned/generated callback with no retained EchoFunction frame (Tail). Both have complete attached descriptors. [policy-findings.tsv](../../../.temp/brief-592/policy-findings.tsv)

### 3. Defect witnesses, verification and every inventory change

Four entries remain owed because correct descriptors exposed failures:

- `EvaluationData.body`: the resume frame loses the requested property projection; capture completion also removes prints already exposed at the pause.
- `TestReport.input`: completing a captured case removes its surfaced print prefix; suspending in a second case also reaches the accumulator panic.
- `TestReport.expectedOutput` and `TestReport.sameTest`: after two distinct test cases suspend, resumed result finalization panics at `debug_tools/test_report/report.rs:116` (`completed TestReport result state must be unique`); the queued run completes correctly.

Each witness records source, answer plan and queued/resumed observations. Failed candidate factories are archived under this brief; no failing descriptor is attached or left active. No production repair or weakened continuation/prefix contract was used.

All Cargo commands ran serially, bounded, offline, with `CARGO_BUILD_JOBS=1`, `RUST_TEST_THREADS=1` and this brief’s TMPDIR. Oracle builds were bounded at 900 seconds each; runtime/ownership/retention at 300; workspace at 3,000. The canonical oracle script’s hardcoded scratch path temporarily resolved into this brief, then its original directory was restored intact. [oracle-script-paths.txt](../../../.temp/brief-592/oracle-script-paths.txt)

`cargo fmt --check`: exit 0 (no output). [logs/fmt.txt](../../../.temp/brief-592/logs/fmt.txt)

Before catalogs (verbatim):
```text
CATALOG hash=true installed=643 manifest=643 GuestEntries=238 NoLocalGuestEffect=405 Boundary=4 entries=479 descriptors=328 historical=209 owed=186
CATALOG hash=false installed=637 manifest=637 GuestEntries=238 NoLocalGuestEffect=399 Boundary=4 entries=479 descriptors=328 historical=209 owed=186
```

`scripts/check-suspension-oracle.sh 900` (final, verbatim):
```text
LIFECYCLE obligations=2394 (plus completion for every variant)
CATALOG hash=true installed=643 manifest=643 GuestEntries=238 NoLocalGuestEffect=405 Boundary=4 entries=479 descriptors=403 historical=209 owed=128
test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 19.72s
LIFECYCLE obligations=2394 (plus completion for every variant)
CATALOG hash=false installed=637 manifest=637 GuestEntries=238 NoLocalGuestEffect=399 Boundary=4 entries=479 descriptors=403 historical=209 owed=128
test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 19.96s
```

[logs/oracle-gate.txt](../../../.temp/brief-592/logs/oracle-gate.txt)

`cargo test -p rum-runtime` (verbatim):
```text
test result: ok. 1139 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 28.05s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.11s
```
[logs/runtime.txt](../../../.temp/brief-592/logs/runtime.txt)

`cargo test -p rum-stdlib --lib ownership_graph -- --nocapture` (verbatim):
```text
test tests::ownership_graph::production_inventory_is_exact ... EXACT INVENTORY self-reaching types: 59 rows
EXACT INVENTORY UnknownHostPayload: 65 rows
EXACT INVENTORY UnknownConstruct: 7683 rows
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1120 filtered out; finished in 60.35s
```
[logs/ownership.txt](../../../.temp/brief-592/logs/ownership.txt)

`cargo test -p rum-stdlib --lib regenerate_local_contract_fingerprints -- --ignored --nocapture` (verbatim):
```text
test tests::retention_closure::local::contract_tests::regenerate_local_contract_fingerprints ... Regenerated 68 fingerprints at /home/remi/dev/rum/target/brief-544/local-contracts.tsv
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1139 filtered out; finished in 0.20s
```
[logs/fingerprints.txt](../../../.temp/brief-592/logs/fingerprints.txt)

The first certificate run in each build failed only on the expected stale C01 certificate and wrote replacement cells. Those real failures are preserved:
```text
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=true"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1139 filtered out; finished in 18.03s
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=false"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1133 filtered out; finished in 17.86s
```

`cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture` (final, verbatim):
```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1139 filtered out; finished in 17.81s
```
[retention-default.txt](../../../.temp/brief-592/retention-default.txt)

`cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture` (final, verbatim):
```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1133 filtered out; finished in 17.80s
```
[retention-no-default.txt](../../../.temp/brief-592/retention-no-default.txt)

All seven ownership inventories are byte-identical. No existing row changed/disappeared, and no new UnknownHostPayload, self-reaching or untouched-code row was introduced. [ownership-reconciliation.tsv](../../../.temp/brief-592/ownership-reconciliation.tsv)

All **68** local-contract rows change only the final generated fingerprint cell. Identities, contract text, statuses, strengths and exclusions are unchanged. Every row and its exact before/after fingerprint:

| ID | Before | After |
|---|---|---|
| `B01` | `fnv1a64:09fb729acdd30088` | `fnv1a64:7f4bb8a77e20a25c` |
| `B02` | `fnv1a64:082dfbc3732d4395` | `fnv1a64:c25ecb2859b75d81` |
| `B03` | `fnv1a64:f60d7de4965599ea` | `fnv1a64:ddbf7b6583592db2` |
| `B04` | `fnv1a64:a283051fa0877270` | `fnv1a64:daf8bd876007a6a4` |
| `B05` | `fnv1a64:399b7e512c527cc9` | `fnv1a64:1d4c40a7bfecadd5` |
| `B06` | `fnv1a64:03cc0df4a72a0874` | `fnv1a64:7cd1a8a4c21a1d78` |
| `B07` | `fnv1a64:e0792374202cc579` | `fnv1a64:ede55fc648cf3b65` |
| `B08` | `fnv1a64:bb50920644d87b4e` | `fnv1a64:288b1d74598f20a6` |
| `B09` | `fnv1a64:7e36f34437ff66dc` | `fnv1a64:d37e9397372c7860` |
| `B10` | `fnv1a64:d881046ea526c757` | `fnv1a64:011087ebb8badd07` |
| `B11` | `fnv1a64:63dab67cba7b9870` | `fnv1a64:9c506ee479fbcca4` |
| `B12` | `fnv1a64:3d4373401407fbdc` | `fnv1a64:928b139313350d60` |
| `I01` | `fnv1a64:e7dd5d983a8af708` | `fnv1a64:5d2da3a4ead898dc` |
| `I02` | `fnv1a64:90dcfbe5248904c9` | `fnv1a64:748dbe3bb82335d5` |
| `I03` | `fnv1a64:7c62421b73cb461c` | `fnv1a64:45f1a98320475620` |
| `I04` | `fnv1a64:a929287ba98af7b5` | `fnv1a64:83a87664ac77bfa1` |
| `P01` | `fnv1a64:36de32a0ffc4d914` | `fnv1a64:5e376a346c5b5818` |
| `P02` | `fnv1a64:61216bf135187aa7` | `fnv1a64:62b31e811a173fb7` |
| `P03` | `fnv1a64:981794e59a972782` | `fnv1a64:3d6280111d50278a` |
| `P04` | `fnv1a64:fd0919d8f7d65f95` | `fnv1a64:b739e93dde607981` |
| `P05` | `fnv1a64:3f36ee6a33cdab18` | `fnv1a64:b18711cfc50b960c` |
| `P06` | `fnv1a64:8dc29810a037955c` | `fnv1a64:e30a38639f64a6e0` |
| `P070` | `fnv1a64:c81034527ead0f5c` | `fnv1a64:1d57d4a57dda20e0` |
| `P080` | `fnv1a64:76d786290fa68097` | `fnv1a64:71cfe90186e0e3c7` |
| `P090` | `fnv1a64:36590fbe2087f4d5` | `fnv1a64:f1c250f94c446241` |
| `P100` | `fnv1a64:ba3e5f9c49311aa0` | `fnv1a64:474a50765e9f6d34` |
| `P110` | `fnv1a64:4d215c5d45198baf` | `fnv1a64:d61f349130b4007f` |
| `P120` | `fnv1a64:a1c509aabc14df6e` | `fnv1a64:202478035b7ecdc6` |
| `P130` | `fnv1a64:84467cd14ee09f79` | `fnv1a64:91b2b92377831565` |
| `P140` | `fnv1a64:fd4ab4c6d4ca106d` | `fnv1a64:14e653cffe8b44f9` |
| `P150` | `fnv1a64:0f2ddcb398966ff5` | `fnv1a64:7c5b9e214862e661` |
| `P071` | `fnv1a64:c407e34f4219b399` | `fnv1a64:388b89f2e9d41f05` |
| `P081` | `fnv1a64:09c0880088294a7c` | `fnv1a64:074a6004f8db2180` |
| `P091` | `fnv1a64:1926204358fd9554` | `fnv1a64:7d758eb184554fd8` |
| `P101` | `fnv1a64:45244d2a7a7d28af` | `fnv1a64:ce22255e66179d7f` |
| `P111` | `fnv1a64:480315bbec42898c` | `fnv1a64:60c23dd2f9c595f0` |
| `P121` | `fnv1a64:70336fa687d408c3` | `fnv1a64:3e8761c9846ff333` |
| `P131` | `fnv1a64:402d2785d6db93f0` | `fnv1a64:78a2dfed965bc824` |
| `P141` | `fnv1a64:9ce8379ec8fc86b4` | `fnv1a64:b4f51b69a74afd38` |
| `P151` | `fnv1a64:36eec4d2766ff72e` | `fnv1a64:bc16b60ccc8f7506` |
| `B13` | `fnv1a64:91dca17dd7c6c474` | `fnv1a64:0ae23c2df2b6d978` |
| `R01` | `fnv1a64:53dd7e4b09e1ba85` | `fnv1a64:108df6b2e32de791` |
| `R02` | `fnv1a64:0576f8d275a848c2` | `fnv1a64:cc00bd9b20bd984a` |
| `R03` | `fnv1a64:4c9527d848059aa1` | `fnv1a64:661639b9ee752ecd` |
| `R040` | `fnv1a64:e9c3155707476c25` | `fnv1a64:7d78c2d5e2dd2931` |
| `R050` | `fnv1a64:08dfff674945ad78` | `fnv1a64:47f4f6e079ccd06c` |
| `R060` | `fnv1a64:e7377f00d149d6dd` | `fnv1a64:2ef510362ea734c9` |
| `R070` | `fnv1a64:722e501a69cb097c` | `fnv1a64:6fb8281eda7ce080` |
| `R080` | `fnv1a64:702d916e790fc0b5` | `fnv1a64:4aacdf577bfc88a1` |
| `R090` | `fnv1a64:3b8e91f6a16499e0` | `fnv1a64:8afdbe398b3dd5f4` |
| `R100` | `fnv1a64:bb332237402b446a` | `fnv1a64:a2e51fb82d2ed832` |
| `R110` | `fnv1a64:06aa4c9abae6695f` | `fnv1a64:4c021de658b4e18f` |
| `R041` | `fnv1a64:781604a7eeee8374` | `fnv1a64:f11b9f5809de9878` |
| `R051` | `fnv1a64:c47b3dd3adb60155` | `fnv1a64:7fe47f0ed9726ec1` |
| `R061` | `fnv1a64:2af9ecabb733ed9e` | `fnv1a64:cdd75898cddf61d6` |
| `R071` | `fnv1a64:16316f0e90cabc2b` | `fnv1a64:b1a0530d99a2c99b` |
| `R081` | `fnv1a64:7628c8f0e4dcefa0` | `fnv1a64:0334b9cafa4b4234` |
| `R091` | `fnv1a64:7a3c7df38df6602d` | `fnv1a64:5619e608c1efef39` |
| `R101` | `fnv1a64:de7e635956fec61d` | `fnv1a64:f318568bd0338489` |
| `R111` | `fnv1a64:0cf6399215f46476` | `fnv1a64:133aad1676a7c7ee` |
| `R12` | `fnv1a64:0a9551df9b209936` | `fnv1a64:b219cee4b2a5ad2e` |
| `R13` | `fnv1a64:04613db045fda5ba` | `fnv1a64:8926387d8f56d1a2` |
| `R14` | `fnv1a64:d4966e1225e1daa0` | `fnv1a64:61a25eec3b502d34` |
| `R15` | `fnv1a64:ffc61c145e5a0dc4` | `fnv1a64:ea36271c7034eda8` |
| `O010` | `fnv1a64:d7f588cba8e18e7e` | `fnv1a64:ec77bfee3b29c536` |
| `O0200` | `fnv1a64:f812569b2381366d` | `fnv1a64:0fadf5a44d426af9` |
| `O011` | `fnv1a64:29c03b88dedb9b5d` | `fnv1a64:717dccbe3c38f949` |
| `O0210` | `fnv1a64:8f7fa35a76a16f16` | `fnv1a64:1397d9c4b823250e` |

Only the two build-specific certificate cells in promoted rows **C01** and **C02** change. Replay guards prove all differences are direct-contract fingerprints: four per C01 cell and seven per C02 cell. Instance identities, body/domain fingerprints, source positions, marks, preconditions, strengths and exclusions remain unchanged. Every full before/after cell is in [certificate-changes.tsv](../../../.temp/brief-592/certificate-changes.tsv). No other inventory row changed. The 186 changed rows are 58 effects attachments, 58 owed removals, 68 local contracts, and two promoted rows. The exhaustive raw row disclosure is [all-inventory-changes.tsv](../../../.temp/brief-592/all-inventory-changes.tsv).

`cargo test --workspace --no-fail-fast -- --nocapture` (verbatim suite results):
```text
     Running unittests src/main.rs (target/debug/deps/rum_cli-675a9b7fb4c09027)
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
     Running tests/cli.rs (target/debug/deps/cli-417da8e232f3f31f)
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
     Running tests/input_requests.rs (target/debug/deps/input_requests-79d242612dc5614e)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
     Running tests/terminal.rs (target/debug/deps/terminal-cd41974a9dfe37a5)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.90s
     Running unittests src/lib.rs (target/debug/deps/rum_core-49fc2a810a19f202)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_derive-b50628eb478e3209)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/accumulator_chain.rs (target/debug/deps/accumulator_chain-f4b1762065518378)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/derive.rs (target/debug/deps/derive-b38e62f915ec5c7d)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_host-a119f062096badf7)
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.65s
     Running tests/adapter_contract.rs (target/debug/deps/adapter_contract-36e2ac93dcf516de)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
     Running tests/batch_boundary_ownership.rs (target/debug/deps/batch_boundary_ownership-62cec20e1429084a)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
     Running tests/guest_loading_resume.rs (target/debug/deps/guest_loading_resume-4d8893fc9e0cdd96)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
     Running tests/host_payload_destruction_contract.rs (target/debug/deps/host_payload_destruction_contract-1be17d3af592a343)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.14s
     Running tests/installer.rs (target/debug/deps/installer-8ebb99c00d41e78e)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
     Running tests/lifecycle_buildout.rs (target/debug/deps/lifecycle_buildout-5e85c48ede3e9593)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
     Running tests/lifecycle_contract.rs (target/debug/deps/lifecycle_contract-a2c563982853655a)
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
     Running tests/random_password_policy.rs (target/debug/deps/random_password_policy-ddd2fec076417963)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
     Running tests/stress.rs (target/debug/deps/stress-5ec2a75152f1305e)
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.30s
     Running tests/suspended_timeout_contract.rs (target/debug/deps/suspended_timeout_contract-258c488f1b37cde7)
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.06s
     Running tests/suspension_abandonment_contract.rs (target/debug/deps/suspension_abandonment_contract-ac93b53b9ae0d1d1)
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
     Running unittests src/lib.rs (target/debug/deps/rum_jupyter-edb9ee3aea58135e)
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.74s
     Running unittests src/bin/rum-jupyter-kernel.rs (target/debug/deps/rum_jupyter_kernel-965588c9ac9ffa23)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_repl-edcfd3382d4bfc76)
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.38s
     Running tests/input_abandonment.rs (target/debug/deps/input_abandonment-144216c549618286)
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
     Running tests/input_continuation.rs (target/debug/deps/input_continuation-ebdbc4f7476ef252)
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
     Running unittests src/lib.rs (target/debug/deps/rum_runtime-562b258c6eef9b02)
test result: ok. 1139 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 27.86s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-c8c5a89643371dc7)
test result: ok. 1087 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 1027.91s
     Running unittests src/bin/generate-builtin-reference.rs (target/debug/deps/generate_builtin_reference-7e0be655aec7b381)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/default_fallback_continuations.rs (target/debug/deps/default_fallback_continuations-c846b88aba46e50a)
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
     Running tests/destructuring_continuations.rs (target/debug/deps/destructuring_continuations-dfe1523f84cb69ee)
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
     Running tests/guest_loading_resume.rs (target/debug/deps/guest_loading_resume-0992a185a5c665db)
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.06s
     Running tests/lazy_pull_continuations.rs (target/debug/deps/lazy_pull_continuations-4b49946100b6004b)
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.54s
     Running tests/loop_continuations.rs (target/debug/deps/loop_continuations-d0d938f953da2f0b)
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
     Running tests/option_reader_continuations.rs (target/debug/deps/option_reader_continuations-5a0cec5f31c0a5b0)
test result: ok. 100 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.96s
     Running tests/rule_pattern_continuations.rs (target/debug/deps/rule_pattern_continuations-c61abf30d2dd2405)
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.57s
     Running tests/schema_continuations.rs (target/debug/deps/schema_continuations-65e2f27929ed24fb)
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.09s
     Running tests/sow_trace_continuations.rs (target/debug/deps/sow_trace_continuations-93367493ef68b082)
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.43s
     Running tests/suspension_oracle.rs (target/debug/deps/suspension_oracle-1756073e73a214f6)
test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 20.06s
     Running unittests src/lib.rs (target/debug/deps/rum_syntax-e24f301251f42f42)
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.51s
     Running unittests src/lib.rs (target/debug/deps/rum_terminal-bae91ae0a1529dd6)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_core
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_derive
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
   Doc-tests rum_host
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.99s
   Doc-tests rum_jupyter
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_repl
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_runtime
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.50s
   Doc-tests rum_stdlib
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_syntax
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_terminal
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Native ZeroMQ: **9 skips**, not successful native transport executions. `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1` permits these only when loopback binding is denied:
```text
test zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::basic::native_zeromq_backend_round_trips_execute_over_loopback_sockets ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_frontend_style_control_interrupt_round_trip ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::execute::native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::session_flow::native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::stdin::native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
```

Full workspace output: [logs/workspace.txt](../../../.temp/brief-592/logs/workspace.txt). All mandatory commands ran and their final gates passed. The earlier stale-certificate failures were resolved by generated replay; the diagnostic continuation failures remain owed and were not repaired.

### 4. Stop-loss counters and remaining director decisions

- Entries drained: **58/77**; requested entries kept owed: **19** (four defective, fourteen absent roles, one documented boundary).
- Added variants: **375/375 per build**; lifecycle obligations: **450/450 per build**, plus completion/fresh requests.
- Falsifiers: **75/75 descriptor pairs per build**, plus the duration normalization guard.
- Defect entries recorded: **4**. Existing test names changed/removed: **0**.
- Stop-loss did not trigger: implementation attempts advanced passing variants/lifecycle or recorded findings; one compiler-error attempt was corrected. After attachment, remaining runs were required verification, stronger package-state assertions, and generated maintenance replay.

Director decisions: the four defective entries require repair groups; the fourteen nonexistent roles need catalog disposition; the restore-only boundary needs an explicit oracle disposition; the two reported policies need correction at the gate. Everything is left uncommitted. Final checkpoint: [PROGRESS.md](../../../.temp/brief-592/PROGRESS.md).
