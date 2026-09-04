# brief-445 — make the stability walk fail closed, now that the frame fits

**Shape: implementation.** One variable: a strided native-stack check inside the
stability walk. No frame work, no semantic change to stability itself.

## Why this is worth re-measuring

brief-443 measured three fail-closed shapes and rejected all three. Two of them
failed for a reason that no longer holds:

| shape | then | why it failed |
|---|---|---|
| V1, floor checked every level | 272.1 B/level, bails at 3,797 | taking the address of a local **in the hot frame** forced a stack slot and grew the frame |
| V2, the same check on a stride | 256 B/level, bails at 4,040 | frame unchanged, but **57 levels short of the ruled 4,097** |
| V3, bare depth counter | 256 B/level, certificates pass | not stack-size-honest, and regressed `do-count-null` |

brief-444 then took the walk from 256 to **192 bytes per level** by removing
off-by-default profiling state from the frame (prologue 248 → 184;
`/proc` slope 191.1). V2's shortfall was arithmetic, not structural: at 256 B a
1 MiB budget holds 4,096 levels; at 192 B it holds 5,461.

Scaling V2's measured bail-out by the same ratio predicts **≈5,386 levels**,
about 31% above the contract. That is a projection. Measure it.

## The design, settled — do not re-explore V1 or V3

**Bail-out semantics.** The walk returns `(false, bloom_accumulated_so_far)` —
conservatively unstable, byte-for-byte the shape of the existing early-unstable
short-circuit. Stability is an optimization: `cached_expr_is_evaluation_stable`
is consumed as a plain `bool` (`crates/rum-runtime/src/runtime/resolution.rs:305-321`),
so `false` only means the expression is evaluated normally, through the
evaluator's own guarded and polled path. **No `Result`, no `Option`, no depth
parameter on the recursive signature** — that shape cost 2-7% in briefs 440 and
441 and is not to be revisited.

**Why V1 grew the frame, and how to avoid repeating it.** V1 took
`&local as *const u8` inside the hot function, which forces a stack slot there.
The address-of must live inside a `#[cold] #[inline(never)]` callee, which is
exactly what `poll_native_stack_limit` already does
(`crates/rum-runtime/src/runtime/eval_core/mod.rs:66-78` — it declares
`stack_marker` in its own frame). Follow that.

**The counter is a throttle, not a depth.** The decision is made by comparing
the real stack pointer against the budget, so the counter only governs how often
to ask. It therefore increments monotonically and is never decremented — one
read-modify-write per recursion, not the two a balanced depth counter would
need, and no unwind bookkeeping to get wrong. Use the wrapping-mask pattern
already in this file (`poll_host_interrupt_if_due`,
`crates/rum-runtime/src/runtime/eval_core/mod.rs:80-90`), but put the increment
and mask test in the hot path and call the cold function **only when the stride
fires** — that helper checks its counter inside itself, which would be a call
per level here.

## Deliverable

1. **A bool-returning sibling of the existing poll.** Add
   `native_stack_budget_exhausted(&self) -> bool`, `#[cold] #[inline(never)]`,
   beside `poll_native_stack_limit` and measuring the same way: consumption from
   `self.native_stack_base` against `self.resource_limits.native_stack_limit`
   (`crates/rum-runtime/src/runtime/eval_core/mod.rs:66-78`;
   `crates/rum-runtime/src/runtime/model/session.rs:84`). It returns a verdict
   rather than an `EvalError` because the walk has no error channel and must not
   grow one.

2. **A throttle counter on `Session`**, initialized with its neighbours
   (`crates/rum-runtime/src/runtime/session/session_state.rs`), and a named
   stride constant beside the evaluator's
   (`crates/rum-runtime/src/runtime/metadata.rs`). State the overshoot
   arithmetic in the report: stride × 192 B against the 512 KiB
   `NATIVE_STACK_POLL_HEADROOM_BYTES` reserve that
   `DEFAULT_NATIVE_STACK_LIMIT` is derived from. A stride of 16 gives about
   3 KiB, which is trivially inside it; justify whatever you choose.

3. **The check in the recursive stability path**
   (`crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:204-217` and
   the call path below it). Increment, mask-test, and on the stride ask the cold
   helper. On exhaustion return conservatively unstable.

4. **The frame must not move.** Prologue stays 184 bytes and the measured slope
   stays 192 B/level. This is the thing V1 got wrong; check it before anything
   else and stop if it moved.

## Verification (MANDATORY)

```
cargo fmt --check
cargo test -p rum-runtime
cargo test -p rum-stdlib
cargo test --workspace
cargo test -p rum-stdlib --release --lib tests::iterative_traversal_depth -- --ignored
RUM_PROFILE_EVALUATION_CACHE=1 cargo test -p rum-runtime --lib runtime::evaluation_cache::tests::call_stability_profile_reports_per_source_head_outcomes -- --ignored --exact --nocapture
```

