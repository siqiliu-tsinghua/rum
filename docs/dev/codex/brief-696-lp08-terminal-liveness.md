# Brief 696 — LP08 residual: the terminal liveness form

**Cost design on the frozen entry_split basis: `Scope: NONE — no live
source, workload, manifest, inventory or version-control change.`
Disposable build probes only, under this brief's root. Entry_split is the
placement measured in brief 695 (green on every bar except the terminal
reload); it is frozen here. The 686 form stays the enabled/stack control.**

Brief 695 measured two things. First, the keyed completion interior is
red on three bars (terminal 55, stack +64 B per level, preparation and
evaluation residuals) and is excluded. Second, the entry split — the four
`eval_core/call.rs` pause sites bound to a new `pause_call_stage_at_finish`
entry carrying the candidate2 pause body, with the base
`pause_phase_at_finish` kept for the two `resolution.rs`
`evaluate.once_unstable` sites — is green on everything except one
instruction: the terminal branch of
`resume_suspended_call_with_dispatch_plan` measures 55 against base 54,
loading `0x328(%rsp)` twice instead of once. Its enabled battery, 386-
snapshot trace, exact 18,304/18,896 stack, 94/96 coverage witnesses and
five falsifiers are all green.

Brief 695 also measured that the disabled workload executes none of the
new code: `complete_phase`, both pause slow functions and
`publish_stashed_phases_slow` record zero self and zero calls in every
cost profile. The +1 is therefore a codegen consequence of the source's
presence, not of its execution, and it is confined by the entry split to
the terminal function itself.

This round answers one question with measured evidence: **with the
entry_split placement frozen byte-identical, can a terminal-local
ownership/liveness change inside
`resume_suspended_call_with_dispatch_plan` — moving locals, not control
flow — make the terminal branch execute the exact base instruction
sequence again (54 instructions, one `0x328(%rsp)` load, base
opcode/operand sequence at both iteration counts), with the entry-split
green battery, trace, exact stack, coverage witnesses and five watched
falsifiers preserved; or is that measured impossible?**

## Stage A — calibration and the frozen basis

1. **The build protocol (new, user ruling 2026-11-07).** Probe builds go
   through `scripts/probe-build.py`. Read its docstring first. The caller
   assembles the complete tree and passes `--tree <dir> --out <binary>
   --receipt <json>`; the script copies it to the stable path
   `.temp/probe/active`, keys the artifact cache on the assembled-tree
   hash × rustc version × flags, and reuses a byte-identical binary when
   the tree matches. There is **one build per (arm, kind)**; the second
   build index (`build1/2`, `diagnosis1/2`, `coverage1/2`) is retired.
   Measurements still use **two draws**. One arm per round must be
   rebuilt with `--force` and its sha256 compared to the cache: that is
   the determinism spot-check, and its verdict line must be quoted.
   Never set `CARGO_TARGET_DIR` yourself; never change opt-level,
   codegen-units or incremental. A cache hit is acceptable for base and
   entry_split (hash identity is the evidence); the new candidate arm is
   a fresh build.
2. **Reproduce the 695 control on the frozen basis.** From
   `.temp/brief-695/src/entry_split/` and the retained patches
   (`entry_split.patch`), rebuild base, form, candidate2, keyed_completion
   (the excluded keyed form, for the control) and entry_split with the
   same probe inputs as 695 (`brief-695/inputs/`). Reproduce: base 54
   with one load; form, candidate2, keyed and entry_split 55 with two;
   the 386-snapshot trace; the exact 18,304/18,896 stack; coverage 94/96;
   the five falsifiers; the enabled 20/20, 67/67, 131/19 battery; the
   disabled zero-hit panel for the new code. Every 695 number is a claim,
   not evidence.
3. **The terminal source.** Read `resume_suspended_call_with_dispatch_plan`
   in the entry_split tree (the 695 report points at `call.rs:319`, with
   the branch at `:328` and the restoration match at `:329`). Reproduce
   the measured reload: on the None route the `unevaluated_restore` stack
   slot is loaded into `%rax` for the discriminant and again into `%r13`
   for ownership/drop handling; base loads it once into `%r13`. State the
   exact source statements and their line numbers in the entry_split
   tree, and keep that finding as the basis for Stage B.
4. **The retained green.** Persist the control's per-address cost panels,
   cell traces and stack rows before judging a candidate.

## Stage B — at most two terminal-local candidates

5. **Shape 1, the terminal liveness form (primary).** Change only local
   bindings in `resume_suspended_call_with_dispatch_plan`, inside the
   entry_split tree: hoist, bind or reorder the `unevaluated_restore`
   value and its ownership decision so the terminal branch reads the
   stack slot once and reuses it for both the discriminant and the
   ownership/drop handling. Constraints:
   - no control-flow change: the `stage_index >= plan.len()` branch, the
     Some/None split and every early exit stay;
   - no semantic change: Some restores all prepared arguments and every
     Unevaluated wrapper before normalization, None uses the final shared
     arguments, the restoration plan is owned and dropped exactly once;
   - no new state, no signature change, no new `unsafe`, no duplicated
     body, no `unreachable!`/`expect` on a state the types allow;
   - only that function changes; the entry_split placement, the new
     pause entry, the four call sites, the base entry, the queue, the
     collector and every other file stay byte-identical.
   At most two candidates, one variable each. Each candidate carries a
   source-level argument for why it could remove the reload, then the
   measurement decides.
