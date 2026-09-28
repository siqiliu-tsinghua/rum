# brief-550 — implement resumable guest package loading, atomically across its callers

Implementation brief. **Do not run `git` in any form — not `status`, not
`diff`, not `log` — at any point, including while first loading this
repository.** Everything you need about the tree's state is in this brief.

## Goal

When a file loaded by guest `Get`, `Needs`, `BeginPackage`'s dependency list or
a `DeclarePackage` stub's autoload requests input, evaluation must **suspend**,
and `resume_eval_request_with_input` must continue the load at the form that
asked, finish the file, and then finish the builtin that loaded it — exactly as
if the input had been queued up front. Today the loader converts that
suspension into `$Failed` and leaks the pending suspension owner, which wedges
the session and lets a second input request fail the kernel's assertion
(`suspension/core.rs:228`).

The design of record is **brief-549's report §3-§6**
(`docs/dev/codex/brief-549-resumable-guest-loading.md`, from line 277), as
amended below. Implement the whole guest chain — the driver and all four
callers — in this one brief: a `Get` that suspends correctly while `Needs`,
`BeginPackage` or the stub still discard their completion work would move the
defect, not remove it.

## Rulings and director decisions — do not reopen

- **Guest loading is resumable** (user ruling, 2026-09-28).
- **The host batch APIs are not part of this brief.** `HostSession::eval_script`
  / `eval_file` and the public `Session::eval_script` / `eval_file` keep their
  documented non-resumability (`crates/rum-host/src/session.rs:378-412`,
  `docs/guide/embedding.md:364-380`). Guest loading gets its own internal driver
  (brief-549 §5); **do not route guest loading through the public batch entry,
  and do not make batch resumable.** The batch path's own owner leak is the
  next brief's work; see the fence.
- **A guest load inherits the enclosing request's resource budget.** This
  answers brief-549's open question 2, and it is a correctness fix, not a
  policy change: the director measured that the per-form
  `begin_top_level_interrupt_budget` call (`session/session_helpers.rs:30-38`,
  reached through `eval_input_with_terminated_handling`, `session_eval.rs:215`)
  lets guest code escape its limits by loading a file.
  - `MemoryConstrained[(x = Range[50000]; Length[x]), 100000]` returns
    `$Aborted`; `MemoryConstrained[Get["Mem`"], 100000]`, where `Mem.rum` is
    `x = Range[50000];` and `Length[x]`, returns **`50000`**.
  - With `set_eval_timeout(800 ms)`, three forms of about 465 ms each written
    inline end at 801 ms with `TerminatedEvaluation["TimeConstraint"]`; the same
    three forms loaded by `Get` run **1414 ms and return `"three-done"`**.

  Probes and raw output: `.temp/director/budget-probe2.rs`,
  `.temp/director/budget-probe2.txt`, `.temp/director/budget-probe4.rs`,
  `.temp/director/budget-probe4-timeout800.txt`. The guest driver must not
  renew the deadline, reset `current_memory_limit`, reset the interrupt poll
  counter or re-arm accounting for a new form, and must not do so on resume.
  `TimeConstrained` around `Get` is already enforced (the director checked)
  and must stay so.
- **`BeginPackage` restores its own package frame on every non-success exit**,
  including a propagated `Exit` (brief-549 open question 3). Today an ordinary
  dependency failure rolls back and `Exit` does not; make them consistent, and
  test it.
- **No leases.** The retention line's R11/R12 will convert the loader later;
  name the payloads your frames carry so that conversion can build on them
  (brief-549 §4, "Retention and organization").
- Overlapping public requests are rejected, not superseded (brief-411).

## Red tests first

`.temp/brief-549/probe.rs` is a runnable archive of brief-549's probes with an
in-memory read-only package provider. Its raw result is
`.temp/brief-549/probe-05-final.txt`: 6 passed, 44 failed. The director re-ran
it and got the same verdict for every test (`.temp/director/brief549-probe-rerun.txt`).

Install as **permanent tests in the repository's own test suites**:

1. The 43 guest-loading reds — every `red_*` test in the archive **except**
   `red_batch_guest_get_reports_input_without_panic`, which belongs to the next
   brief. They cover `Get`, `Needs`, `BeginPackage` and the stub, each alone and
   followed by a second input; streams and `$InputFileName`; twice in one file;
   nested loads; ordinary errors after a resume; syntax errors; `Return`,
   `Abort`, `Exit` and cycle forms; message checkpoints under `Check` and
   `Quiet`; forms lowered only when reached; the evaluation stack across forms;
   cancellation at both input positions for each entry; and cleanup that loads a
   file during abandonment.
