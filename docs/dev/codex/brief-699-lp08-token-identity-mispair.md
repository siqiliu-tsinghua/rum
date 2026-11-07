# Brief 699 — LP08 token-identity mispair falsifiers and the live draft

**Design-only brief: `Scope: NONE — no live source, workload, manifest,
inventory or version-control change.` Disposable probes only, under this
brief's root. The entry split and the one-line `terminal_owner` binding are
frozen; the work ledger is closed at brief 698.**

Brief 698 closed the terminal work ledger (54 -> 42 instructions, 17
additions paired, 29 removals attributed, one load, no call, equal frame,
memory not increased), discharged the direct flush, the 150 key routes and
the contract fixtures, and stopped on two Stage C items. The director gate
(`.temp/director/brief-698-gate.md`) settled them:

- the host no-discard clause was **my error** and is withdrawn:
  `HostSession::reset` documents that it drops queued input responses and
  that dropping a suspended request runs no guest cleanup
  (`crates/rum-host/src/session.rs:166-175`); the probe's profiling
  obligations (pending cleared, no suffix, no old `Ok`, 29 open records
  retired exactly once) are green. Guest-frame preservation is not this
  line's bar.
- the three inherited free-index mispair mutants hit and cannot kill; the
  measured impossibility is accepted for those exact mutants, because the
  mutant increments the free-index argument (`logical/mod.rs:241-243`) while
  the completion uses the token-held `call`
  (`complete_definition_rule(call, result)`), so the logical counter key is
  unchanged. But the class the bar protects — a publication completing under
  the wrong logical owner — has never been watched fail. The direct-flush
  counter has no working falsifier. That is this round's subject.

This round builds the token/record-identity mispair mutants and requires an
actual kill, then — only if the class is falsified — drafts the live
implementation brief.

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

`/home/remi/dev/rum/.temp/brief-698/src/terminal_owner/` is the frozen tree
(call.rs sha256 `d01a3a5e2b2ce007c336cf50039cfa97816630f64b55e504b4935cc0ce83c1b1`);
its entry-split basis is `.temp/brief-698/src/entry_split/`. Copy, never
edit the retained roots.

## Stage A — the control and the identity chain

1. **The build protocol.** Probe builds go through
   `scripts/probe-build.py` (read its docstring). Assemble the complete tree,
   pass `--tree <dir> --out <binary> --receipt <json>`; the script builds at
   the stable path `.temp/probe/active`, keys the artifact cache on the
   assembled-tree hash × rustc version × flags, and reuses byte-identical
   binaries. There is **one build per (arm, kind)**; one kind must be rebuilt
   with `--force` and its sha256 compared. Never set `CARGO_TARGET_DIR`;
   never change opt-level, codegen-units or incremental. Quote the
   `PROBE_BUILD` verdict line.
2. **The control.** On the frozen tree, run the direct definition flush for
   the three kinds (`down_value_rhs_eval`, `up_value_rhs_eval`,
   `sub_value_rhs_eval`): `Outcome::Some`, `faults=0`, counter
   `[1,1,0,1,0,0,0,0,0,0]`, quiescent, semantic equality disabled. Persist
   the raw logs and the exact counter key before any mutant runs.
3. **State the identity chain with `path:line`.** From the frozen source:
   `record_profile_definition_rule_timing_adopted_slow` destructures
   `DefinitionRuleRecord { start, call, _storage }` from the token
   (`logical/mod.rs:234-240`), passes the separate `index` to the record
   function, and completes with `complete_definition_rule(call, result)`;
   `defer_definition` stores the token's call (`logical/mod.rs:293`,
   `completion.rs:12-16`); the deferred pop is `take_deferred_definition(&control)`
   (`completion.rs:25-30`); the counter key is
   `Definition { kind, symbol, index }` from the call. State precisely which
   field the free-index mutant changes and why that leaves the logical key
   identical; this is the 698 measured impossibility, restated from source.
   Persist the raw 698 control lines as control evidence; do not re-run the
   whole cost campaign.
4. **Persist before judging.** Save the control panels, counters and the
   direct-flush raw logs before any mutant is accepted.

## Stage B — the token/record-identity mispair mutants

Build a disposable downstream tree carrying env-gated mutant hooks in the
logical completion path. The hooks are probe-only; the frozen source and
every assertion stay byte-identical. Each mutant must print a named
`MUTANT_HIT` line (with both identities involved) and must be refused or
must fail the named check — a mutant that runs and passes silently is a
design red, not a pass.

