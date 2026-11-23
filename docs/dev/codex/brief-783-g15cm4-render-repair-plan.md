# Brief 783 — G15-CM4 Stage D repair: the deterministic render — the plan completion and the measured switch control (design-only)

**Design-only brief. `Scope: NONE`.** It continues ruling 791 (gate c141) and brief 782
(`2b6b4915`). Brief 782 stopped correctly in its stage D: `lp03_adopted_phases_are_exact_in_
completed_and_resumed_runs` has no inventory and no ignored writer, because the five
LP-family reds are by contract red without `RUM_PROFILE_LEXICAL_SUBSTITUTION` — the seven
documented skips of the project's standing acceptance suite (brief 739, repeated 701–729 and
in brief 743's CM4 acceptance), five of them closed by a flag-on subset run, `lp655`/`lp656`
the disclosed standing two-arm exclusions. My stage-D bar had excluded only the nine loopback
refusals (F19). This brief completes the plan the render repair needs, with the corrected
classes, and measures the seven-class on the install tree instead of citing it.

## Goal

When this brief is done, in `.temp/brief-783/` only:

- **The red universe fixed and classified.** The union of the 776 and 780 workspace reds
  (`.temp/brief-776/logs/stage-d-workspace.log`,
  `.temp/brief-780/logs/stage-d-workspace.log`) re-derived by you from the `failures:`
  blocks, not from `... FAILED` lines (the 780 lp03 line is mangled by interleaved output:
  `rum-profile logical epoch=test lp03_… … 2FAILED valid=false reason=`). Expected: 72 and
  74 unique names, intersection 71, **union 75** = nine loopback + seven documented
  exclusions + **59 to disposition**, where the 59 = 56 in-776 non-excluded names + the three
  new F13 contract reds (`local_concrete_contract_controls`,
  `local_option_read_claims_match_stable_rustc`, `local_contract_falsifiers`), minus
  `cm_mutation_source_authentication_controls` (green in 780). Any different count: state
  yours with the exact evidence and reconcile against mine; an unreconciled difference is a
  stop.

- **The 59 dispositions, zero unknown, grouped by owning inventory.** For each name, exactly
  one class, with source evidence:
  1. **inventory regeneration** — the owning file(s), the ignored generator or script (its
     path and exact command), the review copy, the adoption target(s), the falsifier that
     proves the regeneration, and its position in the DAG;
  2. **repair** — the exact site and the minimal change, inside the authorized scope (only
     F14 is a live edit; the rest are composed-tree or generator changes), with its falsifier;
  3. **named stop** — only with a **standing authority** (an existing ruling or brief that
     names the obligation and keeps it open, e.g. Q25's thirteen rows for
     `q25_p2379_proof::q25_p2379_ownership_proof`, whose 780 panic is
     `Q25_SELECTOR fail-closed got=Err(NotPresent) want=Ok("P2379")` at
     `q25_p2379_proof.rs:605`);
  4. **excluded class** — only the nine loopback and the seven documented names, and only
     with the citation.
  A name you cannot place in one of these four from source is a stop, not a guess. A new
  waiver is never a disposition.

- **F5's complete inventory and full schema footprint**, with a regeneration command and a
  falsifier for every affected artifact. Answer explicitly, quoting source lines:
  - the target file under a `tests/` path (the hygiene scan skips `/tests/` and `/bin/`) and
    its declaration mechanics: the composed `lib.rs` carries the module inside an inline
    module, the moved file's module path, the two `use` statements (`use super::*;`,
    `use std::sync::Arc;`), the new test name, and every standing reference to the old name
    (the 9 hits in 5,420 files: brief 782, brief 743 ×3, brief 623 ×2, `TODO.md` ×3);
  - the composer artifacts and the anchor: `route_a.py` (`MODULE_SRC = … "descendant.rs.in"`
    line 3; the `lib.rs insert route_a_module` anchor line 72) and whether the new file's
    bytes also derive from `descendant.rs.in`; the 768/769 re-anchoring and allowance
    generator's role;
  - the three pinned cells that shift with the module's host lines (`BOUND_LIVE` = `lib.rs`
    496, `OPERATIONS` = `lib.rs` 3103, `AROUND_HANDOFF` = `q25_ownership.rs` 376), and every
    consumer of those pins;
  - the full footprint: does the new file enter `additions.tsv` (8 → 9) and what is its
    digest source; does `lib.rs` stay an overlay row (147) or does a declaring file become
    one; does `emission_chain.tsv` grow (152 → 153) or keep 152; what does the class table
    do; what does `mutants.tsv`'s `source_schema` require; what does the verifier's
    `copied == baseline ⊎ additions` rule demand; the six receipts' row tables; the arm
    bijection; the six-case source-gate evidence. A hand edit of any derived file is a stop.

