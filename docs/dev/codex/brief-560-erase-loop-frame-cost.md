# brief-560 — erase the ordinary-path cost brief-559's loop frames added

Implementation brief. **Do not run `git` in any form — not `status`, not
`diff`, not `log` — at any point, including before you have read this brief.**
The director owns git.

## Goal

brief-559 (`e0125af4`) made `Scan`, `OptionValue`, `ReplaceRepeated`,
three-operand `Dot` and `Differences` resume into their own loops after a
suspension. It is correct and gated. But it made the **ordinary path** — a
call that never suspends — more expensive for three of them, in per-iteration
user instructions (pinned `taskset -c 2`, `cpu_core/instructions:u`,
`(count(2N) - count(N)) / N`, N = 20 000):

| Label | Base | Candidate | Delta, sample 1 / sample 2 |
| --- | ---: | ---: | --- |
| `differences-no-input` (`Differences[{1,4,9,16,25}]`) | 9 906 | 10 090 | +1.855% / +1.863% |
| `replacerepeated-no-input` | 40 256 | 40 414 | +0.393% / +0.420% |
| `optionvalue-three-no-input` | 9 765 | 9 802 | +0.384% / +0.386% |

The director reproduced the `Differences` row independently: +1.917%, with
`dot-three-no-input` at -0.026% as the control. Allocation counts and peaks are
identical at both N; this is instruction cost on the path that suspends
nothing. The design said the frames cost nothing unless the call actually
suspends. Make that true. When you are done, all nine labels brief-559 measured
sit at or below the base in both samples (a delta no larger than +0.05% in
either sample counts as at the base), and every brief-559 correctness test
still passes.

## What brief-559 already learned — do not repeat it blindly

brief-559's report §3 records two rejected experiments. `#[inline(always)]` on
the newly shared helpers erased `OptionValue`, reduced `ReplaceRepeated`, and
made `Differences` worse (+2.19%). Moving the error handling into cold helpers
and computing the resume cursor only on error brought `Differences` to +1.10%
but moved `Dot` to +0.114%. Read the saved binaries and sources for both
(`.temp/brief-559/candidate-inline/`, `candidate-cold/`).

## Where the director would look first

In `crates/rum-stdlib/src/structure_ops/list_plumbing.rs` the ordinary path
changed shape: the tight `for pair in items.windows(2)` loop became
`differences_from_pair`, iterating `windows(2).enumerate().skip(next_pair)`
with a `match` per element, behind an extra call layer; the order loop wraps
each row in `resume_or_suspend!`. A common way to make a resumable loop free on
the ordinary path is to **keep the original loop verbatim for the path that
starts at the beginning**, and enter a separate resumed-loop function — the one
that skips to the cursor and appends to a carried prefix — only from a frame's
`resume`. The frame is built in a cold, non-inlined branch reached only when a
call returns `InputRequired`. The same shape may apply to `OptionValue`'s name
loop and `ReplaceRepeated`'s pass loop. These are hypotheses; measure each
change, one variable at a time.

## Measurement — the preserved base is the reference

Use `.temp/brief-559/base/rum-cli` and `rum-cli-alloc` as the base; do not
rebuild them. Build each candidate into its own directory under
`.temp/brief-560/`, keep every candidate binary you measure, and record its
hash. Labels: `.temp/brief-559/perf/*.rum` — `scan-no-input`,
`optionvalue-three-no-input`, `replacerepeated-no-input`, `dot-three-no-input`,
`differences-no-input`, `do-count-null`, `do-iterator-null`,
`table-iterator-null`, `replaceall-simple`, at N = 20 000 and 40 000.
Allocation counts first, then instructions, two samples per (label, N, build),
one label per run, `rum-cli` fed `< /dev/null`. **Every candidate is measured on
all nine labels** — the cold-helper experiment's `Dot` regression is exactly
the kind of neighbour effect this catches. Timing is not needed.

## Scope

- **Objective:** the ordinary-path shape of the five brief-559 sites, so that
  all nine labels meet the threshold, with the brief-559 correctness tests
  unchanged and green.
- **Standing duty — always in scope** (`docs/dev/codex/TEMPLATE.md`, Scope):
  replay the retention fingerprints and certificates and reconcile ownership
  inventories; new `UnknownConstruct` rows for code you add or change are
  expected and disclosed; STOP on a changed or vanished existing row, a new
  `UnknownHostPayload` row, or a row in code you did not touch.
- **Fence:** only the five brief-559 sites and their private helpers. No
  semantic change: `crates/rum-stdlib/tests/loop_continuations.rs` and every
  other test stay exactly as they are and must stay green. No other brief-548
  group, no enforcement work, no `Expr` change, no leases, no new dependency,
  no `unsafe`. Do not edit `AGENTS.md`, `PROGRESS.md`, `TODO.md`, earlier briefs
  or unrelated docs.
- **Disclosure duty:** every changed file with line ranges and why; each
  candidate you measured, what it changed, and its nine-label rows.

## rum / Rust pitfalls (read twice)

1. **One variable per measurement.** Change one thing, measure all nine labels.
2. **An improvement on one label that regresses a neighbour is rejected.**
3. **Instruction counts decide.** Do not substitute timing.
4. **Stop-loss is keyed on progress**: the number of labels above the
   threshold must fall. Stop when two consecutive compiling candidates reduce
   neither that number nor the largest remaining delta, and report the curve.

## Verification (MANDATORY — you run it; iterate to green)

Run everything serially (user instruction): every cargo command with
`env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1`, never two cargo commands at once,
each timeout derived from a measured serial run of the same shape, through
`python3 scripts/run-bounded.py --timeout <seconds> --`.

```
cargo fmt --check
cargo test -p rum-stdlib --test loop_continuations
cargo test -p rum-runtime
cargo test -p rum-stdlib
cargo test --workspace
```

Plus the retention replay in the `--no-default-features` build as the template
describes. Paste commands, bounds, exit statuses and counts verbatim; list
native ZeroMQ skips.

## Checkpointing — protect the scene

Scratch under `.temp/brief-560/`. After each candidate write
`.temp/brief-560/PROGRESS.md`: its change, its nine-label rows, the counters,
the exact next action and the tree's state.

## Hard rules

- Do NOT `git commit`, branch, push or touch git config. Do not run `git`.
- Do NOT modify files outside the objective, the standing duty and the fence;
  STOP and report instead.
- Touch nothing outside this repository, user configuration included. No
  network. Commands: file reads and edits within scope, `rg`/`grep`, `python3`,
  bounded `cargo`, the release `rum-cli`, bounded `taskset`/`perf`.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **No new `unsafe`.**
- You MUST paste real verbatim output and say plainly what did not run.

## Report format (your final message, appended to this file)

1. What changed — per file with line ranges.
2. Every candidate measured, with its nine-label allocation and instruction
   rows, both samples.
3. Verification — every command's verbatim result.
4. Progress counters, disclosures, anything unresolved.

---

**Execution report — stop-loss reached; performance objective unmet; correctness gates green.**

Optimization stopped after candidates 07 and 08 made no progress. The tree retains candidate 06 plus the formatter-only OptionValue call wrap. Eight labels meet the +0.05% threshold in both samples; **Differences remains +0.694711% / +0.877091%**. No zero-cost claim is made. All required correctness suites and both-feature retention acceptance are green.

1. **What changed — final tree.**

| File | Final line ranges | Why |
| --- | --- | --- |
| [crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs](../../../crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs) | 102, 133–223 | Route the ordinary entry through a direct pass loop; allocate existing resume frames only on suspension in cold helpers. |
| [crates/rum-runtime/src/runtime/options/builtins.rs](../../../crates/rum-runtime/src/runtime/options/builtins.rs) | 262–298, 301–305 | Restore the ordinary name loop; freeze items, prefix and cursor only through a cold error helper. |
| [crates/rum-stdlib/src/functional_ops/arrays/mod.rs](../../../crates/rum-stdlib/src/functional_ops/arrays/mod.rs) | 48–86 | Restore the ordinary operand fold and keep the cursor helper for frame resume. |
| [crates/rum-stdlib/src/structure_ops/list_plumbing.rs](../../../crates/rum-stdlib/src/structure_ops/list_plumbing.rs) | 28–57, 143, 149–179 | Restore direct ordinary order/row loops with cold error helpers; inline only the ordinary row helper. |
| [crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv](../../../crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv) | 436–441, 600–602, 1033, 1330–1332 | Replay 13 position-only row moves. |
| [crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv](../../../crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv) | 888–898, 1176–1180, 2072, 2691–2702 | Replay 29 position-only row moves. |
| [crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv](../../../crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv) | 743–750, 1120–1124, 2206–2207, 2791–2801 | Replay 20 position-only moves and add six UnknownConstruct macro rows in changed code. |
| [crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv](../../../crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv) | 2–69 | Refresh 68 generated fingerprints; preserve all first 18 fields, including 62 Discharged and six Pending statuses. |
| [crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv](../../../crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv) | 2–3 | Install the four replayed C01/C02 certificate hash cells; preserve identity, status, strength, preconditions and exclusions. |
| `docs/dev/codex/brief-560-erase-loop-frame-cost.md` | 142–3319 | Append this execution report only; original brief prefix preserved. |

Scan, all test sources, Expr, dependencies, leases, AGENTS.md, the root PROGRESS.md, TODO.md, earlier briefs and unrelated docs are unchanged. The final audit verifies 490 unchanged test/fixture/inventory files outside the five authorized maintenance inventories, all initial files still present, no new unsafe and unchanged preserved base binaries. Changes were compared against initial file snapshots with Python; no git command was run.

Size assessment: options/builtins.rs is 20,485 characters and crosses into the split-plan range. Keep this fenced performance experiment together for review; a future boundary can separate OptionValue query/resume driving from option parsing and SetOptions handling. repeated.rs is 19,858 characters and can later separate pass/resume driving from rule/limit normalization. list_plumbing.rs is 16,075 characters and can later separate Differences driving from padding/array plumbing. arrays/mod.rs is 7,031 characters. No unrelated split or TODO edit was made.

Review artifacts: [snapshot comparison](../../../.temp/brief-560/review.patch), [file hashes and exact change ranges](../../../.temp/brief-560/file-audit.json), [checkpoint history](../../../.temp/brief-560/PROGRESS.md). Every measured source/binary is preserved under `.temp/brief-560/candidate-NN/`; final source bytes are under `.temp/brief-560/final-source/`. Generated build/test artifacts remain in their scratch locations.

2. **Every measured candidate — allocation and instruction rows, both samples.**

Reference binaries were read from `.temp/brief-559/base/` and never rebuilt. All probes used the exact saved `.temp/brief-559/perf/*.rum` sources at N=20,000 and 40,000, one label per process, `taskset -c 2`, stdin `/dev/null`, and `cpu_core/instructions/u`. For every build, both allocation samples preceded both instruction samples. Profiling variables were cleared; allocation runs alone set RUM_PROFILE_ALLOC=1. The per-iteration instruction value is `(count(40000) - count(20000)) / 20000`; each candidate sample is compared with the corresponding preserved-base sample. No timing gate was substituted.

Exactly 648 unique process runs completed: 324 allocation and 324 instruction runs, all exit 0; no tuple was sampled more than twice. All allocation counts and peak live bytes equal the base at both N values in both samples for every candidate. Total allocated bytes are exactly eight bytes higher per process for every candidate, matching the longer executable argv path; per-iteration byte slopes are equal. The initial cap was 144 (base plus candidate 01), extended by 72 for each distinct later candidate to 648; no probes followed the stop-loss.

Full exact process commands, bounds, exit statuses, stdout, allocation lines and raw perf CSV output: [648-process transcript](../../../.temp/brief-560/performance-command-transcript.md). Environment snapshots and raw results are under `.temp/brief-560/perf/{alloc,instructions}/{build}/{sample}/`.

Preserved base hashes:
```text
rum-cli  f40c5b1f206d9a0f5771ccd71990227b13aa2d470a91eb6a49ec7596cfbfab5c
rum-cli-alloc  046be5b3e9b64e9a0ecb2fac12d5f5e829c18433b7f846097c73de19138fc716
```

Base allocation rows, copied from the actual probe stderr (both samples and both N values):

```text
sample=1	scan-no-input	N=20000	rum-profile alloc count=661661 bytes_total=51204971 peak_live_bytes=942414
sample=1	scan-no-input	N=40000	rum-profile alloc count=1321661 bytes_total=101924971 peak_live_bytes=1582414
sample=1	optionvalue-three-no-input	N=20000	rum-profile alloc count=181794 bytes_total=30460125 peak_live_bytes=9134068
sample=1	optionvalue-three-no-input	N=40000	rum-profile alloc count=361794 bytes_total=53020125 peak_live_bytes=9774068
sample=1	replacerepeated-no-input	N=20000	rum-profile alloc count=1221717 bytes_total=154311068 peak_live_bytes=948049
sample=1	replacerepeated-no-input	N=40000	rum-profile alloc count=2441717 bytes_total=308131068 peak_live_bytes=1588049
sample=1	dot-three-no-input	N=20000	rum-profile alloc count=11921740 bytes_total=477979704 peak_live_bytes=8678236
sample=1	dot-three-no-input	N=40000	rum-profile alloc count=23841740 bytes_total=948059704 peak_live_bytes=9318236
sample=1	differences-no-input	N=20000	rum-profile alloc count=261670 bytes_total=21969680 peak_live_bytes=9591399
sample=1	differences-no-input	N=40000	rum-profile alloc count=521670 bytes_total=36049680 peak_live_bytes=10231399
sample=1	do-count-null	N=20000	rum-profile alloc count=1588 bytes_total=478931 peak_live_bytes=299668
sample=1	do-count-null	N=40000	rum-profile alloc count=1588 bytes_total=478931 peak_live_bytes=299668
sample=1	do-iterator-null	N=20000	rum-profile alloc count=1612 bytes_total=1119811 peak_live_bytes=940146
sample=1	do-iterator-null	N=40000	rum-profile alloc count=1612 bytes_total=1759811 peak_live_bytes=1580146
sample=1	table-iterator-null	N=20000	rum-profile alloc count=1614 bytes_total=1119731 peak_live_bytes=940025
sample=1	table-iterator-null	N=40000	rum-profile alloc count=1614 bytes_total=1759731 peak_live_bytes=1580025
sample=1	replaceall-simple	N=20000	rum-profile alloc count=101746 bytes_total=27530285 peak_live_bytes=943388
sample=1	replaceall-simple	N=40000	rum-profile alloc count=201746 bytes_total=54570285 peak_live_bytes=1583388
sample=2	scan-no-input	N=20000	rum-profile alloc count=661661 bytes_total=51204971 peak_live_bytes=942414
sample=2	scan-no-input	N=40000	rum-profile alloc count=1321661 bytes_total=101924971 peak_live_bytes=1582414
sample=2	optionvalue-three-no-input	N=20000	rum-profile alloc count=181794 bytes_total=30460125 peak_live_bytes=9134068
sample=2	optionvalue-three-no-input	N=40000	rum-profile alloc count=361794 bytes_total=53020125 peak_live_bytes=9774068
sample=2	replacerepeated-no-input	N=20000	rum-profile alloc count=1221717 bytes_total=154311068 peak_live_bytes=948049
sample=2	replacerepeated-no-input	N=40000	rum-profile alloc count=2441717 bytes_total=308131068 peak_live_bytes=1588049
sample=2	dot-three-no-input	N=20000	rum-profile alloc count=11921740 bytes_total=477979704 peak_live_bytes=8678236
sample=2	dot-three-no-input	N=40000	rum-profile alloc count=23841740 bytes_total=948059704 peak_live_bytes=9318236
sample=2	differences-no-input	N=20000	rum-profile alloc count=261670 bytes_total=21969680 peak_live_bytes=9591399
sample=2	differences-no-input	N=40000	rum-profile alloc count=521670 bytes_total=36049680 peak_live_bytes=10231399
sample=2	do-count-null	N=20000	rum-profile alloc count=1588 bytes_total=478931 peak_live_bytes=299668
sample=2	do-count-null	N=40000	rum-profile alloc count=1588 bytes_total=478931 peak_live_bytes=299668
sample=2	do-iterator-null	N=20000	rum-profile alloc count=1612 bytes_total=1119811 peak_live_bytes=940146
sample=2	do-iterator-null	N=40000	rum-profile alloc count=1612 bytes_total=1759811 peak_live_bytes=1580146
sample=2	table-iterator-null	N=20000	rum-profile alloc count=1614 bytes_total=1119731 peak_live_bytes=940025
sample=2	table-iterator-null	N=40000	rum-profile alloc count=1614 bytes_total=1759731 peak_live_bytes=1580025
sample=2	replaceall-simple	N=20000	rum-profile alloc count=101746 bytes_total=27530285 peak_live_bytes=943388
sample=2	replaceall-simple	N=40000	rum-profile alloc count=201746 bytes_total=54570285 peak_live_bytes=1583388
```

**candidate-01 — Separate the ordinary Differences order and windows(2) row loops from the cursor loops; add cold, non-inlined frame capture helpers. Existing resumed loops and frames remain.**

```text
rum-cli  ae828eb01fcd44166375f434341494b8ba834516dccbefe3ba3c240dbec2160e
rum-cli-alloc  91800a6a22a16614b6f2af24a0f4dfb2d6cf6e084eb53760f2d33cff3f7bb77e
```

Verbatim reducer rows: allocation counts are the N/2N slopes; peak pairs are N,2N; both samples are explicit. Instruction columns are instructions per iteration and percent delta.

```tsv
label	alloc_count_per_iter_base/candidate_s1;s2	peak_base/candidate_N,2N_s1;s2	instr_base_s1	instr_candidate_s1	delta_s1_pct	instr_base_s2	instr_candidate_s2	delta_s2_pct
scan-no-input	33/33;33/33	942414,1582414/942414,1582414;942414,1582414/942414,1582414	30081.09900	30016.32295	-0.215338	30081.41360	30016.37370	-0.216213
optionvalue-three-no-input	9/9;9/9	9134068,9774068/9134068,9774068;9134068,9774068/9134068,9774068	9765.60370	9810.13780	+0.456030	9764.06485	9810.14880	+0.471975
replacerepeated-no-input	61/61;61/61	948049,1588049/948049,1588049;948049,1588049/948049,1588049	40255.78520	40407.78230	+0.377578	40277.82735	40421.89135	+0.357676
dot-three-no-input	596/596;596/596	8678236,9318236/8678236,9318236;8678236,9318236/8678236,9318236	256551.87380	256899.41140	+0.135465	256552.02685	256898.99525	+0.135243
differences-no-input	13/13;13/13	9591399,10231399/9591399,10231399;9591399,10231399/9591399,10231399	9918.24750	10007.90500	+0.903965	9900.28660	10009.13305	+1.099427
do-count-null	0/0;0/0	299668,299668/299668,299668;299668,299668/299668,299668	498.01670	496.95870	-0.212443	498.21620	496.99650	-0.244813
do-iterator-null	0/0;0/0	940146,1580146/940146,1580146;940146,1580146/940146,1580146	1138.07175	1137.08400	-0.086792	1137.86315	1137.11905	-0.065395
table-iterator-null	0/0;0/0	940025,1580025/940025,1580025;940025,1580025/940025,1580025	349.16675	349.01690	-0.042916	349.05840	349.12855	+0.020097
replaceall-simple	5/5;5/5	943388,1583388/943388,1583388;943388,1583388/943388,1583388	11431.25005	11447.54360	+0.142535	11497.30280	11447.40405	-0.434004
```

Raw N/2N counts for this build and its base, both samples, follow. Allocation rows contain count, total bytes and peak bytes; instruction rows contain the original perf counter.

```tsv
mode	build	sample	label	N	count_or_instructions	bytes_total	peak_live_bytes
alloc	base	1	scan-no-input	20000	661661	51204971	942414
alloc	base	1	scan-no-input	40000	1321661	101924971	1582414
alloc	base	1	optionvalue-three-no-input	20000	181794	30460125	9134068
alloc	base	1	optionvalue-three-no-input	40000	361794	53020125	9774068
alloc	base	1	replacerepeated-no-input	20000	1221717	154311068	948049
alloc	base	1	replacerepeated-no-input	40000	2441717	308131068	1588049
alloc	base	1	dot-three-no-input	20000	11921740	477979704	8678236
alloc	base	1	dot-three-no-input	40000	23841740	948059704	9318236
alloc	base	1	differences-no-input	20000	261670	21969680	9591399
alloc	base	1	differences-no-input	40000	521670	36049680	10231399
alloc	base	1	do-count-null	20000	1588	478931	299668
alloc	base	1	do-count-null	40000	1588	478931	299668
alloc	base	1	do-iterator-null	20000	1612	1119811	940146
alloc	base	1	do-iterator-null	40000	1612	1759811	1580146
alloc	base	1	table-iterator-null	20000	1614	1119731	940025
alloc	base	1	table-iterator-null	40000	1614	1759731	1580025
alloc	base	1	replaceall-simple	20000	101746	27530285	943388
alloc	base	1	replaceall-simple	40000	201746	54570285	1583388
alloc	base	2	scan-no-input	20000	661661	51204971	942414
alloc	base	2	scan-no-input	40000	1321661	101924971	1582414
alloc	base	2	optionvalue-three-no-input	20000	181794	30460125	9134068
alloc	base	2	optionvalue-three-no-input	40000	361794	53020125	9774068
alloc	base	2	replacerepeated-no-input	20000	1221717	154311068	948049
alloc	base	2	replacerepeated-no-input	40000	2441717	308131068	1588049
alloc	base	2	dot-three-no-input	20000	11921740	477979704	8678236
alloc	base	2	dot-three-no-input	40000	23841740	948059704	9318236
alloc	base	2	differences-no-input	20000	261670	21969680	9591399
alloc	base	2	differences-no-input	40000	521670	36049680	10231399
alloc	base	2	do-count-null	20000	1588	478931	299668
alloc	base	2	do-count-null	40000	1588	478931	299668
alloc	base	2	do-iterator-null	20000	1612	1119811	940146
alloc	base	2	do-iterator-null	40000	1612	1759811	1580146
alloc	base	2	table-iterator-null	20000	1614	1119731	940025
alloc	base	2	table-iterator-null	40000	1614	1759731	1580025
alloc	base	2	replaceall-simple	20000	101746	27530285	943388
alloc	base	2	replaceall-simple	40000	201746	54570285	1583388
alloc	candidate-01	1	scan-no-input	20000	661661	51204979	942414
alloc	candidate-01	1	scan-no-input	40000	1321661	101924979	1582414
alloc	candidate-01	1	optionvalue-three-no-input	20000	181794	30460133	9134068
alloc	candidate-01	1	optionvalue-three-no-input	40000	361794	53020133	9774068
alloc	candidate-01	1	replacerepeated-no-input	20000	1221717	154311076	948049
alloc	candidate-01	1	replacerepeated-no-input	40000	2441717	308131076	1588049
alloc	candidate-01	1	dot-three-no-input	20000	11921740	477979712	8678236
alloc	candidate-01	1	dot-three-no-input	40000	23841740	948059712	9318236
alloc	candidate-01	1	differences-no-input	20000	261670	21969688	9591399
alloc	candidate-01	1	differences-no-input	40000	521670	36049688	10231399
alloc	candidate-01	1	do-count-null	20000	1588	478939	299668
alloc	candidate-01	1	do-count-null	40000	1588	478939	299668
alloc	candidate-01	1	do-iterator-null	20000	1612	1119819	940146
alloc	candidate-01	1	do-iterator-null	40000	1612	1759819	1580146
alloc	candidate-01	1	table-iterator-null	20000	1614	1119739	940025
alloc	candidate-01	1	table-iterator-null	40000	1614	1759739	1580025
alloc	candidate-01	1	replaceall-simple	20000	101746	27530293	943388
alloc	candidate-01	1	replaceall-simple	40000	201746	54570293	1583388
alloc	candidate-01	2	scan-no-input	20000	661661	51204979	942414
alloc	candidate-01	2	scan-no-input	40000	1321661	101924979	1582414
alloc	candidate-01	2	optionvalue-three-no-input	20000	181794	30460133	9134068
alloc	candidate-01	2	optionvalue-three-no-input	40000	361794	53020133	9774068
alloc	candidate-01	2	replacerepeated-no-input	20000	1221717	154311076	948049
alloc	candidate-01	2	replacerepeated-no-input	40000	2441717	308131076	1588049
alloc	candidate-01	2	dot-three-no-input	20000	11921740	477979712	8678236
alloc	candidate-01	2	dot-three-no-input	40000	23841740	948059712	9318236
alloc	candidate-01	2	differences-no-input	20000	261670	21969688	9591399
alloc	candidate-01	2	differences-no-input	40000	521670	36049688	10231399
alloc	candidate-01	2	do-count-null	20000	1588	478939	299668
alloc	candidate-01	2	do-count-null	40000	1588	478939	299668
alloc	candidate-01	2	do-iterator-null	20000	1612	1119819	940146
alloc	candidate-01	2	do-iterator-null	40000	1612	1759819	1580146
alloc	candidate-01	2	table-iterator-null	20000	1614	1119739	940025
alloc	candidate-01	2	table-iterator-null	40000	1614	1759739	1580025
alloc	candidate-01	2	replaceall-simple	20000	101746	27530293	943388
alloc	candidate-01	2	replaceall-simple	40000	201746	54570293	1583388
instructions	base	1	scan-no-input	20000	605160277	-	-
instructions	base	1	scan-no-input	40000	1206782257	-	-
instructions	base	1	optionvalue-three-no-input	20000	201087575	-	-
instructions	base	1	optionvalue-three-no-input	40000	396399649	-	-
instructions	base	1	replacerepeated-no-input	20000	808713464	-	-
instructions	base	1	replacerepeated-no-input	40000	1613829168	-	-
instructions	base	1	dot-three-no-input	20000	5137818636	-	-
instructions	base	1	dot-three-no-input	40000	10268856112	-	-
instructions	base	1	differences-no-input	20000	203808966	-	-
instructions	base	1	differences-no-input	40000	402173916	-	-
instructions	base	1	do-count-null	20000	13473293	-	-
instructions	base	1	do-count-null	40000	23433627	-	-
instructions	base	1	do-iterator-null	20000	26289658	-	-
instructions	base	1	do-iterator-null	40000	49051093	-	-
instructions	base	1	table-iterator-null	20000	10511109	-	-
instructions	base	1	table-iterator-null	40000	17494444	-	-
instructions	base	1	replaceall-simple	20000	233119239	-	-
instructions	base	1	replaceall-simple	40000	461744240	-	-
instructions	base	2	scan-no-input	20000	605182683	-	-
instructions	base	2	scan-no-input	40000	1206810955	-	-
instructions	base	2	optionvalue-three-no-input	20000	201103532	-	-
instructions	base	2	optionvalue-three-no-input	40000	396384829	-	-
instructions	base	2	replacerepeated-no-input	20000	808714348	-	-
instructions	base	2	replacerepeated-no-input	40000	1614270895	-	-
instructions	base	2	dot-three-no-input	20000	5137813258	-	-
instructions	base	2	dot-three-no-input	40000	10268853795	-	-
instructions	base	2	differences-no-input	20000	203928166	-	-
instructions	base	2	differences-no-input	40000	401933898	-	-
instructions	base	2	do-count-null	20000	13472665	-	-
instructions	base	2	do-count-null	40000	23436989	-	-
instructions	base	2	do-iterator-null	20000	26290197	-	-
instructions	base	2	do-iterator-null	40000	49047460	-	-
instructions	base	2	table-iterator-null	20000	10511246	-	-
instructions	base	2	table-iterator-null	40000	17492414	-	-
instructions	base	2	replaceall-simple	20000	232678100	-	-
instructions	base	2	replaceall-simple	40000	462624156	-	-
instructions	candidate-01	1	scan-no-input	20000	603854371	-	-
instructions	candidate-01	1	scan-no-input	40000	1204180830	-	-
instructions	candidate-01	1	optionvalue-three-no-input	20000	202022248	-	-
instructions	candidate-01	1	optionvalue-three-no-input	40000	398225004	-	-
instructions	candidate-01	1	replacerepeated-no-input	20000	812754707	-	-
instructions	candidate-01	1	replacerepeated-no-input	40000	1620910353	-	-
instructions	candidate-01	1	dot-three-no-input	20000	5144753558	-	-
instructions	candidate-01	1	dot-three-no-input	40000	10282741786	-	-
instructions	candidate-01	1	differences-no-input	20000	205828890	-	-
instructions	candidate-01	1	differences-no-input	40000	405986990	-	-
instructions	candidate-01	1	do-count-null	20000	13453721	-	-
instructions	candidate-01	1	do-count-null	40000	23392895	-	-
instructions	candidate-01	1	do-iterator-null	20000	26267872	-	-
instructions	candidate-01	1	do-iterator-null	40000	49009552	-	-
instructions	candidate-01	1	table-iterator-null	20000	10511662	-	-
instructions	candidate-01	1	table-iterator-null	40000	17492000	-	-
instructions	candidate-01	1	replaceall-simple	20000	231677288	-	-
instructions	candidate-01	1	replaceall-simple	40000	460628160	-	-
instructions	candidate-01	2	scan-no-input	20000	603854596	-	-
instructions	candidate-01	2	scan-no-input	40000	1204182070	-	-
instructions	candidate-01	2	optionvalue-three-no-input	20000	202029249	-	-
instructions	candidate-01	2	optionvalue-three-no-input	40000	398232225	-	-
instructions	candidate-01	2	replacerepeated-no-input	20000	812035475	-	-
instructions	candidate-01	2	replacerepeated-no-input	40000	1620473302	-	-
instructions	candidate-01	2	dot-three-no-input	20000	5144758609	-	-
instructions	candidate-01	2	dot-three-no-input	40000	10282738514	-	-
instructions	candidate-01	2	differences-no-input	20000	205826350	-	-
instructions	candidate-01	2	differences-no-input	40000	406009011	-	-
instructions	candidate-01	2	do-count-null	20000	13453057	-	-
instructions	candidate-01	2	do-count-null	40000	23392987	-	-
instructions	candidate-01	2	do-iterator-null	20000	26268398	-	-
instructions	candidate-01	2	do-iterator-null	40000	49010779	-	-
instructions	candidate-01	2	table-iterator-null	20000	10512505	-	-
instructions	candidate-01	2	table-iterator-null	40000	17495076	-	-
instructions	candidate-01	2	replaceall-simple	20000	231679278	-	-
instructions	candidate-01	2	replaceall-simple	40000	460627359	-	-
```

