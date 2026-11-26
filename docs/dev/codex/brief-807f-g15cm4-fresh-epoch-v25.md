# Brief 807f — G15-CM4 the fresh epoch under budget v2.5

Read briefs 798, 799 (Amendment 4), 804, 805, 806, 807, 807a, 807b, 807c, 807d, 807e and
rulings 832, 844, 845, 846, 847, 848, 849, 850. This is the D.2 line's final measurement gate,
resumed after brief 807e's Stage C STOP on the first base hardware draw. Everything in briefs
807, 807a, 807b, 807c, 807d and 807e stands except where this brief changes it; where they
differ, this brief wins. No tag, no network, no new dependency, no new `unsafe` in
`rum-core`/`rum-runtime`. Scratch under `.temp/brief-807f/`; never `/tmp`. No git.
Measurement only: no live edit, no inventory replay, no release mutation, no acceptance suite.
The sealed records at `.temp/brief-807/`, `.temp/brief-807a/`, `.temp/brief-807b/`,
`.temp/brief-807c/`, `.temp/brief-807d/` and `.temp/brief-807e/` are read-only (copying their
scripts, ELFs and receipts as pinned inputs is allowed; nothing there may be modified).

## Why v2.5 (the 807e STOP, accepted as correct)

Brief 807e (unit `brief-807-cdx-20261126-182643-3920920`, 18:26:43–18:41:16) froze the v2.4
policy first at `457e5a77…` (0444, 18:28:48, zero calibration and zero measurement runs),
passed the backend and passive-sampler preflights, and took all nine calibration cells on
their first attempt: three quiet `CAL GREEN … verdict=ADMIT` (E14 +0.081019 / +0.108603 /
+0.103788; F14 +0.021019 / +0.018603 / +0.023788; sliding maxima +0.022680 / +0.016751 /
+0.025685) and six `CAL RED … verdict=REJECT`, each meeting the v2.3 proof (certified burn
1.000000098–1.000000480 s, contained and wholly inside the job; E 1.030–1.110; F 1.000–1.020;
sliding maximum ≥ 0.511 on the injected CPU; injector exit 0; foreign sibling). Zero quiet
retries were needed. The first base hardware draw then exhausted its three declared attempts:

`HARDWARE TREE id=perf-base-1-try1 samples=113 effective_hz=100.030 last_both_ns=1054711507 last_child_ns=1017587256 root_only_samples=2 last_root_only_ns=48526304 wait4_ns=1075857000 delta_ns=1027330696 cap_ns=20000000 signal_calls=0`

