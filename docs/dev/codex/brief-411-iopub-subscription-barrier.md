# brief-411 — implement: a real IOPub subscription barrier, and parent-scoped frame collection

Shape: **implementation**. Family B, the last of the four brief-406 identified
and the hardest. Families A, C and D have landed (`67988c16`, `ee2a1e33`,
`e8b116da`).

## Where this came from

brief-406 established the mechanism from the pinned dependency's own source:

- `SubSocket::subscribe` stores the filter, queues a subscription message to
  current peers, and returns (`zeromq-0.5.0/src/sub.rs:131-159`).
- The publisher processes that message **later**, in an independently spawned
  coroutine (`zeromq-0.5.0/src/pub.rs:34-72,93-135`).
- PUB sends only through subscription filters already present at send time
  (`zeromq-0.5.0/src/pub.rs:158-200`).

So `subscribe().await` is not publisher acknowledgement, and early IOPub
frames can be dropped before the filter is installed.

The repository's answer to this today is not a barrier at all.
`acknowledge_iopub_subscription` (`zeromq_native_test_support.rs:271-292`)
connects a **Req/Rep** socket and does a heartbeat round trip, then calls the
subscription ready. There is no source edge ordering that heartbeat after the
PUB worker's subscription coroutine. It proves the kernel is alive; it proves
nothing about PUB/SUB.

The second half of the problem is the collection side. Four tests assume the
next frames arriving on their SUB socket belong to their request. The four-frame
sequence itself is genuinely promised — `adapter/iopub.rs:4-17` and
`adapter/mod.rs:481-500` construct it structurally — but *which* frames are
those four is not established by arrival order.

## Read this before anything else

**Nothing is failing.** On this host, measured by me after family C landed:
native suite 0 failures in 30 samples, workspace 2286 passed / 0 failed / 7
ignored with no skip opt-out set. You are removing an invalid mechanism, not
chasing a red test, and "it still passes" is not evidence that the barrier
works — the old one also passed.

**You cannot execute any of this.** Your sandbox cannot bind loopback, so the
native tests will skip. Say so. The socket-free harness tests brief-410 added
(`zeromq_native_test_support.rs:346-407`) are the part you *can* run; extend
them where the new logic admits socket-free coverage. Authoritative native
verification is mine.

## Goal

Subscription readiness is established over the same PUB/SUB path the test then
depends on, and each test collects the frames belonging to its own request
rather than the next frames to arrive.

## The barrier, and the one loop that is allowed

A sound barrier means: **the test has observed a message on its SUB socket
that travelled the backend's real PUB path.** Anything else — a heartbeat, a
sleep, a quiet period — is what we are removing.

The natural construction uses only production paths: issue a distinct,
harmless request whose IOPub output carries a unique parent header, and
consume on the SUB socket until a frame with that parent arrives. If the
subscription is not yet installed, that sentinel's frames are dropped, so the
barrier must be **retried with a fresh unique parent** until one is observed,
bounded by the existing harness stop-loss.

That retry is permitted, and the distinction matters: **retrying to establish
a precondition is legitimate; retrying until an assertion passes is not.** No
assertion in this brief may be retried, loosened, or made conditional on
timing.

If you conclude a sound barrier cannot be built on the test side, **stop and
report it** and name what would be needed. The obvious alternative — switching
the production socket from PUB to XPUB so subscriptions are observable — is a
production socket-type change and a design decision that is mine, not yours.
Do not make it.

## Collection

Replace "the next four frames" with: consume frames, discard any whose parent
is not the request under test, and stop at that parent's terminal Idle. Then
assert that parent's exact sequence and count. A late duplicate sentinel must
be discarded by the same rule rather than counted.

## Context pointers

- Read `AGENTS.md`, brief-406's committed report (its native table and the
  pinned-dependency analysis), and briefs 408-410's committed reports.
- The pseudo-barrier and its four callers:
  `zeromq_native_test_support.rs:271-292`, called from `native/signed.rs`,
  `native/frontend/execute.rs`, `native/frontend/stdin.rs`,
  `native/frontend/session_flow.rs`.
- Structural IOPub sequence: `adapter/iopub.rs:4-17`, `adapter/mod.rs:481-500`.
- Stage-labelled waits and the 30-second harness stop-loss from brief-409 live
  in `zeromq_native_test_support.rs`; use them, do not add new bounds.
- The skip gate `RUM_ALLOW_NATIVE_ZEROMQ_SKIP` (brief-408) and the
  resolved-endpoint propagation must both keep working.

## Scope

- Files you MAY modify, all under `crates/rum-jupyter/src/`:
  - `zeromq_native_test_support.rs`;
  - `zeromq/tests/native/signed.rs`;
  - `zeromq/tests/native/frontend/execute.rs`, `stdin.rs`, `session_flow.rs`;
  - `zeromq/tests/native/basic.rs` and `frontend/control.rs` only if a shared
    helper signature changes — say so if it does.
