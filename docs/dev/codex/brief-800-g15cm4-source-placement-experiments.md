# Brief 800 — G15-CM4: the source-placement experiment campaign

Status: experiment/diagnostic brief — `Scope: NONE` on the live tree; every write goes under
`.temp/brief-800/`; the sealed role images and every earlier brief's evidence are read-only.
Dispatched after ruling 832 (gate c180). Brief 799's design obligations are discharged except
Stage D.1, which needs measured prices; this brief produces them. It settles four decision points
with isolated same-path, one-codegen-unit builds and deterministic instruction-count
measurements: **(1)** which of four evaluator placements sheds the executed flag-address class
(+14,124,585 Ir; 24 executed ENABLED LEAs against 2 in each control) and at what measured price,
with the enabled/deferred/reset falsifiers preserved; **(2)** the source classification of the ten
machine-priced rows left after row 1056 (together +830,693 / +924; with row 1056 the open mass is
+900,753); **(3)** whether the 13 outlined bodies' cold-twin call boundaries are avoidable and at
what measured price (the two proved helpers +8,972,494 / +6,273,782; the ten new-only bodies
+5,063,036 gross / +1,821,580 net with their callers; `apply_replace_all` +112,032); **(4)**
whether row 1056's five non-gate instructions per visit are required class-form work or avoidable
placement. No live source change, no acceptance suite, no epoch, no timing or hardware decision.
Carve: **A 250 / B 800 / C 3,000 / D 500 / E 4,200 / F 1,400 / G 450 = 10,600 s**, funded by brief
799's returned 1,432.0 s and a new explicit 9,168.0 s allocation (user ruling 2026-11-19
「预算不够就追加吧」; ruling 832).

## Goal

For each experiment cell: one measured price (in instructions, against that role's own reference
built at the same absolute path with the same flags), one emitted-text/symbol verdict, and one
verbatim falsifier outcome — so the director can write the repair plan with measured numbers
instead of opportunities. A stage that cannot settle its question STOPs with the exact missing
measurement; a STOP in one stage does not block a later stage. The deliverable is evidence, not a
code change, and **no measured price is a promise**: candidates are measured individually and are
never summed into a predicted saving.

## Context pointers

- Read `AGENTS.md` (hard lines) and the brief-799 report's D.0/B.4r sections before forming a
  claim: `python3 -I scripts/report.py show 799 --tag evaluator`, `--tag call-edges`,
  `--tag family-inventory`, `--tag qualification`, `--tag bar`.
- **Sealed inputs and pins (read-only; verify, do not re-render):**
  - `.temp/brief-798/report.json` SHA-256
    `f281efa552833287103ec247cc3928b9a07d21a22803df4a5040f09721add08e`; `report.py check 798`.
  - ELFs: base `06d0d7b4254f87379a33939d3f7298b9fd9b2e30dd97c30f5bf054dce5453e09`, new
    `2348c4685dbafbd452c1f239e29bc4433d87ca348cfc05788fd27b1e677dbecf`, control
    `2e0927716fd8822bafd7d99acf354e76f5b77c46ccb15aaa5c58de07c9f07998`.
  - Role trees (file trees, 5,851 files unless noted): real `022d60af…`, armed `318933db…`,
    control tree `94c8f966…` (real + the 795 one-file layout patch), baseline `ec602109…` (5,842;
    no class form — not a build role here).
  - The build recipe of record: `.temp/brief-798/evidence/amendment2/build-new.json` (cwd
    `.temp/brief-798/active`; `CARGO_HOME` private; `CARGO_TARGET_DIR` external; rust 1.85.0
    ahead on PATH; `cargo build --offline --locked -j 8 --release -p rum-cli --bin rum-cli`;
    `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`; 88.942 s, 9 GiB peak).
  - The canonical program `.temp/brief-795/panel/canonical/core-language-microbench.rum` and its
    pin `canonical` in `.temp/brief-798/evidence/amendment3/capture-plan.json`; the functional
    program's pin `functional` in the same file; the 14 `RUM_PROFILE_*` flags all `0`
    (`profile_off` in that file); `RUM_CORE_MICRO_RETAINED_MODULE_GC_BATCHES=3`.
  - The accepted harness to copy and adapt (read-only): `.temp/brief-798/tools/` —
    `a3_capture.py` (the Callgrind command and the pinned env), `a3_entry.py` (the whole-panel
    job), `a3_whole.py` (the body/family reducer), `a3_localize.py` (the PC/edge joins),
    `a2_common.py`/`instruments.py` (paths, pins, env). The 799 evidence to reuse:
    `.temp/brief-799/a4-flag-pcs.tsv`, `a4-helper-prices.tsv`, `a4-b4-cut.log`,
    `a4-b4-separated-joins.log`, `a4-e-inventory.log`, `a4-c-accounting.log`.
