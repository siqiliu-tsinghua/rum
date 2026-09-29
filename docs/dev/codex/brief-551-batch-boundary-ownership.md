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