**candidate-02 — Over 01, add only #[inline(always)] to the new ordinary successive_differences helper.**

```text
rum-cli  164da2227890353b3a4f1b34092f3eb5340aaf35c745a4aeb7ec44b0a4583cdd
rum-cli-alloc  03855a7bf39038ab90b166e9bbad8dc5763967a4a5df9181355ce7a74d080512
```

Verbatim reducer rows: allocation counts are the N/2N slopes; peak pairs are N,2N; both samples are explicit. Instruction columns are instructions per iteration and percent delta.

```tsv
label	alloc_count_per_iter_base/candidate_s1;s2	peak_base/candidate_N,2N_s1;s2	instr_base_s1	instr_candidate_s1	delta_s1_pct	instr_base_s2	instr_candidate_s2	delta_s2_pct
scan-no-input	33/33;33/33	942414,1582414/942414,1582414;942414,1582414/942414,1582414	30081.09900	30015.06015	-0.219536	30081.41360	30013.42885	-0.226003
optionvalue-three-no-input	9/9;9/9	9134068,9774068/9134068,9774068;9134068,9774068/9134068,9774068	9765.60370	9810.53440	+0.460091	9764.06485	9810.92955	+0.479971
replacerepeated-no-input	61/61;61/61	948049,1588049/948049,1588049;948049,1588049/948049,1588049	40255.78520	40421.74355	+0.412260	40277.82735	40421.85870	+0.357595
dot-three-no-input	596/596;596/596	8678236,9318236/8678236,9318236;8678236,9318236/8678236,9318236	256551.87380	256899.43115	+0.135473	256552.02685	256899.11975	+0.135291
differences-no-input	13/13;13/13	9591399,10231399/9591399,10231399;9591399,10231399/9591399,10231399	9918.24750	9986.33805	+0.686518	9900.28660	9985.63800	+0.862110
do-count-null	0/0;0/0	299668,299668/299668,299668;299668,299668/299668,299668	498.01670	496.84385	-0.235504	498.21620	497.00600	-0.242907
do-iterator-null	0/0;0/0	940146,1580146/940146,1580146;940146,1580146/940146,1580146	1138.07175	1137.00470	-0.093759	1137.86315	1136.99085	-0.076661
table-iterator-null	0/0;0/0	940025,1580025/940025,1580025;940025,1580025/940025,1580025	349.16675	349.00400	-0.046611	349.05840	349.02930	-0.008337
replaceall-simple	5/5;5/5	943388,1583388/943388,1583388;943388,1583388/943388,1583388	11431.25005	11403.24495	-0.244987	11497.30280	11403.06955	-0.819612
```

Raw N/2N counts for this build and its base, both samples, follow. Allocation rows contain count, total bytes and peak bytes; instruction rows contain the original perf counter.

```tsv
mode	build	sample	label	N	count_or_instructions	bytes_total	peak_live_bytes
alloc	base	1	scan-no-input	20000	661661	51204971	942414
alloc	base	1	scan-no-input	40000	1321661	101924971	1582414
alloc	base	1	optionvalue-three-no-input	20000	181794	30460125	9134068
alloc	base	1	optionvalue-three-no-input	40000	361794	53020125	9774068
alloc	base	1	replacerepeated-no-input	20000	1221717	154311068	948049
alloc	base	1	replacerepeated-no-input	40000	2441717	308131068	1588049
alloc	base	1	dot-three-no-input	20000	11921740	477979704	8678236
alloc	base	1	dot-three-no-input	40000	23841740	948059704	9318236
alloc	base	1	differences-no-input	20000	261670	21969680	9591399
alloc	base	1	differences-no-input	40000	521670	36049680	10231399
alloc	base	1	do-count-null	20000	1588	478931	299668
alloc	base	1	do-count-null	40000	1588	478931	299668
alloc	base	1	do-iterator-null	20000	1612	1119811	940146
alloc	base	1	do-iterator-null	40000	1612	1759811	1580146
alloc	base	1	table-iterator-null	20000	1614	1119731	940025
alloc	base	1	table-iterator-null	40000	1614	1759731	1580025
alloc	base	1	replaceall-simple	20000	101746	27530285	943388
alloc	base	1	replaceall-simple	40000	201746	54570285	1583388
alloc	base	2	scan-no-input	20000	661661	51204971	942414
alloc	base	2	scan-no-input	40000	1321661	101924971	1582414
alloc	base	2	optionvalue-three-no-input	20000	181794	30460125	9134068
alloc	base	2	optionvalue-three-no-input	40000	361794	53020125	9774068
alloc	base	2	replacerepeated-no-input	20000	1221717	154311068	948049
alloc	base	2	replacerepeated-no-input	40000	2441717	308131068	1588049
alloc	base	2	dot-three-no-input	20000	11921740	477979704	8678236
alloc	base	2	dot-three-no-input	40000	23841740	948059704	9318236
alloc	base	2	differences-no-input	20000	261670	21969680	9591399
alloc	base	2	differences-no-input	40000	521670	36049680	10231399
alloc	base	2	do-count-null	20000	1588	478931	299668
alloc	base	2	do-count-null	40000	1588	478931	299668
alloc	base	2	do-iterator-null	20000	1612	1119811	940146
alloc	base	2	do-iterator-null	40000	1612	1759811	1580146
alloc	base	2	table-iterator-null	20000	1614	1119731	940025
alloc	base	2	table-iterator-null	40000	1614	1759731	1580025
alloc	base	2	replaceall-simple	20000	101746	27530285	943388
alloc	base	2	replaceall-simple	40000	201746	54570285	1583388
alloc	candidate-02	1	scan-no-input	20000	661661	51204979	942414
alloc	candidate-02	1	scan-no-input	40000	1321661	101924979	1582414
alloc	candidate-02	1	optionvalue-three-no-input	20000	181794	30460133	9134068
alloc	candidate-02	1	optionvalue-three-no-input	40000	361794	53020133	9774068
alloc	candidate-02	1	replacerepeated-no-input	20000	1221717	154311076	948049
alloc	candidate-02	1	replacerepeated-no-input	40000	2441717	308131076	1588049
alloc	candidate-02	1	dot-three-no-input	20000	11921740	477979712	8678236
alloc	candidate-02	1	dot-three-no-input	40000	23841740	948059712	9318236
alloc	candidate-02	1	differences-no-input	20000	261670	21969688	9591399
alloc	candidate-02	1	differences-no-input	40000	521670	36049688	10231399
alloc	candidate-02	1	do-count-null	20000	1588	478939	299668
alloc	candidate-02	1	do-count-null	40000	1588	478939	299668
alloc	candidate-02	1	do-iterator-null	20000	1612	1119819	940146
alloc	candidate-02	1	do-iterator-null	40000	1612	1759819	1580146
alloc	candidate-02	1	table-iterator-null	20000	1614	1119739	940025
alloc	candidate-02	1	table-iterator-null	40000	1614	1759739	1580025
alloc	candidate-02	1	replaceall-simple	20000	101746	27530293	943388
alloc	candidate-02	1	replaceall-simple	40000	201746	54570293	1583388
alloc	candidate-02	2	scan-no-input	20000	661661	51204979	942414
alloc	candidate-02	2	scan-no-input	40000	1321661	101924979	1582414
alloc	candidate-02	2	optionvalue-three-no-input	20000	181794	30460133	9134068
alloc	candidate-02	2	optionvalue-three-no-input	40000	361794	53020133	9774068
alloc	candidate-02	2	replacerepeated-no-input	20000	1221717	154311076	948049
alloc	candidate-02	2	replacerepeated-no-input	40000	2441717	308131076	1588049
alloc	candidate-02	2	dot-three-no-input	20000	11921740	477979712	8678236
alloc	candidate-02	2	dot-three-no-input	40000	23841740	948059712	9318236
alloc	candidate-02	2	differences-no-input	20000	261670	21969688	9591399
alloc	candidate-02	2	differences-no-input	40000	521670	36049688	10231399
alloc	candidate-02	2	do-count-null	20000	1588	478939	299668
alloc	candidate-02	2	do-count-null	40000	1588	478939	299668
alloc	candidate-02	2	do-iterator-null	20000	1612	1119819	940146
alloc	candidate-02	2	do-iterator-null	40000	1612	1759819	1580146
alloc	candidate-02	2	table-iterator-null	20000	1614	1119739	940025
alloc	candidate-02	2	table-iterator-null	40000	1614	1759739	1580025
alloc	candidate-02	2	replaceall-simple	20000	101746	27530293	943388
alloc	candidate-02	2	replaceall-simple	40000	201746	54570293	1583388
instructions	base	1	scan-no-input	20000	605160277	-	-
instructions	base	1	scan-no-input	40000	1206782257	-	-
instructions	base	1	optionvalue-three-no-input	20000	201087575	-	-
instructions	base	1	optionvalue-three-no-input	40000	396399649	-	-
instructions	base	1	replacerepeated-no-input	20000	808713464	-	-
instructions	base	1	replacerepeated-no-input	40000	1613829168	-	-
instructions	base	1	dot-three-no-input	20000	5137818636	-	-
instructions	base	1	dot-three-no-input	40000	10268856112	-	-
instructions	base	1	differences-no-input	20000	203808966	-	-
instructions	base	1	differences-no-input	40000	402173916	-	-
instructions	base	1	do-count-null	20000	13473293	-	-
instructions	base	1	do-count-null	40000	23433627	-	-
instructions	base	1	do-iterator-null	20000	26289658	-	-
instructions	base	1	do-iterator-null	40000	49051093	-	-
instructions	base	1	table-iterator-null	20000	10511109	-	-
instructions	base	1	table-iterator-null	40000	17494444	-	-
instructions	base	1	replaceall-simple	20000	233119239	-	-
instructions	base	1	replaceall-simple	40000	461744240	-	-
instructions	base	2	scan-no-input	20000	605182683	-	-
instructions	base	2	scan-no-input	40000	1206810955	-	-
instructions	base	2	optionvalue-three-no-input	20000	201103532	-	-
instructions	base	2	optionvalue-three-no-input	40000	396384829	-	-
instructions	base	2	replacerepeated-no-input	20000	808714348	-	-
instructions	base	2	replacerepeated-no-input	40000	1614270895	-	-
instructions	base	2	dot-three-no-input	20000	5137813258	-	-
instructions	base	2	dot-three-no-input	40000	10268853795	-	-
instructions	base	2	differences-no-input	20000	203928166	-	-
instructions	base	2	differences-no-input	40000	401933898	-	-
instructions	base	2	do-count-null	20000	13472665	-	-
instructions	base	2	do-count-null	40000	23436989	-	-
instructions	base	2	do-iterator-null	20000	26290197	-	-
instructions	base	2	do-iterator-null	40000	49047460	-	-
instructions	base	2	table-iterator-null	20000	10511246	-	-
instructions	base	2	table-iterator-null	40000	17492414	-	-
instructions	base	2	replaceall-simple	20000	232678100	-	-
instructions	base	2	replaceall-simple	40000	462624156	-	-
instructions	candidate-02	1	scan-no-input	20000	603883130	-	-
instructions	candidate-02	1	scan-no-input	40000	1204184333	-	-
instructions	candidate-02	1	optionvalue-three-no-input	20000	202013449	-	-
instructions	candidate-02	1	optionvalue-three-no-input	40000	398224137	-	-
instructions	candidate-02	1	replacerepeated-no-input	20000	812036092	-	-
instructions	candidate-02	1	replacerepeated-no-input	40000	1620470963	-	-
instructions	candidate-02	1	dot-three-no-input	20000	5144747557	-	-
instructions	candidate-02	1	dot-three-no-input	40000	10282736180	-	-
instructions	candidate-02	1	differences-no-input	20000	205378086	-	-
instructions	candidate-02	1	differences-no-input	40000	405104847	-	-
instructions	candidate-02	1	do-count-null	20000	13454584	-	-
instructions	candidate-02	1	do-count-null	40000	23391461	-	-
instructions	candidate-02	1	do-iterator-null	20000	26269444	-	-
instructions	candidate-02	1	do-iterator-null	40000	49009538	-	-
instructions	candidate-02	1	table-iterator-null	20000	10511852	-	-
instructions	candidate-02	1	table-iterator-null	40000	17491932	-	-
instructions	candidate-02	1	replaceall-simple	20000	231680034	-	-
instructions	candidate-02	1	replaceall-simple	40000	459744933	-	-
instructions	candidate-02	2	scan-no-input	20000	603884085	-	-
instructions	candidate-02	2	scan-no-input	40000	1204152662	-	-
instructions	candidate-02	2	optionvalue-three-no-input	20000	202017152	-	-
instructions	candidate-02	2	optionvalue-three-no-input	40000	398235743	-	-
instructions	candidate-02	2	replacerepeated-no-input	20000	812033401	-	-
instructions	candidate-02	2	replacerepeated-no-input	40000	1620470575	-	-
instructions	candidate-02	2	dot-three-no-input	20000	5144759812	-	-
instructions	candidate-02	2	dot-three-no-input	40000	10282742207	-	-
instructions	candidate-02	2	differences-no-input	20000	205386864	-	-
instructions	candidate-02	2	differences-no-input	40000	405099624	-	-
instructions	candidate-02	2	do-count-null	20000	13453606	-	-
instructions	candidate-02	2	do-count-null	40000	23393726	-	-
instructions	candidate-02	2	do-iterator-null	20000	26269785	-	-
instructions	candidate-02	2	do-iterator-null	40000	49009602	-	-
instructions	candidate-02	2	table-iterator-null	20000	10513386	-	-
instructions	candidate-02	2	table-iterator-null	40000	17493972	-	-
instructions	candidate-02	2	replaceall-simple	20000	231681846	-	-
instructions	candidate-02	2	replaceall-simple	40000	459743237	-	-
```

**candidate-03 — Over 02, add only #[inline(always)] to difference_expr. Rejected; this attribute was rolled back before 04.**

```text
rum-cli  53fe835bc367231c4d58e11fdf1ac1b42dcec41070a10e9736fc7447cdda24f8
rum-cli-alloc  9f47b6e32ad673db7152736625bf6c8766e5b2814672ef9ae17f5a0fde505d0d
```

Verbatim reducer rows: allocation counts are the N/2N slopes; peak pairs are N,2N; both samples are explicit. Instruction columns are instructions per iteration and percent delta.

```tsv
label	alloc_count_per_iter_base/candidate_s1;s2	peak_base/candidate_N,2N_s1;s2	instr_base_s1	instr_candidate_s1	delta_s1_pct	instr_base_s2	instr_candidate_s2	delta_s2_pct
scan-no-input	33/33;33/33	942414,1582414/942414,1582414;942414,1582414/942414,1582414	30081.09900	30016.31425	-0.215367	30081.41360	30013.50175	-0.225760
optionvalue-three-no-input	9/9;9/9	9134068,9774068/9134068,9774068;9134068,9774068/9134068,9774068	9765.60370	9810.57190	+0.460475	9764.06485	9811.25290	+0.483283
replacerepeated-no-input	61/61;61/61	948049,1588049/948049,1588049;948049,1588049/948049,1588049	40255.78520	40421.84150	+0.412503	40277.82735	40421.66350	+0.357110
dot-three-no-input	596/596;596/596	8678236,9318236/8678236,9318236;8678236,9318236/8678236,9318236	256551.87380	256898.85215	+0.135247	256552.02685	256898.96980	+0.135233
differences-no-input	13/13;13/13	9591399,10231399/9591399,10231399;9591399,10231399/9591399,10231399	9918.24750	10017.29555	+0.998645	9900.28660	10015.92915	+1.168073
do-count-null	0/0;0/0	299668,299668/299668,299668;299668,299668/299668,299668	498.01670	496.87280	-0.229691	498.21620	496.94995	-0.254157
do-iterator-null	0/0;0/0	940146,1580146/940146,1580146;940146,1580146/940146,1580146	1138.07175	1137.15665	-0.080408	1137.86315	1136.98085	-0.077540
table-iterator-null	0/0;0/0	940025,1580025/940025,1580025;940025,1580025/940025,1580025	349.16675	349.22050	+0.015394	349.05840	349.11035	+0.014883
replaceall-simple	5/5;5/5	943388,1583388/943388,1583388;943388,1583388/943388,1583388	11431.25005	11403.40725	-0.243567	11497.30280	11403.36000	-0.817086
```

Raw N/2N counts for this build and its base, both samples, follow. Allocation rows contain count, total bytes and peak bytes; instruction rows contain the original perf counter.

```tsv
mode	build	sample	label	N	count_or_instructions	bytes_total	peak_live_bytes
alloc	base	1	scan-no-input	20000	661661	51204971	942414
alloc	base	1	scan-no-input	40000	1321661	101924971	1582414
alloc	base	1	optionvalue-three-no-input	20000	181794	30460125	9134068
alloc	base	1	optionvalue-three-no-input	40000	361794	53020125	9774068
alloc	base	1	replacerepeated-no-input	20000	1221717	154311068	948049
alloc	base	1	replacerepeated-no-input	40000	2441717	308131068	1588049
alloc	base	1	dot-three-no-input	20000	11921740	477979704	8678236
alloc	base	1	dot-three-no-input	40000	23841740	948059704	9318236
alloc	base	1	differences-no-input	20000	261670	21969680	9591399
alloc	base	1	differences-no-input	40000	521670	36049680	10231399
alloc	base	1	do-count-null	20000	1588	478931	299668
alloc	base	1	do-count-null	40000	1588	478931	299668
alloc	base	1	do-iterator-null	20000	1612	1119811	940146
alloc	base	1	do-iterator-null	40000	1612	1759811	1580146
alloc	base	1	table-iterator-null	20000	1614	1119731	940025
alloc	base	1	table-iterator-null	40000	1614	1759731	1580025
alloc	base	1	replaceall-simple	20000	101746	27530285	943388
alloc	base	1	replaceall-simple	40000	201746	54570285	1583388
alloc	base	2	scan-no-input	20000	661661	51204971	942414
alloc	base	2	scan-no-input	40000	1321661	101924971	1582414
alloc	base	2	optionvalue-three-no-input	20000	181794	30460125	9134068
alloc	base	2	optionvalue-three-no-input	40000	361794	53020125	9774068
alloc	base	2	replacerepeated-no-input	20000	1221717	154311068	948049
alloc	base	2	replacerepeated-no-input	40000	2441717	308131068	1588049
alloc	base	2	dot-three-no-input	20000	11921740	477979704	8678236
alloc	base	2	dot-three-no-input	40000	23841740	948059704	9318236
alloc	base	2	differences-no-input	20000	261670	21969680	9591399
alloc	base	2	differences-no-input	40000	521670	36049680	10231399
alloc	base	2	do-count-null	20000	1588	478931	299668
alloc	base	2	do-count-null	40000	1588	478931	299668
alloc	base	2	do-iterator-null	20000	1612	1119811	940146
alloc	base	2	do-iterator-null	40000	1612	1759811	1580146
alloc	base	2	table-iterator-null	20000	1614	1119731	940025
alloc	base	2	table-iterator-null	40000	1614	1759731	1580025
alloc	base	2	replaceall-simple	20000	101746	27530285	943388
alloc	base	2	replaceall-simple	40000	201746	54570285	1583388
alloc	candidate-03	1	scan-no-input	20000	661661	51204979	942414
alloc	candidate-03	1	scan-no-input	40000	1321661	101924979	1582414
alloc	candidate-03	1	optionvalue-three-no-input	20000	181794	30460133	9134068
alloc	candidate-03	1	optionvalue-three-no-input	40000	361794	53020133	9774068
alloc	candidate-03	1	replacerepeated-no-input	20000	1221717	154311076	948049
alloc	candidate-03	1	replacerepeated-no-input	40000	2441717	308131076	1588049
alloc	candidate-03	1	dot-three-no-input	20000	11921740	477979712	8678236
alloc	candidate-03	1	dot-three-no-input	40000	23841740	948059712	9318236
alloc	candidate-03	1	differences-no-input	20000	261670	21969688	9591399
alloc	candidate-03	1	differences-no-input	40000	521670	36049688	10231399
alloc	candidate-03	1	do-count-null	20000	1588	478939	299668
alloc	candidate-03	1	do-count-null	40000	1588	478939	299668
alloc	candidate-03	1	do-iterator-null	20000	1612	1119819	940146
alloc	candidate-03	1	do-iterator-null	40000	1612	1759819	1580146
alloc	candidate-03	1	table-iterator-null	20000	1614	1119739	940025
alloc	candidate-03	1	table-iterator-null	40000	1614	1759739	1580025
alloc	candidate-03	1	replaceall-simple	20000	101746	27530293	943388
alloc	candidate-03	1	replaceall-simple	40000	201746	54570293	1583388
alloc	candidate-03	2	scan-no-input	20000	661661	51204979	942414
alloc	candidate-03	2	scan-no-input	40000	1321661	101924979	1582414
alloc	candidate-03	2	optionvalue-three-no-input	20000	181794	30460133	9134068
alloc	candidate-03	2	optionvalue-three-no-input	40000	361794	53020133	9774068
alloc	candidate-03	2	replacerepeated-no-input	20000	1221717	154311076	948049
alloc	candidate-03	2	replacerepeated-no-input	40000	2441717	308131076	1588049
alloc	candidate-03	2	dot-three-no-input	20000	11921740	477979712	8678236
alloc	candidate-03	2	dot-three-no-input	40000	23841740	948059712	9318236
alloc	candidate-03	2	differences-no-input	20000	261670	21969688	9591399
alloc	candidate-03	2	differences-no-input	40000	521670	36049688	10231399
alloc	candidate-03	2	do-count-null	20000	1588	478939	299668
alloc	candidate-03	2	do-count-null	40000	1588	478939	299668
alloc	candidate-03	2	do-iterator-null	20000	1612	1119819	940146
alloc	candidate-03	2	do-iterator-null	40000	1612	1759819	1580146
alloc	candidate-03	2	table-iterator-null	20000	1614	1119739	940025
alloc	candidate-03	2	table-iterator-null	40000	1614	1759739	1580025
alloc	candidate-03	2	replaceall-simple	20000	101746	27530293	943388
alloc	candidate-03	2	replaceall-simple	40000	201746	54570293	1583388
instructions	base	1	scan-no-input	20000	605160277	-	-
instructions	base	1	scan-no-input	40000	1206782257	-	-
instructions	base	1	optionvalue-three-no-input	20000	201087575	-	-
instructions	base	1	optionvalue-three-no-input	40000	396399649	-	-
instructions	base	1	replacerepeated-no-input	20000	808713464	-	-
instructions	base	1	replacerepeated-no-input	40000	1613829168	-	-
instructions	base	1	dot-three-no-input	20000	5137818636	-	-
instructions	base	1	dot-three-no-input	40000	10268856112	-	-
instructions	base	1	differences-no-input	20000	203808966	-	-
instructions	base	1	differences-no-input	40000	402173916	-	-
instructions	base	1	do-count-null	20000	13473293	-	-
instructions	base	1	do-count-null	40000	23433627	-	-
instructions	base	1	do-iterator-null	20000	26289658	-	-
instructions	base	1	do-iterator-null	40000	49051093	-	-
instructions	base	1	table-iterator-null	20000	10511109	-	-
instructions	base	1	table-iterator-null	40000	17494444	-	-
instructions	base	1	replaceall-simple	20000	233119239	-	-
instructions	base	1	replaceall-simple	40000	461744240	-	-
instructions	base	2	scan-no-input	20000	605182683	-	-
instructions	base	2	scan-no-input	40000	1206810955	-	-
instructions	base	2	optionvalue-three-no-input	20000	201103532	-	-
instructions	base	2	optionvalue-three-no-input	40000	396384829	-	-
instructions	base	2	replacerepeated-no-input	20000	808714348	-	-
instructions	base	2	replacerepeated-no-input	40000	1614270895	-	-
instructions	base	2	dot-three-no-input	20000	5137813258	-	-
instructions	base	2	dot-three-no-input	40000	10268853795	-	-
instructions	base	2	differences-no-input	20000	203928166	-	-
instructions	base	2	differences-no-input	40000	401933898	-	-
instructions	base	2	do-count-null	20000	13472665	-	-
instructions	base	2	do-count-null	40000	23436989	-	-
instructions	base	2	do-iterator-null	20000	26290197	-	-
instructions	base	2	do-iterator-null	40000	49047460	-	-
instructions	base	2	table-iterator-null	20000	10511246	-	-
instructions	base	2	table-iterator-null	40000	17492414	-	-
instructions	base	2	replaceall-simple	20000	232678100	-	-
instructions	base	2	replaceall-simple	40000	462624156	-	-
instructions	candidate-03	1	scan-no-input	20000	603885233	-	-
instructions	candidate-03	1	scan-no-input	40000	1204211518	-	-
instructions	candidate-03	1	optionvalue-three-no-input	20000	202014842	-	-
instructions	candidate-03	1	optionvalue-three-no-input	40000	398226280	-	-
instructions	candidate-03	1	replacerepeated-no-input	20000	812036400	-	-
instructions	candidate-03	1	replacerepeated-no-input	40000	1620473230	-	-
instructions	candidate-03	1	dot-three-no-input	20000	5144757800	-	-
instructions	candidate-03	1	dot-three-no-input	40000	10282734843	-	-
instructions	candidate-03	1	differences-no-input	20000	205983364	-	-
instructions	candidate-03	1	differences-no-input	40000	406329275	-	-
instructions	candidate-03	1	do-count-null	20000	13455830	-	-
instructions	candidate-03	1	do-count-null	40000	23393286	-	-
instructions	candidate-03	1	do-iterator-null	20000	26267028	-	-
instructions	candidate-03	1	do-iterator-null	40000	49010161	-	-
instructions	candidate-03	1	table-iterator-null	20000	10510193	-	-
instructions	candidate-03	1	table-iterator-null	40000	17494603	-	-
instructions	candidate-03	1	replaceall-simple	20000	231678645	-	-
instructions	candidate-03	1	replaceall-simple	40000	459746790	-	-
instructions	candidate-03	2	scan-no-input	20000	603884342	-	-
instructions	candidate-03	2	scan-no-input	40000	1204154377	-	-
instructions	candidate-03	2	optionvalue-three-no-input	20000	202012005	-	-
instructions	candidate-03	2	optionvalue-three-no-input	40000	398237063	-	-
instructions	candidate-03	2	replacerepeated-no-input	20000	812036892	-	-
instructions	candidate-03	2	replacerepeated-no-input	40000	1620470162	-	-
instructions	candidate-03	2	dot-three-no-input	20000	5144755005	-	-
instructions	candidate-03	2	dot-three-no-input	40000	10282734401	-	-
instructions	candidate-03	2	differences-no-input	20000	205985505	-	-
instructions	candidate-03	2	differences-no-input	40000	406304088	-	-
instructions	candidate-03	2	do-count-null	20000	13453109	-	-
instructions	candidate-03	2	do-count-null	40000	23392108	-	-
instructions	candidate-03	2	do-iterator-null	20000	26268343	-	-
instructions	candidate-03	2	do-iterator-null	40000	49007960	-	-
instructions	candidate-03	2	table-iterator-null	20000	10511702	-	-
instructions	candidate-03	2	table-iterator-null	40000	17493909	-	-
instructions	candidate-03	2	replaceall-simple	20000	231680333	-	-
instructions	candidate-03	2	replaceall-simple	40000	459747533	-	-
```

**candidate-04 — Over 02, separate the ordinary OptionValue name loop from its resumed cursor helper and capture frames in a cold helper. Candidate 03 is not included.**

