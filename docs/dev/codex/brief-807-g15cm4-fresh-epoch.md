# Brief 807 — G15-CM4 the fresh epoch: the qualified whole-panel measurement on the installed D.2 render

Read briefs 798, 799 (Amendment 4), 804, 805 and 806 first, and rulings 832, 844. This is the
D.2 line's final measurement gate: the whole-panel captures that were admitted under the
defective absolute 0.069595 s rule are re-taken under the replaced prospective interference
budget on the installed render, and the fresh residual is stated for the director and the user.
No tag, no network, no new dependency, no new `unsafe` in `rum-core`/`rum-runtime`. Scratch under
`.temp/brief-807/`; never `/tmp`. No git. Measurement only: no live edit, no inventory replay, no
release mutation, no acceptance suite.

## Why this brief

Ruling 832 replaced the qualification rule: the old rule subtracts wait4 user+sys from CPU-14
busy ticks against an absolute bound traced to brief 795's `0.1×T + 2/HZ` on one short draw
(`0.0695952211029362`), with no calibration behind it, and its admission of the 798 whole-panel
captures is not evidence. The replacement — a prospective interference budget with red/green
calibrations — was ruled to belong to the epoch brief, not to brief 800. The render is now
installed (brief 806 DONE, gate c192): the live crates equal the rendered armed image, the ten
inventory outputs are landed, and the four role trees are re-frozen. The epoch's inputs are
therefore the 806 render's roles and the 806 receipts, not the 797-era scene.

## The pins (verify every one before measuring; a stale pin is a STOP)

| Input | Pin |
| --- | --- |
| role `baseline` | `.temp/brief-806/completion/images/baseline` — `cb9205a6d119b8a1b49886bb05a6c50fc82621627791b29071f690694d4f2644`, 5,842 |
| role `real` | `.temp/brief-806/completion/images/real` — `dda9146ea7fca02c9ebd4eb89fadd42489a9780575d5982fc2e16e50f1886c29`, 5,851 |
| role `disabled` | `.temp/brief-806/completion/images/disabled` — `30cd48fc9730c8885705ada36654219388ba7bdc5b09dc6eef9196520e08bc89`, 5,851 |
| role `armed` | `.temp/brief-806/completion/images/armed` — `df0db63ce9645b63382ae15f577888e401826116c94bdd17ef4913957b5b962c`, 5,851 |
| the ten installed outputs | live == the produced pins of the 806 install: local-contracts `d3588fb9…`, promoted `1c48e15a…`, class-provenance `70a6da97…`, emission_chain `99f7c5bb…`, unknown_attributes `dc648d65…`, unknown_captures `8e29c41f…`, unknown_macros `2ac559d1…`, additions `feca9443…`, mutants `c214a4aa…`, overlays `44bb8f11…` |
| 806 report / seal | `report.json` `6e17e444…` (check OK 1,284), `finish-e/seal.json` 158 artifacts |
| 806 install / verify | the ten-row install receipt; `R4 LIVE VERIFY PASS mapping=190 … crates=b5440879a057/4700 content_delta=3` |
| the director's workspace gate | `.temp/director/gate-806e/verdict.json` and `workspace/run.json` |
| the accepted ledger | the 799/804/805 inventory: 1,743 body rows / 93 families; the 38 publication gates at 1,464,055 Ir (4 Ir/visit, one reuse at 3); the fastpath's 41,712 Ir inside the 51,790 band; the joint price armed 37,472,818 − real 23,916,394 = **13,556,424 Ir**, `additive_claimed=false` |
| the instruments | brief 795's `run/build/measure/hardware/probe/ir/reduce/verdict/pins.py`, the panel harness `02b9e0b5…`, the replay reducer `cb889245…`, `probe-selection.json` `e141a5ae…`; brief 796's full-panel tools; brief 799's capture accounting (`a4-capture-accounting.tsv`, `a4-qualification.md`) as the starting point for the budget machinery |
| the control patch | brief 795's `control.patch` `04a56f39…` |

