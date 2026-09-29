# brief-551 — the batch boundary: a nested load's cancellation panics, and a failed restore wedges the session

Implementation brief. **Do not run `git` in any form — not `status`, not
`diff`, not `log` — at any point, including while first loading this
repository.** Everything you need about the tree's state is in this brief.

## Goal

Step 2 of the suspension-continuation branch. brief-550 (`c6e95dd1`) made guest
package loading suspend and resume. The script-shaped batch entry points —
`Session::eval_script_request`, `eval_file_request`,
`eval_snapshot_restore_request`, and `HostSession::eval_script`, `eval_file`,
`restore_from_bytes` — keep their documented non-resumability
(`crates/rum-host/src/session.rs:378-412`, `docs/guide/embedding.md:364-380`).
The director re-probed that boundary on the current tree and found **two
defects**, both reachable only with `FileRead`. Fix both, and install a check
that makes the class they belong to fail loudly.

## What the director measured on `c6e95dd1`

Probes and raw output: `.temp/director/gate-550/zz_director_order_probe2.rs`
with `.temp/director/gate-550/order-probe2.txt` (runtime), and
`.temp/director/gate-550/zz_director_restore_probe.rs` with
`.temp/director/gate-550/restore-probe.txt` (host). Files:

```text
Outer.rum:  Global`marker=5 / AppendTo[Global`log,1] / Get["Inner`"] / 99
Inner.rum:  AppendTo[Global`log,2] / InputString["p: "] / 7
```

Each case starts with `log = {}`, takes a request cancel handle, starts the
request, cancels, then re-enters with `resume_eval_request_with_input` as the
host guide prescribes, and reads `{Global`log, Global`marker, $InputFileName}`
and `1 + 1`.

| Case | After cancellation |
| --- | --- |
| **public `eval_file_request("Outer.rum")`** — suspends inside the nested guest `Get` | **panic**: `guest_loading/file.rs:122`, "guest files close inner to outer exactly once", left 1, right 2 |
| public `eval_file_request` of a flat file that asks for input itself | `Cancelled`; `{{}, 0, ""}`; `2` |
| `eval_script_request` with the same three forms | `Cancelled`; `{{1}, 5, ""}` — scripts are not transactional, the inner file rolled back; `2` |
| `eval_snapshot_restore_request` with the same three forms | same as the script |
| guest `eval_request("Get[\"Outer`\"]")` | `Cancelled`; `{{}, 0, ""}`; `2` |

**Defect 1 — the panic.** Public `Session::eval_file`
(`session/session_eval.rs:389-412`) pops `load_stack` and restores its loader
snapshot on **every** error, `InputRequired` included. When the suspension came
from a nested guest load, the entry it pops is the *inner* file's, and the
snapshot it restores predates the inner file's own. The inner file's
transaction is still live in the pending continuation; when cancellation drives
its abandonment, `close_guest_file` finds the stack one short and asserts.
Before brief-550 the guest `Get` swallowed the input and closed its own entry,
so the eager rollback never met a live inner transaction; brief-550 made the
guest load propagate correctly and exposed it. This is exactly the ordering
brief-549 warned about (§5: *never restore an outer file snapshot before still
-live inner rollback actions*). The director committed brief-550 without having
probed this combination.

**Defect 2 — the wedged restore.**
`HostSession::restore_from_bytes(b"x = 1\nInputString[\"p: \"]\ny = 2")`
returns `Err(Other "snapshot restore unexpectedly requested input")`, and then
`eval("{x, y}")` fails with "cannot start a new evaluation while the session is
waiting for host input" and `resume_with_input` fails with "no suspended
evaluation is waiting for host input". The control, a restore that ends in
`Throw[3]` instead, returns its failure and then `eval("{x, y}")` is
`Success List[1, y]`. The rustdoc (`session.rs:205-212`) promises exactly the
control's behaviour: a failed restore leaves the partial state installed and
the session usable. An input request is converted into a failure without
retiring the suspension owner that asked for it.

## Rulings and director decisions — do not reopen

- **Batch stays non-resumable, exactly as documented.** A suspended batch
  returns `InputRequired`; `resume_with_input` fails with `Other`; new requests
  are rejected until cancellation, `reset`, `restore_from_bytes` or
  replacement; cancellation then re-entry reports `Cancelled` and makes the next
  request usable. None of that changes.
- **A suspended public file's rollback is deferred to abandonment** (director
  decision recorded with brief-549's review): its file transaction — snapshot
  restore and its own `load_stack` entry — is owned by the pending
  continuation as an abandonment-only obligation, **ordered after every inner
  frame's abandonment**, and runs when cancellation abandons the batch. On
  success and on an ordinary error the existing immediate commit or rollback is
  unchanged. `reset` rebuilds the session and runs no guest cleanup, as today.
  This moves the moment a suspended public file rolls back from "at
  suspension" to "at cancellation"; read-only inspection while blocked will see
  the un-rolled-back state. Document that.
- **A failed restore retires its own suspension.** Before
  `restore_from_bytes` returns its input failure, the suspension the restore
  created is abandoned exactly once (structurally, inner to outer), its prompt
  and request checkpoints cleared, and its accepted identity retired, so the
  session is left as the rustdoc promises: partial state installed, usable, no
  second reset. Put the policy in the runtime restore path if that is where
  the owner lives, so the host wrapper only translates the failure. Do not
  touch a pre-existing unrelated request if restore admission itself failed.
- **This branch builds no leases**, and the guest-loading design of
  brief-549/550 is not reopened.

## The class, and the check that closes it

Both defects are one shape: **a route that ends a request, or reports a
terminal outcome, while a suspension owner it created is still installed** —
the same shape as the loader leak brief-550 fixed. Add a check at the runtime
request boundary: when a public request completes with a value, or fails with
any error other than `InputRequired`, no pending suspension owner may remain.
Make it a `debug_assert!` (or an equivalent test-build check) so it costs
nothing in release, and put it where every request's outcome is packaged, not
at each call site.

**Watch it fail before relying on it** (the project's falsifier rule): a
test-only fixture that deliberately leaves an owner installed at completion
must trip it — a `#[should_panic]` test with the expected message — and the
fixture must be one that exists only under `cfg(test)`. Then run the whole
workspace with the check live: **any existing test it trips is a finding to
report, not a test to adjust.**

## Red tests first

Install as permanent tests, and save their failing output under
`.temp/brief-551/red/` before any production change:

1. Runtime: public `eval_file_request("Outer.rum")` as above, cancelled at the
   inner input → `Cancelled`, `{{}, 0, ""}`, no pending owner, `1 + 1` is `2`.
   Today: panic.
2. The same with two levels of nesting (a public file that `Get`s a file that
   `Get`s the file that asks), and with the input answered once before
   cancelling at a second input deeper in.
3. The same through `HostSession::eval_file` with its cancel handle and
   re-entry. Today the host guard catches the panic and poisons the session.
4. `reset` after the nested suspension of case 1: no panic, a usable session,
   and no guest cleanup run (use a `WithCleanup` in the outer file whose cleanup
   appends to a log visible to the host through a print sink or equivalent).
5. Host: `restore_from_bytes` of `x = 1`, `InputString["p: "]`, `y = 2` →
   the input failure, then `eval("{x, y}")` is `Success List[1, y]`,
   `resume_with_input` reports no suspension, and no prompt is pending. Also
   with the input request inside a nested guest `Get` in the snapshot source.
6. The invariant's `#[should_panic]` falsifier.

Controls that already pass and must keep passing: the flat public file, the
script and snapshot rows above, the guest `Get` row, brief-549's archived
`boundaries_and_cancellation` expectations, brief-550's 47 guest-loading tests
and its host tests, and the `Throw` restore control.

## Context pointers

- `AGENTS.md`, `docs/testing-strategy.md`, `docs/code-organization-guidelines.md`.
- brief-549's report §5 (the batch design) and its draft
  `.temp/brief-549/draft-batch-ownership.md` — raw material; this brief
  overrides it where they differ (no panic premise, no new public state).
- brief-550's report and code: `session/guest_loading/` (the guest file
  transaction, `close_guest_file`), `suspension/core.rs` (abandonment
  worklist and structural rollback operations it added).
- `session/session_eval.rs:389-412` (public `eval_file`),
  `session/session_requests.rs` (batch entries, cancellation re-entry, failure
  packaging), `session/session_api.rs:74` (snapshot restore request),
  `crates/rum-host/src/session.rs:165-240` (`reset`, `restore_from_bytes`).
- The director's probes named above; the guest-loading harness
  `crates/rum-stdlib/tests/guest_loading_resume.rs` and
  `crates/rum-host/tests/guest_loading_resume.rs` as models.

## Scope

- **Objective:** the public file's deferred, correctly ordered rollback; the
  restore path's retirement of its own suspension; the request-boundary
  invariant and its falsifier; the permanent tests; the rustdoc and guide text
  that describes the rollback timing and restore-failure state.
- **Fence:** no batch resumability; no change to the documented blocking,
  cancellation or reset behaviour; no overlap supersession; no leases; no
  guest-loading redesign; no front-end changes (`rum-cli`, `rum-repl`,
  `rum-jupyter`); no G1-G8 work; no `Expr`, `Symbol` or serde change; no
  dependency or lockfile change; no default-path cost. Do not edit `AGENTS.md`,
  `PROGRESS.md`, `TODO.md`, earlier briefs or unrelated docs.
- **Disclosure duty:** list every changed file with line ranges and why; every
  frame, field and root edge added; every observable behaviour that changed,
  with before and after; every existing test the new invariant tripped. If the
  objective needs a change outside the fence, STOP and report it.

## rum / Rust pitfalls (read twice)

1. **Inner before outer.** Every abandonment obligation runs inner to outer;
   the public file's restore runs after every nested guest file's. A restore
   that runs first and is then partially undone by an inner one resurrects
   state.
2. **`load_stack` is popped exactly once per file, by its owner, checked
   against depth and path.** The public file must not pop an entry it did not
   push.
3. **A cleared prompt is not a retired owner.** Retiring means the pending
   continuation is taken and its abandonment driven; check the owner, the
   pending and resuming queues and the request checkpoint, not the prompt.
4. **`reset` is not abandonment.** It rebuilds the session and runs no guest
   cleanup; a test must show that stays true.
5. **Probe that the suspension happened** before asserting what follows it.
6. **Stop-loss is keyed on progress**: counters are red tests turned green,
   invariant trips resolved, and instruction regression rows eliminated.
   Reducing compiler errors is progress; the stop-loss fires only when two
   consecutive compiling attempts move none of the counters.

## Verification (MANDATORY — you run it; iterate to green)

Wrap every command in `python3 scripts/run-bounded.py --timeout <seconds> --`,
bounds derived from a measured run of the same shape, and run cargo as
`env CARGO_BUILD_JOBS=4 cargo …`. Paste commands, bounds, exit statuses and
test counts verbatim.

```
cargo fmt --check
cargo test -p rum-runtime
cargo test -p rum-stdlib
cargo test -p rum-host
cargo test --workspace
```

Native ZeroMQ tests that cannot bind loopback in the sandbox need
`RUM_ALLOW_NATIVE_ZEROMQ_SKIP`; if you set it, say so and list them.

**Performance gate.** The change touches request completion and batch entry,
not the per-step evaluator; show it. Preserve base binaries under
`.temp/brief-551/base/` **before editing**, built from the unmodified tree into
their own target directory (brief-550's first preservation silently returned a
stale cached artifact — check that the base and candidate differ, and that the
base lacks the new code). Allocation counts first; then per-iteration
`cpu_core/instructions:u` with `taskset -c 2`, `rum-cli` fed `< /dev/null`,
`(count(2N) - count(N)) / N`, one label per run, at most two samples per
(label, N, build), on `literal-integer`, `symbol-ownvalue-read`,
`set-ownvalue`, `do-count-null`, `block-local` and `table-iterator-null` (use N
at least 20 000 where the iteration limit allows; small N is noise). Timing is
secondary and optional here; if you run it, check first that no CPU-heavy
process competes.

## Checkpointing — protect the scene

Scratch under `.temp/brief-551/`. After each milestone — base binaries, reds,
each defect fixed, the invariant and its falsifier, the workspace run, the
performance panel — write `.temp/brief-551/PROGRESS.md`: what is done and
verified with its evidence, the exact next action, the tree's state, and the
counters.

## Hard rules

- Do NOT `git commit`, branch, push or touch git config. Do not run `git`.
  Leave all changes in the working tree.
- Do NOT modify files outside the objective and fence; STOP and report instead.
- Touch nothing outside this repository — including reading user configuration
  such as `~/.cargo`. No network. Commands: file reads and edits within scope,
  `rg`/`grep`, `python3`, bounded `cargo`, the release `rum-cli`, bounded
  `taskset`/`perf`.
- **Testing Policy is a hard line** (`AGENTS.md`): rum against rum only.
- **No new `unsafe`.**
- Do not weaken an assertion, a host contract or a rollback to turn a test
  green; report the conflict.
- You MUST paste real verbatim output, and say plainly what did not run.

## Report format (your final message, appended to this file)

1. What changed — per file with line ranges and why; frames, fields, roots;
   observable changes before and after.
2. Verification — every command's verbatim result; red before and green after
   for each permanent test; the invariant's falsifier watched failing; every
   existing test the invariant tripped and what it meant.
3. Allocation and per-iteration instruction rows, per label.
4. Progress counters, disclosures, unresolved risks and questions.

## Implementation report

### 1. What changed

Both reported defects are fixed. A suspended public file now owns a deferred
rollback behind every inner abandonment obligation. A failed host restore
retires its own input suspension before returning, leaving the documented
partial state usable. Admitted terminal request outcomes share an owner check.

