# Brief 801 — G15-CM4: the source-placement experiments under stable currencies

Status: experiment/diagnostic brief — `Scope: NONE` on the live tree; every write goes under
`.temp/brief-801/`; the sealed role images and every earlier brief's evidence (797/798/799/800)
are read-only. Dispatched after ruling 833 (gate c181). Brief 800 delivered the ten-row
classification and the three reference baselines but stopped at its own calibration rule: two
builds of the same armed tree with byte-identical `.text` measured a −5,060,074 whole-panel Ir
difference, and the sealed 798 attempt-1 receipts themselves contain same-binary three-run spreads
of 1,382,569 / 12,362,850 / 6,650,041. Whole-panel instruction counts are **not** deterministic
for this program. This brief re-runs the four evaluator placements, the 13 outlined-body
placements and the row-1056 2×2 with **per-iteration and per-visit structural counts** as the
decision currency, and settles each cell with its stable counts, its emitted-text/symbol verdict
and its verbatim falsifier outcomes. Carve: **A 250 / B 400 / C 2,600 / D 3,100 / E 1,000 / F 700
= 8,050 s**, funded by brief 800's remaining 8,125.8 s (10,600 − 2,474.2; 75.8 s left
unallocated). No live source change, no acceptance suite, no epoch, no timing or hardware
decision.

## Goal

For each experiment cell: one measured price in stable counts, one emitted-text/symbol verdict,
one verbatim falsifier outcome — so the director can write the repair plan (D.1) with measured
numbers instead of opportunities. Prices are measured per cell and are **never summed** into a
predicted saving. A cell whose stable currency cannot separate the factors is
`unpriced (within band)`; a stage that cannot settle its question STOPs with the exact missing
measurement. A STOP in one stage does not block a later stage.

## The measurement theory (binding; ruling 833)

- **Stable currencies** (decision inputs; reproduce across the 800 identity within the stated
  bands):
  - the four StringReplace short-label **flag-address slopes** (flag-class Ir per iteration):
    armed reference **22 / 22 / 58 / 26**, real reference **3 / 3 / 5 / 5**; identical between
    each role's reference and its identity;
  - the armed **flag-class total** (15,416,582) and per-PC executed multiplicities, band ±~200
    Ir (identity +154); the real class (1,291,997), band ±~100 (identity +57);
  - the row-1056 continuation's **non-gate Ir per visit** (armed 218; base/control 213;
    identity delta 0) and its visits (14,012);
  - each helper's **self Ir per caller visit** and its caller-edge calls (the `a4-helper-prices.tsv`
    rates; per-row identity band measured in Stage B);
  - the publication **invariant**: 38 reached sites, 37 independent gates at 4 Ir/visit, one
    reused-load gate at 3, zero eliminated, zero slow/lexical self, perturbator 0, both
    StringReplace sites 4/visit;
  - the driven ledger's exact bytes/hash and the normalized functional output.
- **Noisy; report, never gate by equality**:
  - whole-panel totals: same-binary spreads 1,382,569 / 12,362,850 / 6,650,041 at the ~16.5B
    scale (sealed 798 three-run draws); the 800 identity −5,060,074;
  - per-PC hit counts (the armed identity moved one PC by 14 hits on 1,055,192; class +154);
  - symbol self totals (body 1116 +3,185 on 11.68M, 0.027%);
  - publication visits (±26 between reference and identity: 366,912→366,919; the sealed bar was
    366,893);
  - small-family signs (16 of 93 family signs differ from the historical table).
- **Identity rule**: a control passes when the stable currencies of the class under test
  reproduce; a whole-panel zero delta is not an identity criterion. Falsifiers (output equality,
  event structure, ledger) remain exact.
- **Text identity**: compare only the symbol ranges under test. The mimalloc compilation stamp
  changes 136 bytes across 24 allocator symbols on every rebuild (`real-identity-text.log`);
  those symbols are excluded from text-identity checks.

## Context pointers

