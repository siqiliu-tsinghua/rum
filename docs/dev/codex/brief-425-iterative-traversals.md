# brief-425 — ten traversals must reach height 4,097 on a 2 MiB stack

Shape: **implementation**. Worker: **cdx (Codex)**.

Second step of the traversal line. brief-424 measured which traversals fall
short of the ruled bound; this converts them.

## The ruling and the target

`MAX_PHYSICAL_EXPR_DEPTH` will be **at least 4,097** (user ruling 2026-09-03),
preserving plan-330's documented 4,096-link contract. The bound cannot be
installed until the traversals below can handle a value at that height on the
smallest stack an embedder gets by default — **2 MiB, which is Rust's own
default for a spawned thread** (`docs/guide/resource-limits-and-sandbox.md:122`).

Director-measured on a 2 MiB stack, release, at physical height 4,097. These ten
do not complete:

`ToString`, `Map`, `Apply`, `ReplaceAll`, `Cases`, `Level`, `MemberQ`, `Count`,
`Position`, `FreeQ`

`ToString`'s ceiling is **1,550-1,600**, by far the worst. The other nine are
recorded only as "below 4,097"; bracketing them is part of this work.

**Do not convert what already clears the bound.** `Expr`'s own `Display`
handles 4,097 on 2 MiB with headroom to roughly 5,000-8,000, and `Depth`,
`LeafCount` and `ByteCount` clear it too. `Display`, the derived traits and drop
glue sit in the implicit-and-universal category where three earlier iterative
designs cost 8-49% and were rejected (`TODO.md:1061-1065`). Leave them alone.

## Start with `ToString`, and here is why it is not merely worst

Error messages reach the fat renderer: `format_message_default_fragment`
(`crates/rum-runtime/src/runtime/messages/message_format.rs:165-176`) calls
`render_input_like_expr` for a `HoldForm`, `HoldCompleteForm`, `Defer` or
`Unevaluated` wrapper. So a message that embeds one of those around a deep value
renders through the path that gives out at 1,550. Until that path is total,
every other depth failure has a reporting route that can itself fail.

The two renderers differ, and only one is short:

- `FullForm` is `Ok(Expr::string(expr.to_string()))`
  (`crates/rum-stdlib/src/structure_ops/surface.rs:78-89`) — `Expr`'s `Display`,
  which clears the bound.
- `ToString` goes through `ToStringFormat` dispatch
  (`crates/rum-stdlib/src/strings/text/builtins.rs:624-640`) into
  `render_to_string_expr_with_precedence`
  (`crates/rum-runtime/src/runtime/kernel_data/text_rendering.rs:25-34`).

So this is scoped to the precedence-aware formatter, not to rendering in
general. Establish where its per-level cost actually goes before rewriting it —
if a cheap structural change recovers the needed factor without a full explicit
stack, say so and take it.

## Deliverable

1. **Each of the ten completes at physical height 4,097 on a 2 MiB thread**, in
   release.
2. **A bracket for each**, before and after, so the report shows what each
   conversion bought. Bracket the nine that brief-424 left at "below 4,097".
3. **A release-only depth test per converted traversal**, using the existing
   harness `run_on_two_mib_thread`
   (`crates/rum-runtime/src/runtime/tests/left_chain_lowering.rs:375-383`) and
   marked `#[ignore = "release-only deployment-stack probe; run explicitly with
   --release"]`, matching plan-330's precedent
   (`left_chain_lowering.rs:562-587`). Debug frames are fatter; a 4,097 test
   must not run in the ordinary debug gate.
4. **A statement, per traversal, of what you changed and why that was the
   cheapest change that reaches the target.** An explicit work stack is one
   answer; a smaller frame is another; tail-position restructuring is another.

## What must not change

This is the part I will gate hardest.

- **No observable behavior change except that deeper values now work.** Same
  results, same messages, same evaluation order.
- **`Cases`, `Level` and `Position` produce results in traversal order** and
  take level specifications. An iterative rewrite must preserve the exact
  order and the exact level semantics, including negative and `Infinity` specs.
- **`MemberQ`, `FreeQ` and `Count` short-circuit.** Preserve *where* they stop,
  not merely what they answer — a pattern can carry a `Condition` whose guard
  runs during matching.
- **`Map` and `Apply` are `Listable`-adjacent and level-aware.** Preserve
  argument evaluation order and held-argument handling.
- **Do not weaken an existing test** to accommodate a rewrite. If a test's
  expectation genuinely must move, STOP and report it with the old and new
  values; that is my decision.

