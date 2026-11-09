# Brief 714 — the profiling lease exclusion: the 37-draw source-unchanged validation

**Type:** measurement-only acceptance campaign. **Scope: NONE.** No source,
test, inventory, host or docs edit; no git. All work under
`.temp/brief-714/`. The director owns the commit and the host gate.

## Goal

Prove, on the accepted source `03b4c133` (the conclusion of brief 713), that
the profiling test-lease exclusion no longer flakes under the schedule that
failed before the repair: **37 consecutive full-lib draws at 8 threads, every
one exactly `1413 passed; 0 failed; 5 ignored; 0 measured; 2 filtered out`**
with only the two standing root-context skips. Then prove that the repair has
no production presence: the release artifact rebuilt from this frozen source
must be byte-identical to the accepted C28 artifact
`d9f2339d05f2babf3c37f6da750e6ec72702febf2d42785a96f600c493642627`, and the
four-column erasure certificate must be re-emitted on it.

The pre-repair schedule failed 5/12 (stage-A base), 3/12 (brief-711
candidate) and 4/12 (the brief-712 tree) at 8 threads; all arms were green at
1. With the plug-in null rate p_hat = 4/12 = 1/3, 37 green draws give
(2/3)^37 = 3.05227328079e-7. This is a plug-in independent-draw null
probability, not a confidence bound on an unknown rate; correlated schedules
weaken that reading. Do not substitute a friendlier rate: the director's
earlier (7/12)^24 = 2.41e-6 is not the bar.

## Context pointers

- Brief 713 (`docs/dev/codex/brief-713-profiling-lease-exclusion.md`,
  committed `03b4c133`): the class, rounds C24-C28, the accepted gate, and
  the Q39 release-artifact ruling that retires the fixed archive SHA in
  favour of a certificate bound to its own artifact.
- The pre-repair measurements: brief 712's report and logs under
  `.temp/brief-712/` (the 12-draw base rate and the two deterministic
  probes).
- The accepted series: run 28's report (`python3 -I scripts/report.py show
  713 --run c28-runtime-8-1`), and the director's own workspace gate
  (4,212/0/61 at 8 threads over 63 binaries, with the seven standing skips).
- The accepted release certificate:
  `.temp/brief-713/evidence/c28-release-certificate.json`, the extracted
  members under `.temp/brief-713/evidence/c28-q30/runtime/`, and the
  emitter `.temp/brief-713/certificate_c28.py`.

## Scope

- Only `.temp/brief-714/`: evidence, logs, the extracted artifact, the draw
  index.
- No live edit, no git, no inventory/census replay, no host suite, no
  workspace run (the director's), no timing/instruction/microbench lane.
- Verify the source freeze without re-hashing git-tracked source (user
  ruling 2026-11-04): `git rev-parse HEAD` equals
  `03b4c1337f665a9d69aac06dd8887fa1399951be` and `git status --porcelain` is
  empty at the start. The built test binary's SHA-256 is the untracked-
  artifact receipt that must not change across the campaign.

## Deliverable

1. **One build** of the runtime lib test binary from the frozen tree:
   `cargo test -p rum-runtime --lib --offline --locked -j 8 --no-run`, its
   path and SHA-256 recorded. Every draw runs that binary directly, not
   `cargo test` again, so no rebuild can enter the series.
2. **The 37-draw series.** Each draw: `--test-threads=8`, exactly
   `--skip lp655_root_context_falsifier --skip
   lp656_public_root_context_falsifier`. Save each log before parsing it. A
   draw is green only on exit 0 with exactly `ok. 1413 passed; 0 failed; 5
   ignored; 0 measured; 2 filtered out`. Any other outcome (a failure, a
   different count, a crash, a timeout, a changed binary) ends the streak:
   save the log, reproduce that same binary once at `--test-threads=1`,
   report and STOP. Never restart or re-label the series; never delete a
   failed draw.
3. **A features control**, separate from the 37 and not a bar: one draw at 8
   threads with `--features
   test-builtin-memory-growth,test-continuation-slots`, recorded with its
   count.
4. **Production erasure on the frozen source.** The required release build
   (`cargo build --release --offline --locked -j 8`: features [], opt-level
   3, test false), the artifact path and SHA-256, then the four-column
   certificate on that artifact: every named machinery item has zero object
   symbols and zero object strings across all object members; the qualified
   `profiling::lease` namespace is absent; the pre-existing occurrences are
   enumerated with source provenance; `lib.rmeta` names are listed separately
   with the standalone rustc probe calibration. Because the source is frozen
   and the accepted artifact was built from this same source, the rebuild
   **must** be byte-identical to `d9f2339d…`; a mismatch is a STOP with the
   member-by-member comparison, never a same-SHA claim and never a source
   workaround.
5. No disabled-path measurement beyond the artifact identity and the
   certificate: the repair is `cfg(test)`-only, and the director's member
   comparison already shows the release object code is symbol/address/
   string-identical to the pre-repair artifact. If you believe a measurement
   is missing, ask; do not silently add a lane.

## Cost bar and must-not-change list

- No repository change of any kind. A concern goes back to the director, not
  into an edit.
- One cargo command at a time; builds `-j8`; tests at 8 threads;
  offline/locked; scratch only under `.temp/brief-714/`.
- The 37 draws are the budget: do not rebuild per draw, and do not run other
  work between draws.
- The two standing skips only; no new skip, filter, environment exclusion,
  assertion change, retry or silent draw replacement.
- `report.py check` must pass; every verdict line is copied, not
  paraphrased.

## Verification (you run it)

1. Source freeze: `git rev-parse HEAD` = `03b4c133…`, clean tree, no active
   mutant; the binary SHA recorded once.
2. The 37 draws, each count quoted verbatim, with the per-draw index.
3. The features control.
4. The release artifact and certificate; the archive SHA comparison.
5. `report.py` assembly and check; the final message is `report.py summary`.
   Stop for the director's gate; a green campaign does not authorize a
   commit.

## Checkpointing

An index file (`draw N`, start time, exit, the count line, the binary SHA)
written after each draw before the next starts; each draw's full log saved. A
failure stops with the index intact; never overwrite a completed draw.

## Hard rules

`AGENTS.md` and `TEMPLATE.md`; the testing policy is rum-only (this campaign
compares rum against itself); no git, no live edit, no network, no
third-party engine, no new `unsafe`; a red is reproduced at
`--test-threads=1` before it is called a defect; all output under `.temp/`;
the report is `rum-report/1` JSON with copied verdicts and the
command/log/exit/count table.