- Read `AGENTS.md` (hard lines), ruling 833 (`.temp/director/ruling-833.md`), and the 800 report:
  `python3 -I scripts/report.py show 800 --tag entry --tag references --tag evaluator --tag
  ten-rows --tag outlines --tag row1056 --tag class-form --tag synthesis --tag verification`.
- **Retained references (read-only; verify, do not rebuild the originals):**
  - `.temp/brief-800/report.json` (2,855,476 B) and its check; `.temp/brief-800/evidence/`:
    `release-*.json`, `build-*.json`, `evaluator.tsv`, `outline.tsv`, `ten-rows.tsv`,
    `ten-rows.json`, `classification-ledger.tsv`, `row1056-A.json`, `identity-*-comparison.json`,
    `identity-*-flag-pcs.tsv`, `real-identity-text.json`, `class-form.tsv`, `live-entry.tsv`,
    `live-exit.tsv`, `entry-index.json`, `exit-index.json`.
  - Reference ELFs and `.text`: `.temp/brief-800/bins/{base,new,control,identity-base,identity-new}`
    and `*.text` (SHA-256 pinned in the 800 report; base/new/control `.text` reproduce the sealed
    798 ELFs `06d0d7b4…` / `2348c468…` / `2e092771…`).
  - Raw whole captures: `.temp/brief-800/callgrind/whole-{base,new,control,identity-new,
    identity-base}.out` and the 32 short captures; their SHA-256s are in the 800 report's
    `coverage.raw_sha256`.
  - The sealed 798 draws: `.temp/brief-798/evidence/amendment3/whole-attempts.json` (three runs
    per arm with totals) and `.temp/brief-798/report.json` SHA-256
    `f281efa552833287103ec247cc3928b9a07d21a22803df4a5040f09721add08e`.
  - Tools to copy and adapt: `.temp/brief-798/tools/{a3_capture.py,a3_entry.py,a3_whole.py,
    a3_localize.py,a2_common.py,instruments.py}` and the 800 runners under
    `.temp/brief-800/tools/`.
- **Source declarations (the only files any variant may touch, in the staged copy):**
  - evaluator cells: `crates/rum-runtime/src/runtime/profiling/{state.rs,config.rs,timing.rs,
    reset.rs}` and `crates/rum-runtime/src/runtime/eval_core/mod.rs`;
  - outlined-body cells: the `armed_decl` cells of `.temp/brief-799/a4-helper-prices.tsv` (13
    rows: 1034, 1383, 1038, 1276, 1618, 1263, 1083, 1184, 1086, 1199, 1185, 1110, 1014);
  - row-1056 cells: `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs`,
    `.../replace/once/options.rs`, `crates/rum-runtime/src/runtime/definitions/definition_catalog/
    rewrite_rules.rs`, `crates/rum-runtime/src/runtime/options/core.rs`, and
    `crates/rum-runtime/src/runtime/eval_core/mod.rs` only if the private-copy call site requires
    it. Any other file is a STOP for a ruling.
- The canonical program `.temp/brief-795/panel/canonical/core-language-microbench.rum`
  (SHA-256 `dc6af977130ad0e85280d080d199b268e24bacb30c6eda6756f2a2e6a56bd633`); the 14
  `RUM_PROFILE_*` flags all `0`; `RUM_CORE_MICRO_RETAINED_MODULE_GC_BATCHES=3`; the build recipe
  of record `.temp/brief-798/evidence/amendment2/build-new.json` (cwd the one absolute source,
  private `CARGO_HOME`, external `CARGO_TARGET_DIR`, rust 1.85.0 ahead on PATH,
  `cargo build --offline --locked -j 8 --release -p rum-cli --bin rum-cli`,
  `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`).

## Scope

- You MAY write only under `.temp/brief-801/`: `tools/`, `active/` (the one build path),
  `target/` (the one external target and private offline home), `bins/`, `callgrind/`, `logs/`,
  `evidence/`, `report.json`, `PROGRESS.md`.
