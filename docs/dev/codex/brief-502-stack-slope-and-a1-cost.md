# brief-502 — what A1 costs in stack bytes, instructions and allocations

## Goal

Two measurements on the candidate representation already in the tree, in this
order, because the first one can end the line on its own:

1. **Stack bytes per level.** brief-501's gate found a 48-level recursion that
   succeeds on stage1 and reports `TerminatedEvaluation["StackLimit"]` on the
   candidate — in **debug**; release passes. Type sizes did not change. So the
   candidate costs more stack per level than stage1, and neither an
   instruction count nor a `size_of` would ever have shown it.
2. **Instructions and allocations** — brief-501's Stage C, which its stop
   never reached.

**Diagnostic only. No source change, no repair, nothing lands.** If you find
the mechanism, report it; do not fix it.

## Where the tree is

Branch `spike/brief-499-symbol-non-copy` at `45522235`, which carries the
candidate representation: `Arc<SymbolText>` with owned `String`s, a seedless
FNV-1a content fingerprint, `Arc::ptr_eq` equality and ordering fast paths,
`interned_id` as the low 32 bits of the fingerprint. The interner is still
process-global and still leaking, deliberately.

`main` is clean at `3024c33c`. **Do not switch, merge, rebase, commit or
stash.** The director has built every reference binary you need.

## The arms, already built for you

All built by the director at this same path, so layout is comparable:

| Path | Commit | Profile | Features |
|---|---|---|---|
| `target/brief-502/rum-base` | `3024c33c` main | release | default |
| `target/brief-502/rum-stage1` | `a9dbc0c0` | release | default |
| `target/brief-502/rum-stage1-debug` | `a9dbc0c0` | debug | default |
| `target/brief-502/rum-base-alloc` | `3024c33c` main | release | `alloc-count` |
| `target/brief-502/rum-stage1-alloc` | `a9dbc0c0` | release | `alloc-count` |

`stage1` is non-`Copy` `Symbol` with the **old** representation; `base` is
today's kernel. You build the candidate's three binaries from the working
tree, in place: release default, debug default, and release with
`--features alloc-count`.

**brief-501 was right that the earlier pair had no allocation counters** —
`alloc-count` is an off-by-default `rum-cli` feature (`crates/rum-cli/Cargo.toml:17`)
and the plain binaries cannot report counts. That is fixed above. **Never mix
them**: allocation counts come only from the `-alloc` binaries, instruction
counts only from the plain ones, because the counting allocator adds per-alloc
cost.

## Stage 1 — stack bytes per level (write this section before starting stage 2)

### What is known

- The failing test is
  `runtime::tests::holding_and_control_part2::bounded_recursive_definition_below_native_stack_limit_succeeds`
  (`crates/rum-runtime/src/runtime/tests/holding_and_control_part2.rs:488-505`):
  it peels `wrap[...]` at **depth 48** through two rewrite rules and expects
  `done`. Director-reproduced: fails in debug on the candidate, **passes in
  release on the candidate**, passes in both on stage1.
- Sizes are identical across arms: `Symbol` 24, `Atom` 32, `Expr` 32,
  `SharedExpr` 8.
- The contract this threatens is in
  `docs/guide/resource-limits-and-sandbox.md:151` — 4 096 links accepted, link
  4 097 refused with its operator token — and the line that set it measured a
  **184 B prologue and 192 B/level** by the `/proc` VmStk method
  (`docs/dev/codex/brief-446-stability-poll-placement.md:110-125`).

### The hypothesis, which you must confirm or refute rather than assume

On stage1 `Symbol` is non-`Copy` but its drop is **trivial** — it is a plain
struct of `Copy` fields. On the candidate it owns an `Arc`, so its drop glue
becomes an atomic decrement with a cold deallocation path that frees
`SymbolText` and two `String`s. Every frame that must be able to drop a
`Symbol`, or an `Expr` containing one, therefore carries more drop and unwind
state — most visibly in debug, where nothing is inlined away.

Report `std::mem::needs_drop::<Symbol>()`, `::<Atom>()` and `::<Expr>()` for
both arms. Note before you start that `Expr` already needed drop on stage1
(it holds `String` and `Arc`), so a flip is not the whole story and the
interesting quantity is the **size of the glue**, not merely its existence.

### The measurement

Use the `/proc` VmStk high-water method the traversal-depth line settled on;
gdb ptrace is denied in this sandbox, so do not spend time on it and do not
quote a number you could not take. Sample at four or more heights, fit the
slope, and report prologue and bytes/level for **each arm in each profile**:

| arm | profile | prologue B | B/level | slope R² or residuals |
|---|---|---|---|---|

Arms: `stage1` and `candidate`, debug and release — four rows. Include `base`
in release as the reference the 192 B/level figure was taken against.

Then the two consequences:

- **The depth bracket**, per arm per profile, at `ulimit -s 2048` — the ruled
  2 MiB embedder stack. Report the **last height that succeeds and the first
  that fails**, not a single number: this project's records show a ceiling
  flipping inside a band tens of levels wide, so a point estimate is not a
  result. Use the `peel`/`wrap` shape of the failing test, driven through the
  CLI as a `.rum` file, so no source changes.
- **The contract check**, release, `ulimit -s 2048`: does the candidate still
  accept 4 096 links and refuse 4 097 in band, and what is the physical-height
  margin against the 30% the line recorded? A margin that has gone is a
  finding that outranks everything in stage 2.

## Stage 2 — instructions and allocations

Only after stage 1's section is written.

### The instrument

