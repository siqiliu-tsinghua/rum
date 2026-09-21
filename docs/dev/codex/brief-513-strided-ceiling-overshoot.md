# brief-513 — design: the memory ceiling that a loop walks straight through

`Scope: NONE — no source change.` Design-only. Survey, reason, return a plan
and drafted implementation briefs. Write no code.

## The finding, director-reproduced 2026-09-22 with a calibrated control

The 2026-09-02 audit's second HIGH. Both halves re-probed; one of them needs
correcting before any repair is designed.

**It reproduces, and the ceiling does nothing:**

```
gSeed = StringRepeat["x", 1000];
gOut = MemoryConstrained[Nest[StringJoin[#, #] &, gSeed, 16], 8000000];
Print[StringLength[gOut]];
```

An 8 MB modelled ceiling; the expression **completes successfully**, returns a
65 536 000-character string, and peaks at **~350 MB RSS**. No abort, no
message.

**The control proves the ceiling is armed and does work:**

```
gOut = MemoryConstrained[StringRepeat["x", 100000000], 8000000];
```

Same 8 MB ceiling, single large allocation → **`$Aborted`**.

So the mechanism is not a dead limit. It is a limit that a sequence of
individually-innocent steps walks straight through.

## The audit's mechanism is wrong, and the repair depends on this

The audit said `eval_string_join`
(`crates/rum-stdlib/src/strings/text/builtins.rs:13-42`) "has no
projected-byte precheck at all, unlike `StringRepeat`". That framing points
at the wrong repair.

`StringJoin` is declared `Preflight`
(`crates/rum-stdlib/src/catalog/memory_growth_contract.rs:546`), and the
enforcement for that contract is: a `Preflight` builtin whose result exceeds
its arguments plus an allowance, and which did not project, panics
(`crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:403-415`). **`StringJoin`
does not amplify per call** — concatenation rearranges bytes, so
`result_bytes ≈ argument_bytes` — which is why the enforcement correctly
passes it and why it is not on
`KNOWN_UNPROJECTED_PREFLIGHT_BUILTINS` (`eval_dispatch/mod.rs:7-20`).

The overshoot comes from **repeating a non-amplifying step between strided
full walks**: `ACTIVE_MEMORY_LIMIT_CHECK_STRIDE = 64`
(`crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:9`),
and a doubling reaches 2 GB from 1 MB in about eleven steps. Adding a
precheck to `StringJoin` would not have caught this.

**The comment above the stride states the false premise plainly**
(`constraints.rs:5-8`): "between sampled walks, overshoot is bounded by the
existing per-step element, integer, and iteration budgets." Those budgets
bound **counts**. The quantity at risk is **bytes**. Whatever else this line
produces, that comment must stop asserting a bound that does not exist.

## What I want designed

### 1. The option space, scored

For each: what it bounds, what the residual overshoot becomes, its
**measured or estimated** hot-path cost, and how it is falsified.

- **Unstride the full walk while a limit is armed.** Simplest and strictest.
  The stride exists for a reason — establish what walking every armed check
  actually costs, rather than assuming it is prohibitive.
- **Adaptive stride.** Sample every time once the last observed usage is
  within some factor of the limit, stride widely when far below. Bounds
  overshoot to roughly that factor instead of leaving it unbounded. Say what
  the factor must be and what it costs.
- **An incremental byte counter** maintained at construction sites rather
  than walked. This is the `Incremental` contract
  (`crates/rum-runtime/src/runtime/metadata.rs:346-348`), which
  `TODO.md`'s memory-accounting entry records as **still unbuilt** — so
  scope it honestly rather than treating it as available.
- Anything else you find. Include the option of bounding only the
  *doubling-shaped* case if you can define it precisely, and say why that is
  or is not principled.

### 2. The hot-path question, answered with a number or an explicit gap

The stride is a performance decision, so its removal or narrowing is a
hot-path change and this project gates those with measurement, not
intention. Specify the A/B the implementation must pass — instruction counts
via `taskset -c 2 perf stat -e instructions:u`, not the timing panel — and
name the labels. Note that the check only runs **when a limit is armed**, so
say plainly whether the default path moves at all; if it does not, the
acceptance argument is very different and should be made explicitly rather
than left implied.

### 3. The falsifier

This project's standing rule: a check nobody has watched fail is not known
to work. Design the test that fires on the reproduction above, and the proof
that it fails when the repair is removed. The reproduction and its control
are both given above — use them.

### 4. The scope boundary

`MemoryConstrained` is the in-language frame; `eval_memory_limit` is the
per-request ceiling. Say whether the repair covers both, and do **not**
silently widen into the process-global interner quota, which is a separate
line blocked on a user ruling (`TODO.md` §The interner ruling of 2026-09-21).

## Method requirements

- Read `AGENTS.md`, `TODO.md` §Opened by the 2026-09-02 security audit, and
  the memory-accounting entries before proposing anything.
- **Re-probe rather than inherit.** The audit's mechanism was wrong here and
  the director's own `Module` premise was wrong two briefs ago. Verify any
  claim you lean on, including the ones in this brief, and say how.
- Where you assert a cost, say whether it is measured, inferred or a guess.
- Name what would falsify your recommendation.

## Hard rules

- **No source change.** No `.rs`, no script, no fixture, no doc except
  appending your report to this file. Scratch and `.rum` probes under
  `target/brief-513/`.