- Read-only and immutable: `.temp/brief-797/` (four role images), `.temp/brief-798/` (report,
  ELFs, tables, tools), `.temp/brief-799/` (all evidence), `.temp/brief-800/` (report,
  references, raw captures, tools), `.temp/director/`. Verify their pins at entry and exit.
- The live tree is untouched: no edit under `crates/`, `docs/`, `scripts/`; zero builds in the
  live tree; `git status` clean at exit; every build happens in `.temp/brief-801/active` with the
  external target.
- `.temp` only, never `/tmp`. No network. No new dependency. No new `unsafe` in
  `rum-core`/`rum-runtime`. No `git` command of any kind.

## Stage A — entry and retained-input pins (250 s)

Verify and index read-only: the 800 report hash and `report.py check 800`; the 800 ELFs,
`.text` files and raw-capture SHA-256s; the 798 report and the whole-attempts table; the four
role-tree hashes; the canonical/functional pins and the 14 off-state flags (in
`.temp/brief-798/evidence/amendment1/capture-plan.json`); the toolchain and CPU-14 identity; and
the live tree's **5,878-file** metadata manifest — the 800 exit manifest's 5,877 paths all
present and identical, plus `docs/dev/codex/brief-801-…md` (this brief). The 800 brief's own
document is part of the 5,877 and must match. Emit `.temp/brief-801/evidence/entry-index.json`.
A missing or differing pinned input is a STOP.

## Stage B — calibration and the same-path control (400 s)

1. Compute the bands from the retained evidence and record them in `evidence/calibration.tsv`:
   - the sealed 798 same-binary three-run spreads per arm (base/new/control);
   - the armed and real identity readings: flag PCs (57/24 armed, 33/2 real), class deltas
     (+154 / +57), short-label flag slopes (22/22/58/26 and 3/3/5/5), row-1056 per-visit (218,
     delta 0), publication (38 sites; 4/visit; visits 366,912 vs 366,919; Ir 1,464,131 vs
     1,464,159);
   - the 13 helper rows' reference values (`a4-helper-prices.tsv` and the 800 `outline.tsv`) and
     their identity-vs-reference deltas computed from the retained armed reference and
     `whole-identity-new` captures — this is each outline cell's band;
   - the 1186-group rows (1184, 1185, 1199) and caller 1186, counted once.
2. **Same-path control**: copy the armed tree byte-for-byte to `.temp/brief-801/active`, build
   with the recipe of record, save the ELF and `.text`, capture the canonical whole panel with
   the exact 798 command, reduce, and compare the stable currencies to the 800 armed reference.
   The mimalloc stamp may differ; the measured symbol ranges must not. Emit `PATHCTL`. A
   mismatch on any stable currency is a STOP: no cell is run until the path control matches.

Print:

```
BAND arm=<base|new|control> runs=3 min=<n> max=<n> spread=<n>
STABLE key=<flag_armed|flag_real|short_armed|short_real|row1056|publication|helper:NNNN> reference=<...> identity=<...> delta=<n>
PATHCTL role=armed text=<identical|changed:mimalloc> flag_pcs=<n/n> slopes=<a,b,c,d> row1056=<n> publication=<sites,visits,Ir> verdict=<match|mismatch>
```

## Stage C — the evaluator placements (2,600 s)

One cell at a time, serially, at the one path: copy the role's sealed tree, apply exactly one edit,
build, save the ELF/`.text`, extract the flag-class PC table (static PCs, destinations, executed
multiplicities, disassembly — the `a4-flag-pcs.tsv` shape), capture the canonical whole panel,
reduce, and run the battery. Each cell is paired with its role's reference (the 800 capture, plus
the Stage B path control).

**The measurement**: the exact 798 chunk `taskset -c 14 valgrind --tool=callgrind
--dump-instr=yes --vgdb=no --enable-debuginfod=no --log-file=<vg> --callgrind-out-file=<raw>
<elf> --file <canonical>`, cwd the repo root, stdin `/dev/null`, env `ordinary_env() ∪
profile_off() ∪ {DEBUGINFOD_URLS:""}`. Report: the four short-label flag-address slopes and their
deltas (the per-iteration price), the armed flag-class total and per-PC executed multiplicities,
body rows 1115/1116, the publication invariant, the caller-inclusive deltas, and the whole-panel
delta **with the Stage B band** (informational, never a gate). Instruction counts are stable only
in the per-visit/per-iteration currency; no interference qualification is needed for them, but
every run is serial and alone, and every raw file is retained before parsing.