## Scope

Files you MAY modify:

- `crates/rum-runtime/src/runtime/kernel_data/text_rendering.rs`
- `crates/rum-stdlib/src/strings/text/builtins.rs` — `ToString` only
- `crates/rum-stdlib/src/functional_ops/apply_map/` — `Map`, `Apply`
- `crates/rum-runtime/src/runtime/tree_walkers/` — `Level`, `Position`
- the rewrite paths for `ReplaceAll` and `Cases`
  (`crates/rum-runtime/src/runtime/definitions/application/replace_all.rs`,
  `crates/rum-stdlib/src/rewrite_builtins.rs`,
  `crates/rum-runtime/src/runtime/kernel_data/functional_data.rs`)
- `crates/rum-runtime/src/runtime/tree_walkers/count_like.rs` — `MemberQ`,
  `Count`, `FreeQ`; `position.rs` and `level.rs` for `Position` and `Level`
- module roots of any of the above whose exported types change
- test files and test module roots in either crate
- this brief, to append your report

A file that needs only the mechanical follow-on of a signature change is in
scope. Anything more, **STOP and report**.

Files you MUST NOT touch: `crates/rum-core/src/expr.rs`'s `Display`, derived
traits, or drop glue; the memory-growth classifications or the `Preflight`
allowlist; `debug_tools/resources/`; any `ResourceLimits` default or the stride;
`AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**` except appending your report;
`.git`, git config.

## Hard rules

- **No new `unsafe`.** `rum-core` and `rum-runtime` are zero-`unsafe`.
- **Do not raise a stack, spawn a wider thread, or move work off-thread** to
  reach the target. The traversal must fit the budget it is given.
- **No default-path cost.** These are ordinary builtins on ordinary paths; an
  explicit work stack that allocates per call where recursion allocated nothing
  is a cost. Measure it (below) rather than assuming it is free.
- Do NOT `git commit`, branch, push, stash, checkout, or touch git config.
  Leave everything in the working tree. Do not touch `stash@{0}`.
- Touch nothing outside this repo except `/tmp` scratch you remove afterwards.
  No network; pass `--offline` to every `cargo` command.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only.
- **Bound every probe run** with `scripts/run-bounded.py --timeout SECONDS`,
  sized from a prior measured run of the same shape.
- **This host can bind loopback**; the nine native ZeroMQ tests genuinely run
  here — I observed 9 of 9 at the brief-422 gate.
- **Append your report to this brief file** before finishing.

## Verification (MANDATORY — you run it, iterate to green)

```
cargo fmt --check
cargo test --offline --workspace
cargo test --offline --release -- --ignored
```

The third runs the release-only depth probes, including plan-330's existing
ones; they must stay green alongside yours.

Then a microbench A/B, base against candidate in one window, because ten
ordinary builtins changed shape:

```
bash scripts/core-language-microbench.sh --output-dir <base-dir>
bash scripts/core-language-microbench.sh --output-dir <cand-dir>
```

Report the rows for labels exercising the converted builtins and their
same-tier neighbours, plus `replaceall-simple` and the matcher rows.
**Name your control labels before looking at the candidate numbers.** A
whole-run offset moves every label together and a lopsided direction split is
not evidence; if the spread is noisy, raise the iteration count rather than
reading the noise. Profiling env off for every timing number.

## Report format

1. What changed — per traversal, with the mechanism and why it was the cheapest
   change that reached the target.
2. The bracket table: each traversal before and after, on a 2 MiB stack.
3. Semantic preservation: how you established order, level and short-circuit
   behaviour are unchanged, beyond the suite passing.
4. Verification — every command, verbatim output.
5. The A/B rows with named controls.
6. Anything whose expectation had to move — reported, not decided.
7. Stop-loss / open questions.

---

## Codex report — 2026-09-03

Baseline: `6b57cf3f3672cd1c185032957f6fd249de9cd565`. All probe
sources were rum source, all probe processes were bounded, every cargo command
was offline, and no vendor engine was used.

### 1. What changed

- **`ToString`:** replaced the recursive precedence renderer with a
  `RenderTask` work stack and one output buffer. The canonical transparent
  `HoldForm` rewrite is iterative too. Atoms and flat calls with a non-System
  symbol head retain a direct `Display` fast path. This was the cheapest
  conversion because it changes only the short precedence-aware renderer;
  `Expr::Display`, `FullForm`, derived traits and drop glue are untouched.
