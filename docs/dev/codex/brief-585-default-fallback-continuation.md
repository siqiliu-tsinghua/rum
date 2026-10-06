# brief-585 — G10: a suspended direct `Default` lookup continues into the ordinary scan

Implementation brief. **Do not run `git` in any form**, including before you
have read this brief. Leave every change uncommitted for the director's gate.

## Goal

brief-583's re-census found group G10, the last Lost site: in
`try_apply_down_values`
(`crates/rum-runtime/src/runtime/definitions/application/down_values.rs:62-79`),
a call whose head is `Default` first tries the direct default lookup; if that
returns `None` the eager path continues into the ordinary down-value scan. When
a guard inside the direct lookup asks for input, the callee's own frame resumes
the lookup, but nothing holds the caller's decision — *`Some` returns, `None`
continues* — so a resumed decline ends the call instead of continuing:

```
log = {}; n = 0;
Default[d] := 7 /; (n = n + 1; AppendTo[log, n]; If[n <= 2, InputString["p: "]]; n >= 3);
Default[d]
```

With answers `x`, `y` both paths reach two prompts; the resumed path returns
`Default[d]` with log `{1, 2}`, the queued path `7` with log `{1, 2, 3}`.
Lost continuations **1 -> 0** by the re-census.

Hold the caller's decision in a frame and enter **one** ordinary-fallback
helper shared by the eager and the resumed path: `Some` completes, an ordinary
error forwards, `None` continues into the normal scan — exactly once, never
restarting the direct lookup or replaying a declined guard.

brief-583's report and `.temp/brief-583/draft-G10.md` are the raw material;
**this brief governs** where they differ.

## Context pointers

- brief-583 and its report; `.temp/brief-583/{draft-G10.md,new-lost.tsv,new-probes.json,new-probes.tsv}` (P0226).
- `definitions/application/down_values.rs:16-145`, `default_values.rs:15-258`,
  `eval_core/call.rs:291-340`, the `Option<Expr>` continuation codec in
  `suspension/primitive.rs`; `docs/guide/patterns-and-rules.md:165-184`, the
  guide's Suspension section.
- The accepted continuation pattern of briefs 559-584 (cold capture on the
  error edge, immutable frame, working copy only on resume).

## Scope

**Objective:** the caller's completion phase, its frame, the tests, and the
standing and performance gates.

**Edit surface:** `down_values.rs` (only the split into a prelude and the
shared fallback helper, and module wiring); a new responsibility submodule
`down_values/default_fallback.rs` (a `resume_frames.rs` child if size
requires); `default_values.rs` only for a narrowly required shared completion
type or helper; new `crates/rum-stdlib/tests/default_fallback_continuations.rs`
and its support children (reuse the existing oracle helpers without weakening
them); `docs/guide/patterns-and-rules.md`, only to state the existing
direct-`Default` guard-decline continuation; rum-runtime's frame lists
(exactly the new types).

**Standing duty — always in scope** (`TEMPLATE.md`): fingerprints, both
certificates, ownership reconciliation, every row disclosed. **STOP** on any
existing row change or disappearance, any new `UnknownHostPayload` or
`self_reaching` row, any new row in untouched code.

**Fence:** exact- and implicit-default lookup order and snapshots, definition
precedence, the candidate cache, the matcher, `Expr`, host APIs, catalog
metadata, manifests and the lockfile unchanged; **no refactor of the down-value
dispatcher beyond the split**; no leases; `scripts/core-language-microbench.sh`
unchanged. No edits to `AGENTS.md`, `TODO.md`, root `PROGRESS.md`, earlier
briefs or other docs.

**Disclosure duty:** every changed function, state field and root, inventory
row, measured label, adjacent defect — including any semantic conflict a live
definition change in a guard exposes (report it; do not change precedence).

## Deliverable

The eager path keeps its behaviour: the prelude computes what it computes today
(including the `down_values_cannot_match` decision) and the shared helper
performs the ordinary scan, re-reading live definitions exactly as the eager
path already does after the direct lookup runs guest code. On `InputRequired`
from the direct lookup (on the error edge only), a cold helper freezes the
call's head, shared arguments, attributes, symbol and the prelude's decision —
no borrowed `Definition`, no candidate built or cloned before the cold branch.
On resume the child's raw outcome is decoded explicitly as `Option<Expr>`:
`Some` completes, `None` enters the same helper once, an error forwards. A
second suspension inside the fallback is carried by its own existing frames.
Pure abandonment runs no later guard, fallback or body.

## Tests

Red first, recorded before any production edit, through the rum-vs-rum oracle
(value, error, termination, ordered messages and prints, `log`, actual
suspension, exact prompts, no owner after, fresh `1 + 1`): the witness above;
its accepting control (`n == 1` / `n >= 2`, `7` and `{1, 2}` on both paths);
brief-583's optional-default declining control; a terminal `None` (no other
candidate); a fallback candidate that is first and one that is in the middle;
unguarded hit and miss; a nested enclosing caller; two pauses with distinct
logs; an ordinary error and a guard side effect that finishes normally;
cancellation and reset at both pauses. Every earlier group's suite stays green.

## rum / Rust pitfalls (read twice)

1. Never make a pre-fix value the expected answer.
2. The callee's frame does not complete its caller; `None` is a live resumed
   outcome here, not a preflight result.
3. `try_apply_down_values` is on the hottest path of every call: the split must
   not move work onto it. Capture state only in the cold branch.
4. One change per measured candidate; stop after two consecutive compiling
   candidates that move none of: repaired sites, passing fixtures, regressing
   labels, worst regression.

## Verification (MANDATORY — run to green or evidence)

Serial, through `python3 scripts/run-bounded.py --timeout <s> -- env
CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true
TMPDIR=<repo>/.temp/brief-585/tmp …`. Bounds: focused 150 s; runtime,
ownership and retention 300 s; a one-unit release build 900 s; a metric
process 150 s; `cargo test --workspace --no-fail-fast` 2 400 s.

```
cargo fmt --check
cargo test -p rum-stdlib --test default_fallback_continuations
cargo test -p rum-stdlib --no-default-features --test default_fallback_continuations
cargo test -p rum-runtime
cargo test -p rum-stdlib --lib ownership_graph
cargo test -p rum-stdlib --lib local_production_acceptance
cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance
cargo test --workspace --no-fail-fast
```

Report native ZeroMQ skips as skips.

**Performance gate.** Base: `.temp/brief-585/base/rum-cli` and
`rum-cli-alloc` — brief-584's selected one-unit candidate, whose production
source is the incoming tree (hashes in `sha256.txt`); do not rebuild them.
Candidates with `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, default and
`--features rum-cli/alloc-count` (two arguments), fresh target directory
each, hash checked. Panel (scratch workloads, each result checked first):
direct `Default[d]` unguarded hit and miss, a guarded direct `Default` that
declines into a fallback candidate, the ordinary down-value hit, miss and
guarded hit of an ordinary head (`f[1]`), an optional-pattern default consumer
(`g[x_, y_.]`), an own-value read, and the controls `dot-three-no-input`,
`differences-no-input`, `take-multispec-small` (`.temp/brief-563/perf/`).
**Noise floor first** (base against itself, two pairs); a label regresses when
both candidate samples exceed `max(+0.05%, floor)`. Allocation counts, then two
paired samples of `taskset -c 2 perf stat -x, -e cpu_core/instructions/u`, one
label per process, stdin `/dev/null`, `(count(2N) - count(N)) / N`, raw output
first. No timing suite. At most eight measured candidates.

## Checkpointing — protect the scene

Everything under `.temp/brief-585/`; `PROGRESS.md` after the red capture, each
compiling candidate and each gate.

## Hard rules

- **No `git`, in any form.** No network; nothing outside this repository
  (except the legacy certificate output `.temp/brief-546/certificates/`).
- Commands: scoped reads and edits, `rg`/`grep`, `python3`, bounded serial
  Cargo, the release `rum-cli` on passing programs, `taskset`/`perf`,
  `valgrind --tool=callgrind`. No probe that aborts a process.
- **Testing Policy is a hard line** (`AGENTS.md`). No new `unsafe`. Never store
  a `Session` borrow, the cancellation observer or a `NativeStackDepth` in a
  frame.
- Paste real output; say plainly what did not run or did not pass.

## Report format (append to this brief)

1. Changed files and functions, state fields and roots.
2. Verbatim red and green output, ownership and certificate results, the broad
   suites; every inventory row change.
3. The repaired obligation (1 -> 0 by the re-census); the noise floors and every
   measured row.
4. Stop-loss counters, residuals, open questions.

---

## Executor report — brief-585

### 1. Changed files, functions, fields and roots

**Performance gate failed; stop-loss triggered. The current tree is not accepted.** Implemented G10 with one cold caller frame and one shared ordinary fallback helper. The scoped re-census of the incoming outstanding obligation is **Lost 1 -> 0**: P0226 is now Framed. This is the repaired-site re-census, not a rerun of brief-583's whole-source AST census. [re-census.tsv](../../../.temp/brief-585/re-census.tsv); [re-census-source-evidence.txt](../../../.temp/brief-585/re-census-source-evidence.txt); [re-census.txt](../../../.temp/brief-585/re-census.txt).

- `down_values.rs`: `try_apply_down_values` retains the original prelude, including `down_values_cannot_match`, and explicitly handles direct-lookup success, decline, input and ordinary error. `try_apply_down_values_fallback` is the original ordinary-scan suffix, shared by eager and resumed calls; [fallback-body-proof.txt](../../../.temp/brief-585/fallback-body-proof.txt) records byte identity. No dispatcher refactor beyond this split.
- `down_values/default_fallback.rs`: `suspend` captures only on `InputRequired`; `DirectDefaultFallbackFrame::resume` explicitly decodes `Option<Expr>`, returns `Some`, forwards errors, or calls that same helper once on `None`. The frame stays immutable; the existing fallback scan owns any later suspension.
- New frame fields: `head: Expr`, `args: SharedArgs`, `attributes: AttributeSet`, `symbol: Symbol`, `down_values_cannot_match: bool`. The head and shared arguments trace guest expression roots, and the bare symbol is independently traced as a symbol root. Attributes and the boolean prelude decision carry no expression or symbol roots. There is no retained Definition, Session borrow, cancellation observer or NativeStackDepth. Derived `FrameRootTrace` and `PureFrameAbandon` account for roots and pure abandonment.
- Runtime frame lists: exactly one new entry in each derive list, and `suspension_protocol_repository_invariants_hold` changes the named-frame count from 563 to 564.
- `patterns-and-rules.md`: states the existing direct-Default decline/fallback continuation and live-definition policy.

New test functions (all under `default_fallback_continuations`):

- `cases.rs`: `direct_decline_continues_to_first_ordinary_candidate`, `accepting_direct_control`, `optional_default_declining_control`, `terminal_none_still_finishes_ordinary_scan`, `ordinary_fallback_first_pattern_candidate`, `unguarded_direct_hit`, `unguarded_direct_miss`, `nested_caller_keeps_its_suffix`, `fallback_suspends_again_with_distinct_effects`, `direct_guard_error_forwards_without_fallback`, `fallback_guard_error_forwards_without_body`, `accepted_body_can_suspend`, `completed_guard_side_effects_are_not_replayed`, `fallback_reads_live_definitions_after_guard`, `ordinary_fallback_middle_candidate`.
- `lifecycle.rs`: `at_pause`, `cancel`, `reset`, `cancel_direct`, `cancel_fallback`, `abandon_direct_without_answer`, `abandon_fallback_without_answer`, `reset_direct`, `reset_fallback`.
- `support.rs`: `run`, `compare`.

The test root wires those modules and reuses the unchanged `rule_pattern_continuations/support.rs` oracle. `Case` descriptors have `label`, `source`, `prompts`, `value`, and `log` fields. The comparison checks value/error/termination, ordered messages/prints, effect logs, actual suspension, exact prompts, retired ownership and fresh `1+1`. Reset tests rebuild and replace the inner Session exactly as HostSession::reset does; the test-owned print sink observes that dropping the old request runs no later guard or body. Cancellation covers both resume-with-cancel and abandonment without an answer.

All source/scope changes and hashes: [changed-files.json](../../../.temp/brief-585/changed-files.json); patch from the saved incoming files: [final.patch](../../../.temp/brief-585/final.patch). `down_values.rs` remains in the warning-size range; the brief forbids wider dispatcher refactoring. The new completion submodule is under 2 KB, and each new test module is under 6 KB. The large existing frame inventories remain closed lists; only the mandated entries/count were added.

### 2. Verbatim verification and every inventory row

All Cargo and metric child processes were serial, bounded, offline, single-job/single-test-thread, with repository-local TMPDIR. No git command, network, new unsafe, lease, vendor engine, aborting probe or out-of-scope source edit was used.

Red evidence was captured before any production edit. Initial red fixture refinements corrected expectations for the existing queued ordering and arranged a real middle candidate; equality assertions were never weakened. The four raw red captures are [red-01.log](../../../.temp/brief-585/red-01.log), [red-02.log](../../../.temp/brief-585/red-02.log), [red-03.log](../../../.temp/brief-585/red-03.log), [red-04-middle.log](../../../.temp/brief-585/red-04-middle.log). The final full-panel red and final middle-candidate red follow verbatim.

[red-03.log](../../../.temp/brief-585/red-03.log)
Command: `python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-585/tmp cargo test -p rum-stdlib --test default_fallback_continuations -- --nocapture` (exit 101).
```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 7.56s
     Running tests/default_fallback_continuations.rs (target/debug/deps/default_fallback_continuations-febb1fd0b2e2f327)

