# Brief 698 — LP08 final form: the work ledger, the attribution and the downstream obligations

**Design-only brief: `Scope: NONE — no live source, workload, manifest,
inventory or version-control change.` Disposable probes only, under this
brief's root. The entry split is the placement of record from brief 695; the
one-line `terminal_owner` binding is frozen here.**

Brief 697 measured the frozen `terminal_owner` form against the literal
operand-identity bar ("no added opcode or operand"), found 17 added and 29
removed operand-level entries, and stopped at Stage B step 5. The director
gate (`.temp/director/brief-697-gate.md`) retired that sub-bar — it requires
register/slot allocation identity from a source change that necessarily
changes a local's live range, the same artifact as the exact-sequence
criterion retired at 696 — and restated the acceptance bar as a **work
ledger**:

1. executed instructions <= base at 50 and 100 iterations in two draws
   (42 vs 54), and no function other than the terminal function changes
   against `entry_split` except the documented identical-binary lookup rows;
2. no opcode is executed more often than in base: the measured delta is
   exactly `{xor -3, test -2, cmpq -1, jne -1, sete -1, mov -1, je -2,
   or -1}` (net -12), added opcodes `{}`;
3. no `call` on either side of the terminal branch, exactly one
   `0x328(%rsp)` load, equal frame bytes (`0x2e8`);
4. memory traffic not increased: loads 9 -> 8, stores 11 -> 10, RMW 1 -> 1,
   every access 8-byte, no new memory region;
5. every delta closed by a finite ledger: 17 added operand entries paired
   one-to-one with 17 re-encoded removals on the same source statement under
   an explicit register/slot map; the remaining 12 removals attributed to
   the duplicate niche/drop test and the drop-flag bookkeeping of the `mut
   unevaluated_restore` parameter; the net -12 decomposed as
   `-6 (late duplicate niche/drop test) -7 (drop flags) +1 (the fused
   cmpq re-encoded as mov+test when the marker load sinks to the branch)`;
6. the ownership invariant, the terminal counter and the five falsifiers
   stay green.

Stage C (the downstream obligations deferred since brief 680) runs only
after Stage B is green on that ledger.

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

`/home/remi/dev/rum/.temp/brief-697/src/terminal_owner/` is the frozen tree
(call.rs sha256 `d01a3a5e2b2ce007c336cf50039cfa97816630f64b55e504b4935cc0ce83c1b1`);
its entry-split basis is `.temp/brief-697/src/entry_split/`. Copy, never edit
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
2. **Reproduce the control on the frozen basis.** Rebuild base, the 686
   form, the frozen entry_split and terminal_owner with the 697 probe inputs
   (`.temp/brief-697/inputs/`, byte-identical). Reproduce: base 54 with one
   `0x328(%rsp)` load; entry_split 55 with two; terminal_owner 42 with one;
   the 386-snapshot trace; the exact 18,304/18,896 stack; coverage
   94/96/416 and the 606 initial witness; the enabled 20/20, 67/67,
   131/19 battery; the 145 -> 129 and 108 -> 84 witnesses; and the five
   falsifiers. Every 697 number is a claim, not evidence.
3. **The terminal source.** State the exact statements of the frozen
   terminal branch (`call.rs` of the terminal_owner tree) and the ownership
   decision they encode, with line numbers. State which path the disabled
   cost workload takes at 50 and 100 iterations (Some or None, by counter,
   not by reading).
4. **Persist before judging.** Save the control's per-address panels, cell
   traces, stack rows, witness panels and falsifier raw logs before any
   attribution is accepted.

## Stage B — the work ledger and its attribution

5. **Re-derive the multiset from the raw panels.** For base and
   terminal_owner, same function, same case, same binary pair, at 50/100
   iterations in both draws: the exactly executed instruction list, the
   opcode multiset, the operand multiset (registers and stack slots exact;
   branch targets normalized for relocation only), the load/store/RMW and
   width counts, the call count, the frame bytes, and the `0x328(%rsp)`
   load count. Callgrind relative offsets are decimal bytes; state the
   resolver you used and show one raw line resolved. Re-derive the 17/29
   multiset and the opcode delta; the 697 numbers are claims. Any added
   instruction, a second load, a call, a wider or new memory region, or an
   opcode increase is a red.
