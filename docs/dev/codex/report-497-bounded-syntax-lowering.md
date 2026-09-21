# report-497 — bounded syntax lowering

**STOPPED under the repeated-obstacle hard rule; not ready to accept.** The
resumed dispatch preserved the existing implementation and certificate tests,
rebuilt the final layout, ran ownership discovery, and measured three candidate
instruction runs on each required workload. Two successive attempts to match
every shifted inventory row to a source witness failed in the temporary audit
helper. Neither attempt reached an inventory write. The baseline evaluation
measurements, allocation equality, inventory regeneration, and fresh mandatory
workspace verification remain unfinished. No performance or workspace pass is
claimed.

The earlier certificate, falsifier, and ceiling evidence below is retained
verbatim, not represented as newly run. The rebuilt candidate binary is
byte-identical to the one used for those ceiling checks; the two lowering
sources and certificate tests are unchanged from the start of this dispatch.
The revised brief permits inventory regeneration; the earlier scope objection
is resolved and is **not** the current stop reason.

## 1. What changed

| File | Lines | Change and reason |
|---|---|---|
| `crates/rum-runtime/src/runtime/lowering.rs` | 1–151 | A shared, unwind-safe thread-local recursion budget covers all existing recursive helpers, including those in the unmodified binary child modules. Leaves stay inline; recursive locals are outlined. The private `lower_syntax_expr_with_front_depth` entry reports whether the tail ran. |
| Same file | 238–251, 346–406 | Preserve direct symbol heads without entering a recursive frame; share prefix/postfix value assembly with the tail. |
| Same file | 418–637 | A cold, nonrecursive work-stack traversal lowers calls, arguments, patterns, containers, operators, comparisons, and compound expressions. Native recursion stops after 16 lowering levels; the heap traversal rejects more than 5,500 nested lowering steps. Existing flat binary/statement spine limits remain separate. |
| `crates/rum-runtime/src/runtime/lowering/binary.rs` | 8–10, 34–127, 168–176 | Share binary value assembly and compound-item flattening without changing the existing left-chain/comparison helper files. |
| Same file | 228–378 | Cold-tail support preserves left-chain accumulator distinctions, numeric division, comparison shape, and source traversal order. |
| `crates/rum-runtime/src/runtime/lowering/tests.rs` | 1–314 | Six tests cover the branched certificate, all sugar families and resolver order, both public entry points, exact admitted head-chain elements, and budget restoration after errors. |
| This report | whole file | Evidence, stop reason, and proposed director-owned documentation correction. |

Closed failure uses **`EvalError::RecursionLimitExceeded { limit: 5500 }`**.
The bound counts nested lowering steps, not native bytes, so
`StackLimitExceeded { limit }` would mislabel its unit. The existing error is
classifiable by embedders and requires no public signature or error-enum change.
5,500 leaves the required 5,000 applications plus their enclosing source forms
admitted and refuses the settled 6,000 failure point. Both the free function and
`Session::lower_syntax_expr` inherit the guard; `resolution.rs` was not changed.

The first layout kept recursive locals in the main lowering function. Its
instruction A/B regressed. The current layout keeps atomic cases inline and
outlines `lower_recursive_syntax`, moving the guard and composite-expression
locals off leaf calls. The resumed dispatch measured this layout on both candidate workloads; the
complete instruction A/B and allocation check remain unfinished (section 5).

File-size assessment: lowering is 23,679 bytes, binary lowering 13,587 bytes,
and tests 11,469 bytes. The natural future split is the cold traversal into
`lowering/iterative.rs` and its binary planning/assembly into a sibling tail
module. New production modules are outside this brief's explicit file list,
so that split was not performed. The current duplication of some cold binary
folding logic should be consolidated when those module boundaries are in scope.

