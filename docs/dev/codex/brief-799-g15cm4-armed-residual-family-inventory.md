# Brief 799 — G15-CM4: the armed residual family and the whole-panel capture qualification (design-only)

Status: design-only, `Scope: NONE — no source change`. Dispatched after ruling 827 (gate c175);
Amendment 1 after ruling 828 (gate c176); Amendment 2 after ruling 829 (gate c177); Amendment 3
after ruling 830 (gate c178). The first two attempts stopped in Stage A on defects in this brief,
not in the evidence: Amendment 1 fixed the
negative-family count, the unguarded label's two sites, the flag-formation PC count and a
presumed calibration; Amendment 2 fixes the three cost labels (the body rows are **self** Ir; the
inclusive costs are the incoming-edge sums), the allocation population ("the one unstable family"
is true only among the eight largest positive families), and a leftover "58 PCs" wording, and it
refines the STOP rule below. **Amendment 3 accepts the Amendment 2 attempt's Stage B
source-attribution STOP as correct, applies the four `brief_defects` it recorded, carries the
allocation family as measured non-systematic variation (removing its STOP power), and replaces
the all-nonzero-body assignment bar with the decision-relevant ≥10,000 bar.** Amendments are
marked "(Amendment 1)", "(Amendment 2)" or "(Amendment 3)" where they apply. All three attempts'
scratch is retained under `.temp/brief-799/` (the first attempt's `evidence-index.json`,
`entry-verdicts.log`, `pinned-files.log`, `index-final.log`, `bar-pin-lines.log`; the second's
`a1-*` pin checks and `e1-*` verification; the third's `a2-*` checks, logs and
`family-inventory.tsv`); you may re-verify and reuse it, but every pin you cite must be re-read.
Amendment 3's carve: B.4 200 s / C 700 s / D 1,000 s / E 400 s = 2,300 s.
Every statement here is a pinned input from the retained evidence; the STOP rule below governs
identity and decision-changing mismatches.

## Goal

Produce one reviewable, zero-unknown plan that (a) explains all 93 families and every body row
carrying a material difference (|Δ| ≥ 10,000) of the armed-vs-real residual (+37,644,521
translated Ir, +36,180,466 disjoint non-gate; +38,126,375 native), attributing each to its exact
source construct with `path:line` evidence and pricing it at its measured multiplicity, with the
remaining rows aggregated by family mechanism and their bound printed (Amendment 3); (b) classifies each family as a
default-path violation of the hard line, a required class-form cost, or a measurement artifact —
with source proof, never assumption; (c) adjudicates the `stringreplace-pattern-unguarded` result
(16 Ir per iteration against the written bar of 8) from the dispatch structure and states the
correct per-iteration budget rule; (d) diagnoses why the whole-panel captures qualified 0/3 and
designs a pre-registered, arm-identical replacement for the absolute 0.069595 s bound, with a
red/green falsifier; and (e) drafts the follow-up implementation briefs in `TEMPLATE.md` shape,
including each repair's isolation falsifier and the expected panel delta. You write no code, run
no build and start no capture.

## Context pointers

- Read `AGENTS.md` (hard lines) and this brief's evidence before forming any claim.
- Brief 798 (`docs/dev/codex/brief-798-g15cm4-repaired-whole-panel-ab.md`) and its Amendment 3
  paragraph; ruling 827 is `.temp/director/ruling-827.md`; the 796/797 briefs hold the gate's
  history.
