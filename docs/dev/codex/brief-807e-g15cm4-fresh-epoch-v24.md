# Brief 807e — G15-CM4 the fresh epoch under budget v2.4

Read briefs 798, 799 (Amendment 4), 804, 805, 806, 807, 807a, 807b, 807c, 807d and rulings 832,
844, 845, 846, 847, 848, 849. This is the D.2 line's final measurement gate, resumed after
brief 807d's Stage A2 STOP on the base quiet cell. The whole-panel captures admitted under the
defective absolute 0.069595 s rule are re-taken under the replaced prospective interference
budget, now v2.4: the v2.3 passive partition and certified-burn calibration with the quiet
cell granted a declared three-attempt rule, because the host's transient foreign CPU-14 load is
outside the instrument's control (my probes reproduce the historical 0.10–0.11 s baseline in
quiet windows and 0.8–2.4 s excursions in active ones). Everything in briefs 807, 807a, 807b,
807c and 807d stands except where this brief changes it; where they differ, this brief wins.
No tag, no network, no new dependency, no new `unsafe` in `rum-core`/`rum-runtime`. Scratch
under `.temp/brief-807e/`; never `/tmp`. No git. Measurement only: no live edit, no inventory
replay, no release mutation, no acceptance suite. The sealed records at `.temp/brief-807/`,
`.temp/brief-807a/`, `.temp/brief-807b/`, `.temp/brief-807c/` and `.temp/brief-807d/` are
read-only (copying their scripts, ELFs and receipts as pinned inputs is allowed; nothing there
may be modified).

## Why v2.4 (the 807d STOP, accepted as correct)

Brief 807d (unit `brief-807-cdx-20261126-175700-3910768`, 17:57:00–18:07) froze the v2.3
policy first at `0c8d99bc…` (0444, 17:59:25, zero calibration and zero measurement runs),
carried the 807c measurement path byte-identically, and ran the first base quiet cell:

`BUDGET id=cal-base-quiet-try1 verdict=REJECT E14=+0.813502 F14=+0.753502 max_window14=+0.040190 E15=+0.010000 F15=+0.000000 max_window15=+0.000000 cap_E=0.255253 cap_F=0.255253 cap_window=0.070000 T=47.050509`

4,705 samples at 100.020 Hz, max gap 0.010314 s, exit 0, 120 labels equal, zero signal calls,
zero observation stops, tree-vs-wait4 agreement 0.91 ms. One cell failed, eight unattempted,
zero measurement draws. The STOP is correct. The instrument is not at fault: my replications of
the identical job under the carried sampler give `E14=+0.091700` (the 807c cell), then
`+0.813502` (the 807d cell), `+1.093822` and `+2.398637` in two probe windows at 18:13–18:16,
and `+0.103140`, `+0.112114`, `+0.107107` in three windows at 18:26–18:31, and a thread-level
watcher saw no other task running on CPU 14 in the quiet windows. The host alternates between
the historical baseline and transient excursions (the desktop and agent sessions are active);
a single-shot 47-s quiet cell is therefore a coin flip whenever the session is active. Ruling
849 grants the quiet cell a declared retry rule and nothing else.

## The v2.4 policy (freeze before any output; quote its sha256)

Write `.temp/brief-807e/policy.json` as **exactly** the frozen 807d policy at
`.temp/brief-807d/policy.json` (`0c8d99bc…`), read-only, with these substitutions and no
others:

- `version`: `"rum-epoch-interference/2.4"`.
- `inputs`: keep the block, replacing the entry receipt path/hash with this attempt's own
  Stage A0 entry-pins receipt.
- `calibration`: keep every key of the 807d block unchanged, and add
  `"quiet_retry": { "max_attempts": 3, "fresh_prelaunch_each": true, "rule": "A quiet cell's attempt may be re-run only when its verdict is REJECT and all of: the sampler's signal_call_count and observation_stops are zero; the last schedstat tree sum agrees with terminal wait4 within 2/HZ; the workload exited 0 with 120 normalized-equal labels; and the failure is a pure integral excursion — E and/or F above its cap while both sliding one-second maxima stay under the window cap. Every attempt is retained under its own try folder and its budget verdict is persisted before any workload output is read. An attempt that admits is the cell's verdict line. A quiet failure outside this rule is a STOP; three contaminated attempts are a STOP. Red cells remain one-shot and any red-cell proof failure is a STOP." }`;
  the `failure` text stands and is read together with `quiet_retry`.
- All other keys unchanged, including the partition, the coverage rules, the caps, the
  prelaunch, the attempts, the orders, hardware, timing, sites, whole_profiles and inputs.

Freeze it (write, chmod 0444, sha256) and quote the hash in the report before the first
calibration result is read. A policy that differs from these substitutions is a STOP.

## The tools (carry the validated measurement path byte-identically)

The 807c/807d measurement path is validated and is carried as pinned inputs:
`.temp/brief-807d/tools/passive.py`
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
`.temp/brief-807e/tools/` and verify each hash; any change to any of them is a STOP. The
calibration driver (807d `calibrate.py`
`d07a882b66b0e53184861ab99cba4329cd044dfde0beda9a908df5d08f8480c0`) may change **only** in
the quiet cell's attempt loop (up to `quiet_retry.max_attempts` tries, fresh prelaunch per
try, retention under `try` folders, the retry predicate reading the v2.4 rule) and in reading
the policy's new block; its red-cell proof block must stay as in the 807d file. The policy
writer may be adapted to v2.4. The report must quote the exact diffs against the 807d copies.
The sampler must stay passive — no signal to any measured task — and the budget reducer's
decision path (caps, windows, verdict) must be unchanged.

