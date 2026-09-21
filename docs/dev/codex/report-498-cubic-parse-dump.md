# brief-498 — bounded, output-proportional parse dump

## 1. What changed

| File | Lines | Change and reason |
| --- | --- | --- |
| `crates/rum-syntax/src/document.rs` | 380, 450–451 | Route the syntax section through a private emitter while preserving `debug_dump() -> String`, both production callers, and the token/trivia section. |
| Same file | 494–552 | Add layout comparisons across the recursive/iterative boundary, all syntax variants with varied leaves, and a 1,000-application dump on a 512 KiB stack. |
| `crates/rum-syntax/src/syntax_dump.rs` | 1–209 | New private emitter: one indentation writer, a 16-step recursive front, and a cold work-stack tail. Borrowed tasks cover every AST edge; derived Debug is used only for values whose nesting is independent of tree depth. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv` | 2942, 3041–3043 | Discovery-generated correction of one macro endpoint and three new macro witnesses; full disclosure in §7. |
| `docs/dev/codex/report-498-cubic-parse-dump.md` | Entire file | This report. |

The emitter writes padding directly in chunks of at most 64 spaces. No byte
passes through one formatting adapter per ancestor. Each AST node expands to
a fixed maximum of 11 tasks; slice continuations visit each element once. Each
padding byte is written once, and remaining formatting work is linear in leaf
text and nodes. Work is therefore O(emitted bytes). Pending tasks borrow the
AST and have no recursive destruction; wide sequences do not enqueue every
sibling at once.

The new implementation is 6,407 characters. `document.rs` retains 12,738
production characters and its existing source-diagnostics responsibility; the
new traversal is separated into the private module. Further separation of
`SourceText` diagnostic rendering from `ParsedSource` is outside this brief.
No evaluator hot path changed, so the brief's parse-dump A/B is the relevant
performance gate.

## 2. Exponent, before and after

Fresh release builds used Rust 1.85.0, `cargo build -p rum-cli --release
--offline --locked`, and the same CPU/event recipe in both arms:
`taskset -c 2 perf stat -e instructions:u -x, -- <binary> --parse <source>`.
Python passed each `.rum` fixture's contents as one argv element; stdout was
saved as the full dump, and stderr as the raw perf CSV. The binaries are
`target/brief-498/rum-before` and `target/brief-498/rum-after`.

No build, test suite, or other CPU-heavy workload ran alongside these timing
arms. All elapsed durations below use Python's monotonic clock. Perf exposes
this hybrid CPU's events as `cpu_atom/instructions/u` and
`cpu_core/instructions/u`; CPU 2's counted core event supplies the table.
The atom event is explicitly reported as not counted.

| Applications | Before instructions | Before ratio | After instructions | After ratio | Output bytes | Output ratio |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 100 | 1,363,488,600 | — | 5,195,416 | — | 807,076 | — |
| 200 | 10,459,596,615 | 7.671× | 10,544,770 | 2.030× | 3,134,476 | 3.884× |
| 400 | 81,949,260,237 | 7.835× | 25,226,982 | 2.392× | 12,350,288 | 3.940× |
| 800 | 648,797,348,239 | 7.917× | 78,048,473 | 3.094× | 49,025,888 | 3.970× |

The old ratios tend to 8. The new ratios are 2.030, 2.392, and 3.094: below 4
and increasing toward the quadratic output term as fixed/linear work becomes
less prominent. These finite samples are **not** being described as three
already-4.0 ratios. Combined with the bounded-work traversal above, they show
that the cubic component is gone. Instructions per output byte fall from
6.44 to 1.59 in the candidate; they increase from about 1,689 to 13,234 in the
baseline.

Baseline n=100 completed in 0.236147375 s under a 60 s bound. Each subsequent
baseline bound was `max(15, ceil(16 * preceding observed seconds))`: an 8x
cubic doubling allowance with 2x margin. Bounds for n=200/400/800 were
15/15/138 s; observed completion times were 0.744040871/8.572417238/71.683990434 s.
The outer baseline harness was bounded at 500 s, above the sum of those
per-run bounds. Candidate runs each had a 15 s bound, exceeding the preceding
recorded same-shape duration with the same conservative doubling allowance.
The candidate harness was bounded at 120 s. All completed successfully.

Raw perf output, verbatim:

`before-100.perf`:

```text
<not counted>,,cpu_atom/instructions/u,0,0.00,,
1363488600,,cpu_core/instructions/u,141285688,100.00,,
```

`before-200.perf`:

```text
<not counted>,,cpu_atom/instructions/u,0,0.00,,
10459596615,,cpu_core/instructions/u,560095054,100.00,,
```

`before-400.perf`:

```text
<not counted>,,cpu_atom/instructions/u,0,0.00,,
81949260237,,cpu_core/instructions/u,8436444775,100.00,,
```

`before-800.perf`:

```text
<not counted>,,cpu_atom/instructions/u,0,0.00,,
648797348239,,cpu_core/instructions/u,71529463140,100.00,,
```

`after-100.perf`:

```text
<not counted>,,cpu_atom/instructions/u,0,0.00,,
5195416,,cpu_core/instructions/u,8607600,100.00,,
```

`after-200.perf`:

```text
<not counted>,,cpu_atom/instructions/u,0,0.00,,
10544770,,cpu_core/instructions/u,7680837,100.00,,
```

`after-400.perf`:

```text
<not counted>,,cpu_atom/instructions/u,0,0.00,,
25226982,,cpu_core/instructions/u,19188986,100.00,,
```

`after-800.perf`:

```text
<not counted>,,cpu_atom/instructions/u,0,0.00,,
78048473,,cpu_core/instructions/u,56189507,100.00,,
```

## 3. The 1,000-application result

**Completed in 0.245555826 s, 108,660,945 user instructions, under a 15 s
bound.** The entire 76,483,688-byte dump was emitted successfully. The elapsed
measurement includes the bounded launcher and perf startup. This is compared
with the brief's previously observed >60 s timeout; no new baseline timeout
or larger fatal/ceiling probe was attempted.

Verbatim measurement record and perf output:

```text
{"phase": "after", "depth": 1000, "bound_seconds": 15, "seconds": 0.24555582599714398, "exit": 0, "bytes": 76483688}
<not counted>,,cpu_atom/instructions/u,0,0.00,,
108660945,,cpu_core/instructions/u,73769588,100.00,,
```

## 4. Empty output diffs

All full CLI dumps match, including syntax, tokens, locations, and trivia.
The shape fixture contains empty and populated lists/associations, patterns,
all slot forms, prefix/postfix/binary sugar, a derivative, a comparison chain,
compound expressions, strings, Unicode symbols, and a comment. The permanent
unit test additionally compares control-character/Unicode string escaping and
moves the varied tree across the front/tail boundary.

The following is the diff driver's verbatim output. Each `diff -u` produced
zero stdout bytes and zero stderr bytes:

```text
$ diff -u target/brief-498/before/head-0.dump target/brief-498/after/head-0.dump
exit=0; diff_bytes=0; stderr_bytes=0
$ diff -u target/brief-498/before/head-1.dump target/brief-498/after/head-1.dump
exit=0; diff_bytes=0; stderr_bytes=0
$ diff -u target/brief-498/before/head-8.dump target/brief-498/after/head-8.dump
exit=0; diff_bytes=0; stderr_bytes=0
$ diff -u target/brief-498/before/head-16.dump target/brief-498/after/head-16.dump
exit=0; diff_bytes=0; stderr_bytes=0
$ diff -u target/brief-498/before/head-17.dump target/brief-498/after/head-17.dump
exit=0; diff_bytes=0; stderr_bytes=0
$ diff -u target/brief-498/before/head-32.dump target/brief-498/after/head-32.dump
exit=0; diff_bytes=0; stderr_bytes=0
$ diff -u target/brief-498/before/head-50.dump target/brief-498/after/head-50.dump
exit=0; diff_bytes=0; stderr_bytes=0
$ diff -u target/brief-498/before/head-100.dump target/brief-498/after/head-100.dump
exit=0; diff_bytes=0; stderr_bytes=0
$ diff -u target/brief-498/before/head-200.dump target/brief-498/after/head-200.dump
exit=0; diff_bytes=0; stderr_bytes=0
$ diff -u target/brief-498/before/head-400.dump target/brief-498/after/head-400.dump
exit=0; diff_bytes=0; stderr_bytes=0
$ diff -u target/brief-498/before/head-800.dump target/brief-498/after/head-800.dump
exit=0; diff_bytes=0; stderr_bytes=0
$ diff -u target/brief-498/before/shapes.dump target/brief-498/after/shapes.dump
exit=0; diff_bytes=0; stderr_bytes=0
```

## 5. Recursion evidence and bound

Before: `SyntaxExpr` and `SyntaxExprKind` derive Debug in `ast.rs:5` and
`ast.rs:149`; `Call.head: Box<SyntaxExpr>` at `ast.rs:181–184` returns to those
same implementations for each application. The old `document.rs:380` invoked
that traversal without a guard. Consequently the call-head shape has native
stack use proportional to depth, in addition to the nested padding adapters.
This is a real finite-stack exhaustion risk, not a timeout diagnosis.

The safe witness used a 512 KiB thread and a `fmt::Write` sink that records
stack-address distance using safe pointer-to-integer casts, without any unsafe
code. It returns `fmt::Error` once the observed distance exceeds 32 KiB, leaving
480 KiB of headroom. At depths 8, 16, and 32 the complete old derived formatter
and the actual old `debug_dump()` both finished. At depths 64 and 100 the
writer returned its in-band error; the actual dump was deliberately not run.
AST parsing and final destruction occurred on the parent thread.

Verbatim output from `target/brief-498/stack_probe.rs` / `stack-before`:

```text
depth=8 stack=524288 guard=32768 formatter_ok=true max_stack_delta=8744 emitted_bytes=7774
depth=8 actual_debug_dump=completed bytes=8700
depth=16 stack=524288 guard=32768 formatter_ok=true max_stack_delta=16040 emitted_bytes=25174
depth=16 actual_debug_dump=completed bytes=26964
depth=32 stack=524288 guard=32768 formatter_ok=true max_stack_delta=30632 emitted_bytes=89158
depth=32 actual_debug_dump=completed bytes=92676
depth=64 stack=524288 guard=32768 formatter_ok=false max_stack_delta=32888 emitted_bytes=11367
depth=64 formatter=stopped_in_band_by_writer_guard; actual_debug_dump=not_run
depth=100 stack=524288 guard=32768 formatter_ok=false max_stack_delta=32888 emitted_bytes=11367
depth=100 formatter=stopped_in_band_by_writer_guard; actual_debug_dump=not_run
```

The completed samples increase by 912 stack bytes per application. Extrapolating
that measured slope to 1,000 applications gives roughly 0.91 MiB, exceeding
the witness thread's 512 KiB stack. That extrapolation is an inference from
the source and safe samples, **not** a performed overflow experiment. The
32 KiB early return belongs only to the witness sink; it was not protection
present in the original production dump.

After: `emit` tests its depth before expanding further. Depths 0 through 16
mean at most 17 simultaneous `emit` frames, followed by `emit_iterative`.
The tail uses a `Vec<Task>` and calls only the nonrecursive expansion helper.
All AST edges, including optional heads, call arguments, containers, and
comparison operands, become tasks. Leaf Debug has a fixed maximum nesting
independent of AST depth; neither indentation nor output buffering recurses
with tree depth.

The permanent 1,000-application test completes on a 512 KiB stack and verifies
all 2,001 syntax expressions and spans plus the token section. It returns the
AST to the parent for destruction, keeping the independent AST-drop issue out
of this emitter test. No parser cap or signature change was needed.

Focused test output, verbatim:

```text
   Compiling rum-syntax v0.2.0 (/home/remi/dev/rum/crates/rum-syntax)
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 12.35s
     Running unittests src/lib.rs (target/debug/deps/rum_syntax-a6fba5405a1c41ca)

