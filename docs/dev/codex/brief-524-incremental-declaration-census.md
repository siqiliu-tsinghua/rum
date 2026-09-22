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

## Report — 2026-09-22

### 1. What changed and the result

**Design only, no source change.** This report is the only repository document changed. All other writes are under `target/brief-524/`. No tests, builds, benchmarks, guest programs, or vendor engines were executed.

The current tree has **165 Incremental declarations: 162 NeedsBuilder and three Unsettled**. There are **zero verified whole-builtin CoveredByOwner, Preflight, or Delegated dispositions** in this census. This does not mean A–E did nothing: it covers the installed/session-owned portion of many of these operations. A local rendering, copied selection, history, bucket, or work list still makes the whole row NeedsBuilder under this brief's mixed-row rule.

The three unsettled declarations are **Nest, RegisterExceptionType, SyntaxInformation**. They are explicitly investigated below, not included in the 162 conversion count. The proposed implementation sequence has seven steps, with feasibility questions that must be resolved in the protocol step before claiming complete coverage.

Scratch deliverables:

- [census.tsv](../../../target/brief-524/census.tsv): all 165 dispositions, declarations, catalogs, handler chains, concrete accumulator evidence, shapes, flow, covered portions, and proposed routes.
- [incremental-declarations-regenerated.tsv](../../../target/brief-524/incremental-declarations-regenerated.tsv), [raw-catalog-contracts.tsv](../../../target/brief-524/raw-catalog-contracts.tsv), [counts-crosscheck.json](../../../target/brief-524/counts-crosscheck.json): independently regenerated inventory and catalog cross-check.
- [families.tsv](../../../target/brief-524/families.tsv), [unsettled.tsv](../../../target/brief-524/unsettled.tsv), [ae-delta.tsv](../../../target/brief-524/ae-delta.tsv): conversion grouping, unresolved rows, and comparison with what brief-516 actually recorded.
- [evidence-snippets.tsv](../../../target/brief-524/evidence-snippets.tsv), `declarations.json`, `census-notes.json`, `function-index.json`: supporting source evidence and intermediate resolution data.
- [microbench-labels.tsv](../../../target/brief-524/microbench-labels.tsv), [draft-scopes.tsv](../../../target/brief-524/draft-scopes.tsv): actual exposed labels and checked current scope paths.
- [commands.jsonl](../../../target/brief-524/commands.jsonl), [commands.md](../../../target/brief-524/commands.md), [reads.jsonl](../../../target/brief-524/reads.jsonl), `raw/`, `source-snapshot.tsv`, `verification.txt`, `brief-original.md`, `bootstrap-exceptions.txt`: command provenance, byte-for-byte source/search captures, counts, and integrity evidence. Report-assembly scratch is also retained.

