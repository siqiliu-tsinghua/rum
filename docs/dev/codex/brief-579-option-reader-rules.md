# brief-579 — G5 step C: the rule-valued readers, and the legacy reader surface closed

Implementation brief. **Do not run `git` in any form**, including before you
have read this brief. Leave every change uncommitted for the director's gate.

## Goal

Steps A (`ecd7b5b2`) and B (`58cd4789`) moved the boolean and the phased
readers onto the typed read-and-complete protocol. This step moves the last
reader — `option_rule_expr_for_symbol`, which evaluates a delayed option and
wraps it as `Rule[option, value]` — and its two consumers: **CheckAbort**
(the default `PropagateAborts` rule, read even under an explicit override)
and **ArrayFilter** (the default `Padding` rule, read only when the fourth
argument is absent). Sites P0452, P0979, P1708: lost continuations
**17 -> 14**, closing group G5. Then it **closes the legacy surface**: after
this step no evaluating option reader may be called outside the protocol,
except the private scalar `OptionValue` path and the certified merged
machinery, and the source gate records zero remaining legacy allowances.

brief-576's report and `.temp/brief-576/draft-G5-C.md` are the raw material;
**this brief governs** where they differ. The rulings recorded in brief-577
stand.

## Context pointers

- brief-576's report (§§2-3), `.temp/brief-576/{inventory.md,design.md,inventory-plan.md,performance-plan.md,cases.json,probe.rs,oracle.tsv,lifecycle.tsv}`.
- briefs 577 and 578 and their reports: the protocol (`options/read_phase.rs`),
  the compile controls, the source gate
  (`crates/rum-stdlib/src/tests/options_suspension/read_inventory.rs`) and its
  recorded legacy allowances, the harness
  (`crates/rum-stdlib/tests/option_reader_continuations/`).
- C01 and C02 stay verbatim.

## Scope

**Objective:** the rule decoder and reader, the two consumers, the closure of
the legacy reader surface, all of brief-576's fixtures as permanent tests, and
the standing and performance gates.

**Edit surface:** `runtime/options/{core.rs,read_phase.rs}`, narrowly the
`options.rs` / `kernel_api.rs` exports or visibility; 
`kernel_data/control_data/abort_cleanup.rs` and `abort_cleanup/options.rs`;
stdlib `functional_ops/arrays/{mod.rs,options.rs}`; the option test modules,
compile controls and `read_inventory.rs`; rum-runtime's frame lists.

**Inventory exceptions, authorized by name:** I01's rule-reader signature and
the final private legacy closure; I02's F03, S13, S14 (**+3** derive entries;
no new frame — the named-frame count stays **560**); I03 for the affected
rows; **I04's second public input,
`Session::option_rule_expr_for_symbol(next)`, as an `UnknownHostPayload` row
with disposition `PublicInput`** (beside step A's row; any other new host
payload row is a STOP); I05; I06's ArrayFilter anchors if they move; I07,
I08. The 33-member merged `OptionReentry` domain and the four unresolved D3
consumer rows are unchanged.

**The certified bodies stay verbatim**, and both `local_production_acceptance`
runs must earn the same two C01 and one C02 marks on their merits.

**Standing duty — always in scope** (`TEMPLATE.md`). STOP on any other row
change or disappearance, any new row in untouched code, any self-reaching
state.

**Fence:** no other suspension group; no tensor or operator-driver change in
ArrayFilter; no `Expr`, matcher or evaluator redesign; no leases, `unsafe`,
dependencies, lockfile, catalog reclassification or request-policy change;
the rum-host API unchanged. No edits to `AGENTS.md`, `TODO.md`, root
`PROGRESS.md`, earlier briefs or docs beyond the protocol's rustdoc.

**Disclosure duty:** every changed function and frozen field, inventory row,
measured label, adjacent defect.

## Deliverable

