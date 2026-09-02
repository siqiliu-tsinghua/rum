# brief-406 — diagnose: the rum-jupyter test suite's timing assumptions

Shape: **diagnostic**. `Scope: NONE — no source change.` The deliverable is
evidence plus drafted implementation briefs, not a fix.

This is deliberately not an implementation brief even though one of the two
halves is already diagnosed. brief-388 closed the two instances that had been
seen rather than the class, because diagnosis and repair both looked only at
recorded sightings. This suite has now produced **seven distinct failing
tests** across two subsystems in one session. Fixing the two I understand
would repeat that mistake.

## Read this first: your sandbox may not be able to measure half of this

`TODO.md` records that the native zeromq tests are **skipped in the worker's
sandbox** — "loopback socket binding is not permitted" — and that the
hypothesis about them was reached by reading the dependency, not by
reproduction. It also records an inconsistency worth taking seriously: during
brief-384 the same worker reported those tests *running* rather than skipping,
which means a 0-of-30 result from that run may have measured nothing at all.

**Therefore your first action, before anything else, is to establish and report
what your sandbox actually does.** Run the native zeromq tests once and paste
the verbatim output showing whether each ran, passed, failed, or skipped. If
they skip, say so plainly and **do not report any rate, count, or stability
claim about them** — analyze them by reading only, and label every such
conclusion as derived from source rather than observed. I will do the
measurement on the host where they demonstrably run.

That instruction is the point of this brief as much as the diagnosis is. An
instrument that silently measures nothing has cost this project a false
result before.

## The evidence I already have

Measured by me on this host, where these tests demonstrably do run.

Whole-suite `cargo test -p rum-jupyter`, sampled repeatedly across four
different trees during briefs 402-405:

| Tree | Samples | Whole-suite runs with ≥1 failure |
|---|---|---|
| `4abdd19f` (before this line) | 10 | 1 |
| after brief-402/403 | 10 | 3 |
| after brief-404 | 30 | 5 |

The failing test **moves between runs**. Every distinct failure seen this
session:

| Test | Note |
|---|---|
| `tests::pump::control_interrupt_cancels_running_execute_and_deferred_shell_runs_after_idle` | brief-397 finding B. **Fixed by brief-404** — 0 occurrences in 30 samples afterwards, against a prior rate near 2 in 10. Not your subject; listed so the family is complete. |
| `tests::pump::event_pump_preserves_routing_ids_on_replies_and_routes_iopub_separately` | brief-397 finding A. `pump.rs:37`, `assert!(pump.pump_execute_completion(...))`. **Fails 30/30 when run in isolation**, yet usually passes inside the whole suite. In-process transport mock, no sockets. |
| `zeromq::tests::native::frontend::stdin::…_stdin_round_trip_emits_iopub_and_shell_reply` | 0/30 in isolation |
| `zeromq::tests::native::signed::…_rejects_tampered_signed_requests_end_to_end` | |
| `zeromq::tests::native::basic::…_round_trips_execute_over_loopback_sockets` | |
| `zeromq::tests::native::basic::…_echoes_heartbeat_over_loopback_req_socket` | |
| `zeromq::tests::native::frontend::session_flow::…_multi_message_session_flow_tracks_counts_and_shutdown` | 0/20 in isolation |

Two things in that table need explaining and neither is explained today:

1. **Finding A is inverted.** It fails deterministically alone and usually
   passes in company. That is the opposite of an ordinary flake and says the
   suite is supplying something the test needs but does not establish.
2. **`TODO.md`'s existing hypothesis does not cover the observed set.** It
   names four files carrying a fixed 50 ms PUB/SUB propagation delay —
   `native/signed.rs:24`, `frontend/execute.rs:24`, `frontend/stdin.rs:39`,
   `frontend/session_flow.rs:48` — but two of the failures I recorded are in
   `basic`, which is not on that list. Either the class is larger than the
   entry says, or there is a second cause.

## The question this brief must answer

**What does each of these tests assume about timing or ordering that the code
under test does not promise?** Not "why did this run fail" — that is a
symptom. For each failure, decide between:

- the test asserts a property the kernel or poller genuinely guarantees, so
  the failure is a real defect; or
- the test asserts a timing property where the implementation promises
  something weaker or different — a priority, an eventual delivery, a
  cooperative check — so the test is wrong.

brief-397 already made that distinction once for finding A and concluded the
immediate-readiness assumption is invalid. Confirm or refute it against the
current tree, then extend the same judgement to the rest.

## Sweep the class, not the sightings

The list above is what happened to fail while I was watching something else.
It is a sample, not the population. **Enumerate every test in
`crates/rum-jupyter` that depends on a timing or ordering property**, whether
or not it has ever failed: fixed sleeps, bounded receive loops, "already
ready" assertions, assumptions about how many messages have arrived,
assumptions about the order of two channels, and any assertion whose truth
depends on a thread having made progress. Give each a row: `path:line`, the
property it assumes, whether the implementation promises it, and whether it
has been observed failing.

That inventory is the primary deliverable. If it turns out that only the seven
above qualify, say so — but derive it, do not assume it.

## What else to establish

- **Why finding A inverts.** What does the whole suite provide that isolation
  does not? Name the mechanism with `path:line`, do not speculate.
- Whether the pump failures and the zeromq failures share a root or are two
  families that merely co-occur.
- Whether the pinned `zeromq` 0.5.0 subscription behaviour described in
  `TODO.md` is the actual mechanism for the `basic` tests too, or whether those
  need a separate explanation.
- Whether any of these tests, if written correctly, would have caught a real
  kernel defect — I want to know what coverage is lost if a test is simply
  deleted rather than repaired.

## Deliverable

1. The sandbox capability statement, first, verbatim.
2. The class inventory table described above.
3. A verdict per test: real defect, or invalid timing assumption, with the
   `path:line` evidence for what the implementation actually promises.
4. The finding-A inversion mechanism.
5. **Drafted implementation briefs** in `TEMPLATE.md` shape, one per repair
   family rather than one per test, each with its own scope, gate, and
   accept-vs-stop criterion. They are raw material for me to rewrite; write
   them as if they were to be dispatched.
6. A statement of what `TODO.md:707-724` gets right and what it misses, since
   my data already contradicts part of it.

## Hard rules

- **No source change.** `Scope: NONE`. You may read anything in the repository
  and run tests; you may not edit a single file, including tests and docs.
- Do NOT `git commit`, branch, push, checkout, stash, or touch git config. Do
  not touch `stash@{0}`.
- Touch nothing outside this repo except `/tmp` scratch you remove afterwards.
  No network. Pass `--offline` to every `cargo` command.
- **Never report a rate for a test you did not observe running.** If it
  skipped, say it skipped. This is the brief's central instruction.
- **Testing Policy is a hard line**: rum's own documented behavior only. No
  vendor engine, oracle harness, or compatibility matrix.
- Every claim carries `path:line` evidence. Say plainly where you are
  reasoning from source rather than from observation.
- Paste genuine verbatim output. If a run did not produce evidence, say so.

## Report format

1. Sandbox capability, verbatim.
2. The class inventory table.
3. Per-test verdicts with evidence.
4. The finding-A inversion mechanism.
5. The drafted implementation briefs.
6. What `TODO.md` gets right and wrong.
7. Stop-loss, risks, open questions.