- Do NOT `git commit`, branch, switch, merge, push, stash, `git checkout` or
  `git restore`, or touch git config or the index.
- **No new `unsafe`.**
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only; no oracle harnesses, no comparisons against other vendors'
  engines.
- No network. Touch nothing outside this repo. Bound anything long with
  `scripts/run-bounded.py --timeout SECONDS`, sized from a measured run of
  the same shape.
- A release `rum-cli` built from `main` is at `target/brief-502/rum-base`;
  reuse it rather than rebuilding, and do not overwrite anything under
  `target/brief-502/` or `target/brief-508/`.
- **Stop-loss, keyed on progress.** If a probe fails twice and the second
  attempt produced no more information, stop and report what you have.
- Paste verbatim evidence; never quote a number you could not take.

## Report format

1. Confirmation or correction of the mechanism stated above.
2. The option table, scored.
3. The recommendation, its reasoning, and what would falsify it.
4. The falsifier design and the required A/B.
5. Drafted follow-up briefs, and any director decisions you hit.

---

## Execution report — 2026-09-22

Design only. The recommendation is to **measure and gate an unstrided armed
check as the first repair candidate**. It removes the sampling defect at the
existing evaluation checkpoints for both kinds of ceiling. It does not turn
the structural proxy into an allocation or RSS bound. The measured cost of
walking is substantial; accepting the armed-path cost requires the actual
candidate A/B, which this no-source-change brief cannot supply.

Only this report was appended to tracked files. All probes and raw outputs
are under `target/brief-513/`. No build, source edit, fixture edit, network
access, git mutation, or change under `target/brief-502/` or
`target/brief-508/` was made. Initial `git status --short` was empty. The
checkout and reused executable were identified as follows, verbatim:

```text
06e24a1bb4ae4c288443010c2493fda01d7b6644 docs: brief-513, the ceiling a loop walks through, with the audit's mechanism corrected
d11303d4ba955aff8e873a85e5ba40280a5e017586cf94e524365743668a2102  target/brief-502/rum-base
```

The binary is the supplied earlier main build, not a new build of this
checkout. Runtime observations below identify that binary; source findings
identify the checkout. An implementation gate must build its own matching
base and candidate.

### 1. Mechanism: confirmed, with three corrections

**The given reproduction and control both reproduce.** Files
`reproduction.rum` and `control.rum` contain the supplied programs, with
`Print[gOut]` added to the control to expose its result. Their stdout and
separately captured metrics are verbatim:

```text
reproduction.stdout:
65536000
Null

reproduction.metrics.json:
{
  "exit": 0,
  "elapsed_monotonic_s": 0.10674547800226719,
  "max_rss_kib": 358452
}

control.stdout:
$Aborted
Null

control.metrics.json:
{
  "exit": 0,
  "elapsed_monotonic_s": 0.005588014999375446,
  "max_rss_kib": 12360
}
```

Both stderr files are empty. RSS was measured with Python's Linux
`resource.getrusage(RUSAGE_CHILDREN).ru_maxrss`, in a separate measuring
process for each engine run. It is a child high-water mark, not the kernel's
proxy. The first attempt used `/usr/bin/time`, which is absent; switching to
`getrusage` produced the measurements above. Elapsed values use
`time.monotonic` solely to size execution bounds, never to accept a
performance change.

The initial short calibration used the same seed and an eight-step `Nest`:

```text
{"exit": 0, "elapsed_monotonic_s": 0.00539892600136227, "max_rss_kib": 18360}
256000
Null
```

That tiny calibration had a five-second bootstrap watchdog. Scaling its
payload by the next eight doublings gave a conservative duration estimate
below two seconds; the exact reproduction had a ten-second watchdog. The
subsequent semantic batch had a five-second bound based on the measured
exact reproduction. All engine/perf invocations were enclosed by
`scripts/run-bounded.py`; no unbounded large allocation was attempted.

**The source establishes the sampling mechanism.** In
`runtime/debug_tools/resources/constraints.rs`, the first armed check walks
`current_memory_proxy_bytes() + expr_memory_proxy_bytes(expr)`, sets the
countdown to 63, and the next 63 checks only decrement it. Projected-byte
checks at lines 175–188 always perform their walk when armed. These are
evaluation checkpoints, not `Nest` iterations: `resolution.rs:321,400` checks
fixed-point inputs and `eval_core/mod.rs:59` checks successful unstable
results. `functional_ops/iteration.rs:266` onward evaluates the callback on
each iteration. No bound on the byte growth between sampled checkpoints is
established by these count budgets.

`StringJoin` is indeed `Preflight` in
`rum-stdlib/src/catalog/memory_growth_contract.rs:546`, is absent from
`KNOWN_UNPROJECTED_PREFLIGHT_BUILTINS`, and calls no projection API. The
growth assertion compares **result bytes with aggregate argument bytes**;
it does not compare successive loop results. A small direct byte-count probe
printed this verbatim:

```text
{1032, 2032}
74600
Null
```

The first row is `{ByteCount[gSeed],
ByteCount[StringJoin[gSeed, gSeed]]}` for the 1,000-character seed.
Thus the result's 2,032 bytes are less than the two arguments' combined
2,064 bytes. The second row is `MemoryInUse[]` in that particular probe,
not a general session baseline. Concatenation is non-amplifying by this
comparison even though successive results double. Also, it allocates:
`eval_string_join` makes owned `pieces` and concatenates them while input
values are still live. Non-amplifying result size is not a promise of zero
temporary allocation.

