# brief-567 — design: a resumable, depth-safe schema validator (group G3)

`Scope: NONE — no production change.` Design-only, with one declared scratch
test (see Scope). **Do not run `git` in any form**, including before you have
read this brief.

## Goal

Group G3 of the suspension-continuation branch: `ConformSchema` and
`SchemaErrors` (`crates/rum-stdlib/src/schema.rs`) resume **past** their work
when a predicate or default asks for input. brief-548 recorded seven sites:

| ID | Site | Edge |
| --- | --- | --- |
| P2056 | `schema.rs:99` | `finish_schema_errors` → `conform_value` |
| P2061 | `schema.rs:150` | `finish_conform_schema` → `conform_value` |
| P2065 | `schema.rs:233` | `conform_association` → `conform_value` |
| P2066 | `schema.rs:246` | `conform_association` → `evaluate` (a default) |
| P2067 | `schema.rs:247` | `conform_association` → `conform_value` |
| P2068 | `schema.rs:297` | `conform_list` → `conform_value` |
| P2069 | `schema.rs:309` | `conform_leaf` → `match_leaf` |

The director re-ran brief-548's probes on `2df7681b`: all nine G3 witnesses
still diverge (`ConformSchema-leaf`, `-association`, `-default`,
`-default-predicate`, `SchemaErrors-leaf`, `-association`, `-default`,
`Schema-twice`, `Schema-defaults-twice`; `.temp/director/brief548-probe-rerun-2df7681b.txt`).
Lost continuations today: 66; G3 closes seven, predicted 66 -> 59.

**The director's adjacent-boundary probe found the validator also aborts the
process.** On the ruled 2 MiB stack with a current one-codegen-unit release
build (`.temp/brief-566/candidate-06/rum-cli`, source identical to HEAD):

```
v = ConstantArray[1, Table[1, {n}]];   (* builds fine: Depth[v] == n + 1 *)
ConformSchema[v, v]
```

