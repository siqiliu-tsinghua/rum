# brief-517 — specify the occurrence model, and build its oracle

## Goal

The `Incremental` line replaces an O(session) walk at every armed checkpoint
(2.26x-43.34x today) with a maintained total. Before anything is maintained,
the thing to be maintained has to be **specified**, and an **independent
oracle** has to exist to check it against.

This brief builds that specification and that oracle. **No cached total, no
maintained ledger, no change to checkpoint frequency or to ownership
behavior.** What lands is: a per-field occurrence breakdown computed by an
independent traversal, an exhaustive schema that preserves every existing
exemption, and exact tests pinning the formulas that later briefs must keep
reproducing.

## Why the model is the deliverable

The existing walker charges repeated shared subtrees **by occurrence**. The
user's ruling (2026-09-22) keeps that model: the ceiling means what it
already means, and a unique/physical counting model would be a separate
explicit ruling. So this brief writes the occurrence semantics down as a
*specification*, not as whatever the current code happens to compute.

**Where the two differ, report the difference — do not retune anything to
hide it.** If specifying the model exposes that today's walk is inconsistent
with itself, that is a finding and it belongs in the report, not in a quiet
adjustment of a limit or a constant.

## Director rulings this brief operates under

- **Occurrence-based, session total first, local builders later** — the
  order is settled; do not re-litigate it.
- **Overflow: conservative saturation**, with a **non-lossy or explicitly
  invalid state for releases**. Both directions must fail closed — a
  saturated charge makes a session look over its limit, and a
  non-representable release must leave the total too high rather than too
  low. Specify and test this arithmetic; **no impossible-size allocation is
  needed to do it**.
- **The accountant is charged too.** No ledger exists yet, but the rule is
  set now so it is not discovered later: every bookkeeping byte a future
  brief adds appears in the proxy. Exempting the accountant is the interner's
  mistake in miniature.
- **The armed walk stays.** This line makes it cheap; it does not restore
  sampling, and there is no stride and no adaptive policy.

## Deliverable

1. **An independent field breakdown.** A traversal that reports the proxy
   total *per field* rather than as one number, alongside the existing
   sampled-peak wrapper. It must be independent enough to serve as an oracle:
   a later maintained total is checked against it, so it must not be
   refactored into sharing the same arithmetic path.
2. **An exhaustive schema that preserves every exemption.** Briefs 415-416
   left `Session` with 52 walked fields and 41 recorded exemptions, generated
   by a macro that makes a new field **fail to compile until classified**.
   That property is the point; keep it. Say how many fields exist now and
   whether the count moved.
3. **Exact tests for the formulas**, covering at minimum: sharing (the same
   subtree reached twice), capacity versus length, `ByteArray` (charged its
   base64 text, not its decoded payload), exception payloads, dispatch
   structures, and frame payloads. These are what every later brief in the
   line regression-tests against.
4. **Synthetic overflow and release-after-overflow tests** under the approved
   rule.
5. **Recorded baseline sizes** for `Session` and the relevant frame and index
   structures, so a later brief can show what its bookkeeping added.
6. **Documented occurrence semantics**, including what is deliberately
   omitted, in `docs/guide/resource-limits-and-sandbox.md`.

## The falsifier, which must be watched failing

The schema's value is the compile barrier. **Demonstrate it**: add a field to
`Session` without classifying it, show the compile failure verbatim, remove
it, show the schema green again. A barrier nobody has watched stop anything
is not known to work.

## Scope

Files you MAY modify:

- `crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs` and new
  pure accounting/oracle modules beside it, plus the module declarations they
  require
- `crates/rum-runtime/src/runtime/tests/session_memory.rs` and sibling
  `session_memory*` test files
- `crates/rum-runtime/src/runtime/model/session.rs` only if the schema
  requires it — say why
- `crates/rum-runtime/src/runtime/builtin_catalog/debug.rs` memory-entry
  documentation
- `docs/guide/resource-limits-and-sandbox.md`, and `docs/reference/builtins.md`
  **only** by regeneration if the drift test demands it
- this file, for your report

Anything else the compiler forces is a **disclosure duty**: list it with the
reason. A change that is not compiler-forced and not listed is a stop.

Do NOT touch: evaluator dispatch, ownership or retention behavior, builtin
builders, `Expr`/`SharedExpr`/`SharedArgs`, the interner, `scripts/**`,
`AGENTS.md`, `TODO.md`, `PROGRESS.md`, `Cargo.toml`, `Cargo.lock`, `.git`,
the ownership-graph inventory fixtures.

## rum / Rust pitfalls (read twice)

1. **Do not replace traversal with cached values anywhere.** That is the next
   brief's job and doing it here destroys the oracle.
2. **Do not deduplicate `Arc` roots.** Occurrence counting is the model;
   deduplicating silently switches to the unique model the ruling excluded.
3. Do not let a diagnostic path clear caches or otherwise change what it
   measures by measuring it.
4. Do not silently remove a conservative inline term because it looks
   redundant. If it is redundant, say so and leave it.
5. `sum::<usize>()` and saturated subtraction both need explicit reasoning
   written down, not just correct-looking code.
6. **Changing a helper's arithmetic is a runtime change**, even in a
   diagnostics module, and is gated like one.

## Verification (MANDATORY — you run it, paste it verbatim)

```
cargo fmt --check
cargo test -p rum-runtime session_memory
cargo test --workspace --no-fail-fast
```

**Expected state on `main`:** 2 493 passed, 0 failed, 57 ignored on a host
that can bind loopback. Run **without** `RUM_ALLOW_NATIVE_ZEROMQ_SKIP`; if
your sandbox denies loopback you will see nine native ZeroMQ failures —
report them as environmental and say plainly those tests did not execute.
Any other failure is yours.

Plus the compile-barrier demonstration described above, both directions,
verbatim.

**Performance:** if no production traversal or arithmetic changed, say so
plainly and run no A/B — do not perform the ceremony. If any did, the gate is
the instruction-count recipe (`taskset -c 2 perf stat -e instructions:u`),
armed and unarmed, and **the unarmed path must not move**.

Bound the suite with `scripts/run-bounded.py --timeout 900 --`; a recorded
run of the same shape took about 160 s.

## Hard rules

- Do NOT `git commit`, branch, switch, merge, push, stash, `git checkout` or
  `git restore`, or touch git config or the index. Leave changes in the
  working tree; the director gates and commits.
- **No new `unsafe`.**
- **No maintained ledger, no cached total, no stride, no adaptive policy.**
- **Do not weaken or delete a test to make the suite pass**, and do not
  resolve an unapproved overflow or metadata policy by implementing one.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only; no oracle harnesses against other vendors' engines, no
  compatibility matrices. ("Oracle" here means this line's own independent
  traversal, which is fine; a third-party engine is not.)
- No network. Touch nothing outside this repo; read `target/brief-5*/`
  freely and overwrite nothing there.
- **Stop-loss, keyed on progress.** If the same step fails twice and the
  second attempt produced no more information, stop and report.
- Paste real verbatim output, including failures.

## Report format

1. The per-field model: what each field contributes and why, with the
   occurrence semantics stated.
2. Field count now versus the 52 walked / 41 exempt recorded by briefs
   415-416, and any classification that moved.
3. **Every arithmetic difference between the specified model and what the
   code computed before** — reported, not tuned away.
4. The compile-barrier demonstration, both directions, verbatim.
5. Baseline sizes for `Session` and the frame and index structures.
6. Verification, verbatim, with the complete failure set.
7. Anything you refused, and open questions.