6. **The pairing ledger.** Pair all 17 added operand entries one-to-one
   with 17 of the 29 removed entries. Each pair must state: the added
   address and instruction, the replaced base address and instruction, the
   register/slot map entry that makes them the same statement, and the
   source `path:line` both implement. The expected map is
   `r14 -> r13`, `r13 -> rax`, `rcx -> rsi`, `r14 -> rbx` and
   `0x18 <-> 0x8`, `0x30 <-> 0x38`, `0x20 <-> 0x28`, `0x0 <-> 0x10`,
   `0x8 <-> 0x18`, `0x38(%rsp) <-> %rbp`; the director's independent
   reduction is a hypothesis, not evidence. An unpaired addition or a pair
   whose source statement differs is a red.
7. **The 12 unpaired removals.** Attribute every one to its base source
   statement and show it performs no observable work on the executed path:
   the late duplicate niche/drop test (`mov (%r13),%rcx`, `test`,
   `sete %dl`, `or %al,%dl`, `jne`, `xorl %eax,%eax`) and the drop-flag
   bookkeeping (`movb $0x1,%bpl`, `xorl %ebx,%ebx`,
   `xorl %r15d,%r15d`, `testb %bl,%bl` + `je`, `testb %r15b,%r15b` + `je`).
   Give every removed instruction's base address, its base source
   `path:line`, the taken path at 50/100, and prove the gated drops are not
   executed in base either (the flags are zero on this path). Explain the
   `+1`: the early fused `cmpq $0,0x0(%r13)` is re-encoded as
   `mov (%rax),%rsi` + `test %rsi,%rsi` because the marker load sinks from
   function entry to the branch; this is the only added instruction, and the
   load it introduces is one base also performs.
8. **The ownership invariant, with `path:line`.** Prove on both arms: Some
   consumes `prepared_args` once (`into_vec`), restores every wrapper, in
   order, before normalization, and drops each exactly once; None consumes
   `state.into_final_shared_args()`; the restore Option is moved, never
   cloned, taken, replaced, duplicated or fabricated; the Some arm still
   drops `restore.wrappers` on the non-executed path; the None arm's flags
   are statically false because the null niche selects None. Cite the type
   definition (`SuspendedUnevaluatedRestorePlan` first field
   `prepared_args: SharedArgs(Arc<Vec<Expr>>)`, `rum-core/src/expr.rs:638`)
   and the branch (`call.rs:328-344`). Watch the restoration counter
   (`[1,2,1]`, `wrapper_strong=1`, `prepared_alive=false`) and the
   dropped-Some mutant (`[0,0,0]`, exit 101 named `COUNTER_ASSERT`).
9. **The witness deltas.** Attribute 145 -> 129 (`resumed-call`) and
   108 -> 84 (`resumed-fixed`) instruction for instruction to the same
   bookkeeping, with their base addresses and source lines. These probes
   are `brief690_disabled_resuspension` runs asserting
   `!lexical_profile_enabled` and their raw verdicts say `profiling=false`:
   they are **default-path** witnesses, not enabled ones. Prove no
   publication, guest value, request or API work is removed: the pair
   counters, the 386 snapshots, the 150 key rows and the mirror rows stay
   equal. State the mode on every witness row.
10. **No collateral movement.** Compare every function-level signed delta
    row of terminal_owner against entry_split, in both draws and both
    builds. The only permitted differences are the terminal function
    (55 -> 42 on the argument case; 145 -> 129; 110 -> 84) and the
    `default_builtin_dispatch_spec_for_symbol` rows that the retained
    identical-binary control already shows varying draw to draw; those are
    an instrument qualification, never a cost offset. Any other executed
    count difference is a red or an attributed finding.