```
taskset -c 2 perf stat -e instructions:u -x, -- <binary> <workload>.rum < /dev/null
```

`cpu_core/instructions/u`, `100.00` multiplex coverage required, three runs
per arm per workload, profiling env OFF, stdin closed or the probes hang.
Judge deltas against the observed within-arm spread. **Do not run the timing
panel** — its own untouched controls have moved +3.70% to +12.45% on this
host, while this recipe repeats to 0.00005%.

### Calibration the director already did — read before writing a workload

**The iteration ceiling will silently eat your probe.** A `Do` over 200 000
iterations dies with `evaluation exceeded iteration work limit (100000)`, exits
1 after ~3.4M instructions, and both arms then read the same startup cost.
This instrument has lied that way three times in this project's records.

So **verify the instruction count scales with `N` before trusting any number.**
On the supplied `target/brief-501/smoke25000.rum` and `smoke50000.rum` the base
arm reads 689 784 941 and 1 376 080 468 — ratio 1.995, which is what proves the
loop ran. Every workload needs a liveness assertion *and* a scaling check.

Measured baseline, three runs each on `smoke50000.rum`:

```text
rum-base    1376080436 1376080812 1376081173     spread 0.00005%
rum-stage1  1376848929 1376848933 1376850995     spread 0.00015%
```

`base → stage1` is **+0.056%**: the ownership migration alone costs about
nothing, and not exactly zero. A candidate delta of that order on workload 4
is the migration, not the representation.

### Three arms, four workloads

`base → stage1` isolates the migration, `stage1 → candidate` the
representation, `base → candidate` the total — **and the total decides A1**,
because the migration is not separable from it. Report all three deltas.

1. **Cold intern** — dominated by first-time interning of distinct names;
   `Symbol[StringJoin["...", ToString[i]]]` over a `Table` is the shape the
   panel uses (`scripts/core-language-microbench.sh:884-891`).
2. **Warm intern** — the same names re-interned: lookup plus `Arc` clone
   rather than allocation.
3. **Symbol-hot evaluation** — the EXACT statements of these labels, at their
   own iteration counts, nothing added around them: `literal-integer`
   (`scripts/core-language-microbench.sh:479`, the control that should not
   move), `symbol-ownvalue-read` (`:481`), `set-ownvalue` (`:487`),
   `module-local` (`:497`), `downvalue-exact-hit` (`:530`),
   `matcher-single-tiny` (`:569`), `replaceall-simple` (`:586`),
   `sort-symbol-default-width4000` (`:982`), `...width16000` (`:988`).
4. **Evaluation-dominated ordinary** — `target/brief-501/smoke50000.rum` is
   already written and calibrated; use it as-is or replace it and say which.

Then the allocation table, from the `-alloc` binaries only, in their own runs:
counts per workload per arm. The candidate allocates two `String`s and an
`Arc` where the base leaked two `&str` and a `SymbolText`, so cold intern must
move; a move on workload 3 or 4 is a finding.

**Report the split; do not tune.** Acceptance is the director's.

## Scope

- **No source change.** This brief edits no `.rs` file. If you believe a
  measurement requires one, STOP and report.
- You MAY write under `target/brief-502/**` (workloads, captures, logs) and
  append your report to this file. Nothing else.
- **MUST NOT touch**: any `crates/**` file, `scripts/**`, `docs/**` except
  this file, `AGENTS.md`, `TODO.md`, `PROGRESS.md`, `Cargo.toml`,
  `Cargo.lock`, `.git`, and the ownership-graph inventory fixtures.

## Verification

There is no test gate, because there is no source change — the evidence is the
deliverable. Confirm once, and paste it, that `git status --porcelain` shows
no modification under `crates/`.

**About the workspace suite, so it is not re-litigated.** At `a9dbc0c0` the
whole workspace is 2 473 passed / 1 failed / 57 ignored, the one failure being
the ownership-graph inventory row, which is expected churn the director
verified. brief-501 then reported twelve more, and the director triaged them:

- **nine native ZeroMQ failures are environmental** — your sandbox cannot bind
  loopback. The brief should have told you to run the suite **without**
  `RUM_ALLOW_NATIVE_ZEROMQ_SKIP` and report those nine as environmental,
  saying plainly that those tests did not execute. That omission was the
  director's.
- **two ownership-graph ledger failures** are the same coordinate churn as the
  inventory row.
- **one is real** and is this brief's stage 1.

You are not asked to run the suite. If you run one anyway, use that rule.

## Hard rules

- Do NOT `git commit`, branch, switch, merge, push, stash, `git checkout`, or
  touch git config or the index. (`git checkout` restores from the index,
  which on uncommitted output is a delete.)
- **No new `unsafe`** and no source change at all in this brief.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only; no oracle harnesses, no third-party engine comparisons.
- No network. Touch nothing outside this repo. Commands: writes under
  `target/brief-502/`, `cargo build` for the three candidate binaries,
  `perf`/`taskset`, the binaries named above, `scripts/run-bounded.py`,
  `python3` for parsing, and `ulimit` in your own shell.
- Do not measure while another CPU-heavy workload runs. Build first, then
  measure.
- **Stop-loss, keyed on progress.** If a measurement fails twice and the
  second attempt produced no more information than the first, stop and report
  what you have. Do not keep re-running a probe that is not converging.
- Paste real verbatim output. If a stage did not produce its evidence, say so
  — never fake or interpolate a number.

## Report format (your final message, and appended to this file)

1. **Stage 1** — the `needs_drop` answers, the four-row slope table, the depth
   brackets, the contract check, and your verdict on the hypothesis:
   confirmed, refuted, or undetermined with what would settle it. Written
   before stage 2 is attempted.
