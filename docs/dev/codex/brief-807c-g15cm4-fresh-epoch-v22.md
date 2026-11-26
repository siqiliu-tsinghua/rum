# Brief 807c — G15-CM4 the fresh epoch under budget v2.2

Read briefs 798, 799 (Amendment 4), 804, 805, 806, 807, 807a and 807b first, and rulings 832,
844, 845, 846, 847. This is the D.2 line's final measurement gate, resumed after brief 807b's
Stage A2 STOP. The whole-panel captures admitted under the defective absolute 0.069595 s rule
are re-taken under the replaced prospective interference budget, now v2.2: the v2.1 partition
with the endpoint sampler made passive, so the instrument never signals the measured child.
Everything in briefs 807, 807a and 807b stands except where this brief changes it; where they
differ, this brief wins. No tag, no network, no new dependency, no new `unsafe` in
`rum-core`/`rum-runtime`. Scratch under `.temp/brief-807c/`; never `/tmp`. No git.
Measurement only: no live edit, no inventory replay, no release mutation, no acceptance suite.
The sealed STOP records at `.temp/brief-807/`, `.temp/brief-807a/` and `.temp/brief-807b/` are
read-only (copying their scripts, ELFs and receipts as pinned inputs is allowed; nothing there
may be modified).

## Why v2.2 (the 807b STOP, accepted as correct)

Brief 807b (unit `brief-807-cdx-20261126-171837-3893781`, 17:18:37–17:28) froze the v2.1
policy at `a3f80d1c…` before any output, re-verified all 229 pins and the three reused ELFs,
and ran the base arm's quiet whole-panel Callgrind calibration to
`BUDGET … verdict=REJECT E14=+0.465496 F14=+0.385496 max_window14=+0.031881 … cap_E=0.259136
cap_F=0.259136 cap_window=0.070000 T=47.827237` — one genuine cell, eight unattempted, zero
measurement draws — and stopped per the frozen failure rule. The STOP is correct. The cause is
my defect #33: the v2/v2.1 partition binds every endpoint of `meas` to `wait4(user+sys)`, and
wait4 can only be sampled by stopping the child, so the sampler SIGSTOPped the measured child
at every 100 Hz endpoint (4,782 observation stops, `stop_request_through_resume_s=0.398569 s`)
and booked its own stop windows as foreign CPU-14 time (`idle14=0` for the whole job). My
replication of the identical job: pause sampler `E14=+0.463976` (4,642 stops, wall 46.439,
child runtime 45.966024); passive sampler `E14=+0.079120` (no signals, wall 46.390, child
runtime 46.310880) — the latter equals 798's `other_busy[14]` 0.079–0.118 s band, inside the
0.259 s cap. `/proc/<pid>/schedstat` matches wait4 within 2 ms, so the child's own runtime is
sound; the pause footprint is the instrument's. Ruling 847 re-specifies v2.2.

The 807a Stage B is sound and is reused: three release builds at one path, one codegen unit,
offline — base ← real `dda9146e…` (ELF `524c2bc2…`), new ← armed `df0db63c…` (ELF
`2fa652ea…`), control ← real + the re-derived patch (ELF `c166a2ff…`; 712 B `8cfd8faa…`, one
changed file, one added `#[cold] #[inline(never)]` perturbator `brief795_layout_perturbator`,
one `lea` address reference, zero static calls); functional equality across 120 labels. 807b
re-verified all three byte-for-byte.

## The v2.2 policy (freeze before any output; quote its sha256)

Write `.temp/brief-807c/policy.json` as **exactly** the frozen 807b policy at
`.temp/brief-807b/policy.json` (`a3f80d1c…`), read-only, with these substitutions and no
others:

- `version`: `"rum-epoch-interference/2.2"`.
- `inputs`: keep the block, replacing the entry receipt path/hash with this attempt's own
  Stage A0 entry-pins receipt.
- `partition`: "busy_c = (user+nice+system+irq+softirq+steal) delta per CPU; irq_c =
  (irq+softirq) delta; steal_c = steal delta; meas = the measured tree's exact runtime: at
  each 100 Hz endpoint the sum of field 1 (runtime_ns) of
  /proc/<pid>/task/<tid>/schedstat over the measured process and every live descendant,
  read without sending any signal to any measured task; the terminal total is
  wait4(user+sys) of the measured child, which includes its waited-for descendants, and it
  must agree with the last schedstat tree sum within 2/HZ, else the capture is unavailable.
  E14 = busy14 - meas; E15 = busy15. F14 = busy14 - irq14 - steal14 - meas;
  F15 = busy15 - irq15 - steal15. Report every term with its raw fields and signed
  wall/tick remainders; report the unattributed remainder explicitly and never absorb it.
  Use all interval endpoints and the endpoints shifted by one second."
- `coverage`: add `"schedstat_missing_or_unreadable": "unavailable"` and
  `"measured_tree_vs_wait4": "unavailable if they differ by more than 2/HZ"`.