6. **Ordering and semantics.** The DownValues attachment stays
   immediately after the consumer push and before its publication; the
   publication follows the attachment or the stash; `EvaluateOnceUnstable`
   never publishes at completion; exactly one phase completion per phase.
   Owner/epoch/control authentication, the exactly-once drain and the
   ownerless restore stay byte-identical.
7. **The acceptance bar.** The terminal branch executes the exact base
   instruction sequence at 50 and 100 iterations: 54 instructions
   normalized, one `0x328(%rsp)` load, opcode and operand sequences equal
   to base, in two draws and one fresh candidate build, with the sha256
   determinism spot-check. The terminal Some/None ownership invariant is
   stated and preserved. If no candidate reaches it without violating an
   invariant, return a measured impossibility naming the exact reload,
   the candidates, the entailing statements, the excluded forms and the
   smallest alternative. No universal impossibility beyond the
   measurement.
8. **Constraints.**
   - **Zero at every former site.** The entry_split placement is frozen;
     the changed function's non-terminal work must not add a
     default-path instruction. The disabled run must show the base
     sequence and count at every former publication site.
   - **Position.** The 386-observation cell trace must equal the 686
     control's in every case and every draw.
   - **Identity.** The entry_split ownership authentication, the
     exactly-once drain, the ownerless restore, the DownValues consumer
     frontier, the pause / suspend order, the decoded view, the canonical
     `Option<Expr>` decoding, the typed adapter, `LegacyMode`, the
     transport, the guest carriers and the eight definition finishes stay
     byte-identical. No test or fixture is weakened.
   - **No hidden state.** No new evaluator-frame field, no session field
     carrying diagnostic state, no selector, no outlined guest adapter,
     no new `unsafe`, no `+64` publication frame.
   - **One variable per candidate.** At most two cost candidates;
     diagnosis arms are not candidates.

## Stage C — acceptance on the green cost form

9. **Enabled.** Queue 9/9 with parser quiescence `(0,0,0,0,0)` and
   strong-count ownership equality; slot 2/2; record 6/6; direct
   definition flush `Outcome::Some`, `faults=0`, counter
   `[1,1,0,1,0,0,0,0,0,0]`, quiescent; six lifecycle actions
   `open=0 leases=0 fabricated_ok=0`; all 20 pairs `native_same=true`
   `faults=0`; mirrors 67/67; seven Throw counter-only kills; all 150
   key rows with every prior PASS restored.
10. **Stack.** Function 18,304 / Module 18,896 B per level, residual 0,
    four heights twice each, on the same candidate that passed the
    battery.
11. **Coverage.** The two resumed-helper witnesses measure at the
    base-identical 94 and 96 self instructions with equal opcode and
    operand sequences in two draws; the prepared helper stays a
    source-proved exclusion with its revalidation triggers restated.
12. **Cost.** Every former publication site zero versus base in the same
    run; the terminal branch at the base instruction sequence; every
    remaining disabled delta attributed to the queued path's own standing
    start/record calls or removed; the `builtin_catalog` lookup variation
    qualified by a same-binary repeated comparison. No netting of
    unrelated negative deltas.
13. **Falsifiers.** Re-watch the five 692 falsifiers on the final
    measured shape: position (`R,G,B` → `B,R,G`), owner-loss,
    wrong-owner / premature, exactly-once, and the terminal invariant
    (dropped or duplicated Some restoration observed by its counter
    assertion). Each carries its exact counter or trace assertion and its
    observed red.
14. **If Stage C is green** — do not run the downstream obligations
    (mispair, the 15 key routes, the contract fixtures) in this round;
    record the green cost form and draft the next round for them. If it
    is red, record the measured exclusion and stop.

## Goal

When this round is done, the design record states, with measurement and
not argument:

1. **The frozen-basis control.** Every 695 bar reproduced on the
   entry_split tree under the new build protocol, with the cache-hit and
   spot-check verdict lines quoted.
2. **The terminal answer.** A terminal-local ownership/liveness form
   that restores the exact base terminal sequence with every entry-split
   green preserved, or the exact measured impossibility.
3. **The preserved green.** The position trace, enabled battery, exact
   stack, coverage witnesses and five watched falsifiers on the final
   shape.
4. **The plan.** The updated drafted implementation brief in
   `TEMPLATE.md` shape, with the ripple inventory, risks and open
   questions.

## Context pointers

- `.temp/brief-695/report-design.json` — the 695 exclusion record and the
  entry_split measurements; `entry_split.patch` and
  `src/entry_split/` — the frozen placement.
