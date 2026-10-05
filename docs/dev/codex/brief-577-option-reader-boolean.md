# brief-577 — G5 step A: a typed read-and-complete boundary, and the boolean readers

Implementation brief. **Do not run `git` in any form**, including before you
have read this brief. Leave every change uncommitted for the director's gate.

## Goal

A delayed option value is evaluated when its builtin reads it; a suspension
inside it completes the request with the option instead of the builtin's
result (`Count[…, Heads :> (…; InputString["p: "]; …)]` resumes to `False`
where the queued path returns `3`). brief-576 designed the repair: a shared,
typed **read-and-complete** boundary entered at each existing read position —
the reader takes the caller's continuation and returns the completed
builtin's `Expr`, so a read that forgets its continuation does not compile —
with one generic option-read suspension frame and per-family frozen
continuation data. This step installs that protocol and migrates every
`system_option_bool` consumer: P0193, P0711, P0714, P0764, P0767, P0848,
P1385, P1389, P1393, P1401, P1410, P2354 (Trace, Cases, DeleteCases, Replace,
Count, MemberQ, FreeQ, Level, Position, Depth). Lost continuations
**43 -> 31**.

brief-576's report and `.temp/brief-576/draft-G5-A.md` are the raw material;
**this brief governs** where they differ.

## Director rulings on brief-576's questions

1. **The typed CPS surface and the kernel signature changes are approved.**
   No legacy getter that returns a bare `bool` survives: an escape hatch would
   keep the defect compilable.
2. **I04 is authorized, exactly:** the new public generic inputs —
   `Session::system_option_bool(next)` (and, in step C,
   `Session::option_rule_expr_for_symbol(next)`),
   `OptionReadContinuation::finish(value)`,
   `FrozenOptionReadContinuation::finish(value)`, and the generic frame's
   owned port if discovery reports one — enter the ownership inventories as
   `UnknownHostPayload` rows with disposition **`PublicInput`**, beside and
   distinct from the existing `OptionReentry` row (which is not touched). They
   are honest: a caller-supplied continuation is public input. **Any other new
   `UnknownHostPayload` row is a STOP.**
3. The cold-only reparse of pure data (step B's Names pattern, step C's
   ArrayFilter shape) and the A/B/C split are accepted.
4. **Performance:** instruction slopes on one-codegen-unit builds decide; **no
   timing suite** (`TEMPLATE.md` pitfall 8; briefs 566-575).

## Context pointers

- brief-576 and its report; `.temp/brief-576/inventory.md`, `design.md`,
  `inventory-plan.md` (I01-I08), `performance-plan.md`,
  `reader-references.tsv`, `g5-current-rows.json`, `cases.json`, `probe.rs`,
  `oracle.tsv`, `lifecycle.tsv`.
- `crates/rum-runtime/src/runtime/options/core.rs` (and the certified C01
  `:87-103` and C02 `:190-204`, which stay verbatim);
  `crates/rum-stdlib/src/tests/retention_closure/local/{certificates,production,contracts}.rs`.
- `suspension/core.rs:764-855`; the accepted continuation patterns of briefs
  563, 568, 572-575.

## Scope

**Objective:** the read-and-complete protocol, the boolean readers'
migration, the enforcement tests, the owning semantic tests, and the standing
and performance gates.

**Edit surface:** `crates/rum-runtime/src/runtime/options/{core.rs,read_phase.rs}`,
`runtime/options.rs`, `runtime/kernel_api.rs`; `debug_tools/trace.rs` and new
`trace/options.rs`; `kernel_data/rewrite_data/cases/{collect.rs,collect/options.rs,delete.rs,delete/options.rs}`;
`kernel_data/rewrite_data/replace/{once.rs,once/options.rs}`;
`tree_walkers/{count_like.rs,count_like/options.rs,position.rs,position/options.rs,level.rs,level/options.rs}`;
stdlib `structure_ops/{surface.rs,surface/options.rs}`; new
`options/tests/read_phase.rs` with its compile-fail and compile-pass controls;
new `crates/rum-stdlib/src/tests/options_suspension/read_inventory.rs` and its
module line; new `crates/rum-stdlib/tests/option_reader_continuations.rs` and
children (`support`, `boolean`, `lifecycle`); rum-runtime's frame lists.

**Inventory exceptions, authorized by name** (`inventory-plan.md`): I01 (the
boolean signature, the options-private helper, the exports), I02's F00, F01
and S01-S08 (frame lists: **+10** derive entries, **+1** `PureFrameAbandon`,
named frames **559 -> 560**), I03 for the affected source rows only, I04 as
ruled above, I05, I06's Cases / Level / Position / Replace / Trace-trio
evidence anchors, I07, I08. No addition to the `OptionReentry` domain.

**The certified bodies stay verbatim.** C01 and C02, the accounting drivers,
the `OptionReentry` domain and the 33-member consumer protocol are unchanged,
and both `local_production_acceptance` runs must earn the same two C01 and one
C02 marks **on their merits** — a refreshed fingerprint alone is not a pass.
If a certified body must change, STOP.

