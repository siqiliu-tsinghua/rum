# Brief 780 — G15-CM4 Stage D repair: the deferred acceptance completion — the corrected F3 checker, the flag-on rerun, the workspace and the fmt state

**Implementation brief, staged (A–D), one gate.** It continues ruling 788 (gate c138) and
brief 779 (`80d18572`). Brief 779 carried the repaired install tree, refreshed `TODO.md`,
rebuilt the baseline and the three images, and ran the complete six-case source gate green
(stage C `ok=true`, 870.2 s); its stage D stopped correctly on F3's stale filtered-count
pin: the fresh flag-on class-lease run reports `197 passed; 0 failed` with **593** filtered
(the carried tree adds `cm_mutation_source_gate_result`, which does not match the filter),
while the receipt-diff checker hardcodes 592. This brief completes the deferred 774
acceptance: the corrected checker (a copy), the fresh flag-on run, the diff's PASS line, the
flag-off workspace, and `cargo fmt --check`.

## Goal

When this brief is done, in `.temp/brief-780/` only:

- The 779 artifacts re-authenticated: install/armed tree `e01d13cf…`, baseline `ac36bc81…`,
  disabled `9e682e49…`, real `4dd8b8a1…`, the six primary receipts, the class table
  `8c8b2232…`.
- `.temp/brief-780/receipt_diff.py`: a copy of `.temp/brief-774/receipt_diff.py`
  (`96393534…`) whose **only** changes are (i) the harness-line rule extracted into
  `result_accepted(line)` with the tree-independent prefix
  `test result: ok. 197 passed; 0 failed; 0 ignored;` (the filtered count recorded, not
  pinned), (ii) the selftest gaining three result arms — a 593-filtered line accepted, a
  196-passed line refused, a 0-passed line refused — and (iii) the selftest writing under
  `.temp/brief-780/tmp/receipt-selftest/`. Every other byte identical; the diff shows only
  those hunks; the 774 original is untouched.
- The fresh flag-on class-lease run on the armed image under
  `RUM_PROFILE_LEXICAL_SUBSTITUTION=1`, exit 0, `test result: ok. 197 passed; 0 failed; 0
  ignored; 0 measured; 593 filtered out` (the filtered count recorded), then the corrected
  checker printing `RECEIPT DIFF PASS rows=97 lost=0 new_miss=0 withheld_bad=0 digest=PASS`,
  with the three receipts (C5, P0492, P2299) quoted, and the selftest printing
  `RECEIPT SELFTEST PASS glued=true corrupt=false result=true`.
- One flag-off workspace run on the armed image (`--skip
  class_lease_positive_c5_default_profiled_finish --skip class_lease_positive_p0492`,
  `RUM_CM740_BASELINE` = the 779 baseline), every red and ignored test listed and classified
  against the 776 stage-D reference set; any other red — and any `source_auth` red — stops.
- `cargo fmt --check` on the armed image, red only on `cm740_probe/q25_ownership.rs` and
  `q25_p2379_proof.rs`.
- The 779 trees byte-unchanged after every run (the four tree hashes re-asserted at the end).

Q25's thirteen rows stay named stops; 744/745/746 stay held; CM5 stays unauthorized.

## Context pointers

- Ruling `.temp/director/ruling-788.md`; brief 779
  (`docs/dev/codex/brief-779-g15cm4-source-ab-completion.md`, committed `80d18572`), its
  report `.temp/brief-779/report.json` and logs (`stage-c.log` — the six-case gate;
  `stage-d.log`, `stage-d-receipt-diff.log`, `stage-d-receipt-selftest.log`).
- The 779 artifacts: `.temp/brief-779/install/` `e01d13cf…` (5,850 files),
  `.temp/brief-779/baseline/` `ac36bc81…` (5,842), `.temp/brief-779/images/{armed,disabled,real}`
  `e01d13cf…` / `9e682e49…` / `4dd8b8a1…`, `.temp/brief-779/receipts/` (six primary plus
  three negative receipts), `.temp/brief-779/bins/` (the six primary binaries).
