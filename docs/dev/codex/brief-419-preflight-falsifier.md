# brief-419 — make `Preflight` mean something, then tell me who is lying

Shape: **implementation**, and deliberately a small one. Worker: **cdx (Codex)**.

Gap B, step 2. Not your brief-414 draft for this step: that draft went straight
to repairing "every inventory site whose maximum output bytes can be computed
before construction", which is a scope the worker decides. This brief measures
first.

## Why this shape

`MemoryGrowthContract::Preflight`'s own documentation says "a conservative
result-size projection **can** run before construction"
(`metadata.rs:301-302`), and the type's doc adds that `Preflight` "is the
attachment point for the projector functions added by the subsequent
memory-accounting work" (`:293-295`). That was honest of brief-417. It also
means the declaration is currently a statement of possibility, not of fact.

Director-measured: **136 builtins are declared `Preflight`**, and there are
**18 calls to `enforce_active_memory_limit_for_projected_bytes`** in the entire
tree outside its own definition (`constraints.rs:175`). So at most eighteen of
the hundred and thirty-six actually project bytes before allocating, and I do
not know which, nor how badly the rest amplify.

brief-417 taught the lesson I am applying here. Its static inventory did not
find the eleven wrong declarations; the falsifier did, in one run, with numbers.
Scoping a hundred-odd repairs from a source reading would repeat exactly the
mistake that instrument avoided — so build the instrument, run it, and report.

## Goal

A `Preflight` builtin that returns materially more than its arguments **without
having projected those bytes first** fails a test, and I get the list of every
builtin that does so, with numbers.

**Repair nothing.** That is brief-420, and I will scope it from your list.

## Context pointers

- `AGENTS.md` first.
- `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:189-333` — the existing
  `NonAmplifying` falsifier, its arming predicate, and
  `assert_non_amplifying_result`. Extend this machinery; do not build a second
  one beside it.
- `crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:174-186`
  — `enforce_active_memory_limit_for_projected_bytes`, the single byte-projection
  choke point, with 18 callers.
- `docs/dev/codex/brief-414-memory-accounting-gaps.md`, appended report §3 —
  the inventory. Useful for interpreting what you find; **not** a work list here.
- `crates/rum-runtime/src/runtime/tests/session_memory.rs` — brief-415's
  invariant. If you add a `Session` field for this, that macro will stop
  compiling until you classify it. That is the structure working; give it an
  accurate row rather than routing around it.

## Deliverable

### 1. The `Preflight` falsifier

Under the same arming as the existing one (test builds through the
`test-builtin-memory-growth` feature, or `RUM_TEST_BUILTIN_MEMORY_GROWTH`, and
**not** a production memory ceiling — brief-418 settled that), after a
`Preflight` handler returns `Ok`:

- if the result's proxy bytes exceed the arguments' by more than the existing
  allowance, **and** the handler made no byte projection during the call, that
  is a failure naming the builtin and both figures.

The cheap way is a monotonic counter on `Session` bumped inside
`enforce_active_memory_limit_for_projected_bytes`, snapshotted before the
handler and compared after. A handler that returns early, passes through
symbolically, or produces something small is untouched by this rule, which is
the point: it only fires when real growth happened with no projection behind it.

Suspension matters here and is easy to get wrong: a handler that suspends and
resumes must be judged across the whole call, not across one fragment. Say what
you did about that. brief-417 kept its contract on the dispatch-stage frame; do
the same for the counter snapshot if that is what correctness needs.

### 2. Evidence the falsifier can fire

A checked-in `#[should_panic]` test with a deliberately unprojected `Preflight`
builtin, in the shape of brief-417's two. An instrument that has never been
seen to fail is not known to work — this project has been caught by that four
times, and brief-417's own falsifier only became credible when it was watched
failing.

### 3. The list

Run the armed workspace suite and report **every** builtin the falsifier
catches: name, result bytes, argument bytes, and the test that surfaced it.
Sort by the ratio. This list is the deliverable I care about most; brief-420 is
scoped from it.

