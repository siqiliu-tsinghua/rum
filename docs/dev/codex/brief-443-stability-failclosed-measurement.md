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

---

## Appended measurement report — 2026-09-04

`stability_checks=508 call_stability_hits=128 call_stability_misses=145`

HEAD, V1, V2, and V3 emitted that byte-identical counter line.

Verdict: **No measured variant satisfies all three conditions. I recommend none, and the working tree is clean.**

## Semantic confirmation

[resolution.rs](crates/rum-runtime/src/runtime/resolution.rs:320) treats stability as a Boolean optimization: `true` returns the current value; `false` enters the ordinary guarded evaluator. Therefore a conservative `false` cannot change the computed language value when evaluation completes. [stability.rs](crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:271) caches only stable verdicts, so a bailout is not persisted.

The implementation-level change is that the value is evaluated rather than skipped and may consequently contain a fresh `Arc`; pointer identity is not language-observable. There is an operational qualification: V1/V2’s 1 MiB floor precedes physical exhaustion, so they turn currently passing height-4,097 certificates into in-band resource-limit failures. Values beyond actual physical exhaustion abort today, but the budget-based bailout can affect earlier successful workloads—that is the measured contract conflict.

All variants retained `(bool, StabilityBloom)` and bailed with `false` plus the bloom accumulated so far. No `Result`, `Option`, or depth parameter was introduced.

## Prologues

Bytes count leading register pushes plus the first stack subtraction.

| Symbol | HEAD | V1 | V2 | V3 |
|---|---:|---:|---:|---:|
| `cached_expr_stability_with_bloom` | 248 | **264** | 248 | 248 |
| `all_args_stability_with_bloom` | 120 | 120 | 120 | 120 |
| `association_data_stability_with_bloom` | 392 | 392 | 392 | 392 |
| `rule_data_stability_with_bloom` | 104 | 104 | 104 | 104 |
| `rule_delayed_data_stability_with_bloom` | 88 | 88 | 88 | 88 |
| `failure_data_stability_with_bloom` | 152 | 152 | 152 | 152 |
| `failure_details_would_normalize` | 168 | 168 | 168 | 168 |
| `symbol_is_atom_evaluation_stable` | 120 | 120 | 120 | 120 |
| `call_head_has_evaluation_effect` | 168 | 168 | 168 | 168 |
| `cache_call_stability_result` | 328 | 328 | 328 | 328 |
| profile source-key helper | 104 | 104 | 104 | 104 |
| profile outcome helper | 200 | 200 | 200 | 200 |
| profile timing start/record | 40 / 264 | 40 / 264 | 40 / 264 | 40 / 264 |

The recursive call itself adds an 8-byte return address. Thus V1’s static step is 272 bytes; V2/V3 are 256 bytes.

## V1 — floor checked every level

Precomputed the native-stack floor at `evaluate`/resume entry and compared a local address against it on every uncached call level.

- Prologue: **264 bytes**
- Measured slope: **272.1 bytes/level**
- 1 MiB: last success 3,796; first bailout 3,797
- 512 KiB: last success 1,869; first bailout 1,870
- Calculation: `524288 / (3797 − 1870) = 272.076`
- `/proc` high-water corroboration: approximately 270.8 bytes/level

A/B values are seconds; positive deltas mean candidate slower.

| Label | Base spread % | V1 spread % | Base 1 | V1 1 | Base 2 | V1 2 | Δ1 % | Δ2 % |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| downvalue-exact-miss-repeat | 3.637 | 0.892 | .00129207 | .00124359 | .00124508 | .00125468 | -3.752 | +0.771 |
| downvalue-pattern-miss-repeat | 2.627 | 1.937 | .00133230 | .00128983 | .00129730 | .00131482 | -3.188 | +1.350 |
| matcher-single-tiny | 0.082 | 0.154 | .000690991 | .000684604 | .000691560 | .000685659 | -0.924 | -0.853 |
| cases-wide-no-match | 4.309 | 1.084 | .000163648 | .000155960 | .000156597 | .000157651 | -4.698 | +0.673 |
| rewrite-guarded-hit | 1.115 | 0.475 | .00180528 | .00178774 | .00182541 | .00179624 | -0.972 | -1.598 |
| replaceall-simple | 1.060 | 0.149 | .000354956 | .000356069 | .000358717 | .000355539 | +0.314 | -0.886 |
| do-count-null | 0.568 | 0.131 | .000252765 | .000253526 | .000254201 | .000253195 | +0.301 | -0.396 |
| do-iterator-null | 1.805 | 3.575 | .000052310 | .000052028 | .000051366 | .000053888 | -0.539 | +4.910 |
| symbol-ownvalue-read | 0.373 | 0.576 | .000214590 | .000210956 | .000213789 | .000212171 | -1.693 | -0.757 |
| set-ownvalue | 0.287 | 0.266 | .000259353 | .000256699 | .000258609 | .000256016 | -1.023 | -1.003 |
| list-literal-small | 3.612 | 3.085 | .000080419 | .000077610 | .000077514 | .000080004 | -3.493 | +3.212 |
| replace-wide-no-match | 0.091 | 0.116 | .000085804 | .000086244 | .000085882 | .000086144 | +0.513 | +0.305 |