running 4 tests
test document::tests::parsed_source_debug_dump_includes_syntax_tokens_and_trivia ... ok
test document::tests::debug_dump_matches_alternate_debug_across_front_and_tail ... ok
test document::tests::debug_dump_preserves_every_syntax_variant_and_leaf_layout ... ok
test document::tests::debug_dump_deep_head_chain_completes_on_small_stack ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 189 filtered out; finished in 1.57s
```

## 6. Formatting and full workspace suite

Commands:

```bash
CARGO_NET_OFFLINE=true CARGO_TERM_COLOR=never TMPDIR="$PWD/target/brief-498/tmp" python3 scripts/run-bounded.py --timeout 30 -- cargo fmt --check
CARGO_NET_OFFLINE=true CARGO_TERM_COLOR=never TMPDIR="$PWD/target/brief-498/tmp" RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 python3 scripts/run-bounded.py --timeout 900 -- cargo test --workspace --offline --locked --no-fail-fast
```

`cargo fmt --check`: exit 0, empty output (`logs/fmt.log`, zero bytes).
Full workspace: exit 0; **2,474 passed, 0 failed, 57 ignored**, summed across
32 test binaries/doc-test groups. The exact inventory and host-payload ledger
checks are included. The workspace compiled in 24.83 s. The 900 s bound used
brief-495's recorded successful workspace test durations (119.62 s total),
this dispatch's 21.92 s discovery build, and more than 6x margin.

`RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1` permits the existing sandbox skip when
loopback binding is denied; workspace green is not presented as proof of live
native transport coverage here. The existing runtime test-helper dead-code
warnings remain; no check failed.

Verbatim workspace result lines:

```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 89 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.46s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 39.61s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.18s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
test result: ok. 106 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s
test result: ok. 996 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 5.12s
test tests::ownership_graph::host_payload_ledger::production_host_payload_ledger_is_complete_and_attributed ... ok
test tests::ownership_graph::production_inventory_is_exact ... ok
test result: ok. 932 passed; 0 failed; 52 ignored; 0 measured; 0 filtered out; finished in 18.78s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.30s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.66s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## 7. Inventory diff, classified