Three premises need qualification or correction before designing the fix:

1. **The growth assertion is an instrument, not the production ceiling.**
   `eval_dispatch/mod.rs:201` bypasses it unless the test feature or
   `RUM_TEST_BUILTIN_MEMORY_GROWTH` enables it. Its 1,024-byte allowance is
   not a production memory allowance. I reran the exact reproduction with
   `RUM_TEST_BUILTIN_MEMORY_GROWTH=1`; it still printed `65536000` and `Null`,
   exited zero, and had empty stderr (`reproduction-growth-check.*`). This
   agrees with the aggregate-argument comparison, not a missing allowlist
   entry.
2. **A proper precheck in `StringJoin` would catch this example.** The
   brief's categorical statement to the contrary is incorrect. A precheck
   which passes the projected output byte count to
   `enforce_active_memory_limit_for_projected_bytes` performs an unstrided
   check. Even ignoring all other roots, it would refuse no later than the
   call proposing 8,192,000 output bytes: `1000 * 2^13 > 8000000`
   (arithmetic inference, not a measured patched build). Output-size
   projection is different from asking whether output exceeds aggregate
   arguments. This is a valid narrow remedy, but not a general repair of
   sampled session growth. No patched `StringJoin` was built here.
3. **Last-sample adaptive stride alone has no useful factor bound.** A
   last reading far below the limit says nothing about growth during the
   next skipped interval. The factor argument requires an independently
   enforced growth envelope, discussed below.

One attempted corroboration was unsuitable and is not counted as proof:
replacing the callback by `StringRepeat[#, 2] &` stopped on the independent
generated-element budget before testing the desired byte-ceiling behavior:

```text
! evaluation exceeded generated element limit (100000)
error: evaluation exceeded generated element limit (100000)
```

It exited 1 (`repeated-projection.*`). I did not retry the same unsuitable
probe. Conversely, the supplied single-allocation control returns
`$Aborted` because `StringRepeat` projects memory before its generated-element
check (`strings/text/builtins.rs:664–668`). A generic "any error" assertion
would lose this distinction.

**Readings and scope checks.** I read `AGENTS.md`, the audit section of
`TODO.md`, its full-walk/cache and later memory-accounting notes, the current
interner ruling, the testing strategy, the relevant architecture/host/guide
contracts, and the implementation and test sites cited here. The specific
TODO entry saying `Incremental` is unbuilt was not found in this checkout.
Its implementation status is independently clear: `metadata.rs:346–348`
declares it, while `validate_builtin_memory_growth_slow` does nothing for
`Incremental`; brief-420 also explicitly records its missing falsifier.
There is no universal incremental accounting mechanism available to reuse.

The current TODO also contains interner rulings later than the summary in
this brief. None changes this task's boundary: process-global interner quota,
default size, ownership, admission, and reclamation are separate work.

The guide promises a local `General::memlim`, but this control emitted no
message, and `finish_memory_constrained` consumes the error without an
explicit message emission. Record that discrepancy separately; this report
does not claim a message was observed or weaken the documented contract.

### 2. Option table, scores, and measured cost

Scores below are **engineering judgments**, not measurements. Each tuple is
`coverage / cheap armed execution / implementation readiness`, each from 1
(weak) to 5 (strong). Coverage means preventing repeated unchecked model
growth at the intended boundaries, not providing a physical RSS guarantee.
Do not sum the scores: a cheaper partial mitigation does not replace a
complete sampling repair.

Let `L` be the limit, `C` the number of armed evaluation checks, and `W` the
cost of a session-plus-current-expression walk at a particular state. Neither
`W` nor the amount of native work between checks is constant across programs.