```text
rum-cli  39a35656f9bf75fba7d2348bd4ac4d843e52297a51982390fc4690748233666f
rum-cli-alloc  fe8b5ae9148577c9b85e107c607bd272b71abbc9d0683b7c11378e1d840a202a
```

Verbatim reducer rows: allocation counts are the N/2N slopes; peak pairs are N,2N; both samples are explicit. Instruction columns are instructions per iteration and percent delta.

```tsv
label	alloc_count_per_iter_base/candidate_s1;s2	peak_base/candidate_N,2N_s1;s2	instr_base_s1	instr_candidate_s1	delta_s1_pct	instr_base_s2	instr_candidate_s2	delta_s2_pct
scan-no-input	33/33;33/33	942414,1582414/942414,1582414;942414,1582414/942414,1582414	30081.09900	30014.18540	-0.222444	30081.41360	30014.73420	-0.221663
optionvalue-three-no-input	9/9;9/9	9134068,9774068/9134068,9774068;9134068,9774068/9134068,9774068	9765.60370	9747.32430	-0.187181	9764.06485	9746.98505	-0.174925
replacerepeated-no-input	61/61;61/61	948049,1588049/948049,1588049;948049,1588049/948049,1588049	40255.78520	40443.76490	+0.466963	40277.82735	40421.90965	+0.357721
dot-three-no-input	596/596;596/596	8678236,9318236/8678236,9318236;8678236,9318236/8678236,9318236	256551.87380	256899.18725	+0.135377	256552.02685	256899.97165	+0.135623
differences-no-input	13/13;13/13	9591399,10231399/9591399,10231399;9591399,10231399/9591399,10231399	9918.24750	9987.75925	+0.700847	9900.28660	9987.21795	+0.878069
do-count-null	0/0;0/0	299668,299668/299668,299668;299668,299668/299668,299668	498.01670	496.86165	-0.231930	498.21620	497.05335	-0.233403
do-iterator-null	0/0;0/0	940146,1580146/940146,1580146;940146,1580146/940146,1580146	1138.07175	1136.97330	-0.096519	1137.86315	1137.08170	-0.068677
table-iterator-null	0/0;0/0	940025,1580025/940025,1580025;940025,1580025/940025,1580025	349.16675	349.03155	-0.038721	349.05840	348.90755	-0.043216
replaceall-simple	5/5;5/5	943388,1583388/943388,1583388;943388,1583388/943388,1583388	11431.25005	11381.23225	-0.437553	11497.30280	11447.33270	-0.434625
```

Raw N/2N counts for this build and its base, both samples, follow. Allocation rows contain count, total bytes and peak bytes; instruction rows contain the original perf counter.

```tsv
mode	build	sample	label	N	count_or_instructions	bytes_total	peak_live_bytes
alloc	base	1	scan-no-input	20000	661661	51204971	942414
alloc	base	1	scan-no-input	40000	1321661	101924971	1582414
alloc	base	1	optionvalue-three-no-input	20000	181794	30460125	9134068
alloc	base	1	optionvalue-three-no-input	40000	361794	53020125	9774068
alloc	base	1	replacerepeated-no-input	20000	1221717	154311068	948049
alloc	base	1	replacerepeated-no-input	40000	2441717	308131068	1588049
alloc	base	1	dot-three-no-input	20000	11921740	477979704	8678236
alloc	base	1	dot-three-no-input	40000	23841740	948059704	9318236
alloc	base	1	differences-no-input	20000	261670	21969680	9591399
alloc	base	1	differences-no-input	40000	521670	36049680	10231399
alloc	base	1	do-count-null	20000	1588	478931	299668
alloc	base	1	do-count-null	40000	1588	478931	299668
alloc	base	1	do-iterator-null	20000	1612	1119811	940146
alloc	base	1	do-iterator-null	40000	1612	1759811	1580146
alloc	base	1	table-iterator-null	20000	1614	1119731	940025
alloc	base	1	table-iterator-null	40000	1614	1759731	1580025
alloc	base	1	replaceall-simple	20000	101746	27530285	943388
alloc	base	1	replaceall-simple	40000	201746	54570285	1583388
alloc	base	2	scan-no-input	20000	661661	51204971	942414
alloc	base	2	scan-no-input	40000	1321661	101924971	1582414
alloc	base	2	optionvalue-three-no-input	20000	181794	30460125	9134068
alloc	base	2	optionvalue-three-no-input	40000	361794	53020125	9774068
alloc	base	2	replacerepeated-no-input	20000	1221717	154311068	948049
alloc	base	2	replacerepeated-no-input	40000	2441717	308131068	1588049
alloc	base	2	dot-three-no-input	20000	11921740	477979704	8678236
alloc	base	2	dot-three-no-input	40000	23841740	948059704	9318236
alloc	base	2	differences-no-input	20000	261670	21969680	9591399
alloc	base	2	differences-no-input	40000	521670	36049680	10231399
alloc	base	2	do-count-null	20000	1588	478931	299668
alloc	base	2	do-count-null	40000	1588	478931	299668
alloc	base	2	do-iterator-null	20000	1612	1119811	940146
alloc	base	2	do-iterator-null	40000	1612	1759811	1580146
alloc	base	2	table-iterator-null	20000	1614	1119731	940025
alloc	base	2	table-iterator-null	40000	1614	1759731	1580025
alloc	base	2	replaceall-simple	20000	101746	27530285	943388
alloc	base	2	replaceall-simple	40000	201746	54570285	1583388
alloc	candidate-04	1	scan-no-input	20000	661661	51204979	942414
alloc	candidate-04	1	scan-no-input	40000	1321661	101924979	1582414
alloc	candidate-04	1	optionvalue-three-no-input	20000	181794	30460133	9134068
alloc	candidate-04	1	optionvalue-three-no-input	40000	361794	53020133	9774068
alloc	candidate-04	1	replacerepeated-no-input	20000	1221717	154311076	948049
alloc	candidate-04	1	replacerepeated-no-input	40000	2441717	308131076	1588049
alloc	candidate-04	1	dot-three-no-input	20000	11921740	477979712	8678236
alloc	candidate-04	1	dot-three-no-input	40000	23841740	948059712	9318236
alloc	candidate-04	1	differences-no-input	20000	261670	21969688	9591399
alloc	candidate-04	1	differences-no-input	40000	521670	36049688	10231399
alloc	candidate-04	1	do-count-null	20000	1588	478939	299668
alloc	candidate-04	1	do-count-null	40000	1588	478939	299668
alloc	candidate-04	1	do-iterator-null	20000	1612	1119819	940146
alloc	candidate-04	1	do-iterator-null	40000	1612	1759819	1580146
alloc	candidate-04	1	table-iterator-null	20000	1614	1119739	940025
alloc	candidate-04	1	table-iterator-null	40000	1614	1759739	1580025
alloc	candidate-04	1	replaceall-simple	20000	101746	27530293	943388
alloc	candidate-04	1	replaceall-simple	40000	201746	54570293	1583388
alloc	candidate-04	2	scan-no-input	20000	661661	51204979	942414
alloc	candidate-04	2	scan-no-input	40000	1321661	101924979	1582414
alloc	candidate-04	2	optionvalue-three-no-input	20000	181794	30460133	9134068
alloc	candidate-04	2	optionvalue-three-no-input	40000	361794	53020133	9774068
alloc	candidate-04	2	replacerepeated-no-input	20000	1221717	154311076	948049
alloc	candidate-04	2	replacerepeated-no-input	40000	2441717	308131076	1588049
alloc	candidate-04	2	dot-three-no-input	20000	11921740	477979712	8678236
alloc	candidate-04	2	dot-three-no-input	40000	23841740	948059712	9318236
alloc	candidate-04	2	differences-no-input	20000	261670	21969688	9591399
alloc	candidate-04	2	differences-no-input	40000	521670	36049688	10231399
alloc	candidate-04	2	do-count-null	20000	1588	478939	299668
alloc	candidate-04	2	do-count-null	40000	1588	478939	299668
alloc	candidate-04	2	do-iterator-null	20000	1612	1119819	940146
alloc	candidate-04	2	do-iterator-null	40000	1612	1759819	1580146
alloc	candidate-04	2	table-iterator-null	20000	1614	1119739	940025
alloc	candidate-04	2	table-iterator-null	40000	1614	1759739	1580025
alloc	candidate-04	2	replaceall-simple	20000	101746	27530293	943388
alloc	candidate-04	2	replaceall-simple	40000	201746	54570293	1583388
instructions	base	1	scan-no-input	20000	605160277	-	-
instructions	base	1	scan-no-input	40000	1206782257	-	-
instructions	base	1	optionvalue-three-no-input	20000	201087575	-	-
instructions	base	1	optionvalue-three-no-input	40000	396399649	-	-
instructions	base	1	replacerepeated-no-input	20000	808713464	-	-
instructions	base	1	replacerepeated-no-input	40000	1613829168	-	-
instructions	base	1	dot-three-no-input	20000	5137818636	-	-
instructions	base	1	dot-three-no-input	40000	10268856112	-	-
instructions	base	1	differences-no-input	20000	203808966	-	-
instructions	base	1	differences-no-input	40000	402173916	-	-
instructions	base	1	do-count-null	20000	13473293	-	-
instructions	base	1	do-count-null	40000	23433627	-	-
instructions	base	1	do-iterator-null	20000	26289658	-	-
instructions	base	1	do-iterator-null	40000	49051093	-	-
instructions	base	1	table-iterator-null	20000	10511109	-	-
instructions	base	1	table-iterator-null	40000	17494444	-	-
instructions	base	1	replaceall-simple	20000	233119239	-	-
instructions	base	1	replaceall-simple	40000	461744240	-	-
instructions	base	2	scan-no-input	20000	605182683	-	-
instructions	base	2	scan-no-input	40000	1206810955	-	-
instructions	base	2	optionvalue-three-no-input	20000	201103532	-	-
instructions	base	2	optionvalue-three-no-input	40000	396384829	-	-
instructions	base	2	replacerepeated-no-input	20000	808714348	-	-
instructions	base	2	replacerepeated-no-input	40000	1614270895	-	-
instructions	base	2	dot-three-no-input	20000	5137813258	-	-
instructions	base	2	dot-three-no-input	40000	10268853795	-	-
instructions	base	2	differences-no-input	20000	203928166	-	-
instructions	base	2	differences-no-input	40000	401933898	-	-
instructions	base	2	do-count-null	20000	13472665	-	-
instructions	base	2	do-count-null	40000	23436989	-	-
instructions	base	2	do-iterator-null	20000	26290197	-	-
instructions	base	2	do-iterator-null	40000	49047460	-	-
instructions	base	2	table-iterator-null	20000	10511246	-	-
instructions	base	2	table-iterator-null	40000	17492414	-	-
instructions	base	2	replaceall-simple	20000	232678100	-	-
instructions	base	2	replaceall-simple	40000	462624156	-	-
instructions	candidate-04	1	scan-no-input	20000	603869199	-	-
instructions	candidate-04	1	scan-no-input	40000	1204152907	-	-
instructions	candidate-04	1	optionvalue-three-no-input	20000	200729603	-	-
instructions	candidate-04	1	optionvalue-three-no-input	40000	395676089	-	-
instructions	candidate-04	1	replacerepeated-no-input	20000	812035201	-	-
instructions	candidate-04	1	replacerepeated-no-input	40000	1620910499	-	-
instructions	candidate-04	1	dot-three-no-input	20000	5144759092	-	-
instructions	candidate-04	1	dot-three-no-input	40000	10282742837	-	-
instructions	candidate-04	1	differences-no-input	20000	205376937	-	-
instructions	candidate-04	1	differences-no-input	40000	405132122	-	-
instructions	candidate-04	1	do-count-null	20000	13455790	-	-
instructions	candidate-04	1	do-count-null	40000	23393023	-	-
instructions	candidate-04	1	do-iterator-null	20000	26270869	-	-
instructions	candidate-04	1	do-iterator-null	40000	49010335	-	-
instructions	candidate-04	1	table-iterator-null	20000	10513352	-	-
instructions	candidate-04	1	table-iterator-null	40000	17493983	-	-
instructions	candidate-04	1	replaceall-simple	20000	232120226	-	-
instructions	candidate-04	1	replaceall-simple	40000	459744871	-	-
instructions	candidate-04	2	scan-no-input	20000	603856863	-	-
instructions	candidate-04	2	scan-no-input	40000	1204151547	-	-
instructions	candidate-04	2	optionvalue-three-no-input	20000	200737030	-	-
instructions	candidate-04	2	optionvalue-three-no-input	40000	395676731	-	-
instructions	candidate-04	2	replacerepeated-no-input	20000	812035164	-	-
instructions	candidate-04	2	replacerepeated-no-input	40000	1620473357	-	-
instructions	candidate-04	2	dot-three-no-input	20000	5144743845	-	-
instructions	candidate-04	2	dot-three-no-input	40000	10282743278	-	-
instructions	candidate-04	2	differences-no-input	20000	205388177	-	-
instructions	candidate-04	2	differences-no-input	40000	405132536	-	-
instructions	candidate-04	2	do-count-null	20000	13454026	-	-
instructions	candidate-04	2	do-count-null	40000	23395093	-	-
instructions	candidate-04	2	do-iterator-null	20000	26267427	-	-
instructions	candidate-04	2	do-iterator-null	40000	49009061	-	-
instructions	candidate-04	2	table-iterator-null	20000	10513975	-	-
instructions	candidate-04	2	table-iterator-null	40000	17492126	-	-
instructions	candidate-04	2	replaceall-simple	20000	231679477	-	-
instructions	candidate-04	2	replaceall-simple	40000	460626131	-	-
```

**candidate-05 — Over 04, add a direct ordinary ReplaceRepeated pass loop with cold replacement/evaluation error capture. Preserve the existing resumed functions and frames.**

```text
rum-cli  db67a3f314dd8cddd8357d8cf57e96b8605b132f88ab1b403385059490f01ecc
rum-cli-alloc  b4ae73353d5e4820ef7bbef829b7e8af299b9f43331f03ddb18dee36efa65756
```

Verbatim reducer rows: allocation counts are the N/2N slopes; peak pairs are N,2N; both samples are explicit. Instruction columns are instructions per iteration and percent delta.

```tsv
label	alloc_count_per_iter_base/candidate_s1;s2	peak_base/candidate_N,2N_s1;s2	instr_base_s1	instr_candidate_s1	delta_s1_pct	instr_base_s2	instr_candidate_s2	delta_s2_pct
scan-no-input	33/33;33/33	942414,1582414/942414,1582414;942414,1582414/942414,1582414	30081.09900	30014.90780	-0.220042	30081.41360	30014.26535	-0.223222
optionvalue-three-no-input	9/9;9/9	9134068,9774068/9134068,9774068;9134068,9774068/9134068,9774068	9765.60370	9747.34740	-0.186945	9764.06485	9746.27925	-0.182154
replacerepeated-no-input	61/61;61/61	948049,1588049/948049,1588049;948049,1588049/948049,1588049	40255.78520	40125.82370	-0.322839	40277.82735	40125.73845	-0.377600
dot-three-no-input	596/596;596/596	8678236,9318236/8678236,9318236;8678236,9318236/8678236,9318236	256551.87380	256898.81415	+0.135232	256552.02685	256898.39490	+0.135009
differences-no-input	13/13;13/13	9591399,10231399/9591399,10231399;9591399,10231399/9591399,10231399	9918.24750	9985.55935	+0.678667	9900.28660	9985.50635	+0.860781
do-count-null	0/0;0/0	299668,299668/299668,299668;299668,299668/299668,299668	498.01670	496.93295	-0.217613	498.21620	497.06775	-0.230512
do-iterator-null	0/0;0/0	940146,1580146/940146,1580146;940146,1580146/940146,1580146	1138.07175	1136.95165	-0.098421	1137.86315	1137.04905	-0.071546
table-iterator-null	0/0;0/0	940025,1580025/940025,1580025;940025,1580025/940025,1580025	349.16675	349.04040	-0.036186	349.05840	349.06175	+0.000960
replaceall-simple	5/5;5/5	943388,1583388/943388,1583388;943388,1583388/943388,1583388	11431.25005	11381.09245	-0.438776	11497.30280	11403.23405	-0.818181
```

Raw N/2N counts for this build and its base, both samples, follow. Allocation rows contain count, total bytes and peak bytes; instruction rows contain the original perf counter.

```tsv
mode	build	sample	label	N	count_or_instructions	bytes_total	peak_live_bytes
alloc	base	1	scan-no-input	20000	661661	51204971	942414
alloc	base	1	scan-no-input	40000	1321661	101924971	1582414
alloc	base	1	optionvalue-three-no-input	20000	181794	30460125	9134068
alloc	base	1	optionvalue-three-no-input	40000	361794	53020125	9774068
alloc	base	1	replacerepeated-no-input	20000	1221717	154311068	948049
alloc	base	1	replacerepeated-no-input	40000	2441717	308131068	1588049
alloc	base	1	dot-three-no-input	20000	11921740	477979704	8678236
alloc	base	1	dot-three-no-input	40000	23841740	948059704	9318236
alloc	base	1	differences-no-input	20000	261670	21969680	9591399
alloc	base	1	differences-no-input	40000	521670	36049680	10231399
alloc	base	1	do-count-null	20000	1588	478931	299668
alloc	base	1	do-count-null	40000	1588	478931	299668
alloc	base	1	do-iterator-null	20000	1612	1119811	940146
alloc	base	1	do-iterator-null	40000	1612	1759811	1580146
alloc	base	1	table-iterator-null	20000	1614	1119731	940025
alloc	base	1	table-iterator-null	40000	1614	1759731	1580025
alloc	base	1	replaceall-simple	20000	101746	27530285	943388
alloc	base	1	replaceall-simple	40000	201746	54570285	1583388
alloc	base	2	scan-no-input	20000	661661	51204971	942414
alloc	base	2	scan-no-input	40000	1321661	101924971	1582414
alloc	base	2	optionvalue-three-no-input	20000	181794	30460125	9134068
alloc	base	2	optionvalue-three-no-input	40000	361794	53020125	9774068
alloc	base	2	replacerepeated-no-input	20000	1221717	154311068	948049
alloc	base	2	replacerepeated-no-input	40000	2441717	308131068	1588049
alloc	base	2	dot-three-no-input	20000	11921740	477979704	8678236
alloc	base	2	dot-three-no-input	40000	23841740	948059704	9318236
alloc	base	2	differences-no-input	20000	261670	21969680	9591399
alloc	base	2	differences-no-input	40000	521670	36049680	10231399
alloc	base	2	do-count-null	20000	1588	478931	299668
alloc	base	2	do-count-null	40000	1588	478931	299668
alloc	base	2	do-iterator-null	20000	1612	1119811	940146
alloc	base	2	do-iterator-null	40000	1612	1759811	1580146
alloc	base	2	table-iterator-null	20000	1614	1119731	940025
alloc	base	2	table-iterator-null	40000	1614	1759731	1580025
alloc	base	2	replaceall-simple	20000	101746	27530285	943388
alloc	base	2	replaceall-simple	40000	201746	54570285	1583388
alloc	candidate-05	1	scan-no-input	20000	661661	51204979	942414
alloc	candidate-05	1	scan-no-input	40000	1321661	101924979	1582414
alloc	candidate-05	1	optionvalue-three-no-input	20000	181794	30460133	9134068
alloc	candidate-05	1	optionvalue-three-no-input	40000	361794	53020133	9774068
alloc	candidate-05	1	replacerepeated-no-input	20000	1221717	154311076	948049
alloc	candidate-05	1	replacerepeated-no-input	40000	2441717	308131076	1588049
alloc	candidate-05	1	dot-three-no-input	20000	11921740	477979712	8678236
alloc	candidate-05	1	dot-three-no-input	40000	23841740	948059712	9318236
alloc	candidate-05	1	differences-no-input	20000	261670	21969688	9591399
alloc	candidate-05	1	differences-no-input	40000	521670	36049688	10231399
alloc	candidate-05	1	do-count-null	20000	1588	478939	299668
alloc	candidate-05	1	do-count-null	40000	1588	478939	299668
alloc	candidate-05	1	do-iterator-null	20000	1612	1119819	940146
alloc	candidate-05	1	do-iterator-null	40000	1612	1759819	1580146
alloc	candidate-05	1	table-iterator-null	20000	1614	1119739	940025
alloc	candidate-05	1	table-iterator-null	40000	1614	1759739	1580025
alloc	candidate-05	1	replaceall-simple	20000	101746	27530293	943388
alloc	candidate-05	1	replaceall-simple	40000	201746	54570293	1583388
alloc	candidate-05	2	scan-no-input	20000	661661	51204979	942414
alloc	candidate-05	2	scan-no-input	40000	1321661	101924979	1582414
alloc	candidate-05	2	optionvalue-three-no-input	20000	181794	30460133	9134068
alloc	candidate-05	2	optionvalue-three-no-input	40000	361794	53020133	9774068
alloc	candidate-05	2	replacerepeated-no-input	20000	1221717	154311076	948049
alloc	candidate-05	2	replacerepeated-no-input	40000	2441717	308131076	1588049
alloc	candidate-05	2	dot-three-no-input	20000	11921740	477979712	8678236
alloc	candidate-05	2	dot-three-no-input	40000	23841740	948059712	9318236
alloc	candidate-05	2	differences-no-input	20000	261670	21969688	9591399
alloc	candidate-05	2	differences-no-input	40000	521670	36049688	10231399
alloc	candidate-05	2	do-count-null	20000	1588	478939	299668
alloc	candidate-05	2	do-count-null	40000	1588	478939	299668
alloc	candidate-05	2	do-iterator-null	20000	1612	1119819	940146
alloc	candidate-05	2	do-iterator-null	40000	1612	1759819	1580146
alloc	candidate-05	2	table-iterator-null	20000	1614	1119739	940025
alloc	candidate-05	2	table-iterator-null	40000	1614	1759739	1580025
alloc	candidate-05	2	replaceall-simple	20000	101746	27530293	943388
alloc	candidate-05	2	replaceall-simple	40000	201746	54570293	1583388
instructions	base	1	scan-no-input	20000	605160277	-	-
instructions	base	1	scan-no-input	40000	1206782257	-	-
instructions	base	1	optionvalue-three-no-input	20000	201087575	-	-
instructions	base	1	optionvalue-three-no-input	40000	396399649	-	-
instructions	base	1	replacerepeated-no-input	20000	808713464	-	-
instructions	base	1	replacerepeated-no-input	40000	1613829168	-	-
instructions	base	1	dot-three-no-input	20000	5137818636	-	-
instructions	base	1	dot-three-no-input	40000	10268856112	-	-
instructions	base	1	differences-no-input	20000	203808966	-	-
instructions	base	1	differences-no-input	40000	402173916	-	-
instructions	base	1	do-count-null	20000	13473293	-	-
instructions	base	1	do-count-null	40000	23433627	-	-
instructions	base	1	do-iterator-null	20000	26289658	-	-
instructions	base	1	do-iterator-null	40000	49051093	-	-
instructions	base	1	table-iterator-null	20000	10511109	-	-
instructions	base	1	table-iterator-null	40000	17494444	-	-
instructions	base	1	replaceall-simple	20000	233119239	-	-
instructions	base	1	replaceall-simple	40000	461744240	-	-
instructions	base	2	scan-no-input	20000	605182683	-	-
instructions	base	2	scan-no-input	40000	1206810955	-	-
instructions	base	2	optionvalue-three-no-input	20000	201103532	-	-
instructions	base	2	optionvalue-three-no-input	40000	396384829	-	-
instructions	base	2	replacerepeated-no-input	20000	808714348	-	-
instructions	base	2	replacerepeated-no-input	40000	1614270895	-	-
instructions	base	2	dot-three-no-input	20000	5137813258	-	-
instructions	base	2	dot-three-no-input	40000	10268853795	-	-
instructions	base	2	differences-no-input	20000	203928166	-	-
instructions	base	2	differences-no-input	40000	401933898	-	-
instructions	base	2	do-count-null	20000	13472665	-	-
instructions	base	2	do-count-null	40000	23436989	-	-
instructions	base	2	do-iterator-null	20000	26290197	-	-
instructions	base	2	do-iterator-null	40000	49047460	-	-
instructions	base	2	table-iterator-null	20000	10511246	-	-
instructions	base	2	table-iterator-null	40000	17492414	-	-
instructions	base	2	replaceall-simple	20000	232678100	-	-
instructions	base	2	replaceall-simple	40000	462624156	-	-
instructions	candidate-05	1	scan-no-input	20000	603853965	-	-
instructions	candidate-05	1	scan-no-input	40000	1204152121	-	-
instructions	candidate-05	1	optionvalue-three-no-input	20000	200730835	-	-
instructions	candidate-05	1	optionvalue-three-no-input	40000	395677783	-	-
instructions	candidate-05	1	replacerepeated-no-input	20000	806115645	-	-
instructions	candidate-05	1	replacerepeated-no-input	40000	1608632119	-	-
instructions	candidate-05	1	dot-three-no-input	20000	5144757385	-	-
instructions	candidate-05	1	dot-three-no-input	40000	10282733668	-	-
instructions	candidate-05	1	differences-no-input	20000	205386868	-	-
instructions	candidate-05	1	differences-no-input	40000	405098055	-	-
instructions	candidate-05	1	do-count-null	20000	13453814	-	-
instructions	candidate-05	1	do-count-null	40000	23392473	-	-
instructions	candidate-05	1	do-iterator-null	20000	26270290	-	-
instructions	candidate-05	1	do-iterator-null	40000	49009323	-	-
instructions	candidate-05	1	table-iterator-null	20000	10510296	-	-
instructions	candidate-05	1	table-iterator-null	40000	17491104	-	-
instructions	candidate-05	1	replaceall-simple	20000	232122506	-	-
instructions	candidate-05	1	replaceall-simple	40000	459744355	-	-
instructions	candidate-05	2	scan-no-input	20000	603867383	-	-
instructions	candidate-05	2	scan-no-input	40000	1204152690	-	-
instructions	candidate-05	2	optionvalue-three-no-input	20000	200745835	-	-
instructions	candidate-05	2	optionvalue-three-no-input	40000	395671420	-	-
instructions	candidate-05	2	replacerepeated-no-input	20000	806115293	-	-
instructions	candidate-05	2	replacerepeated-no-input	40000	1608630062	-	-
instructions	candidate-05	2	dot-three-no-input	20000	5144762209	-	-
instructions	candidate-05	2	dot-three-no-input	40000	10282730107	-	-
instructions	candidate-05	2	differences-no-input	20000	205386885	-	-
instructions	candidate-05	2	differences-no-input	40000	405097012	-	-
instructions	candidate-05	2	do-count-null	20000	13453401	-	-
instructions	candidate-05	2	do-count-null	40000	23394756	-	-
instructions	candidate-05	2	do-iterator-null	20000	26268846	-	-
instructions	candidate-05	2	do-iterator-null	40000	49009827	-	-
instructions	candidate-05	2	table-iterator-null	20000	10512210	-	-
instructions	candidate-05	2	table-iterator-null	40000	17493445	-	-
instructions	candidate-05	2	replaceall-simple	20000	231679961	-	-
instructions	candidate-05	2	replaceall-simple	40000	459744642	-	-
```

**candidate-06 — Over 05, separate the ordinary Dot fold from its resumed cursor helper; calculate the next operand only on error. This is the retained best candidate.**

```text
rum-cli  18953372e2be9d4ec55d52aac82ae9b7f6b19048ce13b5b00121371d5f3f41ac
rum-cli-alloc  ad839c652ca347ecd9a077804a9224727a1df14b6881241e997c22ac3ba105e3
```

Verbatim reducer rows: allocation counts are the N/2N slopes; peak pairs are N,2N; both samples are explicit. Instruction columns are instructions per iteration and percent delta.

```tsv
label	alloc_count_per_iter_base/candidate_s1;s2	peak_base/candidate_N,2N_s1;s2	instr_base_s1	instr_candidate_s1	delta_s1_pct	instr_base_s2	instr_candidate_s2	delta_s2_pct
scan-no-input	33/33;33/33	942414,1582414/942414,1582414;942414,1582414/942414,1582414	30081.09900	30008.02470	-0.242924	30081.41360	30007.81565	-0.244663
optionvalue-three-no-input	9/9;9/9	9134068,9774068/9134068,9774068;9134068,9774068/9134068,9774068	9765.60370	9738.52010	-0.277337	9764.06485	9737.94730	-0.267486
replacerepeated-no-input	61/61;61/61	948049,1588049/948049,1588049;948049,1588049/948049,1588049	40255.78520	40139.74955	-0.288246	40277.82735	40117.76780	-0.397389
dot-three-no-input	596/596;596/596	8678236,9318236/8678236,9318236;8678236,9318236/8678236,9318236	256551.87380	256452.04930	-0.038910	256552.02685	256452.27880	-0.038880
differences-no-input	13/13;13/13	9591399,10231399/9591399,10231399;9591399,10231399/9591399,10231399	9918.24750	9987.15070	+0.694711	9900.28660	9987.12115	+0.877091
do-count-null	0/0;0/0	299668,299668/299668,299668;299668,299668/299668,299668	498.01670	497.07670	-0.188749	498.21620	497.11625	-0.220778
do-iterator-null	0/0;0/0	940146,1580146/940146,1580146;940146,1580146/940146,1580146	1138.07175	1137.00145	-0.094045	1137.86315	1137.08585	-0.068312
table-iterator-null	0/0;0/0	940025,1580025/940025,1580025;940025,1580025/940025,1580025	349.16675	349.24280	+0.021780	349.05840	349.08860	+0.008652
replaceall-simple	5/5;5/5	943388,1583388/943388,1583388;943388,1583388/943388,1583388	11431.25005	11381.16485	-0.438143	11497.30280	11447.39490	-0.434084
```

