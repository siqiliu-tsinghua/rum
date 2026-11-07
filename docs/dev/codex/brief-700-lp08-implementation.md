# Brief 700 — LP08 live implementation: the frozen completion form and terminal owner

**Implementation brief. One live change, one gate.** The design record is
closed: the entry split is the placement of record (briefs 693–695), the
one-line `terminal_owner` binding's work ledger is closed (briefs 696–698),
and the token-identity class falsifiers all kill (brief 699: M1 3/3, M2 1/1,
M3 3/3, exit 101, assertions unchanged). This brief installs that frozen form
in the live tree, byte-for-byte, and re-proves it on fresh binaries.

The frozen target is
`/home/remi/dev/rum/.temp/brief-699/src/terminal_owner/` (call.rs sha256
`d01a3a5e2b2ce007c336cf50039cfa97816630f64b55e504b4935cc0ce83c1b1`); its
frozen base is `.temp/brief-699/src/base/`. The complete live source scope is
the 15-file delta between them. The design trees are read/copy only.

## Goal

Install the accepted LP08 form with **exactly** the frozen target bytes,
preserve the guest evaluation, capture/observation order, typed continuation
shape, every logical owner and outcome, and the measured disabled-path work.
Replay the standing inventories before acceptance, run the full acceptance
suite once, and re-measure every gate below on fresh binaries:

- the terminal work ledger (54 → 42 instructions, 17 paired additions, 29
  attributed removals, `-6-7+1=-12`, one `0x328(%rsp)` load, no call, frame
  `0x2e8`, memory `9/11/1 → 8/10/1`);
- the ownership counter and the dropped-Some falsifier;
- the 386-snapshot position check, the 18,304/18,896 stack, coverage
  `94/96/416/606`;
- the default-path witnesses 145 → 129 and 108 → 84, with per-instruction
  attribution;
- the five placement/ownership falsifiers;
- the direct definition flush, the 150 key routes, the contract/lifecycle
  fixtures and the host reset's profiling obligations;
- the three identity classes M1/M2/M3 on the **new** binary;

and run the rum-vs-rum core-language microbench A/B. Leave the working tree
uncommitted for the director's gate.

## Context pointers

- Read `AGENTS.md`, `docs/testing-strategy.md`, `docs/dev/profiling.md`,
  `docs/code-organization-guidelines.md`, and the `HostSession::reset`
  rustdoc (`crates/rum-host/src/session.rs:166-179`).
- Frozen source: `.temp/brief-699/src/base/` and
  `.temp/brief-699/src/terminal_owner/`; the complete delta is
  `.temp/brief-699/frozen-base-to-terminal.txt`.
- Design record: `.temp/director/brief-695-gate.md` … `brief-699-gate.md`;
  the 699 report and harness under `.temp/brief-699/` are read/copy only.
- The plan basis: `.temp/brief-699/drafts/implementation-lp08-final.md`
  (515 lines). My brief overrides the draft wherever they differ; do not
  dispatch, copy or edit the draft.
- The 699 identity harness (`identity_mutants.rs`, `identity_fixture.rs`, the
  three hook edits and the appended key-route test) is **probe-only** and
  must not enter the live tree.

## Scope

The following table is the complete live source scope, derived by comparing
the frozen base and target bytes. Files you MAY modify or add:

| File under `crates/rum-runtime/src/runtime/` | Action | Responsibility | Target bytes |
| --- | --- | --- | ---: |
| `eval_call_prepare.rs` | modified | Resume append observation at the same boundary as the queued append. | 22104 |
| `eval_core/call.rs` | modified | Four call-stage entry bindings and the whole-Option terminal owner. | 27054 |
| `profiling/logical/collector.rs` | added | Enabled process-wide stash and owner-specific drain hooks. | 3292 |
| `profiling/logical/completion.rs` | added | Deferred definition flush and decoded completion policies. | 4910 |
| `profiling/logical/epoch.rs` | modified | Deferred-record ownership, reset retirement and a valid existing epoch. | 14668 |
| `profiling/logical/mod.rs` | modified | Pause/complete plumbing, outcome policies and the call-stage split. | 42746 |
| `profiling/logical/queue_api.rs` | modified | Typed queue completion entry and downstream boundary insertion. | 3803 |
| `profiling/logical/segments.rs` | modified | Resume ancestry through completed records without restarting them. | 10977 |
| `profiling/timing.rs` | modified | Preserve the physical recorder while completing the logical carrier. | 24981 |
| `resolution.rs` | modified | Existing fixed-point pause entry; resumed equality/stability/trace observations. | 20706 |
| `suspension/core.rs` | modified | Register the existing pending profiled owner. | 24527 |
| `suspension/core/profile_queue.rs` | modified | Exact consumer identity, owner registry and diagnostic boundary placement. | 22080 |
| `suspension/core/profile_queue/arena.rs` | modified | AVL insertion before the supplied consumer identity. | 13727 |
| `suspension/core/profile_queue/tests.rs` | modified | `queue.root` → `queue.root_id()` only; every assertion stays. | 5783 |
| `suspension/core/storage.rs` | modified | Publish only in the existing Profiled successful-push branch. | 9283 |

