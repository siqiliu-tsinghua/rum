# Brief 786 — G15-CM4 Stage D repair: the proof-closure re-derivation installed, then the render-repair continuation

Base: `90978e2a` (the c143 gate; brief 785 dispatched from it). You are the implementation
worker for this brief, not the project director. `CLAUDE.md` belongs to the director and is
not your instruction file. Follow `AGENTS.md` and this brief. Do not commit, branch, or push.

## Why this brief exists

Brief 784 (the render repair) stopped at its own stop rule: `validate_closure`
(`crates/rum-stdlib/tests/suspension_oracle/fixtures/obligations/proof_controls.rs:87-169`)
returns `Closure` after the authorized expected-array update. Brief 785 (the design-only
inventory, ruling 794) re-derived every check from the render's declared sites, calibrated
eleven falsifier cases, and drafted the replacement file at
`.temp/brief-785/draft/proof_controls.rs`, sha256
`5835c1004180a675cf5ec123c35fa0f9a3e219999cf0230abdb1dea26c4622eb`. This brief installs
that file on the armed and disabled images, proves the two proof tests green there under
cargo, and then runs brief 784's stages B–D unchanged on the same images.

## The carried state (re-measure it yourself; any drift is a stop)

The 784 images under `.temp/brief-784/images/` are the state brief 784's stage A left and
brief 785 measured, under the worker's own `tree_hash` (`.temp/brief-785/tools/common.py`:
`os.walk` with `EXCLUDES = {.git,target,.temp,__pycache__,.mypy_cache}`, sorted dirnames and
filenames, digest of `b"F\0" + relpath + b"\0" + content`):

- baseline `3d0b6829b400974b1f1e3d3ee42f0251929180996db127a015c85860cc2322fa` / 5,842
- armed `b11a687b4cab943ca629583fdc55203ea4097cb0ea1be80d741424c57dc4de58` / 5,851
- disabled `3ed28bddb1a74d1edddaec84247aca46e6e2865eea7f1f4503421084585b88f2` / 5,851
- real `57e3533e9be0d52bfb5296f54fdb384c53d1256b794880ade61fcaa3c10ec1ce` / 5,851
- debris 0 on each.

`proof_controls.rs` is `a3092419e6d4e87a3363c2804fed303a74a92abef24afb6dc5b0998716c15d35`
on live, baseline and real, and `0465ea9dc080caaac3cc85271279bd9db2851cdac1a29a669441cf7500033a18`
on armed and disabled (the one-line expected-array update). The F5 file
`crates/rum-runtime/src/cm740_probe/tests/cm743_route_a_falsifiers.rs` is
`406c7dc30f6726b4526de217704d5bc268a6fdb54025b9486df09f17bad4501f` / 12,598 B on armed,
disabled and real. `observer_policy_tests.rs` (F14) is
`d17e200b63713a37e933225867ec4ebdd5ad0124f7b6013d9362f8dc91418a4f` on live and all four
copies. The live tree carries the 23 inherited porcelain entries recorded at
`.temp/director/gate-c88/porcelain.txt`; do not run git in any form — use the record.

At the entry bar, on the armed image, the two proof tests are red:
`0 passed; 2 failed`, both `Err(Closure)` (`proof_tests.rs:19:30` and `:187:48`); the
replicated 18-check scan gives live and baseline all-pass and armed and disabled exactly
`builders, plan_state, lhs_from, stage_index, prepared_occ`. Re-run both before you edit;
either mismatch is a stop.

## Scope

Stage A edits only `crates/rum-stdlib/tests/suspension_oracle/fixtures/obligations/proof_controls.rs`
on the armed and the disabled image, both to one identical file. Do not edit live, baseline,
real, `proof_tests.rs`, `proofs.rs`, the F5 file, the observer file, or anything under
`.temp/brief-784/` and `.temp/brief-785/`. Every write you make goes under a new
`.temp/brief-786/`. Stages B–D then edit exactly what brief 784's stages B–D authorize.

## Stage A — install the re-derived closure and prove it (1,800 s)