Raw N/2N counts for this build and its base, both samples, follow. Allocation rows contain count, total bytes and peak bytes; instruction rows contain the original perf counter.

```tsv
mode	build	sample	label	N	count_or_instructions	bytes_total	peak_live_bytes
alloc	base	1	scan-no-input	20000	661661	51204971	942414
alloc	base	1	scan-no-input	40000	1321661	101924971	1582414
alloc	base	1	optionvalue-three-no-input	20000	181794	30460125	9134068
alloc	base	1	optionvalue-three-no-input	40000	361794	53020125	9774068
alloc	base	1	replacerepeated-no-input	20000	1221717	154311068	948049
alloc	base	1	replacerepeated-no-input	40000	2441717	308131068	1588049
alloc	base	1	dot-three-no-input	20000	11921740	477979704	8678236
alloc	base	1	dot-three-no-input	40000	23841740	948059704	9318236
alloc	base	1	differences-no-input	20000	261670	21969680	9591399
alloc	base	1	differences-no-input	40000	521670	36049680	10231399
alloc	base	1	do-count-null	20000	1588	478931	299668
alloc	base	1	do-count-null	40000	1588	478931	299668
alloc	base	1	do-iterator-null	20000	1612	1119811	940146
alloc	base	1	do-iterator-null	40000	1612	1759811	1580146
alloc	base	1	table-iterator-null	20000	1614	1119731	940025
alloc	base	1	table-iterator-null	40000	1614	1759731	1580025
alloc	base	1	replaceall-simple	20000	101746	27530285	943388
alloc	base	1	replaceall-simple	40000	201746	54570285	1583388
alloc	base	2	scan-no-input	20000	661661	51204971	942414
alloc	base	2	scan-no-input	40000	1321661	101924971	1582414
alloc	base	2	optionvalue-three-no-input	20000	181794	30460125	9134068
alloc	base	2	optionvalue-three-no-input	40000	361794	53020125	9774068
alloc	base	2	replacerepeated-no-input	20000	1221717	154311068	948049
alloc	base	2	replacerepeated-no-input	40000	2441717	308131068	1588049
alloc	base	2	dot-three-no-input	20000	11921740	477979704	8678236
alloc	base	2	dot-three-no-input	40000	23841740	948059704	9318236
alloc	base	2	differences-no-input	20000	261670	21969680	9591399
alloc	base	2	differences-no-input	40000	521670	36049680	10231399
alloc	base	2	do-count-null	20000	1588	478931	299668
alloc	base	2	do-count-null	40000	1588	478931	299668
alloc	base	2	do-iterator-null	20000	1612	1119811	940146
alloc	base	2	do-iterator-null	40000	1612	1759811	1580146
alloc	base	2	table-iterator-null	20000	1614	1119731	940025
alloc	base	2	table-iterator-null	40000	1614	1759731	1580025
alloc	base	2	replaceall-simple	20000	101746	27530285	943388
alloc	base	2	replaceall-simple	40000	201746	54570285	1583388
alloc	candidate-06	1	scan-no-input	20000	661661	51204979	942414
alloc	candidate-06	1	scan-no-input	40000	1321661	101924979	1582414
alloc	candidate-06	1	optionvalue-three-no-input	20000	181794	30460133	9134068
alloc	candidate-06	1	optionvalue-three-no-input	40000	361794	53020133	9774068
alloc	candidate-06	1	replacerepeated-no-input	20000	1221717	154311076	948049
alloc	candidate-06	1	replacerepeated-no-input	40000	2441717	308131076	1588049
alloc	candidate-06	1	dot-three-no-input	20000	11921740	477979712	8678236
alloc	candidate-06	1	dot-three-no-input	40000	23841740	948059712	9318236
alloc	candidate-06	1	differences-no-input	20000	261670	21969688	9591399
alloc	candidate-06	1	differences-no-input	40000	521670	36049688	10231399
alloc	candidate-06	1	do-count-null	20000	1588	478939	299668
alloc	candidate-06	1	do-count-null	40000	1588	478939	299668
alloc	candidate-06	1	do-iterator-null	20000	1612	1119819	940146
alloc	candidate-06	1	do-iterator-null	40000	1612	1759819	1580146
alloc	candidate-06	1	table-iterator-null	20000	1614	1119739	940025
alloc	candidate-06	1	table-iterator-null	40000	1614	1759739	1580025
alloc	candidate-06	1	replaceall-simple	20000	101746	27530293	943388
alloc	candidate-06	1	replaceall-simple	40000	201746	54570293	1583388
alloc	candidate-06	2	scan-no-input	20000	661661	51204979	942414
alloc	candidate-06	2	scan-no-input	40000	1321661	101924979	1582414
alloc	candidate-06	2	optionvalue-three-no-input	20000	181794	30460133	9134068
alloc	candidate-06	2	optionvalue-three-no-input	40000	361794	53020133	9774068
alloc	candidate-06	2	replacerepeated-no-input	20000	1221717	154311076	948049
alloc	candidate-06	2	replacerepeated-no-input	40000	2441717	308131076	1588049
alloc	candidate-06	2	dot-three-no-input	20000	11921740	477979712	8678236
alloc	candidate-06	2	dot-three-no-input	40000	23841740	948059712	9318236
alloc	candidate-06	2	differences-no-input	20000	261670	21969688	9591399
alloc	candidate-06	2	differences-no-input	40000	521670	36049688	10231399
alloc	candidate-06	2	do-count-null	20000	1588	478939	299668
alloc	candidate-06	2	do-count-null	40000	1588	478939	299668
alloc	candidate-06	2	do-iterator-null	20000	1612	1119819	940146
alloc	candidate-06	2	do-iterator-null	40000	1612	1759819	1580146
alloc	candidate-06	2	table-iterator-null	20000	1614	1119739	940025
alloc	candidate-06	2	table-iterator-null	40000	1614	1759739	1580025
alloc	candidate-06	2	replaceall-simple	20000	101746	27530293	943388
alloc	candidate-06	2	replaceall-simple	40000	201746	54570293	1583388
instructions	base	1	scan-no-input	20000	605160277	-	-
instructions	base	1	scan-no-input	40000	1206782257	-	-
instructions	base	1	optionvalue-three-no-input	20000	201087575	-	-
instructions	base	1	optionvalue-three-no-input	40000	396399649	-	-
instructions	base	1	replacerepeated-no-input	20000	808713464	-	-
instructions	base	1	replacerepeated-no-input	40000	1613829168	-	-
instructions	base	1	dot-three-no-input	20000	5137818636	-	-
instructions	base	1	dot-three-no-input	40000	10268856112	-	-
instructions	base	1	differences-no-input	20000	203808966	-	-
instructions	base	1	differences-no-input	40000	402173916	-	-
instructions	base	1	do-count-null	20000	13473293	-	-
instructions	base	1	do-count-null	40000	23433627	-	-
instructions	base	1	do-iterator-null	20000	26289658	-	-
instructions	base	1	do-iterator-null	40000	49051093	-	-
instructions	base	1	table-iterator-null	20000	10511109	-	-
instructions	base	1	table-iterator-null	40000	17494444	-	-
instructions	base	1	replaceall-simple	20000	233119239	-	-
instructions	base	1	replaceall-simple	40000	461744240	-	-
instructions	base	2	scan-no-input	20000	605182683	-	-
instructions	base	2	scan-no-input	40000	1206810955	-	-
instructions	base	2	optionvalue-three-no-input	20000	201103532	-	-
instructions	base	2	optionvalue-three-no-input	40000	396384829	-	-
instructions	base	2	replacerepeated-no-input	20000	808714348	-	-
instructions	base	2	replacerepeated-no-input	40000	1614270895	-	-
instructions	base	2	dot-three-no-input	20000	5137813258	-	-
instructions	base	2	dot-three-no-input	40000	10268853795	-	-
instructions	base	2	differences-no-input	20000	203928166	-	-
instructions	base	2	differences-no-input	40000	401933898	-	-
instructions	base	2	do-count-null	20000	13472665	-	-
instructions	base	2	do-count-null	40000	23436989	-	-
instructions	base	2	do-iterator-null	20000	26290197	-	-
instructions	base	2	do-iterator-null	40000	49047460	-	-
instructions	base	2	table-iterator-null	20000	10511246	-	-
instructions	base	2	table-iterator-null	40000	17492414	-	-
instructions	base	2	replaceall-simple	20000	232678100	-	-
instructions	base	2	replaceall-simple	40000	462624156	-	-
instructions	candidate-06	1	scan-no-input	20000	603713318	-	-
instructions	candidate-06	1	scan-no-input	40000	1203873812	-	-
instructions	candidate-06	1	optionvalue-three-no-input	20000	200573122	-	-
instructions	candidate-06	1	optionvalue-three-no-input	40000	395343524	-	-
instructions	candidate-06	1	replacerepeated-no-input	20000	805955226	-	-
instructions	candidate-06	1	replacerepeated-no-input	40000	1608750217	-	-
instructions	candidate-06	1	dot-three-no-input	20000	5135813577	-	-
instructions	candidate-06	1	dot-three-no-input	40000	10264854563	-	-
instructions	candidate-06	1	differences-no-input	20000	205387079	-	-
instructions	candidate-06	1	differences-no-input	40000	405130093	-	-
instructions	candidate-06	1	do-count-null	20000	13452931	-	-
instructions	candidate-06	1	do-count-null	40000	23394465	-	-
instructions	candidate-06	1	do-iterator-null	20000	26267906	-	-
instructions	candidate-06	1	do-iterator-null	40000	49007935	-	-
instructions	candidate-06	1	table-iterator-null	20000	10512173	-	-
instructions	candidate-06	1	table-iterator-null	40000	17497029	-	-
instructions	candidate-06	1	replaceall-simple	20000	232120667	-	-
instructions	candidate-06	1	replaceall-simple	40000	459743964	-	-
instructions	candidate-06	2	scan-no-input	20000	603713993	-	-
instructions	candidate-06	2	scan-no-input	40000	1203870306	-	-
instructions	candidate-06	2	optionvalue-three-no-input	20000	200581870	-	-
instructions	candidate-06	2	optionvalue-three-no-input	40000	395340816	-	-
instructions	candidate-06	2	replacerepeated-no-input	20000	805955599	-	-
instructions	candidate-06	2	replacerepeated-no-input	40000	1608310955	-	-
instructions	candidate-06	2	dot-three-no-input	20000	5135815893	-	-
instructions	candidate-06	2	dot-three-no-input	40000	10264861469	-	-
instructions	candidate-06	2	differences-no-input	20000	205388785	-	-
instructions	candidate-06	2	differences-no-input	40000	405131208	-	-
instructions	candidate-06	2	do-count-null	20000	13451804	-	-
instructions	candidate-06	2	do-count-null	40000	23394129	-	-
instructions	candidate-06	2	do-iterator-null	20000	26268541	-	-
instructions	candidate-06	2	do-iterator-null	40000	49010258	-	-
instructions	candidate-06	2	table-iterator-null	20000	10511545	-	-
instructions	candidate-06	2	table-iterator-null	40000	17493317	-	-
instructions	candidate-06	2	replaceall-simple	20000	231681272	-	-
instructions	candidate-06	2	replaceall-simple	40000	460629170	-	-
```

**candidate-07 — Over 06, outline only the symbolic difference_expr fallback behind #[inline(never)], leaving integer arithmetic unchanged. Rejected.**

```text
rum-cli  bd0fb3a2a6882edc3c54c087479aef8585864684261498e8a72ab06c95a5cdb6
rum-cli-alloc  b0a85784987908cd7f8eb62c2072ad295918f2aa3a52bf0078f7a5a379f7823f
```

Verbatim reducer rows: allocation counts are the N/2N slopes; peak pairs are N,2N; both samples are explicit. Instruction columns are instructions per iteration and percent delta.

```tsv
label	alloc_count_per_iter_base/candidate_s1;s2	peak_base/candidate_N,2N_s1;s2	instr_base_s1	instr_candidate_s1	delta_s1_pct	instr_base_s2	instr_candidate_s2	delta_s2_pct
scan-no-input	33/33;33/33	942414,1582414/942414,1582414;942414,1582414/942414,1582414	30081.09900	30007.86185	-0.243466	30081.41360	30007.95465	-0.244200
optionvalue-three-no-input	9/9;9/9	9134068,9774068/9134068,9774068;9134068,9774068/9134068,9774068	9765.60370	9738.36830	-0.278891	9764.06485	9738.46510	-0.262183
replacerepeated-no-input	61/61;61/61	948049,1588049/948049,1588049;948049,1588049/948049,1588049	40255.78520	40117.79590	-0.342781	40277.82735	40117.77435	-0.397372
dot-three-no-input	596/596;596/596	8678236,9318236/8678236,9318236;8678236,9318236/8678236,9318236	256551.87380	256452.00415	-0.038928	256552.02685	256452.12955	-0.038938
differences-no-input	13/13;13/13	9591399,10231399/9591399,10231399;9591399,10231399/9591399,10231399	9918.24750	9987.29950	+0.696212	9900.28660	9987.55130	+0.881436
do-count-null	0/0;0/0	299668,299668/299668,299668;299668,299668/299668,299668	498.01670	497.02460	-0.199210	498.21620	496.91155	-0.261864
do-iterator-null	0/0;0/0	940146,1580146/940146,1580146;940146,1580146/940146,1580146	1138.07175	1136.91270	-0.101843	1137.86315	1137.04480	-0.071920
table-iterator-null	0/0;0/0	940025,1580025/940025,1580025;940025,1580025/940025,1580025	349.16675	349.09605	-0.020248	349.05840	349.05490	-0.001003
replaceall-simple	5/5;5/5	943388,1583388/943388,1583388;943388,1583388/943388,1583388	11431.25005	11447.45445	+0.141755	11497.30280	11403.45415	-0.816267
```

Raw N/2N counts for this build and its base, both samples, follow. Allocation rows contain count, total bytes and peak bytes; instruction rows contain the original perf counter.

```tsv
mode	build	sample	label	N	count_or_instructions	bytes_total	peak_live_bytes
alloc	base	1	scan-no-input	20000	661661	51204971	942414
alloc	base	1	scan-no-input	40000	1321661	101924971	1582414
alloc	base	1	optionvalue-three-no-input	20000	181794	30460125	9134068
alloc	base	1	optionvalue-three-no-input	40000	361794	53020125	9774068
alloc	base	1	replacerepeated-no-input	20000	1221717	154311068	948049
alloc	base	1	replacerepeated-no-input	40000	2441717	308131068	1588049
alloc	base	1	dot-three-no-input	20000	11921740	477979704	8678236
alloc	base	1	dot-three-no-input	40000	23841740	948059704	9318236
alloc	base	1	differences-no-input	20000	261670	21969680	9591399
alloc	base	1	differences-no-input	40000	521670	36049680	10231399
alloc	base	1	do-count-null	20000	1588	478931	299668
alloc	base	1	do-count-null	40000	1588	478931	299668
alloc	base	1	do-iterator-null	20000	1612	1119811	940146
alloc	base	1	do-iterator-null	40000	1612	1759811	1580146
alloc	base	1	table-iterator-null	20000	1614	1119731	940025
alloc	base	1	table-iterator-null	40000	1614	1759731	1580025
alloc	base	1	replaceall-simple	20000	101746	27530285	943388
alloc	base	1	replaceall-simple	40000	201746	54570285	1583388
alloc	base	2	scan-no-input	20000	661661	51204971	942414
alloc	base	2	scan-no-input	40000	1321661	101924971	1582414
alloc	base	2	optionvalue-three-no-input	20000	181794	30460125	9134068
alloc	base	2	optionvalue-three-no-input	40000	361794	53020125	9774068
alloc	base	2	replacerepeated-no-input	20000	1221717	154311068	948049
alloc	base	2	replacerepeated-no-input	40000	2441717	308131068	1588049
alloc	base	2	dot-three-no-input	20000	11921740	477979704	8678236
alloc	base	2	dot-three-no-input	40000	23841740	948059704	9318236
alloc	base	2	differences-no-input	20000	261670	21969680	9591399
alloc	base	2	differences-no-input	40000	521670	36049680	10231399
alloc	base	2	do-count-null	20000	1588	478931	299668
alloc	base	2	do-count-null	40000	1588	478931	299668
alloc	base	2	do-iterator-null	20000	1612	1119811	940146
alloc	base	2	do-iterator-null	40000	1612	1759811	1580146
alloc	base	2	table-iterator-null	20000	1614	1119731	940025
alloc	base	2	table-iterator-null	40000	1614	1759731	1580025
alloc	base	2	replaceall-simple	20000	101746	27530285	943388
alloc	base	2	replaceall-simple	40000	201746	54570285	1583388
alloc	candidate-07	1	scan-no-input	20000	661661	51204979	942414
alloc	candidate-07	1	scan-no-input	40000	1321661	101924979	1582414
alloc	candidate-07	1	optionvalue-three-no-input	20000	181794	30460133	9134068
alloc	candidate-07	1	optionvalue-three-no-input	40000	361794	53020133	9774068
alloc	candidate-07	1	replacerepeated-no-input	20000	1221717	154311076	948049
alloc	candidate-07	1	replacerepeated-no-input	40000	2441717	308131076	1588049
alloc	candidate-07	1	dot-three-no-input	20000	11921740	477979712	8678236
alloc	candidate-07	1	dot-three-no-input	40000	23841740	948059712	9318236
alloc	candidate-07	1	differences-no-input	20000	261670	21969688	9591399
alloc	candidate-07	1	differences-no-input	40000	521670	36049688	10231399
alloc	candidate-07	1	do-count-null	20000	1588	478939	299668
alloc	candidate-07	1	do-count-null	40000	1588	478939	299668
alloc	candidate-07	1	do-iterator-null	20000	1612	1119819	940146
alloc	candidate-07	1	do-iterator-null	40000	1612	1759819	1580146
alloc	candidate-07	1	table-iterator-null	20000	1614	1119739	940025
alloc	candidate-07	1	table-iterator-null	40000	1614	1759739	1580025
alloc	candidate-07	1	replaceall-simple	20000	101746	27530293	943388
alloc	candidate-07	1	replaceall-simple	40000	201746	54570293	1583388
alloc	candidate-07	2	scan-no-input	20000	661661	51204979	942414
alloc	candidate-07	2	scan-no-input	40000	1321661	101924979	1582414
alloc	candidate-07	2	optionvalue-three-no-input	20000	181794	30460133	9134068
alloc	candidate-07	2	optionvalue-three-no-input	40000	361794	53020133	9774068
alloc	candidate-07	2	replacerepeated-no-input	20000	1221717	154311076	948049
alloc	candidate-07	2	replacerepeated-no-input	40000	2441717	308131076	1588049
alloc	candidate-07	2	dot-three-no-input	20000	11921740	477979712	8678236
alloc	candidate-07	2	dot-three-no-input	40000	23841740	948059712	9318236
alloc	candidate-07	2	differences-no-input	20000	261670	21969688	9591399
alloc	candidate-07	2	differences-no-input	40000	521670	36049688	10231399
alloc	candidate-07	2	do-count-null	20000	1588	478939	299668
alloc	candidate-07	2	do-count-null	40000	1588	478939	299668
alloc	candidate-07	2	do-iterator-null	20000	1612	1119819	940146
alloc	candidate-07	2	do-iterator-null	40000	1612	1759819	1580146
alloc	candidate-07	2	table-iterator-null	20000	1614	1119739	940025
alloc	candidate-07	2	table-iterator-null	40000	1614	1759739	1580025
alloc	candidate-07	2	replaceall-simple	20000	101746	27530293	943388
alloc	candidate-07	2	replaceall-simple	40000	201746	54570293	1583388
instructions	base	1	scan-no-input	20000	605160277	-	-
instructions	base	1	scan-no-input	40000	1206782257	-	-
instructions	base	1	optionvalue-three-no-input	20000	201087575	-	-
instructions	base	1	optionvalue-three-no-input	40000	396399649	-	-
instructions	base	1	replacerepeated-no-input	20000	808713464	-	-
instructions	base	1	replacerepeated-no-input	40000	1613829168	-	-
instructions	base	1	dot-three-no-input	20000	5137818636	-	-
instructions	base	1	dot-three-no-input	40000	10268856112	-	-
instructions	base	1	differences-no-input	20000	203808966	-	-
instructions	base	1	differences-no-input	40000	402173916	-	-
instructions	base	1	do-count-null	20000	13473293	-	-
instructions	base	1	do-count-null	40000	23433627	-	-
instructions	base	1	do-iterator-null	20000	26289658	-	-
instructions	base	1	do-iterator-null	40000	49051093	-	-
instructions	base	1	table-iterator-null	20000	10511109	-	-
instructions	base	1	table-iterator-null	40000	17494444	-	-
instructions	base	1	replaceall-simple	20000	233119239	-	-
instructions	base	1	replaceall-simple	40000	461744240	-	-
instructions	base	2	scan-no-input	20000	605182683	-	-
instructions	base	2	scan-no-input	40000	1206810955	-	-
instructions	base	2	optionvalue-three-no-input	20000	201103532	-	-
instructions	base	2	optionvalue-three-no-input	40000	396384829	-	-
instructions	base	2	replacerepeated-no-input	20000	808714348	-	-
instructions	base	2	replacerepeated-no-input	40000	1614270895	-	-
instructions	base	2	dot-three-no-input	20000	5137813258	-	-
instructions	base	2	dot-three-no-input	40000	10268853795	-	-
instructions	base	2	differences-no-input	20000	203928166	-	-
instructions	base	2	differences-no-input	40000	401933898	-	-
instructions	base	2	do-count-null	20000	13472665	-	-
instructions	base	2	do-count-null	40000	23436989	-	-
instructions	base	2	do-iterator-null	20000	26290197	-	-
instructions	base	2	do-iterator-null	40000	49047460	-	-
instructions	base	2	table-iterator-null	20000	10511246	-	-
instructions	base	2	table-iterator-null	40000	17492414	-	-
instructions	base	2	replaceall-simple	20000	232678100	-	-
instructions	base	2	replaceall-simple	40000	462624156	-	-
instructions	candidate-07	1	scan-no-input	20000	603714131	-	-
instructions	candidate-07	1	scan-no-input	40000	1203871368	-	-
instructions	candidate-07	1	optionvalue-three-no-input	20000	200573241	-	-
instructions	candidate-07	1	optionvalue-three-no-input	40000	395340607	-	-
instructions	candidate-07	1	replacerepeated-no-input	20000	805956757	-	-
instructions	candidate-07	1	replacerepeated-no-input	40000	1608312675	-	-
instructions	candidate-07	1	dot-three-no-input	20000	5135818863	-	-
instructions	candidate-07	1	dot-three-no-input	40000	10264858946	-	-
instructions	candidate-07	1	differences-no-input	20000	205386250	-	-
instructions	candidate-07	1	differences-no-input	40000	405132240	-	-
instructions	candidate-07	1	do-count-null	20000	13451510	-	-
instructions	candidate-07	1	do-count-null	40000	23392002	-	-
instructions	candidate-07	1	do-iterator-null	20000	26271331	-	-
instructions	candidate-07	1	do-iterator-null	40000	49009585	-	-
instructions	candidate-07	1	table-iterator-null	20000	10510842	-	-
instructions	candidate-07	1	table-iterator-null	40000	17492763	-	-
instructions	candidate-07	1	replaceall-simple	20000	231678039	-	-
instructions	candidate-07	1	replaceall-simple	40000	460627128	-	-
instructions	candidate-07	2	scan-no-input	20000	603713882	-	-
instructions	candidate-07	2	scan-no-input	40000	1203872975	-	-
instructions	candidate-07	2	optionvalue-three-no-input	20000	200587915	-	-
instructions	candidate-07	2	optionvalue-three-no-input	40000	395357217	-	-
instructions	candidate-07	2	replacerepeated-no-input	20000	805956011	-	-
instructions	candidate-07	2	replacerepeated-no-input	40000	1608311498	-	-
instructions	candidate-07	2	dot-three-no-input	20000	5135817845	-	-
instructions	candidate-07	2	dot-three-no-input	40000	10264860436	-	-
instructions	candidate-07	2	differences-no-input	20000	205379345	-	-
instructions	candidate-07	2	differences-no-input	40000	405130371	-	-
instructions	candidate-07	2	do-count-null	20000	13454040	-	-
instructions	candidate-07	2	do-count-null	40000	23392271	-	-
instructions	candidate-07	2	do-iterator-null	20000	26269500	-	-
instructions	candidate-07	2	do-iterator-null	40000	49010396	-	-
instructions	candidate-07	2	table-iterator-null	20000	10512700	-	-
instructions	candidate-07	2	table-iterator-null	40000	17493798	-	-
instructions	candidate-07	2	replaceall-simple	20000	231679544	-	-
instructions	candidate-07	2	replaceall-simple	40000	459748627	-	-
```

**candidate-08 — Over 07, put the exact-integer subtraction arm directly in the ordinary row loop; the symbolic arm uses the outlined fallback. Also apply the formatter-only OptionValue call wrap identified by fmt. Rejected; stop-loss triggered.**

```text
rum-cli  3264a577ff1a0dacdfe8aa26b8cf69f7324dad73c8ec2f2942b0171b54e9dc8e
rum-cli-alloc  fdf16853a3c37a805b2167e3f04b455a70abcd8009e6da84cee51415772c4055
```

Verbatim reducer rows: allocation counts are the N/2N slopes; peak pairs are N,2N; both samples are explicit. Instruction columns are instructions per iteration and percent delta.

```tsv
label	alloc_count_per_iter_base/candidate_s1;s2	peak_base/candidate_N,2N_s1;s2	instr_base_s1	instr_candidate_s1	delta_s1_pct	instr_base_s2	instr_candidate_s2	delta_s2_pct
scan-no-input	33/33;33/33	942414,1582414/942414,1582414;942414,1582414/942414,1582414	30081.09900	30013.59685	-0.224401	30081.41360	30013.62185	-0.225361
optionvalue-three-no-input	9/9;9/9	9134068,9774068/9134068,9774068;9134068,9774068/9134068,9774068	9765.60370	9746.75265	-0.193035	9764.06485	9747.01300	-0.174639
replacerepeated-no-input	61/61;61/61	948049,1588049/948049,1588049;948049,1588049/948049,1588049	40255.78520	40114.72990	-0.350398	40277.82735	40147.87020	-0.322652
dot-three-no-input	596/596;596/596	8678236,9318236/8678236,9318236;8678236,9318236/8678236,9318236	256551.87380	256728.22860	+0.068740	256552.02685	256728.30955	+0.068712
differences-no-input	13/13;13/13	9591399,10231399/9591399,10231399;9591399,10231399/9591399,10231399	9918.24750	9998.22425	+0.806360	9900.28660	9996.62910	+0.973128
do-count-null	0/0;0/0	299668,299668/299668,299668;299668,299668/299668,299668	498.01670	497.01005	-0.202132	498.21620	496.95070	-0.254006
do-iterator-null	0/0;0/0	940146,1580146/940146,1580146;940146,1580146/940146,1580146	1138.07175	1137.11450	-0.084112	1137.86315	1137.14160	-0.063413
table-iterator-null	0/0;0/0	940025,1580025/940025,1580025;940025,1580025/940025,1580025	349.16675	349.04685	-0.034339	349.05840	349.09945	+0.011760
replaceall-simple	5/5;5/5	943388,1583388/943388,1583388;943388,1583388/943388,1583388	11431.25005	11403.31490	-0.244375	11497.30280	11403.40915	-0.816658
```

Raw N/2N counts for this build and its base, both samples, follow. Allocation rows contain count, total bytes and peak bytes; instruction rows contain the original perf counter.

