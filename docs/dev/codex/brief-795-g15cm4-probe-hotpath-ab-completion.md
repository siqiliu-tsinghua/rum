# Brief 795 — G15-CM4 the probe and hot-path A/B completion on the final release

**Diagnostic measurement brief (`Scope: .temp/brief-795/` only).** One gate. It continues ruling
808 (gate c156) and brief 794's stage-A bar. The released source is read-only; this brief measures
the ordinary path of the final armed image against the final real image and decides whether the
armed render adds ordinary-path work.

## Goal

When this brief is done, in `.temp/brief-795/` only:

- The **structural closure reproduced**: the four sealed role pins, `COMPARE` with `ws=0`,
  `HOT MATCHED covered=21 unmatched=0 missing=0`, the four `session.rs` facade hits, the three
  attributed verbatim probe rows, `ATTRIBUTION rows=120 … unexplained=0` and
  `PROBE CLOSURE facades=4 outside=3 attributed=3 unexplained=0`, all from brief 794's driver on
  the same bytes.
- **Three release binaries at one stable path**, each built with one codegen unit and a complete
  receipt: real (base), armed (new), and the layout control (the real tree plus one
  behaviour-neutral, cold-referenced perturbator in a scratch copy only).
- **The measured A/B**: the canonical panel generated once and replayed byte-identically on every
  arm; four interleaved passes per arm (twelve panels); every `timings.tsv` and raw capture; the
  per-label tier-median quotients; whole-panel `perf stat` instruction counts (3+ per arm); and,
  for every label the instrument must settle, the 703-style Callgrind Ir slope probe at N=50/100.
- **The verdict**: one disposition per label, the control envelope, the ordinary-path cost
  result and zero unexplained deltas; all four images re-pinned at exit; the report
  (`rum-report/1`).

## Context pointers

- Brief 794 and its `evidence/stage-a-bar.md`, `evidence/attribution.tsv`,
  `evidence/probe-attribution.tsv`, `evidence/attribution.md` and `logs/amended-verify.log`, all
  under `.temp/brief-794/`; rulings 806, 807 and 808.
- Brief 781 (`docs/dev/codex/brief-781-g15cm4-probe-hotpath-ab.md`) for the three-arm A/B design;
  brief 637 (continuation 13 ruling 3) for the interleaved order and continuation 8 ruling 3 for
  the degenerate label; brief 703 and 708 for the timing reducers, the layout control, the
  corrected 703 criterion and the Callgrind Ir slope settlement.
- The sealed inputs `.temp/brief-793/images/{baseline,armed,disabled,real}`; the pinned release's
  `scripts/core-language-microbench.sh` (inspect its actual panel and emission fields).
- The 794 instrument's inputs are read-only and must stay byte-exact: `.temp/brief-781/stage_a.py`
  (`b342b07f…`) and `report.json`, `.temp/brief-790/frame-delta.tsv`,
  `.temp/brief-791/manifest.tsv`, the four images, and the 794 driver, tools and evidence. If any
  is absent or altered, STOP.

## Scope

- You MAY write only under `.temp/brief-795/`: `active/` (the one build path), `target/`
  (external Cargo targets), `tmp/`, `bins/`, `logs/`, `panel/`, `callgrind/`, `evidence/`,
  `PROGRESS.md`, `report.json`.
- MUST NOT touch: live, the four 793 images, brief 794 and its evidence, the sealed tables,
  `.git`, `Cargo.lock`, any `docs/` file. No network, no new dependency, no new `unsafe`, no
  default-path cost in live.
- The single source perturbation is permitted only in a scratch copy of the real tree under
  `.temp/brief-795/`. No workspace suite, no epoch, no standing-inventory replay (the released
  source is read-only).

## Stage A — entry and the structural closure (700 s)

From the repository root, capture the full output and exit status of

```sh
python3 -B scripts/run-bounded.py --timeout 700 -- python3 -B .temp/brief-794/driver/classify_final.py verify
```

Archive every diff, extension, probe and hot-match line. These are the measured lines on the
pinned trees; require them verbatim:

```text
ENTRY TREE role=baseline sha256=fa01b3d1e38574a78d6fb9c4cf80bf36e32470d294646b28aabede99c669be80 files=5842 debris=0
ENTRY TREE role=armed sha256=c04bbf8245a8b41c6b069f36df0c2da3785f25e7565a6938e2dff29d77a14afb files=5851 debris=0
ENTRY TREE role=disabled sha256=60ad65edb34b7ce83c5b924c63b308b0c1f4b5c5d1316125efce976312038610 files=5851 debris=0
ENTRY TREE role=real sha256=d703b1655dc3d2c5ad8cdceaf0b285a1d1261c1742ddf736b5144852a1ac7503 files=5851 debris=0
COMPARE verbatim=13581 ws=0 wrapper=51 dispatch=122 cold=334 facade=4 extension=25 diff=95
files=1257 diff_rows=95 probe_hits=7
PROBE crates/rum-host/src/session.rs cm740_enter
PROBE crates/rum-host/src/session.rs cm740_ordinary
PROBE crates/rum-host/src/session.rs cm740_hit
PROBE crates/rum-host/src/session.rs cm740_driven
PROBE VERBATIM-PROBE crates/rum-stdlib/src/tests/ownership_graph/host_payload_ledger.rs check_entry
PROBE VERBATIM-PROBE crates/rum-stdlib/src/tests/ownership_graph/reconcile.rs is_local_construct
PROBE VERBATIM-PROBE crates/rum-stdlib/src/tests/ownership_graph/reconcile.rs reconcile_falsifier_disposition
COLD TWINS count=334
HOT MATCHED covered=21 unmatched=0 missing=0
ATTRIBUTION rows=120 reported_rows=120 diff=95 extension=25 shapes=26 sha256=280e9ee3141d2a345261a4c8eceb4b1b91339e2889f412a7a19dd520da821886 unexplained=0
PROBE CLOSURE facades=4 outside=3 attributed=3 unexplained=0
C94 HOT PATH PASS arm=armed files=1257 cold_twins=334 covered_hot_functions=21 accepted_nonhot_shapes=95 unexplained=0
```