| File | Final line ranges | Why |
| --- | --- | --- |
| [crates/rum-host/src/session.rs](../../../crates/rum-host/src/session.rs) | 13–15, 175, 214–218, 227, 235, 392–393, 415–419, 778 | Translate the runtime's terminal restore failure; remove the leaking host conversion; clarify partial state, deferred rollback and reset cleanup rustdoc. |
| [crates/rum-host/tests/batch_boundary_ownership.rs](../../../crates/rum-host/tests/batch_boundary_ownership.rs) | 1–171 | Permanent host cancellation, no-cleanup reset, flat/nested restore retirement and Throw control tests. |
| [crates/rum-runtime/src/runtime/session/batch_file.rs](../../../crates/rum-runtime/src/runtime/session/batch_file.rs) | 1–68 | New abandonment-only PublicFileRollbackFrame and exact depth/path validation for public file closure. |
| [crates/rum-runtime/src/runtime/session/mod.rs](../../../crates/rum-runtime/src/runtime/session/mod.rs) | 1 | Register the batch file ownership module. |
| [crates/rum-runtime/src/runtime/session/session_api.rs](../../../crates/rum-runtime/src/runtime/session/session_api.rs) | 83–109 | Add terminal restore_snapshot_request policy while preserving the existing cancellable snapshot batch API. |
| [crates/rum-runtime/src/runtime/session/session_eval.rs](../../../crates/rum-runtime/src/runtime/session/session_eval.rs) | 389–393, 397, 403–405, 408–411 | Defer InputRequired rollback to the pending owner; preserve immediate success/error closure; document inspection timing. |
| [crates/rum-runtime/src/runtime/session/session_requests.rs](../../../crates/rum-runtime/src/runtime/session/session_requests.rs) | 67–68, 162, 190, 302–310, 566–567, 570, 574–575, 591, 595–623, 629, 632–633 | Central terminal-outcome owner check; route completion, resumed results and cancellation packaging through it. |
| [crates/rum-runtime/src/runtime/suspension/core.rs](../../../crates/rum-runtime/src/runtime/suspension/core.rs) | 38, 290–299 | Add FailedSnapshotRestore reason and the restricted public-file structural rollback operation; retain the existing abandonment driver. |
| [crates/rum-runtime/src/runtime/tests/frame_root_trace.rs](../../../crates/rum-runtime/src/runtime/tests/frame_root_trace.rs) | 40, 483 | Classify the one new stateful frame and update the exact closed frame count from 544 to 545. |
| [crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs](../../../crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs) | 568 | Add the one new derived frame root type; no tracing exemption. |
| [crates/rum-runtime/src/runtime/tests/session_requests/batch_boundary.rs](../../../crates/rum-runtime/src/runtime/tests/session_requests/batch_boundary.rs) | 1–397 | Permanent cancellation/queued-input/control/root/cleanup regressions and five deliberate test-only owner leak falsifiers. |
| [crates/rum-runtime/src/runtime/tests/session_requests/batch_boundary/restore.rs](../../../crates/rum-runtime/src/runtime/tests/session_requests/batch_boundary/restore.rs) | 1–107 | Private terminal restore identity, queue, cleanup ordering and rejected-admission checks. |
| [crates/rum-runtime/src/runtime/tests/session_requests/mod.rs](../../../crates/rum-runtime/src/runtime/tests/session_requests/mod.rs) | 4 | Register the permanent batch ownership test family under the existing cfg(test) tree. |
| [crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payload_ledger.tsv](../../../crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payload_ledger.tsv) | 38–44, 48–56 | Rebase the same host payload coordinates, preserving every attribution/disposition column. |
| [crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payloads.tsv](../../../crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payloads.tsv) | 37–43, 47–55 | Rebase source coordinates for the same 63 host payload identities. |
| [crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv](../../../crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv) | 795, 809 | Keep discovery exact for the new frame derive and conditional test/debug assertion; both remain unresolved inventory entries. |
| [crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv](../../../crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv) | 1634–1650, 1666, 1679–1691, 1761–1762 | Rebase moved source/closure coordinates without shrinking the 2,932-row capture inventory. |
| [crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv](../../../crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv) | 1628–1631, 1650–1662, 1680–1690, 1751–1759 | Rebase moved macros and record the seven added boundary macros; no suppression. |
| [crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_storage.tsv](../../../crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_storage.tsv) | 59 | Record the new rollback frame PathBuf as unresolved foreign representation. |
| [crates/rum-stdlib/src/tests/retention_closure/census/tests.rs](../../../crates/rum-stdlib/src/tests/retention_closure/census/tests.rs) | 189 | Rebase the same three abandonment sites by 11 lines; preserve all 927 rejection sites and kind counts. |
| [crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv](../../../crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv) | 2–69 | Refresh source fingerprints only; all first 18 columns, 62 Discharged and 6 Pending records remain unchanged. |
| [crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv](../../../crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv) | 2–3 | Install independently replayed default/no-default certificate cells for the same two bodies; preserve all claims, exclusions and obligations. |
| [docs/dev/codex/brief-551-batch-boundary-ownership.md](../../../docs/dev/codex/brief-551-batch-boundary-ownership.md) | 262–2087 | Append this implementation report and verbatim verification/performance evidence. |
| [docs/guide/embedding.md](../../../docs/guide/embedding.md) | 388–395, 407–408, 444–449 | Document deferred public-file rollback, read-only partial state, reset without cleanup, and usable partial restore failure. |

Exactly one frame was added: `PublicFileRollbackFrame`, with `path: PathBuf`,
`depth: usize` and `snapshot: LoaderSnapshot`. Path and depth carry no
Expr/Symbol roots. The added root edge is the existing pending frame queue's
Arc → this frame → the existing derived LoaderSnapshot graph, including all
captured Expr/Symbol-bearing definition, attribute, option, package, message and
trace state. The same existing visitor follows it into the abandonment
worklist. Its resume method is unreachable: it is an abandonment-only
obligation. No Session field, owner slot, lease, Expr/Symbol representation or
serde format was added. `FailedSnapshotRestore` is one new internal reason.
The structural rollback method adds no evaluation capability. Guest-loading
frames and the abandonment worklist algorithm are unchanged.

The new `restore_snapshot_request` applies terminal restore policy in the
runtime; the host only translates the failure. The existing
`eval_snapshot_restore_request` stays a blocked, non-resumable batch with its
original cancellation control. Only an admitted InputRequired result is
retired; an admission rejection leaves the unrelated generation, prompt,
checkpoint and accepted identity untouched.

The central check is `assert!` under `cfg(any(debug_assertions, test))`, the
permitted test-build equivalent of `debug_assert!`. It checks pending/resuming
owners, abandonment work, prompt and request checkpoint for terminal outcomes.
It covers evaluation/expression, script/file/snapshot, resume and cancellation
packaging; rejected admission bypasses it. Optimized unit-test falsifiers prove
the check stays active in tests. Production release omits it.

| Observable behavior | Before | After |
| --- | --- | --- |
| Cancel a public file suspended inside nested Get | Inner close panics; host catches it and poisons the session. | Inner transactions close first, public snapshot rolls back last, cancellation is reported, state is `List[List[], 0, ""]`, and fresh `1+1` gives `2`. |
| Inspect a suspended public file | Public snapshot eagerly restored; an inner stack entry incorrectly popped. | Partial state remains visible while blocked; every file entry remains owned until abandonment. |
| Flat/nested host restore reaches input | Other input failure leaves an unreachable live owner; next eval rejects. | Same input-failure code/message; suspension/prompt/checkpoint and accepted identity retired; `x=1` remains, `y` stays undefined, fresh eval succeeds without a second reset. |
| Terminal result with a leaked owner | Could silently wedge the session. | Debug/test boundary assertion fails loudly. |

Controls preserve non-resumability, rejected overlaps, queued batch input,
ordinary file commit/rollback, script/snapshot and guest cancellation, guest
resumption, and reset without guest cleanup. The first answer in the two-input
batch regression is pre-queued: ordinary batch resume remains rejected.

Organization assessment: the new ownership module is under 10k chars, the
main new runtime test file under 15k, and restore cases form a child module.
Existing split-plan files remain scoped: request admission/completion versus
editor/diagnostic requests is the future boundary for session_requests;
snapshot APIs versus configuration for session_api; abandonment context versus
driver for suspension/core; lifecycle/snapshot versus evaluation/editor facade
for host/session. session_eval remains in the warning range, with batch/file
entry versus single-form evaluation as its future boundary. Their small
boundary edits do not justify an unrelated split here. The larger exact root
and source inventories remain declarative catalogs.

The source-evidence refresh is mechanical maintenance, with no G1–G8 work or
new retention coverage. All 63 host payload identities and ledger dispositions
remain; the 2,932 capture rows are not reduced. Added opaque derive/path/macro
records remain unresolved. The same 927 census rejections/kind counts remain,
with three source lines rebased by 11. Contract columns 1–18 are unchanged:
62 Discharged and 6 Pending. The two existing production rows retain all
bodies, claims, obligations and exclusions; only independently replayed
default/no-default certificate cells changed.

### 2. Verification

The permanent required reds were installed and saved before production edits:
runtime `3 passed; 9 failed` and host `2 passed; 3 failed`. The earlier runtime
red compile had two fixture typing mistakes, corrected without changing its
expectations. Reset and Throw controls already passed.

| Permanent test | Red/control evidence | Green evidence |
| --- | --- | --- |
| `public_file_cancel_nested` | FAILED in saved required red run | final workspace: passed |
| `public_file_cancel_two_levels` | FAILED in saved required red run | final workspace: passed |
| `public_file_first_input_answered_then_cancel_deeper_second` | FAILED in saved required red run | final workspace: passed |
| `public_file_rollback_is_deferred_while_blocked` | FAILED in saved required red run | final workspace: passed |
| `flat_public_file_cancellation_control` | already passing control | final workspace: passed |
| `script_snapshot_and_guest_cancellation_controls` | already passing control | final workspace: passed |
| `public_file_success_and_ordinary_error_control` | already passing control | final workspace: passed |
| `public_file_snapshot_roots_survive_suspension` | additional structural check after required reds | final workspace: passed |
| `public_file_cleanup_observes_inner_then_outer_state_once` | additional structural check after required reds | final workspace: passed |
| `invariant_falsifier_value` | did not panic as expected | final workspace and optimized unit falsifier run: passed |
| `invariant_falsifier_error` | did not panic as expected | final workspace and optimized unit falsifier run: passed |
| `invariant_falsifier_script` | did not panic as expected | final workspace and optimized unit falsifier run: passed |
| `invariant_falsifier_file` | did not panic as expected | final workspace and optimized unit falsifier run: passed |
| `invariant_falsifier_snapshot` | did not panic as expected | final workspace and optimized unit falsifier run: passed |
| `terminal_restore_retires_identity_and_all_owner_state` | additional structural check after required reds | final workspace: passed |
| `restore_admission_rejection_preserves_unrelated_request` | additional structural check after required reds | final workspace: passed |
| `restore_abandons_nested_cleanup_and_file_rollbacks_once` | additional structural check after required reds | final workspace: passed |
| `nested_public_file_cancellation_keeps_host_usable` | FAILED in saved required red run | final workspace: passed |
| `reset_nested_public_file_runs_no_guest_cleanup` | already passing control | final workspace: passed |
| `input_restore_failure_retires_owner_preserving_partial_state` | FAILED in saved required red run | final workspace: passed |
| `nested_input_restore_failure_retires_owner_preserving_partial_state` | FAILED in saved required red run | final workspace: passed |
| `throw_restore_failure_control_keeps_partial_state_usable` | already passing control | final workspace: passed |

Verbatim green lines from the final workspace run:

```text
test runtime::tests::session_requests::batch_boundary::public_file_cancel_nested ... ok
test runtime::tests::session_requests::batch_boundary::public_file_cancel_two_levels ... ok
test runtime::tests::session_requests::batch_boundary::public_file_first_input_answered_then_cancel_deeper_second ... ok
test runtime::tests::session_requests::batch_boundary::public_file_rollback_is_deferred_while_blocked ... ok
test runtime::tests::session_requests::batch_boundary::flat_public_file_cancellation_control ... ok
test runtime::tests::session_requests::batch_boundary::script_snapshot_and_guest_cancellation_controls ... ok
test runtime::tests::session_requests::batch_boundary::public_file_success_and_ordinary_error_control ... ok
test runtime::tests::session_requests::batch_boundary::public_file_snapshot_roots_survive_suspension ... ok
test runtime::tests::session_requests::batch_boundary::public_file_cleanup_observes_inner_then_outer_state_once ... ok
test runtime::tests::session_requests::batch_boundary::invariant_falsifier_value - should panic ... ok
test runtime::tests::session_requests::batch_boundary::invariant_falsifier_error - should panic ... ok
test runtime::tests::session_requests::batch_boundary::invariant_falsifier_script - should panic ... ok
test runtime::tests::session_requests::batch_boundary::invariant_falsifier_file - should panic ... ok
test runtime::tests::session_requests::batch_boundary::invariant_falsifier_snapshot - should panic ... ok
test runtime::tests::session_requests::batch_boundary::restore::terminal_restore_retires_identity_and_all_owner_state ... ok
test runtime::tests::session_requests::batch_boundary::restore::restore_admission_rejection_preserves_unrelated_request ... ok
test runtime::tests::session_requests::batch_boundary::restore::restore_abandons_nested_cleanup_and_file_rollbacks_once ... ok
test nested_public_file_cancellation_keeps_host_usable ... ok
test reset_nested_public_file_runs_no_guest_cleanup ... ok
test input_restore_failure_retires_owner_preserving_partial_state ... ok
test nested_input_restore_failure_retires_owner_preserving_partial_state ... ok
test throw_restore_failure_control_keeps_partial_state_usable ... ok
```

All five deliberate leaky builtin fixtures exist only under the runtime's
`cfg(test)` module tree. Their `#[should_panic]` tests initially failed because
no panic occurred. With the check, all five visibly panic with
`terminal request outcome retained a suspension owner` in both debug and
optimized unit tests.

**Existing tests tripped by the new invariant: none.** The first stdlib gate's
20 failures were stale source-coordinate inventories and contract/certificate
fingerprints. They were resolved without weakening assertions, removing
obligations, or changing contract status. Expected compile-negative diagnostics
in successful suites are retained in raw output.

Final workspace revalidates all crates after the guard's profile adjustment.

| Gate | Passed | Failed | Ignored |
| --- | --- | --- | --- |
| gate-runtime-01 | 1116 | 0 | 3 |
| gate-stdlib-02 | 1072 | 0 | 53 |
| gate-host | 166 | 0 | 2 |
| gate-workspace-final | 2756 | 0 | 58 |
| gate-no-default-retention | 49 | 0 | 1 |
| release-falsifiers | 5 | 0 | 0 |
| archive-boundaries | 1 | 0 | 0 |

The recorded commands below include bounds, actual exit statuses and verbatim
test-result lines. Full unedited stdout/stderr is linked. The runner sets
`CARGO_NET_OFFLINE=true` and in-repository `TMPDIR`; Cargo uses
`CARGO_BUILD_JOBS=4`. Initial caps derive from measured brief-550 runs (fresh
release 34.523s, runtime 19.964s, host 61.623s, workspace 132.353s, no-default
retention 245.628s); subsequent caps use this brief's same-shape measurements.
Monotonic elapsed durations size bounds only, not performance conclusions.
The candidate alloc build's 10s cap derives from the measured 1.942s base
feature build. Each performance child has a 6s cap; the panel's 80s bound
covers twelve such children plus orchestration. Measured children finished
in 0.059–0.231s.

[.temp/brief-551/raw/base-default-build.txt](../../../.temp/brief-551/raw/base-default-build.txt)

```text
python3 scripts/run-bounded.py --timeout 140 -- env CARGO_BUILD_JOBS=4 cargo build --release -p rum-cli --target-dir .temp/brief-551/base-target
bound_seconds=140.0 exit=0 elapsed_seconds=35.98613159000524
    Finished `release` profile [optimized] target(s) in 35.85s
```

[.temp/brief-551/raw/base-alloc-build.txt](../../../.temp/brief-551/raw/base-alloc-build.txt)

```text
python3 scripts/run-bounded.py --timeout 145 -- env CARGO_BUILD_JOBS=4 cargo build --release -p rum-cli --features alloc-count --target-dir .temp/brief-551/base-target
bound_seconds=145.0 exit=0 elapsed_seconds=1.9416552869952284
    Finished `release` profile [optimized] target(s) in 1.76s
```

[.temp/brief-551/red/runtime-01.txt](../../../.temp/brief-551/red/runtime-01.txt)

```text
python3 scripts/run-bounded.py --timeout 80 -- env CARGO_BUILD_JOBS=4 cargo test -p rum-runtime batch_boundary -- --nocapture
bound_seconds=80.0 exit=101 elapsed_seconds=2.8886139660025947
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
error[E0599]: no method named `full_name` found for reference `&&rum_core::Symbol` in the current scope
   --> crates/rum-runtime/src/runtime/tests/session_requests/batch_boundary.rs:114:65
    |
114 |     let marker = s.definitions.iter().find(|(symbol, _)| symbol.full_name() == "Global`marker")
    |                                                                 ^^^^^^^^^ method not found in `&&Symbol`