```tsv
mode	build	sample	label	N	count_or_instructions	bytes_total	peak_live_bytes
alloc	base	1	scan-no-input	20000	661661	51204971	942414
alloc	base	1	scan-no-input	40000	1321661	101924971	1582414
alloc	base	1	optionvalue-three-no-input	20000	181794	30460125	9134068
alloc	base	1	optionvalue-three-no-input	40000	361794	53020125	9774068
alloc	base	1	replacerepeated-no-input	20000	1221717	154311068	948049
alloc	base	1	replacerepeated-no-input	40000	2441717	308131068	1588049
alloc	base	1	dot-three-no-input	20000	11921740	477979704	8678236
alloc	base	1	dot-three-no-input	40000	23841740	948059704	9318236
alloc	base	1	differences-no-input	20000	261670	21969680	9591399
alloc	base	1	differences-no-input	40000	521670	36049680	10231399
alloc	base	1	do-count-null	20000	1588	478931	299668
alloc	base	1	do-count-null	40000	1588	478931	299668
alloc	base	1	do-iterator-null	20000	1612	1119811	940146
alloc	base	1	do-iterator-null	40000	1612	1759811	1580146
alloc	base	1	table-iterator-null	20000	1614	1119731	940025
alloc	base	1	table-iterator-null	40000	1614	1759731	1580025
alloc	base	1	replaceall-simple	20000	101746	27530285	943388
alloc	base	1	replaceall-simple	40000	201746	54570285	1583388
alloc	base	2	scan-no-input	20000	661661	51204971	942414
alloc	base	2	scan-no-input	40000	1321661	101924971	1582414
alloc	base	2	optionvalue-three-no-input	20000	181794	30460125	9134068
alloc	base	2	optionvalue-three-no-input	40000	361794	53020125	9774068
alloc	base	2	replacerepeated-no-input	20000	1221717	154311068	948049
alloc	base	2	replacerepeated-no-input	40000	2441717	308131068	1588049
alloc	base	2	dot-three-no-input	20000	11921740	477979704	8678236
alloc	base	2	dot-three-no-input	40000	23841740	948059704	9318236
alloc	base	2	differences-no-input	20000	261670	21969680	9591399
alloc	base	2	differences-no-input	40000	521670	36049680	10231399
alloc	base	2	do-count-null	20000	1588	478931	299668
alloc	base	2	do-count-null	40000	1588	478931	299668
alloc	base	2	do-iterator-null	20000	1612	1119811	940146
alloc	base	2	do-iterator-null	40000	1612	1759811	1580146
alloc	base	2	table-iterator-null	20000	1614	1119731	940025
alloc	base	2	table-iterator-null	40000	1614	1759731	1580025
alloc	base	2	replaceall-simple	20000	101746	27530285	943388
alloc	base	2	replaceall-simple	40000	201746	54570285	1583388
alloc	candidate-08	1	scan-no-input	20000	661661	51204979	942414
alloc	candidate-08	1	scan-no-input	40000	1321661	101924979	1582414
alloc	candidate-08	1	optionvalue-three-no-input	20000	181794	30460133	9134068
alloc	candidate-08	1	optionvalue-three-no-input	40000	361794	53020133	9774068
alloc	candidate-08	1	replacerepeated-no-input	20000	1221717	154311076	948049
alloc	candidate-08	1	replacerepeated-no-input	40000	2441717	308131076	1588049
alloc	candidate-08	1	dot-three-no-input	20000	11921740	477979712	8678236
alloc	candidate-08	1	dot-three-no-input	40000	23841740	948059712	9318236
alloc	candidate-08	1	differences-no-input	20000	261670	21969688	9591399
alloc	candidate-08	1	differences-no-input	40000	521670	36049688	10231399
alloc	candidate-08	1	do-count-null	20000	1588	478939	299668
alloc	candidate-08	1	do-count-null	40000	1588	478939	299668
alloc	candidate-08	1	do-iterator-null	20000	1612	1119819	940146
alloc	candidate-08	1	do-iterator-null	40000	1612	1759819	1580146
alloc	candidate-08	1	table-iterator-null	20000	1614	1119739	940025
alloc	candidate-08	1	table-iterator-null	40000	1614	1759739	1580025
alloc	candidate-08	1	replaceall-simple	20000	101746	27530293	943388
alloc	candidate-08	1	replaceall-simple	40000	201746	54570293	1583388
alloc	candidate-08	2	scan-no-input	20000	661661	51204979	942414
alloc	candidate-08	2	scan-no-input	40000	1321661	101924979	1582414
alloc	candidate-08	2	optionvalue-three-no-input	20000	181794	30460133	9134068
alloc	candidate-08	2	optionvalue-three-no-input	40000	361794	53020133	9774068
alloc	candidate-08	2	replacerepeated-no-input	20000	1221717	154311076	948049
alloc	candidate-08	2	replacerepeated-no-input	40000	2441717	308131076	1588049
alloc	candidate-08	2	dot-three-no-input	20000	11921740	477979712	8678236
alloc	candidate-08	2	dot-three-no-input	40000	23841740	948059712	9318236
alloc	candidate-08	2	differences-no-input	20000	261670	21969688	9591399
alloc	candidate-08	2	differences-no-input	40000	521670	36049688	10231399
alloc	candidate-08	2	do-count-null	20000	1588	478939	299668
alloc	candidate-08	2	do-count-null	40000	1588	478939	299668
alloc	candidate-08	2	do-iterator-null	20000	1612	1119819	940146
alloc	candidate-08	2	do-iterator-null	40000	1612	1759819	1580146
alloc	candidate-08	2	table-iterator-null	20000	1614	1119739	940025
alloc	candidate-08	2	table-iterator-null	40000	1614	1759739	1580025
alloc	candidate-08	2	replaceall-simple	20000	101746	27530293	943388
alloc	candidate-08	2	replaceall-simple	40000	201746	54570293	1583388
instructions	base	1	scan-no-input	20000	605160277	-	-
instructions	base	1	scan-no-input	40000	1206782257	-	-
instructions	base	1	optionvalue-three-no-input	20000	201087575	-	-
instructions	base	1	optionvalue-three-no-input	40000	396399649	-	-
instructions	base	1	replacerepeated-no-input	20000	808713464	-	-
instructions	base	1	replacerepeated-no-input	40000	1613829168	-	-
instructions	base	1	dot-three-no-input	20000	5137818636	-	-
instructions	base	1	dot-three-no-input	40000	10268856112	-	-
instructions	base	1	differences-no-input	20000	203808966	-	-
instructions	base	1	differences-no-input	40000	402173916	-	-
instructions	base	1	do-count-null	20000	13473293	-	-
instructions	base	1	do-count-null	40000	23433627	-	-
instructions	base	1	do-iterator-null	20000	26289658	-	-
instructions	base	1	do-iterator-null	40000	49051093	-	-
instructions	base	1	table-iterator-null	20000	10511109	-	-
instructions	base	1	table-iterator-null	40000	17494444	-	-
instructions	base	1	replaceall-simple	20000	233119239	-	-
instructions	base	1	replaceall-simple	40000	461744240	-	-
instructions	base	2	scan-no-input	20000	605182683	-	-
instructions	base	2	scan-no-input	40000	1206810955	-	-
instructions	base	2	optionvalue-three-no-input	20000	201103532	-	-
instructions	base	2	optionvalue-three-no-input	40000	396384829	-	-
instructions	base	2	replacerepeated-no-input	20000	808714348	-	-
instructions	base	2	replacerepeated-no-input	40000	1614270895	-	-
instructions	base	2	dot-three-no-input	20000	5137813258	-	-
instructions	base	2	dot-three-no-input	40000	10268853795	-	-
instructions	base	2	differences-no-input	20000	203928166	-	-
instructions	base	2	differences-no-input	40000	401933898	-	-
instructions	base	2	do-count-null	20000	13472665	-	-
instructions	base	2	do-count-null	40000	23436989	-	-
instructions	base	2	do-iterator-null	20000	26290197	-	-
instructions	base	2	do-iterator-null	40000	49047460	-	-
instructions	base	2	table-iterator-null	20000	10511246	-	-
instructions	base	2	table-iterator-null	40000	17492414	-	-
instructions	base	2	replaceall-simple	20000	232678100	-	-
instructions	base	2	replaceall-simple	40000	462624156	-	-
instructions	candidate-08	1	scan-no-input	20000	603883282	-	-
instructions	candidate-08	1	scan-no-input	40000	1204155219	-	-
instructions	candidate-08	1	optionvalue-three-no-input	20000	200740807	-	-
instructions	candidate-08	1	optionvalue-three-no-input	40000	395675860	-	-
instructions	candidate-08	1	replacerepeated-no-input	20000	806335948	-	-
instructions	candidate-08	1	replacerepeated-no-input	40000	1608630546	-	-
instructions	candidate-08	1	dot-three-no-input	20000	5141333922	-	-
instructions	candidate-08	1	dot-three-no-input	40000	10275898494	-	-
instructions	candidate-08	1	differences-no-input	20000	205606003	-	-
instructions	candidate-08	1	differences-no-input	40000	405570488	-	-
instructions	candidate-08	1	do-count-null	20000	13452611	-	-
instructions	candidate-08	1	do-count-null	40000	23392812	-	-
instructions	candidate-08	1	do-iterator-null	20000	26269218	-	-
instructions	candidate-08	1	do-iterator-null	40000	49011508	-	-
instructions	candidate-08	1	table-iterator-null	20000	10511257	-	-
instructions	candidate-08	1	table-iterator-null	40000	17492194	-	-
instructions	candidate-08	1	replaceall-simple	20000	231678933	-	-
instructions	candidate-08	1	replaceall-simple	40000	459745231	-	-
instructions	candidate-08	2	scan-no-input	20000	603881474	-	-
instructions	candidate-08	2	scan-no-input	40000	1204153911	-	-
instructions	candidate-08	2	optionvalue-three-no-input	20000	200737552	-	-
instructions	candidate-08	2	optionvalue-three-no-input	40000	395677812	-	-
instructions	candidate-08	2	replacerepeated-no-input	20000	806112909	-	-
instructions	candidate-08	2	replacerepeated-no-input	40000	1609070313	-	-
instructions	candidate-08	2	dot-three-no-input	20000	5141339697	-	-
instructions	candidate-08	2	dot-three-no-input	40000	10275905888	-	-
instructions	candidate-08	2	differences-no-input	20000	205606819	-	-
instructions	candidate-08	2	differences-no-input	40000	405539401	-	-
instructions	candidate-08	2	do-count-null	20000	13456427	-	-
instructions	candidate-08	2	do-count-null	40000	23395441	-	-
instructions	candidate-08	2	do-iterator-null	20000	26268610	-	-
instructions	candidate-08	2	do-iterator-null	40000	49011442	-	-
instructions	candidate-08	2	table-iterator-null	20000	10512250	-	-
instructions	candidate-08	2	table-iterator-null	40000	17494239	-	-
instructions	candidate-08	2	replaceall-simple	20000	231677982	-	-
instructions	candidate-08	2	replaceall-simple	40000	459746165	-	-
```

Base variability is retained rather than resampled: Differences base instructions/iteration are 9918.24750 and 9900.28660; ReplaceAll base values are 11431.25005 and 11497.30280. Several rejected candidates show a failing ReplaceAll first sample. Both samples are used as requested; no third sample or substituted baseline was used.

The binary inspection was read-only Python ELF parsing and disassembly through the locally installed LLVM library. The preserved base has a 5,324-byte eval_differences with no separate difference_expr symbol; candidate 06 has a 2,439-byte eval_differences and 2,129-byte difference_expr. The forced-inline candidate 03 has a 3,457-byte eval_differences and worsens measured instructions. These observations informed candidates 07/08, which failed; they do not establish a complete mechanism for the residual cost. [Saved disassembly](../../../.temp/brief-560/disassembly).

3. **Verification — exact captured commands, bounds, exits and verbatim result excerpts.**

All cargo commands used `env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-560/tmp` and the bounded runner. Commands and test suites ran serially. The seven outer candidate runners (02–08) also used a 300-second bound derived from candidate 01’s measured 107.77-second release build, 2.35-second allocation build and approximately 12 seconds for its four probe panels. Every individual build and probe had its own bound.

Bound provenance: saved brief-559 serial release 108.350640s → 240s; allocation build 2.321020s → 20s; full nine-label panel ≤3.023627s → 20s (the established per-process 10s bound retained); focused continuation suite 17.372344s → 80s; fmt 1.870673s → 10s; ownership discovery 19.643402s → 80s; default/no-default certificate runs 31.034173/32.242861s → 120/130s; stdlib 1046.980532s → 2200s; workspace 1011.854062s → 2200s; no-default retention 896.844157s → 1900s. Short replay/audit scripts used 10s bounds against saved serial 0.064–0.29s runs.

Runtime-bound disclosure: no standalone prior serial `cargo test -p rum-runtime` ledger entry was found. Its 100s bound used the identical runtime unit/doc-test payloads inside the saved serial workspace: 19.01s + 2.19s, together with that run’s 22.95s serial compilation (44.15s total). This is component/superset provenance, not a prior standalone command measurement. The literal same-command interpretation of the brief’s bound rule is therefore not claimed.

The fingerprint command’s first 10s bound used a warm 0.264735s predecessor and omitted the recompilation triggered by ownership inventory changes; it timed out at exit 124. The retry used 80s based on the just-measured same-crate serial single-test compilation (21.89s plus fingerprint work) and passed. Both failures and repairs are retained below. Certificate generation initially exited 101 only for stale C01 cells; both feature builds wrote replay evidence, which was installed with non-fingerprint fields checked unchanged, then accepted in the full default and no-default suites. The first fmt check exited 1 for one new call wrap; the required final fmt check exited 0.

The following ledger includes every captured cargo/build/probe-batch/replay/audit command. Result excerpts are copied from linked raw logs, not invented. Intentional compiler-negative-control diagnostics in successful full suites are retained in their raw logs; the final test result lines determine pass/fail. The 648 individual probe commands are additionally preserved in the exact process transcript above.

**1. [.temp/brief-560/base-alloc-1.txt](../../../.temp/brief-560/base-alloc-1.txt) — bound 20s; exit 0; elapsed 2.902027s.**

```text
python3 scripts/run-bounded.py --timeout 20 -- python3 .temp/brief-560/perf-panel.py alloc base 1
```

```text
alloc base sample=1 scan-no-input N=20000 exit=0
alloc base sample=1 scan-no-input N=40000 exit=0
alloc base sample=1 optionvalue-three-no-input N=20000 exit=0
alloc base sample=1 optionvalue-three-no-input N=40000 exit=0
alloc base sample=1 replacerepeated-no-input N=20000 exit=0
alloc base sample=1 replacerepeated-no-input N=40000 exit=0
alloc base sample=1 dot-three-no-input N=20000 exit=0
alloc base sample=1 dot-three-no-input N=40000 exit=0
alloc base sample=1 differences-no-input N=20000 exit=0
alloc base sample=1 differences-no-input N=40000 exit=0
alloc base sample=1 do-count-null N=20000 exit=0
alloc base sample=1 do-count-null N=40000 exit=0
alloc base sample=1 do-iterator-null N=20000 exit=0
alloc base sample=1 do-iterator-null N=40000 exit=0
alloc base sample=1 table-iterator-null N=20000 exit=0
alloc base sample=1 table-iterator-null N=40000 exit=0
alloc base sample=1 replaceall-simple N=20000 exit=0
alloc base sample=1 replaceall-simple N=40000 exit=0
```

**2. [.temp/brief-560/base-alloc-2.txt](../../../.temp/brief-560/base-alloc-2.txt) — bound 20s; exit 0; elapsed 2.906857s.**

```text
python3 scripts/run-bounded.py --timeout 20 -- python3 .temp/brief-560/perf-panel.py alloc base 2
```

```text
alloc base sample=2 scan-no-input N=20000 exit=0
alloc base sample=2 scan-no-input N=40000 exit=0
alloc base sample=2 optionvalue-three-no-input N=20000 exit=0
alloc base sample=2 optionvalue-three-no-input N=40000 exit=0
alloc base sample=2 replacerepeated-no-input N=20000 exit=0
alloc base sample=2 replacerepeated-no-input N=40000 exit=0
alloc base sample=2 dot-three-no-input N=20000 exit=0
alloc base sample=2 dot-three-no-input N=40000 exit=0
alloc base sample=2 differences-no-input N=20000 exit=0
alloc base sample=2 differences-no-input N=40000 exit=0
alloc base sample=2 do-count-null N=20000 exit=0
alloc base sample=2 do-count-null N=40000 exit=0
alloc base sample=2 do-iterator-null N=20000 exit=0
alloc base sample=2 do-iterator-null N=40000 exit=0
alloc base sample=2 table-iterator-null N=20000 exit=0
alloc base sample=2 table-iterator-null N=40000 exit=0
alloc base sample=2 replaceall-simple N=20000 exit=0
alloc base sample=2 replaceall-simple N=40000 exit=0
```

**3. [.temp/brief-560/base-instructions-1.txt](../../../.temp/brief-560/base-instructions-1.txt) — bound 20s; exit 0; elapsed 3.050790s.**

```text
python3 scripts/run-bounded.py --timeout 20 -- python3 .temp/brief-560/perf-panel.py instructions base 1
```

```text
instructions base sample=1 scan-no-input N=20000 exit=0
instructions base sample=1 scan-no-input N=40000 exit=0
instructions base sample=1 optionvalue-three-no-input N=20000 exit=0
instructions base sample=1 optionvalue-three-no-input N=40000 exit=0
instructions base sample=1 replacerepeated-no-input N=20000 exit=0
instructions base sample=1 replacerepeated-no-input N=40000 exit=0
instructions base sample=1 dot-three-no-input N=20000 exit=0
instructions base sample=1 dot-three-no-input N=40000 exit=0
instructions base sample=1 differences-no-input N=20000 exit=0
instructions base sample=1 differences-no-input N=40000 exit=0
instructions base sample=1 do-count-null N=20000 exit=0
instructions base sample=1 do-count-null N=40000 exit=0
instructions base sample=1 do-iterator-null N=20000 exit=0
instructions base sample=1 do-iterator-null N=40000 exit=0
instructions base sample=1 table-iterator-null N=20000 exit=0
instructions base sample=1 table-iterator-null N=40000 exit=0
instructions base sample=1 replaceall-simple N=20000 exit=0
instructions base sample=1 replaceall-simple N=40000 exit=0
```

**4. [.temp/brief-560/base-instructions-2.txt](../../../.temp/brief-560/base-instructions-2.txt) — bound 20s; exit 0; elapsed 2.847494s.**

```text
python3 scripts/run-bounded.py --timeout 20 -- python3 .temp/brief-560/perf-panel.py instructions base 2
```

```text
instructions base sample=2 scan-no-input N=20000 exit=0
instructions base sample=2 scan-no-input N=40000 exit=0
instructions base sample=2 optionvalue-three-no-input N=20000 exit=0
instructions base sample=2 optionvalue-three-no-input N=40000 exit=0
instructions base sample=2 replacerepeated-no-input N=20000 exit=0
instructions base sample=2 replacerepeated-no-input N=40000 exit=0
instructions base sample=2 dot-three-no-input N=20000 exit=0
instructions base sample=2 dot-three-no-input N=40000 exit=0
instructions base sample=2 differences-no-input N=20000 exit=0
instructions base sample=2 differences-no-input N=40000 exit=0
instructions base sample=2 do-count-null N=20000 exit=0
instructions base sample=2 do-count-null N=40000 exit=0
instructions base sample=2 do-iterator-null N=20000 exit=0
instructions base sample=2 do-iterator-null N=40000 exit=0
instructions base sample=2 table-iterator-null N=20000 exit=0
instructions base sample=2 table-iterator-null N=40000 exit=0
instructions base sample=2 replaceall-simple N=20000 exit=0
instructions base sample=2 replaceall-simple N=40000 exit=0
```

**5. [.temp/brief-560/candidate-01/loops.txt](../../../.temp/brief-560/candidate-01/loops.txt) — bound 80s; exit 0; elapsed 11.436547s.**

```text
python3 scripts/run-bounded.py --timeout 80 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-560/tmp cargo test -p rum-stdlib --test loop_continuations
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 11.18s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
```

**6. [.temp/brief-560/candidate-01/default-build.txt](../../../.temp/brief-560/candidate-01/default-build.txt) — bound 240s; exit 0; elapsed 107.768639s.**

```text
python3 scripts/run-bounded.py --timeout 240 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-560/tmp cargo build --release --target-dir .temp/brief-560/candidate-01/target -p rum-cli
```

```text
    Finished `release` profile [optimized] target(s) in 1m 47s
```

**7. [.temp/brief-560/candidate-01/alloc-build.txt](../../../.temp/brief-560/candidate-01/alloc-build.txt) — bound 20s; exit 0; elapsed 2.352099s.**

```text
python3 scripts/run-bounded.py --timeout 20 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-560/tmp cargo build --release --target-dir .temp/brief-560/candidate-01/target -p rum-cli --features alloc-count
```

```text
    Finished `release` profile [optimized] target(s) in 2.25s
```

**8. [.temp/brief-560/candidate-01/alloc-1.txt](../../../.temp/brief-560/candidate-01/alloc-1.txt) — bound 20s; exit 0; elapsed 2.906466s.**

```text
python3 scripts/run-bounded.py --timeout 20 -- python3 .temp/brief-560/perf-panel.py alloc candidate-01 1
```

```text
alloc candidate-01 sample=1 scan-no-input N=20000 exit=0
alloc candidate-01 sample=1 scan-no-input N=40000 exit=0
alloc candidate-01 sample=1 optionvalue-three-no-input N=20000 exit=0
alloc candidate-01 sample=1 optionvalue-three-no-input N=40000 exit=0
alloc candidate-01 sample=1 replacerepeated-no-input N=20000 exit=0
alloc candidate-01 sample=1 replacerepeated-no-input N=40000 exit=0
alloc candidate-01 sample=1 dot-three-no-input N=20000 exit=0
alloc candidate-01 sample=1 dot-three-no-input N=40000 exit=0
alloc candidate-01 sample=1 differences-no-input N=20000 exit=0
alloc candidate-01 sample=1 differences-no-input N=40000 exit=0
alloc candidate-01 sample=1 do-count-null N=20000 exit=0
alloc candidate-01 sample=1 do-count-null N=40000 exit=0
alloc candidate-01 sample=1 do-iterator-null N=20000 exit=0
alloc candidate-01 sample=1 do-iterator-null N=40000 exit=0
alloc candidate-01 sample=1 table-iterator-null N=20000 exit=0
alloc candidate-01 sample=1 table-iterator-null N=40000 exit=0
alloc candidate-01 sample=1 replaceall-simple N=20000 exit=0
alloc candidate-01 sample=1 replaceall-simple N=40000 exit=0
```

**9. [.temp/brief-560/candidate-01/alloc-2.txt](../../../.temp/brief-560/candidate-01/alloc-2.txt) — bound 20s; exit 0; elapsed 2.905443s.**

```text
python3 scripts/run-bounded.py --timeout 20 -- python3 .temp/brief-560/perf-panel.py alloc candidate-01 2
```

```text
alloc candidate-01 sample=2 scan-no-input N=20000 exit=0
alloc candidate-01 sample=2 scan-no-input N=40000 exit=0
alloc candidate-01 sample=2 optionvalue-three-no-input N=20000 exit=0
alloc candidate-01 sample=2 optionvalue-three-no-input N=40000 exit=0
alloc candidate-01 sample=2 replacerepeated-no-input N=20000 exit=0
alloc candidate-01 sample=2 replacerepeated-no-input N=40000 exit=0
alloc candidate-01 sample=2 dot-three-no-input N=20000 exit=0
alloc candidate-01 sample=2 dot-three-no-input N=40000 exit=0
alloc candidate-01 sample=2 differences-no-input N=20000 exit=0
alloc candidate-01 sample=2 differences-no-input N=40000 exit=0
alloc candidate-01 sample=2 do-count-null N=20000 exit=0
alloc candidate-01 sample=2 do-count-null N=40000 exit=0
alloc candidate-01 sample=2 do-iterator-null N=20000 exit=0
alloc candidate-01 sample=2 do-iterator-null N=40000 exit=0
alloc candidate-01 sample=2 table-iterator-null N=20000 exit=0
alloc candidate-01 sample=2 table-iterator-null N=40000 exit=0
alloc candidate-01 sample=2 replaceall-simple N=20000 exit=0
alloc candidate-01 sample=2 replaceall-simple N=40000 exit=0
```

**10. [.temp/brief-560/candidate-01/instructions-1.txt](../../../.temp/brief-560/candidate-01/instructions-1.txt) — bound 20s; exit 0; elapsed 2.952052s.**

```text
python3 scripts/run-bounded.py --timeout 20 -- python3 .temp/brief-560/perf-panel.py instructions candidate-01 1
```

```text
instructions candidate-01 sample=1 scan-no-input N=20000 exit=0
instructions candidate-01 sample=1 scan-no-input N=40000 exit=0
instructions candidate-01 sample=1 optionvalue-three-no-input N=20000 exit=0
instructions candidate-01 sample=1 optionvalue-three-no-input N=40000 exit=0
instructions candidate-01 sample=1 replacerepeated-no-input N=20000 exit=0
instructions candidate-01 sample=1 replacerepeated-no-input N=40000 exit=0
instructions candidate-01 sample=1 dot-three-no-input N=20000 exit=0
instructions candidate-01 sample=1 dot-three-no-input N=40000 exit=0
instructions candidate-01 sample=1 differences-no-input N=20000 exit=0
instructions candidate-01 sample=1 differences-no-input N=40000 exit=0
instructions candidate-01 sample=1 do-count-null N=20000 exit=0
instructions candidate-01 sample=1 do-count-null N=40000 exit=0
instructions candidate-01 sample=1 do-iterator-null N=20000 exit=0
instructions candidate-01 sample=1 do-iterator-null N=40000 exit=0
instructions candidate-01 sample=1 table-iterator-null N=20000 exit=0
instructions candidate-01 sample=1 table-iterator-null N=40000 exit=0
instructions candidate-01 sample=1 replaceall-simple N=20000 exit=0
instructions candidate-01 sample=1 replaceall-simple N=40000 exit=0
```

**11. [.temp/brief-560/candidate-01/instructions-2.txt](../../../.temp/brief-560/candidate-01/instructions-2.txt) — bound 20s; exit 0; elapsed 3.100223s.**

```text
python3 scripts/run-bounded.py --timeout 20 -- python3 .temp/brief-560/perf-panel.py instructions candidate-01 2
```

```text
instructions candidate-01 sample=2 scan-no-input N=20000 exit=0
instructions candidate-01 sample=2 scan-no-input N=40000 exit=0
instructions candidate-01 sample=2 optionvalue-three-no-input N=20000 exit=0
instructions candidate-01 sample=2 optionvalue-three-no-input N=40000 exit=0
instructions candidate-01 sample=2 replacerepeated-no-input N=20000 exit=0
instructions candidate-01 sample=2 replacerepeated-no-input N=40000 exit=0
instructions candidate-01 sample=2 dot-three-no-input N=20000 exit=0
instructions candidate-01 sample=2 dot-three-no-input N=40000 exit=0
instructions candidate-01 sample=2 differences-no-input N=20000 exit=0
instructions candidate-01 sample=2 differences-no-input N=40000 exit=0
instructions candidate-01 sample=2 do-count-null N=20000 exit=0
instructions candidate-01 sample=2 do-count-null N=40000 exit=0
instructions candidate-01 sample=2 do-iterator-null N=20000 exit=0
instructions candidate-01 sample=2 do-iterator-null N=40000 exit=0
instructions candidate-01 sample=2 table-iterator-null N=20000 exit=0
instructions candidate-01 sample=2 table-iterator-null N=40000 exit=0
instructions candidate-01 sample=2 replaceall-simple N=20000 exit=0
instructions candidate-01 sample=2 replaceall-simple N=40000 exit=0
```

**12. [.temp/brief-560/candidate-02/default-build.txt](../../../.temp/brief-560/candidate-02/default-build.txt) — bound 240s; exit 0; elapsed 106.509094s.**

```text
python3 scripts/run-bounded.py --timeout 240 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-560/tmp cargo build --release --target-dir .temp/brief-560/candidate-02/target -p rum-cli
```

```text
    Finished `release` profile [optimized] target(s) in 1m 46s
```

**13. [.temp/brief-560/candidate-02/alloc-build.txt](../../../.temp/brief-560/candidate-02/alloc-build.txt) — bound 20s; exit 0; elapsed 2.301837s.**

```text
python3 scripts/run-bounded.py --timeout 20 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-560/tmp cargo build --release --target-dir .temp/brief-560/candidate-02/target -p rum-cli --features alloc-count
```

```text
    Finished `release` profile [optimized] target(s) in 2.24s
```

**14. [.temp/brief-560/candidate-02/alloc-1.txt](../../../.temp/brief-560/candidate-02/alloc-1.txt) — bound 20s; exit 0; elapsed 2.907398s.**

```text
python3 scripts/run-bounded.py --timeout 20 -- python3 .temp/brief-560/perf-panel.py alloc candidate-02 1
```