- The rule decoder keeps the requested `Symbol` and produces
  `Some(Rule[option, value])` on both the ordinary and the resumed path;
  absence yields `None` directly. Only then does CheckAbort decode its
  default, read its explicit option and run its primary and fallback, and
  ArrayFilter decode its padding and apply its operator. CheckAbort's
  unconditional default read and ArrayFilter's skipped read under explicit
  padding are preserved. ArrayFilter's cold resume reparses only the
  already-evaluated arguments' pure shape and radius.
- **Closure:** discovery shows zero evaluating reader calls outside the
  protocol, save the private scalar `OptionValue` path and the certified
  merged machinery; the raw non-evaluating Input / Dialog / Once calls stay
  explicitly recorded; the source gate has no remaining legacy allowance and
  its falsifiers still fire.

## Tests

Red first, recorded before any production edit: `CheckAbort-option` and its
twice variant (`True` against `7`; log `{2}` against `{2, 3}`),
`ArrayFilter-option` and twice (`0` against `List[3, 6, 5]`), the unread
explicit-padding control, a second suspension in CheckAbort's explicit
right-hand side and body and in ArrayFilter's operator after the option
phase, cancellation at a suspended rule option. Then **all 70** of brief-576's
fixtures run as owning permanent tests (66 queued/resumed comparisons and four
cancel/abandon cases), with the reader capability, root and tail controls. No
input request may complete with the decoded option as the whole builtin.
Every earlier group's suite stays green.

## rum / Rust pitfalls (read twice)

1. Never make a pre-fix value the expected answer.
2. Every reader call is a tail call; the source gate enforces it.
3. Existing body scopes and frames keep their cleanup duties; pure
   abandonment of the option phase runs nothing pending.
4. One change per measured candidate; stop after two consecutive compiling
   candidates that move none of: repaired sites, passing fixtures, regressing
   labels, worst regression.

## Verification (MANDATORY — run to green or evidence)

Serial, through `python3 scripts/run-bounded.py --timeout <s> -- env
CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true
TMPDIR=<repo>/.temp/brief-579/tmp …`. Bounds: focused 150 s; runtime,
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

**Performance gate.** Base: `.temp/brief-579/base/rum-cli` and
`rum-cli-alloc` — brief-578's selected one-unit candidate, whose production
source is the incoming tree (hashes in `sha256.txt`); do not rebuild them.
Candidates with `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, default and
`--features rum-cli/alloc-count` (two arguments), fresh target directory
each, hash checked. Panel: `performance-plan.md`'s step-C labels
(`ArrayFilter[Total,{1,2,3},1]` with default and explicit padding,
`CheckAbort[7,9]` with default and explicit `PropagateAborts`, their
delayed-but-not-suspending defaults) plus the step-A and step-B panels
(`.temp/brief-577/perf/`, `.temp/brief-578/perf/`) and the controls. Each
result checked first. **Noise floor first:** the base against itself, two
pairs, each label's largest difference its floor; a label regresses when both
candidate samples exceed `max(+0.05%, floor)`. Allocation counts, then two
paired samples of `taskset -c 2 perf stat -x, -e cpu_core/instructions/u`,
one label per process, stdin `/dev/null`, `(count(2N) - count(N)) / N`, raw
output first. Also report the cumulative G5 delta against
`.temp/brief-577/base/` (pre-G5) for every label. No timing suite. At most
eight measured candidates.

## Checkpointing — protect the scene

Everything under `.temp/brief-579/`; `PROGRESS.md` after the red capture, each
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

1. Changed files and functions, frozen fields, the closed legacy surface.
2. Verbatim red and green output (all 70 fixtures), compile controls, the
   source gate, ownership and certificate results, the broad suites; every
   inventory row change, the I04 row named.
3. Three sites repaired (17 -> 14 predicted; G5 closed); the noise floors,
   every measured row and the cumulative pre-G5 deltas.
4. Stop-loss counters, residuals, open questions.

---
