# brief-632 — the perf-panel layer of the director's gate (grk)

Worker: grk (Grok Build, grok-4.7 xhigh), a new session. Codex has no quota;
grk is the first choice (user ruling 2026-10-29: "好，写 brief-632 派给 grk
吧"). One stage, then STOP and report.

## Goal

When a brief touches a hot path, my gate ends with an instruction A/B:

1. build both arms from one source path with one codegen unit;
2. path-calibrate the arms and diff their functions;
3. run the resume and CLI panels interleaved for 15 reps;
4. attribute every label that sits apart.

The instruments for this live in `.temp/`, scattered over five briefs, with
hard-coded paths. Each gate rebuilds a copy of them. This brief moves them
into the repo as one tool, `scripts/perf-panel.py`, plus its data,
`tools/perf-panel/`. It also adds three opt-in steps to
`scripts/director-gate.py`: `arms`, `structure` and `panels`. Each ported
instrument must give the same output as the one it replaces, on stored
inputs, and the panel and attribution must reproduce known answers on stored
binaries. No production code changes.

## Context pointers

- `AGENTS.md`, the Testing Policy: rum against rum only. The panels compare
  two rum builds; nothing else is measured.
- `docs/dev/codex/TEMPLATE.md`: pitfalls 8 (one codegen unit), 9
  (concurrency), 11 (re-entry safe pipelines); the Report format.
- `scripts/director-gate.py`: its `Gate`, `result`, `run`, `show`,
  `selftest` and `base_frontier`. The new steps follow its patterns.
- **The instruments being ported.** Read them; do not edit them. They are
  the reference outputs.

| Source | What it does |
| --- | --- |
| `.temp/director/gate-630b2/build-arms.sh`, `build-cand.sh` | build both arms (or only the candidate) at one path `same/src`, one codegen unit |
| `.temp/brief-629/instruments/path-calibration.sh` | the Expr drop-glue symbol-hash sets of two binaries must be equal |
| `.temp/director/gate-630b2/fndiff.py`, `fndiff3.py` | functions that differ after address normalization; layout-only vs code |
| `.temp/brief-629/instruments/instr.py` | the label table, the command per label and size, the `perf stat` count with its stdout check |
| `.temp/director/gate-630b2/recheck.py` | the interleaved panel: per rep base N, cand N, base 2N, cand 2N; slope = (median 2N − median N) / N |
| `.temp/director/gate-630b2/panel-summary.py` | median, max, min, labels above +0.1%, and the labels apart in every rep |
| `.temp/director/gate-630b2/attr.py` | callgrind exclusive Ir per function, six processes per arm at N and 2N, slopes |
| `.temp/brief-629/instruments/callers.py`, `instr_slope.py` | per-caller call-count slopes of a callee; per-instruction self-Ir slope of one function |
| `.temp/brief-629/instruments/resume-shape.sh` | one function's size, stack reservation, memcpy sites and direct call sites |
| `.temp/brief-632/harness-630b2/` | the resume harness (`main.rs` plus `s5`–`s9`, `g15`, `lhs`, `g15c1`) |
| `.temp/brief-626/perf/labels.json` and `<label>-<n>.rum` | the 313-label set: 121 resume labels, 192 CLI labels |

- **Stored inputs, read-only.** Prepared by the director:
  - `.temp/brief-632/calib/{b2base,b2v1,b2final,b3}/{rum-cli,resume}`, with
    `SHA256SUMS`. Each pair below was built from one source path, and each
    passes the path calibration.
    - `b2base`: brief-630 B2's base, `9d833875`.
    - `b2v1`: B2's first candidate, before its cost fix.
    - `b2final`: B2 as committed, `1c115117`; also B3's base.
    - `b3`: B3 as committed, `ba1e1e52`.
  - `.temp/brief-632/head-src/`: an export of HEAD (`git archive`), the base
    source for the end-to-end build.
  - The panel TSVs `.temp/director/gate-630b3/logs/recheck-resume.tsv` and
    `.temp/director/gate-630b2/logs/recheck-cli.tsv`.

## Scope

- Files you MAY create or modify:
  - `scripts/perf-panel.py` (new);
  - `tools/perf-panel/**` (new): `harness/` (the resume harness crate, with
    its `Cargo.lock`), `labels.json`, `labels/` (the CLI label files);
  - `scripts/director-gate.py`: the three steps, the `--perf` option, the
    `rule` subcommand, the `head_worktree` helper, the docstring, selftest
    cases;
  - `scripts/README.md`: one entry for `perf-panel.py`, and the
    `director-gate.py` entry's step list.
- MUST NOT touch:
  - `crates/**`. Nothing here changes rum. The harness is not a workspace
    member, so no retention fingerprint or inventory moves; if one does,
    STOP.
  - `Cargo.toml`, `Cargo.lock` at the root, `docs/**`, `TODO.md`,
    `CLAUDE.md`, `AGENTS.md`;
  - anything under `.temp/director/`, `.temp/brief-608/`, `.temp/brief-626/`,
    `.temp/brief-629/`, or `.temp/brief-632/calib/`. They are references; copy
    from them.
- Your scratch goes in `.temp/brief-632/` (work dirs, logs, PROGRESS.md, the
  report).
- **Disclosure duty.** Some reference parts may not be ported: a label file
  no command reads, a script nothing calls. List each one in `disclosures`
  with the reason. Any departure from the shape below is also a disclosure,
  and a change to a decision rule is a STOP.

## The shape

### Files

```
tools/perf-panel/
  harness/Cargo.toml       # package rum-resume-harness, own [workspace], bin "resume"
  harness/Cargo.lock       # from harness-630b2; resolves --offline
  harness/src/*.rs         # harness-630b2's sources; rum-host = { path = "../../../crates/rum-host" }
  labels.json              # the 313 labels, only the fields that decide a command (below)
  labels/<label>-<n>.rum   # the CLI label files at N and 2N, byte-identical to brief-626's
scripts/perf-panel.py      # one file, subcommands below, stdlib only
```

`labels.json` is a list, in brief-626's order, of:

```json
{"label": "resume-table", "engine": "resume", "N": 1000, "harness_label": "table", "args": [], "resumes": 1}
{"label": "lazy-range-take-small", "engine": "cli", "N": 20000, "kind": "repeat"}
```

- `args` holds the harness's extra argv. Today that is `["cand"]` for the
  `lhs-` labels, as `instr.command` adds.
- `resumes` is the resolved count per iteration:
  `resumes_per_iteration`, else `TWO_PROMPT`, else 1.
- The expectation is derived exactly as in `instr.command`:
  - a resume label wants `PASS <harness_label> resumes=<n × resumes>`;
  - a CLI `repeat` label wants stdout `Null`;
  - any other CLI kind has no stdout check, and its rc must be 0 (the files
    `Exit[2]` on a wrong value).

### `scripts/perf-panel.py`

```
perf-panel.py arms --work DIR --base-src SRC --cand-src SRC [--only cand] [--no-scope] --json OUT
perf-panel.py pathcal BIN_A BIN_B
perf-panel.py structure ARM_A ARM_B [--json OUT]
perf-panel.py panel ARM_A ARM_B --engine resume|cli [--labels a,b | --all] [--reps 15] --out TSV
perf-panel.py summary TSV [--top 25] [--json OUT]
perf-panel.py attr LABEL ARM_A ARM_B --out DIR [--runs 6]
perf-panel.py callers DIR NEEDLE N
perf-panel.py fnslope DIR FUNCTION N
perf-panel.py shape BIN FUNCTION [--callee PATTERN ...]
perf-panel.py selftest
```

An ARM is a directory holding `rum-cli` and `resume`.

- **`arms`** builds both arms at the one path `DIR/same/src`. For each arm,
  in order (base, then candidate; `--only cand` skips the base):
  1. copy the arm's source into `DIR/same/src` with
     `rsync -rlpD --checksum --delete`, excluding `/target/`, `/.temp/` and
     `/.git`;
  2. copy the candidate's `tools/perf-panel/harness/` over it, so both arms
     use one harness: the harness is the instrument, `crates/` is the
     subject;
  3. check with `diff -rq` that `DIR/same/src/crates` equals the arm's
     `crates`;
  4. build `cargo build --release -p rum-cli --offline` with
     `CARGO_TARGET_DIR=DIR/same/target-<arm>`, and the harness with
     `--manifest-path DIR/same/src/tools/perf-panel/harness/Cargo.toml` and
     `CARGO_TARGET_DIR=DIR/same/target-<arm>-harness`. Both run with
     `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1` and `CARGO_BUILD_JOBS=8`;
  5. copy the binaries to `DIR/arms/<arm>/{rum-cli,resume}`.

  - **The rsync leaves out `-t`, on purpose.** A file the copy changes gets
    the current mtime, so cargo rebuilds it. A file it leaves alone keeps its
    mtime, so nothing rebuilds needlessly. Copying the source's own mtime
    (`-a`) can make cargo skip a changed file when that mtime is older than
    the target's outputs.
  - Without `--no-scope`, each cargo runs as `scripts/memory-scope.sh
    <unit> 14G ...`. You run with `--no-scope`, because the dispatcher's
    scope already holds you and your sandbox cannot make one.
  - The `--json` receipt records, per arm: the source path, source-equal,
    each build's rc and seconds, and each binary's sha256. It also records
    `pathcal` for both binaries. Logs go to `DIR/logs/`.
- **`pathcal`** reimplements `path-calibration.sh` exactly: the set of
  `drop_in_place$LT$rum_core..expr..Expr$GT$17h<hash>` hashes from `nm`;
  PASS iff non-empty and equal.
- **`structure`** is `fndiff.py` and `fndiff3.py` in one pass, per binary.
  It reports `functions_a`, `functions_b`, `differ`, `layout_only`,
  `code` (name and instruction counts per instance), `only_a` and `only_b`
  (name and count). It keeps the reference normalizations exactly: absolute
  addresses, rip-relative offsets, symbol hashes, and hex constants for the
  layout-only split.
- **`panel`** is `recheck.py`.
  - One rep runs base N, cand N, base 2N, cand 2N for each label, under
    `taskset -c 2 perf stat -x, -e cpu_core/instructions/u`, with stdin
    `/dev/null` and `RUM_PROFILE_*` stripped from the environment. Every
    run's rc and stdout are checked as `instr.ok` checks them.
  - The TSV has a header line. It carries the sha256 of both arms' binary
    for the engine, the engine and the rep count. Each row holds `label, N,
    a_slope, b_slope, delta, delta_pct, a_runs_N, a_runs_2N, b_runs_N,
    b_runs_2N, secs`, in recheck's number formats, plus `secs`.
  - **Re-entry.** Rerunning on an existing TSV skips the labels it holds,
    but only if the header matches: same arms sha256, engine and reps.
    Otherwise it moves the old file aside to `<TSV>.<mtime>` and starts over.
    It never resumes a panel measured on other binaries (pitfall 11).
- **`summary`** is `panel-summary.py`, with one change: it reads the new TSV
  (header and `secs`) as well as the old headerless one.
  - It prints the median, max and min of `delta_pct` and the count above
    +0.1%.
  - It reports **apart above** and **apart below**. A rep's slope is (count
    2N − count N) / N, taken within that rep. A label is apart above when
    the candidate's minimum rep slope exceeds the base's maximum, and apart
    below when the candidate's maximum is under the base's minimum.
  - It lists the top apart labels and the total seconds.