The 798 capture protocol's Amendment 3 (the old busy-time bound, the quarantine rule) is
**superseded by this brief's Stage A budget**: no capture is admitted by the old rule; every
whole-panel and callgrind capture runs under the budget, and an unavailable budget is a STOP for
that capture, never a silent pass.

## Stage A — the qualification instrument and the calibrations (1,200 s)

Implement the prospective interference budget from ruling 832 as a brief-local instrument beside
the 795 tools, and freeze its policy file **before any output is viewed** (sha256 it and quote it
in the report):

- driver/sampler/reducer on CPUs 0–13; the measured process and threads pinned to CPU 14; both
  siblings' separate `/proc/stat` fields polled at 100 Hz on monotonic timestamps; lossless
  host-visible `sched_switch`, fork/exit and hard/soft IRQ entry/exit events on each sibling;
  intervals partitioned by IRQ first, then by scheduled measured/foreign task or idle; steal and
  the unaccountable remainder reported; full host visibility with zero lost events and known
  boundary owners required, else the capture is `unavailable`.
- Prelaunch: 1 s until both CPUs read ≤ `0.05×T + 2/HZ`. During a job: `E14 = busy14 −
  wait4(user+sys)`, `E15 = busy15`, each ≤ `0.005×T + 2/HZ`; scheduled foreign non-idle non-IRQ
  time `F` integral ≤ `0.005×T` and every sliding 1-s window ≤ `0.050 s`; identical limits for
  all arms.
- **The calibrations, per arm, before any measurement** (three whole-panel runs each: quiet; a
  1.000 measured CPU-second spinner on CPU 14 released at 5.000 s after launch; the same on
  CPU 15), with the exact lines
  `CAL GREEN arm=ARM injected=none verdict=ADMIT`,
  `CAL RED arm=ARM cpu=14 injected_cpu_s=1.000 verdict=REJECT`,
  `CAL RED arm=ARM cpu=15 injected_cpu_s=1.000 verdict=REJECT`.
  A failed calibration stops deployment of the rule for director review — no measurement runs.

## Stage B — the three arms (700 s)

Build at the one absolute source path `.temp/brief-807/active` with one external target, one
cargo command at a time, `-j 8`, exactly as brief 798's Stage B: **base** ← the 806 `real` tree
(`dda9146e…`); **new** ← the 806 `armed` tree (`df0db63c…`); **control** ← the 806 `real` tree plus
the 795 control patch (`04a56f39…`) in the brief-local copy. Byte-for-byte tree copies, re-hashed,
the sealed parent pins required; `cargo build --offline --locked -j 8 --release -p rum-cli --bin
rum-cli` with `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, a private `CARGO_HOME` from the locked
registry, the 1.85.0 toolchain ahead on `PATH`, identical flags and environment for all three
arms; save each ELF and its receipt; functional equality across the three arms on the canonical
program; prove the control perturbator is called zero times.

The 795 control patch predates brief 805's coldout refactor of
`crates/rum-runtime/src/runtime/profiling/timing.rs`, so its single hunk may not apply to the 806
`real` tree. Apply it with `git apply --check`-equivalent discipline; if the hunk does not apply,
re-derive the same shape at a cold site of the final file — exactly one changed file, one added
`#[cold] #[inline(never)]` perturbator, one cold `black_box` reference, no other byte changed —
declare the new patch with its path, bytes and sha256, and quote the derivation; if the shape
cannot be met on the final file, STOP. The proof of zero calls and zero self Ir to the
perturbator in the Stage D control profiles is required either way.

## Stage C — the qualified whole-panel captures and the timing panel (4,200 s)

