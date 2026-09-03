# brief-424 — which traversals cannot reach 4,097 on a 2 MiB stack

Shape: **diagnostic**. `Scope: NONE — no source change, not one file.`
Worker: **cdx (Codex)**.

First step of the traversal line, and narrower than your brief-423 draft 423A
because the open question it was going to answer has been settled.

## The ruling that changes this step

**`MAX_PHYSICAL_EXPR_DEPTH` will be at least 4,097** (user ruling, 2026-09-03).
`L = 1,024` was rejected: it supersedes plan-330's documented and tested
4,096-link contract, whose test builds a 4,096-deep `Expr` and drops it
naturally on a 2 MiB thread
(`docs/guide/resource-limits-and-sandbox.md:125-135`;
`crates/rum-runtime/src/runtime/tests/left_chain_lowering.rs:562-587`), and
breaking a documented user-facing contract is semantic loss where extra
conversion work is only work.

So this brief is no longer "measure everything and recommend `L`". It is:
**which traversals cannot handle a 4,097-high value on the smallest stack this
kernel supports, and therefore have to become iterative before the bound can be
called safe.** That list is the scope of the next brief, and I do not want it
guessed.

## Two director rulings that shape what you measure

1. **Do not design a new knob for traversals.** rum already pairs a fixed,
   documented, portable limit with a measured backstop:
   `recursion_limit` (1024) and `native_stack_limit`, where
   `poll_native_stack_limit` (`eval_core/mod.rs:68-78`) measures *actual bytes
   consumed* against a host-configurable budget. Traversals have neither. The
   design will extend that pair rather than invent a third mechanism, so
   measure with that shape in mind.
2. **`DEFAULT_NATIVE_STACK_LIMIT = 6 MiB` (`metadata.rs:6`) fails open on a
   small stack** — it cannot fire on a 2 MiB thread, as brief-422 §6 observed.
   That default will be inverted to be safe on the smallest supported stack and
   raised by hosts that have more. Include in your measurement what budget
   would actually be safe on 2 MiB, since that number becomes the new default.

## What to measure

For every traversal bucket in **brief-422's appended report §3(d) and §3(e)** and
in your own brief-423 inventory — do not re-derive either list, and do not run a
recursive-function regex as a substitute — bracket the maximum physical
expression height it completes, on an explicit 2 MiB thread, in **both debug and
release**.

The repository already has the harness: `run_on_two_mib_thread`
(`crates/rum-runtime/src/runtime/tests/left_chain_lowering.rs:375-383`) spawns a
`stack_size(2 * 1024 * 1024)` thread and joins it. Use that shape.

Report per bucket:

- the height at which it still completes, and the first height at which it does
  not, in both profiles;
- whether 4,097 is inside or outside that bracket — this is the pass/fail that
  decides the next brief's scope;
- bytes of stack consumed per level, derived from the bracket and the 2 MiB
  budget, so the conversion list can be ordered by how far short it falls.

Then three specific questions:

1. **What `native_stack_limit` would be safe on a 2 MiB thread**, given the
   worst per-level cost you measured. This becomes the inverted default.
2. **Can the existing backstop poll be applied to a traversal at all?**
   `poll_native_stack_limit` takes `&mut self` on `Session`
   (`eval_core/mod.rs:68`). Which of the traversal buckets have a `Session` in
   scope, and which do not — `Expr`'s `Display`, derived traits, and drop glue
   plainly do not. Say what that implies for buckets with no session reachable.
3. **What does 4,097 cost in bytes** for the cheapest and dearest buckets, so I
   can see the whole range rather than the worst case alone.

## Method requirements

- **Physical height, not the `Depth` builtin.** Your own report established that
  `Depth` omits heads and treats associations specially
  (`structure_ops/mod.rs:115-140`), so it certifies a different tree than the one
  drop and the renderers walk. Build your probe values so the physical height is
  what you say it is, and state how you constructed them.
- **Calibrate before believing.** A probe that reports a boundary must be shown
  to actually reach the traversal under test. On this line an earlier probe of
  mine appeared to clear the encoders when it had never reached them, because
  `Nest` stopped at the iteration limit before building anything deep;
  `x = 1; Do[x = {x}, {i, N}]` is the route that works. Report how you confirmed
  each probe arrived.
- **Every claim carries `path:line`.**
- **Cross-check any script-derived count against live evidence.**
- **State first, verbatim, what your environment can actually run.**
- **A confident wrong answer is worse than "I could not establish this."**

## Hard rules

- **`Scope: NONE`. Change no file** — not source, not tests, not docs. You may
  read anything, run `cargo` and the release `rum-cli`, and build throwaway
  crates under `/tmp` that you remove afterwards. If you believe a file must
  change, that is a finding for the report.
- **Bracket, do not tune.** Do not change a limit, a default, the stride, or a
  stack size to make a bucket pass. The measurement is the deliverable.
- Do NOT `git commit`, branch, push, checkout, stash, or touch git config. Do
  not touch `stash@{0}`.
- Touch nothing outside this repo except `/tmp` scratch you remove afterwards.
  No network; pass `--offline` to every `cargo` command.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only. No comparison against any other engine.