5. **M1 — the wrong-owner call.** Two suspended definition requests A and B
   in one session and epoch, with distinct owners/keys. With the mutant set,
   complete A with B's call/token (or B's with A's) at the completion point.
   Bar: the completion does not silently count under B's key (or under A's
   after the swap). The exact direct-flush counter assertion
   (`brief677_probe.rs:733`, `COUNTER_ASSERT brief698 direct definition
   flush`) or a named isolation fixture must fail with exit 101 / non-zero,
   and the raw counters must show the wrong key or the open record. Record
   the guard that rejects it if the swap is rejected before the counter.
6. **M2 — the stale-epoch token.** Capture a call/token, force a profiling
   reset or a new epoch, then complete with the stale token. Bar: the old
   token cannot pop or close a new-epoch record. `logical_reset_does_not_pop_a_new_epoch`
   (or the exact counter) must fail, exit non-zero, with the stale key and
   the guard named. This mutant is also the live gate bar for the
   "old tokens cannot publish into a new epoch" obligation.
7. **M3 — the wrong-record pop within a token.** Defer under record R1 and
   pop/complete under R2 (or mutate the key carried in the call), so the
   counter's expected tuple belongs to the wrong record. Bar: the exact
   counter turns red with the wrong tuple printed, exit 101, assertion text
   unchanged.
8. **The rigor.** Each mutant is built and executed; a compile or run
   failure is disclosed with its log. A measured impossibility is allowed
   only after a real attempted mutant: it must name the exact assertion, the
   `path:line` that prevents the mispair (type or API), and the attempted
   mutant's compile/run record. An argument-only impossibility is not
   evidence. The counter assertion may not be weakened, generalized or
   moved; the harness may add the mutant switch only.
9. **The controls.** With the mutants off, the four identity fixtures must
   pass under their correct module paths —
   `runtime::profiling::tests::record_engine::logical_reset_does_not_pop_a_new_epoch`,
   `runtime::profiling::tests::epoch_contract::no_cross_attribution`,
   `runtime::profiling::logical::evaluator::lp657_thread_handoff_resume`,
   `runtime::profiling::logical::evaluator::lp664_carrier_external_reset_invalidates_context` —
   plus the 20/20 pair counters, the 67 mirrors and the 150 key rows. The
   paths are the 698-resolved ones; a filter that matches zero tests is not
   a run.
10. **The stop.** If any mutant hits and passes without a named structural
    impossibility, if any counter assertion is weakened, or if the frozen
    source changes, STOP with the exact panel. No form repair, no live
    change, no draft.

## Stage C — the live implementation draft (only if Stage B is green)

11. **The draft.** In `TEMPLATE.md` shape, with:
    - the entry split (four call-site bindings and
      `pause_call_stage_at_finish(_slow)`) plus the one-line whole-Option
      binding, byte-identical to the frozen trees;
    - the re-derived ripple inventory: four pause-call bindings, two other
      pause sites, eight definition finishes, 50 keys, 11 `complete_phase`
      callers — re-derived from the frozen source, not copied;
    - the live gate bars: the work ledger (54 -> 42; 17 pairs; 12
      attributions; -6-7+1; one load; no call; frame `0x2e8`; memory
      9/11/1 -> 8/10/1), the ownership invariant and the terminal counter,
      the 386-snapshot position check, the 18,304/18,896 stack, the
      coverage 94/96/416/606, the five falsifiers, the token-identity
      mispair mutants and their exact kills, the no-collateral rows, the
      direct flush, the 150 routes, the contract fixtures, and the host
      reset's profiling obligations (exactly-once retirement, no suffix, no
      old publication); **not** guest-frame preservation, which is withdrawn
      and documented as the reset's own semantics;
    - the risks and the inventory/acceptance ordering: standing inventory
      replays before the acceptance suite, the rum-vs-rum microbench gate
      for the performance line.
12. **No live change.** The draft is a document under this brief's root. No
    source, manifest, test or version-control action.

## Goal

When this round is done, the design record states, with measurement:

1. **The identity fact.** The logical key is the token-held call's
   `Definition { kind, symbol, index }`, not the free-index argument, with
   `path:line`.
2. **The class falsifier.** The wrong-owner, stale-epoch and wrong-record
   mutants each hit and kill the exact check, or each returns a measured
   impossibility after a real attempted mutant; or the exact red.
3. **The control.** The direct flush, the four identity fixtures, the 20
   pairs, 67 mirrors and 150 key rows stay green with the mutants off.
4. **The plan.** The live implementation brief in `TEMPLATE.md` shape, or a
   measured exclusion and no draft.

## Context pointers

- `.temp/director/brief-698-gate.md` — the accepted ledger, the withdrawn
  host clause, and the re-aimed mispair bar.