| Option; scores | What it bounds; remaining overshoot | Cost and evidence status | How to falsify it |
| --- | --- | --- | --- |
| **A. Full walk at every armed check; 5 / 1 / 5** | Every existing checked state must satisfy the existing model. Removes the 63 skipped-check window. It detects an oversized result on return, but one unchecked construction can still allocate before that return; native temporaries and omitted roots remain outside the model. No universal RSS or one-step byte factor follows. | Inferred walk count changes from about `ceil(C/64)` to `C`, up to 64 times the sampled-walk component, not necessarily 64 times total evaluation. Measured walk surrogates below. Actual candidate instruction delta: **not measured**. No additional unarmed algorithmic work if confined to the existing armed branch. | Original reproduction survives; any armed phase skips an oversized checked value; or the required armed A/B demonstrates unacceptable cost. Also falsified as a claimed hard allocation bound by a large allocation within one unchecked native operation. |
| **B. Stride 64 below `L/2`, stride 1 above; 1 / 3 / 3** | Bounds only how frequently the next decision is made from stale information. Does **not** bound overshoot to 2x. A low sample followed by repeated growth defeats it before the threshold is observed. | Inferred: existing cheap countdown plus threshold/scheduling state, then A's cost near the limit. Unarmed path can remain unchanged. No candidate measurement. | Start below the threshold, grow immediately after a sample, and cross both threshold and ceiling before the next full walk. Sweep countdown phases. |
| **C. Reserve/charge/release bytes at construction and retention boundaries; 5 / 4 / 1** | With complete coverage and a sound definition of charged bytes, reservation **before** growth can bound that model; charge-after-growth leaves one allocation/chunk of overshoot. A monotonic allocation-total counter would incorrectly charge sequential discarded work as simultaneous residency. | Inferred `O(1)` ledger work per reservation/release, but construction, clone, root mutation, and drop coverage is a broad change. Default-path overhead is not known to be zero. No implementation or measured delta exists. Maintaining current structural-proxy semantics may require more than an allocation counter. | An uncharged constructor/cache insertion/resume path, lost release, double release, cross-session value, or a counter that disagrees with an independent walk. Sequential allocate/discard must continue to fit a residency budget. |
| **D. Always check atomic string size when armed; 2 / 5 / 4** | Rejects any single checked string whose `size_of::<Expr>() + capacity` exceeds `L`. Covers this doubling example at an evaluation boundary. Leaves many individually small retained strings, trees, and native temporaries governed by the old sampled walk. | Inferred constant-time tag/capacity work per armed check; no byte scan for a string. Full walks remain strided. No candidate measurement; unarmed work need not change. | Original string reproduction survives; non-string doubling or many small retained values escapes. The latter falsifies any claim of general closure, not the explicitly narrow guarantee. |
| **E. Preflight `StringJoin`'s output before making owned pieces; 1 / 4 / 5** | Bounds this builtin's projected output plus currently walked roots at admission. Catches the given reproduction. Does not bound all its temporary allocations merely by counting output, and does not cover other builders or callback-driven retention. | Inferred byte-length sum over arguments plus one unstrided session walk per armed join. A new predicate or projection call may add default-path instructions; measure it. No candidate measurement. | The reproduction survives; list-of-strings/multibyte forms bypass the projection; or a join-free growth program escapes a claimed general repair. |
| **F. Walk every current expression, sample session roots; 3 / 3 / 4** | Every individually oversized checked expression is rejected. This is a principled necessary condition for the full model, broader than D. Accumulation across definitions, reaps, prints, caches, or suspension roots can still exceed `L` between full walks while each checked expression fits. | Inferred `O(current-expression nodes)` at every armed check, plus sampled session walks. Strings are cheap, wide/shared trees need repeated traversals. No candidate measurement; unarmed work need not change. | Aggregate retained growth with small results escapes; an expression-heavy armed workload loses the expected cost advantage. |

**The precise adaptive factor.** Suppose, hypothetically, that total model
usage can grow by at most a known multiplicative factor `g` per skipped
checkpoint interval, with no unaccounted additive growth. From a sample `U`,
allowing `k` intervals before the next sample permits `g^k * U`. To bound
sampled overshoot by `F * L`, require
`k <= floor(log_g(F * L / U))`, and check immediately if `U > L`.
For a proved doubling envelope and a desired factor 2, this becomes
`k <= floor(log2(2 * L / U))`. These are conditional mathematical bounds,
not established properties of this evaluator.

A fixed 64-wide phase below `L/a` would need `a >= 2^63` even for that
restricted doubling envelope and a factor-2 target. A threshold of `L/2`
does not work. For example, a sample of 1 MB under an 8 MB ceiling can
become 16 MB after four doublings without another 64-stride sample. There
is no proved `g` for arbitrary evaluated callbacks, root retention, and
native constructions. Without one, use stride 1 or add a sound byte-reservation
mechanism; do not advertise a factor guarantee from adaptive sampling alone.

**What "only doubling-shaped" can honestly mean.** Option D is defined by
the checked value's representation, not by recognizing `Nest`, a callback's
source, or the `StringJoin` name. For a stipulated sequence of atomic strings
where each successive checked string is at most twice its predecessor, the
first oversized *string payload* is at most twice `L`. That says nothing
about simultaneous live copies or RSS. Removing the factor-two stipulation
still gives D's useful guarantee: no checked over-limit string is accepted.
F extends that necessary-condition guarantee to all expression shapes. Either
can be a named containment step; neither closes the session-ceiling finding.

**Measured walk-cost surrogate.** Source reads show that `MemoryInUse[]`
calls `current_memory_proxy_bytes` unconditionally, and that a string is
charged from its capacity while a call recursively visits its children.
Therefore I measured two unchanged-binary programs for each retained root:
identical setup and output, followed by either `Do[Null, {1024}]` or
`Do[MemoryInUse[], {1024}]`. Each program prints the root's `ByteCount` and
`done`. All ended with the expected output, exit zero, and empty stderr.

Each pair was run five times with alternating idle/walk order, serially,
profiling/test diagnostic environment variables removed, using:

```text
scripts/run-bounded.py --timeout 5 -- taskset -c 2 perf stat -e instructions:u -x, -o <raw.csv> -- target/brief-502/rum-base <probe.rum>
```

Input was EOF and stdout/stderr were separate files under `target/brief-513/`.
No other workload was launched concurrently. A representative capability
check's PMU output is verbatim:

```text
<not counted>,,cpu_atom/instructions/u,0,0.00,,
4187653,,cpu_core/instructions/u,5368811,100.00,,
```

CPU 2 counted the core event at 100% coverage. The uncounted atom event is
not a zero instruction count. Every cost sample was checked for one numeric
instruction row with 100% coverage.