- **`attr`** is `attr.py`, with the arms and the label as arguments. Copy
  `~/.cache/debuginfod_client` into `DIR/debuginfod` once if missing. Set
  `DEBUGINFOD_CACHE_PATH` to it, `DEBUGINFOD_URLS=http://127.0.0.1:9` and
  `DEBUGINFOD_TIMEOUT=1` (see attr.py: the sandbox cannot open the real
  cache for writing). It keeps its per-label cache of callgrind outputs in
  `DIR/<label>/`. If the arms' sha256 differs from the cached outputs', it
  moves that cache aside and remeasures (memory: attr caches per label).
- **`callers`, `fnslope`** are `callers.py` and `instr_slope.py` over
  `attr`'s output directory.
- **`shape`** is `resume-shape.sh` for any function name. It prints size,
  frame, memcpy_sites, and one `calls[<pattern>]` per `--callee`.
- **`selftest`** covers the pure parts, each with a case:
  - the slope arithmetic;
  - the apart rule, both directions, including a tie (equal is not apart);
  - the TSV header round trip, and re-entry refusing a different sha256;
  - the label command and expectation for one label of each engine and
    kind, including an `lhs-` and a two-prompt label;
  - the disassembly normalization and the layout-only/code split, on
    synthetic objdump text;
  - the pathcal hash extraction, on synthetic `nm` text.

