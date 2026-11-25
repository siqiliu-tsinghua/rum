# Brief 802 — G15-CM4: the source-placement experiments under corrected anchors and per-currency admission

Status: experiment/diagnostic brief — `Scope: NONE` on the live tree; every write goes under
`.temp/brief-802/`; the sealed role images and every earlier brief's evidence (797/798/799/800/801)
are read-only. Dispatched after ruling 834 (gate c182). Brief 801's mandatory same-path control
matched on the armed `.text` (byte-identical, ELF 2766c742…), the 57/24 flag PCs, the four short
slopes, row 1056, the publication invariant, twelve of thirteen helper rates and the canonical
120-label output — and then stopped every cell on two findings: (a) helper row 1199's hash-probe
rate moved run to run at fixed caller multiplicity (227,147 / 225,576 / 224,323 Ir over 2,402
calls; 26 PCs inside the probe loop; the input state unidentified), and (b) the brief's binding
flag-class centers were the sealed 798 draws (15,416,582 / 1,291,997), while the 800 measured
references are **15,417,000 / 1,291,991**. (b) is my defect; (a) is a real data-dependent
currency that binds only the group-1186 cell. This brief re-anchors the calibration on the 800
measured references, adds the real-role path control, admits each cell on its own currencies, and
re-runs the four evaluator placements, the 13 outlined-body placements and the row-1056 2×2.
Carve: **A 150 / B 400 / C 2,800 / D 2,850 / E 1,000 / F 800 = 8,000 s**, funded by brief 801's
remaining 7,073.032 s plus the 800 remainder's unallocated 75.832 s (7,148.832 s) and an explicit
851.168 s new allocation (「预算不够就追加吧」). No live source change, no acceptance suite, no
epoch, no timing or hardware decision.

## Goal

For each experiment cell: one measured price in stable counts with its measured band, one
emitted-text/symbol verdict, one verbatim falsifier outcome — so the director can write the
repair plan (D.1) with measured numbers instead of opportunities. Prices are measured per cell and
are **never summed** into a predicted saving. A cell whose measured net does not exceed its band
is `unpriced (within band)`; a stage that cannot settle its question STOPs with the exact missing
measurement. A STOP in one stage does not block a later stage.

## The measurement theory (binding; rulings 833–834)

- **Binding anchors are the 800 measured references, with raw provenance** —
  armed **15,417,000** (`.temp/brief-800/callgrind/whole-new.out`, sha256 `f63a6962…`),
  real **1,291,991** (`whole-base.out`, sha256 `65c19dd0…`). The sealed 798 draws
  (15,416,582 / 1,291,997) are historical and must not be printed as references.
- **Stable currencies** (decision inputs; ≥3 same-role draws; band = `max |draw − binding|`):
  - the four StringReplace short-label **flag-address slopes**: armed **22 / 22 / 58 / 26**
    (reference, identity and 801 path control identical), real **3 / 3 / 5 / 5** (reference,
    identity; the real path control adds the third draw);
  - the armed **flag-class total** 15,417,000 (draws 0, +154, −44; band 200 pre-registered) and
    the real class 1,291,991 (draws 0, +57, +real path control; band 100);
  - the row-1056 continuation's **non-gate Ir per visit** (armed 218; base/control 213; three
    draws identical) and its visits (14,012);
  - each helper's **self Ir per caller visit**, its caller rows and their union — twelve rows
    exact across the three armed draws, row 1199 **data-dependent** (see below);
  - the evaluator's body rows 1115/1116 and the disjoint caller/evaluator union — armed 1115
    exact, 1116 draws 0 / +3,185 / −910 (band 3,185), union draws 0 / +5,026 / −1,372 (band
    5,026); real 1116 draws 0 / +8,341 / +real path control;
  - the publication **invariant**: 38 reached sites, 37 independent gates at 4 Ir/visit, one
    reused-load gate at 3, zero eliminated, zero slow/lexical self, perturbator 0, both
    StringReplace sites 4/visit (exact in all three armed draws);
  - the driven ledger's exact bytes/hash and the normalized functional output.