error[E0308]: mismatched types
   --> crates/rum-runtime/src/runtime/tests/session_requests/batch_boundary.rs:168:43
    |
168 |     if args.is_empty() { Ok(Expr::integer(42)) } else { Err(EvalError::Exit { code: 3 }) }
    |                             ------------- ^^ expected `BigInt`, found integer
    |                             |
    |                             arguments to this function are incorrect
    |
note: associated function defined here
   --> /home/remi/dev/rum/crates/rum-core/src/expr.rs:781:12
    |
781 |     pub fn integer(value: BigInt) -> Self {
    |            ^^^^^^^

Some errors have detailed explanations: E0308, E0599.
For more information about an error, try `rustc --explain E0308`.
error: could not compile `rum-runtime` (lib test) due to 2 previous errors
```

[.temp/brief-551/red/host-01.txt](../../../.temp/brief-551/red/host-01.txt)

```text
python3 scripts/run-bounded.py --timeout 160 -- env CARGO_BUILD_JOBS=4 cargo test -p rum-host --test batch_boundary_ownership -- --nocapture
bound_seconds=160.0 exit=101 elapsed_seconds=5.498043872008566
   Compiling rum-host v0.2.0 (/home/remi/dev/rum/crates/rum-host)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 5.34s
     Running tests/batch_boundary_ownership.rs (target/debug/deps/batch_boundary_ownership-ada39e42563d5836)

running 5 tests

thread 'nested_public_file_cancellation_keeps_host_usable' panicked at crates/rum-runtime/src/runtime/session/guest_loading/file.rs:122:9:
assertion `left == right` failed: guest files close inner to outer exactly once
  left: 1
 right: 2
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

thread 'nested_public_file_cancellation_keeps_host_usable' panicked at crates/rum-host/tests/batch_boundary_ownership.rs:51:18:
expected usable cancellation, got Failure(HostFailure { code: Panic, exit_status: None, message: "host session caught panic: assertion `left == right` failed: guest files close inner to outer exactly once\n  left: 1\n right: 2", syntax_rendered: None, messages: [], prints: [], trace: [], outputs: [] })
test nested_public_file_cancellation_keeps_host_usable ... FAILED
test throw_restore_failure_control_keeps_partial_state_usable ... ok

thread 'input_restore_failure_retires_owner_preserving_partial_state' panicked at crates/rum-host/tests/batch_boundary_ownership.rs:39:18:
expected List[1, y], got Failure(HostFailure { code: Other, exit_status: None, message: "cannot start a new evaluation while the session is waiting for host input; call resume_with_input or reset first", syntax_rendered: None, messages: [], prints: [], trace: [], outputs: [] })
test input_restore_failure_retires_owner_preserving_partial_state ... FAILED

thread 'nested_input_restore_failure_retires_owner_preserving_partial_state' panicked at crates/rum-host/tests/batch_boundary_ownership.rs:39:18:
expected List[1, y], got Failure(HostFailure { code: Other, exit_status: None, message: "cannot start a new evaluation while the session is waiting for host input; call resume_with_input or reset first", syntax_rendered: None, messages: [], prints: [], trace: [], outputs: [] })
test nested_input_restore_failure_retires_owner_preserving_partial_state ... FAILED
test reset_nested_public_file_runs_no_guest_cleanup ... ok

failures:

failures:
    input_restore_failure_retires_owner_preserving_partial_state
    nested_input_restore_failure_retires_owner_preserving_partial_state
    nested_public_file_cancellation_keeps_host_usable

test result: FAILED. 2 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

error: test failed, to rerun pass `-p rum-host --test batch_boundary_ownership`
```

[.temp/brief-551/red/runtime-02.txt](../../../.temp/brief-551/red/runtime-02.txt)

```text
python3 scripts/run-bounded.py --timeout 80 -- env CARGO_BUILD_JOBS=4 cargo test -p rum-runtime batch_boundary -- --nocapture
bound_seconds=80.0 exit=101 elapsed_seconds=11.266585570992902
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
warning: function `register_test_if_builtin` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:320:27
    |
320 | pub(in crate::runtime) fn register_test_if_builtin(session: &mut Session) {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` on by default

warning: function `register_test_return_builtin` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:330:27
    |
330 | pub(in crate::runtime) fn register_test_return_builtin(session: &mut Session) {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `register_test_vm_template_builtins` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:358:27
    |
358 | pub(in crate::runtime) fn register_test_vm_template_builtins(session: &mut Session) {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: `rum-runtime` (lib test) generated 3 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 11.14s
     Running unittests src/lib.rs (target/debug/deps/rum_runtime-95cd5fbb46da4f1c)

running 12 tests
test runtime::tests::session_requests::batch_boundary::invariant_falsifier_snapshot - should panic ... FAILED
test runtime::tests::session_requests::batch_boundary::invariant_falsifier_error - should panic ... FAILED
test runtime::tests::session_requests::batch_boundary::invariant_falsifier_file - should panic ... FAILED
test runtime::tests::session_requests::batch_boundary::invariant_falsifier_script - should panic ... FAILED

thread 'runtime::tests::session_requests::batch_boundary::public_file_rollback_is_deferred_while_blocked' panicked at crates/rum-runtime/src/runtime/tests/session_requests/batch_boundary.rs:113:5:
assertion `left == right` failed
  left: ["Outer.rum"]
 right: ["Outer.rum", "Inner.rum"]
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test runtime::tests::session_requests::batch_boundary::invariant_falsifier_value - should panic ... FAILED
test runtime::tests::session_requests::batch_boundary::flat_public_file_cancellation_control ... 
thread 'runtime::tests::session_requests::batch_boundary::public_file_cancel_two_levels' panicked at crates/rum-runtime/src/runtime/session/guest_loading/file.rs:122:9:
assertion `left == right` failed: guest files close inner to outer exactly once
  left: 2
 right: 3
ok

thread 'runtime::tests::session_requests::batch_boundary::public_file_cancel_nested' panicked at crates/rum-runtime/src/runtime/session/guest_loading/file.rs:122:9:
assertion `left == right` failed: guest files close inner to outer exactly once
  left: 1
 right: 2

thread 'runtime::tests::session_requests::batch_boundary::public_file_first_input_answered_then_cancel_deeper_second' panicked at crates/rum-runtime/src/runtime/session/guest_loading/file.rs:122:9:
assertion `left == right` failed: guest files close inner to outer exactly once
  left: 2
 right: 3
test runtime::tests::session_requests::batch_boundary::public_file_rollback_is_deferred_while_blocked ... FAILED
test runtime::tests::session_requests::batch_boundary::public_file_cancel_nested ... FAILED
test runtime::tests::session_requests::batch_boundary::public_file_cancel_two_levels ... FAILED
test runtime::tests::session_requests::batch_boundary::public_file_first_input_answered_then_cancel_deeper_second ... FAILED
test runtime::tests::session_requests::batch_boundary::public_file_success_and_ordinary_error_control ... ok
test runtime::tests::session_requests::batch_boundary::script_snapshot_and_guest_cancellation_controls ... ok

failures:

---- runtime::tests::session_requests::batch_boundary::invariant_falsifier_snapshot stdout ----
note: test did not panic as expected
---- runtime::tests::session_requests::batch_boundary::invariant_falsifier_error stdout ----
note: test did not panic as expected
---- runtime::tests::session_requests::batch_boundary::invariant_falsifier_file stdout ----
note: test did not panic as expected
---- runtime::tests::session_requests::batch_boundary::invariant_falsifier_script stdout ----
note: test did not panic as expected
---- runtime::tests::session_requests::batch_boundary::invariant_falsifier_value stdout ----
note: test did not panic as expected

failures:
    runtime::tests::session_requests::batch_boundary::invariant_falsifier_error
    runtime::tests::session_requests::batch_boundary::invariant_falsifier_file
    runtime::tests::session_requests::batch_boundary::invariant_falsifier_script
    runtime::tests::session_requests::batch_boundary::invariant_falsifier_snapshot
    runtime::tests::session_requests::batch_boundary::invariant_falsifier_value
    runtime::tests::session_requests::batch_boundary::public_file_cancel_nested
    runtime::tests::session_requests::batch_boundary::public_file_cancel_two_levels
    runtime::tests::session_requests::batch_boundary::public_file_first_input_answered_then_cancel_deeper_second
    runtime::tests::session_requests::batch_boundary::public_file_rollback_is_deferred_while_blocked

test result: FAILED. 3 passed; 9 failed; 0 ignored; 0 measured; 1080 filtered out; finished in 0.02s

error: test failed, to rerun pass `-p rum-runtime --lib`
```

[.temp/brief-551/raw/file-fix-runtime.txt](../../../.temp/brief-551/raw/file-fix-runtime.txt)

```text
python3 scripts/run-bounded.py --timeout 80 -- env CARGO_BUILD_JOBS=4 cargo test -p rum-runtime batch_boundary::public_file -- --nocapture
bound_seconds=80.0 exit=101 elapsed_seconds=2.4925885049888166
             (A, B, C, D, E)
             (A, B, C, D, E, F)
           and 145 others
note: required by a bound in `runtime::suspension::core::SuspensionFrame`
  --> crates/rum-runtime/src/runtime/suspension/core.rs:80:28
   |
80 | pub trait SuspensionFrame: FrameRootTrace + FrameAbandon + Send + Sync + 'static {
   |                            ^^^^^^^^^^^^^^ required by this bound in `SuspensionFrame`

Some errors have detailed explanations: E0277, E0405, E0433.
For more information about an error, try `rustc --explain E0277`.
error: could not compile `rum-runtime` (lib test) due to 4 previous errors
```

[.temp/brief-551/raw/file-fix-runtime-02.txt](../../../.temp/brief-551/raw/file-fix-runtime-02.txt)

```text
python3 scripts/run-bounded.py --timeout 80 -- env CARGO_BUILD_JOBS=4 cargo test -p rum-runtime batch_boundary::public_file -- --nocapture
bound_seconds=80.0 exit=0 elapsed_seconds=11.125224707007874
    Finished `test` profile [unoptimized + debuginfo] target(s) in 10.94s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 1087 filtered out; finished in 0.01s
```

[.temp/brief-551/raw/file-fix-host.txt](../../../.temp/brief-551/raw/file-fix-host.txt)

```text
python3 scripts/run-bounded.py --timeout 80 -- env CARGO_BUILD_JOBS=4 cargo test -p rum-host --test batch_boundary_ownership nested_public_file -- --nocapture
bound_seconds=80.0 exit=101 elapsed_seconds=10.57504278999113
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
   Compiling rum-host v0.2.0 (/home/remi/dev/rum/crates/rum-host)
error[E0599]: no method named `restore_snapshot_request` found for struct `Session` in the current scope
   --> crates/rum-host/src/session.rs:221:45
    |
221 |             |session| match session.session.restore_snapshot_request(source) {
    |                                             ^^^^^^^^^^^^^^^^^^^^^^^^
    |
help: there is a method `eval_request` with a similar name
    |
221 |             |session| match session.session.eval_request(source) {
    |                                             ~~~~~~~~~~~~

For more information about this error, try `rustc --explain E0599`.
error: could not compile `rum-host` (lib) due to 1 previous error
warning: build failed, waiting for other jobs to finish...
```

[.temp/brief-551/raw/restore-fix-host.txt](../../../.temp/brief-551/raw/restore-fix-host.txt)

```text
python3 scripts/run-bounded.py --timeout 80 -- env CARGO_BUILD_JOBS=4 cargo test -p rum-host --test batch_boundary_ownership -- --nocapture
bound_seconds=80.0 exit=0 elapsed_seconds=12.076491341998917
    Finished `test` profile [unoptimized + debuginfo] target(s) in 11.92s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```

[.temp/brief-551/raw/invariant-and-runtime.txt](../../../.temp/brief-551/raw/invariant-and-runtime.txt)

```text
python3 scripts/run-bounded.py --timeout 80 -- env CARGO_BUILD_JOBS=4 cargo test -p rum-runtime batch_boundary -- --nocapture
bound_seconds=80.0 exit=0 elapsed_seconds=12.681997061998118
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
warning: function `register_test_if_builtin` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:320:27
    |
320 | pub(in crate::runtime) fn register_test_if_builtin(session: &mut Session) {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` on by default

warning: function `register_test_return_builtin` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:330:27
    |
330 | pub(in crate::runtime) fn register_test_return_builtin(session: &mut Session) {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `register_test_vm_template_builtins` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:358:27
    |
358 | pub(in crate::runtime) fn register_test_vm_template_builtins(session: &mut Session) {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: `rum-runtime` (lib test) generated 3 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 12.53s
     Running unittests src/lib.rs (target/debug/deps/rum_runtime-95cd5fbb46da4f1c)

running 12 tests

thread 'runtime::tests::session_requests::batch_boundary::invariant_falsifier_script' panicked at crates/rum-runtime/src/runtime/session/session_requests.rs:607:9:
terminal request outcome retained a suspension owner
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

thread 'runtime::tests::session_requests::batch_boundary::invariant_falsifier_snapshot' panicked at crates/rum-runtime/src/runtime/session/session_requests.rs:607:9:
terminal request outcome retained a suspension owner

thread 'runtime::tests::session_requests::batch_boundary::invariant_falsifier_file' panicked at crates/rum-runtime/src/runtime/session/session_requests.rs:607:9:
terminal request outcome retained a suspension owner
test runtime::tests::session_requests::batch_boundary::invariant_falsifier_script - should panic ... ok
test runtime::tests::session_requests::batch_boundary::invariant_falsifier_snapshot - should panic ... ok
test runtime::tests::session_requests::batch_boundary::invariant_falsifier_file - should panic ... ok
test runtime::tests::session_requests::batch_boundary::public_file_rollback_is_deferred_while_blocked ... ok

thread 'runtime::tests::session_requests::batch_boundary::invariant_falsifier_value' panicked at crates/rum-runtime/src/runtime/session/session_requests.rs:607:9:
terminal request outcome retained a suspension owner
test runtime::tests::session_requests::batch_boundary::flat_public_file_cancellation_control ... ok
test runtime::tests::session_requests::batch_boundary::invariant_falsifier_value - should panic ... ok

thread 'runtime::tests::session_requests::batch_boundary::invariant_falsifier_error' panicked at crates/rum-runtime/src/runtime/session/session_requests.rs:607:9:
terminal request outcome retained a suspension owner
test runtime::tests::session_requests::batch_boundary::public_file_cancel_nested ... ok
test runtime::tests::session_requests::batch_boundary::invariant_falsifier_error - should panic ... ok
test runtime::tests::session_requests::batch_boundary::public_file_cancel_two_levels ... ok
test runtime::tests::session_requests::batch_boundary::public_file_first_input_answered_then_cancel_deeper_second ... ok
test runtime::tests::session_requests::batch_boundary::public_file_success_and_ordinary_error_control ... ok
test runtime::tests::session_requests::batch_boundary::script_snapshot_and_guest_cancellation_controls ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 1080 filtered out; finished in 0.01s
```

[.temp/brief-551/raw/format.txt](../../../.temp/brief-551/raw/format.txt)

```text
python3 scripts/run-bounded.py --timeout 10 -- env CARGO_BUILD_JOBS=4 cargo fmt
bound_seconds=10.0 exit=0 elapsed_seconds=2.5880459140025778
(no stdout/stderr)
```

[.temp/brief-551/raw/gate-runtime-01.txt](../../../.temp/brief-551/raw/gate-runtime-01.txt)

```text
python3 scripts/run-bounded.py --timeout 80 -- env CARGO_BUILD_JOBS=4 cargo test -p rum-runtime -- --nocapture
bound_seconds=80.0 exit=0 elapsed_seconds=20.751242757993168
    Finished `test` profile [unoptimized + debuginfo] target(s) in 13.14s
test result: ok. 1094 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 4.50s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.93s
```

[.temp/brief-551/raw/gate-stdlib-01.txt](../../../.temp/brief-551/raw/gate-stdlib-01.txt)

```text
python3 scripts/run-bounded.py --timeout 1020 -- env CARGO_BUILD_JOBS=4 cargo test -p rum-stdlib -- --nocapture
bound_seconds=1020.0 exit=101 elapsed_seconds=259.6562009140034
    Finished `test` profile [unoptimized + debuginfo] target(s) in 33.36s
test result: FAILED. 1005 passed; 20 failed; 53 ignored; 0 measured; 0 filtered out; finished in 225.96s
```

[.temp/brief-551/raw/ownership-discovery.txt](../../../.temp/brief-551/raw/ownership-discovery.txt)

```text
python3 scripts/run-bounded.py --timeout 80 -- env CARGO_BUILD_JOBS=4 cargo test -p rum-stdlib tests::ownership_graph::discovery_report_without_inventory_input -- --exact --nocapture
bound_seconds=80.0 exit=0 elapsed_seconds=5.893386444004136
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.11s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1077 filtered out; finished in 5.64s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 47 filtered out; finished in 0.00s
```

[.temp/brief-551/raw/reconcile-inventories.txt](../../../.temp/brief-551/raw/reconcile-inventories.txt)

```text
python3 scripts/run-bounded.py --timeout 10 -- python3 .temp/brief-551/reconcile-test-inventories.py
bound_seconds=10.0 exit=0 elapsed_seconds=0.1789618979964871
    "old": 3090,
    "new": 3097,
    "added": 36,
    "removed": 29
  },
  "host_payload_ledger.tsv": {
    "old": 64,
    "new": 64,
    "added": 14,
    "removed": 14
  }
}
```

[.temp/brief-551/raw/refresh-contract-fingerprints.txt](../../../.temp/brief-551/raw/refresh-contract-fingerprints.txt)

```text
python3 scripts/run-bounded.py --timeout 15 -- python3 .temp/brief-551/refresh-contract-fingerprints.py
bound_seconds=15.0 exit=0 elapsed_seconds=0.8300791799993021
{"context": "fnv1a64:5499537955016613", "rows": 68, "discharged": 62, "pending": 6, "elapsed": 0.6222464260063134, "unchanged_columns": 18}
```

[.temp/brief-551/raw/replay-default-certificates.txt](../../../.temp/brief-551/raw/replay-default-certificates.txt)

```text
python3 scripts/run-bounded.py --timeout 175 -- env CARGO_BUILD_JOBS=4 cargo test -p rum-stdlib tests::retention_closure::local::production::local_production_acceptance -- --exact --nocapture
bound_seconds=175.0 exit=101 elapsed_seconds=37.28254562499933
    Finished `test` profile [unoptimized + debuginfo] target(s) in 19.73s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1077 filtered out; finished in 17.39s
```

[.temp/brief-551/raw/replay-no-default-certificates.txt](../../../.temp/brief-551/raw/replay-no-default-certificates.txt)

```text
python3 scripts/run-bounded.py --timeout 175 -- env CARGO_BUILD_JOBS=4 cargo test -p rum-stdlib --no-default-features tests::retention_closure::local::production::local_production_acceptance -- --exact --nocapture
bound_seconds=175.0 exit=101 elapsed_seconds=43.921912227000576
    Finished `test` profile [unoptimized + debuginfo] target(s) in 25.99s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1071 filtered out; finished in 17.64s
```

[.temp/brief-551/raw/install-certificates.txt](../../../.temp/brief-551/raw/install-certificates.txt)

```text
python3 scripts/run-bounded.py --timeout 10 -- python3 .temp/brief-551/install-certificates.py
bound_seconds=10.0 exit=0 elapsed_seconds=0.12575646200275514
{"body_rows": 2, "instances_per_build": 3, "builds": 2, "updated_columns": [4, 5], "other_columns_unchanged": true, "all_replayed_instance_and_probe_rejections": 0}
```

[.temp/brief-551/raw/gate-stdlib-02.txt](../../../.temp/brief-551/raw/gate-stdlib-02.txt)

```text
python3 scripts/run-bounded.py --timeout 1040 -- env CARGO_BUILD_JOBS=4 cargo test -p rum-stdlib -- --nocapture
bound_seconds=1040.0 exit=0 elapsed_seconds=266.43874775100267
    Finished `test` profile [unoptimized + debuginfo] target(s) in 17.79s
test result: ok. 1025 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 246.57s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.85s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

[.temp/brief-551/raw/gate-fmt.txt](../../../.temp/brief-551/raw/gate-fmt.txt)

```text
python3 scripts/run-bounded.py --timeout 10 -- env CARGO_BUILD_JOBS=4 cargo fmt --check
bound_seconds=10.0 exit=0 elapsed_seconds=1.7021410549932625
(no stdout/stderr)
```

[.temp/brief-551/raw/gate-host.txt](../../../.temp/brief-551/raw/gate-host.txt)

```text
python3 scripts/run-bounded.py --timeout 250 -- env CARGO_BUILD_JOBS=4 cargo test -p rum-host -- --nocapture
bound_seconds=250.0 exit=0 elapsed_seconds=85.06226082000649
    Finished `test` profile [unoptimized + debuginfo] target(s) in 23.06s
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.48s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.60s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.61s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.59s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.75s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.37s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.58s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 53.35s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.78s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.23s
```

[.temp/brief-551/raw/gate-workspace.txt](../../../.temp/brief-551/raw/gate-workspace.txt)

```text
python3 scripts/run-bounded.py --timeout 550 -- env CARGO_BUILD_JOBS=4 RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test --workspace -- --nocapture
bound_seconds=550.0 exit=0 elapsed_seconds=145.87431022699457
    Finished `test` profile [unoptimized + debuginfo] target(s) in 49.84s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.42s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.14s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.14s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.13s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.15s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.14s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.17s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.82s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.25s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.77s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 106 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s
test result: ok. 1094 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 4.97s
test result: ok. 1025 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 68.27s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.84s
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.35s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.79s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.85s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

[.temp/brief-551/raw/release-falsifiers.txt](../../../.temp/brief-551/raw/release-falsifiers.txt)

```text
python3 scripts/run-bounded.py --timeout 145 -- env CARGO_BUILD_JOBS=4 cargo test --release -p rum-runtime --target-dir .temp/brief-551/candidate-target batch_boundary::invariant_falsifier -- --nocapture
bound_seconds=145.0 exit=0 elapsed_seconds=36.22736915899441
   Compiling proc-macro2 v1.0.106
   Compiling quote v1.0.45
   Compiling unicode-ident v1.0.24
   Compiling autocfg v1.5.0
   Compiling serde_core v1.0.228
   Compiling serde v1.0.228
   Compiling num-traits v0.2.19
   Compiling thiserror v2.0.18
   Compiling syn v2.0.117
   Compiling num-integer v0.1.46
   Compiling serde_derive v1.0.228
   Compiling thiserror-impl v2.0.18
   Compiling rum-derive v0.2.0 (/home/remi/dev/rum/crates/rum-derive)
   Compiling rum-syntax v0.2.0 (/home/remi/dev/rum/crates/rum-syntax)
   Compiling num-bigint v0.4.6
   Compiling rum-core v0.2.0 (/home/remi/dev/rum/crates/rum-core)
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
warning: function `register_test_if_builtin` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:320:27
    |
320 | pub(in crate::runtime) fn register_test_if_builtin(session: &mut Session) {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` on by default

warning: function `register_test_return_builtin` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:330:27
    |
330 | pub(in crate::runtime) fn register_test_return_builtin(session: &mut Session) {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `register_test_vm_template_builtins` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:358:27
    |
358 | pub(in crate::runtime) fn register_test_vm_template_builtins(session: &mut Session) {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: `rum-runtime` (lib test) generated 3 warnings
    Finished `release` profile [optimized] target(s) in 36.04s
     Running unittests src/lib.rs (.temp/brief-551/candidate-target/release/deps/rum_runtime-6fbe23fa3fbc760e)

running 5 tests

thread 'runtime::tests::session_requests::batch_boundary::invariant_falsifier_value' panicked at crates/rum-runtime/src/runtime/session/session_requests.rs:608:9:
terminal request outcome retained a suspension owner
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

thread 'runtime::tests::session_requests::batch_boundary::invariant_falsifier_error' panicked at crates/rum-runtime/src/runtime/session/session_requests.rs:608:9:
terminal request outcome retained a suspension owner

thread 'runtime::tests::session_requests::batch_boundary::invariant_falsifier_script' panicked at crates/rum-runtime/src/runtime/session/session_requests.rs:608:9:
terminal request outcome retained a suspension owner

thread 'runtime::tests::session_requests::batch_boundary::invariant_falsifier_snapshot' panicked at crates/rum-runtime/src/runtime/session/session_requests.rs:608:9:
terminal request outcome retained a suspension owner

thread 'runtime::tests::session_requests::batch_boundary::invariant_falsifier_file' panicked at crates/rum-runtime/src/runtime/session/session_requests.rs:608:9:
terminal request outcome retained a suspension owner
test runtime::tests::session_requests::batch_boundary::invariant_falsifier_file - should panic ... ok
test runtime::tests::session_requests::batch_boundary::invariant_falsifier_snapshot - should panic ... ok
test runtime::tests::session_requests::batch_boundary::invariant_falsifier_script - should panic ... ok
test runtime::tests::session_requests::batch_boundary::invariant_falsifier_value - should panic ... ok
test runtime::tests::session_requests::batch_boundary::invariant_falsifier_error - should panic ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 1091 filtered out; finished in 0.00s
```

[.temp/brief-551/raw/ownership-discovery-final.txt](../../../.temp/brief-551/raw/ownership-discovery-final.txt)

```text
python3 scripts/run-bounded.py --timeout 80 -- env CARGO_BUILD_JOBS=4 cargo test -p rum-stdlib tests::ownership_graph::discovery_report_without_inventory_input -- --exact --nocapture
bound_seconds=80.0 exit=0 elapsed_seconds=25.98076832399238
    Finished `test` profile [unoptimized + debuginfo] target(s) in 20.44s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1077 filtered out; finished in 5.37s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 47 filtered out; finished in 0.00s
```

[.temp/brief-551/raw/reconcile-inventories-final.txt](../../../.temp/brief-551/raw/reconcile-inventories-final.txt)

```text
python3 scripts/run-bounded.py --timeout 10 -- python3 .temp/brief-551/reconcile-test-inventories-final.py
bound_seconds=10.0 exit=0 elapsed_seconds=0.12504870499833487
{
  "unknown_attributes.tsv": {
    "old": 1428,
    "new": 1429,
    "added": 1,
    "removed": 0
  },
  "unknown_captures.tsv": {
    "old": 2932,
    "new": 2932,
    "added": 6,
    "removed": 6
  },
  "unknown_macros.tsv": {
    "old": 3097,
    "new": 3097,
    "added": 8,
    "removed": 8
  }
}
```

[.temp/brief-551/raw/refresh-contract-fingerprints-final.txt](../../../.temp/brief-551/raw/refresh-contract-fingerprints-final.txt)

```text
python3 scripts/run-bounded.py --timeout 15 -- python3 .temp/brief-551/refresh-contract-fingerprints-final.py
bound_seconds=15.0 exit=0 elapsed_seconds=0.7760888229968259
{"context": "fnv1a64:a967e363a50573c3", "rows": 68, "discharged": 62, "pending": 6, "elapsed": 0.6072169179969933, "unchanged_columns": 18}
```

[.temp/brief-551/raw/replay-default-certificates-final.txt](../../../.temp/brief-551/raw/replay-default-certificates-final.txt)

```text
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=4 cargo test -p rum-stdlib tests::retention_closure::local::production::local_production_acceptance -- --exact --nocapture
bound_seconds=150.0 exit=101 elapsed_seconds=31.998286012996687
    Finished `test` profile [unoptimized + debuginfo] target(s) in 14.46s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1077 filtered out; finished in 17.40s
```

[.temp/brief-551/raw/replay-no-default-certificates-final.txt](../../../.temp/brief-551/raw/replay-no-default-certificates-final.txt)

```text
python3 scripts/run-bounded.py --timeout 180 -- env CARGO_BUILD_JOBS=4 cargo test -p rum-stdlib --no-default-features tests::retention_closure::local::production::local_production_acceptance -- --exact --nocapture
bound_seconds=180.0 exit=101 elapsed_seconds=33.15372105198912
    Finished `test` profile [unoptimized + debuginfo] target(s) in 15.69s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1071 filtered out; finished in 17.32s
```

[.temp/brief-551/raw/install-certificates-final.txt](../../../.temp/brief-551/raw/install-certificates-final.txt)

```text
python3 scripts/run-bounded.py --timeout 10 -- python3 .temp/brief-551/install-certificates-final.py
bound_seconds=10.0 exit=0 elapsed_seconds=0.12561432700022124
{"body_rows": 2, "instances_per_build": 3, "builds": 2, "updated_columns": [4, 5], "other_columns_unchanged": true, "all_replayed_instance_and_probe_rejections": 0}
```

[.temp/brief-551/raw/gate-fmt-final.txt](../../../.temp/brief-551/raw/gate-fmt-final.txt)

```text
python3 scripts/run-bounded.py --timeout 10 -- env CARGO_BUILD_JOBS=4 cargo fmt --check
bound_seconds=10.0 exit=0 elapsed_seconds=1.7074801920098253
(no stdout/stderr)
```

[.temp/brief-551/raw/gate-no-default-retention.txt](../../../.temp/brief-551/raw/gate-no-default-retention.txt)

```text
python3 scripts/run-bounded.py --timeout 990 -- env CARGO_BUILD_JOBS=4 cargo test -p rum-stdlib --no-default-features tests::retention_closure -- --nocapture
bound_seconds=990.0 exit=0 elapsed_seconds=272.9813849469938
    Finished `test` profile [unoptimized + debuginfo] target(s) in 14.83s
test result: ok. 49 passed; 0 failed; 1 ignored; 0 measured; 1022 filtered out; finished in 257.83s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 47 filtered out; finished in 0.00s
```

[.temp/brief-551/raw/gate-workspace-final.txt](../../../.temp/brief-551/raw/gate-workspace-final.txt)

```text
python3 scripts/run-bounded.py --timeout 590 -- env CARGO_BUILD_JOBS=4 RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test --workspace -- --nocapture
bound_seconds=590.0 exit=0 elapsed_seconds=351.7885232800036
    Finished `test` profile [unoptimized + debuginfo] target(s) in 40.10s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.51s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.82s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.05s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.91s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.06s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.06s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.29s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 39.10s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.81s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
test result: ok. 106 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s
test result: ok. 1094 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 6.21s
test result: ok. 1025 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 247.73s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.85s
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.32s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.49s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.83s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

[.temp/brief-551/raw/archive-boundaries.txt](../../../.temp/brief-551/raw/archive-boundaries.txt)

```text
python3 scripts/run-bounded.py --timeout 145 -- env CARGO_BUILD_JOBS=4 cargo test --manifest-path .temp/brief-551/archive/Cargo.toml --target-dir .temp/brief-551/archive-target --test archive boundaries_and_cancellation -- --exact --nocapture
bound_seconds=145.0 exit=0 elapsed_seconds=25.342948536999756
   Compiling proc-macro2 v1.0.106
   Compiling unicode-ident v1.0.24
   Compiling quote v1.0.45
   Compiling version_check v0.9.5
   Compiling autocfg v1.5.0
   Compiling generic-array v0.14.7
   Compiling typenum v1.20.1
   Compiling num-traits v0.2.19
   Compiling serde_core v1.0.228
   Compiling syn v2.0.117
   Compiling serde v1.0.228
   Compiling block-buffer v0.10.4
   Compiling crypto-common v0.1.7
   Compiling cfg-if v1.0.4
   Compiling thiserror v2.0.18
   Compiling digest v0.10.7
   Compiling num-integer v0.1.46
   Compiling crc32fast v1.5.0
   Compiling cpufeatures v0.2.17
   Compiling sha2 v0.10.9
   Compiling sha1 v0.10.6
   Compiling md-5 v0.10.6
   Compiling serde_derive v1.0.228
   Compiling thiserror-impl v2.0.18
   Compiling rum-derive v0.2.0 (/home/remi/dev/rum/crates/rum-derive)
   Compiling rum-syntax v0.2.0 (/home/remi/dev/rum/crates/rum-syntax)
   Compiling num-bigint v0.4.6
   Compiling rum-core v0.2.0 (/home/remi/dev/rum/crates/rum-core)
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
   Compiling brief551-archive-controls v0.0.0 (/home/remi/dev/rum/.temp/brief-551/archive)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 25.10s
     Running ../../brief-549/probe.rs (.temp/brief-551/archive-target/debug/deps/archive-05c7be462bb14d21)

running 1 test
BOUNDARY	script	initial=View { status: "InputRequired { request: RuntimeInputRequest { kind: InputString, prompt: \"p: \", password: false } }", value: None, termination: "None", messages: [], prints: [] }	pending=Some(RuntimeInputRequest { kind: InputString, prompt: "p: ", password: false })
BOUNDARY	script	next=View { status: "Other { message: \"cannot start a new evaluation while the session is waiting for host input; call resume_with_input or reset first\" }", value: None, termination: "None", messages: [], prints: [] }
BOUNDARY	script	resume=View { status: "Other { message: \"no suspended evaluation is waiting for host input\" }", value: None, termination: "None", messages: [], prints: [] }
BOUNDARY	script	cancel=View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: [] }
BOUNDARY	script	after_cancel=View { status: "OK", value: Some("2"), termination: "None", messages: [], prints: [] }	state="List[List[1], \"Global`\", List[\"System`\", \"Global`\"], List[], \"\", 0, 100]"
BOUNDARY	file	initial=View { status: "InputRequired { request: RuntimeInputRequest { kind: InputString, prompt: \"p: \", password: false } }", value: None, termination: "None", messages: [], prints: [] }	pending=Some(RuntimeInputRequest { kind: InputString, prompt: "p: ", password: false })
BOUNDARY	file	next=View { status: "Other { message: \"cannot start a new evaluation while the session is waiting for host input; call resume_with_input or reset first\" }", value: None, termination: "None", messages: [], prints: [] }
BOUNDARY	file	resume=View { status: "Other { message: \"no suspended evaluation is waiting for host input\" }", value: None, termination: "None", messages: [], prints: [] }
BOUNDARY	file	cancel=View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: [] }
BOUNDARY	file	after_cancel=View { status: "OK", value: Some("2"), termination: "None", messages: [], prints: [] }	state="List[List[], \"Global`\", List[\"System`\", \"Global`\"], List[], \"\", 0, 100]"
BOUNDARY	snapshot	initial=View { status: "InputRequired { request: RuntimeInputRequest { kind: InputString, prompt: \"p: \", password: false } }", value: None, termination: "None", messages: [], prints: [] }	pending=Some(RuntimeInputRequest { kind: InputString, prompt: "p: ", password: false })
BOUNDARY	snapshot	next=View { status: "Other { message: \"cannot start a new evaluation while the session is waiting for host input; call resume_with_input or reset first\" }", value: None, termination: "None", messages: [], prints: [] }
BOUNDARY	snapshot	resume=View { status: "Other { message: \"no suspended evaluation is waiting for host input\" }", value: None, termination: "None", messages: [], prints: [] }
BOUNDARY	snapshot	cancel=View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: [] }
BOUNDARY	snapshot	after_cancel=View { status: "OK", value: Some("2"), termination: "None", messages: [], prints: [] }	state="List[List[1], \"Global`\", List[\"System`\", \"Global`\"], List[], \"\", 0, 100]"
CANCEL	get_cancel	initial=Some(View { status: "InputRequired { request: RuntimeInputRequest { kind: InputString, prompt: \"p: \", password: false } }", value: None, termination: "None", messages: [], prints: [] })	pending=Some(RuntimeInputRequest { kind: InputString, prompt: "p: ", password: false })
CANCEL	get_cancel	finish=Some(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: [] })	next=View { status: "OK", value: Some("2"), termination: "None", messages: [], prints: [] }	state="List[List[], \"Global`\", List[\"System`\", \"Global`\"], List[], \"\", 0, 100]"
REBUILD	get_cancel	next=View { status: "OK", value: Some("2"), termination: "None", messages: [], prints: [] }	pending=None
CANCEL	needs_cancel	initial=Some(View { status: "InputRequired { request: RuntimeInputRequest { kind: InputString, prompt: \"p: \", password: false } }", value: None, termination: "None", messages: [], prints: [] })	pending=Some(RuntimeInputRequest { kind: InputString, prompt: "p: ", password: false })
CANCEL	needs_cancel	finish=Some(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: [] })	next=View { status: "OK", value: Some("2"), termination: "None", messages: [], prints: [] }	state="List[List[], \"Global`\", List[\"System`\", \"Global`\"], List[], \"\", 0, 100]"
REBUILD	needs_cancel	next=View { status: "OK", value: Some("2"), termination: "None", messages: [], prints: [] }	pending=None
CANCEL	begin_cancel	initial=Some(View { status: "InputRequired { request: RuntimeInputRequest { kind: InputString, prompt: \"p: \", password: false } }", value: None, termination: "None", messages: [], prints: [] })	pending=Some(RuntimeInputRequest { kind: InputString, prompt: "p: ", password: false })
CANCEL	begin_cancel	finish=Some(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: [] })	next=View { status: "OK", value: Some("2"), termination: "None", messages: [], prints: [] }	state="List[List[], \"Global`\", List[\"System`\", \"Global`\"], List[], \"\", 0, 100]"
REBUILD	begin_cancel	next=View { status: "OK", value: Some("2"), termination: "None", messages: [], prints: [] }	pending=None
CANCEL	nested_cancel	initial=Some(View { status: "InputRequired { request: RuntimeInputRequest { kind: InputString, prompt: \"p: \", password: false } }", value: None, termination: "None", messages: [], prints: [] })	pending=Some(RuntimeInputRequest { kind: InputString, prompt: "p: ", password: false })
CANCEL	nested_cancel	finish=Some(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: [] })	next=View { status: "OK", value: Some("2"), termination: "None", messages: [], prints: [] }	state="List[List[], \"Global`\", List[\"System`\", \"Global`\"], List[], \"\", 0, 100]"
REBUILD	nested_cancel	next=View { status: "OK", value: Some("2"), termination: "None", messages: [], prints: [] }	pending=None
CANCEL	stub_cancel	initial=Some(View { status: "InputRequired { request: RuntimeInputRequest { kind: InputString, prompt: \"p: \", password: false } }", value: None, termination: "None", messages: [], prints: [] })	pending=Some(RuntimeInputRequest { kind: InputString, prompt: "p: ", password: false })
CANCEL	stub_cancel	finish=Some(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: [] })	next=View { status: "OK", value: Some("2"), termination: "None", messages: [], prints: [] }	state="List[List[], \"Global`\", List[\"Dep549`\", \"System`\", \"Global`\"], List[], \"\", 0, 100]"
REBUILD	stub_cancel	next=View { status: "OK", value: Some("2"), termination: "None", messages: [], prints: [] }	pending=None
CANCEL	cleanup_get_cancel	initial=Some(View { status: "InputRequired { request: RuntimeInputRequest { kind: InputString, prompt: \"p: \", password: false } }", value: None, termination: "None", messages: [], prints: [] })	pending=Some(RuntimeInputRequest { kind: InputString, prompt: "p: ", password: false })
CANCEL	cleanup_get_cancel	finish=Some(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: [] })	next=View { status: "OK", value: Some("2"), termination: "None", messages: [], prints: [] }	state="List[List[], \"Global`\", List[\"System`\", \"Global`\"], List[], \"\", 0, 100]"
REBUILD	cleanup_get_cancel	next=View { status: "OK", value: Some("2"), termination: "None", messages: [], prints: [] }	pending=None
CANCEL	cleanup_cancel	initial=Some(View { status: "InputRequired { request: RuntimeInputRequest { kind: InputString, prompt: \"p: \", password: false } }", value: None, termination: "None", messages: [], prints: [] })	pending=Some(RuntimeInputRequest { kind: InputString, prompt: "p: ", password: false })
CANCEL	cleanup_cancel	finish=Some(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: [] })	next=View { status: "OK", value: Some("2"), termination: "None", messages: [], prints: [] }	state="List[List[8], \"Global`\", List[\"System`\", \"Global`\"], List[], \"\", 0, 100]"
REBUILD	cleanup_cancel	next=View { status: "OK", value: Some("2"), termination: "None", messages: [], prints: [] }	pending=None
OVERLAP	initial=View { status: "InputRequired { request: RuntimeInputRequest { kind: InputString, prompt: \"first: \", password: false } }", value: None, termination: "None", messages: [], prints: [] }	file=View { status: "Other { message: \"cannot start a new evaluation while the session is waiting for host input; call resume_with_input or reset first\" }", value: None, termination: "None", messages: [], prints: [] }	reads=[]
OVERLAP	resume=View { status: "OK", value: Some("\"one\""), termination: "None", messages: [], prints: [] }	next=View { status: "OK", value: Some("2"), termination: "None", messages: [], prints: [] }
test boundaries_and_cancellation ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 49 filtered out; finished in 0.05s
```

[.temp/brief-551/raw/candidate-default-build.txt](../../../.temp/brief-551/raw/candidate-default-build.txt)

```text
python3 scripts/run-bounded.py --timeout 145 -- env CARGO_BUILD_JOBS=4 cargo build --release -p rum-cli --target-dir .temp/brief-551/candidate-target
bound_seconds=145.0 exit=0 elapsed_seconds=32.6273602059955
    Finished `release` profile [optimized] target(s) in 32.37s
