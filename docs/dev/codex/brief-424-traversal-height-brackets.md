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
