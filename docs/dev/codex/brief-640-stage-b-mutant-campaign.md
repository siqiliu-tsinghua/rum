# Brief 640 — Stage B: the isolated production-mutant campaign

**Shape: DIAGNOSTIC. Negative controls in scratch copies. `Scope: NONE` in the
authoritative tree.** The deliverable is evidence, not a patch: for every
mutant in the audit's frozen manifest, the exact ignored test must be green on
the unchanged copy and then either red at the named assertion or classified
SURVIVED / INCONCLUSIVE with its cause. Nothing moves from scratch into
production.

## Goal

Close the audit's central question: **which certificates and contracts are
known to be able to fail, and which are not.** Brief-639's 61 rows contain 52
that need a mutant or an open obligation; it designed 45 production mutants
covering 51 of them, and I07 is the 52nd, OPEN with no honest localized
mutant (the other nine rows are behaviour, instruments, the generator and the
falsifier). This brief runs the 45, one bounded scratch copy at a time, and
reports one of:

- **KILLED** — the mutant compiles, the named target fails at the manifest's
  assertion with the predicted changed value, and a `--test-threads=1` retry
  reproduces it;
- **SURVIVED** — the target is green under the mutation (a real finding: the
  certificate cannot see that change);
- **INCONCLUSIVE** — compile error, timeout, OOM, zero tests selected, signal
  without causal evidence, or a red for the wrong reason;
- **EXCLUDED** — outside the authorized subset (none in this brief);
- **OPEN** — I07/D01, which has no honest localized production mutant.

## Context pointers

- `docs/dev/codex/brief-639-test-audit-inventory.md` — the accepted audit
  record. Its **inventory** table gives every row's function, file, reason,
  mode/feature/env, one-line claim and assertion sites; its **mutants** table
  gives every mutant's production `path:line`, the one-line change, the
  changed value → assertion → no-swallow path, and its limits. **These two
  tables are the frozen manifest.** Do not re-derive, extend or reinterpret
  them; when the manifest and the code disagree, STOP that mutant as manifest
  drift.
- The audit record's **obligations** D01–D09 bound what this brief may do.
- `docs/dev/codex/TEMPLATE.md` (the shape and the standing-inventory rule at
  `:88`), `AGENTS.md`, `docs/testing-strategy.md`.
- The director's clean-copy calibration: `.temp/director/audit-638/bounds/`
  (`timings.md`, `base-run2.log`, `feature-cold.log`). The bounds below are
  derived from it; do not re-measure them.

## Scope

- **Files you MAY modify: NONE in the authoritative tree.** Every mutation
  lives in `.temp/brief-640/mutants/<ID>/<attempt>/workspace/`, a byte copy of
  the current tree.
- Scratch only under `.temp/brief-640/`. Do not write under `.temp/brief-639/`,
  `.temp/director/`, or anywhere else.
- Mutants to run — exactly these 45: **M03, M06, M06b, M08, M09, M10, M12,
  M13, M15, M16, M17a, M17b, M18, M19, M21, M22, M23, M24, M25, M26, M28,
  M29, M30, M31, M32, M33, M34, M35, M36, M37, M38, M39, M40, M41, M42, M43,
  M45, M46, M47, M48, M50, M52, M53, M54, M54d.** I07 is OPEN by D01 and has
  no mutant.
- **Excluded, do not run**: the two perf instrument drivers (I04, I05), the
  SCRATCH_ENV falsifier `forced_red_probe` (I61), the generator I55 as a
  target, every behaviour row (I01, I02, I58, I59, I60), and every test not
  listed as a target of a manifest mutant. No full workspace suite, ever.
- M54 and M54d are authorized (R-639-6) to use the feature build
  `-p rum-stdlib --lib --features rum-runtime/test-native-stack-guard-zero`.
  No other feature-gated build is authorized.
- No network, no git in any form (not even `git status`), no dependency
  changes, no edits to tests, helpers, fixtures, expected values, inventories
  or assertions.

## Deliverable

For every mutant and every listed target:

1. The copy receipt: `manifest.json` with the copy's path, creation time, and
   the SHA256 of every file the copy mutates plus `Cargo.lock`; no hardlinks
   or symlinks back to the authoritative tree.
2. The **baseline** receipt: the exact target, run unchanged in that copy
   with the row's package/profile/features/env, GREEN, with the quoted
   `test result: ok. N passed` line.
3. The **mutant** receipt: the mutation applied (exact old snippet and new
   snippet, both quoted, plus the byte offset or line), the compile verdict,
   the run verdict, and on a red the actual failing test name, assertion
   text, and `test result: FAILED` line, verbatim.
4. A **serial retry** at `--test-threads=1` for every claimed KILLED, with
   the same quoted evidence.
5. A classification per target with the manifest's prediction compared to
   the observed value. A KILLED whose assertion line or changed value differs
   from the manifest is **INCONCLUSIVE (wrong reason)**, not KILLED.
