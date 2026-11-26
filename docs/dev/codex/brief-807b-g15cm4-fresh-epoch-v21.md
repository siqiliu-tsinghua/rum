# Brief 807b — G15-CM4 the fresh epoch under budget v2.1

Read briefs 798, 799 (Amendment 4), 804, 805, 806, 807 and 807a first, and rulings 832, 844,
845, 846. This is the D.2 line's final measurement gate, resumed after brief 807a's Stage A2
STOP. The whole-panel captures admitted under the defective absolute 0.069595 s rule are
re-taken under the replaced prospective interference budget, now v2.1: the v2 partition with
the calibration job restored to the Callgrind whole panel and the F caps given their declared
tick allowance. Everything in brief 807 stands except where this brief changes it; where they
differ, this brief wins. No tag, no network, no new dependency, no new `unsafe` in
`rum-core`/`rum-runtime`. Scratch under `.temp/brief-807b/`; never `/tmp`. No git.
Measurement only: no live edit, no inventory replay, no release mutation, no acceptance suite.
The sealed STOP records at `.temp/brief-807/` and `.temp/brief-807a/` are read-only (copying
their scripts, ELFs and receipts as pinned inputs is allowed; nothing there may be modified).

## Why v2.1 (the 807a STOP, accepted as correct)

Brief 807a (unit `brief-807-cdx-20261126-165718-3881870`, 16:57:18–17:14:45) ran the base
arm's quiet calibration to `CAL GREEN arm=base injected=none verdict=ADMIT`, then could not
fire the CPU-14 red cell: the calibration job was the native hardware-draw command minus
`perf`, which runs 1.067653 s, while the fixed spinner release is at 5.000 s
(`INJECTION … workload_s=1.067653 release_offset_s=5.000 actual_release_s=5.000242 … 
in_job_lower_s=0.000000`), so the required injected REJECT was absent and the attempt stopped
with seven cells unattempted and zero measurement draws. The fixed 5.000 s offset, the 1-s
window and the fractional caps were designed for the whole-panel Callgrind job (~48–50 s;
799's a4 receipts show nine ~48.2–50.1 s attempts); my v2 substitution moved the job to the
hardware-draw shape (1.18–1.26 s in 798) — brief defect #31 — and left the F integral cap
without the tick allowance, 5.3 ms at T≈1.2 s, below one 10 ms tick; the quiet cell passed by
0.6 ms (F14=+0.004670 vs cap_F=0.005292) — defect #32. Ruling 846 re-specifies v2.1.

The attempt's Stage B is sound and is reused: three release builds at one path, one codegen
unit, offline — base ← real `dda9146e…` (ELF `524c2bc2…`, 52.760 s), new ← armed `df0db63c…`
(ELF `2fa652ea…`, 81.392 s), control ← real + the re-derived patch (ELF `c166a2ff…`,
55.318 s); functional equality across the three arms on the canonical program (120 labels,
identical normalized output); the 795 EOF hunk failed with zero fuzz and the declared shape
was re-derived — one changed file, one added `#[cold] #[inline(never)]` perturbator
`brief795_layout_perturbator` (30 B at 0xec180, one `lea` address reference, zero static
calls), one cold `black_box` reference in `runtime_path_timing_start_slow`, patch 712 B
`8cfd8faa…`.

## The v2.1 policy (freeze before any output; quote its sha256)

Write `.temp/brief-807b/policy.json` as **exactly** the frozen 807a policy at
`.temp/brief-807a/policy.json` (`1a7327f7…`), read-only, with these substitutions and no
others:

- `version`: `"rum-epoch-interference/2.1"`.
- `inputs`: keep the block, replacing the entry receipt path/hash with this attempt's own
  Stage A0 entry-pins receipt.
- `calibration.job`: restored to the v1 text, exactly: "whole-panel Callgrind with the 796
  full-panel options on the canonical program" — T ≈ 48–50 s per cell; the fixed
  `release_offset_s` 5.000, `inject_thread_cpu_s` 1.000,
  `minimum_tagged_cpu_in_window_s` 0.990 and the failure rule stay verbatim.
- `job`: add `"foreign_tick_allowance":2`; the F14/F15 integral caps become
  `0.005×T + 2/HZ` (the E caps, prelaunch cap and window cap already carry their allowance).
- All other keys unchanged, including the v2 partition, coverage, decision, attempts, orders,
  hardware, timing, sites, whole_profiles and environment.

Freeze it (write, chmod 0444, sha256) and quote the hash in the report before the first
calibration result is read. A policy that differs from these substitutions is a STOP.

## The pins (re-verify every one before measuring; a stale pin is a STOP)

Brief 807's pin table verbatim, cross-checked against the 807/807a receipts: the four 806
role roots (`cb9205a6…`/5,842, `dda9146e…`/5,851, `30cd48fc…`/5,851, `df0db63c…`/5,851), the
ten installed outputs (`d3588fb9…`, `1c48e15a…`, `99f7c5bb…`, `dc648d65…`, `8e29c41f…`,
`2ac559d1…`, `70a6da97…`, `feca9443…`, `c214a4aa…`, `44bb8f11…`), the 806 report
`6e17e444…` (check OK 1,284) and seal 158, the `R4 LIVE VERIFY PASS` line, the director's
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
≥ 50 Hz, max sample gap ≤ 0.100 s, `PRELAUNCH … quiet=true` lines for CPUs 14 and 15.

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
quote the per-cell T, E14/E15, F14/F15 and sliding maxima. A missing, failed or unavailable
calibration stops deployment for director review — no measurement may start before nine
genuine verdict lines.

## Stage C — the qualified whole-panel captures and the timing panel (4,200 s)

As brief 807's Stage C, under the v2.1 budget:

- **Whole-panel hardware.** At least three admitted pinned draws per arm, interleaved
  `base/new/control`, `control/new/base`, `new/base/control`:
  `taskset -c 14 perf stat -x, -e cpu_core/instructions/u -o <event.csv> -- <arm ELF> --file
  <canonical.rum>`, each draw under the v2.1 budget (its own T ≈ 1.2 s; the declared 2/HZ
  allowances apply), ID-qualified, with the enabled/running scale (> 99% running), the binary
  and workload hashes, exits and raw counts retained. Emit all draws, min/max/median and the
  signed differences. The fresh envelope: with `B` the fresh base median, `new_median − B`
  must lie in `[min(control) − B, max(control) − B]`. The historical references (base median
  `16,069,730,355`, control `16,069,316,559`, old armed `+80,382,406`, and the 798 draws) are
  references only.
- **The 120-label panel.** The canonical panel on every arm with `--scale 1 --reps 7`, the
  exact approved sources and internal AbsoluteTiming, four independent triples in the fixed
  order `base/new/control`, `control/new/base`, `new/base/control`, `control/base/new`;
  CPU-pinned, serialized; the 795 reducer; 119 eligible labels and the 637 exclusion visible.
  No label is called recovered on timing alone.
- The whole-panel Callgrind profiles per the policy's `whole_profiles` (three draws per arm,
  nine pairings), each under the v2.1 budget.
