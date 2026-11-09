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
- No live edit, no git of any kind (R4: the dispatch prompt forbids it and
  the freeze receipt is the director's), no inventory/census replay, no host
  suite, no workspace run (the director's), no
  timing/instruction/microbench lane.
- The source freeze is bound without git and without re-hashing
  git-tracked source (user rulings 2026-11-04 and the dispatch prompt's git
  prohibition), per R4: quote the director's receipt
  (`.temp/director/freeze-714.json`, written after this ruling's commit),
  measure the live artifact's sha (R2.1) and the fresh build's 16/16
  object-code identity (R2.2), and record the one draw binary's sha. The
  built test binary's SHA-256 is the untracked-artifact receipt that must
  not change across the campaign.

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
compares rum against itself); no git of any kind (R4), no live edit, no
network, no third-party engine, no new `unsafe`; a red is reproduced at
`--test-threads=1` before it is called a defect; all output under `.temp/`;
the report is `rum-report/1` JSON with copied verdicts and the
command/log/exit/count table.

---

## Stage-29 ruling — 2026-11-09: run 29's preflight STOP is well taken; the freeze query is read-only, and the release bar is object-code identity

Run 29 stopped at the source-freeze preflight and did nothing else: no
build, no draw, no live edit, no git call. The draw index is empty, the
report quotes the contradiction it found (`checks: 2 PASS, 1 FAIL, 3 NA`,
Q1), and its evidence is archived at `.temp/brief-714/evidence/run29/`. The
STOP is accepted as well taken. Both premises of the contradiction are mine
and wrong, and the preflight exposed a third defect before the 37-draw
budget could be spent on it.

### R1 — the freeze query is read-only and authorized

The Scope required `git rev-parse HEAD` / `git status --porcelain` and then
forbade "git" in the same document. "No git" forbids **mutation** — add,
commit, restore, checkout, switch, stash, clean, branch, tag, fetch, pull,
push. Exactly these three read-only commands are authorized, once, at the
start, stdout recorded verbatim in the report:

1. `git rev-parse HEAD` — record the value. It is the docs commit carrying
   this brief, not `03b4c133`.
2. `git status --porcelain` — must be empty.
3. `git diff --quiet 03b4c1337f665a9d69aac06dd8887fa1399951be HEAD --
   Cargo.toml Cargo.lock crates` — must exit 0.

(3) is the freeze measurement: the tree under test is the accepted
`03b4c133` source. (1) is a receipt, not a bar, because this brief's own
commits move HEAD. "No active mutant" needs no separate check: a live
mutant would break (2) or (3), and mutant copies live under `.temp/`, not in
the source tree. Any other git command, and any source rehash, remains
prohibited. My own receipt at ruling time: HEAD
`12ef177b60a9cd3e8d0cbafe1e41fb28c57425dc`, `git status --porcelain` empty,
(3) exit 0; the worker records its own values after this ruling's commit.

### R2 — the release bar is object-code identity; the rlib SHA cannot be reproduced

The clause "the rebuild must be byte-identical to `d9f2339d…`" is refuted by
my own measurement — the Q39 class again: an anchor a rebuild cannot
reproduce is not a bar. Two fresh private-target builds of the frozen tree
at different paths (`.temp/director/probe-714/`,
`.temp/director/probe-714b/`; `cargo build --release --offline --locked
-j8`, 28 s and 27 s wall, peak 2.35 GB) and the accepted artifact agree
exactly where the source decides:

- **all 16 object members of `librum_runtime-c71407e195689fc9.rlib` are
  byte-identical across all three**;
- `lib.rmeta` is 18,810,704 B in every one, and the only differences are
  **7 sixteen-byte dependency-hash entries** at fixed offsets (1069, 1120,
  1269, 1373, 1424, 1474, 18,809,661; 112 bytes fresh-vs-accepted, 111
  fresh-vs-fresh);
- both causes are named and are not source: `liblibmimalloc_sys`'s
  `b85de32113adef8e-static.o` embeds the C compiler's `__TIME__`
  (`11:20:51.Nov  9 2026` in the accepted artifact, `18:54:48.Nov  9 2026`
  in a fresh one; 5 bytes, same size), and the `serde` / `thiserror` rmeta
  embed their build-script `OUT_DIR`
  (`…/target/release/build/serde-b83050e6dac453a9/out/private.rs` against
  the `.temp/…` twin; 893,848 vs 893,880 B).

The production-erasure bar is therefore:

1. **Live identity (read-only; no build in the live target):**
   `target/release/deps/librum_runtime-c71407e195689fc9.rlib` sha256 =
   `d9f2339d05f2babf3c37f6da750e6ec72702febf2d42785a96f600c493642627`.
2. **Fresh build:** `CARGO_TARGET_DIR=.temp/brief-714/target cargo build
   --release --offline --locked -j 8`; artifact path and sha recorded;
   extract it and the live artifact and require **16/16 object members
   byte-identical** and `lib.rmeta` equal in size with differences confined
   to at most 7 runs of ≤ 16 contiguous bytes; anything else is a STOP with
   the member table.
3. **The four-column certificate runs on the fresh artifact**, so it binds
   an artifact this run built; it names both shas and carries the
   object-identity table.
4. `target/release/librum_runtime.rlib`, the uplifted copy with sha
   `4118c335…` (the retired C24 anchor), is **not** the artifact: never
   certificate it.

### R3 — the draw binary is built fresh, privately, once

In the warm live `./target` the same trap applies to the test binary, so it
is built in the same private target:

- `CARGO_TARGET_DIR=.temp/brief-714/target cargo test -p rum-runtime --lib
  --offline --locked -j 8 --no-run` → the one draw binary, path and sha
  recorded once; every draw runs that file directly, never `cargo test`
  again;
- the features control builds its own binary in the same private target
  with `--features test-builtin-memory-growth,test-continuation-slots`,
  separate from the 37 and not a bar;
- bounds: each build ≤ 900 s, each draw ≤ 600 s (the frozen source's known
  serial full-lib wall is ~600 s; an 8-thread draw is a fraction of it), all
  other brief text unchanged.

### State

The dispatch log is archived as `.temp/logs/archive/cdx_brief_714.run29`;
the draw index restarts empty and **no draw was consumed**. The series, the
count bar (`1413 passed; 0 failed; 5 ignored; 0 measured; 2 filtered out`),
the two standing skips, the failure protocol and the no-restart rule are
unchanged. Report and stop for the director's gate.

## Stage-30 ruling — 2026-11-09: the freeze receipt is the director's; the worker runs no git

Run 30's second preflight STOP is well taken for the same reason as the
first, one level up: the dispatch prompt itself carries a user-level
instruction, "Do not run git in any form, not even git status: the director
owns git", which outranks a ruling inside the brief. R1 is inoperable as
written and is withdrawn from the worker's duties: git stays entirely mine.
Run 30 did no build, no draw and no edit (0/37, empty index; its evidence is
archived at `.temp/brief-714/evidence/run30/`).

The starting freeze is bound by four receipts, each quoted in the report:

1. **Director receipt** (`.temp/director/freeze-714.json`, written by me
   after this ruling's commit and push): the committed HEAD, `git status
   --porcelain` empty, `git diff --quiet
   03b4c1337f665a9d69aac06dd8887fa1399951be HEAD -- Cargo.toml Cargo.lock
   crates` exit 0. It is a director measurement; quote it verbatim.
2. **Live artifact identity** (worker, R2.1): sha256 of
   `target/release/deps/librum_runtime-c71407e195689fc9.rlib` =
   `d9f2339d05f2babf3c37f6da750e6ec72702febf2d42785a96f600c493642627`.
3. **Source binding by code** (worker, R2.2): the fresh private-target
   build's 16/16 object-code identity with the live artifact. Object code
   is a function of source and flags alone — my two-path probe reproduced
   all 16 members byte-for-byte while only metadata hashes moved — so the
   tree this build compiles is the tree the accepted artifact came from.
4. **No live edit** (worker): the report's `changes: 0 files`; the director
   re-runs the git checks at the gate.

The worker runs no git command of any kind and rehashes no tracked source.
Everything else stands unchanged: R2's live-identity, fresh-build,
object-identity and certificate bars; R3's build discipline and bounds; the
37-draw series, its count bar (`1413 passed; 0 failed; 5 ignored; 0
measured; 2 filtered out`), the two standing skips, the failure protocol and
the no-restart rule. Resume with the run-30 session
`01a1bff6-dd94-7ca3-9cb6-8cd02384aa0a` as fork; report and stop for the
director's gate.