- **The experiments' source declarations (the only files any variant may touch, in the staged
  copy):** the evaluator/state files named by the four candidates below; the 13 bodies at the
  `armed_decl`/`base_decl` cells of `a4-helper-prices.tsv`; and for row 1056
  `crates/rum-runtime/src/runtime/definitions/definition_catalog/rewrite_rules.rs` plus
  `crates/rum-runtime/src/runtime/options/core.rs` (the
  `system_option_bool`/`system_option_bool_cm740_driven` pair) and the cold twin at
  `rewrite_rules.rs:238–250`.

## Scope

- You MAY write only under `.temp/brief-800/`: `tools/`, `active/` (the one build path),
  `target/` (the one external target and private CARGO_HOME), `bins/`, `callgrind/`, `logs/`,
  `evidence/`, `report.json`, `PROGRESS.md`.
- Read-only and immutable: `.temp/brief-797/` (the four role images and the render/install
  evidence), `.temp/brief-798/` (the signed report, ELFs, tables, tools, logs),
  `.temp/brief-799/` (all evidence), `.temp/director/`. Verify their hashes at entry and exit.
- The live tree is untouched: no edit under `crates/`, `docs/`, `scripts/`; zero builds in the
  live tree; `git status` clean at exit; every build happens in `.temp/brief-800/active` with the
  external target.
- `.temp` only, never `/tmp`. No network. No new dependency. No new `unsafe` in
  `rum-core`/`rum-runtime`. No `git` command of any kind.

## Stage A — entry and pins (250 s)

Verify and index read-only: the 798 report hash and `report.py check 798`; the three ELF hashes;
the four role tree hashes and file counts; the 799 report's structure and its verdict lines; the
canonical/functional program pins and the 14 profile flags; the machine (CPU 14 identity, valgrind,
perf, the 1.85.0 toolchain, RAM); and the live tree's 5,876-file metadata manifest. Emit
`.temp/brief-800/evidence/entry-index.json` (new files hashed; sealed files cite the receipts'
hashes). STOP on any mismatch.

## Stage B — staging and the three references (800 s)

One source path `.temp/brief-800/active`, one external target `.temp/brief-800/target/build`, a
private `.temp/brief-800/target/offline-home`, rust 1.85.0 ahead on PATH, exactly the recipe of
record, one cargo command at a time, `-j 8`, `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`. For each
arm: copy the complete sealed tree byte-for-byte (excludes `.git`, `.temp`, `target`,
`__pycache__`, `.mypy_cache`), re-hash it, delete the previous `active`/`target` as the recipe
does, build, copy the ELF to `.temp/brief-800/bins/<arm>`, and save a receipt (tree hash,
rustc/cargo identity, env, argv, cwd, exit, elapsed, ELF sha256, stdout/stderr).

1. **base** ← real `022d60af…`.
2. **new** ← armed `318933db…`; require the ELF `.text` sha256 and the executed-PC/body-sum
   receipts to reproduce the 798 `new` receipts; an ELF-level difference confined to
   path-dependent bytes is recorded; a `.text` difference is a STOP for a comparability ruling.
3. **control** ← real + the 795 one-file layout patch; reproduce tree `94c8f966…` exactly (inspect
   the one changed file: `crates/rum-runtime/src/runtime/profiling/timing.rs`, one cold
   `black_box` reference, the perturbator retained); a different tree is a STOP.

Then measure the two references' off-state whole panels (Stage C's method) and record their
totals and body tables — these are the pairing baselines for every cell. **The historical
numbers are the map, not the bar**: the body table must have 1,743 body rows and 93 families with
`family_unassigned=0`, and the new-vs-base deltas must be in the historical neighborhood (name any
family whose sign flips); the experiment's prices are always variant-vs-its-own-reference.

## Stage C — the evaluator placements (3,000 s)

One cell at a time, serially, at the one path: stage the edit on the role's tree, build, save the
ELF and its `.text`, extract the flag-class PC table (static PCs, destinations, executed
multiplicities, disassembly lines — the `a4-flag-pcs.tsv` shape), profile the whole panel, reduce,
and run the battery. Each cell is paired with its own role reference.