Paths below use **R/** = `crates/rum-runtime/src/runtime/`, **S/** = `crates/rum-stdlib/src/`, and **H/** = `crates/rum-host/`. TSV evidence uses full repository paths. A reference such as `:269,291` means two anchors in that same file.

### 2. Verification and its limits

The exact Python heredoc derivation commands, including every embedded rg invocation and parsing operation, are recorded in `commands.jsonl`. `reads.jsonl` maps reads to full raw byte snapshots and SHA256 values, and searches to raw stdout/stderr and exit status. The derivation operations were:

1. Read AGENTS, the controlling brief, the architecture/testing/organization documents, the relevant TODO rulings, TEMPLATE, brief-516, and the A–E report evidence.
2. Run `rg --files crates scripts docs -g '*.rs' -g '*.md' -g '*.sh' -g '*.py'`; save the listing, snapshot source bytes, and record 1,491 file hashes.
3. Parse anchored inventory declarations; independently enumerate `BuiltinSpec::...` constructor blocks in the runtime and stdlib catalogs. Compare the complete name/contract sets, including duplicate runtime/stdlib registrations.
4. Resolve function candidates from the saved source files, then follow live wrapper/dispatch/helper reads. Remove test helper homonyms; treat the four special variables as symbol evaluation, not ordinary handlers. Function-name matches locate code; they are not evidence of accounting.
5. Follow positive builder paths, including borrowed mutation, entry-based buckets, collects, persistent/resumable state, and formatting. Assign NeedsBuilder only with an identified local payload; separately record partial owner/projection coverage. A positive uncovered branch settles NeedsBuilder without claiming every other branch has been audited for conversion.
6. Read the two old brief-516 TSVs only after generating the current declarations; compare names and resolve the old adapter shortcomings. Both old files are **headerless** and contain 165 rows, not 164. An intermediate exploratory display subtracted a nonexistent header; the final cross-check corrects it.
7. Parse literal `rumMicroBench`/`rumMicroOnce` calls from the saved benchmark script; manually check representative statements and missing-family searches. Validate every final evidence path/line and every proposed current scope path.
8. Generate the final tables and report from those saved artifacts; check the original brief remains an unchanged prefix and the source snapshot has no other changes.

The raw-first rule was applied to source/search material before extraction, including the independent catalog re-extraction. Failed reads of guessed namespace/string/polynomial paths and failed draft-path assertions remain in the command history; current-tree rg discovery corrected them before final tables. No failed read supplies a census conclusion.

**Process exception:** before the brief's command whitelist had been read, the bootstrap included `git status --short && rg --files -g AGENTS.md -g '!target' -g '!vendor'`, alongside reading this brief. That read-only git command was outside the whitelist. Its output showed `AGENTS.md` and no status entries; no git mutation occurred. Bootstrap output was not saved contemporaneously before inspection. A later scratch-only progress read was also logged after inspection. `bootstrap-exceptions.txt` discloses these exceptions; a later capture is not presented as retroactive compliance.

Regenerated output, verbatim from `verification.txt`:

```text
INSTALLED_DECLARATIONS 643
NonAmplifying 290
Incremental 165
Preflight 133
Delegated 55
RAW_CATALOG_ENTRIES 661
RAW_CATALOG_UNIQUE_NAMES 643
CATALOG_MISSING 0; EXTRA 0; CONTRACT_MISMATCHES 0
NO_DEFAULT_HASH_DECLARATIONS 637 (six digest Preflight rows omitted)
OLD_RESOLVED_ROWS 165; OLD_ENTRYPOINT_ROWS 165; BOTH_HEADERLESS
OLD_TO_CURRENT_NAME_SYMMETRIC_DIFFERENCE 0
CENSUS_ROWS 165; UNIQUE_NAMES 165
CoveredByOwner 0
Preflight 0
NeedsBuilder 162
Delegated 0
Unsettled 3
NEEDS_COMPANION_ROUTE 158; NEEDS_SYMBOL_PATH 4; ALREADY_SUFFICIENT_EXISTING_R_PATH 0
EXPOSED_MICROBENCH_LABELS 119
S_SCHEMA_ROWS 95; ACCOUNTED_OWNERS 53
TESTS_RUN 0; BENCHMARKS_RUN 0; CARGO_RUNS 0
```

The 643-name inventory independently matches **661 raw catalog entries, deduplicating to 643 names**, with zero missing names, extras, or contract mismatches. Default features enable hash (`S/../Cargo.toml:13`); disabling it removes six Preflight digest names, leaving 637 total and still 165 Incremental. The existing inventory test explicitly checks both totals and installed registrations (`S/catalog/memory_growth_contract.rs:668–749`); its code was read, **not run**.

Live source cross-checks included: the four `BuiltinSpec::new` special variables in `R/builtin_catalog/namespace/variables.rs:9,25,30,40`; a normal stdlib constructor at `S/catalog/functional.rs:9,65`; runtime `Session::eval_part` at `R/builtin_catalog/rules.rs:5`; bare adapters for Public/PackageScope at `R/builtin_adapters.rs:422,431`; Table's wrapper at `S/control_flow.rs:107`; rewrite wrappers at `S/rewrite_builtins.rs:14,38,46,54,62,70`; Print and Message handlers and their owner helpers. These cover constructor shapes and special routes rather than merely checking the first regex match.

Two corrections to the brief's premises matter:

- The current S schema has **95 fields and 53 accounted owners**, not 52: `R/tests/session_memory.rs:22,28`, `R/accounting_mode/mod.rs:29`.
- The validator is diagnostic/test enforcement, not normal production charging. In addition to test/feature builds, `R/eval_dispatch/mod.rs:366–381` permits explicit `RUM_TEST_BUILTIN_MEMORY_GROWTH` opt-in in an ordinary binary. Its Incremental arm remains empty in all modes (`:276–278`).

The request for examples from every class and two class changes caused by A–E cannot be satisfied truthfully from these findings. Zero-member classes have no representative rows. Brief-516 §3.3 explicitly left **every row awaiting final disposition** (`:612–617`); it did not assign historical CoveredByOwner/Preflight/NeedsBuilder/Delegated classes. I have not invented either kind of example. The following full TSV records include the positive class and all unsettled rows, plus Print/Message/Trace where A–E changes the partial attribution.

```tsv
name	class	family	declaration	catalog	handler_entry	accumulator	evidence	shape	flow	covered_part_or_missing_proof	route
$Path	NeedsBuilder	namespace	crates/rum-stdlib/src/catalog/memory_growth_contract.rs:25	crates/rum-runtime/src/runtime/builtin_catalog/namespace/variables.rs:40	symbol evaluation (no callable handler) => crates/rum-runtime/src/runtime/eval_core/symbol.rs:41	paths	crates/rum-runtime/src/runtime/eval_core/symbol.rs:41	output vector of path strings	iterative	PackageSearchPaths owns source only	existing symbol evaluation mode
GroupBy	NeedsBuilder	associations	crates/rum-stdlib/src/catalog/memory_growth_contract.rs:243	crates/rum-stdlib/src/catalog/associations_transform.rs:307	associations::eval_group_by => crates/rum-stdlib/src/associations/grouping.rs:143	grouped	crates/rum-stdlib/src/associations/grouping/list.rs:98; crates/rum-stdlib/src/associations/grouping/association.rs:103; crates/rum-stdlib/src/associations/grouping.rs:29,49	bucket map and index	iterative grouping with nested recursive/resumable reducers	none verified	AccountingContext
Map	NeedsBuilder	functional	crates/rum-stdlib/src/catalog/memory_growth_contract.rs:330	crates/rum-stdlib/src/catalog/functional.rs:87	functional_ops::eval_map => crates/rum-stdlib/src/functional_ops/apply_map/mod.rs:78	work / completed / transformed_args	crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:103,107,174	work and output vectors	bounded recursion plus iterative fallback	Comap/ComapApply evaluate only after building mapped tree	AccountingContext
Message	NeedsBuilder	diagnostics	crates/rum-stdlib/src/catalog/memory_growth_contract.rs:351	crates/rum-runtime/src/runtime/builtin_catalog/namespace/symbols.rs:4	builtin_message => crates/rum-runtime/src/runtime/messages/builtins_messages.rs:5	arguments rendering; rendered_text	crates/rum-runtime/src/runtime/messages/message_catalog.rs:154; crates/rum-runtime/src/runtime/messages/builtins_messages.rs:22	formatted string and argument strings	iterative/resumable	Messages charge maintained/diagnostics.rs:20; release :36; local formatting precedes publication	AccountingContext companion
Nest	Unsettled	histories	crates/rum-stdlib/src/catalog/memory_growth_contract.rs:375	crates/rum-stdlib/src/catalog/functional.rs:262	functional_ops::eval_nest => crates/rum-stdlib/src/functional_ops/iteration.rs:47	current; empty items on collect_all=false	crates/rum-stdlib/src/functional_ops/iteration.rs:253; crates/rum-stdlib/src/functional_ops/mod.rs:120	see unsettled investigation	see unsettled investigation	callback branch delegates to Session::evaluate; zero-count/operator-form/error branches have no named charged callee or exact projection; no growing history demonstrated	re-probe before choosing route
NestList	NeedsBuilder	histories	crates/rum-stdlib/src/catalog/memory_growth_contract.rs:376	crates/rum-stdlib/src/catalog/functional.rs:288	functional_ops::eval_nest_list => crates/rum-stdlib/src/functional_ops/iteration.rs:126	items	crates/rum-stdlib/src/functional_ops/iteration.rs:334,338	history vector / persistent sequence	recursive driver and resumable	callback value is checked by evaluate; earlier items stay local	AccountingContext
Print	NeedsBuilder	diagnostics	crates/rum-stdlib/src/catalog/memory_growth_contract.rs:440	crates/rum-runtime/src/runtime/builtin_catalog/scoping.rs:282	builtin_print => crates/rum-runtime/src/runtime/builtins_input.rs:104	rendered (fragment Vec then join)	crates/rum-runtime/src/runtime/builtins_input.rs:105,108	string vector and string	iterative	Prints charge maintained/diagnostics.rs:60; release :73; local formatting precedes push; sink branch session_helpers.rs:383	AccountingContext
RegisterExceptionType	Unsettled	state	crates/rum-stdlib/src/catalog/memory_growth_contract.rs:462	crates/rum-stdlib/src/catalog/control.rs:378	control_flow::eval_register_exception_type => crates/rum-stdlib/src/control_flow.rs:452 -> crates/rum-runtime/src/runtime/kernel_data/control_data/exceptions.rs:27	type_name; requested_parent; registration output	crates/rum-runtime/src/runtime/kernel_data/control_data/exceptions.rs:36; crates/rum-runtime/src/runtime/kernel_data/control_data/exceptions.rs:79	see unsettled investigation	see unsettled investigation	ExceptionTypes refresh :81 covers registry; local cloned/formatted strings and conflict/invalid output paths lack full owner/projection proof	re-probe before choosing route
Sow	NeedsBuilder	diagnostics	crates/rum-stdlib/src/catalog/memory_growth_contract.rs:525	crates/rum-runtime/src/runtime/builtin_catalog/debug.rs:172	Session::eval_sow => crates/rum-runtime/src/runtime/debug_tools/reap.rs:21	matched	crates/rum-runtime/src/runtime/debug_tools/reap.rs:32,40	match-position vector	iterative; matcher can suspend	ReapScopes incoming value/tag charged :52/:54	AccountingContext
StringReplace	NeedsBuilder	strings	crates/rum-stdlib/src/catalog/memory_growth_contract.rs:556	crates/rum-stdlib/src/catalog/strings_match.rs:140	strings::eval_string_replace => crates/rum-stdlib/src/strings/patterns/search.rs:198	output; rendered	crates/rum-stdlib/src/strings/patterns/search/rules.rs:447; crates/rum-stdlib/src/strings/patterns/search/guarded/replace.rs:523	growing string plus list-of-string output	iterative/recursive/resumable	none for the cited local payload	AccountingContext companion
SyntaxInformation	Unsettled	state	crates/rum-stdlib/src/catalog/memory_growth_contract.rs:579	crates/rum-runtime/src/runtime/builtin_catalog/namespace/symbols.rs:234	builtin_syntax_information => crates/rum-runtime/src/runtime/namespace_tools/syntax.rs:4	stored clone or fixed rules template	crates/rum-runtime/src/runtime/namespace_tools/syntax.rs:24; crates/rum-runtime/src/runtime/namespace_tools/syntax.rs:122	see unsettled investigation	see unsettled investigation	stored read is not owner mutation; fallback builds fixed templates; full branch projection/callee proof missing	re-probe before choosing route
Table	NeedsBuilder	table	crates/rum-stdlib/src/catalog/memory_growth_contract.rs:581	crates/rum-stdlib/src/catalog/control.rs:123	control_flow::eval_table => crates/rum-stdlib/src/control_flow.rs:107 -> crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver.rs:53	collected	crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver.rs:269,291	output vector / persistent accumulator	iterative and resumable	spec.rs:338 projects list shell only; suspended frames are S	AccountingContext
Trace	NeedsBuilder	diagnostics	crates/rum-stdlib/src/catalog/memory_growth_contract.rs:624	crates/rum-runtime/src/runtime/builtin_catalog/debug.rs:5	builtin_trace => crates/rum-runtime/src/runtime/debug_tools/trace.rs:28	events / depths / trace builder	crates/rum-runtime/src/runtime/debug_tools/trace.rs:161,198,199	event copies and rendering output	capture/filter resumable; rendering builder	TraceEvents/TraceEventDepths maintained during capture; local copies survive truncate :203	AccountingContext
```

### 3. Full census and A–E delta

A NeedsBuilder entry identifies an actual local construction/retention site. It is not a claim that today's metadata causes charging, or that an owner of the source also owns a copied/rendered output. Shape/flow descriptions cover the cited implementation path; a shared family helper is evidence only where the named handler reaches it. “None” in the covered column means no sufficient coverage of the cited local payload, not that the builtin has no ordinary S/W checks.

The declaration/catalog/handler chain columns are in the TSV. The complete disposition table follows; no names are omitted.

| Name | Class / family | Accumulator and evidence | Shape / flow | Covered part or missing proof |
| --- | --- | --- | --- | --- |
| $ContextPath | NeedsBuilder / namespace | contexts — R/eval_core/symbol.rs:15 | output vector of owned strings; iterative | ContextPath owns source only |
| $MessageList | NeedsBuilder / namespace | messages/current_message_list — R/eval_core/symbol.rs:49 | output vector; iterative | LastMessageList/Messages own source only |
| $Packages | NeedsBuilder / namespace | packages — R/eval_core/symbol.rs:24 | output vector of owned strings; iterative | LoadedPackages owns source only |
| $Path | NeedsBuilder / namespace | paths — R/eval_core/symbol.rs:41 | output vector of path strings; iterative | PackageSearchPaths owns source only |
| Accumulate | NeedsBuilder / functional | accumulated — S/functional_ops/sequence/window.rs:299,331 | output history; iterative and resumable | none verified |
| AddTo | NeedsBuilder / state | paths / replacements / updated_target — R/definitions/definition_updates.rs:339,556,557 | Part path and replacement work vectors; iterative and resumable | Definitions cover symbol stores; all operations admit Part targets at :252; AppendTo fast path only one branch |
| AppendTo | NeedsBuilder / state | paths / replacements / updated_target — R/definitions/definition_updates.rs:339,556,557 | Part path and replacement work vectors; iterative and resumable | Definitions cover symbol stores; all operations admit Part targets at :252; AppendTo fast path only one branch |
| Apply | NeedsBuilder / functional | work / completed / transformed_args — S/functional_ops/apply_map/traversal.rs:103,107,174 | work and output vectors; bounded recursion plus iterative fallback | Comap/ComapApply evaluate only after building mapped tree |
| ApplyTo | NeedsBuilder / state | paths / replacements / updated_target — R/definitions/definition_updates.rs:339,556,557 | Part path and replacement work vectors; iterative and resumable | Definitions cover symbol stores; all operations admit Part targets at :252; AppendTo fast path only one branch |
| ArgumentsOptions | NeedsBuilder / state | options; flattened — R/options/arguments.rs:70; R/options/arguments.rs:108 | option/output vectors; iterative | none for the cited local payload |
| Array | NeedsBuilder / functional | outputs — S/functional_ops/arrays/generators.rs:202,209,224 | tensor output vector; iterative and resumable | generated-node projection omits callback result payloads |
| ArrayFilter | NeedsBuilder / functional | operator_args / items / outputs — S/functional_ops/arrays/filter.rs:34,101; S/functional_ops/arrays/tensor.rs:276,304 | window and output vectors; recursive block assembly; iterative/resumable callbacks | none verified |
| ArrayReduce | NeedsBuilder / functional | operator_args / values / outputs — S/functional_ops/arrays/tensor.rs:121,123,276 | tensor work and output vectors; recursive tensor assembly; iterative/resumable callbacks | none verified |
| AssociateTo | NeedsBuilder / associations | entries / updates / replacements — S/associations/mutation.rs:269,270,652,669 | entry and Part replacement vectors; iterative and resumable | Definitions owns installed target; literal/Part preparation remains local |
| Association | NeedsBuilder / associations | evaluated_entries — S/associations/basic.rs:49,79 | entry vector; iterative and resumable | none verified |
| AssociationKeyDeflatten | NeedsBuilder / associations | nested / work / completed — S/associations/key_reshape/traversal.rs:80,316,317 | nested bucket tree and worklist; bounded recursion plus iterative tail | none verified |
| AssociationKeyFlatten | NeedsBuilder / associations | flattened / path / work — S/associations/key_reshape/traversal.rs:406,410,453,457 | path vectors and output entries; bounded recursion plus iterative tail | none verified |
| AssociationMap | NeedsBuilder / associations | entries / mapped_items — S/associations/resumable/map.rs:57,81,139,230 | entry/output vector; iterative and resumable | none verified |
| AssociationThread | NeedsBuilder / associations | association_thread_entries collect — S/associations/transforms.rs:192,198,209 | entry output vector; iterative; delayed/immediate forms | none verified |
| BlockMap | NeedsBuilder / functional | results — S/functional_ops/sequence/window.rs:386,414 | output vector; iterative and resumable | none verified |
| Cases | NeedsBuilder / rewrite | walker; matches — R/kernel_data/rewrite_data/cases/collect.rs:220; R/kernel_data/rewrite_data/cases/collect.rs:221 | traversal work list and match vector; iterative/resumable | none for the cited local payload |
| CoefficientList | NeedsBuilder / algebra | terms; coefficients; term_exprs — S/polynomials/core.rs:123; S/polynomials/core/construction.rs:62; S/polynomials/core.rs:410 | polynomial intermediates and output vectors/maps; recursive/iterative | none for the cited local payload |
| Comap | NeedsBuilder / functional | work / completed / transformed_args — S/functional_ops/apply_map/traversal.rs:103,107,174 | work and output vectors; bounded recursion plus iterative fallback | Comap/ComapApply evaluate only after building mapped tree |
| ComapApply | NeedsBuilder / functional | work / completed / transformed_args — S/functional_ops/apply_map/traversal.rs:103,107,174 | work and output vectors; bounded recursion plus iterative fallback | Comap/ComapApply evaluate only after building mapped tree |
| ConformSchema | NeedsBuilder / schema | conformed_by_index; default_entries; errors — S/schema.rs:207; S/schema.rs:208; S/schema.rs:334 | conformance output buckets and error vector; recursive; entry evaluation resumable | none for the cited local payload |
| Contexts | NeedsBuilder / namespace | contexts — R/builtins_context_loader.rs:124 | BTreeSet of context strings then output vector; iterative | none for the cited local payload |
| Counts | NeedsBuilder / associations | values / counts / positions — S/associations/aggregate.rs:196,526,545 | value vector and bucket index; iterative; association values resumable | none verified |
| CountsBy | NeedsBuilder / associations | counts — S/associations/resumable/aggregate.rs:225 | indexed count buckets; iterative and resumable | none verified |
| D | NeedsBuilder / algebra | derivatives; terms — S/calculus/differentiation.rs:91; S/calculus/differentiation.rs:110; S/calculus/differentiation/iterative.rs:33 | derivative vectors and work list; bounded recursive plus iterative | none for the cited local payload |
| Decrement | NeedsBuilder / state | paths / replacements / updated_target — R/definitions/definition_updates.rs:339,556,557 | Part path and replacement work vectors; iterative and resumable | Definitions cover symbol stores; all operations admit Part targets at :252; AppendTo fast path only one branch |
| DefaultValues | NeedsBuilder / state | items — R/messages/builtins_messages_inspect.rs:195 | rendered rule vector; iterative | DefaultValues owns source only |
| DefinePackage | NeedsBuilder / state | rewrites / public / package_scope / private_index — R/scoping/packages.rs:255,256,257,283 | maps, declaration vectors and body forms; iterative preparation and resumable install | SealedUnits retained after installation; pre-install plan remains local |
| Definition | NeedsBuilder / state | statements — R/messages/builtins_messages_inspect.rs:240 | statement vector and formatted output; iterative | Definitions owns source only |
| DeleteDuplicatesBy | NeedsBuilder / structure | keys / kept — S/structure_ops/dedup.rs:164,165,190,191 | adaptive seen set and output vector; iterative and resumable | none verified |
| Differences | NeedsBuilder / structure | current / differences — S/structure_ops/list_plumbing.rs:25,78,80 | difference output vectors; iterative callback loop | none verified |
| DivideBy | NeedsBuilder / state | paths / replacements / updated_target — R/definitions/definition_updates.rs:339,556,557 | Part path and replacement work vectors; iterative and resumable | Definitions cover symbol stores; all operations admit Part targets at :252; AppendTo fast path only one branch |
| Dot | NeedsBuilder / functional | terms / output_scalars — S/functional_ops/arrays/tensor.rs:367,368,432,453 | contraction and output vectors; iterative and resumable; recursive tensor parse | none verified |
| DownValues | NeedsBuilder / state | items — R/definitions/definition_catalog/entries.rs:125 | definition-rule output vector; iterative | Definitions owns source only |
| DumpSave | NeedsBuilder / namespace | blocks; rendered — R/package_tools.rs:421; R/package_tools.rs:289 | rendered definition string list and joined string; iterative | Definitions owns source only; no snapshot format change |
| Echo | NeedsBuilder / diagnostics | render_echo_fragment / format output — R/debug_tools/basic.rs:37,41 | rendered string; rendering traversal | Prints only after push_print |
| EvaluationData | NeedsBuilder / diagnostics | messages; prints; trace; output entries — R/debug_tools/evaluation_data.rs:74; R/debug_tools/evaluation_data.rs:146 | taken diagnostics plus rendered output vectors; iterative/resumable | Messages/Prints/TraceEvents release source at take; locals survive |
| ExceptionTypes | NeedsBuilder / diagnostics | list_expr collect — R/kernel_data/control_data/exceptions.rs:99,105 | output registration vector; iterative | ExceptionTypes owns source only |
| Expand | NeedsBuilder / algebra | terms; coefficients; term_exprs — S/polynomials/core.rs:123; S/polynomials/core/construction.rs:62; S/polynomials/core.rs:410 | polynomial intermediates and output vectors/maps; recursive/iterative | none for the cited local payload |
| Extract | NeedsBuilder / structure | items; missing_keys — R/kernel_data/structure_data/selectors.rs:218; R/kernel_data/structure_data/selectors.rs:255 | selected-output and missing-key vectors; iterative/recursive | none for the cited local payload |
| Factor | NeedsBuilder / algebra | terms; coefficients; term_exprs — S/polynomials/core.rs:123; S/polynomials/core/construction.rs:62; S/polynomials/core.rs:410 | polynomial intermediates and output vectors/maps; recursive/iterative | none for the cited local payload |
| FileNames | NeedsBuilder / namespace | matches; pending; child_dirs — R/builtins_context_loader/file_names_glob.rs:91; R/builtins_context_loader/file_names_glob.rs:93; R/builtins_context_loader/file_names_glob.rs:114 | output and directory work lists; iterative | none for the cited local payload |
| FixedPoint | NeedsBuilder / histories | seen / items — S/functional_ops/iteration.rs:439,465,467 | cycle history and output history; recursive driver and resumable | FixedPoint without a cap keeps seen even when collect_all=false |
| FixedPointList | NeedsBuilder / histories | seen / items — S/functional_ops/iteration.rs:439,465,467 | cycle history and output history; recursive driver and resumable | FixedPoint without a cap keeps seen even when collect_all=false |
| Fold | NeedsBuilder / histories | items — S/functional_ops/sequence/fold/plain.rs:139,176 | retained copied input vector; iterative and resumable | current result delegates to evaluate; no output history when collect_all=false |
| FoldList | NeedsBuilder / histories | history / items — S/functional_ops/sequence/fold/plain.rs:34,63,139 | history and input vectors; iterative and resumable | none verified |
| FoldPair | NeedsBuilder / histories | state.items / current_state / last_output — S/functional_ops/sequence/fold/pair.rs:8,32,227 | input work vector and retained pair values; iterative and resumable | not a growing output history; items and next_state survive projector callbacks |
| FoldWhile | NeedsBuilder / histories | history — S/functional_ops/sequence/fold/while_loop.rs:37,70,278 | history vector; iterative and resumable | none verified |
| GatherBy | NeedsBuilder / structure | state.keys / decorated — S/structure_ops/collection_stats.rs:400,483,509 | key matrix and buckets; iterative and resumable | none verified |
| GroupBy | NeedsBuilder / associations | grouped — S/associations/grouping/list.rs:98; S/associations/grouping/association.rs:103; S/associations/grouping.rs:29,49 | bucket map and index; iterative grouping with nested recursive/resumable reducers | none verified |
| Increment | NeedsBuilder / state | paths / replacements / updated_target — R/definitions/definition_updates.rs:339,556,557 | Part path and replacement work vectors; iterative and resumable | Definitions cover symbol stores; all operations admit Part targets at :252; AppendTo fast path only one branch |
| Information | NeedsBuilder / state | entries — R/messages/builtins_messages_inspect.rs:336 | metadata association output; iterative | definition/option owners own sources only |
| Inner | NeedsBuilder / functional | terms / output_scalars — S/functional_ops/arrays/tensor.rs:367,368,432,453 | contraction and output vectors; iterative and resumable; recursive tensor parse | none verified |
| Insert | NeedsBuilder / structure | canonical_paths / canonical — S/structure_ops/sequence/insert.rs:46,49,134 | path vectors and reconstruction work; iterative planning; structural rebuild | none verified |
| JoinAcross | NeedsBuilder / associations | positions / matched_right / joined / index — S/associations/join_across.rs:47,224,225,281 | join bucket map and output vectors; iterative | none verified |
| KeyDropFrom | NeedsBuilder / associations | selected / kept / replacements — S/associations/mutation.rs:412,416,652,669 | key set and replacement/output vectors; iterative and resumable | Definitions owns installed target; literal branch :321 builds output only |
| KeyGroupBy | NeedsBuilder / associations | grouped — S/associations/key_reshape.rs:101,127 | indexed entry buckets; iterative and resumable | none verified |
| KeyIntersection | NeedsBuilder / associations | keys / per-collection output — S/associations/transforms/key_collections.rs:210,233,236 | key index and output vectors; iterative | none verified |
| KeyMap | NeedsBuilder / associations | render_association collect — S/associations/transforms.rs:36 | entry output vector; iterative construction | none verified |
| KeySelect | NeedsBuilder / associations | selected — S/associations/resumable/key_select.rs:43,69 | entry selection vector; iterative and resumable | none verified |
| KeySortBy | NeedsBuilder / associations | decorated — S/associations/resumable/key_sort.rs:150,182,272 | sort-key vector; iterative and resumable; comparator continuation | none verified |
| KeyUnion | NeedsBuilder / associations | union_entries / seen / fill_values — S/associations/transforms/key_collections.rs:330,357,420,441 | union index, missing-value and output vectors; iterative and resumable | none verified |
| KeyValueMap | NeedsBuilder / associations | List collect — S/associations/transforms.rs:73,74 | callback-call output vector; iterative construction | none verified |
| Keys | NeedsBuilder / associations | keys / work / completed — S/associations/basic/keys.rs:74,109,110 | output and work vectors; bounded recursion plus iterative tail | none verified |
| LazyTuples | NeedsBuilder / lazy | list_of_lists — S/lazy.rs:495,511 | repeated-list descriptor vector; iterative construction | one-argument form shares input; two-argument form allocates count occurrences |
| Level | NeedsBuilder / rewrite | walker; parts — R/tree_walkers/level.rs:80; R/tree_walkers/level.rs:81 | traversal work list and selected output vector; bounded recursive plus iterative | none for the cited local payload |
| Lookup | NeedsBuilder / associations | positions / outcomes / rendered — S/associations/basic/lookup.rs:295,517,222 | lookup index and nested outputs; bounded recursion plus iterative/resumable paths | none verified |
| Map | NeedsBuilder / functional | work / completed / transformed_args — S/functional_ops/apply_map/traversal.rs:103,107,174 | work and output vectors; bounded recursion plus iterative fallback | Comap/ComapApply evaluate only after building mapped tree |
| MapAll | NeedsBuilder / functional | work / completed / transformed_args — S/functional_ops/apply_map/traversal.rs:103,107,174 | work and output vectors; bounded recursion plus iterative fallback | Comap/ComapApply evaluate only after building mapped tree |
| MapApply | NeedsBuilder / functional | work / completed / transformed_args — S/functional_ops/apply_map/traversal.rs:103,107,174 | work and output vectors; bounded recursion plus iterative fallback | Comap/ComapApply evaluate only after building mapped tree |
| MapAt | NeedsBuilder / functional | state.paths / current — S/functional_ops/targeted_map.rs:299,305,316 | path work vector and replacement tree; iterative and resumable | none verified |
| MapIndexed | NeedsBuilder / functional | transformed_args / work / completed — S/functional_ops/apply_map/traversal/map_indexed.rs:144,205,210 | path worklists and output vectors; bounded recursion plus iterative fallback | none verified |
| MapThread | NeedsBuilder / functional | all_args / mapped / child_inputs — S/functional_ops/targeted_map.rs:499,520,522 | threaded output vectors; recursive | none verified |
| MaxBy | NeedsBuilder / structure | decorated — S/structure_ops/collection_stats.rs:745,769 | retained callback keys and selection output; iterative and resumable | none verified |
| MaximalBy | NeedsBuilder / structure | decorated — S/structure_ops/collection_stats.rs:745,769 | retained callback keys and selection output; iterative and resumable | none verified |
| Merge | NeedsBuilder / associations | grouped / merged — S/associations/resumable/aggregate.rs:108,135 | bucket and output vectors; iterative and resumable | none verified |
| MergeNested | NeedsBuilder / associations | merged / work — S/associations/resumable/aggregate/merge_nested.rs:53,83,136 | nested bucket worklist and output; bounded recursion plus iterative/resumable tail | none verified |
| Message | NeedsBuilder / diagnostics | arguments rendering; rendered_text — R/messages/message_catalog.rs:154; R/messages/builtins_messages.rs:22 | formatted string and argument strings; iterative/resumable | Messages charge maintained/diagnostics.rs:20; release :36; local formatting precedes publication |
| MessageDefinitions | NeedsBuilder / diagnostics | items — R/messages/message_catalog.rs:315 | message metadata output vector; iterative | message owners own source only |
| MessageTags | NeedsBuilder / diagnostics | tags — R/messages/message_catalog.rs:355 | tag vector/set; iterative | message owners own source only |
| Messages | NeedsBuilder / diagnostics | items — R/messages/message_catalog.rs:267 | message-rule output vector; iterative | Messages/message owners own source only |
| MinBy | NeedsBuilder / structure | decorated — S/structure_ops/collection_stats.rs:745,769 | retained callback keys and selection output; iterative and resumable | none verified |
| MinimalBy | NeedsBuilder / structure | decorated — S/structure_ops/collection_stats.rs:745,769 | retained callback keys and selection output; iterative and resumable | none verified |
| Names | NeedsBuilder / namespace | names; symbols — R/namespace_tools/names.rs:194; R/namespace_tools/names.rs:276 | name set and output vector; iterative | none for the cited local payload |
| Nest | Unsettled / histories | current; empty items on collect_all=false — S/functional_ops/iteration.rs:253; S/functional_ops/mod.rs:120 | see unsettled investigation; see unsettled investigation | callback branch delegates to Session::evaluate; zero-count/operator-form/error branches have no named charged callee or exact projection; no growing history demonstrated |
| NestList | NeedsBuilder / histories | items — S/functional_ops/iteration.rs:334,338 | history vector / persistent sequence; recursive driver and resumable | callback value is checked by evaluate; earlier items stay local |
| NestWhile | NeedsBuilder / histories | history — S/functional_ops/iteration.rs:610,617 | history vector / persistent sequence; recursive driver and resumable | none verified |
| NestWhileList | NeedsBuilder / histories | history — S/functional_ops/iteration.rs:610,617 | history vector / persistent sequence; recursive driver and resumable | none verified |
| Normal | NeedsBuilder / associations | normalize_association_expr collect — S/associations/basic.rs:270,278 | recursive output vectors; recursive; lazy branch must also be covered | none verified |
| Off | NeedsBuilder / diagnostics | selectors — R/debug_tools/message_switch.rs:32,68,71 | selector vector; iterative | DisabledMessageSelectors insertion :58/refresh :52 covers stored selectors only |
| On | NeedsBuilder / diagnostics | selectors — R/debug_tools/message_switch.rs:32,68,71 | selector vector; iterative | DisabledMessageSelectors insertion :58/refresh :52 covers stored selectors only |
| Once | NeedsBuilder / state | options / cache_key — R/package_tools.rs:105,134,140 | compiled option vector and retained cache key; resumable | OnceValues insert/refresh :145/:146 covers final cache; local options and key span evaluation |
| OptionValue | NeedsBuilder / state | values; names — R/options/builtins.rs:263; R/options/builtins.rs:340 | query and result vectors; iterative/resumable | SymbolOptions and OptionValueScopes own source only |
| Options | NeedsBuilder / state | rule_exprs result — R/options.rs:111 | rule output vector; iterative | SymbolOptions owns source only |
| Outer | NeedsBuilder / functional | outputs — S/functional_ops/arrays/generators.rs:202,209,224 | tensor output vector; iterative and resumable | generated-node projection omits callback result payloads |
| OwnValues | NeedsBuilder / state | items — R/definitions/definition_catalog/entries.rs:109 | definition-rule output vector; iterative | Definitions owns source only |
| PackageScope | NeedsBuilder / state | args.to_vec() — R/builtin_adapters.rs:437 | output argument vector; straight-line copy | identity-like marker; no accountable callee |
| Part | NeedsBuilder / structure | selected; first_position_by_key; selected_values — R/kernel_data/structure_data/selectors/apply.rs:256; R/kernel_data/structure_data/selectors/apply.rs:367; R/kernel_data/structure_data/selectors/apply.rs:381 | selection tree, lookup map and output vector; recursive; lazy branch evaluates/resumes | fast scalar path returns directly; multi-selector path remains local |
| Position | NeedsBuilder / rewrite | walker; positions — R/tree_walkers/position.rs:197; R/tree_walkers/position.rs:198 | traversal work list and position vector; iterative/resumable | none for the cited local payload |
| PositionIndex | NeedsBuilder / associations | entry pairs / IndexedGroups.entries — S/associations/aggregate.rs:135,140; S/associations/grouping.rs:57,60 | position buckets; iterative | none verified |
| PreDecrement | NeedsBuilder / state | paths / replacements / updated_target — R/definitions/definition_updates.rs:339,556,557 | Part path and replacement work vectors; iterative and resumable | Definitions cover symbol stores; all operations admit Part targets at :252; AppendTo fast path only one branch |
| PreIncrement | NeedsBuilder / state | paths / replacements / updated_target — R/definitions/definition_updates.rs:339,556,557 | Part path and replacement work vectors; iterative and resumable | Definitions cover symbol stores; all operations admit Part targets at :252; AppendTo fast path only one branch |
| PrependTo | NeedsBuilder / state | paths / replacements / updated_target — R/definitions/definition_updates.rs:339,556,557 | Part path and replacement work vectors; iterative and resumable | Definitions cover symbol stores; all operations admit Part targets at :252; AppendTo fast path only one branch |
| Print | NeedsBuilder / diagnostics | rendered (fragment Vec then join) — R/builtins_input.rs:105,108 | string vector and string; iterative | Prints charge maintained/diagnostics.rs:60; release :73; local formatting precedes push; sink branch session_helpers.rs:383 |
| Product | NeedsBuilder / table | collected — R/kernel_data/control_data/branching_loops/iteration_aggregate.rs:132,154 | term vector / persistent accumulator; iterative and resumable | arithmetic reduction runs only after collection; iteration-count bound is not memory accounting |
| Public | NeedsBuilder / state | args.to_vec() — R/builtin_adapters.rs:428 | output argument vector; straight-line copy | identity-like marker; no accountable callee |
| Reap | NeedsBuilder / diagnostics | state.groups / rendered_groups — R/debug_tools/reap.rs:94,135,157 | bucket groups and output vectors; iterative and resumable | ReapScopes charged while body runs; pop :95 transfers to invisible locals |
| RegisterExceptionType | Unsettled / state | type_name; requested_parent; registration output — R/kernel_data/control_data/exceptions.rs:36; R/kernel_data/control_data/exceptions.rs:79 | see unsettled investigation; see unsettled investigation | ExceptionTypes refresh :81 covers registry; local cloned/formatted strings and conflict/invalid output paths lack full owner/projection proof |
| Replace | NeedsBuilder / rewrite | state.work; state.completed; rebuilt_args — R/kernel_data/rewrite_data/replace/once/traversal.rs:146; R/kernel_data/rewrite_data/replace/once/traversal.rs:235 | traversal work and rebuilt output vectors; iterative/resumable | none for the cited local payload |
| ReplaceAll | NeedsBuilder / rewrite | state.work; state.completed; rewritten_args — R/definitions/application/replace_all.rs:217; R/definitions/application/replace_all.rs:301 | rewrite work and output vectors; repeated also retains compiled rules; iterative/resumable | none for the cited local payload |
| ReplaceAllUnheld | NeedsBuilder / rewrite | state.work; state.completed; rewritten_args — R/definitions/application/replace_all.rs:217; R/definitions/application/replace_all.rs:301 | rewrite work and output vectors; repeated also retains compiled rules; iterative/resumable | none for the cited local payload |
| ReplaceAt | NeedsBuilder / rewrite | paths; rules — R/kernel_data/structure_data/replace/replace_at.rs:75; R/kernel_data/structure_data/replace/replace_at.rs:117 | position work list and compiled rules retained through replacement callbacks; recursive/resumable | none for the cited local payload |
| ReplaceList | NeedsBuilder / rewrite | collected; pending; rule_results — R/kernel_data/rewrite_data/replace/list.rs:179; R/kernel_data/rewrite_data/replace/list.rs:180; R/kernel_data/rewrite_data/replace/list.rs:229 | replacement-output and binding-work vectors; iterative/resumable | none for the cited local payload |
| ReplacePart | NeedsBuilder / rewrite | merged_plan; collected — R/kernel_data/structure_data/replace/replace_part.rs:110; R/kernel_data/structure_data/replace/replace_part.rs:181 | replacement plan tree and target vector; recursive/resumable | none for the cited local payload |
| ReplaceRepeated | NeedsBuilder / rewrite | state.work; state.completed; rewritten_args — R/definitions/application/replace_all.rs:217; R/definitions/application/replace_all.rs:301 | rewrite work and output vectors; repeated also retains compiled rules; iterative/resumable | none for the cited local payload |
| ReverseSortBy | NeedsBuilder / structure | decorated — S/structure_ops/collection_stats.rs:745,769 | retained callback keys and selection output; iterative and resumable | none verified |
| Riffle | NeedsBuilder / structure | insertion_positions / riffled — S/structure_ops/collection_stats.rs:448,453 | position mask and output vector; iterative | element-limit check :451 is not a byte projection |
| RumProfileSnapshot | NeedsBuilder / diagnostics | entries — R/evaluation_cache/profile/report.rs:58 | profile snapshot/sort vector; iterative; profile-enabled branch | env gate skips builder when disabled; enabled branch still local |
| Save | NeedsBuilder / namespace | blocks; rendered — R/package_tools.rs:421; R/package_tools.rs:289 | rendered definition string list and joined string; iterative | Definitions owns source only; no snapshot format change |
| SchemaErrors | NeedsBuilder / schema | conformed_by_index; default_entries; errors — S/schema.rs:207; S/schema.rs:208; S/schema.rs:334 | conformance output buckets and error vector; recursive; entry evaluation resumable | none for the cited local payload |
| Select | NeedsBuilder / functional | selected — S/functional_ops/sequence/select.rs:140,242,246 | selection output vector; iterative and resumable | precheck at :242/:264 counts nodes only, not selected children/source copies |
| SelectPositions | NeedsBuilder / structure | state.positions — S/structure_ops/list_lookup.rs:21,58,80 | position output vector; iterative and resumable | none verified |
| SequenceFold | NeedsBuilder / histories | history / call_args — S/functional_ops/sequence/fold/sequence_fold.rs:42,49,85 | history and callback argument vectors; iterative and resumable | none verified |
| SequenceFoldList | NeedsBuilder / histories | history / call_args — S/functional_ops/sequence/fold/sequence_fold.rs:42,49,85 | history and callback argument vectors; iterative and resumable | none verified |
| Set | NeedsBuilder / state | call.prepared / unevaluated_restores — R/definitions/definition_lhs_preparation.rs:526,551,593 | prepared argument vector and traversal state; iterative and resumable | Definitions/MessageDefinitions stores maintained; mixed structural LHS preparation |
| SetAttributes | NeedsBuilder / state | targets — R/attributes.rs:74,82,123 | symbol work vector; iterative and resumable | Attributes updated :140; vector survives attribute evaluation |
| SetDelayed | NeedsBuilder / state | call.prepared / unevaluated_restores — R/definitions/definition_lhs_preparation.rs:526,551,593 | prepared argument vector and traversal state; iterative and resumable | Definitions/MessageDefinitions stores maintained; mixed structural LHS preparation |
| SetOptions | NeedsBuilder / state | targets; current; results — R/options/builtins.rs:38; R/options/builtins.rs:178; R/options/builtins.rs:203 | target/options/output vectors; recursive/resumable | SymbolOptions owns installed result only |
| Short | NeedsBuilder / diagnostics | rendered / prefix / suffix — R/debug_tools/basic.rs:145,146,153 | strings and char vector; iterative formatting | none verified |
| SortBy | NeedsBuilder / structure | decorated — S/structure_ops/ordering_sets.rs:730,764,836,870 | retained sort keys; iterative and resumable | none verified |
| Sow | NeedsBuilder / diagnostics | matched — R/debug_tools/reap.rs:32,40 | match-position vector; iterative; matcher can suspend | ReapScopes incoming value/tag charged :52/:54 |
| Stack | NeedsBuilder / diagnostics | items — R/debug_tools/basic.rs:101,106 | held-stack output vector; iterative | EvaluationStack owns source only |
| StringApply | NeedsBuilder / strings | outputs; joined; converted — S/strings.rs:202; S/strings.rs:369; S/strings.rs:380 | callback-output vector and recursive string joins; iterative/recursive/resumable | none for the cited local payload |
| StringCases | NeedsBuilder / strings | outputs; matches — S/strings/patterns/search.rs:353; S/strings/patterns/search/rules/cases.rs:425 | match and per-input result vectors; iterative/resumable | none for the cited local payload |
| StringReplace | NeedsBuilder / strings | output; rendered — S/strings/patterns/search/rules.rs:447; S/strings/patterns/search/guarded/replace.rs:523 | growing string plus list-of-string output; iterative/recursive/resumable | none for the cited local payload |
| SubValues | NeedsBuilder / state | items — R/definitions/definition_catalog/entries.rs:147 | definition-rule output vector; iterative | Definitions owns source only |
| SubsetMap | NeedsBuilder / functional | paths / prefix_index / replacements — S/functional_ops/targeted_map.rs:414,585,643 | path trie and replacement vectors; iterative and resumable | none verified |
| SubtractFrom | NeedsBuilder / state | paths / replacements / updated_target — R/definitions/definition_updates.rs:339,556,557 | Part path and replacement work vectors; iterative and resumable | Definitions cover symbol stores; all operations admit Part targets at :252; AppendTo fast path only one branch |
| Sum | NeedsBuilder / table | collected — R/kernel_data/control_data/branching_loops/iteration_aggregate.rs:132,154 | term vector / persistent accumulator; iterative and resumable | arithmetic reduction runs only after collection; iteration-count bound is not memory accounting |
| SyntaxInformation | Unsettled / state | stored clone or fixed rules template — R/namespace_tools/syntax.rs:24; R/namespace_tools/syntax.rs:122 | see unsettled investigation; see unsettled investigation | stored read is not owner mutation; fallback builds fixed templates; full branch projection/callee proof missing |
| Table | NeedsBuilder / table | collected — R/kernel_data/control_data/branching_loops/iteration/driver.rs:269,291 | output vector / persistent accumulator; iterative and resumable | spec.rs:338 projects list shell only; suspended frames are S |
| TagSet | NeedsBuilder / state | call.prepared / unevaluated_restores — R/definitions/definition_lhs_preparation.rs:526,551,593 | prepared argument vector and traversal state; iterative and resumable | Definitions/MessageDefinitions stores maintained; mixed structural LHS preparation |
| TagSetDelayed | NeedsBuilder / state | call.prepared / unevaluated_restores — R/definitions/definition_lhs_preparation.rs:526,551,593 | prepared argument vector and traversal state; iterative and resumable | Definitions/MessageDefinitions stores maintained; mixed structural LHS preparation |
| TakeLargestBy | NeedsBuilder / structure | decorated — S/structure_ops/collection_stats.rs:745,769 | retained callback keys and selection output; iterative and resumable | none verified |
| TakeList | NeedsBuilder / structure | counts / chunks — S/structure_ops/sequence/take_list.rs:23,73,80 | chunk output vector; iterative | none verified |
| TakeSmallestBy | NeedsBuilder / structure | decorated — S/structure_ops/collection_stats.rs:745,769 | retained callback keys and selection output; iterative and resumable | none verified |
| TakeWhile | NeedsBuilder / functional | sequence / render_prefix — S/functional_ops/sequence/window.rs:479,482,546 | copied input plus selected output vector; iterative and resumable | none verified |
| Tally | NeedsBuilder / structure | counts — S/structure_ops/collection_stats.rs:115,121,142 | count buckets; iterative with resumable predicate branch | none verified |
| TestReport | NeedsBuilder / diagnostics | rendered_results; merged_results — R/debug_tools/test_report/report.rs:350; R/debug_tools/test_report/report.rs:536 | result history/output vectors; recursive/resumable | none for the cited local payload |
| Through | NeedsBuilder / functional | transformed_args / per-head call arguments — S/functional_ops/callable/head_ops.rs:71,72,79 | output vector/tree; recursive | session.evaluate at :15 happens after local expansion |
| TimesBy | NeedsBuilder / state | paths / replacements / updated_target — R/definitions/definition_updates.rs:339,556,557 | Part path and replacement work vectors; iterative and resumable | Definitions cover symbol stores; all operations admit Part targets at :252; AppendTo fast path only one branch |
| ToExpression | NeedsBuilder / parser | syntax; lowered; parser items — S/strings/text/builtins.rs:460; crates/rum-syntax/src/parser/helpers.rs:66 | syntax tree then lowered expression construction; recursive parser/lowering; evaluation resumable | S/W only see later evaluation; AST inclusion needs explicit R shape specification |
| Together | NeedsBuilder / algebra | terms; coefficients; term_exprs — S/polynomials/core.rs:123; S/polynomials/core/construction.rs:62; S/polynomials/core.rs:410 | polynomial intermediates and output vectors/maps; recursive/iterative | none for the cited local payload |
| Trace | NeedsBuilder / diagnostics | events / depths / trace builder — R/debug_tools/trace.rs:161,198,199 | event copies and rendering output; capture/filter resumable; rendering builder | TraceEvents/TraceEventDepths maintained during capture; local copies survive truncate :203 |
| TraceDetail | NeedsBuilder / diagnostics | events / depths / trace builder — R/debug_tools/trace.rs:161,198,199 | event copies and rendering output; capture/filter resumable; rendering builder | TraceEvents/TraceEventDepths maintained during capture; local copies survive truncate :203 |
| TracePrint | NeedsBuilder / diagnostics | events / depths / trace builder — R/debug_tools/trace.rs:161,198,199 | event copies and rendering output; capture/filter resumable; rendering builder | TraceEvents/TraceEventDepths maintained during capture; local copies survive truncate :203 |
| Unique | NeedsBuilder / state | symbols / generated / candidates — R/scoping/unique.rs:103,104,128 | output symbol vectors; recursive list processing and iterative batch | Definitions/Attributes registration covers stored metadata; interner/gensym quota explicitly excluded |
| UpSet | NeedsBuilder / state | call.prepared / unevaluated_restores — R/definitions/definition_lhs_preparation.rs:526,551,593 | prepared argument vector and traversal state; iterative and resumable | Definitions/MessageDefinitions stores maintained; mixed structural LHS preparation |
| UpSetDelayed | NeedsBuilder / state | call.prepared / unevaluated_restores — R/definitions/definition_lhs_preparation.rs:526,551,593 | prepared argument vector and traversal state; iterative and resumable | Definitions/MessageDefinitions stores maintained; mixed structural LHS preparation |
| UpValues | NeedsBuilder / state | items — R/definitions/definition_catalog/entries.rs:136 | definition-rule output vector; iterative | Definitions owns source only |
| Values | NeedsBuilder / associations | values / work / completed — S/associations/basic/values.rs:74,109,110 | output and work vectors; bounded recursion plus iterative tail | none verified |

#### Conversion families

Each group lists all of its NeedsBuilder declarations exactly once. The evidence-files column in `families.tsv` gives the actual implementation files; the row table above provides their line anchors.

| Family | Count | Declarations |
| --- | ---: | --- |
| algebra | 5 | CoefficientList, D, Expand, Factor, Together |
| associations | 25 | AssociateTo, Association, AssociationKeyDeflatten, AssociationKeyFlatten, AssociationMap, AssociationThread, Counts, CountsBy, GroupBy, JoinAcross, KeyDropFrom, KeyGroupBy, KeyIntersection, KeyMap, KeySelect, KeySortBy, KeyUnion, KeyValueMap, Keys, Lookup, Merge, MergeNested, Normal, PositionIndex, Values |
| diagnostics | 19 | Echo, EvaluationData, ExceptionTypes, Message, MessageDefinitions, MessageTags, Messages, Off, On, Print, Reap, RumProfileSnapshot, Short, Sow, Stack, TestReport, Trace, TraceDetail, TracePrint |
| functional | 21 | Accumulate, Apply, Array, ArrayFilter, ArrayReduce, BlockMap, Comap, ComapApply, Dot, Inner, Map, MapAll, MapApply, MapAt, MapIndexed, MapThread, Outer, Select, SubsetMap, TakeWhile, Through |
| histories | 11 | FixedPoint, FixedPointList, Fold, FoldList, FoldPair, FoldWhile, NestList, NestWhile, NestWhileList, SequenceFold, SequenceFoldList |
| lazy | 1 | LazyTuples |
| namespace | 9 | $ContextPath, $MessageList, $Packages, $Path, Contexts, DumpSave, FileNames, Names, Save |
| parser | 1 | ToExpression |
| rewrite | 10 | Cases, Level, Position, Replace, ReplaceAll, ReplaceAllUnheld, ReplaceAt, ReplaceList, ReplacePart, ReplaceRepeated |
| schema | 2 | ConformSchema, SchemaErrors |
| state | 34 | AddTo, AppendTo, ApplyTo, ArgumentsOptions, Decrement, DefaultValues, DefinePackage, Definition, DivideBy, DownValues, Increment, Information, Once, OptionValue, Options, OwnValues, PackageScope, PreDecrement, PreIncrement, PrependTo, Public, Set, SetAttributes, SetDelayed, SetOptions, SubValues, SubtractFrom, TagSet, TagSetDelayed, TimesBy, Unique, UpSet, UpSetDelayed, UpValues |
| strings | 3 | StringApply, StringCases, StringReplace |
| structure | 18 | DeleteDuplicatesBy, Differences, Extract, GatherBy, Insert, MaxBy, MaximalBy, MinBy, MinimalBy, Part, ReverseSortBy, Riffle, SelectPositions, SortBy, TakeLargestBy, TakeList, TakeSmallestBy, Tally |
| table | 3 | Product, Sum, Table |

**Routes:** 158 of the 162 need an armed ordinary-handler companion through `register_accounting_builtin`. Four (`$ContextPath`, `$MessageList`, `$Packages`, `$Path`) need R operations in the existing symbol-evaluation path; they do not have callable BuiltinFn handlers. This means **zero already sufficient existing R paths**, not four already-instrumented symbols. `evaluate_once_in_accounting_mode<M>` exists at `R/eval_core/mod.rs:28`, but `evaluate_symbol` currently loses that typed mode (`R/eval_core/symbol.rs:10`). The three unsettled rows are excluded from these route counts.

#### What A–E changed, and what it did not

`ae-delta.tsv` lists all 165 old records as **Pending final disposition**, alongside this census. There is no historical final-class transition count to calculate. In particular, “before NeedsBuilder, after CoveredByOwner” would misrepresent both the earlier report and the current code. Brief-516 already identified the Table/history/group/string/diagnostic builder gaps in §3.3 (`:619–629`); those examples remain NeedsBuilder.

These concrete attribution deltas are visible in the current tree:

| Names / portion | Landed A–E evidence | Remaining local portion / current disposition |
| --- | --- | --- |
| Print, Echo — buffered output | Prints append charges payload/capacity at `R/debug_tools/resources/maintained/diagnostics.rs:49–64`; split releases at `:68–78`; abandonment restores at `R/suspension/core.rs:373–375` | Print's fragment Vec/join at `R/builtins_input.rs:105–108`; Echo's formatting at `R/debug_tools/basic.rs:37–41`. NeedsBuilder, including sink paths which need a live string before delivery. |
| Message — stored diagnostic | Messages append at `maintained/diagnostics.rs:9–24`; split release `:28–44`; abandon truncation `R/suspension/core.rs:370–372` | Format arguments/string before `push_message`, including delayed-template resumption: `R/messages/message_catalog.rs:154`, `R/messages/builtins_messages.rs:22,160`. NeedsBuilder. |
| Trace, TraceDetail, TracePrint — captured events/depths | TraceEvents/TraceEventDepths are maintained; depth append at `maintained/diagnostics.rs:125–135`; error/abandon truncation refreshes both at `R/suspension/core.rs:377–384` | `events`/`depths` copies at `R/debug_tools/trace.rs:198–199`, then structured rendering `:163–184`; the local copies remain after source truncation. NeedsBuilder. |
| Sow, Reap — stored scopes/buckets | `R/debug_tools/reap.rs:51–55` charges sown value/tag occurrences; `:69` installs an accounted scope; scoped removal/abandon uses the maintained scope APIs | Sow's `matched` position vector `:32–40`; Reap's removed/rendered groups `:94,135,157`. NeedsBuilder. |
| Set/SetDelayed/TagSet/TagSetDelayed/UpSet/UpSetDelayed; update operators | Definitions and associated lookup-owner totals are now maintained by the landed mutation paths | LHS `call.prepared`/`unevaluated_restores` and update `paths`/`replacements` precede owner publication; the exact sites are in the census. NeedsBuilder. This is partial-owner attribution, not an every-exit owner proof. |
| Special variables and metadata/value readers | Their source fields are among the maintained 53 owners | Symbol collect/rendered-rule vectors are additional occurrences, not source-owner mutations. NeedsBuilder except SyntaxInformation's unresolved fixed-template/read branch. |
| RegisterExceptionType | Explicit `ExceptionTypes` refresh after insertion at `R/kernel_data/control_data/exceptions.rs:79–83` | This improves the registry portion; local cloning, conflict formatting, and returned registration/invalid forms still need a whole-branch disposition. Unsettled. |

None of these partial improvements discharges a whole row. In particular, I have not inferred error/suspend/abandon coverage from an owner name. Those every-exit proofs are required before any future row can become CoveredByOwner.

#### Incidental findings outside this line — report only

These are source findings, not newly executed ceiling falsifiers. **None is included in a conversion brief below.** Shared helpers must preserve these routes until separately authorized work.

| Declaration today | Concrete local storage / projection gap |
| --- | --- |
| DeleteDuplicates — Preflight | `S/structure_ops/dedup.rs:82,107` projects the output call; `:414–417` retains a separate seen HashSet and kept vector. Output projection does not cover both live structures. |
| Duplicates — Preflight | `S/structure_ops/list_lookup.rs:104–116,131–135`: seen/duplicates, then seen/emitted hash sets; no precheck on this construction path. |
| Catenate — NonAmplifying | `S/associations/aggregate.rs:359–373`: `flattened` builds while values remain available; the resumable family retains additional state. |
| ClearAttributes — NonAmplifying | `R/attributes.rs:51–52,74–82`: normalized targets survive attribute-expression evaluation. |
| Protect, Unprotect — NonAmplifying | `R/attributes.rs:55–60,152,171–173`: target collection and rendered-name output; no whole-operation precheck identified. |
| Coefficient — NonAmplifying | `S/polynomials/mod.rs:49` enters the polynomial construction used by Expand; `S/polynomials/core.rs:123–127` can build cross-product terms before extracting one coefficient. |
| ConstantArray, PadLeft, PadRight — Preflight | `S/structure_ops/list_plumbing.rs:50,127–129,173–185` projects generated Expr nodes, not repeated arbitrary child payloads; padding additionally constructs `result` at `:246,303`. This is a shell-only projection. |
| SyntaxQ — NonAmplifying | `S/strings/text/builtins.rs:498` parses an AST then returns a boolean; `crates/rum-syntax/src/parser/helpers.rs:66,83` builds syntax-item vectors. This is also subject to the explicit AST-model boundary discussed below. |

The already-recorded local MemoryConstrained `General::memlim` documentation conflict remains separate (`TODO.md:1541–1547`; `R/debug_tools/resources/constraints.rs:257–279`). No draft here changes its message behavior or weakens the guide.

### 4. Proposed R protocol on the landed seam

This is a design, not a claim that a working builder API exists. `AccountingContext` currently exposes evaluate, shadow observations, one mutation example and CompoundExpression. Its `AccountingShadow::Counter` records events; those counters are **not byte reservations** (`R/accounting_mode/counter.rs:5–32`). Only CompoundExpression is registered through the companion route (`S/lib.rs:74–82`). There is no existing generic builder receipt to reuse.

#### Quantities and admission

Keep today's expression occurrence weight **W** and the independent session weight **S**. Add **R**, a separate live-builder/reservation quantity, only to admission:

- Ordinary armed expression checkpoint: checked/conservatively invalidating `S + W(current) + R`.
- Projected allocation: `S + R_other + proposed_new_weight_of_this_builder + W_other`, with an explicit handoff for an occurrence moving between W and that builder.
- Existing projected-byte calls must include live R as well. Their current implementation checks only `S + bytes` (`R/debug_tools/resources/constraints.rs:175–193`).
- `MemoryInUse[]`, `MaxMemoryUsed[]` and `ByteCount` retain their existing meanings. Do not feed R into the S peak or into either S oracle. A separate diagnostic snapshot may expose R/live reservations, clearly labelled as a different quantity.
- Keep every existing armed checkpoint, the W walk there, and the independent S walker. In production A–E has already replaced the session scan at a checkpoint with a maintained total; this line neither restores repeated S scans nor removes the test/observation walker.

An explicit transfer changes the bookkeeping location of **one modeled occurrence**. It never cancels two real occurrences just because they share an Arc. For example, a history entry, the same subtree in a seen set, and a callback argument can all coexist. Count each according to the fixed occurrence model. No allocation-identity table, subtree deduplication, interner change, or Expr layout change is proposed.

R needs a documented structural formula of its own, not a claim of RSS accuracy:

| Shape | Proposed incremental contribution |
| --- | --- |
| Expr vector / deque | Allocated slots plus child occurrence payloads; avoid charging the same inline Expr slot twice. Charge each new child using the existing W formula. Release a removed child's cached contribution. |
| Strings / rendered chunks | String/sequence shell and capacities plus retained chunk storage. Reserve known incoming bytes before extend; reconcile observed capacity afterwards. Include temporary rendered fragments which survive another allocation/callback. |
| Bucket / seen / lookup map | Explicit shell/capacity formula and every key/value occurrence. A map key that duplicates a bucket value remains another occurrence. Define BTree/map structural constants without claiming inaccessible allocator internals. |
| Work lists / plans / polynomial intermediates | Typed sum of stored work variants, paths, rules, numeric payloads and capacity. Account entry-based insertion and borrowed mutation, not just push. |
| Frozen builder | Existing S-traced expression/symbol portion plus a separately retained R complement for bytes S intentionally omits. See suspension below. |
| Accountant | All introduced inline/heap bookkeeping, capacities, control blocks and retained handles have an explicit admission contribution. No silently exempt registry or token allocation. |

Each shape needs independent checked arithmetic; saturation/overflow is a sticky invalid state, never repaired by `saturating_sub`. A release after overflow cannot make the ledger look small again. Rebuild at a justified cold boundary or reject admission until reset. Synthetic arithmetic tests exercise this without allocating impossible-sized values.

#### Owned leases, operations, and callbacks

Use a safe, generation-bound **linear builder lease**, not a guard borrowing `&mut Session`. An armed lease owns access to a request ledger and a stable builder ID. The payload's container owner and lease move together. The lease is not freely Clone; copying payload into a second owner uses an explicit accountable clone/import that creates another occurrence receipt. No public mutable Vec/String escape hatch may bypass the accounted operations.

A mode-specialized container uses ordinary Vec/String/map storage under Unarmed and an armed storage/lease implementation under Armed. The concrete armed prototype can use `Arc<Mutex<...>>` for its own movable payload/control state, with callbacks always outside locks; all such allocation/locking is absent from the Unarmed monomorph. This is a prototype choice whose armed overhead must be measured, not a reason to add Mutex to the default path. A lighter owned-payload/saved-weight implementation is preferable where it can still supply safe independent observation. Neither design holds a Session borrow across guest evaluation.

Required operations:

1. **Register/import:** establish the request identity, builder ID, shape and existing payload weight before it is retained across another step. Importing an arbitrary tree may walk it once.
2. **Reserve:** check only the proposed delta/replacement reservation, including all sibling and outer builders. Reserve shell/capacity and any exactly known incoming payload before allocation. Never add the entire accumulated prefix again.
3. **Commit/charge:** before retaining an unknown callback result or starting the next iteration, measure its actual occurrence, reconcile actual capacity, and commit the reservation. If it fails admission, discard the candidate and unwind/return through the existing error contract. The callback itself might allocate a large value before returning; do not promise to precheck an unknown result.
4. **Replace/pop/clear/shrink/release:** remove the saved contribution of what actually leaves, update capacity if it changed, and keep arithmetic invalidity sticky. Charge-before-release is conservative where both old and new payloads really coexist; a move can use an atomic ownership handoff.
5. **Publish/transfer:** move a receipt to another accounted container, an S owner, a frozen frame, or the outgoing W result. A transfer guard leaves the source charged until the destination is established. If destination installation fails/unwinds, ownership determines which receipt survives; there must be no uncharged gap.
6. **Drop:** release exactly the outstanding lease once; do not evaluate guest code or unwind again from Drop. Poisoned/partly updated bookkeeping becomes invalid and fails closed.

The production ledger can keep totals, per-builder saved weights, reservations and ownership state. A test verifier additionally needs access to actual payloads. Safe weak payload handles allow an independent test walker to borrow containers without cloning Expr roots or changing their COW behavior. Do not compare R with a second sum of the very same saved weights and call that independent. At a quiescent check, snapshot registry handles under its lock, release that lock, then inspect payloads; no guest callback runs under either lock. Containers compare their actual shape weight at every mutation boundary; the request oracle compares all live entries and frozen complements. Fault injection changes the candidate update only, never the payload walker.

For nested builders, an outer lease remains registered while its callback enters a second companion handler. They share the request ledger but have distinct IDs. For sibling buckets, transfer a value into the chosen bucket and charge any retained lookup key separately. For callback returns, adopt W into R before the next call. The old parent payload remains charged if the callback also receives a clone of it. A scalar “at least one charge happened in this invocation” proves none of these properties.

#### Request and limit lifetimes

Bind identity to the **accepted request's control identity plus generation**, not an expression pointer, builtin name, or `SuspensionGeneration`. The existing request reservation handles wraparound by replacing control identity (`R/session/session_api.rs:157–175`). `SuspensionGeneration` identifies continuation ownership (`R/suspension/core.rs:11–26`); it is not the same generation and cannot qualify builder leases by itself.

Raw Session entry points without an accepted host ticket need a distinct cold evaluation-epoch token with the same lifetime rules. A standalone `accounting_context()` used by an embedding host also needs a defined root scope; it cannot silently borrow the most recent completed request's ledger. The prototype must name and test these entry points, including recursive guest evaluation and host mutation while a request is suspended.

A ledger belongs to the request/evaluation lifetime, **not to one MemoryConstrained interval**. Tightening/restoring a nested limit changes admission policy, not builder ownership. Returning to no active limit must not destroy a still-live lease, and a later arm must see every eligible live builder. Completing a request invalidates old tokens; a late drop may release only its original ledger, never a reused generation. Cancellation, reset, poison recovery and raw Session catch_unwind must finish or invalidate the appropriate ledger before reuse.

| Exit | Required ownership behavior and check |
| --- | --- |
| Normal Expr return | Construct/check the outgoing W while its source is still owned; transfer its occurrence out of R exactly once. Return only after admission. An unchanged-input branch may have zero growth. |
| Publication to Session | Existing owner mutation charges S; retire only the corresponding R occurrence at the same boundary. Extra live clones and work buffers remain R. Do not add another S charge on top of A–E's charge. |
| Early return / invalid arguments | Release every local lease actually created; preserve diagnostics and returned held form. Zero-growth branches must be explicitly recorded rather than forced to make dummy charges. |
| EvalError, throw, timeout, interrupt | Same owned-drop rule; pending suspension is the explicit exception, not an error path that drops its live payload charge. Verify immediately at the boundary, even when the dispatch validator skips Err today. |
| Panic unwind | Safe Drop releases owned locals; partial bookkeeping is invalid. Host poison/reset rules remain; a caught raw Session panic must not reuse a falsely valid ledger. Drop itself does not panic. |
| Suspension | Move/freeze each builder exactly once; expression roots become visible through the frame's existing S owner, while the non-S complement stays charged by the frame-owned R lease. |
| Resume | Validate request identity; transfer or accountably copy into an active builder before continuing. Release the frozen complement only when its actual occurrence goes away. |
| Abandonment | Pending/resuming/worklist ownership changes do not drop charge early. Release at actual frame destruction; account nested cleanup that suspends again. Repeated abandon cannot double-release. |
| Session drop/reset / completed request | Retire the ledger only after live owned payloads are disposed/transferred. Stale handles cannot access the next request. Assert no unexplained active-builder remainder; legitimate retained session roots stay S. |

#### Suspension is a partition, not “release all R”

The compile boundary `SuspensionFrame: FrameRootTrace + FrameAbandon + Send + Sync + 'static` is real (`R/suspension/core.rs:76`). Use derived frame traversal and typed builder/frozen wrappers. A new field whose type lacks FrameRootTrace must fail compilation. A handwritten no-op implementation can still lie; derive plus field/payload tests are needed as well.

The existing visitor only visits Expr and Symbol (`R/suspension/trace.rs:12–20`); String tracing is explicitly empty (`:355–356`), and Vec tracing walks elements, not spare slots (`:70–75`). Therefore freezing a growing String and retiring its entire R charge would reopen the hole. **Keep S numerically separate:** add a typed frozen-complement admission lease, not String/capacity traversal to the S oracle. Its shape formula partitions the exact terms S already observes from the additional R terms; do not use an unproved numeric `max(B-S,0)` to hide mismatched definitions.

Pending insertion currently refreshes PendingSuspension (`R/suspension/core.rs:508–523`), resumption moves it to ResumingSuspension (`:545–572`), and abandonment moves through AbandonmentWorklist (`:605–659`). Each transfer adapter must preserve the new complement. On resume the driver calls through `Arc<Self>` while the current frame is still Session-rooted, then removes/drops it (`:703–708`). A cloned working payload can therefore coexist with the frozen occurrence. Charge both while both are live; “resume happened” is not permission to release the frame's roots. Prefer moves when the existing immutable-frame protocol permits them, without changing continuation semantics to obtain an accounting shortcut.

#### Mechanical conversion recipe

For Table, as a concrete example:

1. Keep the public catalog spec and legacy BuiltinFn behavior. Register its explicit armed companion during `install_core`, following the CompoundExpression pattern; give the companion a real declaration ID/disposition. Do not classify it merely because the name appears in a companion list.
2. Have the companion accept `&mut AccountingContext<'_>` and enter a mode-specialized kernel driver. Extend the narrow kernel API for the needed operation. The current context has no general Session forwarding surface; explicit context methods or a scoped reborrow used only while the owned mode/lease remains valid are necessary. Do not pretend all old `session.*` calls compile unchanged.
3. Replace `collected` creation/reserve/push/freeze/finalization in `iteration/driver.rs` with the accounted operations, including plain count, named iterator, nested iteration and persistent resumed paths. Charge incoming actual children before the next evaluation. Keep the existing exact list-shell precheck in `iteration/spec.rs:338`; do not count its reservation a second time.
4. Preserve the existing error/throw/suspend routing and local-symbol restoration. Move leases with named continuation state; verify every early exit and resume branch.
5. Keep the unarmed instantiation's container, calls, layout, and Drop behavior equivalent to the baseline. Inspect emitted loop/Drop code and measure instruction slopes and timings. Source-level zero-sized mode types alone do not establish erasure.
6. Add branch receipts, independent payload checks, a real low-limit progress falsifier, and the declaration's closure record. A final-result-only rejection does not pass.

Apply this to 158 companions and the four symbol cases. Existing owner APIs remain the S destination for mixed stateful rows; they do not replace their local-builder conversion. Closed Preflight/Delegated alternatives require their own actual per-builtin falsifier/callee proofs before changing a declaration.

#### Two protocol feasibility gaps that this survey cannot honestly erase

**Arming inside an initially unarmed builder.** The current generic mode is selected at entry, whereas an arbitrary callback may later enter MemoryConstrained. An outer unarmed Table may already hold many collected values that are absent from S/W. A zero-sized no-op lease has recorded none of them. Merely reselecting the mode inside `Session::evaluate` cannot discover safe-Rust stack locals.

P1 must prototype an explicit callback activation bridge that can expose the outer builder payload at the existing evaluation boundary and import it when a nested limit arms. A borrowed payload descriptor/callback boundary is one candidate; a continuation boundary that exposes the payload before guest execution is another. Both must be walked and measured for unarmed code/stack/loop cost. Neither may be declared free by inspection. A test must accumulate a large prefix **before** arming in a later callback and then distinguish limits with/without that retained prefix. Restricting R to builders which happened to start armed would change coverage and is not adopted here.

**Bookkeeping placement versus unchanged S.** `AccountingRoute::memory_bytes` already counts the route box, adapter capacity and `size_of::<Counter>()` (`R/accounting_mode/mod.rs:46–63`); the independent formula repeats those exact terms (`R/debug_tools/resources/occurrence/schema.rs:52–58`). Adding a field to Counter/AccountingRoute, expanding registered adapter storage, or enlarging a stored frame can change an S reading even if no one writes “S += R”.

The constraint remains: R and its extension-only observation must not be smuggled into MemoryInUse/MaxMemoryUsed/ByteCount, and the accountant must not be free. P1 must show a concrete placement for admission-only ledger storage and explicit callback/frame handles, with all bytes attributed and unchanged S fixtures for isolated R activity. There is **no demonstrated placement proof in this design-only survey**. If “numerically unchanged” means bit-for-bit equality with the pre-line layout even after installing 158 extra adapter entries, direct repeated use of today's registration vector cannot meet that reading. This is a concrete director clarification about metadata/layout attribution and allowable storage placement, not a proposal to add R to S or weaken the separate oracle. Until settled, the drafts are implementation research drafts, not a claim of a dispatch-ready complete protocol.

### 5. Enforcement and watched falsifiers

#### What the empty arm can check

Keep ordinary production checks in the armed builder operations, transfer operations and `S + W + R` admission seam. Cost is weighing each newly retained occurrence/capacity delta plus armed ledger arithmetic; no repeated walk of an unchanged accumulated prefix. Production needs generation validation and fail-closed arithmetic. A legacy host builtin can still allocate arbitrary private Rust data invisible to the kernel: companion registration is the accountable extension API, and this limitation must be explicit. Do not reject or silently “cover” all existing public BuiltinSpec registrations by changing their API.

Use the existing test/diagnostic dispatch gate for **invocation receipts**, not a pretend Rust-local scanner. Move Incremental validation ahead of today's `let Ok(result) = result else { return; }` for its error/suspend receipts. It can check:

- a current installed Incremental name has a typed disposition and the selected adapter/path matches it;
- the invocation's owned builder IDs finish, transfer, or remain correctly frame-owned on every exit;
- every declared branch receipt is observed by its required tests, and no pending reservation escapes;
- independent R payload equality at mutation/callback/transfer boundaries, with independent S equality remaining separate;
- nested invocation receipts are attributed to their own IDs, rather than a child charge falsely satisfying the parent.

It **cannot** discover an arbitrary bare Vec/String local that was never registered; an empty receipt or one charge is not an implementation proof. Return-size comparisons cannot see a huge local work list that was freed before return. A source audit plus typed container boundaries and declaration-specific behavioral falsifiers is still necessary. The test walker may be expensive and is compiled out of ordinary release; optional diagnostics use the existing cold gate, without a new per-push disabled flag load. The production checks remain active under a limit when diagnostics are off.

#### Closed declaration census

Extend the existing inventory test's actual installed/catalog set comparison, not a second disconnected constant of 165 names:

1. Introduce a sorted unique disposition registry keyed by the same declared builtin identity. Each entry carries a route kind (ordinary companion / symbol path / owner / exact preflight / accountable delegate), owning implementation and a named witness. During staged work, pending rows are explicitly allowed only in the stage's decreasing, reviewed worklist; the final gate permits **none**.
2. In the required stdlib suite, compute the actual set of installed Incremental specs with install_core and independently the raw runtime/stdlib catalog union, then compare each to the registry key set. This catches a new name, a contract changed to Incremental, an omitted or stale disposition, catalog shadowing, and duplicates. Run with default features and `--no-default-features`; neither cfg combination may hide a name.
3. Check the four symbol declarations separately against the explicit symbol route registry. “No handler” is expected for them and must not exempt them.
4. Verify armed companions are actually registered for current standard-library handler identities. A later public register_builtin override removes the companion today; test and document that legacy extension boundary. A name-based static assertion must not falsely cover an overridden host handler.
5. Make every closed row name a callable witness function/typed descriptor, so removing a witness breaks compilation. A required exhaustive test supplies the declaration-set barrier. Rust string catalogs do not have the Session struct's automatic exhaustiveness property; do not claim an equivalent compile barrier without a shared generated macro.
6. Supplement, within each converted module, with a branch inventory for fast/slow, scalar/list, valid/invalid, callback/no-callback and resume paths. Do not use a brittle global “no Vec::push” grep as semantic proof; unrelated, bounded and non-Incremental helpers legitimately share files.

The current S exhaustive struct destructuring stays exactly as a separate barrier. The census registry runs in required tests; it adds no default runtime lookup per builder operation. Production selects the armed companion only through the existing armed branch.

#### Proof that the checks fire

Each implementation step must preserve raw bad-then-good output from the **same** test/workload. No falsifier was executed in this brief.

| Check | Deliberate fault and required red observation | Green observation after removing fault |
| --- | --- | --- |
| Independent payload equality | Omit a real Table/history/bucket/string entry charge but still mutate its payload | Exact builder ID/term discrepancy disappears; repeat same mutation sequence |
| Spare capacity | Skip a capacity increase without changing length | Shape oracle disagrees at that boundary; then agrees with actual capacity |
| Release | Skip a real pop/drop/split release; repeat complete/reclaim cycles | Positive residue detected immediately, not just eventual low-limit failure; matched cycles return to the correct live state |
| Admission | Disable one real progress admission check while leaving the oracle disabled, choose a limit between successive retained prefixes | Production-mode guest reaches a forbidden next callback / succeeds when it should terminate; good version rejects before that callback |
| S plus sibling R | Exclude the outer/other builder from a nested reservation | Combined-budget test fails even though each builder alone fits; good version rejects combined retention |
| Unknown callback result | Move charge after the next callback | Callback-count/output marker proves an extra iteration ran; corrected version stops before it |
| Publication / suspension | Omit or double one actual transfer, including frozen String complement | Immediate S/R partition or generation mismatch; no compensating later allocation needed |
| Resume / abandon | Reuse a stale request ID or release a frozen lease twice; exercise cleanup that suspends again | Failure at transfer/teardown; good run is repeatable and fresh request unaffected |
| Overflow / unwind | Permit saturating-sub recovery after synthetic overflow; leave a partially updated ledger valid after panic | Admission/validity assertion goes red; good path fails closed and follows poison/reset behavior |
| Frame compile barrier | Add a real test-frame builder field without its trace implementation, or remove the required derive in an isolated negative build | Compiler rejects SuspensionFrame bound; restored type compiles. A separate payload test catches a lying/no-op traversal |
| Declaration closure | Change a real catalog builtin to Incremental without a registry entry, add a name without a disposition, remove a companion, and remove a witness | Required census/route test fails or witness reference fails compilation; restore and rerun |
| Exact-preflight alternative | On the actual builtin being reclassified, remove its claimed exact projection and drive the growth branch | Required low-budget/receipt test fails; restoring it succeeds. Existing unrelated Preflight tests are not sufficient |
| Separate observation | Deliberately include a live R-only builder in S/peak/ByteCount or omit ledger storage from admission | Paired observation/metadata test fails; good version preserves separate readings and accounts overhead |
| No unarmed operations | Deliberately route a normal unarmed pilot through its armed storage; separately inspect the correct emitted loop/Drop | Allocation/route counter test catches the wrong route; optimized candidate shows no lease/lock/byte-walk in the unarmed loop, backed by measured slopes |

The production admission test must be run with `RUM_TEST_BUILTIN_MEMORY_GROWTH` and profiling disabled, not only in stdlib unit tests (whose dev dependency enables the diagnostic feature). Oracle-red and production-limit-red are distinct evidence. Gate inputs use rum's documented behavior and host APIs; no vendor oracle or generated third-party outputs.


### 6. Cost plan — actual labels, missing labels, and cumulative baseline

No timing or instruction result was produced in this brief. The costs below are a measurement plan. The script has **119 literal labels** (saved with source lines in `microbench-labels.tsv`). It has no formal tier metadata; the neighbor panels below are explicit comparable-path panels to freeze before editing, not invented tier annotations.

| Family / step | Existing exposed target labels | Same-path/tier neighbors to report |
| --- | --- | --- |
| Table / P2 | `table-count-null`, `table-iterator-null`, `table-iterator-symbol`, `table-small`, `table-iterator-plus`, `table-iterator-pair-list` | `loop-empty`, `do-count-null`, `do-iterator-null`, `do-small`, `list-literal-small`, `symbol-ownvalue-read`, `set-ownvalue` |
| Fold/FoldList; Nest/NestList / P3 | **None.** Nest appears in setup and output checks, not as a timed Nest label. No Fold/FoldList labels are exposed. | All Table targets; `downvalue-pattern-hit`, `module-local`, `block-local`, `with-local`, `list-literal-small` |
| Map / P4 | **None.** Add a direct Map label before measuring either arm. | `cases-wide-no-match`, `position-wide-no-match`, `replace-wide-no-match`, `replace-leaf-rebuild-small`, `part-list`, `table-small` |
| GroupBy/GatherBy and association grouping / P4–P5 | **No direct GroupBy/GatherBy labels.** `merge-nested-small` is exposed but is a neighbor, not a grouping measurement. | `merge-nested-small`, `association-small-literal-repeat`, `keysortby-callback-small`, `keysortby-callback-large`, `keysortby-default-wide`, `sortby-callback-small`, `sortby-callback-large`, `sortby-default-wide` |
| StringReplace / P4 | `stringreplace-single-literal-unguarded`, `stringreplace-pattern-unguarded`, `stringreplace-guarded-hit`, `stringreplace-guarded-decline-fallback` | `string-repeat-small`, `character-range-small`, and all six `stringtrim-{literal-list,single-literal,whitespace}-{4000,32000}` labels |
| Rewrite / P5 | `replaceall-simple`, `replace-wide-no-match`, `replace-leaf-rebuild-small`, `replacepart-deep-path-small`, `replacepart-wide-rule-list`, `replacepart-guarded-hit`, `replacepart-guarded-decline-fallback`, `cases-wide-no-match`, `position-wide-no-match` | `matchq-integer-success`, `matchq-integer-fail`, `matcher-single-tiny`, `matcher-single-wide-small`, `matcher-single-wide-large`, `count-wide-no-match`, `deletecases-wide-no-match`, `rule-rhs-deep-small`, `rewrite-guarded-hit`, `rewrite-guarded-decline-fallback` |
| Structure/order/algebra / P5 | `take-multispec-small`, `sortby-callback-small`, `sortby-callback-large`, `sortby-default-wide`, `d-nested-sin-small`; direct polynomial/schema/parser labels are absent | All exposed sort/reverse-sort/key-sort/default/callback width pairs, `part-list`, `association-part-key-read-width4000`, `association-part-key-read-width32000`, and positional-read width counterparts |
| State/diagnostics/namespace / P6 | `set-ownvalue`, both `appendto-list-incremental-{small,large}`, both `appendto-part-outer1000-{small,large}`, both `association-set-key-incremental-{small,large}`, `set-part-fixed100-len100`, `set-part-fixed100-len800`, `set-part-fill-small`, `set-part-fill-large` | `symbol-ownvalue-read`, `symbol-ownvalue-guarded-read`, all exposed downvalue/upvalue/subvalue/default hit/miss/guard labels, block/module pairs, `valueq-undefined-symbol-definitions-2000`, `valueq-undefined-symbol-definitions-8000` |
| Protocol and closure / P1, P7 | Entire exposed suite, plus all added direct family labels | Every label: dispatch, resource checking and continuation plumbing are shared |

P3 must add direct `fold-small`, `foldlist-small`, `nest-small`, `nestlist-small` and size-paired workloads. P4 must add `map-small`, `groupby-small`, `gatherby-small` and size-paired workloads; these names are **proposed, not claimed current script output**. Use existing rum syntax/guide contracts, a fixed list/callback setup outside the timed statement, and exact direct family calls. Keep scalar/result size and callback count pairs separate so fixed startup does not hide loop cost. P5/P6 add direct representative workloads where their owning family is otherwise absent; list the final statements and labels before editing source.

A new measurement label must be executed against the saved step base and the candidate with **identical benchmark-source bytes**. Do not compare a new candidate-only label with a different old neighbor. Preserve every existing label statement. No assignment, Print, or sink is inserted into an existing timed expression to “use” its result.

#### Baselines and accounting for A–E's cost

Brief-523 reports **instruction slopes**, not microbench wall timings: +0.965100% smoke, +3.239163% symbol-ownvalue-read, +1.018234% downvalue-pattern-hit. Its report explicitly says no timing panel was run (`:481–484`). Do not apply these percentages to unrelated timing labels or pretend they are fresh measurements here.

Its reported pre-slice binary is `target/accounting-route-518/rum-base`, copied to `target/brief-523/rum-base`, SHA256 `beda3aa055222aa5ca4db7320684e05848ad2bbad5f247431e84e6d02d103376`; final A–E candidate SHA256 `628473e831aa7aa09f16f2063bf02f4e1e04e52b1a4acc7ef2f9cd40b24f2356` (`brief-523:458–498`). These are **reported provenance**, not freshly verified executable hashes in this survey. Each implementation executor must verify them and the recorded build/execution-path recipe before reuse. An unavailable or nonmatching baseline means “cumulative comparison unavailable” until the correct artifact is supplied/rebuilt; never substitute the incorrect old strided binary.

For **each step**, preserve/build the correct current step base before edits and identify all three comparisons:

1. Candidate versus immediate step base.
2. Candidate versus pre-R-line (landed A–E) base.
3. Candidate versus the preserved **pre-A–E line** base used for the cumulative +0.965/+3.239/+1.018 figures.

Report raw per-label `timings.tsv` rows and instruction curves, not an overall average. For the same metric/label, cumulative ratios multiply: `(1 + AE_delta) * product(1 + step_delta) - 1`; percentages are not simply added. Prefer the direct candidate/pre-line measurement as the reported cumulative value and use chaining as a cross-check.

Retain the three-size unarmed smoke/ownvalue/pattern curves and high-low plus adjacent slopes from brief-523; separately measure changed family loops at two or more sizes. Preserve the four armed unchanged-root controls (empty/list10k × 16/1024 checkpoints), cold arm/rearm, realistic non-aborting builder budgets, and resume/abandon pairs. Correctness-rejection runs are separate from performance runs: early termination is not a speedup.

Run the suite through its bounded runner, using in-kernel AbsoluteTiming/Instant values; no wall-clock differencing and no other CPU-heavy workload. Profiling, builder-oracle and diagnostic-growth flags are off for release timing. Set repo-local output/tmp locations and full environment before both arms. Derive timeout bounds from prior same-shape records, not guesses.

Freeze a finite instruction panel and launch count before each step (three alternating pairs per chosen label/size, plus the required pre-line comparisons). Record the arithmetic `labels × sizes × baselines × repetitions`; do not silently inherit the old 108-launch cap while adding a third baseline and more families. Start with one complete timing A/B per required baseline comparison; a second pair only for a named unresolved concern, never unlimited reruns. Publish failed/incomplete runs and all neighbor regressions.

The traversal-depth token hazard was **2–7%**; a parameter threaded through a hot inner loop can cost instructions even when its methods do nothing. Each pilot must inspect its unarmed loop and Drop code and measure its size slope. Later batches must repeat that proof for their actual wrappers and resumable state. Erasure is a per-step result, not inherited from P1.

A cost bar is evidence, **not a stop condition**. If the first design regresses, report the curve, walk the alternative ownership/callback route, and measure it. Fix or report hot-label/neighbor defects; do not restore stride, weaken the ceiling, average regressions away, or offer an unmeasured “stop here” as the result. A demonstrated correctness/scope conflict still goes to the director; it is not concealed as a performance choice.

### 7. Seven drafted implementation briefs

These drafts follow TEMPLATE's sections and are review material for the director's rewrite, not authorization to execute them in this design task. Their scopes were generated and checked against the **current tree**; `draft-scopes.tsv` contains all expanded repository paths. A listed directory permits Rust submodules for that responsibility, including appropriately split new modules within it. It does not authorize unrelated semantic cleanup.

**Common instructions, incorporated in every draft:** read AGENTS, this report, the immediately preceding accepted report, testing-strategy, code-organization-guidelines and the resource-limit guide. Safe Rust only; no new dependencies, git mutation, network, outside-repo writes, vendor comparisons, Expr/SharedExpr/SharedArgs/Symbol representation changes, interner/gensym-quota work, or snapshot-format changes. Preserve the occurrence model, unstrided armed checks, S oracle and separate observations. Keep BuiltinSpec extension behavior. The eleven incidental NonAmplifying/Preflight declarations reported above are excluded even when a shared file is in scope; specialize the Incremental route without silently repairing their separate contracts.

Each draft may append its assigned report and write scratch under its assigned `target/<brief>/` directory. Only the exact named documents may change, for the memory-coverage contract actually changed. Do not alter the pending local-memlim diagnostic conflict. Required tests use rum's own documented contracts.

All implementation commands which can run long/spawn children use `scripts/run-bounded.py --timeout <bound derived from a recorded same-shape run> -- ...`. The following gate names mean those bounded commands, not permission to run them bare:

- FMT: `cargo fmt --check`.
- RT: `cargo test -p rum-runtime`.
- WS: `cargo test --workspace`.
- NOHASH: `cargo test -p rum-stdlib --no-default-features`.
- CATALOG: `cargo test -p rum-stdlib installed_builtin_memory_growth_inventory_is_sorted_and_exhaustive`, plus the new required Incremental disposition/route test.
- AB: the exact release-binary, full-suite and instruction-curve comparisons in §6, against immediate base, landed A–E base, and pre-A–E base. Preserve raw timings.tsv/perf/completion records and verify no early aborts.

Every step reports actual stdout/stderr/exit status for green and deliberately red tests. A planned test name below is a requirement to implement, not a claim that it already exists. Emit no fault branch in normal unarmed builds. Show unarmed emitted-loop/Drop evidence **for that step** and family-size instruction slopes. Report all target and neighbor labels, bookkeeping layout/capacity, limits, and unresolved coverage; no “all covered” based on one sample. Performance bars do not terminate exploration.

#### P1 — Accounted builder protocol, activation/lifetime prototype, and registry foundation

**Goal.** Establish a safe separate R admission protocol on the existing AccountingContext route, with owned leases, independent payload observation, request identity, frozen complements and a falsifiable unarmed-erasure claim. Resolve the two feasibility questions in §4 with concrete prototypes/evidence before bulk conversion. Establish the exhaustive declaration registry with an explicit current pending set; close no production declaration merely for registering its name.

**Context pointers.** `R/accounting_mode/mod.rs:20–22,70–118,129–194`; `counter.rs:5–32,62–103`; `R/eval_dispatch/mod.rs:205,253,308,366`; `R/session/session_api.rs:150–175`; `R/suspension/core.rs:76,508–523,545,605`; `S/catalog/memory_growth_contract.rs:668`. Read both §4 feasibility gaps and TODO's settled rulings.

**Scope — MAY modify only these current paths:**

- `R/accounting_mode/`
- `R/eval_dispatch/mod.rs`
- `R/eval_core/mod.rs`
- `R/kernel_api.rs`
- `R/metadata.rs`
- `R/debug_tools/resources/constraints.rs`
- `R/session/session_helpers.rs`
- `R/session/session_requests.rs`
- `R/session/session_api.rs`
- `R/suspension/`
- `R/tests/session_memory.rs`
- `R/tests/session_memory_arithmetic.rs`
- `R/tests/memory_growth_contract.rs`
- `S/lib.rs`
- `S/catalog/memory_growth_contract.rs`
- `S/tests/accounting_mode_route.rs`
- `S/tests/debug_tools_resources.rs`
- `H/tests/suspension_abandonment_contract.rs`
- `docs/guide/resource-limits-and-sandbox.md`
- `docs/host-embedding-and-hardening.md`
- `R/tests/mod.rs`
- `S/tests.rs`
- `S/catalog/mod.rs`
- `docs/reference/builtins.md`

Existing S observation/oracle formulas, owner implementation and Expr layout remain read-only. New protocol/test-support modules belong under the scoped accounting_mode directory. Context/kernel_api additions are narrow explicit operations; no unrestricted public dereference escape.

**Deliverable.** Lease operations for reserve/commit/replace/release/publish/freeze/resume; arithmetic invalidity; request-local and raw-Session generation rules; an actual-payload oracle; test/production separation; pending/closed disposition registry and installed-route checks. Prototype scalar/Vec/String and two simultaneous builders. Demonstrate nested arming after an unarmed prefix, bookkeeping placement with separate readings, legacy host fallback, override behavior, and host calls during suspension. If a constraint cannot yet be met, return the precise failing mechanism and measured alternatives; do not silently adopt armed-entry-only coverage.

**rum / Rust pitfalls.** AccountingShadow counts events, not bytes. A Session-free lease must still reconnect to the right request on legacy callback entry. FrameRootTrace omits String capacity. A late drop cannot decrement a fresh generation. A borrowed payload descriptor may cost default-path instructions even when no bytes are charged.

**Verification.** FMT, RT, WS, NOHASH, CATALOG. Add and run protocol tests covering every exit in §4, separate MemoryInUse/MaxMemoryUsed/ByteCount, nested/sibling reserve, unknown callback result, cold rearm, overflow, panic and raw Session reuse. Watch skipped charge, skipped release, wrong generation, frozen-string omission and new-undisposed real declaration faults red then green. Compile a frame without the required traversal and record the rejection. Exercise the admission path in an ordinary release build with diagnostics off. AB targets the full 119-label suite plus the inherited smoke/ownvalue/pattern/armed-root/cold-arm panels. Inspect Unarmed builder creation, callback boundary and Drop; document prototype code-size/layout effects.

**Hard rules.** Common instructions apply. Do not call a registry entry a coverage proof, fold R into S readings, discard live outer builders on limit exit, or publish a “zero cost” conclusion from source generics alone. Do not resolve the storage ambiguity by silently changing an oracle.

**Report format.** Changes by scoped file; complete commands/results; transition table and all watched faults; actual S/R/metadata partition; default/armed curves and remaining pending names; explicit achieved/not-achieved verdict on activation and storage.

#### P2 — Table, Sum and Product pilot

**Goal.** Discharge exactly **Table, Sum, Product** using the accepted P1 protocol, retaining the current iterator/aggregate semantics and exact shell precheck. Close all plain, named, multi-iterator and suspended accumulation paths.

**Context pointers.** Census table family `table`; `R/kernel_data/control_data/branching_loops/iteration/driver.rs:269,291`; `iteration/spec.rs:338`; `iteration_aggregate.rs:132,154`; `S/control_flow.rs:65,74,107`. P1 acceptance evidence, especially nested arm and frame complement.

**Scope — MAY modify only these current paths:**

- `R/accounting_mode/`
- `R/kernel_api.rs`
- `R/kernel_data/control_data/mod.rs`
- `R/kernel_data/control_data/branching_loops/iteration/`
- `R/kernel_data/control_data/branching_loops/iteration_aggregate.rs`
- `S/control_flow.rs`
- `S/catalog/control.rs`
- `S/lib.rs`
- `S/catalog/memory_growth_contract.rs`
- `S/tests/debug_tools_resources.rs`
- `S/tests/holding_and_control.rs`
- `S/tests/session_requests/functional_iteration_resume.rs`
- `scripts/core-language-microbench.sh`
- `R/tests/mod.rs`
- `S/tests.rs`
- `docs/guide/resource-limits-and-sandbox.md`
- `docs/reference/builtins.md`

**Deliverable.** Three armed companions, typed kernel entry/driver plumbing, accounted `collected`/aggregate inputs and iterator scratch, exactly-once persistent continuation transfers, and three closed registry/witness entries. Preserve separate accounting for any evaluated numeric aggregate intermediate. Produce a branch list with every push/freeze/publication/release site.

**rum / Rust pitfalls.** The Table projection covers list slots, not values obtained from the held body. Sum/Product may collect nonnumeric terms instead of reducing a scalar. Count duplicated children by occurrence. A frame clone and active resumed payload may coexist.

**Verification.** FMT, RT, WS, CATALOG. Real low-limit cases with constant-size individually fitting callback returns whose retained prefixes exceed the limit; count callbacks to require failure before the next iteration. Include nested iterators, nonnumeric aggregates, error/throw, Input suspension after a prefix, repeated resume, abandon, nested cleanup, and arming only inside a later body callback. Watch one real `collected.push` charge omitted and one freeze/release fault red then green; remove the current shell projection in an isolated bad variant to show its own falsifier still fires. AB: all six Table target labels and the §6 do/list/symbol/set neighbors, plus smoke/ownvalue/pattern cumulative curves and realistic armed size pairs. Inspect plain and resumed Unarmed loops/Drop separately.

**Hard rules.** Common instructions apply. Do not reduce test result sizes or rely on the final List checkpoint to satisfy progress. Do not transfer a projection reservation and a second full prefix charge for the same occurrence.

**Report format.** Three branch/exit receipts, raw bad/green results, observed stop iteration and threshold calculation, exact per-label step/pre-line rows, remaining pending registry names.

#### P3 — History/sequence pilot and Nest disposition

**Goal.** Close the **11 NeedsBuilder history-family rows**: FixedPoint, FixedPointList, Fold, FoldList, FoldPair, FoldWhile, NestList, NestWhile, NestWhileList, SequenceFold, SequenceFoldList. Separately resolve **Nest** using a complete branch proof; do not invent a Nest history when collect_all is false.

**Context pointers.** `S/functional_ops/iteration.rs:253,334,439,610`; `S/functional_ops/mod.rs:120`; `S/functional_ops/sequence/fold/plain.rs:34,63,139,176`; `pair.rs`, `while_loop.rs`, `sequence_fold.rs`; corresponding census rows.

**Scope — MAY modify only these current paths:**

- `R/accounting_mode/`
- `R/kernel_api.rs`
- `S/functional_ops/mod.rs`
- `S/functional_ops/iteration.rs`
- `S/functional_ops/iteration/`
- `S/functional_ops/sequence/fold/`
- `S/functional_ops/suspended_state.rs`
- `S/catalog/functional.rs`
- `S/lib.rs`
- `S/catalog/memory_growth_contract.rs`
- `S/tests/functional_ops/iteration.rs`
- `S/tests/functional_ops/sequence.rs`
- `S/tests/session_requests/functional_iteration_resume.rs`
- `S/tests/session_requests/functional_sequence_resume.rs`
- `scripts/core-language-microbench.sh`
- `R/tests/mod.rs`
- `S/tests.rs`
- `S/functional_ops/sequence.rs`
- `docs/guide/resource-limits-and-sandbox.md`
- `docs/reference/builtins.md`

**Deliverable.** Accounted histories/seen structures, copied input/item state and callback argument lists, including fixed-point early termination and non-list fold inputs. Transfer all persistent/frozen state and preserve recursion/iteration behavior. Resolve Nest's callback, zero-count, operator form, invalid-argument and resume branches with either actual accountable construction, exact tested projection or a fully named delegation proof. Add direct timed Fold/FoldList/Nest/NestList labels and size pairs; execute identical new source on every baseline.

**rum / Rust pitfalls.** FixedPoint's seen set is distinct from returned history. Fold can retain a copied item sequence even when it returns only the final value. Arity-dependent NestWhile callback arguments may duplicate history occurrences. Do not charge a caller's child callbacks as proof that its own retained history is visible.

**Verification.** FMT, RT, WS, CATALOG; focused functional iteration/sequence and request-resume modules in Scope. Test repeated shared large elements, early stop, cycle detection, unknown-large callback results, callback errors, nested builders, first/middle/final suspension and abandon. Watch an omitted actual history append, omitted seen/item retention, and stale resume transfer red then green. Watch a changed real Nest declaration/claimed branch disposition fail its witness test before accepting any classification change. AB: new direct family labels and all §6 Table/downvalue/module/block/with/list neighbors, plus inherited cumulative curves. Inspect Unarmed normal and resumed monomorphs for each distinct state type; publish per-element cost.

**Hard rules.** Common instructions apply. No reclassification to an empty Delegated arm; no blanket assumption that identity branches prove all branches. Preserve actual FixedPoint/NestWhile stopping semantics.

**Report format.** Eleven closed builder rows plus the resolved-or-still-unsettled Nest proof, every declared test branch, actual red/green transcripts and measured loop/resume curves. An unsettled Nest keeps line closure incomplete.

#### P4 — Map, GroupBy and StringReplace pilot

**Goal.** Close **Map, GroupBy, StringReplace** as three different builder shapes: traversal stack/output, callback-keyed buckets, and growing rendered strings. Validate the protocol beyond history vectors before the remaining family batch.

**Context pointers.** `S/functional_ops/apply_map/traversal.rs:103,107,174` and its recursive/map-indexed submodules; `S/associations/grouping/list.rs:98`, `association.rs:103`; `S/strings/patterns/search.rs:198–269`, `search/rules.rs:447`, `search/guarded/replace.rs:523`. The helper scopes contain other declarations; only these three receive closure credit in this pilot.

**Scope — MAY modify only these current paths:**

- `R/accounting_mode/`
- `R/kernel_api.rs`
- `S/functional_ops/apply_map/`
- `S/associations/grouping.rs`
- `S/associations/grouping/`
- `S/associations/resumable/`
- `S/strings/patterns/search.rs`
- `S/strings/patterns/search/`
- `S/strings/suspended_state.rs`
- `S/catalog/functional.rs`
- `S/catalog/associations_transform.rs`
- `S/catalog/strings_match.rs`
- `S/lib.rs`
- `S/catalog/memory_growth_contract.rs`
- `S/tests/associations/associations_part2.rs`
- `S/tests/strings/search_replace.rs`
- `S/tests/session_requests/transforms_resume_associations.rs`
- `S/tests/session_requests/functional_tensor_string_resume.rs`
- `scripts/core-language-microbench.sh`
- `R/tests/mod.rs`
- `S/tests.rs`
- `S/functional_ops/suspended_state.rs`
- `S/strings/patterns.rs`
- `S/strings.rs`
- `docs/guide/resource-limits-and-sandbox.md`
- `docs/reference/builtins.md`

**Deliverable.** Map covers recursive and iterative/resumable reconstruction plus work lists. GroupBy covers list/association inputs, nested keys/reducers, key/value/bucket/index occurrences and output conversion. StringReplace covers scalar/list input, literal/pattern and guarded/unguarded rules, immediate/delayed rendered results, ordinary/persistent output and rendering temporaries. No callback runs under a builder lock/borrow. Add direct Map/GroupBy/GatherBy timing labels (GatherBy is a measured neighbor, not closed here).

**rum / Rust pitfalls.** Group keys can be larger than the original item. Persistent string chunks and their final flattening can coexist. A no-callback literal branch still builds output. Releasing all R on String suspension loses bytes which FrameRootTrace omits. A recursive fast path and iterative fallback need separate evidence.

**Verification.** FMT, RT, WS, CATALOG; focused named Map/association/string/request modules. Test two simultaneous buckets, repeated shared subtrees, delayed/guard callbacks, invalid patterns/options, empty/no-match, nested builders, error/unwind and repeated suspend/resume/abandon. Watch a Map completed-child charge, a GroupBy entry/bucket charge and StringReplace capacity/append charge separately omitted; each must fail at the affected step, then pass. Include a frozen-string complement omission and a sibling-ledger omission. Ordinary release must reject before the next callback with diagnostics off. AB: new direct labels, all four existing StringReplace labels, and the exact Map/group/string neighbors in §6. Inspect Unarmed traversal, hash-entry and string-extend paths separately; measure small/wide slopes and armed non-aborting budgets.

**Hard rules.** Common instructions apply. Do not optimize by changing sharing, callback order, grouping order, guard semantics or persistence shape merely to reduce counted occurrences. No credit for other family rows without their own branch witnesses.

**Report format.** Three whole-row closure records and shape-specific fault receipts, capacity/frozen-complement formulas, per-label regression curves, and remaining family obligations.

#### P5 — Remaining functional/association/structure/lazy/rewrite/algebra/schema/parser builders

**Goal.** Close the **83 remaining NeedsBuilder rows** in these census groups: functional except Map (20), associations except GroupBy (24), structure (18), lazy (1), rewrite (10), algebra (5), schema (2), parser (1), strings except StringReplace (2). Existing pilot witnesses remain green. No row outside that exact set is silently brought into scope.

**Context pointers.** The 83 row names and accumulator anchors in `families.tsv`/census; accepted P1–P4 protocols; `R/definitions/application/replace_all.rs`; `R/kernel_data/rewrite_data/`, `R/kernel_data/structure_data/`; `S/polynomials/core.rs`; `S/schema.rs`; `S/strings/text/builtins.rs:460–472`; `crates/rum-syntax/src/parser/helpers.rs:66,83`.

**Scope — MAY modify only these current paths:**

- `R/accounting_mode/`
- `R/kernel_api.rs`
- `R/kernel_data/rewrite_data/`
- `R/kernel_data/structure_data/`
- `R/definitions/application/replace_all.rs`
- `R/definitions/application/replace_all/`
- `R/tree_walkers/`
- `R/lowering/`
- `S/functional_ops/`
- `S/associations/`
- `S/structure_ops/`
- `S/lazy.rs`
- `S/strings.rs`
- `S/strings/patterns/`
- `S/strings/text/`
- `S/calculus.rs`
- `S/calculus/`
- `S/polynomials/`
- `S/schema.rs`
- `S/rewrite_builtins.rs`
- `S/lib.rs`
- `S/catalog/`
- `S/tests/functional_ops/`
- `S/tests/associations/`
- `S/tests/strings/`
- `S/tests/schema.rs`
- `S/tests/polynomials.rs`
- `S/tests/traversal.rs`
- `S/tests/preflight_memory.rs`
- `S/tests/session_requests/`
- `R/tests/definitions_and_rules_cases.rs`
- `R/tests/definitions_and_rules_part4.rs`
- `crates/rum-syntax/src/parser/`
- `crates/rum-syntax/src/parser.rs`
- `scripts/core-language-microbench.sh`
- `R/tests/mod.rs`
- `S/tests.rs`
- `crates/rum-syntax/src/lib.rs`
- `R/tree_walkers.rs`
- `R/definitions/definition_catalog/rewrite_rules.rs`
- `R/kernel_data/evaluated_part_selectors.rs`
- `S/strings/suspended_state.rs`
- `crates/rum-syntax/src/lexer/`
- `docs/guide/resource-limits-and-sandbox.md`
- `docs/reference/builtins.md`

Catalog directory scope is limited to the named 83 declarations' companion/contract witness metadata. The shared parser/lexer/lowering scope is for an explicitly defined ToExpression accounting path only; do not change syntax, AST/Expr representation, parser error semantics or SyntaxQ's separate contract. The AST inclusion decision in §8 must be resolved before that subfamily is marked closed.

**Deliverable.** Work through responsibility-sized batches within this brief, recording each batch's direct routes and all helper accumulators, then the complete 83-name closure set. Cover selection indexes, replacement plans/binding results, recursion plus iterative fallbacks, numeric term/candidate intermediates, schema diagnostics/default insertion, strings' list outputs/joins, and the small-input-state copies of LazyTuples. Define and verify a structural syntax/token/lowering R formula if ToExpression's AST is included; an exact projection is an alternative only if it really bounds the whole simultaneously live construction and its falsifier passes. Keep AST, lowered output and optional wrapper-evaluation lifetimes distinct.

**rum / Rust pitfalls.** A pure Rust loop can escape a ceiling without calling evaluate. Selection can duplicate subtrees and retain indexes. Polynomial simplification can have large intermediates and small results. A parser using rum-syntax cannot depend on rum-runtime: any optional accounting hook must preserve the crate boundary and erase in ordinary parsing. Shared non-Incremental call sites remain on their old route; no incidental repair of Coefficient, SyntaxQ, Duplicates, Catenate or other findings.

**Verification.** FMT, RT, WS, NOHASH, CATALOG and all scoped family suites. Every converted declaration needs at least one reachable growth-path witness and explicit empty/invalid/no-growth coverage; each distinct helper/control-flow shape needs error/suspend/abandon coverage where supported. Add focused production low-limit tests for no-callback loops. Watch omitted selection-index/work-list, rewrite-result/plan, polynomial-intermediate, schema-error/default, string-join and ToExpression parser/lowering charges red then green. Preserve existing preflight tests and use a real per-builtin bad projection if proposing any reclassification. AB after each responsibility batch and at full step end: all §6 relevant target/neighbor panels, new direct representative labels for absent families, and the cumulative baseline curves. Do not infer erasure for generic parser/numeric helpers from Map's result.

**Hard rules.** Common instructions apply. No new arithmetic compatibility oracle, CAS promises, snapshot/interner changes or parser limit relaxation. Do not drop an intermediate early solely to evade its test. A positive builder census is a starting point, not permission to miss another helper in the same row.

**Report format.** 83-name table of actual closed witnesses and branches, honest residual list, each omitted-charge/release fault, doc/model changes, new direct benchmark statements and complete per-batch/per-step/pre-line curves.

#### P6 — Runtime state, diagnostics, namespace and symbol paths

**Goal.** Close the **62 remaining NeedsBuilder rows**: state (34), diagnostics (19), namespace (9), including the four special variables. Separately resolve **RegisterExceptionType and SyntaxInformation**. Reuse A–E S mutations and cover the locals around them.

**Context pointers.** The full grouped name lists/census; `R/builtin_adapters.rs`; `R/eval_core/symbol.rs:14–50`; `R/builtins_input.rs:104`; `R/debug_tools/reap.rs:32–55`; `R/debug_tools/trace.rs:150–206`; `R/definitions/definition_lhs_preparation.rs`; current owner helpers under `R/debug_tools/resources/maintained/`.

**Scope — MAY modify only these current paths:**

- `R/accounting_mode/`
- `R/kernel_api.rs`
- `R/builtin_adapters.rs`
- `R/builtin_catalog/`
- `R/attributes.rs`
- `R/builtins_context_loader.rs`
- `R/builtins_context_loader/`
- `R/builtins_input.rs`
- `R/debug_tools/`
- `R/definitions/`
- `R/eval_core/symbol.rs`
- `R/messages/`
- `R/namespace_tools/`
- `R/options.rs`
- `R/options/`
- `R/package_tools.rs`
- `R/package_tools/`
- `R/scoping/packages.rs`
- `R/scoping/unique.rs`
- `R/kernel_data/control_data/exceptions.rs`
- `R/kernel_data/control_data/exceptions/`
- `R/evaluation_cache/profile/report.rs`
- `R/profiling/summary.rs`
- `S/control_flow.rs`
- `S/lib.rs`
- `S/catalog/control.rs`
- `S/catalog/memory_growth_contract.rs`
- `R/tests/`
- `S/tests/debug_tools_resources.rs`
- `S/tests/definition_lhs_assignment_resume.rs`
- `S/tests/incidental_debug_tools_resources.rs`
- `S/tests/strings/incidental_runtime/`
- `H/tests/suspension_abandonment_contract.rs`
- `scripts/core-language-microbench.sh`
- `R/tests/mod.rs`
- `S/tests.rs`
- `docs/guide/resource-limits-and-sandbox.md`
- `docs/reference/builtins.md`

The broad definitions/debug/test directory scopes permit only these names' local builder plumbing, S/R transfer integration and owning tests. Existing S formulas/oracle and unrelated owner maintenance remain read-only; do not redo A–E. Save/DumpSave scope covers current rendering/storage accounting only, never snapshot format. Unique scope excludes gensym policy/quota/interner changes.

**Deliverable.** Armed companions for the 58 ordinary NeedsBuilder names, mode-aware special-variable construction for the other four, owner-to-local and local-to-owner handoffs, and definition/message/package/options/report rendering coverage. Cover direct print sinks as well as buffers, all trace output modes, Reap post-pop rendering, Set/TagSet/UpSet preparation, update targets/replacements, name/path collection, Once state, and the enabled RumProfileSnapshot branch. Resolve the two unsettled rows by full-branch proof or actual accounted construction, not a presumed small bound.

**rum / Rust pitfalls.** Held assignment inputs are not evaluated output bounds. S charge/release does not count surviving local outputs after split/take. Print sinks and buffers have different publication lifetimes. The four variables bypass BuiltinFn dispatch. A profiling-disabled test says nothing about the branch which builds sorted profile vectors. User-defined message/template evaluation may suspend.

**Verification.** FMT, RT, WS, NOHASH, CATALOG, owning runtime/stdlib tests plus H/suspension_abandonment_contract. Test successful/error/abandoned assignment and diagnostics, stack/name/value-table outputs, package rendering failures without changing formats, sink and buffer paths, nested Reap, report callbacks and all four symbols. Watch a Print formatting charge, a post-take Reap/Trace/report charge, an assignment-preparation charge, a special-variable charge and one release/abandon transfer separately omitted, red then green. Use deterministic in-repo fake file providers for FileNames/Save/DumpSave; no external filesystem/network. Account profiling diagnostic storage in a focused flag-enabled correctness test while all timing runs keep profiling off. AB: definition/append/Part/association-set targets and neighbors in §6, plus direct diagnostic/rendering/namespace labels declared before editing. Inspect unarmed assignment, formatting and symbol branches; report cumulative costs.

**Hard rules.** Common instructions apply. No owner-only closure without charge/release and every-exit evidence. No “existing metadata read” exemption for copied outputs. Preserve capability and message behavior. This step cannot fix the separate local-memlim docs conflict.

**Report format.** 62 whole-row closure records plus the two resolved-or-unsettled rows, direct/symbol routing proof, existing-owner versus R partition, raw fault/test output and full step/pre-line cost curves.

#### P7 — Closed census and final acceptance evidence

**Goal.** Require a final justified disposition for every installed Incremental declaration, **zero pending/unsettled entries**, separate unchanged S observation semantics, complete lifetime/production falsifiers, and a final cumulative cost report. Evidence determines acceptance.

**Context pointers.** This census and all accepted P1–P6 reports; existing installed inventory and S schema barriers; docs/testing-strategy and host-embedding contract. P3 owns Nest settlement; P6 owns RegisterExceptionType/SyntaxInformation settlement. Do not turn unresolved rows into generic “delegates to evaluate” entries.

**Scope — MAY modify only these current paths:**

- `R/tests/session_memory.rs`
- `R/tests/session_memory_oracle.rs`
- `R/tests/session_memory_maintained.rs`
- `R/tests/session_memory_request_owners.rs`
- `R/tests/session_memory_debug_owners.rs`
- `R/tests/memory_growth_contract.rs`
- `R/tests/suspension_abandonment.rs`
- `S/catalog/memory_growth_contract.rs`
- `S/tests/accounting_mode_route.rs`
- `S/tests/debug_tools_resources.rs`
- `S/tests/preflight_memory.rs`
- `H/tests/suspension_abandonment_contract.rs`
- `docs/guide/resource-limits-and-sandbox.md`
- `docs/host-embedding-and-hardening.md`
- `R/tests/mod.rs`
- `S/tests.rs`
- `docs/reference/builtins.md`

Implementation modules are read-only at close-out. If a gate identifies a source defect, report the exact narrow follow-up scope for the director, keep the line unaccepted, and continue independent verification; no silent repair outside this scope.

**Deliverable.** Regenerate installed/raw catalog/closed registry equality in default and no-hash configurations. Remove the staged pending allowlist. Verify all witnesses exist and actually exercise their declared route. Preserve the S oracle, independent R payload oracle, frame compile barrier, public host extension limitation and full output/projection behavior. Publish exact accounting-storage sizes/threshold effects, observation invariance evidence, real production rejection timing, and cumulative per-label and slope results.

**rum / Rust pitfalls.** A name-count of 165 can pass while one missing name and one extra cancel. Compare sets and contracts. A green unit suite with growth diagnostics enabled is not production admission proof. A test-only receipt cannot find an unregistered Rust local. A handwritten no-op frame trace can compile and still be wrong.

**Verification.** FMT, RT, WS, NOHASH, CATALOG. Watch the final gate fail for a newly Incremental real catalog declaration without a disposition; separately remove a companion, a witness, and one symbol path; restore and run green. Re-run the real omitted-progress/omitted-release/suspension/generation faults from every distinct converted shape, not mock ledger-only faults. Include exact-reading tests, request cancellation/reset/poison, repeated suspend/abandon, host override/fallback and nested arm. AB uses the entire now-exposed suite and all frozen instruction panels against immediate, A–E and pre-A–E baselines; publish the final cumulative triple and every label/neighbor regression. Recheck unarmed emitted code where final shared changes invalidate earlier evidence.

**Hard rules.** Common instructions apply. No acceptance with a pending row, unproven transfer, silently invisible extension allocation, weakened test or unmeasured alternative. Do not treat a cost bar as the completion criterion; report defects and measured paths honestly. No commit/release/tag action.

**Report format.** Source-derived final census and artifact hashes, every required green/red transcript, exact production/observation limits, final step and cumulative curves, and an achieved/not-achieved verdict for every acceptance requirement. The director performs the authoritative gate afterward.

#### Which old F–J material survives

| Old draft | Still useful | Obsolete / replaced here |
| --- | --- | --- |
| F | Separate builder R, callback-safe ownership, Table/history/string pilots, lifecycle and bad-then-good tests | Its hypothetical “accepted accounting modules” seam and combined protocol-plus-three-shapes batch. P1 uses the concrete context API; P2–P4 split the pilots. Full R-to-S release on freeze is insufficient for strings/capacity. |
| G | Functional/association/structure/lazy family grouping, seen/index caution, fast/recursive/resume audits | Assumption that every row has one uniform undischarged kind. Current per-row evidence drives P3–P5; the shared-owner portions are already maintained. Director-to-expand scope language is replaced by verified current lists. |
| H | Rewrite/string/algebra/schema/ToExpression intermediates and pure-loop falsifiers | Blanket “leave syntax trees outside unless separately approved” cannot simultaneously close ToExpression under the full local-builder reading; its AST boundary is now an explicit open decision. Actual parser/lowering paths are named. |
| I | Runtime-local outputs, mixed S/R stateful paths and the four special variables | Reimplementing definition/print/message/reap/trace owner accounting after A–E. P6 changes local construction/transfer only. Owner presence alone is no longer offered as a whole-row proof. |
| J | Set equality, witnessed fault injection, lifecycle tests, independent oracles and final honest verdict | Pre-A–E 94-field/52-owner assumptions; current code is 95/53. The old stop-at-cost/reproducible-regression shortcut is superseded by the 2026-09-22 measure-every-path ruling; unaccepted defects are still reported. Cost now includes the landed triple and both step/line baselines. |

Nothing here reopens unique/physical counting, stride removal, Symbol/interner, gensym quota, snapshot format, or R's admission-only role.

### 8. Unsettled list and director decisions

The unsettled list is **not empty**. These are the only three unclassified declarations; they remain in both the 165-row census and `unsettled.tsv`.

| Name | What was actually inspected | What prevents an honest whole-row class | Required next evidence |
| --- | --- | --- | --- |
| Nest | Ordinary/operator/invalid branches at `S/functional_ops/iteration.rs:47–76`; driver `:253–328`; continuation `:330–341`; apply/finish `:643–659,755`; operator constructor `S/functional_ops/mod.rs:120–131` | With collect_all=false, `items` is empty and no list history append occurs. The callback branches use Session::evaluate, but zero iterations/operator forms/invalid returns lack a named whole-branch charged delegate or exact construction projection. Thus neither “has a history” nor “entirely Delegated” is proved. | P3 must trace complete output/temporary publication for those branches and watch its actual disposition falsifier. A tested exact projection/accounted constructor may settle the finite wrapper path; do not relabel it to avoid proof. |
| RegisterExceptionType | Entire handler `R/kernel_data/control_data/exceptions.rs:27–88`, owner refresh after insertion `:81–83`, existing/conflict/invalid branches and registry restoration in `R/session/session_helpers.rs` | The registry portion is maintained. Type/parent clones, conflict formatting, returned registration and invalid forms are outside that insertion; no every-exit owner-only proof or whole-operation exact projection was demonstrated. There is no long accumulating registry loop inside this one handler to use as a substitute proof. | P6 needs a per-branch W/local/owner partition, release/reset/error evidence, and a real low-budget/omitted-charge test before declaring CoveredByOwner or building the missing portions. |
| SyntaxInformation | Stored `Expr` clone at `R/namespace_tools/syntax.rs:24–28`, setter refresh `:31–47`, fixed system-name templates starting `:122`, normal/invalid target entry branches | The getter is not a mutation through the SyntaxInformation owner. Default templates are fixed constructors; the stored source may be large. Neither an exact full template/output precheck nor a charged callee on every branch was found. “Small templates” is not the proof required by this brief. | P6 must demonstrate the exact wrapper/default/invalid and stored-read bounds and publication checks, or give them explicit accounted construction. Do not claim that a setter's owner refresh accounts for a getter's new output. |

Additional decisions/evidence needed, without reopening the settled constraints:

1. **Activation bridge feasibility.** Require the initially-unarmed-prefix/nested-arm test before accepting the protocol. Which concrete callback/continuation prototype should be rewritten into P1's implementation scope? Both plausible safe approaches need actual default-cost measurement. The simple “mode chosen when builtin starts” design is insufficient.
2. **Bookkeeping placement and observation fixtures.** Clarify how the strict unchanged-S requirement applies to newly installed companion entries and inline frame/route metadata already measured by today's S formulas. The constraint that R enters admission only is fixed. Require a concrete storage/layout partition, accounted overhead and exact-reading fixtures; do not approve invisible accountant bytes or silently broaden the S oracle. If literal pre-line layout equality is required, the straightforward enlarged Counter/adapter vector is not an acceptable implementation.
3. **ToExpression syntax/token retention.** Its parser/lowering locals are real, but the current S/W model and old H draft leave syntax trees outside their stated observation. Does the approved local-builder extension include a separately specified R syntax/token formula? The recommended full discharge does, restricted to ToExpression's explicit accounted path. If it does not, ToExpression needs an explicit coverage-boundary ruling and cannot be quietly recorded as completely charged. This does not authorize SyntaxQ or general parser semantics work.
4. **The three unsettled whole-row proofs.** P3/P6 own them; they must be resolved before P7. Accept the measured accountable-construction or exact-proof route which actually works. There is no requested blanket metadata reclassification.
5. **Public legacy extensions.** Preserve their registration behavior and state the precise limit: the kernel cannot see a host builtin's private unregistered Rust accumulator. Companions can participate; ordinary S/W walking remains for legacy callbacks. Full arbitrary host heap accounting is outside this structural line. No name-only census exemption may misrepresent an overridden standard handler as the covered implementation.
6. **Absent benchmark labels.** Approve the direct family labels as measurement work in P3/P4 and the named later batches. A neighboring label cannot stand in for an unwalked target. Keep pre-A–E artifact provenance and all three baseline comparisons explicit.

**Acceptance status of this design brief:** the full current declaration census and positive builder evidence are delivered; three source dispositions and the stated protocol/boundary questions remain explicit. There is no fabricated CoveredByOwner/Preflight/Delegated example and no fabricated historical class delta. The literal “one row of every class / two A–E class changes” sample request is therefore unsatisfied for evidence-based reasons, as explained in §2. Implementation falsifiers, erasure and cost claims remain plans, not executed results.

The only actual instruction-following exceptions were the disclosed bootstrap read-only git command and noncontemporaneous bootstrap/progress capture. No source/test/Cargo change, prohibited git mutation, external write, network call, or guest execution was made. Work stopped at the requested design boundary, not at a performance bar.

### 9. Final artifact and scope check

```text
SNAPSHOT_FILES 1491
CHANGED_BEFORE_REPORT_APPEND 0
CENSUS_UNIQUE_NAMES 165
NEEDS_BUILDER_FAMILY_SUM 162
UNSETTLED_NAMES Nest; RegisterExceptionType; SyntaxInformation
VERBATIM_EXAMPLE_ROWS 13
COMPLETE_TEMPLATE_DRAFTS 7
CURRENT_SCOPE_ENTRIES 187
SOURCE_TEST_CARGO_EDITS 0
AUTHORIZED_DOCUMENT_CHANGE append docs/dev/codex/brief-524-incremental-declaration-census.md
```

The original brief is preserved as an unchanged prefix. The final census contains 165 unique names, exactly matching the regenerated Incremental set; the family rows sum to 162; the three unsettled names are disjoint from them. All 187 scope entries name current files/directories. The source integrity check covers the 1,491 snapshotted files under crates/scripts/docs; it is a content comparison, not an executed Rust gate. No code/test success is inferred from it.
