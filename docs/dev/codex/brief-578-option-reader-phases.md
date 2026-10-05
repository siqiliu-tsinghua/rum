# brief-578 — G5 step B: the runtime-valued readers and their ordered phases

Implementation brief. **Do not run `git` in any form**, including before you
have read this brief. Leave every change uncommitted for the director's gate.

## Goal

brief-577 (`ecd7b5b2`) installed the typed read-and-complete protocol
(`crates/rum-runtime/src/runtime/options/read_phase.rs`) and migrated the
boolean readers. This step brings the runtime-valued and syntax readers onto
it, for the builtins whose option reads form ordered phases: **Monitor**
(option syntax, then EvaluationMonitor, then StepMonitor, then the monitor
scope and body), **TestReport** (SameTest, TimeConstraint, MemoryConstraint,
then the report), **Names** (SpellingCorrection and its check,
ResolveContextAliases and its check, IgnoreCase, then enumeration) and
**DumpSave** (CompressionLevel, then the write). Fourteen sites — P0118,
P0119, P0120, P0122, P0157, P0158, P0159, P0161, P0167, P0168, P0962, P0963,
P0964, P0997. Lost continuations **31 -> 17**.

brief-576's report and `.temp/brief-576/draft-G5-B.md` are the raw material;
**this brief governs** where they differ. The rulings recorded in brief-577
(no legacy getter; I04's host-input rows by name; one-unit instruction gate,
no timing suite) stand.

## Context pointers

- brief-576's report (§§2-3: the inventory, the phase rules, the frozen state
  S09-S12 and T01-T05), `.temp/brief-576/{inventory.md,design.md,inventory-plan.md,performance-plan.md,cases.json,probe.rs,oracle.tsv}`.
- brief-577 and its report: the accepted protocol, its compile controls, the
  source tail-use gate (`crates/rum-stdlib/src/tests/options_suspension/read_inventory.rs`),
  and the harness `crates/rum-stdlib/tests/option_reader_continuations/`.
- The certified C01 (`options/core.rs`, merged reader) and C02
  (`evaluate_runtime_option_expr`) — verbatim.

## Scope

**Objective:** the runtime and syntax decoders, the four phased builtins'
continuations, their tests, and the standing and performance gates.

**Edit surface:** `runtime/options/{core.rs,read_phase.rs}`;
`debug_tools/{monitor.rs,monitor/options.rs}`;
`debug_tools/test_report/{evaluation.rs,report.rs,options.rs}`;
`namespace_tools/{names.rs,names/options.rs}`; `package_tools.rs` and
`package_tools/persist_options.rs`; module declarations; additions to
`option_reader_continuations/` (phases, lifecycle), to `read_inventory.rs`
and the compile controls; rum-runtime's frame lists.

**Inventory exceptions, authorized by name** (`inventory-plan.md`): I01's
runtime-reader and helper splits; I02's F02, F04, S09-S12 and T01-T05
(**+11** derive entries, including `FrameRootTrace` on the two existing
`PersistContextSelection` / `PersistContextFilters`; **no** new frame — the
named-frame count stays **560**); I03 for the touched phase rows; I05; I06's
Names / TestReport / Save / DumpSave evidence positions; I07; I08. **Any new
`UnknownHostPayload` row is a STOP** — no new public reader is needed here.

**The certified bodies stay verbatim**, and both `local_production_acceptance`
runs must earn the same two C01 and one C02 marks on their merits.

**Standing duty — always in scope** (`TEMPLATE.md`). STOP on any other row
change or disappearance, any new row in untouched code, any self-reaching
state.

**Fence:** step C's rule readers (CheckAbort, ArrayFilter, the rule wrapper)
stay; no `Expr`, matcher or evaluator redesign; no regex implementation
change; no leases, `unsafe`, dependencies, lockfile, catalog reclassification
or request-policy change; the rum-host API unchanged. No edits to `AGENTS.md`,
`TODO.md`, root `PROGRESS.md`, earlier briefs or docs beyond the protocol's
rustdoc.

**Disclosure duty:** every changed function, stage and frozen field, inventory
row, measured label, adjacent defect.

## Deliverable

brief-576 §3's phase rules: each phase looks up **one** entry when it reaches
it, from the explicit snapshot the builtin already compiled and the *current*
defaults (the `Names-later-default-mutation` case requires this); it advances
only after success and validation; Names' invalid-value return happens before
the next read. Runtime `RuleDelayed` values stay held (`runtime_option_expr`'s
inversion is preserved). Monitor starts its scope, display and body only after
all reads; TestReport keeps its two-argument property fallback, its invalid
third-argument residual, its one-argument defaults and its report and
test-case drivers, and starts no test, timer or capture scope before the
reads; DumpSave keeps its pre-read expanded context filters (never
re-expanded) and its post-read order, and Save stays on its no-compression
path. On an actual resume, only the Names pattern is rebuilt — a pure reparse;
no guest expression and no context-filter expansion runs again. Capture only
on `InputRequired`; a second suspension installs the new continuation once;
pure abandonment runs nothing pending.

