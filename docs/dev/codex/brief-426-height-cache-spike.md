# brief-426 — can we afford to know an expression's height?

Shape: **implementation, and a measured spike**. Worker: **cdx (Codex)**.

Third step of the traversal line. brief-424 measured, brief-425 converted, and
this answers the question that decides whether the bound is affordable at all.
**It installs no bound.** Enforcement is the next brief, and only if this one's
numbers permit it.

## Why a spike and not the bound

Your brief-423 design puts a saturating physical height inside the private
`Arc` allocations so that admission is O(1) instead of a re-walk. That is the
right shape, and it is also a change to the hottest path in the tree: every
`SharedExpr::new` and `SharedArgs::new` would compute and store it. Your own
plan says to stop the line here if construction or memory labels regress, and I
agree — this project does not trade a hot-path regression for a
pathological-path benefit.

So the deliverable is the height, correct and measured, with nothing depending
on it yet.

## The target the numbers have to serve

`MAX_PHYSICAL_EXPR_DEPTH` is ruled at **`L >= 4097`** (user, 2026-09-03). My
brief-425 gate then pinned the other end: six of the ten converted traversals
complete at 4,500 and give out at 4,700 on a 2 MiB stack, and it is neither the
matcher nor structural equality — `MatchQ[x, _]` and `SameQ[x, x]` both clear
20,000 there. So `L` will land in roughly **[4,097, 4,500]**, with about ten
percent of margin at the ruled minimum. You do not need that here; it is context
for why the height must be exact rather than approximate near the boundary.

## Deliverable

### 1. The cached height

A saturating physical height in the private allocations —
`SharedExpr(Arc<Expr>)` (`crates/rum-core/src/expr.rs:573-574`) and
`SharedArgs(Arc<Vec<Expr>>)` (`:637-638`), with `WeakSharedArgs`
(`:686-687`) pointing at the same inner allocation. Define height as your design
did: `h(Atom) = 1`, `h(Call) = 1 + max(h(head), h(each raw argument))`.

**Saturating**, so an already-deep tree built before any bound exists cannot
wrap to a small number.

### 2. Four properties that must not move

These are derived today and each one is a way to break something quietly:

- **`SharedExpr` and `SharedArgs` and `WeakSharedArgs` stay pointer-sized.**
  Report `size_of` for all three, before and after.
- **`Eq` and `PartialEq` compare exactly what they compare today.** The height
  is a function of the value, so including it is harmless in result and wrong in
  cost — it adds a comparison to a hot path for no information.
- **`Hash` produces the same hash for the same expression.** Exclude the height.
- **`Serialize`/`Deserialize` produce byte-identical output.** These derives are
  on both wrappers; a changed shape is a stop-loss, not a migration to perform
  here. Prove it with a round-trip over a non-trivial expression, comparing
  serialized bytes before and after.

### 3. The measurement trap, which you must defeat before measuring

**Nothing reads the height in this brief, so the compiler may elide computing
it, and you will measure a change that is not there.** A neutral A/B obtained
that way is worse than no A/B: it would license the next brief on false
evidence.

So make the height observable in a way that cannot be optimized away, and say
what you chose — a `Diagnostic`-gated builtin reporting it, a debug assertion,
whatever fits. Then **show the computation actually happens**: an expression
whose height you can predict, read back through your observation point, at
several shapes including a deep one. If you cannot demonstrate the height is
being computed during the timed runs, the A/B does not count and you should say
so rather than report it.

### 4. Correctness against a reference

The cached height must equal a reference computed by direct recursion, for:
atoms, a call with a deep head and shallow arguments, a call with shallow head
and deep arguments, an `Association`, and a tree assembled from shared subtrees
where the same child appears at two positions. That last one is where an
incremental height is most likely to be wrong.

### 5. The A/B, and the stop-loss it feeds

Base against candidate in one window, profiling off:

```
bash scripts/core-language-microbench.sh --output-dir <base-dir>
bash scripts/core-language-microbench.sh --output-dir <cand-dir>
```

Report the construction and allocation labels, the list-literal rows, the
matcher rows, the definition-dispatch rows, and their same-tier neighbours.
**Name your control labels before looking at the candidate numbers.** If the
spread is noisy, raise the iteration count rather than reading the noise — that
worked in brief-420, where the same controls went from a -17%..+7% spread at 1x
to under 2% at 5x.

Also report the allocation counters, which are load-immune where timings are
not: `shared_expr_alloc` and `shared_args_alloc` must not move at all, since
this brief adds no allocation.

**If construction or memory labels regress, STOP and report.** Do not tune, do
not try a second layout, do not proceed to enforcement. A regression here is the
answer to the question this brief asks, and it is a useful answer.

## Scope

Files you MAY modify:

- `crates/rum-core/src/expr.rs`
- `crates/rum-core/src/lib.rs` if an export changes
- whatever the mechanical follow-on of the wrapper change touches — that is
  expected; anything beyond it, **STOP and report**
- a new test module, and the test module roots it needs
- this brief, to append your report

Files you MUST NOT touch: the memory-growth classifications or the `Preflight`
allowlist, `debug_tools/resources/`, any `ResourceLimits` default or the stride,
the traversal conversions brief-425 landed, `AGENTS.md`, `PROGRESS.md`,
`TODO.md`, `docs/**` except appending your report, `.git`, git config.

## Hard rules

- **Install no bound and refuse nothing.** No construction fails in this brief.
  If you find yourself writing a limit check, that is the next brief.
- **No new `unsafe`.** `rum-core` is zero-`unsafe`.
- **No default-path cost beyond the height itself.** The observation point from
  deliverable 3 must not be on the ordinary path when it is not being observed.
- Do NOT `git commit`, branch, push, stash, checkout, or touch git config.
  Leave everything in the working tree. Do not touch `stash@{0}`.
- Touch nothing outside this repo except `/tmp` scratch you remove afterwards.
  No network; pass `--offline` to every `cargo` command.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **This host can bind loopback**; the nine native ZeroMQ tests genuinely run
  here — I observed 9 of 9 at the brief-425 gate.
- **Append your report to this brief file** before finishing.

## Verification (MANDATORY — you run it, iterate to green)

```
cargo fmt --check
cargo test --offline --workspace
cargo test --offline --release --workspace -- --ignored
```

The third covers plan-330's deployment-stack probes and brief-425's ten depth
probes; a height field that changes frame sizes could move them, and I want to
know if it does.

## Report format

1. What changed — per file, line ranges, one-line why.
2. `size_of` for the three wrappers, before and after.
3. How you proved `Eq`, `Hash` and serde are unmoved, with the serde bytes.
4. **How you made the height observable, and your evidence that it is actually
   computed during the timed runs.**
5. Correctness against the reference, including the shared-subtree case.
6. The A/B rows with named controls, and the two allocation counters.
7. Verification — every command, verbatim output.
8. Your verdict: is the height affordable, and does the next brief proceed?