The 13 existing files are byte-identical to the frozen base at HEAD; the two
additions are absent. Verify all 15 prerequisites before the first edit:
every live file must match its frozen-base bytes. If any does not, STOP for
director reconciliation; do not overwrite unrelated work.

Standing maintenance is always in scope, whatever the fence says: the
retention contract fingerprints, the certificate cells, the ownership
discovery/positions, the LP01 producer/binding/loss receipts, the typed-slot
receipts, the frame-root closed lists and the rejection census. Replay them
with the repository's own mechanisms; never hand-edit, reclassify or weaken a
row. Disclose every changed row.

MUST NOT touch: any other live file; `rum-core`; public APIs; serde shapes;
features; manifests; `Cargo.lock`; dependencies; workloads; docs; `.git` or
any version-control command; the retained `.temp/brief-686…699` roots except
to read/copy; `.temp/probe/` except through `scripts/probe-build.py`; any
new `unsafe`.

## The frozen forms

The four call-stage sites keep their existing witnessed consumer push and
then call the following entry. The two fixed-point sites keep their separate
`pause_phase_at_finish` entry. Copy the function verbatim from the frozen
target; do not re-derive it:

```rust
#[inline]
pub(in crate::runtime) fn pause_call_stage_at_finish(
    session: &mut crate::runtime::Session,
    phase: &'static str,
) {
    if !crate::runtime::profiling::lexical_profile_enabled() {
        return;
    }
    pause_call_stage_at_finish_slow(session, phase)
}

#[cold]
#[inline(never)]
fn pause_call_stage_at_finish_slow(session: &mut crate::runtime::Session, phase: &'static str) {
    if let Some(key) = PhaseKey::from_name(phase) {
        if key == PhaseKey::CallStageDownValues {
            // Director-approved exception for this diagnostic marker only.
            // The witnessed dispatch consumer has just been pushed; bind this
            // invocation before another push can replace its insertion handle.
            let Some(entry) = take_phase(key) else { return; };
            let Some(call) = entry.call else { return; };
            if let Some(boundary) = call.pause() {
                session.logical_insert_down_values_before_consumer(boundary);
            }
        } else {
            complete_phase(
                session,
                key,
                super::RuntimeProfileOutcome::InputRequired,
                |_| {},
            );
        }
        // The fixed-point witness has not pushed its R/G consumer yet.
        // All other finish pauses follow their witnessed consumer.
        if key != PhaseKey::EvaluateOnceUnstable {
            publish_stashed_phases_slow(session);
        }
    }
}
```

The terminal branch is verbatim; the surrounding function signature, `mut`
parameter, loop, Some body and return stay frozen:

```rust
            if stage_index >= plan.len() {
                let terminal_restore = unevaluated_restore;
                let final_args = match terminal_restore {
                    Some(restore) => {
                        let mut restored_args = restore.prepared_args.into_vec();
                        for (position, wrapper) in restore.wrappers.iter() {
                            restored_args[*position] = wrapper.clone();
                        }
                        self.normalize_call_args_shared(
                            evaluated_head.as_ref(),
                            SharedArgs::new(restored_args),
                            plan.attributes,
                        )
                    }
                    None => state.into_final_shared_args(),
                };
                return Ok(Expr::call_shared(evaluated_head, final_args));
            }
```

The entry-split-to-terminal delta changes only `call.rs`: one whole-Option
binding and the match scrutinee. Do not replace it with `.take()`, an
`as_ref` match, field moves at a different time, clones, a helper call or an
early return.