- The checker: `.temp/brief-774/receipt_diff.py` `96393534…`, its baseline log
  `.temp/brief-772/logs/class-lease-fixtures.log` `b58bb401…`, its selftest's hardcoded
  directory `.temp/brief-777/tmp/receipt-selftest/` (F11 — the copy must not write there).
- The workspace reference set: `.temp/brief-776/logs/stage-d-workspace.log` (4,488 passed,
  85 failed, 71 ignored; the failing test names are the reference set — the environment's
  nine native ZeroMQ loopback refusals, the retention census's `unreconciled source file:
  …/cm740_probe/class_rows.rs`, `read_inventory`'s `_cm740_driven` readers, the
  definition-store call-site inventory, the recursion registries, the frame-root-trace rule,
  the resume-evaluation boundary, the typed-slot census, lp03/04/05/648 and the Q25
  selector/proof rows).
- The 779 stage-D run to reproduce: cwd `.temp/brief-779/images/armed`, `CARGO_TARGET_DIR`
  under the brief's own directory, `RUM_CM740_BASELINE` = `.temp/brief-779/baseline`,
  `RUM_PROFILE_LEXICAL_SUBSTITUTION=1`, the module filter `class_lease`.

## Scope

- You MAY write only under `.temp/brief-780/`: `tmp/`, `target/`, the script copy, the logs,
  the evidence and the report. The live tree, `.temp/brief-779/` (all four trees and the
  receipts), `.temp/brief-774/` (the checker is copied, never edited), `.temp/brief-776/`,
  `.temp/brief-777/` and `.temp/brief-772/` are read-only.
- No file of the 779 trees may change; no `target/` may appear inside them (the external
  `CARGO_TARGET_DIR` is mandatory); re-assert all four tree hashes at the end.

## Stage A — entry and re-authentication (300 s)

Re-authenticate, with line-quoted verdicts: the four tree hashes and file counts; the six
primary receipts' fields (`arm`, `baseline_tree=ac36bc81…`, `binary`, `codegen_units=1`,
`configuration`, `profile=release`, `tree`) and the four disabled/armed receipts' 147
overlay rows against the table; the class table `8c8b2232…`; the checker and baseline-log
digests (`96393534…`, `b58bb401…`). Any mismatch stops.

## Stage B — the corrected checker (200 s)

Copy `.temp/brief-774/receipt_diff.py` to `.temp/brief-780/receipt_diff.py` and apply
exactly the three ordered changes (the `result_accepted` extraction with the prefix rule; the
three selftest result arms; the selftest directory under `.temp/brief-780/tmp/`). Diff the
copy against the original and quote the hunks — nothing else may differ. Run
`--selftest` expecting `RECEIPT SELFTEST PASS glued=true corrupt=false result=true`.

## Stage C — the flag-on rerun and the diff (200 s)

One fresh flag-on class-lease module run on the armed image under
`RUM_PROFILE_LEXICAL_SUBSTITUTION=1` (the 779 stage-D invocation shape, bounded) expecting
exit 0 and `test result: ok. 197 passed; 0 failed; 0 ignored; 0 measured; 593 filtered out`;
record the filtered count as data. Then run the corrected checker against
`.temp/brief-772/logs/class-lease-fixtures.log` and the fresh log: expect
`RECEIPT DIFF PASS rows=97 lost=0 new_miss=0 withheld_bad=0 digest=PASS`, with the C5,
P0492 and P2299 receipt lines quoted.

## Stage D — the workspace, fmt and the closure (700 s focused + the workspace bucket)

- One flag-off workspace run on the armed image with the two named skips and
  `RUM_CM740_BASELINE` = `.temp/brief-779/baseline`; quote the result counts; list **every**
  red and ignored test by name and classify each against the 776 reference set. Acceptable:
  (i) the environment's nine native ZeroMQ loopback refusals; (ii) reds whose names are in
  the 776 stage-D reference set (the inventory-regeneration class above). Any other red, and
  any `source_auth` red, stops. The full workspace green is the render brief's bar.
- `cargo fmt --check` on the armed image; the red must be exactly
  `cm740_probe/q25_ownership.rs` and `q25_p2379_proof.rs` (the render brief owns their
  formatting with F5).
- Re-assert the four 779 tree hashes (`e01d13cf…`, `ac36bc81…`, `9e682e49…`, `4dd8b8a1…`)
  and the absence of any `target/` inside them.

## Verification and acceptance

The brief passes only with all of: the entry re-authentication; the corrected checker's diff
showing exactly the three ordered hunks and its selftest green on all three arms; the fresh
run's `197 passed; 0 failed` with the filtered count recorded; the diff's exact PASS line
with the three receipts quoted; the workspace's fully classified red list with nothing
outside the reference set; the fmt red exactly on the two Q25 carriers; and the four tree
hashes unchanged. Quote every verdict line verbatim; disclose every deviation.

## Checkpointing and stops

Checkpoint after every stage (`.temp/brief-780/PROGRESS.md`, logs under
`.temp/brief-780/logs/`). Stop, with the exact evidence, on: any 779 tree or receipt pin
mismatch; a checker diff beyond the three ordered hunks; a selftest arm that fails to
separate accepted from refused; a flag-on run that does not report `197 passed; 0 failed`
or that exits non-zero; a diff that does not print the exact PASS line; a workspace red or
ignored test outside the reference set; a `source_auth` red; a fmt red outside the two Q25
carrier files; a changed 779 tree; or a projected overrun. A stop is a correct terminal
state; never weaken a rule, skip a file, or edit the 774 original.

## Hard rules

Never edit `.temp/brief-774/receipt_diff.py` or `.temp/brief-772/logs/*`; never widen the
checker's rules beyond the three ordered hunks; never edit, rebuild or re-anchor any 779
tree; never write into `.temp/brief-777/`; never touch the live tree or the sealed scene;
never weaken `Preimage`, `Postimage`, `BaselineSeal`, `Inventory`, `OverlaySet` or
`UnlistedFile`; no new `unsafe`; no default-path cost; no network; no new dependency; no git
of any kind; `.temp` only; never `/tmp`; never read `~/.config/ccds/api-key`; Q25 stays
open; 744/745/746 stay held; CM5 stays unauthorized.

## Budget

Focused **1,400 s** (A 300 / B 200 / C 200 / D 700) + workspace **2,700 s** + epoch **0**,
carved from the stage reserve after brief 779 (10,240.08378847017 →
**6,140.08378847017 s**; 779 charged 3,140.8 s of its focused 3,250 and returned 109.2 s;
its workspace 2,700 s did not run). Unspent returns; a projected overrun stops for a
director ruling, never a scope cut.

## Report format

`rum-report/1` at `.temp/brief-780/report.json`, appended as stage `A / c139`; `summary`,
`changes`, `deviations`, `questions`, every verdict line quoted, and the final message
`scripts/report.py summary`. The director gates from raw evidence.

## Gate c138 note (for the reader)

Brief 779's stages A–C are accepted from raw evidence: the install tree's delta against the
carry is the one declared refresh, the baseline and the three images are constructed exactly,
the six receipts are right, and the six-case gate is green in both configurations. Its stage
D stopped correctly on F3's `592 filtered out` suffix — a pin I copied from the 774 scene; it
was already stale when brief 777 added `cm_mutation_source_gate_result`, which does not match
the `class_lease` filter. Ruling 788 accepts 593 as this tree's count and corrects the bar to
the tree-independent `197 passed; 0 failed; 0 ignored` plus the 97-receipt digest, with the
filtered count recorded, not pinned. After this brief: the probe and hot-path A/B, the
deterministic render (F5, the Q25 formatting, the inventory regeneration, the full workspace
green, the source-gate re-run), the re-freeze, the fresh epoch, the Stage C/C10 readmission
and a further 744 restart, each gated.