**The cells.** Each edits only the listed files; one source edit per cell; a semantic change to
whether or when any profiling event is recorded is a STOP, never a cost change.

- **E2.1 fast-path at `lexical_profile_configured_admitted`** (`profiling/config.rs:70–85`): read
  the same atomic once at that exact hook — return false on 2, true on 1, take the cold fallback
  on 0. Keep the test lease admission ahead of access. Inspect all 29 ENABLED sites for
  duplicated address materialization and the 28 ACTIVE sites for unintended reachability.
- **E2.2 cold out-of-line work** (`profiling/timing.rs` veneers; `config.rs:115`): keep an inline
  configured-disabled exit; funnel the initialized-on/unknown path into one `#[cold]
  #[inline(never)]` routine that commissions or reads ACTIVE using the current lease and performs
  the existing operation. Preserve state-0 initialization and on → active/deferred semantics
  exactly once. A call on every disabled hook violates the fast-path bar; moving instructions into
  a callee counts only if combined executed Ir falls.
- **E2.3 fold the address, preserve the mutable byte** (`profiling/state.rs:20`,
  `config.rs:78,570`): bind a private constant reference to the existing static
  (`&'static AtomicU8`) and use it at the configured reads; each relaxed state load and
  initialization stays at its original site. A permanently false atomic/feature is invalid (it
  drops the enabled rows). An identical code image prices this candidate at zero.
- **E2.4 hoist only across a proven invariant interval** (`eval_core/mod.rs:250`,
  `config.rs:39`, `profiling/reset.rs:126`): thread the configured flag address from
  `evaluate_inner_unstable` into its inline timing veneers; a configured-off specialization may
  be selected at entry only after proving the mode immutable across that interval in every
  supported lease/reset path. Keep the configured-on path reading ACTIVE at each original start
  and completion, reselect on reset, callback and resume boundaries, and never cache “active”
  across user evaluation. If the proof fails, test address hoisting alone.

Each candidate is staged on **both roles**, one at a time, each paired with its role reference.
On `real` the battery is the off-state canonical output equality and the flag-class table (the
real role has no PUBLISHED symbol and no class-form publication); on `armed` the full battery:

1. Off-state equality: the canonical 120-label program normalizes clock lines and is byte-equal
   in non-clock output and label order vs the armed reference; the six functional runs likewise;
   all 14 `RUM_PROFILE_*` = 0.
2. Enabled/deferred/reset: fresh processes with `RUM_PROFILE_LEXICAL_SUBSTITUTION=1` and
   `DEFER=0`, and with `DEFER=1` around `RumProfileReset[]`; identical enabled phase keys, counts,
   order, completion/abandonment records and post-reset/active transitions — compare event
   structure, never durations.
3. `cargo test -p rum-stdlib --test suspension_oracle -- logical` at `RUST_TEST_THREADS=8` in the
   staged tree; quote the `test result:` line and the LP03/LP04/LP05 lines; require the same
   pass/fail counts as the reference run and at least one selected test (zero selected is a STOP).
4. The permanently-off negative control in an isolated diagnostic: the enabled tests must reject
   a permanently-false flag.
5. The publication invariant: 38 reached sites; both StringReplace sites 4 Ir/visit; typed 4+3;
   zero eliminated; zero slow/lexical self; perturbator 0; the short-profile 4-Ir site cap; the
   visits/IR reported with the Stage B band.
6. The driven ledger: 26 ordered lines / 1,741 bytes, SHA-256
   `a2c48c90fab83d49ffceb07e8c74992268fea353cf5e03574f8cbe63435d3895`, the real P0256 hit, the 12
   routes, balanced owners, the mid-request Print sequence (`entry=false,
   callback_1_before=false, callback_1_after=true, callback_2=true, after_end=false, output=42`)
   with both reached StringReplace edges, and resumed == queued in rum.

