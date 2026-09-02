# brief-409 — implement: condition-driven native progress, and structural negative assertions

Shape: **implementation**. Family C of the four brief-406 identified, taken
third. Family B — the IOPub subscription barrier — is next and will build on
the helpers you leave behind.

## Read this before anything else: there is nothing failing

brief-407 (family A) and brief-408 (family D) landed. On this host, measured
by me:

- the whole `rum-jupyter` suite: **0 failures in 20 samples**;
- the native suite alone: **0 failures in 25 samples**;
- the workspace gate: 2285 passed, 0 failed, with no skip opt-out set, so a
  silent native skip would have failed it.

**So this brief is not fixing red tests.** It is removing assertions that are
invalid regardless of whether they currently pass. A test that passes because
a machine happened to be fast is not verifying anything; it is supplying false
confidence and will fire on different hardware or under load. That is the
whole justification, and it is sufficient — but it also means **"the failure
went away" is not available as evidence.** Your gate is structural.

Do not "fix" anything by making a wait longer. Do not add a retry. If you
find yourself tuning a duration, you have left this brief's premise.

## Where this came from

brief-406's class inventory graded every timing-dependent test in the crate.
Family C is the native, non-IOPub half: Dealer/Router, Req/Rep, Control, and
the signature-rejection path. Its shapes:

- **Fixed connection sleeps.** `signed.rs:158,184` and `control.rs:25,115`
  each sleep 50 ms and then treat the connection as ready.
- **Elapsed time as a pass condition.** `basic.rs:59,117`, `signed.rs:85,111`,
  and the shared loops at `zeromq_native_test_support.rs:104,117-129,137-149,157-169`
  turn "within two seconds" into a semantic requirement. The native receive
  and evaluator workers are asynchronous
  (`zeromq_native/threading.rs`, `adapter/worker.rs`); no duration is promised.
- **Proof of absence by waiting.** `signed.rs:203` and `control.rs:222` assert
  that no reply arrives within 200 ms.

## The part that matters most

**You cannot prove absence by waiting.** A 200 ms window that sees no reply
says the reply had not arrived yet; it does not say the dispatch never
happened. Both of those assertions must become **structural**:

- After `pump_once` returns `InvalidSignature`, assert from state that no
  dispatch was produced and no reply was enqueued — the backend's own
  bookkeeping already knows this. Do not wait to find out.
- After a shutdown is observed, assert that the Shell request is still pending
  and unprocessed, rather than that a reply did not arrive in 200 ms.

If the state needed to make either assertion is not reachable from the test,
say so and stop — do not substitute a longer window.

## Goal

Every stage in the native non-IOPub tests advances on an observed condition
with a stage-specific diagnostic; the two negative assertions become
structural; and no surviving duration is a pass condition.

## Context pointers

- Read `AGENTS.md` and **brief-406's committed report**, especially the native
  table and its verdicts for `basic`, `signed`, and `frontend/control`.
- brief-406 established that `basic` is a third family: its setup contains no
  `SubSocket` and no IOPub endpoint, so the PUB/SUB subscription race is **not**
  its mechanism. Do not import family B's explanation into it.
- Shared helpers: `zeromq_native_test_support.rs:100-175`.
- Fixed sleeps: `signed.rs:158,184`, `control.rs:25,115`.
- Negative windows: `signed.rs:203`, `control.rs:222`.
- The Control-priority test's explicit both-channels-pending precondition
  (`control.rs`, the shutdown test) is genuine and promised by
  `zeromq_native/socket.rs`; keep it.
- brief-408 just landed resolved-endpoint propagation and the skip gate in
  these same files. Do not undo either; `RUM_ALLOW_NATIVE_ZEROMQ_SKIP` must
  keep working exactly as it does now.
- **Your sandbox cannot bind loopback.** These tests will refuse to skip
  unless you set `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`, which is the correct thing
  for you to do — and it means you cannot execute a single line of what you
  are changing. Say so. Authoritative verification is mine.

## Scope

- Files you MAY modify, all under `crates/rum-jupyter/src/`:
  - `zeromq_native_test_support.rs`;
  - `zeromq/tests/native/basic.rs`;
  - `zeromq/tests/native/signed.rs`;
  - `zeromq/tests/native/frontend/control.rs`;
  - `zeromq/tests/native/frontend/{execute,stdin,session_flow}.rs` **only** to
    keep them compiling against a changed shared helper signature — not to
    touch their own timing assumptions, which are family B. State every such
    edit explicitly.
  - production files under `zeromq_native/` or `zeromq.rs` **only** if a
    structural negative assertion genuinely needs state the test cannot reach;
    if so, keep it minimal, add no default-path cost, and call it out
    prominently in the report.