### `scripts/director-gate.py`

New steps after `workspace`, opt-in:

```python
STEPS = [..., "workspace", "arms", "structure", "panels"]
PERF_WORK = ROOT / ".temp/director/perf"   # one fixed path for every gate's arms


def head_worktree(rev):
    """The worktree base_frontier uses, checked out at `rev` (factored out of it)."""
    wt = STATE / "worktree"
    if wt.exists():
        git("checkout", "--detach", "--force", rev, cwd=wt)
    else:
        STATE.mkdir(parents=True, exist_ok=True)
        git("worktree", "add", "--detach", str(wt), rev)
    return wt


def step_arms(g, a):        # SKIP without --perf; else perf-panel.py arms, base = head_worktree("HEAD"), cand = ROOT
def step_structure(g, a):   # SKIP without --perf; perf-panel.py structure on PERF_WORK/arms/{base,cand}
def step_panels(g, a):      # SKIP without --perf; one panel per engine named in --perf, into g.dir/panel-<engine>.tsv
```

- `run` gains `--perf resume,cli`, a comma list of engines. Without it, the
  three steps are SKIP "not requested".
  - `base_frontier` calls `head_worktree`.
  - Each step runs `perf-panel.py` as a subprocess, logs to `g.log(...)`,
    and reads its `--json`.
- **Verdicts.**
  - `arms` is PASS iff both builds have rc 0, both copies are source-equal,
    and pathcal passes for both binaries. A FAIL stops the run.
  - `structure` is PASS, informational, with this detail per binary: `differ
    D (layout L, code C), new N, gone G`. Its notes are the CODE and ONLY
    lines.
  - `panels` is FAIL on any failed run; FLAGGED iff any label sits apart
    above in any panel; otherwise PASS. Its detail per engine is `labels,
    median, max, apart above/below`. Its notes are the apart rows. A label
    apart below is reported, not flagged.
- **`rule`.** `director-gate.py rule NNN [--stage S] STEP --why TEXT` turns
  a FLAGGED step into `RULED`. It keeps the step's detail and adds the note
  `RULED: <why>`. It refuses a step that is not FLAGGED. A rerun of the step
  replaces the ruling. GREEN counts RULED as passing. The panels need this:
  a label apart because of a layout effect is ruled after attribution, as at
  B2.
- The docstring's last paragraph names the new steps. Steps run serially, so
  the panels run alone after the workspace.
- `selftest` gains cases for `rule` and for the three steps' detail strings,
  built from canned JSON. You cannot run git in your sandbox. `head_worktree`
  is given above as code; I test it at my gate.

## Calibration (each against a known answer; record every result)

Run them in this order. Stop at the first one that misses its answer, and
report what you saw.

- **C0, the preflight.**
  - `taskset -c 2 perf stat -x, -e cpu_core/instructions/u true` prints a
    count.
  - `nm`, `objdump`, `valgrind`, `rsync` and `taskset` run.
  - `sha256sum -c` passes in `.temp/brief-632/calib`.

  If perf cannot count in your sandbox, STOP. The director then runs the
  panels.
- **C1, equivalence on stored inputs.** Each new subcommand's output equals
  its predecessor's, run by you on the same inputs. Save both outputs and
  `diff` them.
  - `pathcal` on the three pairs (b2base, b2v1), (b2base, b2final) and
    (b2final, b3), for both binaries: six PASS lines, with hash sets equal
    to `path-calibration.sh`'s.
  - `structure` on (b2final, b3), for both binaries:

    | binary | differ | layout_only | code | only_a | only_b |
    | --- | --- | --- | --- | --- | --- |
    | resume | 3 | 0 | 3 | 0 | 1 |
    | rum-cli | 3 | 0 | 3 | 0 | 1 |

    - The CODE rows are `abandon_frames_profiled` [502]→[520],
      `drain_abandonment_worklist` [1217]→[1236] and `mi_bin` [23]→[25].
    - The only_b row is `report_abandoned_cleanup_error` (44).
    - Everything else matches `fndiff.py` and `fndiff3.py` line for line.
  - `summary` on the two stored TSVs equals `panel-summary.py`'s output.
    For the B3 resume TSV that is: median +0.0031%, max +0.1642%, min
    −0.0967%, 8 labels above +0.1%, apart above 0 and below 0.
  - **The labels.** Check all 313 labels at N and 2N. The command (argv
    after the binary, with the label directory normalized) and the
    expectation equal `instr.command`'s. Import instr.py from its directory
    to get them. Every CLI command's file exists in `tools/perf-panel/labels/`
    and is byte-identical to brief-626's copy.
  - `shape` equals `resume-shape.sh` on `calib/b2final/resume` and
    `calib/b3/resume` for `resume_eval_request_from_checkpoint`, with
    `--callee poll_resumed_native_stack --callee
    'Session.*native_stack_budget_exhausted'`.
- **C2, the null.** `panel` resume, `--all`, 15 reps, with `calib/b3` as both
  arms. The answer: apart above 0, apart below 0, |median| ≤ 0.05%.
- **C3, B3's answer.** `panel` resume, `--all`, 15 reps, `calib/b2final`
  against `calib/b3`. The answer: apart above 0, apart below 0, |median| ≤
  0.05%. The record is +0.0031%; see the stored TSV.