```

[.temp/brief-551/raw/candidate-alloc-build.txt](../../../.temp/brief-551/raw/candidate-alloc-build.txt)

```text
python3 scripts/run-bounded.py --timeout 10 -- env CARGO_BUILD_JOBS=4 cargo build --release -p rum-cli --features alloc-count --target-dir .temp/brief-551/candidate-target
bound_seconds=10.0 exit=0 elapsed_seconds=1.7846273869945435
    Finished `release` profile [optimized] target(s) in 1.62s
```

[.temp/brief-551/perf/alloc-base.txt](../../../.temp/brief-551/perf/alloc-base.txt)

```text
python3 scripts/run-bounded.py --timeout 80 -- python3 .temp/brief-551/perf_panel.py alloc base 1
bound_seconds=80.0 exit=0 elapsed_seconds=1.2316136000008555
(per-label raw commands/results below)
```

[.temp/brief-551/perf/alloc-candidate.txt](../../../.temp/brief-551/perf/alloc-candidate.txt)

```text
python3 scripts/run-bounded.py --timeout 80 -- python3 .temp/brief-551/perf_panel.py alloc candidate 1
bound_seconds=80.0 exit=0 elapsed_seconds=1.2301011719973758
(per-label raw commands/results below)
```

[.temp/brief-551/perf/instructions-base-1.txt](../../../.temp/brief-551/perf/instructions-base-1.txt)

```text
python3 scripts/run-bounded.py --timeout 80 -- python3 .temp/brief-551/perf_panel.py instructions base 1
bound_seconds=80.0 exit=0 elapsed_seconds=1.7329890099936165
(per-label raw commands/results below)
```

[.temp/brief-551/perf/instructions-candidate-1.txt](../../../.temp/brief-551/perf/instructions-candidate-1.txt)

```text
python3 scripts/run-bounded.py --timeout 80 -- python3 .temp/brief-551/perf_panel.py instructions candidate 1
bound_seconds=80.0 exit=0 elapsed_seconds=1.0023680459999014
(per-label raw commands/results below)
```

[.temp/brief-551/perf/instructions-base-2.txt](../../../.temp/brief-551/perf/instructions-base-2.txt)

```text
python3 scripts/run-bounded.py --timeout 80 -- python3 .temp/brief-551/perf_panel.py instructions base 2
bound_seconds=80.0 exit=0 elapsed_seconds=0.9997830170032103
(per-label raw commands/results below)
```

[.temp/brief-551/perf/instructions-candidate-2.txt](../../../.temp/brief-551/perf/instructions-candidate-2.txt)

```text
python3 scripts/run-bounded.py --timeout 80 -- python3 .temp/brief-551/perf_panel.py instructions candidate 2
bound_seconds=80.0 exit=0 elapsed_seconds=1.0551745209959336
(per-label raw commands/results below)
```

[.temp/brief-551/perf/compare.txt](../../../.temp/brief-551/perf/compare.txt)

```text
python3 scripts/run-bounded.py --timeout 5 -- python3 .temp/brief-551/compare-performance.py
bound_seconds=5.0 exit=0 elapsed_seconds=0.059951722010737285
(per-label raw commands/results below)
```


Workspace runs set `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`. Loopback binding was
denied, so these nine tests took their explicit skip path. Cargo prints `ok`
for that path; it is not native transport coverage:

```text
test zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket ... ok
test zeromq::tests::native::basic::native_zeromq_backend_round_trips_execute_over_loopback_sockets ... ok
test zeromq::tests::native::frontend::control::native_zeromq_backend_frontend_style_control_interrupt_round_trip ... ok
test zeromq::tests::native::frontend::control::native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages ... ok
test zeromq::tests::native::frontend::execute::native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata ... ok
test zeromq::tests::native::frontend::session_flow::native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown ... ok
test zeromq::tests::native::frontend::stdin::native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply ... ok
test zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages ... ok
test zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end ... ok
```

The 58 ordinary ignored tests are listed in
[ignored-tests.txt](../../../.temp/brief-551/ignored-tests.txt).

### 3. Allocation and per-iteration instructions

Both base binaries were freshly built into the initially absent
`.temp/brief-551/base-target/` and copied before source/test edits. Original
source bytes/hashes were also preserved. Candidate builds use a separate
target directory. Binary hashes/byte checks prove the base lacks the new
rollback code, the candidate contains it, and the production release candidate
does not contain the terminal-owner diagnostic message:

```json
[
  {
    "build": "base",
    "feature": "default",
    "path": ".temp/brief-551/base/rum-cli",
    "sha256": "eea410075552dc066eb986b20c6851cec19e876744dba6e336daa0c0a1b739fd",
    "bytes": 13643800,
    "new_code_markers": {
      "public files close inner to outer exactly once": false,
      "a public file rollback obligation is abandonment-only": false
    },
    "terminal_owner_test_guard_present": false
  },
  {
    "build": "candidate",
    "feature": "default",
    "path": ".temp/brief-551/next/rum-cli",
    "sha256": "52f4ed1135097fcd5e7455a2ec5748b47a3c01c9d23d0a52af3321b4212839c2",
    "bytes": 13655016,
    "new_code_markers": {
      "public files close inner to outer exactly once": true,
      "a public file rollback obligation is abandonment-only": true
    },
    "terminal_owner_test_guard_present": false
  },
  {
    "build": "base",
    "feature": "alloc-count",
    "path": ".temp/brief-551/base/rum-cli-alloc",
    "sha256": "8925626103770858c515af25db3775a5ff46f5a26542e85f1106b03d7d8054d2",
    "bytes": 13675216,
    "new_code_markers": {
      "public files close inner to outer exactly once": false,
      "a public file rollback obligation is abandonment-only": false
    },
    "terminal_owner_test_guard_present": false
  },
  {
    "build": "candidate",
    "feature": "alloc-count",
    "path": ".temp/brief-551/next/rum-cli-alloc",
    "sha256": "91f4c098313291e9341f7ed0a58f52e475e9abcab3e4c35059e9c235dcb35114",
    "bytes": 13686432,
    "new_code_markers": {
      "public files close inner to outer exactly once": true,
      "a public file rollback obligation is abandonment-only": true
    },
    "terminal_owner_test_guard_present": false
  }
]
```

Each label uses the timing suite's exact statement shape, identical setup,
one label per fresh process, N=20,000 and 2N=40,000. Allocation runs precede
instructions. Binary path components `base` and `next` have equal lengths.
Instructions use `taskset -c 2`, `cpu_core/instructions/u` and stdin
`< /dev/null`. Slopes are `(count(2N)-count(N))/N`. There is one allocation
sample and at most two instruction samples per tuple. No per-step evaluator,
dispatch, matcher, Expr construction or Drop implementation changed.

| Label | N count base/new | 2N count base/new | Allocations/iteration base/new | N bytes_total base/new | N peak_live_bytes base/new |
| --- | --- | --- | --- | --- | --- |
| literal-integer | 1890 / 1890 | 1890 / 1890 | 0.0 / 0.0 | 1144400 / 1144400 | 945378 / 945378 |
| symbol-ownvalue-read | 21891 / 21891 | 41891 / 41891 | 1.0 / 1.0 | 3704617 / 3704617 | 945519 / 945519 |
| set-ownvalue | 21902 / 21902 | 41902 / 41902 | 1.0 / 1.0 | 3704787 / 3704787 | 945670 / 945670 |
| do-count-null | 1859 / 1859 | 1859 / 1859 | 0.0 / 0.0 | 503658 / 503658 | 306269 / 306269 |
| block-local | 161937 / 161937 | 321937 / 321937 | 8.0 / 8.0 | 16188824 / 16188824 | 948830 / 948830 |
| table-iterator-null | 1885 / 1885 | 1885 / 1885 | 0.0 / 0.0 | 1144458 / 1144458 | 945223 / 945223 |

| Label | Sample | Base N / 2N counts | New N / 2N counts | Base instructions/iteration | New instructions/iteration | Delta |
| --- | --- | --- | --- | --- | --- | --- |
| literal-integer | 1 | 22295917 / 40875696 | 22291778 / 40877339 | 928.98895 | 929.27805 | +0.28910 (+0.0311%) |
| literal-integer | 2 | 22294020 / 40873850 | 22292563 / 40875502 | 928.99150 | 929.14695 | +0.15545 (+0.0167%) |
| symbol-ownvalue-read | 1 | 47823297 / 91932781 | 47825816 / 91936273 | 2205.47420 | 2205.52285 | +0.04865 (+0.0022%) |
| symbol-ownvalue-read | 2 | 47824335 / 91932313 | 47825657 / 91934822 | 2205.39890 | 2205.45825 | +0.05935 (+0.0027%) |
| set-ownvalue | 1 | 98251673 / 192784983 | 98252671 / 192784529 | 4726.66550 | 4726.59290 | -0.07260 (-0.0015%) |
| set-ownvalue | 2 | 98252251 / 192789088 | 98255110 / 192783870 | 4726.84185 | 4726.43800 | -0.40385 (-0.0085%) |
| do-count-null | 1 | 13657034 / 23616556 | 13658364 / 23615798 | 497.97610 | 497.87170 | -0.10440 (-0.0210%) |
| do-count-null | 2 | 13657626 / 23614130 | 13660629 / 23617250 | 497.82520 | 497.83105 | +0.00585 (+0.0012%) |
| block-local | 1 | 311132653 / 618526452 | 311132901 / 618527251 | 15369.68995 | 15369.71750 | +0.02755 (+0.0002%) |
| block-local | 2 | 311133100 / 618527408 | 311133623 / 618529041 | 15369.71540 | 15369.77090 | +0.05550 (+0.0004%) |
| table-iterator-null | 1 | 10694083 / 17674050 | 10696774 / 17676420 | 348.99835 | 348.98230 | -0.01605 (-0.0046%) |
| table-iterator-null | 2 | 10693125 / 17674707 | 10697284 / 17675501 | 349.07910 | 348.91085 | -0.16825 (-0.0482%) |

Allocation counts, bytes_total and peak_live_bytes match exactly between base
and candidate for all 12 label/size pairs. Allocation slopes also match: 0, 1,
1, 0, 8 and 0 allocations/iteration in the listed label order.

All 48 instruction processes exited 0 and reported 100.00% event running time.
Measured paired changes range from -0.04820% to +0.03112%. Seven of the twelve
paired slope rows are positive; they are reported, not rounded to zero:

- literal-integer: +0.28910 / +0.15545 instructions per iteration
  (+0.03112% / +0.01673%).
- symbol-ownvalue-read: +0.04865 / +0.05935 (+0.00221% / +0.00269%).
- block-local: +0.02755 / +0.05550 (+0.00018% / +0.00036%).
- do-count-null sample 2: +0.00585 (+0.00118%); sample 1 is -0.10440.

The other five paired rows decrease. These sub-instruction differences are
small, but two samples cannot establish that every positive difference is
noise or prove an exact zero default-path cost. No instruction regression row
was eliminated by a follow-up code change; the seven observed positive rows
remain disclosed for review. No third sample was taken. No timing panel was
run, and these are instruction-count conclusions only.

Full N/2N allocation counts/bytes/peaks and instruction slopes are retained in
[comparison.json](../../../.temp/brief-551/perf/comparison.json).
The raw performance commands/results follow. Timing was optional for this
boundary change and was not run.

```text
python3 scripts/run-bounded.py --timeout 6 -- /home/remi/dev/rum/.temp/brief-551/base/rum-cli-alloc --file /home/remi/dev/rum/.temp/brief-551/perf/literal-integer-20000.rum < /dev/null
exit=0 bound_seconds=6
Null
rum-profile alloc count=1890 bytes_total=1144400 peak_live_bytes=945378
```

```text
python3 scripts/run-bounded.py --timeout 6 -- /home/remi/dev/rum/.temp/brief-551/base/rum-cli-alloc --file /home/remi/dev/rum/.temp/brief-551/perf/literal-integer-40000.rum < /dev/null
exit=0 bound_seconds=6
Null
rum-profile alloc count=1890 bytes_total=1784400 peak_live_bytes=1585378
```

```text
python3 scripts/run-bounded.py --timeout 6 -- /home/remi/dev/rum/.temp/brief-551/base/rum-cli-alloc --file /home/remi/dev/rum/.temp/brief-551/perf/symbol-ownvalue-read-20000.rum < /dev/null
exit=0 bound_seconds=6
Null
rum-profile alloc count=21891 bytes_total=3704617 peak_live_bytes=945519
```

```text
python3 scripts/run-bounded.py --timeout 6 -- /home/remi/dev/rum/.temp/brief-551/base/rum-cli-alloc --file /home/remi/dev/rum/.temp/brief-551/perf/symbol-ownvalue-read-40000.rum < /dev/null
exit=0 bound_seconds=6
Null
rum-profile alloc count=41891 bytes_total=6904617 peak_live_bytes=1585519
```

```text
python3 scripts/run-bounded.py --timeout 6 -- /home/remi/dev/rum/.temp/brief-551/base/rum-cli-alloc --file /home/remi/dev/rum/.temp/brief-551/perf/set-ownvalue-20000.rum < /dev/null
exit=0 bound_seconds=6
Null
rum-profile alloc count=21902 bytes_total=3704787 peak_live_bytes=945670
```

```text
python3 scripts/run-bounded.py --timeout 6 -- /home/remi/dev/rum/.temp/brief-551/base/rum-cli-alloc --file /home/remi/dev/rum/.temp/brief-551/perf/set-ownvalue-40000.rum < /dev/null
exit=0 bound_seconds=6
Null
rum-profile alloc count=41902 bytes_total=6904787 peak_live_bytes=1585670
```

```text
python3 scripts/run-bounded.py --timeout 6 -- /home/remi/dev/rum/.temp/brief-551/base/rum-cli-alloc --file /home/remi/dev/rum/.temp/brief-551/perf/do-count-null-20000.rum < /dev/null
exit=0 bound_seconds=6
Null
rum-profile alloc count=1859 bytes_total=503658 peak_live_bytes=306269
```

```text
python3 scripts/run-bounded.py --timeout 6 -- /home/remi/dev/rum/.temp/brief-551/base/rum-cli-alloc --file /home/remi/dev/rum/.temp/brief-551/perf/do-count-null-40000.rum < /dev/null
exit=0 bound_seconds=6
Null
rum-profile alloc count=1859 bytes_total=503658 peak_live_bytes=306269
```

```text
python3 scripts/run-bounded.py --timeout 6 -- /home/remi/dev/rum/.temp/brief-551/base/rum-cli-alloc --file /home/remi/dev/rum/.temp/brief-551/perf/block-local-20000.rum < /dev/null
exit=0 bound_seconds=6
Null
rum-profile alloc count=161937 bytes_total=16188824 peak_live_bytes=948830
```

```text
python3 scripts/run-bounded.py --timeout 6 -- /home/remi/dev/rum/.temp/brief-551/base/rum-cli-alloc --file /home/remi/dev/rum/.temp/brief-551/perf/block-local-40000.rum < /dev/null
exit=0 bound_seconds=6
Null
rum-profile alloc count=321937 bytes_total=31868824 peak_live_bytes=1588830
```

```text
python3 scripts/run-bounded.py --timeout 6 -- /home/remi/dev/rum/.temp/brief-551/base/rum-cli-alloc --file /home/remi/dev/rum/.temp/brief-551/perf/table-iterator-null-20000.rum < /dev/null
exit=0 bound_seconds=6
Null
rum-profile alloc count=1885 bytes_total=1144458 peak_live_bytes=945223
```

```text
python3 scripts/run-bounded.py --timeout 6 -- /home/remi/dev/rum/.temp/brief-551/base/rum-cli-alloc --file /home/remi/dev/rum/.temp/brief-551/perf/table-iterator-null-40000.rum < /dev/null
exit=0 bound_seconds=6
Null
rum-profile alloc count=1885 bytes_total=1784458 peak_live_bytes=1585223
```

```text
python3 scripts/run-bounded.py --timeout 6 -- /home/remi/dev/rum/.temp/brief-551/next/rum-cli-alloc --file /home/remi/dev/rum/.temp/brief-551/perf/literal-integer-20000.rum < /dev/null
exit=0 bound_seconds=6
Null
rum-profile alloc count=1890 bytes_total=1144400 peak_live_bytes=945378
```

```text
python3 scripts/run-bounded.py --timeout 6 -- /home/remi/dev/rum/.temp/brief-551/next/rum-cli-alloc --file /home/remi/dev/rum/.temp/brief-551/perf/literal-integer-40000.rum < /dev/null
exit=0 bound_seconds=6
Null
rum-profile alloc count=1890 bytes_total=1784400 peak_live_bytes=1585378
```

```text
python3 scripts/run-bounded.py --timeout 6 -- /home/remi/dev/rum/.temp/brief-551/next/rum-cli-alloc --file /home/remi/dev/rum/.temp/brief-551/perf/symbol-ownvalue-read-20000.rum < /dev/null
exit=0 bound_seconds=6
Null
rum-profile alloc count=21891 bytes_total=3704617 peak_live_bytes=945519
```

```text
python3 scripts/run-bounded.py --timeout 6 -- /home/remi/dev/rum/.temp/brief-551/next/rum-cli-alloc --file /home/remi/dev/rum/.temp/brief-551/perf/symbol-ownvalue-read-40000.rum < /dev/null
exit=0 bound_seconds=6
Null
rum-profile alloc count=41891 bytes_total=6904617 peak_live_bytes=1585519
```

```text
python3 scripts/run-bounded.py --timeout 6 -- /home/remi/dev/rum/.temp/brief-551/next/rum-cli-alloc --file /home/remi/dev/rum/.temp/brief-551/perf/set-ownvalue-20000.rum < /dev/null
exit=0 bound_seconds=6
Null
rum-profile alloc count=21902 bytes_total=3704787 peak_live_bytes=945670
```

```text
python3 scripts/run-bounded.py --timeout 6 -- /home/remi/dev/rum/.temp/brief-551/next/rum-cli-alloc --file /home/remi/dev/rum/.temp/brief-551/perf/set-ownvalue-40000.rum < /dev/null
exit=0 bound_seconds=6
Null
rum-profile alloc count=41902 bytes_total=6904787 peak_live_bytes=1585670
```

```text
python3 scripts/run-bounded.py --timeout 6 -- /home/remi/dev/rum/.temp/brief-551/next/rum-cli-alloc --file /home/remi/dev/rum/.temp/brief-551/perf/do-count-null-20000.rum < /dev/null
exit=0 bound_seconds=6
Null
rum-profile alloc count=1859 bytes_total=503658 peak_live_bytes=306269
```

```text
python3 scripts/run-bounded.py --timeout 6 -- /home/remi/dev/rum/.temp/brief-551/next/rum-cli-alloc --file /home/remi/dev/rum/.temp/brief-551/perf/do-count-null-40000.rum < /dev/null
exit=0 bound_seconds=6
Null
rum-profile alloc count=1859 bytes_total=503658 peak_live_bytes=306269
```

```text
python3 scripts/run-bounded.py --timeout 6 -- /home/remi/dev/rum/.temp/brief-551/next/rum-cli-alloc --file /home/remi/dev/rum/.temp/brief-551/perf/block-local-20000.rum < /dev/null
exit=0 bound_seconds=6
Null
rum-profile alloc count=161937 bytes_total=16188824 peak_live_bytes=948830
```

```text
python3 scripts/run-bounded.py --timeout 6 -- /home/remi/dev/rum/.temp/brief-551/next/rum-cli-alloc --file /home/remi/dev/rum/.temp/brief-551/perf/block-local-40000.rum < /dev/null
exit=0 bound_seconds=6
Null
rum-profile alloc count=321937 bytes_total=31868824 peak_live_bytes=1588830
```

```text
python3 scripts/run-bounded.py --timeout 6 -- /home/remi/dev/rum/.temp/brief-551/next/rum-cli-alloc --file /home/remi/dev/rum/.temp/brief-551/perf/table-iterator-null-20000.rum < /dev/null
exit=0 bound_seconds=6
Null
rum-profile alloc count=1885 bytes_total=1144458 peak_live_bytes=945223
```

```text
python3 scripts/run-bounded.py --timeout 6 -- /home/remi/dev/rum/.temp/brief-551/next/rum-cli-alloc --file /home/remi/dev/rum/.temp/brief-551/perf/table-iterator-null-40000.rum < /dev/null
exit=0 bound_seconds=6
Null
rum-profile alloc count=1885 bytes_total=1784458 peak_live_bytes=1585223
```

```text
python3 scripts/run-bounded.py --timeout 6 -- taskset -c 2 perf stat -x, -e cpu_core/instructions/u -o /home/remi/dev/rum/.temp/brief-551/perf/instructions/base/1/literal-integer-20000.csv -- /home/remi/dev/rum/.temp/brief-551/base/rum-cli --file /home/remi/dev/rum/.temp/brief-551/perf/literal-integer-20000.rum < /dev/null
exit=0 bound_seconds=6
Null
# started on Wed Sep 30 03:55:34 2026