**The measurement** (all cells): the canonical whole-panel Callgrind run, exactly
`taskset -c 14 valgrind --tool=callgrind --dump-instr=yes --vgdb=no --enable-debuginfod=no
--log-file=<vg> --callgrind-out-file=<raw> <elf> --file <canonical>`, cwd the repo root, stdin
`/dev/null`, env `ordinary_env() ∪ profile_off() ∪ {DEBUGINFOD_URLS:""}` with all 14
`RUM_PROFILE_*` = `0` and `RUM_CORE_MICRO_RETAINED_MODULE_GC_BATCHES=3`; the 798 reducer adapted to
the new paths. Report: the whole total delta, body rows 1115/1116, the flag-class counts and
per-PC execution, the four StringReplace short-label per-iteration prices, and the caller-inclusive
deltas. Instruction counts are deterministic; no interference qualification is needed for them —
but every run is serial and alone, and every raw file is retained before parsing.

**The cells.**

- **E1.0 identity.** Build armed a second time; require byte-identical `.text` to the Stage B
  reference and a zero measured delta. Targets zero of 57 flag PCs.
- **E1.1 fast-path at `lexical_profile_configured_admitted`.** Read the same atomic once at that
  exact hook: return false on 2, true on 1, take the cold fallback on 0. Keep test lease
  admission ahead of access. Inspect all 29 ENABLED sites for duplicated address
  materialization and the 28 ACTIVE sites for unintended reachability.
- **E1.2 cold out-of-line work.** At the timing veneer keep an inline configured-disabled exit;
  funnel the initialized-on/unknown path into one `#[cold] #[inline(never)]` routine that
  commissions or reads ACTIVE using the current lease and performs the existing operation.
  Preserve state-0 initialization and the on → active/deferred semantics exactly once. A call on
  every disabled hook violates the fast-path bar; moving instructions into a callee counts only
  if combined executed Ir falls.
- **E1.3 fold the address, preserve the mutable byte.** Bind a private constant reference to the
  existing static (`&'static AtomicU8`) in the comparison experiment and use that reference at
  configured reads; each relaxed state load and initialization stays at its original site. A
  permanently false atomic/feature is invalid (it drops the enabled rows). An identical code
  image prices this candidate at zero.
- **E1.4 hoist only across a proven invariant interval.** Experiment with threading the configured
  flag address from `evaluate_inner_unstable` into its inline timing veneers. A specialization on
  configured-off may be selected at entry only after proving the mode immutable across that
  interval in every supported lease/reset path. Keep configured-on reading ACTIVE at each original
  start and completion and reselect on reset, callback and resume boundaries. Do not cache
  "active" across user evaluation. If the proof fails, test address hoisting alone.

Each candidate is staged on **both roles**, one at a time, each paired with its role reference:
on `real` the battery is the off-state canonical output equality and the publication-gate census;
on `armed` the full battery below.

**The armed battery (every candidate, including the identity):**

1. Off-state equality: the canonical 120-label program normalizes clock lines and is byte-equal
   in non-clock output and label order vs the armed reference; the six functional runs likewise;
   all `RUM_PROFILE_*` = 0.
2. Enabled/deferred/reset: fresh processes with `RUM_PROFILE_LEXICAL_SUBSTITUTION=1` and
   `DEFER=0`, and with `DEFER=1` around `RumProfileReset[]`; identical enabled phase keys,
   counts, order, completion/abandonment records and post-reset/active transitions (compare
   event structure, never nondeterministic durations).
3. `cargo test -p rum-stdlib --test suspension_oracle -- logical` at `RUST_TEST_THREADS=8` in the
   staged tree; quote its `test result:` line and the LP03/LP04/LP05 lines; require the same
   pass/fail counts as the reference run and at least one selected test (zero selected is a STOP,
   not a re-selection).
4. The permanently-off negative control in an isolated diagnostic: the enabled tests must reject
   a permanently-false flag.
5. The publication gate: 38 reached sites, 366,893 visits, 1,464,055 Ir, `exclusive_Ir_per_visit=4`
   at `guarded/replace.rs:38` and `:775`, typed 4+3, zero eliminated, zero slow/lexical self Ir,
   perturbator 0, the short-profile 4-Ir site cap.
6. The driven ledger: 26 ordered lines / 1,741 bytes, SHA-256
   `a2c48c90fab83d49ffceb07e8c74992268fea353cf5e03574f8cbe63435d3895`, the real P0256 hit, the
   12 routes, balanced owners, the mid-request Print sequence (`entry=false,
   callback_1_before=false, callback_1_after=true, callback_2=true, after_end=false, output=42`)
   with both reached StringReplace edges, and resumed == queued in rum.

A failed falsifier is recorded `FAIL` and that candidate cannot be a repair. A cell whose text is
identical to the reference but whose measured delta is nonzero is a measurement fault — STOP,
name it. Print per cell, verbatim in the log:

```
EVAL role=<real|armed> candidate=<identity|fastpath|coldout|foldaddr|hoist> text=<identical|changed:syms> whole_delta=<n> body1115_delta=<n> body1116_delta=<n> flag_static=<n> flag_executed=<n> flag_class_delta=<n> prices=<p1,p2,p3,p4> falsifiers=<PASS|FAIL:name>
```

## Stage D — the ten-row source classification (500 s)

Classify, from the source trees and the retained machine bills, the ten rows the STOP left
priced: `1059` resolve_cases_traversal_owned (+10,575), `1143` try_dispatch_builtin_call'2
(+541,202/+913), `1202` eval_control_data_builtin'2 (+34,722/+11), `1205`
resume_compound_expression_from_index (+24,753), `1206` resume_compound_expression_from_index'2
(+85,905), `1245` finish_cases_with_traversal (+25,592), `1249`
finish_delete_cases_at_current_node (−107,216), `1253` resume_match_q_after_value (+168,152),
`1465` eval_input_with_terminated_handling (+33,004), `1615`
replace_string_value_with_guarded_rules (+14,004). For each: the exact source (`path:line`), the
incoming/outgoing edge census, the per-PC class deltas from `a4-b4-separated-joins.log`, the
mechanism (construct pair / new-only body / required class-form / avoidable placement / caller
movement), and the disposition. STOP on any row that cannot be assigned, naming the missing
measurement; the STOP does not block Stages E–F. Print:

```
TENROWS assigned=<n>/10 stop=<none|row> mass=<n>
```

## Stage E — the outlined-body placements (4,200 s)

The 13 bodies, at the declarations in `a4-helper-prices.tsv` (`row | armed_decl`):
`1034` `definition_analysis_helpers.rs:254`; `1383` `pattern_matching/core.rs:1085`; `1038`
`definition_analysis_helpers.rs:485`; `1276` `structure_data/replace/guarded_plan.rs:4`; `1618`
`strings/patterns/search/guarded/replace.rs:751`; `1263` `rewrite_data/replace/once.rs:99`; `1083`
`definition_updates.rs:382`; `1184` `association_data.rs:846`; `1086` `definition_updates.rs:976`;
`1199` `association_lvalue.rs:7`; `1185` `association_data.rs:878`; `1110`
`value_table_assignment.rs:109`; `1014` `application/replace_all.rs:83` (all under
`.temp/brief-797/amendment6/images/armed/crates/`). Out of scope and named: the adapter pair
(`1147`/`1448`; its two pairs cannot independently spend the adapter delta, net −43,841) and the
drop pair (±6,361,600, combined zero).

For each body, two cells in order: `#[inline]` at the declaration; if the standalone symbol
survives, `#[inline(always)]`. Each cell is built on both roles, one at a time at the one path,
and paired with that role's reference. Record: the emitted symbol's presence, the body's visits
and self Ir, the combined caller+helper self (disjoint groups only — rows 1184/1185/1199 share
caller 1186 and their individual pair sums must never be added: report `OUTLINE group=1186
disjoint_delta=<n>` once), the absolute process total delta, and the affected separated rows' new
deltas. On `real`, if the variant's ELF is byte-identical to the real reference, record the
identity and skip its profile.

The battery per armed cell: the off-state canonical output equality and six functional runs; the
publication-gate census and four short-label prices; the driven ledger, typed 4+3, P0256, owners,
callback sequence, resumed == queued; the cold twin still reachable and every materialized copy's
reachability preserved; the original operations in their original order. **Do not remove or
duplicate a semantic body to obtain a symbol-table result.** A cell that changes an observable row
is a STOP for that body, naming the change; it is not a repair. Print per cell:

```
OUTLINE row=<n> role=<real|armed> cell=<inline|always> symbol=<present|gone> body_self_delta=<n> combined_delta=<n> process_delta=<n> falsifiers=<PASS|FAIL:name>
```

## Stage F — the row-1056 2×2 (1,400 s)

Four armed cells, one at a time at the one path, paired with the armed reference (cell A, the
current armed layout):

| cell | driven caller | driven option branch |
|---|---|---|
| A | shares the ordinary resume implementation (reference) | inlined in the ordinary path |
| B | shares the ordinary resume implementation | behind a cold boundary, sharing the ordinary argument construction |
| C | private equivalent cold copy of the resume path | inlined in the ordinary path |
| D | private equivalent cold copy of the resume path | behind a cold boundary |