Require every hot name and occurrence matched, zero whitespace-only rows, exactly the four
callable session facade sites, and an exact evidence multiset for every additional verbatim probe
hit. The residual multiset includes distinct same-named occurrences. Counts describe the observed
pair; an equal count cannot substitute for an equal multiset. Hash-compare live's `crates/` subtree
to the R4 identity (``a7de3a76b780be90b85d1aad5d51e531784dec1122789de760517b8b1f86ce85`, 4,700
files) at entry and again at exit. STOP on any pin or tool mismatch, any missing or unmatched hot
occurrence, any whitespace-only row, a missing facade, an extra unbound hit, a missing or forged
witness, or a nonzero unexplained count.

## Stage B — three release binaries at one stable path (1,200 s)

Build in order. **Base** is real. **New** is armed. **Control** is real plus a single
behaviour-neutral, cold-referenced perturbator. Copy each *complete* tree to the same absolute
`.temp/brief-795/active/` in turn, excluding `.git`, `target`, `.temp`, `__pycache__` and
`.mypy_cache`. Re-hash the staged source with the 794 tree algorithm and record its file count
before every build; verify the base and new digests against Stage A. For control, record its own
complete hash, real's parent hash, the full one-file diff and before/after file hashes.

Use the workspace Rust toolchain. Set `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, an external
`CARGO_TARGET_DIR`, `CARGO_NET_OFFLINE=true` and `TMPDIR` under brief 795. Build from the same
source pathname and a fresh target at the same absolute target pathname for each arm, with

```sh
cargo build --offline --locked -j 8 --release -p rum-cli --bin rum-cli
```

The control's perturbator lives in `crates/rum-runtime/src/runtime/profiling/timing.rs`. Inside
`runtime_path_timing_start_slow`, take and black-box the function pointer
`brief795_layout_perturbator as fn(usize) -> usize`; the private
`#[cold] #[inline(never)]` function returns
`std::hint::black_box(value).wrapping_mul(17).rotate_left(7)` and is never called. Save the complete
source patch, the exact reference site and the linked symbol/address or disassembly showing
retention, address use and a layout difference. Prove zero invocations from source, disassembly and
the measured profiles (and a zero counter if the chosen control has one). The reference is confined
to the profiling slow path. Prove byte-identical functional panel output under base and control:
keep raw stdout/stderr and compare every non-clock byte, including label order and iteration counts.
For a deterministic output control, run the same panel bodies and setup with only the emitted
seconds field fixed at its source, save that control program's digest and compare the complete
output streams byte for byte.

For each arm save `bins/<arm>`, the binary sha256, source sha256, file count, absolute stable
source and target paths, the Rust compiler identity, the actual cargo argv and environment,
`profile=release`, `codegen_units=1`, exit status and monotonic elapsed seconds. Verify each saved
binary's bytes before its first run. STOP on a stale or incomplete receipt, a build at another path
or profile, a missing layout perturbation, an executed perturbator, an output difference, or source
drift.

## Stage C — the measured A/B (2,200 s)

Pin one fixed CPU of PMU type `cpu_core` in the process affinity mask; record the affinity, actual
CPU, PMU event, idle checks and command environment. Run serially and alone with stdin closed.
Clear inherited `RUM_PROFILE_*`, CM740 drive/control and campaign instrumentation variables before
every arm, and preserve the panel's own profiling-off settings. Stop measuring if another
CPU-heavy job competes. Build and capture in separate phases.

1. **One panel.** Hash the release microbench script. Generate the canonical
   `core-language-microbench.rum` once through that script in the first base pass with
   `--scale 1 --reps 7 --output-dir <brief-pass-dir> --rum-cli <absolute-bins/base>`. Pin
   `RUM_CORE_MICRO_RETAINED_MODULE_GC_BATCHES` identically across arms. Keep the canonical source
   and its sha256. The stock script emits its here-document on each invocation; for replay, stage
   an otherwise byte-identical scratch copy of the script whose single emission step copies the
   pinned program into the pass directory, preserving its engine invocation, bounded runner,
   environment and TSV extraction. Keep the original and the exact harness diff, and assert each
   program's hash before the engine consumes it. Check each recorded binary is executable before
   invoking the script so its rebuild fallback cannot fire. Use that pass's `--rum-cli` receipt
   binary and a distinct `--output-dir`. Measured durations are the kernel's monotonic
   `AbsoluteTiming` results.
2. **Four interleaved timing passes per arm** in this order; save every `timings.tsv`,
   `measurements.tsv`, run record and raw stdout/stderr, and enforce identical source bytes, label
   sets, iteration counts, benchmark results and successful exits across all twelve panels.

   | Pass | First | Second | Third |
   | --- | --- | --- | --- |
   | 1 | base | new | control |
   | 2 | control | new | base |
   | 3 | new | base | control |
   | 4 | control | base | new |

3. **Timing reduction.** Retain each label's four in-kernel min-of-seven samples under every arm,
   with both its minimum and median across those four. For each label compute new/base and
   control/base using like statistics. Assign each label to a tier by its base median reported
   `rum_seconds` duration: below 50 μs, 50 to below 200 μs, 200 μs to below 1 ms, and 1 ms or
   more. Compute each tier's median of the per-label quotients. Record all labels, the hot labels
   and their same-tier neighbours. Record the 637 exclusion
   `valueq-undefined-symbol-definitions-8000` (its N does not drive work), with its presence or
   absence; retain its raw row if present and exclude it from ordinary-panel gating and tier
   decisions.
4. **Hardware instructions.** Use `perf stat -x, -e cpu_core/instructions/u` on a complete run of
   the pinned program under each saved binary, with `--file`, the same CPU and stdin at EOF.
   Capture at least three independent successful counts per arm; record the reason and bound for
   any additional draw. Keep each raw count, event and CPU, elapsed time, exit status, workload and
   binary sha256. Record the control's instruction spread. For the timing envelope report the
   median, p90 and maximum of `abs(control_median/base_median − 1)` over the eligible panel; use
   the measured maximum for the 708 panel-wide positive-movement comparison.
5. **Settle every material label.** Flag every outside-envelope positive timing movement, every
   positively separated distribution and every label implicated by conflicting instruction
   evidence; record how each was selected and all its same-tier neighbours. For each selected label
   extract its exact body and required setup into two `.rum` programs at N=50 and N=100, with the
   same warmup and no added per-iteration sink; record their hashes. Run Callgrind under base, new
   and control, four independent draws per (arm, N) cell — 24 profiles per selected label. Preserve
   total Ir, per-function **self** Ir, resolved objects/symbols and the raw files for each draw.
   For both total and self counts take `(Ir100 − Ir50) / 50` per paired draw. Report all slopes,
   the median and spread under all three binaries, per-function deltas, and fixed intercepts
   separately. Attribute process/hash variation against control and resolve changed functions;
   retain call identities, including abbreviated Callgrind function IDs. Relative offsets are
   decimal bytes and `*` repeats the previous address.

## Stage D — the verdict, the cleanup and the report (300 s)

Apply the corrected 703/708 criterion. Timing is secondary. A positive timing movement is a cost
finding when it exceeds the control envelope **and** the instruction evidence shows added
ordinary-path work; a within-envelope movement with equivalent executed instructions is noise. For
an outside or separated movement, cite its three-arm slope and self-Ir evidence before calling it
noise. An attributable new ordinary-path instruction cost stops the brief regardless of timings
elsewhere. A whole-panel total cannot settle an individual label whose slope remains open. Print
one disposition per label, the envelope, the probe-set coverage and a terminal verdict stating the
ordinary-path cost result and the unexplained count.

Retain receipts, original captures, every timing TSV, the reduction, the label profiles and the
verdict. Clean only the disposable `active/` sources and external build targets after preserving
the control diff and receipts. Re-hash baseline, armed, disabled and real with the 794 algorithm,
recheck every binary digest, and establish live's `crates/` identity against R4
(`a7de3a76b780be90b85d1aad5d51e531784dec1122789de760517b8b1f86ce85`, 4,700 files). Verify no
cache, target or scratch debris inside the sealed trees; quote the four final tree lines.

## Verification and acceptance

A pass requires the 794 structural closures, three complete one-codegen-unit release receipts, the
control's retention and non-execution, byte-identical functional output, four passes per arm on
identical workload bytes, every tier and label quotient, at least three hardware counts per arm,
and every required 50/100 probe. Every movement must be explained by the control or settled by the
instruction instrument. Close with no new ordinary-path cost, zero unexplained deltas, and all four
sealed role identities intact. Record any `perf` or Callgrind access failure verbatim and STOP at
the first refusal of a required instrument. A timing-only verdict is inadmissible.

## Checkpointing and STOP rules

After A, B, C and D, update `.temp/brief-795/PROGRESS.md`: completed proofs, saved evidence, the
state of all four images, the exact next command and the director handoff. Keep failed attempts
with their outputs; on resume reuse valid receipts and completed captures. Bound every long or
child-spawning command with `scripts/run-bounded.py --timeout SECONDS -- …`, sizing limits from the
previous same-shape receipt and the stage carve.

STOP, with the fact named, on: any Stage A pin, tool or closure mismatch; any Stage B condition
above; contaminated or incomplete measurements; a missing required profile; an unresolved
attribution; a new ordinary-path cost; a refused `perf`/Callgrind; changed release or live bytes;
any write outside `.temp/brief-795/`; any refused git attempt (disclose it even though the fence
refuses it); or a projected overrun.

## Hard rules

- Do NOT `git commit`, branch, push or touch git config; the fence logs every attempt and the
  report discloses it. The director owns git.
- No network, new dependency, new `unsafe`, live source edit, sealed edit, inventory hand edit.
  Preserve brief 780's checker, brief 794's tools and evidence, and all earlier trees.
- `.temp` only, never `/tmp`; never read `~/.config/ccds/api-key`.
- This is a rum-vs-rum measurement with workspace suites and epoch runs outside scope. Q25's
  thirteen rows stay named stops; briefs 744/745/746 remain held; CM5 is unauthorized. The
  re-freeze, the fresh epoch, Stage C/C10 and any later 744 restart require their own gates.

## Budget

Focused **4,400 s** (A 700 / B 1,200 / C 2,200 / D 300) + workspace 0 + epoch 0 = **4,400 s**.
The 2,665 s returned by brief 794 plus an explicit **1,735 s** top-up under the standing
2026-11-19 ruling fund it → reserve 0 at the carve. Unspent returns; a projected overrun stops for
a director ruling, never a scope cut.

## Report

`rum-report/1` at `.temp/brief-795/report.json`; every attempt with its actual argv, exit status,
receipts, checks and measurements; the observed counts, scope, exclusions and any refused git
attempt; verbatim verdict lines from saved brief-local logs, including STOP evidence. Run
`python3 scripts/report.py check 795` before handoff.