If the list is empty, do not celebrate — say so plainly and then explain how
that squares with 136 declarations against 18 projection sites, because those
two numbers cannot both be right without an explanation. Some `Preflight`
builtins may bound their output by element count or integer work instead of
bytes; if that is what you find, say which, and say whether a count bound
discharges a *byte* promise. My view is that it does not — bounding counts
while bytes are the quantity at risk is the original defect of this whole line —
but I want your reading with evidence, not my assumption echoed back.

## Scope

Files you MAY modify:

- `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs`
- `crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs` — the
  counter bump only. **Do not change any limit, the stride, or what the ceiling
  admits.**
- `crates/rum-runtime/src/runtime/model/session.rs` and
  `crates/rum-runtime/src/runtime/model/mod.rs` — the counter field
- `crates/rum-runtime/src/runtime/session/session_state.rs` — if the frame or
  the reset path needs it
- `crates/rum-runtime/src/runtime/tests/session_memory.rs` — the field's
  inventory row
- `crates/rum-runtime/src/runtime/tests/memory_growth_contract.rs` and
  `crates/rum-runtime/src/runtime/tests/mod.rs`
- this brief, to append your report

If the mechanism needs a file outside this list, **STOP and report** before
editing it.

Files you MUST NOT touch: the growth classifications themselves, any builtin
handler, `crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs`,
`ResourceLimits` defaults, `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**`
except appending your report, `.git`, git config.

## Hard rules

- **Repair nothing.** No handler gains a projection in this brief, and no
  declaration changes. If the list is long, that is the finding, not a reason
  to start fixing. A builtin you cannot leave alone is a stop-loss.
- **No default-path cost.** Disarmed, the check stays a single cheap predicate
  that returns immediately. The counter bump inside the projection helper runs
  on the armed path only if you can do it for free; if it would cost the
  unarmed path, gate it and say how.
- **No behaviour change.** Nothing refuses anything new. No message changes, no
  limit changes, no stride change.
- **No new `unsafe`.** Do not weaken an existing test.
- Do NOT `git commit`, branch, push, stash, checkout, or touch git config.
  Leave everything in the working tree. Do not touch `stash@{0}`.
- Touch nothing outside this repo except `/tmp` scratch you remove afterwards.
  No network; pass `--offline` to every `cargo` command.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **This host can bind loopback**; the nine native ZeroMQ tests genuinely run
  here — I observed 9 of 9 at the brief-418 gate. If your sandbox denies it and
  you need `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`, say so and mark those nine results
  unobserved.
- **Append your report to this brief file.** brief-414 and brief-418 both left
  theirs in the dispatch log only; the first cost a wasted dispatch.

## Verification (MANDATORY — you run it, iterate to green)

```
cargo fmt --check
cargo test --offline --workspace
cargo build --offline --release
```

The workspace suite is expected to FAIL while the falsifier is catching real
liars — that is deliverable 3, not a defect. Report those failures as the list.
Then, so the tree can be committed, put the falsifier behind an explicit
opt-in for the catches you found: a named allowlist of the builtins currently
known to be unprojected, checked in and sorted, so the suite is green while the
list is visible and brief-420 can delete entries as it repairs them. **Do not
silence a catch by reclassifying the builtin or loosening the allowance.**

No microbench A/B is required if the disarmed path gains nothing measurable;
say so explicitly, and if you had to add anything to the unarmed path, stop and
tell me instead.

## Report format

1. What changed — per file, line ranges, one-line why.
2. How the falsifier works, including what you did about suspension.
3. Deliverable 2's proof, verbatim.
4. **The list**, sorted by ratio, with the test that surfaced each.
5. Your reading of the 136-vs-18 arithmetic, with evidence.
6. The allowlist you checked in, and its exact contents.
7. Verification — every command, verbatim output.
8. Stop-loss / open questions.