The apparent `replace-wide-no-match` shift was reproduced almost exactly by the semantic-neutral layout control (+0.577%/+0.314%), so it is layout noise. No repeatable V1 panel regression remained.

## V2 — floor checked every 16 levels

Used a Session counter incremented each call level, with the address comparison every sixteenth level.

- Prologue: **248 bytes**
- Measured slope from budget cutoffs: **256.5 bytes/level**, quantized by the 16-level polling stride
- Direct `/proc` slope: **256.0 bytes/level**
- 1 MiB: last success 4,039; first bailout 4,040
- 512 KiB: last success 1,995; first bailout 1,996
- Calculation: `524288 / (4040 − 1996) = 256.501`

| Label | Base spread % | V2 spread % | Base 1 | V2 1 | Base 2 | V2 2 | Δ1 % | Δ2 % |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| downvalue-exact-miss-repeat | 2.804 | 0.061 | .00127422 | .00125864 | .00123849 | .00125787 | -1.223 | +1.565 |
| downvalue-pattern-miss-repeat | 2.091 | 0.083 | .00131782 | .00131779 | .00129026 | .00131669 | -0.002 | +2.048 |
| matcher-single-tiny | 1.559 | 1.982 | .000696075 | .000703019 | .000685224 | .000689087 | +0.998 | +0.564 |
| cases-wide-no-match | 0.135 | 0.208 | .000157901 | .000156573 | .000157688 | .000156899 | -0.841 | -0.500 |
| rewrite-guarded-hit | 1.102 | 0.794 | .00180705 | .00181388 | .00178713 | .00182828 | +0.378 | +2.303 |
| replaceall-simple | 2.175 | 1.517 | .000363416 | .000354737 | .000355511 | .000360118 | -2.388 | +1.296 |
| do-count-null | 0.024 | 2.774 | .000253475 | .000246848 | .000253535 | .000253695 | -2.614 | +0.063 |
| do-iterator-null | 7.815 | 2.307 | .000049829 | .000048808 | .000053723 | .000049934 | -2.049 | -7.053 |
| symbol-ownvalue-read | 6.559 | 7.231 | .000228478 | .000210384 | .000213491 | .000225597 | -7.919 | +5.670 |
| set-ownvalue | 2.304 | 2.496 | .000263944 | .000260224 | .000257863 | .000266720 | -1.409 | +3.435 |
| list-literal-small | 1.609 | 8.322 | .000079550 | .000080765 | .000080830 | .000087486 | +1.527 | +8.235 |
| replace-wide-no-match | 0.109 | 0.793 | .000086049 | .000086871 | .000086143 | .000086182 | +0.955 | +0.045 |

The excursions were not repeatable across both passes and remained within the corresponding noisy same-binary behavior. No stable V2 regression was established.

## V3 — fixed depth 4,096

Used a Session depth counter with no stack-address read. It is deliberately not stack-size-honest.

- Prologue: **248 bytes**
- `/proc` slope: **256.0 bytes/level**
- High-water: 532 KiB at height 2,000; 1,032 KiB at height 4,000
- Stability bailout begins at depth 4,096
- The guarded evaluation subsequently succeeds through height 4,315; first visible `TerminatedEvaluation["StackLimit"]` is height 4,316