No inventory content changed in this dispatch. Discovery reported 87 new-location
or added macro/closure rows and 58 old-location rows, all in the two permitted
lowering files. Aggregate checks found no change in host-payload rows or loss of
construct multiplicities. The stricter per-row witness audit did not complete;
those aggregate checks are not substituted for the required disclosure. The
source scanner and its tests were left unchanged. Section 8 records both audit
failures.

## 2. Certificate and pre-conversion premise

The fixture has four and 24 call-head applications. Each application has a
`branch[{index, g[h[leaf]]}]` argument. The first, deepest branch alone uses
`0.125`; the others use their indices. The expected runtime tree is constructed
independently with `Expr` constructors, including exactly `Rational[1, 8]` at
that unique deepest position.

For both fixture sizes:

- Front 0 and front 16 must each equal the entire expected tree and each other.
- Following every call head to the first application, then argument indices
  `[0, 0, 1, 0, 0]`, must reach exactly the rational value `1/8`.
- Front 0 must report that the tail ran.
- Front 16 must report no tail for the four-application fixture and a tail
  handoff for the 24-application fixture.

**Premise probe:** before converting the lowering worker, a thin implementation
of the new private entry delegated to the unchanged old worker and reported
`used_tail: false`. The exact-result test passed on both fixture sizes; the
separate certificate failed on `front zero must execute the iterative tail`.
That is the observed old-build failure. No claim is made that exact output alone
fails on the old implementation: preserving that output is a requirement.
The later front-16 witness assertion is not claimed as an observed old-build
failure, because the front-zero assertion stopped that test first.

An initial premise run had a typo in a redundant expected-display string and
failed for the wrong reason. That string assertion was removed; the independent
expected `Expr` and exact deepest-rational assertions remain. Only the corrected
run below is credited. Logs for both attempts remain in `target/brief-497/`.

Corrected pre-conversion run, verbatim:
```text
running 2 tests

thread 'runtime::lowering::tests::branched_lowering_certifies_front_and_tail' panicked at crates/rum-runtime/src/runtime/lowering/tests.rs:84:9:
front zero must execute the iterative tail
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test runtime::lowering::tests::branched_lowering_certifies_front_and_tail ... FAILED
test runtime::lowering::tests::branched_lowering_matches_original_exact_result ... ok

failures:

failures:
    runtime::lowering::tests::branched_lowering_certifies_front_and_tail

test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 993 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p rum-runtime --lib`
```
## 3. Falsifier: red, then manually restored green

The tail was changed to set its tail witness and immediately return
`Expr::symbol("FalsifiedLoweringTail")`. This deliberately returns a result
without lowering anything. The certificate failed on **`front zero exact
result`**, not a compilation error, depth summary, or witness-only assertion.
The inserted return was removed with an explicit patch; no checkout, stash,
index restoration, or test-only production branch was used.

Command for both runs:

```bash
TMPDIR=/home/remi/dev/rum/target/brief-497/tmp python3 scripts/run-bounded.py --timeout 600 -- cargo test -p rum-runtime --offline --locked   runtime::lowering::tests::branched_lowering_certifies_front_and_tail   -- --exact --nocapture
```

Red, verbatim:
```text
running 1 test