The target bytes are the acceptance shape. The frozen target contains a
42,746-byte `logical/mod.rs` and several 20k–25k files; this brief carries a
**form freeze** for this mechanical install. Do not split, rename, reorganize
or reformat those files. A separate design line after LP08 owns the split and
must re-measure it.

## Stages

**A — Transfer.** Snapshot the live base and verify the 15 prerequisites.
Copy the exact frozen target bytes for the 15 files; change nothing else.
Run `cargo check --all-targets` and the focused `-p rum-runtime` tests while
integrating. No helper redesign, duplicated body, representation flattening,
silent-drop helper, lazy initialization, capture movement or
`debug_assertions`-keyed inline tuning is allowed. A departure from the
frozen form is a STOP, not a repair.

**B — Ownership and source stop.** Check the whole target byte identity: the
15 files equal the frozen target bytes; every other file in `crates/` equals
the frozen base bytes. Confirm the ownership invariant: the terminal Some arm
consumes `prepared_args` exactly once, restores every wrapper in order before
normalization, and releases the wrappers exactly once; the None arm consumes
`state.into_final_shared_args()`; the whole Option is moved, never cloned,
taken, replaced, duplicated or fabricated. The logical key remains the
token-held Call's record path, never the free physical-timing index. Preserve
the typed suspension frames, InputRequired propagation, marker identity,
original consumer position and incoming Return/Throw sampling. STOP on any
byte difference, any `unsafe`, any public-API/serde/feature/manifest change,
or any default-path cost.

**C — Replay, then acceptance.** Replay the standing inventories as the last
step of the edit stage, before the first acceptance suite; then run
`cargo fmt --check`, `cargo test -p rum-runtime`, and `cargo test --workspace`
once. Run the two-feature retention/suspension contract gates in the
repository's final command set. A red whose only cause is a stale derived cell
is an ordering artifact: replay and rerun; never weaken a test, fixture or
assertion. After any later source edit, repeat the replay and the acceptance
ordering.

**D — Fresh proof and performance.** Build the base and candidate probe arms
from the new live tree and from the live base, with the frozen harness overlay
copied into a disposable tree under `.temp/brief-700/`; the identity hooks and
mutant switch stay there and never enter the live tree. Rebuild through
`scripts/probe-build.py` (stable `.temp/probe/active`, one build per arm/kind,
one `--force` sha256 spot-check per round; never `CARGO_TARGET_DIR`). Run the
unmodified controls and calibrated falsifiers first, then the serial
instruction/stack/coverage checks and the rum-vs-rum microbench. Do not infer
a disabled-path pass from enabled tests. Do not net a negative row against a
gain; every row is retained and reduced before judgment.

## Measured bars and their protected behavior

