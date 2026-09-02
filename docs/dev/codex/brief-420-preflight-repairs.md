# brief-420 — make the eighteen honest, or say why they cannot be

Shape: **implementation**. Worker: **cdx (Codex)**.

Gap B, step 3. brief-419 measured; this repairs. Its allowlist
(`eval_dispatch/mod.rs:7`) is the work list, and it is the gate: delete an
entry, and the falsifier immediately checks whether your repair is real.

## Goal

Every entry in `KNOWN_UNPROJECTED_PREFLIGHT_BUILTINS` either

- **gains a real byte projection before it allocates**, and leaves the list, or
- **stays, with a recorded reason** stating why no sound and useful up-front
  projection exists for it.

An empty list is not the target. A list whose every remaining entry carries an
argued reason is.

## The one rule that matters most

**No reclassification. None.** Do not move a builtin from `Preflight` to
`Incremental`, `Delegated`, or anything else in this brief, even when you are
confident that is where it belongs.

This is not caution, it is a specific defect I am refusing to repeat.
brief-417 reclassified eleven builtins out of `NonAmplifying`; five of them —
`Append`, `Commonest`, `DeleteDuplicates`, `DeleteMissing`, `Union` — landed in
`Preflight`, which at that moment had no enforcement, and brief-419's falsifier
caught all five the day it gained teeth. Moving a builtin out of an enforced
contract into an unenforced one is a deferral wearing a repair's clothes, and
neither of us noticed at that gate. `Incremental` has no falsifier yet. Until it
does, a move into it is invisible, and I will not accept one.

If your honest conclusion for a builtin is "this is `Incremental`, not
`Preflight`", **write that as the recorded reason and leave it in the list**.
That is the right answer and I want it — as a sentence, not as an edit.

## Context pointers

- `AGENTS.md` first.
- `docs/dev/codex/brief-419-preflight-falsifier.md`, appended report §4 — the
  table, with each builtin's observed result and argument bytes and the test
  that surfaced it. Your starting evidence.
- `crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:174-190`
  — `enforce_active_memory_limit_for_projected_bytes`, the choke point every
  repair must go through. It no-ops when no ceiling is armed, and it records the
  projection for the falsifier before that check.
- `crates/rum-stdlib/src/strings/text/builtins.rs:660-668` (`StringRepeat`) and
  `:753-757` (`CharacterRange`) — the two payload-aware projections already in
  the tree. Copy their shape.
- `docs/dev/codex/brief-414-memory-accounting-gaps.md`, appended report §3 —
  the inventory, for growth factors.

## Two things I already determined, so you do not spend the run on them

**`Range` is a false positive and needs a different fix.**
`enforce_range_memory_constraint` (`structure_ops/sequence/range.rs:28-39`)
returns at `:29-31` when no ceiling is armed, so it never reaches its
projection and the falsifier cannot see it. Its projection is real and does run
in production. I grepped the tree: it is the **only** site with that shape.

Fix it so the falsifier stops reporting it, **without adding cost to the
unarmed path** — note that `bigint_count_to_usize` does a `to_string().parse()`,
which is not free, and `Range` is used in loops. Recording the projection
before the early return is one way; find the cheapest one and say what you
chose. Then remove `Range` from the allowlist.

**My expectation about the rest, which I want you to confirm or refute.** Seven
look genuinely projectable from their arguments: `Append` (input plus one
element), `Union`, `DeleteDuplicates`, `DeleteMissing`, `Commonest` (all
bounded by the sum of their inputs), `Power` (computable bit length), and
`Diff` (bounded by both sequences). Ten look like decoders and tokenizers whose
output size depends on content, not input length — the six `From*` codecs,
`RumTokens`, `TokenizeWith`, `UrlParse`, `ParseArgs` — where any sound bound is
`input_bytes` times a per-node constant, which is loose enough to refuse work
that would have succeeded.

**That is my reading from the outside and it may be wrong.** Check each one and
say so if it is. I would rather be corrected than agreed with.

## The hazard that decides how loose a projection may be

A projection that over-estimates refuses work that would have fit. That is a
usability defect this line has no licence to introduce, and it is harder to
notice than an under-count because nothing fails loudly — a script just gets
`TerminatedEvaluation["MemoryLimit"]` where it used to get an answer.

