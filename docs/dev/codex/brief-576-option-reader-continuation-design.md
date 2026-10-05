# brief-576 — design: option readers that resume into their builtin (group G5)

`Scope: NONE — no production change.` Design-only, with one declared scratch
test (see Scope). **Do not run `git` in any form**, including before you have
read this brief.

## Goal

Group G5 of the suspension-continuation branch: an option value can be
delayed (`Heads :> (…; InputString["p: "]; True)`) and is evaluated when the
builtin reads it. If that evaluation asks for input, the resumed request
completes with the option's value instead of the builtin's result —
`Monitor[(AppendTo[log, 3]; 77), 0, (…; InputString["p: "]; …; EvaluationMonitor :> Null)]`
resumes to `List[RuleDelayed[EvaluationMonitor, Null]]` where the queued path
returns `77`. brief-548 recorded 29 sites (`.temp/brief-548/draft-G5.md` has
the list), all reaching the shared readers — `system_option_bool`,
`runtime_option_expr_for_symbol`, `option_rule_expr_for_symbol` →
`resolve_option_entry_value` (`crates/rum-runtime/src/runtime/options/core.rs:76`)
— from `Monitor`, `TestReport`, `Trace`, `CheckAbort`, `Cases`,
`DeleteCases`, `Replace`, `Names`, `PersistPackageState`/`DumpSave`, `Count`,
`MemberQ`, `FreeQ`, `Level`, `Position`, `ArrayFilter` and `Depth`.

The director re-ran brief-548's probes on `5cec7b67`
(`.temp/director/brief548-probe-rerun-5cec7b67.txt`): 37 still diverge, and
every G5 witness is among them (`Monitor-options`, `Monitor-options-twice`,
`Cases-Heads-option`, `Cases-Heads-after-target`, `TestReport-option-*`,
`Names-*`, `CheckAbort-option`, `ArrayFilter-option`, `DumpSave-option`,
`Depth-Heads-option`, `Level-Heads-option`, …). Lost continuations today: 43;
G5 accounts for 29.

## What the design must settle

1. **The inventory.** Every reader and every caller, with `path:line`: which
   options are read, when (before or after the builtin's own guest work — note
   the `*-after-target` witnesses), which readers evaluate delayed values, and
   what each caller does after the read. Group the callers by what a
   continuation must carry.
2. **Per-site repair or one mechanism.** A frame at each of 29 call sites is
   one option. A shared mechanism is another — for example, a builtin's option
   reads resolved in a resumable phase whose completion hands the resolved
   values to the rest of the builtin, so a suspension inside an option value
   is a suspension of that phase and nothing else. `CLAUDE.md`'s standard: a
   fix the compiler or a type enforces beats one that relies on each site
   remembering, even if larger. Compare the options on correctness,
   enforceability and how a future builtin that reads an option gets the right
   behaviour by construction; recommend one, with the evidence.
3. **Evaluation order is observable.** Options read after the builtin has done
   guest work (`Cases-Heads-after-target`, `DeleteCases-Heads-after-target`)
   must stay after it; options read in a particular order must keep that
   order; an option never read on a path must stay unread. Say how the design
   preserves each.
4. **The retention line's certified option region.** `options/core.rs` holds
   the bodies the retention line certified (brief-546, certificates C01/C02;
   `docs/dev/retention-closure-design.md`). The standing duty refreshes their
   fingerprints, but a change to a certified **body** must still pass
   `local_production_acceptance` on its merits in both feature sets — not by a
   refreshed fingerprint alone. Say which certified bodies the design touches,
   and whether it can avoid touching them; if it must, say what the
   certificates will then check and why they still hold.
5. **Suspension protocol**: brief-563/568/572's settled rules — capture only
   on `InputRequired` on the error edge, immutable Session-rooted frames with a
   cold working copy on an actual resume, `FrameRootTrace`/`PureFrameAbandon`,
   no `Session` borrow, observer or `NativeStackDepth` in a frame, rum-runtime's
   frame lists, a second suspension, abandonment.
6. **The ordinary path.** Option reading is on many hot paths (`Cases`,
   `Count`, `MemberQ`, `FreeQ`, `Position`, `Replace` read `Heads` on every
   call). Name the labels and the plan; **both arms with one codegen unit**
   (`TEMPLATE.md` pitfall 8). Depth is not expected to matter (option lists
   are flat) — say so with evidence or show otherwise.
7. **Inventory, census and layout consequences**, listed so the director can
   authorize them by name, and the step split.

## What I want back

1. The inventory (point 1) with evidence.
2. The design (points 2-7), the frame inventory, and the comparison.
3. Red tests, run in the scratch test and recorded: every brief-548 G5 witness
   through the rum-vs-rum oracle (value, `log`, messages, prints, suspension
   asserted, exact prompts, no owner after, `1 + 1`), the twice variants, the
   after-target order cases, an ordinary error inside an option value after a
   resume, cancellation at a suspended option, an option that is present but
   never read on the taken path.
4. Drafted implementation brief(s) in `TEMPLATE.md` shape — each with its red
   tests, predicted lost-continuation count, named inventory rows and frame
   lists, the retention-certificate obligation, the one-unit performance gate,
   the standing duty, and serial runs (`env CARGO_BUILD_JOBS=1
   RUST_TEST_THREADS=1`).
5. Open questions for the director.

## Context pointers

- brief-548's report, `.temp/brief-548/draft-G5.md`, `probes.json`,
  `lost.tsv`, `probe.rs`.
- brief-530 (one lost continuation on the option path, fixed earlier) and
  brief-559's `OptionValue` loop frame
  (`crates/rum-runtime/src/runtime/options/builtins.rs`).
- briefs 563, 568, 572-575 (the accepted continuation and measurement
  patterns).
- `crates/rum-runtime/src/runtime/options/`, `suspension/core.rs`;
  `docs/dev/retention-closure-design.md`; the retention inventories under
  `crates/rum-stdlib/src/tests/retention_closure/`.
- `docs/dev/codex/TEMPLATE.md`, `AGENTS.md`,
  `docs/code-organization-guidelines.md`.

## Scope

- Files you MAY modify: **NONE**, except: append your report to this file;
  scratch under `.temp/brief-576/`; **one** scratch integration test,
  `crates/rum-stdlib/tests/zz_brief576_probe.rs`, **deleted before the report
  is appended**, its final content copied to `.temp/brief-576/probe.rs`.
- Everything else is read-only.

## Hard rules

- **No `git`, in any form.**
- Do NOT modify any file outside the exceptions; STOP and report instead.
- No network; nothing outside this repository; all scratch and `TMPDIR` under
  `.temp/brief-576/`.
- **Run no probe that aborts a process.**
- Commands, serial, each through `python3 scripts/run-bounded.py`:
  `env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --test zz_brief576_probe`
  (150 s); `.temp/brief-575/candidate-08/rum-cli` (source identical to HEAD)
  on passing programs (60 s); `valgrind --tool=callgrind` on it (150 s). File
  reads, `rg`/`grep`, `python3`.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **No production code.** No new `unsafe`.

## Report format (your final message, appended to this file)

1. What changed — "design only", scratch, the deleted test.
2. The inventory.
3. The design, frame inventory, comparison, retention obligation.
4. The red tests with their pre-fix values.
5. The drafted implementation brief(s).
6. Open questions for the director.

---