- **`Map`:** retained the old recursive transform for call depth at most 16,
  then falls back to an explicit visit/finish stack. The iterative path reuses
  an original call when no descendant changes. The bounded hybrid avoids an
  ordinary shallow-call allocation while putting a fixed ceiling on native
  recursion.
- **`Apply`:** uses the same bounded-recursive/iterative hybrid as `Map`, with
  its own head-replacement finish operation and unchanged Association flavor
  rules. Sharing the transform driver was the smallest change that kept
  `Apply` and `Map` depth accounting identical to their previous recursive
  implementations.
- **`ReplaceAll`:** replaced recursive descent with a top-down `Visit` /
  `FinishCall` work stack. A successful rule still stops descent at that node;
  unmatched calls visit head then eligible arguments in their old order.
  Matcher scan cursors and the traversal cursor are retained across
  suspension. Atoms keep a no-stack fast path, and unchanged calls reuse their
  original shared representation. This reuses the existing indexed rule plan
  rather than introducing another rewriting engine.
- **`Cases`:** drives a shared post-order cursor and keeps ordinary results in
  a `Vec`. Persistent output is introduced only after a matcher suspends, and
  is normalized to a shared prefix before a second suspension so repeated
  resumes cannot leave a supposedly unique tail aliased. This removes the
  recursive collection frames without taxing the non-suspending result path.
- **`Level`:** uses the shared post-order cursor for deep input and the exact
  prior recursive collector for call depth at most 16. This keeps the shallow
  default path allocation-free and makes only the depth-threatening tail
  iterative.
- **`MemberQ`:** stores the shared post-order cursor in its suspension state
  and returns immediately at the first accepted match, at the same candidate
  as before.
- **`Count`:** uses the same cursor and visits every eligible candidate in the
  same post-order, preserving guard execution count and suspension position.
- **`FreeQ`:** uses the same cursor and returns immediately at the first
  accepted match, preserving its default head traversal and stop point.
- **`Position`:** uses a post-order cursor with an arena of parent-linked path
  components, so walking deeper does not clone the complete path at every
  level. Paths are materialized only for matches. Its accumulator remains
  persistent only on suspension and now safely flattens a shared tail after
  repeated resumes.

The common cursor preserves head-first child traversal, argument order,
Association value-only traversal, `Key[key]` positions, and the rule that heads
are selectable but do not contribute to negative-level depth. The cursor and
all new resumable states implement `FrameRootTrace`; the closed derived-frame
inventory and suspension-protocol inventory were updated mechanically. No
`unsafe` was added.

The largest touched implementation files remain below the 20k split threshold:
`text_rendering.rs` is 19,146 bytes, `cases/collect.rs` 17,507,
`apply_map/traversal.rs` 16,669, and `replace_all.rs` 16,205. The reusable
post-order cursor and bounded recursive `Map`/`Apply` path are separate modules
rather than growing those files past the threshold.

### 2. Before/after brackets on 2 MiB

For physical height `N`, every probe built `x = 1; Do[x = {x}, {i, N - 1}]`.
The baseline and candidate binaries were run under `ulimit -s 2048`. The table
uses reproducible brackets: each lower endpoint completed in three fresh
processes and each upper endpoint overflowed in three fresh processes. Very
narrow adjacent brackets moved by a few links with process stack layout, so I
did not present those one-process edges as exact ceilings.

`Map` and `Apply` used the same root-spanning level probe,
`{-100000, 0}`, on both binaries. Successful certificates were `2 N - 1` for
`ToString`, `N` for `Map`/`Apply`/`ReplaceAll`, zero for no-match collections,
`N - 1` for `Level`, `False` for `MemberQ`, zero for `Count`, and `True` for
`FreeQ`.

| Traversal | Baseline: completes / overflows | Candidate: completes / overflows |
|---|---:|---:|
| `ToString` | 1,500 / 1,650 | **43,000 / 44,000** |
| `Map` | 3,600 / 3,800 | **20,000 / 22,000** |
| `Apply` | 3,600 / 3,800 | **20,000 / 22,000** |
| `ReplaceAll` | 1,300 / 1,400 | **4,500 / 4,700** |
| `Cases` | 1,100 / 1,150 | **4,500 / 4,700** |
| `Level` | 4,000 / 4,100 | **43,000 / 44,000** |
| `MemberQ` | 1,450 / 1,550 | **4,500 / 4,700** |
| `Count` | 1,450 / 1,550 | **4,500 / 4,700** |
| `Position` | 1,100 / 1,200 | **4,500 / 4,700** |
| `FreeQ` | 1,450 / 1,550 | **4,500 / 4,700** |