- **F13, F14 and the Q25 formatting as exact recipes.** F13:
  `regenerate_local_contract_fingerprints` (`contract_tests.rs:8`, writing
  `target/brief-544/local-contracts.tsv` at 25-27, every field except the last), the
  `fingerprint` walk (`contracts.rs:88/99`, extensions `rs`/`py`/`toml` at 106), the exact
  command and inputs on the final tree, the adoption on both sides (`local-contracts.tsv`
  `6b390887…` live vs `ead1d6de…` install; `promoted.tsv` `03ef55aa…` vs `4337950c…`), the
  ordering after F5, Q25's formatting and F14, and the three falsifiers
  (`local_concrete_contract_controls`, `local_option_read_claims_match_stable_rustc`,
  `local_contract_falsifiers`) with the transition each is watched across. Same for
  `promoted.tsv` with its own generator and consumers. F14: the one-line hermeticity fix at
  `observer_policy_tests.rs:480-482` (create the symlink's parent), its falsifier (passes in
  a tree without `.temp/brief-743/`), its control (the tree-with-link case still passes), its
  blast radius. Q25: the exact rustfmt invocation for
  `crates/rum-runtime/src/cm740_probe/q25_ownership.rs` and
  `crates/rum-stdlib/tests/suspension_oracle/q25_p2379_proof.rs`, the scene mirroring, and
  the order before the fingerprint regeneration.

- **The measured switch control on the install tree.** Run exactly these two commands on
  `.temp/brief-779/install` (byte-pinned `e01d13cf…` / 5,850 files), with
  `CARGO_TARGET_DIR` under `.temp/brief-783/target`, `TMPDIR` under `.temp`,
  `CARGO_NET_OFFLINE=true`, `RUST_TEST_THREADS=8`:
  1. flag-off: `cargo test -p rum-host --test lp03_phase_rows --test
     lp04_definition_rows --test lp05_assignment_rows --no-fail-fast` — bar: exactly
     `0 passed; 5 failed` and the five names (`lp03_adopted_phases_…`, `lp658_host_eval_reset`,
     `lp658_host_resume_reset`, `lp04_adopted_definition_rows_…`, `lp05_adopted_assignment_
     rows_…`);
  2. flag-on: the same command under `RUM_PROFILE_LEXICAL_SUBSTITUTION=1` — bar: exactly
     `5 passed; 0 failed`.
  Any other outcome: stop with the log; it is a render finding, not a bar to move. Neither
  command may write into the install tree: remove any `target/` or `.temp` scratch the run
  creates there, re-assert `e01d13cf…`/5,850 and debris 0 after, and disclose.

- **The deterministic DAG of the whole repair**, with the invalidation rule (any source edit
  after a regeneration invalidates it) and no cycle: every live-side edit (only F14), every
  composed-tree edit (F5, Q25), every regeneration (F13, `promoted.tsv`, the class table, the
  adopted sync), the 768 re-anchoring, the 769 allowance certificate, the trees' rebuild, the
  six-case source gate, the workspace, and `cargo fmt --check`.

- **The final-tree run book**: the final armed image under `RUM_CM740_BASELINE` = the rebuilt
  baseline; the **nine** flag-off skips (the standing seven —
  `lp03_adopted_phases_are_exact_in_completed_and_resumed_runs`,
  `lp04_adopted_definition_rows_are_exact_in_completed_and_resumed_runs`,
  `lp05_adopted_assignment_rows_are_exact_in_completed_and_resumed_runs`,
  `lp658_host_eval_reset`, `lp658_host_resume_reset`, `lp655_root_context_falsifier`,
  `lp656_public_root_context_falsifier` — plus `class_lease_positive_c5_default_profiled_
  finish` and `class_lease_positive_p0492`); the flag-on subset (the three LP binaries 5/5
  and the class-lease fixture suite); the nine loopback tests green on the host; the red bar
  after the 59 close (the only remaining names are the named stops; the Q25 p2379 proof is
  one); the ignored bar (the 71 reference names, set-identical); the flags, the external
  `CARGO_TARGET_DIR`, `RUST_TEST_THREADS=8` with `--no-fail-fast` as a cargo option (before
  `--`), the F16 scratch-removal and re-assert duty, and the final pins. State why this is
  the project's own suite (brief 739's acceptance clause; brief 743's CM4 acceptance "the
  seven exact skips and the nine host-only" tests) and not a new bar.

- **A drafted implementation brief 784** in `TEMPLATE.md` shape, with stages, component
  budgets, the exact bars, the stops, and a realistic forecast with the basis for each
  number.

Every open question gets a recommendation; none is left to the implementer to guess.

## Context pointers

- Ruling `.temp/director/ruling-791.md`; brief 782 and its report; rulings 790/789.
- The composed postimages and composer: `.temp/brief-749/attempt-4/generator/brief-743/
  stage-a/` (`route_a.py`, `descendant.rs.in`), brief 743's record in
  `docs/dev/codex/brief-743-g15cm4-observation-policy.md`.
- The tables and generators: `.temp/brief-779/install` (`overlays.tsv`, `mutants.tsv`,
  `emission_chain.tsv`, `additions.tsv`, `class-provenance.tsv`), briefs 768/769, brief 777's
  `stage_a.py` (the twelve `ADOPTED` paths, lines 86–99).
