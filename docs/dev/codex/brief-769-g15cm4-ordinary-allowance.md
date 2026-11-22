# Brief 769 — G15-CM4 the declared hook-site ordinary-body certificate (scene-only, no build)

**Director-dispatched, one stage, one gate, no production change.** This continues ruling 777
and brief 768's accepted STOP. Brief 768 closed the byte chain, the 147-row overlays, the five
additions, the class-anchored loader table, the `mutants.tsv` cell and the whole-tree
declaration, and stopped on the ordinary-body certificate: the 14 rows it named are the 759
contract's own hook sites, which my brief's allowance under-enumerated. This brief completes
that certificate with a declared, byte-exact, fail-closed allowance, in `.temp/brief-769/`
only. No build, no test, no binary execution, no live edit.

## Goal

When this brief is done, in `.temp/brief-769/` only: the 759/760 ordinary-body certificate
**passes** over the six 759-covered files with the declared hook-site allowance — 12 existing
bodies whose change is exactly the declared added/removed lines, 2 contract-declared helpers,
the q25 module's functions and the proof file's functions named — checked byte-for-byte with
three falsifiers watched failing; and the whole 768 artifact set is re-emitted **byte-identical**
(overlays, additions, class table, `mutants` cells, `source_schema`, chains, tree declaration,
KATs). The allowance is a constant table in the generator, never inferred from the observation
it checks.

## Context pointers

- Read `AGENTS.md`; ruling `.temp/director/ruling-777.md`; brief 768's
  `.temp/brief-768/report.json` and `logs/ordinary.log` (the 14 stops and the calibration);
  brief 759's Deliverable 2 (the contract clauses: the common raw-insertion registration, the
  driver-scoped offer, the one-use `Offered → Claimed` consumption, retirement on completion /
  abandon / cancel / decode fault / error / unwind / episode end, the withheld-only seam).
- **The six files and their endpoints** (749 armed → 759 output): `lib.rs`
  `9cd4fa21…` → `15d91070…`; `core.rs` `380e6dec…` → `e618623c…` (then 760 → `7766b20e…`);
  `core/driver.rs` `34a37717…` → `7f800a7f…`; `rules/cases.rs` `bd5ec2f2…` → `2d915381…`;
  `rules/resume_frames.rs` `498639bc…` → `8e996d07…`; `tests/suspension_oracle.rs`
  `993b3f33…` → `7dd5b684…`. The copies: `.temp/brief-749/attempt-7/copies/armed`,
  `.temp/brief-759/copy`, `.temp/brief-760/copy`.
- **The 14 sites and the checker's own reading.** `class_forms.check_ordinary`
  (`.temp/brief-766/generator/class_forms.py`, `fn_split` under
  `brief-742/continuation-88/`) hashes each whole function span and skips any name containing
  `cm740_driven`. It stopped: `lib.rs` `begin`/`end`/`arm_source`/`pause`; `core.rs`
  `insert_suspension_frame`/`insert_typed_erased_frame_named` (plus the new
  `insert_suspension_frame_q25`); `core/driver.rs` `drive_frames`; `rules/cases.rs`
  `resume_collect_string_cases_from_index`/`resume_collect_literal_string_cases`/
  `drive_string_cases_output_result`; `rules/resume_frames.rs` `abandon`/`resume` (plus the new
  `challenge_equal_siblings`). The 71 `named_new` are the q25 module's 40 functions and the
  proof file's 31.
- **The differences, as the gate read them** (you re-derive them, do not trust these counts):
  `lib.rs` 12 added / 0 removed; `core.rs` 45/5, the removals the four-line
  `record_descendant_frame` call re-expressed with the identical callee and arguments;
  `core/driver.rs` 6/0; `rules/cases.rs` 39/23, the `resume_or_suspend!` block de-sugared — its
  every call site passes the literal `Err(EvalError::InputRequired { request })`, whose macro
  arm is exactly `insert_suspension_frame($frame); Err(InputRequired { request })` — plus the
  `None` handoff argument at the ordinary call sites; `rules/resume_frames.rs` 34/0;
  `tests/suspension_oracle.rs` 2/0, a module declaration.
- **The 768 artifacts to reproduce** (their digests): overlays
  `73297529dff14b93ab70129ce81b6dd9f17cd00171028e46902e5a8cef89b739`; additions
  `78106c66b98152e6c85f866bf69942acd7458b34c126c64b7ce24482280890c0`; class-provenance
  `8c8b22321d5434a4a3a38365eb533c27a1bfc36353ccbb5b72fa0d156e6cfcb7`; mutants-reanchored
  `17355964031d20a7…`; `source_schema` `73297529…`; and the phase-log verdict lines (the
  152 closed chains, the tree declaration 147 + 5 + 5,663, the ordinary calibration 840/118).
  The 768 scene, its generator and its evidence are **read-only**.