Every candidate's reproducible lower endpoint is above 4,097. Independently,
the ten release-only tests build height 4,097 inside an actual
`std::thread::Builder::stack_size(2 * 1024 * 1024)` thread and check the exact
certificate.

### 3. Semantic preservation

- Existing `Cases`, `Level` and `Position` tests pin depth-first leaves-before-
  roots order, exact/range/negative/`Infinity` levels, head traversal,
  Association value-only behavior, keyed positions, and match limits. The
  cursor schedules head first and arguments left-to-right, then yields the
  parent, matching those tests structurally rather than sorting afterward.
- A new guard-side-effect test observes exact stopping behavior. For candidates
  `1, 2, 3`, the certificates are `List[True, 2]` for `MemberQ`,
  `List[False, 2]` for `FreeQ`, and `List[1, 3]` for `Count`: the first two stop
  after the second guard and `Count` executes all three guards.
- Two new request tests suspend matching at both the first and second
  candidates. `Cases` resumes to `List[1, 2, 3]` and `Position` to
  `List[List[1], List[2], List[3]]` without replay, loss, or an aliased-tail
  panic. The full matcher-resume and frame-root/protocol suites pass.
- `Map`/`Apply` argument normalization and evaluation were not changed. The
  transform still processes an already-prepared immutable expression, visits a
  selected head before its arguments, retains the old Association branches,
  and computes negative levels from the same head/argument depths. Existing
  operator-form, level, head and Association tests pass in both the bounded
  recursive and explicit-stack shapes.
- `ReplaceAll` still attempts rules at a node before scheduling children,
  preserves first-rule priority and held-argument filtering, and splices a
  changed `Sequence` only after the rewritten head is known. Rule-order,
  condition, pattern-test, held traversal and suspension suites pass.
- The 12-test string surface suite pins every specialized input-like syntax
  branch. `FullForm` remains on `Expr::Display`; only the precedence-aware
  formats use the work stack.

No existing expectation moved and no existing test was weakened.

### 4. Verification

Long-running commands below were wrapped in `scripts/run-bounded.py`; the
inner cargo commands are the requested commands and are all offline.

`cargo --offline fmt --check`

No output; exit status 0.

The required workspace command was run exactly (inside the bounded wrapper):

`cargo test --offline --workspace`

This execution sandbox denied every loopback bind before the native subjects
could run. Verbatim failure excerpt and result:

```text
native ZeroMQ test cannot bind loopback: Operation not permitted (os error 1); set RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 to explicitly allow this test to skip

failures:
    zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket
    zeromq::tests::native::basic::native_zeromq_backend_round_trips_execute_over_loopback_sockets
    zeromq::tests::native::frontend::control::native_zeromq_backend_frontend_style_control_interrupt_round_trip
    zeromq::tests::native::frontend::control::native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages
    zeromq::tests::native::frontend::execute::native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata
    zeromq::tests::native::frontend::session_flow::native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown
    zeromq::tests::native::frontend::stdin::native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply
    zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end
    zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages

test result: FAILED. 97 passed; 9 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s

error: test failed, to rerun pass `-p rum-jupyter --lib`
```

The non-native workspace was then checked with the repository's explicit
opt-out:

`RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test --offline --workspace`

Verbatim `test result:` lines:

```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 87 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.11s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
test result: ok. 106 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
test result: ok. 946 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 3.83s
test result: ok. 843 passed; 0 failed; 13 ignored; 0 measured; 0 filtered out; finished in 4.11s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 190 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.83s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

The required release command was also run exactly:

`cargo test --offline --release -- --ignored`

```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 28 filtered out; finished in 0.00s
```

Because the root manifest has `default-members = ["crates/rum-cli"]`, that
literal command cannot reach the runtime or stdlib probes. I therefore also
ran:

`cargo test --offline --release --workspace -- --ignored`

The relevant verbatim results, including both existing plan-330 probes and all
ten new probes, were:

```text
running 2 tests
test runtime::tests::left_chain_lowering::left_chain_two_mib_full_round_trip_accepts_and_rejects_without_abort ... ok
test runtime::tests::left_chain_lowering::left_chain_two_mib_nested_expr_drops_naturally_at_the_boundary ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 945 filtered out; finished in 0.05s

