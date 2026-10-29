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