- Files/areas you MUST NOT touch:
  - `crates/rum-jupyter/src/tests/**` and `adapter/**` — family A;
  - the wire protocol, message schema, public serde shapes, pinned `zeromq`
    version, `Cargo.lock`, dependency edges;
  - `rum-runtime`, `rum-host`, `rum-stdlib`, `rum-core`.
- If you need a file outside this list, **STOP and report**. My MAY lists have
  been short eight times on this project.

## The accept-vs-stop criterion

Replacing a duration with an observed condition is in scope. **Weakening or
deleting a semantic assertion is a stop**, and so is lengthening a wait,
adding a retry, or making an assertion conditional on timing. Signature
rejection, Control-before-Shell priority, heartbeat echo, execute round-trip,
shutdown observation and pending-Shell reporting all survive unchanged.

If a structural negative assertion needs production state that is not exposed,
**stop and report it** rather than reaching for a window.

## Deliverable

1. The four 50 ms connection sleeps are gone, replaced by an observed
   readiness condition on the same socket path the test then uses.
2. `basic`, `signed`, and `control` advance stage by stage on conditions, each
   with a diagnostic that **names the stage that failed** — connection,
   request delivery, worker completion, reply, or absence.
3. The two negative assertions are structural, as described above.
4. **One harness stop-loss, and it is not a pass condition.** A single
   generous bound may remain solely to turn a hang into a failure. Its panic
   message must say it is a harness timeout and not a latency contract, and it
   must not be per-stage tuned. **Do not claim to have derived it from
   measured data** — you cannot run these tests. Pick something plainly
   generous and say that is what you did.
5. `RUM_ALLOW_NATIVE_ZEROMQ_SKIP` and the resolved-endpoint propagation from
   brief-408 are untouched and still work.
6. A per-test accounting: for each test, the mechanism removed and the semantic
   assertions preserved. Zero deletions.

## rum / Rust pitfalls (read twice)

1. A condition wait that spins is still a spin. Prefer an actual signal or a
   state read; if the only available mechanism is polling a socket, keep it
   bounded by the single harness stop-loss and say so.
2. `basic` has no SUB socket. Its readiness question is Dealer/Router and
   Req/Rep connection establishment, not subscription.
3. Do not conflate "the backend observed the request" with "the worker
   finished"; brief-406's inventory separates those and so should the
   diagnostics.
4. The signature-rejection path must still be end-to-end: keep the tampered
   message going over a real socket, not short-circuited into the codec.
5. No new `unsafe`. No dependency change. No wire-format change.
6. Do not let a structural assertion read state that only exists in test
   builds unless it is `#[cfg(test)]` and adds no production cost — and say so
   if you do.

## Verification (MANDATORY — you run what you can, honestly)

Pass `--offline` to every `cargo` command. You will need
`RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`; that is expected and correct for your
sandbox, and it means the native tests early-return without executing.

```
scripts/run-bounded.py --timeout 300  -- cargo fmt --check
scripts/run-bounded.py --timeout 600  -- cargo test --offline -p rum-jupyter
scripts/run-bounded.py --timeout 2400 -- cargo test --offline --workspace --no-fail-fast
```

**Structural evidence, since you cannot measure:**

- a search over your four files showing every remaining `Duration`,
  `sleep`, and `timeout`, with a one-line justification for each survivor;
- the two negative assertions, quoted before and after, with the state they
  now read and the `path:line` where that state is produced;
- confirmation that the skip gate still fires both ways.

**State plainly that you executed none of the native paths you changed.** I
will run the native suite repeatedly on the host where it binds; my baseline
is 0 failures in 25 native-only samples and 0 in 20 whole-suite samples, so a
regression will be visible.

No timing A/B. If you change ordinary evaluation or dispatch, the premise is
exceeded — stop and report.

## Hard rules

- Do NOT `git commit`, branch, push, checkout, stash, or touch git config. Do
  not touch `stash@{0}`. Leave changes in the working tree.
- Modify only Scope. If you must go outside it, STOP and write that in the
  report. Do not work around a missing entry.
- Touch nothing outside this repo except `/tmp` scratch you remove afterwards.
  No network. No probe artifacts in the repo.
- **Never report a rate for a test you did not observe running.**
- **Testing Policy is a hard line**: rum's own documented behavior only.
- **No new `unsafe`.** No test deleted, no assertion weakened.
- Minimal diff; no drive-by reformatting or renames.
- Paste genuine verbatim output. If a run did not reach green, say so.

## Report format

1. Per-file change with line ranges and a one-line why.
2. The two negative assertions, before and after, with the state each now
   reads and where it is produced.
3. The per-test accounting from Deliverable 6.
4. Every surviving duration, with its justification.
5. Verification output, with an explicit statement of what you could not run.
6. Any production file touched, and why it was unavoidable.
7. Stop-loss, risks, open questions.