- **Row 1199 is a data-dependent currency** (`association_lvalue_has_valid_direct_proof`, the
  hashbrown probe loop at 0x9cf7e8–0x9cf846): three draws 227,147 / 225,576 / 224,323 Ir over
  2,402 identical edge calls → band **2,824 Ir** (1.1757 per call); the group-1186 union draws
  1,736,061 / 1,734,490 / 1,733,237 → the same band. Its only cell is the group-1186 cell.
- **Per-cell admission**: a currency binds only the cells that measure it. Every cell below is
  admitted with its own currency bands; the 801 global hold is superseded. A cell's price must
  exceed its band; otherwise it is `unpriced (within band)` — an outcome, not a fault. A cell
  whose **unchanged** text cannot reproduce its binding within its band is a measurement fault —
  STOP, name it.
- **Noisy; report, never gate by equality**: whole-panel totals (same-binary spreads 1,382,569 /
  12,362,850 / 6,650,041; the 801 path-control whole delta +7,550,673 inside the spread);
  per-PC hit counts; publication visits (±~26); whole symbol totals.
- **Text identity**: compare only the symbol ranges under test. The mimalloc compilation stamp
  changes 136 bytes across 24 allocator symbols on a real-tree rebuild; those symbols are excluded.

## Context pointers

- Read `AGENTS.md` (hard lines), ruling 834 (`.temp/director/ruling-834.md`), ruling 833, the 801
  report, and the 800 report:
  `python3 -I scripts/report.py show 801 --tag entry --tag calibration --tag evaluator --tag
  outlines --tag row1056 --tag synthesis --tag verification` and
  `show 800 --tag evaluator --tag outlines --tag ten-rows --tag row1056 --tag class-form`.
- **Retained references (read-only; verify, do not rebuild the originals):**
  - `.temp/brief-800/report.json` (2,855,476 B, sha256
    `677b718538ace0ce96e40fbaf31eb8c2cafcceafaf8624f989d3fc1a2fd1e5cd`)
    and `.temp/brief-800/evidence/`: `release-*.json`, `build-*.json`, `evaluator.tsv`,
    `outline.tsv`, `ten-rows.tsv`, `ten-rows.json`, `classification-ledger.tsv`, `row1056-A.json`,
    `identity-*-comparison.json`, `identity-*-flag-pcs.tsv`, `real-identity-text.json`,
    `class-form.tsv`, `live-entry.tsv`, `live-exit.tsv`, `entry-index.json`, `exit-index.json`.
  - The 800 binaries and `.text`: `.temp/brief-800/bins/{base,new,control,identity-base,
    identity-new}` and `*.text`; the four raw whole captures
    `.temp/brief-800/callgrind/{whole-base.out,whole-new.out,whole-identity-new.out,
    whole-identity-base.out}` with SHA-256 `65c19dd0…`, `f63a6962…`, `3db41c05…`, `f7f370cf…`,
    and the 32 short captures (hashes in the 800 report).
  - `.temp/brief-801/report.json` (2,460,537 B, sha256
    `a73bcf317156611ca27a52ed57e7a96296fb92692395d4ee826ecaa62673d176`) and its check; and
    `.temp/brief-801/evidence/`: `calibration.tsv`, `pathctl.json`, `flag-anchor-provenance.tsv`,
    `helper1199-pcs.tsv`, `pathctl-helper-comparison.tsv`, `pathctl-flag-comparison.tsv`,
    `pathctl-measured-symbols.tsv`, `evaluator.tsv`, `outline.tsv`, `row1056-A.json`,
    `row1056.tsv`, `live-entry.tsv`, `live-exit.tsv`, `entry-index.json`, `exit-index.json`.
  - The 801 armed path control: ELF `.temp/brief-801/bins/pathctl` (2766c742…), `.text`
    `.temp/brief-801/bins/pathctl.text` (2a2ae65e…, 12,792,096 B), raw
    `.temp/brief-801/callgrind/whole-pathctl.out` (fa4e7109…), and the eight short raws
    (`c70052dc…`, `e613a732…`, `4724dafc…`, `4dc82773…`, `87919a34…`, `88b12e38…`, `4010036a…`,
    `45fa9ae2…`).
  - The sealed 798 draws: `.temp/brief-798/evidence/amendment3/whole-attempts.json`; the 798
    report sha256 `f281efa552833287103ec247cc3928b9a07d21a22803df4a5040f09721add08e`.
  - Tools to copy and adapt: `.temp/brief-798/tools/{a3_capture.py,a3_entry.py,a3_whole.py,
    a3_localize.py,a2_common.py,instruments.py}` and the 800/801 runners under
    `.temp/brief-800/tools/`, `.temp/brief-801/tools/`.
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
- The role trees (read-only): `.temp/brief-797/amendment6/images/{baseline,real,disabled,armed}`
  with source sha256 `ec602109…`/5,842, `022d60af…`/5,851, `c7156d50…`/5,851, `318933db…`/5,851.