---

# Report — 2026-09-03

## 1. What changed

| File | Lines | Change |
|---|---:|---|
| `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs` | 7–26, 193–274, 349–474 | Extended the existing growth-contract dispatch falsifier with the sorted known-liar allowlist, an armed projection snapshot, the `Preflight` postcondition, and suspension-frame ownership of the snapshot. |
| `crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs` | 175–188 | Records an armed byte-projection event at the existing single projection choke point; limits, stride, and admission logic are unchanged. |
| `crates/rum-runtime/src/runtime/model/session.rs` | 29–36 | Added the session-local monotonic `Cell<u64>` projection counter. |
| `crates/rum-runtime/src/runtime/session/session_state.rs` | 576–585 | Initializes the projection counter to zero in `Session::empty`. |
| `crates/rum-runtime/src/runtime/tests/session_memory.rs` | 31–44, 128–138 | Classified the fixed-size counter as covered by `size_of::<Session>()` and advanced the exhaustive field count from 93 to 94. |
| `crates/rum-runtime/src/runtime/tests/memory_growth_contract.rs` | 217–353 | Added direct and resumed `#[should_panic]` proofs, direct and resumed recorded-projection controls, and the allowlist ordering/uniqueness invariant. |
| `docs/dev/codex/brief-419-preflight-falsifier.md` | appended report | Persisted the findings required to scope brief-420. |

No builtin handler or memory-growth declaration changed. No resource limit,
default, stride, message, or production refusal changed.

## 2. How the falsifier works

`try_dispatch_builtin_call` still has one cheap test-mode predicate on the
ordinary disarmed handler path. When it is false, dispatch calls the handler
and returns without a byte walk, allocation, lock, counter access, or
validation. When it is true, dispatch snapshots
`Session::builtin_memory_projection_count` immediately before calling the
handler.

`enforce_active_memory_limit_for_projected_bytes` remains the only projection
choke point. It calls an inline recorder; the recorder has the same test-mode
predicate and returns immediately when disarmed, while the armed arm performs
a saturating `Cell<u64>` increment. Thus the counter is monotonic and the
counter write exists only in test/explicit-opt-in mode. The pre-existing
memory-ceiling logic then runs unchanged.

After an armed handler returns `Ok`, the existing slow validator behaves as
follows:

- `NonAmplifying` retains its old proxy-byte postcondition and 1,024-byte
  allowance.
- `Preflight` first compares the current counter with the pre-handler snapshot.
  If they differ, a projection occurred during the call and no proxy walk is
  needed. If they are equal, it computes the existing argument and result
  proxy-byte figures. Material growth beyond the same 1,024-byte allowance
  panics unless the builtin is in the explicit known-unprojected allowlist.
- `Incremental`, `Delegated`, errors, small results, symbolic pass-throughs,
  and early returns remain untouched.

Suspension is judged across the whole call. The armed dispatch-stage frame owns
the builtin name, contract, original shared arguments, and the counter snapshot.
It is placed behind the handler's continuation frames, so it validates only
when the handler ultimately resumes to `Ok`; nested or repeated input requests
keep that same frame and snapshot. The positive
`preflight_contract_remembers_projection_across_input_resume` control records a
projection before suspension and returns a 4,128-byte result after resume
without a false panic. The negative resumed proof below shows that an
unprojected resumed result does panic.

The checker is still armed only by `cfg(test)`, the
`test-builtin-memory-growth` feature, or explicit
`RUM_TEST_BUILTIN_MEMORY_GROWTH`; a production memory ceiling does not arm it.

## 3. Falsifiability proof

Command:

```text
$ scripts/run-bounded.py --timeout 300 -- cargo test --offline -p rum-runtime preflight_contract_is_falsifiable_ -- --nocapture
```

Relevant test-harness output, verbatim (the omitted compiler prelude contained
only the same three pre-existing `dead_code` warnings from `test_builtins.rs`):