| Retained root | Root proxy bytes printed | Idle instructions, median [min, max] | Walk instructions, median [min, max] | Extra instructions per `MemoryInUse[]`, derived |
| --- | ---: | ---: | ---: | ---: |
| Integer `0` | 32 | 3,985,993 [3,985,067, 3,988,265] | 267,941,558 [267,939,946, 267,941,632] | 257,769.11 |
| 1,000,000-byte string | 1,000,032 | 10,846,472 [10,845,025, 10,857,579] | 303,823,638 [303,813,708, 303,825,931] | 286,110.51 |
| `Range[1000]` | 32,075 | 4,138,526 [4,136,797, 4,139,167] | 321,539,400 [321,538,541, 321,540,786] | 309,961.79 |
| `Range[10000]` | 320,075 | 5,241,318 [5,240,156, 5,242,066] | 801,873,565 [801,872,372, 801,874,621] | 777,961.18 |

The last column is `(median(walk) - median(idle)) / 1024`, not a directly
isolated native-function count. Verbatim summarizer output for the two
endpoints:

```text
{"shape": "empty", "idle_median": 3985993, "idle_min": 3985067, "idle_max": 3988265, "walk_median": 267941558, "walk_min": 267939946, "walk_max": 267941632, "extra_instructions_per_MemoryInUse": 257769.1064453125}
{"shape": "list10k", "idle_median": 5241318, "idle_min": 5240156, "idle_max": 5242066, "walk_median": 801873565, "walk_min": 801872372, "walk_max": 801874621, "extra_instructions_per_MemoryInUse": 777961.1787109375}
```

These differences include language dispatch, result construction, and any
cache changes caused by `MemoryInUse`. They omit the repaired check's walk
of the current expression and may have different active roots. The string
setup also warms different caches. They are **measured surrogates**, not
upper bounds, exact `W`, or candidate/base percentages.

For context I also measured the current armed implementation with the same
roots and `Print[MemoryConstrained[Do[Null, {1024}], 1000000000]]`.
It printed `Null` for the constrained operation in every sample; a discarded
`$Aborted` cannot masquerade as completion. Five-run medians [min, max]:

| Root | Current armed instructions |
| --- | ---: |
| Integer `0` | 8,349,346 [8,347,425, 8,350,721] |
| String | 15,688,053 [15,686,710, 15,689,441] |
| `Range[1000]` | 9,385,755 [9,382,932, 9,388,239] |
| `Range[10000]` | 18,442,953 [18,441,803, 18,444,716] |

This table includes an extra `Print`, so it is context rather than an exact
isolation of arming cost. The earlier `cost-*-armed` runs discarded the
constrained result and are superseded for evidentiary purposes by
`cost-*-armed-live` and `armed-live-cost-results.json`.

The cost calibration was 16 walks over a 1,000-element retained list, then
64 walks over a 10,000-element list. Their counted-event running times were
6,864,940 ns and 11,219,472 ns respectively. Scaling the latter by 16 for
1,024 walks left ample room in the five-second per-run bounds and the
30-second bound for the 40-run idle/walk batch. Actual measured wrapper
durations, used only to verify headroom, were:

```text
{"cost_run_max_monotonic_s": 0.19639844300036202, "cost_run_sum_monotonic_s": 5.797499502987193}
```

Raw evidence is in `cost-*.perf.csv`, `cost-*.stdout`, `cost-*.stderr`,
`cost-results.json`, `cost-summary.json`, and
`armed-live-cost-{results,summary}.json`. The underlying `.rum` files are
retained beside them. No timing-panel result is used as the requested gate.

### 3. Recommendation, scope, and ways it can fail

**Make A the first implementation candidate.** It is the smallest broad
repair of the identified sampling policy and has an intelligible promise:
every existing armed checkpoint checks the model. Keep the check inside
`if let Some(limit) = current_memory_limit`; retain time-limit ordering,
saturating arithmetic, the current model, and projected prechecks. Replace
the false count-budget comment with the exact checkpoint guarantee and its
single-construction limitation. Do not silently introduce a new adaptive
heuristic when measuring an expensive result.

**Default execution versus armed execution must be reported separately.**
The current default is `eval_memory_limit: None`. Source inspection shows
that the countdown and full walks are already inside the armed branch.
Replacing its interior adds **zero new algorithmic operations to the
unarmed path**. This is a source-level conclusion, not a measured claim of
zero retired-instruction delta. Code layout, inlining, and removing the
countdown field can still change default-path machine code or `Session`
layout. Measure those effects; do not add an unconditional size walk,
counter, mutex, allocation, or instrumentation branch.

The armed workload is intentionally a different acceptance question. It
asked for a ceiling and currently receives an unenforced interval. The
surrogates show that enforcing it at every checkpoint may cost a great deal,
especially with retained trees. They do not establish that cost is acceptable
or prohibitive. The actual armed A/B is an explicit remaining gap, and any
regression must be fixed or reported for a director decision before landing.
An unarmed performance regression is not excused by an armed safety benefit.

If A's measured armed cost is rejected, D or F can be a deliberately partial
containment step while the accounting line is designed. D has the narrowest
predictable check cost; F has broader per-expression coverage and potentially
substantial traversal cost. E is also a real fix for this particular
constructor, but a collection of named-builtin patches is not proof that
the sampling gap is closed. B, without a proved growth envelope, is not a
defensible byte-bound repair.

