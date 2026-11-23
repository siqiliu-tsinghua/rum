# Brief 772 — G15-CM4 the class fixtures, third attempt: the corrected invocation on the carried scene

**Director-dispatched, one stage, one gate, no live change.** This continues ruling 780 and
ruling 779. Brief 771 carried the verified class-fixture scene byte-identically but its one
invocation died before any test: the command placed `--no-fail-fast` after `--`, and libtest
1.85 has no such option — it is a **cargo** flag and must precede `--`. The director
reproduced the rejection against the built scene binary. This brief carries the same scene
again and runs the corrected command exactly once.

## Goal

When this brief is done, in `.temp/brief-772/` only: a scene copy of `.temp/brief-771/scene/`
(byte-identical, walk `12a687e2…`/5,816) whose class fixture module runs **green** in one
bounded 8-thread invocation of the corrected command — `guest::programs()` passes the
canonical-source loop, the guest-route checks and the program-table seal — with the
guest-dependency branch recorded as it runs. A red is a STOP with the exact failing rows.
No live edit, no fixture change, no weakened check, no receipt.

## Context pointers

- Read `AGENTS.md`; rulings `.temp/director/ruling-780.md` and `.temp/director/ruling-779.md`;
  brief 771 (`docs/dev/codex/brief-771-g15cm4-class-fixtures-second.md`) with its Gate c130.
- **The red baseline**: `.temp/brief-766/logs/class-lease-fixtures.log` — 196 tests,
  2 passed / 194 failed, 592 filtered out, test time 0.61 s,
  `FIXTURE PROGRAMS row=P0216 mode=positive err=SourceIdentity`.
- **The scene source**: `.temp/brief-771/scene` — walk `12a687e2…`/5,816 under the walk that
  skips {`.git`, `target`, `.temp`, `__pycache__`, `.mypy_cache`}; the class table
  `crates/rum-stdlib/tests/suspension_oracle/inventory/obligations/class-provenance.tsv`
  (4,803,954 B, `8c8b2232…`); the scene `provenance.tsv` `965eca10…`; the two declared loader
  edits (`guest.rs:1343` `provenance()` → `class_provenance()`; the `class_provenance()`
  accessor in `coverage/bindings.rs`, baseline at 106 / class at 114); the scene's `target/`
  is **not** carried (874 MB of build output stays in brief 771's scene).
- **The invocation**: `scripts/run-bounded.py`; the recorded fixture sample 2,961.635 s and
  the 347.342 s image; `stack_1x = 3,488.977 s` (image + fixture + the 180 s close);
  `3× sample = 8,884.905 s`.
- **The corrected command (why)**: `--no-fail-fast` is a cargo option; libtest rejects it
  after `--` with `error: Unrecognized option: 'no-fail-fast'` (brief 771's failure, exit
  101 at 53.012 s, 0 tests; the director reproduced it against the built binary). The next
  cargo test in this line must place it before `-p` and must not add it after `--`.

## Deliverables

1. **Entry re-authentication.** Recompute the five tree digests (source `97eaeb45…`/5,810,
   armed `bd8b7548…`/5,810, disabled `b41d1873…`/5,810, 764 `0f8cbc32…`/5,812, 766
   `cce4cc03…`/5,815), the 199 live pins, the 13 OPEN Q25 rows, and the 768/769 evidence
   digests (overlays `73297529…` additions `78106c66…`, class table `8c8b2232…`,
   `source_schema` `3faa2999…`, inputs `aa99638d…`). STOP on any mismatch.
2. **The scene carry.** Copy `.temp/brief-771/scene/` (excluding `target`) to
   `.temp/brief-772/scene/`; verify its walk digest equals `12a687e2…`/5,816, the class table
   is `8c8b2232…`, the scene `provenance.tsv` is `965eca10…`, and the loader diff against
   the 766 tree is exactly the two declared edits. Record the exact diff and the post-copy
   walk.
3. **The budget gate.** Start the invocation only when `remaining ≥ stack_1x` (3,488.977 s)
   **and** `remaining ≥ 3× the fixture sample` (8,884.905 s), recorded verbatim. No suite
   shrinkage, no retry; a shortfall stops for a director ruling.
4. **The one invocation.** From the scene, one bounded run, exactly:
   `python3 scripts/run-bounded.py --timeout 8884.905 -- cargo test --locked --offline -j8
   --no-fail-fast -p rum-stdlib --test suspension_oracle class_lease -- --nocapture
   --test-threads=8`
   with `RUST_TEST_THREADS=8`. `--no-fail-fast` is a cargo flag before `-p`; nothing goes
   after `--` except `--nocapture` and `--test-threads=8`. Do not run the lane twice; a red
   is a STOP with the exact failing rows.
5. **The acceptance.** The class fixture module green: zero failed, the test count recorded,
   and the loader's check families unchanged (the canonical-source loop, the guest-route
   checks, the program-table seal; the guest-dependency branch recorded as it runs — its
   non-firing is never claimed). The green must be the scene binary's: record the binary path
   under the scene's `target/`, the exact command, the elapsed seconds and the verdict lines.
   If the run is red, name every failing row, the harness lines and whether any
   `RunFailure::Native` appeared.