So for **every** builtin you repair, add both halves:

1. a low-ceiling test proving refusal happens **before** the large allocation —
   check the process high-water mark, not just the returned error, the way
   brief-413's `StringRepeat` probe did; and
2. a **realistic-ceiling test proving an ordinary input still succeeds**, sized
   so that a projection which is loose by more than a small factor would fail
   it.

The second is the one that matters here. If you cannot write it for a builtin,
that builtin's projection is too loose and it belongs in the "recorded reason"
column instead.

## Scope

Files you MAY modify:

- `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs` — the allowlist and its
  recorded reasons only
- `crates/rum-runtime/src/runtime/arithmetic/canonical.rs` — `Power`
- `crates/rum-stdlib/src/structure_ops/sequence/range.rs` — `Range`
- `crates/rum-stdlib/src/structure_ops/surface.rs`, `ordering_sets.rs`,
  `dedup.rs` — `Append`, `Union`, `DeleteDuplicates`
- `crates/rum-stdlib/src/associations/missing.rs` — `DeleteMissing`
- `crates/rum-stdlib/src/arithmetic/stats.rs` — `Commonest`
- `crates/rum-stdlib/src/strings/similarity/mod.rs` — `Diff`
- `crates/rum-stdlib/src/{csv,json,ini,toml,xml,yaml}.rs` and their existing
  decode submodules
- `crates/rum-stdlib/src/strings/tokenize.rs`,
  `strings/scripting/url.rs`, `strings/scripting/args.rs`
- the module roots of anything above whose exported types change
- test files and test module roots in either crate
- this brief, to append your report

A file that needs **only** a projection call added is expected and in scope
even if I did not name it. Anything more than that, **STOP and report**.

Files you MUST NOT touch: the growth classifications
(`catalog/memory_growth_contract.rs`, `tests/memory_growth_contract.rs`'s
inventory rows), `debug_tools/resources/memory.rs`, any `ResourceLimits`
default or the stride, `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**` except
appending your report, `.git`, git config.

## Hard rules

- **No reclassification**, per the section above. This is the stop-loss I care
  about most.
- **Do not loosen the falsifier**, its allowance, or the meaning of the
  allowlist to make an entry disappear. An entry leaves the list only by the
  handler actually projecting bytes.
- **No default-path cost.** A projection call on an unarmed session must cost
  what the existing eighteen cost: the choke point's own early return. If a
  repair needs an expensive size computation, that computation must not run when
  no ceiling is armed — `Range` is the worked example.
- **Saturating or checked arithmetic only.** A projection that overflows must
  fail closed, never wrap to a small number.
- **Do not weaken an existing generated-element or integer-work bound.** Add
  the byte projection alongside; those bounds stay.
- **No new `unsafe`.** Do not weaken an existing test.
- Do NOT `git commit`, branch, push, stash, checkout, or touch git config.
  Leave everything in the working tree. Do not touch `stash@{0}`.
- Touch nothing outside this repo except `/tmp` scratch you remove afterwards.
  No network; pass `--offline` to every `cargo` command.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **This host can bind loopback**; the nine native ZeroMQ tests genuinely run
  here — I observed 9 of 9 at the brief-419 gate. If your sandbox denies it and
  you need `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`, say so and mark those nine
  unobserved.
- **Append your report to this brief file** before finishing.

## Verification (MANDATORY — you run it, iterate to green)

```
cargo fmt --check
cargo test --offline -p rum-stdlib
cargo test --offline -p rum-runtime
cargo test --offline --workspace
```

The workspace run must be green with your shortened allowlist. Since the
falsifier is armed structurally in test builds (brief-418), a repair that does
not actually project will fail it — that is the gate, and it is automatic.

Then a microbench A/B, base against candidate in one window, because you are
adding calls to builtin paths:

```
bash scripts/core-language-microbench.sh --output-dir <base-dir>
bash scripts/core-language-microbench.sh --output-dir <cand-dir>
```

Report the rows for the labels touching the repaired builtins and their
same-tier neighbours. **Name your control labels before looking at the
candidate numbers.** A whole-run offset moves every label together and a
lopsided direction split is not evidence. Profiling env off for every timing
number.

## Report format

