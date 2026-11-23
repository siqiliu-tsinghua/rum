# Brief 785 — G15-CM4 Stage D repair: the proof-closure re-derivation inventory — design-only

Base: `518ca9d9` (the c142 gate; brief 784 dispatched from it). You are the design worker for
this brief, not the project director. `CLAUDE.md` belongs to the director and is not your
instruction file. Follow `AGENTS.md` and this brief. Do not commit, branch, or push.

## Why this brief exists

Brief 784 (the render repair implementation) stopped at the brief's own stop:
`validate_closure` (`crates/rum-stdlib/tests/suspension_oracle/fixtures/obligations/
proof_controls.rs:87-169`) fails after the authorized proof-line update. The first check (the
`occurrences(TARGET)` array) now matches the armed image's four boundary hits `(LHS 268),
(CALL 625), (CALL 923), (CALL 989)`; five further checks then fail, previously masked by the
early `ClosedCallers` return: the `build_call_dispatch_plan` population (11 sites against a
declared 8), the prep/opt handoff layout (the `(if crate::cm740_probe::driven() { … } else
{ … })` wrapper re-indents it), `lhs_from` (the same wrapper), the `stage_index +=` count
(6 = 3 per path × the plain and `_cm740_driven` twins now inside the span), and the
`resume_prepared_call_dispatch_frame` positions (504/767/805 → 903/1214/1256; count still 3).
On live and the baseline all 18 checks pass. The three runtime files are byte-equal to the 779
preimage, so the divergence is the class-form render, not brief 784's edits.

The plan's "Closed callers (2)" disposition (`.temp/brief-783/plan.md:238-252`) re-derived only
the first array, so its own falsifier ("both tests unwrap `Ok`") cannot hold under the
authorized update. This brief is the plan completion demanded at the stop: a complete
zero-unknown inventory and design for re-deriving the whole source-reading proof-closure
control family, so the continuation can certify the armed image without inventing proof
semantics. **Design only: `Scope: NONE — no source change.`** You write no source; you plan,
with `path:line` evidence, and you draft the continuation implementation brief 786.

## The carried state (the entry bar; re-measure it yourself)

The 779 sources are `.temp/brief-779/{baseline,images/armed,images/disabled,images/real}`;
brief 784 copied them to `.temp/brief-784/images/{baseline,armed,disabled,real}` and applied
stage A. The pins under the worker's own `tree_hash` (the algorithm in
`.temp/brief-784/tools/common.py`: `os.walk` with `EXCLUDES = {.git,target,.temp,
__pycache__,.mypy_cache}`, sorted dirnames and filenames, digest of `b"F\0" + relpath + b"\0" +
content`) are: baseline `3d0b6829b400974b1f1e3d3ee42f0251929180996db127a015c85860cc2322fa`/5,842;
armed `b11a687b4cab943ca629583fdc55203ea4097cb0ea1be80d741424c57dc4de58`/5,851; disabled
`3ed28bddb1a74d1edddaec84247aca46e6e2865eea7f1f4503421084585b88f2`/5,851; real
`57e3533e9be0d52bfb5296f54fdb384c53d1256b794880ade61fcaa3c10ec1ce`/5,851; debris 0 on each.
The armed pin equals the tree in the brief-784 stage logs, so the state is exactly what the
stop saw. Live: `git status --porcelain` is 23 inherited entries (do not run git; use the
record); the live `crates/rum-stdlib/tests/suspension_oracle/mutations/source_auth/
observer_policy_tests.rs` sha is `d17e200b63713a37e933225867ec4ebdd5ad0124f7b6013d9362f8dc91418a4f`
(F14, the only live edit; equal on all four images). `proof_controls.rs` is `a3092419…` on
live, the 779 install, baseline and real, and `0465ea9d…` on armed and disabled (the one-line
expected array). The three arm-token files are `lib.rs`, `runtime/eval_dispatch/mod.rs`,
`runtime/profiling/logical/segments.rs`; armed↔disabled differs in nothing else.

Any drift from these pins is a stop.

## Scope: NONE — no source change

Read-only. Your writes go to `.temp/brief-785/` only. Do not edit the images, live source,
`.temp/brief-784/` or anything else. Running cargo (the two proof tests and the replicated
scan) is allowed; use a per-tree `CARGO_TARGET_DIR` under `.temp/brief-785/` (never a shared
dir across trees), a measured `pwd`, and log the tree hash with every run.

## The proof-closure family (three files; confirm by your own sweep)

- `crates/rum-stdlib/tests/suspension_oracle/fixtures/obligations/proof_controls.rs`:
  `current_sources`, `occurrences`, `span`, `ordinary_reachable`, `validate_closure`.
- `crates/rum-stdlib/tests/suspension_oracle/ledger/proof_tests.rs`: `cm_proof_certificates_exact`
  (line 19) and `cm_proof_negative_controls` (line 187), including the 15 controls and the
  `CM PROOF CONTROLS PASS controls=15 …` line.
- `crates/rum-stdlib/tests/suspension_oracle/ledger/proofs.rs`: `validate_proof`'s P0496
  branch at 276-277 (the certificate path that embeds `validate_closure`), `reconcile`.

Also sweep the 75-name red universe (the two `failures:` blocks of
`.temp/brief-780/logs/stage-d-workspace.log` and `.temp/brief-776/logs/stage-d-workspace.log`)
for every test that reads `crates/**` source text at runtime (`workspace()`,
`current_sources()`, `include_str!`, `read_to_string` of repo source): list each name, its
file, and whether it is source-reading. Any other source-reading name must be green at the
carried state or named as a stop — zero unknown.

## Stage A — entry and the instrument (300 s)

Re-measure the pins and the live state. Re-run the two proof tests on the armed image (full
paths, `--exact`; expect `0 passed; 2 failed` with `Err(Closure)` at proof_tests.rs:19 and
:187). Replicate the 18 checks with your own scanner on live, baseline, armed and disabled:
the first array (old and new), each boolean's value, and the five failing names. Quote the
values. The instrument is calibrated only if you reproduce live/baseline all-pass and
armed/disabled five-fail; a mismatch is a stop.

## Stage B — the check inventory (900 s)

Enumerate every sub-expression of `validate_closure` and `ordinary_reachable` with: an id
(the first array, then the 17 booleans in order, then `ordinary_reachable`); the property it
certifies (one sentence); the preimage value (live/baseline); the postimage value
(armed/disabled); the `ProofError` variant it participates in; and the control that can
currently fire it. The six `span()` calls and their literals are part of the inventory. Zero
unknown: every expression mapped, none "unclear".

## Stage C — the postimage population and per-site attribution (1,200 s)

For every occurrence the populated checks count, give: path, line, enclosing function (a
precise parse — `syn` or a verified scanner), whether it is a `_cm740_driven` twin of an
unchanged live site, the live counterpart (path:line:function), and the declaration that
introduces it — the class table cell, the emission-chain row, the overlay row, or the F5
addition (`crates/rum-runtime/src/cm740_probe/tests/cm743_route_a_falsifiers.rs`, sha
`406c7dc3…`) — or "unchanged live site". Populations: the 4 TARGET hits; the 11
`build_call_dispatch_plan` sites (LHS 918, eval_call_prepare 697/715, call.rs 314/338, plan 42,
holding 253/256/265/276); the 3 `resume_prepared_call_dispatch_frame` sites; the 6
`stage_index +=` (which function each belongs to); the wrapper sites; and the preimage
counterparts of each. An unattributable site is a stop. State the +399 attribution for call.rs
(the render's insertions: which declared sites account for the line growth).

## Stage D — the per-check dispositions, the falsifiers, the overlay plan (900 s)

For each check, exactly one disposition:

- **re-pin** — the new declared value comes from stage C (each entry attributed), never from
  `occurrences()` on the certified expression; the exact new expression is drafted.
- **restate** — a stronger, layout-independent form. The doubled path must not become a single
  count-6 pin: split the plain and `_cm740_driven` functions and assert 3 increments each, no
  decrement/assignment. Layout literals become syn-based or whitespace-normalized argument
  checks, each with an argument that it accepts the wrapper form and still rejects an argument
  change. State the strength relation to today's check (equal or stronger; weaker is a stop).
- **named stop** — only with a standing authority.

Each re-derived check ships a falsifier: a scratch-map mutation (a synthetic site added,
removed, or relocated; an argument reordered) that makes it red with a named variant,
calibrated red and green. Keep the error precedence `ClosedCallers` → `PreparedSkip` →
`Closure`, the two P0496 controls, and the 15-control count (dropping a control is a stop).
Give the overlay plan: preimage `a3092419…` bytes unchanged on live/baseline/real;
armed/disabled carry the re-derived `proof_controls.rs` (one byte-identical file on both);
the overlay row's cells are recomputed from bytes by the stage-C writer, never hand-typed.
`proof_tests.rs` and `proofs.rs` stay unchanged unless stage C proves a required change; a
required change there is its own drafted step with a falsifier.

## Stage E — the drafted continuation brief 786 (300 s)

Draft brief 786 in this template's shape: (1) apply the stage-D dispositions to the
armed/disabled `proof_controls.rs` only (the exact new file content as code); (2) run the
falsifiers calibrated in D; (3) then brief 784's stages B–D on the carried images unchanged —
the four per-tree target dirs; the host-payload generators by full module path; the population
campaign with `--nocapture` on armed and disabled only (never baseline); F13 on
baseline/armed/disabled/live; `local_production_acceptance` in both modes on each; the writers
in order (overlays 147 + the unconditional 13 + conditionals, additions 9, chain 152 recomputed
from bytes, the class table's 201 `cases.rs` cells, mutants `source_schema`, the host-payload
tsvs and `reconcile.rs` pins, the allowance verified); the four rebuilt images; the twelve
receipts with `rows=9`; the six-case gate; `cargo fmt --check`; the default-mode workspace with
the nine skips and exactly the ten red names (the nine loopback refusals plus
`q25_p2379_ownership_proof`; any other red stops); the 71 ignored set-identical; the flag-on
`5 passed; 0 failed` and the class-lease fixtures; the F16 cleanup and the final pins. State
the forecast.

## Checkpointing and stops

Write `.temp/brief-785/PROGRESS.md` and the stage log after each stage. Stop, with the exact
evidence, on: any pin drift in the entry bar; a scan that does not reproduce the five failing
checks; a check you cannot state semantically; an unattributable occurrence; a proposed
disposition that is weaker than today's check, tautological (expected from the certified
expression), or that drops a control; a required change outside `proof_controls.rs` without
its own drafted step; any live edit; any need for network, a new dependency or `unsafe`.
A stop is a correct terminal state.

## Hard rules

No git in any form. No network. No new dependency. No `unsafe`. No committed-file edit. No
sealed-artifact edit. No hand edit of a derived file. No CM5. No 744/745/746. Q25's thirteen
rows stay open. `.temp` only; never `/tmp`; never read `~/.config/ccds/api-key`. One
`CARGO_TARGET_DIR` per tree, always under `.temp/brief-785/`; no baseline campaign.

## Budget

Focused **3,600 s** (A 300 / B 900 / C 1,200 / D 900 / E 300) + workspace **0** + epoch **0**,
carved after brief 784 (charged 3,014.104 s of its 10,200 s authorization and returned
7,185.896 s; reserve 7,223.49478847017 s) → reserve after the carve
**3,623.49478847017 s**. Unspent returns; a projected overrun stops for a director ruling,
never a scope cut.

## Report format

`rum-report/1` at `.temp/brief-785/report.json`, stage `A-E / c144`; `summary`, `changes`
(none expected outside `.temp/brief-785/`), `disclosures`, `deviations`, `questions`, checks
and measurements with every verdict line quoted, and the final message
`scripts/report.py summary`. A DONE report has no FAIL row. The director gates from raw
evidence.