- `environment`: append "The sampler is passive: no signal is sent to any measured task; the
  measured child runs uninterrupted from exec to exit."
- All other keys unchanged, including the caps, the calibration block (nine cells on the
  whole-panel Callgrind job, 1.000 CPU-s spinner, release at 5.000 s, the failure rule), the
  prelaunch, the attempts, the orders, hardware, timing, sites, whole_profiles and inputs.

Freeze it (write, chmod 0444, sha256) and quote the hash in the report before the first
calibration result is read. A policy that differs from these substitutions is a STOP.

## The pins (re-verify every one before measuring; a stale pin is a STOP)

Brief 807's pin table verbatim, cross-checked against the 807/807a/807b receipts: the four 806
role roots (`cb9205a6…`/5,842, `dda9146e…`/5,851, `30cd48fc…`/5,851, `df0db63c…`/5,851), the
ten installed outputs (`d3588fb9…`, `1c48e15a…`, `99f7c5bb…`, `dc648d65…`, `8e29c41f…`,
`2ac559d1…`, `70a6da97…`, `feca9443…`, `c214a4aa…`, `44bb8f11…`), the 806 report `6e17e444…`
(check OK 1,284) and seal 158, the `R4 LIVE VERIFY PASS` line, the director's
`.temp/director/gate-806e/verdict.json`, the accepted ledger (1,743 body rows / 93 families;
the 38 publication gates at 1,464,055 Ir; the fastpath's 41,712 Ir inside the 51,790 band; the
joint price 13,556,424 with `additive_claimed=false`), the 795/796/799 instruments and the
control patch basis `04a56f39…`.

## Stage A0 — pins, build reuse, policy freeze and backend preflight (200 s)

Re-authenticate the pins above. Re-verify the 807a Stage B outputs and reuse them: base ELF
`524c2bc2…` (parent real `dda9146e…`), new ELF `2fa652ea…` (parent armed `df0db63c…`),
control ELF `c166a2ff…` (parent real `dda9146e…` + patch `8cfd8faa…`, 712 B, one changed
file, one added cold perturbator with one address reference and zero static calls), each
against its release receipt and the functional-equality receipts (120 labels). A mismatch
forces a rebuild exactly as brief 807's Stage B, with a disclosure; an unreconstructable
mismatch is a STOP. Write and hash policy.json. Backend preflight: both CPUs' `/proc/stat`
fields at 100 Hz for 1 s on CLOCK_MONOTONIC, every required column present, effective rate
≥ 50 Hz, max sample gap ≤ 0.100 s, `PRELAUNCH … quiet=true` lines for CPUs 14 and 15; plus a
passive-sampler preflight: spawn one 1-s CPU-burning child pinned to CPU 14, sample it with
the production sampler, and require the sampler's signal-call count to be zero, the tree sum
to track the child's burn, and `|tree_sum − wait4| ≤ 2/HZ` at exit. Quote the sampler's own
source in the report (the brief-local tool), and state the signal-call count.

## Stage A2 — the nine calibrations on the built ELFs (600 s)

Per arm, three whole-panel Callgrind calibration runs on that arm's ELF under the 796
full-panel options (v2.1 `calibration.job`; each with a fresh prelaunch screen and its own T):

- quiet → `CAL GREEN arm=ARM injected=none verdict=ADMIT`;
- a 1.000 CPU-s foreign spinner pinned to CPU 14, released at 5.000 s after launch →
  `CAL RED arm=ARM cpu=14 injected_cpu_s=1.000 verdict=REJECT`;
- the same spinner on CPU 15 →
  `CAL RED arm=ARM cpu=15 injected_cpu_s=1.000 verdict=REJECT`.

Each red cell must prove the spinner executed ≥ 0.990 CPU-s inside its 1-s window and that
the foreign time landed inside the measured job (the job's own T ≈ 48–50 s). Emit exactly
nine verdict lines (base, new, control), retain every raw sample stream and budget line, and
quote the per-cell T, E14/E15, F14/F15 and sliding maxima. The quiet cells must be measured
by the passive sampler and are expected at the historical baseline (E14 ≈ 0.08–0.12 s,
idle14 nonzero only if the host truly idled while the child ran); a quiet cell that shows a
pause-shaped footprint (E14 ≳ 0.01×T with `stop_request_through_resume_s > 0`) means the
sampler still signals — a STOP, not a retry. A missing, failed or unavailable calibration
stops deployment for director review — no measurement may start before nine genuine verdict
lines.

## Stage C — the qualified whole-panel captures and the timing panel (4,200 s)

As brief 807's Stage C, under the v2.2 budget:

- **Whole-panel hardware.** At least three admitted pinned draws per arm, interleaved
  `base/new/control`, `control/new/base`, `new/base/control`:
  `taskset -c 14 perf stat -x, -e cpu_core/instructions/u -o <event.csv> -- <arm ELF> --file
  <canonical.rum>`, each draw under the v2.2 budget (its own T ≈ 1.2 s; the measured tree is
  `perf` plus its child ELF; the declared 2/HZ allowances apply), ID-qualified, with the
  enabled/running scale (> 99% running), the binary and workload hashes, exits and raw counts
  retained. Emit all draws, min/max/median and the signed differences. The fresh envelope:
  with `B` the fresh base median, `new_median − B` must lie in
  `[min(control) − B, max(control) − B]`. The historical references (base median
  `16,069,730,355`, control `16,069,316,559`, old armed `+80,382,406`, and the 798 draws) are
  references only.
- **The 120-label panel.** The canonical panel on every arm with `--scale 1 --reps 7`, the
  exact approved sources and internal AbsoluteTiming, four independent triples in the fixed
  order `base/new/control`, `control/new/base`, `new/base/control`, `control/base/new`;
  CPU-pinned, serialized; the 795 reducer; 119 eligible labels and the 637 exclusion visible.
  No label is called recovered on timing alone.
- The whole-panel Callgrind profiles per the policy's `whole_profiles` (three draws per arm,
  nine pairings), each under the v2.2 budget.