11. **The stop.** If any added instruction is unpaired, any removal cannot
    be attributed, any opcode or memory count increases, or the ownership
    invariant fails, STOP with the exact panel and the smallest alternative.
    The alternative direction (a terminal-local lifetime change for
    `CallDispatchState`/`evaluated_head`) is **not authorized in this
    round**; return the measured exclusion for the director to brief
    separately. Do not alter the frozen form to make the ledger close.

## Stage C — the downstream obligations (only after Stage B is green)

12. **The direct flush.** A direct definition finish with no enclosing
    phase records `Outcome::Some`, `faults=0`, counter
    `[1,1,0,1,0,0,0,0,0,0]`, quiescent; the eight call-site signatures and
    the token-first interface unchanged.
13. **The three mispair mutants.** Against that green control they must
    execute and turn the exact counter red with semantic equality disabled.
    A mutant that cannot be killed returns a measured impossibility naming
    its exact counter, the assertion line and why the form cannot produce
    the mispair. "Hit and pass" is not a kill.
14. **The 15 OPEN key routes.** Every OPEN row among the 150 (19 OPEN in
    697) closes by an executed probe or a `path:line` citation. The four
    `evaluate_atom_fast_path.stable_symbol` / `.atom` pause and error rows
    stay source-impossible with `eval_core/mod.rs:168-181` and are not
    reopened. Every prior PASS is restored (131 PASS / 150 rows); a
    `phase_absent` or `faults=N` stays open with its director question.
15. **The host reset probe.** Construct a real `HostSession`, suspend, call
    `HostSession::reset` (`crates/rum-host/src/session.rs:176`), and prove
    old-epoch markers cannot publish into the new epoch, no queued frame is
    discarded, there are no suffix effects, and the retirement is exactly
    once. `session.rs:176` is not patched. A separate bounded host build is
    allowed if the probe requires it; record its exact command and raw
    output.
16. **The contract fixtures.** The `docs/dev/profiling.md` obligations:
    cancellation, Session reset/drop, failed restore, checkpoint rollback,
    explicit discard and profiling reset retire the applicable leases
    exactly once, with no suffix effects or invented returns; old tokens and
    queue markers cannot publish into a new epoch; an owner can resume on
    another thread without losing records; unrelated profiled requests are
    not part of a program's vector. Run the named fixtures in the probe
    build; do not run the workspace acceptance suite.
17. **The draft.** If Stage C is green, draft the live implementation brief
    in `TEMPLATE.md` shape: the entry split plus the frozen one-line
    binding, the ripple inventory, the live gate bars, the downstream rows
    already discharged, and the risks. If any Stage C item is red, record
    the measured exclusion and stop; no live change in this round.

## Goal

When this round is done, the design record states, with measurement and not
argument:

1. **The terminal answer.** The terminal branch executes the base multiset
   minus the attributed bookkeeping, with every addition paired to a base
   instruction on the same source statement, one load, equal frame, no
   call, no memory increase; or the exact measured exclusion.
2. **The ownership answer.** The Some/None invariant with `path:line`, the
   counter and the falsifier; or the exact measured exclusion.
3. **The witness attribution.** The 145 -> 129 and 108 -> 84 deltas
   explained as the same bookkeeping, with the profiling-disabled mode
   stated and no publication or guest work removed; or the exact measured
   exclusion.
4. **The downstream record.** The direct flush, the three mispair results,
   the 15 key routes, the host reset and the contract fixtures, each green
   or a measured exclusion.
5. **The plan.** The live implementation brief in `TEMPLATE.md` shape.

## Context pointers

- `.temp/director/brief-697-gate.md` — the restated work-ledger bar and the
  frozen form.
- `.temp/brief-697/report-design.json`, `subset-results697.json`,
  `terminal-subset.tsv`, `all-delta-rows697.json`, `all-stack-rows697.json`,
  `control-results697.json`, `closure-results697.json`,
  `src/terminal_owner/`, `src/entry_split/`, `inputs/`, the 697 cost,
  coverage and trace profiles — read/copy only.
- `.temp/director/probe-697/terminal-ledger.json` and `pairing.json` — the
  director's independent reduction; a hypothesis, not evidence.