```text
alloc candidate-02 sample=1 scan-no-input N=20000 exit=0
alloc candidate-02 sample=1 scan-no-input N=40000 exit=0
alloc candidate-02 sample=1 optionvalue-three-no-input N=20000 exit=0
alloc candidate-02 sample=1 optionvalue-three-no-input N=40000 exit=0
alloc candidate-02 sample=1 replacerepeated-no-input N=20000 exit=0
alloc candidate-02 sample=1 replacerepeated-no-input N=40000 exit=0
alloc candidate-02 sample=1 dot-three-no-input N=20000 exit=0
alloc candidate-02 sample=1 dot-three-no-input N=40000 exit=0
alloc candidate-02 sample=1 differences-no-input N=20000 exit=0
alloc candidate-02 sample=1 differences-no-input N=40000 exit=0
alloc candidate-02 sample=1 do-count-null N=20000 exit=0
alloc candidate-02 sample=1 do-count-null N=40000 exit=0
alloc candidate-02 sample=1 do-iterator-null N=20000 exit=0
alloc candidate-02 sample=1 do-iterator-null N=40000 exit=0
alloc candidate-02 sample=1 table-iterator-null N=20000 exit=0
alloc candidate-02 sample=1 table-iterator-null N=40000 exit=0
alloc candidate-02 sample=1 replaceall-simple N=20000 exit=0
alloc candidate-02 sample=1 replaceall-simple N=40000 exit=0
```

**15. [.temp/brief-560/candidate-02/alloc-2.txt](../../../.temp/brief-560/candidate-02/alloc-2.txt) — bound 20s; exit 0; elapsed 2.850615s.**

```text
python3 scripts/run-bounded.py --timeout 20 -- python3 .temp/brief-560/perf-panel.py alloc candidate-02 2
```

```text
alloc candidate-02 sample=2 scan-no-input N=20000 exit=0
alloc candidate-02 sample=2 scan-no-input N=40000 exit=0
alloc candidate-02 sample=2 optionvalue-three-no-input N=20000 exit=0
alloc candidate-02 sample=2 optionvalue-three-no-input N=40000 exit=0
alloc candidate-02 sample=2 replacerepeated-no-input N=20000 exit=0
alloc candidate-02 sample=2 replacerepeated-no-input N=40000 exit=0
alloc candidate-02 sample=2 dot-three-no-input N=20000 exit=0
alloc candidate-02 sample=2 dot-three-no-input N=40000 exit=0
alloc candidate-02 sample=2 differences-no-input N=20000 exit=0
alloc candidate-02 sample=2 differences-no-input N=40000 exit=0
alloc candidate-02 sample=2 do-count-null N=20000 exit=0
alloc candidate-02 sample=2 do-count-null N=40000 exit=0
alloc candidate-02 sample=2 do-iterator-null N=20000 exit=0
alloc candidate-02 sample=2 do-iterator-null N=40000 exit=0
alloc candidate-02 sample=2 table-iterator-null N=20000 exit=0
alloc candidate-02 sample=2 table-iterator-null N=40000 exit=0
alloc candidate-02 sample=2 replaceall-simple N=20000 exit=0
alloc candidate-02 sample=2 replaceall-simple N=40000 exit=0
```

**16. [.temp/brief-560/candidate-02/instructions-1.txt](../../../.temp/brief-560/candidate-02/instructions-1.txt) — bound 20s; exit 0; elapsed 2.898113s.**

```text
python3 scripts/run-bounded.py --timeout 20 -- python3 .temp/brief-560/perf-panel.py instructions candidate-02 1
```

```text
instructions candidate-02 sample=1 scan-no-input N=20000 exit=0
instructions candidate-02 sample=1 scan-no-input N=40000 exit=0
instructions candidate-02 sample=1 optionvalue-three-no-input N=20000 exit=0
instructions candidate-02 sample=1 optionvalue-three-no-input N=40000 exit=0
instructions candidate-02 sample=1 replacerepeated-no-input N=20000 exit=0
instructions candidate-02 sample=1 replacerepeated-no-input N=40000 exit=0
instructions candidate-02 sample=1 dot-three-no-input N=20000 exit=0
instructions candidate-02 sample=1 dot-three-no-input N=40000 exit=0
instructions candidate-02 sample=1 differences-no-input N=20000 exit=0
instructions candidate-02 sample=1 differences-no-input N=40000 exit=0
instructions candidate-02 sample=1 do-count-null N=20000 exit=0
instructions candidate-02 sample=1 do-count-null N=40000 exit=0
instructions candidate-02 sample=1 do-iterator-null N=20000 exit=0
instructions candidate-02 sample=1 do-iterator-null N=40000 exit=0
instructions candidate-02 sample=1 table-iterator-null N=20000 exit=0
instructions candidate-02 sample=1 table-iterator-null N=40000 exit=0
instructions candidate-02 sample=1 replaceall-simple N=20000 exit=0
instructions candidate-02 sample=1 replaceall-simple N=40000 exit=0
```

**17. [.temp/brief-560/candidate-02/instructions-2.txt](../../../.temp/brief-560/candidate-02/instructions-2.txt) — bound 20s; exit 0; elapsed 2.908837s.**

```text
python3 scripts/run-bounded.py --timeout 20 -- python3 .temp/brief-560/perf-panel.py instructions candidate-02 2
```

```text
instructions candidate-02 sample=2 scan-no-input N=20000 exit=0
instructions candidate-02 sample=2 scan-no-input N=40000 exit=0
instructions candidate-02 sample=2 optionvalue-three-no-input N=20000 exit=0
instructions candidate-02 sample=2 optionvalue-three-no-input N=40000 exit=0
instructions candidate-02 sample=2 replacerepeated-no-input N=20000 exit=0
instructions candidate-02 sample=2 replacerepeated-no-input N=40000 exit=0
instructions candidate-02 sample=2 dot-three-no-input N=20000 exit=0
instructions candidate-02 sample=2 dot-three-no-input N=40000 exit=0
instructions candidate-02 sample=2 differences-no-input N=20000 exit=0
instructions candidate-02 sample=2 differences-no-input N=40000 exit=0
instructions candidate-02 sample=2 do-count-null N=20000 exit=0
instructions candidate-02 sample=2 do-count-null N=40000 exit=0
instructions candidate-02 sample=2 do-iterator-null N=20000 exit=0
instructions candidate-02 sample=2 do-iterator-null N=40000 exit=0
instructions candidate-02 sample=2 table-iterator-null N=20000 exit=0
instructions candidate-02 sample=2 table-iterator-null N=40000 exit=0
instructions candidate-02 sample=2 replaceall-simple N=20000 exit=0
instructions candidate-02 sample=2 replaceall-simple N=40000 exit=0
```

**18. [.temp/brief-560/candidate-03/default-build.txt](../../../.temp/brief-560/candidate-03/default-build.txt) — bound 240s; exit 0; elapsed 106.526934s.**

```text
python3 scripts/run-bounded.py --timeout 240 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-560/tmp cargo build --release --target-dir .temp/brief-560/candidate-03/target -p rum-cli
```

```text
    Finished `release` profile [optimized] target(s) in 1m 46s
```

**19. [.temp/brief-560/candidate-03/alloc-build.txt](../../../.temp/brief-560/candidate-03/alloc-build.txt) — bound 20s; exit 0; elapsed 2.300922s.**

```text
python3 scripts/run-bounded.py --timeout 20 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-560/tmp cargo build --release --target-dir .temp/brief-560/candidate-03/target -p rum-cli --features alloc-count
```

```text
    Finished `release` profile [optimized] target(s) in 2.22s
```

**20. [.temp/brief-560/candidate-03/alloc-1.txt](../../../.temp/brief-560/candidate-03/alloc-1.txt) — bound 20s; exit 0; elapsed 2.903910s.**

```text
python3 scripts/run-bounded.py --timeout 20 -- python3 .temp/brief-560/perf-panel.py alloc candidate-03 1
```

```text
alloc candidate-03 sample=1 scan-no-input N=20000 exit=0
alloc candidate-03 sample=1 scan-no-input N=40000 exit=0
alloc candidate-03 sample=1 optionvalue-three-no-input N=20000 exit=0
alloc candidate-03 sample=1 optionvalue-three-no-input N=40000 exit=0
alloc candidate-03 sample=1 replacerepeated-no-input N=20000 exit=0
alloc candidate-03 sample=1 replacerepeated-no-input N=40000 exit=0
alloc candidate-03 sample=1 dot-three-no-input N=20000 exit=0
alloc candidate-03 sample=1 dot-three-no-input N=40000 exit=0
alloc candidate-03 sample=1 differences-no-input N=20000 exit=0
alloc candidate-03 sample=1 differences-no-input N=40000 exit=0
alloc candidate-03 sample=1 do-count-null N=20000 exit=0
alloc candidate-03 sample=1 do-count-null N=40000 exit=0
alloc candidate-03 sample=1 do-iterator-null N=20000 exit=0
alloc candidate-03 sample=1 do-iterator-null N=40000 exit=0
alloc candidate-03 sample=1 table-iterator-null N=20000 exit=0
alloc candidate-03 sample=1 table-iterator-null N=40000 exit=0
alloc candidate-03 sample=1 replaceall-simple N=20000 exit=0
alloc candidate-03 sample=1 replaceall-simple N=40000 exit=0
```

**21. [.temp/brief-560/candidate-03/alloc-2.txt](../../../.temp/brief-560/candidate-03/alloc-2.txt) — bound 20s; exit 0; elapsed 2.902127s.**

```text
python3 scripts/run-bounded.py --timeout 20 -- python3 .temp/brief-560/perf-panel.py alloc candidate-03 2
```

```text
alloc candidate-03 sample=2 scan-no-input N=20000 exit=0
alloc candidate-03 sample=2 scan-no-input N=40000 exit=0
alloc candidate-03 sample=2 optionvalue-three-no-input N=20000 exit=0
alloc candidate-03 sample=2 optionvalue-three-no-input N=40000 exit=0
alloc candidate-03 sample=2 replacerepeated-no-input N=20000 exit=0
alloc candidate-03 sample=2 replacerepeated-no-input N=40000 exit=0
alloc candidate-03 sample=2 dot-three-no-input N=20000 exit=0
alloc candidate-03 sample=2 dot-three-no-input N=40000 exit=0
alloc candidate-03 sample=2 differences-no-input N=20000 exit=0
alloc candidate-03 sample=2 differences-no-input N=40000 exit=0
alloc candidate-03 sample=2 do-count-null N=20000 exit=0
alloc candidate-03 sample=2 do-count-null N=40000 exit=0
alloc candidate-03 sample=2 do-iterator-null N=20000 exit=0
alloc candidate-03 sample=2 do-iterator-null N=40000 exit=0
alloc candidate-03 sample=2 table-iterator-null N=20000 exit=0
alloc candidate-03 sample=2 table-iterator-null N=40000 exit=0
alloc candidate-03 sample=2 replaceall-simple N=20000 exit=0
alloc candidate-03 sample=2 replaceall-simple N=40000 exit=0
```

**22. [.temp/brief-560/candidate-03/instructions-1.txt](../../../.temp/brief-560/candidate-03/instructions-1.txt) — bound 20s; exit 0; elapsed 2.908473s.**

```text
python3 scripts/run-bounded.py --timeout 20 -- python3 .temp/brief-560/perf-panel.py instructions candidate-03 1
```

```text
instructions candidate-03 sample=1 scan-no-input N=20000 exit=0
instructions candidate-03 sample=1 scan-no-input N=40000 exit=0
instructions candidate-03 sample=1 optionvalue-three-no-input N=20000 exit=0
instructions candidate-03 sample=1 optionvalue-three-no-input N=40000 exit=0
instructions candidate-03 sample=1 replacerepeated-no-input N=20000 exit=0
instructions candidate-03 sample=1 replacerepeated-no-input N=40000 exit=0
instructions candidate-03 sample=1 dot-three-no-input N=20000 exit=0
instructions candidate-03 sample=1 dot-three-no-input N=40000 exit=0
instructions candidate-03 sample=1 differences-no-input N=20000 exit=0
instructions candidate-03 sample=1 differences-no-input N=40000 exit=0
instructions candidate-03 sample=1 do-count-null N=20000 exit=0
instructions candidate-03 sample=1 do-count-null N=40000 exit=0
instructions candidate-03 sample=1 do-iterator-null N=20000 exit=0
instructions candidate-03 sample=1 do-iterator-null N=40000 exit=0
instructions candidate-03 sample=1 table-iterator-null N=20000 exit=0
instructions candidate-03 sample=1 table-iterator-null N=40000 exit=0
instructions candidate-03 sample=1 replaceall-simple N=20000 exit=0
instructions candidate-03 sample=1 replaceall-simple N=40000 exit=0
```

**23. [.temp/brief-560/candidate-03/instructions-2.txt](../../../.temp/brief-560/candidate-03/instructions-2.txt) — bound 20s; exit 0; elapsed 2.852635s.**

```text
python3 scripts/run-bounded.py --timeout 20 -- python3 .temp/brief-560/perf-panel.py instructions candidate-03 2
```

```text
instructions candidate-03 sample=2 scan-no-input N=20000 exit=0
instructions candidate-03 sample=2 scan-no-input N=40000 exit=0
instructions candidate-03 sample=2 optionvalue-three-no-input N=20000 exit=0
instructions candidate-03 sample=2 optionvalue-three-no-input N=40000 exit=0
instructions candidate-03 sample=2 replacerepeated-no-input N=20000 exit=0
instructions candidate-03 sample=2 replacerepeated-no-input N=40000 exit=0
instructions candidate-03 sample=2 dot-three-no-input N=20000 exit=0
instructions candidate-03 sample=2 dot-three-no-input N=40000 exit=0
instructions candidate-03 sample=2 differences-no-input N=20000 exit=0
instructions candidate-03 sample=2 differences-no-input N=40000 exit=0
instructions candidate-03 sample=2 do-count-null N=20000 exit=0
instructions candidate-03 sample=2 do-count-null N=40000 exit=0
instructions candidate-03 sample=2 do-iterator-null N=20000 exit=0
instructions candidate-03 sample=2 do-iterator-null N=40000 exit=0
instructions candidate-03 sample=2 table-iterator-null N=20000 exit=0
instructions candidate-03 sample=2 table-iterator-null N=40000 exit=0
instructions candidate-03 sample=2 replaceall-simple N=20000 exit=0
instructions candidate-03 sample=2 replaceall-simple N=40000 exit=0
```

**24. [.temp/brief-560/candidate-04/loops.txt](../../../.temp/brief-560/candidate-04/loops.txt) — bound 80s; exit 0; elapsed 15.106559s.**

```text
python3 scripts/run-bounded.py --timeout 80 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-560/tmp cargo test -p rum-stdlib --test loop_continuations
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 14.91s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
```

**25. [.temp/brief-560/candidate-04/default-build.txt](../../../.temp/brief-560/candidate-04/default-build.txt) — bound 240s; exit 0; elapsed 108.080313s.**

```text
python3 scripts/run-bounded.py --timeout 240 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-560/tmp cargo build --release --target-dir .temp/brief-560/candidate-04/target -p rum-cli
```

```text
    Finished `release` profile [optimized] target(s) in 1m 47s
```

**26. [.temp/brief-560/candidate-04/alloc-build.txt](../../../.temp/brief-560/candidate-04/alloc-build.txt) — bound 20s; exit 0; elapsed 2.355072s.**

```text
python3 scripts/run-bounded.py --timeout 20 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-560/tmp cargo build --release --target-dir .temp/brief-560/candidate-04/target -p rum-cli --features alloc-count
```

```text
    Finished `release` profile [optimized] target(s) in 2.24s
```

**27. [.temp/brief-560/candidate-04/alloc-1.txt](../../../.temp/brief-560/candidate-04/alloc-1.txt) — bound 20s; exit 0; elapsed 2.851117s.**

```text
python3 scripts/run-bounded.py --timeout 20 -- python3 .temp/brief-560/perf-panel.py alloc candidate-04 1
```

```text
alloc candidate-04 sample=1 scan-no-input N=20000 exit=0
alloc candidate-04 sample=1 scan-no-input N=40000 exit=0
alloc candidate-04 sample=1 optionvalue-three-no-input N=20000 exit=0
alloc candidate-04 sample=1 optionvalue-three-no-input N=40000 exit=0
alloc candidate-04 sample=1 replacerepeated-no-input N=20000 exit=0
alloc candidate-04 sample=1 replacerepeated-no-input N=40000 exit=0
alloc candidate-04 sample=1 dot-three-no-input N=20000 exit=0
alloc candidate-04 sample=1 dot-three-no-input N=40000 exit=0
alloc candidate-04 sample=1 differences-no-input N=20000 exit=0
alloc candidate-04 sample=1 differences-no-input N=40000 exit=0
alloc candidate-04 sample=1 do-count-null N=20000 exit=0
alloc candidate-04 sample=1 do-count-null N=40000 exit=0
alloc candidate-04 sample=1 do-iterator-null N=20000 exit=0
alloc candidate-04 sample=1 do-iterator-null N=40000 exit=0
alloc candidate-04 sample=1 table-iterator-null N=20000 exit=0
alloc candidate-04 sample=1 table-iterator-null N=40000 exit=0
alloc candidate-04 sample=1 replaceall-simple N=20000 exit=0
alloc candidate-04 sample=1 replaceall-simple N=40000 exit=0
```

**28. [.temp/brief-560/candidate-04/alloc-2.txt](../../../.temp/brief-560/candidate-04/alloc-2.txt) — bound 20s; exit 0; elapsed 2.853362s.**

```text
python3 scripts/run-bounded.py --timeout 20 -- python3 .temp/brief-560/perf-panel.py alloc candidate-04 2
```

```text
alloc candidate-04 sample=2 scan-no-input N=20000 exit=0
alloc candidate-04 sample=2 scan-no-input N=40000 exit=0
alloc candidate-04 sample=2 optionvalue-three-no-input N=20000 exit=0
alloc candidate-04 sample=2 optionvalue-three-no-input N=40000 exit=0
alloc candidate-04 sample=2 replacerepeated-no-input N=20000 exit=0
alloc candidate-04 sample=2 replacerepeated-no-input N=40000 exit=0
alloc candidate-04 sample=2 dot-three-no-input N=20000 exit=0
alloc candidate-04 sample=2 dot-three-no-input N=40000 exit=0
alloc candidate-04 sample=2 differences-no-input N=20000 exit=0
alloc candidate-04 sample=2 differences-no-input N=40000 exit=0
alloc candidate-04 sample=2 do-count-null N=20000 exit=0
alloc candidate-04 sample=2 do-count-null N=40000 exit=0
alloc candidate-04 sample=2 do-iterator-null N=20000 exit=0
alloc candidate-04 sample=2 do-iterator-null N=40000 exit=0
alloc candidate-04 sample=2 table-iterator-null N=20000 exit=0
alloc candidate-04 sample=2 table-iterator-null N=40000 exit=0
alloc candidate-04 sample=2 replaceall-simple N=20000 exit=0
alloc candidate-04 sample=2 replaceall-simple N=40000 exit=0
```

**29. [.temp/brief-560/candidate-04/instructions-1.txt](../../../.temp/brief-560/candidate-04/instructions-1.txt) — bound 20s; exit 0; elapsed 2.900686s.**

```text
python3 scripts/run-bounded.py --timeout 20 -- python3 .temp/brief-560/perf-panel.py instructions candidate-04 1
```

```text
instructions candidate-04 sample=1 scan-no-input N=20000 exit=0
instructions candidate-04 sample=1 scan-no-input N=40000 exit=0
instructions candidate-04 sample=1 optionvalue-three-no-input N=20000 exit=0
instructions candidate-04 sample=1 optionvalue-three-no-input N=40000 exit=0
instructions candidate-04 sample=1 replacerepeated-no-input N=20000 exit=0
instructions candidate-04 sample=1 replacerepeated-no-input N=40000 exit=0
instructions candidate-04 sample=1 dot-three-no-input N=20000 exit=0
instructions candidate-04 sample=1 dot-three-no-input N=40000 exit=0
instructions candidate-04 sample=1 differences-no-input N=20000 exit=0
instructions candidate-04 sample=1 differences-no-input N=40000 exit=0
instructions candidate-04 sample=1 do-count-null N=20000 exit=0
instructions candidate-04 sample=1 do-count-null N=40000 exit=0
instructions candidate-04 sample=1 do-iterator-null N=20000 exit=0
instructions candidate-04 sample=1 do-iterator-null N=40000 exit=0
instructions candidate-04 sample=1 table-iterator-null N=20000 exit=0
instructions candidate-04 sample=1 table-iterator-null N=40000 exit=0
instructions candidate-04 sample=1 replaceall-simple N=20000 exit=0
instructions candidate-04 sample=1 replaceall-simple N=40000 exit=0
```

**30. [.temp/brief-560/candidate-04/instructions-2.txt](../../../.temp/brief-560/candidate-04/instructions-2.txt) — bound 20s; exit 0; elapsed 2.850068s.**

```text
python3 scripts/run-bounded.py --timeout 20 -- python3 .temp/brief-560/perf-panel.py instructions candidate-04 2
```

```text
instructions candidate-04 sample=2 scan-no-input N=20000 exit=0
instructions candidate-04 sample=2 scan-no-input N=40000 exit=0
instructions candidate-04 sample=2 optionvalue-three-no-input N=20000 exit=0
instructions candidate-04 sample=2 optionvalue-three-no-input N=40000 exit=0
instructions candidate-04 sample=2 replacerepeated-no-input N=20000 exit=0
instructions candidate-04 sample=2 replacerepeated-no-input N=40000 exit=0
instructions candidate-04 sample=2 dot-three-no-input N=20000 exit=0
instructions candidate-04 sample=2 dot-three-no-input N=40000 exit=0
instructions candidate-04 sample=2 differences-no-input N=20000 exit=0
instructions candidate-04 sample=2 differences-no-input N=40000 exit=0
instructions candidate-04 sample=2 do-count-null N=20000 exit=0
instructions candidate-04 sample=2 do-count-null N=40000 exit=0
instructions candidate-04 sample=2 do-iterator-null N=20000 exit=0
instructions candidate-04 sample=2 do-iterator-null N=40000 exit=0
instructions candidate-04 sample=2 table-iterator-null N=20000 exit=0
instructions candidate-04 sample=2 table-iterator-null N=40000 exit=0
instructions candidate-04 sample=2 replaceall-simple N=20000 exit=0
instructions candidate-04 sample=2 replaceall-simple N=40000 exit=0
```

**31. [.temp/brief-560/candidate-05/loops.txt](../../../.temp/brief-560/candidate-05/loops.txt) — bound 80s; exit 0; elapsed 13.896100s.**

```text
python3 scripts/run-bounded.py --timeout 80 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-560/tmp cargo test -p rum-stdlib --test loop_continuations
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 13.68s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
```

**32. [.temp/brief-560/candidate-05/default-build.txt](../../../.temp/brief-560/candidate-05/default-build.txt) — bound 240s; exit 0; elapsed 106.921862s.**

```text
python3 scripts/run-bounded.py --timeout 240 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-560/tmp cargo build --release --target-dir .temp/brief-560/candidate-05/target -p rum-cli
```

```text
    Finished `release` profile [optimized] target(s) in 1m 46s
```

**33. [.temp/brief-560/candidate-05/alloc-build.txt](../../../.temp/brief-560/candidate-05/alloc-build.txt) — bound 20s; exit 0; elapsed 2.303773s.**

```text
python3 scripts/run-bounded.py --timeout 20 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-560/tmp cargo build --release --target-dir .temp/brief-560/candidate-05/target -p rum-cli --features alloc-count
```

```text
    Finished `release` profile [optimized] target(s) in 2.21s
```

**34. [.temp/brief-560/candidate-05/alloc-1.txt](../../../.temp/brief-560/candidate-05/alloc-1.txt) — bound 20s; exit 0; elapsed 2.954888s.**

```text
python3 scripts/run-bounded.py --timeout 20 -- python3 .temp/brief-560/perf-panel.py alloc candidate-05 1
```

```text
alloc candidate-05 sample=1 scan-no-input N=20000 exit=0
alloc candidate-05 sample=1 scan-no-input N=40000 exit=0
alloc candidate-05 sample=1 optionvalue-three-no-input N=20000 exit=0
alloc candidate-05 sample=1 optionvalue-three-no-input N=40000 exit=0
alloc candidate-05 sample=1 replacerepeated-no-input N=20000 exit=0
alloc candidate-05 sample=1 replacerepeated-no-input N=40000 exit=0
alloc candidate-05 sample=1 dot-three-no-input N=20000 exit=0
alloc candidate-05 sample=1 dot-three-no-input N=40000 exit=0
alloc candidate-05 sample=1 differences-no-input N=20000 exit=0
alloc candidate-05 sample=1 differences-no-input N=40000 exit=0
alloc candidate-05 sample=1 do-count-null N=20000 exit=0
alloc candidate-05 sample=1 do-count-null N=40000 exit=0
alloc candidate-05 sample=1 do-iterator-null N=20000 exit=0
alloc candidate-05 sample=1 do-iterator-null N=40000 exit=0
alloc candidate-05 sample=1 table-iterator-null N=20000 exit=0
alloc candidate-05 sample=1 table-iterator-null N=40000 exit=0
alloc candidate-05 sample=1 replaceall-simple N=20000 exit=0
alloc candidate-05 sample=1 replaceall-simple N=40000 exit=0
```

**35. [.temp/brief-560/candidate-05/alloc-2.txt](../../../.temp/brief-560/candidate-05/alloc-2.txt) — bound 20s; exit 0; elapsed 2.903613s.**

```text
python3 scripts/run-bounded.py --timeout 20 -- python3 .temp/brief-560/perf-panel.py alloc candidate-05 2
```

```text
alloc candidate-05 sample=2 scan-no-input N=20000 exit=0
alloc candidate-05 sample=2 scan-no-input N=40000 exit=0
alloc candidate-05 sample=2 optionvalue-three-no-input N=20000 exit=0
alloc candidate-05 sample=2 optionvalue-three-no-input N=40000 exit=0
alloc candidate-05 sample=2 replacerepeated-no-input N=20000 exit=0
alloc candidate-05 sample=2 replacerepeated-no-input N=40000 exit=0
alloc candidate-05 sample=2 dot-three-no-input N=20000 exit=0
alloc candidate-05 sample=2 dot-three-no-input N=40000 exit=0
alloc candidate-05 sample=2 differences-no-input N=20000 exit=0
alloc candidate-05 sample=2 differences-no-input N=40000 exit=0
alloc candidate-05 sample=2 do-count-null N=20000 exit=0
alloc candidate-05 sample=2 do-count-null N=40000 exit=0
alloc candidate-05 sample=2 do-iterator-null N=20000 exit=0
alloc candidate-05 sample=2 do-iterator-null N=40000 exit=0
alloc candidate-05 sample=2 table-iterator-null N=20000 exit=0
alloc candidate-05 sample=2 table-iterator-null N=40000 exit=0
alloc candidate-05 sample=2 replaceall-simple N=20000 exit=0
alloc candidate-05 sample=2 replaceall-simple N=40000 exit=0
```

**36. [.temp/brief-560/candidate-05/instructions-1.txt](../../../.temp/brief-560/candidate-05/instructions-1.txt) — bound 20s; exit 0; elapsed 2.953460s.**

```text
python3 scripts/run-bounded.py --timeout 20 -- python3 .temp/brief-560/perf-panel.py instructions candidate-05 1
```

```text
instructions candidate-05 sample=1 scan-no-input N=20000 exit=0
instructions candidate-05 sample=1 scan-no-input N=40000 exit=0
instructions candidate-05 sample=1 optionvalue-three-no-input N=20000 exit=0
instructions candidate-05 sample=1 optionvalue-three-no-input N=40000 exit=0
instructions candidate-05 sample=1 replacerepeated-no-input N=20000 exit=0
instructions candidate-05 sample=1 replacerepeated-no-input N=40000 exit=0
instructions candidate-05 sample=1 dot-three-no-input N=20000 exit=0
instructions candidate-05 sample=1 dot-three-no-input N=40000 exit=0
instructions candidate-05 sample=1 differences-no-input N=20000 exit=0
instructions candidate-05 sample=1 differences-no-input N=40000 exit=0
instructions candidate-05 sample=1 do-count-null N=20000 exit=0
instructions candidate-05 sample=1 do-count-null N=40000 exit=0
instructions candidate-05 sample=1 do-iterator-null N=20000 exit=0
instructions candidate-05 sample=1 do-iterator-null N=40000 exit=0
instructions candidate-05 sample=1 table-iterator-null N=20000 exit=0
instructions candidate-05 sample=1 table-iterator-null N=40000 exit=0
instructions candidate-05 sample=1 replaceall-simple N=20000 exit=0
instructions candidate-05 sample=1 replaceall-simple N=40000 exit=0
```

**37. [.temp/brief-560/candidate-05/instructions-2.txt](../../../.temp/brief-560/candidate-05/instructions-2.txt) — bound 20s; exit 0; elapsed 2.957672s.**

```text
python3 scripts/run-bounded.py --timeout 20 -- python3 .temp/brief-560/perf-panel.py instructions candidate-05 2
```

