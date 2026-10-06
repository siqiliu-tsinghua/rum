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

## Report — brief-583

1. **What changed — design only.**

   Re-ran the census, reviewed the unresolved edges, ran bounded rum-vs-rum
   probes, and wrote the E1 design, E2 outline and three implementation drafts.
   All evidence is under [.temp/brief-583](../../../.temp/brief-583/README.md).
   The sole scratch integration test was copied to
   [probe.rs](../../../.temp/brief-583/probe.rs) and deleted before this report
   was appended. No production change, git command, network access, unsafe,
   lease or process-aborting probe.

   The final scope audit checks 1,817 unchanged baseline files other than this
   brief, plus unexpected added source; only this report and authorized scratch
   remain. No workspace test, formatting command or benchmark was run: this
   brief permits only the named scratch test and census scripts.

2. **The re-census.**

   The adapted brief-548 AST/graph tooling found 8,597 functions, 1,731 possible
   effect owners and 2,443 candidate pairs. Obsolete line-number Lost/boundary
   overrides were removed, rather than applied to new source positions.

   | Population | TailCorrect | FrameCarries | Unsettled | Lost | Boundary/Transformed |
   | --- | ---: | ---: | ---: | ---: | ---: |
   | brief-548, 2,399 pairs | 1,175 | 605 | 495 | 86 | 38 Transformed |
   | Fresh raw, 2,443 pairs | 1,238 | 608 | 597 | 0 | 0 |
   | Reviewed, 2,443 pairs | 1,650 | 743 | 0 | 4 | 46 Boundary |

   Raw zero Lost was not a proof. The source review found four. Another
   [25 supplemental edges](../../../.temp/brief-583/supplemental-edges.tsv)
   cover typed option finish/read calls, their outer wrappers, legacy dispatch
   and the stored LHS callback missed by the old filter; all are local tails.
   The expanded population is 2,468. Eight CLI test-only helper rows are
   explicitly excluded from production while retained for comparison.
   New/moved function identities contribute 98 reviewed candidate rows.

   [All 86 former Lost sites](../../../.temp/brief-583/former-lost-resolution.tsv)
   are accounted for: 68 FrameCarries, 15 caller-complete tails, three obsolete
   wrappers removed. Group totals remain G1=1, G2=14, G3=7, G4=16, G5=29,
   G6=1, G7=14, G8=4. None remains Lost.

   All 195 historical probes passed actual-suspension and typed
   value/termination/messages/prints/log comparisons, with a fresh request
   after completion. The final scratch run reported:

   ~~~text
   running 4 tests
   test ast_inventory ... ok
   test baseline_controls ... ok
   test installed_catalog ... ok
   test new_diagnostics ... ok

   test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.17s
   ~~~

   Command: TMPDIR=/home/remi/dev/rum/.temp/brief-583/tmp python3
   scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1
   RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib
   --test zz_brief583_probe. Raw output:
   [probe-run-03.txt](../../../.temp/brief-583/probe-run-03.txt).
   The diagnostic test records divergences; its green status does not claim
   the new defects are repaired. No-default catalog execution was not authorized.