- **The retained evidence (read-only, hashed):**
  - `.temp/brief-798/report.json` SHA-256
    `f281efa552833287103ec247cc3928b9a07d21a22803df4a5040f09721add08e` — the signed STOP report
    (`report.py summary 798` prints its structure; `show 798 --tag …` pulls a section).
  - `.temp/brief-798/logs/a3-disposition-final.log` `a5f94992…`, `a3-whole.log` `4041e291…`,
    `a3-family-proof.log` `a1effe76…`, `a3-family-pcs.log` `ba5bb028…`, `a3-slopes-final.log`
    `e225effb…`, `a3-census.log` `c23c1860…`.
  - `.temp/brief-798/evidence/amendment3/`: `evaluator-executed-pcs.tsv` (146,867 B),
    `evaluator-opcode-classes.tsv`, `quarantine-diagnostic-whole-bodies.tsv` (247,923 B, 1,743
    rows, sums +37,644,521 / −9,895,018 exactly), `quarantine-diagnostic-whole-families.tsv`,
    `residual-disassembly/` (24 body groups, 56 files; some arms have no body at a group),
    `whole-panel-*-try{1,2,3}.json` (nine
    receipts), `capture-plan.json`, `disposition-120.tsv`, `auxiliary-falsifiers.json`, and the
    rest of the 303-entry evidence directory.
  - `.temp/brief-798/callgrind/amendment3/` (144 label captures + 9 whole attempts; every raw,
    vglog, resolved and receipt file retained) and `.temp/brief-798/logs/` (every run log).
  - The three release ELFs: `.temp/brief-798/bins/base` `06d0d7b4…`, `bins/new` `2348c468…`,
    `bins/control` `2e092771…`, and the auxiliary consumers named in `auxiliary-falsifiers.json`.
  - The sealed role trees: `.temp/brief-797/amendment6/images/{baseline,real,disabled,armed}` with
    pins `ec602109…`/5,842, `022d60af…`/5,851, `c7156d50…`/5,851, `318933db…`/5,851; `armed` is
    the R4-certified class-form render and carries the seven R4 tables; the stale
    `.temp/brief-797/images/` root is forbidden.
  - The harness that enforced the capture rule: `.temp/brief-798/tools/a3_capture.py`,
    `a3_data.py`, `a3_whole.py`, `a3_localize.py`, `a3_family_proof.py` (read them; the
    replacement rule must be specified against their exact decision code).
  - The provenance of the 0.069595 s bound: the exact number occurs in the sealed trees only as
    the brief-622 workload log line
    `{"label": "replacepart-deep-path-small", "N": "check", "exit": 0, "elapsed_seconds": 0.06959539299714379, ...}`
    (e.g. `.temp/brief-797/amendment6/images/real/docs/dev/codex/brief-622-g15c1-replaceall-application-tail.md:4555`).
    Trace where the 796/797 harness adopted it and state, from evidence, whether it was ever a
    calibrated sibling-load bound and on which window shape; quote every provenance line you find.

## Scope

- **Files you MAY modify: NONE — design only.** You may create scratch files only under
  `.temp/brief-799/`; every command's output goes there or to `.temp/logs/` as a brief-local log.
- Files you MUST NOT touch: anything under `crates/`, `docs/`, `scripts/`, `.git` (including
  config), `Cargo.toml`/`Cargo.lock`, and every sealed/retained path above. Never write into
  `.temp/brief-797/` or `.temp/brief-798/`; read them only.
- No build, no `cargo`, no new capture, no perf/valgrind run, no network, no new dependency, no
  tag, no `git` command of any kind. The retained raw files are the only measurement evidence you
  may read.
- No new `unsafe` anywhere; `.temp` only, never `/tmp`.

## The measured inputs (restated; a mismatch is a STOP)

- Signed hardware: base median 16,067,360,789; control median 16,066,520,862 (delta −839,927);
  new median 16,105,487,164; `new_minus_base=+38,126,375`; interval
  `[control_min−base_median, control_max−base_median] = [−1,399,860, +3,517,248]`, width 4,917,108
  (= control spread); above the maximum by 34,609,127; every new draw above every base and control
  draw. Nine draws, 100.00% running, CPU 14.