```text
running 2 tests

thread 'runtime::tests::memory_growth_contract::preflight_contract_is_falsifiable_after_input_resume' panicked at crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:402:5:
builtin InputString declared Preflight but returned 4128 proxy bytes from 39 argument proxy bytes without projecting bytes first (allowance: 1024 bytes)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

thread 'runtime::tests::memory_growth_contract::preflight_contract_is_falsifiable_through_dispatch' panicked at crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:402:5:
builtin DeliberatelyUnprojectedPreflightBuiltin declared Preflight but returned 4128 proxy bytes from 0 argument proxy bytes without projecting bytes first (allowance: 1024 bytes)
test runtime::tests::memory_growth_contract::preflight_contract_is_falsifiable_after_input_resume - should panic ... ok
test runtime::tests::memory_growth_contract::preflight_contract_is_falsifiable_through_dispatch - should panic ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 942 filtered out; finished in 0.00s
```

The two positive controls also passed in the focused eight-test module run:

```text
test runtime::tests::memory_growth_contract::preflight_contract_accepts_recorded_projection ... ok
test runtime::tests::memory_growth_contract::preflight_contract_remembers_projection_across_input_resume ... ok
```

## 4. The list

The first empty-allowlist, armed `--workspace --no-fail-fast` run exposed 13
distinct production names. Adding only those names exposed four more on the
second run, then `Append` on the third. A fourth run had no new catch. This
iteration matters because a panic ends that test before its later builtin calls.

There are **18 caught builtins**. The table is sorted by `result / arguments`;
when a builtin fired more than once, it records the largest observed ratio and
the test which produced those exact figures. Every listed test is in
`rum-stdlib`'s library test target.

| Builtin | Result bytes | Argument bytes | Ratio | Surfacing test |
|---|---:|---:|---:|---|
| `Range` | 1,600,075 | 32 | 50,002.344x | `tests::stress_corpus::bug_list_pipeline_range_50000_overflows_native_stack` |
| `FromToml` | 93,214 | 549 | 169.789x | `tests::codecs_toml::from_toml_depth_limit_accepts_each_descent_at_the_limit` |
| `RumTokens` | 4,013 | 54 | 74.315x | `tests::strings::tokenize::rum_tokens_preserves_trivia_and_operator_coverage` |
| `TokenizeWith` | 5,202,282 | 104,606 | 49.732x | `tests::stress_corpus::tokenize_with_ini_grammar_handles_generated_100kb_config` |
| `Diff` | 1,911 | 70 | 27.300x | `tests::strings::similarity::alphabet_distance_alignment_and_diff_extend_cl07` |
| `FromIni` | 1,225 | 49 | 25.000x | `tests::codecs_text::ini_to_ini_output_and_round_trips_are_pinned` |
| `FromYaml` | 1,286 | 56 | 22.964x | `tests::codecs_yaml::from_yaml_golden_corpus_is_byte_stable` |
| `UrlParse` | 1,299 | 61 | 21.295x | `tests::strings::scripting::scripting_pipeline_combines_args_url_and_aligned_output` |
| `FromXml` | 1,162 | 64 | 18.156x | `tests::codecs_xml::xml_round_trips_are_pinned` |
| `FromJson` | 1,451 | 81 | 17.914x | `tests::json::json_codec_supports_transform_round_trip` |
| `FromCsv` | 3,095 | 317 | 9.763x | `tests::codecs_text::csv_headers_tsv_ragged_rows_and_round_trips_are_pinned` |
| `ParseArgs` | 1,255 | 217 | 5.783x | `tests::strings::scripting::html_shell_path_url_and_args_utilities_cover_boundaries` |
| `DeleteMissing` | 2,004 | 925 | 2.166x | `tests::associations_missing_surface::missing_and_delete_missing_cover_first_pass_subset` |
| `Commonest` | 327,755 | 160,075 | 2.048x | `tests::exact_reduction_shapes::commonest_preserves_wide_first_occurrence_order` |
| `Append` | 320,075 | 160,107 | 1.999x | `tests::exact_reduction_shapes::commonest_preserves_wide_first_occurrence_order` |
| `Power` | 6,860 | 3,436 | 1.997x | `tests::polynomials::wide_factor_collections_preserve_polynomial_shapes` |
| `DeleteDuplicates` | 8,267 | 6,023 | 1.373x | `tests::structure_ops::dedup::custom_delete_duplicates_test_preserves_wide_pair_count_without_growing_the_stack` |
| `Union` | 8,267 | 6,641 | 1.245x | `tests::structure_ops::ordering_sets::wide_custom_same_test_set_operations_preserve_comparison_counts` |