Discovery ran before and after using:

```bash
CARGO_NET_OFFLINE=true CARGO_TERM_COLOR=never TMPDIR="$PWD/target/brief-498/tmp" python3 scripts/run-bounded.py --timeout 180 -- cargo test -p rum-stdlib --lib --offline --locked tests::ownership_graph::discovery_report_without_inventory_input -- --exact --nocapture
```

The regeneration helper first reproduced all six baseline partitions
byte-for-byte from baseline discovery. It then regenerated all six from
candidate discovery, after verifying every delta. Only `unknown_macros.tsv`
changed. The host-payload ledger was not edited. Host payloads remain 58,
self-reaching types remain 60, and UnknownConstruct increases from 7,340 to
7,343. Owning edges remain 3,568 and resolved Drop owners remain 15.

Every content change and source witness, verbatim from
`target/brief-498/logs/inventory-audit.log`:

```text
Baseline discovery regeneration: all six partitions byte-identical.
(a) macro endpoint position drift
OLD UnknownConstruct	crates/rum-syntax/src/document.rs	380	9	unexpanded macro writeln! (ends at 380:52)
OLD WITNESS writeln!(&mut output, "{:#?}", self.syntax).unwrap();
NEW UnknownConstruct	crates/rum-syntax/src/document.rs	380	9	unexpanded macro writeln! (ends at 380:69)
NEW WITNESS writeln!(&mut output, "{}", syntax_dump::Dump(&self.syntax)).unwrap();
(a) new macro from this brief
NEW UnknownConstruct	crates/rum-syntax/src/syntax_dump.rs	67	13	unexpanded macro assert! (ends at 67:29)
NEW WITNESS const { assert!(N <= 11) };
(a) new macro from this brief
NEW UnknownConstruct	crates/rum-syntax/src/syntax_dump.rs	87	23	unexpanded macro vec! (ends at 87:33)
NEW WITNESS let mut pending = vec![task];
(a) new macro from this brief
NEW UnknownConstruct	crates/rum-syntax/src/syntax_dump.rs	136	13	unexpanded macro write! (ends at 136:41)
NEW WITNESS write!(writer, "{value:#?}")?;
Regenerated all six partitions from discovery; only unknown_macros.tsv changed.
Semantic rows removed: 0; reclassifications: 0; new UnknownHostPayload: 0; class (b): 0.
```