Run the workspace gate **without** `RUM_ALLOW_NATIVE_ZEROMQ_SKIP`; if loopback
binding is unavailable, report the nine failures as environmental and say
plainly that those tests did not execute.

**All twelve release certificates must pass.** That is the contract this brief
exists to preserve; nine of twelve was V2's failure mode.

**Counters first.** For the shallow probe they must be byte-identical to HEAD's:
`stability_checks=508 call_stability_hits=128 call_stability_misses=145`.
Produce that line before any timing number.

**Frame and slope.** Prologue by disassembly; slope by the `/proc` VmStk
high-water method brief-444 used (gdb ptrace is denied in this sandbox — do not
spend time on it, and do not claim a gdb number you could not take). Four
heights, as before.

**The bail-out point.** Bisecting is **safe here and expected**, because a
working bail-out returns instead of aborting — this is the one thing that
changes from brief-444. Report the last height that succeeds and the first that
bails, and check the arithmetic against 1 MiB ÷ 192 B.

**What the user sees past it.** At height 20,001, report the exact message and
exit code. It must be an in-band resource outcome, not an abort. Say which guard
produced it: the walk bails to unstable, then the evaluator evaluates the value
through its own 4.7 KiB/level path and stops on its budget, so the visible
result should be `TerminatedEvaluation["StackLimit"]` / `General::reslim`.
Confirm that is what happens rather than assuming it.

**A/B, same protocol.** Four passes base/candidate/base/candidate in one window,
same-binary spread first; a layout-control binary with hashes checked if a label
still looks real; within-run quotients if the whole panel shifts. Base is HEAD.
Panel: `downvalue-exact-miss-repeat`, `downvalue-pattern-miss-repeat`,
`matcher-single-tiny`, `cases-wide-no-match`, `rewrite-guarded-hit`,
`replaceall-simple`, plus the untouched index `do-count-null`,
`do-iterator-null`, `symbol-ownvalue-read`, `set-ownvalue`,
`list-literal-small`, `replace-wide-no-match`. `do-count-null` is the label V3
regressed, and `cases-wide-no-match` has been inconclusive in three separate
windows — treat it as noisy rather than as a clean baseline.

**Stop-loss.** Stop and report, without a second tuning revision, if: the
prologue or slope moves; the bail-out lands at or below 4,097; any certificate
fails; the counters change; height 20,001 aborts instead of returning; or a
panel label regresses beyond its own same-binary spread. A measured "the margin
is still too thin" is a useful outcome — report the number, do not tune toward
it.

## Explicitly out of scope

- Any change to the stability walk's frame, its outlining, or its return type.
- brief-439's steps 2-8, the per-site traversal conversions.
- The same off-by-default-state defect in the evaluator's own chain, where
  brief-437 projected 0.55-0.85 KiB per level from making
  `RuntimePathTimingToken` a one-word cookie. Recorded, not touched.
- Splitting any file, though three are over the organization threshold.

## Scope

- Files you MAY modify:
  `crates/rum-runtime/src/runtime/eval_core/mod.rs`,
  `crates/rum-runtime/src/runtime/evaluation_cache/stability.rs`,
  `crates/rum-runtime/src/runtime/model/session.rs`,
  `crates/rum-runtime/src/runtime/session/session_state.rs`,
  `crates/rum-runtime/src/runtime/metadata.rs`,
  `crates/rum-runtime/src/runtime/evaluation_cache/tests.rs`,
  and `crates/rum-runtime/src/runtime/tests/session_memory.rs` if a new
  `Session` field must be declared there.
- **General accept-vs-stop criterion**: adding one `Session` field plus its
  initialization and any field-inventory test it must be declared in is in
  scope, even where I did not name the file. Any other non-test source change is
  a stop — write it in the report instead of making it.
- Do not modify `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs`, the
  `Expr`/`Symbol` representation, serde shapes, level-spec semantics, or
  `Cargo.lock`.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Do not apply, drop or
  otherwise touch the stashes.
- **Do not run a deep probe until the bail-out is in place and working.** Before
  that the walk still aborts; after it, bisection is safe and is what I asked
  for.
- No network. No third-party engine, oracle, or comparison corpus
  (`AGENTS.md` Testing Policy).
- No new `unsafe` in `rum-core` or `rum-runtime`.
- No default-path cost for diagnostics or off-by-default features — the counter
  is a correctness mechanism, not instrumentation, and must not be env-gated.

## Report format

The prologue and `/proc` slope before and after; the counter line; the measured
bail-out height with its arithmetic; the height-20,001 message and exit code
verbatim, naming the guard that produced it; verbatim `cargo fmt --check`, the
test runs, the twelve certificates and the profiling certificate; the full A/B
panel with the same-binary column first; and an explicit verdict on whether the
margin over 4,097 is real.