```text
instructions candidate-05 sample=2 scan-no-input N=20000 exit=0
instructions candidate-05 sample=2 scan-no-input N=40000 exit=0
instructions candidate-05 sample=2 optionvalue-three-no-input N=20000 exit=0
instructions candidate-05 sample=2 optionvalue-three-no-input N=40000 exit=0
instructions candidate-05 sample=2 replacerepeated-no-input N=20000 exit=0
instructions candidate-05 sample=2 replacerepeated-no-input N=40000 exit=0
instructions candidate-05 sample=2 dot-three-no-input N=20000 exit=0
instructions candidate-05 sample=2 dot-three-no-input N=40000 exit=0
instructions candidate-05 sample=2 differences-no-input N=20000 exit=0
instructions candidate-05 sample=2 differences-no-input N=40000 exit=0
instructions candidate-05 sample=2 do-count-null N=20000 exit=0
instructions candidate-05 sample=2 do-count-null N=40000 exit=0
instructions candidate-05 sample=2 do-iterator-null N=20000 exit=0
instructions candidate-05 sample=2 do-iterator-null N=40000 exit=0
instructions candidate-05 sample=2 table-iterator-null N=20000 exit=0
instructions candidate-05 sample=2 table-iterator-null N=40000 exit=0
instructions candidate-05 sample=2 replaceall-simple N=20000 exit=0
instructions candidate-05 sample=2 replaceall-simple N=40000 exit=0
```

**38. [.temp/brief-560/candidate-06/loops.txt](../../../.temp/brief-560/candidate-06/loops.txt) — bound 80s; exit 0; elapsed 12.247810s.**

```text
python3 scripts/run-bounded.py --timeout 80 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-560/tmp cargo test -p rum-stdlib --test loop_continuations
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 12.03s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
```

**39. [.temp/brief-560/candidate-06/default-build.txt](../../../.temp/brief-560/candidate-06/default-build.txt) — bound 240s; exit 0; elapsed 106.753918s.**

```text
python3 scripts/run-bounded.py --timeout 240 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-560/tmp cargo build --release --target-dir .temp/brief-560/candidate-06/target -p rum-cli
```

```text
    Finished `release` profile [optimized] target(s) in 1m 46s
```

**40. [.temp/brief-560/candidate-06/alloc-build.txt](../../../.temp/brief-560/candidate-06/alloc-build.txt) — bound 20s; exit 0; elapsed 2.299725s.**

```text
python3 scripts/run-bounded.py --timeout 20 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-560/tmp cargo build --release --target-dir .temp/brief-560/candidate-06/target -p rum-cli --features alloc-count
```

```text
    Finished `release` profile [optimized] target(s) in 2.21s
```

**41. [.temp/brief-560/candidate-06/alloc-1.txt](../../../.temp/brief-560/candidate-06/alloc-1.txt) — bound 20s; exit 0; elapsed 2.957949s.**

```text
python3 scripts/run-bounded.py --timeout 20 -- python3 .temp/brief-560/perf-panel.py alloc candidate-06 1
```

```text
alloc candidate-06 sample=1 scan-no-input N=20000 exit=0
alloc candidate-06 sample=1 scan-no-input N=40000 exit=0
alloc candidate-06 sample=1 optionvalue-three-no-input N=20000 exit=0
alloc candidate-06 sample=1 optionvalue-three-no-input N=40000 exit=0
alloc candidate-06 sample=1 replacerepeated-no-input N=20000 exit=0
alloc candidate-06 sample=1 replacerepeated-no-input N=40000 exit=0
alloc candidate-06 sample=1 dot-three-no-input N=20000 exit=0
alloc candidate-06 sample=1 dot-three-no-input N=40000 exit=0
alloc candidate-06 sample=1 differences-no-input N=20000 exit=0
alloc candidate-06 sample=1 differences-no-input N=40000 exit=0
alloc candidate-06 sample=1 do-count-null N=20000 exit=0
alloc candidate-06 sample=1 do-count-null N=40000 exit=0
alloc candidate-06 sample=1 do-iterator-null N=20000 exit=0
alloc candidate-06 sample=1 do-iterator-null N=40000 exit=0
alloc candidate-06 sample=1 table-iterator-null N=20000 exit=0
alloc candidate-06 sample=1 table-iterator-null N=40000 exit=0
alloc candidate-06 sample=1 replaceall-simple N=20000 exit=0
alloc candidate-06 sample=1 replaceall-simple N=40000 exit=0
```

**42. [.temp/brief-560/candidate-06/alloc-2.txt](../../../.temp/brief-560/candidate-06/alloc-2.txt) — bound 20s; exit 0; elapsed 2.856887s.**

```text
python3 scripts/run-bounded.py --timeout 20 -- python3 .temp/brief-560/perf-panel.py alloc candidate-06 2
```

```text
alloc candidate-06 sample=2 scan-no-input N=20000 exit=0
alloc candidate-06 sample=2 scan-no-input N=40000 exit=0
alloc candidate-06 sample=2 optionvalue-three-no-input N=20000 exit=0
alloc candidate-06 sample=2 optionvalue-three-no-input N=40000 exit=0
alloc candidate-06 sample=2 replacerepeated-no-input N=20000 exit=0
alloc candidate-06 sample=2 replacerepeated-no-input N=40000 exit=0
alloc candidate-06 sample=2 dot-three-no-input N=20000 exit=0
alloc candidate-06 sample=2 dot-three-no-input N=40000 exit=0
alloc candidate-06 sample=2 differences-no-input N=20000 exit=0
alloc candidate-06 sample=2 differences-no-input N=40000 exit=0
alloc candidate-06 sample=2 do-count-null N=20000 exit=0
alloc candidate-06 sample=2 do-count-null N=40000 exit=0
alloc candidate-06 sample=2 do-iterator-null N=20000 exit=0
alloc candidate-06 sample=2 do-iterator-null N=40000 exit=0
alloc candidate-06 sample=2 table-iterator-null N=20000 exit=0
alloc candidate-06 sample=2 table-iterator-null N=40000 exit=0
alloc candidate-06 sample=2 replaceall-simple N=20000 exit=0
alloc candidate-06 sample=2 replaceall-simple N=40000 exit=0
```

**43. [.temp/brief-560/candidate-06/instructions-1.txt](../../../.temp/brief-560/candidate-06/instructions-1.txt) — bound 20s; exit 0; elapsed 2.910096s.**

```text
python3 scripts/run-bounded.py --timeout 20 -- python3 .temp/brief-560/perf-panel.py instructions candidate-06 1
```

```text
instructions candidate-06 sample=1 scan-no-input N=20000 exit=0
instructions candidate-06 sample=1 scan-no-input N=40000 exit=0
instructions candidate-06 sample=1 optionvalue-three-no-input N=20000 exit=0
instructions candidate-06 sample=1 optionvalue-three-no-input N=40000 exit=0
instructions candidate-06 sample=1 replacerepeated-no-input N=20000 exit=0
instructions candidate-06 sample=1 replacerepeated-no-input N=40000 exit=0
instructions candidate-06 sample=1 dot-three-no-input N=20000 exit=0
instructions candidate-06 sample=1 dot-three-no-input N=40000 exit=0
instructions candidate-06 sample=1 differences-no-input N=20000 exit=0
instructions candidate-06 sample=1 differences-no-input N=40000 exit=0
instructions candidate-06 sample=1 do-count-null N=20000 exit=0
instructions candidate-06 sample=1 do-count-null N=40000 exit=0
instructions candidate-06 sample=1 do-iterator-null N=20000 exit=0
instructions candidate-06 sample=1 do-iterator-null N=40000 exit=0
instructions candidate-06 sample=1 table-iterator-null N=20000 exit=0
instructions candidate-06 sample=1 table-iterator-null N=40000 exit=0
instructions candidate-06 sample=1 replaceall-simple N=20000 exit=0
instructions candidate-06 sample=1 replaceall-simple N=40000 exit=0
```

**44. [.temp/brief-560/candidate-06/instructions-2.txt](../../../.temp/brief-560/candidate-06/instructions-2.txt) — bound 20s; exit 0; elapsed 2.905559s.**

```text
python3 scripts/run-bounded.py --timeout 20 -- python3 .temp/brief-560/perf-panel.py instructions candidate-06 2
```

```text
instructions candidate-06 sample=2 scan-no-input N=20000 exit=0
instructions candidate-06 sample=2 scan-no-input N=40000 exit=0
instructions candidate-06 sample=2 optionvalue-three-no-input N=20000 exit=0
instructions candidate-06 sample=2 optionvalue-three-no-input N=40000 exit=0
instructions candidate-06 sample=2 replacerepeated-no-input N=20000 exit=0
instructions candidate-06 sample=2 replacerepeated-no-input N=40000 exit=0
instructions candidate-06 sample=2 dot-three-no-input N=20000 exit=0
instructions candidate-06 sample=2 dot-three-no-input N=40000 exit=0
instructions candidate-06 sample=2 differences-no-input N=20000 exit=0
instructions candidate-06 sample=2 differences-no-input N=40000 exit=0
instructions candidate-06 sample=2 do-count-null N=20000 exit=0
instructions candidate-06 sample=2 do-count-null N=40000 exit=0
instructions candidate-06 sample=2 do-iterator-null N=20000 exit=0
instructions candidate-06 sample=2 do-iterator-null N=40000 exit=0
instructions candidate-06 sample=2 table-iterator-null N=20000 exit=0
instructions candidate-06 sample=2 table-iterator-null N=40000 exit=0
instructions candidate-06 sample=2 replaceall-simple N=20000 exit=0
instructions candidate-06 sample=2 replaceall-simple N=40000 exit=0
```

**45. [.temp/brief-560/candidate-07/default-build.txt](../../../.temp/brief-560/candidate-07/default-build.txt) — bound 240s; exit 0; elapsed 108.091418s.**

```text
python3 scripts/run-bounded.py --timeout 240 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-560/tmp cargo build --release --target-dir .temp/brief-560/candidate-07/target -p rum-cli
```

```text
    Finished `release` profile [optimized] target(s) in 1m 48s
```

**46. [.temp/brief-560/candidate-07/alloc-build.txt](../../../.temp/brief-560/candidate-07/alloc-build.txt) — bound 20s; exit 0; elapsed 2.354027s.**

```text
python3 scripts/run-bounded.py --timeout 20 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-560/tmp cargo build --release --target-dir .temp/brief-560/candidate-07/target -p rum-cli --features alloc-count
```

```text
    Finished `release` profile [optimized] target(s) in 2.26s
```

**47. [.temp/brief-560/candidate-07/alloc-1.txt](../../../.temp/brief-560/candidate-07/alloc-1.txt) — bound 20s; exit 0; elapsed 2.911514s.**

```text
python3 scripts/run-bounded.py --timeout 20 -- python3 .temp/brief-560/perf-panel.py alloc candidate-07 1
```

```text
alloc candidate-07 sample=1 scan-no-input N=20000 exit=0
alloc candidate-07 sample=1 scan-no-input N=40000 exit=0
alloc candidate-07 sample=1 optionvalue-three-no-input N=20000 exit=0
alloc candidate-07 sample=1 optionvalue-three-no-input N=40000 exit=0
alloc candidate-07 sample=1 replacerepeated-no-input N=20000 exit=0
alloc candidate-07 sample=1 replacerepeated-no-input N=40000 exit=0
alloc candidate-07 sample=1 dot-three-no-input N=20000 exit=0
alloc candidate-07 sample=1 dot-three-no-input N=40000 exit=0
alloc candidate-07 sample=1 differences-no-input N=20000 exit=0
alloc candidate-07 sample=1 differences-no-input N=40000 exit=0
alloc candidate-07 sample=1 do-count-null N=20000 exit=0
alloc candidate-07 sample=1 do-count-null N=40000 exit=0
alloc candidate-07 sample=1 do-iterator-null N=20000 exit=0
alloc candidate-07 sample=1 do-iterator-null N=40000 exit=0
alloc candidate-07 sample=1 table-iterator-null N=20000 exit=0
alloc candidate-07 sample=1 table-iterator-null N=40000 exit=0
alloc candidate-07 sample=1 replaceall-simple N=20000 exit=0
alloc candidate-07 sample=1 replaceall-simple N=40000 exit=0
```

**48. [.temp/brief-560/candidate-07/alloc-2.txt](../../../.temp/brief-560/candidate-07/alloc-2.txt) — bound 20s; exit 0; elapsed 2.902349s.**

```text
python3 scripts/run-bounded.py --timeout 20 -- python3 .temp/brief-560/perf-panel.py alloc candidate-07 2
```

```text
alloc candidate-07 sample=2 scan-no-input N=20000 exit=0
alloc candidate-07 sample=2 scan-no-input N=40000 exit=0
alloc candidate-07 sample=2 optionvalue-three-no-input N=20000 exit=0
alloc candidate-07 sample=2 optionvalue-three-no-input N=40000 exit=0
alloc candidate-07 sample=2 replacerepeated-no-input N=20000 exit=0
alloc candidate-07 sample=2 replacerepeated-no-input N=40000 exit=0
alloc candidate-07 sample=2 dot-three-no-input N=20000 exit=0
alloc candidate-07 sample=2 dot-three-no-input N=40000 exit=0
alloc candidate-07 sample=2 differences-no-input N=20000 exit=0
alloc candidate-07 sample=2 differences-no-input N=40000 exit=0
alloc candidate-07 sample=2 do-count-null N=20000 exit=0
alloc candidate-07 sample=2 do-count-null N=40000 exit=0
alloc candidate-07 sample=2 do-iterator-null N=20000 exit=0
alloc candidate-07 sample=2 do-iterator-null N=40000 exit=0
alloc candidate-07 sample=2 table-iterator-null N=20000 exit=0
alloc candidate-07 sample=2 table-iterator-null N=40000 exit=0
alloc candidate-07 sample=2 replaceall-simple N=20000 exit=0
alloc candidate-07 sample=2 replaceall-simple N=40000 exit=0
```

**49. [.temp/brief-560/candidate-07/instructions-1.txt](../../../.temp/brief-560/candidate-07/instructions-1.txt) — bound 20s; exit 0; elapsed 2.857454s.**

```text
python3 scripts/run-bounded.py --timeout 20 -- python3 .temp/brief-560/perf-panel.py instructions candidate-07 1
```

```text
instructions candidate-07 sample=1 scan-no-input N=20000 exit=0
instructions candidate-07 sample=1 scan-no-input N=40000 exit=0
instructions candidate-07 sample=1 optionvalue-three-no-input N=20000 exit=0
instructions candidate-07 sample=1 optionvalue-three-no-input N=40000 exit=0
instructions candidate-07 sample=1 replacerepeated-no-input N=20000 exit=0
instructions candidate-07 sample=1 replacerepeated-no-input N=40000 exit=0
instructions candidate-07 sample=1 dot-three-no-input N=20000 exit=0
instructions candidate-07 sample=1 dot-three-no-input N=40000 exit=0
instructions candidate-07 sample=1 differences-no-input N=20000 exit=0
instructions candidate-07 sample=1 differences-no-input N=40000 exit=0
instructions candidate-07 sample=1 do-count-null N=20000 exit=0
instructions candidate-07 sample=1 do-count-null N=40000 exit=0
instructions candidate-07 sample=1 do-iterator-null N=20000 exit=0
instructions candidate-07 sample=1 do-iterator-null N=40000 exit=0
instructions candidate-07 sample=1 table-iterator-null N=20000 exit=0
instructions candidate-07 sample=1 table-iterator-null N=40000 exit=0
instructions candidate-07 sample=1 replaceall-simple N=20000 exit=0
instructions candidate-07 sample=1 replaceall-simple N=40000 exit=0
```

**50. [.temp/brief-560/candidate-07/instructions-2.txt](../../../.temp/brief-560/candidate-07/instructions-2.txt) — bound 20s; exit 0; elapsed 2.802031s.**

```text
python3 scripts/run-bounded.py --timeout 20 -- python3 .temp/brief-560/perf-panel.py instructions candidate-07 2
```

```text
instructions candidate-07 sample=2 scan-no-input N=20000 exit=0
instructions candidate-07 sample=2 scan-no-input N=40000 exit=0
instructions candidate-07 sample=2 optionvalue-three-no-input N=20000 exit=0
instructions candidate-07 sample=2 optionvalue-three-no-input N=40000 exit=0
instructions candidate-07 sample=2 replacerepeated-no-input N=20000 exit=0
instructions candidate-07 sample=2 replacerepeated-no-input N=40000 exit=0
instructions candidate-07 sample=2 dot-three-no-input N=20000 exit=0
instructions candidate-07 sample=2 dot-three-no-input N=40000 exit=0
instructions candidate-07 sample=2 differences-no-input N=20000 exit=0
instructions candidate-07 sample=2 differences-no-input N=40000 exit=0
instructions candidate-07 sample=2 do-count-null N=20000 exit=0
instructions candidate-07 sample=2 do-count-null N=40000 exit=0
instructions candidate-07 sample=2 do-iterator-null N=20000 exit=0
instructions candidate-07 sample=2 do-iterator-null N=40000 exit=0
instructions candidate-07 sample=2 table-iterator-null N=20000 exit=0
instructions candidate-07 sample=2 table-iterator-null N=40000 exit=0
instructions candidate-07 sample=2 replaceall-simple N=20000 exit=0
instructions candidate-07 sample=2 replaceall-simple N=40000 exit=0
```

**51. [.temp/brief-560/candidate-08/fmt-check.txt](../../../.temp/brief-560/candidate-08/fmt-check.txt) — bound 10s; exit 1; elapsed 1.850796s.**

```text
python3 scripts/run-bounded.py --timeout 10 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-560/tmp cargo fmt --check
```

```text
Diff in /home/remi/dev/rum/crates/rum-runtime/src/runtime/options/builtins.rs:270:
```

**52. [.temp/brief-560/candidate-08/loops.txt](../../../.temp/brief-560/candidate-08/loops.txt) — bound 80s; exit 0; elapsed 14.101218s.**

```text
python3 scripts/run-bounded.py --timeout 80 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-560/tmp cargo test -p rum-stdlib --test loop_continuations
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 13.91s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
```

**53. [.temp/brief-560/candidate-08/default-build.txt](../../../.temp/brief-560/candidate-08/default-build.txt) — bound 240s; exit 0; elapsed 107.764382s.**

```text
python3 scripts/run-bounded.py --timeout 240 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-560/tmp cargo build --release --target-dir .temp/brief-560/candidate-08/target -p rum-cli
```

```text
    Finished `release` profile [optimized] target(s) in 1m 47s
```

**54. [.temp/brief-560/candidate-08/alloc-build.txt](../../../.temp/brief-560/candidate-08/alloc-build.txt) — bound 20s; exit 0; elapsed 2.458969s.**

```text
python3 scripts/run-bounded.py --timeout 20 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-560/tmp cargo build --release --target-dir .temp/brief-560/candidate-08/target -p rum-cli --features alloc-count
```

```text
    Finished `release` profile [optimized] target(s) in 2.34s
```

**55. [.temp/brief-560/candidate-08/alloc-1.txt](../../../.temp/brief-560/candidate-08/alloc-1.txt) — bound 20s; exit 0; elapsed 2.954043s.**

```text
python3 scripts/run-bounded.py --timeout 20 -- python3 .temp/brief-560/perf-panel.py alloc candidate-08 1
```

```text
alloc candidate-08 sample=1 scan-no-input N=20000 exit=0
alloc candidate-08 sample=1 scan-no-input N=40000 exit=0
alloc candidate-08 sample=1 optionvalue-three-no-input N=20000 exit=0
alloc candidate-08 sample=1 optionvalue-three-no-input N=40000 exit=0
alloc candidate-08 sample=1 replacerepeated-no-input N=20000 exit=0
alloc candidate-08 sample=1 replacerepeated-no-input N=40000 exit=0
alloc candidate-08 sample=1 dot-three-no-input N=20000 exit=0
alloc candidate-08 sample=1 dot-three-no-input N=40000 exit=0
alloc candidate-08 sample=1 differences-no-input N=20000 exit=0
alloc candidate-08 sample=1 differences-no-input N=40000 exit=0
alloc candidate-08 sample=1 do-count-null N=20000 exit=0
alloc candidate-08 sample=1 do-count-null N=40000 exit=0
alloc candidate-08 sample=1 do-iterator-null N=20000 exit=0
alloc candidate-08 sample=1 do-iterator-null N=40000 exit=0
alloc candidate-08 sample=1 table-iterator-null N=20000 exit=0
alloc candidate-08 sample=1 table-iterator-null N=40000 exit=0
alloc candidate-08 sample=1 replaceall-simple N=20000 exit=0
alloc candidate-08 sample=1 replaceall-simple N=40000 exit=0
```

**56. [.temp/brief-560/candidate-08/alloc-2.txt](../../../.temp/brief-560/candidate-08/alloc-2.txt) — bound 20s; exit 0; elapsed 2.950245s.**

```text
python3 scripts/run-bounded.py --timeout 20 -- python3 .temp/brief-560/perf-panel.py alloc candidate-08 2
```

```text
alloc candidate-08 sample=2 scan-no-input N=20000 exit=0
alloc candidate-08 sample=2 scan-no-input N=40000 exit=0
alloc candidate-08 sample=2 optionvalue-three-no-input N=20000 exit=0
alloc candidate-08 sample=2 optionvalue-three-no-input N=40000 exit=0
alloc candidate-08 sample=2 replacerepeated-no-input N=20000 exit=0
alloc candidate-08 sample=2 replacerepeated-no-input N=40000 exit=0
alloc candidate-08 sample=2 dot-three-no-input N=20000 exit=0
alloc candidate-08 sample=2 dot-three-no-input N=40000 exit=0
alloc candidate-08 sample=2 differences-no-input N=20000 exit=0
alloc candidate-08 sample=2 differences-no-input N=40000 exit=0
alloc candidate-08 sample=2 do-count-null N=20000 exit=0
alloc candidate-08 sample=2 do-count-null N=40000 exit=0
alloc candidate-08 sample=2 do-iterator-null N=20000 exit=0
alloc candidate-08 sample=2 do-iterator-null N=40000 exit=0
alloc candidate-08 sample=2 table-iterator-null N=20000 exit=0
alloc candidate-08 sample=2 table-iterator-null N=40000 exit=0
alloc candidate-08 sample=2 replaceall-simple N=20000 exit=0
alloc candidate-08 sample=2 replaceall-simple N=40000 exit=0
```

**57. [.temp/brief-560/candidate-08/instructions-1.txt](../../../.temp/brief-560/candidate-08/instructions-1.txt) — bound 20s; exit 0; elapsed 3.054683s.**

```text
python3 scripts/run-bounded.py --timeout 20 -- python3 .temp/brief-560/perf-panel.py instructions candidate-08 1
```

```text
instructions candidate-08 sample=1 scan-no-input N=20000 exit=0
instructions candidate-08 sample=1 scan-no-input N=40000 exit=0
instructions candidate-08 sample=1 optionvalue-three-no-input N=20000 exit=0
instructions candidate-08 sample=1 optionvalue-three-no-input N=40000 exit=0
instructions candidate-08 sample=1 replacerepeated-no-input N=20000 exit=0
instructions candidate-08 sample=1 replacerepeated-no-input N=40000 exit=0
instructions candidate-08 sample=1 dot-three-no-input N=20000 exit=0
instructions candidate-08 sample=1 dot-three-no-input N=40000 exit=0
instructions candidate-08 sample=1 differences-no-input N=20000 exit=0
instructions candidate-08 sample=1 differences-no-input N=40000 exit=0
instructions candidate-08 sample=1 do-count-null N=20000 exit=0
instructions candidate-08 sample=1 do-count-null N=40000 exit=0
instructions candidate-08 sample=1 do-iterator-null N=20000 exit=0
instructions candidate-08 sample=1 do-iterator-null N=40000 exit=0
instructions candidate-08 sample=1 table-iterator-null N=20000 exit=0
instructions candidate-08 sample=1 table-iterator-null N=40000 exit=0
instructions candidate-08 sample=1 replaceall-simple N=20000 exit=0
instructions candidate-08 sample=1 replaceall-simple N=40000 exit=0
```

**58. [.temp/brief-560/candidate-08/instructions-2.txt](../../../.temp/brief-560/candidate-08/instructions-2.txt) — bound 20s; exit 0; elapsed 3.001161s.**

```text
python3 scripts/run-bounded.py --timeout 20 -- python3 .temp/brief-560/perf-panel.py instructions candidate-08 2
```

```text
instructions candidate-08 sample=2 scan-no-input N=20000 exit=0
instructions candidate-08 sample=2 scan-no-input N=40000 exit=0
instructions candidate-08 sample=2 optionvalue-three-no-input N=20000 exit=0
instructions candidate-08 sample=2 optionvalue-three-no-input N=40000 exit=0
instructions candidate-08 sample=2 replacerepeated-no-input N=20000 exit=0
instructions candidate-08 sample=2 replacerepeated-no-input N=40000 exit=0
instructions candidate-08 sample=2 dot-three-no-input N=20000 exit=0
instructions candidate-08 sample=2 dot-three-no-input N=40000 exit=0
instructions candidate-08 sample=2 differences-no-input N=20000 exit=0
instructions candidate-08 sample=2 differences-no-input N=40000 exit=0
instructions candidate-08 sample=2 do-count-null N=20000 exit=0
instructions candidate-08 sample=2 do-count-null N=40000 exit=0
instructions candidate-08 sample=2 do-iterator-null N=20000 exit=0
instructions candidate-08 sample=2 do-iterator-null N=40000 exit=0
instructions candidate-08 sample=2 table-iterator-null N=20000 exit=0
instructions candidate-08 sample=2 table-iterator-null N=40000 exit=0
instructions candidate-08 sample=2 replaceall-simple N=20000 exit=0
instructions candidate-08 sample=2 replaceall-simple N=40000 exit=0
```

**59. [.temp/brief-560/verification/fmt-check.txt](../../../.temp/brief-560/verification/fmt-check.txt) — bound 10s; exit 0; elapsed 1.748333s.**

```text
python3 scripts/run-bounded.py --timeout 10 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-560/tmp cargo fmt --check
```

No output; the captured command exited 0.

**60. [.temp/brief-560/verification/loops.txt](../../../.temp/brief-560/verification/loops.txt) — bound 80s; exit 0; elapsed 12.796412s.**

```text
python3 scripts/run-bounded.py --timeout 80 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-560/tmp cargo test -p rum-stdlib --test loop_continuations
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 12.59s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
```

**61. [.temp/brief-560/verification/runtime.txt](../../../.temp/brief-560/verification/runtime.txt) — bound 100s; exit 0; elapsed 34.822397s.**

```text
python3 scripts/run-bounded.py --timeout 100 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-560/tmp cargo test -p rum-runtime
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 12.59s
test result: ok. 1094 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 19.18s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.18s
```

**62. [.temp/brief-560/verification/ownership-discovery.txt](../../../.temp/brief-560/verification/ownership-discovery.txt) — bound 80s; exit 0; elapsed 27.207268s.**

```text
python3 scripts/run-bounded.py --timeout 80 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-560/tmp cargo test -p rum-stdlib --lib tests::ownership_graph::discovery_report_without_inventory_input -- --exact --nocapture
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 21.89s
UnknownConstruct count: 7568
Self-reaching type count: 60
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1077 filtered out; finished in 5.24s
```

**63. [.temp/brief-560/verification/regenerate-fingerprints.txt](../../../.temp/brief-560/verification/regenerate-fingerprints.txt) — bound 10s; exit 124; elapsed 10.084344s.**

```text
python3 scripts/run-bounded.py --timeout 10 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-560/tmp cargo test -p rum-stdlib --lib tests::retention_closure::local::contract_tests::regenerate_local_contract_fingerprints -- --exact --ignored --nocapture
```

```text
run-bounded: timed out after 10s: env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-560/tmp cargo test -p rum-stdlib --lib tests::retention_closure::local::contract_tests::regenerate_local_contract_fingerprints -- --exact --ignored --nocapture
```

**64. [.temp/brief-560/verification/regenerate-fingerprints-retry.txt](../../../.temp/brief-560/verification/regenerate-fingerprints-retry.txt) — bound 80s; exit 0; elapsed 13.660943s.**

```text
python3 scripts/run-bounded.py --timeout 80 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-560/tmp cargo test -p rum-stdlib --lib tests::retention_closure::local::contract_tests::regenerate_local_contract_fingerprints -- --exact --ignored --nocapture
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 13.40s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1077 filtered out; finished in 0.15s
```

**65. [.temp/brief-560/verification/reconcile-fingerprints.txt](../../../.temp/brief-560/verification/reconcile-fingerprints.txt) — bound 10s; exit 0; elapsed 0.064147s.**

```text
python3 scripts/run-bounded.py --timeout 10 -- python3 .temp/brief-560/reconcile-fingerprints.py
```

```text
Contract fingerprints: 68 refreshed; 68 regenerated; first 18 fields identical; 62 Discharged, 6 Pending unchanged.
```

