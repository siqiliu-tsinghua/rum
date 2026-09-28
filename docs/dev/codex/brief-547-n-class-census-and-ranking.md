# brief-547 — design: settle the N class, group it by mechanism, rank the regions

`Scope: NONE — no source change.` Design-only. Survey, classify, return a
settled census, a ranked region plan and two drafted briefs. Write no code.
**Do not run `git` in any form**, including while first loading this
repository.

## Goal

The retention line has its first certified production coverage: C01
`Session::merged_option_exprs_for_symbol` and C02
`Session::evaluate_runtime_option_expr` are `LocalCovered` (`ee81758a`,
`inventory/promoted.tsv`) and cost nothing unarmed (`cae6f550`). The line now
converts the rest of the N class one region at a time, and it cannot order that
work, because the N class is not known:

- brief-525 settled 478 declarations and found **78** `InClass-NoProjection`.
  That is a lower bound.
- The 165 declarations carried from brief-524 were never split. 15 were
  sampled and all 15 were projectable; **the other 147 have no subclass at
  all**. A 15-row sample is not a finding about 147 rows.
- **90 rows are `Unsettled`**: 87 new, plus `Nest`, `RegisterExceptionType`
  and `SyntaxInformation` carried from brief-524. An unsettled row is not known
  to be outside N.
- The census was taken against the tree of 2026-09-22. Briefs 529-546 have
  since rewritten the option path and the files it runs through.

When you are done, every one of the 643 installed declarations stands, against
the **current** tree, in exactly one of `InClass-NoProjection` (N),
`InClass-ProjectableFromChargedInput` (P), `OutOfClass`, or an explicit
unsettled list that states exactly what evidence is missing and what would
produce it. The N class is then cut into **regions by conversion mechanism**,
ranked, and the next two briefs are drafted.

## Why grouping by named helper is not enough

brief-525's `helper-groups.tsv` names 163 work packages, 32 of which contain N
rows. The option path (`H100`) carried 31 N declarations and is done. After it
the leverage by helper is flat: 5 (`Count` `DeleteCases` `FreeQ` `MatchQ`
`MemberQ`), 4 (`And` `Nand` `Nor` `Or`), 3, 3, 3, 2, 2, 2, then **23
singletons**.

Many of those singletons read as one shape under different helper names: a
single guest-evaluated value held live across the next guest call —
`CompoundExpression` (through its installed companion `compound_from`,
`crates/rum-runtime/src/runtime/accounting_mode/continuation.rs:33`),
four-argument `If`, `Check`, `WithCleanup`, `Catch`, `CatchExceptions`, the
`Confirm*` family, `Assert`, `CheckAbort`, `Switch`, `TimeConstrained`,
`RepeatedTiming`, `Quiet`, `Enclose`, `LexicographicOrder`. If one protocol
shape closes them, that is a region the helper table hides. **That is a
hypothesis to confirm or refute from the code, not a finding to adopt.** The
same question applies to the 147 carried rows: they are the
`Incremental`-declared accumulators, and their evidence clusters in a few
files (`functional_ops/iteration.rs`, `functional_ops/apply_map/traversal.rs`,
`structure_ops/collection_stats.rs`, `definitions/definition_updates.rs`,
`definitions/definition_lhs_preparation.rs`), which suggests shared machinery.

## The population, and what each part owes

| Cohort | Rows | Where the prior finding is | Owed |
| --- | ---: | --- | --- |
| Carried positives, never split | 147 | `.temp/brief-525/carry-forward.tsv` (`NeedsBuilder`) | A full N/P split, one row each, from the code. No sampling. |
| Unsettled | 90 | `.temp/brief-525/unsettled.tsv` (87) and `carry-forward.tsv` (3) | Settle each, or state exactly what evidence is missing and what would produce it. |
| New N | 78 | `.temp/brief-525/census.tsv` | Re-verify at the current tree. For the 31 option rows, state what C01/C02 closed and every path that remains N, the four escaping copies included. |
| New P | 257 | `census.tsv` | Not re-audited. Screen every row: flag any whose bounding quantity is a callback result, a guest-evaluated value, or a count that is not already charged. A flagged row gets a full re-audit. |
| Sampled old P | 15 | `.temp/brief-525/carry-sample-changes.tsv` | Carried, subject to the same screen. |
| OutOfClass | 56 | `census.tsv` | Carried. Re-verify rows whose cited files changed (below). |
| **Total** | **643** | | Confirm 643 against the installed inventory. Any declaration outside these cohorts gets a full row. |