(and try2 `delta_ns=1027451624`, try3 `delta_ns=1035372380`), each workload exit 0 with 120
normalized-equal labels and 100.00 % running, against the `2/HZ` = 0.020 s cap. Zero hardware
draws admitted; timing panel, whole profiles, site prices, falsifiers and fresh residual
unmeasured. The STOP is correct, and the defect is mine (#35): the partition defined `meas`
over the measured process "and every live descendant", so when the ELF child exits before
`perf` the child's entire runtime leaves the last tree sum while `wait4`'s `RUSAGE_BOTH` still
carries it (the delta is exactly `wait4` minus the root-only sum). Reduced from the retained
rows, the same artifact books the vanished child as +1.027764 / +1.026109 / +1.029758 s in the
sliding one-second window (cap 0.070 s), although E14/F14 stay inside their caps; rebuilt
under monotone accounting all three attempts admit (E14 −0.005857 / −0.001715 / −0.000487,
F14 −0.015857 / −0.001715 / −0.000487, maxima +0.011779 / +0.011658 / +0.013034) and the
agreement residuals become +0.009743 / +0.004288 / +0.012717 s against a 0.040 s cap. The
host was quiet throughout (all three quiet decisions on their first attempt), so this is an
instrument definition defect, not contamination. Ruling 850 grants the v2.5 tree rule.

## The v2.5 policy (freeze before any output; quote its sha256)

Write `.temp/brief-807f/policy.json` as **exactly** the frozen 807e policy at
`.temp/brief-807e/policy.json` (`457e5a77…`), read-only, with these substitutions and no
others:

- `version`: `"rum-epoch-interference/2.5"`.
- `inputs`: keep the block, replacing the entry receipt path/hash with this attempt's own
  Stage A0 entry-pins receipt.
- `partition`: replace verbatim with:

  `busy_c = (user+nice+system+irq+softirq+steal) delta per CPU; irq_c = (irq+softirq) delta; steal_c = steal delta; meas = the measured tree's exact runtime, read without sending any signal to any measured task: at each 100 Hz endpoint the sum over the measured process and every task ever observed in its tree since launch of each task's last successfully read field 1 (runtime_ns) of /proc/<pid>/task/<tid>/schedstat, monotone — a task that disappears keeps its last value and a task key's value never decreases, so PID reuse cannot lower the sum; the terminal total is wait4(user+sys) of the measured child, which includes its waited-for descendants, and it must agree with the last endpoint's tree sum within 2/HZ x n_tasks, where n_tasks is the number of distinct task keys ever observed (at least 1), because each task's unobservable tail after its last read is bounded by one sampling period plus scan time; otherwise the capture is unavailable. E14 = busy14 - meas; E15 = busy15. F14 = busy14 - irq14 - steal14 - meas; F15 = busy15 - irq15 - steal15. Report every term with its raw fields and signed wall/tick remainders, including n_tasks and the per-task last-seen map; report the unattributed remainder explicitly and never absorb it. Use all interval endpoints and the endpoints shifted by one second.`

- `coverage`: keep every key, replacing only `measured_tree_vs_wait4` with
  `"unavailable if they differ by more than 2/HZ x n_tasks (n_tasks = distinct schedstat task keys ever observed, minimum 1)"`.
- `calibration.quiet_retry`: keep every key, with the rule text of 807e's block and its clause
  "the last schedstat tree sum agrees with terminal wait4 within 2/HZ" replaced by "the v2.5
  tree agreement holds (the last schedstat tree sum within 2/HZ x n_tasks of the terminal
  wait4)". Every other sentence stands: zero signal calls and zero observation stops; exit 0
  with 120 normalized-equal labels; a pure integral excursion (E and/or F above its cap while
  both sliding one-second maxima stay under the window cap); every attempt retained under its
  own try folder with its budget verdict persisted before any workload output is read; an
  admitting attempt is the cell's verdict line; a quiet failure outside the rule, or three
  contaminated attempts, is a STOP; red cells remain one-shot and any red-cell proof failure
  is a STOP.
- All other keys unchanged, including the partition fields, the caps, the prelaunch, the
  attempts, the decision, the orders, hardware, timing, sites, whole_profiles, environment and
  the `trace` note.

Freeze it (write, chmod 0444, sha256) and quote the hash in the report before the first
calibration result is read. A policy that differs from these substitutions is a STOP.

## The tools (the 807c/807d/807e measurement path, one declared change)

Copy the ten tools of `.temp/brief-807e/tools/` and verify each hash: `passive.py`
`f597c80542f2a78265cd6dbd8c6becf2b16101e8a3cbb50bd6db1105dbf42e4b`, `budget.py`
`b7921f10ea141da565036353cc93ac2121fa66b2d4abb62c4cda63d34145af76`, `preflight.py`
`0f360da765b8c97660ab32712d025204a15509c0e3063c9022fc70f0426d81c4`, `preflight_burn.py`
`76caf087a32be4c7f68fe3c03562c4a3b4cac19144aba804bdf8e7382d692b52`, `run.py`
`5d3108f0b0eba4a45850b47afd0f1a9e6e786eff61092200b1b672c00f56d917`, `scope.py`
`4e2c4430eb532a70736bc4a19aa543fc2b2b4b152e7c291ed0f95f872f190c06`, `functional.py`
`e7edd84eb040373328cac01604386111bca3e04ed39f0d9e128f99a015287099`, `pins.py`
`9233a1215e296ea60e6ecc6626c07fe38143c997964f8d9d8e7961aca25641fd`, `entry.py`
`d995458b6efd18d72fd7d7051c15c4da01acd0b66522ab80ba4f44d3e783c653`, `freeze.py`
`bc9af6cb0cea7bb3c16335141df5eef1b358045da58a80ce6192672f26453f85`.

`passive.py` may change **only** in the tree accounting:

- maintain a per-task last-seen map over every task key ever observed in the measured tree,
  each value the maximum `runtime_ns` read for that key; every schedstat endpoint's `meas_ns`
  is the map's sum (a vanished task keeps its last value; a key's value never decreases);
- `boundaries.json` gains `n_tree_tasks`, `last_seen_ns` (the map), `live_tasks_last` (the live
  task keys at the last periodic endpoint) and `agreement_cap_ns = 2*NS/HZ*max(1,n_tree_tasks)`,
  with `tree_agreement = abs(terminal_wait4_ns - last_tree_ns) <= agreement_cap_ns`, and the
  failure message quotes the delta and the cap;
- nothing else: the audit hook, the fork/exec, the endpoint schedule, the deadline, the rows
  and the signal-free path keep their 807e bytes.