22295917,,cpu_core/instructions/u,16366115,100.00,,
```

```text
python3 scripts/run-bounded.py --timeout 6 -- taskset -c 2 perf stat -x, -e cpu_core/instructions/u -o /home/remi/dev/rum/.temp/brief-551/perf/instructions/base/1/literal-integer-40000.csv -- /home/remi/dev/rum/.temp/brief-551/base/rum-cli --file /home/remi/dev/rum/.temp/brief-551/perf/literal-integer-40000.rum < /dev/null
exit=0 bound_seconds=6
Null
# started on Wed Sep 30 03:55:34 2026

40875696,,cpu_core/instructions/u,8530805,100.00,,
```

```text
python3 scripts/run-bounded.py --timeout 6 -- taskset -c 2 perf stat -x, -e cpu_core/instructions/u -o /home/remi/dev/rum/.temp/brief-551/perf/instructions/base/1/symbol-ownvalue-read-20000.csv -- /home/remi/dev/rum/.temp/brief-551/base/rum-cli --file /home/remi/dev/rum/.temp/brief-551/perf/symbol-ownvalue-read-20000.rum < /dev/null
exit=0 bound_seconds=6
Null
# started on Wed Sep 30 03:55:34 2026

47823297,,cpu_core/instructions/u,9738384,100.00,,
```

```text
python3 scripts/run-bounded.py --timeout 6 -- taskset -c 2 perf stat -x, -e cpu_core/instructions/u -o /home/remi/dev/rum/.temp/brief-551/perf/instructions/base/1/symbol-ownvalue-read-40000.csv -- /home/remi/dev/rum/.temp/brief-551/base/rum-cli --file /home/remi/dev/rum/.temp/brief-551/perf/symbol-ownvalue-read-40000.rum < /dev/null
exit=0 bound_seconds=6
Null
# started on Wed Sep 30 03:55:34 2026

