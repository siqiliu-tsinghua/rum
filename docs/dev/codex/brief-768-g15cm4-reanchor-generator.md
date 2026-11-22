# Brief 768 — G15-CM4 certified re-anchoring generator: the two-anchor tables, the five additions and the certificate (scene-only, no build)

**Director-dispatched, one stage, one gate, no production change.** This continues ruling 776 and
brief 767's accepted reconciliation. The committed byte-pinning inventories are baseline-derived;
the accepted class-form pipeline changed their subjects. Brief 767 delivered the complete
zero-unknown inventory and the strategy (certified generated re-anchoring, fail-closed). Ruling
776 decides the mechanism its drafts missed: **one pin table cannot serve both consumers.**
`guest::programs` hashes the working tree against the provenance seals
(`fixtures/obligations/guest.rs:1319-1356`); `source_auth::verify` hashes the **baseline snapshot**
against the same seals and checks the overlay preimages against the baseline
(`mutations/source_auth/mod.rs:141-187`), and its whole-tree key-set equality rejects extras
(`:189-190, :211-212`). So: **two anchors, two generated tables, one certificate.** This brief
builds the prescribed generator and its evidence, in `.temp/brief-768/` only. No build, no test,
no binary execution, no live edit.

## Goal

When this brief is done, in `.temp/brief-768/` only: a generator that **re-derives, from the
recorded chain and nothing else**, (1) the per-file certificate for every transformed file and
every addition, fail-closed at every link; (2) the 759/760 ordinary-body certificates the 766
certificate lacks; (3) the re-anchored overlays table (147 rows, both arms, preimage = the live
pin); (4) the class-anchored loader table (the provenance row identities with the certified final
sha256 cells for exactly the 60 changed paths); (5) `mutants.tsv`'s `source_schema` cell; (6) the
declared five-addition set; (7) the whole-tree declaration against the constructed baseline. The
generator is idempotent, its inputs are digested, and no output is hand-edited.

**Row-level stops.** A `stop` verdict is a row result with the exact missing fact; it does not halt
the remaining rows. Only a calibration failure or an unclosable chain link stops the brief.

## Context pointers

- Read `AGENTS.md`; brief 623's "Campaign invariants"; rulings
  `.temp/director/ruling-775.md` and `.temp/director/ruling-776.md` (the decision this brief
  implements); brief 767's `report.json` and `logs/{inventory,disposition,certificates,chain}.log`.
- **The accepted chain (read-only).** `.temp/brief-749/attempt-7/copies/{source,armed,disabled}`
  (trees `97eaeb45…`, `bd8b7548…`, `b41d1873…`; the render changed 144 files on each arm; the arms
  differ only on 3 files); `.temp/brief-759/evidence/{preimages,postimages}.tsv` (the six covered
  byte endpoints); `.temp/brief-760/evidence/edits.json` (the q25 move and `core.rs`
  `e618623c…` → `7766b20e…`); `.temp/brief-766/copy` (**the emitted class tree, 5,815 files, tree
  `cce4cc03dc72dd1b35488bc7bb846619d8b916772930336a0166aada0af677ce`** under the scene walk
  `F\0<rel>\0<bytes>`, sorted subdirs/files, excluding child dirs `target`, `.temp`,
  `__pycache__`, `.mypy_cache`, `.git`) with `evidence/class-forms.json` (24 inverses) and
  `logs/reemit-3.log` (the verdict lines).
- **The baseline (read-only).** The live tree: `provenance.tsv` `965eca10…` (the 199 source pins
  all equal to live), `overlays.tsv` `22751919…` (135 rows; the 749 source copy's table is the
  scene `5c3b02a3…`), `manifests.rs` 9 pins (`pre` = live), `mutants.tsv` 151 rows
  (`source_schema` `22751919…`). The eight live paths that differ from the 749 source copy
  (`CLAUDE.md`, `TODO.md`, `mutants.tsv`, `overlays.tsv`, briefs 623/743, two director `.d.ts`)
  are none of them pins or overlay subjects. Twenty-one `docs/dev/codex/brief-74x-76x.md` files
  postdate the 749 source copy.