running 21 tests
test default_fallback_continuations::cases::accepted_body_can_suspend ... PAUSE	accepted-body	["body: "]	[]	[]
ORACLE	accepted-body	queued=(View { status: "OK", value: Some("7"), termination: "None", messages: [], prints: [] }, "List[1, 2]", [])	resumed=(View { status: "OK", value: Some("7"), termination: "None", messages: [], prints: [] }, "List[1, 2]", ["body: "])
ok
test default_fallback_continuations::cases::accepting_direct_control ... PAUSE	accepting-control	["p: "]	[]	[]
ORACLE	accepting-control	queued=(View { status: "OK", value: Some("7"), termination: "None", messages: [], prints: [] }, "List[1, 2]", [])	resumed=(View { status: "OK", value: Some("7"), termination: "None", messages: [], prints: [] }, "List[1, 2]", ["p: "])
ok
test default_fallback_continuations::cases::completed_guard_side_effects_are_not_replayed ... PAUSE	guard-finishes	["guard: "]	[]	[]
ORACLE	guard-finishes	queued=(View { status: "OK", value: Some("7"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])	resumed=(View { status: "OK", value: Some("7"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", ["guard: "])
ok
test default_fallback_continuations::cases::direct_decline_continues_to_first_ordinary_candidate ... PAUSE	witness-first-fallback	["p: "]	[]	[]
PAUSE	witness-first-fallback	["p: ", "p: "]	[]	[]
ORACLE	witness-first-fallback	queued=(View { status: "OK", value: Some("7"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])	resumed=(View { status: "OK", value: Some("Default[d]"), termination: "None", messages: [], prints: [] }, "List[1, 2]", ["p: ", "p: "])

thread 'default_fallback_continuations::cases::direct_decline_continues_to_first_ordinary_candidate' panicked at crates/rum-stdlib/tests/default_fallback_continuations/support.rs:47:5:
assertion `left == right` failed: value/error/termination/ordered messages/prints
  left: View { status: "OK", value: Some("Default[d]"), termination: "None", messages: [], prints: [] }
 right: View { status: "OK", value: Some("7"), termination: "None", messages: [], prints: [] }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test default_fallback_continuations::cases::direct_guard_error_forwards_without_fallback ... PAUSE	direct-error	["error: "]	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }]	["before"]
ORACLE	direct-error	queued=(View { status: "Other { message: \"uncaught throw: \\\"default boom\\\" with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"default boom\"] returned to top level." }], prints: ["before", "after"] }, "List[1, 2]", [])	resumed=(View { status: "Other { message: \"uncaught throw: \\\"default boom\\\" with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"default boom\"] returned to top level." }], prints: ["before", "after"] }, "List[1, 2]", ["error: "])
ok
test default_fallback_continuations::cases::fallback_guard_error_forwards_without_body ... PAUSE	fallback-error	["direct: "]	[]	[]
PAUSE	fallback-error	["direct: ", "error: "]	[]	[]
ORACLE	fallback-error	queued=(View { status: "Other { message: \"uncaught throw: \\\"fallback boom\\\" with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"fallback boom\"] returned to top level." }], prints: [] }, "List[1, 2, 3]", [])	resumed=(View { status: "Other { message: \"uncaught throw: \\\"fallback boom\\\" with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"fallback boom\"] returned to top level." }], prints: [] }, "List[1, 2, 3]", ["direct: ", "error: "])
ok
test default_fallback_continuations::cases::fallback_reads_live_definitions_after_guard ... PAUSE	live-fallback-definition	["guard: "]	[]	[]
ORACLE	live-fallback-definition	queued=(View { status: "OK", value: Some("9"), termination: "None", messages: [], prints: [] }, "List[1, 2, 9]", [])	resumed=(View { status: "OK", value: Some("9"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 9]", ["guard: "])

thread 'default_fallback_continuations::cases::fallback_reads_live_definitions_after_guard' panicked at crates/rum-stdlib/tests/default_fallback_continuations/support.rs:48:5:
assertion `left == right` failed: effect log
  left: "List[1, 2, 3, 9]"
 right: "List[1, 2, 9]"
FAILED
test default_fallback_continuations::cases::fallback_suspends_again_with_distinct_effects ... PAUSE	fallback-second-pause	["direct: "]	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }]	["direct"]
PAUSE	fallback-second-pause	["direct: ", "fallback: "]	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }]	["direct", "fallback"]
ORACLE	fallback-second-pause	queued=(View { status: "OK", value: Some("7"), termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }], prints: ["direct", "fallback", "body"] }, "List[1, 10, 2, 30, 3, 40]", [])	resumed=(View { status: "OK", value: Some("Default[d]"), termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }], prints: ["direct", "fallback"] }, "List[1, 10, 2, 30]", ["direct: ", "fallback: "])

thread 'default_fallback_continuations::cases::fallback_suspends_again_with_distinct_effects' panicked at crates/rum-stdlib/tests/default_fallback_continuations/support.rs:47:5:
assertion `left == right` failed: value/error/termination/ordered messages/prints
  left: View { status: "OK", value: Some("Default[d]"), termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }], prints: ["direct", "fallback"] }
 right: View { status: "OK", value: Some("7"), termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }], prints: ["direct", "fallback", "body"] }
FAILED
test default_fallback_continuations::cases::nested_caller_keeps_its_suffix ... PAUSE	nested-caller	["p: "]	[]	[]
PAUSE	nested-caller	["p: ", "p: "]	[]	[]
ORACLE	nested-caller	queued=(View { status: "OK", value: Some("List[17, 11]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 4, 3]", [])	resumed=(View { status: "OK", value: Some("List[17, 11]"), termination: "None", messages: [], prints: [] }, "List[1, 4, 2, 3]", ["p: ", "p: "])

thread 'default_fallback_continuations::cases::nested_caller_keeps_its_suffix' panicked at crates/rum-stdlib/tests/default_fallback_continuations/support.rs:48:5:
assertion `left == right` failed: effect log
  left: "List[1, 4, 2, 3]"
 right: "List[1, 2, 4, 3]"
FAILED
test default_fallback_continuations::cases::optional_default_declining_control ... PAUSE	optional-decline-control	["p: "]	[]	[]
PAUSE	optional-decline-control	["p: ", "p: "]	[]	[]
ORACLE	optional-decline-control	queued=(View { status: "OK", value: Some("d[]"), termination: "None", messages: [], prints: [] }, "List[1, 1]", [])	resumed=(View { status: "OK", value: Some("d[]"), termination: "None", messages: [], prints: [] }, "List[1, 1]", ["p: ", "p: "])
ok
test default_fallback_continuations::cases::ordinary_fallback_first_pattern_candidate ... PAUSE	first-pattern-fallback	["p: "]	[]	[]
ORACLE	first-pattern-fallback	queued=(View { status: "OK", value: Some("9"), termination: "None", messages: [], prints: [] }, "List[1, 9]", [])	resumed=(View { status: "OK", value: Some("9"), termination: "None", messages: [], prints: [] }, "List[1, 2, 9]", ["p: "])

thread 'default_fallback_continuations::cases::ordinary_fallback_first_pattern_candidate' panicked at crates/rum-stdlib/tests/default_fallback_continuations/support.rs:48:5:
assertion `left == right` failed: effect log
  left: "List[1, 2, 9]"
 right: "List[1, 9]"
FAILED
test default_fallback_continuations::cases::ordinary_fallback_middle_candidate ... PAUSE	middle-fallback	["p: "]	[]	[]
ORACLE	middle-fallback	queued=(View { status: "OK", value: Some("9"), termination: "None", messages: [], prints: [] }, "List[1, 9, 90]", [])	resumed=(View { status: "OK", value: Some("9"), termination: "None", messages: [], prints: [] }, "List[1, 2, 9, 90]", ["p: "])

thread 'default_fallback_continuations::cases::ordinary_fallback_middle_candidate' panicked at crates/rum-stdlib/tests/default_fallback_continuations/support.rs:48:5:
assertion `left == right` failed: effect log
  left: "List[1, 2, 9, 90]"
 right: "List[1, 9, 90]"
FAILED
test default_fallback_continuations::cases::terminal_none_still_finishes_ordinary_scan ... PAUSE	terminal-none	["p: "]	[]	[]
PAUSE	terminal-none	["p: ", "p: "]	[]	[]
ORACLE	terminal-none	queued=(View { status: "OK", value: Some("Default[d]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4]", [])	resumed=(View { status: "OK", value: Some("Default[d]"), termination: "None", messages: [], prints: [] }, "List[1, 2]", ["p: ", "p: "])

thread 'default_fallback_continuations::cases::terminal_none_still_finishes_ordinary_scan' panicked at crates/rum-stdlib/tests/default_fallback_continuations/support.rs:48:5:
assertion `left == right` failed: effect log
  left: "List[1, 2]"
 right: "List[1, 2, 3, 4]"
FAILED
test default_fallback_continuations::cases::unguarded_direct_hit ... ORACLE	unguarded-hit	queued=(View { status: "OK", value: Some("7"), termination: "None", messages: [], prints: [] }, "List[]", [])	resumed=(View { status: "OK", value: Some("7"), termination: "None", messages: [], prints: [] }, "List[]", [])
ok
test default_fallback_continuations::cases::unguarded_direct_miss ... ORACLE	unguarded-miss	queued=(View { status: "OK", value: Some("Default[d]"), termination: "None", messages: [], prints: [] }, "List[]", [])	resumed=(View { status: "OK", value: Some("Default[d]"), termination: "None", messages: [], prints: [] }, "List[]", [])
ok
test default_fallback_continuations::lifecycle::abandon_direct_without_answer ... CANCEL	stop=0	without_answer=true	View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["direct"] }	List[1]
ok
test default_fallback_continuations::lifecycle::abandon_fallback_without_answer ... CANCEL	stop=1	without_answer=true	View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["direct", "fallback"] }	List[1, 10, 2]
ok
test default_fallback_continuations::lifecycle::cancel_direct ... CANCEL	stop=0	without_answer=false	View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["direct"] }	List[1]
ok
test default_fallback_continuations::lifecycle::cancel_fallback ... CANCEL	stop=1	without_answer=false	View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["direct", "fallback"] }	List[1, 10, 2]
ok
test default_fallback_continuations::lifecycle::reset_direct ... RESET	stop=0	prints=["direct"]	owner=None	fresh=2
ok
test default_fallback_continuations::lifecycle::reset_fallback ... RESET	stop=1	prints=["direct", "fallback"]	owner=None	fresh=2
ok

failures:

failures:
    default_fallback_continuations::cases::direct_decline_continues_to_first_ordinary_candidate
    default_fallback_continuations::cases::fallback_reads_live_definitions_after_guard
    default_fallback_continuations::cases::fallback_suspends_again_with_distinct_effects
    default_fallback_continuations::cases::nested_caller_keeps_its_suffix
    default_fallback_continuations::cases::ordinary_fallback_first_pattern_candidate
    default_fallback_continuations::cases::ordinary_fallback_middle_candidate
    default_fallback_continuations::cases::terminal_none_still_finishes_ordinary_scan

test result: FAILED. 14 passed; 7 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s

error: test failed, to rerun pass `-p rum-stdlib --test default_fallback_continuations`
```

[red-04-middle.log](../../../.temp/brief-585/red-04-middle.log)
Command: `python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-585/tmp cargo test -p rum-stdlib --test default_fallback_continuations ordinary_fallback_middle_candidate -- --nocapture` (exit 101).
```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 6.84s
     Running tests/default_fallback_continuations.rs (target/debug/deps/default_fallback_continuations-febb1fd0b2e2f327)

running 1 test
test default_fallback_continuations::cases::ordinary_fallback_middle_candidate ... PAUSE	middle-fallback	["p: "]	[]	[]
ORACLE	middle-fallback	queued=(View { status: "OK", value: Some("9"), termination: "None", messages: [], prints: [] }, "List[1, 8, 9, 90]", [])	resumed=(View { status: "OK", value: Some("9"), termination: "None", messages: [], prints: [] }, "List[1, 2, 8, 9, 90]", ["p: "])

thread 'default_fallback_continuations::cases::ordinary_fallback_middle_candidate' panicked at crates/rum-stdlib/tests/default_fallback_continuations/support.rs:48:5:
assertion `left == right` failed: effect log
  left: "List[1, 2, 8, 9, 90]"
 right: "List[1, 8, 9, 90]"
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED

failures:

failures:
    default_fallback_continuations::cases::ordinary_fallback_middle_candidate

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 20 filtered out; finished in 0.01s

error: test failed, to rerun pass `-p rum-stdlib --test default_fallback_continuations`
```

Default and no-default focused green output:
[candidate-01-focused.log](../../../.temp/brief-585/candidate-01-focused.log)
Command: `python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-585/tmp cargo test -p rum-stdlib --test default_fallback_continuations -- --nocapture` (exit 0).
```text
    Blocking waiting for file lock on build directory
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 34.32s
     Running tests/default_fallback_continuations.rs (target/debug/deps/default_fallback_continuations-febb1fd0b2e2f327)