- Translated whole-panel (population `quarantine-diagnostic`): base 16,464,292,914 / new
  16,501,937,435 / control 16,454,397,896 (attempt 1); new−base +37,644,521 = publication checks
  1,464,055 + disjoint non-gate 36,180,466; control−base −9,895,018; 1,743 bodies, 93 families,
  354 nonzero new rows. The 798 classifier's `unknown` field covers publication groups only (an
  unapplied family sum or an unproven reached publication site); causal source assignment is this
  brief's separate bar — Amendment 3, from the recorded defect and `a3_whole.py:44,127`.
- Gate: 38 reached sites, 2,594 references, 8 unvisited copies, 366,893 visits, 1,464,055 Ir
  (4.0/visit; `kernel_data/rewrite_data/cases/delete.rs:142` = 3). Short probes: 24/24 per label,
  `exclusive_Ir_per_visit=4` at `guarded/replace.rs:38` and `:775` in all draws, typed 4+3,
  slow/lexical self Ir 0, perturbator 0.
- Family leaders: `eval_core` +11,786,999 (`evaluate_inner_unstable` +11,795,606 self Ir over
  base/new/control visits 1,055,174/1,055,154/1,055,196, the two body rows +11,680,078 and
  +115,528; +11.19/visit on the displayed rates (249.25 → 260.44; the unrounded difference is
  11.183761852440 — 11.18 if rounded at the end; Amendment 3); opcode class "lexical profiler flag
  address formation" +14,124,585 vs control +33; flag-formation PCs 33/33/57 base/control/new per
  `residual-family-proof.json` `flag_formation_PCs`, i.e. 24 extra PCs in `new` — not 58: the 58
  is the `pattern-unguarded` absolute new price per iteration, 5 base + 53 delta (Amendment 1);
  qualified per-iteration prices +19/+19/+53/+21 vs both controls); `definitions` +9,725,686
  (`collect_upvalue_candidate_symbols` 665,503 new-only visits, **self** +52,180,871 Ir,
  incoming-edge **inclusive** 2,063,338,931 Ir from two recording contexts at one physical caller
  PC `0x965d0c` (caller symbols `try_apply_up_values` and `try_apply_up_values'2`) — Amendment 3);
  `allocation and reclamation` +7,482,626
  (range [−3,922,868, +8,302,460]; control [−8,648,385, −3,121,525]); `pattern_matching` +6,273,782
  (`BindingSnapshot::capture` 426,421 new-only, self +33,261,184, incoming-edge inclusive
  141,432,726 from two recording contexts at one physical caller PC `0xad2d1e` (caller symbols
  `match_compiled_owned` and `match_compiled_owned'2`) — Amendment 3); `scoping` +2,100,570
  (`resume_module_initializers` 5,157 new-only); the `drop_in_place<eval_set_cm740_driven::{{closure}}>`
  body 397,600 new-only visits (self +6,361,600, incoming-edge inclusive 14,313,600 from
  `association_lvalue_current` — Amendment 2); then `kernel_data` +1,277,282,
  `eval_dispatch` +548,784, `rum_stdlib::strings` +240,168, `drop glue` +203,558,
  `evaluation_cache` +138,487, `loader` +51,842, `session` +33,013, and the
  negatives led by `builtin_adapters` −2,050,856 and `alloc::raw_vec` −1,122,580 (−3.72M total).
  All `RUM_PROFILE_*` flags were 0 in every capture; no profiling was on.
- The nine whole attempts: each ~48.2–50.1 s at 100% on CPU 14; user 48.0–49.9 s, system
  0.055–0.080 s; `other_busy` 0.079–0.118 s against the fixed bound 0.069595 s; every pre-launch
  window idle before launch (control try3 consumed five, four at 0.08–0.21 s); decision basis fixed
  before any counter was read; all attempts retained.
- `stringreplace-pattern-unguarded`: two sites, visit multiplicities 1 and 3 per iteration
  (`site_visits_per_iteration=1,3`, paired visits 50/150), 4 Ir/visit, 16 per iteration against the
  written 8; four of four draws. The other three labels are 1,1 → 8. The label's timing verdict is
  `inside selection threshold`, `positive_separation=False`.

## Stage A — entry and evidence index (300 s)