2. Two budget reds from the decision above: `MemoryConstrained[Get[...], …]`
   must abort, and a `Get` of forms each under the request timeout must still
   terminate at the request timeout.
3. The archive's passing controls (`green_best_effort_discards_and_next_input`,
   the `ValueQ` and `Monitor` rows) as controls, so a repair cannot break the
   constructs that already swallow correctly.

Every oracle test asserts that the suspension actually happened
(`SessionRequestError::InputRequired`) and the prompt order, then compares the
resumed path with the queued-input path in value, `log`, termination, messages
and prints, and finally requires that a fresh request (`1 + 1`) is accepted.
Keep the archive's assertions; do not loosen one to make it pass. **Run the new
tests before any production change and save their failing output** under
`.temp/brief-550/red/`.

## Context pointers

- `AGENTS.md`, `docs/testing-strategy.md`, `docs/code-organization-guidelines.md`.
- brief-549's report: §3 the mechanism, with the exact lines; §4 the design —
  the frame inventory table (`GuestScriptFormFrame`, `GuestFileCloseFrame`,
  `GetCompleteFrame`, `NeedsCompleteFrame`, `BeginPackageDependencyFrame`,
  `DeclaredStubLoadFrame`, and the pending owner's guest-form environment), the
  nested top-level entry, per-entry behaviour, abandonment; §6 the red tests.
- `.temp/brief-549/`: `probe.rs`, `case-sources.json` (exact sources, files and
  responses), `streams-and-state.md` (both sides' streams and state for every
  case), `probe-05-final.txt`.
- `crates/rum-runtime/src/runtime/session/session_eval.rs` (form and file
  evaluation), `session/session_helpers.rs` and `session/session_state.rs`
  (loader snapshot, interrupt budget), `builtins_context_loader.rs` (the
  callers), `eval_core/symbol.rs` (the stub), `suspension/core.rs` (queue,
  driver, abandonment worklist), `session/session_requests.rs` (request
  admission and failure packaging).
- Normative docs brief-549 §4 "Documentation delta" names:
  `docs/guide/scoping-and-contexts.md:94-120`, `docs/guide/evaluation.md:282-289`,
  and the generated reference entries for `BeginPackage`, `DeclarePackage`,
  `Get`, `InputString` and `Needs`.

## Scope

- **Objective:** the guest-only loader driver and every caller continuation and
  structural cleanup needed to complete it; the inherited budget; the
  `BeginPackage` exit restore; the permanent tests; the guide and catalog text
  that describes guest loading's suspension behaviour, with the reference
  regenerated from the catalog rather than hand-edited.
- **Fence:** no batch resumability and no change to the host batch path's
  behaviour; no overlap supersession; no leases or retention promotions; no
  `Expr`, `Symbol` or serde changes; no new builtin semantics beyond the
  decisions above; no front-end changes (`rum-cli`, `rum-repl`, `rum-jupyter`);
  no other continuation-sweep group (G1-G8); no dependency or lockfile change; no
  default-path diagnostics. Do not edit `AGENTS.md`, `PROGRESS.md`, `TODO.md`,
  earlier briefs or unrelated docs.
- **Disclosure duty:** list every changed file with its line ranges and why it
  serves the objective; every new frame, its fields and its root edges; every
  existing synchronous behaviour you changed, with before and after; every
  callback or error path you did not cover. If the objective needs a change
  outside the fence, STOP and report it.
- Do NOT add unconditional cost to the default evaluation path. Any
  instrumentation goes behind the existing env-gated cold-load `AtomicU8`
  pattern.

## Deliverable

1. The permanent red tests, run and saved red before implementation.
2. The guest script plan and cursor, per-form completion and scope handling,
   one file-transaction owner, and the `Get`, `Needs`, `BeginPackage` and stub
   continuations, carrying exactly the state in brief-549 §4. Lower each form
   only when it is reached. Initial execution and resumption call the same
   finish helpers.
3. The pending guest-form environment on the cold path, reinstalled before the
   first child frame runs on resume: message-list scope, handled-error tail,
   evaluation-stack prefix, and the paused form's budget and identity.
4. The inherited budget: no renewal for a new form or on resume.
5. Frames derive `FrameRootTrace`; only genuinely pure frames derive
   `PureFrameAbandon`; the loader and form-scope frames have explicit
   structural abandonment. No rollback obligation held in both a `Drop` guard
   and a frame. No full-snapshot clone per resume.
6. White-box tests: inner-to-outer restoration; two nested files; both
   suspension positions; exact `load_stack` pop and snapshot-restore counts;
   pending, resuming and worklist queues empty after completion and after
   cancellation; `FrameRootTrace` reaching definitions and buffered values; a
   paused form's deadline preserved; `$MessageList` scopes; no replayed prefix
   output reaching a test print sink.
7. Host tests through `HostSession::eval` and resume: resumed loading, no
   poisoning, reset and replacement from either suspended position, cancelled
   old handles, fresh requests accepted. Reset stays a rebuild with no guest
   cleanup.
8. Guide and catalog text for the four guest entries; the reference
   regenerated. The host batch passages stay true as written.

## rum / Rust pitfalls (read twice)

1. **A source containing `InputString` is not proof it suspended.** Assert the
   suspension and the prompt order before comparing results.
2. **A post-child frame cannot restore the message scope its child runs in.**
   That is what the cold pending guest-form environment is for (brief-549 §4).
3. **Preserve the form semantics that exist**: `ConvertCyclesToValue`,
   `Return`/`Abort` per form, `Exit` propagating, ordinary-error rollback, and
   `Get`'s outer `termination` staying `None`.
4. **`BeginPackage` changes context before loading its dependencies**, and an
   earlier successful dependency is not undone by a later failure. The stub
   resumes after `Needs`, into the symbol read; it does not re-enter the
   autoload branch, and the enclosing fixed-point evaluator keeps its retry
   policy.
5. **`InputRequired` is propagation, not failure.** Never close the file
   transaction on it. Cancellation propagates through every caller after
   rollback and never runs a success suffix.
6. **`load_stack` is popped exactly once per file**, checked against the
   expected depth and path — never a blind `pop`.
7. **Stop-loss is keyed on progress.** Counters: permanent reds now green with
   an actual suspension; caller continuation paths completed; rollback and root
   invariants proved; instruction or allocation regression rows eliminated.
   Report them after each attempt. **Reducing compiler errors is progress**; the
   stop-loss is measured only once the tree compiles, and fires when two
   consecutive compiling attempts move none of the counters.

## Verification (MANDATORY — you run it; iterate to green)

Wrap every command in `python3 scripts/run-bounded.py --timeout <seconds> --`,
with bounds derived from a measured run of the same shape. Paste commands,
bounds, exit statuses and test counts verbatim.

```
cargo fmt --check
cargo test -p rum-runtime
cargo test -p rum-stdlib
cargo test -p rum-host
cargo test --workspace
```

Native ZeroMQ tests that cannot bind loopback in the sandbox fail loudly unless
`RUM_ALLOW_NATIVE_ZEROMQ_SKIP` is set; if you set it, say so and list the
skipped tests.

**Performance gate.** The change touches request entry, the resume driver and
suspension ownership, not the per-step evaluator, but that has to be shown,
not assumed. Before editing, build and preserve base binaries under
`.temp/brief-550/base/` — a default release `rum-cli` and an `alloc-count`
build — because there is no git to reconstruct a base from.

- **Allocation counts first** (`rum-cli/alloc-count`, `RUM_PROFILE_ALLOC=1`):
  `count`, `bytes_total` and `peak_live_bytes` from the raw stderr.
- **Per-iteration instructions decide.** For each label, run its exact
  statement from `scripts/core-language-microbench.sh` at N and 2N iterations
  with
  `taskset -c 2 perf stat -x, -e instructions:u -o <csv> -- <rum-cli> --file <label-N.rum>`
  on both builds, feeding `rum-cli` `< /dev/null`, and report the amortized
  `(count(2N) - count(N)) / N`. One label per run; never aggregate.
- **Timing is secondary**: one complete `scripts/core-language-microbench.sh`
  A/B, pinned with `taskset -c 2`, profiling env off, output under
  `.temp/brief-550/`. Timing on this host has reported +145% where instructions
  showed +0.04%; a timing difference without an instruction difference is
  noise.
- Labels: `literal-integer`, `literal-string`, `symbol-ownvalue-read`,
  `symbol-ownvalue-guarded-read`, `set-ownvalue`, `downvalue-exact-hit`,
  `downvalue-exact-miss`, `downvalue-pattern-hit`, `downvalue-pattern-miss`,
  `if-true`, `block-local`, `module-local`, `do-count-null`,
  `do-iterator-null`, `table-iterator-null`,
  `valueq-undefined-symbol-definitions-2000`,
  `valueq-undefined-symbol-definitions-8000`. There is no loader label; add a
  scratch loader panel (a queued normal load, a suspend-and-resume load, an
  error load) as a supplement, never a replacement.
- If CPU 2 or `perf` is unavailable in the sandbox, save the failure and report
  the lane unavailable; do not substitute timing for instructions.
- At most two samples per (label, N, build) per compiling iteration. A
  regression on any label is a defect to fix or report.

## Checkpointing — protect the scene

All scratch goes under `.temp/brief-550/`. After each milestone — reds saved,
driver compiling, each caller converted, budgets, white-box tests, host tests,
docs, each gate — write `.temp/brief-550/PROGRESS.md` stating what is done and
verified with its saved evidence, the exact next action, the tree's state (does
it compile, which tests are green, which files are half-edited and how), and
the progress counters. Prefer to leave the tree compiling at each checkpoint.