- `.temp/brief-698/report-design.json`, `downstream-results698.json`,
  `source-attribution698.json`, `terminal-ledger698.json`,
  `contract-fixtures698.json`, `key-campaign698.json`, `host-reset698.json`,
  `src/terminal_owner/`, `src/entry_split/`, `inputs/`, the 698 cost,
  coverage and trace profiles — read/copy only.
- `.temp/director/probe-697/` and `probe-698/` — my own reductions; the
  pairing and work ledger are hypotheses to re-derive, not evidence.
- `scripts/probe-build.py`, `scripts/run-bounded.py`, `scripts/report.py`.
- `docs/dev/profiling.md` — the target contract.
- Live sites: `eval_core/call.rs` (`resume_suspended_call_with_dispatch_plan`),
  `profiling/logical/mod.rs` (`record_profile_definition_rule_timing_adopted_slow`,
  `complete_definition_rule`, the defer path),
  `profiling/logical/completion.rs` (`defer_definition`,
  `take_deferred_definition`, the epoch validity filter),
  `profiling/logical/epoch.rs`, `profiling/timing.rs`,
  `profiling/logical/collector.rs`, `suspension/core/profile_queue.rs`,
  `crates/rum-host/src/session.rs`.
- `docs/dev/codex/TEMPLATE.md` and `AGENTS.md`.

## Scope

Files you MAY write: anything under `.temp/brief-699/` — probe trees, builds,
binaries, receipts, harness code, mutant hooks, the draft implementation
brief and the report.

MUST NOT touch:

- the live repository tree — no source, test, workload, manifest, inventory,
  docs or `.git` change; you run no version-control command;
- `.temp/brief-686/` … `.temp/brief-698/` or any other retained root, except
  to read and copy; never execute a file in place;
- `.temp/probe/` except through `scripts/probe-build.py`;
- the public API, features, serde, manifests, dependencies, `unsafe`.

Do not run the workspace acceptance suites, the semantics matrix, the census
or inventory replays, or any source gate. This round falsifies the identity
class and drafts the plan; it repairs nothing live.

## Hard bars

- **The identity chain.** The logical key is the token-held call, not the
  free index; state it with `path:line` and show the free-index mutant's
  unchanged key from the raw lines.
- **The mutants.** Same binary pair, control green, mutant hit, exact kill;
  no weakened assertion; no silent pass; a measured impossibility only after
  a real attempted mutant with its compile/run record.
- **The frozen forms.** The entry split and the one-line binding are
  byte-identical to `.temp/brief-698/src/`; the frozen call.rs sha is
  `d01a3a5e…`. No form change may be made to make a mutant pass.
- **The controls.** The direct flush, the four identity fixtures, the 20/20
  pairs, the 67 mirrors and the 150 key rows keep their 698 values. No
  assertion or fixture is weakened.
- **The withdrawn host clause.** Do not re-run the no-discard host bar and do
  not carry it into the draft. The host reset's profiling obligations are
  already discharged; its documented guest-frame discard is not a defect.
- **Every arm reduced.** Every measurement exists in a persisted artifact; a
  run that exists only as a log is not evidence. No row dropped, no negative
  netted. Raw output is persisted before parsing.
- Every count, key, route and `path:line` is re-derived from the frozen
  trees; a brief-698 number is a claim, never evidence.

## Verification

Probe trees are assembled by you and built only through
`scripts/probe-build.py` (one build per arm and kind; `--force` on one kind;
never `CARGO_TARGET_DIR`). One cargo command at a time; the script enforces
a per-command timeout and a build lock. Serial Callgrind is not needed this
round; if any cost row is run, it is serial, CPU-pinned, release, with every
`RUM_PROFILE_*` cleared except the declared mode, and the witness row states
its mode. A timeout is not data. Derive any bound from data.

## Checkpointing

All scratch under `.temp/brief-699/`; keep `PROGRESS.md` with the exact next
action and the receipts already saved. Leave every tree compiling at each
checkpoint.

## Deliverable

- `.temp/brief-699/report-design.json` in `rum-report/1`, with the control,
  the identity chain, M1/M2/M3 each green (hit and kill) or a measured
  impossibility after a real attempt, the identity-fixture controls, the
  excluded-item rows and every verdict line.
- `.temp/brief-699/drafts/implementation-lp08-final.md` — the live
  implementation brief in `TEMPLATE.md` shape (only if Stage B is green).
- Run `python3 -I scripts/report.py check 699` (without a version-control
  option) and return its summary and all compliance verdict lines.

## Report format

Every run and verdict line quoted; the draft under `drafts/`; the report's
`summary` states the measured identity answer, the three mutant outcomes and
the draft's bar list.
