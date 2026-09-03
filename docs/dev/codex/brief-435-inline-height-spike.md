# brief-435 — spike: the height inside the value, not behind the Arc

**Shape: spike. The deliverable is a cost measurement.** No enforcement, no
limit constant, no refusal path, no public contract. If the numbers are good I
will design the bound afterwards; if they are not, this line closes.

## Why this shape and no other

Three attempts have now put a cached physical height behind the `Arc`
allocations. brief-426 measured −30.04% and blamed its constructor scan;
brief-434 decomposed it and found the constructor scan real (+3.9% to +11.4%)
and the mutation rescan free. Then the comparison neither of us had run settled
it: a build with **the height field present but never computed** already costs
+14.21% on `stringtrim-literal-list-32000` — a label that builds no call spines
and never reads a height — and +9.58% of `set-part-fixed100-len800`'s +12.21%
total.

**The floor is the 8 bytes per Arc allocation**, not any computation.
`SharedExprInner` goes 48→56 and `SharedArgsInner` 40→48, which under a
16-byte-granular allocator moves the args inner into the next chunk class and
costs cache density everywhere that allocates. No maintenance strategy recovers
that, so both the constructor redesign and the O(1) conservative update are
retired as answers.

What is left is measured, not guessed. Verified by compile-time assertion on
this toolchain:

```rust
assert!(std::mem::size_of::<Expr>() == 32);
assert!(std::mem::size_of::<Atom>() == 32);
assert!(std::mem::size_of::<SharedExpr>() == 8);
assert!(std::mem::size_of::<SharedArgs>() == 8);
```

`Expr::Call { head, args }` carries 16 bytes of payload inside a 32-byte enum,
because `Atom` is what makes `Expr` 32. **There are about 16 spare bytes inside
the value that already exists.** A height stored there grows no allocation, and
reading a child's height becomes a field read at a fixed offset in contiguous
memory instead of an Arc dereference.

## Goal

Put a saturating physical height in `Expr::Call` itself and measure it. The
question this brief answers is exactly one: **does the height cost anything
when it does not grow an allocation?**

Build it the simplest way that is honest:

- A new field on the `Expr::Call` variant, `u16` or `u32` — whichever keeps
  `size_of::<Expr>()` at 32.
- The `Expr::call*` constructors compute it: `max(head_height, max over args) +
  1`, saturating. Atoms are height 1, matching the existing convention.
- `SharedExprInner` and `SharedArgsInner` **must not gain a field.** Their
  allocation sizes must not change. That is the entire point.
- A public `Expr::physical_height()` that reads the field, so nothing is
  dead-code eliminated.
- Every other construction site and pattern updated mechanically; rustc will
  name them. Roughly 73 struct-literal constructions and ~900 `Expr::Call`
  mentions, most of them patterns needing `..`.

**Do not add** a limit, a refusal, a `Result` constructor, an enforcement test,
a `#[non_exhaustive]` attribute, or any API narrowing. Those belong to the
design that follows if this is affordable.

## The two hard invariants, and how to prove them

1. **`size_of::<Expr>()` stays 32.** Put a `const` assertion in the source so a
   regression is a compile error, and paste it. If it becomes 40, **STOP and
   report** — the spike has failed and nothing after that is worth measuring.
2. **No Arc allocation grows.** brief-426 established the technique: read the
   allocation request size from the release disassembly of `SharedExpr::new`
   and `SharedArgs::new` (it found `0x30`→`0x38` and `0x28`→`0x30` there).
   Show the same two numbers unchanged against base. Allocation *counts* must
   also be identical — the microbench reports them and they are load-immune, so
   check them before you look at a single timing.

## Correctness, kept cheap

This is a spike, but a wrong height would make the timings meaningless. Add a
focused test comparing `Expr::physical_height()` against a straightforward
recursive reference over a corpus: atoms, unary and wide calls, call-valued
heads, nested heads, and a spine of depth 65. Include one case built through
each public construction route you touched. That is enough; do not build the
equivalence apparatus the normalizer line needed.

`cargo test --workspace` must be green. If a test fails because it constructs
`Expr::Call` literally, updating it mechanically is in scope; changing what it
asserts is not.

## The A/B — the protocol, and a panel chosen from where the cost landed

```
cargo build --release -p rum-cli          # BASE, before any edit
cp target/release/rum-cli /tmp/rum-cli-base435
# ... make the change ...
cargo build --release -p rum-cli
cp target/release/rum-cli /tmp/rum-cli-cand435
# four passes in one window: base, candidate, base, candidate
```

Report **the whole panel**, not a selection, plus these three columns: the
same-binary delta (base pass 1 → base pass 2), and both cross deltas normalised
by the **median ratio over all 112 labels** rather than by a named control
group. The last brief's panel was chosen from where the cost was expected and
missed `sort-default-wide`, `reversesort-default-wide`, `set-part-fill-*` and
`keysort-default-wide`, which were among the worst rows.

Call out these rows explicitly, because they are where the rejected designs
hurt: `stringtrim-literal-list-32000`, `stringtrim-literal-list-4000`,
`stringtrim-single-literal-32000`, `set-part-fixed100-len800`,
`set-part-fixed100-len100`, `set-part-fill-large`, `set-part-fill-small`,
`sort-default-wide`, `reversesort-default-wide`, `keysort-default-wide`,
`table-iterator-pair-list`, `appendto-part-outer1000-small`,
`appendto-part-outer1000-large`, `appendto-list-incremental-small`,
`appendto-list-incremental-large`.

**A movement inside a label's own same-binary spread is not a finding.** Say so
per row rather than reporting every delta.

## Stop-loss

- `size_of::<Expr>()` is not 32 → stop, report, change nothing further.
- Either Arc allocation request size changes → stop and report.
- Allocation counts differ from base → stop and report; do not proceed to
  timings.
- A coherent family is slower in both replicates and outside its same-binary
  spread by more than 3% → stop and report. **Do not tune.** Three designs have
  now been rejected on measurement and a fourth rejection is a perfectly good
  outcome; what is not acceptable is a tuned number.

## Scope

- You may modify whatever the change requires across the workspace. Nothing is
  committed; I decide what survives.
- `docs/**` is read-only except appending your report to
  `docs/dev/codex/brief-435-inline-height-spike.md`.
- Do not change what any existing test asserts. Mechanical updates for the new
  field are in scope; semantic edits are not.

## Hard rules

- Do NOT `git commit`, branch, push, stash, checkout, or restore. Leave your
  changes in the working tree.
- No network. Touch nothing outside this repository.
- **No new `unsafe`**, including `unsafe` attributes. If you find yourself
  reaching for a union, a transmute, or bit-stealing to fit the field, STOP and
  report — that is a design decision, not a spike detail.
- **Write no probe that builds a deep value to see it fail.** This brief is
  timings, counters and one correctness test; there is no stack-overflow
  reproduction in it and none is wanted.
- Profiling env OFF for every timing number. Bound long commands with
  `scripts/run-bounded.py --timeout SECONDS`, sized from a measured run.
- Paste real verbatim output. If a build failed or a run did not complete, say
  so.
- **Append your report to this file** before you finish.

## Report format

1. What changed — per file, with line ranges, and the total site count.
2. The two invariants — the `size_of` assertion, and the two allocation request
   sizes against base, verbatim.
3. Allocation counters, base against candidate.
4. The full-panel timing table, same-binary column first.
5. The verdict in one sentence, before any explanation: is the height free when
   it does not grow an allocation?
