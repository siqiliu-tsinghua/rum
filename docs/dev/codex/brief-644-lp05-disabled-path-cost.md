# Brief 644 — the disabled path must not pay for the adopted timing token

**Shape: IMPLEMENTATION, cost repair on the profiling hot path. Scope: the
adopted runtime-path timing helpers, the timing token, and the two
`eval_set` sites that call them. No record changes, no semantics changes,
no default-path cost.**

## Goal

Brief-642 A1 made `Set`'s self-evaluating RHS branch cost more with the
profiling switch off. Callgrind, exact N=2000/4000 on the frozen
`set-ownvalue` label (`Do[xOwn = 123,{2000}]`), `builtin_set` own Ir:

| arm | own Ir at N=2000 / 4000 | per call |
| --- | --- | --- |
| base `ab3c1c89` | 216,161 / 432,161 | 108 |
| A1 `16bec656` (director-built) | 236,141 / 472,141 | 118 |
| A2 `731a4bc7` | 236,141 / 472,141 | 118 |

The A1-only arm reproduces A2's row exactly, so A2 contributes nothing on
this label and the cost enters with A1's change at
`definitions.rs:30-35` (`set.rhs.evaluate`, `set.rhs.suspend`):
`runtime_path_timing_start_adopted` +
`record_profile_runtime_path_timing_adopted_ok` replaced the base's
`runtime_path_timing_start` + `record_profile_runtime_path_timing`. The
emitted-path difference is two 56-byte stack copies (8 instructions each)
at those sites, less 6 removed outcome-classification instructions.

The 56 bytes are consistent with the token, and the type arithmetic says:
`Symbol` is 24 bytes (`expr.rs:36-40`: `u32`, `Option<NonZeroU64>`,
`&'static SymbolText`), `RuntimePathTimingKey` is 40 (`mod.rs:427-430`),
`RuntimePathTimingToken` is 56 (`Instant` 16 + key 40), and `Option` of it
is still 56 through the string-pointer niche. **Confirm or refute that
mapping against the emitted code before choosing the fix** — the arithmetic
is a hypothesis, not the proof.

Requirement: with every `RUM_PROFILE_*` variable unset, `builtin_set` must
cost **no more than 108 own Ir per call**, and no other rum-side function
may absorb the difference. The diagnostic record set must be unchanged.

## Context pointers

- `crates/rum-runtime/src/runtime/definitions/definitions.rs:22-70` — the
  two `expr_is_self_evaluating_atom` sites (`:30-35`) and the allocated
  branch below them. `expr_is_self_evaluating_atom` is
  `expr_helpers.rs:34-39`.
- `crates/rum-runtime/src/runtime/profiling/timing.rs:128-186, 214-241,
  605-640` — `runtime_path_timing_start_adopted` and its slow path,
  `record_profile_runtime_path_timing_adopted_ok` and its slow path, and
  the generic adopted-boundary recorder. The base shapes are still in the
  file beside them (`runtime_path_timing_start` `:91-106`,
  `record_profile_runtime_path_timing` `:669-700`).
- `crates/rum-runtime/src/runtime/profiling/mod.rs:427-430, 484-487` — the
  key and the token.
- `.temp/brief-642/b/` — the Stage B instruments and evidence:
  `run_copy.py` (writes the N/2N programs and captures), `attribute_target.py`
  (self-cost table and reconciliation), `attribute_pcs.py` (executed-PC
  slopes), `attribution/set-ownvalue/` (the base and candidate disassembly
  and `executed-path.diff`). Reuse them; do not rewrite the method.
- `.temp/director/gate-642b/repro/` — my own four captures and the A1-only
  arm's two; same command shape, `callgrind_annotate --inclusive=no` for
  the self cost.

## Scope

Files you MAY modify:

- `crates/rum-runtime/src/runtime/profiling/timing.rs`
- `crates/rum-runtime/src/runtime/profiling/mod.rs` — the token and key
  types only, plus their constructors if the token shape changes
- `crates/rum-runtime/src/runtime/definitions/definitions.rs` — the
  `eval_set` call sites only
- `crates/rum-runtime/src/runtime/profiling/tests/` and the test modules in
  `crates/rum-runtime/src/runtime/profiling/logical/` — additive tests only