- **Consumers to keep unchanged.** `guest::programs` (`fixtures/obligations/guest.rs:1319-1356`);
  `source_auth::verify` (`mutations/source_auth/mod.rs:106-216`); `schema::file_postimage`
  (`schema.rs:28-44`); `records.rs:287-291`. None of them is touched: no Rust changes at all.

## Deliverables

1. **Entry re-authentication.** Recompute the five tree digests and counts
   (source `97eaeb45…`/5,810, armed `bd8b7548…`/5,810, disabled `b41d1873…`/5,810, 764
   `0f8cbc32…`/5,812, 766 `cce4cc03…`/5,815), the 199 live pins, the 13 OPEN Q25 rows, and the
   768 evidence digests as inputs. STOP on any mismatch.
2. **The allowance table.** `allowance.tsv` in the scene: header `path step fn kind added
   removed contract`, one row per site — the 12 existing bodies, the 2 new helpers, the q25
   module (file-level) and the proof file (file-level). `kind` ∈ {`insert`, `refactor`,
   `desugar`, `new-helper`, `module`, `proof`}. For each of the 12 bodies the row carries the
   exact declared added-line count, removed-line count and the sha256 of the newline-joined
   added/removed line sets; the contract column quotes the brief-759 clause the site serves.
   The declaration is a **constant table in the generator source**; it is compared against
   the observation, never derived from it at check time.
3. **The per-site byte-exact check.** For each of the 12 existing bodies: take the same whole
   function span `class_forms.check_ordinary` takes; assert that removing exactly the declared
   removed lines and inserting exactly the declared added lines at their declared positions
   reproduces the final text byte-for-byte; any other difference is a STOP with the offending
   line. Shape rules enforced mechanically: every added line of an `insert` site contains a
   q25 reference (or is a declared withhold-seam statement of the `pause` site); a `refactor`
   site's removed call and added call name the same callee with the same argument expressions;
   the `desugar` site's removed text contains the `resume_or_suspend!` invocation and its
   added text contains `insert_suspension_frame` and the re-raised `Err(EvalError::
   InputRequired`; a `new-helper` is absent from the baseline and present in the final under
   its declared name. The q25 module's 40 and the proof's 31 functions stay named. The 760
   `core.rs` step keeps its recorded named repair (`Arc<F>` coercion,
   `e618623c…` → `7766b20e…`).
4. **Three falsifiers, each watched failing.** (a) A mutated declared insertion (one byte
   changed) is rejected; (b) an undeclared line injected into a body is rejected; (c) a
   declared insertion deleted is rejected. Calibrate the checker as 768 did on the 766
   producer set (`ORDINARY BODY PASS functions=840 shifted_paired=118`) and keep that
   falsifier.
5. **The re-emission, byte-identical.** Regenerate the full 768 artifact set in the new scene
   and assert each digest equals the 768 value above (overlays, additions, class-provenance,
   mutants-reanchored, `source_schema`, the 152-chain `CERTIFICATE PASS`, the tree
   declaration, the KATs), with the ordinary phase **PASS `files=6 stops=0`** and the
   allowance rows counted. Record the generator's own two digests (the 768 original and the
   769 copy) and the recordings' input digest; the only permitted difference from the 768
   artifacts is the certificate/ordinary logs' content and the report.
6. **The report.** `rum-report/1` JSON at `.temp/brief-769/report.json` via
   `scripts/report.py`, sections: entry, allowance, sites, falsifiers, certificate,
   re-emission, budget. The summary states the zero-unknown result, the artifact identities,
   and the exact open facts (the hash-only 760 intermediate remains named). `report.py check
   769` green.

## Hard rules

- `Scope: NONE` in the live tree: work under `.temp/brief-769/` only; `.temp/brief-768/` and
  every accepted scene are read-only; no build, no test, no binary execution; no new Rust; no
  live table, inventory, generator, render, selection or epoch edit; `.temp` only, never
  `/tmp`; no network; no new dependency; no git; never read `~/.config/ccds/api-key`.
- The allowance is declared, not inferred; no hand edit of any generated artifact; every
  number re-derived from the raw recordings.
- No new `unsafe`; no default-path cost; no new program, lane, selection, policy or epoch
  label; no receipt is importable; Q25's thirteen rows stay named stops; 744/745/746 stay
  held; CM5 stays unauthorized.
- A wrong derivation in a calibration case stops the brief; an undeclared body difference is
  a named STOP with the exact offending line, and the remaining rows continue.

## Budget

**focused 900 s + workspace 0 s + epoch 0**, carved from the corrected stage reserve
(34,728.69578847017 → **33,828.69578847017 s**); unspent returns; a projected overrun stops for
a director ruling, never a scope cut. No build is authorized. The scene's stop line is
`remaining < stack_1x` (cold samples of pending commands plus the 180 s close) or `remaining <
3× the next command sample`.

## Report format

`rum-report/1` JSON at `.temp/brief-769/report.json` via `scripts/report.py`; sections as in
Deliverable 6; `summary` states the zero-unknown result, the artifact identities and the exact
open facts.