3. **Unsettled resolution and new groups.**

   Every row includes resolved target, local verdict and source evidence:

   | Table | TailCorrect | FrameCarries | Boundary | Lost |
   | --- | ---: | ---: | ---: | ---: |
   | [597 fresh rows](../../../.temp/brief-583/unsettled-resolution.tsv) | 412 | 135 | 46 | 4 |
   | [495 historical rows](../../../.temp/brief-583/historical-unsettled-resolution.tsv) | 386 | 97 | 8 | 4 |

   The historical mapping preserves 476 function/call identities and explicitly
   maps 19 moved/replaced sites. The [complete reviewed table](../../../.temp/brief-583/reviewed-census.tsv)
   distinguishes local tails from transitive exposure to a defective caller.
   It is a source review, not compiler-certified effect analysis. Generic
   callback traits resolve to their symbolic dispatch domains; arbitrary
   external implementations remain outside the reviewed installed domain.

   **G9 — list-destructuring assignment:** P0354-P0356 at
   definitions/definitions.rs:467,480,485 lose the complete RHS result, remaining
   sibling cursor and leaf completion. These are three obligations in one
   mechanism. A suspended Part target returns 7 instead of List[7,9] and skips
   the later store; the twice witness reaches only its first input.
   Repair with an owned traversal cursor/parent stack, pending-leaf phase and
   retained original RHS. Completed stores remain committed; abandonment drops
   only pending work.

   **G10 — direct Default fallback:** P0226 at
   definitions/application/down_values.rs:67 lets a suspended direct lookup
   return None without continuing the ordinary down-value scan. The bounded
   twice-decline witness returns Default[d], log List[1,2], while queued
   evaluation returns 7, log List[1,2,3]. Retain the caller's Some/None decision
   and enter a shared ordinary-fallback helper after None.

   Both groups were hidden in the old Unsettled population: old P0363-P0365
   and P0226. They are newly confirmed defects, not regressions attributed to
   new modules. [Fourteen final probes](../../../.temp/brief-583/new-probes.tsv)
   all complete: nine comparisons agree and five witnesses diverge (four G9,
   one G10). The earlier answer-exhausted Default experiment is preserved
   separately and is not used as a completed equality witness.

4. **E1 design, E2 outline and sequence.**

   [E1 design](../../../.temp/brief-583/oracle-design.md) uses the existing
   installed-spec iterator: **643 specs observed** by a fresh installed session.
   Every spec must declare explicit guest-effect entries or a reviewed
   no-local-guest disposition with delegated evaluator coverage. Exact catalog
   identity, source-owner drift, descriptor variants and lifecycle obligations
   are mandatory; no silent feature or unknown-target exclusions.

   Descriptor factories run two fresh sessions, assert actual prompt/kind/count
   and middle/twice positions, compare typed results, output and state, test
   cancellation/reset/pending rejection, and require a fresh request. Existing
   per-group suites share descriptors while retaining specialized assertions;
   all 195 historical labels get an explicit migration mapping. Four falsifiers
   exercise omitted registration, no suspension, false tail and missing prefix.
   The proposed required script runs default/no-default jobs serially. The five
   optional digest specs suggest 638 without hash; E1 must verify that live.
   This adds no production-path work and needs no performance gate.

   [E2 outline](../../../.temp/brief-583/e2-outline.md) follows green E1.
   A no-Try result blocks bare propagation; a restricted companion gate must
   also reject discard, laundering, false tails and opaque syntax. Prototype
   Session::evaluate's return shape and API ripple first, measuring hottest
   labels and same-tier neighbors on baseline/candidate builds with **one
   codegen unit**. E1 remains the semantic falsifier. Sequence: G9 and G10
   repairs, complete E1, separate E2 design/measurement brief, then any migration.

5. **Drafted implementation briefs and authorization scope.**

   [G9 draft](../../../.temp/brief-583/draft-G9.md),
   [G10 draft](../../../.temp/brief-583/draft-G10.md), and
   [E1 draft](../../../.temp/brief-583/draft-E1.md) follow TEMPLATE.md:
   exact fences, red witnesses, deliverables, serial full environment,
   verification, checkpointing, hard rules and stop-loss. They are raw material
   for director rewrite, not dispatched briefs.

   E1 proposes one integration root with family fixtures, shared observation/
   lifecycle support, coverage inventories, mutations and a two-feature gate
   script. Test edits still require fingerprint/certificate replay and ownership
   inventory reconciliation. Its test-only mutation frame needs an explicit
   fixture population/list/count in the existing runtime frame tests; existing
   production dispositions and counts stay unchanged. G9/G10 add their actual
   production frame types to the standing closed inventories and require
   measured microbench A/B gates.

6. **Open questions for the director.**

   * Approve G9/G10 as the next two repair groups, with the stated local
     obligation count and preserved eager semantics?
   * Approve E1's test layout, explicit mutation-frame inventory population and
     scripts/check-suspension-oracle.sh as the required two-feature gate?
   * Supply final brief numbers, calibrated broad-suite/benchmark bounds and
     baseline build paths; approve the proposed new target benchmark labels.
   * In the later E2 brief, settle exported kernel callback/API compatibility
     and the restricted disposition mechanism after the return-shape prototype.