C is the long-term route to cheap, predictive enforcement only after its
accounting contract is settled. The existing walker charges repeated shared
subtrees by occurrence; a counter of unique allocations would not reproduce
that model. It must account for capacity growth, expression import/sharing,
retention in definitions and caches, prints/traces/reaps, suspension and
abandonment, and release on all exits. Core constructors currently have no
universal session ledger. Incremental accounting of selected builtin
accumulators is useful, but is not the same as a complete session counter.
Cached subtree byte sizes are another way to reduce some walks; they would
change construction/storage costs and still need root accounting, so are
separate measured work, not an available shortcut here.

**Both public ceiling routes are in scope.**
`begin_top_level_interrupt_budget` sets `current_memory_limit` from
`resource_limits.eval_memory_limit` and resets the countdown. A nested
`MemoryConstrained` installs the minimum of its own limit and the active
one, then restores the prior limit on completion/abandonment. A repair of
the common checker therefore applies to both. Preserve the local fallback
or `$Aborted` behavior and the outer request's structured `MemoryLimit`
termination. The stock CLI exposes no setting for `eval_memory_limit`;
host behavior was source-verified here, not re-probed with a new Rust
harness, which the hard rules prohibit.

**The recommendation is falsified** if the reproduction still succeeds,
an oversized modeled checked state is missed at any armed phase, nested or
resumed frames escape the common policy, an ordinary supported workload is
wrongly refused under the stated model, or the measured cost is rejected.
It is also falsified if presented as a hard RSS bound: the native code can
allocate before the next check, and the model has explicit exclusions.
Closing the sampling defect must not be reported as closing every memory
finding. The interner, cache byte-policy work, and local-message discrepancy
remain separately identified work.

### 4. Falsifier and required A/B

**The negative instrument was actually watched failing.**
`reproduction-falsifier.rum` uses the exact seed, limit, operation, and
iteration count above and demands `SameQ[gOut, $Aborted]`. It prints only
the unexpected string length and exits 2 on failure. The paired control
demands `$Aborted` from the supplied `StringRepeat` control. Results were:

```text
{"label": "reproduction-falsifier", "exit": 2, "elapsed_monotonic_s": 0.14785536300041713}
FAIL: repeated growth returned 65536000 characters
{"label": "control-falsifier", "exit": 0, "elapsed_monotonic_s": 0.0021899049970670603}
PASS: projected allocation aborted
Null
```

This proves the proposed assertion rejects today's behavior while its
calibrated control succeeds. It is not a claim that an implementation or
a repair-removal mutation has already passed.

The implementation's certificate must include:

1. **Exact language regression, owned by `rum-stdlib`.** In a fresh core
   session, create the seed first, evaluate the exact `MemoryConstrained`
   expression, and assert the returned expression is `$Aborted`. Also
   assert the supplied control returns `$Aborted`; a generated-element
   error, unrelated evaluation failure, host panic, or process death is not
   success. Avoid formatting the unexpected large value in assertions.
   Keep the known 65,536,000-character case as a bounded subprocess
   certificate when running a broken implementation, rather than scaling
   toward the audit's unverified GB peak.
2. **Ordinary admission and fallback.** The same eight-step, 256,000-character
   workload must fit under the 8 MB frame; the sixteen-step workload must
   still succeed with no ceiling. Test the explicit local fallback, nested
   smaller/larger ceilings, and a following `1 + 1` to prove restoration.
   The sixteen-step admission control is a separately bounded memory-heavy
   run. Promote the checkpoint guarantee into the resource guide in the
   repair, so these tests cite rum's contract.
3. **A phase-independent kernel test.** Build a small expression with known
   proxy bytes greater than a positive active limit and invoke the common
   checker on the first, second, and subsequent checks after a reset.
   Advance the checker with fitting values to exercise each of the 64 old
   phases, using fresh state per phase. It must reject at every phase.
   Include equality with the limit versus one byte above it using the
   measured test-session baseline, and a fixture whose session roots alone
   exceed the limit while its current expression is small. This separates
   a genuine full-model repair from a string-only or result-only patch.
   If the counter field is removed, advance checks rather than adding it
   back for the test.
4. **Request/host contract.** In `rum-host`, arm
   `HostResourceLimits::eval_memory_limit = Some(8_000_000)`, seed in a
   prior successful request, and evaluate the bare `Nest` in one request.
   Require `TerminatedEvaluation["MemoryLimit"]`,
   `HostTerminationReason::MemoryLimit`, and `General::memlim`; then
   require a successful next request. Use the existing
   `host_eval_memory_ceiling_reports_structured_termination_and_session_recovers`
   as the contract pattern. Verify the exact base outcome rather than
   assuming its phase matches the local frame. If base already catches
   that host shape, report it as positive coverage, not a repair-sensitive
   negative test. The kernel phase test supplies the independent negative.
   Include a local frame inside a higher host ceiling and suspension/resume
   plus abandonment tests for active-limit restoration. Do not put a later
   successful script form after the host breach when checking termination;
   that tests request aggregation as well as the intended boundary.
5. **Repair-removal proof.** Keep tests unchanged, disable only A's new
   every-check policy by restoring the old 64-stride behavior with an
   explicit source edit, rebuild, and run the focused certificate. Require
   the exact language test and nonzero-phase kernel test to fail for the
   intended reason while the single-allocation control remains green.
   Reapply the repair and watch them pass. Save commands, exit codes, and
   raw logs for all three states. This mutation belongs to the authorized
   follow-up; none was made in this design pass. A whole-test skip, timeout,
   allocation failure, or unrelated compile failure is not the falsifier.