| Gate | Exact acceptance bar | Must not change |
| --- | --- | --- |
| Terminal work | 54 → 42 instructions; 17 additions paired; all 29 removals attributed; 12 unpaired removals explained; `-6-7+1=-12`; no opcode increases; one `0x328(%rsp)` load; no call; frame `0x2e8`; explicit 8-byte memory loads/stores/RMW `9/11/1 → 8/10/1` | Same branch, captures, plan pointer, result stores, Some restoration, drop order and guest work. `%rcx` at entry is the plan pointer; `%rdi` is sret. |
| Ownership | Terminal counter `[1,2,1]`, `wrapper_strong=1`, `prepared_alive=false`; dropped-Some counter `[0,0,0]` must kill the unchanged `COUNTER_ASSERT` | Whole-Option move; each prepared argument and wrapper restored once before normalization; no clone, `take`, or omitted release. |
| Position | 386 snapshots across 20 cases and two repeats; queue cells and observation order identical | Fixed-point `R,G,B` order; exact down-values consumer; every repeat and queue position. |
| Stack | Function body 18,304 and Module body 18,896 bytes/level; residual zero over the retained heights/repeats | Native continuation shape and all stack rows; no moving work to hide a frame. |
| Coverage | Disabled-profile helper witnesses `94/96/416/606` for resumed call / resumed fixed point / call stage / initial fixed point | Actual execution of each helper, original inputs, `profiling=false`; keep all other coverage rows. |
| Disabled witnesses | `resumed-call` 145 → 129 and `resumed-fixed` 108 → 84, complete per-instruction attribution; four resolved-path calls remain | No publication, guest value, request or payload-copy work removed. State the mode on every row. |
| No collateral | 2,466 rows: 2,453 unchanged, 8 terminal rows (the accepted ledger), 5 qualified `default_builtin_dispatch_spec_for_symbol` lookup rows | The identical-binary process-hash variation is disclosed and never used as a cost offset; all other nonterminal deltas are zero. |
| Disabled diagnostics | Cold completion/pause/publication helper calls and self counts are zero on the disabled witnesses | Only the existing cold-load flag path; no off-mode mutex, allocation or extra per-call branch. |
| Direct definition flush | All three RHS kinds, no enclosing phase: `Outcome::Some`, faults 0, `[1,1,0,1,0,0,0,0,0,0]`, quiescent, semantic equality disabled; exact counter at `brief677_probe.rs` `COUNTER_ASSERT brief698 direct definition flush` | Exact counter text/line, token-held key and physical recorder signature; no key or assertion edit. |
| Pair/mirror/key controls | 20/20 native pairs, 67/67 mirrors, all 150 original rows unchanged; 131 original PASS + 13 supplemental PASS + six source-impossible rows, zero unresolved | Every key, multiplicity, owner, route, result channel and assertion; no deletion or recategorization of original rows. |
| Contract/host lifecycle | Exactly-once retirement, no suffix effects or old success; old-token/queue isolation, thread handoff, unrelated-owner isolation | Reset retains its documented guest-discard semantics (`session.rs:166-179`). Pending clears; 29 open records retire once; a second reset adds nothing; a fresh evaluation works. |
| Identity kills (new binary) | Controls green with the mutants off; M1 3/3, M2 1/1, M3 3/3 hit and kill their exact checks with exit 101; same binary for control and mutant | The 699 assertion texts and fixture bytes; no hook in the live tree; no assertion weakened. The 699 SHA is provenance only. |
| Microbench | `scripts/core-language-microbench.sh` on base and candidate; compare each `timings.tsv` label | Same statements and modes, all hot labels and same-tier neighbors; profiling off, in-kernel `AbsoluteTiming`, no competing CPU-heavy workload; no hidden regression. |

The six source-impossible routes are the pause/error routes for
`evaluate_inner_unstable.atom` (`eval_core/mod.rs:159`, direct `Ok(other)`) and
for `evaluate_atom_fast_path.stable_symbol` / `.atom`
(`eval_core/mod.rs:174`, direct `Ok(expr)`). They are exclusions with source
proof, never executed passes. The other 13 formerly open rows are the 698
supplemental fixtures; report the 150-row original matrix and the
supplemental dispositions separately.

### Five existing falsifiers

Run each unchanged control and mutant on both probe arms; the counter checks
and the position reducer are immutable:

- `owner-loss`: mutant exit 101,
  `COUNTER_ASSERT post-consumer publication exactly once without owner loss`.
- `wrong-owner`: mutant exit 101,
  `COUNTER_ASSERT foreign-control remains stashed until own consumer`.
- `exactly-once`: mutant exit 101,
  `COUNTER_ASSERT post-consumer publication exactly once without owner loss`.
- `position`: mutant exit 1, `TRACE_ASSERT fixed-point tail must remain
  R,G,B; mutant produced B,R,G` and `TRACE_ASSERT post-consumer cell
  sequence`.
- `dropped-Some`: mutant exit 101, `COUNTER_ASSERT terminal Some restoration
  exactly once, all prepared arguments and every wrapper before
  normalization`.

### Identity kills on the new binary

Assemble a disposable identity tree from the new implementation source plus
the 699 harness. Copy `identity_mutants.rs` and `identity_fixture.rs`, and
insert the three hook calls at the same functions, not at fixed line numbers:
`complete_definition_rule` (M1 wrong-owner), the `flush_definition_boundaries`
pop (M3 wrong-record), and `Segment::end` before the unchanged triplet guard
(M2 stale-epoch). Keep the assertions byte-identical. The controls run with
`BRIEF699_MUTANT` unset; the only switch is that variable. Each class must
print both identities and kill the named check; a mutant that hits and passes
is a STOP, not a pass. The frozen live tree must contain no hook, no switch
and no mutant module.

## Verification (mandatory)

- `cargo fmt --check`.
- `cargo test -p rum-runtime`.
- `cargo test --workspace` — once, at acceptance; mid-change is
  `check --all-targets` plus `-p rum-runtime`.
- The two-feature retention/suspension contract gates in the director's
  finalized command set.