All other nine tools must stay byte-identical; any change to any of them is a STOP. The
calibration driver (`f003f21d…`) may change only in the quiet-retry predicate (reading
`tree_agreement` and `n_tree_tasks` from `boundaries.json` instead of recomputing `2/HZ`) and
its policy reference; the policy writer may be adapted to v2.5; the capture, stage and report
drivers may change only to print and record the v2.5 fields (`n_tree_tasks`, `last_tree_ns`,
`agreement_cap_ns`, the per-task map). The report must quote the exact diffs against the 807e
copies. The sampler must stay passive — no signal to any measured task — and the budget
reducer's decision path (caps, windows, verdict) must be unchanged.

## The pins (re-verify every one before measuring; a stale pin is a STOP)

Brief 807's pin table verbatim, cross-checked against the 807/807a/807b/807c/807d/807e
receipts: the four 806 role roots (`cb9205a6…`/5,842, `dda9146e…`/5,851, `30cd48fc…`/5,851,
`df0db63c…`/5,851), the ten installed outputs (`d3588fb9…`, `1c48e15a…`, `99f7c5bb…`,
`dc648d65…`, `8e29c41f…`, `2ac559d1…`, `70a6da97…`, `feca9443…`, `c214a4aa…`, `44bb8f11…`),
the 806 report `6e17e444…` (check OK 1,284) and seal 158, the `R4 LIVE VERIFY PASS` line, the
director's `.temp/director/gate-806e/verdict.json`, the accepted ledger (1,743 body rows / 93
families; the 38 publication gates at 1,464,055 Ir; the fastpath's 41,712 Ir inside the 51,790
band; the joint price 13,556,424 with `additive_claimed=false`), the 795/796/799 instruments
and the control patch basis `04a56f39…`. Plus, from the sealed epoch records: 807c `seal.json`
`6225ab9fa363a4a1d116d8fa2945f59afdf90b7b2666106254f65b49032c7d26`, 807c `report.json`
`48823559662a4139fd6cac534f0ab6c9c6ab2f87338bb3327339c726a568dc57`, 807d `seal.json`
`1127138af7c5b125449d5803b56b0f0a3aa516b5e1156a149eff7a74f66660ba`, 807d `report.json`
`06da09a4002820ad25eae2c9d99f22841b851fa2c37dbeca44740218026d8116`, 807e `seal.json`
`b6e661bdbdfc6c6e8a08b2e64d566520af49239b2aedc5f5a68ac635f4d1ed6c`, 807e `report.json`
`167d1b1bbd9691261624c73df1d1e4f4ffcf0d2f6c2c78a303c8215283376b37`, the v2.4 policy
`457e5a77…`, the v2.3 policy `0c8d99bc…`, the ten tool hashes above, the 807e calibration
driver `f003f21d85cc03683152c4ed4585bb92a9d1bfa5fa6a83972f33e7490d103c7c` and the 807d
calibration driver `d07a882b…`. Re-hash the three 807a ELFs: base `524c2bc2…`, new
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
and 15. Then two sampler preflights:

- the single-process burn (carried `preflight_burn.py`, unchanged): one 1-s CPU-burning child
  pinned to CPU 14, the sampler's signal-call count zero, the tree sum tracking the child's
  burn, `|tree_sum − wait4| ≤ 2/HZ` at exit;
- **the vanishing-child preflight** (new `.temp/brief-807f/tools/preflight_tree.py`): run
  `taskset -c 14 perf stat -x, -e cpu_core/instructions/u -o <folder>/event.csv -- /usr/bin/python3 -I -B -c '<burner printing its 1-s thread_time>'`
  through the carried `collect` with the v2.5 sampler. PASS requires exit 0,
  `signal_call_count=0`, `observation_stops=0`, coverage ok, exactly two process ids in
  `last_seen_ns` (the root and one child), the child's process absent from `live_tasks_last`,
  the child's last-seen sum ≥ 0.99 s, `last_tree_ns ≥ 0.99 s`, and `tree_agreement` true with
  `agreement_cap_ns = 2/HZ × n_tree_tasks`. Emit
  `TREE PREFLIGHT PASS burn=… child_last_seen=… n_tasks=… delta=… cap=…`.

Quote the sampler's source in the report and state both preflights' signal-call counts.

## Stage A2 — the nine calibrations on the built ELFs (1,500 s)

Per arm, three whole-panel Callgrind calibration cells on that arm's ELF under the 796
full-panel options (the policy's `calibration.job`; each attempt with a fresh prelaunch screen
and its own T):

- quiet → `CAL GREEN arm=ARM injected=none verdict=ADMIT`, with up to three attempts under
  the policy's `quiet_retry` rule;