running 13 tests
test tests::iterative_traversal_depth::apply_completes_at_physical_height_4097_on_two_mib ... ok
test tests::stress_corpus::rule_based_differentiator_handles_heavy_generated_polynomial ... ok
test tests::stress_corpus::peano_arithmetic_heavy_multiplication ... ok
test tests::iterative_traversal_depth::tostring_completes_at_physical_height_4097_on_two_mib ... ok
test tests::iterative_traversal_depth::map_completes_at_physical_height_4097_on_two_mib ... ok
test tests::iterative_traversal_depth::level_completes_at_physical_height_4097_on_two_mib ... ok
test tests::stress_corpus::tokenize_with_ini_grammar_handles_heavy_generated_config ... ok
test tests::iterative_traversal_depth::replaceall_completes_at_physical_height_4097_on_two_mib ... ok
test tests::iterative_traversal_depth::position_completes_at_physical_height_4097_on_two_mib ... ok
test tests::iterative_traversal_depth::memberq_completes_at_physical_height_4097_on_two_mib ... ok
test tests::iterative_traversal_depth::count_completes_at_physical_height_4097_on_two_mib ... ok
test tests::iterative_traversal_depth::freeq_completes_at_physical_height_4097_on_two_mib ... ok
test tests::iterative_traversal_depth::cases_completes_at_physical_height_4097_on_two_mib ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 843 filtered out; finished in 1.32s
```

The focused repeated-suspension regression command was
`cargo test --offline -p rum-runtime collection_survives_multiple`:

```text
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 946 filtered out; finished in 0.00s
```

### 5. Performance A/B

Controls were named before candidate execution. Stock controls were
`head-list`, `length-list`, `deletecases-wide-no-match`,
`deletecases-wide-association-no-match`, and `replace-wide-no-match`; all
`matchq-integer-*` and `matcher-single-*` rows were also required neighbors.
Supplemental shallow controls were `Depth`, `Reverse`, and `MatchQ`; deep
controls were `FullForm`, `Depth`, `Replace`, and `DeleteCases`. Every profiling
environment variable was explicitly zero for supplemental timings, and the
stock script zeroes its profiling variables internally.

The final stock run used scale 8 and 21 repetitions after lower-repetition
samples showed a directionally lopsided spread. Commands and verbatim output:

```text
$ scripts/run-bounded.py --timeout 300 -- bash scripts/core-language-microbench.sh --output-dir /tmp/brief425-micro-base-final9 --rum-cli /tmp/rum-cli-brief425-base --scale 8 --reps 21
core microbench output: /tmp/brief425-micro-base-final9
$ scripts/run-bounded.py --timeout 300 -- bash scripts/core-language-microbench.sh --output-dir /tmp/brief425-micro-candidate-final9 --rum-cli /tmp/brief425-candidate-target/release/rum-cli --scale 8 --reps 21
core microbench output: /tmp/brief425-micro-candidate-final9
```

| Stock label | Iterations | Base seconds | Candidate seconds | Delta |
|---|---:|---:|---:|---:|
| `head-list` (control) | 16,000 | 0.00877023 | 0.00898426 | +2.44% |
| `length-list` (control) | 16,000 | 0.00880617 | 0.00897494 | +1.92% |
| `matchq-integer-success` | 8,000 | 0.00250277 | 0.00249401 | -0.35% |
| `matchq-integer-fail` | 8,000 | 0.00250622 | 0.00254600 | +1.59% |
| `matcher-single-tiny` | 8,000 | 0.00546806 | 0.00542088 | -0.86% |
| `matcher-single-wide-small` | 800 | 0.000038104 | 0.000038445 | +0.89% |
| `matcher-single-wide-large` | 1,600 | 0.000074349 | 0.000075408 | +1.42% |
| `replaceall-simple` | 4,000 | 0.00287402 | 0.00290504 | +1.08% |
| `cases-wide-no-match` | 12,800 | 0.00150246 | 0.00125588 | -16.41% |
| `count-wide-no-match` | 12,800 | 0.00118235 | 0.00116259 | -1.67% |
| `position-wide-no-match` | 12,800 | 0.00185876 | 0.00142097 | -23.55% |
| `deletecases-wide-no-match` (control) | 12,800 | 0.00131487 | 0.00138029 | +4.98% |
| `deletecases-wide-association-no-match` (control) | 12,800 | 0.00357165 | 0.00356246 | -0.26% |
| `replace-wide-no-match` (control) | 12,800 | 0.000704542 | 0.000702214 | -0.33% |

Across all 112 stock labels, 103 were within +/-5%, six were above +5%, three
were below -5%, and the arithmetic mean delta was +0.37%. The controls and
matcher neighbors split directionally; there is no coherent whole-run
regression. The two directly exercised new collectors improve materially,
`Count` is neutral, and `ReplaceAll` is neutral.

The stock corpus has no timed `ToString`, `Map`, `Apply`, `Level`, `MemberQ` or
`FreeQ` rows, so I also ran fixed-certificate traversals. Shallow results (50,000
iterations, 11 repetitions) measure the ordinary path:

| Shallow label | Base seconds | Candidate seconds | Delta |
|---|---:|---:|---:|
| `tostring-shallow` | 0.054279876 | 0.045159253 | -16.80% |
| `depth-shallow-control` | 0.033444907 | 0.033391948 | -0.16% |
| `map-shallow` | 0.138310374 | 0.135954690 | -1.70% |
| `apply-shallow` | 0.062764226 | 0.062679329 | -0.14% |
| `reverse-shallow-control` | 0.056842686 | 0.056412943 | -0.76% |
| `replaceall-shallow-no-match` | 0.077564342 | 0.075788966 | -2.29% |
| `cases-shallow-no-match` | 0.078255754 | 0.073152423 | -6.52% |
| `level-shallow-all` | 0.062919864 | 0.061661701 | -2.00% |
| `matchq-shallow-control` | 0.022705293 | 0.023343816 | +2.81% |
| `memberq-shallow-no-match` | 0.050724910 | 0.049171570 | -3.06% |
| `count-shallow-no-match` | 0.049275112 | 0.047792509 | -3.01% |
| `position-shallow-no-match` | 0.088113782 | 0.078743375 | -10.63% |
| `freeq-shallow-no-match` | 0.055321126 | 0.052787643 | -4.58% |

Depth-65 results (1,000 iterations, 11 repetitions) exercise the explicit
stacks while remaining far below the deployment probe, so timing is stable:

| Depth-65 label | Base seconds | Candidate seconds | Delta |
|---|---:|---:|---:|
| `tostring-deep65` | 0.004796121 | 0.003312389 | -30.94% |
| `fullform-deep65-control` | 0.003172129 | 0.003096119 | -2.40% |
| `depth-deep65-control` | 0.000899545 | 0.000923292 | +2.64% |
| `map-deep65-root` | 0.008821072 | 0.007474986 | -15.26% |
| `apply-deep65-root` | 0.009561187 | 0.007333896 | -23.30% |
| `replaceall-deep65-no-match` | 0.157042414 | 0.157534117 | +0.31% |
| `replace-deep65-no-match-control` | 0.159263113 | 0.163786992 | +2.84% |
| `cases-deep65-no-match` | 0.291329258 | 0.290393679 | -0.32% |
| `level-deep65-all` | 0.006123099 | 0.005269285 | -13.94% |
| `memberq-deep65-no-match` | 0.149093633 | 0.147784846 | -0.88% |
| `count-deep65-no-match` | 0.149089581 | 0.147796390 | -0.87% |
| `position-deep65-no-match` | 0.212986845 | 0.153921502 | -27.73% |
| `deletecases-deep65-no-match-control` | 0.155292304 | 0.159142504 | +2.48% |
| `freeq-deep65-no-match` | 0.157251557 | 0.155226035 | -1.29% |

No target has a measured default-path regression relative to its same-shape
controls. The stop-loss did not fire.

### 6. Expectations moved

None. No result, message, ordering, level, guard-count, held-argument or resume
expectation changed.

### 7. Stop-loss / open questions

- The exact mandatory workspace command is red only because this execution
  sandbox returns `EPERM` from loopback bind. The nine native ZeroMQ subjects
  are **unobserved, not passes**. An environment that permits loopback must run
  the unmodified command to close that external gate; no skip is claimed as a
  native pass here.
- The prescribed literal release command addresses only the root default
  member and runs zero ignored tests. The workspace-wide form is what actually
  ran and passed all 12 deployment-stack probes plus three ignored stdlib stress
  probes.
- Running the root release test command can leave `target/release/rum-cli`
  linked with the CLI's dev-only memory-growth feature through feature
  unification. Final A/B measurements therefore used a clean offline candidate
  target under `/tmp`; this changes no repository file or Cargo configuration.
- Adjacent native-stack transitions are process-layout-sensitive. The reported
  brackets deliberately trade precision for three-run reproducibility; every
  candidate lower bound still clears 4,097 with at least 403 links of measured
  headroom.
