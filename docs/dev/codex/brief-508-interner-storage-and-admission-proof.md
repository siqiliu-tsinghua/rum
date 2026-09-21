# brief-508 — prove interner storage can be charged, and refused before it is published

## Goal

A safe, **scratch-only** proof of the two mechanisms the ruled quota rests
on, neither of which exists yet:

1. **Exact charging.** The interner's storage can be accounted in the ruled
   proxy units, including the backing-table bytes that an opaque `HashSet`/
   `HashMap` will not tell you.
2. **Refusal before publication.** An insertion that would exceed the budget
   is refused with **nothing leaked and nothing published** — the permit is
   consumed only by a step that cannot fail.

This is brief-505's Draft A, rewritten under the director's D1-D7 rulings and
brief-506's census. **It is not a production interner, not a performance
claim, and nothing lands.** The scratch package is evidence.

## Why this is unblocked

brief-507 found the `Module` repair feasible in representation but blocked on
a user ruling about display labels versus source-resolvable names
(`TODO.md` §The Module repair is feasible but BLOCKED ON A USER RULING). The
quota is required under **every** outcome of that ruling, so this proceeds
now. Its *sizing* waits; its *mechanism* does not.

## Settled, and not to be reopened

- **`Symbol` does not change**: size, `Copy`-ness, id-based fast-accept
  equality, and its lifetime-free `'static`-compatible public type are all
  frozen (user ruling 2026-09-21).
- **No reclamation, no per-session attribution.** The quota is process-wide.
- **D2: the charging model is a proxy, and only a proxy.** Exact Rust
  allocation-layout bytes plus a conservative fixed reserve, explicitly
  excluding allocator metadata, rounding and RSS. The portable-physical /
  RSS reading is rejected; do not reintroduce it under the word
  "conservative".
- **The direction of error is not free.** An undercharge is a fail-open. Where
  a byte count cannot be taken exactly in safe Rust, the bound must err
  **high**, and you must say which bound you used and why it is an upper one.
- **You do not choose the default.** D1 reserves that for the director. Report
  what the reserved vocabulary costs and what headroom a given cap leaves;
  do not select one.

## What the census already established — inputs, not things to re-derive

From brief-506 (verify anything you lean on, but do not re-run the suite):

- Bootstrap: **662 distinct strings, 661 pairs, ~77 004-85 033 proxy bytes**.
  661 of those pairs are `System\``, and 580 are installed non-canonical
  catalog names rather than `CANONICAL_CORE_HEADS`.
- Ordinary workspace suite, **maximum over test binaries**:
  **14 413 346-15 855 999 bytes**. 8 MiB is below this repository's own test
  suite; 32 MiB is the first envelope clearing bootstrap, suite and panel.
- **Lookup is itself an insertion ingress.** Resolving a fresh unqualified
  name constructs candidates per probed context and charges **two** pairs,
  sharing one text payload
  (`crates/rum-runtime/src/runtime/resolution.rs:101-138`). An admission
  model that assumes one pair per textual symbol is wrong, and this is the
  single most important thing for your seam to get right.
- Draft A's original 8 MiB subtraction is **not** calibration, and the
  present container's bytes-per-call average is **not** evidence about a
  replacement index's budget.

## Deliverables

Write each section of your report before starting the next.

### 1. The storage substrate, explicitly sized

Safe Rust, no `unsafe`, in the scratch package: an explicitly sized index
whose slot, control-byte and padding storage you can state exactly, standing
in for today's `HashSet<&'static str>` and `HashMap<(usize,usize), &'static SymbolText>`
(`crates/rum-core/src/expr.rs:41-44`). Report its layout per capacity class
and the growth transitions.

State the accounting unit and enumerate **every** allocation: the text
itself, the pair record, the index backing storage, any staging or
conversion allocation, the old/new table overlap during a growth, and the
fixed reserve. Use checked arithmetic. **No live-index growth before
admission.**

### 2. The staged insertion transaction

Reserve → stage → admit → publish, where **publish cannot fail**. Show, with
drop evidence, that a refused insertion frees everything it staged. Prove
the decision ordering through a private seam: **an existing-pair hit is
returned before any permit is requested**, so a full interner never refuses
a lookup of a name it already holds.

Model the two-pair lookup ingress explicitly: a single fresh unqualified name
arriving through resolution must be admissible or refusable **as a unit**, or
you must state what happens when the first pair is admitted and the second
refused. That partial state is the interesting case; do not leave it
undefined.

