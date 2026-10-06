# brief-583 — design: re-census, the Unsettled edges, and the catalog-wide suspension oracle (E1)

`Scope: NONE — no production change.` Design-only, with one declared scratch
test (see Scope). **Do not run `git` in any form**, including before you have
read this brief.

## Goal

The suspension-continuation branch has repaired every group brief-548 found:
G1-G8, 86 lost continuations, through briefs 549-582. On `033711f9` all 195 of
brief-548's probes now agree between the resumed and the queued path
(`.temp/director/brief548-probe-rerun-033711f9.txt`: 0 divergent). Two steps
remain before the branch closes:

- **E1 — a required, catalog-wide suspension oracle**: every installed
  builtin's guest-effect entries declare executable suspension coverage, and
  the repaired cases become mandatory equality and lifecycle regressions, so a
  new builtin cannot silently lose a continuation.
- **E2 — a may-suspend result type with no `Try`**, so a bare `?` on it does
  not compile.

brief-548's census is now stale. It classified 2 399 candidate edges:
TailCorrect 1 175, FrameCarries 605, **Unsettled 495**, Lost 86, Transformed 38
(`.temp/brief-548/census.tsv`). The 86 are repaired; the 495 Unsettled were
never resolved, and since then the tree has gained many modules (flat walks,
the option read protocol, new frames). This design settles where the branch
actually stands and how E1 makes it hold.

## What the design must settle

1. **Re-census.** Re-run brief-548's tooling (`.temp/brief-548/build_graph.py`,
   `classify.py` and their inputs; adapt into `.temp/brief-583/`) on the
   current tree. Report the counts by verdict, confirm every one of the 86
   former Lost sites now classifies as repaired (or explain why the tool cannot
   see it), and list every **new** Lost site with evidence — code added since
   brief-548 included.
2. **The Unsettled edges.** For each Unsettled row (495 at brief-548; the
   re-census count now), resolve the call target and give a verdict with
   source evidence: carried by a frame, a correct tail, a documented boundary
   (a batch API or sink that is non-resumable by ruling), or **Lost**. Where a
   row cannot be resolved by reading, a probe through the rum-vs-rum oracle in
   the scratch test settles it. Group any new Lost sites by mechanism, as
   brief-548 did, with a repair sketch — these become new groups in this
   branch, because a line once opened is finished.
3. **The E1 oracle** (`.temp/brief-548/enforcement.md` §2 is the starting
   point): a test-only enumerator of the specs `install_core` actually
   installs; a coverage manifest in which every installed spec declares its
   guest-effect entries (callbacks, option defaults, pattern tests and guards,
   delayed association entries, schema predicates and defaults, lazy
   consumers, …) or a reviewed no-guest-effect disposition, failing when the
   catalog changes without it; descriptor-built fixtures run in two fresh
   sessions (resumed against queued), with the suspension asserted, middle and
   twice positions, abandonment, and a fresh request after; mutation tests for
   an omitted registration, a non-suspending fixture, a false tail and a
   missing prefix. Say how the existing per-group suites
   (`*_continuations.rs`) fold into it, how optional catalog features are
   covered, and that it adds no default-path work.
4. **E2's relation to E1.** Read `.temp/brief-548/enforcement.md` §1 and §3 and
   `draft-E2.md`; say what E1 establishes that E2 then enforces at compile
   time, what E2 must prototype and measure first (`Session::evaluate`'s
   return shape on the hottest labels, one codegen unit), and the sequence.
   Design E2 in outline only; its own design brief comes after E1.
5. **Inventory and layout consequences** of E1, listed for authorization.

## What I want back

1. The re-census: counts, the 86 confirmed, every new Lost site with evidence.
2. The Unsettled resolution table (every row: target, verdict, evidence), the
   new groups if any, with repair sketches.
3. The E1 oracle design and the E2 outline and sequence.
4. Drafted implementation brief(s) for E1 (and for any new repair group) in
   `TEMPLATE.md` shape — red tests, the standing duty, serial runs (`env
   CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1`); E1 is test-only, so its gate is
   correctness and coverage, not performance.
5. Open questions for the director.

## Context pointers

- brief-548 and its report (`docs/dev/codex/brief-548-suspension-continuation-sweep.md`);
  `.temp/brief-548/{census.tsv,lost.tsv,probes.json,probe.rs,enforcement.md,draft-E1.md,draft-E2.md,repair-plan.md}` and its scripts.
- The repairs: briefs 549-582 and their reports; the per-group suites under
  `crates/rum-stdlib/tests/*continuations*.rs`.
- `crates/rum-runtime/src/runtime/suspension/`, the catalog and `install_core`.
- `docs/dev/codex/TEMPLATE.md`, `AGENTS.md`,
  `docs/code-organization-guidelines.md`.

## Scope

- Files you MAY modify: **NONE**, except: append your report to this file;
  scratch under `.temp/brief-583/`; **one** scratch integration test,
  `crates/rum-stdlib/tests/zz_brief583_probe.rs`, **deleted before the report
  is appended**, its final content copied to `.temp/brief-583/probe.rs`.
- Everything else is read-only.

## Hard rules

- **No `git`, in any form.**
- Do NOT modify any file outside the exceptions; STOP and report instead.
- No network; nothing outside this repository; all scratch and `TMPDIR` under
  `.temp/brief-583/`.
- **Run no probe that aborts a process.**
- Commands, serial, each through `python3 scripts/run-bounded.py`:
  `env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --test zz_brief583_probe`
  (150 s); `python3` on the census scripts (300 s each). File reads,
  `rg`/`grep`.
- **Testing Policy is a hard line** (`AGENTS.md`): rum-vs-rum only.
- **No production code.** No new `unsafe`.

## Report format (your final message, appended to this file)

1. What changed — "design only", scratch, the deleted test.
2. The re-census.
3. The Unsettled resolution and any new groups.
4. The E1 design, the E2 outline, the sequence.
5. The drafted implementation brief(s).
6. Open questions for the director.

---
