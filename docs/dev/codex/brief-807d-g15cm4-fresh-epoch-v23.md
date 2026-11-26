# Brief 807d — G15-CM4 the fresh epoch under budget v2.3

Read briefs 798, 799 (Amendment 4), 804, 805, 806, 807, 807a, 807b, 807c and rulings 832,
844, 845, 846, 847, 848. This is the D.2 line's final measurement gate, resumed after brief
807c's Stage A2 STOP. The whole-panel captures admitted under the defective absolute
0.069595 s rule are re-taken under the replaced prospective interference budget, now v2.3: the
v2.2 passive partition with the calibration's one-second injection proof replaced by a
certified-total-burn certificate, because CFS shares the measured CPU between the job and the
spinner and no one-second window can hold the full 1.000 CPU-s burn. Everything in briefs 807,
807a, 807b and 807c stands except where this brief changes it; where they differ, this brief
wins. No tag, no network, no new dependency, no new `unsafe` in `rum-core`/`rum-runtime`.
Scratch under `.temp/brief-807d/`; never `/tmp`. No git. Measurement only: no live edit, no
inventory replay, no release mutation, no acceptance suite. The sealed records at
`.temp/brief-807/`, `.temp/brief-807a/`, `.temp/brief-807b/` and `.temp/brief-807c/` are
read-only (copying their scripts, ELFs and receipts as pinned inputs is allowed; nothing there
may be modified).

## Why v2.3 (the 807c STOP, accepted as correct)

Brief 807c (unit `brief-807-cdx-20261126-174213-3904624`, 17:42:13–17:52) froze the v2.2
policy first at `0e146abe…` (0444, 17:44:31, zero calibration and zero measurement runs),
re-verified all 229 pins and the three reused 807a ELFs, and ran two calibration cells on the
base arm. The quiet whole-panel Callgrind cell admitted:

`BUDGET id=cal-base-quiet-try1 verdict=ADMIT E14=+0.091700 F14=+0.031700 max_window14=+0.019245 E15=+0.010000 F15=+0.000000 max_window15=+0.000000 cap_E=0.249348 cap_F=0.249348 cap_window=0.070000 T=45.869501`

with `CAL GREEN arm=base injected=none verdict=ADMIT`, 4,588 samples, effective 100.001 Hz,
max gap 0.0103 s, exit 0, 120 labels equal, zero signal calls and tree-vs-wait4 agreement
within 10.0 ms — the passive sampler reproduces 798's `other_busy[14]` 0.079–0.118 s band. The
CPU-14 injection cell rejected:

`BUDGET id=cal-base-14-try1 verdict=REJECT E14=+1.087461 F14=+1.017461 max_window14=+0.511799 E15=+0.010000 F15=+0.000000 max_window15=+0.000000 cap_E=0.256189 cap_F=0.256189 cap_window=0.070000 T=47.237817`

but the brief's own injection proof failed: the spinner's certified burn is 1.000000203 CPU-s
over 2.000042858 wall seconds (start +5.000445 s, end +7.000488 s, wholly inside the 47.237817
s job), while the fixed one-second window [+5.000, +6.000] contains 0.499533668–0.499535129
CPU-s against the required 0.990. One genuine cell, one failed, seven unattempted, zero
measurement draws. The STOP is correct. The rule is my defect #34: the job and the spinner are
two runnable tasks on one CPU, so CFS shares it at about one half each; the burn takes ~2.0
wall seconds and no one-second window can hold 0.990 CPU-s of it. The disputed second's own
witness shows the sharing: 0.493287–0.504824 s of measured tree plus 0.4995 s of spinner fill
the CPU's second. The proof's intent — the full burn inside the measured job, and the budget
seeing it — was satisfied. Ruling 848 re-specifies the proof as v2.3.

The 807a Stage B is sound and is reused: three release builds at one path, one codegen unit,
offline — base ← real `dda9146e…` (ELF `524c2bc2…`), new ← armed `df0db63c…` (ELF
`2fa652ea…`), control ← real + the re-derived patch (ELF `c166a2ff…`; 712 B `8cfd8faa…`, one
changed file, one added `#[cold] #[inline(never)]` perturbator `brief795_layout_perturbator`,
one `lea` address reference, zero static calls); functional equality across 120 labels. 807b
and 807c re-verified all three byte-for-byte.

## The v2.3 policy (freeze before any output; quote its sha256)

