# brief-541 — design: converting the option region, the first production coverage

`Scope: NONE — no source change.` Design-only. Survey, classify, return a plan
and drafted implementation briefs. Write no code. **Do not run `git` in any
form**, including while first loading this repository.

## Goal

The local retention lint works and lease-correct code lints clean
(`d69cbaff`). The option region does not, as it must not yet:

- **C01** `Session::merged_option_exprs_for_symbol`
  (`crates/rum-runtime/src/runtime/options/core.rs:87`) — **17** rejections.
- **C02** `Session::evaluate_runtime_option_expr` (`:248`) — **34** rejections.

Both raw constructions are among them: the `Vec` accumulator at `:101` and the
`vec![args[0].clone(), value]` at `:259`, each live across a call that can run
guest code. Converting this region onto the lease protocol and making both
bodies lint clean would be **the first production coverage this line has
ever produced**.

Plan that conversion. The last implementation brief on this line stopped twice
because the director misclassified a construct (`Expr == Expr` is a call to a
trait impl, not an operator admission), so **classify every one of the 51
rejections before drafting anything**.

## Current rejections, as the director tallied them

C01: expression type unknown (8), use after ownership transfer (2), an opaque
imported trait candidate, a method instance with unknown bounds or arity, a
signature outside A1/A2, `path R has 0 exact bindings`, a concrete method
contract that cannot exclude an imported trait candidate, one callee without a
mark.

C02: expression type unknown (19), calls outside A2's checked scope inside the
rejected macro (3), `macro has no admission rule` (2), argument differs from
concrete parameter — coercion not admitted (2), no unique inherent definition,
`Pending contract B10 is not a mark`, `Pending contract B11 is not a mark`,
pattern state not admitted, only the exact probe continuation struct admitted,
`let-else` not admitted, one callee without a mark.

The per-row files are `target/brief-539/C01-rejections.tsv` and
`C02-rejections.tsv`; regenerate them from the current tree rather than
trusting them.

## What I want back

### 1. A disposition for every rejection

Each row gets exactly one mechanism, with `path:line`:

- **Production conversion** — the code must change to use the lease protocol
  or the activation bridge (`dc6a1969`). The two raw constructions are this.
- **Contract** — condition 2: an exactly resolved concrete callee needs an
  installed contract record in brief-537's format. B10 (`Expr::call`) and B11
  (`system_expr("Rule")`) are here and are `Pending`; say what discharges
  each, and what B04 (`Expr` clone — compiler-checked only for fieldwise
  cloning, quantitative charge pending) needs before a covered body may call
  it.
- **Admission** — condition 1's surface: `resume_or_suspend!` (brief-536 row
  13 gave the adapter shape: definition, arm and hygiene checked, exact
  success / InputRequired / error branches from
  `runtime/suspension/core.rs:761-777`), `let-else`, the pattern state, the
  coercions, and anything else.
- **Type machinery** — cascaded "type unknown" rows that disappear once
  something upstream resolves. Identify the upstream cause for each cluster.
- **Generic `R`** — the loop is generic over `R: OptionReentry`. B03 is now a
  runtime-closed set of registered `TypeId`s (`dc6a1969`). Say whether the
  lint can treat `R` as that finite registered set the way brief-540 treated
  the accounting mode, **and what keeps the lint's set and the runtime's set
  from drifting apart** — a lint that believes in a closed set the runtime
  does not enforce is a facade.
- **Never-admit** — if any row genuinely needs `dyn`, an open generic, or an
  unproved dispatch, say so; that body then needs a different shape or stays
  uncovered.

A row whose mechanism you cannot settle goes on an explicit unsettled list.

### 2. The production changes, and their cost

The conversion changes code on the option path, which every option-consuming
builtin takes. For each change: what it is, which suspension and abandonment
paths it touches (the continuation frames from brief-530/531 carry the
accumulated prefix — they must now carry it as a lease, transferred exactly
once), and the microbench labels it exposes. The measurement discipline is
settled: allocation counts first; per-iteration **instruction** slopes via
`perf stat -e instructions:u` pinned with `taskset -c 2` on the exposed labels;
timing only as a secondary panel, because timing on this host has produced
false regressions of 7-12% that instructions refuted.

### 3. The order, with a prediction per step

Cut the work into implementation briefs along real boundaries. For each: the
rejections it removes and a **predicted** C01/C02 count afterwards, so each
step's result can be checked against a number written down in advance.

### 4. What promotion means for this region

When C01/C02 lint clean, they become covered and `promoted.tsv` gains its
first production rows. State exactly what that certifies and what it does
**not** — the four `UnresolvedConsumer` escaping copies recorded in
`inventory/consumer-obligations.tsv` are outside these two bodies, so the
region's callers are not thereby covered. Say how the promotion is recorded
so it cannot be read as more than it is.

### 5. Drafted implementation briefs

In `TEMPLATE.md` shape. Scope sections state **an objective, a fence and a
disclosure duty**, not a list of permitted files. Scratch and checkpoints go
under `.temp/brief-NNN/` (the template now says so; `/tmp` and `target/` are
not used for scratch).

### 6. Open questions for the director

## Settled — do not reopen

- Traversal is retired; coverage is local (`docs/dev/retention-closure-design.md`).
- Condition 1 is retention across a call that can run guest code, not
  construction (`a388d11a`).
- Only B03, B09 and B12 are installed contracts; the other thirteen of
  brief-537's records are `Pending` and are not marks.
- The line targets the N class; P is a documented bound.
- `rum-core`, the interner, gensym, the snapshot format and the `S` model are
  out of scope.

## Scope

- Files you MAY modify: **NONE**, with two exceptions: append your report to
  **this** file, and write scratch under `.temp/brief-541/`.
- Everything else is read-only.

## Verification (MANDATORY — the evidence IS the deliverable)

- State every command; persist raw output under `.temp/brief-541/` before
  parsing it.
- Paste the full disposition table, and the per-step predictions.
- Run the checker once to confirm the tree is as described: 37 checker tests
  green; C01 17 and C02 34.
- State anything you could not settle.

## Checkpointing — protect the scene

Write `.temp/brief-541/PROGRESS.md` after each milestone with what is done,
the exact next action, and anything the director needs to take over.

## Hard rules

- **No `git`, in any form.**
- Do NOT modify any file outside the two exceptions.
- **No code.** Editing a `.rs` file means you have left the brief.
- **Testing Policy is a hard line** (`AGENTS.md`).
- Do not settle a hard row by widening the admitted surface generally or by
  treating a `Pending` record as a mark.

## Report format (your final message, appended to this file)

1. What changed — "design only", plus scratch artifacts.
2. Verification — commands and verbatim tables.
3. The disposition table for all 51 rows, and the unsettled list.
4. The production changes and their exposed labels.
5. The ordered steps with predicted counts.
6. What promotion certifies and does not.
7. The drafted implementation briefs.
8. Open questions.