thread 'runtime::lowering::tests::branched_lowering_certifies_front_and_tail' panicked at crates/rum-runtime/src/runtime/lowering/tests.rs:107:9:
assertion `left == right` failed: front zero exact result
  left: Atom(Symbol(Symbol { id: 85, identity: None, text: SymbolText { id: 85, context: "Global`", name: "FalsifiedLoweringTail" } }))
 right: Call { head: SharedExpr(Call { head: SharedExpr(Call { head: SharedExpr(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 0, identity: None, text: SymbolText { id: 0, context: "Global`", name: "f" } }))), args: SharedArgs([Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "Global`", name: "branch" } }))), args: SharedArgs([Call { head: SharedExpr(Atom(Symbol(Symbol { id: 5, identity: None, text: SymbolText { id: 5, context: "System`", name: "List" } }))), args: SharedArgs([Atom(Integer(Small(0))), Call { head: SharedExpr(Atom(Symbol(Symbol { id: 2, identity: None, text: SymbolText { id: 2, context: "Global`", name: "g" } }))), args: SharedArgs([Call { head: SharedExpr(Atom(Symbol(Symbol { id: 3, identity: None, text: SymbolText { id: 3, context: "Global`", name: "h" } }))), args: SharedArgs([Atom(Rational(Rational { numerator: 1, denominator: 8 }))]) }]) }]) }]) }]) }), args: SharedArgs([Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "Global`", name: "branch" } }))), args: SharedArgs([Call { head: SharedExpr(Atom(Symbol(Symbol { id: 5, identity: None, text: SymbolText { id: 5, context: "System`", name: "List" } }))), args: SharedArgs([Atom(Integer(Small(1))), Call { head: SharedExpr(Atom(Symbol(Symbol { id: 2, identity: None, text: SymbolText { id: 2, context: "Global`", name: "g" } }))), args: SharedArgs([Call { head: SharedExpr(Atom(Symbol(Symbol { id: 3, identity: None, text: SymbolText { id: 3, context: "Global`", name: "h" } }))), args: SharedArgs([Atom(Integer(Small(1)))]) }]) }]) }]) }]) }), args: SharedArgs([Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "Global`", name: "branch" } }))), args: SharedArgs([Call { head: SharedExpr(Atom(Symbol(Symbol { id: 5, identity: None, text: SymbolText { id: 5, context: "System`", name: "List" } }))), args: SharedArgs([Atom(Integer(Small(2))), Call { head: SharedExpr(Atom(Symbol(Symbol { id: 2, identity: None, text: SymbolText { id: 2, context: "Global`", name: "g" } }))), args: SharedArgs([Call { head: SharedExpr(Atom(Symbol(Symbol { id: 3, identity: None, text: SymbolText { id: 3, context: "Global`", name: "h" } }))), args: SharedArgs([Atom(Integer(Small(2)))]) }]) }]) }]) }]) }), args: SharedArgs([Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "Global`", name: "branch" } }))), args: SharedArgs([Call { head: SharedExpr(Atom(Symbol(Symbol { id: 5, identity: None, text: SymbolText { id: 5, context: "System`", name: "List" } }))), args: SharedArgs([Atom(Integer(Small(3))), Call { head: SharedExpr(Atom(Symbol(Symbol { id: 2, identity: None, text: SymbolText { id: 2, context: "Global`", name: "g" } }))), args: SharedArgs([Call { head: SharedExpr(Atom(Symbol(Symbol { id: 3, identity: None, text: SymbolText { id: 3, context: "Global`", name: "h" } }))), args: SharedArgs([Atom(Integer(Small(3)))]) }]) }]) }]) }]) }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test runtime::lowering::tests::branched_lowering_certifies_front_and_tail ... FAILED

failures:

failures:
    runtime::lowering::tests::branched_lowering_certifies_front_and_tail

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 998 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p rum-runtime --lib`
```
Restored green, verbatim:
```text
running 1 test
test runtime::lowering::tests::branched_lowering_certifies_front_and_tail ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 998 filtered out; finished in 0.00s
```
## 4. Ceiling gate: pre-change and current candidate

Both release binaries were built at `target/release/rum-cli` with
`cargo build -p rum-cli --release --offline --locked`. Each run used
`RLIMIT_STACK = 2 * 1024 * 1024` (the same limit as `ulimit -s 2048`), disabled
core dumps, closed stdin, and a 30-second bounded runner. The pre-change source
is HEAD `f820ec56`; the current candidate is the revised layout described above.
There were three runs per point, and no depth probe exceeded 6,000.
These are the retained prior-dispatch runs. This dispatch did not repeat the
known old-build abort. A fresh release build at the same path produced the
identical candidate binary (SHA-256
`81c0fc9a3e85091d3ebc5d0f596fa8fda8925e075ec183d66bc77e325efb18c6`).
Current HEAD `a0cb88c4` has no tracked crate changes relative to `f820ec56`,
and both saved pre-change lowering sources match current HEAD exactly.