A failed falsifier is recorded `FAIL` and that candidate cannot be a repair. A cell whose
unchanged text (the measured symbol ranges, excluding mimalloc) cannot reproduce the reference's
stable currencies within the Stage B band is a measurement fault — STOP, name it. Print per cell:

```
EVAL role=<real|armed> candidate=<fastpath|coldout|foldaddr|hoist> text=<identical|changed:syms> flag_static=<n> flag_executed=<n> slopes=<a,b,c,d> class_delta=<n> whole_delta=<n> band=<n> prices=<Δa,Δb,Δc,Δd> falsifiers=<PASS|FAIL:name> price=<n|unpriced>
```

## Stage D — the outlined-body placements (3,100 s)

The 13 rows are armed-role cells; the bodies are absent from the real build (the 800 outline table
shows `ref_symbol_present=False` there), so no real-role outline cell is run. For each row, one
cell at a time at the one path: copy the armed tree, add exactly one `#[inline]` at the `armed_decl`
cell, build, save the ELF/`.text` and symbol table, capture the canonical whole panel, reduce the
helper's self/visits and its caller rows (`.temp/brief-799/a4-helper-prices.tsv` shape), and run
the light battery. If the symbol survives, replace the attribute with `#[inline(always)]` and
repeat once; record both. Do not change any body text or publication choice — rows 1618, 1263 and
1014 carry a publication choice inside the ordinary body that must be retained; a cell that
removes or moves it is a STOP.

