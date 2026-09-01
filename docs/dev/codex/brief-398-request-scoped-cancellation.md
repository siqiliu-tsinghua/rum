# brief-398 — design: request-scoped cancellation at the host boundary

**Scope: NONE — design and plan only, no source change.** Reading, grepping and
throwaway probes are in scope; revert any temporary instrumentation before
reporting and name it. Plan-first is the standing rule for cross-cutting work,
and this is cross-cutting: it touches the runtime's interrupt flag, the host's
public cancel handle, a documented contract test, and every embedder.

**Write your findings and design first, before running any verification
commands.**

## The defect, already established

brief-397 diagnosed it and I verified the mechanism from source myself. Three
facts, all confirmed:

- `crates/rum-jupyter/src/adapter/worker.rs:87-88` — `spawn_eval` obtains
  `session.cancel_handle()` *before* it spawns the worker thread, so the handle
  is live and callable the instant the worker struct exists.
- `crates/rum-runtime/src/runtime/session/session_requests.rs:112` —
  `eval_request`'s first statement is `begin_top_level_interrupt_budget()`.
- `crates/rum-runtime/src/runtime/session/session_helpers.rs:31` — that
  function's first line is `self.interrupt_flag.store(false, Ordering::Relaxed)`.

So a cancel that lands after the handle exists but before the worker's
evaluation prologue runs is **erased**, and nothing re-delivers it. brief-397
made this switchable in both directions with forced ordering (5/5 failures with
the worker delayed, 5/5 passes with the parent delayed), and traced
`begin_top_level_interrupt_budget clears prior=true` on the failing side.

The user-visible consequence is not a test artifact: a Jupyter frontend can
send `interrupt_request`, receive an acknowledgement, and the accepted execute
runs to completion anyway.

## The ruling — this is decided, do not re-open it

brief-397 proposed repairing this inside the `rum-jupyter` adapter, by latching
`interrupted` and reasserting the cancel handle on each completion poll.
**That is rejected: it fixes at the wrong layer.** The mechanism lives in
`rum-runtime` (an unconditional clear) and `rum-host` (a handle that is live
before the evaluation it targets has started). Any embedder that cancels from
another thread inside that window loses the cancel; `rum-jupyter` is merely the
first consumer to reach it. Patching one adapter leaves the hole open for every
other embedder of a project whose stated product is an embeddable, sandbox-first
host contract.

The user's ruling (2026-09-01) invoked the project's two standing principles
directly: **pick the best and most correct option, cost arguments do not count**,
and **a fix the compiler or the type system enforces beats one that relies on
people remembering**. Design accordingly. Do not return a variant of the
adapter latch, and do not argue for it on effort grounds.

## The real constraint you must satisfy

The clear is deliberate, not an oversight. `rum-host`'s contract test requires
that a cancel issued *before* an evaluation is a **no-op for a later
evaluation** (`crates/rum-host/tests/lifecycle_contract.rs:107-117`), so a stale
cancel from a finished request cannot kill an unrelated later one. That intent
must survive.

So the design problem is exactly: **distinguish a stale cancel from a cancel
aimed at the request that is starting right now.** A cancel that names no
request cannot be told apart from one that names a finished one, which is why
the current code can only choose between two wrong answers. The obvious shape
is to give cancellation a request identity — a generation, epoch, or token
minted when a request is accepted and carried by the handle — but the shape is
yours to design and justify; I am naming the problem, not the solution.

Whatever you choose, state precisely:

1. **Where the identity is minted, and by whom.** The window opens when the
   embedder takes a handle for a request that has not yet begun evaluating, so
   the identity has to exist before `eval_request` runs. Say what creates it and
   what happens to a handle taken outside any request.
2. **What the new documented contract sentence is**, replacing the current
   "a cancel before an evaluation is a no-op". It must still make a stale cancel
   inert, and it must make a cancel for an accepted-but-unstarted request
   effective. Write the sentence.
3. **How it is enforced structurally rather than by convention.** A rule that a
   future caller must remember is not acceptable here; prefer a typed handle
   that cannot be constructed without an identity, in the spirit of the
   `EvaluatedPartSelectors` and `TrustedSystemDownValueLhs` repairs.
4. **What `interrupt_armed` does.** It is computed in the same prologue
   (`session_helpers.rs:31-39`) from the deadline and the flag's strong count.
   Say whether the arming decision has the same window, and whether the elapsed
   timeout shares the defect.

## What to inventory

- **Every construction and use of a cancel handle**, across `rum-runtime`,
  `rum-host`, `rum-jupyter`, `rum-cli`, `rum-repl` and the tests, with
  `path:line` and a one-line role. Include `spawn_resume`
  (`crates/rum-jupyter/src/adapter/worker.rs:98-105`), which brief-397 reports
  has the identical startup window, and say whether resume needs the same
  identity or a different one.
- **Every test that synchronizes a cross-thread cancel with a sleep or a print.**
  `TODO.md` already records four sites the consistency review flagged as this
  exact shape and never observed failing
  (`crates/rum-host/tests/lifecycle_contract.rs:118`,
  `crates/rum-host/src/tests/eval_contract.rs:210`,
  `crates/rum-host/tests/stress.rs:52`,
  `crates/rum-runtime/src/runtime/tests/session_requests/api.rs:502`).
  For each, say whether the new design lets it become deterministic by
  construction, and whether it was passing for the right reason before.
- **Every place the interrupt flag is stored, cleared, or read**, so the plan
  can state which of them the identity has to reach.

## Required properties of the repair you plan

- A cancel for the current request is never lost, whether it arrives before the
  evaluation prologue, during evaluation, or between a suspension and its
  resume.
- A cancel from a finished or unrelated request never affects a later one.
- No new unconditional cost on the evaluation hot path. The prologue runs on
  every top-level request; whatever you add there must be a couple of stores,
  not an allocation or a lock. If the design cannot avoid hot-path cost, say so
  and quantify it — that becomes a microbench-gated question.
- No new `unsafe`.
- The determinism test must hold **by construction** — a worker-start gate,
  barrier, or channel handshake — never a sleep. brief-397's own repro used a
  100 ms sleep as a probe; that is a probe, not an acceptable test.

## Deliverable

- The design, with the four numbered answers above.
- The three inventories.
- A step-by-step plan, one step per independently gateable change, each naming
  files, functions, its gate test, and what it must not break. Say explicitly
  which step first makes the failing ordering deterministic, because that test
  is the acceptance evidence for the whole line.
- **Drafts of the follow-up implementation briefs**, in `TEMPLATE.md` shape,
  one per step — full drafts, not step descriptions.
- Risks, open questions, and any correction to the background above.

## Explicitly out of scope

- Finding A from brief-397 (the immediate-completion assertion in
  `tests/pump.rs:37`). It is independent and gets its own brief; do not repair
  or plan it here.
- The `rum-jupyter` native-ZeroMQ skip mechanism.
- Any change to the `Do` driver or the evaluator's amortized poll path.
  brief-397 established the absence of a `Do`-driver poll is not what loses this
  request.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- **No source change.** Temporary instrumentation only if reverted before
  reporting and named; verify with `git status` and paste it.
- Do not modify `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**` outside this
  file, or anything under `docs/dev/codex/` except appending your report.
- Leave no probe files in the repository.
- **Testing Policy is a hard line** (`AGENTS.md`).
- Paste verbatim output for every claim; if a run did not produce the evidence,
  say so.

## Report format

1. The design, answering the four numbered questions.
2. The three inventories.
3. The plan, step by step.
4. The drafted implementation briefs.
5. Stop-loss, risks, open questions, and corrections to my background.