- **C4, the positive control.** `panel` resume, `--all`, 15 reps,
  `calib/b2base` against `calib/b2v1`. My run on these binaries (old
  instruments, 79 s) gave a median of +2.6301% and 120 of 121 apart above,
  with `resume-cleanup` at +1423.2. The answer:
  - apart above ≥ 115;
  - apart below 0;
  - median in [+2.3%, +3.0%];
  - `resume-cleanup` in [+1350, +1500].
- **C5, CLI deltas.** `panel` cli, 15 reps, `calib/b2base` against
  `calib/b2final`, on seven labels. My 3-run probe, and brief-630's callgrind
  record, gave these. The answer: each delta within max(3, 10%) of my probe,
  with the same sign.

  | label | probe | record |
  | --- | --- | --- |
  | `table-iterator-pair-list` | −7.0 | −6.99 |
  | `take-multispec-small` | −24.0 | −23.88 |
  | `lazy-depth-drop-all-one-8` | +31.4 | +32.67 |
  | `lazy-depth-take-map-64` | +236.4 | +265.27 |
  | `upvalue-guarded-hit` | +4.9 | +4.87 |
  | `replacepart-wide-rule-list` | +1677.0 | +1673.57 |
  | `replaceall-leaf-rebuild-small` | −884.0 | −881.20 |
- **C6, attribution.** `attr`, `calib/b2base` against `calib/b2final`. The
  answers are brief-630's record:
  - `lazy-depth-drop-all-one-8`: the `do_reserve_and_handle` row is +32
    (±1);
  - `replaceall-leaf-rebuild-small`: the memcmp row
    (`__memcmp_avx2_movbe`) is −884 (±10).

  If libc stays unsymbolized (memcmp shows as an address), the debuginfod
  copy failed. Fix that; it is not a reason to stop.
- **C7, the end-to-end build.** `arms --work .temp/brief-632/perf
  --base-src .temp/brief-632/head-src --cand-src <repo> --no-scope`.
  - The answer: both builds rc 0, both source-equal, pathcal PASS for both
    binaries.
  - The crates are identical, so report whether the binaries are
    byte-identical. If they are not, `structure` must show differ 0, only_a
    0 and only_b 0.
- **C8, the rebuild falsifier.** Make `.temp/brief-632/mutant-src` with `cp
  -a` of `head-src`. Insert `std::hint::black_box(0x5a5a_u32);` as the first
  statement of `report_abandoned_cleanup_error` in
  `crates/rum-runtime/src/runtime/suspension/core/abandon.rs`. Then:
  1. `arms --only cand --cand-src .temp/brief-632/mutant-src`. The answer:
     `structure` names `report_abandoned_cleanup_error`, or a function that
     inlines it, in CODE or ONLY.
  2. `arms --only cand --cand-src <repo>` again. The answer: both candidate
     binaries are sha256-equal to C7's. That proves the switch back rebuilt.
- **C9, the end-to-end panels.** On C7's arms:
  - `panel` resume `--all`, 15 reps: apart above 0, apart below 0;
  - `panel` cli `--all`, `--reps 1`: every one of the 192 labels runs at N
    and 2N on both arms, with its stdout and rc checked. Report the total
    seconds and the five slowest labels.

## Verification

- `python3 -I scripts/perf-panel.py selftest` and `python3 -I
  scripts/director-gate.py selftest` PASS.
- **Mutants.** Write `.temp/brief-632/mutants.py`, modelled on
  `.temp/director/gate-mutants.py`. Each mutant of `perf-panel.py` must turn
  a named selftest case red. At least these:
  - the apart rule with `>=` for `>`;
  - the slope without the division by N;
  - re-entry ignoring the sha256;
  - the layout split without hex masking;
  - the `lhs-` args dropped;
  - pathcal comparing counts instead of sets.

  Report each as CAUGHT or MISSED. A MISSED mutant needs a new selftest case.
- `cargo fmt --check --manifest-path tools/perf-panel/harness/Cargo.toml`
  passes.
- C0–C9 as above, each with its answer and the verbatim line that shows it.
- No workspace run: nothing under `crates/` changes. `diff -rq crates
  .temp/brief-632/head-src/crates` prints nothing; paste its exit status. If
  anything under `crates/` differs, that is a STOP.

## Concurrency plan

- Everything runs serially. One cargo build at a time.
- `perf stat`, the panels and callgrind run alone, never beside a build.
- Order: C0 and C1, which are cheap and need no build; then C2–C6 on the
  stored binaries; then the C7 and C8 builds; then C9.
- **Bounds.** Use `scripts/run-bounded.py` and these bounds, set from data:
  - a resume panel with 15 reps took 79 s: bound 1800 s;
  - one CLI rep over all labels is about 290 s (the 15-rep panel took
    4377 s, 3051 s of it on `lazy-depth-fold-through-source-256`): bound
    1800 s;
  - each build: bound 3600 s;
  - `attr` on a small label: bound 1800 s.

  If a bound fires, report it with the log tail. Do not raise it and
  rerun.

## Hard rules

- `TEMPLATE.md`'s hard rules apply: no git (your sandbox denies `.git`), no
  file outside Scope, no network.
- Do not edit the reference instruments or the stored inputs. If a
  reference disagrees with its own known answer, report it.
- A decision rule is fixed by this brief: the slope, the apart rule, the
  stdout checks, the verdicts. If one looks wrong to you, STOP and put it in
  `questions`.

## Report

The `rum-report/1` file `.temp/brief-632/report.json` (TEMPLATE.md, Report
format). Sections, with these tags:

- `shape`: the files and subcommands as built;
- `equivalence`: C1;
- `calibration`: C2–C9, each with its answer and the verbatim line;
- `mutants`;
- `disclosures`: what was not ported, and why.

Your final message is `python3 scripts/report.py summary 632`.

## Worker report (grk, DONE)

scripts/perf-panel.py and tools/perf-panel/ are the rum-vs-rum panel, and director-gate.py adds opt-in arms, structure, and panels plus rule. C1 matched the reference instruments on stored inputs. C2 and C3 were apart 0/0 (medians -0.0021% and +0.0021%). C4 was median +2.6552%, 120 apart above, below 0, resume-cleanup +1408.65. C5 matched the seven probes. C6 attributed do_reserve_and_handle +32.0000 and __memcmp_avx2_movbe -884.2220. C7 builds were rc 0, source-equal, pathcal PASS, about 51s each; shas differ and structure is differ 0, only_a 0, only_b 0. C8 named report_abandoned_cleanup_error [44] to [47], and the restored candidate shas match C7. C9 resume was apart 0/0; CLI ran all 192 labels in 288.013s. Six mutants CAUGHT. crates diff exit 0.

