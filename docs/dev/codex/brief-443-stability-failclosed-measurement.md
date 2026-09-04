# brief-443 — what fail-closed costs in the stability walk, and whether it fits

**Shape: measurement.** You may write throwaway variants in order to measure
them; the deliverable is numbers and a recommendation, and only the best variant
(if any) stays in the tree.

## The situation

brief-442 took the stability walk from 811 to **255.6 bytes per level**
(bisected: 3,905 levels on a 1024 KiB stack, 8,008 on 2048 KiB). Physical height
4,097 now completes for a rebuilding traversal with about 2x headroom, and all
twelve release certificates pass.

The walk is still **unbounded and unpolled**. Above about 8,008 levels on a
2 MiB stack it ends the process, so `SECURITY.md`'s promise that exceeding a
limit yields an in-band value rather than a crash is still not kept on this
path.

**And there is a measured conflict, which is the real question of this brief.**
The evaluator's byte budget is 1 MiB (`DEFAULT_NATIVE_STACK_LIMIT`,
`crates/rum-runtime/src/runtime/metadata.rs`). At 256 bytes per level that is
4,096 levels — one short of the ruled 4,097 — and in practice fewer, because the
walk does not start at the bottom of the stack: my 1024 KiB bisection reached
3,905, not 4,096. **So installing a poll against the existing budget would
convert the 4,097 certificates from passing to failing in band.** Trading a
contract that now works for a fail-closed promise is not obviously the right
trade, and I will not make it blind.

## The semantic design is settled — do not add an error channel

The walk has no `Result` and does not need one. `cached_expr_is_evaluation_stable`
is consumed as a plain `bool` in the fixed-point loop
(`crates/rum-runtime/src/runtime/resolution.rs:305-321`), and stability is an
optimization: `true` means "evaluation would not change this, skip it", so
returning `false` is always semantically safe — it just means the expression is
evaluated normally, through the evaluator's own guarded, polled path.

Therefore **the bail-out is `return (false, bloom_accumulated_so_far)`**, which
is byte-for-byte the shape of the existing early-unstable short-circuit
(`crates/rum-runtime/src/runtime/eval_core/stability.rs`). No new error type, no
`Result` return, no `Option` return.

That constraint is not negotiable. brief-440 and brief-441 both added a depth
parameter and an `Option` return to this function and both cost 2-7% on the
evaluator's hottest label families; they are preserved as `stash@{0}` and
`stash@{1}` and are not to be revisited.

Confirm from source in your report that a conservative `false` cannot change an
observable result for any expression that succeeds today, and say what it does
change (a value deeper than the bail-out threshold is re-evaluated instead of
skipped, so its result may be a fresh `Arc` where it was previously
pointer-identical — but every such value aborts the process today, so nothing
that works now is affected).

## Question 1 — what does detection cost?

Build and measure these three shapes. Each is a variant; do not stack them.

- **V1, precomputed floor, checked every level.** Where `native_stack_base` is
  captured (`crates/rum-runtime/src/runtime/resolution.rs:288-294` and
  `crates/rum-runtime/src/runtime/suspension/core.rs:482-510`), also store a
  precomputed `usize` floor. Each call level compares the address of a local
  against that field. One field read, one address-of, one compare, no
  read-modify-write.
- **V2, the same check on a stride.** A counter field on `Session`, polled every
  16 levels the way the evaluator polls every 4
  (`crates/rum-runtime/src/runtime/eval_core/mod.rs:39-42`). Trades a
  read-modify-write per level for fewer address comparisons.
- **V3, depth counter only.** Bail at a fixed level constant with no
  stack-pointer read. Cheapest to execute, but not stack-size-honest — a host on
  a smaller stack is not protected. Measure it anyway, as the floor of what
  detection can cost.

**The risk to watch, and it is the likely outcome:** taking the address of a
local may force a stack slot and grow the frame back. Re-read the prologue after
every variant. A variant that buys fail-closed by returning the walk to 400
bytes per level has not helped.

Report per variant: the prologue bytes of `cached_expr_stability_with_bloom`,
the measured per-level slope, and the A/B panel.

## Question 2 — how much further can the frame shrink?

brief-442 outlined the data-constructor arms, cache insertion, the profiling
paths and the miss-outcome arms, reaching a 248-byte prologue. Establish what
is left in it and what the floor is. The margin matters directly: at 256 bytes
per level a 1 MiB budget holds 4,096 levels and the contract is 4,097, so any
poll needs the frame smaller than it is now, not merely no larger.

Concrete targets, so the answer is checkable: 192 bytes per level would give
5,461 levels against a 1 MiB budget (33% margin over the contract); 128 would
give 8,192. Say which, if either, is reachable, and by what.

