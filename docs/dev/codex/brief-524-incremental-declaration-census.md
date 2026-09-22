# brief-524 — design: discharge the 165 `Incremental` declarations, post-A-E

`Scope: NONE — no source change.` Design-only. Survey, verify, return a plan
and drafted implementation briefs. Write no code.

## Goal

165 installed builtins declare `MemoryGrowthContract::Incremental`
(`crates/rum-stdlib/src/catalog/memory_growth_contract.rs`) against a contract
that promises "retained growth must be charged as data-dependent construction
proceeds" (`crates/rum-runtime/src/runtime/metadata.rs:346-347`). The
`Incremental` arm of the growth validator is literally empty
(`crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:276`), and that validator
is test-only (`:205`, `:366`). So today the declaration has neither an
implementation nor a falsifier.

Return **a verified disposition for every one of the 165**, a design for the
local-builder accounting the undischarged ones need, a design for the
enforcement that keeps the class closed, and the implementation briefs to
build all of it.

## Why this is a design brief, and why the previous plan cannot be dispatched

brief-516 already drafted this work as F-J
(`docs/dev/codex/brief-516-incremental-accounting-design.md:1441-1700`). Those
drafts were written **before A-E landed**, and A-E changed the ground under
them:

- `46126cab` made all 52 `Session` memory owners maintain their own term, so
  an armed checkpoint reads a maintained total instead of walking the session.
- `902aea1f` specified the occurrence model per field with an independent
  oracle and an exhaustive schema whose compile barrier was watched stopping an
  unclassified field.
- `d1e5de5c` made every armed checkpoint walk (unstrided).

That plausibly discharges part of the 165 outright. One concrete example:
`Print` declares `Incremental`, and `MemoryOwner::Prints` is now charged and
released in place at
`crates/rum-runtime/src/runtime/debug_tools/resources/maintained/diagnostics.rs:60,:73`.
`Message` and `Trace` have the same shape. Roughly 35-40 of the 165 names are
session-mutating (`Set`, `SetDelayed`, `AddTo`, `AppendTo`, `Sow`, `TagSet`,
`SetAttributes`, `$Path`, `$MessageList`, …) and may be in the same position.

**Re-probing before briefing a fix is a standing project rule**, and this is
exactly its case: the premise "nothing discharges them" is one day old and a
batch landed in between. So the first deliverable is not a conversion plan, it
is a census whose attribution is *verified* rather than written — the same
standard the ownership line applied to its 50 `UnknownHostPayload` rows.

## The gap that remains, stated so the plan aims at it

An armed checkpoint tests `S + W(expr)` against the limit: `S` is the
maintained session total, `W` is the in-flight expression. A builtin's
Rust-side local accumulator is in **neither**. `Table`'s `collected`
(`crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver.rs:269,:291`)
does not reach `Session` until publication or suspension, so walking the whole
session at every evaluation inside that loop does not count it. Same shape:
`NestList`/`FoldList` histories, `GroupBy`/`GatherBy` buckets,
`StringReplace`'s growing output, `Reap` collections.

That is the same class as the HIGH this line opened on — a ceiling a loop
walks straight through — relocated inside a single builtin call. Call this
third quantity `R`, as brief-516's section 1 does.

A-E left the seam for it: `AccountingBuiltinFn`, `AccountingContext` and
`Session::register_accounting_builtin`
(`crates/rum-runtime/src/runtime/accounting_mode/mod.rs:22,:110`). Exactly one
builtin uses it today — `CompoundExpression`, at
`crates/rum-stdlib/src/lib.rs:76` — and the module header states the
consequence plainly: *"Every other builtin uses the legacy adapter, even if it
declares Incremental"* (`accounting_mode/mod.rs:5-7`).

## Context pointers

- Read `AGENTS.md` (operational notes, Testing Policy hard lines) first.
- `crates/rum-stdlib/src/catalog/memory_growth_contract.rs` — the 643
  declarations; 165 `Incremental`, 133 `Preflight`, 290 `NonAmplifying`,
  55 `Delegated`.
- `crates/rum-runtime/src/runtime/metadata.rs:346-347` — the contract's text.
- `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:200-285` — the growth
  validator, its test-only gate, and the empty `Incremental` arm.
- `crates/rum-runtime/src/runtime/accounting_mode/` — the landed mode, the
  maintained owner array, and the accounting-builtin registration seam.
- `crates/rum-runtime/src/runtime/debug_tools/resources/maintained/` — the
  charge/release sites of the maintained owners (`owners.rs`, `scopes.rs`,
  `diagnostics.rs`).
- `crates/rum-runtime/src/runtime/debug_tools/resources/occurrence/schema.rs:20-42`
  — the exhaustive field schema: neither generated `let Session { .. }` pattern
  carries a `..`, so an unclassified new field cannot compile. Its inventory
  test is `crates/rum-runtime/src/runtime/tests/session_memory.rs:18-40`
  (95 rows today; brief-516 recorded 94).
- `crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/`
  — `spec.rs:338` is the existing exact projection for `Table`'s list shell;
  `driver.rs:269,:291` is the local accumulator it does not cover.
