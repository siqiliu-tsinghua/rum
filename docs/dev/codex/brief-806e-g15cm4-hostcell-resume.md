# Brief 806e — G15-CM4 D.2: the host-cell admission and the resumed acceptance

Read briefs 806, 806a, 806b, 806c and 806d first. This continuation supersedes 806d's Stage A
checkpoint rule only (the two host cells) and extends 806d's Stage D whole-tree allowance; the B2
declaration and every other bar stand. No tag, no network, no new dependency, no new `unsafe` in
`rum-core`/`rum-runtime`. Scratch under `.temp/brief-806/`; never `/tmp`. No git.

## Why this brief

The 806d attempt stopped at Stage A on two checkpoint cells under
`tools/claude-mod/rum-director/.claude-plugin/types/`. The drift is host-side and was verified by
the director: Claude Code rewrote its own bundled type definitions at the restart of 14:29:33/34
(before the dispatch at 14:31:30); both paths are untracked and git-ignored
(`tools/claude-mod/rum-director/.gitignore:2`), and the rendered role trees still hold the old
bytes. The STOP is correct; ruling 843 admits the two cells as host-owned. Everything 806d
authenticated in Stage A stands and is reused: the four role roots, the ten outputs, the seven
tables reread at the nine-source hashes, the 190-row map, the B2 line with its four declared
cells and six controls, the presentation/fmt receipts and the 340-artifact seal
(`.temp/brief-806/finish-d/`, seal `447bdddc…`, 939 citations).

## The host-cell rule

- **Host-owned paths** (reported, never gated; the live tree keeps what the harness wrote):
  - `tools/claude-mod/rum-director/.claude-plugin/types/claude-code/index.d.ts` —
    898,749 B, `c41b7f1ac2fac7a6b90bc50b7a78ad1dc791778a71cedcd87dd3b34c75b89748`, mode 0644.
  - `tools/claude-mod/rum-director/.claude-plugin/types/claude-code-tools/index.d.ts` —
    208,996 B, `7bebc82f545d755f8c85d0b7f340fa70fc622ad1912f3ae7e281c2142925842c`, mode 0644.
  At entry and at exit print one `HOST CELL path=… bytes=… sha256=… mode=… mtime_ns=…` line per
  path; a change there (including mode or mtime) is reported and does not stop. Do not copy them
  back to the rendered bytes.
- **Everything else** keeps the byte rule: each non-host path of the 806d exit census
  (`finish-d/live-entry.json`, 5,889 cells of which these two are host cells) must equal its
  recorded sha256, bytes, mode and mtime_ns; the only admitted addition is this brief, hashed at
  entry. The live tree at entry is **5,890 paths** (5,888 non-host + 2 host). Any other addition,
  removal or cell delta stops with the cell. This rule governs Stages A, D and F alike.

## Stage A — re-entry (200 s)

Confirm the retained checkpoint from the sealed receipts — the four role roots
(`baseline cb9205a6…`/5,842, `real dda9146e…`/5,851, `disabled 30cd48fc…`/5,851,
`armed df0db63c…`/5,851), the ten outputs, the seven tables, the 190-row map (`bfb2fe15…`,
install pairing `8f7973e3…`, identity rows 35/45/51/170, the ten changed rows
160/164/165/166/171/172/182/183/184/186), the B2 record (`stage-B2.json`, 310.423 s; the exact
line `SCENE PROJECTION PASS rows=126 diffs=101 extensions=25 shapes=27 owners=15 hot=21
missing=0 unmatched=0 ws=0`; four `DECLARED CELL PASS`; six `CONTROL PASS`; attribution sha
`487dfbb3…`), the presentation 0.750 s and fmt 2.762 s receipts, and the seal — then check the
live census under the host-cell rule. The partial lib job
(`d2/acceptance/jobs/lib/raw.log`, 3,594 lines, no verdict) stays discarded. Do not redo the
render or B2.

## Stage C — the acceptance on the rendered tree (10,000 s)

As briefs 806b/806c, restarting at the lib gate: `cargo fmt --check` is not rerun (receipt
retained); the **lib suite 1,115 passed / 0 failed / 58 ignored**; the class gate under the live
class-gate env (flag-on, no `RUM_CM740_ADDITIONS`) **197/0/0**; the fourteen selectors with the
lane bind (`--ro-bind …/lane/p0543-tree-98c913453… …/stable-probe/active`) **14/0/0**; the eight
mutation/observer selectors **8/8**; the census selector 1 passed with the dump **`4b5fcd37…`**
and the `ACCEPTANCE_BASELINE` line with `PUBLIC_WRAPPER=16`; the source-auth control with
`RUM_CM740_ADDITIONS` = the rendered scene root (not the 797 image) green; the F48 pair
**2/0/0**. Every focused receipt is retained with its raw log; a new red or an unexpected pass
stops with the names. The full `cargo test --workspace` stays the director's — hand the rendered
tree over with the focused receipts and a clean scope report.

## Stage D — the install and the re-freeze (6,000 s)

As briefs 806a/806b: install the 190 rows with the 797 semantics (every live parent rechecked,
the atomic copy, the post-copy hash); the changed count is reported with its pairing, not pinned
to 18. Run the re-adopted verifier (rendered pins per ruling 840, sealed original untouched)
with its whole-tree delta admitting `TODO.md`, the declared `docs/dev/codex/` extras and the two
host cells — each declared with path, size and hash; its `R4 LIVE VERIFY PASS …` line states
`content_delta=3` with the host cells named. Any other delta stops. The crates subtree must equal
the rendered armed image exactly. Re-freeze the scene and the counters.

## Stage F — the report (800 s)

The `rum-report/1` DONE report for brief 806: the render's deltas, the four declared cells, the
row-170 correction, the map update, the install and verify lines, the acceptance receipts, the
host-cell record and the charge; the seal, `python3 -I scripts/report.py check 806`, the detached
summary, and the exit audit (the five admitted briefs, the render's installed outputs with their
produced pins, the two host cells declared, zero worker source edits outside the render's
outputs; zero unexplained).

## Charge

17,000 s of the 24,541.643 s remaining: A 200 / C 10,000 / D 6,000 / F 800. Every stage inside
its box; a box that would overrun stops at the deadline with the exact missing measurement. The
7,541.643 s reserve stays unallocated.

## STOP rules

As brief 806d: any non-host checkpoint cell mismatch, any scene cell outside the declared union,
any new red or unexpected pass, or any path delta outside the admitted set and the render's
declared outputs stops with the cell. Q25's thirteen named STOPs stand (P0214, P0216, P1070,
P2348, P2354, P2379, P2381, P2385, P2387, P2409, P2411, P2416, P2418); 744/745/746 held; CM5
unauthorized; no tag; no network; no new dependency; no new `unsafe`; `.temp` only, never
`/tmp`; no git.