Files and areas you MUST NOT touch:

- `PhaseKey`, `Policy`, `open_phase`, `complete_phase`, or any record's
  meaning: the same phases must open and complete at the same points with
  the same outcomes. An edit that changes *whether or when* a record is
  emitted is a STOP, not a cost fix.
- the switch names, the `lexical_profile_enabled` cold-load pattern, the
  `#[cold]`/`#[inline(never)]` slow-path structure, or the site strings
  (`set.rhs.evaluate`, `set.rhs.suspend`, `set.prepare_lhs`, ...);
- the assertions of the 21 assignment tests, the A1/A2 oracle fixtures,
  `crates/rum-host/tests/lp05_assignment_rows.rs`, or any inventory by
  hand;
- `Cargo.lock`, any dependency, any feature, any `unsafe`, `AGENTS.md`,
  `TODO.md`, other briefs, `.git`.

**Standing maintenance duty (always in scope).** The retention contract
fingerprints hash every `.rs` under `crates/`, so these edits expire
`inventory/local-contracts.tsv` and the certificate cells in
`inventory/promoted.tsv`; the ownership discovery and the suspension-oracle
producer inventories may also move. Replay exactly as `TEMPLATE.md`
describes, before the first acceptance suite, and disclose every changed
row. A red whose only cause is a stale derived cell is an ordering
artifact, never a regression. STOP instead if a committed row would change
identity, status, disposition or exclusion, or if a new
`UnknownHostPayload` row appears, or if a new row points at code this brief
did not touch.

## Deliverable

1. **Red first.** Reproduce the 118/call row on HEAD with the Stage B
   instrument, and the A1-only 118 row from `16bec656` if you rebuild it.
   Paste the six numbers (two sizes × three arms).
2. **Mechanism.** Name the source construct that emits the copies, with the
   emitted instruction sequence and the source line it belongs to. State
   whether the copies are `Option<RuntimePathTimingToken>` moves and why
   the base shape does not emit them (the flag re-read, the missing inline,
   the by-value argument, or whatever the code shows).
3. **Repair.** One mechanism, no parallel path. The disabled path must not
   materialize, move or spill the token. Any of these is allowed if the
   measurement proves it: a smaller token, a by-reference hot wrapper, an
   out-of-line enabled path selected once, or a call-site restructure.
   Forbidden: duplicated bodies, a second token, a bypass of the record
   machinery, `#[inline(always)]` added without showing the wrapper was not
   inlined, lazy initialization, or attributes keyed on `debug_assertions`.
4. **Green.** The measured bar below, the record set unchanged, and the
   inventories replayed.

## Verification (MANDATORY — you run it; iterate to green or STOP)

The cost gate, on the base export `.temp/brief-642/base/` (`ab3c1c89`) and
the repaired tree, both built at one source path with one codegen unit:

- `cargo build --release --offline --locked -p rum-cli` with
  `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, `CARGO_BUILD_JOBS=8`, separate
  `CARGO_TARGET_DIR`s, no `RUM_PROFILE_*` in the environment;
- `taskset -c 2 valgrind --tool=callgrind --dump-instr=yes
  --collect-jumps=yes` on `Do[xOwn = 123,{2000}]` and `{4000}`, serial and
  alone, `/dev/null` stdin, bounded via `scripts/run-bounded.py`;
- `callgrind_annotate --inclusive=no` for `builtin_set`'s self cost, and the
  `attribute_target.py` reconciliation (every own row sums to the whole
  delta);
- `attribute_pcs.py` for the executed path: every retained PC slope one per
  iteration, and the opcode delta summing to the measured slope.

Acceptance:

- `builtin_set` ≤ **108.0** own Ir/call at N=2000/4000;
- every rum-side function with a non-zero per-call delta is non-positive
  (the cost must not move into a helper);
- the focused perf panel is non-positive per call on `set-ownvalue`,
  `symbol-ownvalue-read`, `lhs-set-ownvalue-arg`, `lhs-set-literal`,
  `lhs-set-head-eval`, `lhs-setdelayed-pattern`, and the same-tier
  neighbours of the two sites;
- `cargo fmt --check`; `cargo check --workspace --all-targets`;
- `cargo test -p rum-runtime --lib` at eight threads;
  `cargo test -p rum-runtime --lib --no-default-features` at eight threads;
- `RUM_PROFILE_LEXICAL_SUBSTITUTION=1 cargo test -p rum-host --test
  lp05_assignment_rows --offline` — `1 passed`;
- `cargo test -p rum-stdlib --test suspension_oracle --offline
  -- logical_counter_port`, the full oracle, and the oracle without default
  features — both builds must pass and every adopted `set.*` cell must be
  SAME in `all` and `timing` modes;
- the loss ledger and the census replay as brief-642 A2 recorded them:
  census 1,067 paired with 0 new, losses 92,096, receipts 4,375, mass
  725,052, ownership 61 / 79 / 8,006, fingerprints 68 with 0 changed.

The workspace suite is the director's: do not run `cargo test --workspace`;
the gate runs it once at acceptance. If `perf` or Callgrind is unavailable,
that gate is unpassed — report it, never substitute wall clock.

**Stop conditions.** STOP and report if: an edit changes whether or when a
record is emitted; the bar cannot be met after two attempts that each lower
the measured `builtin_set` slope; the mechanism cannot be named from the
emitted code; a committed inventory row would change identity; or a new
`UnknownHostPayload` row appears. Stop-loss is keyed on progress: two
consecutive attempts that do not lower the measured slope end the brief with
a checkpoint.

## Hard rules

- No version-control command in any form; leave the tree for the director.
- No network, no new dependency, no `unsafe`, no default-path cost, no new
  feature, no `#[ignore]`.
- Timing, perf and Callgrind runs are serial and alone, profiling off.
- All scratch under `.temp/`, never `/tmp`. The dispatcher already runs this
  session inside `scripts/memory-scope.sh ... 16G`; **do not invoke that
  wrapper again from inside the sandbox** — it cannot reach the user bus and
  the attempt exits 1. Run cargo directly with `CARGO_BUILD_JOBS=8`, as
  brief-642 A2 and Stage B did; tests at `RUST_TEST_THREADS=8` with
  `--no-fail-fast`.
- Paste real output; say plainly what did not run.

## Report format

`rum-report/1` at `.temp/brief-644/report.json`; `python3
scripts/report.py check 644` must pass before the final message, and the
final message is exactly `python3 scripts/report.py summary 644`. The
report lists every run with its command, log, exit code and counts; the
`changes` list names every touched file with line ranges and the reason;
`disclosures` lists every inventory row that moved; `questions` carries any
STOP.

## Director amendment — execution context, and the mechanism is proven

The first attempt stopped on my rule, not on the code: I wrote that every
cargo command must be wrapped in `scripts/memory-scope.sh`, but the
dispatcher already holds the whole session in a 16G unit and a nested
wrapper cannot reach the user bus inside the sandbox. The rule above now
says so. That STOP consumed no attempt; the tree is unchanged.

The red-side work that run did is accepted as evidence and should not be
redone from scratch:

- The +10 is confirmed on the retained source-verified binaries, base
  216,161/432,161 (108/call) against 236,141/472,141 (118/call).
- The two eight-instruction groups are **moves of the by-value
  `Option<RuntimePathTimingToken>` argument** staged at `definitions.rs:30`
  and `:33` for the inlined `record_profile_runtime_path_timing_adopted_ok`
  wrapper and its cold `_slow` callee. Both wrappers are already inlined;
  the caller must prepare the 56-byte aggregate even when the following
  flag test skips the call.
- The base does not pay it because its inline recorder destructures
  `Some(token)` before passing a **bare token** to its cold recorder, so the
  disabled path never stages the aggregate.
- The type mapping is 40-byte key + 16-byte `Instant` = 56-byte token, and
  the `None` niche is the `Instant` nanosecond field (`1_000_000_000`
  written at `+48`), not the string pointer. My string-pointer-niche guess
  was wrong; the arithmetic was right.
- The +10 reconciles as +16 at the two RHS copies and net -6 on the finish
  route (three fewer staging instructions, five fewer classification
  instructions, two more in the finish token copy).

Nothing above changes the Goal or the bar. The base shape is the obvious
starting point, but the repair is still yours to measure and prove; a
different mechanism is fine if the emitted path shows it.

The first audit's report is `.temp/brief-644/report.json`; the resumed run
replaces it.