- **The measured disposition.** 60 of the 199 provenance-pinned paths differ in the emitted tree
  and are exactly the render-changed pinned files: 45 bytes equal both 749 arms, `segments.rs`
  equals the armed byte only, 13 were emitted past the render, and
  `crates/rum-stdlib/src/strings/patterns/search/rules/cases.rs` is the frozen `2d915381…`
  (live `627c97d5…`, both arms `bd5ec2f2…`, 764/766 `2d915381…`). The computed postimages match
  the 749 arms on 100 of the 135 committed overlay rows; the same 35 miss both arms. Nine render
  files lie outside the committed 135; three emission files lie outside it too. The five
  additions are the three 766 class files plus `cm740_probe/q25_ownership.rs` and
  `tests/suspension_oracle/q25_p2379_proof.rs`. The three arm-differing constants:
  `crates/rum-runtime/src/lib.rs` `ENABLED: bool = !cfg!(disabled) && !cfg!(live)` vs `false`,
  `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs` `if true` vs `if false` at the CM4
  copied-carrier omit tail, and
  `crates/rum-runtime/src/runtime/profiling/logical/segments.rs` `if true` vs `if false` at
  `cm740_c10_arm_cm740_driven`.
- **Consumers to keep unchanged.** `guest.rs:1319-1356` (the loader, on the working tree; the
  class-anchored table is the one it will later read, but **this brief does not change any
  Rust**); `source_auth::verify` (`mod.rs:106-216`) on the baseline; `schema::file_postimage`
  (`schema.rs:28-44`, table-driven); `mutations/schema.rs:29`; `records.rs:287-291`.

## Deliverables

1. **Entry re-authentication.** Recompute the five tree digests above, the 766 copy file count
   (5,815), the 764 copy (`0f8cbc32…`, 5,812), the five additions absent from the live tree, the
   eight drift paths, the 21 docs, and the 199 pins against live. STOP on any mismatch.
2. **The certificate.** For each of the 147 transformed files (the 144 render files plus
   `suspension/typed/chain.rs`, `suspension/typed/witness.rs`, `tests/suspension_oracle.rs`) and
   each of the five additions: the ordered chain of byte endpoints (live pin → 749 arm → 759
   postimage → 760 `core.rs` step → 766 inverse → final), each digest and its evidence citation,
   and proof that each step's preimage equals the previous postimage. Fail closed if a link does
   not close. Record `cases.rs`'s history explicitly. For the three arm-differing files record
   both columns: the armed byte is the final; the disabled byte is the final with the one
   recorded constant reverted (single-occurrence exact replacement, fail closed if the constant is
   not found exactly once) — this is the only arm difference the 749 evidence allows.
3. **The 759/760 ordinary-body certificate.** For the six 759-covered files and the 760 `core.rs`
   step: parse the baseline and the final bytes; every non-driven function's normalized body
   digest present and equal; new functions only those the 759/760 contract declares (the q25
   module and its proof), each named. The emitter's ordinary-body checker may be reused, not
   trusted blind: calibrate it on the 766 `ORDINARY BODY PASS functions=840 shifted_paired=118`
   producer set and on a known-different falsifier. Fail closed: a body that differs without a
   named reason is a STOP.
4. **The re-anchored overlays table (147 rows).** Header `file preimage disabled_sha256
   armed_sha256 disabled armed`; preimage = the live file digest (equal to the committed
   preimage cell for the 135); armed = the certified final byte; disabled = the armed byte for
   the 144 non-arm-differing rows and the reverted-constant byte for the three; hex payloads
   decode to the bytes and match their sha columns. `mutants.tsv`'s 151 `source_schema` cells =
   sha256 of the generated table. **The five additions are not rows here**: emit a separate
   `additions.tsv` with header `file sha256 bytes arm`, five rows, each the emitted file's byte.
5. **The class-anchored loader table.** Same rows, ids, paths, counts and dependency cells as
   `provenance.tsv`, byte-identical except the sha256 cells, and changed only for the 60
   certified paths (each equal to the certified final byte). Assert: 18,203 rows, the same
   header, zero identity-cell differences, exactly 60 paths differ, and the two `lifecycle.rs`
   rows still share a digest. Name it `class-provenance.tsv` in the scene; the loader will read
   it in the next brief. Record its digest.
6. **The whole-tree declaration.** Construct the A/B's baseline recipe on paper and assert it
   mechanically where possible: the ordinary program bytes from the certificate plus the emitted
   tree's inventory and non-transformed files; differences from the emitted tree are exactly the
   147 overlay files plus the five additions; the eight drift paths and the 21 docs are identical
   on both sides; no other path differs. `source_auth::verify`'s whole-tree rules
   (`mod.rs:179-212`) are quoted as the consumer this declaration must satisfy.
7. **The generator and the evidence.** One prescribed script (or a small set) that regenerates
   every artifact from the recordings, idempotent, with its inputs digested and its calibration
   KATs (a row re-derived two ways; a deliberately corrupted chain link rejected). The evidence
   package, the budget ledger, and `scripts/report.py check 768` green.

