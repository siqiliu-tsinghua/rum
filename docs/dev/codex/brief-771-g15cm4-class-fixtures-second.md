# Brief 771 — G15-CM4 the class fixtures, second attempt (scene carried, one bounded invocation)

**Director-dispatched, one stage, one gate, no live change.** Brief 770 stopped correctly at
the budget gate before the fixture invocation: the scene is built and verified, but my carve
covered neither the 180 s close nor 3× the fixture sample. This brief carries that scene and
runs the one invocation under a carve that meets the scene's own stop line.

## Goal

When this brief is done, in `.temp/brief-771/` only: the carried scene (byte-identical to brief
770's verified scene) runs the class fixture module **once** at 8 threads within a bounded
timeout, and the result is recorded — green with the test count and the loader's verdict lines,
or red as a named STOP with the exact failing rows and their error class. No live edit, no
fixture change, no weakened check, no receipt.

## Context pointers

- Read `AGENTS.md`; ruling `.temp/director/ruling-779.md`; ruling `.temp/director/ruling-776.md`
  (two anchors); the scene `.temp/brief-770/scene/` — post-edit walk `12a687e2…`/5,816, built
  from the 766 tree `cce4cc03…`/5,815 by exactly three declared changes: the class table
  `8c8b2232…` at `crates/rum-stdlib/tests/suspension_oracle/inventory/obligations/
  class-provenance.tsv`, the `class_provenance()` accessor in `coverage/bindings.rs`, and
  `guest.rs:1343` `provenance()` → `class_provenance()`. `provenance.tsv` is `965eca10…`.
- The loader's check families at `fixtures/obligations/guest.rs`: the program-table seal at
  1336-1348, the canonical-source loop at 1351, the guest-route checks at 1427, the
  guest-dependency branch at 1544 (predicate `p.key.row() == "C10.prepared.profiled_consumer"`,
  55 programs, one dependency row each, `segments.rs` `cc403f33…` with pin `73ec2ff5…` — the
  class table carries exactly these).
- The red baseline: `.temp/brief-766/logs/class-lease-fixtures.log` (196 tests, 2 passed /
  194 failed, `FIXTURE PROGRAMS row=P0216 mode=positive err=SourceIdentity`); the recorded
  command `cargo test --locked --offline -j8 -p rum-stdlib --test suspension_oracle class_lease
  -- --nocapture --test-threads=8`.

## Deliverables

1. **Entry re-authentication.** Recompute the five tree digests (source `97eaeb45…`/5,810,
   armed `bd8b7548…`/5,810, disabled `b41d1873…`/5,810, 764 `0f8cbc32…`/5,812, 766
   `cce4cc03…`/5,815), the 199 live pins, the 13 OPEN Q25 rows, and the five 768/769 evidence
   digests (overlays `73297529…`, additions `78106c66…`, class table `8c8b2232…`,
   `source_schema` `3faa2999…`, inputs `aa99638d…`). STOP on any mismatch.
2. **The scene carry.** Copy `.temp/brief-770/scene/` (excluding `target`) to
   `.temp/brief-771/scene/`; verify the walk equals `12a687e2…`/5,816; re-check the class
   table's `8c8b2232…`, the scene `provenance.tsv`'s `965eca10…`, and that the loader diff is
   exactly the two declared edits (the accessor and the one-line call-site switch). Any
   difference is a STOP.
3. **The budget gate before the invocation.** Compute the stop line from the scene's samples
   (image 347.342 s, fixture 2,961.635 s, close 180 s): start the invocation only when
   `remaining ≥ stack_1x` (3,488.977 s) **and** `remaining ≥ 3× the fixture sample`
   (8,884.905 s). Record the gate line verbatim. If it fails, STOP — do not shrink the suite.
4. **The one invocation.** From the scene:
   `cargo test --locked --offline -j8 -p rum-stdlib --test suspension_oracle class_lease --
   --nocapture --test-threads=8 --no-fail-fast`, under `scripts/run-bounded.py --timeout
   8884.905` (3× the recorded fixture bound; never shorter than 2,961.635 s). One run; a red
   is a STOP with the failing rows, not a retry.
5. **The result.** Record the exact command, the binary path under the scene's `target/`, the
   test count, the elapsed seconds, and the verdict lines. Acceptance is the module green with
   zero failed. The guest-dependency branch is recorded as the run shows it (its non-firing is
   **not** claimed); note any `RunFailure::Native` explicitly. Q25's 13 rows stay OPEN; zero
   importable receipts.
6. **The report.** `rum-report/1` JSON at `.temp/brief-771/report.json` via
   `scripts/report.py`, sections: entry, scene, gate, invocation, result, scope, budget.
   `report.py check 771` green.

## Hard rules

- **Scope: NONE in the live tree.** Work under `.temp/brief-771/` only; `.temp/brief-770/` and
  every accepted scene are read-only (the scene copy is the one permitted read); no build in
  the live tree.
- **No fixture weakening, no loader check change.** The class fixture file, `programs.tsv`,
  the five additions and the overlays are not edited; the loader's check families are
  unchanged; `provenance()` and its non-loader consumers stay on the baseline table.
- Direct cargo inside the dispatch scope at `-j8` (no nested `scripts/memory-scope.sh`); tests
  `RUST_TEST_THREADS=8 --no-fail-fast`; timing/perf serial.
- `.temp` only, never `/tmp`; no network; no new dependency; no new `unsafe`; no new program,
  lane, selection, policy or epoch label; no git; never read `~/.config/ccds/api-key`.
- Q25's thirteen rows stay named stops; 744/745/746 stay held; CM5 stays unauthorized.

## Budget

**focused 9,000 s + workspace 0 s + epoch 0**, carved from the stage reserve
(34,708.52278847017 → **25,708.52278847017 s**); the carve covers the scene's gate after the
entry and scene work and the invocation's 8,884.905 s timeout. Unspent returns; a projected
overrun stops for a director ruling — more is carved, never a scope cut.

## Report format

`rum-report/1` JSON at `.temp/brief-771/report.json` via `scripts/report.py`; sections as in
Deliverable 6; `summary` states the result, the test count, the loader verdict lines and the
exact open facts.

## Gate c130 (2026-11-23) — accepted STOP; the scene is exact, the invocation died on the option placement

Run: `grk` (grok-4.7, effort xhigh), unit `brief-771-grk-20261123-160617-1169256`,
session `6d57ac82-59de-4078-9e78-ac309d868c2e`, 16:06:17 – 16:23 CST,
`status=success turns=39 cost_usd=2.902466 refused=0`, rc 0, memory peak 2.718 GB;
scene `.temp/brief-771/` (1.0 GB with `target/` 874 MB); `report.py check 771` OK,
50 citations; 11 PASS / 1 FAIL checks, 1 PASS measurement, 0 deviations, 1 question,
`changes` empty.

Verified: `ENTRY PASS trees=5 pins=199 q25=13 evidence=5` against the five tree digests,
199 live pins (198 distinct), the 13 Q25 OPEN ids, the five evidence digests and
`EMIT PASS rows=97 files=27 importable_receipts=0`; the carried scene is byte-identical
to `.temp/brief-770/scene` — my own walk of both reproduces `12a687e2…`/5,816 with an
empty file-level diff — the class table is `8c8b2232…` (4,803,954 B), the scene
`provenance.tsv` `965eca10…`, and the loader diff is exactly the two declared edits
(`guest.rs:1343` `provenance()` → `class_provenance()`, `c52` seal at 2387 still on
`provenance()`; the accessor, baseline at 106 / class at 114), with the untouched files
equal to the 766 bytes. The gate passed at 8,984.193 s and the launch recheck at
8,906.301 s (≥ 8,884.905 s and ≥ 3,488.977 s). One invocation, exit 101, elapsed
53.012 s, `attempts=1 retry=false`: the from-scratch build finished in 52.83 s and
produced the scene binary (394,353,328 B), then libtest printed `error: Unrecognized
option: 'no-fail-fast'` — 0 tests executed, no fixture row, no `RunFailure::Native`.
My own reproduction: the binary's `--help` has no `--no-fail-fast`, and running it with
the flag reproduces the error; it is a cargo flag that must precede `--`. Scope clean
(0 live writes, 0 accepted-scene writes, 23 inherited git entries).

Findings: **F0** — the STOP is correct and the cause is my brief's command: the flag
belongs to cargo, so the corrected command is `cargo test --locked --offline -j8
--no-fail-fast -p rum-stdlib --test suspension_oracle class_lease -- --nocapture
--test-threads=8`. **F1** — none; the worker executed the stop rule exactly. **Q1
answered**: option 1, the cargo-level flag before `--`; the suite and the 8,884.905 s
timeout stand.

Budget: 771 charged 68.819 s (command 15.807 + invocation 53.012) of 9,000; unspent
8,931.181 s returns; reserve 34,639.70378847017 s. Next: brief 772, the same scene
carried and the corrected one invocation.