running 21 tests
test default_fallback_continuations::cases::accepted_body_can_suspend ... PAUSE	accepted-body	["body: "]	[]	[]
ORACLE	accepted-body	queued=(View { status: "OK", value: Some("7"), termination: "None", messages: [], prints: [] }, "List[1, 2]", [])	resumed=(View { status: "OK", value: Some("7"), termination: "None", messages: [], prints: [] }, "List[1, 2]", ["body: "])
ok
test default_fallback_continuations::cases::accepting_direct_control ... PAUSE	accepting-control	["p: "]	[]	[]
ORACLE	accepting-control	queued=(View { status: "OK", value: Some("7"), termination: "None", messages: [], prints: [] }, "List[1, 2]", [])	resumed=(View { status: "OK", value: Some("7"), termination: "None", messages: [], prints: [] }, "List[1, 2]", ["p: "])
ok
test default_fallback_continuations::cases::completed_guard_side_effects_are_not_replayed ... PAUSE	guard-finishes	["guard: "]	[]	[]
ORACLE	guard-finishes	queued=(View { status: "OK", value: Some("7"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])	resumed=(View { status: "OK", value: Some("7"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", ["guard: "])
ok
test default_fallback_continuations::cases::direct_decline_continues_to_first_ordinary_candidate ... PAUSE	witness-first-fallback	["p: "]	[]	[]
PAUSE	witness-first-fallback	["p: ", "p: "]	[]	[]
ORACLE	witness-first-fallback	queued=(View { status: "OK", value: Some("7"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])	resumed=(View { status: "OK", value: Some("7"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", ["p: ", "p: "])
ok
test default_fallback_continuations::cases::direct_guard_error_forwards_without_fallback ... PAUSE	direct-error	["error: "]	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }]	["before"]
ORACLE	direct-error	queued=(View { status: "Other { message: \"uncaught throw: \\\"default boom\\\" with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"default boom\"] returned to top level." }], prints: ["before", "after"] }, "List[1, 2]", [])	resumed=(View { status: "Other { message: \"uncaught throw: \\\"default boom\\\" with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"default boom\"] returned to top level." }], prints: ["before", "after"] }, "List[1, 2]", ["error: "])
ok
test default_fallback_continuations::cases::fallback_guard_error_forwards_without_body ... PAUSE	fallback-error	["direct: "]	[]	[]
PAUSE	fallback-error	["direct: ", "error: "]	[]	[]
ORACLE	fallback-error	queued=(View { status: "Other { message: \"uncaught throw: \\\"fallback boom\\\" with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"fallback boom\"] returned to top level." }], prints: [] }, "List[1, 2, 3]", [])	resumed=(View { status: "Other { message: \"uncaught throw: \\\"fallback boom\\\" with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"fallback boom\"] returned to top level." }], prints: [] }, "List[1, 2, 3]", ["direct: ", "error: "])
ok
test default_fallback_continuations::cases::fallback_reads_live_definitions_after_guard ... PAUSE	live-fallback-definition	["guard: "]	[]	[]
ORACLE	live-fallback-definition	queued=(View { status: "OK", value: Some("9"), termination: "None", messages: [], prints: [] }, "List[1, 2, 9]", [])	resumed=(View { status: "OK", value: Some("9"), termination: "None", messages: [], prints: [] }, "List[1, 2, 9]", ["guard: "])
ok
test default_fallback_continuations::cases::fallback_suspends_again_with_distinct_effects ... PAUSE	fallback-second-pause	["direct: "]	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }]	["direct"]
PAUSE	fallback-second-pause	["direct: ", "fallback: "]	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }]	["direct", "fallback"]
ORACLE	fallback-second-pause	queued=(View { status: "OK", value: Some("7"), termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }], prints: ["direct", "fallback", "body"] }, "List[1, 10, 2, 30, 3, 40]", [])	resumed=(View { status: "OK", value: Some("7"), termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }], prints: ["direct", "fallback", "body"] }, "List[1, 10, 2, 30, 3, 40]", ["direct: ", "fallback: "])
ok
test default_fallback_continuations::cases::nested_caller_keeps_its_suffix ... PAUSE	nested-caller	["p: "]	[]	[]
PAUSE	nested-caller	["p: ", "p: "]	[]	[]
ORACLE	nested-caller	queued=(View { status: "OK", value: Some("List[17, 11]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 4, 3]", [])	resumed=(View { status: "OK", value: Some("List[17, 11]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 4, 3]", ["p: ", "p: "])
ok
test default_fallback_continuations::cases::optional_default_declining_control ... PAUSE	optional-decline-control	["p: "]	[]	[]
PAUSE	optional-decline-control	["p: ", "p: "]	[]	[]
ORACLE	optional-decline-control	queued=(View { status: "OK", value: Some("d[]"), termination: "None", messages: [], prints: [] }, "List[1, 1]", [])	resumed=(View { status: "OK", value: Some("d[]"), termination: "None", messages: [], prints: [] }, "List[1, 1]", ["p: ", "p: "])
ok
test default_fallback_continuations::cases::ordinary_fallback_first_pattern_candidate ... PAUSE	first-pattern-fallback	["p: "]	[]	[]
ORACLE	first-pattern-fallback	queued=(View { status: "OK", value: Some("9"), termination: "None", messages: [], prints: [] }, "List[1, 9]", [])	resumed=(View { status: "OK", value: Some("9"), termination: "None", messages: [], prints: [] }, "List[1, 9]", ["p: "])
ok
test default_fallback_continuations::cases::ordinary_fallback_middle_candidate ... PAUSE	middle-fallback	["p: "]	[]	[]
ORACLE	middle-fallback	queued=(View { status: "OK", value: Some("9"), termination: "None", messages: [], prints: [] }, "List[1, 8, 9, 90]", [])	resumed=(View { status: "OK", value: Some("9"), termination: "None", messages: [], prints: [] }, "List[1, 8, 9, 90]", ["p: "])
ok
test default_fallback_continuations::cases::terminal_none_still_finishes_ordinary_scan ... PAUSE	terminal-none	["p: "]	[]	[]
PAUSE	terminal-none	["p: ", "p: "]	[]	[]
ORACLE	terminal-none	queued=(View { status: "OK", value: Some("Default[d]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4]", [])	resumed=(View { status: "OK", value: Some("Default[d]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4]", ["p: ", "p: "])
ok
test default_fallback_continuations::cases::unguarded_direct_hit ... ORACLE	unguarded-hit	queued=(View { status: "OK", value: Some("7"), termination: "None", messages: [], prints: [] }, "List[]", [])	resumed=(View { status: "OK", value: Some("7"), termination: "None", messages: [], prints: [] }, "List[]", [])
ok
test default_fallback_continuations::cases::unguarded_direct_miss ... ORACLE	unguarded-miss	queued=(View { status: "OK", value: Some("Default[d]"), termination: "None", messages: [], prints: [] }, "List[]", [])	resumed=(View { status: "OK", value: Some("Default[d]"), termination: "None", messages: [], prints: [] }, "List[]", [])
ok
test default_fallback_continuations::lifecycle::abandon_direct_without_answer ... CANCEL	stop=0	without_answer=true	View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["direct"] }	List[1]
ok
test default_fallback_continuations::lifecycle::abandon_fallback_without_answer ... CANCEL	stop=1	without_answer=true	View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["direct", "fallback"] }	List[1, 10, 2]
ok
test default_fallback_continuations::lifecycle::cancel_direct ... CANCEL	stop=0	without_answer=false	View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["direct"] }	List[1]
ok
test default_fallback_continuations::lifecycle::cancel_fallback ... CANCEL	stop=1	without_answer=false	View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["direct", "fallback"] }	List[1, 10, 2]
ok
test default_fallback_continuations::lifecycle::reset_direct ... RESET	stop=0	prints=["direct"]	owner=None	fresh=2
ok
test default_fallback_continuations::lifecycle::reset_fallback ... RESET	stop=1	prints=["direct", "fallback"]	owner=None	fresh=2
ok

test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
```

[candidate-03-focused.log](../../../.temp/brief-585/candidate-03-focused.log)
Command: `python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-585/tmp cargo test -p rum-stdlib --test default_fallback_continuations -- --nocapture` (exit 0).
```text
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 15.50s
     Running tests/default_fallback_continuations.rs (target/debug/deps/default_fallback_continuations-febb1fd0b2e2f327)

running 21 tests
test default_fallback_continuations::cases::accepted_body_can_suspend ... PAUSE	accepted-body	["body: "]	[]	[]
ORACLE	accepted-body	queued=(View { status: "OK", value: Some("7"), termination: "None", messages: [], prints: [] }, "List[1, 2]", [])	resumed=(View { status: "OK", value: Some("7"), termination: "None", messages: [], prints: [] }, "List[1, 2]", ["body: "])
ok
test default_fallback_continuations::cases::accepting_direct_control ... PAUSE	accepting-control	["p: "]	[]	[]
ORACLE	accepting-control	queued=(View { status: "OK", value: Some("7"), termination: "None", messages: [], prints: [] }, "List[1, 2]", [])	resumed=(View { status: "OK", value: Some("7"), termination: "None", messages: [], prints: [] }, "List[1, 2]", ["p: "])
ok
test default_fallback_continuations::cases::completed_guard_side_effects_are_not_replayed ... PAUSE	guard-finishes	["guard: "]	[]	[]
ORACLE	guard-finishes	queued=(View { status: "OK", value: Some("7"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])	resumed=(View { status: "OK", value: Some("7"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", ["guard: "])
ok
test default_fallback_continuations::cases::direct_decline_continues_to_first_ordinary_candidate ... PAUSE	witness-first-fallback	["p: "]	[]	[]
PAUSE	witness-first-fallback	["p: ", "p: "]	[]	[]
ORACLE	witness-first-fallback	queued=(View { status: "OK", value: Some("7"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])	resumed=(View { status: "OK", value: Some("7"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", ["p: ", "p: "])
ok
test default_fallback_continuations::cases::direct_guard_error_forwards_without_fallback ... PAUSE	direct-error	["error: "]	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }]	["before"]
ORACLE	direct-error	queued=(View { status: "Other { message: \"uncaught throw: \\\"default boom\\\" with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"default boom\"] returned to top level." }], prints: ["before", "after"] }, "List[1, 2]", [])	resumed=(View { status: "Other { message: \"uncaught throw: \\\"default boom\\\" with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"default boom\"] returned to top level." }], prints: ["before", "after"] }, "List[1, 2]", ["error: "])
ok
test default_fallback_continuations::cases::fallback_guard_error_forwards_without_body ... PAUSE	fallback-error	["direct: "]	[]	[]
PAUSE	fallback-error	["direct: ", "error: "]	[]	[]
ORACLE	fallback-error	queued=(View { status: "Other { message: \"uncaught throw: \\\"fallback boom\\\" with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"fallback boom\"] returned to top level." }], prints: [] }, "List[1, 2, 3]", [])	resumed=(View { status: "Other { message: \"uncaught throw: \\\"fallback boom\\\" with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"fallback boom\"] returned to top level." }], prints: [] }, "List[1, 2, 3]", ["direct: ", "error: "])
ok
test default_fallback_continuations::cases::fallback_reads_live_definitions_after_guard ... PAUSE	live-fallback-definition	["guard: "]	[]	[]
ORACLE	live-fallback-definition	queued=(View { status: "OK", value: Some("9"), termination: "None", messages: [], prints: [] }, "List[1, 2, 9]", [])	resumed=(View { status: "OK", value: Some("9"), termination: "None", messages: [], prints: [] }, "List[1, 2, 9]", ["guard: "])
ok
test default_fallback_continuations::cases::fallback_suspends_again_with_distinct_effects ... PAUSE	fallback-second-pause	["direct: "]	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }]	["direct"]
PAUSE	fallback-second-pause	["direct: ", "fallback: "]	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }]	["direct", "fallback"]
ORACLE	fallback-second-pause	queued=(View { status: "OK", value: Some("7"), termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }], prints: ["direct", "fallback", "body"] }, "List[1, 10, 2, 30, 3, 40]", [])	resumed=(View { status: "OK", value: Some("7"), termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }], prints: ["direct", "fallback", "body"] }, "List[1, 10, 2, 30, 3, 40]", ["direct: ", "fallback: "])
ok
test default_fallback_continuations::cases::nested_caller_keeps_its_suffix ... PAUSE	nested-caller	["p: "]	[]	[]
PAUSE	nested-caller	["p: ", "p: "]	[]	[]
ORACLE	nested-caller	queued=(View { status: "OK", value: Some("List[17, 11]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 4, 3]", [])	resumed=(View { status: "OK", value: Some("List[17, 11]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 4, 3]", ["p: ", "p: "])
ok
test default_fallback_continuations::cases::optional_default_declining_control ... PAUSE	optional-decline-control	["p: "]	[]	[]
PAUSE	optional-decline-control	["p: ", "p: "]	[]	[]
ORACLE	optional-decline-control	queued=(View { status: "OK", value: Some("d[]"), termination: "None", messages: [], prints: [] }, "List[1, 1]", [])	resumed=(View { status: "OK", value: Some("d[]"), termination: "None", messages: [], prints: [] }, "List[1, 1]", ["p: ", "p: "])
ok
test default_fallback_continuations::cases::ordinary_fallback_first_pattern_candidate ... PAUSE	first-pattern-fallback	["p: "]	[]	[]
ORACLE	first-pattern-fallback	queued=(View { status: "OK", value: Some("9"), termination: "None", messages: [], prints: [] }, "List[1, 9]", [])	resumed=(View { status: "OK", value: Some("9"), termination: "None", messages: [], prints: [] }, "List[1, 9]", ["p: "])
ok
test default_fallback_continuations::cases::ordinary_fallback_middle_candidate ... PAUSE	middle-fallback	["p: "]	[]	[]
ORACLE	middle-fallback	queued=(View { status: "OK", value: Some("9"), termination: "None", messages: [], prints: [] }, "List[1, 8, 9, 90]", [])	resumed=(View { status: "OK", value: Some("9"), termination: "None", messages: [], prints: [] }, "List[1, 8, 9, 90]", ["p: "])
ok
test default_fallback_continuations::cases::terminal_none_still_finishes_ordinary_scan ... PAUSE	terminal-none	["p: "]	[]	[]
PAUSE	terminal-none	["p: ", "p: "]	[]	[]
ORACLE	terminal-none	queued=(View { status: "OK", value: Some("Default[d]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4]", [])	resumed=(View { status: "OK", value: Some("Default[d]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4]", ["p: ", "p: "])
ok
test default_fallback_continuations::cases::unguarded_direct_hit ... ORACLE	unguarded-hit	queued=(View { status: "OK", value: Some("7"), termination: "None", messages: [], prints: [] }, "List[]", [])	resumed=(View { status: "OK", value: Some("7"), termination: "None", messages: [], prints: [] }, "List[]", [])
ok
test default_fallback_continuations::cases::unguarded_direct_miss ... ORACLE	unguarded-miss	queued=(View { status: "OK", value: Some("Default[d]"), termination: "None", messages: [], prints: [] }, "List[]", [])	resumed=(View { status: "OK", value: Some("Default[d]"), termination: "None", messages: [], prints: [] }, "List[]", [])
ok
test default_fallback_continuations::lifecycle::abandon_direct_without_answer ... CANCEL	stop=0	without_answer=true	View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["direct"] }	List[1]
ok
test default_fallback_continuations::lifecycle::abandon_fallback_without_answer ... CANCEL	stop=1	without_answer=true	View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["direct", "fallback"] }	List[1, 10, 2]
ok
test default_fallback_continuations::lifecycle::cancel_direct ... CANCEL	stop=0	without_answer=false	View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["direct"] }	List[1]
ok
test default_fallback_continuations::lifecycle::cancel_fallback ... CANCEL	stop=1	without_answer=false	View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["direct", "fallback"] }	List[1, 10, 2]
ok
test default_fallback_continuations::lifecycle::reset_direct ... RESET	stop=0	prints=["direct"]	owner=None	fresh=2
ok
test default_fallback_continuations::lifecycle::reset_fallback ... RESET	stop=1	prints=["direct", "fallback"]	owner=None	fresh=2
ok

test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
```

[focused-no-default-final.log](../../../.temp/brief-585/focused-no-default-final.log)
Command: `python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-585/tmp cargo test -p rum-stdlib --no-default-features --test default_fallback_continuations -- --nocapture` (exit 0).
```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 10.53s
     Running tests/default_fallback_continuations.rs (target/debug/deps/default_fallback_continuations-6a52649b61eb24e8)

running 21 tests
test default_fallback_continuations::cases::accepted_body_can_suspend ... PAUSE	accepted-body	["body: "]	[]	[]
ORACLE	accepted-body	queued=(View { status: "OK", value: Some("7"), termination: "None", messages: [], prints: [] }, "List[1, 2]", [])	resumed=(View { status: "OK", value: Some("7"), termination: "None", messages: [], prints: [] }, "List[1, 2]", ["body: "])
ok
test default_fallback_continuations::cases::accepting_direct_control ... PAUSE	accepting-control	["p: "]	[]	[]
ORACLE	accepting-control	queued=(View { status: "OK", value: Some("7"), termination: "None", messages: [], prints: [] }, "List[1, 2]", [])	resumed=(View { status: "OK", value: Some("7"), termination: "None", messages: [], prints: [] }, "List[1, 2]", ["p: "])
ok
test default_fallback_continuations::cases::completed_guard_side_effects_are_not_replayed ... PAUSE	guard-finishes	["guard: "]	[]	[]
ORACLE	guard-finishes	queued=(View { status: "OK", value: Some("7"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])	resumed=(View { status: "OK", value: Some("7"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", ["guard: "])
ok
test default_fallback_continuations::cases::direct_decline_continues_to_first_ordinary_candidate ... PAUSE	witness-first-fallback	["p: "]	[]	[]
PAUSE	witness-first-fallback	["p: ", "p: "]	[]	[]
ORACLE	witness-first-fallback	queued=(View { status: "OK", value: Some("7"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])	resumed=(View { status: "OK", value: Some("7"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", ["p: ", "p: "])
ok
test default_fallback_continuations::cases::direct_guard_error_forwards_without_fallback ... PAUSE	direct-error	["error: "]	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }]	["before"]
ORACLE	direct-error	queued=(View { status: "Other { message: \"uncaught throw: \\\"default boom\\\" with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"default boom\"] returned to top level." }], prints: ["before", "after"] }, "List[1, 2]", [])	resumed=(View { status: "Other { message: \"uncaught throw: \\\"default boom\\\" with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"default boom\"] returned to top level." }], prints: ["before", "after"] }, "List[1, 2]", ["error: "])
ok
test default_fallback_continuations::cases::fallback_guard_error_forwards_without_body ... PAUSE	fallback-error	["direct: "]	[]	[]
PAUSE	fallback-error	["direct: ", "error: "]	[]	[]
ORACLE	fallback-error	queued=(View { status: "Other { message: \"uncaught throw: \\\"fallback boom\\\" with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"fallback boom\"] returned to top level." }], prints: [] }, "List[1, 2, 3]", [])	resumed=(View { status: "Other { message: \"uncaught throw: \\\"fallback boom\\\" with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"fallback boom\"] returned to top level." }], prints: [] }, "List[1, 2, 3]", ["direct: ", "error: "])
ok
test default_fallback_continuations::cases::fallback_reads_live_definitions_after_guard ... PAUSE	live-fallback-definition	["guard: "]	[]	[]
ORACLE	live-fallback-definition	queued=(View { status: "OK", value: Some("9"), termination: "None", messages: [], prints: [] }, "List[1, 2, 9]", [])	resumed=(View { status: "OK", value: Some("9"), termination: "None", messages: [], prints: [] }, "List[1, 2, 9]", ["guard: "])
ok
test default_fallback_continuations::cases::fallback_suspends_again_with_distinct_effects ... PAUSE	fallback-second-pause	["direct: "]	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }]	["direct"]
PAUSE	fallback-second-pause	["direct: ", "fallback: "]	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }]	["direct", "fallback"]
ORACLE	fallback-second-pause	queued=(View { status: "OK", value: Some("7"), termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }], prints: ["direct", "fallback", "body"] }, "List[1, 10, 2, 30, 3, 40]", [])	resumed=(View { status: "OK", value: Some("7"), termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }], prints: ["direct", "fallback", "body"] }, "List[1, 10, 2, 30, 3, 40]", ["direct: ", "fallback: "])
ok
test default_fallback_continuations::cases::nested_caller_keeps_its_suffix ... PAUSE	nested-caller	["p: "]	[]	[]
PAUSE	nested-caller	["p: ", "p: "]	[]	[]
ORACLE	nested-caller	queued=(View { status: "OK", value: Some("List[17, 11]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 4, 3]", [])	resumed=(View { status: "OK", value: Some("List[17, 11]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 4, 3]", ["p: ", "p: "])
ok
test default_fallback_continuations::cases::optional_default_declining_control ... PAUSE	optional-decline-control	["p: "]	[]	[]
PAUSE	optional-decline-control	["p: ", "p: "]	[]	[]
ORACLE	optional-decline-control	queued=(View { status: "OK", value: Some("d[]"), termination: "None", messages: [], prints: [] }, "List[1, 1]", [])	resumed=(View { status: "OK", value: Some("d[]"), termination: "None", messages: [], prints: [] }, "List[1, 1]", ["p: ", "p: "])
ok
test default_fallback_continuations::cases::ordinary_fallback_first_pattern_candidate ... PAUSE	first-pattern-fallback	["p: "]	[]	[]
ORACLE	first-pattern-fallback	queued=(View { status: "OK", value: Some("9"), termination: "None", messages: [], prints: [] }, "List[1, 9]", [])	resumed=(View { status: "OK", value: Some("9"), termination: "None", messages: [], prints: [] }, "List[1, 9]", ["p: "])
ok
test default_fallback_continuations::cases::ordinary_fallback_middle_candidate ... PAUSE	middle-fallback	["p: "]	[]	[]
ORACLE	middle-fallback	queued=(View { status: "OK", value: Some("9"), termination: "None", messages: [], prints: [] }, "List[1, 8, 9, 90]", [])	resumed=(View { status: "OK", value: Some("9"), termination: "None", messages: [], prints: [] }, "List[1, 8, 9, 90]", ["p: "])
ok
test default_fallback_continuations::cases::terminal_none_still_finishes_ordinary_scan ... PAUSE	terminal-none	["p: "]	[]	[]
PAUSE	terminal-none	["p: ", "p: "]	[]	[]
ORACLE	terminal-none	queued=(View { status: "OK", value: Some("Default[d]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4]", [])	resumed=(View { status: "OK", value: Some("Default[d]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4]", ["p: ", "p: "])
ok
test default_fallback_continuations::cases::unguarded_direct_hit ... ORACLE	unguarded-hit	queued=(View { status: "OK", value: Some("7"), termination: "None", messages: [], prints: [] }, "List[]", [])	resumed=(View { status: "OK", value: Some("7"), termination: "None", messages: [], prints: [] }, "List[]", [])
ok
test default_fallback_continuations::cases::unguarded_direct_miss ... ORACLE	unguarded-miss	queued=(View { status: "OK", value: Some("Default[d]"), termination: "None", messages: [], prints: [] }, "List[]", [])	resumed=(View { status: "OK", value: Some("Default[d]"), termination: "None", messages: [], prints: [] }, "List[]", [])
ok
test default_fallback_continuations::lifecycle::abandon_direct_without_answer ... CANCEL	stop=0	without_answer=true	View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["direct"] }	List[1]
ok
test default_fallback_continuations::lifecycle::abandon_fallback_without_answer ... CANCEL	stop=1	without_answer=true	View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["direct", "fallback"] }	List[1, 10, 2]
ok
test default_fallback_continuations::lifecycle::cancel_direct ... CANCEL	stop=0	without_answer=false	View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["direct"] }	List[1]
ok
test default_fallback_continuations::lifecycle::cancel_fallback ... CANCEL	stop=1	without_answer=false	View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["direct", "fallback"] }	List[1, 10, 2]
ok
test default_fallback_continuations::lifecycle::reset_direct ... RESET	stop=0	prints=["direct"]	owner=None	fresh=2
ok
test default_fallback_continuations::lifecycle::reset_fallback ... RESET	stop=1	prints=["direct", "fallback"]	owner=None	fresh=2
ok

test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
```

Standing gates:
[fmt-01.log](../../../.temp/brief-585/fmt-01.log)
Command: `python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-585/tmp cargo fmt --check` (exit 0).
```text

```

[runtime-01.log](../../../.temp/brief-585/runtime-01.log)
Command: `python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-585/tmp cargo test -p rum-runtime` (exit 0).
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 33.36s
     Running unittests src/lib.rs (target/debug/deps/rum_runtime-95cd5fbb46da4f1c)
test result: ok. 1138 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 29.71s
   Doc-tests rum_runtime
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.50s
```

[ownership-discovery.log](../../../.temp/brief-585/ownership-discovery.log)
Command: `python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-585/tmp cargo test -p rum-stdlib --lib ownership_graph::discovery_report_without_inventory_input -- --nocapture` (exit 0).
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 31.93s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-3a556b9db7fd59ac)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1139 filtered out; finished in 5.52s
```

[ownership-01.log](../../../.temp/brief-585/ownership-01.log)
Command: `python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-585/tmp cargo test -p rum-stdlib --lib ownership_graph -- --nocapture` (exit 0).
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 14.62s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-3a556b9db7fd59ac)
EXACT INVENTORY UnknownHostPayload: 65 rows
EXACT INVENTORY UnknownConstruct: 7682 rows
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1120 filtered out; finished in 59.81s
```

[fingerprints-replay.log](../../../.temp/brief-585/fingerprints-replay.log)
Command: `python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-585/tmp cargo test -p rum-stdlib --lib regenerate_local_contract_fingerprints -- --ignored --nocapture` (exit 0).
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-3a556b9db7fd59ac)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1139 filtered out; finished in 0.26s
```

[certificates-true-replay.log](../../../.temp/brief-585/certificates-true-replay.log)
Command: `python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-585/tmp cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture` (exit 101).
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 15.21s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-3a556b9db7fd59ac)
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::Unarmed;R=rum_stdlib::option_consumers::OptionConsumer;hash=true	0
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=rum_stdlib::option_consumers::OptionConsumer;hash=true	0
C02	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr;M=runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=none (nongeneric C02);hash=true	0
LeaseProbe538::probe	sealed modes	0
LeaseProbe538::start	sealed modes	0
LeaseProbe538::drive	sealed modes	0
LeaseProbe538::resume	sealed modes	0
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1139 filtered out; finished in 17.99s
```

[certificates-false-replay.log](../../../.temp/brief-585/certificates-false-replay.log)
Command: `python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-585/tmp cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture` (exit 101).
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 31.62s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-5d6fedc676d25fda)
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::Unarmed;R=rum_stdlib::option_consumers::OptionConsumer;hash=false	0
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=rum_stdlib::option_consumers::OptionConsumer;hash=false	0
C02	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr;M=runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=none (nongeneric C02);hash=false	0
LeaseProbe538::probe	sealed modes	0
LeaseProbe538::start	sealed modes	0
LeaseProbe538::drive	sealed modes	0
LeaseProbe538::resume	sealed modes	0
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1133 filtered out; finished in 17.90s
```

[certificates-true-final.log](../../../.temp/brief-585/certificates-true-final.log)
Command: `python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-585/tmp cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture` (exit 0).
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 14.37s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-3a556b9db7fd59ac)
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::Unarmed;R=rum_stdlib::option_consumers::OptionConsumer;hash=true	0
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=rum_stdlib::option_consumers::OptionConsumer;hash=true	0
C02	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr;M=runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=none (nongeneric C02);hash=true	0
LeaseProbe538::probe	sealed modes	0
LeaseProbe538::start	sealed modes	0
LeaseProbe538::drive	sealed modes	0
LeaseProbe538::resume	sealed modes	0
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1139 filtered out; finished in 18.04s
```

[certificates-false-final.log](../../../.temp/brief-585/certificates-false-final.log)
Command: `python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-585/tmp cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture` (exit 0).
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 14.08s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-5d6fedc676d25fda)
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::Unarmed;R=rum_stdlib::option_consumers::OptionConsumer;hash=false	0
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=rum_stdlib::option_consumers::OptionConsumer;hash=false	0
C02	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr;M=runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=none (nongeneric C02);hash=false	0
LeaseProbe538::probe	sealed modes	0
LeaseProbe538::start	sealed modes	0
LeaseProbe538::drive	sealed modes	0
LeaseProbe538::resume	sealed modes	0
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1133 filtered out; finished in 17.97s
```

[fmt-final.log](../../../.temp/brief-585/fmt-final.log)
Command: `python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-585/tmp cargo fmt --check` (exit 0).
```text

