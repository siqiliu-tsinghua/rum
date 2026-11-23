# Brief 770 — G15-CM4 the class fixtures on the class-anchored loader (scene copy, one fixture invocation)

**Director-dispatched, one stage, one gate, no live change.** This continues ruling 778 and
closes the 766 STOP's remaining red: `guest::programs()` failed at the class fixtures because
the committed pins are baseline-derived while the scene tree is the accepted class-form
render. The 768/769 generator emitted the class-anchored loader table; this brief installs it
in a fresh scene copy of the 766 class tree, switches the loader's seal source to it (the
two-anchor rule: `provenance.tsv` stays the baseline anchor for `source_auth`, the loss table
and the proof table), and runs the class fixture module once at 8 threads.

## Goal

When this brief is done, in `.temp/brief-770/` only: a scene copy of the 766 class tree whose
**loader** reads the class-anchored table (byte-identical to the 769 evidence) and whose class
fixture module runs **green** in one bounded 8-thread invocation — `guest::programs()` passes
the canonical-source loop, the guest-route checks and the program-table seal, with the
guest-dependency branch recorded as before. No live edit, no fixture change, no weakened
check, no receipt.

## Context pointers

- Read `AGENTS.md`; ruling `.temp/director/ruling-778.md`; ruling `.temp/director/ruling-776.md`
  (two anchors, two generated tables, one certificate); draft
  `.temp/brief-767/drafts/02-fixtures.md` (with F2's corrections).
- **The red baseline**: `.temp/brief-766/logs/class-lease-fixtures.log` —
  `FIXTURE PROGRAMS row=P0216 mode=positive err=SourceIdentity`, 196 tests,
  2 passed / 194 failed, 592 filtered out, test time 0.61 s; the command recorded in
  `.temp/brief-766/report.json` (`runs[class-fixtures]`: `cargo test --locked --offline -j8
  -p rum-stdlib --test suspension_oracle class_lease -- --nocapture --test-threads=8`). The
  threads-1 reproduction is `class-lease-fixtures-threads-1.log`.
- **The scene source**: `.temp/brief-766/copy` — the class tree, walk digest `cce4cc03…` over
  5,815 files under the walk that skips {`.git`, `target`, `.temp`, `__pycache__`,
  `.mypy_cache`}. **The class table**: `.temp/brief-769/evidence/class-provenance.tsv`
  (4,803,954 B, `8c8b2232…`) — same header and row identities as `provenance.tsv`, 18,203
  rows, only the 60 changed sha256 cells differ and they equal the 766 bytes.
- **The loader**: `fixtures/obligations/guest.rs` `programs()` at line 84, `load_programs()` at
  1321, `load_programs_with` at 1331-1360: the program-table seal at 1336-1348, the
  canonical-source loop at 1351, the guest-route checks at 1427, the guest-dependency branch
  at 1544. The shared accessor is `coverage/bindings.rs:104` `provenance()`; its other
  consumers are `mutations/source_auth/mod.rs:146`, `ledger/known_losses.rs:66`,
  `ledger/proofs.rs:103` and `guest.rs:2383` (the `c52-native-values` seal). **`provenance()`
  and every one of those consumers keep the baseline table.**
- The class fixture module: `crates/rum-stdlib/tests/suspension_oracle/class_lease_fixtures.rs`
  (766 final `3738e894…`, 41,447 B), declared in `tests/suspension_oracle.rs:52-53`.
- The five additions and the overlays are unchanged by this brief; the class table itself is a
  new tree file that the later source-A/B brief must carry as a declared addition.

## Deliverables

1. **Entry re-authentication.** Recompute the five tree digests (source `97eaeb45…`/5,810,
   armed `bd8b7548…`/5,810, disabled `b41d1873…`/5,810, 764 `0f8cbc32…`/5,812, 766
   `cce4cc03…`/5,815), the 199 live pins, the 13 OPEN Q25 rows, and the 768/769 evidence
   digests (overlays `73297529…`, additions `78106c66…`, class table `8c8b2232…`,
   `source_schema` `3faa2999…`, inputs `aa99638d…`). STOP on any mismatch.
2. **The scene.** Copy the 766 tree (excluding `.git` and `target`) to
   `.temp/brief-770/scene/`; verify its walk digest equals `cce4cc03…`/5,815. Install the
   class table **byte-identical** at
   `crates/rum-stdlib/tests/suspension_oracle/inventory/obligations/class-provenance.tsv`.
   Switch the loader's seal source to that table: the permitted source edits are the loader's
   seal read and, if needed, one dedicated accessor for the class table; `provenance()` and
   every other consumer stay on `provenance.tsv`, which stays byte-unchanged. Record the exact
   diff and the post-edit walk; the scene must differ from the 766 tree in **only** those
   files: the class table, the loader edit, and the accessor (if used).
3. **The one invocation.** From the scene, one bounded run:
   `cargo test --locked --offline -j8 -p rum-stdlib --test suspension_oracle class_lease --
   --nocapture --test-threads=8 --no-fail-fast`, with a timeout derived from the recorded
   2,961.635 s bound (never shorter). Do not run the lane twice; a red is a STOP with the exact
   failing rows, not a retry.
4. **The acceptance.** The class fixture module green: zero failed, the test count recorded,
   and the loader's check families unchanged (the canonical-source loop, the guest-route
   checks, the program-table seal; the guest-dependency branch still has zero C10 ids and its
   non-firing is recorded). The green must be the scene binary's: record the binary path under
   the scene's `target/`, the exact command, and the verdict lines.