Hold the publication read at the same logical point in all four; verify each cell's emitted graph
against its intended factor (call edges: which function calls which resume/option reader). Record
the inputs, 14,012 visits, all child call counts, the four publication gate PCs (re-anchored per
cell; the reference has them at 0x96fd91), disjoint per-PC Ir, caller+callee totals, the stack
frame size, and each cell's disassembly for the three call-delimited regions. Compiler-internal
reports are optional isolated companions; they are accepted only if they reproduce the measured
text bytes/normalized PC sequences. Test published/ordinary, driven, callback, suspension,
resumed/queued, capture order and typed 4+3 before interpreting any cost. If an otherwise
equivalent armed cell reproduces the ordinary placement (213 non-gate instructions per visit) and
sheds the cost, it prices that boundary; a cell that changes observable behavior, folds a
required gate, merges the planned variants or changes visit multiplicity cannot settle it. If the
contrasts still fail to separate, retain the STOP. No counterfactual saving or native attribution
follows from the five instructions. Print:

```
ROW1056 cell=<A|B|C|D> non_gate_per_visit=<n> delta=<n> frame=<0x...> graph=<verified|not> verdict=<prices|required|avoidable|cannot_settle>
ROW1056 STOP retained=<true|false>
```

## Stage G — synthesis and handoff (450 s)

One table and one report section per family, each price carrying its falsifier outcome; the
class-form-required remainder stated separately; `additive_savings_claimed=false` verbatim. The
updated classification ledger if Stage D completed (expected `89 individually assigned, 22
carried, 1,631 aggregate, 1 STOP` = 1,743; if Stage D STOPped, state the exact open count). The
live-tree metadata comparison (5,876 paths), `git status`, and the report check. Print:

```
SYNTHESIS eval_prices=<n> outline_prices=<n> row1056=<verdict> ledger=<a/b/c/d> additive_claimed=false
```

## Verification (MANDATORY — you run it; iterate to evidence)

This is a diagnostic/experiment brief: the named artifacts and the verbatim printed lines ARE the
deliverable. Every run's full output is saved under `.temp/brief-800/logs/` before parsing; every
raw Callgrind file is retained; every falsifier line is quoted, not paraphrased. Run
`python3 -B scripts/run-bounded.py --timeout 20 -- python3 scripts/report.py check 800` before
sealing and fix every PROBLEM. Do not run the workspace suite — there is no live change.

## Pitfalls that apply here

- **One variable per cell.** One source edit per variant; the reference is the same role built at
  the same path with the same flags. An experiment that changes two things cannot price either.
- **Deterministic counts, not timings.** All decisions come from Callgrind instruction counts
  (exact for a fixed binary and input) and from verbatim functional output. No wall-clock, no
  `perf`, no hardware counter is a decision input in this brief.
- **The build recipe is the measurement.** Same absolute path, same flags, one codegen unit,
  private CARGO_HOME, offline, one cargo command at a time, `-j 8`; the 798 recipe of record is
  normative down to the argv and env.
- **A path-dependent byte difference is recorded, not discarded.** The ELF SHA may differ from the
  sealed receipts (embedded path strings); the `.text` digest and the executed-PC/body-sum
  receipts are the comparability bar.
- **Do not repair anything.** No source edit outside the staged copies; no live-tree change; no
  Cargo.toml, lockfile, formatter or docs edit.
- **Suspension safety.** If an edit touches an `eval_*`/`resume_*` path, the continuation shape,
  publication points and resume witnesses must stay observable; a cell that changes the order or
  multiplicity of an observable event is a STOP for that cell.
- **Re-entry safety.** Completed cells are not rerun after later stages consume them; a resumed
  block skips completed cells on their receipts; the report lists every attempt including passes.
- **Checkpointing.** After each stage write `.temp/brief-800/PROGRESS.md`: done and verified, the
  exact next action, the tree state, and anything the director needs. Leave the staged tree
  compiling where possible.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. No `git` command at all.
- Do NOT modify files outside Scope. If you believe you must, STOP and write that in the report.
- Touch NOTHING outside this repo; no network; no new dependency; no new `unsafe` in
  `rum-core`/`rum-runtime`; `.temp` only, never `/tmp`.
- The Testing Policy is a hard line (`AGENTS.md`): rum's own documented behavior only; no external
  engine comparisons, no third-party outputs.
- Paste the real verbatim output. If a cell did not reach its evidence, say so explicitly — never
  fake it.
- Report format: `rum-report/1` at `.temp/brief-800/report.json`, one `runs` entry per run, every
  verdict line verbatim, `changes` limited to `.temp/brief-800/`, deviations disclosed. Your final
  message is `python3 -I scripts/report.py summary 800`.