1. Copy `.temp/brief-785/tools/` to `.temp/brief-786/tools/` and
   `.temp/brief-785/draft/` to `.temp/brief-786/draft/`, byte-for-byte (no retyping).
   Confirm `sha256sum .temp/brief-786/draft/proof_controls.rs` is
   `5835c1004180a675cf5ec123c35fa0f9a3e219999cf0230abdb1dea26c4622eb`.
2. Copy that file onto the armed image and onto the disabled image. After the copy both
   image files and the draft are one byte-identical file. Baseline, real and live stay
   `a3092419…`; a copy onto any of them is a stop. The re-derived checker returns
   `ClosedCallers` on the preimage population by design — do not "fix" that.
3. Replay the brief 785 falsifier calibration on the copy:
   `python3 -I .temp/brief-786/tools/derive.py`, with its logs landing in
   `.temp/brief-786/logs/`. Bar lines, copied from
   `.temp/brief-785/logs/stage-d-falsifiers.log` and required verbatim in the new log:
   `GREEN label=armed variant=Ok failing=-`, `GREEN label=disabled variant=Ok failing=-`,
   `COUNT6 label=plain-2-twin-4 span_plus=6 variant=Closure failing=stage_index count6_pin_would_pass=true`,
   `ORDINARY STRENGTH old_span_holds=true new_variant=Closure failing=ordinary`,
   `FALSIFIER SUMMARY red=10 green=1 mismatch=0 cases=11`,
   `PRECEDENCE order=ClosedCallers,PreparedSkip,Closure preserved=true`,
   `CONTROLS count=15 dropped=0 proof_tests=unchanged proofs=unchanged`,
   `STAGE D VERDICT stop=false`. A mismatch is a stop; never weaken a check or a falsifier.
4. Run the two proof tests on **armed** and on **disabled**, each with its own
   `CARGO_TARGET_DIR` under `.temp/brief-786/` (never shared across trees), a measured
   `pwd`, the tree hash in the log, `RUST_TEST_THREADS=8`, `-p rum-stdlib --test
   suspension_oracle --offline --locked --no-fail-fast -- --exact` and the full module
   paths `oracle::obligations::ledger::proof_tests::cm_proof_certificates_exact` and
   `oracle::obligations::ledger::proof_tests::cm_proof_negative_controls`. Bar on each
   tree: `2 passed; 0 failed` and the line
   `CM PROOF CONTROLS PASS controls=15 ordinary_branch_reachable=true Identity_positive_calibration=false validator_controls_credit=0 owners_changed=0`.
   Any other result, including a compile error, is a stop.

The installed file's dispositions, for your record (each equal or stronger than the check
it replaces; ruling 794): the TARGET array re-pinned to `LHS 268, CALL 625, CALL 923,
CALL 989`; `build_call_dispatch_plan` re-pinned to the ten measured sites (`LHS 918`,
`eval_call_prepare 697/715`, `call.rs 314/338`, `plan 42`, `holding 253/256/265/276`);
`plan_state` and `lhs_from` restated as normalized whole-argument checks that accept the
`(if crate::cm740_probe::driven() { … } else { … })` wrapper and reject an argument change;
`stage_index` restated as three increments in `resume_suspended_call_with_dispatch_plan`
and three in its `_cm740_driven` twin, no decrement and no assignment, never a count of 6;
`resume_prepared_call_dispatch_frame` re-pinned to `CALL 903, 1214, 1256`; `ordinary`
restated on the plain function body. Every other check is byte-identical in substance.
`proof_tests.rs` and `proofs.rs` need no change.

## Stages B–D — brief 784, unchanged, on these images

After stage A's green, execute `docs/dev/codex/brief-784-g15cm4-render-repair.md` from
"### Stage B — discovery and fingerprints (1,800 s)" through the stage D acceptance bar,
including its verification paragraph and its F16 duty. That text is the authority; the
carried requirements are restated here so this brief stands alone:

- **B** — four per-tree `CARGO_TARGET_DIR`s; the host-payload generators by full module
  path (`tests::ownership_graph::reconcile::regenerate_host_payload_inventory`,
  `::regenerate_host_payload_ledger_review`, `::regenerate_self_reaching_inventory`,
  `::regenerate_unknown_construct_inventory`, each `--ignored --exact` on armed); the
  population campaign with `--nocapture` on armed and disabled only, full path
  `oracle::obligations::tests::obligations_discovery_campaign`, never baseline; F13
  `regenerate_local_contract_fingerprints` on baseline, armed, disabled and live, each
  with its own target dir, after every `.rs` edit; `local_production_acceptance` in default
  mode and `--no-default-features` on each of those four trees.
