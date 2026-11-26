# Brief 807a — G15-CM4 the fresh epoch under the tracepoint-free budget (v2)

Read briefs 798, 799 (Amendment 4), 804, 805, 806 and 807 first, and rulings 832, 844, 845.
This is the D.2 line's final measurement gate, resumed after brief 807's Stage A STOP. The
whole-panel captures admitted under the defective absolute 0.069595 s rule are re-taken under
the replaced prospective interference budget, now specified as v2 because this host denies
host tracepoints to the measurement user. Everything in brief 807 stands except where this
brief changes it; where they differ, this brief wins. No tag, no network, no new dependency,
no new `unsafe` in `rum-core`/`rum-runtime`. Scratch under `.temp/brief-807a/`; never `/tmp`.
No git. Measurement only: no live edit, no inventory replay, no release mutation, no
acceptance suite. The sealed STOP record at `.temp/brief-807/` is read-only (copying its
scripts as a starting point is allowed; nothing there may be modified).

## Why v2 (the 807 STOP, accepted as correct)

Brief 807 (unit `brief-807-cdx-20261126-162957-3869797`, 16:29:57–16:52:06) froze its policy
at `5d0109b6bad1fde276580850487ac950d98e7597aa2d34100e1a9a44a856211b` before the first
attempt, passed three fresh 100 Hz prelaunch screens on CPUs 14 and 15, then received EACCES
on all seven required tracepoints on every attempt (`perf` exit 129): zero calibration
workloads and zero measurement workloads ran. I reproduced the cause outside any sandbox as
uid 1000: `/sys/kernel/tracing` is `drwx------ root root` and
`/proc/sys/kernel/perf_event_paranoid` is 2, so even the tracepoint `id` files are unreadable.
This is a host kernel-security posture, not a sandbox artifact; this brief does not change host
settings. Ruling 845 strikes the per-event lossless partition as unimplementable and re-
specifies the budget as v2 below. Also corrected there (brief defect #30): the calibrations
must follow the arm builds they bind, so the stage order is A → B → A2 → C → D → E → F.

## The v2 policy (freeze before any output; quote its sha256)

Write `.temp/brief-807a/policy.json` as **exactly** the frozen v1 policy at
`.temp/brief-807/policy.json` (5,029 B, `5d0109b6…`), read-only, with these substitutions and
no others:

- `version`: `"rum-epoch-interference/2"`.
- `inputs`: keep the block, replacing the entry receipt path/hash with this attempt's own
  Stage A entry-pins receipt; keep the recipes/label-order path and hash, the canonical
  `dc6af977…`, and the ELF-binding rule (bind each arm's ELF SHA-256 to its 807a Stage B
  receipt and its 806 role parent before its first calibration run).
- `trace_events`: removed.
- `trace`: replaced by exactly:
  "Host tracepoints are unavailable to the measurement user on this host: tracefs
  /sys/kernel/tracing is 0700 root:root and kernel.perf_event_paranoid=2; reproduced outside
  the sandbox. Qualification uses /proc/stat per-CPU fields plus the measured child's
  wait4(user+sys) only. No event stream is claimed; the partition is tick-resolved and the
  2/HZ allowance covers jiffy quantization."
- `partition`: replaced by exactly:
  "busy_c = (user+nice+system+irq+softirq+steal) delta per CPU; irq_c = (irq+softirq) delta;
  steal_c = steal delta; meas = wait4(user+sys) delta of the measured child. E14 = busy14 -
  meas; E15 = busy15. F14 = busy14 - irq14 - steal14 - meas; F15 = busy15 - irq15 - steal15.
  Report every term with its raw fields and signed wall/tick remainders; report the
  unattributed remainder explicitly and never absorb it. Use all interval endpoints and the
  endpoints shifted by one second."
- `job`: `{"aggregate_fraction_each_cpu":0.005,"tick_allowance":2,"foreign_fraction_each_cpu":0.005,"foreign_window_s":1,"foreign_window_limit_s":0.050,"window_tick_allowance":2,"inclusive_upper_bounds":true}`
  (the limits are `E14`, `E15` ≤ `0.005×T + 2/HZ`; `F14`, `F15` integrals ≤ `0.005×T`;
  every sliding 1-s window ≤ `0.050 s + 2/HZ`; identical all arms).
- `coverage` (new key): `{"samples_hz":100,"min_effective_hz":50,"max_sample_gap_s":0.100,"backend_error":"unavailable","missing_stat_column":"unavailable"}`.
- `decision`: same rule with E/F as partitioned above; persist the budget verdict before
  reading workload output, hardware counts or Callgrind totals.
- `calibration.job`: "the arm's Stage B ELF on the canonical program (--file
  <canonical.rum>), native, taskset -c 14, the same command as the Stage C hardware draw
  minus perf stat". Keep `release_offset_s` 5.000, `inject_thread_cpu_s` 1.000,
  `minimum_tagged_cpu_in_window_s` 0.990 and the failure rule verbatim; the spinner is a
  foreign process (never a descendant of the measured child), and the driver's wait4 covers
  only the measured child.