- The canonical program `.temp/brief-795/panel/canonical/core-language-microbench.rum`
  (SHA-256 `dc6af977130ad0e85280d080d199b268e24bacb30c6eda6756f2a2e6a56bd633`); the 14
  `RUM_PROFILE_*` flags all `0`; `RUM_CORE_MICRO_RETAINED_MODULE_GC_BATCHES=3`; the build recipe
  of record `.temp/brief-798/evidence/amendment2/build-new.json` (cwd the one absolute source,
  private `CARGO_HOME`, external `CARGO_TARGET_DIR`, rust 1.85.0 ahead on PATH,
  `cargo build --offline --locked -j 8 --release -p rum-cli --bin rum-cli`,
  `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`); the capture-plan fields live in
  `.temp/brief-798/evidence/amendment1/capture-plan.json` and the job invocations in
  `.temp/brief-798/evidence/amendment3/capture-plan.json`.

## Scope

- You MAY write only under `.temp/brief-802/`: `tools/`, `active/` (the one build path),
  `target/` (the one external target and private offline home), `bins/`, `callgrind/`, `logs/`,
  `evidence/`, `report.json`, `PROGRESS.md`.
- Read-only and immutable: `.temp/brief-797/` (four role images), `.temp/brief-798/` (report,
  ELFs, tables, tools), `.temp/brief-799/` (all evidence), `.temp/brief-800/` (report,
  references, raw captures, tools), `.temp/brief-801/` (report, path control, captures), and
  `.temp/director/`. Verify their pins at entry and exit.
- The live tree is untouched: no edit under `crates/`, `docs/`, `scripts/`; zero builds in the
  live tree; `git status` clean at exit; every build happens in `.temp/brief-802/active` with the
  external target.
- `.temp` only, never `/tmp`. No network. No new dependency. No new `unsafe` in
  `rum-core`/`rum-runtime`. No `git` command of any kind.

## Stage A — entry and retained-input pins (150 s)

Verify and index read-only: the 800 and 801 report hashes and `report.py check 800` / `check 801`;
the 800 reference ELFs, `.text` files and the four whole raw captures; the 801 path-control ELF,
`.text`, raw whole capture and eight short raws; the 798 report and the whole-attempts table; the
four role-tree hashes; the canonical/functional pins and the 14 off-state flags; the toolchain and
CPU-14 identity; and the live tree's **5,879-file** metadata manifest — the 801 exit manifest's
5,878 paths (sha256 `a03a74f8…`) all present and identical, plus
`docs/dev/codex/brief-802-…md` (this brief). Emit `.temp/brief-802/evidence/entry-index.json`.
A missing or differing pinned input is a STOP.

## Stage B — corrected calibration, the real path control, and admission (400 s)