2. **Stage 2** — the four workloads × three arms × three runs, within-arm
   spreads, the three deltas per workload, and the allocation table.
3. **What you ran** — every command, and the `git status --porcelain` proof of
   no source change.
4. **Verdict** — what the numbers say about A1 versus the A2 fallback, stated
   as evidence rather than a decision, plus what this still does not
   establish: session ownership, reclamation on `reset()`, escaped-value
   lifetime, and cross-session equality cost.
5. **Stop-loss / open questions** — where you stopped and why; empty if none.

## Execution report — brief-502

### 1. Stage 1 — completed before Stage 2

The candidate increases the debug `peel` stack slope by **11.39%** relative
to stage1 and reproduces the depth-48 failure. Its release `peel` slope is
**3.36% smaller** than stage1's. The release stability walk retains its
physical-height margin: the candidate's conservative passing endpoint is
**5,934**, or **44.84% above 4,097**, against the historical 30%. The
4,096/4,097 source-chain boundary also passes its release check.

The supplied reference binaries were used unchanged. HEAD was
`2a05ca67a4403df9b79d5b59988d34809d604307`; `git diff --stat 45522235 HEAD`
showed only this brief, so the candidate source is the representation named
in the instructions. All three candidate builds completed offline with
Rust 1.85.0 before any runtime measurement. Binary hashes and build logs are
under `target/brief-502/logs/`.

**Drop requirements.** These are structural answers from the actual type
definitions, not output from a newly compiled Rust probe (none was written).
The supplied `target/brief-501/expr-stage1.rs` has only `u32`,
`Option<NonZeroU64>`, and `&'static SymbolText` in `Symbol`; the candidate
owns `Arc<SymbolText>`. Both `Atom` definitions already own `String` and
boxed numeric data, and both `Expr` definitions already own `Atom` and
shared call data.

| `std::mem::needs_drop::<T>()` | stage1 | candidate |
|---|---|---|
| `Symbol` | false | true |
| `Atom` | true | true |
| `Expr` | true | true |

The linked debug binaries corroborate this: no standalone stage1
`drop_in_place<Symbol>` was found; candidate glue has 13 code bytes for
`Symbol` and 100 for `SymbolText`. Debug `drop_in_place<Atom>` grows
107 → 121 code bytes; the outer `drop_in_place<Expr>` stays 145 bytes and
delegates to the changed child glue. There were no emitted `needs_drop`
symbols to invoke in either binary. Release `Atom` glue has observed
188 → 205-byte instances and general `Expr` glue 299 → 319-byte instances;
multiple optimized instances exist, so these are not total linked glue sizes.
All sizes come from ELF symbol records parsed with Python; raw function
prefixes are in `stack/elf-functions.json`.

**Method.** Each CLI process had stdin closed, profiling environment
variables removed, CPU 2 affinity, and `ulimit -s 2048`. Core dumps were
disabled in that child shell. `/proc/<pid>/status` was sampled about every
0.2 ms. The fitted runs retain their result in `gStackResult`, pause
`1/20` second after evaluation, then return that result; this preserves the
stack mapping long enough to sample its high-water mark. The pause is
outside the recursive operation. The initial pilot lacked that plateau
and missed some short release peaks; its VmStk readings were excluded
from every fit. It still supplied valid outcome evidence.

There are three fresh processes per height. Ordinary least squares uses
all samples, with an intercept; pages are 4,096 bytes and ASLR affects
the intercept. `prologue` below is read from actual x86-64 pushes and
`sub rsp` bytes, not inferred from VmStk. No ptrace or debugger was used.

`peel` uses the exact two definitions from the failing test and
`peel[Nest[wrap, done, N]]`; `N` counts `wrap` nodes. This avoids the
independent 128-delimiter parser limit at larger heights. The separate
literal-depth-48 probe uses the test's fully written-out expression.
The prologue column in this first table names
`evaluate_inner_unstable`, **one resident frame, not the total recursive
packet**. The fit includes every frame in the observed descent.

| arm | profile | prologue B | fit intercept B | B/level | R² | max absolute residual B |
|---|---|---:|---:|---:|---:|---:|
| stage1 | debug | 984 | 96392.5 | 20821.33 | 0.999871 | 4915.2 |
| candidate | debug | 1272 | 104448.0 | 23193.60 | 0.999855 | 5734.4 |
| base | release | 1064 | 44316.2 | 7580.08 | 0.999888 | 5153.5 |
| stage1 | release | 1064 | 39187.5 | 7650.21 | 0.999928 | 3679.0 |
| candidate | release | 1096 | 39430.8 | 7393.28 | 0.999884 | 5024.4 |

The stability workload constructs a nested list iteratively, rebuilds its
leaf using `Map[deepMapProbe, x, {-1}]`, and checks the complete returned
`InputForm` string against Python's exact expected string. This exercises
a fresh stability traversal rather than a cache hit. Here `H` is the
**physical height of the rebuilt result**: `H-2` list wrappers around
`deepMapProbe[1]`.

The debug stability packet consists of the prologues of
`cached_expr_stability_with_bloom` (168 B),
`call_stability_with_bloom` (440 B), and
`call_stability_uncached_with_bloom` (440 B): **1,048 B**, plus three
8-byte return addresses, giving **1,072 B/level statically**.
Release inlines that chain into the 168-byte cached-expression frame,
giving **176 B/level statically**. These prologues are identical across
the applicable arms. The supplied modern base therefore measures about
176 B/level, not the historical brief-446 192 B/level; the old figure
was not substituted for a measurement.