6. A **results table**: `mutant | row | package | target | baseline | mutant
   | assertion | classification`.
7. An **integrity receipt**: the SHA256 of every `crates/**/*.rs` before and
   after the campaign (the authoritative tree is read-only), identical.
8. For M08/M12 only: the child's captured output. A bare abort, process
   signal, timeout or stack overflow with no evidence that the named
   destructor ran is INCONCLUSIVE (D02); quote whatever the child printed.

## rum / Rust pitfalls (read twice)

- **Manifest drift.** The old snippet must match byte-for-byte at the named
  production site in the copy. If it does not, STOP that mutant and record
  the current bytes; do not adapt the mutation.
- **One mutation, one target run.** Apply exactly the listed change; no
  compensating edit, no extra panic, no test change, no assertion weakening.
- **The observation path is the contract.** The manifest says why nothing
  downstream swallows the red. If the test is red for another reason, it did
  not prove the claim.
- **Equivalent classes are not merged runs.** M09/I09+I11, M10/I10+I11,
  M13/I13+I14, M19/I19+I20, M26/I26+I27, M42/I42+I44, M48/I48+I49+I51 and
  M54/M54d/I54+I56+I57 each run **every** listed target separately, even
  though one mutation serves them.
- **Feature shape.** I54/I56/I57 need
  `--features rum-runtime/test-native-stack-guard-zero` and are **not** run
  anywhere else.
- **I03's environment.** M03 needs `RUM_PROFILE_EVALUATION_CACHE=1`; the test
  manages its own child marker (`evaluation_cache/tests.rs:537`). The
  env-precondition red at `tests.rs:531` is not M03's red.
- **Test selection.** Resolve each target's fully-qualified name with
  `cargo test ... -- --list --ignored` in the copy, then run it as
  `<full::path> --exact`. A partial name with `--exact` selects zero tests
  and is INCONCLUSIVE, not a green.
- **Copy discipline.** Copy file bytes (rsync or cp), never hardlink,
  never follow a symlink out of the copy, exclude `.git`, `.temp` and
  `target`. The copy's own `target/`, `TMPDIR` and logs live inside its
  attempt directory.