- a 1.000 CPU-s foreign spinner pinned to CPU 14, released at 5.000 s after launch →
  `CAL RED arm=ARM cpu=14 injected_cpu_s=1.000 verdict=REJECT` (one-shot);
- the same spinner on CPU 15 → `CAL RED arm=ARM cpu=15 injected_cpu_s=1.000 verdict=REJECT`
  (one-shot).

A quiet attempt that does not admit may be re-run only under `quiet_retry` (REJECT verdict;
zero signal calls and zero observation stops; the v2.5 tree agreement; exit 0 with 120
normalized-equal labels; E and/or F above the integral cap while both sliding one-second
maxima stay under the window cap), at most three attempts, every attempt retained under its
own `try` folder. A quiet cell that shows a pause-shaped footprint
(`stop_request_through_resume_s > 0` or any signal call) is a STOP, not a retry. Each red cell
must pass the v2.3 proof unchanged: the spinner thread's certified total burn ≥ 0.99 CPU-s;
its certified first-to-last execution wholly inside the measured job's span; E ≥ 0.97 and
F ≥ 0.90 on the injected CPU; the injected CPU's sliding one-second maximum ≥ 0.25; verdict
`REJECT`; injector exited 0 and is a foreign sibling. Emit exactly nine verdict lines (base,
new, control), retain every raw sample stream, attempt folder and budget line, and quote the
per-cell T, E14/E15, F14/F15 and sliding maxima, and for every retried quiet cell the attempt
number and the retry predicate. A missing, failed or unavailable calibration outside the
retry rule stops deployment for director review — no measurement may start before nine genuine
verdict lines. The 807c, 807d and 807e cells are historical evidence and do not qualify; all
nine are re-taken under v2.5.

## Stage C — the qualified whole-panel captures and the timing panel (4,200 s)

As brief 807e's Stage C, under the v2.5 budget: at least three admitted pinned hardware draws
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
`whole_profiles` (three draws per arm, nine pairings), each under the v2.5 budget. Every
capture under the v2.5 budget; a quarantined capture is retained and retried only after a
fresh qualification window, at most twice, with the rule fixed before any result is read; a
capture still unavailable is a STOP for that capture. Every driver, acquisition, sampler and
reducer helper the attempt launches while a capture runs must be pinned to CPUs 0–13; an
unpinned helper running during a measured window is a STOP.

## Stage D — the per-site prices, the four falsifiers and the attack surface (4,000 s)

As brief 807e's Stage D: price all four StringReplace labels at N=50/100 with four paired
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

As brief 807e's Stage E: the whole-panel Ir decomposition on all three arms joins every
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

The `rum-report/1` report at `.temp/brief-807f/report.json` (`"brief": 807`, `"stage":
"807f"`): the policy hash, the nine calibration verdict lines and every quiet attempt with its
retry predicate, both sampler preflights with their signal-call counts and the vanishing-child
proof, the carried tools and every driver diff, the reused builds and their receipts, the
admitted and unavailable captures with their budget lines and `n_tree_tasks`, the panel and
the envelope, the per-site prices and falsifiers, the reconciliation and the fresh residual
table, the charge; the seal of the brief's artifacts, `python3 -I scripts/report.py check
.temp/brief-807f/report.json`, the detached summary and the exit audit (zero live changes,
zero `.git-attempts`, zero unexplained).

## Charge

13,200 s of the 15,752.403 s pool: A0 200 / A2 1,500 / C 4,200 / D 4,000 / E 2,500 / F 800.
Every stage inside its box; a box that would overrun stops at the deadline with the exact
missing measurement. The 2,552.403 s reserve stays unallocated.

## STOP rules

Any stale pin or role mismatch; a policy deviation; a tool hash deviation outside the declared
passive.py and driver blocks; a sampler that signals the measured child; a quiet cell with a
pause-shaped footprint; a quiet failure outside the retry rule or three contaminated attempts;
a red cell whose v2.3 proof fails; a failed or unavailable calibration; a failed
vanishing-child preflight; a measured tree with more than four tasks or one whose descendant
outlives the measured root's wait; a build-reuse mismatch that cannot be rebuilt; a capture
the budget marks unavailable beyond its declared retries; an unpinned helper running during a
measured window; an unexplained whole-panel or Callgrind excess; a per-site price above its
declared bar or a falsifier that fails to fire; any live change; any git attempt. Q25's
thirteen named STOPs stand (P0214, P0216, P1070, P2348, P2354, P2379, P2381, P2385, P2387,
P2409, P2411, P2416, P2418); 744/745/746 held; CM5 unauthorized; no tag; no network; no new
dependency; no new `unsafe`; `.temp` only, never `/tmp`; no git.