## Hard rules

- Do NOT `git commit`, branch, push or touch git config. Do not run `git` at
  all. Leave all changes in the working tree.
- Do NOT modify files outside the objective and fence. If you believe you must,
  STOP and write that in the report.
- Touch nothing outside this repository. No network. Commands: file reads and
  edits within scope, `rg`/`grep`, `python3`, bounded `cargo`, the release
  `rum-cli`, the microbench scripts, and bounded `taskset`/`perf`.
- **Testing Policy is a hard line** (`AGENTS.md`): rum against rum only.
- **No new `unsafe`.**
- Do not weaken an assertion, a host contract or a rollback to turn a red test
  green; report the conflict instead.
- You MUST paste real verbatim output. If a gate did not reach green or a lane
  was unavailable, say so.

## Report format (your final message, appended to this file)

1. What changed — per file, with line ranges and why; the frame, field, root
   and rollback inventory.
2. Verification — every command with its verbatim result; red-before and
   green-after for every permanent test, with the resumed and queued values,
   logs, messages, prints, prompt counts and fresh-request acceptance.
3. Allocation counts, per-iteration instructions and the timing comparison,
   per label, with every unavailable lane disclosed.
4. The progress counter sequence, the disclosure list, and any unresolved risk
   or question.

---

## Resume — the first run was stopped by the host, not by the work