- All other keys (attempts, orders, hardware, timing, sites, whole_profiles, environment)
  unchanged.

Freeze it (write, chmod 0444, sha256) and quote the hash in the report before the first
calibration result is read. A policy that differs from these substitutions is a STOP.

## The pins (re-verify every one before measuring; a stale pin is a STOP)

Brief 807's pin table verbatim, cross-checked against the 807 attempt's receipts at
`.temp/brief-807/evidence/pins.json` (229 pins, drift 0, `ENTRY PINS PASS`): the four 806
role roots (`cb9205a6…`/5,842, `dda9146e…`/5,851, `30cd48fc…`/5,851, `df0db63c…`/5,851), the
ten installed outputs (`d3588fb9…`, `1c48e15a…`, `99f7c5bb…`, `dc648d65…`, `8e29c41f…`,
`2ac559d1…`, `70a6da97…`, `feca9443…`, `c214a4aa…`, `44bb8f11…`), the 806 report
`6e17e444…` (check OK 1,284) and seal 158, the `R4 LIVE VERIFY PASS` line, the director's
`.temp/director/gate-806e/verdict.json`, the accepted ledger (1,743 body rows / 93 families;
the 38 publication gates at 1,464,055 Ir; the fastpath's 41,712 Ir inside the 51,790 band; the
joint price 13,556,424 with `additive_claimed=false`), the 795/796/799 instruments and the
control patch `04a56f39…`. Re-check the workspace verdict's sums (4,553/16/71).

## Stage A — pins, policy freeze and backend preflight (300 s)

Re-authenticate the pins above; write and hash policy.json; run the structural backend
preflight: read both CPUs' `/proc/stat` fields at 100 Hz for 1 s on CLOCK_MONOTONIC, confirm
every required column exists, the effective rate ≥ 50 Hz and the max sample gap ≤ 0.100 s, and
emit `PRELAUNCH … quiet=true` lines for CPUs 14 and 15. There is no tracepoint test. A missing
column, a short rate or a large gap is a STOP for the backend (not a retry into a measurement).

## Stage B — the three arms (700 s)

Exactly brief 807's Stage B at `.temp/brief-807a/active`: one absolute source path, one
external target, one cargo command at a time, `-j 8`; **base** ← the 806 `real` tree
(`dda9146e…`); **new** ← the 806 `armed` tree (`df0db63c…`); **control** ← the 806 `real` tree
plus the 795 control patch (`04a56f39…`) in the brief-local copy. Byte-for-byte tree copies,
re-hashed; `cargo build --offline --locked -j 8 --release -p rum-cli --bin rum-cli` with
`CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, a private `CARGO_HOME`, the 1.85.0 toolchain ahead on
`PATH`, identical flags and environment for all three arms; save each ELF and its receipt;
functional equality across the three arms on the canonical program; prove the control
perturbator is called zero times. If the 795 hunk does not apply after brief 805's coldout
refactor, re-derive the same shape at a cold site of the final `timing.rs` — exactly one
changed file, one added `#[cold] #[inline(never)]` perturbator, one cold `black_box`
reference, no other byte changed — declare the new patch (path, bytes, sha256) and quote the
derivation; if the shape cannot be met, STOP. The proof of zero calls and zero self Ir to the
perturbator in the Stage D control profiles is required either way.