No handler or declaration was repaired after these findings.

## 5. Reading the 136-vs-18 arithmetic

The checked inventories contain three core `Preflight` rows and 133 installed
rows:

```text
$ rg -c '=> Preflight,' crates/rum-stdlib/src/catalog/memory_growth_contract.rs crates/rum-runtime/src/runtime/tests/memory_growth_contract.rs
crates/rum-runtime/src/runtime/tests/memory_growth_contract.rs:3
crates/rum-stdlib/src/catalog/memory_growth_contract.rs:133
```

That is the brief's 136 inventory records. The installed inventory already
includes the three core names; therefore it describes 133 distinct installed
`Preflight` builtin names. A direct source-constructor count is correspondingly
3 core plus 130 stdlib declarations, or 133 production specs. This distinction
does not improve the projection coverage.

Excluding the helper definition and the two new test controls, the production
tree still contains exactly 18 calls to the byte-projection choke point:

```text
$ rg -n 'enforce_active_memory_limit_for_projected_bytes' crates --glob '*.rs' | rg -v 'debug_tools/resources/constraints.rs|runtime/tests/memory_growth_contract.rs' | wc -l
18
```

Those figures are not evidence that the other declarations discharge a byte
promise. The 18 sites are shared by some handlers, 13 use only node-count
projections, and some are themselves conditional. `Range` is the clearest
observed example: it owns one of the 18 sites, but
`enforce_range_memory_constraint` returns before that site when
`active_memory_limit()` is `None`. Test-only arming deliberately does not install
a production ceiling, so the actual call which returned 1,600,075 bytes made no
projection and was caught.

`Power` is the other useful discriminator. Its pre-allocation check projects
integer bit work against `integer_work_limit`; it never calls the byte-projection
choke point, and the falsifier caught a 6,860-byte result from 3,436 argument
bytes. In my reading, an element, iteration, or integer-work count does **not**
discharge a byte promise. It may independently prevent catastrophic work, but
it neither accounts for per-element payload size nor proves the bytes admitted
by an active memory ceiling.

Conversely, unconditional byte-aware sites such as `StringRepeat` and
`CharacterRange` increment the counter even without a ceiling. The falsifier is
dynamic and thresholded, so the absence of another name from this 18-name list
does not prove it honest: its test calls may return within the allowance, pass
through symbolically, exercise only an error/early return, or never cover a
materially amplifying input. The arithmetic and the observed catches therefore
support the brief's premise: most `Preflight` declarations remain statements of
intent, not demonstrated byte preflights.

## 6. Checked-in allowlist

The exact sorted contents are:

```rust
pub(in crate::runtime) const KNOWN_UNPROJECTED_PREFLIGHT_BUILTINS: &[&str] = &[
    "Append",
    "Commonest",
    "DeleteDuplicates",
    "DeleteMissing",
    "Diff",
    "FromCsv",
    "FromIni",
    "FromJson",
    "FromToml",
    "FromXml",
    "FromYaml",
    "ParseArgs",
    "Power",
    "Range",
    "RumTokens",
    "TokenizeWith",
    "Union",
    "UrlParse",
];
```