## The pins (re-verify every one before measuring; a stale pin is a STOP)

Brief 807's pin table verbatim, cross-checked against the 807/807a/807b/807c/807d receipts:
the four 806 role roots (`cb9205a6…`/5,842, `dda9146e…`/5,851, `30cd48fc…`/5,851,
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
`06da09a4002820ad25eae2c9d99f22841b851fa2c37dbeca44740218026d8116`, the v2.3 policy
`0c8d99bc…`, the ten tool hashes above and the 807d calibration driver `d07a882b…`. Re-hash
the three 807a ELFs: base `524c2bc2…`, new `2fa652ea…`, control `c166a2ff…`.

## Stage A0 — pins, build reuse, policy freeze and backend preflight (200 s)

Re-authenticate the pins above. Re-verify the 807a Stage B outputs and reuse them: base ELF
`524c2bc2…` (parent real `dda9146e…`), new ELF `2fa652ea…` (parent armed `df0db63c…`),
control ELF `c166a2ff…` (parent real `dda9146e…` + patch `8cfd8faa…`, 712 B), each against
its release receipt and the functional-equality receipts (120 labels). A mismatch forces a
rebuild exactly as brief 807's Stage B, with a disclosure; an unreconstructable mismatch is a
STOP. Copy and hash the ten tools. Write and hash policy.json. Backend preflight: both CPUs'
`/proc/stat` fields at 100 Hz for 1 s on CLOCK_MONOTONIC, every required column present,
effective rate ≥ 50 Hz, max sample gap ≤ 0.100 s, `PRELAUNCH … quiet=true` lines for CPUs 14
and 15; plus the passive-sampler preflight: one 1-s CPU-burning child pinned to CPU 14, the
sampler's signal-call count zero, the tree sum tracking the child's burn, and
`|tree_sum − wait4| ≤ 2/HZ` at exit. Quote the sampler's source in the report and state the
signal-call count.

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
zero signal calls and zero observation stops; tree-vs-wait4 within `2/HZ`; exit 0 with 120
normalized-equal labels; E and/or F above the integral cap while both sliding one-second
maxima stay under the window cap), at most three attempts, every attempt retained under its
own `try` folder. A quiet cell that shows a pause-shaped footprint (`stop_request_through_resume_s > 0`
or any signal call) is a STOP, not a retry. Each red cell must pass the v2.3 proof unchanged:
the spinner thread's certified total burn ≥ 0.99 CPU-s; its certified first-to-last execution
wholly inside the measured job's span; E ≥ 0.97 and F ≥ 0.90 on the injected CPU; the injected
CPU's sliding one-second maximum ≥ 0.25; verdict `REJECT`; injector exited 0 and is a foreign
sibling. Emit exactly nine verdict lines (base, new, control), retain every raw sample stream,
attempt folder and budget line, and quote the per-cell T, E14/E15, F14/F15 and sliding maxima,
and for every retried quiet cell the attempt number and the retry predicate. A missing,
failed or unavailable calibration outside the retry rule stops deployment for director review
— no measurement may start before nine genuine verdict lines. The 807c and 807d cells are
historical evidence and do not qualify; all nine are re-taken under v2.4.

## Stage C — the qualified whole-panel captures and the timing panel (4,200 s)

As brief 807c's Stage C, under the v2.4 budget: at least three admitted pinned hardware draws
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
`whole_profiles` (three draws per arm, nine pairings), each under the v2.4 budget. Every
capture under the v2.4 budget; a quarantined capture is retained and retried only after a
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

The `rum-report/1` report at `.temp/brief-807e/report.json` (`"brief": 807`, `"stage":
"807e"`): the policy hash, the nine calibration verdict lines and every quiet attempt with its
retry predicate, the passive-sampler source and its signal-call count, the carried tools and
every driver diff, the reused builds and their receipts, the admitted and unavailable captures
with their budget lines, the panel and the envelope, the per-site prices and falsifiers, the
reconciliation and the fresh residual table, the charge; the seal of the brief's artifacts,
`python3 -I scripts/report.py check .temp/brief-807e/report.json`, the detached summary and the
exit audit (zero live changes, zero `.git-attempts`, zero unexplained).

## Charge

13,200 s of the 16,587.001 s pool: A0 200 / A2 1,500 / C 4,200 / D 4,000 / E 2,500 / F 800.
Every stage inside its box; a box that would overrun stops at the deadline with the exact
missing measurement. The 3,387.001 s reserve stays unallocated.

## STOP rules

Any stale pin or role mismatch; a policy deviation; a tool hash deviation outside the declared
calibration-driver blocks; a sampler that signals the measured child or a quiet cell with a
pause-shaped footprint; a quiet failure outside the retry rule or three contaminated attempts;
a red cell whose v2.3 proof fails; a failed or unavailable calibration; a build-reuse mismatch
that cannot be rebuilt; a capture the budget marks unavailable beyond its declared retries; an
unexplained whole-panel or Callgrind excess; a per-site price above its declared bar or a
falsifier that fails to fire; any live change; any git attempt. Q25's thirteen named STOPs
stand (P0214, P0216, P1070, P2348, P2354, P2379, P2381, P2385, P2387, P2409, P2411, P2416,
P2418); 744/745/746 held; CM5 unauthorized; no tag; no network; no new dependency; no new
`unsafe`; `.temp` only, never `/tmp`; no git.
