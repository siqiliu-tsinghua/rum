# Brief 697 — LP08 final form: subset attribution and the downstream obligations

**Design-only brief: `Scope: NONE — no live source, workload, manifest,
inventory or version-control change.` Disposable probes only, under this
brief's root. The `terminal_owner` form is frozen here; the entry split is
the placement of record from brief 695.**

Brief 696 measured both authorized terminal-local forms against the exact
base-sequence bar and stopped: `terminal_fields` executes 56 instructions
with two `0x328(%rsp)` loads; `terminal_owner` executes 42 with one load, but
its sequence is not base's. The director gate
(`.temp/director/brief-696-gate.md`) ruled the exact-sequence criterion
retired as the terminal acceptance bar: it measured a whole-crate codegen
outcome of source presence, not feature execution. The operative hard line is
restated as the **terminal execution bar**:

1. the terminal branch's executed-instruction multiset is a strict subset of
   base's at 50 and 100 iterations, in two draws — no added instruction, no
   new call, no new memory traffic, no new frame byte;
2. it reads `0x328(%rsp)` exactly once;
3. every removed instruction is attributed to the eliminated drop-flag /
   drop-elaboration bookkeeping of `unevaluated_restore`, with no source
   statement's work moved to a callee or a cold path;
4. the terminal restoration counter stays `[1,2,1]` and the five watched
   falsifiers stay green.

This round answers three questions with measured evidence, in order. First,
with the `terminal_owner` source frozen byte-identical, does the terminal
branch meet that bar, with every one of its 12 removed instructions mapped to
its base source statements? Second, does the enabled nonterminal witness
delta (145→129 and 108→84 in 696) attribute to the same eliminated
bookkeeping, with no publication, guest or API work removed? Third — only if
both are green — do the downstream obligations deferred since brief 680
pass on this form: the direct flush, the three mispair mutants, the 15 OPEN
key routes, the host reset probe and the `docs/dev/profiling.md` contract
fixtures?

The frozen source is one line on top of the entry split:

```rust
if stage_index >= plan.len() {
    let terminal_restore = unevaluated_restore;
    let final_args = match terminal_restore {
        Some(restore) => { /* unchanged */ }
        None => state.into_final_shared_args(),
    };
    return Ok(Expr::call_shared(evaluated_head, final_args));
}
```

`/home/remi/dev/rum/.temp/brief-696/src/terminal_owner/` is the frozen tree
(call.rs sha256 `d01a3a5e2b2ce007c336cf50039cfa97816630f64b55e504b4935cc0ce83c1b1`);
its entry-split basis is `.temp/brief-696/src/entry_split/`. Copy, never edit
the retained roots.

## Stage A — the control and the frozen basis

1. **The build protocol.** Probe builds go through
   `scripts/probe-build.py` (read its docstring). The caller assembles the
   complete tree and passes `--tree <dir> --out <binary> --receipt <json>`;
   the script builds at the stable path `.temp/probe/active`, keys the
   artifact cache on the assembled-tree hash × rustc version × flags, and
   reuses byte-identical binaries. There is **one build per (arm, kind)**;
   measurements still use **two draws**. One arm per round must be rebuilt
   with `--force` and its sha256 compared to the cache; quote the
   `PROBE_BUILD` verdict line. Never set `CARGO_TARGET_DIR`; never change
   opt-level, codegen-units or incremental.
2. **Reproduce the 696 control on the frozen basis.** Rebuild base, the 686
   form, the frozen entry_split and terminal_owner with the 696 probe inputs
   (`.temp/brief-696/inputs/`, byte-identical). Reproduce: base 54 with one
   `0x328(%rsp)` load; entry_split 55 with two; terminal_owner 42 with one;
   the 386-snapshot trace; the exact 18,304/18,896 stack; coverage 94/96/416
   and the 606 initial witness; the enabled 20/20, 67/67, 131/19 battery;
   and the five falsifiers. Every 696 number is a claim, not evidence.
3. **The terminal source.** State the exact statements of the frozen
   terminal branch (`call.rs` of the terminal_owner tree) and the
   ownership decision they encode, with line numbers. State which path the
   disabled cost workload takes at 50 and 100 iterations (Some or None, by
   counter, not by reading).
4. **Persist before judging.** Save the control's per-address panels, cell
   traces, stack rows and falsifier raw logs before any attribution is
   accepted.

## Stage B — the subset and its attribution

5. **The subset.** From the raw profiles at 50/100 iterations in both draws,
   derive the exactly executed instruction list of the terminal branch for
   base and terminal_owner: same function, same case, same binary pair. Prove
   the candidate multiset is a strict subset of base's: no added opcode or
   operand, no `call`, no new memory operand, no frame-byte change; exactly
   one `0x328(%rsp)` load. Re-derive the opcode delta from the raw panels;
   the 696 set `{cmpq -1, je -2, jne -1, movb -1, orb -1, sete -1, testb -2,
   xorl -3}` is a claim. An added instruction, a second load or any
   unexplained addition is a red.