- **Whole-panel hardware.** At least three admitted pinned draws per arm, interleaved
  `base/new/control`, `control/new/base`, `new/base/control`:
  `taskset -c 14 perf stat -x, -e cpu_core/instructions/u -o <event.csv> -- <arm ELF> --file
  <canonical.rum>`, each draw under the Stage A budget, ID-qualified, with the enabled/running
  scale (more than 99% running), the binary and workload hashes, exits and raw counts retained.
  Emit all draws, min/max/median and the signed differences. The fresh envelope: with `B` the
  fresh base median, `new_median − B` must lie in `[min(control) − B, max(control) − B]`. The
  historical references (base median `16,069,730,355`, control `16,069,316,559`, old armed
  `+80,382,406`, and the 798 draws) are references only.
- **The 120-label panel.** The canonical panel on every arm with `--scale 1 --reps 7`, the exact
  approved sources and internal AbsoluteTiming, four independent triples in the fixed order
  `base/new/control`, `control/new/base`, `new/base/control`, `control/base/new`; CPU-pinned,
  serialized; the 795 reducer; 119 eligible labels and the 637 exclusion visible. No label is
  called recovered on timing alone.
- Every capture under the budget; a quarantined capture is retained, retried only after a fresh
  qualification window, at most twice, with the rule fixed before any result is read; a capture
  still unavailable is a STOP for that capture.

## Stage D — the per-site prices, the four falsifiers and the attack surface (4,000 s)

As brief 798's Stage D with this brief's budget in place of Amendment 3: price all four
StringReplace labels at N=50/100 with four paired draws per arm in the four orders (24 captures
per label, 96 for the four), per the 795 `probe-selection.json` recipes; compute the per-draw
`(Ir100 − Ir50)/50` and its median; report process and disjoint self/edge slopes, per-visit
exclusive/inclusive quantities, distinct PC rows and visits. The per-site bar (ruling 832): each
reached site's declared visits per iteration from the recipe — the unguarded pattern (1,3) =
16 Ir/iteration, guarded-hit / guarded-decline-fallback / single-literal-unguarded 8/8 — exact
cardinality, 4 Ir per independent visit, 3 per proven live-address reuse, 0 for an eliminated
check. Reproduce the four 797 falsifiers bound to the final sources and ELF: the positive
undriven resume count, the ordinary zero cold self cost, the forced publication that restores the
slow work, and the driven ledger with balanced owners including the mid-request Print-callback
install.

## Stage E — the reconciliation and the fresh residual (2,500 s)

- The whole-panel Ir decomposition on all three arms: every reached hook and the four session
  facades joined through source, caller, visit count and inlined PC groups; `new − base` and
  `control − base` reconciled to the raw totals with zero unexplained excess in hardware and in
  Callgrind separately.
- The join to the accepted inventory: each of the 1,743 body rows and 93 families maps to its
  799/804/805 disposition; the 38 publication gates at 1,464,055 Ir and the fastpath's 41,712 Ir
  are restated at the fresh residual; every family with an unexplained row or a moved cell is
  named. The joint price 13,556,424 Ir is the accepted reference, `additive_claimed=false`; a
  fresh number that differs is reported as data, never silently rebooked.
- The fresh residual table: for each named family and for the two class-form decisions (the 38
  gates, the fastpath) the fresh measurement, the disposition and the exact bar that would
  change it. **No acceptance is taken here**: the two decisions return to the user.

## Stage F — verification, seal and report (800 s)

The `rum-report/1` DONE report for brief 807: the policy hash, the calibration lines, the builds
and receipts, the admitted and unavailable captures with their budget lines, the panel and the
envelope, the per-site prices and falsifiers, the reconciliation and the fresh residual table,
the charge; the seal of the brief's artifacts, `python3 -I scripts/report.py check 807`, the
detached summary and the exit audit (zero live changes, zero `.git-attempts`, zero unexplained).

## Charge