1. What changed — per file, line ranges, one-line why.
2. Per builtin, a table row: repaired or retained, the projection formula if
   repaired, the recorded reason if retained.
3. Whether my seven-versus-ten reading was right, with what corrected it.
4. What you did for `Range`, and how you know it costs the unarmed path
   nothing.
5. The realistic-ceiling tests: what input each admits, and how you sized them
   to catch an over-loose projection.
6. The final allowlist contents.
7. Verification — every command, verbatim output, plus the A/B rows with named
   controls.
8. Stop-loss / open questions.

---

# Execution report

## 1. What changed

No growth contract was reclassified. Every removed allowlist name now reaches
`enforce_active_memory_limit_for_projected_bytes` in its handler before the
allocation it guards; the eight retained names remain `Preflight` and have an
individual recorded reason at the allowlist site.

| File and current lines | Change |
|---|---|
| `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:7-38` | Shortened the known-unprojected allowlist from eighteen names to eight and replaced bare names with the parser/tokenizer-specific reason each remains. |
| `crates/rum-runtime/src/runtime/arithmetic/canonical.rs:27-45,142-192,391-447` | Added exact-integer and symbolic/distributed `Power` byte projections, including a tight power-of-two bit-length case and saturating recursive expression proxies. |
| `crates/rum-runtime/src/runtime/tests/arithmetic.rs:7-81` | Added isolated-process low-ceiling/HWM and realistic-ceiling `Power` tests without changing the integer-work bound. |
| `crates/rum-stdlib/src/structure_ops/sequence/range.rs:28-41` | Made the unarmed `Range` branch call the choke point with zero before returning, while keeping its `BigInt` string conversion exclusively on the armed branch. |
| `crates/rum-stdlib/src/structure_ops/surface.rs:2,235-241,251-292` | Added `Append`'s recursive call-result projection and used exact result-vector capacity; also supplies the shared safe recursive proxy used by the structure handlers. |
| `crates/rum-stdlib/src/structure_ops/ordering_sets.rs:226-281` | Added `Union`'s same-head concatenation projection before group/result allocation and exact capacities for its scan/kept vectors. |
| `crates/rum-stdlib/src/structure_ops/dedup.rs:82,107,116,396` | Added structural and custom-`SameTest` `DeleteDuplicates` projections and exact input-sized kept-vector capacities. |
| `crates/rum-stdlib/src/associations/missing.rs:6-8,70-71,84-237` | Added mode/depth-aware `DeleteMissing` projection, including conservative accounting for delayed-rule wrappers. |
| `crates/rum-stdlib/src/arithmetic/stats.rs:2,46-84,100,123-143` | Added `Commonest`'s input-bounded result projection and exact result capacity; the existing max-frequency scan now also counts winners. |
| `crates/rum-stdlib/src/strings/similarity/mod.rs:373-494` | Added string/list `Diff` projections for character/item buffers, both alignment matrices, aligned pairs/blocks, render wrappers, and payload. |
| `crates/rum-stdlib/src/strings/scripting/args.rs:13-116` | Delayed argument-string cloning until after an armed grammar pre-scan and added a shape-exact `ParseArgs` result projection. |
| `crates/rum-stdlib/src/strings/scripting/url.rs:58-61,75-111,163-205` | Added the fixed-six-field `UrlParse` association projection, including synthesized default-scheme storage; projection now occurs after option validation but before cloning a valid `DefaultScheme`. |
| `crates/rum-stdlib/src/tests.rs:149-150` | Registered the focused stdlib preflight test module. |
| `crates/rum-stdlib/src/tests/preflight_memory.rs:1-280` | Added isolated low-ceiling/HWM and realistic-ceiling probes for all nine repaired stdlib handlers, including `Range`. |

The six codec implementations and two tokenizer implementations were inspected
but not edited because their entries were retained. All size arithmetic added
here is checked or saturating; conversion overflow projects `usize::MAX` and
therefore fails closed.

## 2. Per-builtin disposition

`proxy(x)` below means the recursive in-memory expression proxy: one `Expr`
slot per node plus existing string/symbol text and `BigInt`/rational payload.
All repaired formulas are evaluated only with a production ceiling armed; the
unarmed branch performs only the active-limit check and the choke point's zero
projection/early return.