| File | Kind | Lines | Why |
| --- | --- | --- | --- |
| `scripts/perf-panel.py` | added | 1-1122 | one stdlib tool for arms, pathcal, structure, panel, summary, attr, callers, fnslope, shape |
| `scripts/director-gate.py` | modified | 1-50, 67-79, 295-306, 698-846, 922-937, 1019-1092 | opt-in arms, structure, panels, --perf, rule, head_worktree, and RULED counted as passing |
| `scripts/README.md` | modified | 48-56 | director-gate step list names --perf, arms, structure, panels, and rule; one perf-panel entry |
| `tools/perf-panel/harness/Cargo.toml` | added | 1-17 | package rum-resume-harness, its own workspace, bin resume, rum-host path into crates |
| `tools/perf-panel/harness/Cargo.lock` | added | 1-342 | harness-630b2 lock; package name brief608-resume renamed to rum-resume-harness for --offline |
| `tools/perf-panel/labels.json` | added | 1-2142 | 313 labels in brief-626 order, only the fields that decide a command |
| `tools/perf-panel/harness/src/g15.rs` | added | 1-55 | resume harness source copied from harness-630b2, then rustfmt |
| `tools/perf-panel/harness/src/g15c1.rs` | added | 1-55 | resume harness source copied from harness-630b2, then rustfmt |
| `tools/perf-panel/harness/src/lhs.rs` | added | 1-167 | resume harness source copied from harness-630b2, then rustfmt |
| `tools/perf-panel/harness/src/main.rs` | added | 1-172 | resume harness source copied from harness-630b2, then rustfmt |
| `tools/perf-panel/harness/src/s5.rs` | added | 1-126 | resume harness source copied from harness-630b2, then rustfmt |
| `tools/perf-panel/harness/src/s6.rs` | added | 1-37 | resume harness source copied from harness-630b2, then rustfmt |
| `tools/perf-panel/harness/src/s7.rs` | added | 1-79 | resume harness source copied from harness-630b2, then rustfmt |
| `tools/perf-panel/harness/src/s8.rs` | added | 1-121 | resume harness source copied from harness-630b2, then rustfmt |
| `tools/perf-panel/harness/src/s9.rs` | added | 1-169 | resume harness source copied from harness-630b2, then rustfmt |
| `tools/perf-panel/labels/*.rum` (384 files) | added | 1-3 each | CLI label files at N and 2N, byte copies of brief-626's |

- Disclosed: `.temp/brief-629/instruments/lp01_fingerprint.py`: not in the port table; no panel subcommand calls it
- Disclosed: `.temp/brief-629/instruments/attr.py`: older attr. The port is .temp/director/gate-630b2/attr.py
- Disclosed: `.temp/director/gate-630b2/fndiff2.py`: nothing calls it. structure is fndiff.py plus fndiff3.py's hex mask in one objdump pass
- Disclosed: `.temp/brief-626/perf/`: 216 files no panel command reads were not copied: 202 *-check.rum files plus json, tsv, csv, and notes. The 384 N and 2N .rum files were copied byte for byte
- Disclosed: `scripts/perf-panel.py`: shape prints calls[<pattern>] as the brief specifies. resume-shape.sh prints poll_calls and leaf_calls. The numbers matched on b2final and b3
- Disclosed: `scripts/perf-panel.py`: fnslope prints instr_slope.py's lines and prefixes each arm/rep pair with # arm=<arm> rep=<rep>, because attr stores several pairs in one directory
- Disclosed: `tools/perf-panel/harness/Cargo.lock`: one-line package rename, brief608-resume to rum-resume-harness, so the copied lock resolves --offline under the new package name
- Disclosed: `tools/perf-panel/harness/src/`: rustfmt only. cargo fmt --check is clean. No harness logic change
- Disclosed: `.temp/brief-632/mutant-src/tools/perf-panel/harness`: head-src has no harness. arms overlays the candidate's harness, so the C8 copy of head-src also received tools/perf-panel/harness. The black_box edit is only in that copy of abandon.rs
- Disclosed: `scripts/perf-panel.py`: attr passes an absolute DEBUGINFOD_CACHE_PATH. A relative path is ignored and libc locals stay addresses. The reference path is absolute

| Run | Try | Exit | Tests | Log |
| --- | ---: | ---: | --- | --- |
| c0 | 1 | 0 | - | `.temp/brief-632/logs/c0.log` |
| selftest-perf | 1 | 0 | - | `.temp/brief-632/logs/selftest-perf.log` |
| selftest-gate | 1 | 0 | - | `.temp/brief-632/logs/selftest-gate.log` |
| fmt | 1 | 0 | - | `.temp/brief-632/logs/fmt.log` |
| mutants | 1 | 0 | - | `.temp/brief-632/logs/mutants.log` |
| c1 | 1 | 0 | - | `.temp/brief-632/logs/c1-run.log` |
| c2 | 1 | 0 | - | `.temp/brief-632/logs/c2-summary.log` |
| c3 | 1 | 0 | - | `.temp/brief-632/logs/c3-summary.log` |
| c4 | 1 | 0 | - | `.temp/brief-632/logs/c4-summary.log` |
| c5 | 1 | 0 | - | `.temp/brief-632/logs/c5-panel.log` |
| c6-drop | 1 | 0 | - | `.temp/brief-632/attr/attr-lazy-depth-drop-all-one-8.tsv` |
| c6-memcmp | 2 | 0 | - | `.temp/brief-632/attr/attr-replaceall-leaf-rebuild-small.tsv` |
| c6-relative | 1 | 0 | - | `.temp/brief-632/logs/c6-sym2.out` |
| c7 | 1 | 0 | - | `.temp/brief-632/logs/c7-arms.log` |
| c7-structure | 1 | 0 | - | `.temp/brief-632/logs/c7-structure.log` |
| c8-mutant | 1 | 0 | - | `.temp/brief-632/logs/c8-mutant-arms.json` |
| c8-structure | 1 | 0 | - | `.temp/brief-632/logs/c8-structure.log` |
| c8-restore | 1 | 0 | - | `.temp/brief-632/logs/c8-restore-arms.json` |
| c9-resume | 1 | 0 | - | `.temp/brief-632/logs/c9-resume-summary.log` |
| c9-cli | 1 | 0 | - | `.temp/brief-632/logs/c9-cli-summary.log` |
| c9-coverage | 1 | 0 | - | `.temp/brief-632/logs/c9-cli-coverage.log` |
| crates-diff | 1 | 0 | - | `.temp/brief-632/logs/crates-diff.log` |

