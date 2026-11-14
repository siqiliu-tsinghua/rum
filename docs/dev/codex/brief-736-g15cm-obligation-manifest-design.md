# brief-736 — G15-CM: the obligation manifest (design only)

Scope: **NONE — no source change.** Survey the current tree and return a
reviewable plan plus the drafted implementation briefs.

## Goal

Plan G15-CM: make every discovered producer/consumer edge a standing
obligation, enforced by the suspension oracle, so that no census row can be
settled by argument and no owner can close over an uncaught tail. When you are
done I must be able to review a step-by-step plan — every claim carrying
`path:line` evidence — that states exactly what the manifest will enforce, how
each current obligation is represented, what is already enforced by today's
oracle, and which implementation briefs follow, each with its own mechanical
gate. You write no code in this brief.

## Context pointers

- `AGENTS.md`, `docs/dev/codex/TEMPLATE.md`, the testing policy and
  code-organization guidelines.
- The governing design: `.temp/brief-621/MANIFEST-DESIGN.md` (18,972 B) and
  brief 621's director continuations 1–3; `.temp/brief-621/draft-G15-CM-obligation-manifest.md`
  is raw material for my rewrite, not a specification.
- The census: `.temp/brief-621/continuation3/` (`remaining-owned-rows/G15-C2.tsv`,
  `c2-remaining-composition.json`, `owned-obligations.tsv`, `ledger.jsonl`,
  `required-tuples.tsv`, `mutations-*.tsv`, `ledger_model.py`,
  `owner-states.json`), then the campaign ledger
  `.temp/g15-c2/c68/settled.tsv` (`73ec2ff5…`, 1,042 rows, outcomes
  Boundary 20 / Equal 175 / Identity 5 / Lost 0 / NoObservableTail 126 /
  Residual 118 / Transferred 594 / Unreachable 4) and
  `.temp/g15-c2/c68/required-tuples.tsv` (`3c2dcdff…`, 78,330 requirement
  rows).
- The campaign invariants: `docs/dev/codex/brief-623-g15c2-settle-unowned-census-rows.md`
  ("Campaign invariants" and the move rules) — they govern how rows moved
  during C2 and must not be contradicted by the manifest.
- The oracle today: `crates/rum-stdlib/tests/suspension_oracle/` (catalog,
  descriptors, manifest, coverage, inventory, mutations, fixtures,
  host_facade, completed_segments) and its integration entry
  `crates/rum-stdlib/tests/suspension_oracle.rs`; the standing inventory
  tests that already pin producer sites (`logical_counter_port`,
  `typed_slot_census`, the retention-contract fingerprints and the
  ownership-graph inventories).
- The C2 tail admission: `.temp/g15-c2/c65/tail-admission.tsv` (121 rows at
  C65; now 118 — 77 encoding/G13, 29 parks, 7 unresolved sites, 3 held,
  2 original-restore) and the C67/C68 rulings in
  `.temp/director/state.md` and the memory topic file.

## Scope

- Files you MAY modify: **none** under `crates/`, `docs/`, `scripts/` or
  anywhere tracked. Scratch, copies and logs go under `.temp/brief-736/`.
- Read-only: every historical scene (`.temp/brief-621/`, `.temp/g15-c2/c5*`,
  `c59`–`c68`), the production tree, and the dispatch/target caches.
- No git, network, `unsafe`, inventory edit, `Cargo.lock` change or
  production-source edit. Copied trees only when a survey needs one.

## Deliverable

A plan I can review and rewrite, in this order:

1. **Identity reconciliation.** How the manifest derives its population from
   the current source and the accepted census, and how every one of the
   1,042 C2 rows and the G13-owned rows is covered exactly once. Stable
   source keys only; no line-number identities. State what re-deriving costs
   and what is imported by hash.
2. **Ownership model.** The exact `OwnedObligation { row, owner }` semantics
   against today's five open owners and the 118 C2 residuals: which rows may
   stay C2-owned, which must promote, what `Done` requires, and precisely
   when a gate is allowed to be green while an owner is Open. The draft's
   "C2 must have no owned residual" bar and the current 118 residuals
   disagree; the plan must resolve that conflict, not restate it.
3. **What is already enforced.** Enumerate, with `path:line`, which manifest
   invariants today's oracle and standing inventories already enforce
   (exact required/bound/executed multisets, receipts, owner promotions,
   mutants, negatives), which are asserted only in documents, and which the
   manifest must add.
4. **The enforcement design.** Discovery → requirements → bindings →
   executions/receipts → mutants → owners → promotions, with the failure
   each stage must be able to produce (ship the falsifier shape for each).
5. **Staging.** The implementation briefs in TEMPLATE shape, in dispatch
   order, each with its own shape stop and measurement/campaign stop where
   applicable. Name for each brief: the files it may touch, its gate tests
   and negatives, its maintenance duties, and its bounds.
6. **Cost.** What the expanded oracle costs to build and run, and which
   production hot path (if any) it touches — the mandatory hot-path A/B
   applies before any source-scope change beyond `tests/`.

## rum / Rust pitfalls (read twice)

- Equal requires the complete real SAME tuples **and** the row's own caught
  tail-skip mutant. A nearby frame, observer statement or scalar equality
  cannot substitute.
- Factories cannot define requirements; sets cannot hide duplicates.
- A row settled only by argument is an open obligation. A reading is not a
  verdict.
- A check nobody has watched fail is not known to work; every acceptance
  needs its falsifier, red on both arms.
- G13 rows may retain only their assigned still-Open owners; encoding keys
  are never executed or credited by the manifest.
- Historical scenes are read-only inputs. Never weaken an assertion, delete
  an attempt or re-pin an inventory to make a plan fit.

## Verification (MANDATORY — survey only; you change no source)

Run the existing oracle and inventory tests read-only in both feature modes
within the host discipline (`scripts/memory-scope.sh <unit> 16G`, `-j8`,
`RUST_TEST_THREADS=8`), and report the exact command lines and copied result
lines:

```
cargo test -p rum-stdlib --test suspension_oracle -- --nocapture
cargo test -p rum-stdlib --no-default-features --test suspension_oracle -- --nocapture
cargo test -p rum-stdlib --test typed_slot_census
```

Any test you cannot run in the sandbox is reported as not run, with the
reason. Do not sample to fit a budget.

## Checkpointing — protect the scene

Checkpoint after the identity reconciliation, after the ownership model and
after the drafted briefs. Record the concrete next action, what you ran, and
every open question with its evidence. Stop after two batches that add no
new evidence, or on any unexplained red. Preserve a filter interruption.

## Hard rules

- No git, network, `unsafe` or writes outside this repo and `.temp/brief-736/`.
- Rum against rum only; no weakened contract, invented receipt or silent
  baseline.
- Actual output and unrun gates disclosed. A partial model is not the full
  manifest; the C2 residuals stay Open behind their named bars.
- No source change of any kind is authorized by this brief.

## Report format

One `rum-report/1` via `scripts/report.py` (report path
`.temp/brief-736/report.json`), with sections: identity reconciliation,
ownership model, already-enforced inventory, enforcement design, staged
briefs, cost and risks, questions. Every claim carries `path:line` or a hash.
List every open question with the two candidate resolutions and their
evidence. `scripts/report.py check 736` must pass before you stop.

**Dispatch:** `scripts/dispatch-brief.sh 736` (background, `BRIEF_WORKER`
unset = `cdx`).