91932781,,cpu_core/instructions/u,15570668,100.00,,
```

```text
python3 scripts/run-bounded.py --timeout 6 -- taskset -c 2 perf stat -x, -e cpu_core/instructions/u -o /home/remi/dev/rum/.temp/brief-551/perf/instructions/base/1/set-ownvalue-20000.csv -- /home/remi/dev/rum/.temp/brief-551/base/rum-cli --file /home/remi/dev/rum/.temp/brief-551/perf/set-ownvalue-20000.rum < /dev/null
exit=0 bound_seconds=6
Null
# started on Wed Sep 30 03:55:34 2026

98251673,,cpu_core/instructions/u,17828950,100.00,,
```

```text
python3 scripts/run-bounded.py --timeout 6 -- taskset -c 2 perf stat -x, -e cpu_core/instructions/u -o /home/remi/dev/rum/.temp/brief-551/perf/instructions/base/1/set-ownvalue-40000.csv -- /home/remi/dev/rum/.temp/brief-551/base/rum-cli --file /home/remi/dev/rum/.temp/brief-551/perf/set-ownvalue-40000.rum < /dev/null
exit=0 bound_seconds=6
Null
# started on Wed Sep 30 03:55:34 2026

192784983,,cpu_core/instructions/u,33858313,100.00,,
```

```text
python3 scripts/run-bounded.py --timeout 6 -- taskset -c 2 perf stat -x, -e cpu_core/instructions/u -o /home/remi/dev/rum/.temp/brief-551/perf/instructions/base/1/do-count-null-20000.csv -- /home/remi/dev/rum/.temp/brief-551/base/rum-cli --file /home/remi/dev/rum/.temp/brief-551/perf/do-count-null-20000.rum < /dev/null
exit=0 bound_seconds=6
Null
# started on Wed Sep 30 03:55:34 2026