| Builtin | Disposition | Projection formula or recorded reason |
|---|---|---|
| `Append` | Repaired | `sizeof(Expr) + proxy(head) + sum(proxy(old items)) + proxy(appended item)`. The result vector is allocated at exactly `old_len + 1`. |
| `Commonest` | Repaired | `sizeof(Expr) + proxy(System\`List) + sum(proxy(input items))`; every result is an input subset. Exact result capacity removes the prior `(Expr, count)` vector-capacity overhang. |
| `DeleteDuplicates` | Repaired | `sizeof(Expr) + proxy(input head) + sum(proxy(input items))`; both structural and custom-test results are input subsets, with input-sized kept capacity. |
| `DeleteMissing` | Repaired | `proxy(keyed input) + delayed_wrapper_count(mode, depth) * 64 * sizeof(Expr)`. The recursive count follows the actual list/association and `Rule`/`RuleDelayed` traversal modes; the wrapper allowance covers the delayed evaluation reconstruction. |
| `Diff` | Repaired | `(m+1)(n+1) * (sizeof(isize)+sizeof(AlignmentStep))`, matrix row headers, at most `m+n` aligned pairs/items/blocks, at most `12(m+n)+17` render `Expr` slots, and twice the source payload. String character counting and expression proxy walking are armed-only. |
| `FromCsv` | Retained | Quote-aware row/field cardinality is known only after CSV parsing. One result node per input byte is sound but materially rejects long scalar fields and comment-heavy inputs whose results are small; an incremental parser-integrated check is needed. |
| `FromIni` | Retained | Sections, comments, blank lines, and scalar widths make shape content-dependent. A byte multiplier over-projects long values/ignored lines, while an exact pre-pass duplicates the parser; an incremental parser-integrated check is needed. |
| `FromJson` | Retained | Equal byte lengths can encode one scalar or a punctuation-dense tree. The sound per-byte node bound is unusably loose for long strings/numbers; an incremental parser-integrated check is needed. |
| `FromToml` | Retained | Dotted keys, arrays/tables, temporal values, comments, and strings expand differently. A useful bound requires TOML parse state; an incremental parser-integrated check is needed. |
| `FromXml` | Retained | Markup, attributes, entities, comments, and text have sharply different node counts. The worst per-byte case over-refuses large text nodes; an incremental parser-integrated check is needed. |
| `FromYaml` | Retained | Indentation, block scalars, comments, collections, and plain scalars make shape parse-dependent. A length bound over-refuses ordinary block scalars/comments; an incremental parser-integrated check is needed. |
| `ParseArgs` | Repaired | Armed grammar scan counts positional strings `p`, flags `f`, options `o`, exact copied payload, 22 fixed nodes, `p+f+4o` variable nodes, and the nested association's `2*max(4,next_power_of_two(o))-o` spare slots. Overflow fails closed. |
| `Power` | Repaired | Exact rational/integer powers use `sizeof(Expr)+sizeof(Rational)+ceil(num_bits/8)+ceil(den_bits/8)` with exact `k*n+1` bit length for powers of two and a sound `bits*n` bound otherwise. Symbolic calls use recursive input/result proxy; distribution over `Times` additionally accounts for one `Power[_, exponent]` wrapper per factor. The existing integer-work check remains first and unchanged. |
| `Range` | Repaired false positive | The existing armed formula remains `sizeof(Expr) + count*sizeof(Expr)`. The unarmed branch now records zero through the choke point before returning, so the structural falsifier sees the real site without paying for count conversion. |
| `RumTokens` | Retained | Token/trivia/error-span cardinality is known only after lexing. One span per byte rejects long identifiers, strings, and comments that produce very few spans; an incremental lexer-integrated check is needed. |
| `TokenizeWith` | Retained | User patterns decide every boundary. A character-count bound rejects long unmatched text that becomes one output span, while an exact pre-pass is the tokenization itself; an incremental matcher-integrated check is needed. |
| `Union` | Repaired | For valid same-head inputs, `sizeof(Expr) + proxy(common head) + sum(proxy(all concatenated items))`; the unique result is a subset. Group flattening and custom-test kept storage use exact capacities. |
| `UrlParse` | Repaired | `41*sizeof(Expr) + copied component bytes + stored DefaultScheme bytes`; without an explicit scheme it additionally accounts for the synthesized `scheme://input` source coexisting with its parsed component strings. The six-field result has fixed shape, and the check precedes cloning a validated `DefaultScheme`. |