Use a small kernel phase test for routine coverage and the bounded exact
language witness for fidelity. Peak RSS can corroborate earlier refusal, but
do not make an exact RSS number a portable correctness assertion. The model
check and the returned memory outcome are the deterministic oracle.

**Required instruction A/B, replacing a timing-panel acceptance argument.**
Build plain release base and candidate from the same build path, with the
same compiler/options/allocator and diagnostics disabled, saving each
binary's hash. Run one workload per process through:

```text
scripts/run-bounded.py --timeout <calibrated-seconds> -- taskset -c 2 perf stat -e instructions:u -x, -o <arm-label-repetition.csv> -- <binary> <label.rum>
```

Require successful, identical liveness/value checks for performance rows;
a candidate that aborts a benchmark did less work and has no valid speedup.
Use fresh processes, a warm-up, at least seven alternating A/B pairs, and
retain individual counts, medians, spreads, and per-label deltas. Require a
numeric counted event at full coverage. If unavailable, record an explicit
gap; do not replace instruction counts with elapsed timings. Bound the
pilot and scale from a measured same-shape run. Execute serially with no
competing CPU-heavy task.

Required labels:

- **Default path, existing panel bodies:** `loop-empty`, `do-count-null`,
  `do-iterator-null`, `literal-integer`, `literal-string`,
  `symbol-ownvalue-read`, `set-ownvalue`, `list-literal-small`,
  `module-local`, `table-iterator-symbol`, `table-iterator-plus`,
  `downvalue-exact-hit`, `downvalue-pattern-hit`, `matchq-integer-success`,
  `replaceall-simple`, `string-repeat-small`,
  `appendto-list-incremental-small`, and `appendto-list-incremental-large`.
  These names were verified in `scripts/core-language-microbench.sh`.
  Use their setup/body as labeled standalone instruction workloads; timing
  output from a whole-panel run is not a per-label instruction measurement.
  Include the full existing panel's remaining labels before declaring the
  ordinary path unchanged; fix or report a same-tier neighbor regression.
- **Armed counterparts:** the same bodies with one
  `MemoryConstrained` around the repeated work, not one wrapper per
  iteration. Use the same generous ceiling in both arms, calibrated above
  peak model usage, with an explicit completion assertion. Label these
  `<existing-label>/armed`. Measure representative matching/rewrite,
  mutable-list, and callback rows under the per-request ceiling through a
  release host driver as well; that driver is follow-up scope, not an
  artifact built here.
- **Retained-root sensitivity:** `armed-null/root-empty`,
  `armed-null/root-string1m`, `armed-null/root-list1k`,
  `armed-null/root-list10k`, and `armed-null/definitions-2000`.
  The first four have scratch calibrations in this report. Add
  `armed-nest-string-below-limit` and its unarmed twin, and
  `armed-join-steady` / `unarmed-join-steady` with fixed-size inputs and
  discarded outputs to distinguish repeated sampling from output growth.
  Include a growing but admitted list workload to expose repeated full
  scans as its retained size increases.
- **Correctness-only rejection rows:** the exact reproduction and control,
  plus the host request version. Record outcomes and separately measured
  peak RSS. Do not compare the repaired early abort's instructions to a
  successful 65 MB construction as a performance improvement.

No percentage threshold is invented here. A reproducible default-label
regression is a defect to fix or report. Report all armed deltas, including
the worst retained-root row; their acceptance requires the decision below.
Passing unarmed labels alone does not establish that a configured ceiling
is affordable.

### 5. Draft follow-up briefs and director decisions

**Draft 513-A — enforce the existing model at every armed checkpoint.**
Shape: implementation candidate, then measured review; leave the tree
uncommitted.

- Goal: the common checker evaluates the full current proxy on every armed
  call, while an unarmed call follows the existing time/interrupt behavior.
  Preserve strict `projected > limit` refusal, saturating additions,
  unstrided preflight, nested-min semantics, and error consumption. Fix the
  false stride comment in the same change. Do not touch `StringJoin`, the
  growth-contract classifications, or the memory model to make this pass.
- Allowed implementation scope:
  `rum-runtime/src/runtime/debug_tools/resources/constraints.rs`; mechanical
  removal of the obsolete countdown only in `runtime/model/session.rs`,
  `runtime/session/session_state.rs`, `runtime/session/session_helpers.rs`,
  and its `runtime/tests/session_memory.rs` inventory entry. If removing the
  field changes unarmed code/layout, measure and report it. No new session
  counter or always-on diagnostic is needed.
- Tests: a focused runtime memory-check module; focused stdlib regressions
  beside `tests/debug_tools_resources.rs`; host contract cases beside
  `tests/eval_contract.rs`. Add only the module registrations needed for
  coherent test ownership. Implement section 4's exact and phase witnesses,
  admission controls, nesting, and continuation checks. Keep the exact
  high-memory negative certificate bounded and isolated.
- Docs: resource-limit guide and the TODO sampling finding. State the
  stronger checkpoint guarantee and residual allocation/model gaps. If
  changing builtin catalog documentation, regenerate its reference in the
  same change; do not hand-edit generated reference text. Leave the local
  message discrepancy as a separately recorded contract conflict, not a
  weakened assertion.