- `.temp/brief-695/`, `.temp/brief-694/`, `.temp/brief-693/` — prior controls
  and cuts, read-only.
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

Files you MAY write: anything under `.temp/brief-698/` — probe trees, builds,
binaries, receipts, harness code, the draft implementation brief and the
report.

MUST NOT touch:

- the live repository tree — no source, test, workload, manifest, inventory,
  docs or `.git` change; you run no version-control command;
- `.temp/brief-686/` … `.temp/brief-697/` or any other retained root, except
  to read and copy; never execute a file in place;
- `.temp/probe/` except through `scripts/probe-build.py`;
- the public API, features, serde, manifests, dependencies, `unsafe`.

Do not run the workspace acceptance suites, the semantics matrix, the census
or inventory replays, or any source gate. This round measures and discharges
the named obligations; it repairs nothing live.

## Hard bars

- **The work ledger.** The Stage B rules above. A source-identical function
  is not a cost claim by itself; the panels and the ledger decide.
- **The frozen forms.** The entry split (four call-site bindings and
  `pause_call_stage_at_finish(_slow)`) and the one-line terminal binding are
  byte-identical to `.temp/brief-697/src/`. No form change may be made to
  make a downstream item pass; a downstream red excludes the form, it does
  not license an edit.
- **Position and semantics.** The 386-observation cell trace equals the 686
  control's in every case and every draw; the 20 pairs, 67 mirrors and 150
  key rows keep their 697 values; a publication is never dropped, postponed
  past its owner, or merged into a path that loses an open record.
- **The enabled form.** 20/20, `faults=0`, exact 18,304/18,896 stack in two
  draws, coverage 94/96/416, five falsifiers. No assertion or fixture is
  weakened.
- **The excluded forms.** All forms excluded through brief 697 stay
  excluded: the 683 observe-only guard, the 684 position-only shape, the
  685 ambient guard, the panicking take-path fallback, both 687 cost shapes,
  the 688 owner-position placement, the 689 candidate1 shape, the 690
  reload1 and reload2 terminal shapes, the 691 `attach_helper`/
  `attach_publish` shapes, the 692 whole-body shapes, the 693
  `suspension_local_phase` duplicate-pause shape, the 694
  `suspension_single_completion` move, the 695 `keyed_completion` form and
  the 696 `terminal_fields` shape. `terminal_owner` is not excluded: it is
  the form of record pending this round's ledger.
- **Every arm reduced.** Every measurement exists in a persisted artifact; a
  run that exists only as a log is not evidence. No row dropped, no negative
  netted. Raw output is persisted before parsing.
- Every count, key, route and `path:line` is re-derived from the frozen
  trees; a brief-697 number is a claim, never evidence.

## Verification

Probe trees are assembled by you and built only through
`scripts/probe-build.py` (one build per arm and kind; `--force` on one arm
per round; never `CARGO_TARGET_DIR`). One cargo command at a time; the
script enforces a per-command timeout and a build lock. Two measurement
draws per arm; serial Callgrind, CPU-pinned, release, every `RUM_PROFILE_*`
cleared except the declared modes; four stack heights twice each with the
residual printed. A timeout is not data. Derive any bound from data.

## Checkpointing

All scratch under `.temp/brief-698/`; keep `PROGRESS.md` with the exact next
action and the receipts already saved. Leave every tree compiling at each
checkpoint.

## Deliverable

- `.temp/brief-698/report-design.json` in `rum-report/1`, with the control,
  the work ledger and its source attribution, the ownership evidence, the
  witness attribution, the downstream record, the excluded-item rows and
  every delta row.
- `.temp/brief-698/drafts/implementation-lp08-final.md` — the live
  implementation brief in `TEMPLATE.md` shape (only if Stage B is green;
  Stage C rows may remain marked NOT_RUN only if Stage B is red).
- Run `python3 -I scripts/report.py check 698` (without a version-control
  option) and return its summary and all compliance verdict lines.

## Report format

Every run and verdict line quoted; the draft under `drafts/`; the report's
`summary` states the measured answers and the draft's bar list.
