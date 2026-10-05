# brief-571 — G4 step A: retire three obsolete matcher adapters

Implementation brief. **Do not run `git` in any form**, including before you
have read this brief. Leave every change uncommitted for the director's gate.

## Goal

brief-570 found that three of G4's sixteen lost-continuation sites live in
adapters nothing in production calls:

- `match_expr` (`crates/rum-runtime/src/runtime/pattern_matching/core.rs:11-21`,
  P1019) and `match_expr_interpreted`
  (`pattern_matching_interpreted.rs:312-323`, P1285) — borrowed-bindings
  wrappers over the owned matchers, marked `#[allow(dead_code)]`, used only by
  one parity test (`runtime/tests/definitions_and_rules_part2.rs:47, 50`, with
  the binding comparison at `:58-60`);
- `normalize_cases_spec` (`kernel_data/rewrite_data/cases/match_q.rs:78-85`,
  P0779) — no caller at all; production `Cases` resolves and compiles in
  separately framed phases.

The right repair for a continuation lost in code nothing calls is to delete
the code. Migrate the parity test to the owned outcomes (`.matched`,
`.bindings`), keeping every case and every binding assertion, then remove the
three functions. Lost continuations **59 -> 56** (three retirements; G4
16 -> 13). This step repairs no live witness; brief-570's red G4 cases stay
red until steps B and C.

## Context pointers

- brief-570's report §2 and `.temp/brief-570/recursion.md`,
  `caller-search.txt`, `inventory-plan.md` (section A),
  `draft-A-retire-obsolete-wrappers.md`.
- `crates/rum-runtime/src/runtime/mod.rs:48-49` (the interpreted wrapper is
  `pub(super)` in a `cfg(test)` module); the owned interpreted suspension test
  in `definitions_and_rules_part2.rs:66-120`, which stays.
- `docs/dev/codex/TEMPLATE.md` (standing duty).

## Scope

**Objective:** the three deletions and the parity test's migration.

**Edit surface:** the three functions named above, in their three files; the
parity test in `crates/rum-runtime/src/runtime/tests/definitions_and_rules_part2.rs`.

**Standing maintenance duty — always in scope** (`TEMPLATE.md`): replay the 68
contract fingerprints and both certificates, reconcile the ownership
inventories from discovery (positions move), update stale evidence positions
in `retention_closure/inventory/obligations.tsv`. None of the three functions
owns an inventory row; **any row disappearing is a STOP**, as is any other
identity, status, disposition or exclusion change. The named-frame count stays
556; no frame-list change.

**Fence:** no change to the matcher, its compiler or codecs, any production
caller, `Expr`, public or `#[doc(hidden)]` APIs, manifests or the lockfile; no
compatibility wrapper in place of the deleted ones. A production, public or
`#[doc(hidden)]` caller found during the work **blocks** the removal — STOP and
report it. No edits to `AGENTS.md`, `TODO.md`, root `PROGRESS.md`, earlier
briefs or docs.

**Disclosure duty:** every changed line, the migrated assertions, every
inventory row change, and a repository name search for the three names that
separates them from unrelated identifiers (AST-visitor `match_expr` methods).

## Deliverable

- The parity test compares owned outcomes, `.matched` and `.bindings`, over
  the same cases with the same expected flags; the owned interpreted
  suspension test is untouched.