- `.temp/brief-695/inputs/` — the probe inputs (byte-identical to 694).
- `.temp/brief-695/report-design.json` section "Measured answer and
  smallest alternative" — the reload description and the reason the
  entry split is the smallest scope.
- `.temp/director/brief-695-gate.md` — the rulings and the frozen
  placement.
- `scripts/probe-build.py` — the build protocol (read the docstring).
- `.temp/brief-694/` and `.temp/brief-693/` — the prior controls and
  cuts, read-only.
- `docs/dev/profiling.md` — the contract.
- Live sites: `profiling/logical/mod.rs` (`complete_phase`,
  `pause_phase_at_finish(_slow)`, the entry-split entry,
  `publish_stashed_phases_slow`, `take_phase`, `PhaseKey`),
  `profiling/timing.rs` (both adopted variants), `eval_core/call.rs`
  (`resume_suspended_call_with_dispatch_plan`, the four stage pause
  sites), `resolution.rs`, `profiling/logical/collector.rs`,
  `suspension/core/storage.rs`, `suspension/core/profile_queue.rs`.
- `docs/dev/codex/TEMPLATE.md` and `AGENTS.md`.

## Scope

Files you MAY write: anything under `.temp/brief-696/` — probe trees,
builds, binaries, receipts, the draft implementation brief and the
report.

MUST NOT touch:

- the live repository tree — no source, test, workload, manifest,
  inventory, docs or `.git` change; you run no version-control command;
- `.temp/brief-686/` … `.temp/brief-695/` or any other retained root,
  except to read and copy. Never execute a file in place;
- `.temp/probe/` except through `scripts/probe-build.py`;
- the public API, features, serde, manifests, dependencies, `unsafe`.

Do not run the workspace acceptance suites, the semantics matrix, the
census or inventory replays, or any source gate. This round designs and
probes; it repairs nothing live.

## Hard bars

- **The disabled cost bar.** The default path pays only the queued
  path's own standing `runtime_path_timing_start` /
  `record_profile_runtime_path_timing` calls, measured in the same run.
  The entry_split placement is byte-identical; the changed terminal
  function must not add a gate, a call or an instruction on the disabled
  path, even at +1. A source-identical function is not a cost claim by
  itself.
- **The position bar.** The queue tail and every intermediate snapshot
  equal the 686 control's in all 20 trace cases, in both draws.
- **Publication semantics.** A publication may not be dropped,
  postponed past its owner, or merged into a path that loses an open
  record. Exactly one phase completion; owner/epoch/control
  authentication, exactly-once completion and the ownerless restore stay
  stated with `path:line`.
- **The enabled form.** 20/20, `faults=0`, the unchanged battery and the
  exact 18,304/18,896 stack in two draws and one fresh build plus the
  sha256 spot-check. No assertion or fixture is weakened.
- **The excluded forms.** All forms excluded through brief 695 stay
  excluded: the 683 observe-only guard, the 684 position-only shape, the
  685 ambient-owner guard, the panicking take-path fallback, both 687
  cost shapes, the 688 owner-position placement, the 689 candidate1
  shape, the 690 reload1 and reload2 terminal shapes, the 691
  `attach_helper` and `attach_publish` shapes, the 692 `whole_body` and
  `whole_phase_first` shapes, the 693 `suspension_local_phase`
  duplicate-pause shape, the 694 `suspension_single_completion` move and
  the 695 `keyed_completion` form. Every diagnosis cut is diagnosis only.
- **Every arm reduced.** Every candidate's stack and battery runs are
  reduced into the persisted artifact; a run that exists only as a log
  is not evidence. No row dropped, no negative netted.
- Every count, key, route and `path:line` is re-derived from the frozen
  entry_split tree; a brief-695 number is a claim, never evidence.

## Verification

Probe trees are assembled by you and built only through
`scripts/probe-build.py` (one build per arm and kind; `--force` on one
arm per round; never `CARGO_TARGET_DIR`). One cargo command at a time;
the script enforces a per-command timeout and a build lock. Two
measurement draws per arm; serial Callgrind, CPU-pinned, release, every
`RUM_PROFILE_*` cleared except the declared modes; four stack heights
twice each with the residual printed. A timeout is not data. Persist raw
output before parsing; derive any bound from data.

## Checkpointing

All scratch under `.temp/brief-696/`; keep `PROGRESS.md` with the exact
next action and the receipts already saved. Leave every tree compiling
at each checkpoint.

## Deliverable

- `.temp/brief-696/report-design.json` in `rum-report/1`, with the
  frozen-basis control, the candidate evidence (or the exact
  impossibility), the re-watched falsifiers, the green enabled/stack
  battery and every delta row.
- `.temp/brief-696/drafts/implementation-lp08-residual-20.md` — the
  updated implementation brief in `TEMPLATE.md` shape.
- Run `python3 -I scripts/report.py check 696` (without a
  version-control option) and return its summary and all compliance
  verdict lines.

## Report format

Every run and verdict line quoted; the drafted implementation brief
under the draft directory; the report's `summary` states the measured
answers and the draft's bar list.