- The standing inventory replays, before the first acceptance suite; quote
  every verdict line and every changed row.
- All measured gates above, from fresh binaries and fresh draws; raw output
  persisted before parsing; every run and failed attempt listed.

## Instruments and exclusions

- Probe builds through `scripts/probe-build.py`; one build per (arm, kind);
  one forced sha256 spot-check; never `CARGO_TARGET_DIR`; never change
  opt-level, codegen-units or incremental.
- Callgrind relative offsets are **decimal bytes** (`+26` = 26); `*` repeats
  the previous address. Serial, CPU-pinned, release; every `RUM_PROFILE_*`
  cleared except the declared mode; the witness row states its mode.
- The `default_builtin_dispatch_spec_for_symbol` lookup row varies draw to
  draw on an identical binary; it is an instrument qualification, never cost
  evidence.
- A timeout is not data; derive bounds from saved same-shape measurements.
- Exclusions stay explicit and restated: the six source-impossible routes; the
  process-hash lookup variation; the legacy free-index mutants (they change
  physical timing, not the logical key); the withdrawn host guest-frame
  preservation clause. Do not revive the host no-discard test.
- The 13 former OPEN key rows are supplemented, not recategorized; the
  original 150-row matrix stays intact.
- New `UnknownConstruct` rows for code this brief adds or changes are
  expected: add each with its file and line, leave unknowns unknown. STOP if a
  row at HEAD changes identity, status, disposition or exclusion, if a new
  `UnknownHostPayload` appears, or if a new row points at code this brief did
  not touch.

## Host discipline

- All scratch under `.temp/brief-700/`; never `/tmp`.
- The dispatcher already runs you inside the 16G memory scope; do not call
  `systemd-run` or `memory-scope.sh` yourself. One cargo command at a time;
  builds `CARGO_BUILD_JOBS=8`; tests `RUST_TEST_THREADS=8 --no-fail-fast`; a
  red is reproduced at `--test-threads=1` before it is called a defect.
- Timing, Callgrind and heap captures run serially and alone.
- After the director's commit+push, the director runs `clean-temp.py`; you
  run no version-control command.

## Stop conditions

STOP, with the exact panel and the smallest alternative, if any of the
following holds:

- a live file does not match its frozen-base prerequisite, or any target
  byte differs from the frozen target;
- any frozen form is changed to make a gate pass, or any assertion, fixture,
  key or inventory row is weakened, moved or recategorized;
- any default-path cost appears (off-mode mutex, allocation, extra branch, or
  a non-cold helper on the disabled path);
- any opcode or memory count increases, a second load appears, a call
  appears on the terminal branch, or an added instruction is unpaired;
- the 386 snapshots, the stack residuals, the coverage witnesses or the
  five falsifiers fail;
- a mutant hits and passes, or an identity fixture changes its bytes;
- a standing inventory row's identity/disposition/disappearance changes, or a
  new `UnknownHostPayload` appears;
- any `unsafe`, public-API, serde, feature, manifest, dependency or workload
  change is needed;
- a red is not explained by a stale derived cell after a replay.

No form repair, no live organization change, no version-control action.

## Deliverable

- The exact 15-file live diff, byte-identical to the frozen target, and a
  disclosed row-by-row inventory replay record.
- Build receipts, one forced SHA comparison, all raw controls and
  reductions.
- The terminal instruction/ownership ledger, the 386-snapshot position
  comparison, the stack and coverage panels, the five falsifiers, the three
  identity kills, the no-collateral rows, the 150 route dispositions and the
  lifecycle fixture results.
- Base/candidate `timings.tsv` from the core-language microbench, with target,
  hot-label and same-tier-neighbor comparisons; no regression hidden.
- A `rum-report/1` report at `.temp/brief-700/report.json`, quoting every run,
  failed attempt, verdict line and excluded row. Run
  `python3 -I scripts/report.py check 700` (no version-control option) and
  return its summary and all compliance verdicts.
- The working tree uncommitted for the director's gate.

## Report format

The report is a structured file in the `rum-report/1` schema, not prose. Every
verdict line is quoted; every measurement exists as a persisted artifact; a
run that exists only as a log is not evidence; no row is dropped and no
negative is netted. State the live diff, the exact target hashes, each
inventory replay verdict and changed row, each gate's raw numbers, every
excluded row, and any stop with raw evidence.