Head-chain source, emitted on one physical line:

```text
Print["before"]; x = Hold[f[1]...]; Print["after"];
```

Nested source is `Hold[` followed by 300 `f[` applications around `1` and the
matching closing brackets. Generated files use `.rum` throughout.

| Source | Before, three runs | After, three runs |
|---|---|---|
| Head chain 4,097 | 3 × exit 0, `before` / `after` / `Null` | Same, byte-identical stdout and stderr |
| Head chain 5,000 | 3 × exit 0, `before` / `after` / `Null` | Same, byte-identical stdout and stderr |
| Head chain 6,000 | 3 × explicit stack overflow; bounded wrapper exits 250 | 3 × exit 1, classified recursion-limit error at 5,500; no process kill |
| Nested 300 | 3 × exit 1, parser limit 128 at bytes 260..261 | Same complete diagnostic, byte-identical |

All nine preserved-case stdout/stderr pairs were compared byte for byte and
matched. For 6,000, the old stderr explicitly says `thread 'main' has overflowed
its stack`; it is not inferred from the wrapper's status. Neither arm timed out.
The single-line 6,000 runs produced no stdout; no phase diagnosis was re-derived
from that observation.

Complete captured process output, verbatim:

<details>
<summary>All 24 ceiling runs</summary>

```text
base head-4097 run 1: exit=0
stdout:
before
after
Null
stderr:
```
```text
base head-4097 run 2: exit=0
stdout:
before
after
Null
stderr:
```
```text
base head-4097 run 3: exit=0
stdout:
before
after
Null
stderr:
```
```text
base head-5000 run 1: exit=0
stdout:
before
after
Null
stderr:
```
```text
base head-5000 run 2: exit=0
stdout:
before
after
Null
stderr:
```
```text
base head-5000 run 3: exit=0
stdout:
before
after
Null
stderr:
```
```text
base head-6000 run 1: exit=250
stdout:
stderr:

thread 'main' has overflowed its stack
fatal runtime error: stack overflow
```
```text
base head-6000 run 2: exit=250
stdout:
stderr:

thread 'main' has overflowed its stack
fatal runtime error: stack overflow
```
```text
base head-6000 run 3: exit=250
stdout:
stderr:

thread 'main' has overflowed its stack
fatal runtime error: stack overflow
```
```text
base nested-300 run 1: exit=1
stdout:
stderr:
error: parse nesting exceeded limit 128 at bytes 260..261
 --> 1:261
  |
1 | Hold[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[1]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]
  |                                                                                                                                                                                                                                                                     ^
error: file evaluation failed
```
```text
base nested-300 run 2: exit=1
stdout:
stderr:
error: parse nesting exceeded limit 128 at bytes 260..261
 --> 1:261
  |
1 | Hold[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[1]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]
  |                                                                                                                                                                                                                                                                     ^
error: file evaluation failed
```
```text
base nested-300 run 3: exit=1
stdout:
stderr:
error: parse nesting exceeded limit 128 at bytes 260..261
 --> 1:261
  |
1 | Hold[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[1]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]
  |                                                                                                                                                                                                                                                                     ^
error: file evaluation failed
```
```text
candidate head-4097 run 1: exit=0
stdout:
before
after
Null
stderr:
```
```text
candidate head-4097 run 2: exit=0
stdout:
before
after
Null
stderr:
```
```text
candidate head-4097 run 3: exit=0
stdout:
before
after
Null
stderr:
```
```text
candidate head-5000 run 1: exit=0
stdout:
before
after
Null
stderr:
```
```text
candidate head-5000 run 2: exit=0
stdout:
before
after
Null
stderr:
```
```text
candidate head-5000 run 3: exit=0
stdout:
before
after
Null
stderr:
```
```text
candidate head-6000 run 1: exit=1
stdout:
stderr:
! evaluation exceeded recursion limit (5500)
error: evaluation exceeded recursion limit (5500)
```
```text
candidate head-6000 run 2: exit=1
stdout:
stderr:
! evaluation exceeded recursion limit (5500)
error: evaluation exceeded recursion limit (5500)
```
```text
candidate head-6000 run 3: exit=1
stdout:
stderr:
! evaluation exceeded recursion limit (5500)
error: evaluation exceeded recursion limit (5500)
```
```text
candidate nested-300 run 1: exit=1
stdout:
stderr:
error: parse nesting exceeded limit 128 at bytes 260..261
 --> 1:261
  |
1 | Hold[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[1]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]
  |                                                                                                                                                                                                                                                                     ^
error: file evaluation failed
```
```text
candidate nested-300 run 2: exit=1
stdout:
stderr:
error: parse nesting exceeded limit 128 at bytes 260..261
 --> 1:261
  |
1 | Hold[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[1]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]
  |                                                                                                                                                                                                                                                                     ^
error: file evaluation failed
```
```text
candidate nested-300 run 3: exit=1
stdout:
stderr:
error: parse nesting exceeded limit 128 at bytes 260..261
 --> 1:261
  |
1 | Hold[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[1]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]
  |                                                                                                                                                                                                                                                                     ^
error: file evaluation failed
```
</details>