13657034,,cpu_core/instructions/u,4967656,100.00,,
```

```text
python3 scripts/run-bounded.py --timeout 6 -- taskset -c 2 perf stat -x, -e cpu_core/instructions/u -o /home/remi/dev/rum/.temp/brief-551/perf/instructions/base/1/do-count-null-40000.csv -- /home/remi/dev/rum/.temp/brief-551/base/rum-cli --file /home/remi/dev/rum/.temp/brief-551/perf/do-count-null-40000.rum < /dev/null
exit=0 bound_seconds=6
Null
# started on Wed Sep 30 03:55:35 2026

23616556,,cpu_core/instructions/u,6663038,100.00,,
```

```text
python3 scripts/run-bounded.py --timeout 6 -- taskset -c 2 perf stat -x, -e cpu_core/instructions/u -o /home/remi/dev/rum/.temp/brief-551/perf/instructions/base/1/block-local-20000.csv -- /home/remi/dev/rum/.temp/brief-551/base/rum-cli --file /home/remi/dev/rum/.temp/brief-551/perf/block-local-20000.rum < /dev/null
exit=0 bound_seconds=6
Null
# started on Wed Sep 30 03:55:35 2026

311132653,,cpu_core/instructions/u,48209827,100.00,,
```

```text
python3 scripts/run-bounded.py --timeout 6 -- taskset -c 2 perf stat -x, -e cpu_core/instructions/u -o /home/remi/dev/rum/.temp/brief-551/perf/instructions/base/1/block-local-40000.csv -- /home/remi/dev/rum/.temp/brief-551/base/rum-cli --file /home/remi/dev/rum/.temp/brief-551/perf/block-local-40000.rum < /dev/null
exit=0 bound_seconds=6
Null
# started on Wed Sep 30 03:55:35 2026

618526452,,cpu_core/instructions/u,87582742,100.00,,
```

```text
python3 scripts/run-bounded.py --timeout 6 -- taskset -c 2 perf stat -x, -e cpu_core/instructions/u -o /home/remi/dev/rum/.temp/brief-551/perf/instructions/base/1/table-iterator-null-20000.csv -- /home/remi/dev/rum/.temp/brief-551/base/rum-cli --file /home/remi/dev/rum/.temp/brief-551/perf/table-iterator-null-20000.rum < /dev/null
exit=0 bound_seconds=6
Null
# started on Wed Sep 30 03:55:35 2026

10694083,,cpu_core/instructions/u,4828565,100.00,,
```

```text
python3 scripts/run-bounded.py --timeout 6 -- taskset -c 2 perf stat -x, -e cpu_core/instructions/u -o /home/remi/dev/rum/.temp/brief-551/perf/instructions/base/1/table-iterator-null-40000.csv -- /home/remi/dev/rum/.temp/brief-551/base/rum-cli --file /home/remi/dev/rum/.temp/brief-551/perf/table-iterator-null-40000.rum < /dev/null
exit=0 bound_seconds=6
Null
# started on Wed Sep 30 03:55:35 2026

17674050,,cpu_core/instructions/u,5902071,100.00,,
```

```text
python3 scripts/run-bounded.py --timeout 6 -- taskset -c 2 perf stat -x, -e cpu_core/instructions/u -o /home/remi/dev/rum/.temp/brief-551/perf/instructions/base/2/literal-integer-20000.csv -- /home/remi/dev/rum/.temp/brief-551/base/rum-cli --file /home/remi/dev/rum/.temp/brief-551/perf/literal-integer-20000.rum < /dev/null
exit=0 bound_seconds=6
Null
# started on Wed Sep 30 03:55:53 2026

22294020,,cpu_core/instructions/u,2838130,100.00,,
```

```text
python3 scripts/run-bounded.py --timeout 6 -- taskset -c 2 perf stat -x, -e cpu_core/instructions/u -o /home/remi/dev/rum/.temp/brief-551/perf/instructions/base/2/literal-integer-40000.csv -- /home/remi/dev/rum/.temp/brief-551/base/rum-cli --file /home/remi/dev/rum/.temp/brief-551/perf/literal-integer-40000.rum < /dev/null
exit=0 bound_seconds=6
Null
# started on Wed Sep 30 03:55:53 2026

40873850,,cpu_core/instructions/u,4943293,100.00,,
```

```text
python3 scripts/run-bounded.py --timeout 6 -- taskset -c 2 perf stat -x, -e cpu_core/instructions/u -o /home/remi/dev/rum/.temp/brief-551/perf/instructions/base/2/symbol-ownvalue-read-20000.csv -- /home/remi/dev/rum/.temp/brief-551/base/rum-cli --file /home/remi/dev/rum/.temp/brief-551/perf/symbol-ownvalue-read-20000.rum < /dev/null
exit=0 bound_seconds=6
Null
# started on Wed Sep 30 03:55:53 2026

47824335,,cpu_core/instructions/u,4252678,100.00,,
```

```text
python3 scripts/run-bounded.py --timeout 6 -- taskset -c 2 perf stat -x, -e cpu_core/instructions/u -o /home/remi/dev/rum/.temp/brief-551/perf/instructions/base/2/symbol-ownvalue-read-40000.csv -- /home/remi/dev/rum/.temp/brief-551/base/rum-cli --file /home/remi/dev/rum/.temp/brief-551/perf/symbol-ownvalue-read-40000.rum < /dev/null
exit=0 bound_seconds=6
Null
# started on Wed Sep 30 03:55:53 2026

91932313,,cpu_core/instructions/u,7532018,100.00,,
```

```text
python3 scripts/run-bounded.py --timeout 6 -- taskset -c 2 perf stat -x, -e cpu_core/instructions/u -o /home/remi/dev/rum/.temp/brief-551/perf/instructions/base/2/set-ownvalue-20000.csv -- /home/remi/dev/rum/.temp/brief-551/base/rum-cli --file /home/remi/dev/rum/.temp/brief-551/perf/set-ownvalue-20000.rum < /dev/null
exit=0 bound_seconds=6
Null
# started on Wed Sep 30 03:55:53 2026

98252251,,cpu_core/instructions/u,8384447,100.00,,
```

```text
python3 scripts/run-bounded.py --timeout 6 -- taskset -c 2 perf stat -x, -e cpu_core/instructions/u -o /home/remi/dev/rum/.temp/brief-551/perf/instructions/base/2/set-ownvalue-40000.csv -- /home/remi/dev/rum/.temp/brief-551/base/rum-cli --file /home/remi/dev/rum/.temp/brief-551/perf/set-ownvalue-40000.rum < /dev/null
exit=0 bound_seconds=6
Null
# started on Wed Sep 30 03:55:53 2026

192789088,,cpu_core/instructions/u,15089331,100.00,,
```

```text
python3 scripts/run-bounded.py --timeout 6 -- taskset -c 2 perf stat -x, -e cpu_core/instructions/u -o /home/remi/dev/rum/.temp/brief-551/perf/instructions/base/2/do-count-null-20000.csv -- /home/remi/dev/rum/.temp/brief-551/base/rum-cli --file /home/remi/dev/rum/.temp/brief-551/perf/do-count-null-20000.rum < /dev/null
exit=0 bound_seconds=6
Null
# started on Wed Sep 30 03:55:53 2026

13657626,,cpu_core/instructions/u,2639935,100.00,,
```

```text
python3 scripts/run-bounded.py --timeout 6 -- taskset -c 2 perf stat -x, -e cpu_core/instructions/u -o /home/remi/dev/rum/.temp/brief-551/perf/instructions/base/2/do-count-null-40000.csv -- /home/remi/dev/rum/.temp/brief-551/base/rum-cli --file /home/remi/dev/rum/.temp/brief-551/perf/do-count-null-40000.rum < /dev/null
exit=0 bound_seconds=6
Null
# started on Wed Sep 30 03:55:53 2026

23614130,,cpu_core/instructions/u,3308115,100.00,,
```

```text
python3 scripts/run-bounded.py --timeout 6 -- taskset -c 2 perf stat -x, -e cpu_core/instructions/u -o /home/remi/dev/rum/.temp/brief-551/perf/instructions/base/2/block-local-20000.csv -- /home/remi/dev/rum/.temp/brief-551/base/rum-cli --file /home/remi/dev/rum/.temp/brief-551/perf/block-local-20000.rum < /dev/null
exit=0 bound_seconds=6
Null
# started on Wed Sep 30 03:55:53 2026

311133100,,cpu_core/instructions/u,22732815,100.00,,
```

```text
python3 scripts/run-bounded.py --timeout 6 -- taskset -c 2 perf stat -x, -e cpu_core/instructions/u -o /home/remi/dev/rum/.temp/brief-551/perf/instructions/base/2/block-local-40000.csv -- /home/remi/dev/rum/.temp/brief-551/base/rum-cli --file /home/remi/dev/rum/.temp/brief-551/perf/block-local-40000.rum < /dev/null
exit=0 bound_seconds=6
Null
# started on Wed Sep 30 03:55:54 2026

618527408,,cpu_core/instructions/u,41577978,100.00,,
```

```text
python3 scripts/run-bounded.py --timeout 6 -- taskset -c 2 perf stat -x, -e cpu_core/instructions/u -o /home/remi/dev/rum/.temp/brief-551/perf/instructions/base/2/table-iterator-null-20000.csv -- /home/remi/dev/rum/.temp/brief-551/base/rum-cli --file /home/remi/dev/rum/.temp/brief-551/perf/table-iterator-null-20000.rum < /dev/null
exit=0 bound_seconds=6
Null
# started on Wed Sep 30 03:55:54 2026

10693125,,cpu_core/instructions/u,3047546,100.00,,
```

```text
python3 scripts/run-bounded.py --timeout 6 -- taskset -c 2 perf stat -x, -e cpu_core/instructions/u -o /home/remi/dev/rum/.temp/brief-551/perf/instructions/base/2/table-iterator-null-40000.csv -- /home/remi/dev/rum/.temp/brief-551/base/rum-cli --file /home/remi/dev/rum/.temp/brief-551/perf/table-iterator-null-40000.rum < /dev/null
exit=0 bound_seconds=6
Null
# started on Wed Sep 30 03:55:54 2026

17674707,,cpu_core/instructions/u,3442190,100.00,,
```

```text
python3 scripts/run-bounded.py --timeout 6 -- taskset -c 2 perf stat -x, -e cpu_core/instructions/u -o /home/remi/dev/rum/.temp/brief-551/perf/instructions/candidate/1/literal-integer-20000.csv -- /home/remi/dev/rum/.temp/brief-551/next/rum-cli --file /home/remi/dev/rum/.temp/brief-551/perf/literal-integer-20000.rum < /dev/null
exit=0 bound_seconds=6
Null
# started on Wed Sep 30 03:55:47 2026