**66. [.temp/brief-560/verification/cert-default-generate.txt](../../../.temp/brief-560/verification/cert-default-generate.txt) — bound 120s; exit 101; elapsed 31.726881s.**

```text
python3 scripts/run-bounded.py --timeout 120 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-560/tmp cargo test -p rum-stdlib --lib tests::retention_closure::local::production::local_production_acceptance -- --exact --nocapture
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 14.55s
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=true"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1077 filtered out; finished in 17.03s
```

**67. [.temp/brief-560/verification/cert-nodefault-generate.txt](../../../.temp/brief-560/verification/cert-nodefault-generate.txt) — bound 130s; exit 101; elapsed 38.904074s.**

```text
python3 scripts/run-bounded.py --timeout 130 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-560/tmp cargo test -p rum-stdlib --no-default-features --lib tests::retention_closure::local::production::local_production_acceptance -- --exact --nocapture
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 21.84s
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=false"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1071 filtered out; finished in 16.95s
```

**68. [.temp/brief-560/verification/install-certificates.txt](../../../.temp/brief-560/verification/install-certificates.txt) — bound 10s; exit 0; elapsed 0.064354s.**

```text
python3 scripts/run-bounded.py --timeout 10 -- python3 .temp/brief-560/install-certificates.py
```

```text
Replayed four certificate cells; C01/C02 identities/status/source/preconditions/strength/exclusions unchanged.
```

**69. [.temp/brief-560/verification/scope-audit-before-suites.txt](../../../.temp/brief-560/verification/scope-audit-before-suites.txt) — bound 10s; exit 0; elapsed 0.316757s.**

```text
python3 scripts/run-bounded.py --timeout 10 -- python3 .temp/brief-560/audit.py
```

```text
SCOPE PASS: 4 production files, 5 maintenance inventories; no out-of-scope changes.
UNCHANGED: 490 test/fixture/inventory files outside authorized maintenance; all initial files present; preserved base hashes verified; no new unsafe.
```

**70. [.temp/brief-560/verification/stdlib.txt](../../../.temp/brief-560/verification/stdlib.txt) — bound 2200s; exit 0; elapsed 990.892702s.**

```text
python3 scripts/run-bounded.py --timeout 2200 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-560/tmp cargo test -p rum-stdlib
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 21.71s
test result: ok. 1025 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 967.48s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.78s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**71. [.temp/brief-560/verification/workspace.txt](../../../.temp/brief-560/verification/workspace.txt) — bound 2200s; exit 0; elapsed 651.348842s.**

```text
python3 scripts/run-bounded.py --timeout 2200 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-560/tmp RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test --workspace -- --nocapture
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2m 23s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.68s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.50s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.69s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.72s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.65s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.67s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.67s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.74s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 33.55s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.82s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.53s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
test result: ok. 1094 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 18.88s
UnknownConstruct count: 7568
Self-reaching type count: 60
EXACT INVENTORY UnknownHostPayload: 63 rows
EXACT INVENTORY UnknownConstruct: 7568 rows
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
expired-C01: RED stale or omitted instance certificate C01 hash=true
omitted-C01-instance-0: RED stale or omitted instance certificate C01 hash=true
omitted-C01-instance-1: RED stale or omitted instance certificate C01 hash=true
expired-C02: RED stale or omitted instance certificate C02 hash=true
PRODUCTION_BODY_ROWS=2 N=78 UNSPLIT=147 UNSETTLED=90 LOCAL_OPTION_BODIES=2 PENDING=315
test result: ok. 1025 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 435.78s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.79s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.34s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.88s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.08s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**72. [.temp/brief-560/verification/retention-nodefault.txt](../../../.temp/brief-560/verification/retention-nodefault.txt) — bound 1900s; exit 0; elapsed 900.142033s.**

```text
python3 scripts/run-bounded.py --timeout 1900 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-560/tmp cargo test -p rum-stdlib --no-default-features --lib tests::retention_closure -- --nocapture
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 14.96s
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
expired-C01: RED stale or omitted instance certificate C01 hash=false
omitted-C01-instance-0: RED stale or omitted instance certificate C01 hash=false
omitted-C01-instance-1: RED stale or omitted instance certificate C01 hash=false
expired-C02: RED stale or omitted instance certificate C02 hash=false
PRODUCTION_BODY_ROWS=2 N=78 UNSPLIT=147 UNSETTLED=90 LOCAL_OPTION_BODIES=2 PENDING=315
test result: ok. 49 passed; 0 failed; 1 ignored; 0 measured; 1022 filtered out; finished in 884.98s
```

**73. [.temp/brief-560/verification/scope-audit-final.txt](../../../.temp/brief-560/verification/scope-audit-final.txt) — bound 10s; exit 0; elapsed 0.366113s.**

```text
python3 scripts/run-bounded.py --timeout 10 -- python3 .temp/brief-560/audit.py
```

```text
SCOPE PASS: 4 production files, 5 maintenance inventories; no out-of-scope changes.
UNCHANGED: 490 test/fixture/inventory files outside authorized maintenance; all initial files present; preserved base hashes verified; no new unsafe.
```

Native ZeroMQ skips from the workspace log (nine; the harness reports these early returns among its 2,877 reported passes):

```text
test zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::basic::native_zeromq_backend_round_trips_execute_over_loopback_sockets ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_frontend_style_control_interrupt_round_trip ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::execute::native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::session_flow::native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::stdin::native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
```

Required gate results: fmt exit 0; unchanged loop_continuations 21 passed; runtime 1094 unit + 22 doc tests passed, three ignored; stdlib 1025 unit + 47 builtin-continuation + 21 loop-continuation tests passed, 53 ignored; workspace 2877 reported passed / 0 failed / 58 ignored, including the nine native skips above; no-default retention 49 passed / 0 failed / 1 ignored / 1022 filtered.

4. **Progress counters, ownership disclosures and unresolved work.**

| Build | Labels above +0.05% in either sample | Largest remaining delta | Non-progress streak | Decision |
| --- | ---: | ---: | ---: | --- |
| Entry (brief-559 measurements; not a new entry-build run) | 3 | +1.863% | 0 | Starting defect |
| candidate-01 | 5 | +1.099427% | 0 | Reject as final implementation |
| candidate-02 | 4 | +0.862110% | 0 | Reject as final implementation |
| candidate-03 | 4 | +1.168073% | 1 | Reject as final implementation |
| candidate-04 | 3 | +0.878069% | 0 | Reject as final implementation |
| candidate-05 | 2 | +0.860781% | 0 | Reject as final implementation |
| candidate-06 | 1 | +0.877091% | 0 | Retained best; objective still unmet |
| candidate-07 | 2 | +0.881436% | 1 | Reject as final implementation |
| candidate-08 | 2 | +0.973128% | 2 | Reject; mandatory stop-loss |

Candidate 01 reduced the largest delta despite increasing the number of failing labels; 02 improved both counters; 03 improved neither; 04/05/06 each reduced the failing-label count; 07 and 08 improved neither in succession. Optimization stopped immediately after 08. The final tree is candidate 06 restored byte-for-byte except the formatter-only OptionValue call wrap. Performance rows/hashes refer to the preserved candidate-06 binaries; no post-stop release rebuild or performance rerun was performed for that formatting change.

Standing duty was replayed, not hand-authored. Ownership discovery has 63 UnknownHostPayload rows (unchanged), 7,568 UnknownConstruct rows (six additions), and 60 self-reaching type rows (unchanged). All existing identities, statuses, dispositions and exclusions were preserved. No new host payload, removed identity or addition in untouched code occurred. Six additions and 62 position-only moves follow; unknown constructs remain unknown.

**host_payloads.tsv: 63 → 63; additions 0; position-only moves 0; removals 0.**

**unknown_attributes.tsv: 1434 → 1434; additions 0; position-only moves 13; removals 0.**

```tsv
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	244	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	335	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	288	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	379	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	329	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	420	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	374	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	465	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	407	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	498	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	445	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	536	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/options/builtins.rs	455	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/options/builtins.rs	491	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/options/builtins.rs	489	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/options/builtins.rs	525	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/options/builtins.rs	523	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/options/builtins.rs	559	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/functional_ops/arrays/mod.rs	81	1	unresolved attribute/expansion derive (rum_runtime :: kernel_api :: FrameRootTrace , rum_runtime :: kernel_api :: PureFrameAbandon)
AFTER	UnknownConstruct	crates/rum-stdlib/src/functional_ops/arrays/mod.rs	119	1	unresolved attribute/expansion derive (rum_runtime :: kernel_api :: FrameRootTrace , rum_runtime :: kernel_api :: PureFrameAbandon)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	55	1	unresolved attribute/expansion derive (rum_runtime :: kernel_api :: FrameRootTrace , rum_runtime :: kernel_api :: PureFrameAbandon)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	84	1	unresolved attribute/expansion derive (rum_runtime :: kernel_api :: FrameRootTrace , rum_runtime :: kernel_api :: PureFrameAbandon)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	150	1	unresolved attribute/expansion derive (rum_runtime :: kernel_api :: FrameRootTrace , rum_runtime :: kernel_api :: PureFrameAbandon)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	210	1	unresolved attribute/expansion derive (rum_runtime :: kernel_api :: FrameRootTrace , rum_runtime :: kernel_api :: PureFrameAbandon)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	294	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	354	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq)
```

**unknown_captures.tsv: 2938 → 2938; additions 0; position-only moves 29; removals 0.**

```tsv
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	228	34	closure captures: |symbol| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	319	34	closure captures: |symbol| (capture types/modes unresolved)
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	240	17	closure captures: |limit| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	331	17	closure captures: |limit| (capture types/modes unresolved)
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	241	14	closure captures: |limit| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	332	14	closure captures: |limit| (capture types/modes unresolved)
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	297	40	closure captures: |payload| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	388	40	closure captures: |payload| (capture types/modes unresolved)
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	353	18	closure captures: || (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	444	18	closure captures: || (capture types/modes unresolved)
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	366	26	closure captures: |value| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	457	26	closure captures: |value| (capture types/modes unresolved)
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	392	18	closure captures: || (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	483	18	closure captures: || (capture types/modes unresolved)
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	397	22	closure captures: |limit| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	488	22	closure captures: |limit| (capture types/modes unresolved)
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	399	26	closure captures: |value| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	490	26	closure captures: |value| (capture types/modes unresolved)
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	430	26	closure captures: || (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	521	26	closure captures: || (capture types/modes unresolved)
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	461	40	closure captures: |replaced| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	552	40	closure captures: |replaced| (capture types/modes unresolved)
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/options/builtins.rs	331	26	closure captures: || (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/options/builtins.rs	367	26	closure captures: || (capture types/modes unresolved)
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/options/builtins.rs	349	22	closure captures: || (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/options/builtins.rs	385	22	closure captures: || (capture types/modes unresolved)
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/options/builtins.rs	475	30	closure captures: || (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/options/builtins.rs	511	30	closure captures: || (capture types/modes unresolved)
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/options/builtins.rs	509	30	closure captures: || (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/options/builtins.rs	545	30	closure captures: || (capture types/modes unresolved)
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/options/builtins.rs	538	40	closure captures: |value| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/options/builtins.rs	574	40	closure captures: |value| (capture types/modes unresolved)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/functional_ops/arrays/mod.rs	95	40	closure captures: |current| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/functional_ops/arrays/mod.rs	133	40	closure captures: |current| (capture types/modes unresolved)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	67	40	closure captures: |row| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	96	40	closure captures: |row| (capture types/modes unresolved)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	164	40	closure captures: |value| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	224	40	closure captures: |value| (capture types/modes unresolved)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	226	29	closure captures: || (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	286	29	closure captures: || (capture types/modes unresolved)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	237	14	closure captures: |items| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	297	14	closure captures: |items| (capture types/modes unresolved)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	238	25	closure captures: || (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	298	25	closure captures: || (capture types/modes unresolved)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	281	14	closure captures: |dim| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	341	14	closure captures: |dim| (capture types/modes unresolved)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	331	18	closure captures: |_| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	391	18	closure captures: |_| (capture types/modes unresolved)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	382	22	closure captures: |index| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	442	22	closure captures: |index| (capture types/modes unresolved)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	403	18	closure captures: |items| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	463	18	closure captures: |items| (capture types/modes unresolved)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	415	26	closure captures: |item| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	475	26	closure captures: |item| (capture types/modes unresolved)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	422	26	closure captures: |item| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	482	26	closure captures: |item| (capture types/modes unresolved)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	451	14	closure captures: |_| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	511	14	closure captures: |_| (capture types/modes unresolved)
```

**unknown_macros.tsv: 3102 → 3108; additions 6; position-only moves 20; removals 0.**

```tsv
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	191	9	unexpanded macro crate :: resume_or_suspend! (ends at 201:10)
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	214	9	unexpanded macro crate :: resume_or_suspend! (ends at 224:10)
UnknownConstruct	crates/rum-runtime/src/runtime/options/builtins.rs	294	9	unexpanded macro crate :: resume_or_suspend! (ends at 305:10)
UnknownConstruct	crates/rum-stdlib/src/functional_ops/arrays/mod.rs	77	5	unexpanded macro rum_runtime :: resume_or_suspend! (ends at 86:6)
UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	52	5	unexpanded macro rum_runtime :: resume_or_suspend! (ends at 57:6)
UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	170	5	unexpanded macro rum_runtime :: resume_or_suspend! (ends at 179:6)
```

```tsv
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	154	9	unexpanded macro crate :: resume_or_suspend! (ends at 166:10)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	245	9	unexpanded macro crate :: resume_or_suspend! (ends at 257:10)
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	178	20	unexpanded macro crate :: resume_or_suspend! (ends at 187:10)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	269	20	unexpanded macro crate :: resume_or_suspend! (ends at 278:10)
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	279	13	unexpanded macro vec! (ends at 283:14)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	370	13	unexpanded macro vec! (ends at 374:14)
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	299	17	unexpanded macro panic! (ends at 299:81)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	390	17	unexpanded macro panic! (ends at 390:81)
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	301	13	unexpanded macro assert! (ends at 304:14)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	392	13	unexpanded macro assert! (ends at 395:14)
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	317	13	unexpanded macro debug_assert! (ends at 317:49)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	408	13	unexpanded macro debug_assert! (ends at 408:49)
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/options/builtins.rs	286	28	unexpanded macro crate :: resume_or_suspend! (ends at 297:22)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/options/builtins.rs	322	28	unexpanded macro crate :: resume_or_suspend! (ends at 333:22)
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/options/builtins.rs	340	25	unexpanded macro format! (ends at 340:78)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/options/builtins.rs	376	25	unexpanded macro format! (ends at 376:78)
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/options/builtins.rs	357	17	unexpanded macro format! (ends at 357:89)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/options/builtins.rs	393	17	unexpanded macro format! (ends at 393:89)
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/options/builtins.rs	439	20	unexpanded macro vec! (ends at 443:19)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/options/builtins.rs	475	20	unexpanded macro vec! (ends at 479:19)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/functional_ops/arrays/mod.rs	62	22	unexpanded macro rum_runtime :: resume_or_suspend! (ends at 71:10)
AFTER	UnknownConstruct	crates/rum-stdlib/src/functional_ops/arrays/mod.rs	100	22	unexpanded macro rum_runtime :: resume_or_suspend! (ends at 109:10)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	38	19	unexpanded macro rum_runtime :: resume_or_suspend! (ends at 45:10)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	67	19	unexpanded macro rum_runtime :: resume_or_suspend! (ends at 74:10)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	133	24	unexpanded macro rum_runtime :: resume_or_suspend! (ends at 142:18)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	193	24	unexpanded macro rum_runtime :: resume_or_suspend! (ends at 202:18)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	197	9	unexpanded macro vec! (ends at 203:10)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	257	9	unexpanded macro vec! (ends at 263:10)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	247	9	unexpanded macro format! (ends at 250:10)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	307	9	unexpanded macro format! (ends at 310:10)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	259	9	unexpanded macro format! (ends at 262:10)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	319	9	unexpanded macro format! (ends at 322:10)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	271	9	unexpanded macro format! (ends at 274:10)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	331	9	unexpanded macro format! (ends at 334:10)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	311	21	unexpanded macro vec! (ends at 311:30)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	371	21	unexpanded macro vec! (ends at 371:30)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	369	21	unexpanded macro vec! (ends at 369:60)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	429	21	unexpanded macro vec! (ends at 429:60)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	387	10	unexpanded macro vec! (ends at 387:42)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	447	10	unexpanded macro vec! (ends at 447:42)
```

**unknown_storage.tsv: 88 → 88; additions 0; position-only moves 0; removals 0.**

All 68 changed fingerprint rows, with unchanged identity/status and old/new generated fingerprints (all first 18 TSV columns are identical):

```tsv
line	id	status	before	after
2	B01	Pending	fnv1a64:b60825c0d4ad8bb0	fnv1a64:e847dab3db2c8f18
3	B02	Pending	fnv1a64:f5bcf9719ba2a659	fnv1a64:02fa770079172d65
4	B03	Discharged	fnv1a64:10c54100af192a32	fnv1a64:536c48b12439253e
5	B04	Discharged	fnv1a64:68790df1ee24c998	fnv1a64:a2cd08dbaa4fefc0
6	B05	Discharged	fnv1a64:60cf814c9e09266d	fnv1a64:b16630a2c6210a79
7	B06	Pending	fnv1a64:502664c9643cddac	fnv1a64:8a4fe47bf0331664
8	B07	Pending	fnv1a64:accea6345595dcdd	fnv1a64:03fba078e8e7d549
9	B08	Pending	fnv1a64:c52accb1ffe9f126	fnv1a64:869b403a5487ba12
10	B09	Discharged	fnv1a64:3caa0f57160242d4	fnv1a64:e9bbad261133b22c
11	B10	Discharged	fnv1a64:16bd817dac78d233	fnv1a64:9f268f41407d0a93
12	B11	Discharged	fnv1a64:29d0bf4f0818ef98	fnv1a64:6424ba38c44415c0
13	B12	Discharged	fnv1a64:fbb68f52f20ad7d4	fnv1a64:a8c82d21ed3c472c
14	I01	Discharged	fnv1a64:93ea10be41658230	fnv1a64:c629c5b147e48598
15	I02	Discharged	fnv1a64:b810fee0963fae6d	fnv1a64:08a7ae36be579279
16	I03	Pending	fnv1a64:1477771138785614	fnv1a64:8afdec4d12de58ec
17	I04	Discharged	fnv1a64:de717acf3e818479	fnv1a64:a2dea75aa7dffd05
18	P01	Discharged	fnv1a64:543932c211c8b64c	fnv1a64:0c9b34058a180884
19	P02	Discharged	fnv1a64:26509d0eb1f48903	fnv1a64:b025b60c238f8583
20	P03	Discharged	fnv1a64:db55c84b16a504ca	fnv1a64:5535fa242f311d76
21	P04	Discharged	fnv1a64:ea981787204bc259	fnv1a64:f7d59515fdc04965
22	P05	Discharged	fnv1a64:f8e5e9953ea90780	fnv1a64:a5356c0e6ed10808
23	P06	Discharged	fnv1a64:4c35b4237e3a7154	fnv1a64:f94751f2796be0ac
24	P070	Discharged	fnv1a64:868350655cafeb54	fnv1a64:3394ee3457e15aac
25	P080	Discharged	fnv1a64:e716469d350f3373	fnv1a64:4a30ce5d7f170553
26	P090	Discharged	fnv1a64:5f8713163d3c2199	fnv1a64:e003e2e49ef49125
27	P100	Discharged	fnv1a64:33a6591bc0a94fc8	fnv1a64:250c79c75ce71d90
28	P110	Discharged	fnv1a64:e40035d81e844a0b	fnv1a64:e5dd6284de96406b
29	P120	Discharged	fnv1a64:b831a042d5e6b246	fnv1a64:595714ec677f32b2
30	P130	Discharged	fnv1a64:509bff918449b6dd	fnv1a64:a7c8f9d6179baf49
31	P140	Discharged	fnv1a64:ad6e233c098427d1	fnv1a64:72a6c1dbe6d9bf1d
32	P150	Discharged	fnv1a64:028ef385cd365ab9	fnv1a64:95fe9cd2f5233fc5
33	P071	Discharged	fnv1a64:95fe7ac5cb05a2fd	fnv1a64:3152c9c6b67879e9
34	P081	Discharged	fnv1a64:c2ed9de989bb35f4	fnv1a64:f6221b0eb106564c
35	P091	Discharged	fnv1a64:51550d4fc342448c	fnv1a64:f95896cb8b412944
36	P101	Discharged	fnv1a64:dc0326a553e7e70b	fnv1a64:dde0535213f9dd6b
37	P111	Discharged	fnv1a64:bc05703a2ddb1d04	fnv1a64:8c0340849314457c
38	P121	Discharged	fnv1a64:cd1e18f2aeb116af	fnv1a64:c681b38e370ecaaf
39	P131	Discharged	fnv1a64:062330582478eb18	fnv1a64:40772b41e0a41140
40	P141	Discharged	fnv1a64:ac035c0150b6e2ec	fnv1a64:eeca56f9f23c4b24
41	P151	Discharged	fnv1a64:d04795ad74cdc206	fnv1a64:8dffe1befab74ef2
42	B13	Discharged	fnv1a64:de36f85294d999ac	fnv1a64:1860780520cfd264
43	R01	Discharged	fnv1a64:72be7a3f134dc549	fnv1a64:b40b2659c202a2b5
44	R02	Discharged	fnv1a64:ed2fc76368f2ff0a	fnv1a64:2a66171755ddab36
45	R03	Discharged	fnv1a64:76f9a30d6acc5f65	fnv1a64:08842d6cdd77f811
46	R040	Discharged	fnv1a64:1975f06c44768769	fnv1a64:bdec1f4cce3e7fd5
47	R050	Discharged	fnv1a64:2f062694b1a79a60	fnv1a64:e77180fee7d55fe8
48	R060	Discharged	fnv1a64:77f5ae7226b7f601	fnv1a64:208b8d6b2c66d0ad
49	R070	Discharged	fnv1a64:2b5b66036b5cf4f4	fnv1a64:5e8fe32892a8154c
50	R080	Discharged	fnv1a64:a575e3c20e064d79	fnv1a64:69e3104d7764c605
51	R090	Discharged	fnv1a64:37e2c5b8fd68c708	fnv1a64:9c60bdc7a5f8f650
52	R100	Discharged	fnv1a64:d5eae55358eed4b2	fnv1a64:1891ed03ce0ecfbe
53	R110	Discharged	fnv1a64:3c3829018af8b53b	fnv1a64:16749042075192fb
54	R041	Discharged	fnv1a64:c4705b7cac0158ac	fnv1a64:fe99db2f37f79164
55	R051	Discharged	fnv1a64:eda9412bca6a2e19	fnv1a64:6e2610fa2c229da5
56	R061	Discharged	fnv1a64:2a0b1773c7ea3936	fnv1a64:f74e2ce0ab92d502
57	R071	Discharged	fnv1a64:6ff9a7a982bb1857	fnv1a64:7eb33be3bee52837
58	R081	Discharged	fnv1a64:ef90c2705c5524c8	fnv1a64:e0f6e31bf892f290
59	R091	Discharged	fnv1a64:57221ca13cda9491	fnv1a64:442d70b2f05b025d
60	R101	Discharged	fnv1a64:01b520b0f5aea341	fnv1a64:fd71aee6201a856d
61	R111	Discharged	fnv1a64:f002ab64924cb82e	fnv1a64:a771e9ef13341c1a
62	R12	Discharged	fnv1a64:0412c4569639a2ee	fnv1a64:017f8b2339b1185a
63	R13	Discharged	fnv1a64:5704808f7388b402	fnv1a64:31240b9a02357eae
64	R14	Discharged	fnv1a64:4dfe67919d5a0fc8	fnv1a64:3f64883d3997dd90
65	R15	Discharged	fnv1a64:4937bf2b4cb21cbc	fnv1a64:06f4d76f31366454
66	O010	Discharged	fnv1a64:aac3291ab3077816	fnv1a64:11302f9751a8afe2
67	O0200	Discharged	fnv1a64:a835c510583b4dd1	fnv1a64:6d6e63b03590e51d
68	O011	Discharged	fnv1a64:ba7e6afa3449ba81	fnv1a64:631449f339f8952d
69	O0210	Discharged	fnv1a64:c93ff2d9ca20354e	fnv1a64:75acb080596c5c3a
```

Four changed certificate cells: the complete replayed TSV cells are saved under `.temp/brief-560/verification/final-promoted-hash-{true,false}.tsv`; SHA256 pairs below disclose each changed cell. Every non-fingerprint character and every non-certificate column was checked identical.

```tsv
row	line	column	feature_hash	before_cell_sha256	after_cell_sha256
C01	2	5	true	b1dba4b4ec444b2c256198256ce8acc524f0da122d808b9dcb6f6f7c261b801a	893ed8ef80944d4e7dacfc13ebf2dd79a06ae43f44cbfca3b5ff8f2ff0996e98
C01	2	6	false	e07cb5f71d9e8bf2de3698fae2b5d50ec26fcafc313b2d4da7f5d588501b7583	c804db90b124ac1fc746adb5ad0f221110a009e5e2bbec5810c4548ef0dadb03
C02	3	5	true	5fe6e16d3ae3284d206448232fdc3342f90c0a7797dd354f3ad88746ca391d99	a3ac3804fa22d7c610be7b815cd996dd248f2e77911dad9c76a9a48502df2965
C02	3	6	false	b57f28a9d803497905de408a71fd6286ced7e98aab9b0d66005fcdccc239d898	dc66a4c181f940982961dddff74cb1137887a1ac11c7e59b34a7d140ae4b9315
```

The required test harnesses also emitted their usual generated evidence to `target/brief-*` and `.temp/brief-546/certificates/`; this report’s scripts, probes, snapshots, binaries and logs are under `.temp/brief-560/`. No generated artifact was added to project source.

**Unresolved / did not run:** the all-nine-label performance gate never passed. Differences in the retained candidate remains 68.90320 / 86.83455 user instructions per iteration above the fresh preserved-base samples. No candidate 09, further tuning, post-stop performance probes, timing gate, base rebuild, test modification, git command, network fetch or configuration change was performed. Focused candidate tests were run for 01/04/05/06/08 and the restored final tree; 02/03/07 had release builds and complete performance panels but no separate focused suite. Nine native ZeroMQ tests did not execute their socket paths. All mandated correctness commands did run and ultimately passed; the timeout, fmt repair, certificate-generation failures and runtime bound-provenance limitation are disclosed above.

Handoff: the best partial implementation remains reviewable in the working tree, with every rejected candidate preserved. Further performance work requires a new director decision because this brief’s stop-loss has fired.

---

## Resume — the director attributed the residual; one targeted candidate

The stop-loss was right to fire: after candidate 02 the `Differences` curve sat
at +0.86-0.88% and nothing moved it. The director then measured **where** the
residual lives, with an instrument that counts exactly: `valgrind
--tool=callgrind` on the preserved base and on candidate 06, at N = 20 000 and
40 000, per-iteration = (Ir(40 000) - Ir(20 000)) / 20 000. Calibration: program
totals 9 903.1 (base) and 9 993.9 (candidate) per iteration, +0.92%, matching
`perf`'s +0.88%. Output: `.temp/director/gate-560/cg-*.out` and `ann-*.txt`
(copies under `.temp/brief-560/`).

| Function, exclusive Ir per iteration | Base | Candidate 06 | Delta |
| --- | ---: | ---: | ---: |
| `eval_differences` | 812 | 361 | -451 |
| `difference_expr` | 0 (inlined) | 536 | +536 |
| `Session::cached_expr_stability_with_bloom` | 558.1 | 565.3 | +7.2 |

**The residual is an inlining decision.** In the base, `difference_expr` is
inlined into the one loop that calls it. Candidate 06 gives it a second call
site — the resumed row loop — and the compiler outlines it, adding about 85
instructions per call of `Differences[{1,4,9,16,25}]`. That is also why
`#[inline(always)]` (candidate 03) made things worse: it inlines into both.

**Candidate 09 — the one to try:** give `difference_expr` a single call site
again. The resumed row path must not call `difference_expr` itself; it calls
the **ordinary** row loop `successive_differences` on the remaining slice
(`&items[next_pair..]`, which yields exactly the remaining `windows(2)` pairs)
and prepends the carried prefix to the result. If that inner call suspends
again, the ordinary loop's own cold path builds its own row frame, and the
resumed path pushes a small frame that prepends the carried prefix when the
inner completes — so a second suspension still resumes correctly, as
`loop_continuations.rs` requires. Remove the cursor-driven row loop that this
makes redundant.

Measure candidate 09 on all nine labels exactly as before, both samples, and
confirm with callgrind that `difference_expr` is inlined again (its exclusive
Ir per iteration returns to about 0). If `Differences` reaches the threshold
with no neighbour regression, keep it. If not, report the callgrind table for
it, and stop: do not start another open-ended search. Everything else in the
brief stands — tests unchanged and green, standing duty, serial runs with
`env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1`, every gate. Append your report.