Write `.temp/brief-807d/policy.json` as **exactly** the frozen 807c policy at
`.temp/brief-807c/policy.json` (`0e146abe…`), read-only, with these substitutions and no
others:

- `version`: `"rum-epoch-interference/2.3"`.
- `inputs`: keep the block, replacing the entry receipt path/hash with this attempt's own
  Stage A0 entry-pins receipt.
- `calibration`: keep `arms`, `injection_cpus`, `job`, `release_offset_s`,
  `inject_thread_cpu_s`, `quiet_expected` and `failure` unchanged; remove
  `minimum_tagged_cpu_in_window_s` and add, with exactly these values and semantics:
  - `"minimum_tagged_cpu_s": 0.99` — the spinner thread's certified total CPU time between
    its first and last execution (its own `thread_time_ns`), anywhere in its lifetime;
  - `"burn_containment": "The spinner's certified first-to-last execution lies wholly inside the measured job's span: its start at or after launch plus the release offset, and its end before the job's terminal sample. The per-second rate is not gated: CFS shares the measured CPU between the measured job and the spinner at about one half each."`;
  - `"minimum_integral_foreign_s": 0.97` — on the injected CPU, E ≥ this (= 0.99 − 2/HZ);
  - `"minimum_partitioned_foreign_s": 0.90` — on the injected CPU, F ≥ this;
  - `"minimum_peak_window_s": 0.25` — on the injected CPU, the sliding one-second maximum of
    F ≥ this (the quiet maximum is ~0.019 s and the cap 0.070 s; the expected shared-window
    value is ~0.5 s);
  - `"injected_expected": "REJECT with proven foreign execution on the injected CPU: the certified burn at least the tagged bar and wholly inside the job, and E, F and the sliding one-second maximum at least their bars"`.
- All other keys unchanged, including the partition, the coverage rules, the caps, the
  prelaunch, the attempts, the orders, hardware, timing, sites, whole_profiles and inputs.

Freeze it (write, chmod 0444, sha256) and quote the hash in the report before the first
calibration result is read. A policy that differs from these substitutions is a STOP.

## The tools (carry the validated measurement path byte-identically)

The 807c measurement path is validated and is carried as pinned inputs: `.temp/brief-807c/tools/passive.py`
`f597c80542f2a78265cd6dbd8c6becf2b16101e8a3cbb50bd6db1105dbf42e4b`, `budget.py`
`b7921f10ea141da565036353cc93ac2121fa66b2d4abb62c4cda63d34145af76`, `preflight.py`
`0f360da765b8c97660ab32712d025204a15509c0e3063c9022fc70f0426d81c4`, `preflight_burn.py`
`76caf087a32be4c7f68fe3c03562c4a3b4cac19144aba804bdf8e7382d692b52`, `run.py`
`5d3108f0b0eba4a45850b47afd0f1a9e6e786eff61092200b1b672c00f56d917`, `scope.py`
`4e2c4430eb532a70736bc4a19aa543fc2b2b4b152e7c291ed0f95f872f190c06`, `functional.py`
`e7edd84eb040373328cac01604386111bca3e04ed39f0d9e128f99a015287099`, `pins.py`
`9233a1215e296ea60e6ecc6626c07fe38143c997964f8d9d8e7961aca25641fd`, `entry.py`
`d995458b6efd18d72fd7d7051c15c4da01acd0b66522ab80ba4f44d3e783c653`, `freeze.py`
`bc9af6cb0cea7bb3c16335141df5eef1b358045da58a80ce6192672f26453f85`. Copy them into
`.temp/brief-807d/tools/` and verify each hash; any change to any of them is a STOP. The
calibration driver (`calibrate.py`, 807c `cccf549dbcce8415590696eefb528fbc7c663893eaf7d2029b5b5a845ad9a8d0`)
may be adapted **only** in the red cell's proof block, to read the v2.3 bars and to write the
`injection-proof.json` fields `thread_cpu_s`, `burn_contained`, `full_inside`, `disjoint`,
`burn_start_ns`, `burn_end_ns`, `integrals_s`, `peak_windows_s`; the report must quote the
exact diff against the 807c copy. The sampler must stay passive — no signal to any measured
task — and the budget reducer's decision path (caps, windows, verdict) must be unchanged.

## The pins (re-verify every one before measuring; a stale pin is a STOP)