## 5. Performance and allocations

**Incomplete; no acceptance claim.** Three fresh candidate runs on each required
workload measure the layout actually in the tree. The lower-dominated baseline
below is retained from the earlier dispatch, built at the same
`target/release/rum-cli` path. No fresh baseline arm or evaluation-dominated
baseline was run before the stop. Allocation counters were not collected in
either arm. The required split therefore cannot yet be judged.

Both arms use the release profile, default features, profiling environment OFF,
CPU 2, and stdin closed. Only `cpu_core/instructions/u` is read; all measured
core rows have `100.00` coverage. Every candidate workload process exited 0.
The candidate rebuild was byte-identical to the previous final-layout binary;
these are not measurements of the superseded first layout.

Lowering workload: `target/brief-497/small-forms.rum`, these three held forms
repeated 10,000 times (30,000 forms; 1,360,000 bytes):

```text
Hold[a + b*c, f[x_, #2], <|k -> v|>];
Hold[g[x][y], {1.25, -3/7, h[z]}, p /: q[p_] := r];
Hold[a < b <= c, (f @ g @ x), s::tag, ##2 &];
```

Evaluation workload: `target/brief-497/resumed/evaluation.rum`:

```text
f[x_] := x^2 + 2*x + 1;
total = 0; Do[total = total + f[Mod[i, 97]], {i, 20000}]; Print[total];
```

The short program performs 20,000 rule applications with arithmetic and an
accumulator. Candidate stdout was identical in all three runs:

```text
63645139
Null
```

Each measured run used:

```bash
TMPDIR=/home/remi/dev/rum/target/brief-497/resumed/tmp CARGO_NET_OFFLINE=true python3 scripts/run-bounded.py --timeout 90 -- taskset -c 2 perf stat -e instructions:u -x, -- target/release/rum-cli <workload>.rum < /dev/null
```

| Workload / arm | Run 1 | Run 2 | Run 3 | Mean | Max–min / mean |
|---|---:|---:|---:|---:|---:|
| Lowering / retained base | 2,820,903,417 | 2,820,450,173 | 2,820,970,588 | 2,820,774,726.00 | 0.018449% |
| Lowering / final candidate | 2,826,732,488 | 2,826,496,128 | 2,824,458,199 | 2,825,895,605.00 | 0.080480% |
| Evaluation / final candidate | 1,261,668,641 | 1,261,682,042 | 1,261,666,470 | 1,261,672,384.33 | 0.001234% |
| Evaluation / base | not run | not run | not run | unavailable | unavailable |