The first dispatch (2026-09-28) was killed by the director's harness when the
machine ran critically low on memory; nothing in the run caused it. **The
working tree is exactly as that run left it**, and it compiles. Nothing was
reverted. The director also preserved it, without moving HEAD, as
`refs/wip/brief-550-interrupted` (`5e3cea9f`) and as
`.temp/brief-550/wip-backup/` (`tracked.patch`, `untracked.tar`). You do not
need either; they exist in case the tree is damaged.

**Read `.temp/brief-550/PROGRESS.md` first.** Where it stopped:

- Done and verified: all 45 permanent reds red before the change
  (`red/final.txt`) and green after (`raw/guest-01.txt`, 47 passed including
  the two controls); 4/4 caller paths; 8/8 white-box invariants
  (`raw/whitebox-03.txt`); 4/4 host tests (`raw/host-01.txt`); guide and
  catalog text, with the reference regenerated (`raw/reference-generation.txt`);
  `cargo fmt --check` green (`raw/gate-fmt.txt`); base binaries preserved in
  `.temp/brief-550/base/`; instruction-panel sources written under
  `.temp/brief-550/perf/`.
- Last state: `cargo test -p rum-runtime` run 02 had 1076 passed, 1 failed,
  3 ignored (`raw/gate-runtime-02.txt`); the failure was the explanation string
  of the new scalar `Session` field's exemption, which was then corrected. Run
  03 had not been recorded.

**Do not redo what is done.** Continue with:

1. `cargo test -p rum-runtime` (run 03), then `-p rum-stdlib`, `-p rum-host`,
   `--workspace`, then `cargo fmt --check` again if anything changed.
2. The performance gate exactly as specified above: allocations, then
   per-iteration instructions on every named label at N and 2N against the
   preserved base binaries, then timing.

**Keep memory pressure down.** Run every `cargo` command with
`env CARGO_BUILD_JOBS=4` inside `scripts/run-bounded.py`. Before the timing
lane, check that no other CPU-heavy process is running (for example a `lean`
job); if one is, run the allocation and instruction lanes, and report the
timing lane deferred rather than measured under load.

The fence, the rulings and the stop-loss are unchanged. Append your report to
this file as specified above, covering both runs.