Brief 807's pin table verbatim, cross-checked against the 807/807a/807b/807c receipts: the
four 806 role roots (`cb9205a6…`/5,842, `dda9146e…`/5,851, `30cd48fc…`/5,851, `df0db63c…`/5,851),
the ten installed outputs (`d3588fb9…`, `1c48e15a…`, `99f7c5bb…`, `dc648d65…`, `8e29c41f…`,
`2ac559d1…`, `70a6da97…`, `feca9443…`, `c214a4aa…`, `44bb8f11…`), the 806 report `6e17e444…`
(check OK 1,284) and seal 158, the `R4 LIVE VERIFY PASS` line, the director's
`.temp/director/gate-806e/verdict.json`, the accepted ledger (1,743 body rows / 93 families;
the 38 publication gates at 1,464,055 Ir; the fastpath's 41,712 Ir inside the 51,790 band; the
joint price 13,556,424 with `additive_claimed=false`), the 795/796/799 instruments and the
control patch basis `04a56f39…`. Plus, from the 807c record: `seal.json`
`6225ab9fa363a4a1d116d8fa2945f59afdf90b7b2666106254f65b49032c7d26`, `report.json`
`48823559662a4139fd6cac534f0ab6c9c6ab2f87338bb3327339c726a568dc57`, the v2.2 policy
`0e146abe…` and the ten tool hashes above. Re-hash the three 807a ELFs: base `524c2bc2…`, new
`2fa652ea…`, control `c166a2ff…`.

## Stage A0 — pins, build reuse, policy freeze and backend preflight (200 s)

Re-authenticate the pins above. Re-verify the 807a Stage B outputs and reuse them: base ELF
`524c2bc2…` (parent real `dda9146e…`), new ELF `2fa652ea…` (parent armed `df0db63c…`),
control ELF `c166a2ff…` (parent real `dda9146e…` + patch `8cfd8faa…`, 712 B), each against
its release receipt and the functional-equality receipts (120 labels). A mismatch forces a
rebuild exactly as brief 807's Stage B, with a disclosure; an unreconstructable mismatch is a
STOP. Copy and hash the ten tools. Write and hash policy.json. Backend preflight: both CPUs'
`/proc/stat` fields at 100 Hz for 1 s on CLOCK_MONOTONIC, every required column present,
effective rate ≥ 50 Hz, max sample gap ≤ 0.100 s, `PRELAUNCH … quiet=true` lines for CPUs 14
and 15; plus the passive-sampler preflight: spawn one 1-s CPU-burning child pinned to CPU 14,
sample it with the production sampler, and require the sampler's signal-call count to be zero,
the tree sum to track the child's burn, and `|tree_sum − wait4| ≤ 2/HZ` at exit. Quote the
sampler's source in the report and state the signal-call count.

## Stage A2 — the nine calibrations on the built ELFs (900 s)

Per arm, three whole-panel Callgrind calibration runs on that arm's ELF under the 796
full-panel options (the policy's `calibration.job`; each with a fresh prelaunch screen and its
own T):

- quiet → `CAL GREEN arm=ARM injected=none verdict=ADMIT`;
- a 1.000 CPU-s foreign spinner pinned to CPU 14, released at 5.000 s after launch →
  `CAL RED arm=ARM cpu=14 injected_cpu_s=1.000 verdict=REJECT`;
- the same spinner on CPU 15 → `CAL RED arm=ARM cpu=15 injected_cpu_s=1.000 verdict=REJECT`.

Each red cell must pass the v2.3 proof, all on the frozen policy's bars: the spinner thread's
certified total burn ≥ `minimum_tagged_cpu_s` (0.99); its certified first-to-last execution
wholly inside the measured job's span (start at or after launch + the release offset, end
before the job's terminal sample; the job's own T ≈ 46–50 s); E and F on the injected CPU each
≥ their bars (0.97 / 0.90); the injected CPU's sliding one-second maximum ≥ 0.25; the verdict
`REJECT`; the injector exited 0 and is a foreign sibling of the measured tree. Emit exactly
nine verdict lines (base, new, control), retain every raw sample stream and budget line, and
quote the per-cell T, E14/E15, F14/F15 and sliding maxima. The quiet cells must be measured by
the passive sampler and are expected at the historical baseline (E14 ≈ 0.08–0.12 s); a quiet
cell that shows a pause-shaped footprint (E14 ≳ 0.01×T with `stop_request_through_resume_s >
0`) means the sampler still signals — a STOP, not a retry. A missing, failed or unavailable
calibration stops deployment for director review — no measurement may start before nine
genuine verdict lines. The 807c cells are historical evidence and do not qualify; all nine are
re-taken under v2.3.