Compared with the retained lowering baseline, the final candidate mean is
**+0.181542%**. This exceeds both observed within-arm spreads; it is a measured
cost on the lowering workload. The measurements are from separate dispatches,
and the required evaluation baseline and allocation comparison are absent, so
this is not a completed acceptance A/B. Evaluation delta is **unavailable**.

Allocation counters: **unmeasured**, not assumed equal. The historical timing
panel is ignored for acceptance, as the revised brief directs; its original
artifacts remain in `target/brief-497/`.

Verbatim instruction output:

retained base lowering, run 1:
```text
<not counted>,,cpu_atom/instructions/u,0,0.00,,
2820903417,,cpu_core/instructions/u,275824623,100.00,,
```
retained base lowering, run 2:
```text
<not counted>,,cpu_atom/instructions/u,0,0.00,,
2820450173,,cpu_core/instructions/u,274734024,100.00,,
```
retained base lowering, run 3:
```text
<not counted>,,cpu_atom/instructions/u,0,0.00,,
2820970588,,cpu_core/instructions/u,263733380,100.00,,
```
final candidate lowering, run 1:
```text
<not counted>,,cpu_atom/instructions/u,0,0.00,,
2826732488,,cpu_core/instructions/u,186547637,100.00,,
```
final candidate lowering, run 2:
```text
<not counted>,,cpu_atom/instructions/u,0,0.00,,
2826496128,,cpu_core/instructions/u,201112006,100.00,,
```
final candidate lowering, run 3:
```text
<not counted>,,cpu_atom/instructions/u,0,0.00,,
2824458199,,cpu_core/instructions/u,240171833,100.00,,
```
final candidate evaluation, run 1:
```text
<not counted>,,cpu_atom/instructions/u,0,0.00,,
1261668641,,cpu_core/instructions/u,86523387,100.00,,
```
final candidate evaluation, run 2:
```text
<not counted>,,cpu_atom/instructions/u,0,0.00,,
1261682042,,cpu_core/instructions/u,84933177,100.00,,
```
final candidate evaluation, run 3:
```text
<not counted>,,cpu_atom/instructions/u,0,0.00,,
1261666470,,cpu_core/instructions/u,86210471,100.00,,
```

## 6. Mandatory verification

No fresh `cargo fmt --check` or workspace gate was run before the repeated
witness-audit failure; none was started after the stop. The most recent
mandatory checks remain the prior dispatch's commands:

```bash
TMPDIR=/home/remi/dev/rum/target/brief-497/tmp python3 scripts/run-bounded.py --timeout 180 -- cargo fmt --check
TMPDIR=/home/remi/dev/rum/target/brief-497/tmp RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 python3 scripts/run-bounded.py --timeout 2400 -- cargo test --workspace --offline --locked
```

Prior fmt: exit 0, empty output. Prior workspace: exit 101, **not green** and
not complete. Completed binaries totaled **2,265 passed, 1 failed, 57 ignored**;
these are partial totals. Runtime passed 996 enabled tests, with three ignored.
The failing test was `tests::ownership_graph::production_inventory_is_exact`.
That inventory remains unchanged and unresolved. Verbatim prior result lines:

```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 89 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.47s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 38.65s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.20s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
test result: ok. 106 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
test result: ok. 996 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 5.19s
test result: FAILED. 931 passed; 1 failed; 52 ignored; 0 measured; 0 filtered out; finished in 19.56s
```

This dispatch's discovery-only command did pass:

```bash
TMPDIR=/home/remi/dev/rum/target/brief-497/resumed/tmp CARGO_NET_OFFLINE=true python3 scripts/run-bounded.py --timeout 600 -- cargo test -p rum-stdlib --offline --locked tests::ownership_graph::discovery_report_without_inventory_input -- --exact --nocapture
```

Verbatim discovery summary (not an exact-inventory pass):