6. **The source attribution.** Map every removed instruction to the base
   source statements it serves: the drop-flag/drop-elaboration of the
   partially moved `unevaluated_restore` at the end of the terminal branch.
   Give every removed instruction's base address, its base source
   `path:line`, and the taken path at 50/100 iterations. Prove the
   ownership invariant on both arms: Some consumes `prepared_args` once and
   restores every wrapper, in order, before normalization, dropping each
   exactly once; None consumes `state.into_final_shared_args()`; the
   restore Option is moved, never cloned, taken, replaced, duplicated or
   fabricated; no other local's destruction order changes.
7. **The enabled nonterminal.** The two enabled witnesses measure 145→129
   (`resumed-call`) and 108→84 (`resumed-fixed`) in 696
   (`RESIDUAL_REQUIRES_ATTRIBUTION`). Attribute every removed instruction the
   same way, or prove where the work went. Prove no publication, guest
   value, request or API work is removed: the pair counters, the 386
   snapshots, the 150 key rows and the mirror rows stay equal.
8. **The terminal invariant.** Watch the restoration counter (`[1,2,1]`,
   `wrapper_strong=1`, `prepared_alive=false`) and the dropped-Some mutant
   (`[0,0,0]`, exit 101 named `COUNTER_ASSERT`) on the frozen form. State
   the Some/None ownership invariant with `path:line`.
9. **The stop.** If any added instruction, second load, or unexplained
   removal remains, STOP with the exact panel and the smallest alternative.
   The alternative direction (a terminal-local lifetime change for
   `CallDispatchState`/`evaluated_head`) is **not authorized in this round**;
   return the measured exclusion for the director to brief separately.

## Stage C — the downstream obligations (only after Stage B is green)

10. **The direct flush.** A direct definition finish with no enclosing phase
    records `Outcome::Some`, `faults=0`, counter
    `[1,1,0,1,0,0,0,0,0,0]`, quiescent; the eight call-site signatures and
    the token-first interface unchanged.
11. **The three mispair mutants.** Against that green control they must
    execute and turn the exact counter red with semantic equality disabled.
    A mutant that cannot be killed returns a measured impossibility naming
    its exact counter, the assertion line and why the form cannot produce
    the mispair. "Hit and pass" is not a kill.
12. **The 15 OPEN key routes.** Every OPEN row among the 150 (19 OPEN in
    696) closes by an executed probe or a `path:line` citation. The four
    `evaluate_atom_fast_path.stable_symbol` / `.atom` pause and error rows
    stay source-impossible with `eval_core/mod.rs:168-181` and are not
    reopened. Every prior PASS is restored (131 PASS / 150 rows); a
    `phase_absent` or `faults=N` stays open with its director question.
13. **The host reset probe.** Construct a real `HostSession`, suspend, call
    `HostSession::reset` (`crates/rum-host/src/session.rs:176`), and prove
    old-epoch markers cannot publish into the new epoch, no queued frame is
    discarded, there are no suffix effects, and the retirement is exactly
    once. `session.rs:176` is not patched. A separate bounded host build is
    allowed if the probe requires it; record its exact command and raw
    output.
14. **The contract fixtures.** The `docs/dev/profiling.md` obligations:
    cancellation, Session reset/drop, failed restore, checkpoint rollback,
    explicit discard and profiling reset retire the applicable leases
    exactly once, with no suffix effects or invented returns; old tokens and
    queue markers cannot publish into a new epoch; an owner can resume on
    another thread without losing records; unrelated profiled requests are
    not part of a program's vector. Run the named fixtures in the probe
    build; do not run the workspace acceptance suite.
15. **The draft.** If Stage C is green, draft the live implementation brief
    in `TEMPLATE.md` shape: the entry split plus the frozen one-line
    binding, the ripple inventory, the live gate bars, the downstream rows
    already discharged, and the risks. If any Stage C item is red, record
    the measured exclusion and stop; no live change in this round.

## Goal

When this round is done, the design record states, with measurement and not
argument:

1. **The terminal answer.** The terminal branch executes a strict subset of
   base with one load, every removed instruction attributed to the
   eliminated drop bookkeeping, and the ownership invariant preserved; or
   the exact measured exclusion.
2. **The enabled attribution.** The nonterminal witness deltas are explained
   as the same eliminated bookkeeping, with no publication or guest work
   removed; or the exact measured exclusion.
3. **The downstream record.** The direct flush, the three mispair results,
   the 15 key routes, the host reset and the contract fixtures, each green
   or a measured exclusion.
4. **The plan.** The live implementation brief in `TEMPLATE.md` shape.

## Context pointers

- `.temp/director/brief-696-gate.md` — the restated terminal bar and the
  frozen form.