- Every capture under the v2.2 budget; a quarantined capture is retained, retried only after
  a fresh qualification window, at most twice, with the rule fixed before any result is read;
  a capture still unavailable is a STOP for that capture.

## Stage D — the per-site prices, the four falsifiers and the attack surface (4,000 s)

As brief 807's Stage D: price all four StringReplace labels at N=50/100 with four paired
draws per arm in the four orders (24 captures per label, 96 for the four), per the 795
`probe-selection.json` recipes; compute the per-draw `(Ir100 − Ir50)/50` and its median;
report process and disjoint self/edge slopes, per-visit exclusive/inclusive quantities,
distinct PC rows and visits. The per-site bar (ruling 832): the unguarded pattern (1,3) =
16 Ir/iteration, guarded-hit / guarded-decline-fallback / single-literal-unguarded 8/8; exact
cardinality, 4 Ir per independent visit, 3 per proven live-address reuse, 0 for an eliminated
check. Reproduce the four 797 falsifiers bound to the final sources and ELF: the positive
undriven resume count, the ordinary zero cold self cost, the forced publication that restores
the slow work, and the driven ledger with balanced owners including the mid-request
Print-callback install.

## Stage E — the reconciliation and the fresh residual (2,500 s)

As brief 807's Stage E: the whole-panel Ir decomposition on all three arms joins every reached
hook and the four session facades through source, caller, visit count and inlined PC groups;
`new − base` and `control − base` reconcile to the raw totals with zero unexplained excess in
hardware and in Callgrind separately; each of the 1,743 body rows and 93 families maps to its
799/804/805 disposition; the 38 publication gates at 1,464,055 Ir and the fastpath's 41,712 Ir
are restated at the fresh residual; every family with an unexplained row or a moved cell is
named. The joint price 13,556,424 Ir is the accepted reference, `additive_claimed=false`; a
fresh number that differs is reported as data, never silently rebooked. The fresh residual
table gives, for each named family and for the two class-form decisions, the fresh
measurement, the disposition and the exact bar that would change it. **No acceptance is
taken here**: the two decisions return to the user.

## Stage F — verification, seal and report (800 s)

The `rum-report/1` report at `.temp/brief-807c/report.json` (`"brief": 807`, `"stage":
"807c"`): the policy hash, the nine calibration lines, the passive-sampler source and its
signal-call count, the reused builds and their receipts, the admitted and unavailable captures
with their budget lines, the panel and the envelope, the per-site prices and falsifiers, the
reconciliation and the fresh residual table, the charge; the seal of the brief's artifacts,
`python3 -I scripts/report.py check .temp/brief-807c/report.json`, the detached summary and the
exit audit (zero live changes, zero `.git-attempts`, zero unexplained).

## Charge

12,300 s of the 17,727.302 s pool: A0 200 / A2 600 / C 4,200 / D 4,000 / E 2,500 / F 800.
Every stage inside its box; a box that would overrun stops at the deadline with the exact
missing measurement. The 5,427.302 s reserve stays unallocated.

## STOP rules

Any stale pin or role mismatch; a policy deviation; a sampler that signals the measured child
or a quiet cell with a pause-shaped footprint; a failed or unavailable calibration; a
build-reuse mismatch that cannot be rebuilt; a capture the budget marks unavailable beyond its
declared retries; an unexplained whole-panel or Callgrind excess; a per-site price above its
declared bar or a falsifier that fails to fire; any live change; any git attempt. Q25's
thirteen named STOPs stand (P0214, P0216, P1070, P2348, P2354, P2379, P2381, P2385, P2387,
P2409, P2411, P2416, P2418); 744/745/746 held; CM5 unauthorized; no tag; no network; no new
dependency; no new `unsafe`; `.temp` only, never `/tmp`; no git.