## Question 3 — is there a configuration where both hold?

Combine your answers. State plainly whether there is a variant in which

- the twelve release certificates still pass, **and**
- a value deep enough to exhaust the budget produces an in-band result instead
  of aborting, **and**
- no panel label regresses beyond its own same-binary spread.

If there is not, say so and give the numbers that make it impossible. **A
well-measured "these three cannot all hold" is a successful outcome of this
brief**, and is more useful to me than a variant that passes two of them
quietly. Do not resolve the tension by lowering the certificate height, raising
the evaluator budget, or weakening a test — all three are stops.

## Verification

For each variant you keep long enough to measure:

```
cargo fmt --check
cargo test -p rum-runtime
cargo test -p rum-stdlib
cargo test --workspace
cargo test -p rum-stdlib --release --lib tests::iterative_traversal_depth -- --ignored
```

Run the workspace gate **without** `RUM_ALLOW_NATIVE_ZEROMQ_SKIP`; if loopback
binding is unavailable, report the nine failures as environmental and say
plainly that those tests did not execute.

**Counters.** For the shallow panel workload they must be byte-identical to
base. From my own run of both binaries at HEAD:
`stability_checks=508 call_stability_hits=128 call_stability_misses=145`.
Produce this line first. Deep values will legitimately differ once a bail-out
exists; the shallow workload must not.

**A/B, same protocol.** Four passes base/candidate/base/candidate in one window,
same-binary spread reported first; a layout-control binary with hashes checked
if a label still looks real; within-run quotients if the whole panel shifts.
Base is HEAD. Panel: `downvalue-exact-miss-repeat`,
`downvalue-pattern-miss-repeat`, `matcher-single-tiny`, `cases-wide-no-match`,
`rewrite-guarded-hit`, `replaceall-simple`, plus the untouched index
`do-count-null`, `do-iterator-null`, `symbol-ownvalue-read`, `set-ownvalue`,
`list-literal-small`, `replace-wide-no-match`.

**A note on my own numbers, so you weigh them correctly:** in my window
`cases-wide-no-match` read +2.49%/+1.08% against a 2.32% base spread, and in the
brief-442 worker's window +5.88%/+2.77% against a 5.99% candidate spread —
inconclusive in both, which is why brief-442 was accepted. Treat that label as
noisy rather than as a known-good baseline.

**Deep behaviour.** Once a variant has a working bail-out, measuring its ceiling
by bisection is safe and expected, because the walk returns rather than
aborting. Report what a user actually sees at height 20,001 — the exact message
and exit code. Do **not** run a deep probe against a variant that has no
bail-out, and do not run one against HEAD.

## Stop-loss

Stop and report, without a second tuning revision, if: every variant costs
beyond its own same-binary spread on a named label; the frame cannot be brought
under 256 bytes per level; or the three conditions in Question 3 provably cannot
hold together. Numbers are the deliverable. I decide what to do with them.

## Scope

- Files you MAY modify while measuring:
  `crates/rum-runtime/src/runtime/eval_core/stability.rs`,
  `crates/rum-runtime/src/runtime/evaluation_cache/stability.rs`,
  `crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs`,
  `crates/rum-runtime/src/runtime/model/session.rs`,
  `crates/rum-runtime/src/runtime/session/session_state.rs`,
  `crates/rum-runtime/src/runtime/resolution.rs`,
  `crates/rum-runtime/src/runtime/suspension/core.rs`,
  `crates/rum-runtime/src/runtime/metadata.rs`,
  and a module root that must change only because a symbol it re-exports moved.
- **Leave only the variant you recommend, or a clean tree if you recommend
  none.** Revert the others yourself. Say in the report which is in the tree.
- Do not modify any test file. Do not touch `Expr`/`Symbol` representation,
  serde shapes, or `Cargo.lock`.
- **General accept-vs-stop criterion**: a `Session` field addition plus its
  initialization sites is in scope; any other non-test source change is a stop.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Do not apply, drop or
  otherwise touch the stashes.
- No network. No third-party engine, oracle, or comparison corpus
  (`AGENTS.md` Testing Policy).
- No new `unsafe` in `rum-core` or `rum-runtime`.
- No default-path cost for diagnostics or off-by-default features.
- The memory-proxy walk is **out of scope**: I measured its frame at 40 bytes
  plus the return address, about 43,000 levels on a 2 MiB stack, so it is not a
  practical ceiling and needs no conversion.

## Report format

Per variant: the prologue table, the measured per-level slope, the counter line,
the A/B panel with the same-binary column first, and the deep behaviour. Then
the Question 2 floor with its reasoning, and an explicit yes-or-no on Question 3
with the numbers behind it. Say which variant, if any, is in the tree.