| Check | Bar | Result | Verdict | Note |
| --- | --- | --- | --- | --- |
| c0 | perf prints a count and sha256sum -c passes | 180925 instructions, sha_rc=0 | PASS |  |
| c1 | ported output equals the reference on stored inputs | C1 PASS | PASS |  |
| c2 | apart above 0, apart below 0, \|median\| <= 0.05% | 0/0, median -0.0021% | PASS |  |
| c3 | apart above 0, apart below 0, \|median\| <= 0.05% | 0/0, median +0.0021% | PASS |  |
| c4 | apart above >= 115, below 0, median in [+2.3%, +3.0%], resume-cleanup delta in [+1350, +1500] | 120 / 0 / +2.6552% / +1408.7 per iter | PASS |  |
| c5 | each delta within max(3, 10% of \|probe\|) and the same sign | -6.94, -24.03, +31.63, +242.59, +3.54, +1620.15, -884.53 | PASS |  |
| c6 | do_reserve_and_handle +32 ±1; __memcmp_avx2_movbe -884 ±10 | +32.0000 and -884.2220 | PASS |  |
| c7 | both builds rc 0, source-equal, pathcal PASS; if shas differ, structure differ 0, only_a 0, only_b 0 | pathcal PASS; STRUCTURE differ=0 only_a=0 only_b=0 on both binaries | PASS |  |
| c8 | structure names report_abandoned_cleanup_error or an inliner; switch-back shas equal C7 | CODE [44] -> [47]; shas match C7 | PASS |  |
| c9-resume | apart above 0 and apart below 0 | 0/0 | PASS |  |
| c9-cli | all 192 labels run at N and 2N on both arms, stdout and rc checked | 192 rows, bad 0, total seconds 288.013 | PASS |  |
| crates | diff -rq crates .temp/brief-632/head-src/crates prints nothing | no diff output, CRATES_DIFF_EXIT:0 | PASS |  |
| selftest | both selftests PASS and six mutants CAUGHT | selftest: PASS; six CAUGHT | PASS |  |

| Measurement | Bar | Result | Verdict | Note |
| --- | --- | --- | --- | --- |
| c2-null | b3 vs b3 resume, 15 reps, apart 0/0 and \|median\| <= 0.05% | median -0.0021%, apart above 0 below 0, 78.893s | PASS |  |
| c3-b3 | b2final vs b3 resume, 15 reps, apart 0/0 and \|median\| <= 0.05% | median +0.0021%, apart above 0 below 0, 80.295s | PASS |  |
| c4-positive | b2base vs b2v1, apart above >= 115, below 0, median +2.3% to +3.0%, cleanup +1350 to +1500 | median +2.6552%, apart 120/0, resume-cleanup +1408.7/iter | PASS |  |
| c7-builds | each build under 3600s, rc 0 | base cli 50.756s, base resume 50.259s, cand cli 50.747s, cand resume 50.926s | PASS |  |
| c9-cli-time | one CLI rep over all labels, bound 1800s | 288.013s; slowest lazy-depth-fold-through-source-256 205.436s | PASS |  |

Deviations:
- attr resolves DEBUGINFOD_CACHE_PATH to an absolute path: The debuginfod client ignores a relative cache path. The first replaceall attr then left __memcmp_avx2_movbe as libc.so.6!0x00000000001b1220 (delta -883.813). That run's log was overwritten by the remeasure. The reproduction is .temp/brief-632/logs/c6-sym2.out. The reference attr passes an absolute path.
- C8 mutant-src includes a copy of tools/perf-panel/harness: arms reads the harness from the candidate source and overlays it. head-src, the cp -a source, has no harness.

### Files and subcommands

`scripts/perf-panel.py` is one stdlib file. Subcommands: arms, pathcal, structure, panel, summary, attr, callers, fnslope, shape, selftest.
`tools/perf-panel/harness` is package `rum-resume-harness`, its own `[workspace]`, bin `resume`, `rum-host` at `../../../crates/rum-host`. It is not a root workspace member.
`tools/perf-panel/labels.json` has 313 labels in brief-626 order. `tools/perf-panel/labels/` has the 384 CLI files at N and 2N.
`scripts/director-gate.py` appends `arms`, `structure`, `panels` after `workspace`. `PERF_WORK` is `.temp/director/perf`. Without `--perf` the three steps are SKIP `not requested`. `step_arms` uses `head_worktree("HEAD")` as the base and the repo as the candidate, and it does not pass `--no-scope`. This worker's own arms runs passed `--no-scope` because the sandbox cannot make a scope.
`rule` turns a FLAGGED step into RULED, keeps the detail, appends `RULED: <why>`, and refuses a step that is not FLAGGED. A rerun replaces the ruling. GREEN counts RULED.
`panels` is FAIL on a failed run, FLAGGED when any label is apart above, and PASS otherwise. Apart below is a note, not a flag. `panels` is not in the ruling set, so FLAGGED does not stop the run.

### C1