22291778,,cpu_core/instructions/u,6114453,100.00,,
```

```text
python3 scripts/run-bounded.py --timeout 6 -- taskset -c 2 perf stat -x, -e cpu_core/instructions/u -o /home/remi/dev/rum/.temp/brief-551/perf/instructions/candidate/1/literal-integer-40000.csv -- /home/remi/dev/rum/.temp/brief-551/next/rum-cli --file /home/remi/dev/rum/.temp/brief-551/perf/literal-integer-40000.rum < /dev/null
exit=0 bound_seconds=6
Null
# started on Wed Sep 30 03:55:47 2026

40877339,,cpu_core/instructions/u,4094655,100.00,,
```

```text
python3 scripts/run-bounded.py --timeout 6 -- taskset -c 2 perf stat -x, -e cpu_core/instructions/u -o /home/remi/dev/rum/.temp/brief-551/perf/instructions/candidate/1/symbol-ownvalue-read-20000.csv -- /home/remi/dev/rum/.temp/brief-551/next/rum-cli --file /home/remi/dev/rum/.temp/brief-551/perf/symbol-ownvalue-read-20000.rum < /dev/null
exit=0 bound_seconds=6
Null
# started on Wed Sep 30 03:55:47 2026

47825816,,cpu_core/instructions/u,4195859,100.00,,
```

```text
python3 scripts/run-bounded.py --timeout 6 -- taskset -c 2 perf stat -x, -e cpu_core/instructions/u -o /home/remi/dev/rum/.temp/brief-551/perf/instructions/candidate/1/symbol-ownvalue-read-40000.csv -- /home/remi/dev/rum/.temp/brief-551/next/rum-cli --file /home/remi/dev/rum/.temp/brief-551/perf/symbol-ownvalue-read-40000.rum < /dev/null
exit=0 bound_seconds=6
Null
# started on Wed Sep 30 03:55:47 2026

91936273,,cpu_core/instructions/u,7724856,100.00,,
```

```text
python3 scripts/run-bounded.py --timeout 6 -- taskset -c 2 perf stat -x, -e cpu_core/instructions/u -o /home/remi/dev/rum/.temp/brief-551/perf/instructions/candidate/1/set-ownvalue-20000.csv -- /home/remi/dev/rum/.temp/brief-551/next/rum-cli --file /home/remi/dev/rum/.temp/brief-551/perf/set-ownvalue-20000.rum < /dev/null
exit=0 bound_seconds=6
Null
# started on Wed Sep 30 03:55:47 2026

98252671,,cpu_core/instructions/u,8698650,100.00,,
```

```text
python3 scripts/run-bounded.py --timeout 6 -- taskset -c 2 perf stat -x, -e cpu_core/instructions/u -o /home/remi/dev/rum/.temp/brief-551/perf/instructions/candidate/1/set-ownvalue-40000.csv -- /home/remi/dev/rum/.temp/brief-551/next/rum-cli --file /home/remi/dev/rum/.temp/brief-551/perf/set-ownvalue-40000.rum < /dev/null
exit=0 bound_seconds=6
Null
# started on Wed Sep 30 03:55:47 2026

192784529,,cpu_core/instructions/u,16394968,100.00,,
```

```text
python3 scripts/run-bounded.py --timeout 6 -- taskset -c 2 perf stat -x, -e cpu_core/instructions/u -o /home/remi/dev/rum/.temp/brief-551/perf/instructions/candidate/1/do-count-null-20000.csv -- /home/remi/dev/rum/.temp/brief-551/next/rum-cli --file /home/remi/dev/rum/.temp/brief-551/perf/do-count-null-20000.rum < /dev/null
exit=0 bound_seconds=6
Null
# started on Wed Sep 30 03:55:48 2026

13658364,,cpu_core/instructions/u,2824301,100.00,,
```

```text
python3 scripts/run-bounded.py --timeout 6 -- taskset -c 2 perf stat -x, -e cpu_core/instructions/u -o /home/remi/dev/rum/.temp/brief-551/perf/instructions/candidate/1/do-count-null-40000.csv -- /home/remi/dev/rum/.temp/brief-551/next/rum-cli --file /home/remi/dev/rum/.temp/brief-551/perf/do-count-null-40000.rum < /dev/null
exit=0 bound_seconds=6
Null
# started on Wed Sep 30 03:55:48 2026

23615798,,cpu_core/instructions/u,3403886,100.00,,
```

```text
python3 scripts/run-bounded.py --timeout 6 -- taskset -c 2 perf stat -x, -e cpu_core/instructions/u -o /home/remi/dev/rum/.temp/brief-551/perf/instructions/candidate/1/block-local-20000.csv -- /home/remi/dev/rum/.temp/brief-551/next/rum-cli --file /home/remi/dev/rum/.temp/brief-551/perf/block-local-20000.rum < /dev/null
exit=0 bound_seconds=6
Null
# started on Wed Sep 30 03:55:48 2026

311132901,,cpu_core/instructions/u,23244242,100.00,,
```

```text
python3 scripts/run-bounded.py --timeout 6 -- taskset -c 2 perf stat -x, -e cpu_core/instructions/u -o /home/remi/dev/rum/.temp/brief-551/perf/instructions/candidate/1/block-local-40000.csv -- /home/remi/dev/rum/.temp/brief-551/next/rum-cli --file /home/remi/dev/rum/.temp/brief-551/perf/block-local-40000.rum < /dev/null
exit=0 bound_seconds=6
Null
# started on Wed Sep 30 03:55:48 2026

618527251,,cpu_core/instructions/u,42729411,100.00,,
```

```text
python3 scripts/run-bounded.py --timeout 6 -- taskset -c 2 perf stat -x, -e cpu_core/instructions/u -o /home/remi/dev/rum/.temp/brief-551/perf/instructions/candidate/1/table-iterator-null-20000.csv -- /home/remi/dev/rum/.temp/brief-551/next/rum-cli --file /home/remi/dev/rum/.temp/brief-551/perf/table-iterator-null-20000.rum < /dev/null
exit=0 bound_seconds=6
Null
# started on Wed Sep 30 03:55:48 2026

10696774,,cpu_core/instructions/u,2833025,100.00,,
```

```text
python3 scripts/run-bounded.py --timeout 6 -- taskset -c 2 perf stat -x, -e cpu_core/instructions/u -o /home/remi/dev/rum/.temp/brief-551/perf/instructions/candidate/1/table-iterator-null-40000.csv -- /home/remi/dev/rum/.temp/brief-551/next/rum-cli --file /home/remi/dev/rum/.temp/brief-551/perf/table-iterator-null-40000.rum < /dev/null
exit=0 bound_seconds=6
Null
# started on Wed Sep 30 03:55:48 2026

17676420,,cpu_core/instructions/u,3421806,100.00,,
```

```text
python3 scripts/run-bounded.py --timeout 6 -- taskset -c 2 perf stat -x, -e cpu_core/instructions/u -o /home/remi/dev/rum/.temp/brief-551/perf/instructions/candidate/2/literal-integer-20000.csv -- /home/remi/dev/rum/.temp/brief-551/next/rum-cli --file /home/remi/dev/rum/.temp/brief-551/perf/literal-integer-20000.rum < /dev/null
exit=0 bound_seconds=6
Null
# started on Wed Sep 30 03:55:59 2026

22292563,,cpu_core/instructions/u,3329227,100.00,,
```

```text
python3 scripts/run-bounded.py --timeout 6 -- taskset -c 2 perf stat -x, -e cpu_core/instructions/u -o /home/remi/dev/rum/.temp/brief-551/perf/instructions/candidate/2/literal-integer-40000.csv -- /home/remi/dev/rum/.temp/brief-551/next/rum-cli --file /home/remi/dev/rum/.temp/brief-551/perf/literal-integer-40000.rum < /dev/null
exit=0 bound_seconds=6
Null
# started on Wed Sep 30 03:55:59 2026

40875502,,cpu_core/instructions/u,3684064,100.00,,
```

```text
python3 scripts/run-bounded.py --timeout 6 -- taskset -c 2 perf stat -x, -e cpu_core/instructions/u -o /home/remi/dev/rum/.temp/brief-551/perf/instructions/candidate/2/symbol-ownvalue-read-20000.csv -- /home/remi/dev/rum/.temp/brief-551/next/rum-cli --file /home/remi/dev/rum/.temp/brief-551/perf/symbol-ownvalue-read-20000.rum < /dev/null
exit=0 bound_seconds=6
Null
# started on Wed Sep 30 03:55:59 2026

47825657,,cpu_core/instructions/u,5328742,100.00,,
```

```text
python3 scripts/run-bounded.py --timeout 6 -- taskset -c 2 perf stat -x, -e cpu_core/instructions/u -o /home/remi/dev/rum/.temp/brief-551/perf/instructions/candidate/2/symbol-ownvalue-read-40000.csv -- /home/remi/dev/rum/.temp/brief-551/next/rum-cli --file /home/remi/dev/rum/.temp/brief-551/perf/symbol-ownvalue-read-40000.rum < /dev/null
exit=0 bound_seconds=6
Null
# started on Wed Sep 30 03:55:59 2026

91934822,,cpu_core/instructions/u,8219914,100.00,,
```

```text
python3 scripts/run-bounded.py --timeout 6 -- taskset -c 2 perf stat -x, -e cpu_core/instructions/u -o /home/remi/dev/rum/.temp/brief-551/perf/instructions/candidate/2/set-ownvalue-20000.csv -- /home/remi/dev/rum/.temp/brief-551/next/rum-cli --file /home/remi/dev/rum/.temp/brief-551/perf/set-ownvalue-20000.rum < /dev/null
exit=0 bound_seconds=6
Null
# started on Wed Sep 30 03:55:59 2026

98255110,,cpu_core/instructions/u,8546688,100.00,,
```

```text
python3 scripts/run-bounded.py --timeout 6 -- taskset -c 2 perf stat -x, -e cpu_core/instructions/u -o /home/remi/dev/rum/.temp/brief-551/perf/instructions/candidate/2/set-ownvalue-40000.csv -- /home/remi/dev/rum/.temp/brief-551/next/rum-cli --file /home/remi/dev/rum/.temp/brief-551/perf/set-ownvalue-40000.rum < /dev/null
exit=0 bound_seconds=6
Null
# started on Wed Sep 30 03:56:00 2026

192783870,,cpu_core/instructions/u,15404859,100.00,,
```

```text
python3 scripts/run-bounded.py --timeout 6 -- taskset -c 2 perf stat -x, -e cpu_core/instructions/u -o /home/remi/dev/rum/.temp/brief-551/perf/instructions/candidate/2/do-count-null-20000.csv -- /home/remi/dev/rum/.temp/brief-551/next/rum-cli --file /home/remi/dev/rum/.temp/brief-551/perf/do-count-null-20000.rum < /dev/null
exit=0 bound_seconds=6
Null
# started on Wed Sep 30 03:56:00 2026

13660629,,cpu_core/instructions/u,2736597,100.00,,
```

```text
python3 scripts/run-bounded.py --timeout 6 -- taskset -c 2 perf stat -x, -e cpu_core/instructions/u -o /home/remi/dev/rum/.temp/brief-551/perf/instructions/candidate/2/do-count-null-40000.csv -- /home/remi/dev/rum/.temp/brief-551/next/rum-cli --file /home/remi/dev/rum/.temp/brief-551/perf/do-count-null-40000.rum < /dev/null
exit=0 bound_seconds=6
Null
# started on Wed Sep 30 03:56:00 2026

23617250,,cpu_core/instructions/u,3567470,100.00,,
```

```text
python3 scripts/run-bounded.py --timeout 6 -- taskset -c 2 perf stat -x, -e cpu_core/instructions/u -o /home/remi/dev/rum/.temp/brief-551/perf/instructions/candidate/2/block-local-20000.csv -- /home/remi/dev/rum/.temp/brief-551/next/rum-cli --file /home/remi/dev/rum/.temp/brief-551/perf/block-local-20000.rum < /dev/null
exit=0 bound_seconds=6
Null
# started on Wed Sep 30 03:56:00 2026

311133623,,cpu_core/instructions/u,22797131,100.00,,
```

```text
python3 scripts/run-bounded.py --timeout 6 -- taskset -c 2 perf stat -x, -e cpu_core/instructions/u -o /home/remi/dev/rum/.temp/brief-551/perf/instructions/candidate/2/block-local-40000.csv -- /home/remi/dev/rum/.temp/brief-551/next/rum-cli --file /home/remi/dev/rum/.temp/brief-551/perf/block-local-40000.rum < /dev/null
exit=0 bound_seconds=6
Null
# started on Wed Sep 30 03:56:00 2026

618529041,,cpu_core/instructions/u,42682712,100.00,,
```

```text
python3 scripts/run-bounded.py --timeout 6 -- taskset -c 2 perf stat -x, -e cpu_core/instructions/u -o /home/remi/dev/rum/.temp/brief-551/perf/instructions/candidate/2/table-iterator-null-20000.csv -- /home/remi/dev/rum/.temp/brief-551/next/rum-cli --file /home/remi/dev/rum/.temp/brief-551/perf/table-iterator-null-20000.rum < /dev/null
exit=0 bound_seconds=6
Null
# started on Wed Sep 30 03:56:00 2026

10697284,,cpu_core/instructions/u,3421668,100.00,,
```

```text
python3 scripts/run-bounded.py --timeout 6 -- taskset -c 2 perf stat -x, -e cpu_core/instructions/u -o /home/remi/dev/rum/.temp/brief-551/perf/instructions/candidate/2/table-iterator-null-40000.csv -- /home/remi/dev/rum/.temp/brief-551/next/rum-cli --file /home/remi/dev/rum/.temp/brief-551/perf/table-iterator-null-40000.rum < /dev/null
exit=0 bound_seconds=6
Null
# started on Wed Sep 30 03:56:00 2026

17675501,,cpu_core/instructions/u,3443374,100.00,,
```


### 4. Progress, disclosures and limits

All 12 original required reds are green: four runtime file regressions, five
invariant falsifiers and three host regressions. All five original controls
and five extra ownership/root/retirement tests pass. The 20 initial
source-evidence gate failures are resolved. Exact milestones and next actions
are in [.temp/brief-551/PROGRESS.md](../../../.temp/brief-551/PROGRESS.md).
No existing test contract was weakened. Compiler errors, expected stale
certificate failures and one compile overlapping a source edit remain in the
ledger. The stop-loss did not fire: implementation steps advanced red-test
counters; required suites and feature-specific replays validated the same
implementation rather than repeating unsuccessful edits.

The initial inspection ran `pwd && git status --short` before the brief was
loaded, violating its no-git rule. Exit was 0 and status output was empty.
Initial `cat`/`rg` reads were also unbounded; later read-only pipelines bounded
their producer rather than every filter. These are process deviations. No
further git command ran; no commit, branch, push or config operation occurred.
A scratch preservation command initially had a Python syntax error and was
corrected before preservation/build; it changed no source. All build/test/perf
commands used bounded runners. No explicit user-configuration reads, network,
elevation, subagents or new unsafe were used.

All changes remain in the working tree. Dependencies/production lockfiles,
protected root documents, earlier briefs, front ends, guest-loading design,
leases, G1–G8 algorithms and Expr/Symbol/serde representations are unchanged.
Existing tests generated their normal repository-local artifacts; this brief's
own scratch is under `.temp/brief-551/`. Native loopback coverage and optional
timing did not run. Root visitation proves the snapshot edge, not a new
R11/R12 capacity-accounting or lease claim. Positive instruction rows
are reported above. Exact zero instruction cost remains unproven within the
two-sample limit; no design question remains open.