```

[runtime-final.log](../../../.temp/brief-585/runtime-final.log)
Command: `python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-585/tmp cargo test -p rum-runtime` (exit 0).
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 13.64s
     Running unittests src/lib.rs (target/debug/deps/rum_runtime-95cd5fbb46da4f1c)
test result: ok. 1138 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 29.41s
   Doc-tests rum_runtime
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.47s
```

[ownership-final.log](../../../.temp/brief-585/ownership-final.log)
Command: `python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-585/tmp cargo test -p rum-stdlib --lib ownership_graph -- --nocapture` (exit 0).
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 13.88s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-3a556b9db7fd59ac)
EXACT INVENTORY UnknownHostPayload: 65 rows
EXACT INVENTORY UnknownConstruct: 7683 rows
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1120 filtered out; finished in 61.30s
```

[fingerprints-replay-final.log](../../../.temp/brief-585/fingerprints-replay-final.log)
Command: `python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-585/tmp cargo test -p rum-stdlib --lib regenerate_local_contract_fingerprints -- --ignored --nocapture` (exit 0).
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-3a556b9db7fd59ac)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1139 filtered out; finished in 0.19s
```

[certificates-true-replay-final.log](../../../.temp/brief-585/certificates-true-replay-final.log)
Command: `python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-585/tmp cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture` (exit 101).
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 14.51s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-3a556b9db7fd59ac)
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::Unarmed;R=rum_stdlib::option_consumers::OptionConsumer;hash=true	0
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=rum_stdlib::option_consumers::OptionConsumer;hash=true	0
C02	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr;M=runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=none (nongeneric C02);hash=true	0
LeaseProbe538::probe	sealed modes	0
LeaseProbe538::start	sealed modes	0
LeaseProbe538::drive	sealed modes	0
LeaseProbe538::resume	sealed modes	0
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1139 filtered out; finished in 18.24s
```

[certificates-false-replay-final.log](../../../.temp/brief-585/certificates-false-replay-final.log)
Command: `python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-585/tmp cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture` (exit 101).
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 14.54s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-5d6fedc676d25fda)
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::Unarmed;R=rum_stdlib::option_consumers::OptionConsumer;hash=false	0
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=rum_stdlib::option_consumers::OptionConsumer;hash=false	0
C02	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr;M=runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=none (nongeneric C02);hash=false	0
LeaseProbe538::probe	sealed modes	0
LeaseProbe538::start	sealed modes	0
LeaseProbe538::drive	sealed modes	0
LeaseProbe538::resume	sealed modes	0
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1133 filtered out; finished in 18.13s
```

