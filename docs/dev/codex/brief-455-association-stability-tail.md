# brief-455 — bound the association arm of the stability walk

## Goal

brief-444 and brief-446 shrank the **generic** arm of the evaluation-stability
walk to 192 B per level and made it fail closed. The **association arm** was
never touched, and it costs about 512 B more per logical association wrapper.
That is what binds every operation producing a fresh deep association: `Map`
over an association, a read after a nested key-path assignment, and the rest of
the class brief-454 inventoried.

Make association-rooted stability checking native-stack-bounded so those
complete at raw physical height 4,097, **without changing stability verdicts,
bloom union order, cache keys, cache insertion order, profiling counters, or
the shape of the generic hit/miss path**.

This is draft A of brief-454, dispatched first and alone because it is the
shared root of the class and because it touches the most consulted function in
the system.

## What I measured — treat as given, do not re-run

Release `rum-cli` at `d72942b4`, `ulimit -s 2048`, 1 MiB evaluator budget.

**Heights below are logical association wrappers.** One wrapper is two physical
levels — `Association["k" -> a]` renders as `Association[Rule[…]]`, verified:
at `N=2` the `FullForm` is `Association[Rule["k", Association[Rule["k", 1]]]]`,
physical height 5 = 2N+1. So **the ruled 4,097 physical height is `N=2048`**,
and `Depth` reports 3 for that `N=2` fixture because it hides the `Rule` layer —
do not use `Depth` as the height unit anywhere in this work.

| probe | last ok / first fail (wrappers) | raw physical | mode |
|---|---|---|---|
| `Map[ff, a, {0, Infinity}]` | 966 / 974 | 1,933 | in band |
| nested part assignment then reading the result | 1,145 / 1,153 | 2,291 | in band |
| the same assignment **returning only the scalar** | clears 2,500 | 5,001 | — |
| `MergeNested[{a, b}, Total]` | 1,153 / 1,161 | 2,307 | in band |
| **construction alone** | **5,457 / 5,463** | 10,915 | in band |

Two of these matter for how you work:

- The assignment's own recursion is **not** the constraint — returning the
  scalar clears 2,500 wrappers. What fails is the later read of the freshly
  rebuilt value, i.e. this walk.
- **Construction is the fixture ceiling at 5,457 wrappers**, so no association
  certificate can be built above it. `N=2048` is comfortably below, but a probe
  reporting a ceiling near 5,457 is measuring the fixture, not the operation.
  brief-454's report attributed that common bracket to `Expr` teardown; that is
  refuted — construction alone brackets identically and the failure precedes any
  output. Do not build on the teardown reading.

`MergeNested` is a separate native recursion and is **out of scope here**.

## The mechanism

`crates/rum-runtime/src/runtime/eval_core/stability.rs`:

- `association_data_stability_with_bloom` (`:98-134`) builds
  `linear_keys: Vec<Expr>` and possibly `indexed_keys: Option<HashSet<Expr>>`
  for duplicate-key detection, and **holds both live across the recursive
  `cached_expr_stability_with_bloom(arg)` call at `:127`**, together with the
  parsed entry, the loop position and the accumulated bloom. That is the ~400 B
  frame.
- `rule_data_stability_with_bloom` (`:137-153`) recurses on the lhs (`:147`) and
  then the rhs (`:157`), holding the lhs result and blooms live — ~112 B.
- Both are already `#[inline(never)]`, but they are on the *hot recursive path*,
  not cold: one of each runs per logical wrapper, on top of the 192 B generic
  packet, which is where ~896 B per wrapper comes from.

## Traps — read twice, these are where this change goes wrong

1. **Own the whole rooted subtree, do not restart at each `Association`.** The
   `Map` result interleaves ordinary calls with associations —
   `ff[Association[Rule[key, child]]]` per level. A bound that resets its depth
   at every `Association` node would miss exactly that shape, which is the
   worst-measured probe in the class.
2. **The generic path keeps its exact current shape.** briefs 440 and 441 were
   functionally complete conversions of this walk and were rejected for 2-7% on
   hot labels, because a depth parameter and an `Option` return on the most
   consulted function in the system is not free; brief-442 got to zero by
   outlining cold branches instead. Most recently briefs 451-453 measured a
   context struct at +0.714% and a thread-local at +1.085% on their own hot
   path. So: the association tail may carry whatever state it needs, but a
   generic hit or an ordinary miss must execute the same instructions it does
   today.
3. **Hoisting the key-uniqueness scan changes cache contents.** Today a
   duplicate key at entry *k* returns `false` only *after* entries 1..k-1 have
   been recursed into and cached. Scanning all keys first would skip those
   insertions. If you restructure that loop, prove the cache-insertion trace is
   unchanged; the suite already asserts such traces.
4. **The predicate is one-sided.** `true` is authoritative; `false` may be
   conservative — every consumer re-does the work rather than trusting a
   negative. Any bail-out must return "not stable", never "stable". This is what
   made brief-446's strided poll safe and it is the same here.
5. **No profiling state in the recursive frame.** brief-444 removed 64 bytes of
   off-by-default profiling from this exact walk; do not reintroduce it in a
   task or a frame.

## Deliverable

1. A depth-16 borrowed recursive front for association-rooted stability, whose
   exhausted tail hands off to an evaluator-owned explicit `Visit`/`Finish`
   machine that owns the entire rooted subtree, carrying its depth through
   intervening ordinary calls.