`.temp/brief-632/c1.py` stopped at the first miss. It printed `C1 PASS`.
pathcal on (b2base, b2v1), (b2base, b2final), and (b2final, b3), both binaries: six `PASS` lines, stdout equal to `path-calibration.sh`.
structure on (b2final, b3): fndiff and fndiff3 blocks matched the reference scripts. Both binaries: differ 3, layout_only 0, code 3, only_a 0, only_b 1. CODE rows: `abandon_frames_profiled` [502]->[520], `drain_abandonment_worklist` [1217]->[1236], `mi_bin` [23]->[25]. only_b: `report_abandoned_cleanup_error` count 44.
summary on the two stored TSVs matched `panel-summary.py` byte for byte. B3 resume: `labels 121 median +0.0031% max +0.1642% min -0.0967% above +0.1%: 8` and `cand above base in every rep: 0  below in every rep: 0`.
`PASS labels 313 commands and 384 files`.
shape on `resume_eval_request_from_checkpoint` for b2final and b3: size=25829, frame=0x858, memcpy_sites=19, calls[poll_resumed_native_stack]=0, calls[Session.*native_stack_budget_exhausted]=8. Those numbers equal resume-shape.sh. The key names are the brief's `calls[<pattern>]`.

### C2 to C9

No bound fired. Resume panels were about 80s (bound 1800). The CLI rep was 288s (bound 1800). Each cargo build was about 48-51s (bound 3600 per build; the two-arm C7 wrapper was 7200).
C2, b3 vs b3, resume, 15 reps: `labels 121 median -0.0021% max +0.2010% min -0.1295% above +0.1%: 4` and `cand above base in every rep: 0  below in every rep: 0`.
C3, b2final vs b3, resume, 15 reps: `labels 121 median +0.0021% max +0.3240% min -0.3259% above +0.1%: 12` and apart 0/0. The stored record is +0.0031%.
C4, b2base vs b2v1, resume, 15 reps: `labels 121 median +2.6552% max +7.5507% min +0.6994% above +0.1%: 121`, `cand above base in every rep: 120  below in every rep: 0`, `resume-cleanup +1408.7/iter +7.5507%`. The TSV delta is +1408.65, inside [+1350, +1500].
C5 deltas (probe, tolerance max(3, 10% of |probe|)): table-iterator-pair-list -6.94 (-7.0); take-multispec-small -24.03 (-24.0); lazy-depth-drop-all-one-8 +31.63 (+31.4); lazy-depth-take-map-64 +242.59 (+236.4, tol 23.64); upvalue-guarded-hit +3.54 (+4.9); replacepart-wide-rule-list +1620.15 (+1677.0, tol 167.7); replaceall-leaf-rebuild-small -884.53 (-884.0). Same sign, inside tolerance.
C6: `+32.0000` on `rum-cli!alloc::raw_vec::RawVecInner<A>::reserve::do_reserve_and_handle`. `-884.2220` on `libc.so.6!__memcmp_avx2_movbe`. debuginfod was copied from `~/.cache/debuginfod_client` (build-id e363db7c..., 11947584 bytes).
C7: both arms source-equal, pathcal PASS for resume and rum-cli. Build seconds: base cli 50.756, base resume 50.259, cand cli 50.747, cand resume 50.926. Shas differ. Structure: `STRUCTURE resume differ=0 layout_only=0 code=0 only_a=0 only_b=0` and the same line for rum-cli.
C8: black_box is the first statement of `report_abandoned_cleanup_error` in the mutant copy only. Cand rebuild rc 0 (47.978s and 48.787s). Both binaries name the function: `CODE [44] -> [47] ...report_abandoned_cleanup_error`. Resume also differed in `_mi_bitmap_forall_setc_ranges` [118]->[114] and `mi_bin` [23]->[25]; rum-cli also in `mi_bin` [25]->[23]. The switch-back shas equal C7: resume `0df7e0dd027c0edb4df4ede9f3132e3da11758340d933a6680704f897d0e6421`, rum-cli `98549ed1d79538843c5b08202c83f810373f062200c3cc8dfeea8cb76e5f3912`.
C9 resume on the restored C7 arms: `labels 121 median +0.0067% max +0.1468% min -0.2150% above +0.1%: 6` and `cand above base in every rep: 0  below in every rep: 0`.
C9 CLI, `--reps 1`: `cli labels 192 rows_with_one_count_per_arm_size 192 bad 0`. The panel exited 0, so every stdout and rc check passed. `total seconds: 288.013`. Five slowest: `205.436 lazy-depth-fold-through-source-256`, `13.463 lazy-depth-fold-through-source-64`, `11.554 lazy-depth-map-eight-256`, `6.456 lazy-select-scan-small`, `4.836 deletecases-wide-association-no-match`. At one rep the apart line is `cand above base in every rep: 90  below in every rep: 102`. That is one sample per label on binaries that are not byte-identical. The brief's bar for this run is that all 192 labels run.
`diff -rq crates .temp/brief-632/head-src/crates` printed nothing. Exit status: 0 (`CRATES_DIFF_EXIT:0`).

### Selftest mutants

`.temp/brief-632/mutants.py` copies `scripts/perf-panel.py` to `scripts/perf-panel-mutant.py` (the repo root must be the parent of `scripts/`, or label lookup looks under `.temp`). Each mutant turns one named selftest red. The work file is removed after the run.
`MUTANT apart >= for >: CAUGHT`
`MUTANT slope without / N: CAUGHT`
`MUTANT re-entry ignores sha256: CAUGHT`
`MUTANT layout split without hex masking: CAUGHT`
`MUTANT lhs- args dropped: CAUGHT`
`MUTANT pathcal compares counts: CAUGHT`
Re-run after the absolute-cache-path edit: same six lines, exit 0. `scripts/perf-panel-mutant.py` is absent.

### What was not ported

Not ported: `lp01_fingerprint.py` (not in the port table), brief-629's older `attr.py` (the port is gate-630b2's attr), `fndiff2.py` (structure uses fndiff.py and fndiff3.py), and 216 brief-626 perf files no command reads (202 `*-check.rum` plus json, tsv, csv, and notes).
Specified output differences, checked rather than treated as a rule change: shape uses `calls[<pattern>]` and the numbers equal resume-shape.sh; fnslope adds `# arm=<arm> rep=<rep>` before instr_slope.py's lines.
The harness Cargo.lock renames package `brief608-resume` to `rum-resume-harness`. Harness sources were rustfmt'd and `cargo fmt --check` exits 0.
The C8 mutant tree also holds a copy of the harness, because arms overlays the candidate's harness and head-src does not contain one.
No decision rule was changed. The apart comparison stays strict `>`. Equal is not apart.