| Label | Base spread % | V3 spread % | Base 1 | V3 1 | Base 2 | V3 2 | Δ1 % | Δ2 % |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| downvalue-exact-miss-repeat | 4.011 | 0.564 | .00124431 | .00124232 | .00129422 | .00124933 | -0.160 | -3.468 |
| downvalue-pattern-miss-repeat | 3.520 | 0.680 | .00129364 | .00129209 | .00133917 | .00130088 | -0.120 | -2.859 |
| matcher-single-tiny | 0.121 | 0.791 | .000686440 | .000688539 | .000687273 | .000693983 | +0.306 | +0.976 |
| cases-wide-no-match | 0.399 | 2.301 | .000158775 | .000156712 | .000159408 | .000160318 | -1.299 | +0.571 |
| rewrite-guarded-hit | 0.088 | 0.128 | .00181347 | .00179969 | .00181187 | .00179739 | -0.760 | -0.799 |
| replaceall-simple | 0.441 | 0.431 | .000362232 | .000360468 | .000360633 | .000358916 | -0.487 | -0.476 |
| do-count-null | 0.341 | 0.261 | .000245750 | .000251141 | .000244912 | .000251797 | **+2.194** | **+2.811** |
| do-iterator-null | 7.385 | 1.711 | .000052660 | .000050096 | .000048771 | .000050953 | -4.869 | +4.474 |
| symbol-ownvalue-read | 0.752 | 0.867 | .000215421 | .000214281 | .000213800 | .000212424 | -0.529 | -0.644 |
| set-ownvalue | 0.359 | 1.194 | .000260460 | .000261430 | .000259525 | .000258308 | +0.372 | -0.469 |
| list-literal-small | 0.864 | 2.646 | .000079166 | .000082055 | .000079850 | .000079884 | +3.649 | +0.043 |
| replace-wide-no-match | 0.330 | 1.444 | .000085936 | .000086794 | .000085652 | .000088047 | +0.998 | +2.796 |

V3’s whole-panel medians were −0.115% and +1.447%, so this was not simply a uniform panel shift.

The `do-count-null` result survived the required controls:

- Semantic-neutral layout control: +0.097%/−1.590%, against 0.881%/2.552% same-binary spread; it did not reproduce V3.
- V3 quotient against `symbol-ownvalue-read`: +2.737%/+3.477%.
- V3 quotient against `set-ownvalue`: +1.815%/+3.296%.
- SHA-256 identities:
  - HEAD: `1b0c4d219d4c5f0d1d5ebba5529992076867ec8dbdd43681b855aa31f81c956d`
  - V3: `08ade44a5a866ef77d167ebfa4caff00ddfc3472749b1d1d4d99005a66bdaeac`
  - layout control: `4763b1af6f12c61c1d53e1d5205eb919ee35e2c6fdaf3af4df85345775725b2c`

Therefore V3 has a real named-label regression.

## Deep behavior at height 20,001

All three variants failed closed without SIGABRT or stack-overflow termination:

- Exit code: **1**
- Stdout: empty
- Stderr, exactly:

```text
! evaluation exceeded native stack limit (1048576 bytes)
error: evaluation exceeded native stack limit (1048576 bytes)
```

## Verification

All workspace runs omitted `RUM_ALLOW_NATIVE_ZEROMQ_SKIP`.

| Gate | V1 | V2 | V3 |
|---|---|---|---|
| `cargo fmt --check` | pass | pass | pass |
| `cargo test -p rum-runtime` | 955 passed, 0 failed, 2 ignored | same | same |
| `cargo test -p rum-stdlib` | **844 passed, 2 failed, 15 ignored** | same | 846 passed, 0 failed, 15 ignored |
| `cargo test --workspace` | environmental failure | environmental failure | environmental failure |
| release certificates | **9 passed, 3 failed** | **9 passed, 3 failed** | **12 passed** |

V1/V2 stdlib failures were:

- `from_toml_depth_limit_accepts_each_descent_at_the_limit`
- `from_yaml_depth_limit_accepts_every_collection_descent_at_the_limit`

Both received `StackLimitExceeded { limit: 1048576 }`.

V1/V2 release failures were `Map`, `Apply`, and `ReplaceAll` at physical height 4,097. They failed in band rather than aborting.

Each workspace run reached the `rum-jupyter` test binary with 97 passes and nine native ZeroMQ failures. Loopback bind returned `Operation not permitted (os error 1)` during setup; **those nine native tests did not execute their scenarios**. Consequently the workspace gate is environmentally incomplete, not green.

## Question 2 — remaining frame and floor

The current 256-byte recursive step is:

- 248-byte prologue: six saved registers (48 bytes) plus `sub $0xc8` (200 bytes)
- 8-byte recursive return address
- A mandatory 48-byte ABI result area for `(bool, StabilityBloom)`; `StabilityBloom` contains two aligned `u128`s
- A 16-byte pointer cache key
- A 56-byte `Option<CallStabilitySourceHeadKey>` retained across recursion even with profiling disabled
- The timing token/miss outcome and live session/head/argument/output pointers