- `docs/dev/codex/brief-516-incremental-accounting-design.md` — sections 1-5
  (the model, the field inventory, the falsifier design, the cost recipe) and
  the F-J drafts at `:1441-1700`. **Raw material, not a dispatchable plan.**
- `docs/guide/resource-limits-and-sandbox.md` — the documented contract.
- A prior resolved declaration table exists at
  `target/brief-516/incremental-declarations-resolved.tsv` (165 rows: name,
  declaration site, catalog site, handler). **Regenerate it against the
  current tree**; use the old copy only to cross-check your own extraction.

## Scope

- Files you MAY modify: **NONE**, with two exceptions:
  - append your report to **this** file, `docs/dev/codex/brief-524-incremental-declaration-census.md`;
  - write scratch artifacts under `target/brief-524/` (TSV/CSV tables you
    reference from the report).
- Everything else is read-only. No source change, no test change, no doc
  change, no `Cargo.toml` change.

## What I want back

### 1. The census — one verified disposition per declaration (the core deliverable)

All 165 rows, as a table in the report and a TSV under `target/brief-524/`.
Each row carries a class, `path:line` evidence, and the specific facts below.

- **`CoveredByOwner`** — this builtin's retained growth lands entirely in a
  maintained `MemoryOwner`. Name the owner, the charge site, the release site,
  and state that the error / abandon / suspension exits reach them too. The
  owner merely *existing* is not the evidence; the evidence is that **this
  builtin's** growth reaches it on **every** exit.