n = 600 passes; **n = 1 500 and 3 000 end in `thread 'main' has overflowed its
stack` (exit 134)**; at n = 6 000 the construction itself fails closed with
`evaluation exceeded native stack limit`. A schema containing patterns cannot
be built past ~150-200 levels by evaluation (the evaluator's guard fires), but
a literal schema can, and the validator's own recursion
(`conform_value` → `conform_association` / `conform_list` → `conform_value`)
has no guard. Probe files: `.temp/director/gate-g3/lit-*.rum`, `bis-*.rum`.

This is the same situation G2 was in (briefs 561-563): the structure the
repair must make resumable is a native recursion that already aborts the
process. **G3 includes making the validator depth-safe.** Design both together.

## Amendment — read first (second dispatch, 2026-10-05)

The first run was cut off by the upstream content filter while it was running
pre-fix depth probes that abort the process. Its work so far stays: the
scratch in `.temp/brief-567/` and the scratch test
`crates/rum-stdlib/tests/zz_brief567_probe.rs` — continue from them.

**It also corrected the director's premise, and it was right.**
`ConformSchema` holds its schema argument, so `ConformSchema[v, v]` validates
against the *symbol* `v` through the `MatchQ` leaf: the 1 500-level abort above
is the pattern matcher's, not the validator's (`MatchQ[v, v]` alone aborts at
1 500; the director reproduces it between 800 and 1 200). That matcher defect
is **outside G3** — the director records it for the hardening line. The
validator's own container recursion, reached with a literal schema
(`Apply[ConformSchema, {v, v}]`), passes at 1 500 and **aborts at 6 000 (list)
and 5 000 (association)** in the release build on 2 MiB
(`.temp/brief-567/depth-runs.jsonl`, `depth/`). So G3 still includes depth
safety, for the validator's walk; a leaf's `MatchQ` on a deep literal belongs
to the matcher.

**The pre-fix depth evidence is complete. Do not build or run any further
probe that overflows a stack** — not in-process, not through the CLI. Write the
deep tests for the implementation step (they will run after the fix), measure
only controls and shallow cases now, and take any per-level cost you need from
the existing runs or from `callgrind` on a passing depth.

## The director's preferred direction — evaluate it, do not assume it

G2 settled it for a unary chain: a flat representation under an explicit
cursor made every walk iterative, and the cursor *was* the suspension state.
The validator walks a **tree** (associations and lists have many children), so
the analogue is an explicit, heap-held work stack: one frame per open
container (its schema, its value, its cursor over keys or indices, its
conformed-output prefix, its path), plus a pending phase for the leaf or
default being evaluated. Suspension freezes that stack; resume applies the one
answer and continues the same loop. Compare against keeping the recursion
behind the native stack guard (fail closed with `StackLimitExceeded`) plus a
chain of per-level continuation frames, and choose on correctness and
enforceability (`CLAUDE.md`'s standard). G2's report records why the guarded
alternative lost there: `Clone`, root tracing and `Drop` are infallible and
cannot return the guard's error.

## What the design must settle

1. **The current structure, with `path:line`**: `SchemaValidator<'a>` (it
   borrows the `Session`), every recursive and evaluating edge, how errors
   accumulate (`push_error`), how `SchemaPath` grows per level (and what that
   costs at depth), `SchemaAbort` and how `InputRequired` travels through it,
   options (defaults, strictness), and the two finishers.
2. **The representation and loop**, in implementable detail: the work-stack
   frame for associations (key cursor, entry specs, missing-key defaults,
   extra keys under the strictness option) and for lists, the leaf phase
   (`match_leaf` through `MatchQ`), the default phase (evaluate the default,
   then validate it), and how a finished child is folded into its parent's
   output and errors. Every observable semantic — output shape, error order,
   error paths and messages, when defaults are evaluated, short-circuits —
   must be preserved; say how each is.
3. **Suspension and resume**: what one frame owns (the frozen stack, the
   errors so far, the pending phase and its saved value), how the answer is
   applied once, a second suspension, abandonment (no skipped validation or
   default runs), and how `SchemaErrors`' and `ConformSchema`'s finishers
   complete. Follow brief-563's settled protocol: immutable, Session-rooted
   frames with a cold working copy on an actual resume; capture only on
   `InputRequired`, on the error edge; frames derive `FrameRootTrace` and
   `PureFrameAbandon`; no `Session` borrow, observer or `NativeStackDepth` in a
   frame; rum-runtime's closed frame lists (`TEMPLATE.md` standing duty).
4. **Depth safety, enforced by structure.** No walk — validation, path
   building, error rendering, clone, trace, drop — may recurse natively in
   proportion to the value's or schema's depth. Say how this is **enforced**,
   not demonstrated: brief-563 added `lazy_depth_contract.rs`, which runs the
   recursion scanner (`recursive_edges_from_source`) over the lazy sources per
   file and as one unit, and checks no lazy type is self-reaching, each with a
   falsifier. Propose the equivalent for the schema module and say which
   helpers outside it (path rendering, `MatchQ`, `render_error`) are reached
   and whether they are bounded.
5. **The ordinary path.** A validation that never suspends must not allocate
   or clone for the continuation machinery; the work stack itself replaces the
   native stack and is measured. Name the labels (add schema labels: a small
   valid list, a small association with defaults, a deep literal schema at a
   few depths) and the plan. **Instruction gates build both arms with one
   codegen unit** (`TEMPLATE.md` pitfall 8).
6. **Inventory and layout consequences**: which ownership-graph and
   retention-contract rows the change adds, moves or re-identifies (list them
   so the director can authorize them by name), and the module layout under
   `docs/code-organization-guidelines.md`.
7. **Retention**: which brief-547 region the work stack, the conformed prefix
   and the error list fall in; no leases here.

## What I want back

1. The current structure and the recursion inventory, with evidence.
2. The design (points 2-7), the frame inventory (fields and roots), and the
   comparison with the guarded alternative.
3. Red tests, run in the scratch test and recorded: all nine brief-548 G3
   witnesses through the rum-vs-rum oracle (resumed equals queued in value,
   `log`, messages and prints; suspension asserted; exact prompts; no pending
   owner and a working `1 + 1` after), suspension at two positions, an
   ordinary error after a resume, cancellation at a suspended predicate and at
   a suspended default, a strictness/extra-key case, a nested association in a
   list in an association. **Depth cases:** the director's literal-schema
   abort, and a nested association schema, forcing every level and asserting
   the exact result, on a 2 MiB thread at a depth the descriptor-construction
   control survives in the debug test profile (measure that control first, as
   brief-563 did; do not lower n silently), plus release `rum-cli` evidence
   under `ulimit -s 2048` at a depth beyond the pre-fix ceiling. A native
   overflow aborts the test binary, so record pre-fix deep evidence by running
   each deep case alone or through the release CLI. Measure the pre-fix
   per-level cost and ceiling.
4. Drafted implementation brief(s) in `TEMPLATE.md` shape, each with its red
   tests, its predicted lost-continuation count, the named inventory rows it
   authorizes, the structural depth checks with falsifiers, the one-unit
   performance gate, the standing duty, and serial runs
   (`env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1`).
5. Open questions for the director.

## Context pointers

- brief-548's report, `.temp/brief-548/draft-G3.md`, `probes.json`,
  `lost.tsv`, `probe.rs`.
- briefs 561, 562 and 563 with their reports — the G2 design, its revision for
  depth, and the accepted implementation (`crates/rum-stdlib/src/lazy/`,
  `tests/association_depth_contract/lazy_depth_contract.rs`,
  `tests/lazy_pull_continuations.rs`); briefs 565-566 for the one-unit
  measurement and what the flat loop cost.
- `crates/rum-stdlib/src/schema.rs`; `docs/guide/` for `ConformSchema` and
  `SchemaErrors`; the generated reference.
- `crates/rum-runtime/src/runtime/suspension/core.rs`,
  `suspension/primitive.rs`, `native_stack_guard.rs`.
- `docs/dev/codex/TEMPLATE.md`, `AGENTS.md`,
  `docs/code-organization-guidelines.md`.

## Scope

- Files you MAY modify: **NONE**, except: append your report to this file;
  scratch under `.temp/brief-567/`; **one** scratch integration test,
  `crates/rum-stdlib/tests/zz_brief567_probe.rs`, **deleted before the report
  is appended**, its final content copied to `.temp/brief-567/probe.rs`.
- Everything else is read-only.

## Hard rules

- **No `git`, in any form.**
- Do NOT modify any file outside the exceptions; STOP and report instead.
- No network; nothing outside this repository; all scratch and `TMPDIR` under
  `.temp/brief-567/`.
- Commands, serial, each through `python3 scripts/run-bounded.py`:
  `env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --test zz_brief567_probe`
  (150 s); one-codegen-unit release builds of `rum-cli` from the unchanged
  tree into a target directory under `.temp/brief-567/`
  (`CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, 600 s; the director's took 98 s);
  that binary under `bash -c 'ulimit -s …; ulimit -c 0; …'` with stdin
  `/dev/null` (60 s); `valgrind --tool=callgrind` on it if attribution needs it
  (150 s). File reads, `rg`/`grep`, `python3`.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **No production code.** No new `unsafe`.

## Report format (your final message, appended to this file)

1. What changed — "design only", scratch, the deleted test, any build.
2. The current structure and recursion inventory.
3. The design, frame inventory, and the comparison.
4. The red and depth tests with their pre-fix values and the depth measurements.
5. The drafted implementation brief(s).
6. Open questions for the director.

---

## Execution report — brief 567, amended dispatch (2026-10-05)

### 1. What changed

**Design only. No production change.** Continued the existing scratch and honored the amendment: no further pre-fix stack-overflow probe was built or run, through either the test binary or CLI. No git command, network access, unsafe, external-engine comparison or parallel command was used.

The single allowed scratch integration test was extended, compiled and run. Its final bytes were copied to [.temp/brief-567/probe.rs](../../../.temp/brief-567/probe.rs), SHA-256 `d8fc0e438c6ba0e924b9effea8bf67f0dd00458f4b54e5658469b0b7c23d625d`, and **`crates/rum-stdlib/tests/zz_brief567_probe.rs` was deleted before appending this report**. All artifacts, targets and TMPDIR remain under `.temp/brief-567/`. No new build was needed: the first dispatch's successful one-codegen-unit release binary was reused for four passing-depth Callgrind runs. Its production source still matches the saved baseline.

Artifacts:

- [Detailed design](../../../.temp/brief-567/design.md): current structure, proposed types/roots/loop, exact semantics, external depth dependencies and guarded-alternative comparison.
- [Verification](../../../.temp/brief-567/verification.md), [exact oracle rows](../../../.temp/brief-567/oracle-results.tsv), [final raw test log](../../../.temp/brief-567/amended-suite-03.txt), [nine-source identity check](../../../.temp/brief-567/witness-source-check.json).
- [Inventory plan](../../../.temp/brief-567/inventory-plan.md) and [all 41 incoming schema rows](../../../.temp/brief-567/inventory-before.tsv).
- [Performance plan and passing-depth attribution](../../../.temp/brief-567/perf-plan.md), [raw instruction slopes](../../../.temp/brief-567/cost/slopes.tsv).
- [Implementation draft](../../../.temp/brief-567/draft-G3-implementation.md), [separate dependency-boundary design draft](../../../.temp/brief-567/draft-depth-dependencies-design.md), and [post-fix-only release depth programs](../../../.temp/brief-567/post-fix-depth/README.md).

Hash comparison before report append found **no changed protected file**, with only the permitted scratch-test deletion. The original first-dispatch hash baseline differs in the director's amended brief and TODO; both were already present when this dispatch captured its fresh baseline. Neither TODO nor any other production/documentation file was edited. [scope-audit.json](../../../.temp/brief-567/scope-audit.json) records the final check without using git.

### 2. Current structure and recursion inventory

All short `schema.rs` references here mean `crates/rum-stdlib/src/schema.rs` in the unchanged incoming tree.

`SchemaValidator<'a>` at :49 holds `&mut Session`, options and an error vector. Entry points :55/:106 already suspend safely while evaluating the first argument at :65/:116; their existing SharedArgs frames are :586/:607. Schemas and options are held (guide `structured-data.md:561-565`, reference :899/:4592). Parsing options at :381-413 precedes the validator. The finishers at :92/:143 call into the validator at :99/:150; SchemaErrors discards the normalized result and renders errors, while ConformSchema returns the result or emits capped messages and `$Failed`.

| edge | source | role |
| --- | --- | --- |
| finisher → conform_value | :99 P2056, :150 P2061 | lost top-level completion |
| conform_value → association/list | :184, :188 | native recursive dispatch |
| association → conform_value | :233 P2065, :247 P2067 | present field / evaluated default |
| association → evaluate | :246 P2066 | missing-key default |
| list → conform_value | :297 P2068 | each list item |
| leaf → match_leaf | :309 P2069 | mismatch decision and saved original value |
| match_leaf → evaluate(MatchQ) | :325 | returns True-only boolean interpretation at :330 |

The source scanner identifies four recursive edge identities, five actual calls: dispatch→association, dispatch→list, association→dispatch twice, list→dispatch once. There is no native guard in that SCC. `SchemaAbort::Eval` wraps InputRequired at :246/:329 and transparently unwraps it at :577; the error is not swallowed, but the validation suffix is lost.

Association parsing happens before checking value type (:200-204). Present/default children run in schema order; extra-key errors follow them in input order (:253); final output preserves input order and appends defaults in schema order (:267-280). Lists validate in one-based index order (:289-300). Missing Optional without a default skips the inner spec, and a present key never evaluates its default. A malformed schema aborts immediately; ordinary mismatches accumulate and do not short-circuit later predicates/defaults.

`push_error` :333 saves path/message strings for **every** error. The 64-entry cap is applied only at finish (:530), adding a summary if necessary. `SchemaPath::key/index` :356/:368 copy the entire parent path on each level. A list spine retains path strings of lengths 0,3,…,3n: **3n(n+1)/2 bytes** before allocation overhead; a one-character association-key spine sums to n². The copied-prefix cost is quadratic even though the path functions themselves are nonrecursive.

### 3. Design, frame inventory and comparison

Choose one **flat DFS work stack** and one `SchemaValidationSuspensionFrame`. The existing first-argument frames stay. The pending phase is either a MatchQ result or a default result; the finisher is a scalar enum in the state. There is no per-parent continuation chain.

| owned type | fields and roots |
| --- | --- |
| `SchemaState` | mode, options, Vec of open containers, one mutable path String, accumulated errors; roots are all container fields below |
| `ListWork` | shared input items, item schema, next index, conformed prefix, path-length checkpoint, waiting destination; trace every input/spec/output Expr |
| `AssociationWork` | parsed schema entries, value entries, next schema key, matched bits, conformed slots by original index, appended defaults, optional owned String→index map, path checkpoint, waiting destination; trace specs, all value keys/values, conformed slots and defaults |
| `SchemaDestination` | list item, existing index, or missing default's String key; no Expr roots |
| `SchemaAwait` | leaf's original value/schema, or default's validation spec; trace those saved Exprs |
| `SchemaValidationSuspensionFrame` | typed SchemaState and SchemaAwait; derives FrameRootTrace and PureFrameAbandon |

`SchemaContainer` is a finite enum over list/association records, and `SchemaMode` selects Errors/Conform. No type links to a parent or child state node. Working and frozen records own no Session borrow, observer, native-depth token, closure or pointer into another field. The existing index threshold stays (>16 in both schema and value); an owned index requires copying String keys, which is a measured representation cost, not hidden continuation work.

The driver loops over Visit / Next / ChildDone. A container records its waiting destination before descending. A finished child is appended to its list prefix, placed at the original association value index, or appended to defaults. Cursor advancement happens once; returning to a parent truncates the single path buffer to its saved UTF-8 byte length. Key/index appends preserve existing empty-key, dotted-key and one-based path text exactly. Only actual errors snapshot a path.

Parse only the container/entry reached at the same point as today; do not prevalidate later descendants and reorder default side effects. Preserve complete current association-key parsing before type mismatch; absent Optional laziness; True-only leaf interpretation; original leaf value rather than callback answer; full traversal despite the output cap; strict-extra ordering; Rule normalization and duplicate ordering; and malformed/Eval abort precedence. A shared completion helper renders SchemaErrors or finishes ConformSchema after the root completes, on both ordinary and resumed paths.

Only the InputRequired error edge constructs pending state and moves the machine into a frame. Ordinary success constructs no frame/frozen Arc or whole-machine clone. On actual Ok resume, a cold helper copies the flat state and pending phase, consumes the answer once, and re-enters the same loop. Err forwards without copying. The runtime keeps the dispatched frame Session-rooted until resume returns (`runtime/suspension/core.rs:156-157/:777-786`), so `try_unwrap`, interior mutability or stealing the queued state is not allowed. A second suspension freezes the new working state; runtime appends the existing suffix (:803-827). Default→predicate and two inputs within one predicate remain distinct phases. Abandonment only discards validation state; it never runs skipped defaults, validation, messages or guest cleanup.

The guarded recursive alternative can fail closed on descent but still needs separate safe cloning/tracing/destruction and per-level continuation routing. NativeStackDepth cannot be stored in a frame (`native_stack_guard.rs:19-25/:79-85`); infallible trace/drop cannot return StackLimitExceeded or omit roots. An explicit work stack gives the stronger local structural invariant and removes quadratic live paths. Its costs must still pass the measured gate.

**Structural enforcement:** discover all schema source files, scan each and the combined unit with `recursive_edges_from_source`, and reject every recursive edge. Falsifiers inject direct and cross-file recursion. After ownership discovery/exactness replay, reject every `rum_stdlib::schema::` self-reaching row, with an injected-row falsifier. Add exact-root unit tests over mixed/pending/prefix fields and flat clone/drop/abandon cases. The scanner's trait/macro/indirect-call limitations remain explicit; a clean local graph does not prove the helpers below.

**The unrestricted external depth requirement is not yet satisfied by the repository.** Source evidence, not additional crash probes:

- `rum-core/src/expr.rs:573/:637/:749` uses plain Arc-backed SharedExpr/SharedArgs. There is no iterative Drop; `ownership_graph/inventory/self_reaching.tsv:1-3` records the Expr cycles. Discarding SchemaErrors' rebuilt result or the last prefix owner can recurse through value depth.
- `FrameRootTrace` loops through fields/Vecs, but the actual memory root visitor (`debug_tools/resources/memory.rs:585-587`) invokes recursive `expr_memory_proxy_bytes` (:117/:124). The gensym visitor (`scoping/gensym_gc.rs:223`) reaches recursive `for_each_symbol_in_expr` (:272-280). These are reached by real Session-owned frames, not hypothetical custom visitors.
- `kernel_data/association_data.rs:60-125` recursively flattens nested entry-list/Association shells. Canonicalization :195-231 uses recursively defined Expr equality/hash on compound keys. Normal string-key rule layers are bounded per entry, but held schemas can contain deeper shell shapes.
- Input-like rendering has an iterative main loop (`text_rendering.rs:55-66`), but association and Display fallback paths still reach those parsers and recursive `Expr::Display` (`expr.rs:850-890`). `render_error` itself (`schema.rs:523`) is a fixed two-field record, and string-path rendering is flat.
- Deep whole-value MatchQ is explicitly outside G3 under the amendment. Literal container-schema gates reach a scalar leaf instead. Guest evaluation keeps its existing guard and limits.

The primary draft is consequently **conditional on a director boundary decision or prerequisite helper hardening**. Calling the local schema representation flat is justified; claiming all reached trace/drop/render paths are depth-safe is not. No external source change is proposed under the current scope.

**Inventory/layout:** [inventory-plan.md](../../../.temp/brief-567/inventory-plan.md) names all 41 existing rows. Four old data derives gain FrameRootTrace; the other data/closure/macro rows relocate by responsibility; the two original input frames stay. Remove only the key-path format row :363:27 and re-identify index-path format :370:23 as write!-append. Add 12 root-trace inventory entries (four old types newly derived, seven state types, one frame), one PureFrameAbandon entry, and named-frame count **555→556**. The two `Unsplit:ConformSchema` / `Unsplit:SchemaErrors` obligations remain Pending, with evidence-site relocations only. Replay all 68 local fingerprints and C01/C02 default/no-default certificate cells; unknowns stay unknown. No new host payload or self-reaching schema type is authorized.

Keep the facade and existing frames in schema.rs; extract state, parsing, formatting/finalization, association handling, the machine and cold continuation into six children, each targeting <10k characters. Current schema.rs is 19,179 characters. The dependency order is acyclic: machine returns pending error data and never imports continuation. This is an algorithm replacement and responsibility extraction, not a claimed verbatim-only split.

**Retention:** brief 547 left both declarations Unsplit/Pending. Proposed field-level mapping: R07 work/plan state; R03 association keyed slots/defaults; R02 list prefix across callbacks; R05 saved pending values; R04 discarded predicate answer; R06 error records; R10 path/message text; R14 bit/index capacities. These are future accounting obligations, not census edits or leases. The frozen owner remains S while its resumed working copy is additional live W; tracing does not discharge that overlap.

### 4. Red tests, depth tests and measurements

All tests use rum queued-input versus rum resumed-input execution. Python verified the nine original source expressions match brief-548/probes.json **byte-for-byte**. The harness checks value, log, errors/termination, messages and prints, actual suspension and exact prompts, exact pending request identity, no owner after completion, and a working 1+1. Full values and raw output are linked above.

| original G3 witness | resumed value / log | queued value / log |
| --- | --- | --- |
| ConformSchema-leaf | True / {1,2} | {1,2,3} / {1,2,3} |
| ConformSchema-association | True / {1,2} | Association(a=1,b=2,c=3) / {1,2,3} |
| ConformSchema-default | 2 / {1,2} | Association(a=1,b=2,c=3) / {1,2,3} |
| ConformSchema-default-predicate | True / {2} | Association(a=1,b=2,c=3) / {2} |
| SchemaErrors-leaf | True / {1,2} | {} / {1,2,3} |
| SchemaErrors-association | True / {1,2} | {} / {1,2,3} |
| SchemaErrors-default | 2 / {1,2} | {} / {1,2,3} |
| Schema-twice | True / {1,2}; only first prompt | {1,2,3,4,5} / {1,2,3,4,5} |
| Schema-defaults-twice | 2 / {1,2}; only first prompt | Association(a=1,b=2,c=3,d=4,e=5) / {1,2,3,4,5} |

Association notation in this table is abbreviated; oracle-results.tsv has exact `Association[Rule[…]]` results. Every original prompt is exactly `p: `, required twice in the last two witnesses.

Additional red cases cover first/third distinct prompts, two prompts inside one predicate, default then suspending predicate, a later validation error, preserved earlier errors and nonempty prints/messages, ordinary Throw after resume, malformed schema after resume, strict and allowed extras, association→list→association, and all 67 predicate evaluations despite a 64-error presentation cap. Cancellation at predicate and default both passes with exactly log `{1}`, print `before`, Cancelled termination and a usable Session; absent-Optional and present-default controls also pass.

Commands were serial, child processes bounded at 150s, offline with `env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --test zz_brief567_probe -- --nocapture`. Recording runner and exact env/argv/exit are saved. The first amended compile caught a scratch-only Option borrowing error; it was corrected. Final verbatim summary:

```text
test result: FAILED. 11 passed; 21 failed; 6 ignored; 0 measured; 0 filtered out; finished in 0.30s
```

That is **20 semantic reds plus one recursive-source red**. Three structural falsifier/no-self-reaching checks pass. Six ignored tests are explicitly post-fix, not passes. No production implementation or broad workspace gate was attempted.

The amendment's inherited pre-fix evidence is retained, not reproduced:

| 2 MiB case | passing control / validation | failing evidence |
| --- | --- | --- |
| debug list | Do construction at 1000 passes; literal validation 300/600 passes | original ConstantArray construction at 1000 failed closed; literal validation 1000 unmeasured |
| debug association | Do construction at 500 passes | literal validation 500: SIGABRT, cargo exit 101 |
| one-unit release list | construction 6000 passes; literal validation 1500 passes | literal validation 6000: exit 134 |
| one-unit release association | construction 5000 passes; literal validation 1500 passes | literal validation 5000: exit 134 |

`ConformSchema[v,v]` / `MatchQ[v,v]` at 1500 belongs to the matcher and is not counted as validator evidence. These are observed brackets, not an exact binary-searched ceiling or native-byte estimate.

The amended run rechecked safe construction and shallow validation, including teardown:

```text
PASSED kind=association mode=control n=500 including teardown
PASSED kind=list mode=control n=1000 including teardown
PASSED kind=association mode=literal n=16 including teardown
PASSED kind=list mode=literal n=16 including teardown
```

The archived test writes six fixed-depth post-fix gates at list 1000 / association 500: exact conformed output, empty SchemaErrors, and exact full mismatch path. There is no depth environment override. Six release files reserve control/conformance/errors at **8000** levels, checking each container/key and terminal leaf, beyond both pre-fix failing depths. They are **unrun**, as the amendment requires. Their controls must run first after the fix; no silent depth reduction or substituted whole-value matcher test.

Four passing-depth Callgrind runs on the saved one-unit binary, bounded at 150s each, `ulimit -s 2048; ulimit -c 0`, stdin /dev/null, all exited 0:

| literal-list depth | Ir(10 validations) | Ir(20) | Ir per validation | schema self Ir per validation |
| --- | ---: | ---: | ---: | ---: |
| 32 | 5,903,427 | 7,766,289 | 186,286.2 | 5,347 |
| 128 | 11,934,910 | 18,640,402 | 670,549.2 | 20,323 |

The N/2N difference gives **5,044.40625 instructions per added level** over 32→128, of which schema self-cost is 156. This includes allocation/path work and surrounding evaluator/Apply/disposal costs; no recursive inclusive costs were summed and no wall-clock decision was made. It is Callgrind attribution, not a hardware retirement gate or a native bytes/level measurement.

The implementation panel adds 18 schema labels: both finishers for small list, small association/defaults, literal list/association depths 8/32/128, and 33-key indexed association; retains the complete existing suite and same-tier list/association/MatchQ/table/rewrite/lazy neighbors. Both base/candidate instruction and allocation arms use one codegen unit. Full monotonic microbench A/B remains an implementation gate; there is no candidate to compare in this design-only task.

### 5. Drafted briefs

[The implementation draft](../../../.temp/brief-567/draft-G3-implementation.md) is in TEMPLATE shape with goal, exact source scope, named inventory exceptions, deliverables, original/additional red tests, post-fix depth tests, structural checks/falsifiers, all eight pitfalls, bounded serial verification, one-unit performance gates, standing maintenance duty, checkpoints, hard rules and report format. It closes **exactly seven sites, 66→59**, without touching MatchQ. The predicted new named-frame count is 556. It requires the director to settle the external depth dependency before dispatch.

[A separate dependency design draft](../../../.temp/brief-567/draft-depth-dependencies-design.md) proposes the minimal source/trait/destructor inventory needed to authorize shared-helper hardening or explicitly narrow the boundary. It closes zero continuations (66→66 before G3, or 59→59 after). It sketches safe iterative destruction, borrowed visitor worklists and parser/rendering directions, but deliberately grants **no kernel implementation scope** and no ownership-row deletion. This avoids presenting an unreviewed core representation rewrite as part of a local continuation fix.

### 6. Open questions for the director

1. **Depth contract:** the preferred local work-stack design is enforceable, but the unrestricted trace/drop/render requirement reaches existing unbounded core/runtime helpers beyond MatchQ. Recommended: settle the companion dependency design and authorize the needed prerequisites before accepting that full claim. Alternatively, explicitly narrow G3 to the validator-owned walk and retain these named limitations. A clean schema scan alone must not be the decision.
2. **Performance:** standard no-regression gates remain. The flat stack and an owned association index have real ordinary-path costs, while mutable paths remove quadratic copies. If a measured target regression remains necessary for depth correctness, the implementation must report it for a director decision, not invent an allowance or restore native recursion.
3. **Inventory authorization:** approve the exact 41-row relocation/re-identification plan, four old trace derives, eight new state/frame derives, 12 root-trace registrations, one abandon registration and frame count change before dispatch. Existing core self-reaching rows and schema retention classifications remain untouched.

No production action is pending in this design-only task. The report and reviewable follow-up material are complete; post-fix tests and the external-boundary ruling are explicitly reserved for the director's implementation sequence.