| arm | profile | prologue B | fit intercept B | B/level | R² | max absolute residual B |
|---|---|---:|---:|---:|---:|---:|
| stage1 | debug | 1048 | 87381.3 | 1078.61 | 0.999910 | 3003.7 |
| candidate | debug | 1048 | 90794.7 | 1082.03 | 0.999754 | 6417.1 |
| base | release | 168 | 38912.0 | 175.85 | 0.999906 | 3140.3 |
| stage1 | release | 168 | 35498.7 | 177.49 | 0.999953 | 2730.7 |
| candidate | release | 168 | 42325.3 | 174.76 | 0.999878 | 4096.0 |

Raw VmStk samples (KiB; three runs at each height):

| arm/profile | shape | height: samples KiB |
|---|---|---|
| stage1-debug | peel | 8: 256,256,256; 16: 420,420,424; 24: 584,580,580; 32: 744,748,740; 40: 908,912,904 |
| stage1-debug | stability | 160: 256,252,252; 320: 424,424,424; 480: 592,588,588; 640: 760,760,760 |
| candidate-debug | peel | 8: 280,284,280; 16: 468,468,468; 24: 640,648,648; 32: 824,824,824; 40: 1008,1012,1008 |
| candidate-debug | stability | 160: 256,264,256; 320: 424,428,424; 480: 596,596,596; 640: 768,760,768 |
| base | peel | 32: 284,280,280; 64: 516,520,512; 96: 752,752,752; 120: 932,936,932 |
| base | stability | 1000: 208,212,208; 2000: 380,384,384; 3000: 552,552,552; 4000: 728,724,724 |
| stage1 | peel | 32: 280,276,276; 64: 516,516,520; 96: 752,752,756; 120: 936,936,936 |
| stage1 | stability | 1000: 208,208,208; 2000: 380,380,384; 3000: 552,556,556; 4000: 728,728,728 |
| candidate | peel | 32: 268,272,268; 64: 500,504,500; 96: 732,732,728; 120: 908,908,900 |
| candidate | stability | 1000: 212,212,212; 2000: 384,380,380; 3000: 556,556,556; 4000: 720,724,724 |

**Depth brackets.** Outcome-only runs omit the result assignment and pause
and evaluate `peel[Nest[wrap, done, N]]` directly. Every height in the
reported local scan was run three times. All failures below were handled
native-stack-limit errors, exit 1; there were no aborts or timeouts.

| arm | profile | last successful wrap count | first failing wrap count | scanned neighborhood |
|---|---|---:|---:|---|
| stage1 | debug | 51 | 52 | 46–58 |
| candidate | debug | 47 | 48 | 42–54 |
| base | release | 139 | 140 | 134–146 |
| stage1 | release | 139 | 140 | 134–146 |
| candidate | release | 143 | 144 | 138–150 |

No repeated height had mixed outcomes in these runs. The release
stability boundary is **non-monotone across neighboring heights**, however,
so a single bisection point would have hidden the band:

| arm, release | passing lower endpoint | first observed failure | last observed success | failing upper endpoint | local scan |
|---|---:|---:|---:|---:|---|
| base | 5930 | 5931 | 5938 | 5939 | 5911–5959 |
| stage1 | 5930 | 5931 | 5938 | 5939 | 5911–5959 |
| candidate | 5934 | 5935 | 5942 | 5943 | 5911–5959 |

The candidate's conservative margin is `(5934 - 4097) / 4097 =
44.84%`; its last observed success is 5,942. Base/stage1's conservative
margin is `(5930 - 4097) / 4097 = 44.74%`. These are physical expression
heights at the **configured 1 MiB evaluator budget on a 2 MiB stack**,
not unguarded OS-stack exhaustion measurements.

**Contract check.** Each release arm evaluates a `.rum` file with 4,096
`+` links to `4097`, exit 0. At 4,097 links each reports the explicit
rejected `+` at bytes `8193..8194`, exit 1. The complete long diagnostic
and operator caret are preserved in each `stack/contract-4097-*-r1.stderr`.
Verbatim candidate success output and the first two rejection lines:

```text
4097
```

```text
error: left-associative chain exceeded limit 4096 at bytes 8193..8194
 --> 1:8194
