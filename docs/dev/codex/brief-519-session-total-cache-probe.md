# brief-519 — can the session total be cached instead of routed?

`Scope: NONE — no source change.` Design probe. Survey, measure what can be
measured without changing production, and return a verdict plus a plan. Write
no production code.

## Why this exists

brief-518 refuted the accounting-mode route: it works across all four
crossings but costs the **unarmed** path 0.465994% / 0.697458% / 0.622776%,
and off-by-default machinery charging the default path is a standing hard
line. Its own distinction points somewhere else — *"the generic counter hooks
erase; the complete mode route does not"*. **The counters were not the
problem. Carrying the mode was.**

So: can the 43x be removed **without carrying a mode at all**?

## The idea to evaluate, and its one attractive property

An armed checkpoint today computes `current_memory_proxy_bytes()` — an
O(session) walk — plus `expr_memory_proxy_bytes(expr)`
(`crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs`).
Cache the **session** half and reuse it while the session has not changed, so
a checkpoint costs O(expr) instead of O(session).

Its attractive property is structural: the cache would live **inside** the
`if let Some(limit) = self.current_memory_limit` branch, which the unarmed
path never enters. Unlike a mode route, it has **no default-path cost by
construction** — *except* for whatever is needed to know the cache is stale.

That exception is the whole question.

## The three questions that decide it

### 1. Is there already a counter that advances on every proxy-relevant change?

`Session` carries `evaluation_epoch`, `symbol_cache_generation` and
`call_stability_generation` (`crates/rum-runtime/src/runtime/model/session.rs:36-38`),
plus others. The proxy total, by contrast, is a function of the **52 walked
fields** that briefs 415-416 classified and brief-517 just specified
per-field.

Cross the two lists. For **each** walked field, say which existing counter (if
any) is guaranteed to advance when that field changes. **A single field with
no covering counter makes the cache fail open**, silently reusing a stale
low total — which is worse than a slow walk, so report that plainly rather
than proposing to "mostly" cover it.

### 2. If a new counter is needed, what does bumping it cost the unarmed path?

Mutation sites are far fewer than evaluation steps, so a single `u64`
increment may be much cheaper than a routed mode — or it may not, since
`gAcc = gAcc + i*i` mutates once per iteration. **Measure it**, do not reason
about it: the same instruction-count recipe, the same calibrated workloads,
scaled slopes. The bar is unchanged and is not negotiable: **any unarmed
regression refutes this route too.**

### 3. What is the hit rate where it matters?

The cache only pays off if armed checkpoints mostly find the session
unchanged. Measure the hit rate on at least: the brief-513 reproduction
(`Nest[StringJoin[#, #] &, seed, 16]` under a ceiling — the session holds two
symbols while the *expression* grows, so this should hit nearly always); a
workload that mutates the session every iteration under a ceiling; and one
realistic mixed program. A cache that misses on ordinary armed work buys
nothing.

You may measure hit rates with a throwaway instrumented build, clearly
marked as throwaway and removed before you finish, as brief-506 did.

## What I want back

1. The field-versus-counter cross, produced rather than reasoned, with any
   uncovered field named.
2. The measured unarmed cost of whatever staleness signal is required, with
   the scaling check.
3. The measured hit rates.
4. **A verdict**: proceed, or refute. If this route also fails the unarmed
   bar, say so — two refutations would mean the 43x is a property of this
   design and becomes a documented one, which is a legitimate outcome this
   line has already accepted once.
5. If it proceeds: the plan, the falsifier (a stale total must be detectable,
   and the walker is right there as the oracle brief-517 preserved), and the
   drafted implementation brief.

## Settled — do not reopen

- **No mode routing.** brief-518 measured it; do not re-attempt a variant of
  it here.
- **No global state, no thread-local, no pointer-identity cache.**
- **The occurrence model stands** (user ruling): shared subtrees are charged
  per occurrence. A cache must reproduce the same number, not a cheaper
  different one.
- **The armed walk stays authoritative** until something is proven; this
  brief changes no production code at all.
- `Symbol`'s representation and the parked interner quota are not this
  brief's concern.

## Hard rules

- **No production source change.** Throwaway instrumentation for hit-rate
  measurement is allowed, must be marked as throwaway, and must be removed
  before you finish — state explicitly that the tree is clean of it.
- Do NOT `git commit`, branch, switch, merge, push, stash, `git checkout` or
  `git restore`, or touch git config or the index.
- **No new `unsafe`.**
- **Testing Policy is a hard line** (`AGENTS.md`).
- No network. Touch nothing outside this repo; read `target/brief-5*/` and
  `target/accounting-route-518/` freely, overwrite nothing there.
- Verify instruction counts scale with `N` before trusting them; a `Do` over
  200 000 iterations hits the 100 000 iteration ceiling and measures process
  startup. **Do not run the timing panel.**
- Bound anything long with `scripts/run-bounded.py --timeout SECONDS`, sized
  from a measured run of the same shape.
- **Stop-loss, keyed on progress.** Same step failing twice with no new
  information means stop and report.
- Paste verbatim evidence; never quote a number you could not take.

## Report format

1. The verdict, first.
2. The field-versus-counter cross, with uncovered fields named.
3. Measured unarmed cost of the staleness signal, with the scaling check.
4. Measured hit rates.
5. The plan and falsifier, or the refutation and what it implies.
6. Proof the tree is clean of throwaway instrumentation.