## Tests

Red first, recorded before any production edit: `Monitor-options`,
`Monitor-options-twice`, `Monitor-EvaluationMonitor`, `Monitor-StepMonitor`
and their twice variants; every `TestReport-option-*`, `TestReport-two-args`
and twice; every `Names-*` single and twice; `DumpSave-option` and twice;
Monitor with two runtime options and its streams, Names with two options, a
later-default mutation and a later option error, TestReport with three
options; the unread TestReport and Names controls — through the shared oracle
(value, error, termination, ordered messages and prints including the existing
repeated Names `optval` messages, `log`, actual suspension, exact prompts, no
owner after, `1 + 1`). Every step-A and earlier-group test stays green.

## rum / Rust pitfalls (read twice)

1. Never make a pre-fix value the expected answer.
2. Every reader call is a tail call; the source gate enforces it.
3. Read one default at a time, at the phase that needs it; do not prefetch or
   snapshot every default at entry.
4. One change per measured candidate; stop after two consecutive compiling
   candidates that move none of: repaired sites, passing fixtures, regressing
   labels, worst regression.

## Verification (MANDATORY — run to green or evidence)

Serial, through `python3 scripts/run-bounded.py --timeout <s> -- env
CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true
TMPDIR=<repo>/.temp/brief-578/tmp …`. Bounds: focused 150 s; runtime,
ownership and retention 300 s; a one-unit release build 900 s; a metric
process 150 s; `cargo test --workspace --no-fail-fast` 2 400 s.

```
cargo fmt --check
cargo test -p rum-stdlib --test option_reader_continuations
cargo test -p rum-stdlib --lib options_suspension
cargo test -p rum-runtime
cargo test -p rum-stdlib --lib ownership_graph
cargo test -p rum-stdlib --lib local_production_acceptance
cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance
cargo test --workspace --no-fail-fast
```

Report native ZeroMQ skips as skips.

**Performance gate.** Base: `.temp/brief-578/base/rum-cli` and
`rum-cli-alloc` — the director's copy of brief-577's measured one-unit
candidate, whose production source is the incoming tree (hashes in
`sha256.txt`); do not rebuild them. Candidates: the working tree with
`CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, default and `--features
rum-cli/alloc-count` (two arguments), fresh target directory each, hash
checked. Panel: `performance-plan.md`'s step-B labels (`Monitor[77,0]`, the
held explicit monitor option, `Names["Global`never576"]`, the default and
unread TestReport forms, the latent-ordinary-value setups), the step-A labels
(`.temp/brief-577/perf/`), and the controls `dot-three-no-input`,
`differences-no-input`, `take-multispec-small`. Each result checked first.
Allocation counts, then two paired samples of `taskset -c 2 perf stat -x, -e
cpu_core/instructions/u`, one label per process, stdin `/dev/null`,
`(count(2N) - count(N)) / N`, raw output saved first. **Noise floor:**
brief-577's small single-call labels varied by up to ±1% between samples of
the same binaries (per-process `RandomState` hashing in symbol interning).
First measure the base against itself — two pairs — and take each label's
largest base-to-base difference as its floor; a label regresses when **both**
candidate samples exceed `max(+0.05%, floor)`. Report the floors. No timing
suite. At most eight measured candidates.

## Checkpointing — protect the scene

Everything under `.temp/brief-578/`; `PROGRESS.md` after the red capture, each
compiling candidate and each gate.

## Hard rules

- **No `git`, in any form.** No network; nothing outside this repository
  (except the legacy certificate output `.temp/brief-546/certificates/`).
- Commands: scoped reads and edits, `rg`/`grep`, `python3`, bounded serial
  Cargo, the release `rum-cli` on passing programs, `taskset`/`perf`,
  `valgrind --tool=callgrind`. No probe that aborts a process; DumpSave tests
  use the in-memory provider as brief-576's probe did — no filesystem writes.
- **Testing Policy is a hard line** (`AGENTS.md`). No new `unsafe`. Never store
  a `Session` borrow, the cancellation observer or a `NativeStackDepth` in a
  frame.
- Paste real output; say plainly what did not run or did not pass.

## Report format (append to this brief)

1. Changed files and functions, stages and frozen fields, size assessment.
2. Verbatim red and green output, compile controls, the source gate, ownership
   and certificate results (marks earned), the broad suites; every inventory
   row change.
3. Fourteen sites repaired (31 -> 17 predicted); the noise floors and every
   measured row, both samples.
4. Stop-loss counters, residuals, open questions.

---