### 3. The independently checked ledger

The ledger's expected value must **not** be produced by the same projection
it is checking — a test that calls the projection for its own expected answer
certifies nothing about storage the projection forgets. Derive the expected
bytes a second way and say how.

Boundary cases, each with verbatim output: same string used as both context
and name; a new pair over existing strings; long text; every growth
transition; arithmetic overflow; id exhaustion; exact fit; one byte short.

### 4. The falsifier

Per this project's standing rule, a check nobody has watched fail is not
known to work. Ship checked-in proofs that: refusal fires at the cap;
refusal does **not** fire one byte below it; and enforcement removed from the
admission path is **detected** by the tests rather than passing silently.
Demonstrate that third one by actually disabling enforcement and showing the
red.

Note the trap brief-505 named: a process-global, never-reclaimed quota makes
tests order-dependent. Your seam must be injectable per test without
manufacturing real `Symbol`s in a second domain.

### 5. The reserved vocabulary, sized

Draft the finite reserved set — the names that must be interned before the
quota can bind so that a refusal can always construct its own message.
Enumerate its origins: canonical heads (`crates/rum-core/src/heads.rs:112`),
fixed runtime and stdlib catalog names, option defaults, error and
continuation tags, the `Global`/`Information` helpers. Measure its declared
layout. Report the headroom it leaves at 8, 16 and 32 MiB — as arithmetic for
the director, **not as a recommendation**.

## Scope

- You MAY create and write a standalone offline scratch Rust package and all
  evidence under `target/brief-508/**`, and append your report to this file.
- **MUST NOT** edit any file under `crates/`, `scripts/`, `docs/` other than
  this one, `Cargo.toml`, `Cargo.lock`, `AGENTS.md`, `TODO.md`,
  `PROGRESS.md`, `.git`, or the ownership-graph inventory fixtures.
- No new external dependency. Keep every build artifact inside the scratch
  subtree (set `CARGO_TARGET_DIR` and `TMPDIR` there).

## rum / Rust pitfalls (read twice)

1. `capacity * entry_size` is **not** evidence about an opaque hash table's
   backing bytes. If you use a real `HashMap` anywhere in the ledger's path,
   you cannot state its bytes exactly — that is why §1 asks for an explicit
   index.
2. `Box::new` does not promise recoverable OOM. Refusal must be a budget
   decision you make, never an allocation failure you catch.
3. A lifetime-free `Copy` handle cannot lease a local arena, and the scratch
   package must not export a real `rum-core` `Symbol` backed by its own id
   sequence.
4. A `&'static str` set is not a closed vocabulary.
5. Do not use `unsafe` to obtain a measurement. Bound it and state the
   direction instead.

## Verification (MANDATORY — you run it, paste it verbatim)

```
CARGO_NET_OFFLINE=true scripts/run-bounded.py --timeout <derived> -- \
  cargo test --offline --manifest-path target/brief-508/Cargo.toml
```

Derive the timeout from a recorded run of the same shape and say which; a
fresh small package builds in well under a minute on this host, so a 300 s
bound has ample headroom. Use the workspace-declared toolchain (1.85.0).

Do **not** run the production suite or the timing panel: production source is
unchanged, so there is nothing for them to say. Failure to bound an
allocation is an explicit failed proof, not permission to substitute an
estimate.

## Hard rules

- Do NOT `git commit`, branch, switch, merge, push, stash, `git checkout` or
  `git restore`, or touch git config or the index.
- **No new `unsafe`**, anywhere, for any reason.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only; no oracle harnesses, no comparisons against other vendors'
  engines.
- No network. Touch nothing outside this repo.
- **Stop-loss, keyed on progress.** If the same proof step fails twice and
  the second attempt produced no more information than the first, stop and
  report what you have. If the ledger undercharges, or the storage needs a
  forbidden representation, that is a **failed proof to report**, not a
  problem to engineer around.
- Paste real verbatim output, including failures. No cost claims from a toy
  prototype.

## Report format

1. The storage graph: every allocation, the layout per capacity class, and
   the independent charge calculation.
2. Transaction evidence: staging, drop-on-refusal, ordering, and the
   partial-admission answer for the two-pair ingress.
3. Boundary and falsifier evidence, verbatim, including the deliberately
   disabled-enforcement red run.
4. The reserved vocabulary, its origins and its measured layout, with
   headroom arithmetic at 8/16/32 MiB.
5. Proof verdict, what remains unproven, and the director decisions you hit.