- **C** — the writers in order: overlays start at 147, then the unconditional 13
  (population, `receipts.rs`, `logical-profile-required.tsv`, `registry.rs`,
  `recursion_inventory.rs`, `g7_continuation_contract.rs`,
  `rule_pattern_depth_contract.rs`, `read_inventory.rs`, `definition_store_sealing.rs`,
  `resume_evaluation_context.rs`, `proof_controls.rs`, `local-contracts.tsv`,
  `promoted.tsv`) plus conditionals whose bytes changed; additions 9; the emission chain
  152 recomputed from bytes; the class table's 201 `cases.rs` cells; mutants
  `source_schema`; the host-payload tsvs and `reconcile.rs` pins; the allowance verified,
  never hand-edited. The `proof_controls.rs` overlay cells are the sha256 of the installed
  bytes, recomputed by the writer, never hand-typed. Then the four rebuilt images, the
  twelve receipts with `rows=9` (a new script modeled on brief 779's), the six-case gate,
  and `cargo fmt --check`.
- **D** — `RUM_CM740_BASELINE` set to the rebuilt baseline; the default-mode workspace
  with the nine skips and exactly the ten red names (the nine sandbox loopback refusals
  plus `q25_p2379_ownership_proof`; any other red is a stop); the 71 ignored names
  set-identical to `.temp/brief-783/logs/measure.log`; the flag-on subset at
  `5 passed; 0 failed` and the class-lease fixtures; the F16 scratch-removal duty and the
  closing re-assert of the final tree hashes.

One amendment to brief 784: its stop "a proof row's identity/status/disposition/exclusion
changed, or `validate_closure` failing after the line update" is retired for the
`validate_closure` clause only. Stage A is the re-derivation. The replacement stop is a
proof test that is not `2 passed; 0 failed` on armed or on disabled. The rest of brief 784
stages B–D and its stop list stand.

A red whose only cause is a stale fingerprint, certificate, or position cell is an ordering
artifact: replay the generator, never weaken or hand-edit an inventory.

## Checkpointing and stops

Write `.temp/brief-786/PROGRESS.md` and a stage log after each stage. Stop, with the exact
evidence, on: any pin drift in the carried state; a cargo run whose measured `pwd`, tree
hash or target dir is not the pair the stage names; a copy of the draft anywhere other than
armed and disabled; a falsifier bar line that does not match; a proof test not at
`2 passed; 0 failed` on armed or disabled; a hand-typed derived cell; a red outside the
ten; an ignored set that is not the 71; a flag-on result other than `5 passed; 0 failed`;
any live edit; any edit outside this brief's scope; network, a new dependency, or `unsafe`.
A stop is a correct terminal state.

## Hard rules

No git in any form. No network. No new dependency. No `unsafe`. No committed-file edit
other than the `proof_controls.rs` install above. No sealed-artifact edit. No hand edit of
a derived file. No CM5. No 744/745/746. Q25's thirteen rows stay open. `.temp` only; never
`/tmp`; never read `~/.config/ccds/api-key`. One `CARGO_TARGET_DIR` per tree, always under
`.temp/brief-786/`.

## Budget

Focused **5,300 s** (A 1,800 / B 1,800 / C 1,700) + workspace **2,200 s** + epoch **0**
= **7,500 s**, with an explicit **3,500 s** top-up from the campaign allocation (user
ruling 2026-11-19; reserve 4,068.83478847017 s before the carve) → reserve after the carve
**68.83478847017 s**. Unspent returns; a projected overrun stops for a director ruling,
never a scope cut.

## Report format

`rum-report/1` at `.temp/brief-786/report.json`, stage `A-D / c145`; `summary`, `changes`
(every path you edit, with the images named separately), `disclosures`, `deviations`,
`questions`, checks and measurements with every verdict line quoted, and the final message
`scripts/report.py summary`. A DONE report has no FAIL row. The ten red names are a PASS
check whose expected value is that set; the 71-name ignored set is a PASS check. The
director gates from raw evidence.