```

The source-chain parser boundary and the rebuilt-tree stability margin
are distinct checks; successful flattening of the `+` source alone would
not establish a deep-traversal margin.

Verbatim literal-depth-48 candidate debug stderr (exit 1, empty stdout):

```text
! evaluation exceeded native stack limit (1048576 bytes)
error: evaluation exceeded native stack limit (1048576 bytes)
```

Stage1 debug, stage1 release, base release, and candidate release each
returned `done` with exit 0. The CLI host renders a structured failure
as the diagnostic above; it does not print the kernel test's
`TerminatedEvaluation["StackLimit"]` value on this path.

**Hypothesis verdict: confirmed for the added debug drop/unwind state,
with limited attribution of the full slope.** The representation adds
real ownership/drop glue and larger resident debug frames while type
sizes stay unchanged. Concrete debug prologue/code-size pairs are:

| frame | stage1 prologue / code B | candidate prologue / code B |
|---|---:|---:|
| `evaluate_inner_unstable` | 984 / 1502 | 1272 / 2495 |
| `evaluate_call` | 1288 / 1788 | 1672 / 3102 |
| `resume_call_after_head` | 552 / 735 | 728 / 1359 |
| `resume_call_with_dispatch_plan` (one instantiation) | 1768 / 5203 | 1816 / 5429 |

A specific propagation path is diagnostic state:
`RuntimePathTimingToken` contains a `RuntimePathTimingKey` with
`Option<Symbol>` (`profiling/mod.rs:412,469`). The token formerly had trivial
drop; it now needs drop even though `runtime_path_timing_start` returns
`None` with profiling off (`profiling/timing.rs:97`). The evaluator holds
several such optional tokens across calls. Their extra debug stack slots
and drop flags are visible in the saved machine-code prefixes. The
stability path's unchanged frames demonstrate that this is not universal
growth in every function. The release `peel` result also refutes a blanket
claim that the representation increases stack slope in both profiles.

This establishes the added glue, the affected resident frames, and the
debug regression together; it does not assign every one of the measured
extra 2,372.27 B/peel level exclusively to unwind bookkeeping rather than
other code-generation effects. A controlled representation/drop-state
ablation or compiler frame-liveness analysis would settle that narrower
attribution; no source ablation was performed.

Stage 1 was written and appended before any Stage 2 workload generation,
`perf` invocation, or allocation-counter run.

### 2. Stage 2 — instructions and allocations

**All evidence was produced.** There were 369 formal instruction runs
(41 workload/control variants × three arms × three runs), plus one
initial instrument check; all formal runs had exit 0, their required
liveness output, and `cpu_core/instructions/u` coverage `100.00`.
There were 234 separate allocator runs (26 variants × three arms × three
runs), all with their liveness output and counters. No timing panel ran.

Every instruction run used a **plain release binary**, this recipe, and
a clean profiling environment; stdin was `/dev/null`:

```sh
taskset -c 2 perf stat -e instructions:u -x, -- <binary> <workload>.rum < /dev/null
```

CPU 2 is a core PMU CPU. The extra `cpu_atom/instructions/u` row is
`<not counted>`/0.00 on that CPU; the requested **core** event has full
coverage in every accepted run. Verbatim ordinary candidate run 1:

```text
<not counted>,,cpu_atom/instructions/u,0,0.00,,
1573312997,,cpu_core/instructions/u,154749171,100.00,,
```

The observed ordinary-workload spreads here are 0.000233% / 0.000050% /
0.000301%, not an assumed reuse of the director's earlier spread.
All candidate deltas below exceed the observed spreads.

**Workloads and liveness.** The seven `rumMicroBench` expressions and
iteration counts were checked against the actual panel source and used
as the **bare body of one `Do`**, with no per-iteration sink assignment,
counter, wrapper, or additional expression. Each sort uses its exact
`Sort[gMicroSortSymbolWidth...]` statement once at the named width,
with the panel's `Reverse[Table[Symbol[StringJoin[...]]]]` input shape.
No `AbsoluteTiming` or timing harness was imported.

A final `Print["BRIEF502_LIVE"]` is outside the operation; the driver
requires exactly `BRIEF502_LIVE\nNull\n` and exit 0. Thus a resource-limit
failure cannot be accepted as an instruction measurement. The ordinary
workload uses the supplied `smoke50000.rum` **as-is**, including its
exact sum assertion, and the supplied `smoke25000.rum` for scaling.

| workload | full size | half-size check | exact operation |
|---|---:|---:|---|
| cold intern | 16,000 distinct names | 8,000 | `Table[Symbol[StringJoin["gBrief502Intern", ToString[i]]], {i, 1, N}]` |
| warm intern | same 16,000 names, second pass | 8,000 | same bare `Table` after a separate seed table |
| literal-integer | 2,000 iterations | 1,000 | `123` |
| symbol-ownvalue-read | 2,000 | 1,000 | `xOwn`, initialized to `123` |
| set-ownvalue | 1,000 | 500 | `xOwn = 123` |
| module-local | 500 | 250 | `Module[{x = 1}, x + 1]` |
| downvalue-exact-hit | 2,000 | 1,000 | `fExact[1]`, with `fExact[1] = 2` |
| matcher-single-tiny | 1,000 | 500 | `MatchQ[gMicroWideMatcher[1, 2, 3, 4], gMicroWideMatcher[1, 2, 3, 4]]` |
| replaceall-simple | 500 | 250 | `a /. rulesSimple`, panel rules `{a -> 1, b -> 2, c -> 3}` |
| sort-symbol-default-width4000 | one call, width 4,000 | width 2,000 | `Sort[gMicroSortSymbolWidth4000]` |
| sort-symbol-default-width16000 | one call, width 16,000 | width 8,000 | `Sort[gMicroSortSymbolWidth16000]` |
| ordinary | 50,000 iterations | 25,000 | supplied arithmetic/ownvalue smoke |

Full-process instruction counts follow. Each spread is
`100 * (max - min) / median`. The empty-process row is a separate
startup diagnostic, not one of the four requested workload families.

| workload | arm | run 1 | run 2 | run 3 | spread % |
|---|---|---:|---:|---:|---:|
| cold-intern | base | 411224425 | 411207250 | 411258370 | 0.012431 |
| cold-intern | stage1 | 411698566 | 411688358 | 411631350 | 0.016327 |
| cold-intern | candidate | 486792069 | 486879561 | 486801405 | 0.017973 |
| warm-intern | base | 788933247 | 789012737 | 788858079 | 0.019603 |
| warm-intern | stage1 | 789781237 | 789737662 | 789870682 | 0.016843 |
| warm-intern | candidate | 912692273 | 912910203 | 912741767 | 0.023876 |
| literal-integer | base | 5173658 | 5175010 | 5176908 | 0.062802 |
| literal-integer | stage1 | 5218023 | 5218507 | 5218530 | 0.009715 |
| literal-integer | candidate | 5912032 | 5910989 | 5912273 | 0.021718 |
| symbol-ownvalue-read | base | 7694330 | 7694309 | 7696133 | 0.023706 |
| symbol-ownvalue-read | stage1 | 7733610 | 7731215 | 7731059 | 0.032996 |
| symbol-ownvalue-read | candidate | 8896569 | 8897392 | 8896258 | 0.012746 |
| set-ownvalue | base | 8053784 | 8050456 | 8051569 | 0.041334 |
| set-ownvalue | stage1 | 8064550 | 8064292 | 8064245 | 0.003782 |
| set-ownvalue | candidate | 9586104 | 9584431 | 9584452 | 0.017455 |
| module-local | base | 15041354 | 15041019 | 15041302 | 0.002227 |
| module-local | stage1 | 15083400 | 15082831 | 15082430 | 0.006431 |
| module-local | candidate | 17362284 | 17364131 | 17371535 | 0.053276 |
| downvalue-exact-hit | base | 15328400 | 15326559 | 15328711 | 0.014039 |
| downvalue-exact-hit | stage1 | 15429336 | 15430207 | 15430617 | 0.008302 |
| downvalue-exact-hit | candidate | 18204388 | 18202513 | 18203436 | 0.010300 |
| matcher-single-tiny | base | 15050564 | 15053653 | 15051997 | 0.020522 |
| matcher-single-tiny | stage1 | 15150218 | 15143202 | 15150881 | 0.050686 |
| matcher-single-tiny | candidate | 16730087 | 16730852 | 16729691 | 0.006940 |
| replaceall-simple | base | 9196171 | 9194148 | 9194638 | 0.022002 |
| replaceall-simple | stage1 | 9238071 | 9240846 | 9243133 | 0.054779 |
| replaceall-simple | candidate | 10410225 | 10398414 | 10398097 | 0.116633 |
| sort-symbol-default-width4000 | base | 112987810 | 113017327 | 113024292 | 0.032280 |
| sort-symbol-default-width4000 | stage1 | 113132749 | 113151976 | 113158456 | 0.022719 |
| sort-symbol-default-width4000 | candidate | 132990567 | 132982105 | 132999362 | 0.012976 |
| sort-symbol-default-width16000 | base | 443044555 | 443033214 | 443099066 | 0.014864 |
| sort-symbol-default-width16000 | stage1 | 443522678 | 443551388 | 443562979 | 0.009086 |
| sort-symbol-default-width16000 | candidate | 521415972 | 521420768 | 521463864 | 0.009185 |
| ordinary-smoke | base | 1376125882 | 1376122676 | 1376124245 | 0.000233 |
| ordinary-smoke | stage1 | 1376893934 | 1376894151 | 1376893460 | 0.000050 |
| ordinary-smoke | candidate | 1573312997 | 1573312967 | 1573317702 | 0.000301 |
| empty-process | base | 3363888 | 3367119 | 3364933 | 0.096020 |
| empty-process | stage1 | 3383583 | 3382210 | 3381440 | 0.063361 |
| empty-process | candidate | 3930416 | 3932457 | 3931966 | 0.051908 |

**Deltas and the setup split.** Gross columns compare medians of the
complete processes above. Payload columns subtract a separately measured,
otherwise matching zero-iteration control; warm intern subtracts the
same cold seed at each size, and sorts subtract the same prepared input
at each width. Controls are also three runs per arm. These differences
are estimates of the additional operation in this CLI context, not
isolated machine-instruction counts for `Arc` or the comparator.

This split matters: the candidate empty process itself costs **16.85%**
more instructions. A fresh-process short control therefore includes
changed startup. Warm gross includes its initial cold pass, and sort
gross includes cold interning of its input names.

| workload | gross base→stage1 % | gross stage1→candidate % | gross base→candidate % | payload base→stage1 % | payload stage1→candidate % | payload base→candidate % |
|---|---:|---:|---:|---:|---:|---:|
| cold-intern | +0.1128 | +18.2451 | +18.3785 | +0.1090 | +18.2668 | +18.3958 |
| warm-intern | +0.1075 | +15.5689 | +15.6932 | +0.1113 | +12.6984 | +12.8237 |
| literal-integer | +0.8405 | +13.2897 | +14.2419 | +1.3542 | +8.1009 | +9.5648 |
| symbol-ownvalue-read | +0.4794 | +15.0734 | +15.6250 | +0.4583 | +14.4396 | +14.9640 |
| set-ownvalue | +0.1580 | +18.8505 | +19.0383 | -0.1653 | +21.0534 | +20.8533 |
| module-local | +0.2761 | +15.1251 | +15.4430 | +0.1907 | +14.9166 | +15.1358 |
| downvalue-exact-hit | +0.6642 | +17.9727 | +18.7563 | +0.6954 | +18.6402 | +19.4653 |
| matcher-single-tiny | +0.6525 | +10.4280 | +11.1486 | +0.6762 | +8.8463 | +9.5823 |
| replaceall-simple | +0.5026 | +12.5266 | +13.0922 | +0.4294 | +10.5110 | +10.9855 |
| sort-symbol-default-width4000 | +0.1191 | +17.5327 | +17.6727 | -0.1107 | +5.7954 | +5.6783 |
| sort-symbol-default-width16000 | +0.1144 | +17.5559 | +17.6904 | +0.1586 | +5.9750 | +6.1431 |
| ordinary-smoke | +0.0559 | +14.2654 | +14.3293 | +0.0546 | +14.2620 | +14.3243 |
| empty-process | +0.5134 | +16.2543 | +16.8512 | — | — | — |

For payload differences, an observed range bound is
`(range(full) + range(control)) / payload`. The largest bound is
0.9643% for the stage1 width-4,000 sort; candidate representation/total
deltas exceed the relevant arm bounds even there. Small migration-only
sort differences do **not** exceed those bounds and are not called real.

The integer control **does move**: +14.24% gross and +9.56% after its
matching zero-loop control. It is reported as a finding. A bare `123`
body still runs inside the specified symbol-iterator `Do` and evaluator;
these counts do not attribute that change to integer arithmetic.
No alternative loop, extra per-iteration work, or timing measurement
was substituted to make the control look neutral.

**Scaling evidence.** Each cell is gross `I(N)/I(N/2)` followed by
`(I(N)-I(setup,N))/(I(N/2)-I(setup,N/2))`. Loop controls use `N=0`;
warm/sort setup controls retain the respective full/half fixture.
All underlying counts are in `cost/summary.json` and raw captures.
This avoids demanding raw doubling from short probes dominated by startup.

| workload | base gross / payload ratio | stage1 gross / payload ratio | candidate gross / payload ratio |
|---|---:|---:|---:|
| cold-intern | 1.984690 / 2.001409 | 1.984773 / 2.001570 | 1.985737 / 2.002189 |
| warm-intern | 1.991367 / 1.998714 | 1.991022 / 1.997733 | 1.992138 / 1.999681 |
| literal-integer | 1.201718 / 1.985954 | 1.202913 / 1.985621 | 1.192431 / 1.990726 |
| symbol-ownvalue-read | 1.379305 / 1.997596 | 1.378971 / 1.996744 | 1.376134 / 1.996879 |
| set-ownvalue | 1.397302 / 1.993917 | 1.395199 / 1.992792 | 1.406752 / 1.997166 |
| module-local | 1.587674 / 1.924686 | 1.586939 / 1.924800 | 1.591347 / 1.936518 |
| downvalue-exact-hit | 1.629822 / 1.998524 | 1.630177 / 1.998596 | 1.635766 / 1.998127 |
| matcher-single-tiny | 1.624554 / 1.999234 | 1.624587 / 1.998830 | 1.610142 / 1.998750 |
| replaceall-simple | 1.448705 / 2.002728 | 1.448552 / 2.003719 | 1.435999 / 2.000497 |
| sort-symbol-default-width4000 | 1.943159 / 1.939824 | 1.943133 / 1.941048 | 1.944856 / 1.937728 |
| sort-symbol-default-width16000 | 1.988063 / 1.999499 | 1.988096 / 1.998893 | 1.988370 / 1.995759 |
| ordinary-smoke | 1.994877 / 1.999970 | 1.994855 / 1.999975 | 1.994792 / 1.999971 |

The ordinary ratio is about 1.995 gross and 1.99997 after the zero
control, proving the 50,000-iteration probe was not swallowed by the
100,000-work ceiling. Every other probe also grows with its requested
size and reaches its final sentinel. `Module` and the 4,000-wide sort
give about 1.92–1.94 after subtraction, so they are not described as
exactly linear; the observed counts, rather than an interpolated count,
are used in the comparison.

**Allocations.** Only the `-alloc` release binaries were used, in
their own runs with `RUM_PROFILE_ALLOC=1` and no `perf`. All other
profiling variables were removed. Every count in this table repeated
**identically in all three runs**. Payload uses the same matching
zero/setup controls described above.

| workload | base count | stage1 count | candidate count | base payload count | stage1 payload count | candidate payload count |
|---|---:|---:|---:|---:|---:|---:|
| cold-intern | 481715 | 481715 | 547045 | 480032 | 480032 | 544032 |
| warm-intern | 913846 | 913846 | 979180 | 432094 | 432094 | 432094 |
| literal-integer | 1650 | 1650 | 2980 | 6 | 6 | 6 |
| symbol-ownvalue-read | 3687 | 3687 | 5021 | 2006 | 2006 | 2006 |
| set-ownvalue | 2699 | 2699 | 4033 | 1006 | 1006 | 1006 |
| module-local | 13209 | 13209 | 15543 | 11512 | 11512 | 12512 |
| downvalue-exact-hit | 7712 | 7712 | 9046 | 6006 | 6006 | 6006 |
| matcher-single-tiny | 4739 | 4739 | 6073 | 3010 | 3010 | 3010 |
| replaceall-simple | 4276 | 4276 | 5622 | 2506 | 2506 | 2506 |
| sort-symbol-default-width4000 | 121785 | 121785 | 139119 | 22 | 22 | 22 |
| sort-symbol-default-width16000 | 481791 | 481791 | 547125 | 22 | 22 | 22 |
| ordinary-smoke | 2201763 | 2201763 | 2203097 | 2200002 | 2200002 | 2200002 |
| empty-process | 1551 | 1551 | 2877 | — | — | — |

Verbatim cold-intern candidate allocator output:

```text
rum-profile alloc count=547045 bytes_total=37594073 peak_live_bytes=8616259
```

The findings are:

- Cold interning adds **64,000 payload allocations** for 16,000
  unqualified names, four per requested name. Source explains the factor:
  `eval_symbol` calls `resolve_symbol_name`, which constructs candidates
  for the default `System` and `Global` contexts before choosing the
  current-context symbol (`structure_ops/surface.rs:345`,
  `resolution.rs:101–138`, `session_state.rs:609`). Each distinct pair
  now allocates two owned strings in addition to the pair allocation
  (`rum-core/src/expr.rs:98–113`). The old string interner still leaks
  its canonical strings; the candidate has not removed that layer.
- The additional warm pass has **zero allocation-count delta** after its
  seed control: 432,094 in every arm. Its instruction cost still rises
  **12.82% total**; no inference that a warm lookup is allocation-free as
  a whole is made, since string construction and evaluator work remain.
- `Module` adds **1,000 payload allocations** at 500 iterations:
  **two per iteration**. This is a workload-3 allocation finding.
- The two sort operations themselves stay at 22 payload allocations.
  Their large gross deltas come from preparing new symbol names.
- The remaining six loop labels and the ordinary arithmetic loop have
  identical payload allocation counts across arms. Ordinary gross still
  rises by **1,334** allocations, all in its startup/setup control.
  Empty-process startup alone rises by **1,326** allocations.
- Three-run `bytes_total` and `peak_live_bytes` captures are retained
  alongside counts in `cost/summary.json`; counts were not read from
  plain binaries.

The ordinary instruction result is **+0.0559% migration,
+14.2654% representation, +14.3293% total**. Its allocation split shows
that the added steady-work instruction cost is not explained by more
allocations in that loop.

### 3. What ran and source-change proof

Exact measurement argv, feature/environment separation, and stdin settings
for **all 894 stack processes and 603 formal cost processes** are in
[measurement-commands.jsonl](../../../target/brief-502/logs/measurement-commands.jsonl).
The full shell-command transcript, including preparatory reads, workload
generation, Python analysis, and report writing, is in
[commands.md](../../../target/brief-502/logs/commands.md).
The initial standalone perf check is included in that transcript.

The substantive command families were:

```sh
TMPDIR="$PWD/target/brief-502/tmp" CARGO_NET_OFFLINE=true python3 scripts/run-bounded.py --timeout 900 -- cargo build -p rum-cli --release --offline --locked --target-dir target/brief-502/build
TMPDIR="$PWD/target/brief-502/tmp" CARGO_NET_OFFLINE=true python3 scripts/run-bounded.py --timeout 900 -- cargo build -p rum-cli --offline --locked --target-dir target/brief-502/build
TMPDIR="$PWD/target/brief-502/tmp" CARGO_NET_OFFLINE=true python3 scripts/run-bounded.py --timeout 900 -- cargo build -p rum-cli --release --features alloc-count --offline --locked --target-dir target/brief-502/build
python3 target/brief-502/elf_inspect.py
python3 scripts/run-bounded.py --timeout 300 -- python3 target/brief-502/stack_measure.py pilot
python3 scripts/run-bounded.py --timeout 600 -- python3 target/brief-502/stack_measure.py slopes
python3 scripts/run-bounded.py --timeout 600 -- python3 target/brief-502/stack_measure.py brackets
python3 target/brief-502/write_stage1.py
python3 target/brief-502/cost_measure.py generate
python3 scripts/run-bounded.py --timeout 600 -- python3 target/brief-502/cost_measure.py calibrate
python3 scripts/run-bounded.py --timeout 600 -- python3 target/brief-502/cost_measure.py panel
python3 scripts/run-bounded.py --timeout 600 -- python3 target/brief-502/cost_measure.py alloc
python3 target/brief-502/cost_summarize.py
```

Build outputs were copied with Python to `rum-candidate`,
`rum-candidate-debug`, and `rum-candidate-alloc` before the next build
could replace the executable. All generated data and build outputs stayed
under `target/brief-502/`. Pre-phase CPU activity checks found no active
CPU-heavy competing process; no builds or other heavy workloads ran
concurrently with measurement. No network, test suite, timing panel,
source edit, commit, branch operation, index operation, or repair ran.

The one requested final status check was
`GIT_OPTIONAL_LOCKS=0 git status --porcelain`. Its complete verbatim output:

```text
 M docs/dev/codex/brief-502-stack-slope-and-a1-cost.md