- The three functions are gone, with no remaining reference.
- The standing duty replayed and disclosed; the three frozen IDs recorded as
  retired (brief-548's snapshots are not edited).

## Tests and performance

Record the parity test and the owned interpreted suspension test passing
**before** any edit, and again after. brief-570's red G4 fixtures are evidence
for steps B and C — do not add them here as passing or ignored contracts.

**No performance gate, and why:** two of the functions are compiled only for
tests (`cfg(test)` or used only by a test), and the third has no caller, so the
release binary's code does not change. Do not build a release binary or run a
panel for this step.

## Verification (MANDATORY — run to green or evidence)

Serial, through `python3 scripts/run-bounded.py --timeout <s> -- env
CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true
TMPDIR=<repo>/.temp/brief-571/tmp …`. Bounds: focused 150 s;
`cargo test -p rum-runtime`, `ownership_graph` and `local_production_acceptance`
(either feature set) 300 s; `cargo test --workspace --no-fail-fast` 2 400 s.

```
cargo fmt --check
cargo test -p rum-runtime
cargo test -p rum-stdlib --lib ownership_graph
cargo test -p rum-stdlib --lib local_production_acceptance
cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance
cargo test --workspace --no-fail-fast
```

Report native ZeroMQ skips as skips. At most eight focused attempts; stop
after two compiling attempts that reduce neither remaining references nor
failing tests.

## Checkpointing — protect the scene

Scratch under `.temp/brief-571/`; `.temp/brief-571/PROGRESS.md` after the
baseline, the deletion and the verification.

## Hard rules

- **No `git`, in any form.** No network; nothing outside this repository.
- Commands: scoped reads and edits, `rg`/`grep`, `python3`, bounded serial
  Cargo. No abort probes.
- **Testing Policy is a hard line** (`AGENTS.md`). No new `unsafe`.
- Paste real output; say plainly what did not run.

## Report format (append to this brief)

1. Changed lines and the migrated assertions.
2. Verbatim before/after test output, the name search, every inventory row
   change, the broad suites.
3. The three retired IDs (59 -> 56 predicted); open questions.

---

## Report — brief-571

### 1. Changed lines and migrated assertions

Deleted the three obsolete adapters and migrated the single parity test to owned outcomes. The caller search found no production, public or `#[doc(hidden)]` caller. There is no compatibility replacement. Changes remain uncommitted; no version-control command, network request, release build, performance panel or abort probe was run.

The four source changes are:

- `pattern_matching/core.rs`: deleted old lines 11–22 (the borrowed adapter, its attribute and trailing blank line); `match_expr_owned` now starts at line 11.
- `pattern_matching_interpreted.rs`: deleted old lines 312–324; the owned matcher's existing attribute is now at 312 and its signature at 313. The interpreted module remains test-only.
- `kernel_data/rewrite_data/cases/match_q.rs`: deleted old lines 78–86; the unchanged `resume_match_q_after_value` now starts at 78.
- `tests/definitions_and_rules_part2.rs`: removed the two borrowed binding locals at old lines 43–44; calls at new lines 45 and 48 consume fresh binding maps and return owned outcomes. Assertions at 52, 56 and 60 compare `interpreted.matched`, `compiled.matched` and `compiled.bindings == interpreted.bindings`. All 18 pattern/value/expected cases and all three assertions remain. The owned interpreted suspension test (old 65–121, new 66–122), and every subsequent test, are byte-identical to the captured baseline.

All runtime paths above are relative to `crates/rum-runtime/src/runtime/`. The exact source edit, generated by Python's unified-diff formatter against the captured baseline, is:

```diff
--- before/crates/rum-runtime/src/runtime/pattern_matching/core.rs
+++ after/crates/rum-runtime/src/runtime/pattern_matching/core.rs
@@ -8,18 +8,6 @@
 use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};
 
 impl Session {
-    #[allow(dead_code)]
-    pub(in crate::runtime) fn match_expr(
-        &mut self,
-        pattern: &Expr,
-        value: &Expr,
-        bindings: &mut SymbolExprMap,
-    ) -> Result<bool, EvalError> {
-        let outcome = self.match_expr_owned(pattern.clone(), value.clone(), bindings.clone())?;
-        *bindings = outcome.bindings;
-        Ok(outcome.matched)
-    }
-
     pub(in crate::runtime) fn match_expr_owned(
         &mut self,
         pattern: Expr,
--- before/crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs
+++ after/crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs
@@ -310,19 +310,6 @@
     }
 
     #[allow(dead_code)]
-    pub(super) fn match_expr_interpreted(
-        &mut self,
-        pattern: &Expr,
-        value: &Expr,
-        bindings: &mut SymbolExprMap,
-    ) -> Result<bool, EvalError> {
-        let outcome =
-            self.match_expr_interpreted_owned(pattern.clone(), value.clone(), bindings.clone())?;
-        *bindings = outcome.bindings;
-        Ok(outcome.matched)
-    }
-
-    #[allow(dead_code)]
     pub(super) fn match_expr_interpreted_owned(
         &mut self,
         pattern: Expr,
--- before/crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs
+++ after/crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs
@@ -73,15 +73,6 @@
                 self.resume_call_from_prepared_args(resolved_head, resolved_args)
             }
         }
-    }
-
-    #[allow(dead_code)]
-    pub(in crate::runtime) fn normalize_cases_spec(
-        &mut self,
-        expr: &Expr,
-    ) -> Result<CasesSpec, EvalError> {
-        let resolved = self.resolve_cases_spec_expr_for_matching_owned(expr.clone())?;
-        Ok(self.compile_cases_spec_from_resolved_expr(resolved))
     }
 
     fn resume_match_q_after_value(
--- before/crates/rum-runtime/src/runtime/tests/definitions_and_rules_part2.rs
+++ after/crates/rum-runtime/src/runtime/tests/definitions_and_rules_part2.rs
@@ -40,23 +40,24 @@
             .lower_syntax_expr(&parse_expr(value_src).unwrap())
             .unwrap();
         let value = session.evaluate(value).unwrap();
-        let mut interpreted_bindings = SymbolExprMap::default();
-        let mut compiled_bindings = SymbolExprMap::default();
 
         let interpreted = session
-            .match_expr_interpreted(&pattern, &value, &mut interpreted_bindings)
+            .match_expr_interpreted_owned(pattern.clone(), value.clone(), SymbolExprMap::default())
             .unwrap();
         let compiled = session
-            .match_expr(&pattern, &value, &mut compiled_bindings)
+            .match_expr_owned(pattern, value, SymbolExprMap::default())
             .unwrap();
 
         assert_eq!(
-            interpreted, expected,
+            interpreted.matched, expected,
             "interpreted mismatch for {pattern_src}"
         );
-        assert_eq!(compiled, expected, "compiled mismatch for {pattern_src}");
         assert_eq!(
-            compiled_bindings, interpreted_bindings,
+            compiled.matched, expected,
+            "compiled mismatch for {pattern_src}"
+        );
+        assert_eq!(
+            compiled.bindings, interpreted.bindings,
             "binding mismatch for {pattern_src}"
         );
     }
```

Complete line disclosure: [source changes](../../../.temp/brief-571/source-changes.diff), [all inventory changes](../../../.temp/brief-571/inventory-changes.diff), and [combined diff](../../../.temp/brief-571/changed-lines.diff). These compare the pre-edit filesystem snapshot; they make no claim about pre-existing director changes. The only additional non-scratch edit is this appended report.

Split assessment: `core.rs` is 24,024 characters and the test-only interpreted module 31,888, both still in the split-plan range. This brief authorizes only adapter deletion, so a split would exceed its fence. Future coherent boundaries are compiled matching versus binding snapshots/name collection, and interpreted matching versus continuation frames/codecs. `match_q.rs` is 8,948 characters. The related parity/suspension/definition test file is 17,845 characters, in the test warning range; keeping this existing scenario group together avoids an unrelated reorganization.

### 2. Verification and inventory disclosure

All Cargo commands ran serially through the required wrapper, offline with one build job, one test thread and repository-local `TMPDIR`. The complete commands and recorded exits are in [commands.json](../../../.temp/brief-571/commands.json). The common prefix was:

```text
python3 scripts/run-bounded.py --timeout <bound> -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-571/tmp
```

The workspace command additionally sets `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`: the existing test helper at `zeromq_native_test_support.rs:335–350` requires this variable to skip only when loopback binding is denied. `--nocapture` records the actual skip messages. Bounds were 150 seconds for focused replay/formatting, 300 for runtime/ownership/certificate gates, and 2,400 for the workspace.

Before edits, the two required tests ran separately. Their complete verbatim output follows.

`cargo test -p rum-runtime compiled_matcher_matches_interpreted_matcher_on_supported_subset` (exit 0):

```text
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
warning: function `register_test_if_builtin` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:320:27
    |
320 | pub(in crate::runtime) fn register_test_if_builtin(session: &mut Session) {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` on by default

warning: function `register_test_return_builtin` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:330:27
    |
330 | pub(in crate::runtime) fn register_test_return_builtin(session: &mut Session) {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `register_test_vm_template_builtins` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:358:27
    |
358 | pub(in crate::runtime) fn register_test_vm_template_builtins(session: &mut Session) {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: `rum-runtime` (lib test) generated 3 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 8.51s
     Running unittests src/lib.rs (target/debug/deps/rum_runtime-95cd5fbb46da4f1c)

running 1 test
test runtime::tests::definitions_and_rules_part2::compiled_matcher_matches_interpreted_matcher_on_supported_subset ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1096 filtered out; finished in 0.01s

```

`cargo test -p rum-runtime interpreted_matcher_owned_can_resume_sequence_conditions_across_input` (exit 0):

```text
warning: function `register_test_if_builtin` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:320:27
    |
320 | pub(in crate::runtime) fn register_test_if_builtin(session: &mut Session) {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` on by default

warning: function `register_test_return_builtin` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:330:27
    |
330 | pub(in crate::runtime) fn register_test_return_builtin(session: &mut Session) {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `register_test_vm_template_builtins` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:358:27
    |
358 | pub(in crate::runtime) fn register_test_vm_template_builtins(session: &mut Session) {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: `rum-runtime` (lib test) generated 3 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running unittests src/lib.rs (target/debug/deps/rum_runtime-95cd5fbb46da4f1c)

running 1 test
test runtime::tests::definitions_and_rules_part2::interpreted_matcher_owned_can_resume_sequence_conditions_across_input ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1096 filtered out; finished in 0.01s

```

After edits, `cargo fmt --check` exited 0 with no output. `cargo test -p rum-runtime` passed both required tests as part of the full runtime suite. Verbatim lines from [runtime.txt](../../../.temp/brief-571/logs/runtime.txt):

```text
running 1097 tests
test runtime::tests::definitions_and_rules_part2::compiled_matcher_matches_interpreted_matcher_on_supported_subset ... ok
test runtime::tests::definitions_and_rules_part2::interpreted_matcher_owned_can_resume_sequence_conditions_across_input ... ok
test runtime::tests::frame_root_trace::derived_frame_root_trace_inventory_matches_the_closed_capture_graph ... ok
test runtime::tests::frame_root_trace::suspension_protocol_repository_invariants_hold ... ok
test result: ok. 1094 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 22.59s
running 22 tests
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.63s
```

The three runtime test-helper dead-code warnings were already present in the baseline; no warning was weakened or silenced. Both closed frame-list files remain unchanged, including the named-frame count of 556.

The repository name search used exact word boundaries, excluded version-control internals, scratch and build output, and respected the normal repository ignore rules; it does not confuse the owned method names with the retired adapters:

```text
rg -n --hidden --glob '!.git/**' --glob '!.temp/**' --glob '!target/**' '\b(match_expr|match_expr_interpreted|normalize_cases_spec)\b' .
```

Full before/after results are [name-search-before.txt](../../../.temp/brief-571/name-search-before.txt) and [name-search-after.txt](../../../.temp/brief-571/name-search-after.txt). The latter was captured after deletion and before appending this report, verbatim:

```text
./TODO.md:346:compilation or matching). Two G4 sites (`match_expr`, `match_expr_interpreted`)
./TODO.md:351:three retirements: `match_expr` and `match_expr_interpreted` are used only by
./TODO.md:352:one parity test, and `normalize_cases_spec` has no caller. The two G4-owned
./docs/dev/codex/brief-570-rule-pattern-continuation-design.md:19:  `resolve_match_pattern_subexprs`, `normalize_cases_spec`;
./docs/dev/codex/brief-570-rule-pattern-continuation-design.md:20:- `pattern_matching/core.rs:18` (`match_expr` → `match_expr_owned`) and
./docs/dev/codex/brief-570-rule-pattern-continuation-design.md:21:  `pattern_matching_interpreted.rs:320` (`match_expr_interpreted` →
./docs/dev/codex/brief-570-rule-pattern-continuation-design.md:62:2. **The two dead-code sites.** Are `match_expr` and `match_expr_interpreted`
./docs/dev/codex/brief-570-rule-pattern-continuation-design.md:194:The two allow(dead_code) matcher wrappers have **one test consumer**, rather than being wholly uncalled: tests/definitions_and_rules_part2.rs:47,50 invokes them and :58-60 compares bindings. They have no production or public/doc(hidden) path: the compiled wrapper is pub(in crate::runtime), the interpreted wrapper pub(super) in the cfg(test) module at runtime/mod.rs:48-49. Migrate the test to owned outcomes `.matched`/`.bindings`, then delete both wrappers; preserve the owned interpreted suspension test :66-120. `normalize_cases_spec` has no caller in crates/scripts; production Cases already resolves/compiles in separate framed phases. Delete it too. [caller-search.txt](../../../.temp/brief-570/caller-search.txt) records the uses and unrelated AST-visitor names.
./docs/dev/codex/brief-548-suspension-continuation-sweep.md:639:| `P0779 / G4` | `normalize_cases_spec → resolve_cases_spec_expr_for_matching_owned`<br>`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:83` | `<no production caller>` | Compile CasesSpec after resolver; allow(dead_code) wrapper has no production caller (legacy-callers-raw.txt). | `UNREACHABLE: no production caller; private runtime wrapper` |
./docs/dev/codex/brief-548-suspension-continuation-sweep.md:648:| `P1019 / G4` | `match_expr → match_expr_owned`<br>`crates/rum-runtime/src/runtime/pattern_matching/core.rs:18` | `<no production caller>` | Copy owned match bindings back to borrowed map and return matched bool; callers are cfg(test) only. | `UNREACHABLE: cfg(test)-only callers, legacy-callers-raw.txt` |
./docs/dev/codex/brief-548-suspension-continuation-sweep.md:649:| `P1285 / G4` | `match_expr_interpreted → match_expr_interpreted_owned`<br>`crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:320` | `<no production caller>` | Copy interpreted match bindings back to borrowed map and return matched bool; callers are cfg(test) only. | `UNREACHABLE: cfg(test)-only callers, legacy-callers-raw.txt` |
./docs/dev/codex/brief-548-suspension-continuation-sweep.md:803:**G4 — Rule and held-pattern normalization continuation (16 sites).** Give rule-lhs and held matcher normalization explicit phases for head, active child, pattern wrapper, accumulated child prefix and final rhs construction. Preserve Pattern/Blank/PatternTest/Condition holding rules and RuleDelayed rhs holding. Carry caller phases for eval_rule/eval_rule_delayed and resolve_matcher_pattern_expr_owned as well as the recursive helper phases. Retire the three unused private borrowed-result wrappers (normalize_cases_spec, match_expr, match_expr_interpreted), migrating cfg(test) callers to owned APIs; borrowed mutable binding writeback cannot be retained as a static frame. If a production caller is discovered, stop that removal and report the new API constraint.
./docs/dev/codex/brief-571-retire-obsolete-matcher-adapters.md:11:- `match_expr` (`crates/rum-runtime/src/runtime/pattern_matching/core.rs:11-21`,
./docs/dev/codex/brief-571-retire-obsolete-matcher-adapters.md:12:  P1019) and `match_expr_interpreted`
./docs/dev/codex/brief-571-retire-obsolete-matcher-adapters.md:17:- `normalize_cases_spec` (`kernel_data/rewrite_data/cases/match_q.rs:78-85`,
./docs/dev/codex/brief-571-retire-obsolete-matcher-adapters.md:62:separates them from unrelated identifiers (AST-visitor `match_expr` methods).
./crates/rum-stdlib/src/tests/retention_closure/effects.rs:234:                self.match_expr(f, branch, env, expected)?,
./crates/rum-stdlib/src/tests/retention_closure/local/surface.rs:133:    pub(super) fn match_expr(
./crates/rum-stdlib/src/tests/retention_closure/local/expressions.rs:33:            syn::Expr::Match(m) => self.match_expr(m, expected),
./crates/rum-stdlib/src/tests/retention_closure/effects/branches.rs:204:    pub(super) fn match_expr(
```

There are zero runtime code hits. The four stdlib hits are two unrelated Rust AST-visitor methods and their callers. The remaining hits are historical briefs and roadmap text, intentionally untouched under the fence.

Standing-duty replay ran the two selected tests together, serially: `cargo test -p rum-stdlib --lib -- --include-ignored --nocapture regenerate_local_contract_fingerprints discovery_report_without_inventory_input`. Only the explicitly selected fingerprint regeneration test was taken out of its normal ignored state. Verbatim discovery/regeneration output:

```text
running 2 tests
test tests::ownership_graph::discovery_report_without_inventory_input ... UnknownHostPayload count: 63
UnknownConstruct count: 7604
UnknownHostPayload breakdown: 6 storage edges; 57 public input sites
Self-reaching type count: 59
Production files: 646; test-only files: 409; nominal definitions: 1621; aliases: 43; owning edges: 3685; resolved Drop owners: 18
test tests::retention_closure::local::contract_tests::regenerate_local_contract_fingerprints ... Regenerated 68 fingerprints at /home/remi/dev/rum/target/brief-544/local-contracts.tsv
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1095 filtered out; finished in 5.86s
```

The replay's generated review copies used the standing-duty destinations from `TEMPLATE.md`: `target/brief-544/local-contracts.tsv` and `.temp/brief-546/certificates/`. Copies of the relevant output are retained under this brief's scratch directory. Ownership discovery accepted every existing row with the same identity/classification. Installation checked the source text at each relocated start/end and equality with every discovered row: no additions, removals, identity, status, disposition or exclusion changes.

`cargo test -p rum-stdlib --lib ownership_graph -- --nocapture` passed after installation; verbatim lines:

```text
running 20 tests
test tests::ownership_graph::production_inventory_is_exact ... EXACT INVENTORY self-reaching types: 59 rows
EXACT INVENTORY UnknownHostPayload: 63 rows
EXACT INVENTORY UnknownConstruct: 7604 rows
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1077 filtered out; finished in 58.49s
```

All 104 changed inventory rows are disclosed below. The three ownership files have 29 source-position changes; all 68 contract fingerprints were regenerated; five obligation evidence positions move from the identical surviving line 92 to 83; and two promotion rows each receive both replayed certificate cells. Other row fields are unchanged.

**Ownership source positions**
| Inventory row | Source | Start before → after | Macro end before → after |
| --- | --- | --- | --- |
| unknown_attributes.tsv:387 | kernel_data/rewrite_data/cases/match_q.rs | 200:1 → 191:1 | unchanged / none |
| unknown_attributes.tsv:388 | kernel_data/rewrite_data/cases/match_q.rs | 221:1 → 212:1 | unchanged / none |
| unknown_attributes.tsv:389 | kernel_data/rewrite_data/cases/match_q.rs | 242:1 → 233:1 | unchanged / none |
| unknown_attributes.tsv:613 | pattern_matching/core.rs | 544:1 → 532:1 | unchanged / none |
| unknown_captures.tsv:852 | kernel_data/rewrite_data/cases/match_q.rs | 197:22 → 188:22 | unchanged / none |
| unknown_captures.tsv:853 | kernel_data/rewrite_data/cases/match_q.rs | 214:27 → 205:27 | unchanged / none |
| unknown_captures.tsv:854 | kernel_data/rewrite_data/cases/match_q.rs | 235:30 → 226:30 | unchanged / none |
| unknown_captures.tsv:855 | kernel_data/rewrite_data/cases/match_q.rs | 254:18 → 245:18 | unchanged / none |
| unknown_captures.tsv:856 | kernel_data/rewrite_data/cases/match_q.rs | 257:26 → 248:26 | unchanged / none |
| unknown_captures.tsv:1243 | pattern_matching/core.rs | 555:18 → 543:18 | unchanged / none |
| unknown_macros.tsv:659 | kernel_data/rewrite_data/cases/match_q.rs | 93:23 → 84:23 | 101:10 → 92:10 |
| unknown_macros.tsv:660 | kernel_data/rewrite_data/cases/match_q.rs | 116:9 → 107:9 | 123:10 → 114:10 |
| unknown_macros.tsv:661 | kernel_data/rewrite_data/cases/match_q.rs | 165:5 → 156:5 | 168:6 → 159:6 |
| unknown_macros.tsv:1226 | pattern_matching/core.rs | 45:17 → 33:17 | 45:103 → 33:103 |
| unknown_macros.tsv:1227 | pattern_matching/core.rs | 59:58 → 47:58 | 59:69 → 47:69 |
| unknown_macros.tsv:1228 | pattern_matching/core.rs | 69:17 → 57:17 | 69:138 → 57:138 |
| unknown_macros.tsv:1229 | pattern_matching/core.rs | 99:17 → 87:17 | 99:143 → 87:143 |
| unknown_macros.tsv:1230 | pattern_matching/core.rs | 100:17 → 88:17 | 100:89 → 88:89 |
| unknown_macros.tsv:1231 | pattern_matching/core.rs | 156:9 → 144:9 | 156:103 → 144:103 |
| unknown_macros.tsv:1232 | pattern_matching/core.rs | 168:9 → 156:9 | 181:10 → 169:10 |
| unknown_macros.tsv:1233 | pattern_matching/core.rs | 192:9 → 180:9 | 209:10 → 197:10 |
| unknown_macros.tsv:1234 | pattern_matching/core.rs | 224:17 → 212:17 | 224:116 → 212:116 |
| unknown_macros.tsv:1235 | pattern_matching/core.rs | 225:17 → 213:17 | 225:84 → 213:84 |
| unknown_macros.tsv:1236 | pattern_matching/core.rs | 249:17 → 237:17 | 249:97 → 237:97 |
| unknown_macros.tsv:1237 | pattern_matching/core.rs | 298:25 → 286:25 | 298:170 → 286:170 |
| unknown_macros.tsv:1238 | pattern_matching/core.rs | 299:25 → 287:25 | 299:90 → 287:90 |
| unknown_macros.tsv:1239 | pattern_matching/core.rs | 377:26 → 365:26 | 377:37 → 365:37 |
| unknown_macros.tsv:1240 | pattern_matching/core.rs | 424:25 → 412:25 | 424:162 → 412:162 |
| unknown_macros.tsv:1241 | pattern_matching/core.rs | 451:25 → 439:25 | 451:172 → 439:172 |

**All 68 regenerated contract fingerprints**
| Inventory row | ID | Before | After |
| --- | --- | --- | --- |
| local-contracts.tsv:2 | B01 | fnv1a64:db468d6bf1047ebc | fnv1a64:ff9ea020a988a76d |
| local-contracts.tsv:3 | B02 | fnv1a64:533609524536b96d | fnv1a64:7497ba847b39a2c0 |
| local-contracts.tsv:4 | B03 | fnv1a64:c9429149ac934b46 | fnv1a64:b76672075001201b |
| local-contracts.tsv:5 | B04 | fnv1a64:d99f267597a91fb4 | fnv1a64:4ec723bd1a837c55 |
| local-contracts.tsv:6 | B05 | fnv1a64:9cb88efd030a44d1 | fnv1a64:adacc7ac87d52324 |
| local-contracts.tsv:7 | B06 | fnv1a64:198a9d462375ead8 | fnv1a64:11989e314e6feef9 |
| local-contracts.tsv:8 | B07 | fnv1a64:3728cc3fd44aefc1 | fnv1a64:d03ba3af5502ed74 |
| local-contracts.tsv:9 | B08 | fnv1a64:5b9afac2c593691a | fnv1a64:b9d185c4aeca6d9f |
| local-contracts.tsv:10 | B09 | fnv1a64:3b2b0e8f632f3030 | fnv1a64:e2aa2ad4844a3ec1 |
| local-contracts.tsv:11 | B10 | fnv1a64:eb9cc0333ccf4fdf | fnv1a64:e4e96c42afa58f4e |
| local-contracts.tsv:12 | B11 | fnv1a64:9af6d7d2b19d45b4 | fnv1a64:101ed51a3477a255 |
| local-contracts.tsv:13 | B12 | fnv1a64:fa378e8b3f37c530 | fnv1a64:a1b6aad06052d3c1 |
| local-contracts.tsv:14 | I01 | fnv1a64:b92878695dbc753c | fnv1a64:dd808b1e16409ded |
| local-contracts.tsv:15 | I02 | fnv1a64:f3fa0c90fb40ccd1 | fnv1a64:04ee4540800bab24 |
| local-contracts.tsv:16 | I03 | fnv1a64:d404dbc41b8c8770 | fnv1a64:a8d5b66e0b3f7581 |
| local-contracts.tsv:17 | I04 | fnv1a64:f2ce4c2b528dbf0d | fnv1a64:0a1f0daa2a2a44e0 |
| local-contracts.tsv:18 | P01 | fnv1a64:89cc8091b5d23678 | fnv1a64:ded327f82350a699 |
| local-contracts.tsv:19 | P02 | fnv1a64:ddaf91665b39e56f | fnv1a64:f1ebcd416b79e27e |
| local-contracts.tsv:20 | P03 | fnv1a64:c5303cb68565426e | fnv1a64:e8cef9fddfdfef33 |
| local-contracts.tsv:21 | P04 | fnv1a64:48112767c9dfd56d | fnv1a64:6972d899ffe2bec0 |
| local-contracts.tsv:22 | P05 | fnv1a64:934671414b81e0cc | fnv1a64:bcf0c78eb328a4dd |
| local-contracts.tsv:23 | P06 | fnv1a64:4ab6b35bcb675eb0 | fnv1a64:f235cfa0ec826d41 |
| local-contracts.tsv:24 | P070 | fnv1a64:85044f9da9dcd8b0 | fnv1a64:2c836be2caf7e741 |
| local-contracts.tsv:25 | P080 | fnv1a64:118bf3342da9c21f | fnv1a64:96b1250127232c0e |
| local-contracts.tsv:26 | P090 | fnv1a64:fc2f3138026878ad | fnv1a64:66a37abee41e3680 |
| local-contracts.tsv:27 | P100 | fnv1a64:435f857c2142c324 | fnv1a64:f8ffc1a215bd94a5 |
| local-contracts.tsv:28 | P110 | fnv1a64:a70ff79664b014c7 | fnv1a64:ece51adaf4570d86 |
| local-contracts.tsv:29 | P120 | fnv1a64:35a9924ea360803a | fnv1a64:19d6cea2f9a72c3f |
| local-contracts.tsv:30 | P130 | fnv1a64:daf6259d02fec9c1 | fnv1a64:7408fd0c83b6c774 |
| local-contracts.tsv:31 | P140 | fnv1a64:e2f31a635d376655 | fnv1a64:dafdbe42842b87d8 |
| local-contracts.tsv:32 | P150 | fnv1a64:535f8d6cd2e85b4d | fnv1a64:fbc48a61c1591ca0 |
| local-contracts.tsv:33 | P071 | fnv1a64:5cba0b8882c47e61 | fnv1a64:9270e5591e22a594 |
| local-contracts.tsv:34 | P081 | fnv1a64:a035a6ffe2f2c8d0 | fnv1a64:9c50e49a75057c61 |
| local-contracts.tsv:35 | P091 | fnv1a64:461a9612ddc99cb8 | fnv1a64:14993c2b079e6959 |
| local-contracts.tsv:36 | P101 | fnv1a64:9f12e8639a13b1c7 | fnv1a64:e4e80ba829baaa86 |
| local-contracts.tsv:37 | P111 | fnv1a64:effd6249b49160a0 | fnv1a64:e3a6015a69d15451 |
| local-contracts.tsv:38 | P121 | fnv1a64:18f3d9c7621ccd8b | fnv1a64:7872c5796d800b6a |
| local-contracts.tsv:39 | P131 | fnv1a64:774948dbcdfd4134 | fnv1a64:ec71462350d79dd5 |
| local-contracts.tsv:40 | P141 | fnv1a64:d3e09a4e8ef0fd18 | fnv1a64:c5da97082f96d6b9 |
| local-contracts.tsv:41 | P151 | fnv1a64:134ce9a6b528a5fa | fnv1a64:ba06efc6e7e7cd7f |
| local-contracts.tsv:42 | B13 | fnv1a64:a79b30cf5412a6d8 | fnv1a64:9fa931ba7f0caaf9 |
| local-contracts.tsv:43 | R01 | fnv1a64:e7b1a9d1d23e9f9d | fnv1a64:7e8c6fc1c14ffb90 |
| local-contracts.tsv:44 | R02 | fnv1a64:7f99a3665005bfae | fnv1a64:2de067c3f97163f3 |
| local-contracts.tsv:45 | R03 | fnv1a64:0cd2e7c2c24014f9 | fnv1a64:3c8a2ad1d72d01fc |
| local-contracts.tsv:46 | R040 | fnv1a64:75f2a5fb27e071bd | fnv1a64:35e4ddf97ff3a8b0 |
| local-contracts.tsv:47 | R050 | fnv1a64:51aafe3f5efdc6ac | fnv1a64:0553fbf1b5477c3d |
| local-contracts.tsv:48 | R060 | fnv1a64:296f1c79d7cce405 | fnv1a64:d12edcb889df7b68 |
| local-contracts.tsv:49 | R070 | fnv1a64:08a36f19c49487d0 | fnv1a64:04beacb456a73b61 |
| local-contracts.tsv:50 | R080 | fnv1a64:b9d2b51e2212880d | fnv1a64:d123769cf9af0de0 |
| local-contracts.tsv:51 | R090 | fnv1a64:1ba7b7343daebc64 | fnv1a64:e2df5a74e9d3e465 |
| local-contracts.tsv:52 | R100 | fnv1a64:8e68359c5668f5c6 | fnv1a64:7c8c1659f9d6ca9b |
| local-contracts.tsv:53 | R110 | fnv1a64:2f3c59ba648924b7 | fnv1a64:5381ef2365ca07d6 |
| local-contracts.tsv:54 | R041 | fnv1a64:8dd493f96b3a65d8 | fnv1a64:85e294e4963469f9 |
| local-contracts.tsv:55 | R051 | fnv1a64:8a515f4d8f96852d | fnv1a64:f4c5a8d4714c4300 |
| local-contracts.tsv:56 | R061 | fnv1a64:bb275d7117d2d46a | fnv1a64:11f0b45c0432f60f |
| local-contracts.tsv:57 | R071 | fnv1a64:e3e580c44d736843 | fnv1a64:89a2f4536c3589b2 |
| local-contracts.tsv:58 | R081 | fnv1a64:ff49eed0bcee9824 | fnv1a64:b4ea2af6b16969a5 |
| local-contracts.tsv:59 | R091 | fnv1a64:f6484b4284038f15 | fnv1a64:c2b0e8f6b49b4b18 |
| local-contracts.tsv:60 | R101 | fnv1a64:259df0ab04882e45 | fnv1a64:1b35e82d52327728 |
| local-contracts.tsv:61 | R111 | fnv1a64:45a6078465dcc432 | fnv1a64:c0a03d7415f87987 |
| local-contracts.tsv:62 | R12 | fnv1a64:e3e0be840921e8f2 | fnv1a64:4c499230cf2c3ac7 |
| local-contracts.tsv:63 | R13 | fnv1a64:ddab4ffb0ca7a896 | fnv1a64:4fd04bade1b5310b |
| local-contracts.tsv:64 | R14 | fnv1a64:5db793f1fdf38324 | fnv1a64:1357d017f26e54a5 |
| local-contracts.tsv:65 | R15 | fnv1a64:3b4adc07b763e9e8 | fnv1a64:17ed367394120969 |
| local-contracts.tsv:66 | O010 | fnv1a64:3e90f24bb4a78dca | fnv1a64:5ac25c56cc508f6f |
| local-contracts.tsv:67 | O0200 | fnv1a64:ddbabc37abee8c55 | fnv1a64:d5c56016d2e2add8 |
| local-contracts.tsv:68 | O011 | fnv1a64:6bf7d901e55ea885 | fnv1a64:13b7994097713fe8 |
| local-contracts.tsv:69 | O0210 | fnv1a64:3d54016861300452 | fnv1a64:b0b875b1d4c6eda7 |

**Obligation evidence positions**
| Inventory row | ID | Evidence before → after |
| --- | --- | --- |
| obligations.tsv:57 | N:Count | kernel_data/rewrite_data/cases/match_q.rs:92 → :83 |
| obligations.tsv:66 | N:DeleteCases | kernel_data/rewrite_data/cases/match_q.rs:92 → :83 |
| obligations.tsv:103 | N:FreeQ | kernel_data/rewrite_data/cases/match_q.rs:92 → :83 |
| obligations.tsv:147 | N:MatchQ | kernel_data/rewrite_data/cases/match_q.rs:92 → :83 |
| obligations.tsv:151 | N:MemberQ | kernel_data/rewrite_data/cases/match_q.rs:92 → :83 |

**Certificate cells**

Rows promoted.tsv:2 (C01) and :3 (C02) each change column 5 (hash=true) and column 6 (hash=false). Each cell is copied verbatim from its fresh replay, and an exact comparison after masking 16-digit fingerprint tokens proves no other serialized content changes. Row identities, source, status, preconditions, strength and exclusions are unchanged.

promoted.tsv:2 C01 column 5 hash=true: installed exact replay cell (4148 bytes); only fingerprint tokens changed
promoted.tsv:3 C02 column 5 hash=true: installed exact replay cell (3641 bytes); only fingerprint tokens changed

promoted.tsv:2 C01 column 6 hash=false: installed exact replay cell (4150 bytes); only fingerprint tokens changed
promoted.tsv:3 C02 column 6 hash=false: installed exact replay cell (3642 bytes); only fingerprint tokens changed


Exact old/new cells: certificate-cell-changes-true.json and certificate-cell-changes-false.json. The complete unified line diff for all six inventories is inventory-changes.diff.

Exact serialized certificate cells are disclosed in [default cell changes](../../../.temp/brief-571/certificate-cell-changes-true.json), [no-default cell changes](../../../.temp/brief-571/certificate-cell-changes-false.json), and the full inventory diff linked above. Normalizing only 16-digit fingerprint tokens gives byte-identical old/new cells, so instance identities and serialized preconditions/strength/exclusions are preserved too.

For each feature set, the initial `local_production_acceptance` run generated fresh zero-rejection certificates, then failed the recorded stale C01 comparison. These expected replay failures were retained rather than omitted. After copying both C01/C02 cells from each emitted file, the same command passed. Verbatim failure and green excerpts follow.

`cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture` (hash=true; replay exit 101, final check exit 0):

```text
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=true"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1096 filtered out; finished in 17.82s
```

Successful rerun, verbatim:

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 15.08s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-3a556b9db7fd59ac)

running 1 test
test tests::retention_closure::local::production::local_production_acceptance ... body	instance	rejections
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::Unarmed;R=rum_stdlib::option_consumers::OptionConsumer;hash=true	0
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=rum_stdlib::option_consumers::OptionConsumer;hash=true	0
C02	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr;M=runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=none (nongeneric C02);hash=true	0
LeaseProbe538::probe	sealed modes	0
LeaseProbe538::start	sealed modes	0
LeaseProbe538::drive	sealed modes	0
LeaseProbe538::resume	sealed modes	0
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1096 filtered out; finished in 17.36s

```

`cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture` (hash=false; replay exit 101, final check exit 0):

```text
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=false"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1090 filtered out; finished in 18.53s
```

Successful rerun, verbatim:

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 15.34s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-5d6fedc676d25fda)

running 1 test
test tests::retention_closure::local::production::local_production_acceptance ... body	instance	rejections
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::Unarmed;R=rum_stdlib::option_consumers::OptionConsumer;hash=false	0
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=rum_stdlib::option_consumers::OptionConsumer;hash=false	0
C02	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr;M=runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=none (nongeneric C02);hash=false	0
LeaseProbe538::probe	sealed modes	0
LeaseProbe538::start	sealed modes	0
LeaseProbe538::drive	sealed modes	0
LeaseProbe538::resume	sealed modes	0
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1090 filtered out; finished in 17.93s

```

The full workspace command exited 0:

```text
python3 scripts/run-bounded.py --timeout 2400 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-571/tmp RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test --workspace --no-fail-fast -- --nocapture
```

Verbatim group headers and result lines from [workspace.txt](../../../.temp/brief-571/logs/workspace.txt), also saved in [workspace-summary.txt](../../../.temp/brief-571/workspace-summary.txt):

```text
     Running unittests src/main.rs (target/debug/deps/rum_cli-675a9b7fb4c09027)
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
     Running tests/cli.rs (target/debug/deps/cli-417da8e232f3f31f)
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s
     Running tests/input_requests.rs (target/debug/deps/input_requests-79d242612dc5614e)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s
     Running tests/terminal.rs (target/debug/deps/terminal-cd41974a9dfe37a5)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.95s
     Running unittests src/lib.rs (target/debug/deps/rum_core-49fc2a810a19f202)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_derive-b50628eb478e3209)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/accumulator_chain.rs (target/debug/deps/accumulator_chain-f4b1762065518378)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/derive.rs (target/debug/deps/derive-b38e62f915ec5c7d)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_host-a119f062096badf7)
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.64s
     Running tests/adapter_contract.rs (target/debug/deps/adapter_contract-36e2ac93dcf516de)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/batch_boundary_ownership.rs (target/debug/deps/batch_boundary_ownership-62cec20e1429084a)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/guest_loading_resume.rs (target/debug/deps/guest_loading_resume-4d8893fc9e0cdd96)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
     Running tests/host_payload_destruction_contract.rs (target/debug/deps/host_payload_destruction_contract-1be17d3af592a343)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 46.76s
     Running tests/installer.rs (target/debug/deps/installer-8ebb99c00d41e78e)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/lifecycle_buildout.rs (target/debug/deps/lifecycle_buildout-5e85c48ede3e9593)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/lifecycle_contract.rs (target/debug/deps/lifecycle_contract-a2c563982853655a)
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
     Running tests/random_password_policy.rs (target/debug/deps/random_password_policy-ddd2fec076417963)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/stress.rs (target/debug/deps/stress-5ec2a75152f1305e)
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.30s
     Running tests/suspended_timeout_contract.rs (target/debug/deps/suspended_timeout_contract-258c488f1b37cde7)
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.01s
     Running tests/suspension_abandonment_contract.rs (target/debug/deps/suspension_abandonment_contract-ac93b53b9ae0d1d1)
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
     Running unittests src/lib.rs (target/debug/deps/rum_jupyter-edb9ee3aea58135e)
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.80s
     Running unittests src/bin/rum-jupyter-kernel.rs (target/debug/deps/rum_jupyter_kernel-965588c9ac9ffa23)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_repl-edcfd3382d4bfc76)
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.43s
     Running tests/input_abandonment.rs (target/debug/deps/input_abandonment-144216c549618286)
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
     Running tests/input_continuation.rs (target/debug/deps/input_continuation-ebdbc4f7476ef252)
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
     Running unittests src/lib.rs (target/debug/deps/rum_runtime-562b258c6eef9b02)
test result: ok. 1094 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 22.62s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-c8c5a89643371dc7)
test result: ok. 1044 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 1171.27s
     Running unittests src/bin/generate-builtin-reference.rs (target/debug/deps/generate_builtin_reference-7e0be655aec7b381)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/guest_loading_resume.rs (target/debug/deps/guest_loading_resume-0992a185a5c665db)
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.78s
     Running tests/lazy_pull_continuations.rs (target/debug/deps/lazy_pull_continuations-4b49946100b6004b)
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.18s
     Running tests/loop_continuations.rs (target/debug/deps/loop_continuations-d0d938f953da2f0b)
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
     Running tests/schema_continuations.rs (target/debug/deps/schema_continuations-65e2f27929ed24fb)
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.62s
     Running unittests src/lib.rs (target/debug/deps/rum_syntax-e24f301251f42f42)
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.39s
     Running unittests src/lib.rs (target/debug/deps/rum_terminal-bae91ae0a1529dd6)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_core
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_derive
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
   Doc-tests rum_host
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.58s
   Doc-tests rum_jupyter
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_repl
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_runtime
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.17s
   Doc-tests rum_stdlib
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_syntax
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_terminal
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Nine native ZeroMQ cases **skipped**, because loopback binding was denied. Libtest counts these early returns inside Jupyter's 151 successes; they are not nine executed native passes (142 other Jupyter tests ran). The normal ignored tests shown above retain their existing state. Exact skip output:

```text
test zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::basic::native_zeromq_backend_round_trips_execute_over_loopback_sockets ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_frontend_style_control_interrupt_round_trip ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::execute::native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::session_flow::native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::stdin::native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
```

The final code-only name search (`rg -n '\b(match_expr|match_expr_interpreted|normalize_cases_spec)\b' crates scripts`) reconfirmed exactly the four unrelated AST-visitor hits:

```text
crates/rum-stdlib/src/tests/retention_closure/local/surface.rs:133:    pub(super) fn match_expr(
crates/rum-stdlib/src/tests/retention_closure/local/expressions.rs:33:            syn::Expr::Match(m) => self.match_expr(m, expected),
crates/rum-stdlib/src/tests/retention_closure/effects.rs:234:                self.match_expr(f, branch, env, expected)?,
crates/rum-stdlib/src/tests/retention_closure/effects/branches.rs:204:    pub(super) fn match_expr(
```

Final scope audit, verbatim ([saved evidence](../../../.temp/brief-571/final-scope-check.txt)):

```text
Final scope audit:
- 1258 baseline hashes checked: exactly four authorized source files changed.
- No Rust/Python/TOML source added or removed under crates; manifests, lockfile, root guides, frame lists and frozen brief-548 files are unchanged.
- Three production/test adapter files equal the baseline minus exactly their authorized blocks.
- All 18 parity cases and expected flags, three assertions, and the byte-identical owned suspension test/remainder are preserved.
- Exactly six inventory files / 104 rows changed; no inventory row count changed.
- Named frame count remains 556; both frame-list files unchanged.
- Workspace command exited 0; nine native ZeroMQ cases skipped on denied loopback.
- Issued brief is byte-identical before the authorized report append.
```

Every mandatory gate reached green. Exactly eight focused commands ran: the two baselines, one combined fingerprint/discovery replay, ownership, and four certificate runs (fresh replay and green check for each feature set). Runtime and workspace were the required broad suites. One implementation edit reduced obsolete runtime definitions/references from five to zero; each expected stale-certificate failure was cleared by installing its emitted cells. There were no stalled implementation retries and no stop-loss trigger. Full logs preserve the expected negative compiler controls; their intentional compilation errors are not failed test groups.

### 3. Retired IDs and open questions

The retirement ledger is [retirements.tsv](../../../.temp/brief-571/retirements.tsv):

| Frozen ID | Retired adapter | Frozen lost site |
| --- | --- | --- |
| P0779 | `normalize_cases_spec` | `kernel_data/rewrite_data/cases/match_q.rs:83` |
| P1019 | `match_expr` | `pattern_matching/core.rs:18` |
| P1285 | `match_expr_interpreted` | `pattern_matching_interpreted.rs:320` |

This records three retirements: **lost continuations 59 → 56 predicted; G4 16 → 13**. The adapter removals and absence of runtime references are verified; these totals are the issued brief's retirement arithmetic, not a newly run global continuation census. The frozen brief-548 snapshots are unchanged. No live G4 witness is repaired by this step.

No open question for this removal. The live G4 red fixtures remain work for steps B/C; they were not rerun, added as passing/ignored contracts, or weakened. No release build or performance panel was run, as required by this brief's test-only/uncalled-adapter rationale. No additional performance result or fresh global census is claimed.

Checkpoints and complete raw evidence are under [.temp/brief-571/](../../../.temp/brief-571/), including [PROGRESS.md](../../../.temp/brief-571/PROGRESS.md). The four source files, six maintenance TSVs and this appended report are left uncommitted for the director's gate.