[certificates-true-final-final.log](../../../.temp/brief-585/certificates-true-final-final.log)
Command: `python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-585/tmp cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture` (exit 0).
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 14.29s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-3a556b9db7fd59ac)
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::Unarmed;R=rum_stdlib::option_consumers::OptionConsumer;hash=true	0
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=rum_stdlib::option_consumers::OptionConsumer;hash=true	0
C02	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr;M=runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=none (nongeneric C02);hash=true	0
LeaseProbe538::probe	sealed modes	0
LeaseProbe538::start	sealed modes	0
LeaseProbe538::drive	sealed modes	0
LeaseProbe538::resume	sealed modes	0
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1139 filtered out; finished in 18.18s
```

[certificates-false-final-final.log](../../../.temp/brief-585/certificates-false-final-final.log)
Command: `python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-585/tmp cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture` (exit 0).
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 14.53s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-5d6fedc676d25fda)
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::Unarmed;R=rum_stdlib::option_consumers::OptionConsumer;hash=false	0
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=rum_stdlib::option_consumers::OptionConsumer;hash=false	0
C02	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr;M=runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=none (nongeneric C02);hash=false	0
LeaseProbe538::probe	sealed modes	0
LeaseProbe538::start	sealed modes	0
LeaseProbe538::drive	sealed modes	0
LeaseProbe538::resume	sealed modes	0
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1133 filtered out; finished in 18.19s
```

Each certificate replay round intentionally exposed stale recorded cells in both feature configurations after source changes; their freshly earned marks had zero rejections. Only generated fingerprint/certificate cells were installed, followed by green replay in both feature configurations. Generator output `target/brief-544/local-contracts.tsv` and the legacy `.temp/brief-546/certificates/` files were copied into this brief's evidence. No admission, status, strength, exclusion or ownership disposition changed.

Ownership rows (all changes disclosed; unknown constructs remain unknown):

`host_payloads.tsv`: 65 -> 65.

`unknown_attributes.tsv`: 1497 -> 1498.
```text
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/definitions/application/down_values.rs	4	1	unresolved attribute/expansion derive (Clone , FrameRootTrace)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/definitions/application/down_values.rs	6	1	unresolved attribute/expansion derive (Clone , FrameRootTrace)
```

```text
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/definitions/application/down_values.rs	424	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/definitions/application/down_values.rs	456	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
```

```text
ADDED	UnknownConstruct	crates/rum-runtime/src/runtime/definitions/application/down_values/default_fallback.rs	7	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
```


`unknown_captures.tsv`: 2949 -> 2949.
```text
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/definitions/application/down_values.rs	56	30	closure captures: |signatures| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/definitions/application/down_values.rs	58	30	closure captures: |signatures| (capture types/modes unresolved)
```

```text
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/definitions/application/down_values.rs	59	30	closure captures: |signature| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/definitions/application/down_values.rs	61	30	closure captures: |signature| (capture types/modes unresolved)
```

```text
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/definitions/application/down_values.rs	94	66	closure captures: |_| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/definitions/application/down_values.rs	126	66	closure captures: |_| (capture types/modes unresolved)
```

```text
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/definitions/application/down_values.rs	145	80	closure captures: |definition| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/definitions/application/down_values.rs	177	80	closure captures: |definition| (capture types/modes unresolved)
```

```text
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/definitions/application/down_values.rs	150	22	closure captures: |value| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/definitions/application/down_values.rs	182	22	closure captures: |value| (capture types/modes unresolved)
```

```text
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/definitions/application/down_values.rs	302	54	closure captures: || (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/definitions/application/down_values.rs	334	54	closure captures: || (capture types/modes unresolved)
```

```text
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/definitions/application/down_values.rs	312	53	closure captures: || (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/definitions/application/down_values.rs	344	53	closure captures: || (capture types/modes unresolved)
```

```text
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/definitions/application/down_values.rs	446	24	closure captures: || (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/definitions/application/down_values.rs	478	24	closure captures: || (capture types/modes unresolved)
```

```text
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/definitions/application/down_values.rs	450	22	closure captures: |result| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/definitions/application/down_values.rs	482	22	closure captures: |result| (capture types/modes unresolved)
```


`unknown_macros.tsv`: 3145 -> 3147.
```text
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/definitions/application/down_values.rs	51	42	unexpanded macro matches! (ends at 51:100)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/definitions/application/down_values.rs	53	42	unexpanded macro matches! (ends at 53:100)
```

```text
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/definitions/application/down_values.rs	351	9	unexpanded macro crate :: resume_or_suspend! (ends at 351:114)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/definitions/application/down_values.rs	383	9	unexpanded macro crate :: resume_or_suspend! (ends at 383:114)
```

```text
ADDED	UnknownConstruct	crates/rum-runtime/src/runtime/definitions/application/down_values.rs	82	24	unexpanded macro matches! (ends at 82:72)
```

```text
ADDED	UnknownConstruct	crates/rum-runtime/src/runtime/definitions/application/down_values/default_fallback.rs	47	5	unexpanded macro crate :: resume_or_suspend! (ends at 53:7)
```


`unknown_storage.tsv`: 89 -> 89.

`self_reaching.tsv`: 59 -> 59.

`host_payload_ledger.tsv`: unchanged ledger -> unchanged ledger.
Exact row reconciliation: [ownership-row-changes.json](../../../.temp/brief-585/ownership-row-changes.json); earlier candidate positions: [ownership-row-changes-candidate-01.json](../../../.temp/brief-585/ownership-row-changes-candidate-01.json), [ownership-row-changes-candidate-02.json](../../../.temp/brief-585/ownership-row-changes-candidate-02.json), [ownership-row-changes-candidate-03.json](../../../.temp/brief-585/ownership-row-changes-candidate-03.json). No new UnknownHostPayload or self_reaching row; no row disappeared; all additions are in the new completion module or the touched direct-lookup prelude.

Every contract fingerprint cell (all other 18 columns unchanged):
```text
row	id	before	after
2	B01	fnv1a64:3f0979518b0821b3	fnv1a64:bb83edfb55ef1472
3	B02	fnv1a64:7e0ff69775ffe85a	fnv1a64:01398c9e325c6dff
4	B03	fnv1a64:b2a5f700a154ef79	fnv1a64:851463a6f8fa4f48
5	B04	fnv1a64:a3f7db810a2d93ab	fnv1a64:5efc6be28d86462a
6	B05	fnv1a64:944ffd1213ede956	fnv1a64:200308242799a3bb
7	B06	fnv1a64:29eaf326ca8093d7	fnv1a64:96ae923822ac5c16
8	B07	fnv1a64:e01c42fefbfa8766	fnv1a64:60fb6bacf3aa278b
9	B08	fnv1a64:2666d0698a6dac45	fnv1a64:ee41dc78e345cf04
10	B09	fnv1a64:4d41e98df81c484f	fnv1a64:da17704ae5b75eee
11	B10	fnv1a64:df8feb4d5efd32b0	fnv1a64:9f63038de26abbfd
12	B11	fnv1a64:654f8cde2421b9ab	fnv1a64:20541d3fa77a6c2a
13	B12	fnv1a64:0c4e6989d424dd4f	fnv1a64:9923f046c1bff3ee
14	I01	fnv1a64:1ceb644ef7c01833	fnv1a64:9965d8f8c2a70af2
15	I02	fnv1a64:eb917aa60c247156	fnv1a64:774485b81fd02bbb
16	I03	fnv1a64:3da12d052f716e0f	fnv1a64:613d551a9cc95cae
17	I04	fnv1a64:8ff8da88b4dd8cfa	fnv1a64:588f205f63cb3a1f
18	P01	fnv1a64:251cb8a2732de577	fnv1a64:7cc2ca75d61c42b6
19	P02	fnv1a64:6569d20b7bea10a0	fnv1a64:36761c6f2c28982d
20	P03	fnv1a64:db84ff7a45f156a1	fnv1a64:cb98f71320ee2670
21	P04	fnv1a64:72eb14acfaa9045a	fnv1a64:f614aab3b70589ff
22	P05	fnv1a64:c75400b350f7daa3	fnv1a64:742facb322739da2
23	P06	fnv1a64:5ccd8e5a605476cf	fnv1a64:e9a315174def8d6e
24	P070	fnv1a64:971b2a9c3ec9f0cf	fnv1a64:23f0b1592c65076e
25	P080	fnv1a64:6cb561a7c224bc70	fnv1a64:dcb63941eb95a4bd
26	P090	fnv1a64:5dc864a654b4ab1a	fnv1a64:a0fa892b930035bf
27	P100	fnv1a64:b3cdff23ddca09fb	fnv1a64:6e9ae6f02fe19f7a
28	P110	fnv1a64:6cf699d893b118f8	fnv1a64:a53487051bab21c5
29	P120	fnv1a64:40283cf3bde57f65	fnv1a64:f1956cfbfe4c8724
30	P130	fnv1a64:83e99c5c2aae6166	fnv1a64:04c8c50a225e018b
31	P140	fnv1a64:202cfb210b205482	fnv1a64:aa44bce864c36827
32	P150	fnv1a64:ee8eaa5a9bd8efba	fnv1a64:8af0feffe0220edf
33	P071	fnv1a64:0d0e3da33bd63106	fnv1a64:135db8f522fdd22b
34	P081	fnv1a64:7f3504739ce7b66f	fnv1a64:356f823f86c8fe8e
35	P091	fnv1a64:189ea16a15628b37	fnv1a64:7c40ef459425eb76
36	P101	fnv1a64:64f98aa5c914b5f8	fnv1a64:9d3777d2510ebec5
37	P111	fnv1a64:e0eb9b90a0ae905f	fnv1a64:11252677cfeaba3e
38	P121	fnv1a64:2a789ec6267f34d4	fnv1a64:c22bf7d60150ce61
39	P131	fnv1a64:41a1fde74081b52b	fnv1a64:fca68e48c3da67aa
40	P141	fnv1a64:5cbaa3a25689eb97	fnv1a64:a9a779b5a37bb9d6
41	P151	fnv1a64:c7f783bfcaf694a5	fnv1a64:06b9b4fedf804264
42	B13	fnv1a64:b7fb86affb1d4fd7	fnv1a64:24bf25c153491816
43	R01	fnv1a64:eb0407c19b96402a	fnv1a64:fd5b11634bad848f
44	R02	fnv1a64:5b6aadbcdee26661	fnv1a64:84103636527e8330
45	R03	fnv1a64:a006ec98da3c071e	fnv1a64:acec3864784603c3
46	R040	fnv1a64:807aaf60ec2ac8ca	fnv1a64:e3bb4f0f2bb28a2f
47	R050	fnv1a64:9d43edf79fcec783	fnv1a64:44e0332583deb682
48	R060	fnv1a64:be6971a2ade04a52	fnv1a64:17e9660d02cd1e77
49	R070	fnv1a64:e7a2cc8d7e89756f	fnv1a64:9ddd4a59686abd8e
50	R080	fnv1a64:56fd437b846255fa	fnv1a64:1f9389523350031f
51	R090	fnv1a64:976d1260eded08bb	fnv1a64:856360957138333a
52	R100	fnv1a64:77cb9b534b2a99f9	fnv1a64:4a3a07f9a2cff9c8
53	R110	fnv1a64:13117faf43a5f788	fnv1a64:680cad4db757a295
54	R041	fnv1a64:9e34e9da12450ed7	fnv1a64:0af888eb6a70d716
55	R051	fnv1a64:ebea92bbe1e2b79a	fnv1a64:2f1cb741202e423f
56	R061	fnv1a64:70513d8aa2c3e835	fnv1a64:f8eb6c8525038df4
57	R071	fnv1a64:7178b0f5144c978c	fnv1a64:40c2f306b8a05699
58	R081	fnv1a64:6fb868787975defb	fnv1a64:2a855044cb8d747a
59	R091	fnv1a64:51192339c62484c2	fnv1a64:b97bc35f6caffa67
60	R101	fnv1a64:d103cac95bc25612	fnv1a64:a772228de138c237
61	R111	fnv1a64:2b08e46fcee587dd	fnv1a64:f050068e3278e31c
62	R12	fnv1a64:8c7e814ed7b95a1d	fnv1a64:aef2e7c6a4df9e5c
63	R13	fnv1a64:d233e665deb76ba9	fnv1a64:cdf88af66306cb78
64	R14	fnv1a64:ce260d99ba7ac9fb	fnv1a64:88f2f5660c925f7a
65	R15	fnv1a64:4020736ecc0b2107	fnv1a64:03947c5173bde4c6
66	O010	fnv1a64:25fbb039427d2495	fnv1a64:37497299a68b3554
67	O0200	fnv1a64:1af49cf559d77a82	fnv1a64:a50c5ebcb37a8e27
68	O011	fnv1a64:00f22e2abb720ed2	fnv1a64:5a722295105ee2f7
69	O0210	fnv1a64:85deb18a149adb7d	fnv1a64:78a49536b1fbb23c
```

Intermediate refresh evidence is retained in [standing-candidate-01/fingerprint-row-changes.json](../../../.temp/brief-585/standing-candidate-01/fingerprint-row-changes.json), [standing-candidate-01/certificate-cell-changes-true.json](../../../.temp/brief-585/standing-candidate-01/certificate-cell-changes-true.json), [standing-candidate-01/certificate-cell-changes-false.json](../../../.temp/brief-585/standing-candidate-01/certificate-cell-changes-false.json); the final refresh deltas are [fingerprint-row-changes.json](../../../.temp/brief-585/fingerprint-row-changes.json), [certificate-cell-changes-true.json](../../../.temp/brief-585/certificate-cell-changes-true.json), [certificate-cell-changes-false.json](../../../.temp/brief-585/certificate-cell-changes-false.json).
Every promoted certificate cell:


- `promoted.tsv` row 2 (C01), column 5, hash=true: only fingerprint tokens changed; source anchor crates/rum-runtime/src/runtime/options/core.rs:106:5-122:6 -> crates/rum-runtime/src/runtime/options/core.rs:106:5-122:6.
- `promoted.tsv` row 3 (C02), column 5, hash=true: only fingerprint tokens changed; source anchor crates/rum-runtime/src/runtime/options/core.rs:224:5-238:6 -> crates/rum-runtime/src/runtime/options/core.rs:224:5-238:6.
Exact before/after cells: [certificate-cell-changes-from-incoming-true.json](../../../.temp/brief-585/certificate-cell-changes-from-incoming-true.json).

- `promoted.tsv` row 2 (C01), column 6, hash=false: only fingerprint tokens changed; source anchor crates/rum-runtime/src/runtime/options/core.rs:106:5-122:6 -> crates/rum-runtime/src/runtime/options/core.rs:106:5-122:6.
- `promoted.tsv` row 3 (C02), column 6, hash=false: only fingerprint tokens changed; source anchor crates/rum-runtime/src/runtime/options/core.rs:224:5-238:6 -> crates/rum-runtime/src/runtime/options/core.rs:224:5-238:6.
Exact before/after cells: [certificate-cell-changes-from-incoming-false.json](../../../.temp/brief-585/certificate-cell-changes-from-incoming-false.json).

Broad workspace gate (all result lines are verbatim; native ZeroMQ skips are listed separately):
[workspace-final.log](../../../.temp/brief-585/workspace-final.log)
Command: `python3 scripts/run-bounded.py --timeout 2400 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-585/tmp cargo test --workspace --no-fail-fast -- --nocapture` (exit 101).
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 3m 58s
     Running unittests src/main.rs (target/debug/deps/rum_cli-675a9b7fb4c09027)
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
     Running tests/cli.rs (target/debug/deps/cli-417da8e232f3f31f)
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s
     Running tests/input_requests.rs (target/debug/deps/input_requests-79d242612dc5614e)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
     Running tests/terminal.rs (target/debug/deps/terminal-cd41974a9dfe37a5)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.90s
     Running unittests src/lib.rs (target/debug/deps/rum_core-49fc2a810a19f202)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_derive-b50628eb478e3209)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/accumulator_chain.rs (target/debug/deps/accumulator_chain-f4b1762065518378)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/derive.rs (target/debug/deps/derive-b38e62f915ec5c7d)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_host-a119f062096badf7)
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.63s
     Running tests/adapter_contract.rs (target/debug/deps/adapter_contract-36e2ac93dcf516de)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/batch_boundary_ownership.rs (target/debug/deps/batch_boundary_ownership-62cec20e1429084a)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
     Running tests/guest_loading_resume.rs (target/debug/deps/guest_loading_resume-4d8893fc9e0cdd96)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
     Running tests/host_payload_destruction_contract.rs (target/debug/deps/host_payload_destruction_contract-1be17d3af592a343)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.33s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.21s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.60s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.43s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.18s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.69s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 45.68s
     Running tests/installer.rs (target/debug/deps/installer-8ebb99c00d41e78e)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
     Running tests/lifecycle_buildout.rs (target/debug/deps/lifecycle_buildout-5e85c48ede3e9593)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
     Running tests/lifecycle_contract.rs (target/debug/deps/lifecycle_contract-a2c563982853655a)
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
     Running tests/random_password_policy.rs (target/debug/deps/random_password_policy-ddd2fec076417963)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/stress.rs (target/debug/deps/stress-5ec2a75152f1305e)
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.26s
     Running tests/suspended_timeout_contract.rs (target/debug/deps/suspended_timeout_contract-258c488f1b37cde7)
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.03s
     Running tests/suspension_abandonment_contract.rs (target/debug/deps/suspension_abandonment_contract-ac93b53b9ae0d1d1)
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
     Running unittests src/lib.rs (target/debug/deps/rum_jupyter-edb9ee3aea58135e)
test result: FAILED. 142 passed; 9 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.69s
     Running unittests src/bin/rum-jupyter-kernel.rs (target/debug/deps/rum_jupyter_kernel-965588c9ac9ffa23)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_repl-edcfd3382d4bfc76)
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.39s
     Running tests/input_abandonment.rs (target/debug/deps/input_abandonment-144216c549618286)
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
     Running tests/input_continuation.rs (target/debug/deps/input_continuation-ebdbc4f7476ef252)
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
     Running unittests src/lib.rs (target/debug/deps/rum_runtime-562b258c6eef9b02)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.07s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.06s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.06s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.06s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.05s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.07s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.06s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.06s
test result: ok. 1138 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 27.57s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-c8c5a89643371dc7)
EXACT INVENTORY UnknownHostPayload: 65 rows
EXACT INVENTORY UnknownConstruct: 7683 rows
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::Unarmed;R=rum_stdlib::option_consumers::OptionConsumer;hash=true	0
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=rum_stdlib::option_consumers::OptionConsumer;hash=true	0
C02	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr;M=runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=none (nongeneric C02);hash=true	0
LeaseProbe538::probe	sealed modes	0
LeaseProbe538::start	sealed modes	0
LeaseProbe538::drive	sealed modes	0
LeaseProbe538::resume	sealed modes	0
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
PRODUCTION_BODY_ROWS=2 N=78 UNSPLIT=147 UNSETTLED=90 LOCAL_OPTION_BODIES=2 PENDING=315
test result: ok. 1087 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 1172.27s
     Running unittests src/bin/generate-builtin-reference.rs (target/debug/deps/generate_builtin_reference-7e0be655aec7b381)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/default_fallback_continuations.rs (target/debug/deps/default_fallback_continuations-c846b88aba46e50a)
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s
     Running tests/destructuring_continuations.rs (target/debug/deps/destructuring_continuations-dfe1523f84cb69ee)
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
     Running tests/guest_loading_resume.rs (target/debug/deps/guest_loading_resume-0992a185a5c665db)
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.05s
     Running tests/lazy_pull_continuations.rs (target/debug/deps/lazy_pull_continuations-4b49946100b6004b)
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.98s
     Running tests/loop_continuations.rs (target/debug/deps/loop_continuations-d0d938f953da2f0b)
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
     Running tests/option_reader_continuations.rs (target/debug/deps/option_reader_continuations-5a0cec5f31c0a5b0)
test result: ok. 100 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.99s
     Running tests/rule_pattern_continuations.rs (target/debug/deps/rule_pattern_continuations-c61abf30d2dd2405)
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.54s
     Running tests/schema_continuations.rs (target/debug/deps/schema_continuations-65e2f27929ed24fb)
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.12s
     Running tests/sow_trace_continuations.rs (target/debug/deps/sow_trace_continuations-93367493ef68b082)
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.45s
     Running unittests src/lib.rs (target/debug/deps/rum_syntax-e24f301251f42f42)
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.52s
     Running unittests src/lib.rs (target/debug/deps/rum_terminal-bae91ae0a1529dd6)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_core
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_derive
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.42s
   Doc-tests rum_host
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.01s
   Doc-tests rum_jupyter
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_repl
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_runtime
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.81s
   Doc-tests rum_stdlib
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_syntax
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_terminal
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

The first workspace run above had nine native ZeroMQ failures because loopback bind returned EPERM. The repository helper requires the explicit supported RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 flag; these first-run failures are not counted as skips. The second run uses that flag, without changing source or tests:
[workspace-final-skip.log](../../../.temp/brief-585/workspace-final-skip.log)
Command: `python3 scripts/run-bounded.py --timeout 2400 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-585/tmp env RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test --workspace --no-fail-fast -- --nocapture` (exit 0).
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.22s
     Running unittests src/main.rs (target/debug/deps/rum_cli-675a9b7fb4c09027)
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
     Running tests/cli.rs (target/debug/deps/cli-417da8e232f3f31f)
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.34s
     Running tests/input_requests.rs (target/debug/deps/input_requests-79d242612dc5614e)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
     Running tests/terminal.rs (target/debug/deps/terminal-cd41974a9dfe37a5)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.95s
     Running unittests src/lib.rs (target/debug/deps/rum_core-49fc2a810a19f202)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_derive-b50628eb478e3209)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/accumulator_chain.rs (target/debug/deps/accumulator_chain-f4b1762065518378)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/derive.rs (target/debug/deps/derive-b38e62f915ec5c7d)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_host-a119f062096badf7)
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
     Running tests/adapter_contract.rs (target/debug/deps/adapter_contract-36e2ac93dcf516de)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
     Running tests/batch_boundary_ownership.rs (target/debug/deps/batch_boundary_ownership-62cec20e1429084a)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
     Running tests/guest_loading_resume.rs (target/debug/deps/guest_loading_resume-4d8893fc9e0cdd96)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
     Running tests/host_payload_destruction_contract.rs (target/debug/deps/host_payload_destruction_contract-1be17d3af592a343)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.14s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.13s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.14s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.14s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.16s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.18s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.60s
     Running tests/installer.rs (target/debug/deps/installer-8ebb99c00d41e78e)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
     Running tests/lifecycle_buildout.rs (target/debug/deps/lifecycle_buildout-5e85c48ede3e9593)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
     Running tests/lifecycle_contract.rs (target/debug/deps/lifecycle_contract-a2c563982853655a)
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
     Running tests/random_password_policy.rs (target/debug/deps/random_password_policy-ddd2fec076417963)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
     Running tests/stress.rs (target/debug/deps/stress-5ec2a75152f1305e)
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.35s
     Running tests/suspended_timeout_contract.rs (target/debug/deps/suspended_timeout_contract-258c488f1b37cde7)
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.07s
     Running tests/suspension_abandonment_contract.rs (target/debug/deps/suspension_abandonment_contract-ac93b53b9ae0d1d1)
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s
     Running unittests src/lib.rs (target/debug/deps/rum_jupyter-edb9ee3aea58135e)
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.82s
     Running unittests src/bin/rum-jupyter-kernel.rs (target/debug/deps/rum_jupyter_kernel-965588c9ac9ffa23)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_repl-edcfd3382d4bfc76)
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.42s
     Running tests/input_abandonment.rs (target/debug/deps/input_abandonment-144216c549618286)
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
     Running tests/input_continuation.rs (target/debug/deps/input_continuation-ebdbc4f7476ef252)
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
     Running unittests src/lib.rs (target/debug/deps/rum_runtime-562b258c6eef9b02)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.19s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.06s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.10s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.06s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.05s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.07s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.07s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.07s
test result: ok. 1138 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 29.64s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-c8c5a89643371dc7)
EXACT INVENTORY UnknownHostPayload: 65 rows
EXACT INVENTORY UnknownConstruct: 7683 rows
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::Unarmed;R=rum_stdlib::option_consumers::OptionConsumer;hash=true	0
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=rum_stdlib::option_consumers::OptionConsumer;hash=true	0
C02	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr;M=runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=none (nongeneric C02);hash=true	0
LeaseProbe538::probe	sealed modes	0
LeaseProbe538::start	sealed modes	0
LeaseProbe538::drive	sealed modes	0
LeaseProbe538::resume	sealed modes	0
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
PRODUCTION_BODY_ROWS=2 N=78 UNSPLIT=147 UNSETTLED=90 LOCAL_OPTION_BODIES=2 PENDING=315
test result: ok. 1087 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 496.91s
     Running unittests src/bin/generate-builtin-reference.rs (target/debug/deps/generate_builtin_reference-7e0be655aec7b381)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/default_fallback_continuations.rs (target/debug/deps/default_fallback_continuations-c846b88aba46e50a)
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
     Running tests/destructuring_continuations.rs (target/debug/deps/destructuring_continuations-dfe1523f84cb69ee)
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
     Running tests/guest_loading_resume.rs (target/debug/deps/guest_loading_resume-0992a185a5c665db)
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.98s
     Running tests/lazy_pull_continuations.rs (target/debug/deps/lazy_pull_continuations-4b49946100b6004b)
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.28s
     Running tests/loop_continuations.rs (target/debug/deps/loop_continuations-d0d938f953da2f0b)
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s
     Running tests/option_reader_continuations.rs (target/debug/deps/option_reader_continuations-5a0cec5f31c0a5b0)
test result: ok. 100 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.85s
     Running tests/rule_pattern_continuations.rs (target/debug/deps/rule_pattern_continuations-c61abf30d2dd2405)
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.47s
     Running tests/schema_continuations.rs (target/debug/deps/schema_continuations-65e2f27929ed24fb)
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.03s
     Running tests/sow_trace_continuations.rs (target/debug/deps/sow_trace_continuations-93367493ef68b082)
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.38s
     Running unittests src/lib.rs (target/debug/deps/rum_syntax-e24f301251f42f42)
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.50s
     Running unittests src/lib.rs (target/debug/deps/rum_terminal-bae91ae0a1529dd6)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_core
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_derive
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
   Doc-tests rum_host
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.96s
   Doc-tests rum_jupyter
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_repl
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_runtime
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.55s
   Doc-tests rum_stdlib
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_syntax
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_terminal
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

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