- `.temp/brief-696/report-design.json`, `residual-address-results.json`,
  `terminal_owner-cell-trace-results.json`, `all-delta-rows696.json`,
  `all-stack-rows696.json`, `src/terminal_owner/`, `src/entry_split/`,
  `inputs/`, and the 696 cost/coverage profiles — read/copy only.
- `.temp/brief-695/`, `.temp/brief-694/`, `.temp/brief-693/` — prior
  controls and cuts, read-only.
- `scripts/probe-build.py`, `scripts/run-bounded.py`, `scripts/report.py`.
- `docs/dev/profiling.md` — the target contract.
- Live sites named by the forms: `eval_core/call.rs`
  (`resume_suspended_call_with_dispatch_plan`),
  `profiling/logical/mod.rs` (`complete_phase`, `take_phase`,
  `pause_call_stage_at_finish(_slow)`, `publish_stashed_phases_slow`),
  `profiling/timing.rs`, `resolution.rs`, `profiling/logical/collector.rs`,
  `suspension/core/storage.rs`, `suspension/core/profile_queue.rs`,
  `crates/rum-host/src/session.rs`.
- `docs/dev/codex/TEMPLATE.md` and `AGENTS.md`.

## Scope

Files you MAY write: anything under `.temp/brief-697/` — probe trees, builds,
binaries, receipts, harness code, the draft implementation brief and the
report.

MUST NOT touch:

- the live repository tree — no source, test, workload, manifest, inventory,
  docs or `.git` change; you run no version-control command;
- `.temp/brief-686/` … `.temp/brief-696/` or any other retained root, except
  to read and copy; never execute a file in place;
- `.temp/probe/` except through `scripts/probe-build.py`;
- the public API, features, serde, manifests, dependencies, `unsafe`.

Do not run the workspace acceptance suites, the semantics matrix, the census
or inventory replays, or any source gate. This round measures and discharges
the named obligations; it repairs nothing live.

## Hard bars

- **The terminal execution bar.** The subset, one load, full attribution and
  the terminal invariant as stated above. A source-identical function is not
  a cost claim by itself; the panels decide.
- **The frozen forms.** The entry split (four call-site bindings and
  `pause_call_stage_at_finish(_slow)`) and the one-line terminal binding are
  byte-identical to `.temp/brief-696/src/`. No form change may be made to
  make a downstream item pass; a downstream red excludes the form, it does
  not license an edit.
- **Position and semantics.** The 386-observation cell trace equals the 686
  control's in every case and every draw; the 20 pairs, 67 mirrors and 150
  key rows keep their 696 values; a publication is never dropped, postponed
  past its owner, or merged into a path that loses an open record.
- **The enabled form.** 20/20, `faults=0`, exact 18,304/18,896 stack in two
  draws, coverage 94/96/416, five falsifiers. No assertion or fixture is
  weakened.
- **The excluded forms.** All forms excluded through brief 696 stay excluded:
  the 683 observe-only guard, the 684 position-only shape, the 685 ambient
  guard, the panicking take-path fallback, both 687 cost shapes, the 688
  owner-position placement, the 689 candidate1 shape, the 690 reload1 and
  reload2 terminal shapes, the 691 `attach_helper`/`attach_publish` shapes,
  the 692 whole-body shapes, the 693 `suspension_local_phase` duplicate-pause
  shape, the 694 `suspension_single_completion` move, the 695
  `keyed_completion` form and the 696 `terminal_fields` shape.
- **Every arm reduced.** Every measurement exists in a persisted artifact; a
  run that exists only as a log is not evidence. No row dropped, no negative
  netted. Raw output is persisted before parsing.
- Every count, key, route and `path:line` is re-derived from the frozen
  trees; a brief-696 number is a claim, never evidence.

## Verification

Probe trees are assembled by you and built only through
`scripts/probe-build.py` (one build per arm and kind; `--force` on one arm
per round; never `CARGO_TARGET_DIR`). One cargo command at a time; the
script enforces a per-command timeout and a build lock. Two measurement
draws per arm; serial Callgrind, CPU-pinned, release, every `RUM_PROFILE_*`
cleared except the declared modes; four stack heights twice each with the
residual printed. A timeout is not data. Derive any bound from data.

## Checkpointing

All scratch under `.temp/brief-697/`; keep `PROGRESS.md` with the exact next
action and the receipts already saved. Leave every tree compiling at each
checkpoint.

## Deliverable

- `.temp/brief-697/report-design.json` in `rum-report/1`, with the control,
  the subset and attribution evidence, the enabled attribution, the
  downstream record, the excluded-item rows and every delta row.
- `.temp/brief-697/drafts/implementation-lp08-final.md` — the live
  implementation brief in `TEMPLATE.md` shape (only if Stage B is green;
  Stage C rows may remain marked NOT_RUN only if Stage B is red).
- Run `python3 -I scripts/report.py check 697` (without a version-control
  option) and return its summary and all compliance verdict lines.

## Report format

Every run and verdict line quoted; the draft under `drafts/`; the report's
`summary` states the measured answers and the draft's bar list.