The retained reasons are conclusions, not reclassifications: all eight names
remain in the existing `Preflight` list as required.

## 3. Seven versus ten

The proposed seven projectable handlers were all confirmed. The split among
the seventeen non-`Range` entries is **nine projectable and eight retained**,
not seven and ten: `ParseArgs` and `UrlParse` are the corrections.

- `ParseArgs` is not an arbitrary decoder. Its small option grammar permits an
  armed, allocation-free scan that determines the exact output cardinalities
  and copied string bytes before cloning any argument string. Clean-baseline
  probes also pinned the fixed-shape arithmetic (`ByteCount[ParseArgs[{}]]`
  was 817 bytes).
- `UrlParse` always returns a fixed six-field association. URL components
  partition the input, so input/effective-source bytes plus fixed association
  overhead are a tight bound; `ByteCount[UrlParse["https://h"]]` was 1,306
  bytes versus the 1,321-byte projection.

The other eight really do require parser/lexer/matcher state to avoid an
over-loose length multiplier. `Range` was independently confirmed as the
brief's instrumentation false positive.

## 4. `Range`

I kept the existing active formula and both existing bounds. When no memory
ceiling is armed, `enforce_range_memory_constraint` now returns the result of
`enforce_active_memory_limit_for_projected_bytes(0)`. This lets the structural
falsifier record that the handler reached its projection site. The
`bigint_count_to_usize` call remains below the `Some(limit)` branch, so its
`to_string().parse()` does not run on the ordinary path; neither does any
projection arithmetic or allocation. Source inspection found no other work in
that branch, and the A/B controls in section 7 show no default-path signal.

## 5. Low and realistic ceilings

The stdlib test runs each builtin in its own child process. It constructs the
large argument inside a delegated probe after `MemoryConstrained` is armed,
takes `/proc/self/status`'s `VmHWM` immediately before entering the target
handler, requires the handler to return `MemoryConstraintExceeded`, and limits
post-call HWM growth to 4 MiB. Every low case uses a 1 MiB allowance. `Power`
uses the same isolated-process/HWM pattern directly through evaluation and
expects the `MemoryConstrained` fallback.

| Builtin | Low-ceiling refusal input | Realistic admitted input and allowance |
|---|---|---|
| `Append` | Append to a list containing one 8 MiB string. | One 256 KiB string plus the appended integer, 340,000 bytes. |
| `Commonest` | One-element list containing an 8 MiB string. | One 256 KiB string, 340,000 bytes; additionally 5,000 distinct small integers under the same ceiling. The latter catches the old 327,755-byte over-capacity result: it is now exactly 160,075 bytes. |
| `DeleteDuplicates` | One-element list containing an 8 MiB string. | One 256 KiB string, 340,000 bytes. |
| `DeleteMissing` | One-element list containing an 8 MiB non-missing string. | One 256 KiB non-missing string, 340,000 bytes. |
| `Diff` | Two unlike 2,200-character strings, forcing a 4,844,001-cell alignment. | Two unlike 128-character strings, 380,000 bytes. |
| `ParseArgs` | One short-option group containing 400,000 one-byte flags. | One group containing 4,096 flags, 190,000 bytes. |
| `Power` | `2^100000000`, 1 MiB above live usage, with the integer-work limit raised only enough not to mask the byte check. | `2^1000000`, 250,000 bytes; result is asserted to contain at least 1,000,000 bits. This would reject the generic almost-2x bound and therefore tests the exact power-of-two case. |
| `Range` | `Range[500000]`. | `Range[8192]`, 340,000 bytes. |
| `Union` | Union of a list containing one 8 MiB string. | One 256 KiB string, 340,000 bytes. |
| `UrlParse` | Both an 8 MiB valid `DefaultScheme` with a short schemeless URL, and an explicit URL with an 8 MiB path plus an unused 8 MiB valid default (proving refusal precedes the scheme clone in both branches). | An explicit URL with a 256 KiB path and a schemeless URL with a 128 KiB path plus `DefaultScheme -> "https"`; each gets 340,000 bytes. |