```text
{
  "summary": {
    "groups": 48,
    "passed": 3351,
    "failed": 0,
    "ignored": 58,
    "native_capability_skips": 9,
    "cargo_pass_count_includes_capability_skips": true
  },
  "groups": [
    {
      "group": "Running unittests src/main.rs (target/debug/deps/rum_cli-675a9b7fb4c09027)",
      "result": "ok",
      "passed": 15,
      "failed": 0,
      "ignored": 0,
      "verbatim": "test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s"
    },
    {
      "group": "Running tests/cli.rs (target/debug/deps/cli-417da8e232f3f31f)",
      "result": "ok",
      "passed": 28,
      "failed": 0,
      "ignored": 0,
      "verbatim": "test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.34s"
    },
    {
      "group": "Running tests/input_requests.rs (target/debug/deps/input_requests-79d242612dc5614e)",
      "result": "ok",
      "passed": 6,
      "failed": 0,
      "ignored": 0,
      "verbatim": "test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s"
    },
    {
      "group": "Running tests/terminal.rs (target/debug/deps/terminal-cd41974a9dfe37a5)",
      "result": "ok",
      "passed": 9,
      "failed": 0,
      "ignored": 0,
      "verbatim": "test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.95s"
    },
    {
      "group": "Running unittests src/lib.rs (target/debug/deps/rum_core-49fc2a810a19f202)",
      "result": "ok",
      "passed": 7,
      "failed": 0,
      "ignored": 0,
      "verbatim": "test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s"
    },
    {
      "group": "Running unittests src/lib.rs (target/debug/deps/rum_derive-b50628eb478e3209)",
      "result": "ok",
      "passed": 5,
      "failed": 0,
      "ignored": 0,
      "verbatim": "test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s"
    },
    {
      "group": "Running tests/accumulator_chain.rs (target/debug/deps/accumulator_chain-f4b1762065518378)",
      "result": "ok",
      "passed": 3,
      "failed": 0,
      "ignored": 0,
      "verbatim": "test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s"
    },
    {
      "group": "Running tests/derive.rs (target/debug/deps/derive-b38e62f915ec5c7d)",
      "result": "ok",
      "passed": 3,
      "failed": 0,
      "ignored": 0,
      "verbatim": "test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s"
    },
    {
      "group": "Running unittests src/lib.rs (target/debug/deps/rum_host-a119f062096badf7)",
      "result": "ok",
      "passed": 90,
      "failed": 0,
      "ignored": 0,
      "verbatim": "test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s"
    },
    {
      "group": "Running tests/adapter_contract.rs (target/debug/deps/adapter_contract-36e2ac93dcf516de)",
      "result": "ok",
      "passed": 7,
      "failed": 0,
      "ignored": 0,
      "verbatim": "test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s"
    },
    {
      "group": "Running tests/batch_boundary_ownership.rs (target/debug/deps/batch_boundary_ownership-62cec20e1429084a)",
      "result": "ok",
      "passed": 5,
      "failed": 0,
      "ignored": 0,
      "verbatim": "test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s"
    },
    {
      "group": "Running tests/guest_loading_resume.rs (target/debug/deps/guest_loading_resume-4d8893fc9e0cdd96)",
      "result": "ok",
      "passed": 4,
      "failed": 0,
      "ignored": 0,
      "verbatim": "test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s"
    },
    {
      "group": "Running tests/host_payload_destruction_contract.rs (target/debug/deps/host_payload_destruction_contract-1be17d3af592a343)",
      "result": "ok",
      "passed": 1,
      "failed": 0,
      "ignored": 0,
      "verbatim": "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.60s"
    },
    {
      "group": "Running tests/installer.rs (target/debug/deps/installer-8ebb99c00d41e78e)",
      "result": "ok",
      "passed": 4,
      "failed": 0,
      "ignored": 0,
      "verbatim": "test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s"
    },
    {
      "group": "Running tests/lifecycle_buildout.rs (target/debug/deps/lifecycle_buildout-5e85c48ede3e9593)",
      "result": "ok",
      "passed": 7,
      "failed": 0,
      "ignored": 0,
      "verbatim": "test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s"
    },
    {
      "group": "Running tests/lifecycle_contract.rs (target/debug/deps/lifecycle_contract-a2c563982853655a)",
      "result": "ok",
      "passed": 16,
      "failed": 0,
      "ignored": 0,
      "verbatim": "test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s"
    },
    {
      "group": "Running tests/random_password_policy.rs (target/debug/deps/random_password_policy-ddd2fec076417963)",
      "result": "ok",
      "passed": 2,
      "failed": 0,
      "ignored": 0,
      "verbatim": "test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s"
    },
    {
      "group": "Running tests/stress.rs (target/debug/deps/stress-5ec2a75152f1305e)",
      "result": "ok",
      "passed": 4,
      "failed": 0,
      "ignored": 2,
      "verbatim": "test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.35s"
    },
    {
      "group": "Running tests/suspended_timeout_contract.rs (target/debug/deps/suspended_timeout_contract-258c488f1b37cde7)",
      "result": "ok",
      "passed": 13,
      "failed": 0,
      "ignored": 0,
      "verbatim": "test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.07s"
    },
    {
      "group": "Running tests/suspension_abandonment_contract.rs (target/debug/deps/suspension_abandonment_contract-ac93b53b9ae0d1d1)",
      "result": "ok",
      "passed": 12,
      "failed": 0,
      "ignored": 0,
      "verbatim": "test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s"
    },
    {
      "group": "Running unittests src/lib.rs (target/debug/deps/rum_jupyter-edb9ee3aea58135e)",
      "result": "ok",
      "passed": 151,
      "failed": 0,
      "ignored": 0,
      "verbatim": "test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.82s"
    },
    {
      "group": "Running unittests src/bin/rum-jupyter-kernel.rs (target/debug/deps/rum_jupyter_kernel-965588c9ac9ffa23)",
      "result": "ok",
      "passed": 0,
      "failed": 0,
      "ignored": 0,
      "verbatim": "test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s"
    },
    {
      "group": "Running unittests src/lib.rs (target/debug/deps/rum_repl-edcfd3382d4bfc76)",
      "result": "ok",
      "passed": 48,
      "failed": 0,
      "ignored": 0,
      "verbatim": "test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.42s"
    },
    {
      "group": "Running tests/input_abandonment.rs (target/debug/deps/input_abandonment-144216c549618286)",
      "result": "ok",
      "passed": 10,
      "failed": 0,
      "ignored": 0,
      "verbatim": "test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s"
    },
    {
      "group": "Running tests/input_continuation.rs (target/debug/deps/input_continuation-ebdbc4f7476ef252)",
      "result": "ok",
      "passed": 12,
      "failed": 0,
      "ignored": 0,
      "verbatim": "test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s"
    },
    {
      "group": "Running unittests src/lib.rs (target/debug/deps/rum_runtime-562b258c6eef9b02)",
      "result": "ok",
      "passed": 1138,
      "failed": 0,
      "ignored": 3,
      "verbatim": "test result: ok. 1138 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 29.64s"
    },
    {
      "group": "Running unittests src/lib.rs (target/debug/deps/rum_stdlib-c8c5a89643371dc7)",
      "result": "ok",
      "passed": 1087,
      "failed": 0,
      "ignored": 53,
      "verbatim": "test result: ok. 1087 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 496.91s"
    },
    {
      "group": "Running unittests src/bin/generate-builtin-reference.rs (target/debug/deps/generate_builtin_reference-7e0be655aec7b381)",
      "result": "ok",
      "passed": 0,
      "failed": 0,
      "ignored": 0,
      "verbatim": "test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s"
    },
    {
      "group": "Running tests/default_fallback_continuations.rs (target/debug/deps/default_fallback_continuations-c846b88aba46e50a)",
      "result": "ok",
      "passed": 21,
      "failed": 0,
      "ignored": 0,
      "verbatim": "test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s"
    },
    {
      "group": "Running tests/destructuring_continuations.rs (target/debug/deps/destructuring_continuations-dfe1523f84cb69ee)",
      "result": "ok",
      "passed": 16,
      "failed": 0,
      "ignored": 0,
      "verbatim": "test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s"
    },
    {
      "group": "Running tests/guest_loading_resume.rs (target/debug/deps/guest_loading_resume-0992a185a5c665db)",
      "result": "ok",
      "passed": 47,
      "failed": 0,
      "ignored": 0,
      "verbatim": "test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.98s"
    },
    {
      "group": "Running tests/lazy_pull_continuations.rs (target/debug/deps/lazy_pull_continuations-4b49946100b6004b)",
      "result": "ok",
      "passed": 69,
      "failed": 0,
      "ignored": 0,
      "verbatim": "test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.28s"
    },
    {
      "group": "Running tests/loop_continuations.rs (target/debug/deps/loop_continuations-d0d938f953da2f0b)",
      "result": "ok",
      "passed": 21,
      "failed": 0,
      "ignored": 0,
      "verbatim": "test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s"
    },
    {
      "group": "Running tests/option_reader_continuations.rs (target/debug/deps/option_reader_continuations-5a0cec5f31c0a5b0)",
      "result": "ok",
      "passed": 100,
      "failed": 0,
      "ignored": 0,
      "verbatim": "test result: ok. 100 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.85s"
    },
    {
      "group": "Running tests/rule_pattern_continuations.rs (target/debug/deps/rule_pattern_continuations-c61abf30d2dd2405)",
      "result": "ok",
      "passed": 71,
      "failed": 0,
      "ignored": 0,
      "verbatim": "test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.47s"
    },
    {
      "group": "Running tests/schema_continuations.rs (target/debug/deps/schema_continuations-65e2f27929ed24fb)",
      "result": "ok",
      "passed": 40,
      "failed": 0,
      "ignored": 0,
      "verbatim": "test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.03s"
    },
    {
      "group": "Running tests/sow_trace_continuations.rs (target/debug/deps/sow_trace_continuations-93367493ef68b082)",
      "result": "ok",
      "passed": 51,
      "failed": 0,
      "ignored": 0,
      "verbatim": "test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.38s"
    },
    {
      "group": "Running unittests src/lib.rs (target/debug/deps/rum_syntax-e24f301251f42f42)",
      "result": "ok",
      "passed": 193,
      "failed": 0,
      "ignored": 0,
      "verbatim": "test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.50s"
    },
    {
      "group": "Running unittests src/lib.rs (target/debug/deps/rum_terminal-bae91ae0a1529dd6)",
      "result": "ok",
      "passed": 5,
      "failed": 0,
      "ignored": 0,
      "verbatim": "test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s"
    },
    {
      "group": "Doc-tests rum_core",
      "result": "ok",
      "passed": 0,
      "failed": 0,
      "ignored": 0,
      "verbatim": "test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s"
    },
    {
      "group": "Doc-tests rum_derive",
      "result": "ok",
      "passed": 7,
      "failed": 0,
      "ignored": 0,
      "verbatim": "test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s"
    },
    {
      "group": "Doc-tests rum_host",
      "result": "ok",
      "passed": 1,
      "failed": 0,
      "ignored": 0,
      "verbatim": "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.96s"
    },
    {
      "group": "Doc-tests rum_jupyter",
      "result": "ok",
      "passed": 0,
      "failed": 0,
      "ignored": 0,
      "verbatim": "test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s"
    },
    {
      "group": "Doc-tests rum_repl",
      "result": "ok",
      "passed": 0,
      "failed": 0,
      "ignored": 0,
      "verbatim": "test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s"
    },
    {
      "group": "Doc-tests rum_runtime",
      "result": "ok",
      "passed": 22,
      "failed": 0,
      "ignored": 0,
      "verbatim": "test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.55s"
    },
    {
      "group": "Doc-tests rum_stdlib",
      "result": "ok",
      "passed": 0,
      "failed": 0,
      "ignored": 0,
      "verbatim": "test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s"
    },
    {
      "group": "Doc-tests rum_syntax",
      "result": "ok",
      "passed": 0,
      "failed": 0,
      "ignored": 0,
      "verbatim": "test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s"
    },
    {
      "group": "Doc-tests rum_terminal",
      "result": "ok",
      "passed": 0,
      "failed": 0,
      "ignored": 0,
      "verbatim": "test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s"
    }
  ],
  "native_skips": [
    "test zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip",
    "test zeromq::tests::native::basic::native_zeromq_backend_round_trips_execute_over_loopback_sockets ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip",
    "test zeromq::tests::native::frontend::control::native_zeromq_backend_frontend_style_control_interrupt_round_trip ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip",
    "test zeromq::tests::native::frontend::control::native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip",
    "test zeromq::tests::native::frontend::execute::native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip",
    "test zeromq::tests::native::frontend::session_flow::native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip",
    "test zeromq::tests::native::frontend::stdin::native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip",
    "test zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip",
    "test zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip"
  ]
}
```

Workspace aggregate: [workspace-summary.json](../../../.temp/brief-585/workspace-summary.json). Earlier continuation groups are included in this workspace gate.

### 3. Repaired obligation and every measured row

Prelude decisions and direct lookup snapshots/order are unchanged. The resumed caller never restarts the direct lookup: it decodes the child, then either completes or enters the byte-identical ordinary suffix. The normal evaluator may later revisit an unchanged call; the terminal-None fixture pins the eager log `{1,2,3,4}` on both paths. After the direct lookup declines, the middle-candidate fixture declines one ordinary pattern before accepting the next, preserving `{1,8,9,90}`.

Live-definition observation: the red guard that installs `Default[x_]` yielded queued log `{1,2,9}` and resumed `{1,2,3,9}`. The scoped caller repair makes both `{1,2,9}` with value `9`, using the unchanged eager precedence and live-definition reads. No remaining semantic conflict was observed in that fixture; no precedence repair was made. Nested caller effects preserve queued order `{1,2,4,3}`.

Noise floors were established first from the supplied base binaries, against themselves, with two instruction pairs. Allocation counts precede the candidate instruction panel. Each process runs one checked label, stdin `/dev/null`; instructions use `taskset -c 2 perf stat -x, -e cpu_core/instructions/u` and slopes `(count(2N)-count(N))/N`. A label regresses only when both candidate samples exceed `max(+0.05%, floor)`. No timing suite was run, as this brief directs. Base binaries were never rebuilt.

```text
75566f4023256c11610f42abaa43f9b7829278f8d8ca09021275dff1b702fbf4  rum-cli
3bc29c1ecad109c5c8b8ce74221979d38ede2436171d5d1ff2849bbe7ec5c5d5  rum-cli-alloc
```

```text
label	N	floor	instructions1_base	instructions1_new	instructions1_pct	instructions2_base	instructions2_new	instructions2_pct	regression
direct-default-hit	20000	0.0011981563554530616	9510.3794	9510.4115	0.0003375259666427155	9510.4449	9510.33095	-0.0011981563554530616	False
direct-default-miss	20000	0.001446086357703713	9505.1331	9505.0281	-0.0011046662776292848	9504.96485	9505.1023	0.001446086357703713	False
direct-default-decline-fallback	20000	0.6949470611292696	28491.93285	28293.929	-0.6949470611292696	28359.73795	28359.858	0.00042331138676576785	False
ordinary-downvalue-hit	20000	0.004335507788111315	5317.7162	5317.94675	0.004335507788111315	5317.9401	5317.9574	0.0003253139312375808	False
ordinary-downvalue-miss	20000	0.05636471315141334	389.97515	390.18315	0.053336731840492035	390.049	389.82915	-0.05636471315141334	False
ordinary-downvalue-guarded-hit	20000	0.00017024321576908363	18209.2425	18209.2735	0.00017024321576908363	18209.1141	18209.1205	3.514723432296307e-05	False
optional-default-consumer	20000	0.00022175513783029288	27304.8916	27304.95215	0.00022175513783029288	27304.9512	27304.9528	5.8597431129570055e-06	False
ownvalue-read	20000	0.013279109767461161	1559.4168	1559.4631	0.0029690586891284454	1559.5925	1559.3854	-0.013279109767461161	False
dot-three-no-input	20000	0.00012765575070350366	254200.11025	254199.8687	-9.502356225210562e-05	254199.28065	254199.60515	0.00012765575070350366	False
differences-no-input	20000	0.21113764206086572	10044.94735	10030.7323	-0.1415144301378568	10051.9499	10030.72645	-0.21113764206086572	False
take-multispec-small	20000	0.03269692415095449	49922.86335	49922.88705	4.7473238518058736e-05	49915.0927	49931.4134	0.03269692415095449	False
```