2. Preserved exactly: head-before-args order, the first-unstable short circuit,
   bloom union order, stable-only insertion, postorder cache insertion order and
   the size-cap behaviour (`evaluation_cache/stability.rs:219-264`).
3. Cache-trace equivalence tests covering duplicate keys, `RuleDelayed`, dirty
   generations, an unstable early child, and cache-cap insertion order.
4. Certificates, following the rule this line settled in briefs 439-442:
   - build `N=2048` and **assert with an iterative helper that the raw physical
     height is 4,097** — not `Depth`;
   - a unique sentinel at the deepest value only;
   - a release-only `#[ignore]` deep certificate for `Map[ff, a, {0, Infinity}]`
     asserting the **exact full `InputForm`**, and one for the nested assignment
     that reads `b` back and asserts the exact rebuilt value;
   - three-level shallow companions pinning `{0, Infinity}` and the full key
     selector chain.
5. Microbench labels only if you add one for the association-stability shape;
   do not rename or alter existing labels.

## The falsifier — required section

Before changing any source, add the two shallow companions, run them against
unmodified HEAD, paste the passes, then break each deliberately and paste the
failures. Restore them before continuing.

Do **not** run the deep certificates against unmodified HEAD: at `N=2048` the
pre-change code fails in band, which would look like a test failure rather than
the fixture it is.

## Scope

Files you MAY modify:

- `crates/rum-runtime/src/runtime/eval_core/stability.rs`
- `crates/rum-runtime/src/runtime/evaluation_cache/stability.rs`
- new files beside either, declared from their existing module roots; you MAY
  edit those roots for the `mod` line only
- runtime evaluation-cache tests
- `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs`
- `scripts/core-language-microbench.sh` — additive only

Out of scope: `Expr` representation and serde, every stdlib operation
implementation, `MergeNested`, `set_association_key_path`, documented semantics,
and `Cargo.lock`.

## Measurement — instruction counts, not wall time

**A timing percentage is not evidence on this host and will not be accepted.**
P-cores and E-cores are 1.27x apart with no pinning, so scheduling alone moves a
wall-clock number more than anything this gate judges.

1. **Allocation counts**: `--features alloc-count` build, `RUM_PROFILE_ALLOC=1`,
   isolated probe per label (fixture plus the exact statement in a `Do` loop),
   base and candidate executable paths the **same length** (argv size changes
   the count). Calibrate first: one extra loop iteration must move it.
2. **Instruction counts**: `perf stat -e instructions:u -x, -- <bin> --file
   <probe.rum>`, unprivileged here. The hybrid PMU reports `cpu_atom` and
   `cpu_core` separately — a run that **migrates between core classes reads
   about 55% high and must be discarded** (both counters non-zero);
   `<not counted>` means that PMU never ran; clean runs cluster within
   0.006%-0.08%, so take the minimum of the tight cluster and name what you
   discarded.

**The bar, and it is tight because this is the hottest path in the system.**
These generic labels must each stay within **±0.15%** on instructions, with
allocation counts unchanged: `downvalue-exact-hit`, `downvalue-exact-miss`,
`downvalue-pattern`, `matcher-single-tiny`, `cases-wide-no-match`,
`replaceall-simple`, `list-literal-small`, `do-count-null`. Association labels
may improve; that is the point.

**If two isolated attempts cannot hold that bar, stop and report the numbers for
each.** Two functionally complete conversions of this walk are already preserved
as stashes for exactly this reason, and a well-measured "no shape holds the
generic path flat" is an outcome I will take.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test -p rum-runtime
cargo test -p rum-stdlib
cargo test --workspace
```

`cargo test --workspace` may fail nine `rum-jupyter` native ZeroMQ tests if your
sandbox cannot bind loopback — an environment limit, not a result. Say so and
also run `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test --workspace`, pasting both.

Then, after the shallow suite is green:

```
cargo test -p rum-stdlib --release -- --ignored
```

That panel takes about 430 s because of the `D` certificate; size the timeout
from a measured run, not a guess.

Then the allocation and instruction comparisons above.

## Stop-loss

Stop and report if:

- a cache-insertion-order or bloom-order difference appears twice;
- the generic-path bar cannot be held after two isolated attempts;
- the bound would require changing a stability verdict, or returning "stable"
  on a bail;
- a certificate can only pass by lowering the target height or by using `Depth`
  as the height unit.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave the tree.
- Do NOT modify files outside Scope; nothing outside this repository; no network.
- Testing Policy is a hard line (`AGENTS.md`).
- No new `unsafe` in `rum-core`/`rum-runtime`; no default-path cost; no
  profiling state in a frame or task.
- Use `scripts/run-bounded.py --timeout SECONDS -- <command>` for anything long.
- Paste real verbatim output.

## Report format

1. What changed — per file and line range, and how the machine owns a rooted
   subtree across intervening ordinary calls.
2. Verification output verbatim, including both falsifier passes and their
   deliberate failures, and both workspace runs.
3. Safety and contract as separate outcomes: no process termination, and the
   raw physical height reached, measured with an iterative helper.
4. Allocation and instruction numbers for every named generic label and for the
   association labels, with the calibration run and the discarded migrated runs.
5. Stop-loss / open questions.