1. **Anchors with provenance.** For each role, replay the printed center from the raw captures and
   emit the anchor line:
   - armed: binding 15,417,000 from `whole-new.out`; draws sealed798 15,416,582, reference800
     15,417,000, identity800 15,417,154, pathctl801 15,416,956; band 200.
   - real: binding 1,291,991 from `whole-base.out`; draws sealed798 1,291,997, reference800
     1,291,991, identity800 1,292,048, realctl802 (this stage); band 100.
2. **Currency table.** Recompute every cell's currencies from the retained raws — the flag class
   and its 57/24 (armed) and 33/2 (real) PCs, the four slopes, row 1056, the publication census,
   the 13 helper rows with their caller rows and unions, the evaluator body rows 1115/1116 and
   the disjoint caller/evaluator union — with each draw's value and the band `max |draw −
   binding|`. The twelve exact helpers keep band 0; row 1199 and the group-1186 union carry band
   2,824 Ir; the evaluator armed body 1116 band is 3,185 and its union band 5,026; the real
   numbers are completed by the real path control. Emit `evidence/calibration.tsv`.
3. **Real path control.** Copy the sealed real tree byte-for-byte to `.temp/brief-802/active`,
   build with the recipe of record, save the ELF/`.text`/measured symbol ranges, capture the
   canonical whole panel with the exact 798 command, reduce, and compare the real currencies to
   the binding: the flag class within ±100, the 33/2 PCs, the slopes 3/3/5/5, the body rows and
   the canonical output. The `.text` differs in the mimalloc stamp; the measured symbol ranges
   must not. Emit `PATHCTL role=real`.
4. **Admission.** Emit one `ADMIT` line per cell: the cell's currencies with their bands, the
   cell's band (the maximum), and `verdict=<admit|stop>`. Every cell of Stages C–E is expected
   admissible; a currency that cannot bind its cell makes that cell `unpriced (within band)`, and
   only that cell. A path control whose measured text cannot reproduce a currency within the
   pre-registered flag band is a measurement fault — STOP, name it.

Print:

```
FLAG_ANCHOR role=<armed|real> binding=<n> source=<path> sha256=<hex> draws=<sealed798,reference800,identity800,pathctl801,realctl802> band=<n>
CURRENCY key=<flag_armed|flag_real|short_armed|short_real|row1056|publication|eval_body:NNNN|eval_union:role|helper:NNNN|group:1186> draws=<a,b,c> spread=<n> band=<n> class=<stable|data_dependent>
PATHCTL role=<armed|real> text=<identical|changed:mimalloc> flag_pcs=<n/n> slopes=<a,b,c,d> row1056=<n> publication=<sites,visits,Ir> verdict=<match|mismatch>
ADMIT cell=<eval:E2.1..E2.4:armed|real|outline:NNNN|row1056:B|C|D> currencies=<key:band,...> band=<n> verdict=<admit|unpriced:key|stop>
```

## Stage C — the evaluator placements (2,800 s)

One cell at a time, serially, at the one path: copy the role's sealed tree, apply exactly one edit,
build, save the ELF/`.text`, extract the flag-class PC table (static PCs, destinations, executed
multiplicities, disassembly — the `a4-flag-pcs.tsv` shape), capture the canonical whole panel,
reduce, and run the battery. Each cell is paired with its role's reference (the 800 capture, plus
the path control).

**The measurement**: the exact 798 chunk `taskset -c 14 valgrind --tool=callgrind
--dump-instr=yes --vgdb=no --enable-debuginfod=no --log-file=<vg> --callgrind-out-file=<raw>
<elf> --file <canonical>`, cwd the repo root, stdin `/dev/null`, env `ordinary_env() ∪
profile_off() ∪ {DEBUGINFOD_URLS:""}`. Report: the four short-label flag-address slopes and their
deltas (the per-iteration price), the role's flag-class total and per-PC executed multiplicities,
body rows 1115/1116, the publication invariant, the caller-inclusive deltas, and the whole-panel
delta with the Stage B band (informational, never a gate). Every run is serial and alone, and
every raw file is retained before parsing.

**The cells** (unchanged from 801; each edits only the listed files; one source edit per cell; a
semantic change to whether or when any profiling event is recorded is a STOP, never a cost
change):

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
  and completion, reselect on reset, callback and resume boundaries, and never cache "active"
  across user evaluation. If the proof fails, test address hoisting alone.

Each candidate is staged on **both roles**, one at a time, each paired with its role reference.
On `real` the battery is the off-state canonical output equality and the flag-class table (the
real role has no PUBLISHED symbol); on `armed` the full battery:

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
unchanged text (the measured symbol ranges, excluding mimalloc) cannot reproduce its binding
within the Stage B band is a measurement fault — STOP, name it. Print per cell:

```
EVAL role=<real|armed> candidate=<fastpath|coldout|foldaddr|hoist> text=<identical|changed:syms> flag_static=<n> flag_executed=<n> slopes=<a,b,c,d> class_delta=<n> whole_delta=<n> band=<n> prices=<Δa,Δb,Δc,Δd> falsifiers=<PASS|FAIL:name> price=<n|unpriced>
```

## Stage D — the outlined-body placements (2,850 s)

The 13 rows are armed-role cells; the bodies are absent from the real build, so no real-role
outline cell is run. For each row, one cell at a time at the one path: copy the armed tree, add
exactly one `#[inline]` at the `armed_decl` cell, build, save the ELF/`.text` and symbol table,
capture the canonical whole panel, reduce the helper's self/visits and its caller rows (the
`.temp/brief-799/a4-helper-prices.tsv` shape), and run the light battery. If the symbol survives,
replace the attribute with `#[inline(always)]` and repeat once; record both. Do not change any
body text or publication choice — rows 1618, 1263 and 1014 carry a publication choice inside the
ordinary body that must be retained; a cell that removes or moves it is a STOP.