**Standing duty — always in scope** (`TEMPLATE.md`). STOP on any other
existing row change or disappearance, any `UnknownHostPayload` beyond I04, any
new row in untouched code, any self-reaching state.

**Fence:** no other suspension group (steps B and C keep their readers); no
`Expr`, matcher or evaluator redesign; no leases, `unsafe`, dependencies,
lockfile, catalog reclassification or request-policy change; the rum-host API
unchanged. No edits to `AGENTS.md`, `TODO.md`, root `PROGRESS.md`, earlier
briefs or docs beyond the new protocol's own rustdoc.

**Disclosure duty:** every changed function, frame and root field, inventory
row, measured label, and any adjacent defect.

## Deliverable

brief-576 design.md §3: the reader takes `next: C` and returns the completed
`Expr`; ordinary success decodes and calls `next.finish`; an ordinary error
returns unfrozen; `InputRequired` freezes `next` in a cold, non-inlined helper,
inserts the frame and propagates. Every read stays at its current position:
Trace and Depth read before their arity checks, Heads defaults are still read
when an explicit override follows, Cases and DeleteCases converge both setup
entries on one post-read tail without re-evaluating target or pattern,
Count / MemberQ / FreeQ share one finish discriminator. A second suspension
installs the new continuation once; pure abandonment runs nothing pending.
The enforcement: compile-fail controls for a missing continuation, an
inaccessible raw getter and a non-rootable frozen capture (each with a
compile-pass twin), and a source gate that inventories every reader call,
requires tail use, and records only steps B and C's remaining legacy sites —
a new caller cannot join that list silently.

## Tests

Red first, recorded before any production edit: every `cases.json` witness in
this step's families (the Heads witnesses, both Cases / DeleteCases
after-target cases, their twice variants, Count / MemberQ / FreeQ / Position /
Replace after-target, TraceDetail and TracePrint, the Count option error,
first and second cancellation, answer-free abandonment) through the rum-vs-rum
oracle — value, error, termination, ordered messages, prints, `log`, actual
suspension, exact prompts in execution order, no owner after, `1 + 1`. Keep
the passing controls. Every earlier group's suite stays green.

## rum / Rust pitfalls (read twice)

1. Never make a pre-fix value the expected answer.
2. Every reader call is a tail call; a later unframed `?` discards the
   guarantee — the source gate enforces it.
3. Preserve read order, conditional reachability, the delayed-versus-ordinary
   policy, the current late default lookup and every completed guest prefix.
4. One change per measured candidate; stop after two consecutive compiling
   candidates that move none of: repaired sites, passing fixtures, regressing
   labels, worst regression. Falling compile-error counts are progress.

## Verification (MANDATORY — run to green or evidence)

Serial, through `python3 scripts/run-bounded.py --timeout <s> -- env
CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true
TMPDIR=<repo>/.temp/brief-577/tmp …`. Bounds: focused 150 s; runtime,
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

**Performance gate.** Base: `.temp/brief-577/base/rum-cli` and
`rum-cli-alloc` — the one-codegen-unit build of the incoming source (hashes in
`sha256.txt`); do not rebuild them. Candidates: the working tree with
`CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, default and `--features
rum-cli/alloc-count` (two arguments), fresh target directory each, hash
checked. Panel: `performance-plan.md`'s step-A labels — the small default
reads (`Cases[{1,2,3},_Integer]`, `Count[…]`, `MemberQ[…]`, `FreeQ[…]`,
`Position[…]`, `DeleteCases[…]`, `Replace[…]`, `Level[…]`, `Depth[…]`,
`Trace[1+2]` and its variants), the delayed-but-not-suspending defaults and
the explicit overrides — plus the neighbours `cases-wide-no-match`,
`count-wide-no-match`, `position-wide-no-match`, `deletecases-wide-no-match`,
`replace-wide-no-match`, `matchq-integer-*`, and the controls
`dot-three-no-input`, `differences-no-input`, `take-multispec-small`
(`.temp/brief-563/perf/`). Scratch workloads, each result checked first.
Allocation counts, then two paired samples of `taskset -c 2 perf stat -x, -e
cpu_core/instructions/u`, one label per process, stdin `/dev/null`,
`(count(2N) - count(N)) / N`, raw output saved first. A label regresses when
both samples exceed +0.05%. **No timing suite.** At most eight measured
candidates; report every residual with its callgrind attribution.

## Checkpointing — protect the scene

Everything under `.temp/brief-577/`; `PROGRESS.md` after the red capture, each
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

1. Changed files and functions, frame and root fields, size assessment.
2. Verbatim red and green output, compile controls, the source gate, ownership
   and certificate results (marks earned), the broad suites; every inventory
   row change, the I04 rows named.
3. Twelve sites repaired (43 -> 31 predicted); every measured row, both
   samples.
4. Stop-loss counters, residuals, open questions.

---