## Stage C — the qualified whole-panel captures and the timing panel (4,200 s)

As brief 807c's Stage C, under the v2.3 budget: at least three admitted pinned hardware draws
per arm, interleaved `base/new/control`, `control/new/base`, `new/base/control`, with
`taskset -c 14 perf stat -x, -e cpu_core/instructions/u -o <event.csv> -- <arm ELF> --file
<canonical.rum>` (the measured tree is `perf` plus its child ELF), ID-qualified, > 99%
running, binary and workload hashes, exits and raw counts retained, min/max/median and signed
differences emitted, and the fresh envelope `new_median − B` inside
`[min(control) − B, max(control) − B]`. The 120-label panel on every arm with `--scale 1
--reps 7`, the exact approved sources and internal AbsoluteTiming, four independent triples in
the fixed orders `base/new/control`, `control/new/base`, `new/base/control`, `control/base/new`;
CPU-pinned, serialized; the 795 reducer; 119 eligible labels and the 637 exclusion visible; no
label called recovered on timing alone. The whole-panel Callgrind profiles per the policy's
`whole_profiles` (three draws per arm, nine pairings), each under the v2.3 budget. Every
capture under the v2.3 budget; a quarantined capture is retained and retried only after a
fresh qualification window, at most twice, with the rule fixed before any result is read; a
capture still unavailable is a STOP for that capture.

## Stage D — the per-site prices, the four falsifiers and the attack surface (4,000 s)

As brief 807c's Stage D: price all four StringReplace labels at N=50/100 with four paired
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

As brief 807c's Stage E: the whole-panel Ir decomposition on all three arms joins every
reached hook and the four session facades through source, caller, visit count and inlined PC
groups; `new − base` and `control − base` reconcile to the raw totals with zero unexplained
excess in hardware and in Callgrind separately; each of the 1,743 body rows and 93 families
maps to its 799/804/805 disposition; the 38 publication gates at 1,464,055 Ir and the
fastpath's 41,712 Ir are restated at the fresh residual; every family with an unexplained row
or a moved cell is named. The joint price 13,556,424 Ir is the accepted reference,
`additive_claimed=false`; a fresh number that differs is reported as data, never silently
rebooked. The fresh residual table gives, for each named family and for the two class-form
decisions, the fresh measurement, the disposition and the exact bar that would change it. **No
acceptance is taken here**: the two decisions return to the user.

## Stage F — verification, seal and report (800 s)

The `rum-report/1` report at `.temp/brief-807d/report.json` (`"brief": 807`, `"stage":
"807d"`): the policy hash, the nine calibration lines and their v2.3 proof fields, the
passive-sampler source and its signal-call count, the carried tools and the calibration-driver
diff, the reused builds and their receipts, the admitted and unavailable captures with their
budget lines, the panel and the envelope, the per-site prices and falsifiers, the
reconciliation and the fresh residual table, the charge; the seal of the brief's artifacts,
`python3 -I scripts/report.py check .temp/brief-807d/report.json`, the detached summary and the
exit audit (zero live changes, zero `.git-attempts`, zero unexplained).

## Charge

12,600 s of the 17,179.990 s pool: A0 200 / A2 900 / C 4,200 / D 4,000 / E 2,500 / F 800.
Every stage inside its box; a box that would overrun stops at the deadline with the exact
missing measurement. The 4,579.990 s reserve stays unallocated.

## STOP rules

Any stale pin or role mismatch; a policy deviation; a tool hash deviation outside the
declared calibration-driver block; a sampler that signals the measured child or a quiet cell
with a pause-shaped footprint; a red cell whose v2.3 proof fails; a failed or unavailable
calibration; a build-reuse mismatch that cannot be rebuilt; a capture the budget marks
unavailable beyond its declared retries; an unexplained whole-panel or Callgrind excess; a
per-site price above its declared bar or a falsifier that fails to fire; any live change; any
git attempt. Q25's thirteen named STOPs stand (P0214, P0216, P1070, P2348, P2354, P2379,
P2381, P2385, P2387, P2409, P2411, P2416, P2418); 744/745/746 held; CM5 unauthorized; no tag;
no network; no new dependency; no new `unsafe`; `.temp` only, never `/tmp`; no git.