- Files/areas you MUST NOT touch:
  - **all production code**, including the PUB socket type, `zeromq.rs`,
    `zeromq_native/**`, `adapter/**`, `transport/**`;
  - `crates/rum-jupyter/src/tests/**` (family A);
  - the wire protocol, message schema, pinned `zeromq` version, `Cargo.lock`,
    dependency edges, other crates.
- If you need a production file, that is the XPUB-shaped decision above:
  **STOP and report**, do not proceed.

## The accept-vs-stop criterion

Replacing the pseudo-barrier with a real one, and arrival order with parent
identity, is in scope. **Weakening or deleting a semantic assertion is a
stop.** The exact IOPub sequences and counts, signed-message coverage, stdin
routing and resume, execution counts, and session-flow state all survive
unchanged. Adding a sleep, lengthening a bound, or making an assertion
timing-conditional is a stop. Touching production is a stop.

## Deliverable

1. A barrier that observes a message on the SUB socket which travelled the
   backend's real PUB path, replacing `acknowledge_iopub_subscription` at all
   four call sites.
2. Sentinel traffic identified by unique parent or message ID and excluded from
   every target assertion, including a late duplicate.
3. Parent-scoped collection terminating at that parent's Idle, replacing
   next-N-frames assumptions in all four tests.
4. Stage-labelled diagnostics that distinguish barrier establishment, request
   delivery, worker completion, Shell reply, and IOPub collection.
5. The heartbeat retains its own independent coverage but is no longer called
   subscription acknowledgement anywhere, including in comments.
6. A per-test accounting: mechanism removed, assertions preserved, zero
   deletions.
7. Socket-free coverage extended wherever the new logic permits it — for
   instance the parent-filtering and sentinel-exclusion rules are pure
   functions of message headers and can be tested without a socket. **That is
   your only real gate; treat it as such.**

## rum / Rust pitfalls (read twice)

1. A barrier that sends its sentinel through anything other than the backend's
   own PUB path proves nothing; a second PUB socket created by the test is not
   the same path.
2. The sentinel is a real request, so it advances execution count and session
   state. Account for that in every count assertion, or choose a request shape
   that does not.
3. Discard by parent identity, not by position or by content matching, or a
   late frame will silently shift a sequence.
4. Do not add a new duration; the brief-409 stop-loss already bounds every
   wait.
5. Keep the tampered-signature and shutdown structural assertions from
   brief-409 exactly as they are.
6. No new `unsafe`. No production change of any kind.

## Verification (MANDATORY — you run what you can, honestly)

Pass `--offline` to every `cargo` command; you will need
`RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`, which means the native paths stay unexecuted
by you.

```
scripts/run-bounded.py --timeout 300  -- cargo fmt --check
scripts/run-bounded.py --timeout 600  -- cargo test --offline -p rum-jupyter
scripts/run-bounded.py --timeout 2400 -- cargo test --offline --workspace --no-fail-fast
```

Paste the socket-free tests green, including any you added.

**Structural evidence, since you cannot measure:**
- the barrier's causal chain as `path:line`, from the sentinel's publication
  through the backend's PUB socket to the test's SUB observation;
- the parent-filtering rule, and why a late duplicate cannot be counted;
- confirmation that no production file changed, and that the skip gate and
  resolved-endpoint propagation still work.

**State plainly that you executed no native socket path.** My baselines: native
0 failures in 30 samples, workspace 2286 passed / 0 failed / 7 ignored, no skip
opt-out set. Anything short of that is a stop.

## Hard rules

- Do NOT `git commit`, branch, push, checkout, stash, or touch git config. Do
  not touch `stash@{0}`. Leave changes in the working tree.
- Modify only Scope. If you must go outside it, STOP and write that in the
  report. Do not work around a missing entry.
- Touch nothing outside this repo except `/tmp` scratch you remove afterwards.
  No network. No probe artifacts in the repo.
- **Never report a rate for a test you did not observe running.**
- **Testing Policy is a hard line**: rum's own documented behavior only.
- **No new `unsafe`.** No test deleted, no assertion weakened, no earlier
  brief's work reverted.
- Paste genuine verbatim output. If a run did not reach green, say so.

## Report format

1. Per-file change with line ranges and a one-line why.
2. The barrier's causal chain as `path:line`.
3. The parent-filtering and sentinel-exclusion rules, and the late-duplicate
   argument.
4. The per-test accounting.
5. Socket-free test output, verbatim.
6. Verification output, with an explicit statement of what you could not run.
7. Stop-loss, risks, open questions.