## Director gate

**Rerun by me.**

- Both selftests PASS. The worker's six mutants (`.temp/brief-632/mutants.py`)
  are still CAUGHT on the amended script. `cargo fmt --check` on the harness
  exits 0. `git status crates` is empty.
- The 384 label files are byte-identical to brief-626's. The harness sources
  equal `harness-630b2` up to rustfmt:
  - eight files are equal modulo whitespace, trailing commas and `;}`;
  - `main.rs` is also equal modulo braces and `mod` order;
  - `Cargo.toml` changes only the package name, `[[bin]]` and the `rum-host`
    path; `Cargo.lock` changes only the package name.
- C2 (b3 vs b3): median +0.0036%, apart 0/0.
- C4 (b2base vs b2v1): median +2.6195%, 120 apart above, `resume-cleanup`
  +1458.5. Inside the known answer.
- C3 (b2final vs b3): median +0.0173%, **1 apart above**:
  `resume-s8-repeated-guard +369.7/iter +0.3565% base 103507-103947 cand
  103951-104267`, four instructions apart.
  - The worker's C3 has the same label at +336.11 (+0.3240%), not apart.
  - Callgrind (`attr`, 6 runs per arm) gives +304.19/iter, all of it in
    mimalloc's aligned-allocation paths:
    - `mi_theap_malloc_zero_aligned_at_generic` +124.8;
    - `_overalloc` +117.0;
    - `_mi_theap_malloc_zero` +58.0;
    - `mi_bin` +34.8;
    - `mi_good_size` +22.5;
    - `_mi_bin_size` +12.8.
  - None of B3's changed functions is on this label's path. It is a layout
    effect, so the brief's "0 apart" on this pair is not a stable known
    answer: a panel on it may flag this label, and the flag is ruled after
    attribution.
- C7's shas differ because mimalloc embeds `__TIME__` (`21:39:31` against
  `21:41:13`, beside `Oct 29 2026`). Resume's `.rodata` also shifts by 64
  bytes, a linker string-merge layout: cand keeps a separate `"1"`. The code
  is equal by `structure`.
- Not rerun by me: C1's TSV and command equivalence, C5, C6, C8, C9.

**The git parts the worker could not run.**

`run 632 --steps scope,fmt,arms,structure,panels --perf resume`. The
worktree did not exist, so `head_worktree` took its `worktree add` path:

| Step | Verdict | Secs | Detail |
| --- | --- | --- | --- |
| scope | RULED | 0 | 4 problems: `.gitignore`, `CLAUDE.md`, `TODO.md`, the progress snapshot, all the director's own edits |
| fmt | PASS | 2 | formatted |
| arms | PASS | 202 | base rc 0/0 source-equal=yes, cand rc 0/0 source-equal=yes, pathcal resume PASS, pathcal rum-cli PASS |
| structure | PASS | 21 | resume: differ 0 (layout 0, code 0), new 0, gone 0; rum-cli: the same |
| panels | PASS | 83 | resume: 121 labels, median +0.0104%, max +0.1180%, apart above 0/below 0 |

- **Checkout path.** A second `--steps arms,structure` run took the checkout
  path. Its builds were no-ops (0.1 s and 1.9 s) and the result was the same.
- **Stale arms.** A run on a fresh stage dir (`--stage x`) without `arms`
  gave `structure` and `panels` FAIL "no arms receipt in this gate".
- **`rule`.** It refuses a PASS step ("rule refuses PASS: not FLAGGED"), and
  `rule 632 scope` turned FLAGGED into RULED.
- **Not run, and why.**
  - The CLI panel at 15 reps, about 73 minutes. In `step_panels` it differs
    from resume only in the engine name, and perf-panel's CLI path ran in the
    worker's C5 and C9.
  - The crates-dependent steps (`check` through `workspace`). Nothing under
    `crates/` changed, and the crates tree is `5bffd8d2a0b2`, the tree run
    dg1 gated GREEN.

**Director amendments, in this commit.** Items 2–4 have selftest cases,
each watched to fail on its mutant: 4 perf-panel mutants and 2 director-gate
mutants, all CAUGHT. Item 1 has the falsifier given with it.

1. **`head_worktree` resolves `rev` in the main tree.** Inside the worktree,
   `HEAD` names the worktree's own checkout. The falsifier: with the worktree
   at `HEAD~1`, `head_worktree("HEAD")` left it at `333e446c` while the main
   HEAD was `7f671af5`.
   - After any commit, the next gate would have used the previous commit as
     its base. `base_frontier` would have dumped a stale frontier under the
     new tree's key: brief-631's code, latent because no worktree had
     survived until now.
   - It now checks out the resolved sha and verifies the worktree's HEAD.
2. **`structure` drops alignment padding after a body.** objdump counts the
   padding as the preceding function's instructions: `mi_bin` read 23 or 25
   lines on identical C, with the extra lines being `cs nopw` padding.
   b2final→b3 is now differ 2, code 2: B3's two abandon functions, and
   `report_abandoned_cleanup_error` as the one new function. That replaces
   C1's differ 3.
3. **`structure` keeps every body under a duplicate name.** Each later one is
   kept as `name#k` and grouped back. fndiff kept only the last: resume has 43
   such names, with 82 more instances. New and gone now use hash-stripped
   names, because a crate whose metadata moved renames every symbol it has:
   b2base→b2v1 new/gone went from 2777/2807 to 3/27.
4. **No stale binaries.** `arms` deletes an arm's binaries before building
   it. The gate's `structure` and `panels` FAIL unless the arms on disk are
   the ones this gate's `arms.json` built, so a `--from structure` run cannot
   measure an earlier gate's binaries.
5. **Smaller items.**
   - A comment on `attr`'s debuginfod fallback to `.temp/brief-629/debuginfod`,
     which a sandboxed worker needs and the report did not disclose.
   - `.gitignore` gains `__pycache__/`: the worker's mutants import the script.