- Organization: `constraints.rs` currently measures 16,833 bytes and mixes
  time and memory constraints. Keep this repair small; if a helper split is
  needed, split time versus memory behavior, not arbitrary line ranges.
  Do not extend the 32,019-byte `memory.rs` walker for a sampling change.
  Record the applicable split assessment/TODO required by the agent guide.
- Gate: focused tests, then bounded `cargo test -p rum-runtime`,
  `cargo test -p rum-stdlib`, `cargo test -p rum-host`, and
  `cargo test --workspace` for the cross-crate certificate; `cargo fmt --check`.
  Reuse measured same-shape test/build durations to size bounds. Deliver the
  instruction A/B specified above, including armed costs. A failed
  performance gate is a result to report, not permission to change the
  algorithm or discard a label.
- Hard boundaries: safe Rust; rum-only documented contracts; no network,
  interner work, unrelated refactor, source/fixture weakening, or git
  commit/branch/switch/merge/push/stash/checkout/restore/index/config change.
  Scratch `.rum` files stay under the new brief's target directory.
  Never overwrite the supplied 502/508 artifacts. Stop a failed probe after
  two attempts when the second adds no information.

**Draft 513-B — independently falsify and gate the every-check candidate.**
Shape: review/certificate, not another accounting implementation.

- Re-probe the exact reproduction, allocation control, admitted work, and
  both host/local surfaces on the submitted candidate. Verify the supplied
  source diff cannot unconditionally walk or instrument the default path.
- Execute section 4's repair-removal mutation on the same tests and source
  baseline, making and undoing only the explicit checker edit. Require red
  for repeated growth/nonzero phase, green for the projection control, then
  all green with the repair re-applied. Inspect the failure reason, not only
  the exit code. No git restore/checkout or index operation.
- Repeat only performance rows whose candidate changed, whose measurement
  failed, or whose acceptance remains disputed. Audit full PMU coverage,
  same-path builds, completion assertions, retained-root sizes, and raw
  per-label evidence. Do not accept a whole-panel aggregate that hides an
  armed or same-tier regression. Report any earlier transient-allocation
  gap without relabeling it as repaired.
- Deliver the repair's exact guarantee, measured default/armed instruction
  deltas, mutation logs, and remaining findings. Apply the same bounds,
  safe-Rust, policy, scratch, artifact, and no-git-mutation rules as 513-A.

**Draft 513-C — conditional string-value containment, only if A's cost is
rejected.** Shape: explicitly partial implementation, alternative to landing
A; it does not close the general sampled-session finding.

- Add an always-executed **inside-the-armed-branch** necessary-condition
  check for an atomic string's existing proxy bytes. Keep full session
  sampling and projections. The condition applies to every string,
  regardless of producing builtin, loop driver, or source spelling. Do not
  recognize a particular callback or infer a growth factor from history.
- Scope: common checker and focused runtime/stdlib/host tests/docs; no
  `Expr` layout change, constructor instrumentation, global quota, or
  capability change. Keep the default branch unchanged and replace the
  false comment with the restricted guarantee.
- Require the exact reproduction/control and phase mutation witnesses,
  string admission at the modeled boundary, and a documented retained-small-
  values counterexample showing why this is containment. Do not claim a
  physical factor-2 bound. Gate unarmed neighbors and armed steady strings
  with the same instruction recipe. If the desired containment includes
  trees, re-brief F and measure its traversal cost; do not widen C silently.
- Deliver only after the director explicitly chooses the partial scope.
  Apply 513-A's hard boundaries and stop-loss rules.

**Incremental follow-up is not yet a dispatchable small implementation.**
Before drafting constructor edits, settle whether the charged quantity is
the current structural proxy or a newly documented residency proxy. Then
scope a reservation/ownership API and a compiler-checked inventory of
construction, root retention, import, and release sites. Its implementation
brief must require reserve-before-growth, rollback on failure, release on
normal/error/suspension-abandonment paths, independent walk reconciliation,
an omitted-charge mutation that demonstrably fails, and default-path
instruction measurements. A few accumulator counters plus an
`Incremental` enum label cannot stand in for that work. Interner bytes stay
out of this session ledger unless separately ruled otherwise.

**Director decisions and explicit gaps:**

- **Armed-cost acceptance:** authorize A as the candidate to measure, then
  decide from its actual per-label armed tax. No repaired-build A/B or
  repair-removal result exists in this design pass. If the cost is rejected,
  choose a named partial containment scope or a separately scoped accounting
  project; a heuristic adaptive factor is not a substitute.
- **Closure wording:** approve closure of the *sampling* defect at existing
  checkpoints, with the model's native-allocation and root limits stated.
  A requirement for reservation before every allocation selects the much
  larger accounting line, not this checker-only repair.
- **Separate contract conflict:** the documented local `memlim` emission
  disagrees with this measured control. Record a message-contract repair
  separately; do not resolve it by weakening the guide or silently adding it
  to the performance patch.
- **Interner:** no quota or ownership decision is sought or made here.
  Continue that already separate ruling/measurement line independently.

The design deliverables are complete; implementation, patched-host probes,
full correctness suites, and actual candidate A/B remain the expressly
drafted follow-up work, not tests claimed to have passed here.