Verify and index, read-only: the report hash and `report.py check 798`; the three ELF hashes; the
four role pins' counts (a full re-walk is not required — the 798 Stage A/E receipts and the report's
provenance carry them; cite them); the existence and headers of the six evidence files and the
nine whole receipts; the harness decision code lines. Produce `.temp/brief-799/evidence-index.json`
with every file, its SHA-256 (new files only; do not re-hash the sealed trees or the 144+9 raw
captures — cite the 798 receipts' hashes), and its role. STOP on any missing or mismatched pin.

## Stage B — the family inventory (1,100 s)

One table, `.temp/brief-799/family-inventory.tsv` (one row per family; one line of `path:line`
evidence per field), and the narrative in the report's `sections`:

1. **The evaluator (`eval_core`) family.** From `evaluator-executed-pcs.tsv`,
   `evaluator-opcode-classes.tsv`, `residual-disassembly/`, and the `real`/`armed` source trees:
   - identify the exact instruction groups composing the "lexical profiler flag address formation"
     class and their source constructs (the flag, its accessor, the branch, the inlined region);
   - explain the 33→57 PC difference: the same function body text
     (`function_sha256=002608d0…`) compiles with 24 extra flag-formation PCs in `armed`
     (Amendment 1); name the
     source/definition/attribute difference between the two trees that changed the inlining or the
     constant-folding (the class-form additions around `eval_core/mod.rs`; the lexical-profiler
     definitions; any `#[inline]`/`#[cold]`/statics/atomic initialization), with `path:line` and the
     disassembly rows that prove it;
   - state the per-visit (+11.19) and per-iteration (19/19/53/21) prices and the executed subset
     (3/3/5/5 base = 22/22/58/26 new absolute);
   - classify: is the executed flag formation a default-path cost of an off-by-default facility
     (hard-line violation) or a required semantic read? Quote the flag's definition and the code
     that consumes it; all `RUM_PROFILE_*` were 0.
   - enumerate the repair candidates with their exact source shape and falsifier: (i) the env-gated
     cold-load `AtomicU8` pattern; (ii) an out-of-line `#[cold]`/`#[inline(never)]` check; (iii) a
     const/static form the compiler can fold; (iv) hoisting the check out of the per-value path.
     For each: which of the 57 flag-formation PCs (24 of them extra in `new`) it removes, the
     expected per-visit saving, what it must not change, and how the falsifier shows the flag
     still works when set (Amendment 2).
2. **The new-only call-edge families.** For each of `definitions`
   (`definition_analysis_helpers::collect_upvalue_candidate_symbols`, 665,503), `pattern_matching`
   (`BindingSnapshot::capture`, 426,421), `scoping` (`resume_module_initializers`, 5,157) and the
   `eval_set_cm740_driven::{{closure}}` drop (397,600): the exact caller→callee edge
   (`path:line` at both ends), the caller's own visit count and whence it comes, why `real` and
   `control` execute 0, whether the edge is behind any flag or unconditional in `armed`, the
   per-visit inclusive/exclusive cost, and whether it is the class form's required semantic work
   (observation/driven evaluation) or an avoidable default-path cost — with the source proof.
3. **The allocation family — carried as non-systematic variation (Amendment 3).** The family's
   per-arm totals over the nine retained whole-panel attempts are base 890,743,346 / 889,934,889 /
   889,923,512; new 898,225,972 / 887,510,376 / 886,820,478; control 882,094,961 / 885,302,469 /
   886,801,987 (`all-attempt-family-contrasts.tsv`, the allocation row's `every_attempt` field;
   verify these nine numbers and the per-pairing range). Two of the three new attempts lie below
   every base attempt; the control arm — a one-file layout patch with no class-form source change
   — lies below both arms throughout; the new arm's own spread (11,405,494) exceeds the attempt-1
   new−base delta (+7,482,626); and the empty-head branch counts move by tens of thousands with
   identical request counts (11,321,292±600 calls, 39,023 requests >1,024 B in every run;
   `a2-allocation-conditions.log`, `a2-condition-proof.log`, `a2-allocator-branch.log`). The
   family is therefore **not a repair target and does not STOP this design**: carry the
   nine-pairing range [−3,922,868, +8,302,460] as a measured noise term into the repair plan and
   the re-measurement design, and do not re-open the ordered allocation/free trace here (the
   successor measurement is specified in the Amendment 2 report's allocation section; it would be
   commissioned only if the repaired panel's residual re-enters this family). The 19-of-92
   sign-crossing census stands (allocation is the only large positive one among the eight largest;
   the next-largest crossing families are `rum_core::expr` at [−310,866, +943,263] and
   `evaluation_cache` at [−1,549,867, +564,980]).
4. **The remainder — the decision-relevant assignment bar (Amendment 3; 200 s).** Assign every
   body row whose |`new_minus_base`| or |`control_minus_base`| is at least 10,000 — 112 rows over
   the 1,743-row table, carrying 37,659,664 / −9,973,234; verify both counts and both sums from
   `quarantine-diagnostic-whole-bodies.tsv` — individually to one of the constructs above, the
   carried allocation family, the loader, or a named measurement artifact, with `path:line` or a
   retained-evidence file:line — no anonymous balancing cell. Assign the remaining 1,631 rows
   (|Δ| < 10,000; sums −15,143 new and +78,216 control; max |Δ| 9,912) as an aggregate by family
   mechanism, printing the aggregate and its bound. State the result as `rows_individually=112
   rows_aggregate=1631 unassigned=0 dust_new=−15143 dust_control=+78216` (with the family table's
   `family_unassigned=0` over all 1,743 memberships), or STOP naming the exact missing measurement
   for any row ≥10,000 that cannot be assigned. The seventeen negatives stay named as in the
   Amendment 1 text (19 positive / 57 zero / 17 negative families; −3,722,338; the five hidden
   −1,183).

## Stage C — the bar and the capture rule (700 s)

1. **`stringreplace-pattern-unguarded`.** Read the recipe
   (`.temp/brief-798/callgrind/amendment3/stringreplace-pattern-unguarded-{50,100}.rum`) and the
   two sites (`crates/rum-stdlib/src/strings/patterns/search/rules.rs:59` and `:751`; the
   `guarded/replace.rs:38`/`:775` PCs belong to the two guarded labels, whose multiplicities are
   1,1 and whose per-iteration price is 8 — Amendment 1):
   show the exact statement(s) that produce the 1 and 3 visits per iteration, why the multiplicity
   is 3 (three matches), and that each visit pays the same 4 Ir. State the correct per-iteration
   budget rule (per-visit price × declared multiplicity, with the multiplicity derived from the
   recipe, not guessed) and the exact replacement wording for brief 798's "each StringReplace
   dispatch's two site visits ≤8 per iteration" line and its STOP-list phrase "8 per
   iteration/visit". Adjudicate the retained FAIL row: as written, the bar failed; under the
   corrected rule, state the verdict and its evidence. Do not pass the label silently.
2. **The qualification rule.** From the nine receipts and `a3_capture.py`'s decision code:
   - reconstruct the accounting: `busy − (user + sys)` per attempt, its rate per second of window,
     and the same ratio on the 144 short label captures; trace every retained adoption of the
     0.069595 s number (the 798 harness hard-codes it at `a3_entry.py:89` and `a2_report.py:82`;
     the only occurrence in the sealed trees is the brief-622 workload `elapsed_seconds` value),
     quote each provenance line, and state from evidence whether it was ever a calibrated
     sibling-load bound and on which window shape. The provenance section above is explicit that
     no 795 calibration exists in the retained evidence; if your searches find none, say so and
     name the searches rather than asserting one (Amendment 1);
   - decide from evidence whether the 0.079–0.118 s is kernel-side accounting of the capture itself
     or competing load (the pre-launch windows show episodic 0.08–0.21 s bursts on CPU 14, so
     competition exists at times; prove which component the during-run figure contains — if the
     retained data cannot distinguish, say exactly what sampler is needed and make it part of the
     replacement design, not an unproven claim);
   - design the replacement rule: pre-registered before any result, identical for every arm, that
     can admit an uncontended ~49 s capture and reject a contended one. Specify the exact quantity,
     threshold, sampling rate and decision order, the harness code points to change
     (`a3_capture.py` line numbers), and its calibration on the nine retained attempts;
   - ship the falsifier: a red calibration (a known-contended window — e.g. a 1 s spin on CPU 14 —
     must be rejected) and a green calibration (a quiet window admitted), with the exact recipe and
     the expected verdict lines. If the rule needs a per-pid sampler, specify it.
3. **Re-measurement design.** Specify the post-repair A/B: the fresh signed hardware envelope
   (nine draws, CPU 14, medians — the decision rule), the three qualified whole-panel captures
   under the new rule, the translated/native reconciliations, the 12 timing panels and the 144
   label profiles reuse/redo decision, and the disposition sheet. State the exact STOP conditions.
   The translated reconciliation must report the nine-pairing range (or the median pairing), never
   a single attempt pairing: the retained nine whole receipts give a translated panel delta range
   of +25,281,671 .. +39,027,090 across the new×base pairings (verify from the nine receipts'
   `total` fields), and the carried allocation family alone spans [−3,922,868, +8,302,460] over
   those pairings. State how the decision absorbs that term — the hardware envelope's medians are
   the decision; the translated decomposition reports the range — and do not re-open the allocator
   trace in this design (Amendment 3).

## Stage D — the classification completion, the repair plan and the drafted briefs (1,000 s)

1. **The classification completion (Amendment 3).** For `eval_core`, `definitions`,
   `pattern_matching`, `scoping` and the paired `drop` rows, finish the required-vs-avoidable
   classification with source proof:
   - the evaluator's four candidates from Stage B item 1: state the isolated same-profile build
     experiment that alone can price them (both arms built at one path with one codegen unit; the
     candidate shapes; the falsifier that the flag still works when set) — the Amendment 2 attempt
     established that neither a permanently-false atomic nor an unconditional out-of-line call is
     semantics-preserving, so the experiment is the deliverable, not an assumed saving;
   - what makes `collect_upvalue_candidate_symbols` and `BindingSnapshot::capture` standalone
     symbols in `armed` and inlined in `real`/`baseline` — the source or attribute difference with
     `path:line`, and whether restoring the inlined shape is semantics-preserving (the call counts
     per site are identical across arms; the deltas +8,972,494 and +6,273,782 are consistent with
     call overhead, but that is a hypothesis until the source proof);
   - the net of each re-billing pair (`scoping`/`builtin_adapters` −43,841; the paired closure
     drop +6,361,600 / −6,361,600 at 16 self Ir per drop), with proof;
   - the carried allocation family as stated in Stage B item 3 — no allocation repair is priced.
2. The repair plan, ordered by family, each with: the exact code shape to change (`path:line`), the
   "must not change" list (captured behavior, order of execution, observable rows, the gate's 4 Ir
   per visit, the typed 4+3, the falsifiers), the expected per-visit saving and its evidence, the
   interaction with the other families (e.g., whether removing the flag formation also removes an
   upstream driver of the new-only bodies), and the isolation falsifier — including the class-wide
   falsifier: the repaired armed build's whole-panel delta must land inside the repaired control
   interval, with the resumed path equal to the queued path (rum against rum).
3. The combined expected delta: new translated and native, over the nine-pairing range, with the
   residual that would remain if any family is a required class-form cost and the carried
   allocation noise term. If any family is required, state the exact user decision that would be
   needed — do not assume it.
4. Draft the follow-up implementation briefs in `TEMPLATE.md` shape (one per stage), naming files,
   functions, the gate tests, the standing-inventory replays (retention fingerprints, ownership,
   LP01, typed-slot, rejection census), the workspace gate, and the instruments' standing
   exclusions (the 637 label exclusion, the 0.069595 s provenance and its replacement). The first
   drafted stage is the evaluator's isolated same-profile experiment.

## Stage E — verification and handoff (400 s)

Re-read every cited line in its log (or the retained evidence file), check the inventory's row
counts and sums against the 1,743-row table, verify no live edit occurred (the tree is as found;
list any read-only scratch), run `python3 scripts/report.py check 799`, and write the handoff:
what is settled, what remains open with its exact next measurement, and the drafted briefs'
paths.

## Verification

- `python3 -B scripts/run-bounded.py --timeout 20 -- python3 scripts/report.py check 799` — every
  cited line verbatim, no FAIL hidden, the verdict agrees with the rows.
- The family rows' `new_minus_base` must sum to +37,644,521; the body assignment must cover every
  row with |Δ| ≥ 10,000 individually and the remaining rows as an aggregate with its bound, with
  the family table covering all 1,743 memberships and `family_unassigned = 0`; print the
  assignment line and the dust line (Amendment 3).
- No source change: `Scope: NONE`. If the plan requires a build or a capture, name it and STOP;
  the director owns that dispatch.

## STOP, with the fact named

- **STOP** on a missing, unverifiable or identity-changing input pin (a hash, count, file or
  verdict without which a stage's decision cannot be verified), on any mismatch that changes a
  bar or a decision, on any write outside `.temp/brief-799/`, on any live, role or sealed edit,
  on any `git` command, on any build, capture, perf or valgrind run, and on any network use;
- **record and correct** any mislabeled or misquoted non-identity quantity when the retained
  evidence gives the correct value and no bar or decision depends on the mislabel: proceed with
  the corrected value and list it in the report's `brief_defects` section (the brief line, the
  quoted claim, the corrected value, the evidence `file:line`). The worked examples are the body
  rows' `new_minus_base` values, which are **self** Ir (the inclusive cost is the sum of
  `inclusive_Ir` over the incoming call edges), and "the one unstable family", true only among
  the eight largest positive families; Amendment 2 pre-corrects these, but the rule covers any
  further instance (Amendment 2);
- any body row ≥10,000 or family that cannot be assigned to a construct from the retained
  evidence (name the exact missing measurement); the carried allocation family (Stage B item 3)
  is declared carried by this amendment and is not a STOP;
- any claim without `path:line` or a retained-evidence file:line;
- a classification that rests on assumption (e.g., "probably the profiler") rather than source
  proof;
- any attempt to make the panel pass by re-specifying the envelope, the bar or the qualification
  rule without the evidence and the decision stated above.

## Checkpointing

After each stage append to `.temp/brief-799/PROGRESS.md`: what is done and verified, the evidence
paths, the exact next bounded command, and the director handoff. Every long read or reducer runs
through `scripts/run-bounded.py` with a data-sized bound. The tree stays untouched.

## Report format

Write `.temp/brief-799/report.json` in the `rum-report/1` schema (`python3 scripts/report.py
schema`), one `sections` entry per stage with tags (`family-inventory`, `evaluator`, `call-edges`,
`allocation`, `bar`, `qualification`, `repair-plan`, `drafts`), a `brief-defects` section listing
every recorded-and-corrected non-identity mislabel (Amendment 2), every verdict line quoted
verbatim from a saved log, and every file listed in `changes` (scratch only). Your final message
is `python3 scripts/report.py summary 799` and nothing else.

Standing exclusions restated: the 637 `valueq-undefined-symbol-definitions-8000` exclusion is the
one visible exclusion of the 120-label panel; the 0.069595 s bound is the Amendment 3 rule under
repair; 744/745/746 stay held, Q25's thirteen rows stay named stops, CM5 has no authorization, and
the re-freeze, the fresh epoch and Stage C/C10 remain separately authorized and outside this
design.