**Which files changed since the census.** `.temp/brief-525/source-snapshot.tsv`
holds the SHA-256 of 1 502 files as they stood. Hash the current tree the same
way and diff. Any carried row, in any cohort, whose handler chain, named
payload or evidence anchor sits in a changed file is re-verified, not carried.

The prior artifacts were copied byte-for-byte from `target/brief-524/` and
`target/brief-525/` into `.temp/brief-524/` and `.temp/brief-525/` by the
director on 2026-09-28. **They are hypotheses about an older tree, not
evidence.** Every row this brief reports as settled is settled by a read of the
current source.

## The criterion — brief-525's, unchanged

A declaration is **in the class** when there exists an execution path on which
the builtin holds a local payload whose size depends on input or callback data,
and that payload is, at that moment, none of:

- (a) already charged in `S` — it has been published to a maintained owner;
- (b) covered by an exact projection **taken before the allocation**;
- (c) part of `W`, the in-flight expression being evaluated.

- **`InClass-NoProjection`** — nothing bounds the payload before it exists.
- **`InClass-ProjectableFromChargedInput`** — the payload is bounded by a
  constant factor of a quantity that is **already charged** (an argument's
  proxy bytes, an input string's length, a declared iteration count). Name the
  bounding quantity and say what the factor depends on. Do not estimate the
  factor.

`OutOfClass` requires (a), (b) or (c) with a `path:line`. "Small", "bounded in
practice", "assumed constant", "not currently instrumented" and "the declared
contract says so" are not dispositions.

Two rulings bear directly on the split. Both are made:

- **A P verdict removes a row from this line's work** (user ruling: P is a
  documented bound — `docs/guide/resource-limits-and-sandbox.md`, "What the
  ceiling does not see"). A wrong P hides a hole; a wrong N costs one
  conversion. So **P carries the higher burden of proof**: name the
  already-charged quantity, show that it is still charged **at the moment the
  payload is live**, and show that the factor does not depend on callback or
  guest-evaluated data. A row that cannot meet this is N or unsettled, never P
  by default.
- **A single retained callback result is N.** Its size is unknown until it
  returns (brief-525 pitfall 5), and one value held per level multiplies under
  guest-controlled nesting. Do not classify it P because "it is only one
  value".

Where a row's paths disagree, the row takes the worst path's class, and every
other N path is still listed: the region map below needs all of them, not only
the worst.

## What I want back

### 1. The settled census

One row per declaration for the 147, the 90 and every re-verified row, as a TSV
under `.temp/brief-547/` and summarized in the report, with: name, declared
contract, class, handler chain, every named local payload with `path:line`, the
bounding quantity where one exists (a P row must meet the burden above), and,
for an N row, **the retaining body** — as a `path:line` and as the function
identity the local lint uses (for example
`rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol`)
— and the guest-reaching call the payload is live across.

Then the 643-row reconciliation: a cross-tabulation of declared contract
against class, and how each cohort moved (for example "147 carried → x N, y P,
z OutOfClass, w Unsettled").

### 2. The closure map — N declarations to regions

For every N declaration, **every** N path, and for each path the body that
retains the payload. Define a **region** as a set of retaining bodies closed by
one conversion mechanism, and give both directions:

- `declaration → {regions}`. A declaration is closed only when every region on
  every one of its N paths is covered and no escaping copy remains.
- `region → {bodies, declarations it fully closes, declarations it only
  advances}`. "Fully closes" counts declarations whose **last** open path the
  region closes. That is the number leverage is ranked on.

### 3. The mechanism of each region

For each region:

- the retention shape — single retained value, accumulator of callback results,
  matcher or pattern state, prepared-LHS state, handoff, suspension-carried
  prefix, or whatever else the code shows;
- whether the existing lease protocol already provides that shape —
  `RetentionContext`, `RetainedValues`, `CallbackValue`/`CallbackFailure` (the
  activation bridge), `FrozenValues` with `RetentionContinuation`,
  `RetainedOptions` — citing `accounting_mode/protocol.md` and the source. If it
  does not, name the missing surface exactly and every other region that needs
  the same surface;
- whether it suspends, and what its continuation frame would have to carry;
- every call it makes that would need a mark — a contract in brief-537's
  format, an admission, or another covered body — marking which are
  **installed** today and which are `Pending`
  (`inventory/local-contracts.tsv`);
- the microbench labels its conversion exposes, recorded for the later
  erase-unarmed step. This is not a ranking argument.

### 4. The ranking

Every N region will be converted; the ranking sets the order only. Order by,
in turn:

1. **Prerequisites.** A missing protocol surface, a `Pending` contract, or an
   admission that several regions need becomes its own step, ahead of the
   regions that need it. Draw the dependency graph.
2. **Leverage**: declarations fully closed (§2), then declarations advanced.
3. Nothing else. Size, site count and hot-path exposure are not ranking
   arguments (standing ruling: cost arguments do not count).

Give the ranked list with a predicted closure count per step, so each step's
result can be checked against a number written down in advance.

### 5. The four escaping copies — the first implementation step, by ruling

`inventory/consumer-obligations.tsv` records four `UnresolvedConsumer` rows, an
option-derived value copied out of the leased collection by its consumer:

- `rum_stdlib::predicates::collections::effective_same_test` (`predicates/collections.rs:471`)
- `rum_stdlib::structure_ops::ordering_sets::effective_same_test` (`structure_ops/ordering_sets.rs:593`)
- `rum_stdlib::strings::scripting::text_shape::parse_align_columns_options` (`strings/scripting/text_shape.rs:284`)
- `rum_stdlib::strings::scripting::url::parse_url_parse_options` (`strings/scripting/url.rs:245`)

They are the option region's own leftovers: its 31 declarations are not closed
until these are discharged. **They go before any new region** (director
ruling). Do not re-rank them. For each copy:

- what it is live across — whether the consumer runs guest code (a `SameTest`
  callback, for example) while the copy lives — with `path:line`;
- its disposition: borrow from the collection while it is still leased,
  transfer the copy under a lease, or a P argument that meets the burden above,
  with the charged quantity shown live at the moment the copy is;
- what changes in `promoted.tsv`'s exclusions and in
  `consumer-obligations.tsv` when it is discharged, and the falsifier that
  shows the discharge is real — a check nobody has watched fail is not known to
  work.

Then **draft that implementation brief** in `TEMPLATE.md` shape.

### 6. The top-ranked new region — draft its design brief

Draft the design-only brief for the region you rank first after the four
copies, in the shape of
`docs/dev/codex/brief-541-option-region-conversion-plan.md`, which is the
model.

The local lint has no hook today for pointing it at an arbitrary production
body: `local/production.rs` checks `certificates::roots()` and the brief-538
lease probes only. The draft must say how its worker obtains the region's
rejection rows — the smallest test-harness change that does it, and whether
that change belongs in the design brief as a declared exception or in a step
before it.

Both drafts: Scope states **an objective, a fence and a disclosure duty**, not
a list of permitted files. Scratch and checkpoints go under
`.temp/brief-NNN/`. The stop-loss is keyed on progress (`TEMPLATE.md` pitfall
7).

### 7. Open questions for the director

Anything you could not settle, any row where two readings of the criterion give
different classes, and any decision you think is mine.

## Settled — do not reopen

- Traversal is retired; coverage is local
  (`docs/dev/retention-closure-design.md`). Do not propose a closure walk, and
  do not revive A3-A6.
- Condition 1 is retention across a call that can run guest code or grow
  retained state, not construction (`a388d11a`).
- P is a documented bound, not work. This brief classifies P rows; it does not
  plan them.
- The option region's `R` is a runtime certified domain (user ruling U1,
  2026-09-27). C01/C02's certificates and their recorded exclusions stand.
- `ToExpression` is ruled (brief-525): its parser and lowering locals are
  bounded by the already-charged input string.
- The occurrence model and the `S` model stand. `rum-core`'s `Expr`
  representation, the interner, gensym and the snapshot format are out of
  scope.

## Context pointers

- `AGENTS.md` first.
- `docs/dev/retention-closure-design.md` — why traversal was abandoned, what
  "covered" means, and the three contract shapes. Read it before anything else
  on this line.
- `docs/dev/codex/brief-525-local-retention-class-census.md` — the criterion
  (to line 254) and its report (from line 257).
  `docs/dev/codex/brief-524-incremental-declaration-census.md` — where the 165
  carried rows came from.
- `.temp/brief-525/` — `census.tsv`, `carry-forward.tsv`, `unsettled.tsv`,
  `helper-groups.tsv`, `source-snapshot.tsv`. `.temp/brief-524/census.tsv` —
  the carried rows' original evidence.
- `crates/rum-stdlib/src/catalog/memory_growth_contract.rs` — the 643
  declarations; its test at `:668` pins the count (643 with the default `hash`
  feature, 637 without).
- `crates/rum-runtime/src/runtime/accounting_mode/` — the lease protocol:
  `protocol.md`, `retained.rs`, `bridge.rs`, `frozen.rs`, `lease.rs`,
  `options.rs`, `options/`, `rule.rs`, `continuation.rs`.
- `crates/rum-runtime/src/runtime/options/core.rs` — C01 and C02 as converted:
  the model conversion.
- `crates/rum-stdlib/src/tests/retention_closure/inventory/` — `promoted.tsv`
  with its exclusions, `local-contracts.tsv` (installed versus `Pending`),
  `consumer-obligations.tsv`, `proposed-boundaries.tsv`.
- `crates/rum-stdlib/src/tests/retention_closure/local/` — the lint;
  `production.rs` and `certificates.rs` show how the two production bodies are
  checked.
- `docs/dev/codex/brief-541` through `brief-546` — the option region's six
  steps, the pattern each later region follows.

## Scope

- Files you MAY modify: **NONE**, with two exceptions: append your report to
  **this** file, and write scratch under `.temp/brief-547/`.
- Everything else is read-only, `.temp/brief-524/` and `.temp/brief-525/`
  included. No source, test, doc or `Cargo.toml` change. A corrected class is a
  column in a table, not an edit.

## rum / Rust pitfalls (read twice)

1. **The declared contract is the thing under audit.** Do not use it as
   evidence in either direction.
2. **The prior census describes an older tree.** A row whose files changed is
   re-read. A row whose files did not change is still yours to confirm if you
   report it as settled here.
3. **A P verdict removes work.** Hold it to the burden above.
4. **Payloads hide.** `collect()`, `extend`, a `push` inside a helper, a
   `String` built with `write!`, a `HashSet` built from clones, a value moved
   into a closure capture or into a suspension frame — all are payloads.
   Selection is not non-amplification.
5. **A named helper is not a mechanism.** Two helpers with different names can
   need the same protocol surface, and one helper can hold two different shapes
   on two branches. Group by what the conversion needs.
6. **Suspension carries retention.** A payload live across a call that can
   suspend is carried by a continuation frame; the option region had to carry
   its prefix as a lease transferred exactly once (briefs 543-544).
7. **A count derived by regex is suspect.** Cross-check every count against
   live source reads or the installed-inventory test, and say which you
   checked.

## Verification (MANDATORY — the evidence IS the deliverable)

- State every command, and persist its raw output under `.temp/brief-547/`
  **before** parsing it.
- Run exactly one cargo command, once, to pin the installed population:
  `cargo test -p rum-stdlib --lib installed_builtin_memory_growth_inventory_is_sorted_and_exhaustive`.
  Paste its result line.
- Paste the reconciliation counts, the declared-contract × class
  cross-tabulation, and the per-cohort movement, verbatim.
- Paste at least fifteen full census rows: at least five from the 147, at least
  three from the 90, every row the P screen flagged, and at least two per
  region you rank in the top three.
- Paste the region table and the ranked list with predicted closure counts.
- State the unsettled list explicitly, or state that it is empty.

## Checkpointing — protect the scene

After each milestone — hashes diffed, the 147 split, the 90 settled, the
screen, the closure map, the ranking, each draft — write
`.temp/brief-547/PROGRESS.md` stating what is done and verified with its saved
evidence, the exact next action, and anything the director needs to take over.
This brief is long; an interruption that leaves a readable checkpoint costs
one dispatch.

## Hard rules

- **No `git`, in any form.**
- Do NOT modify any file outside the two exceptions in Scope. If you believe
  you must, STOP and write that in the report instead.
- Touch nothing outside this repository. No network. Commands are limited to
  file reads, `rg`/`grep`, `sha256sum`, `python3` for parsing, and the one
  cargo command above.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **No code.** Editing a `.rs` file means you have left the brief.
- Do not settle a hard row by widening the criterion, and do not pad the census
  to completeness.

## Report format (your final message, appended to this file)

1. What changed — "design only, no source change", plus the scratch artifacts.
2. Verification — the commands, the counts, the cross-tabulation, the rows
   named above.
3. The settled census and the reconciliation; the unsettled list.
4. The closure map and the region table with each region's mechanism.
5. The dependency graph and the ranked list with predicted closure counts.
6. The four escaping copies: dispositions, and the drafted implementation
   brief.
7. The drafted design brief for the top-ranked region.
8. Open questions for the director.

---
