# brief-525 — design: the local-retention class across all 643 declarations

`Scope: NONE — no source change.` Design-only. Survey, verify, return a
census and a mechanism finding. Write no code.

## Goal

brief-524 censused the 165 `Incremental` declarations and found 162 of them
hold a local payload that neither the maintained session total `S` nor the
in-flight expression weight `W` can see. It also found **eight rows declared
`Preflight` or `NonAmplifying` with the same shape**, `DeleteDuplicates`
among them:

```rust
// crates/rum-stdlib/src/structure_ops/dedup.rs:414-417   — declared Preflight
let mut seen: HashSet<Expr> = kept.iter().cloned().collect();
```

`seen` is a full clone of `kept` and is live at the same time; the row's
projection covers the output call only.

So the declared contract is not the boundary of the defect. Establish the
**real** boundary: census all 643 installed declarations under one criterion,
report the corrected declaration for every misdeclared row, and answer the
question the conversion plan is waiting on — whether this class can be closed
by a structure the compiler enforces, or only by several hundred hand edits.

## The criterion, stated once — apply it uniformly and from the code

A declaration is **in the class** when there exists an execution path on
which the builtin holds a local payload whose size depends on input or
callback data, and that payload is, at that moment, none of:

- (a) already charged in `S` — it has been published to a maintained owner;
- (b) covered by an exact projection **taken before the allocation**;
- (c) part of `W`, the in-flight expression being evaluated.

Two sub-classes, and the distinction is the point of this brief:

- **`InClass-NoProjection`** — nothing bounds the payload before it exists.
  This is the `Table`/`GroupBy`/`StringReplace` shape: the size is discovered
  as the work proceeds.