- **Standing inventories.** Inside a copy, after the mutation, determine
  which standing inventory generators cover a file the mutation touched
  (by the generators' own discovery rules). Replay only those, inside the
  copy, before the target run. For most mutants the covered set is empty;
  record the check that showed so. A red caused only by a stale derived cell
  is an **ORDERING ARTIFACT**: replay, retry the same exact target, and do
  not count the first result. A genuine contract rejection during a replay is
  recorded separately and is not the target's red.
- **Do not tune the test or the mutation** to obtain a red. A SURVIVED is a
  finding, not a failure of the campaign.
- **Remove or mark the copy.** After a mutant's receipts are written, remove
  its workspace (free ~3 GB) or mark it `AUDIT SCRATCH — NOT FOR PROMOTION`.
  Never leave more than one full copy's `target/` on disk.

## Measured bounds (from `.temp/director/audit-638/bounds/timings.md`)

Every child process runs under `scripts/run-bounded.py --timeout <bound> --`.
Do not invent a bound; if a measured shape differs from these (e.g. a much
larger copy), STOP and report.

| Shape | Measured | Bound (3x, rounded up) |
| --- | ---: | ---: |
| cold copy of the tree (about 2,587 regular files / 65.3 MiB; the brief is itself in the tree, so the count moves with docs commits) | 1 s | 60 s |
| cold release build + one exact test, base shape | 84 s | 300 s |
| cold release build + one exact test, zero-guard feature shape | 87 s | 300 s |
| incremental rebuild + one exact test in a warm copy | not separately measured; use the cold bound | 300 s |
| one exact test after the build | 0.01 s | 120 s |

Measured 2026-11-02: `.temp/director/audit-638/bounds/timings.md`,
`base-run2.log`, `feature-cold.log`; the base and feature cold shapes both
passed their exact ignored test, peak RSS 2.4 GB, and the copy is one second.
The baseline and the mutant in the same copy share one built `target/`; the
mutant run rebuilds only the changed crate.

The first version of this table said "32 MB"; that was `du` disk usage on a
compressed filesystem, not the byte size. The second said an exact byte
total, which this brief itself invalidates every time it is amended. The
copy gate is therefore a **ceiling, not an equality** (Amendment 3): a copy
of at most 4,000 regular files and 120 MiB (125,829,120 bytes), excluding
`.git`, `.temp` and `target`, passes; anything larger is a STOP. Do not
compare the copy to a literal byte total, and do not stop because the brief
or a docs commit changed it.

Every cargo command runs inside the dispatcher's own 16 GiB memory scope
(the `memory-scope: unit=brief-640-<worker>-...` line in the dispatch log) with
`CARGO_BUILD_JOBS=8`, `RUST_TEST_THREADS=8`, `--offline --locked`, one cargo
command at a time, no other heavy work alongside. **Do not invoke
`scripts/memory-scope.sh` yourself** — inside this sandbox it cannot reach
the systemd user bus (Amendment 1).

## Verification (MANDATORY — you run it)

- Baseline green first, in the same copy and shape, for every target.
- One mutation applied and quoted; compile verdict quoted.
- Red equals the manifest's assertion and changed value; serial retry repeats
  it and is quoted.
- SURVIVED means the exact target ran (quoted `test result: ok. N passed` with
  N ≥ 1) under the mutation.
- INCONCLUSIVE covers compile failure, timeout (exit 124), OOM, a signal, zero
  tests selected, or a wrong-reason red — each with its quoted evidence.
- The authoritative source hashes before and after are identical.
- Report `python3 scripts/report.py check <report>`; quote every verdict line.

## Stop conditions

- Manifest drift at a production site: STOP that mutant, record the bytes.
- A red for the wrong reason: INCONCLUSIVE; do not count it.
- Two consecutive attempts on one mutant with no progress: stop that mutant.
- Any write outside `.temp/brief-640/`: stop the campaign and report.
- A measured shape exceeding its bound three times: stop and report the
  numbers; do not raise the bound yourself.
- No production repair, no test repair, no assertion relaxation, no
  compensating mutation — not even to make a red appear.

## Checkpointing — protect the scene

`PROGRESS.md` records per-mutant status after each one (and at least every
five); a resumed run skips completed mutants and never reruns a consumed
baseline. Keep every log and every failed attempt. The campaign may be long:
stop cleanly at a checkpoint rather than losing receipts.

## Hard rules

- No git, no network, no authoritative change, no test change, no `unsafe`,
  no third-party engine oracle, no compatibility claim, no full workspace run.
- Generated rum sources stay suffixed `.rum` inside the copy.
- Never hand-edit or weaken a standing inventory; replay then retry on an
  ordering artifact.
- The production tree must be byte-identical before and after; the hash
  receipt is the proof.
- Report in `rum-report/1`; a number without a quoted line is an open
  question.

## Amendment 1 (2026-11-02) — the dispatcher's scope, not a per-command launcher

Attempt 1 stopped before any cargo run: the brief required
`scripts/memory-scope.sh` around every cargo command, and that launcher
cannot connect to the user scope bus inside the worker sandbox
(`Failed to connect to user scope bus via local transport: Operation not
permitted`). The worker stopped correctly with no fallback, no copy and no
mutation. The rule is a director rule; the dispatcher already wraps the
worker itself.

- **Do not run `scripts/memory-scope.sh`.** The dispatch log's
  `memory-scope: unit=brief-640-<worker>-...` line is the scope. Run cargo
  directly with `CARGO_BUILD_JOBS=8`, `RUST_TEST_THREADS=8`,
  `--offline --locked`, one cargo command at a time, each as a child of
  `scripts/run-bounded.py --timeout <bound> --`.
- The environment preflight is just `cargo --version` and
  `rustc --version`.
- Everything else in this brief stands unchanged.

## Amendment 3 (2026-11-02) — the copy shape is a ceiling, not a byte equality

Attempt 3 stopped at its own copy-shape preflight: the tree was 68,486,235
bytes against the table's 68,485,831, a 404-byte difference, because this
brief lives in the tree and amendment 2 had just edited it. The gate was
wrong in kind, not in number.

- The copy gate is a **ceiling**: at most 4,000 regular files and 120 MiB
  (125,829,120 bytes), excluding `.git`, `.temp` and `target`.
- Do not compare the copy to a literal byte total; do not stop because the
  brief or a docs commit changed it.
- No copy-shape preflight is required; if one is run, it is this ceiling.
- The 60 s copy bound and every other bound stand unchanged.

## Amendment 4 (2026-11-02) — resume on Kimi

Attempt 4 (Codex) completed six mutants — M03, M06, M06b, M08, M09, M10 —
and then the provider's content filter fired on the crash probes. The user
directed this campaign to Kimi on 2026-11-02 (no GLM quota). The checkpoint
is `.temp/brief-640/PROGRESS.md`; skip every mutant marked COMPLETE and
continue at M12/M13. The deliberately broken production code in each scratch
copy is the authorized purpose of this brief; the production tree stays
untouched. The prior Codex report is preserved beside the new one as
`.temp/brief-640/report-cdx-attempt4.json`. Everything else stands
unchanged.

## Report format

`rum-report/1` at `.temp/brief-640/report.json` with sections `summary`,
`environment`, `bounds`, `results` (the table), `inconclusive` (each row's
quoted evidence), `open-excluded`, `integrity`, `runs` (every attempt with its
exit code and log path). The final response is exactly
`python3 scripts/report.py summary 640`.