13,400 s of the 20,604.875 s remaining: A 1,200 / B 700 / C 4,200 / D 4,000 / E 2,500 / F 800.
Every stage inside its box; a box that would overrun stops at the deadline with the exact missing
measurement. The 7,204.875 s reserve stays unallocated. (The class campaign's Stage C/C10
readmission and the 744 restart follow this epoch's gate as a separate brief; they are not part of
this one.)

## STOP rules

Any stale pin or role mismatch; a failed calibration; a capture the budget marks unavailable
beyond its retries; an unexplained whole-panel or Callgrind excess; a per-site price above its
declared bar or a falsifier that fails to fire; any live change; any git attempt. Q25's thirteen
named STOPs stand (P0214, P0216, P1070, P2348, P2354, P2379, P2381, P2385, P2387, P2409, P2411,
P2416, P2418); 744/745/746 held; CM5 unauthorized; no tag; no network; no new dependency; no new
`unsafe`; `.temp` only, never `/tmp`; no git.

## Director gate (2026-11-26, gate c193)

The attempt stopped correctly at Stage A before any calibration or measurement: the policy was
frozen first at `5d0109b6…` (16:38, `calibration_runs=0 measurement_runs=0`), three fresh 100 Hz
prelaunch screens passed on CPUs 14/15, then every tracepoint attempt returned EACCES on all
seven IDs and `perf` exited 129. I reproduced the cause outside the sandbox as uid 1000:
`/sys/kernel/tracing` is 0700 root:root and `perf_event_paranoid=2`. Pins passed (229, drift 0);
the live tree is unchanged (5,891 files, entry/exit digest equal, zero git attempts); report
check OK 125 citations; seal 44 artifacts; charge 1,287.756 s (A 914.933/1,200, F 372.823/800);
pool 19,317.119 s. Ruling 845 strikes the event partition as unimplementable for this host
(brief defect #29) and corrects the stage order — the per-arm calibrations must follow the
builds they bind (defect #30) — and re-specifies the epoch budget as v2: `/proc/stat` per-CPU
fields plus the measured child's `wait4`, IRQ and steal separated, `E14`/`E15` ≤ `0.005×T+2/HZ`,
`F14`/`F15` ≤ `0.005×T`, sliding 1-s window ≤ `0.050+2/HZ`, 100 Hz coverage (≥50 Hz effective,
max gap ≤0.100 s, else unavailable), the same nine per-arm calibrations; every v1 admission
bound is retained conservatively (`F ≤ E` termwise). Brief 807a resumes the epoch at 13,100 s
(A 300 / B 700 / A2 600 / C 4,200 / D 4,000 / E 2,500 / F 800; reserve 6,217.119 s). The
two class-form decisions remain the user's at the fresh residual.

## Director gate (2026-11-26, gate c194)

Brief 807a stopped correctly at Stage A2: the base quiet cell ADMITTED (`CAL GREEN arm=base
injected=none verdict=ADMIT`), but the CPU-14 red cell could not fire because the calibration
job ran 1.067653 s while the fixed spinner release is at 5.000 s
(`INJECTION … in_job_lower_s=0.000000`); seven cells unattempted, zero measurement draws. The
attempt's Stage B is sound and reused: three release builds at one path with one codegen unit
(base ← real `dda9146e…`, ELF `524c2bc2…`; new ← armed `df0db63c…`, ELF `2fa652ea…`; control
← real + re-derived patch `8cfd8faa…`/712 B, ELF `c166a2ff…`), functional equality across 120
labels, zero static calls to the perturbator. Pins 229/0; live tree clean (5,892 files,
entry/exit digest equal); report check OK 236 citations; seal 9,399 artifacts; charge
1,041.477 s (A 144.552/300, B 344.676/700, A2 132.868/600, F 419.381/800); cumulative
10,124.358 of 28,400; pool 18,275.642 s. Ruling 846 accepts the STOP and corrects my v2
substitution (defects #31, #32): the calibration job is restored to the v1 text — the
whole-panel Callgrind job with the 796 full-panel options (T≈48–50 s), where the fixed 5.000 s
release and the fractional caps are meaningful — and the F integral caps gain the declared
`2/HZ` tick allowance (`F ≤ 0.005×T + 2/HZ`), matching the E, prelaunch and window caps.
Brief 807b resumes the epoch at 12,300 s (A0 200 / A2 600 / C 4,200 / D 4,000 / E 2,500 /
F 800; reserve 5,975.642 s).

## Director gate (2026-11-26, gate c195)

Brief 807b stopped correctly at Stage A2: the policy froze first at `a3f80d1c…` (0444, 17:21,
zero calibration and zero measurement runs), the 229 pins and the three reused 807a ELFs
verified, and the base arm's quiet whole-panel Callgrind cell (T=47.827237 s, 4,784 samples,
effective 100.006 Hz, max gap 0.0112 s, exit 0, 120 labels equal) returned
`BUDGET … verdict=REJECT E14=+0.465496 F14=+0.385496 max_window14=+0.031881 … cap_E=0.259136
cap_F=0.259136 cap_window=0.070000`; one cell failed, eight unattempted, zero measurement
draws. Scope: 5,893 live files entry/exit, equal digests, zero changes, zero `.git-attempts`;
report check OK 150 citations; seal 58 artifacts; charge 548.340 s (A0 159.764/200,
A2 103.474/600, F 285.102/800); cumulative 10,672.698 of 28,400; pool 17,727.302 s. Ruling
847 accepts the STOP and names my defect #33: the v2/v2.1 partition binds every endpoint of
`meas` to `wait4`, which can only be sampled by stopping the child — 4,782 observation stops,
`stop_request_through_resume_s=0.398569 s`, `idle14=0` — so the instrument booked its own
pause windows as foreign CPU-14 time. The identical job replicated under two samplers gives
pause `E14=+0.463976` (4,642 stops) and passive `E14=+0.079120` (no signals), the latter equal
to 798's `other_busy[14]` 0.079–0.118 s band and inside the cap; `schedstat` matches wait4
within 2 ms. Budget **v2.2** makes the sampler passive: `meas` at each 100 Hz endpoint is the
sum of `/proc/<pid>/task/<tid>/schedstat` field 1 over the measured tree (the hardware draws
measure `perf` plus its child), the terminal total is `wait4`, agreement within `2/HZ`
required or the capture is `unavailable`; every cap, coverage rule, calibration and stage
stands. Brief 807c resumes the epoch at 12,300 s (A0 200 / A2 600 / C 4,200 / D 4,000 /
E 2,500 / F 800; reserve 5,427.302 s). The two class-form decisions remain the user's at the
fresh residual.

## Director gate (2026-11-26, gate c196)

Brief 807c stopped correctly at Stage A2. The v2.2 policy froze first at `0e146abe…` (0444,
17:44:31, zero calibration and zero measurement runs; parent `a3f80d1c…`, only the briefed
substitutions), the 229 pins re-verified with 0 missing and 0 bad, the three reused 807a ELFs
byte-equal, the passive-sampler preflight green (`signal_call_count=0`, tree-vs-wait4 5.2 ms),
and two cells ran: the base quiet whole-panel Callgrind cell admitted at
`E14=+0.091700 F14=+0.031700 max_window14=+0.019245 … cap_E=0.249348 cap_F=0.249348
cap_window=0.070000 T=45.869501` — 798's `other_busy[14]` 0.079–0.118 s band, so v2.2 is
validated — and the base CPU-14 injection cell rejected at `E14=+1.087461 F14=+1.017461
max_window14=+0.511799 … cap_E=0.256189 cap_F=0.256189 cap_window=0.070000 T=47.237817`,
but the brief's one-second injection proof failed: the spinner's certified burn is
1.000000203 CPU-s over 2.000042858 wall seconds, wholly inside the job (+5.000445 s to
+7.000488 s of 47.237817 s), while the fixed window [+5.000, +6.000] holds only
0.499533668–0.499535129 CPU-s against the required 0.990. One genuine cell, one failed, seven
unattempted, zero measurement draws. Scope: 5,894 live files, entry and exit digest
`e2cdcc2a…`, zero changes, zero `.git-attempts`; report check OK 188 citations; seal 82
artifacts; charge 547.312 s (A0 140.671/200, A2 124.398/600, F 282.243/800); cumulative
11,220.010 of 28,400; pool 17,179.990 s. Ruling 848 accepts the STOP and names my defect #34:
the measured job and the spinner are two runnable tasks on one CPU, so CFS shares it at about
one half each and no one-second window can hold 0.990 CPU-s of the burn — the rule was
unsatisfiable by construction (the disputed second's witness: 0.493287–0.504824 s of measured
tree plus 0.4995 s of spinner). Budget **v2.3** replaces only the calibration proof: certified
total burn ≥ 0.99, containment inside the job, E ≥ 0.97 and F ≥ 0.90 on the injected CPU, the
sliding one-second maximum ≥ 0.25, the verdict REJECT; everything else stands. Brief 807d
resumes the epoch at 12,600 s (A0 200 / A2 900 / C 4,200 / D 4,000 / E 2,500 / F 800; reserve
4,579.990 s; no top-up), carrying the 807c measurement path byte-identically. The two
class-form decisions remain the user's at the fresh residual.

## Director gate (2026-11-26, gate c197)

Brief 807d stopped correctly at Stage A2 on the base quiet cell. The v2.3 policy froze first
at `0c8d99bc…` (0444, 17:59:25, zero calibration and zero measurement runs; the only deltas
against 807c are `version`, `inputs` and the briefed `calibration` substitution), the pins and
the three reused 807a ELFs re-verified, the 807c measurement path carried byte-identically
(the calibration driver's diff confined to the red proof block), and the cell returned
`BUDGET … verdict=REJECT E14=+0.813502 F14=+0.753502 max_window14=+0.040190 … cap_E=0.255253
cap_F=0.255253 cap_window=0.070000 T=47.050509` with 4,705 samples at 100.020 Hz, exit 0, 120
labels equal, zero signal calls, zero observation stops and tree-vs-wait4 agreement 0.91 ms;
one cell failed, eight unattempted, zero measurement draws. Scope: 5,895 live files, entry and
exit digest `854ab241…`, zero changes, zero `.git-attempts`; report check OK 197 citations;
seal 73 artifacts; charge 592.989 s (A0 147.542/200, A2 93.219/900, F 352.228/800); cumulative
11,813.008 of 28,400; pool 16,587.001 s. Ruling 849 accepts the STOP: the instrument is not at
fault — director replications of the identical job give `E14` +0.091700 (the 807c cell),
+0.813502 (the 807d cell), +1.093822 and +2.398637 in two windows at 18:13–18:16, and
+0.103140, +0.112114, +0.107107 in three windows at 18:26–18:31, with a thread-level watcher
seeing no other task running on CPU 14 in the quiet windows. The host alternates between the
historical baseline and transient excursions while the desktop and agent sessions are active,
so a single-shot 47-s quiet cell is a coin flip. Budget **v2.4** adds only a declared quiet-cell
retry rule: at most three attempts, fresh prelaunch each, every attempt retained, retry only
on a REJECT with zero signal calls and zero stops, tree-vs-wait4 within `2/HZ`, exit 0 with 120
labels, and a pure integral excursion (E and/or F above cap while both sliding maxima stay
under the window cap); a pause footprint, any other failure, or three contaminated attempts
stays a STOP; red cells remain one-shot. Brief 807e resumes the epoch at 13,200 s (A0 200 /
A2 1,500 / C 4,200 / D 4,000 / E 2,500 / F 800; reserve 3,387.001 s; no top-up), carrying the
807c/807d measurement path byte-identically. The two class-form decisions remain the user's at
the fresh residual.