All four content changes are class (a). The changed endpoint is the same
macro site, paired explicitly with its old row; it is not a removed semantic
row. There is no class (b), no new UnknownHostPayload, no removed semantic row,
and no reclassification. No TSV row was hand-edited.

## 8. Stop-loss / open questions

One temporary witness compilation did not reach green: it initially selected
an old release rlib by filename rather than Cargo's artifact report. Rust
rejected it before execution with:

```text
error[E0514]: found crate `rum_syntax` compiled by an incompatible version of rustc
```

The correction was to run the offline `cargo build -p rum-syntax --release
--offline --locked --message-format=json` and use its reported `.rlib` path.
That compiled and ran the same witness successfully. This obstacle did not
recur; no stop-loss condition was reached. Every required build, dump, diff,
format check, inventory gate, and workspace test run completed.

The measured after-ratios are disclosed exactly in §2; the finite samples rise
toward the output's quadratic growth and do not equal 4 at each doubling.
Output itself remains quadratic in chain depth, as required by the preserved
format. Separate derived-Debug callers and recursive AST destruction remain
outside this scope; this repair bounds the `debug_dump` emitter.

All artifacts, temporary sources, build output, and temp directories used by
this dispatch are under `target/brief-498/` or the workspace's normal `target/`.
Generated language fixtures have the `.rum` suffix. The worktree is left
uncommitted. No parser/lowering/core/scanner implementation or other forbidden
file was changed; no new unsafe code, production test-only divergence,
third-party engine comparison, network fetch, crash/overflow probe, branch,
commit, stash, checkout, push, or git-configuration change was made.