- **`InClass-ProjectableFromChargedInput`** — the payload is bounded by a
  constant factor of a quantity that is **already charged** (an argument's
  proxy bytes, an input string's length, a declared iteration count). Name
  the bounding quantity and say what the factor depends on. **Do not
  estimate the factor** — say what would have to be measured to fix it.

`OutOfClass` requires (a), (b) or (c) with a `path:line`. The following are
**not** dispositions and will be rejected: "small", "bounded in practice",
"assumed constant", "not currently instrumented", "the declared contract says
so". The declared contract is exactly what this census may not trust — it is
the thing being audited.

Where a row's paths disagree, the row takes the **worst** path's class, with
the other paths stated.

## What is already settled — do not redo and do not reopen

- **brief-524's 165 rows carry forward.** Do not re-derive them. Verify a
  sample of at most 15 against this criterion and report **only** rows whose
  class would change under it. The new work is the other **478**.
- **The three unsettled rows stay unsettled** (`Nest`,
  `RegisterExceptionType`, `SyntaxInformation`). They are owned by the
  implementation steps, not by this brief.
- **Director rulings, already made — do not re-raise them:**
  - `R` enters admission only. What must stay unchanged is the **`S` model**:
    `R` never enters `S`, and `S`'s formula is not altered to absorb it.
    `S`'s *reading* may legitimately move when the accountant's own storage
    grows, because that storage is charged on purpose — that is why the owner
    count went 52 → 53 when `accounting_route` became an owner
    (`crates/rum-runtime/src/runtime/debug_tools/resources/occurrence/schema.rs:52-58`).
  - The callback **activation bridge** (a builder that accumulates unarmed
    and is armed later by a nested limit) is an acceptance condition of the
    protocol step, not an open question.
  - **`ToExpression` is ruled**: its parser and lowering locals are bounded
    by the already-charged input string, so it is discharged by a projection
    with a measured constant factor, not by a new syntax/token model. Do not
    propose pulling `rum-syntax` into the memory model, and do not extend
    this to general parser work.
- The occurrence model stands. Do not propose unique or physical counting.
- `Symbol`, the interner, the gensym quota and the snapshot format are out of
  scope.

## Context pointers

- Read `AGENTS.md` first (operational notes, Testing Policy hard lines).
- `docs/dev/codex/brief-524-incremental-declaration-census.md` — the report
  from line 308: the criterion's first application, the 165-row census, the
  eight incidental rows (§3, "Incidental findings outside this line"), and
  the `R` protocol design. Its scratch tables are under `target/brief-524/`
  (`census.tsv`, `families.tsv`, `evidence-snippets.tsv`).
- `crates/rum-stdlib/src/catalog/memory_growth_contract.rs` — all 643
  declarations: 290 `NonAmplifying`, 133 `Preflight`, 165 `Incremental`,
  55 `Delegated`.
- `crates/rum-runtime/src/runtime/metadata.rs:341-350` — what each contract
  claims.
- `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:200-285` — the
  validator: the `NonAmplifying` and `Preflight` arms that **do** check
  something, and the empty `Incremental` arm. Note `:364-382`: it is
  off-by-default diagnostic, opt-in in an ordinary binary through
  `RUM_TEST_BUILTIN_MEMORY_GROWTH`, not test-only.
- `crates/rum-runtime/src/runtime/debug_tools/resources/occurrence/schema.rs:20-42`
  — the exhaustive `Session` field schema and its compile barrier: the
  existing example of a class kept closed by the compiler rather than by
  memory.
- `crates/rum-runtime/src/runtime/accounting_mode/mod.rs` — the companion
  registration seam (`register_accounting_builtin`, `:110`) and the legacy
  adapter every other builtin still uses (`:5-7`).

## Scope

- Files you MAY modify: **NONE**, with two exceptions:
  - append your report to **this** file,
    `docs/dev/codex/brief-525-local-retention-class-census.md`;
  - write scratch artifacts under `target/brief-525/`.
- Everything else is read-only. No source change, no test change, no doc
  change, no `Cargo.toml` change. **You may not change a single declaration
  in `memory_growth_contract.rs`** — the corrected contract is a column in a
  table, not an edit.

## What I want back

### 1. The census of the other 478

One row per declaration, as a table in the report and a TSV under
`target/brief-525/`, with: name, declared contract, class
(`InClass-NoProjection` / `InClass-ProjectableFromChargedInput` /
`OutOfClass` / `Unsettled`), the handler chain, the **named local payload**
with its `path:line`, the bounding quantity where one exists, the corrected
contract, and the family/helper that would convert it.

- A row is settled by evidence on the path, not by the family it belongs to.
  Where a helper is shared, say so — one helper covering twenty rows is the
  most useful thing this census can find.
- An `Unsettled` row goes on an explicit list with what you tried and what
  evidence is missing. An honest gap beats a confident wrong row, and I check
  the confident ones.
- Report the class distribution against the **declared** contract, as a
  cross-tabulation. I want to see how badly the declaration face and the
  defect class disagree.

### 2. The corrected declaration table

For every row whose declared contract is wrong under the criterion, the
correction and its evidence. Standing rules apply to the *proposed*
corrections exactly as they would to an edit: a move to `Preflight` names the
projection that exists or would have to be built; a move to `Delegated` names
the accountable callee **on every branch**; a move that only relocates an
obligation into a contract nothing enforces is a deferral and is rejected.

Also state, separately: which declarations are correct **today** only because
the contract they name is unenforced. `NonAmplifying` and `Preflight` have
live validator arms; `Incremental` does not. A row resting on the empty arm
is not evidence of anything.

### 3. The mechanism finding — this is what the census is for

With the full count in hand, answer: **can this class be closed by something
the compiler or a required test enforces, rather than by N hand edits that
must each be remembered?**

- Name concrete candidate mechanisms against real code, with `path:line`:
  a typed accumulator container whose construction requires a lease; a
  registration that cannot compile without a disposition (the `Session` field
  schema at `occurrence/schema.rs:20-42` is the existing model); a derive; a
  lint-shaped required test over the source.
- For each candidate: its **reach** (how many of the census rows it would
  actually cover), what it cannot reach, and its **default-path cost risk**,
  since these containers sit in the evaluator's hottest loops.
- Say plainly where a mechanism would be a facade — a wrapper that compiles
  but that a handler can trivially bypass with a bare `Vec` is worse than an
  honest hand-edit list, because it looks enforced.
- The project's standard applies: **a fix the compiler enforces beats one
  that relies on people remembering, even an order of magnitude more
  expensive.** Size is not an argument against a mechanism. But an
  over-generalized mechanism is still a stop.

### 4. Sizing the conversion

Rows grouped by the conversion they need and by the shared helper that would
carry it, so the implementation steps can be cut along helper boundaries
rather than along catalog files. State how many distinct conversion sites
this actually is, as against the 643 declarations, and how that changes the
seven-step plan brief-524 drafted.

### 5. Open questions for the director

Anything you could not settle, anything where two readings of the criterion
give different classes, and any decision you think is mine.

## rum / Rust pitfalls (read twice)

1. **The declared contract is the thing under audit.** Do not use it as
   evidence for a row's class, in either direction.
2. **A search for `self.field.insert` misses `entry()`, borrowed mutation and
   `OnceLock::get_or_init`.** The same trap applies to finding a local
   accumulator: `collect()`, `extend`, `push` in a helper, a `String` built
   by `write!`, and a `HashSet` built from a clone are all payloads.
3. **Selection is not non-amplification.** A builtin that only selects from
   its input can hold seen sets, cycle sets and position lists simultaneously
   live — that is exactly how `DeleteDuplicates` got past its own projection.
4. **A shell projection is not a payload projection.** `ConstantArray` and
   the padding family project generated `Expr` nodes, not repeated arbitrary
   child payloads.
5. **A callback's result size is unknown before it returns**, so no honest
   precheck covers it. That is what separates the two sub-classes.
6. **Shared children are multiple occurrences** under the occurrence model.
   Do not reason that a repeated subtree is charged once.
7. **A count derived by regex is suspect.** This repository has produced
   wrong counts from backtick-splitting and from `grep -rl | head -1`.
   Cross-check every derived count against live source reads and say which
   you checked.

## Verification (MANDATORY — the evidence IS the deliverable)

No source change, so no test gate. Instead:

- State every command you ran, and persist raw output to a file under
  `target/brief-525/` **before** parsing it.
- Paste the cross-tabulation of declared contract against class, verbatim.
- Paste at least twelve full census rows, chosen to include every class, at
  least three rows whose declared contract is wrong, and at least two rows
  from the shared-helper groups you propose to cut the work along.
- Paste the counts and say how you cross-checked them.
- State your unsettled list explicitly, or state that it is empty.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Do not run `git`
  at all: nothing in this brief needs it.
- Do NOT modify any file outside the two exceptions named in Scope. If you
  believe you must, STOP and write that in the report instead.
- Touch nothing outside this repo. No network. Commands limited to file
  reads, `rg`/`grep`, and `python3` for parsing.
- **Testing Policy is a hard line** (`AGENTS.md`): no black-box comparison
  against other vendors' engines, no oracle harnesses, no compatibility
  matrices, no third-party engine outputs.
- **No code.** If you are editing a `.rs` file, you have left the brief.
- Do not pad the census to completeness, and do not resolve a hard row by
  widening the criterion.

## Report format (your final message, appended to this file)

1. What changed — "design only, no source change", plus the scratch
   artifacts under `target/brief-525/`.
2. Verification — the commands, the verbatim cross-tabulation, rows and
   counts named above.
3. The census, the corrected-declaration table, the mechanism finding, and
   the conversion sizing.
4. Stop-loss / open questions — the unsettled list, and the decisions you
   think are the director's.