The ordinary collection/URL allowance is only about 1.30 times the 256 KiB
payload, the `Diff` and `ParseArgs` allowances similarly leave a small fixed
margin over their concrete formulas, and `Power` deliberately rejects a 2x
bit-length estimate. Thus these tests catch projections that merely use a
large worst-case multiplier while still proving pre-allocation refusal.

## 6. Final allowlist

The exact remaining contents are:

```rust
pub(in crate::runtime) const KNOWN_UNPROJECTED_PREFLIGHT_BUILTINS: &[&str] = &[
    "FromCsv",
    "FromIni",
    "FromJson",
    "FromToml",
    "FromXml",
    "FromYaml",
    "RumTokens",
    "TokenizeWith",
];
```

The existing sorted/unique test passes, and the armed workspace falsifier is
green with this shortened list.

## 7. Verification

Formatting (the global `--offline` satisfies the no-network hard rule):

```text
$ scripts/run-bounded.py --timeout 120 -- cargo --offline fmt --check
```

No output; exit 0.

Mandatory stdlib suite (full output: 852 lines):

```text
$ scripts/run-bounded.py --timeout 900 -- cargo test --offline -p rum-stdlib
running 833 tests
test result: ok. 830 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 4.25s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Mandatory runtime suite (full output: 980 lines):

```text
$ scripts/run-bounded.py --timeout 900 -- cargo test --offline -p rum-runtime
running 946 tests
test result: ok. 944 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 3.76s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

The first mandatory workspace invocation, without an escape hatch, reached
the native adapter target and failed only because this sandbox denied all nine
loopback binds:

```text
$ scripts/run-bounded.py --timeout 1800 -- cargo test --offline --workspace
test zeromq::tests::native::basic::native_zeromq_backend_round_trips_execute_over_loopback_sockets ... FAILED
test zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket ... FAILED
test zeromq::tests::native::frontend::control::native_zeromq_backend_frontend_style_control_interrupt_round_trip ... FAILED
test zeromq::tests::native::frontend::control::native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages ... FAILED
test zeromq::tests::native::frontend::execute::native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata ... FAILED
test zeromq::tests::native::frontend::session_flow::native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown ... FAILED
test zeromq::tests::native::frontend::stdin::native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply ... FAILED
test zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end ... FAILED
test zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages ... FAILED
native ZeroMQ test cannot bind loopback: Operation not permitted (os error 1); set RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 to explicitly allow this test to skip
test result: FAILED. 97 passed; 9 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
error: test failed, to rerun pass `-p rum-jupyter --lib`
```

Per the brief's explicit contingency, the final post-refinement workspace run
used the skip flag (full output: 2,512 lines). Every test-target result line is
reproduced verbatim:

```text
$ RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 scripts/run-bounded.py --timeout 1800 -- cargo test --offline --workspace
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
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
running 106 tests
test result: ok. 106 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 45 tests
test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
running 946 tests
test result: ok. 944 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 3.75s
running 833 tests
test result: ok. 830 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 4.19s
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
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.94s
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

The harness summary is 2,303 passed, 7 ignored, and 0 failed. Its nine
early-return native ZeroMQ cases are reported here as unobserved, not treated
as evidence that their socket behavior passed.

The clean baseline release was built and copied before any edit. The final
candidate build was:

```text
$ scripts/run-bounded.py --timeout 1200 -- cargo build --offline --release -p rum-cli
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
   Compiling rum-host v0.2.0 (/home/remi/dev/rum/crates/rum-host)
   Compiling rum-terminal v0.2.0 (/home/remi/dev/rum/crates/rum-terminal)
   Compiling rum-repl v0.2.0 (/home/remi/dev/rum/crates/rum-repl)
   Compiling rum-cli v0.2.0 (/home/remi/dev/rum/crates/rum-cli)
    Finished `release` profile [optimized] target(s) in 8.16s