5. **The authenticity checks (watched, record-based).** The scene differs from the red 766
   tree only by the declared files (the diff record); the class table's sha256 equals
   `8c8b2232…`; `provenance.tsv` in the scene equals the 766 byte; `source_auth`,
   `known_losses` and `proofs` still resolve their seals through the baseline accessor (read
   the call sites and state the mechanism); a green `AuthenticatedSource::source` is not the
   acceptance and is not substituted for the loader; any Q25 row marked closed or any
   importable receipt is a STOP.
6. **The report.** `rum-report/1` JSON at `.temp/brief-770/report.json` via
   `scripts/report.py`, sections: entry, scene, switch, invocation, authenticity, scope,
   budget. The summary states the test count, the loader verdict lines, the exact open facts
   (the `RunFailure::Native`-style failures if any; the 760 hash-only intermediate remains
   named). `report.py check 770` green.

## Hard rules

- **Scope: NONE in the live tree.** Work under `.temp/brief-770/` only; `.temp/brief-766/`,
  `.temp/brief-769/` and every accepted scene are read-only; no build in the live tree or in
  any accepted scene; no live edit of any kind.
- **No fixture weakening.** The class fixture file, `programs.tsv`, the five additions and the
  overlays are not edited; the loader's checks are not changed — only the seal source.
- Direct cargo inside the dispatch scope at `-j8` (no nested `scripts/memory-scope.sh`); tests
  `RUST_TEST_THREADS=8 --no-fail-fast`; one invocation; timing/perf serial.
- `.temp` only, never `/tmp`; no network; no new dependency; no new `unsafe`; no new program,
  lane, selection, policy or epoch label; no git; never read `~/.config/ccds/api-key`.
- Q25's thirteen rows stay named stops; 744/745/746 stay held; CM5 stays unauthorized.

## Budget

**focused 3,308.977 s + workspace 0 s + epoch 0**, carved from the stage reserve
(34,717.40778847017 → **31,408.43078847017 s**); the carve is the 766 fixture bound
2,961.635 s at 8 threads plus the 347.342 s image. Unspent returns; a projected overrun stops
for a director ruling — more is carved, never a scope cut. The scene's stop line is
`remaining < stack_1x` (cold samples of pending commands plus the 180 s close) or
`remaining < 3× the next command sample`.

## Report format

`rum-report/1` JSON at `.temp/brief-770/report.json` via `scripts/report.py`; sections as in
Deliverable 6; `summary` states the test count, the green loader verdict lines and the exact
open facts.