**Prices and accounting**: the cell price is the net change of its helper self + caller rows vs
the armed binding, in per-visit/per-call counts, and must exceed the cell's Stage B band (the
twelve exact helpers 0; row 1199 and the group-1186 union 2,824 Ir). The 1184/1185/1199 rows share
caller 1186 and are reported as one group with that caller counted once (the disjoint four-row
group +252,880 is the only spendable shared-caller figure; cell sums must not double-spend it).
The two proved helpers 1034 and 1383 are measured first. The adapter pair 1147/1448 is outside the
cells. The light battery: off-state equality (the capture's normalized output), the publication
invariant from the same capture, and the driven ledger; the suspension oracle is deferred to the
repair implementation, because an `#[inline]` attribute is semantics-neutral — state this deferral
verbatim. Print per row and for the group:

```
OUTLINE row=<n> decl=<path:line> symbol=<gone|present> visits=<n> self_per_visit=<f> caller_net=<n> band=<n> price=<n|unpriced> falsifiers=<PASS|FAIL:name>
OUTLINE_GROUP group=1186 members=1184,1185,1186,1199 net=<n> counted_once=true band=2824
```

## Stage E — the row-1056 2×2 (1,000 s)

Four armed cells, one at a time at the one path, paired with the armed reference (cell A, the
current armed layout; reuse the 800 `whole-new` capture and the 801 path-control verification:
218 non-gate Ir per visit, 14,012 visits, frame 0x558, the four publication PCs at 0x96fd91).
The source list is corrected: `once.rs`, `once/options.rs`, `rewrite_rules.rs`,
`options/core.rs`, and `eval_core/mod.rs` only if the private-copy call site requires it.

| cell | driven caller | driven option branch |
|---|---|---|
| A | shares the ordinary resume implementation (reference) | inlined in the ordinary path |
| B | shares the ordinary resume implementation | behind a cold boundary, sharing the ordinary argument construction |
| C | private equivalent cold copy of the resume path | inlined in the ordinary path |
| D | private equivalent cold copy of the resume path | behind a cold boundary |

Hold the publication read at the same logical point in all four; verify each cell's emitted graph
against its intended factor (which function calls which resume/option reader). Record the inputs,
visits, all child call counts, the four publication gate PCs (re-anchored per cell), disjoint per-PC
Ir, caller+callee totals, the stack frame size, and each cell's disassembly for the three
call-delimited regions. Run the battery per cell: off-state equality, `cargo test -p rum-stdlib
--test suspension_oracle -- logical` at 8 threads, the driven ledger, typed 4+3, and resumed ==
queued. If an otherwise equivalent armed cell reproduces the ordinary placement (213 non-gate
instructions per visit) and sheds the cost, it prices that boundary; a cell that changes observable
behavior, folds a required gate, merges the planned variants or changes visit multiplicity cannot
settle it. If the contrasts still fail to separate, retain the STOP. Print:

```
ROW1056 cell=<A|B|C|D> non_gate_per_visit=<n> delta=<n> frame=<0x...> graph=<verified|not> falsifiers=<PASS|FAIL:name> verdict=<prices|required|avoidable|cannot_settle>
ROW1056 STOP retained=<true|false>
```

## Stage F — synthesis and handoff (800 s)

One table and one report section per family, each price carrying its falsifier outcome and its
Stage B band; the class-form-required remainder stated separately; `additive_savings_claimed=false`
verbatim. The updated classification ledger (row 1056's disposition and the ten classified rows),
the live-tree metadata comparison (5,879 paths) and the report check. Print:

```
SYNTHESIS eval_prices=<n> outline_prices=<n> row1056=<verdict> ledger=<a/b/c/d> additive_claimed=false
```

## Verification (MANDATORY — you run it; iterate to evidence)

This is a diagnostic/experiment brief: the named artifacts and the verbatim printed lines ARE the
deliverable. Every run's full output is saved under `.temp/brief-802/logs/` before parsing; every
raw Callgrind file is retained; every falsifier line is quoted, not paraphrased. Run
`python3 -I scripts/report.py check 802` before the seal; the report and its detached summary must
agree. The exit audit re-walks the live tree (5,879 paths) and the sealed pins; a mismatch is a
STOP. Do not summarize a number you did not print from a retained file; every number in the
report cites its raw.

## Pitfalls (binding)

- **One variable per cell.** Each cell changes exactly one source construct at the same path with
  the same flags. A cell that changes two things cannot price either.
- **Stable currencies, not whole totals.** Decisions come from per-iteration/per-visit counts and
  exact falsifier outputs. Whole-panel totals are reported with the Stage B band; an exact
  whole-panel equality requirement is the 800 defect and must not reappear.
- **Anchors are the 800 measured references.** Never print the 798 sealed values as references;
  the provenance lines carry every draw.
- **The allocator stamp.** A real-tree same-source rebuild changes 136 mimalloc bytes; compare
  only the measured symbol ranges for text identity.
- **The publication invariant, not its visits.** 38 sites and 4 Ir/visit are the gate; 366,91x
  visits and 1.46M Ir are reported with the band.
- **Bands, not guesses.** Every cell's decision band is computed in Stage B from ≥3 draws and
  quoted with the price; a price inside it is `unpriced (within band)`.
- **The build recipe is the measurement.** Same absolute path, same flags, one codegen unit,
  private `CARGO_HOME`, offline, one cargo command at a time, `-j 8`; the 798 recipe of record is
  normative down to the argv and env.
- **No summing.** Each price is local to its cell; never add prices across cells or families.

## Checkpointing and hard rules

Write `PROGRESS.md` and a checkpoint receipt after every stage: the stage name, elapsed seconds,
the evidence paths, the printed line, and the next action. A stage that reaches its STOP rule
records the STOP verbatim and does not open the next stage unless the brief order says it is
independent. Keep `.temp/brief-797/` through `.temp/brief-801/` untouched; they are the reference
sources. The sealed report is `.temp/brief-802/report.json` (`rum-report/1`); the summary must be
the report's own `summary`. No tag, no network, no new dependency, no new `unsafe`, no `git`
command, `.temp` only, never `/tmp`. The live tree stays clean: `git status` empty at the seal.