```

Before examining candidate timings, I named
`module-retained-gensym-gc-batch` as the target row because it directly times
`Append`; `appendto-list-incremental-small` and
`appendto-list-incremental-large` as same-tier sequence-allocation controls;
and the already-projected `string-repeat-small` and
`character-range-small` as projection-call controls. The fixed suite has no
timed row for the other repaired builtins; its `Range` calls occur only during
setup or verification, outside timed expressions. The script explicitly sets
all profiling environment switches to zero.

Required back-to-back default window:

```text
$ bash scripts/core-language-microbench.sh --output-dir /tmp/brief420-micro-base-final3 --rum-cli /tmp/brief420-base-rum-cli
core microbench output: /tmp/brief420-micro-base-final3
$ bash scripts/core-language-microbench.sh --output-dir /tmp/brief420-micro-candidate-final3 --rum-cli target/release/rum-cli
core microbench output: /tmp/brief420-micro-candidate-final3
```

| Declared label | Base seconds | Candidate seconds | Delta |
|---|---:|---:|---:|
| `module-retained-gensym-gc-batch` | 0.000066017 | 0.000068077 | +3.12% |
| `appendto-list-incremental-small` | 0.000503844 | 0.000417901 | -17.06% |
| `appendto-list-incremental-large` | 0.001480550 | 0.001583680 | +6.97% |
| `string-repeat-small` | 0.000188791 | 0.000184544 | -2.25% |
| `character-range-small` | 0.000138018 | 0.000132508 | -3.99% |

Whole-process elapsed time was 0.851345 seconds base versus 0.851092 seconds
candidate; 77 of 112 labels moved faster and 35 slower. Because the target is
only a 66-68 microsecond sample inside a broad candidate-fast shift, I ran the
same predeclared controls at 5x scale:

```text
$ bash scripts/core-language-microbench.sh --scale 5 --output-dir /tmp/brief420-micro-base-scale5-final3 --rum-cli /tmp/brief420-base-rum-cli
core microbench output: /tmp/brief420-micro-base-scale5-final3
$ bash scripts/core-language-microbench.sh --scale 5 --output-dir /tmp/brief420-micro-candidate-scale5-final3 --rum-cli target/release/rum-cli
core microbench output: /tmp/brief420-micro-candidate-scale5-final3
```

| Declared label, 5x | Base seconds | Candidate seconds | Delta |
|---|---:|---:|---:|
| `module-retained-gensym-gc-batch` | 0.000310642 | 0.000311734 | +0.35% |
| `appendto-list-incremental-small` | 0.009595940 | 0.009419980 | -1.83% |
| `appendto-list-incremental-large` | 0.027534900 | 0.027921000 | +1.40% |
| `string-repeat-small` | 0.000921665 | 0.000924265 | +0.28% |
| `character-range-small` | 0.000650762 | 0.000660401 | +1.48% |

Whole-process elapsed time was 4.816366 seconds base versus 4.815501 seconds
candidate; 50 of 112 labels moved faster and 62 slower. The target is 0.35%
slower while the four controls range from 1.83% faster to 1.48% slower, and the
whole run differs by 0.018%; there is no measurable default-path regression.

Final source checks:

```text
$ git diff --check
```

No output; exit 0. The changed-path audit contains only the implementation and
test files listed in section 1 plus this appended report. No new `unsafe` Rust
appears in the implementation.

## 8. Stop-loss / open questions

No implementation stop-loss fired. In particular, there was no
reclassification, falsifier/allowance change, generated-element or
integer-work weakening, resource-default/stride edit, `unsafe`, network use,
test weakening, or git mutation. All cargo invocations were offline.

The sandbox contradicted the host note in the brief by returning `EPERM` for
every loopback bind. These nine native ZeroMQ tests are therefore **unobserved**
despite the otherwise-green workspace rerun:

- `native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket`
- `native_zeromq_backend_round_trips_execute_over_loopback_sockets`
- `native_zeromq_backend_frontend_style_control_interrupt_round_trip`
- `native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages`
- `native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata`
- `native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown`
- `native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply`
- `native_zeromq_backend_rejects_tampered_signed_requests_end_to_end`
- `native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages`

The remaining open design work is the eight retained parser/tokenizer cases.
They need reservation/checkpoint logic integrated with their real parse or
match state; a pre-parse input-length multiplier cannot satisfy the brief's
realistic-ceiling requirement. Per the hard rule, I recorded that conclusion
without moving any of them out of `Preflight`.
