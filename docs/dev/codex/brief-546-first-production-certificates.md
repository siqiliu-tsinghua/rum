# brief-546 — record the first production certificates

Implementation brief. **Do not run `git` in any form.** Everything you need
about the tree's state is in this brief.

## Goal

The option region is converted, lints clean, and no longer costs anything
unarmed (`7d7680de`, `cae6f550`). Record what it has earned: **the first
production coverage this line has produced** — two production body rows, C01
(`Session::merged_option_exprs_for_symbol`) and C02
(`Session::evaluate_runtime_option_expr`), each with its finite instance
certificates attached.

This is the last of brief-541's steps. Its draft (§7 "Draft brief-545 — record
the first local production certificates") and its §6 "What promotion
certifies — and does not" are in
`docs/dev/codex/brief-541-option-region-conversion-plan.md` from line 165.
**Read both in full; they apply**, with the director's additions below.

**Predicted end state, written before any work:** C01 0 and C02 0 for the
declared instances; **exactly two production body rows** in `promoted.tsv`;
the four `UnresolvedConsumer` rows unchanged. If a helper genuinely earns its
own production mark, disclose and count it separately — never hide it inside
a region mark to keep the number at two, and never suppress it.

## What a certificate records

Mint a mark only after that body and instance pass their own lint in this run.
Each production row records: the source/body fingerprint; the exact build,
accounting mode and `R` instance; its direct covered-mark and contract
dependencies; preconditions; contract strengths; and **exclusions**, stated in
the row, not only in prose elsewhere. The exclusions include at least:

- its callers and the whole option consumer — coverage is local;
- arbitrary external `R` — and specifically **the limit brief-544 found**: the
  certified-domain installer is public, so a host can install its own
  declaration and assert its own trust; that is outside this repository's
  certificates;
- guest- and host-private allocation, process RSS and allocator overhead,
  interner storage, stack-safe destruction of deep `Expr`s, and any
  universal allocation bound derived from tests;
- the four escaping consumer copies.

## The four consumer obligations stay open

The four `UnresolvedConsumer` rows in
`inventory/consumer-obligations.tsv` are preserved exactly as they stand in
the current tree — their anchors moved in brief-544, so use the file, not the
older copy in brief-541's §6. A required falsifier must reject changing any of
them to covered merely because C01 and C02 are promoted: a clean callee does
not launder its caller.

## Scope

**An objective and a fence, with a disclosure duty.**

**The objective.** Enable earned production marks for the exact declared
instances, implement durable inventory validation and its falsifiers, and
state the exact guarantee in `docs/testing-strategy.md`. **Guidance:**
`local/marks.rs` (it refuses production today), `local/production.rs` and
`retention_closure/mod.rs` (they assert an empty promotion), `interface.rs`
(fingerprints), and the `inventory/` records. Replace the unconditional
empty-inventory and zero-production assertions with exact expected-row checks
that still refuse shrinkage, staleness, redirection and absence.

**The disclosure duty.** Every touched path appears in your report with line
ranges and why.

**The fence — STOP conditions:** production Rust and its performance are
read-only; no new admission or contract, no broader domain, no caller
promotion, no traversal, no consumer-obligation discharge; no `rum-core`,
interner, gensym, snapshot or `S` change; no manifests or lockfile; no
`AGENTS.md`, `PROGRESS.md`, `TODO.md`.

## Falsifiers — watched red, then green

A raw local held across a guest call; a missing, stale or `Pending` mark; an
expired body or domain fingerprint; a removed or redirected promotion; a
runtime certified set that no longer matches the declaration; an omitted
declared instance; an escaping-copy row falsely marked covered. Replay the
fresh C01/C02 certificates and every `LeaseProbe538` check.

## Verification (MANDATORY)

Director baseline: 45 checker tests green (one ignored entry is a fingerprint
regeneration tool), C01 0 for both instances, C02 0; workspace 2 671 passed,
0 failed.

```
python3 scripts/run-bounded.py --timeout 600  -- cargo fmt --check
python3 scripts/run-bounded.py --timeout 1800 -- cargo test -p rum-stdlib retention_closure -- --nocapture
python3 scripts/run-bounded.py --timeout 1800 -- cargo test -p rum-stdlib --no-default-features retention_closure
python3 scripts/run-bounded.py --timeout 2400 -- cargo test --workspace
```

**No new A/B**: production is read-only. Cite the measurement artifacts of
brief-543, brief-544 and brief-545 by path rather than producing new numbers.
Nine native ZeroMQ tests cannot bind loopback in your sandbox; report them as
skipped-by-environment and do not set `RUM_ALLOW_NATIVE_ZEROMQ_SKIP`.

## Checkpointing — protect the scene

All scratch under `.temp/brief-546/`. Update `.temp/brief-546/PROGRESS.md`
after the certificate plumbing, the inventory reconciliation, the
documentation, and the gate.

## Hard rules

No `git`, no production edits, no new `unsafe`, dependency or nightly, no
`Pending` waiver, no generalised admission, no traversal, no weakened test.
Refuse stale or absent evidence. Do not mark the goal achieved if C01 passes
only by omitting unproved executions from a universal claim. Stop-loss keyed
on progress, as the template states: failing required tests, rejection counts
and compiler errors falling are all progress.

## Report format

Changed paths and ranges; every command with verbatim output; the
certificate table — exactly what each row certifies and excludes; the four
`UnresolvedConsumer` rows unchanged; falsifier outcomes; the cited
measurement artifacts; remaining limits. Append to this file and leave the
tree uncommitted.