```text
UnknownHostPayload count: 58
UnknownConstruct count: 7340
UnknownHostPayload breakdown: 6 storage edges; 52 public input sites
Self-reaching type count: 60
Production files: 603; test-only files: 296; nominal definitions: 1544; aliases: 37; owning edges: 3568; resolved Drop owners: 15
test tests::ownership_graph::discovery_report_without_inventory_input ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 983 filtered out; finished in 4.88s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Loopback remains the previously observed sandbox limitation; it was not probed
again in this dispatch. Retained capability output:

```text
loopback bind denied: [Errno 1] Operation not permitted; use RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1
```

The prior workspace run used **`RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`**, enabling the
PermissionDenied skip in `zeromq_native_test_support.rs`. Nine native ZeroMQ
cases returned early via that skip and are not completed transport checks.
Those early returns are separate from Cargo's ignored-test count.

## 7. Proposed director-owned TODO wording

`TODO.md` was not edited. Proposed wording while this candidate remains gated:

> Deep `Expr` and `SyntaxExpr` trees retain recursive destruction; hosts must
> bound directly constructed trees. Parser delimiter/operator guards do not
> bound every source shape: call-head chains bypass those counters. At a 2 MiB
> stack, the diagnosed 6,000-application failure is AST-to-`Expr` lowering,
> not an established drop-time failure. Brief-497's candidate closes that
> lowering failure with `RecursionLimitExceeded { limit: 5500 }` while
> preserving 4,097 and 5,000, but remains gated on exact ownership inventory,
> the complete two-workload instruction/allocation A/B, and a green workspace
> run. Lowering repair does not close the separate destruction debt; the three
> rejected iterative `Expr` drop designs and the host limitation remain
> documented in `docs/host-embedding-and-hardening.md`.

After acceptance, remove the pending-gate clause while preserving the separate
destruction limitation.

## 8. Stop-loss / open questions

The controlling hard rule is **“If the same obstacle occurs twice, STOP and
report.”** The obstacle was failure of the temporary audit helper to associate
every stale inventory row with its exact current source witness. This was a
failure of the worker's matching approach, not a demonstrated forbidden
inventory change. The helper primarily used matching source lines; formatting
and moved wrapper expressions defeated that assumption.

First failed witness audit, verbatim:

```text
STOP: unmatched inventory source witness: UnknownConstruct	crates/rum-runtime/src/runtime/lowering.rs	261	51	unexpanded macro vec! (ends at 261:64) []
```

A revised helper explicitly accounted for the three prefix arms whose enclosing
`Ok` moved to the caller. It then failed on another row, verbatim:

```text
Traceback (most recent call last):
  File "<stdin>", line 25, in <module>
AssertionError: ('UnknownConstruct\tcrates/rum-runtime/src/runtime/lowering/binary.rs\t47\t13\tunexpanded macro vec! (ends at 47:43)', [])
```

Work stopped after that second failure. Neither attempt reached its write phase:
**no TSV content changes, no removed rows, and no reclassifications were made**.
Discovery's aggregate comparison found no new `UnknownHostPayload`, but the
required per-row witness disclosure is incomplete. No class-(b) change was
established, and the revised inventory scope is not being treated as a blocker.

The six candidate performance runs had completed before the stop. After the
stop, actions were limited to reading existing evidence, checking file identity
and status, and updating this report. No third audit attempt, new benchmark,
implementation change, or test run was started.

Outstanding work is the source-witness inventory audit and regeneration, the
baseline arm for both workloads, allocation equality, and fresh mandatory fmt
and workspace gates. The candidate is uncommitted and is not presented as an
accepted repair. Production lowering files and the certificate test file remain
byte-for-byte as found at the start of this dispatch; only this report changed
outside ignored build/probe artifacts.

No commit, branch, stash, checkout, push, git configuration, network operation,
parser cap, AST change, `rum-core` edit, new unsafe code, production test-only
behavior, or external-engine comparison was introduced. No new ceiling probe
or process-abort probe was run. All artifacts are inside the repository under
`target/brief-497/resumed/`; the initial report and source bytes are preserved
there. `resolution.rs`, the ownership scanner, and binary child modules remain
untouched.