Measured candidate-01: [candidate-01/hashes.json](../../../.temp/brief-585/candidate-01/hashes.json); raw-first records [candidate-01/perf/paired-results.json](../../../.temp/brief-585/candidate-01/perf/paired-results.json).
```text
label	N	floor	alloc1_base	alloc1_new	bytes1_base	bytes1_new	instructions1_base	instructions1_new	instructions1_pct	instructions2_base	instructions2_new	instructions2_pct	regression
direct-default-hit	20000	0.0011981563554530616	18.0	18.0	1592.0	1592.0	9510.37015	9511.36365	0.010446491401805424	9510.6541	9511.66875	0.010668561692317446	False
direct-default-miss	20000	0.001446086357703713	18.0	18.0	1592.0	1592.0	9505.12955	9493.21445	-0.12535441981430528	9504.98545	9493.06655	-0.12539629926524576	False
direct-default-decline-fallback	20000	0.6949470611292696	45.0	45.0	4268.0	4268.0	28360.0253	28352.7315	-0.02571859482790595	28359.938	28352.75265	-0.02533626836560554	False
ordinary-downvalue-hit	20000	0.004335507788111315	3.0	3.0	216.0	216.0	5317.98025	5330.96605	0.24418669099044443	5317.98195	5330.904	0.24298784993055378	True
ordinary-downvalue-miss	20000	0.05636471315141334	0.0	0.0	0.0	0.0	389.99995	389.96315	-0.009435898645637764	389.9656	390.15355	0.048196558875956974	False
ordinary-downvalue-guarded-hit	20000	0.00017024321576908363	21.0	21.0	2936.0	2936.0	18209.15515	18222.28715	0.07211756883735987	18209.35765	18222.14465	0.07022213658369747	True
optional-default-consumer	20000	0.00022175513783029288	42.0	42.0	4728.0	4728.0	27304.93225	27317.5672	0.04627350796668761	27304.9972	27317.42325	0.04550833647403163	False
ownvalue-read	20000	0.013279109767461161	1.0	1.0	128.0	128.0	1559.4861	1559.50705	0.0013433912620319788	1559.5625	1559.49255	-0.00448523223660624	False
dot-three-no-input	20000	0.00012765575070350366	596.0	596.0	23504.0	23504.0	254200.41595	254198.9275	-0.0005855419214939594	254199.24865	254198.54015	-0.00027871836906223635	False
differences-no-input	20000	0.21113764206086572	13.0	13.0	704.0	704.0	10037.99725	10033.44335	-0.04536661932240671	10044.9921	10033.80795	-0.11134055545946042	False
take-multispec-small	20000	0.03269692415095449	109.0	109.0	5856.0	5856.0	49923.43995	49922.1067	-0.002670589208875196	49931.6139	49922.3469	-0.01855938407790969	False
```

```text
{
  "candidate": "candidate-01",
  "labels": 11,
  "regressions": [
    "ordinary-downvalue-hit",
    "ordinary-downvalue-guarded-hit"
  ],
  "allocation_regressions": [],
  "byte_regressions": [],
  "worst": 0.24418669099044443,
  "worst_residual": 0.24418669099044443
}
```

Measured candidate-02: [candidate-02/hashes.json](../../../.temp/brief-585/candidate-02/hashes.json); raw-first records [candidate-02/perf/paired-results.json](../../../.temp/brief-585/candidate-02/perf/paired-results.json).
```text
label	N	floor	alloc1_base	alloc1_new	bytes1_base	bytes1_new	instructions1_base	instructions1_new	instructions1_pct	instructions2_base	instructions2_new	instructions2_pct	regression
direct-default-hit	20000	0.0011981563554530616	18.0	18.0	1592.0	1592.0	9510.31325	9514.2903	0.0418182860590921	9510.3168	9514.25465	0.04140608649336741	False
direct-default-miss	20000	0.001446086357703713	18.0	18.0	1592.0	1592.0	9505.0986	9495.21935	-0.10393632318553836	9505.07345	9567.3458	0.6551485406985469	False
direct-default-decline-fallback	20000	0.6949470611292696	45.0	45.0	4268.0	4268.0	28359.93565	28354.8914	-0.017786535421837613	28359.79515	28354.90565	-0.01724095669287662	False
ordinary-downvalue-hit	20000	0.004335507788111315	3.0	3.0	216.0	216.0	5318.13495	5331.98175	0.26036947407661604	5317.86065	5331.7389	0.2609743074031279	True
ordinary-downvalue-miss	20000	0.05636471315141334	0.0	0.0	0.0	0.0	389.9879	390.09305	0.026962374986494453	390.127	390.04155	-0.021903123854549733	False
ordinary-downvalue-guarded-hit	20000	0.00017024321576908363	21.0	21.0	2936.0	2936.0	18209.19065	18223.24455	0.0771802562240742	18209.2233	18223.2976	0.07729214897376213	True
optional-default-consumer	20000	0.00022175513783029288	42.0	42.0	4728.0	4728.0	27305.03785	27318.43415	0.04906164230056298	27304.9361	27318.54245	0.049831099952668545	False
ownvalue-read	20000	0.013279109767461161	1.0	1.0	128.0	128.0	1559.57035	1559.2619	-0.019777883056060208	1559.5093	1559.5471	0.002423839344856127	False
dot-three-no-input	20000	0.00012765575070350366	596.0	596.0	23504.0	23504.0	254200.0882	254199.3076	-0.0003070809320049328	254200.1071	254197.7753	-0.0009173088188596523	False
differences-no-input	20000	0.21113764206086572	13.0	13.0	704.0	704.0	10045.12145	10035.25855	-0.09818597066340029	10051.9813	10033.7712	-0.18115931035407318	False
take-multispec-small	20000	0.03269692415095449	109.0	109.0	5856.0	5856.0	49939.4529	49922.05835	-0.03483127865823166	49931.3962	49921.8744	-0.01906976516711234	False
```

```text
{
  "candidate": "candidate-02",
  "labels": 11,
  "regressions": [
    "ordinary-downvalue-hit",
    "ordinary-downvalue-guarded-hit"
  ],
  "allocation_regressions": [],
  "byte_regressions": [],
  "worst": 0.6551485406985469,
  "worst_residual": 0.2609743074031279
}
```

Measured candidate-03: [candidate-03/hashes.json](../../../.temp/brief-585/candidate-03/hashes.json); raw-first records [candidate-03/perf/paired-results.json](../../../.temp/brief-585/candidate-03/perf/paired-results.json).
```text
label	N	floor	alloc1_base	alloc1_new	bytes1_base	bytes1_new	instructions1_base	instructions1_new	instructions1_pct	instructions2_base	instructions2_new	instructions2_pct	regression
direct-default-hit	20000	0.0011981563554530616	18.0	18.0	1592.0	1592.0	9510.4897	9516.34145	0.061529428920992224	9510.19665	9516.45405	0.06579674669504598	True
direct-default-miss	20000	0.001446086357703713	18.0	18.0	1592.0	1592.0	9504.9813	9497.8837	-0.0746724246579955	9505.251	9498.06815	-0.07556717860476292	False
direct-default-decline-fallback	20000	0.6949470611292696	45.0	45.0	4268.0	4268.0	28401.83685	28357.9117	-0.15465601831312625	28359.84305	28357.90485	-0.006834311447290187	False
ordinary-downvalue-hit	20000	0.004335507788111315	3.0	3.0	216.0	216.0	5317.8332	5331.84105	0.26341273735324133	5317.96845	5331.89125	0.26180674313702745	True
ordinary-downvalue-miss	20000	0.05636471315141334	0.0	0.0	0.0	0.0	390.149	389.98825	-0.041202207361801246	390.00305	390.1293	0.032371541709741436	False
ordinary-downvalue-guarded-hit	20000	0.00017024321576908363	21.0	21.0	2936.0	2936.0	18209.35805	18223.15055	0.075744021080415	18209.29035	18223.0818	0.07573853640046035	True
optional-default-consumer	20000	0.00022175513783029288	42.0	42.0	4728.0	4728.0	27305.0101	27318.7032	0.05014867216621699	27304.91485	27318.5245	0.04984322446990319	False
ownvalue-read	20000	0.013279109767461161	1.0	1.0	128.0	128.0	1559.4928	1559.5004	0.00048733793447386375	1559.38915	1559.4903	0.006486514286696554	False
dot-three-no-input	20000	0.00012765575070350366	596.0	596.0	23504.0	23504.0	254200.524	254198.0807	-0.0009611703239542635	254200.45025	254198.9147	-0.000604070527210343	False
differences-no-input	20000	0.21113764206086572	13.0	13.0	704.0	704.0	10052.0864	10033.9416	-0.18050779985336884	10038.1613	10034.25565	-0.038908021930272874	False
take-multispec-small	20000	0.03269692415095449	109.0	109.0	5856.0	5856.0	49922.90675	49921.66565	-0.0024860331274689784	49939.28145	49921.9568	-0.03469142826443283	False
```

```text
{
  "candidate": "candidate-03",
  "labels": 11,
  "regressions": [
    "direct-default-hit",
    "ordinary-downvalue-hit",
    "ordinary-downvalue-guarded-hit"
  ],
  "allocation_regressions": [],
  "byte_regressions": [],
  "worst": 0.26341273735324133,
  "worst_residual": 0.26341273735324133
}
```

Retained current candidate: `candidate-03` (**not accepted**). [selection.json](../../../.temp/brief-585/selection.json). All source, binary and workload hashes checked against the saved measured state; every raw stdout/stderr/perf CSV is retained below each panel directory.

Cold-interface investigation after candidate 02: bounded callgrind on the same ordinary-hit statement at N=1000/2000 attributed the +14 instructions to lost inlining of the existing indexed-scan helper (+23 self instructions there, -9 in try_apply_down_values). No candidate-cache code was changed. [diagnostics/self-cost-deltas.tsv](../../../.temp/brief-585/diagnostics/self-cost-deltas.tsv).
```text
function	base_self_slope	new_self_slope	delta
rum_runtime::runtime::model::definitions::down_value_dispatch::<impl rum_runtime::runtime::model::definitions::SymbolDefinition>::start_indexed_down_value_scan	0	23.0	23.0
rum_runtime::runtime::definitions::application::down_values::<impl rum_runtime::runtime::model::session::Session>::try_apply_down_values	312.0	303.0	-9.0
rum_core::expr::Symbol::system	0.808	-1.137	-1.945
rum_core::expr::intern_symbol_text	-1.148	0.54	1.688
0x00000000001b1220	-0.248	0.154	0.402
std::sync::poison::once::Once::call_once_force::{{closure}}	0.113	0.216	0.103
rum_core::expr::intern_text	0.58	0.547	-0.03299999999999992
rum_runtime::runtime::builtin_catalog::default_builtin_dispatch_spec_for_symbol	-0.022	0.0	0.022
```

Callgrind raw files: [diagnostics/base-1000.callgrind](../../../.temp/brief-585/diagnostics/base-1000.callgrind), [diagnostics/base-2000.callgrind](../../../.temp/brief-585/diagnostics/base-2000.callgrind), [diagnostics/candidate-02-1000.callgrind](../../../.temp/brief-585/diagnostics/candidate-02-1000.callgrind), [diagnostics/candidate-02-2000.callgrind](../../../.temp/brief-585/diagnostics/candidate-02-2000.callgrind). Candidate 02 changed only the cold argument interface; candidate 03 removed the request/result transfer through the cold helper, forwarding the original error in the caller. All compiling candidates retained 21/21 focused fixtures.
### 4. Stop-loss counters, residuals and open questions

```text
{
  "candidate": "candidate-03",
  "status": "STOP_LOSS_PERFORMANCE_GATE_FAILED",
  "accepted": false,
  "reason": "Two consecutive compiling measured candidates improved none of repaired sites, passing fixtures, regressing-label count, or worst regression. No further production edits.",
  "compiling_candidates": 3,
  "measured_candidates": 3,
  "measured_candidate_cap": 8,
  "consecutive_no_progress": 2,
  "lost_before": 1,
  "lost_after": 0,
  "focused_fixtures_passed": 21,
  "focused_fixtures_total": 21,
  "regressing_labels": [
    "direct-default-hit",
    "ordinary-downvalue-hit",
    "ordinary-downvalue-guarded-hit"
  ],
  "worst_regression_percent": 0.26341273735324133,
  "allocation_regressions": [],
  "byte_regressions": [],
  "best_measured_candidate": "candidate-01",
  "best_measured_worst_regression_percent": 0.24418669099044443,
  "remaining": "Performance gate unresolved; implementation stopped by the brief. Mandatory verification completed. Director review required before any acceptance or further tuning.",
  "final_verification": {
    "format": "passed",
    "focused_default": "21 passed",
    "focused_no_default": "21 passed",
    "runtime": "1138 passed, 3 ignored; 22 doc tests passed",
    "ownership": "20 passed; 65 host payloads and 7683 unknown constructs",
    "certificates": "both feature configurations passed, 3 production instance marks each",
    "workspace": {
      "groups": 48,
      "passed": 3351,
      "failed": 0,
      "ignored": 58,
      "native_capability_skips": 9,
      "cargo_pass_count_includes_capability_skips": true
    },
    "workspace_runs": 2
  }
}
```

Implementation tuning stopped after two consecutive non-improving candidates. Three performance labels remain regressed; the best earlier candidate (01) also failed two labels. No further production change or speculative remeasurement was attempted. Mandatory final verification was completed on the current compiling, unaccepted candidate. [final-audit.json](../../../.temp/brief-585/final-audit.json) records the final scope/hash audit. Changes are left uncommitted for the director's authoritative gate.