- Every capture under the v2.1 budget; a quarantined capture is retained, retried only after
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

The `rum-report/1` report at `.temp/brief-807b/report.json` (`"brief": 807`, `"stage":
"807b"`): the policy hash, the nine calibration lines, the reused builds and their receipts,
the admitted and unavailable captures with their budget lines, the panel and the envelope,
the per-site prices and falsifiers, the reconciliation and the fresh residual table, the
charge; the seal of the brief's artifacts, `python3 -I scripts/report.py check
.temp/brief-807b/report.json`, the detached summary and the exit audit (zero live changes,
zero `.git-attempts`, zero unexplained).

## Charge

12,300 s of the 18,275.642 s pool: A0 200 / A2 600 / C 4,200 / D 4,000 / E 2,500 / F 800.
Every stage inside its box; a box that would overrun stops at the deadline with the exact
missing measurement. The 5,975.642 s reserve stays unallocated.

## STOP rules

Any stale pin or role mismatch; a policy deviation; a failed or unavailable calibration; a
build-reuse mismatch that cannot be rebuilt; a capture the budget marks unavailable beyond its
declared retries; an unexplained whole-panel or Callgrind excess; a per-site price above its
declared bar or a falsifier that fails to fire; any live change; any git attempt. Q25's
thirteen named STOPs stand (P0214, P0216, P1070, P2348, P2354, P2379, P2381, P2385, P2387,
P2409, P2411, P2416, P2418); 744/745/746 held; CM5 unauthorized; no tag; no network; no new
dependency; no new `unsafe`; `.temp` only, never `/tmp`; no git.