```

There is **no modification under `crates/`**. Only this report is a
tracked worktree modification; the diagnostic artifacts are ignored.

### 4. Verdict — evidence for the director

This A1 representation has a real debug-stack regression that causes the
existing depth-48 contract to fail. It also adds measurable instructions
in every requested workload, including the literal control and ordinary
evaluation. The total cost, rather than just the migration-isolated cost,
is recorded above. The release depth contract and its physical-height
margin survive; a lost 30% margin is **not** a finding of this experiment.

The fallback A2 was not implemented or measured here. These results put
concrete stack, instruction, and cold-intern/Module allocation costs on
this A1 candidate; they do not choose A1 or A2 or establish A2's cost.
Nothing was tuned or repaired and nothing lands.

This still does **not** establish session ownership, reclamation on
`reset()`, escaped-value lifetime, or cross-session equality cost. The
interner remains process-global and leaking throughout this experiment.

### 5. Stop-loss / open questions

No stop-loss was triggered. The incomplete high-water pilot was corrected
once by adding a post-evaluation sampling plateau; the subsequent fits
produced evidence. Expected guarded failures used to establish brackets
were outcomes of successful measurements, not failed instruments.

The exact proportion of the debug slope and instruction deltas attributable
to drop/unwind state, optional diagnostic tokens, hash/equality changes,
or other code-generation effects remains unisolated. The `needs_drop`
answers are explicitly structural, not fabricated Rust-probe stdout.
Resolving the causal split would require additional compiler analysis or
an authorized controlled source variant; no such change was made.