- The workspace evidence: `.temp/brief-776/logs/stage-d-workspace.log`,
  `.temp/brief-780/logs/stage-d-workspace.log`, `.temp/brief-780/logs/stage-d.log`,
  `.temp/brief-782/logs/` (stage-a, stage-b, stage-map, stage-d, stage-d-stop); the retention
  contract source (`crates/rum-stdlib/src/tests/retention_closure/local/contracts.rs`,
  `contract_tests.rs`); the `source_auth` test
  (`crates/rum-stdlib/tests/suspension_oracle/mutations/source_auth/observer_policy_tests.rs`);
  the LP-family tests (`crates/rum-host/tests/lp03_phase_rows.rs`, `lp04_definition_rows.rs`,
  `lp05_assignment_rows.rs`) and the profiling gate
  (`crates/rum-runtime/src/runtime/profiling/{timing,config,state}.rs`).
- The standing acceptance suite: brief 739's "Acceptance suite" clause; briefs 701–729;
  brief 743's CM4 acceptance record.
- The six-case gate and the receipts: brief 779's stage-C procedure and
  `.temp/brief-779/receipts/`.

## Scope

- You MAY write only under `.temp/brief-783/`: scripts, logs, the plan, the drafted brief,
  the report. The live tree and every `.temp/brief-7xx`/`78x` tree and every sealed artifact
  are read-only.
- **No source edits.** No cargo except the two commands of the measured switch control, on
  the install tree, with an external `CARGO_TARGET_DIR`. No git, no network, no new
  dependency. Never edit a copied tool or a derived file by hand.

## Stage A — entry and the red universe (300 s)

Re-authenticate the ruling 791 entry: the five trees' pins, the four tables, the class table,
the 147-row control, the tool copies; read the live status honestly from my record (the 23
inherited entries; no git). Re-derive the 75-name universe from the `failures:` blocks and
classify it: nine loopback, seven documented exclusions, 59 to disposition. List every name
once, with its class.

## Stage B — F5's footprint (1,200 s)

The complete inventory and schema footprint as above, every question answered with quoted
lines and a regeneration command plus falsifier per affected artifact.

## Stage C — F13, F14, Q25 (700 s)

The three recipes, their ordering among themselves and against F5, and their falsifiers, as
above.

## Stage D — the 59 dispositions (1,800 s)

Every name mapped with zero unknown, grouped by inventory; the total distinct inventory files
touched; which need a repair and which are named stops with their standing authority. A name
with no source-derived disposition stops the brief.

## Stage E — the DAG, the run book, the draft (500 s)

The deterministic order and the final-tree run book; the drafted brief 784; the report. A
per-stage forecast for 784 (focused and workspace) with the basis for each number.

## Stage W — the measured switch control (2,500 s)

The two commands of the install tree, their exact bars, the tree-safety duties, and the
disclosure. A deviation from either bar stops the brief with the log.

## Verification and acceptance

The brief passes only with: the 75-name universe and the 59 dispositions with zero unknown;
F5's full inventory and schema footprint with commands and falsifiers; F13/F14/Q25 as exact
recipes with their falsifiers; the DAG with the invalidation rule and no cycle; the run book
with the nine skips, the flag-on subset and the red bar; the drafted 784 with its forecast;
the measured control at both bars; every verdict line quoted verbatim; every open question
answered with a recommendation.

## Checkpointing and stops

Checkpoint after every stage (`.temp/brief-783/PROGRESS.md`, logs under
`.temp/brief-783/logs/`). Stop, with the exact evidence, on: an entry pin mismatch; any of the
59 without a source-derived disposition; a disposition that edits the live tree beyond F14; a
named stop without a standing authority; a reference to the F5 test name you cannot
enumerate; a cycle in the DAG; any proposed hand edit of a derived file; any need to edit a
sealed artifact; the measured control missing either bar; a projected overrun; or any need
for network, a new dependency or `unsafe`. A stop is a correct terminal state.

## Hard rules

Design-only (`Scope: NONE`): no source edits; cargo only the two measured commands on the
install tree; no git of any kind; no network; no new dependency; no `unsafe`; `.temp` only;
never `/tmp`; never read `~/.config/ccds/api-key`; never edit a copied tool or a sealed
artifact; Q25's thirteen rows stay named stops; 744/745/746 stay held; CM5 stays
unauthorized.

## Budget

Focused **4,500 s** (A 300 / B 1,200 / C 700 / D 1,800 / E 500) + workspace **2,500 s** +
epoch 0 = **7,000 s**, carved after brief 782 (reserve 3,700.50078847017 s, with an explicit
top-up of 3,500 s from the campaign allocation, user ruling 2026-11-19) → reserve after the
carve **200.50078847017 s**. Brief 782 charged 1,968.4 s of its 3,600 and returned 1,631.6.
Unspent returns; a projected overrun stops for a director ruling, never a scope cut.

## Report format

`rum-report/1` at `.temp/brief-783/report.json`, appended as stage `A / c142`; `summary`,
`changes`, `deviations`, `questions`, every verdict line quoted, and the final message
`scripts/report.py summary`. The director gates from raw evidence.