- **Bound every probe run.** Use `scripts/run-bounded.py --timeout SECONDS`,
  sized from a prior measured run of the same shape, so a probe that does not
  return cannot hold the dispatch open.
- **Append your report to this brief file** before finishing.

## Report format

1. Environment capability, verbatim, including how you built probe values and
   how you confirmed each probe reached its traversal.
2. The bracket table: bucket, debug pass/fail heights, release pass/fail
   heights, bytes per level, and whether 4,097 is inside the bracket.
3. The ordered conversion list — every bucket for which 4,097 is outside the
   bracket, worst shortfall first. This is the deliverable I care about most.
4. The three specific questions above.
5. Anything in the brief-422/423 inventories you could not bracket, and why.
6. Risks and open questions.

---

# Report — 2026-09-03 — executed by the director

## Why this was not run by a worker

Dispatched to Codex twice. The second attempt reached 120,486 tokens and then
hit the upstream content filter — not on this brief's text, which I scanned
clean before dispatch, but on the worker's own generated probe code, whose tail
in the log is:

```
+    std::mem::forget(build);
+    std::mem::forget(session);
```

Deliberately leaking a session to avoid its destructor, in a stack-capacity
probe, is the shape that tripped it. Rewriting the brief cannot fix that: the
measurement inherently produces such code. GLM was not used because its weekly
quota was at 18% and this is a long, mechanical measurement. I ran it myself
against the release CLI.

## Method, and the assumption I verified instead of using

Probe values are built with `x = 1; Do[x = {x}, {i, N}]`, giving physical height
N+1. `Nest` does not work — it re-evaluates and stops at the iteration limit,
which is how an earlier probe of mine appeared to clear the encoders without
ever reaching them.

Rather than measure on 8 MiB and scale, I set the stack directly with
`ulimit -s 2048` and measured on 2 MiB. I checked the scaling relation anyway,
because I had been reasoning with it:

```
ToString, 8192 KiB:  1500 ok  1600 ok  6250 ok  6400 fail
ToString, 2048 KiB:  1500 ok  1600 fail 6250 fail 6400 fail
```

6250/1500 = 4.17, 6400/1600 = 4.00 — linear in stack size, as assumed. Every
number below is nonetheless a direct 2 MiB measurement.

2 MiB is the right target: `docs/guide/resource-limits-and-sandbox.md:122`
records it as Rust's default for a spawned thread, so it is what an embedder
gets without setting `stack_size`. Raising the documented minimum would require
every host to remember a setting, which is the failure mode this project
replaces with structure.

## The bracket table — 2 MiB stack, ruled bound 4,097

| Traversal | Result at height 4,097 | Ceiling | Clears the bound |
|---|---|---|---|
| `ByteCount` | 700619 | far above | yes |
| `Depth` | 4098 | above | yes |
| `LeafCount` | 4098 | above | yes |
| `Expr::Display` (via `FullForm`) | 24583 chars | 5,000–8,000 | yes |
| `ToString` | — | **1,550–1,600** | **no, 2.6x short** |
| `Map` | — | below 4,097 | **no** |
| `Apply` | — | below 4,097 | **no** |
| `ReplaceAll` | — | below 4,097 | **no** |
| `Cases[_, _, Infinity]` | — | below 4,097 | **no** |
| `Level` | — | below 4,097 | **no** |
| `MemberQ[_, _, Infinity]` | — | below 4,097 | **no** |
| `Count[_, _, Infinity]` | — | below 4,097 | **no** |
| `Position` | — | below 4,097 | **no** |
| `FreeQ` | — | below 4,097 | **no** |

## The conversion list

Ten entries: `ToString`, `Map`, `Apply`, `ReplaceAll`, `Cases`, `Level`,
`MemberQ`, `Count`, `Position`, `FreeQ`.

## The structural lead this produced

**There are two renderers and only one of them is short.** `FullForm` is
`Ok(Expr::string(expr.to_string()))` (`structure_ops/surface.rs:78-89`) — it
uses `Expr`'s own `Display` and clears 4,097 on 2 MiB with headroom to roughly
5,000–8,000. `ToString` goes through a `ToStringFormat` dispatch
(`strings/text/builtins.rs:624-640`) and gives out at 1,550.

So the renderer half of the conversion is scoped to `ToString`'s formatter, not
to rendering in general, and `Expr::Display` — which is in the implicit,
universal category the brief-423 predictive rule warns about — does not need
converting at this bound. That is a materially smaller and safer job than the
design priced.

## What I did not establish

- I bracketed `ToString` and `Expr::Display` precisely; the other nine are
  recorded only as "below 4,097", not bracketed. Ordering the conversion work by
  shortfall needs those brackets.
- brief-422 §3(e)'s source-verified rows — string-pattern machinery, calculus
  and polynomial walkers, schema conformance, tensor rank, matcher compile,
  substitution and rewrite traversals, suspension continuation encoding — are
  not measured here at all. They are the tail of the same class.
- Debug-profile brackets are unmeasured; every number above is release.
- The two questions about the backstop poll — what `native_stack_limit` is safe
  on 2 MiB, and which buckets have a `Session` in scope at all — remain open.
  `Expr::Display`, the derived traits, and drop glue plainly do not have one.