`known_unprojected_preflight_builtin_allowlist_is_sorted_and_unique` enforces
strict ordering and uniqueness. The allowlist is consulted only after the
handler both failed to project and exceeded the existing allowance; it changes
neither classifications nor the allowance. Brief-420 can delete an entry as
its handler gains a real byte projection.

## 7. Verification

Formatting:

```text
$ scripts/run-bounded.py --timeout 120 -- cargo --offline fmt --check
```

No output; exit 0.

Mandatory armed workspace gate:

```text
$ RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 env -u RUM_TEST_BUILTIN_MEMORY_GROWTH scripts/run-bounded.py --timeout 1200 -- cargo test --offline --workspace
```

Exit 0. The full output was 2,512 lines. This compact second run extracted every
test-target result line verbatim:

```text
$ RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 env -u RUM_TEST_BUILTIN_MEMORY_GROWTH cargo test --offline --workspace -- --quiet 2>&1 | rg '^(running [0-9]+ tests?|test result:)'
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 28 tests
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
running 7 tests
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 5 tests
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 3 tests
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 87 tests
test result: ok. 87 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s
running 7 tests
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
running 4 tests
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
running 7 tests
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
running 15 tests
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s
running 2 tests
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 6 tests
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.11s
running 12 tests
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
running 106 tests
test result: ok. 106 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 45 tests
test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
running 944 tests
test result: ok. 942 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 3.79s
running 832 tests
test result: ok. 829 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 4.16s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 189 tests
test result: ok. 189 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
running 5 tests
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 2 tests
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.97s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

That is 2,300 passed, 7 ignored, 0 failed. The nine native ZeroMQ tests are
included by the harness but are unobserved for the reason in section 8.

Release build:

```text
$ env -u RUM_TEST_BUILTIN_MEMORY_GROWTH scripts/run-bounded.py --timeout 1200 -- cargo build --offline --release
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
   Compiling rum-host v0.2.0 (/home/remi/dev/rum/crates/rum-host)
   Compiling rum-terminal v0.2.0 (/home/remi/dev/rum/crates/rum-terminal)
   Compiling rum-repl v0.2.0 (/home/remi/dev/rum/crates/rum-repl)
   Compiling rum-cli v0.2.0 (/home/remi/dev/rum/crates/rum-cli)
    Finished `release` profile [optimized] target(s) in 21.25s
```

`git diff --check` produced no output and exited 0. No microbench A/B was run:
the brief explicitly waives it for this diagnostic shape. Disarmed ordinary
dispatch retains one predicate and immediately takes the unchecked handler
path; the projection counter write is separately gated and cannot run when the
checker is off. No proxy walk, allocation, lock, or extra ordinary-handler
predicate was added to the disarmed path.

## 8. Stop-loss / open questions

The sandbox denied loopback despite the host observation in the brief. The
initial run without the skip flag failed all nine native tests with:

```text
native ZeroMQ test cannot bind loopback: Operation not permitted (os error 1); set RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 to explicitly allow this test to skip
```

I therefore used the expressly permitted `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1` for
the green workspace gates. These nine results are unobserved:

- `native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket`
- `native_zeromq_backend_round_trips_execute_over_loopback_sockets`
- `native_zeromq_backend_frontend_style_control_interrupt_round_trip`
- `native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages`
- `native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata`
- `native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown`
- `native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply`
- `native_zeromq_backend_rejects_tampered_signed_requests_end_to_end`
- `native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages`

The 136 figure is an inventory-record count; the current installed catalog has
133 distinct `Preflight` names. I preserved the brief's arithmetic above while
making that overlap explicit. The dynamic list is exhaustive for the current
armed workspace suite after iterative unmasking, but it is not a proof that the
other 115 distinct installed names project bytes; they need targeted large-input
coverage or source repair in subsequent work.

No implementation stop-loss fired. There was no network access, temporary
scratch file, `unsafe`, weakened test, classification change, handler repair,
or git mutation.