## Hard rules

- `Scope: NONE` in the live tree: work under `.temp/brief-768/` only; the live tree and every
  accepted scene are read-only; no build, no test, no binary execution; no new Rust; no live
  table, inventory, generator, render, selection or epoch edit; `.temp` only, never `/tmp`; no
  network; no new dependency; no git; never read `~/.config/ccds/api-key`.
- No hand edit of any generated artifact; the generator is the only writer. Every number is
  re-derived from the raw recordings; brief 767's counts are claims to check.
- No new `unsafe`; no default-path cost; no new program, lane, selection, policy or epoch label;
  no receipt is importable; Q25's thirteen rows stay named stops; 744/745/746 stay held; CM5 stays
  unauthorized.
- A wrong derivation in a calibration case stops the brief; an unclosable chain link is a named
  STOP with the exact missing fact, and the remaining rows continue.

## Budget

**focused 3,000 s + workspace 0 s + epoch 0**, carved from the corrected stage reserve
(34,735.78278847017 → **31,735.78278847017 s**); unspent returns; a projected overrun stops for a
director ruling, never a scope cut. No build is authorized. The scene's stop line is
`remaining < stack_1x` (cold samples of pending commands plus the 180 s close) or `remaining < 3×
the next command sample`.

## Report format

`rum-report/1` JSON at `.temp/brief-768/report.json` via `scripts/report.py`, sections: entry,
certificate, ordinary-bodies, overlays, additions, class-table, tree-declaration, generator,
budget. The summary must state the zero-unknown result, the artifact digests, and the exact open
facts.

## Gate c127 (2026-11-23) — accepted STOP; the ordinary-body allowance is extended

Run: `grk` (grok-4.7, effort xhigh), unit `brief-768-grk-20261123-062527-985714`,
session `866a6861-a6b8-464f-b962-fce463fc348b`, 06:25:33 – 07:00:54 CST,
`status=success turns=90 cost_usd=6.128198 refused=0`, scope peak 536,285,184 B,
rc 0; scene `.temp/brief-768/` (16.9 MB); `report.py check 768` OK, 17 citations;
13 PASS / 1 FAIL checks, 1 PASS measurement, 2 deviations, 0 questions, changes 0.

Verified: the five tree digests and file counts, the 199 live pins, the eight drift
paths and 22 docs, the five absent additions, the 13 Q25 OPEN rows; 152 closed byte
chains (147 transformed + 5 additions, finals equal to the 766 bytes); `cases.rs`
history full and matching; 147 overlay rows with hex/sha agreement, 147 live
preimages, 147 armed = 766 bytes, 144 armed = disabled and exactly the three arm
files reverted by the one recorded constant; the five additions byte-equal to 766;
the class table 18,203 rows with exactly 60 changed paths and 9,029 rows whose only
changed cell is the sha256, all equal to the 766 bytes, `provenance.tsv` unchanged;
the 151 `source_schema` cells equal to the generated overlays; the whole-tree
declaration 147 + 5 + 5,663; the ordinary calibration 840/118 and its falsifier;
the KATs; and an in-place generator rerun reproducing every substantive artifact
byte-identically (only budget timing lines differ), so the evidence is genuine.
Scope clean (0 live writes, 0 scene writes, 23 inherited git entries).

Findings: **F0** — the 14 ordinary-body stops are the 759 contract's own hook sites
(12 existing bodies: the four `lib.rs` arm-lifecycle hooks, the two `core.rs`
insertion registrations, `drive_frames`, the three `cases.rs` string-case sites,
`abandon`/`resume`) plus two contract-declared helpers (`insert_suspension_frame_q25`,
`challenge_equal_siblings`); my brief's allowance named only the q25 module and the
proof, so the worker correctly failed closed. The diffs are pure q25 insertions
except the `core.rs` call re-expression (identical callee and arguments) and the
`cases.rs` macro de-sugaring (identical to the macro arm taken by a literal
`Err(InputRequired)` call site). **F1** — the arm columns live in the overlays table,
not the certificate log; substance verified. **F2** — carried: the A/B's constructed
baseline must be rebuilt from installed bytes, with the eight drift paths asserted,
or `mod.rs:211-212` fires. **F3** — the hash-only 760 intermediate and the 22-doc
recount are accepted.

Budget: 768 charged 7.087 s of 3,000; unspent 2,992.913 s returns; reserve
34,728.69578847017 s. Q25 stays unwaived. Next: brief 769, the declared hook-site
ordinary-body certificate (scene-only, no build, focused 900 s), then the fixture
brief.