## Stage A2 — the nine calibrations on the built ELFs (600 s)

Per arm, three whole-panel calibration runs on that arm's Stage B ELF (the v2 calibration job;
T ≈ 50 s, each under the v2 budget with a fresh prelaunch screen): quiet; a 1.000 CPU-s foreign
spinner pinned to CPU 14 released at 5.000 s after launch; the same on CPU 15. Emit exactly

```
CAL GREEN arm=ARM injected=none verdict=ADMIT
CAL RED arm=ARM cpu=14 injected_cpu_s=1.000 verdict=REJECT
CAL RED arm=ARM cpu=15 injected_cpu_s=1.000 verdict=REJECT
```

for each of base, new, control (nine lines), with each red cell proving the spinner actually
executed ≥ 0.990 CPU-s in its 1-s window. Retain every raw sample stream and budget line. A
missing, failed or unavailable calibration stops deployment for director review — no
measurement may start before nine genuine verdict lines.

## Stage C — the qualified whole-panel captures and the timing panel (4,200 s)

As brief 807's Stage C, under the v2 budget in place of the v1 Stage A budget:

- **Whole-panel hardware.** At least three admitted pinned draws per arm, interleaved
  `base/new/control`, `control/new/base`, `new/base/control`:
  `taskset -c 14 perf stat -x, -e cpu_core/instructions/u -o <event.csv> -- <arm ELF> --file
  <canonical.rum>`, each draw under the v2 budget, ID-qualified, with the enabled/running
  scale (> 99% running), the binary and workload hashes, exits and raw counts retained. Emit
  all draws, min/max/median and the signed differences. The fresh envelope: with `B` the
  fresh base median, `new_median − B` must lie in `[min(control) − B, max(control) − B]`. The
  historical references (base median `16,069,730,355`, control `16,069,316,559`, old armed
  `+80,382,406`, and the 798 draws) are references only.
- **The 120-label panel.** The canonical panel on every arm with `--scale 1 --reps 7`, the
  exact approved sources and internal AbsoluteTiming, four independent triples in the fixed
  order `base/new/control`, `control/new/base`, `new/base/control`, `control/base/new`;
  CPU-pinned, serialized; the 795 reducer; 119 eligible labels and the 637 exclusion visible.
  No label is called recovered on timing alone.
- Every capture under the v2 budget; a quarantined capture is retained, retried only after a
  fresh qualification window, at most twice, with the rule fixed before any result is read; a
  capture still unavailable is a STOP for that capture.

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

The `rum-report/1` report at `.temp/brief-807a/report.json` (`"brief": 807`, `"stage":
"807a"`): the policy hash, the nine calibration lines, the builds and receipts, the admitted
and unavailable captures with their budget lines, the panel and the envelope, the per-site
prices and falsifiers, the reconciliation and the fresh residual table, the charge; the seal
of the brief's artifacts, `python3 -I scripts/report.py check .temp/brief-807a/report.json`,
the detached summary and the exit audit (zero live changes, zero `.git-attempts`, zero
unexplained).

## Charge

13,100 s of the 19,317.119 s pool: A 300 / B 700 / A2 600 / C 4,200 / D 4,000 / E 2,500 /
F 800. Every stage inside its box; a box that would overrun stops at the deadline with the
exact missing measurement. The 6,217.119 s reserve stays unallocated.

## STOP rules

Any stale pin or role mismatch; a policy deviation; a failed or unavailable calibration; a
backend preflight failure; a capture the budget marks unavailable beyond its declared
retries; an unexplained whole-panel or Callgrind excess; a per-site price above its declared
bar or a falsifier that fails to fire; any live change; any git attempt. Q25's thirteen named
STOPs stand (P0214, P0216, P1070, P2348, P2354, P2379, P2381, P2385, P2387, P2409, P2411,
P2416, P2418); 744/745/746 held; CM5 unauthorized; no tag; no network; no new dependency; no
new `unsafe`; `.temp` only, never `/tmp`; no git.