The next structurally reachable floor is **192 bytes/level**, but it was not built after the stop-loss. The concrete route is to remove the 56-byte profiling key and associated timing spill from the ordinary recursive frame—using a compact token or recomputing the key entirely inside the outlined profile-enabled path. Stack alignment turns the resulting approximately 64-byte reduction from 256 into a 192-byte step. That would provide about 5,461 nominal levels per MiB and enough margin over 4,097.

**128 bytes/level is not reachable under the settled combined return/cache shape.** After removing profiling state, the next 64 bytes are already consumed by the 48-byte recursive return area plus the 16-byte cache key, before accounting for the parent bloom and live walk pointers. Reaching 128 would require changing the return ABI, representation, or control-flow split rather than further outlining cold locals.

Thus:

- Measured floor in this brief: **256 bytes/level**
- Assembly-established next target: **192 bytes/level**
- 128 bytes/level: **not reachable under the current contract**

## Question 3 — can all three conditions hold?

**No, not in any measured variant.**

| Variant | 12 certificates | Fails closed at depth | No demonstrated panel regression |
|---|---|---|---|
| V1 | **No: 9/12** | Yes | Yes after layout control |
| V2 | **No: 9/12** | Yes | Yes |
| V3 | Yes: 12/12 | Yes on this stack, but not stack-size-honest | **No: `do-count-null` +2.19%/+2.81%** |

V1 grows to 272.1 bytes/level and bails at 3,797. V2 remains 256 bytes/level but real starting-stack overhead moves its bailout to 4,040, still below 4,097. V3 preserves the certificates but pays a reproducible hot-path regression and does not protect smaller host stacks.

The stop-loss therefore applies; I did not attempt a second tuning revision. No variant remains in the repository, no test was modified, and `git status` is clean.
---

## Director gate — 2026-09-04

**Scope**: clean. No source change, no test change, tree clean — a measurement
brief that reverted all three variants, as it was asked to.

**The answer to Question 3 is a well-measured no, and that is the successful
outcome.** V1 grows the frame to 272.1 B/level and bails at 3,797; V2 holds
256 B/level but real starting-stack overhead moves its bail-out to 4,040; both
turn nine of the twelve certificates into in-band failures. V3 keeps all twelve
but is not stack-size-honest and regresses `do-count-null` +2.19%/+2.81%. The
counter line is byte-identical across HEAD and all three, so none of them did
extra work.

Its method is better than what I asked for: a gdb breakpoint on the recursive
symbol sampling `$rsp` minima, cross-checked against `/proc` high-water. That
measures the slope without a single abort, and it is the technique to reuse.

**The finding that matters more than the verdict is a hard-line violation.**
`cached_expr_stability_with_bloom` holds
`profile_key: Option<CallStabilitySourceHeadKey>` and
`timing_start: Option<Instant>` across the recursive call
(`crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:224-225`).
`call_stability_source_head_key` returns `None` immediately when profiling is
off (`.../profile/source.rs:157-165`), so no *work* is done — but the locals
still occupy the frame. `Symbol` is 24 bytes and the key pairs it with a head
shape, so that is about 72 bytes of off-by-default instrumentation state
resident in the hot recursive frame on every default-path call.

CLAUDE.md's line is "**No default-path cost for diagnostics or off-by-default
features**", and on a recursive walk stack bytes per level are not a lesser
kind of cost than CPU — they are exactly what the depth contract is made of.
This has to be repaired whether or not a poll is ever installed.

Its payoff is also the route out of the conflict: removing that state is the
named path from 256 to 192 B/level, which would give 5,461 levels against a
1 MiB budget — 33% margin over the ruled 4,097 — and would let V2's strided
poll fit above the contract instead of 57 levels below it.

**128 B/level is out of reach and the reason is structural**, not a matter of
more outlining: the 48-byte ABI result area for `(bool, StabilityBloom)` (two
aligned `u128`s) plus the 16-byte pointer cache key already consume the budget
below 192. Reaching it would mean changing the return representation, which is
the shape that cost 2-7% in briefs 440 and 441. Recorded as closed.

**Sequencing.** The profiling-state removal goes first and alone, as brief-444:
it stands on the hard line by itself, its A/B should be neutral or better, and
bundling it with a poll would put two variables in one panel. The poll is
re-measured against the smaller frame afterwards.