**Prices and accounting**: the cell price is the net change of its helper self + caller rows vs
the armed reference, in per-visit/per-call counts; the 1184/1185/1199 rows share caller 1186 and
are reported as one group with that caller counted once (the disjoint four-row group +252,880 is
the only spendable shared-caller figure; cell sums must not double-spend it). The two proved
helpers 1034 and 1383 are measured first. The adapter pair 1147/1448 is outside the cells. The
light battery: off-state equality (the capture's normalized output), the publication invariant
from the same capture, and the driven ledger; the suspension oracle is deferred to the repair
implementation, because an `#[inline]` attribute is semantics-neutral — state this deferral
verbatim. Print per row and for the group:

```
OUTLINE row=<n> decl=<path:line> symbol=<gone|present> visits=<n> self_per_visit=<f> caller_net=<n> price=<n|unpriced> falsifiers=<PASS|FAIL:name>
OUTLINE_GROUP group=1186 members=1184,1185,1186,1199 net=<n> counted_once=true
```

## Stage E — the row-1056 2×2 (1,000 s)

Four armed cells, one at a time at the one path, paired with the armed reference (cell A, the
current armed layout; reuse the 800 `whole-new` capture). The source list is corrected:
`once.rs`, `once/options.rs`, `rewrite_rules.rs`, `options/core.rs`, and `eval_core/mod.rs` only
if the private-copy call site requires it.

| cell | driven caller | driven option branch |
|---|---|---|
| A | shares the ordinary resume implementation (reference) | inlined in the ordinary path |
| B | shares the ordinary resume implementation | behind a cold boundary, sharing the ordinary argument construction |
| C | private equivalent cold copy of the resume path | inlined in the ordinary path |
| D | private equivalent cold copy of the resume path | behind a cold boundary |

Hold the publication read at the same logical point in all four; verify each cell's emitted graph
against its intended factor (which function calls which resume/option reader). Record the inputs,
visits, all child call counts, the four publication gate PCs (re-anchored per cell), disjoint
per-PC Ir, caller+callee totals, the stack frame size, and each cell's disassembly for the three
call-delimited regions. Run the battery per cell: off-state equality, `cargo test -p rum-stdlib
--test suspension_oracle -- logical` at 8 threads, the driven ledger, typed 4+3, and
resumed == queued. If an otherwise equivalent armed cell reproduces the ordinary placement
(213 non-gate instructions per visit) and sheds the cost, it prices that boundary; a cell that
changes observable behavior, folds a required gate, merges the planned variants or changes visit
multiplicity cannot settle it. If the contrasts still fail to separate, retain the STOP. Print:

```
ROW1056 cell=<A|B|C|D> non_gate_per_visit=<n> delta=<n> frame=<0x...> graph=<verified|not> falsifiers=<PASS|FAIL:name> verdict=<prices|required|avoidable|cannot_settle>
ROW1056 STOP retained=<true|false>
```

## Stage F — synthesis and handoff (700 s)

One table and one report section per family, each price carrying its falsifier outcome and its
Stage B band; the class-form-required remainder stated separately; `additive_savings_claimed=false`
verbatim. The updated classification ledger (row 1056's disposition and the ten classified rows),
the live-tree metadata comparison (5,878 paths) and the report check. Print:

```
SYNTHESIS eval_prices=<n> outline_prices=<n> row1056=<verdict> ledger=<a/b/c/d> additive_claimed=false
```

## Verification (MANDATORY — you run it; iterate to evidence)

This is a diagnostic/experiment brief: the named artifacts and the verbatim printed lines ARE the
deliverable. Every run's full output is saved under `.temp/brief-801/logs/` before parsing; every
raw Callgrind file is retained; every falsifier line is quoted, not paraphrased. Run
`python3 -B scripts/run-bounded.py --timeout 20 -- python3 scripts/report.py check 801` before
sealing and fix every PROBLEM. Do not run the workspace suite — there is no live change.

## Pitfalls that apply here

- **One variable per cell.** One source edit per variant; the reference is the same role built at
  the same path with the same flags. A cell that changes two things cannot price either.
- **Stable currencies, not whole totals.** Decisions come from per-iteration/per-visit counts and
  exact falsifier outputs. Whole-panel totals are reported with the Stage B band; an exact
  whole-panel equality requirement is the 800 defect and must not reappear.
- **The allocator stamp.** A same-source rebuild changes 136 mimalloc bytes; compare only the
  measured symbol ranges for text identity.
- **The publication invariant, not its visits.** 38 sites and 4 Ir/visit are the gate; 366,91x
  visits and 1.46M Ir are reported with the band.
- **The build recipe is the measurement.** Same absolute path, same flags, one codegen unit,
  private `CARGO_HOME`, offline, one cargo command at a time, `-j 8`; the 798 recipe of record is
  normative down to the argv and env.
- **No summing.** Each price is local to its cell; never add prices across cells or families.
- **No live edits.** All staging happens on copies under `.temp/brief-801/active`; the live tree
  is read-only even for a moment.

## Checkpointing — protect the scene

Write `PROGRESS.md` and a checkpoint receipt after every stage: the stage name, elapsed seconds,
the evidence paths, the printed line, and the next action. A stage that reaches its STOP rule
records the STOP verbatim and does not open the next stage unless the brief order says it is
independent. Keep `.temp/brief-800/` untouched; it is the reference source.

## Hard rules

- `.temp` only, never `/tmp`; no network; no new dependency; no new `unsafe` in
  `rum-core`/`rum-runtime`; no `git` command of any kind; no tag.
- One cargo command at a time; captures serial and alone; instruction counts only (no wall-clock,
  no perf, no hardware counter as a decision input).
- Persist raw output before parsing; quote verdicts verbatim; never paraphrase a falsifier line.
- Report scope honestly; disclose every deviation with its cause; do not exceed the stage boxes
  silently — an overrun is charged and disclosed.

## Report format

`rum-report/1` JSON via `scripts/report.py`, written to `.temp/brief-801/report.json`, with the
sections `entry`, `calibration`, `evaluator`, `outlines`, `row1056`, `synthesis`, `verification`;
the checks carry their bars, expected/actual and evidence; the printed lines are quoted verbatim;
`additive_savings_claimed=false`; `changes` lists every written path; `verification` points to
the entry/exit indexes, the report check and the summary.