- **`Preflight`** — an exact projection already runs before the allocation.
  Name the projection call site, state what it covers **and what it does
  not** (Table's shell is covered; its element contents are not).
- **`NeedsBuilder`** — a local retained builder that neither `S` nor
  `W(expr)` can see. Name the accumulator binding and its `path:line`, and
  its shape (output vector / history / bucket map / string / work list).
  Note whether the implementation is recursive or iterative/resumable.
- **`Delegated`** — an accountable callee charges it. Name that callee **on
  every branch**. A branch with no named callee makes the row
  `NeedsBuilder`.

Rules for the census, which are acceptance criteria and not advice:

- A row that is **partly** covered is `NeedsBuilder`, with the covered part
  stated. Do not round a mixed row up to covered.
- "Not currently instrumented", "assumed small", "bounded in practice" are
  **not** dispositions. A row you cannot settle goes in an explicit
  **unsettled** list with what you tried and what evidence is missing. That
  list is a deliverable, not a failure — a wrong confident row is worse.
- Report the **A-E delta** separately: every row whose disposition differs
  from what brief-516's section 3 recorded, and why. This is the re-probe and
  I want it visible.
- Report the class distribution, and the `NeedsBuilder` rows grouped by the
  family/file that would convert them.
- If you find a builtin declared `NonAmplifying` or `Preflight` that has an
  uncharged local builder, **report it and do not plan a fix** — that is a
  separate finding outside this line's scope.

### 2. The `R` term, designed against the code that actually landed

Design the accounted-builder protocol on the `AccountingContext` seam that
exists today, not the hypothetical one brief-516 assumed.

- The operations (reserve / charge / release / transfer) and their ownership
  model; what binds a builder's lifetime to the request generation.
- Every lifetime exit: normal publication, early return, `EvalError`, panic
  unwind, drop, suspension, resume, abandonment. A builder alive across a
  suspension must transfer **exactly once** — note that a builder held in a
  suspension frame is a root, and `SuspensionFrame: FrameRootTrace` makes a
  missing traversal a compile error.
- The mechanical recipe for converting one `Incremental` builtin: what
  changes at the registration site, at the handler signature, and inside the
  accumulation loop. Say how many `NeedsBuilder` rows need the accounting
  route and how many can charge through an existing path.
- Nested and sibling builders; a builder whose element size is only known
  after a user callback returns.
- **Constraint I am setting, do not reopen it:** `R` enters the **admission
  check only**. It does not appear in `MemoryInUse[]`, `MaxMemoryUsed[]` or
  `ByteCount` readings. The existing `S` oracle stays separate and
  numerically unchanged — this is decision 5 of the 2026-09-22 rulings, and
  the extension must not be presented as the same number.
- **Constraint:** no default-path cost. The builder operations erase when
  unarmed, on the same mode seam A-E built. Name where that erasure is
  proven, per step.

### 3. Enforcement — what keeps the class closed after the work

- What the empty `Incremental` arm (`eval_dispatch/mod.rs:276`) can actually
  check, and what it cannot. Say whether the check belongs in the test-only
  validator, in production, or both, and what each costs.
- A census test that makes a **new or unclassified `Incremental` declaration
  red**, in the shape of the existing exhaustive schema barrier at
  `crates/rum-runtime/src/runtime/debug_tools/resources/occurrence/schema.rs:20-42`. A
  declaration added without a disposition must not compile or must fail a
  required test — not rely on anyone remembering.

### 4. The falsifier plan

Every check this line ships needs a proof that it can fire, watched going red
by omitting a real charge or misdeclaring a real catalog builtin, then green.
Name them per step. A check nobody has seen fail is not known to work.

### 5. Cost

- The exposed microbench labels for each conversion family (`Table`, `Map`,
  `Fold`/`FoldList`, `Nest`/`NestList`, association grouping, string
  replacement) plus their same-tier neighbors.
- The unarmed cost of this line is **cumulative on top of A-E's landed
  +0.965% / +3.239% / +1.018%**. Each step reports against both its own step
  base and the pre-line base, and the close-out reports the running total.
- A bar is a data point, not a stop condition (standing ruling 2026-09-22).
  If a step costs more than expected, report the curve; do not stop and do
  not offer "stop here" beside an unwalked path.
- Note the known hazard: these builders sit in the evaluator's hottest
  loops, and a token threaded through an inner loop is the shape that cost
  2-7% on the traversal-depth line. That is why pilots precede the bulk
  conversion.

### 6. The drafted implementation briefs

In `TEMPLATE.md` shape, one per step, each with its Scope list built from
**current** file paths, its gate tests, its falsifier, and its A/B labels.
I expect roughly 5-7: the protocol plus 2-3 pilots, then family batches, then
the close-out with the census test and the acceptance gate.

State explicitly which parts of the old F-J drafts survive A-E and which are
obsolete. Do not copy their Scope lists — they were written against a
different tree and several say the director must expand them before dispatch.

### 7. Open questions for the director

Anything you could not settle, anything where two readings of the contract
give different work, and any decision you think is mine rather than yours.

## Settled — do not reopen

- **The occurrence model stands** (`902aea1f`). Repeated shared subtrees are
  charged by occurrence. Do not propose unique or physical allocation
  counting; that would change the ceiling's meaning and is a separate ruling.
- **The unstrided armed walk and the independent walker stay.** Nothing in
  this line removes them.
- **Reclassification is not a way to avoid work.** Moving a declaration to
  `Preflight` requires the `Preflight` falsifier watched red on that builtin;
  moving it to `Delegated` requires a named accountable callee on every
  branch. A reclassification into an unenforced contract is a deferral
  wearing a repair's clothes, and this project has been burned by it before.
- **`Symbol`, the interner, the gensym quota and the snapshot format are out
  of scope.** The snapshot line is next and is not yours to anticipate.
- The parked interner quota work (briefs 505-508) is not reopened here.

## rum / Rust pitfalls (read twice)

1. **A search for `self.field.insert` misses `entry()`, borrowed mutation and
   `OnceLock::get_or_init`.** brief-516 recorded this; the same trap applies
   to finding a builtin's accumulator.
2. **A held argument is not an evaluated result.** Do not size a builtin's
   growth from its raw arguments when the handler evaluates them.
3. **Shared children are multiple occurrences**, not one. A census row that
   reasons "it is the same subtree, so it is charged once" is wrong under the
   occurrence model.
4. **Selection is not non-amplification.** A builtin that only selects from
   its input can still hold auxiliary builder storage (seen sets, cycle sets,
   position lists) simultaneously live.
5. **A callback's result size is unknown before it returns.** An honest
   precheck cannot cover it; the charge has to happen before the value is
   retained and before the next iteration starts.
6. **The growth validator is test-only** (`eval_dispatch/mod.rs:205,:366`).
   Do not describe it as production enforcement, and do not assume a green
   test suite means production charges anything.
7. **A conclusion from a count you extracted with a regex is suspect.** This
   repository has produced wrong counts from backtick-splitting and from
   `grep -rl | head -1` more than once. Cross-check every derived count
   against a handful of live source reads, and say which you checked.

## Verification (MANDATORY — the evidence IS the deliverable)

No source change, so no test gate. Instead:

- State every command you ran to derive the census, and persist its raw
  output to a file under `target/brief-524/` **before** parsing it.
- Paste the class distribution and at least ten full census rows verbatim,
  chosen to include at least one of each class and at least two rows whose
  disposition changed because of A-E.
- Paste the regenerated declaration count and its breakdown, and say how you
  cross-checked it.
- State your unsettled list explicitly, or state that it is empty.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Do NOT modify any file outside the two exceptions named in Scope. If you
  believe you must, STOP and write that in the report instead.
- Touch nothing outside this repo. No network. Commands limited to file
  reads, `cargo` (for `cargo metadata`/`check` only if you need it to resolve
  a symbol), `rg`/`grep`, and `python3` for parsing.
- **Testing Policy is a hard line** (`AGENTS.md`): no black-box comparison
  against other vendors' engines, no oracle harnesses, no compatibility
  matrices, no third-party engine outputs.
- **No code.** This brief returns a plan and drafted briefs. If you find
  yourself editing a `.rs` file, you have left the brief.
- Do not pad the census to completeness. An honest unsettled row is worth
  more than a confident wrong one, and I will check the confident ones.

## Report format (your final message, appended to this file)

1. What changed — "design only, no source change", plus the scratch artifacts
   you wrote under `target/brief-524/`.
2. Verification — the commands, the verbatim rows and counts named above.
3. The census, the `R` design, the enforcement design, the falsifier plan,
   the cost plan, and the drafted implementation briefs.
4. Stop-loss / open questions — the unsettled list, and the decisions you
   think are the director's.