6. **The authenticity checks (watched, record-based).** The scene differs from the red 766
   tree only by the declared files (the diff record); the class table's sha256 equals
   `8c8b2232…`; `provenance.tsv` in the scene equals the 766 byte; `source_auth`,
   `known_losses` and `proofs` still resolve their seals through the baseline accessor (read
   the call sites and state the mechanism); a green `AuthenticatedSource::source` is not the
   acceptance and is not substituted for the loader; any Q25 row marked closed or any
   importable receipt is a STOP.
7. **The report.** `rum-report/1` JSON at `.temp/brief-772/report.json` via
   `scripts/report.py`, sections: entry, scene, switch, invocation, authenticity, scope,
   budget. The summary states the test count, the loader verdict lines, the exact open facts
   (any `RunFailure::Native`-style failures; the 760 hash-only intermediate remains named).
   `report.py check 772` green.

## Hard rules

- **Scope: NONE in the live tree.** Work under `.temp/brief-772/` only; `.temp/brief-771/`,
  `.temp/brief-770/`, `.temp/brief-766/`, `.temp/brief-769/` and every accepted scene are
  read-only; no build in the live tree or in any accepted scene; no live edit of any kind.
- **No fixture weakening.** The class fixture file, `programs.tsv`, the five additions and
  the overlays are not edited; the loader's checks are not changed; `provenance()` and its
  non-loader consumers stay on the baseline table.
- Direct cargo inside the dispatch scope at `-j8` (no nested `scripts/memory-scope.sh`);
  tests `RUST_TEST_THREADS=8`; one invocation; timing/perf serial.
- `.temp` only, never `/tmp`; no network; no new dependency; no new `unsafe`; no new program,
  lane, selection, policy or epoch label; no git; never read `~/.config/ccds/api-key`.
- Q25's thirteen rows stay named stops; 744/745/746 stay held; CM5 stays unauthorized.

## Budget

**focused 9,000 s + workspace 0 s + epoch 0**, carved from the stage reserve
(34,639.70378847017 → **25,639.70378847017 s**); the carve covers the scene's gate after the
entry and scene work and the invocation's 8,884.905 s timeout. Unspent returns; a projected
overrun stops for a director ruling — more is carved, never a scope cut.

## Report format

`rum-report/1` JSON at `.temp/brief-772/report.json` via `scripts/report.py`; sections as in
Deliverable 7; `summary` states the result, the test count, the loader verdict lines and the
exact open facts.

## Gate c131 (2026-11-23) — accepted STOP; the loader runs green, three named positive rows are red

Run: `grk` (grok-4.7, effort xhigh), unit `brief-772-grk-20261123-162731-1180275`,
session `de13e6b6-75ab-42fc-9be8-250710d7f385`, 16:27:31 – 16:45 CST, `status=success
turns=40 cost_usd=2.964548 refused=0`, rc 0, memory peak 2.809 GB; scene
`.temp/brief-772/`; `report.py check 772` OK, 72 citations; 12 PASS / 1 FAIL checks,
0 deviations, 0 questions, `changes` empty.

Verified: `ENTRY PASS trees=5 pins=199 q25=13 evidence=5`; my own walk of the scene gives
`12a687e2…`/5,816, the class table `8c8b2232…`, `provenance.tsv` `965eca10…`, the loader
diff exactly the two declared edits, every baseline consumer on `provenance()`; the gate
passed at 8,993.239 s and the launch recheck at 8,993.157 s; the one invocation (the
corrected command, `--no-fail-fast` at cargo level) ran once and reported `running 196
tests` / `193 passed; 3 failed; 0 ignored; 0 measured; 592 filtered out; finished in
22.05s`, exit 101, 75.979 s; 191 `FIXTURE VERDICT` lines and no `FIXTURE PROGRAMS`
failure — `guest::programs()` returned Ok everywhere, so the loader's four check families
pass on the class table. The three red rows are `FIXTURE POSITIVE_MISS` on
`C5.default.profiled.finish`, `P0492` and `P2299` (the `assert!(committed)` at
`class_lease_fixtures.rs:64`).

Findings: **F0** the STOP is correct and the red is real; the loader is not implicated.
**F1** first-pass mechanisms: P2299's frame is allocated but as
`SchemaValidationSuspensionFrame<alloc::vec::Vec<usize>>`, which `kind_matches`' exact/
`::`-suffix rule cannot match against the recipe's `SchemaValidationSuspensionFrame` (the
756 inventory kept `<Vec<usize>`); C5's contract event is the InputRequired return at
`default_fallback.rs:258-261`, which did not fire (its neighbouring
`DirectDefaultFallbackFrame<false>` is also unmatched); P0492's expected
`ProfiledPreparedDispatchFrame` appears zero times — the enabled profiled prepared stage
is not taken or its hook is missing. **F1 also exposes a masked hole**: `positive_receipt`
accepts any committed receipt, so 15 synchronous rows pass via the return seam while
their recipe-frame allocations never match (P0248 demonstrates it); the positive bar must
be strengthened, never weakened. **F2** the loader switch, the two-anchor rule and the
class table stay validated by use.

Budget: 772 charged 82.740 s of 9,000; unspent 8,917.260 s returns; reserve
34,556.96378847017 s. Next: brief 773, the design-only zero-unknown inventory of the
three misses and the positive-assertion coverage (focused 900 s).
