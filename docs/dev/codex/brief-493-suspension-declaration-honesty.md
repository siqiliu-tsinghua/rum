# brief-493 — design-only: can a suspension frame honestly declare bounded destruction?

**Shape: DESIGN-ONLY. `Scope: NONE — no source change.`** The deliverable is a
recommendation with its argument, and drafted follow-up briefs in
`TEMPLATE.md` shape. You write no production or test code.

This is step D of the host-payload destruction contract (plan brief-480),
stopped before dispatch because the director believes it cannot be executed
honestly as planned. Settle that before anything is written.

## The finding that stopped it

`BoundedDestruction`'s documented promise
(`crates/rum-runtime/src/runtime/host_payload.rs`) is a bound on native stack
consumption while destroying the value **and any state it can reach** — its own
`Drop`, compiler-generated field destruction, captured values, and the
last-strong-holder case.

But:

- **`Expr`'s destruction is recorded as unbounded standing debt.** `TODO.md`
  §Standing engineering debt (around `:1197`): deep `Expr` trees built through
  the Rust API can overflow the native stack on drop; source-created trees are
  depth-guarded; **three iterative `Drop` designs were rejected on a measured
  20-50% hot-path regression**. brief-476's inventory records it as
  `Unbounded; owed_by = standing engineering debt`, alongside `SharedExpr` and
  `SharedArgs`.
- **Frames own it.** A director scan finds **266 of 535 `SuspensionFrame`
  implementor structs carry an `Expr`-family field directly**, and the
  remainder largely reach one through their state types. `FrameAction::Complete`
  carries `Result<Expr, EvalError>` in the protocol itself
  (`runtime/suspension/core.rs:93`).

So making `BoundedDestruction` a supertrait of `SuspensionFrame` and
`PureFrameFinalizer` would require ~598 implementors to assert something the
project has explicitly recorded as false. That is a rubber stamp at scale, and
this line has rejected smaller versions of it twice.

**Confirm or refute this from source before recommending anything.** If the
director's reading of the marker's promise, of the `Expr` debt, or of the
frames' ownership is wrong, say so — a refuted premise is the most valuable
thing this brief can return.

## The question, and the options to weigh

What is the correct treatment of the suspension boundary? At least these:

1. **Refine the marker's meaning** to "bounded except for payloads with an
   explicitly recorded, named standing debt", and declare the frames under that
   reading. Cheapest to reach, but it **changes what steps A, B and C already
   signed** (`8ff4439a`, `ce084977`, `6534de2e`) — say exactly how their
   declarations read under the refinement and whether they stay true.
2. **Do not give frames the marker at all.** Record the two suspension traits
   in the ownership ledger as `Unbounded`, `owed_by` the `Expr` debt, which is
   brief-476's own treatment of `Expr`-carrying types and its stated shape for
   exemptions — "exemptions from remediation, not from discovery or
   classification". The original six owned storage edges then resolve as three
   host boundaries closed (A, B, C) and three rum-internal rows carrying a
   named debt.
3. **A second, narrower marker** for what the accumulator work actually
   achieved — the implementor's *own* structure destroys iteratively — leaving
   payload debt to the ledger. Two markers is a cost in clarity; say whether it
   buys enough.
4. **Bound `Expr`** — closed, on measurement. Do not propose it; name it only
   to say it is closed.

For each option: what becomes true, what becomes enforceable, what a reader of
the declaration would correctly conclude, and what the ledger row looks like
afterwards.

## What the recommendation must also settle

- **Did the accumulator-chain work (briefs 485-492) close anything at the frame
  boundary at all?** Say plainly what it did close — the chain members' own
  destruction — and what it did not.
- **Does `ClosurePrintSink`'s declaration have the same problem?** Its
  constructor asserts about the closure's captured state, and a host closure
  can capture an `Expr`. If step C's declaration is unsound under the strict
  reading, that is a finding about committed work and must be said.
- **What, if anything, should the 539 and 59 implementors do?** If the answer
  is "nothing", say so and explain what a future implementor is then obliged to
  think about, and what tells them.
- **What unblocks step E?** The ledger's strict check is the last step of this
  line; say what this decision hands it.

## Scope

- **Files you MAY modify: NONE.** The report is the deliverable, with the
  drafted briefs inside it — do not create files under `docs/dev/codex/`.
- Do not run any workload that reproduces a crash, an overflow or a process
  termination. Source reading plus, at most, `cargo check`/`cargo test` to
  confirm what compiles today.

## Hard rules

- Do NOT `git commit`, branch, push, touch git config, or run `git stash` /
  `git checkout`.
- Do NOT modify any file. If a change seems needed to answer a question,
  describe it instead.
- Touch nothing outside this repo; no network.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **No new `unsafe`** in any drafted brief.
- **Cost is not an argument** (user ruling 2026-08-27): 598 implementors
  describes the work, it does not rank an option down. But **a declaration that
  is not true is not made acceptable by being cheap**, and a reclassification
  into an unenforced contract is a deferral wearing a repair's clothes — this
  project has been burned by both.
- Recommend the most correct option, not the one that lets the line close
  fastest. "This line should stop here and record a debt" is a legitimate
  recommendation if it is the true one.
- Paste verbatim output for anything you run; if you did not run something,
  say so.

## Report format

1. "Design-only, no source change", and whether the director's premise holds —
   the marker's promise, the `Expr` debt, and the frames' ownership — each with
   `path:line`.
2. Any command you ran, verbatim.
3. The options, the recommendation with its argument, and the drafted briefs.
4. Your answers on step C's soundness, on what briefs 485-492 closed, on what
   the implementors owe, and on what step E inherits.
5. Stop-loss / open questions, separated from the recommendation.
