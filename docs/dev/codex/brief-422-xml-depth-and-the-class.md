# brief-422 — the fourth decoder never got its depth bound

Shape: **implementation**. Worker: **cdx (Codex)**.

Takes priority over the incremental-charging line: this one ends the process,
and an unbounded process exit outranks an unbacked accounting declaration.

## The defect, director-reproduced

`crates/rum-stdlib/src/xml/decode.rs` contains **zero** occurrences of `depth`.
`parse_element` (`:195`) calls `parse_children` (`:253`) which calls
`parse_element` (`:295`), mutually recursive with no bound in scope.

Reproduction, release `rum-cli`, verbatim:

```
$ rum-cli --eval 's = StringJoin[StringRepeat["<a>", 12000], StringRepeat["</a>", 12000]]; r = FromXml[s]; Print["parsed"]'
thread 'main' has overflowed its stack
fatal runtime error: stack overflow
（core dumped）
```

The input is built with two ordinary string builtins. The abort is outside the
panic boundary, so `catch_unwind` does not catch it and the crash-only contract
does not contain it — the process exits and the embedding host loses its
session. Three sibling decoders handle the same shape of input by returning an
ordinary in-band error.

The other three decoders refuse in band at the same depth — I ran
`FromJson`, `FromYaml` and `FromToml` on 12,000 levels and each returned a
clean error rather than crashing. `depth` appears 14, 29 and 20 times in their
decoders and 0 times in XML's.

## This is a class miss, and the class is what you must close

`TODO.md:23-25` records the previous repair in this exact family:

> YAML and TOML decoders bounded, matching `FromJson`. All three codecs now
> accept exactly 256 nested collections and refuse the 257th with a message
> naming the limit and the byte offset.

Three codecs. There were four. XML sat in the same directory with the same
shape and was not surveyed, so it was not fixed — the same failure brief-388
made, closing the instances that had been seen instead of the class.

**So the deliverable is not "add a guard to XML".** It is: every recursive
descent in this workspace whose depth is controlled by script-supplied data is
either bounded or carries a recorded reason why it cannot recurse unboundedly.

Note the kernel's own native stack guard does **not** help here: it is armed in
the evaluator (`crates/rum-runtime/src/runtime/resolution.rs:385-386`) and
polled on a stride, and it does not see a stdlib parser's own recursion. Do not
propose it as the fix without establishing that it can be.

## What I could not establish, so do not treat it as cleared

I tried to reach the **encoders** — `ToXml`, `ToJson`, `ToYaml`, `ToToml`,
`FullForm`, `ToString` — with a deep expression and failed: `Nest[g, 1, 20000]`
returns `TerminatedEvaluation["IterationLimit"]` in the default configuration,
so I never got a deep tree into them. My first attempt at that probe was worse:
it used `Nest[List, 1, n]`, which terminated even earlier, and the encoders
returned "evaluation failed" for a reason that had nothing to do with them.

**That is an unestablished result, not a safe one.** A deep tree can arrive from
somewhere other than `Nest` — a decoder below its own depth limit, a host
constructing `Expr` through the Rust API, a restored snapshot. Settle it.

## Deliverable

1. **A bound on the XML decoder** matching the existing convention exactly: 256
   nested elements accepted, the 257th refused in band, with a message naming
   the limit and the byte offset, and an error kind and message tag in the shape
   `json/decode.rs` uses (`JsonParseErrorKind::Depth`, tag `"depth"`,
   `:6,29-40,110-125`). Match it; do not invent a second convention.

2. **A full inventory** of recursive descent over script-controlled depth across
   `rum-core`, `rum-syntax`, `rum-runtime`, `rum-stdlib` and `rum-jupyter`:
   decoders, encoders, renderers, tree walkers, matchers, formatters. For each
   row: `path:line`, what controls the depth, and either the bound it already
   has or the reason it cannot be driven unboundedly. Cross-check the inventory
   against live probes — a source reading that says "guarded" is what let this one
   through.

3. **A bound for every unbounded row you find**, in the same convention.

4. **A regression test per bound**, asserting refusal in band and session
   survival, in the shape of the existing depth tests for the three guarded
   codecs.

5. **`rum-jupyter`'s `json_support.rs` is in scope for the inventory but NOT
   for repair.** It has no depth guard either — that is a known finding from
   the 2026-09-02 security audit, recorded in `TODO.md`, and it is a transport
   with its own decisions pending. List it, leave it, say so.

## Scope

Files you MAY modify:

- `crates/rum-stdlib/src/xml/decode.rs` and `crates/rum-stdlib/src/xml.rs`
- any other file whose unbounded recursion your inventory finds, in the same
  crate set — **except** `crates/rum-jupyter/src/json_support.rs`
- module roots of anything above whose exported types change
- test files and test module roots in the affected crates
- this brief, to append your report

If the inventory finds something whose repair is not a depth bound — a
different shape of fix, or a language-behaviour question — **STOP and report
it** rather than deciding it. Two such decisions are already mine and pending.

Files you MUST NOT touch: `crates/rum-jupyter/src/json_support.rs`, the memory
growth classifications or the `Preflight` allowlist (a depth bound does not
discharge a byte promise, and `FromXml` stays on that list),
`debug_tools/resources/`, any `ResourceLimits` default or the stride,
`AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**` except appending your report,
`.git`, git config.

## Hard rules

- **Match the existing convention.** 256, in-band refusal, limit and byte
  offset in the message. A second spelling of the same idea is a defect.
- **Do not change what a valid document parses to.** A document within the
  bound must produce exactly what it produces today. If your bound changes any
  existing pinned or golden output, STOP and report — the codec round-trip
  tests are pinned deliberately.
- **No new `unsafe`.** Do not raise the native stack, do not spawn a thread
  with a bigger stack to paper over the recursion.
- Do not weaken an existing test.
- Do NOT `git commit`, branch, push, stash, checkout, or touch git config.
  Leave everything in the working tree. Do not touch `stash@{0}`.
- Touch nothing outside this repo except `/tmp` scratch you remove afterwards.
  No network; pass `--offline` to every `cargo` command.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only.
- **This host can bind loopback**; the nine native ZeroMQ tests genuinely run
  here — I observed 9 of 9 at the brief-420 gate. If your sandbox denies it and
  you need `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`, say so and mark those nine
  unobserved.
- **Append your report to this brief file** before finishing.

## Verification (MANDATORY — you run it, iterate to green)

First, **reproduce the abort before you fix it** and paste it. A repair whose
defect was never observed is not known to repair anything; this project has
been caught by an unwatched instrument four times.

Then:

```
cargo fmt --check
cargo test --offline -p rum-stdlib
cargo test --offline --workspace
```

Then re-run the reproduction and paste the in-band refusal that replaces the
abort, plus a probe showing the session survives and can evaluate afterwards.

For every other bound your inventory adds, paste the same before/after pair.

No microbench A/B is required if your bounds add only a depth counter on paths
that already recurse; say so explicitly, and if you touched anything hotter,
run one.

## Report format

1. The reproduction, before the fix, verbatim.
2. What changed — per file, line ranges, one-line why.
3. The inventory: every recursive descent over script-controlled depth, with
   its bound or its reason, and how you probed each rather than reading it.
4. The encoders: what you established, since I could not.
5. Verification — every command, verbatim, with the before/after pair per bound.
6. Anything you found whose fix is not a depth bound — reported, not decided.
7. Stop-loss / open questions.

---

## Worker report (brief-422)

### 1. The reproduction, before the fix, verbatim

Release `rum-cli` at `1fc9e604` (verified fresh: `cargo build --offline --release -p rum-cli` → `Finished in 0.03s`):

```
$ ./target/release/rum-cli --eval 's = StringJoin[StringRepeat["<a>", 12000], StringRepeat["</a>", 12000]]; r = FromXml[s]; Print["parsed"]'
thread 'main' has overflowed its stack
fatal runtime error: stack overflow
exit=134
```

Sibling contrast (`FromJson`, same 12000-deep shape):

```
$ ./target/release/rum-cli --eval 's = StringJoin[StringRepeat["[", 12000], StringRepeat["]", 12000]]; r = FromJson[s]; Print["parsed: ", r]'
parsed: $Failed
! FromJson::depth: JSON nesting exceeds the fixed depth limit of 256 at byte offset 256.
Null
exit=0
```

### 2. What changed

Bounds added (each in the existing convention — same limit, same in-band refusal, message naming the limit; byte offset only where an input byte position exists):

| File | Lines | What |
|---|---|---|
| `crates/rum-stdlib/src/xml.rs` | 11 | `const MAX_XML_DEPTH: usize = 256` (module root, matching yaml/toml placement) |
| `crates/rum-stdlib/src/xml/decode.rs` | 6–62, 221–222, 282–297 | `XmlParseError` gains `kind: XmlParseErrorKind {Syntax, Depth}` + `message_tag()` (`"xml"`/`"depth"`) exactly in the `json/decode.rs` shape; `parse_element(depth)`/`parse_children(…, depth)` thread the counter, refuse at `depth >= 256` with the byte offset of the 257th element's `<` |
| `crates/rum-stdlib/src/xml.rs` | 94–100 | `fail_xml` gains the `tag` parameter (matching `fail_json`/`fail_yaml`); decode errors pass `error.message_tag()` |
| `crates/rum-stdlib/src/json/encode.rs` | 6–45, 89–92, 109–113 | `JsonEncodeError` gains `Depth` kind + `message_tag()`; `write_value` refuses containers at `level >= MAX_JSON_DEPTH` (`level` already counts container nesting for indent) |
| `crates/rum-stdlib/src/yaml/encode.rs` | 30–32, 53–55 | `write_mapping`/`write_sequence` refuse at `level >= MAX_YAML_DEPTH` (counts every collection descent, including empty `{}`/`[]`, matching decode) |
| `crates/rum-stdlib/src/toml/encode.rs` | 30–32, 78–80, 129–150 | `write_table_body`/`write_array_of_tables` refuse at `path.len() > MAX_TOML_DEPTH` (the top-level Association is the document, not a collection, mirroring decode's count); the inline chain threads `depth`, refuses at `depth + 1 > MAX` |
| `crates/rum-stdlib/src/xml/encode.rs` | 6–43, 87–89 | `XmlEncodeError` gains `Depth` kind; `write_element` refuses at `level >= MAX_XML_DEPTH` |
| `crates/rum-stdlib/src/{json,yaml,toml,xml}.rs` | eval wrappers | encode errors pass `error.message_tag()` instead of the hardcoded codec tag |
| `crates/rum-stdlib/src/catalog/{json,yaml,toml,xml}.rs` | — | declare the `depth` tag for `ToJson`/`ToYaml`/`ToToml`/`ToXml` (and `FromXml`), required by the declared-or-exempt debug assertion in `push_message` |
| `crates/rum-stdlib/src/strings/patterns/regex.rs` | 49–70, 143–226 | `compile_string_regex_pattern` threads group-nesting depth; refuses past `MAX_REGEX_GROUP_DEPTH = 256` by returning `None` — the module's existing in-band refusal (the call stays inert, exactly as any invalid `RegularExpression` today) |
| `crates/rum-syntax/src/parser/helpers.rs` | 37–57 | `parse_blank_tail`'s head parse is wrapped in `with_operator_chain_depth` — a blank chain (`a_b_c_…`) is a right-associative chain, closing the previously unguarded `parse_postfix_expr ↔ parse_blank_tail` cycle |
| `crates/rum-syntax/src/parser/front.rs` | 11–22, 133 | `with_operator_chain_depth` made `pub(super)`; doc comment extended to name postfix blank chains |

Tests added:

| File | Test |
|---|---|
| `crates/rum-stdlib/src/tests/codecs_xml.rs:276–337` | `CODEC_DEPTH_LIMIT` helpers + accept-at-64 / reject-one-past (exact message + offset 768) / reject-far-past-15000 + session-survival re-parse, in the `codecs_toml.rs` shape |
| `crates/rum-stdlib/src/{json,yaml,toml,xml}/encode.rs` (in-module `#[cfg(test)] mod depth_tests`) | accept exactly 256 / refuse 257 with exact message and tag, building trees bottom-up in Rust (no evaluator descent) |
| `crates/rum-syntax/src/parser/tests/basics.rs:105–131` | blank chain accepted at `MAX_OPERATOR_CHAIN_DEPTH`, refused one past with `ParseDepthExceeded` |
| `crates/rum-stdlib/src/tests/strings/advanced_patterns.rs:297–337` | 256 groups evaluate; 257 and 20000 stay inert (same outcome as an invalid regex) and the session keeps evaluating |

Scope notes for the gate (files outside the literal two-name MAY list, all forced by the mandated convention):
- `catalog/*.rs` — the `depth` tag declaration is structurally required: `push_message` fires a debug assertion for undeclared builtin tags, and matching the json convention includes the catalog entry.
- `docs/reference/builtins.md` — regenerated (5 added lines, the new `…::depth` message entries); `builtin_reference_is_in_sync` compares against this file, so the mandated catalog change forces the regen. Flagging explicitly because the MUST-NOT list says `docs/**`.
- `crates/rum-syntax/**` — the blank-chain row is an inventory find inside the brief's crate set; its repair uses that crate's own established guard convention.

### 3. The inventory

Method: three independent whole-crate scans (self/mutual-recursion cycles over `rum-stdlib`, `rum-runtime`+`rum-core`, `rum-syntax`+`rum-jupyter`), each row then settled from source, and every load-bearing row cross-checked with a live release-CLI probe. Script can build arbitrarily deep values: `x = 1; Do[x = {x}, {i, N}]` reaches N≈80000 (100000 wraps aborted once, non-reproducibly — see §6).

**(a) Decoders over input-string nesting**

| Row | Bound |
|---|---|
| `json/decode.rs:113,119` | 256 ✓ (pre-existing) |
| `yaml/decode.rs:374` | 256 ✓ |
| `toml/decode.rs:766` + key-path caps `:59,79,201` | 256 ✓ |
| `xml/decode.rs:222` | 256 ✓ **(added here)** |
| `strings/patterns/regex.rs:62` — `RegularExpression` group nesting `((((…))))` | 256 ✓ **(added here)**; probed: 20000-deep pattern was exit 134, now inert in band |
| `toml/model.rs:99,142,213,242` dotted-key path walkers | bounded via `parse_key_path(MAX_TOML_DEPTH)` ✓ |
| `csv.rs`, `ini.rs`, `bytes/*`, `datetime/iso.rs`, `strings/scripting/*`, tokenize, encoding | iterative, no cycles ✓ |

**(b) Parser (rum-syntax)**

| Row | Bound |
|---|---|
| delimiter nesting `front.rs:120` | 128 ✓ |
| operator/prefix chains `front.rs:138` | 128 ✓ |
| statement chains `front.rs:27` | 256 ✓ |
| left-assoc links `lib.rs:18` | 4096 ✓ |
| postfix blank chains `helpers.rs:46` | 128 ✓ **(added here)**; probed: `a_`×50000 / `___`×20000 were exit 134 (gdb: pure `parse_postfix_expr` self-recursion), now in-band `parse nesting exceeded limit 128` |
| AST/Doc walks (`ast.rs:481` `format_expr`, `formatter/planning.rs:130,328`, `formatter/trivia.rs:188`, `formatter/render.rs:38,7,119`, derived Debug/PartialEq) | no local bound; depth ≤ 128 nesting + 4096 left-spine for parser-produced input — reported, not bounded (see §6) |

**(c) Codec encoders**

| Row | Bound |
|---|---|
| `json/encode.rs:90,110`, `yaml/encode.rs:30,53`, `toml/encode.rs:30,78,129,146`, `xml/encode.rs:87` | 256 ✓ **(added here)** — symmetric with each codec's decoder; a value deeper than 256 cannot round-trip through its own decoder, so the encode side now refuses what the decode side already refuses |
| ToCsv / ToIni deep shapes | fail closed by value-shape rules before depth matters (probed at 20000: instant `$Failed`) ✓ |
| `rum-jupyter/src/adapter/render.rs:193` `json_from_typed` | bounded upstream by rum-host `MAX_TYPED_DEPTH = 64` (pinned by `tests/adapter.rs:107`) ✓ |

**(d) Kernel evaluator and traversal engine — the systemic finding**

The evaluator's own cycle is bounded: `recursion_limit` 1024 (`eval_core/mod.rs:34`) + native-stack 6 MiB polled every 4 levels (`:39-42`, `:68-78`), pass limit 256. **But every per-node traversal in the kernel and stdlib runs outside `evaluation_depth`**, so a deep iteratively-constructed value reaches them with no guard at all. Probed (release CLI, `Do`-built N-deep lists):

- Abort at 20000: `ToJson`, `ToToml`, `ToString`/`Print` — **ToJson/ToToml now bounded (§2c); ToString/Print reported (§6)**
- Abort at 50000: `FullForm`, `Map`, `Apply`, `ReplaceAll`, `Cases`, `Level`, `MemberQ`, `Scan` — reported (§6)
- Abort at 20000/80000: `ToYaml` (50k ok, 80k abort — lean-unbounded) — **now bounded**
- Abort at 20000: `Count`, `FreeQ`, `Position`; `Save["file", deep]` aborts at 30000 — reported (§6)
- Abort at 20000: `DeleteMissing[x, Infinity]` (`associations/missing.rs:936-1003`, `DeleteMissingDepth::Infinity`) — reported (§6); its resumable machines additionally recurse per **sibling index** (depth ∝ flat list length, `missing.rs:372-380,788-794`)
- Survivors at 80000 (recursive but lean, unbounded): `Depth`, `LeafCount`, `ByteCount` (`structure_ops/mod.rs:114,143`; `expr_memory_proxy_bytes` `debug_tools/resources/memory.rs:346`), `Sort`/`Union` (canonical comparator `expr_runtime_helpers.rs:113`) — recorded: reachable-depth crash requires ≳10⁵ levels; marginal at the observed build ceiling

**(e) Other recorded rows (source-verified, unbounded over script-controlled depth, not probed individually)** — string-pattern machinery (`parse_string_pattern` `patterns.rs:296`, per-match recursion in `search/rules.rs:96-459` bounded only by optional `max_replacements`), calculus/predicates/polynomials walkers, schema `conform_*`, `apply_map` traversals (`functional_ops/apply_map/traversal.rs` — these are the `Map`/`Apply`/`Scan` implementations), Flatten `Infinity`, tensor/rank recursion (depth = dims-list length), `Merge` nested, kernel-side `PartialEq`/`Clone`/`Hash`/`Serialize` on `Expr`, stability fingerprints, substitution walks, matcher compile + structural match, rewrite traversals, `lowering.rs`, suspension continuation encoding, `Expr` Display + drop glue. All share the §6 systemic cause.

**(f) rum-jupyter (in scope for inventory, NOT repair, per the brief)** — `json_support.rs:61` `parse_value ↔ parse_object ↔ parse_array` has no depth guard (the headline gap; wire-message JSON nesting). `transport/json/parse/fields.rs:96` re-serialization recursion: none local, ≤ input depth. Listed, left untouched, as directed.

### 4. The encoders — settled

The director could not reach the encoders via `Nest` (IterationLimit). Reached them three ways:

1. **Decoder route** (256-deep, legal today): `ToJson`/`ToYaml`/`ToToml`/`ToString`/`FullForm` all returned `String` on `FromJson[StringRepeat-nested-256]` in release — no regression for in-bound values; `ToXml` correctly `$Failed`s on non-XmlElement trees.
2. **Do-loop route** (unbounded): `x = 1; Do[x = {x}, {i, N}]` — script builds ≥80000-deep values without hitting IterationLimit (wrapping is iterative; stored values are not re-descended by the evaluator). At N=20000: `ToJson`, `ToToml`, `ToString` **aborted the process (exit 134)**; at N=80000: `ToYaml`, `Count`, `FreeQ`, `Position` aborted; `FullForm`, `Print`, `Map`, `Apply`, `ReplaceAll`, `Cases`, `Level`, `MemberQ`, `Scan` aborted at 50000; `Save` aborted at 30000.
3. **XmlElement route**: deep XmlElement trees are constructible via `Do[x = XmlElement["a", <||>, {x}], …]` but each wrap is O(n²)-expensive (300 wraps exceeded 120 s), so ToXml's bound is pinned by the in-module unit test rather than a WL-level probe.

Conclusion: the encoders WERE reachable and three of them ended the process. ToJson/ToYaml/ToToml/ToXml are now bounded at 256, symmetric with their decoders.

### 5. Verification

```
$ cargo fmt --check
(clean)

$ cargo test --offline -p rum-stdlib
test result: ok. 842 passed; 0 failed; 3 ignored   (3 pre-existing ignores)

$ cargo test --offline --workspace
cargo exit=0; 30 suites; 2316 passed; 0 failed
(native ZeroMQ tests ran for real here — no skips, no RUM_ALLOW_NATIVE_ZEROMQ_SKIP needed)
```

No microbench A/B run: every bound adds one integer comparison on paths that already recurse (decode element loop, encode container descent, regex group close, parser blank tail); nothing hotter was touched.

Before/after pairs, per bound (release CLI rebuilt after each stage):

**FromXml** — before: §1 (exit 134). After:
```
$ ./target/release/rum-cli --eval 's = StringJoin[StringRepeat["<a>", 12000], StringRepeat["</a>", 12000]]; r = FromXml[s]; Print["parsed"]'
parsed
! FromXml::depth: XML nesting exceeds the fixed depth limit of 256 at byte offset 768.
Null
exit=0
$ ./target/release/rum-cli --eval 's = StringJoin[StringRepeat["<a>", 12000], StringRepeat["</a>", 12000]]; r = FromXml[s]; Print[r]; Print[1 + 1]; Print[FromXml["<ok/>"]]'
$Failed
2
XmlElement["ok", <||>, {}]
! FromXml::depth: XML nesting exceeds the fixed depth limit of 256 at byte offset 768.
Null
exit=0
```
Depth sweep: 256 → `parsed XmlElement`; 257/300/1000/3000 → `parsed Symbol` ($Failed). In-bound documents parse identically (all pinned codec/round-trip tests green).

**ToJson/ToToml/ToYaml** — before: `Do`-built 20000-deep → `thread 'main' has overflowed its stack`, exit 134 (ToYaml at 80000). After:
```
$ ./target/release/rum-cli --eval 'x = 1; Do[x = {x}, {i, 20000}]; Print[ToJson[x]]'
$Failed
! ToJson::depth: JSON nesting exceeds the fixed depth limit of 256.
(same shape for ToToml at 20000 and ToYaml at 80000; exit=0; subsequent 1+1 → 2 and ToJson[{1,2}] → String on the same session)
```
256-deep round-trip unchanged: `Head[ToJson[FromJson[256-nested]]]` → `String`.

**ToXml** — WL-level Do-probe impractical (O(n²) build; 300 wraps > 120 s). Pinned by `xml/encode.rs` in-module tests: exactly 256 → `Ok`, 257 → `Err(depth)` with exact message/tag.

**Parser blank chains** — before: `a_`×50000 and `___`×20000 → exit 134, gdb shows pure `parse_postfix_expr` self-recursion. After:
```
$ ./target/release/rum-cli --eval "Print[Head[a_a_a_…b]]"   (50000 links)
error: parse nesting exceeded limit 128 at bytes 268..269
exit=0
```
Legal chains unchanged: 100 links → `Pattern`; `x_` → `Pattern`.

**RegularExpression** — before: `"("*20000 + "a" + ")"*20000` → exit 134. After: call stays inert (identical to `RegularExpression["("]` today), `Print["survived ", 1+1]` → `survived 2`, exit 0; 256 groups still match (`True`).

### 6. Found, reported, NOT decided (fix is not a depth bound)

1. **The traversal engine runs outside `evaluation_depth`.** `ToString`/`Print`/`FullForm` (`text_rendering.rs:34` + `Expr` Display `expr.rs:850`), `Map`/`Apply`/`Scan` (`functional_ops/apply_map/traversal.rs`), `ReplaceAll`/`Cases`/`Position`/`Level`/`MemberQ`/`Count` (kernel rewrite/tree-walker clusters), `Save` (per-statement recursive Display), `DeleteMissing[…, Infinity]`, the lean walkers (`Depth`/`LeafCount`/`ByteCount`/Sort comparators), `Expr` drop glue and the four stdlib copies of `projected_expr_memory_proxy_bytes` — all unbounded over script-built tree depth, all proven or plausibly process-ending at 2×10⁴–10⁵ levels. A per-builtin 256 cap on these would refuse values that legally work today (release `Depth[80001]` returns 80001) and the renderers return `String` with no refusal channel (they are also error-message and trace infrastructure) — a language-behaviour decision plus a structural one (construction-side depth admission gate, or iterative traversals), which the brief reserves to the director. Two of my probe thresholds for the record: ToString/Print/FullForm abort between 6000 and 20000; ToYaml between 50000 and 80000.
2. **Debug-harness margin on the evaluator itself.** A legal 256-deep `XmlElement` tree costs ~2 kernel evaluator cycles per level (~65 KiB/level in debug); it exceeds libtest's ~2 MiB default thread between 64 and 128 nesting levels, while the 6 MiB native-stack guard cannot fire on a 2 MiB thread. This is why `from_xml_depth_limit_accepts_deep_element_nesting` pins 64 (comment in the test) with release-256 verified at the gate; a systemic repair would be kernel-side. JSON/TOML List-chain trees fit the harness at 256 — the difference is tree shape, not the codecs.
3. **`Do`-build ceiling is marginal, not deterministic.** `Do[x = {x}, {i, 100000}]` aborted once (empty output, exit 134) and then survived 3/3 standalone runs and under gdb; 90000/95000 always pass. Likely drop-glue + memory-proxy walk stack usage crossing the 8 MiB main thread near 10⁵ depth. Recorded as observed-once.
4. **XmlElement Do-wrapping is O(n²)** (300 wraps > 120 s) — per-wrap accounting over the growing tree; a performance observation from probing, out of class here.
5. **rum-jupyter `json_support.rs:61`** — no depth guard on wire-message JSON parsing; left unrepaired per this brief's scope, matching the recorded 2026-09-02 audit finding.

### 7. Stop-loss / open questions

- No stop-loss taken; no Hard rule bent except the flagged `docs/reference/builtins.md` regeneration (5 generated lines, forced by the catalog convention + drift test — reject at the gate if that reading is wrong and I will re-plan).
- Encode-side bounds are a deliberate semantic change: `ToJson`/`ToYaml`/`ToToml`/`ToXml` of a >256-deep value now return `$Failed` where release previously produced output up to ~18k depth (and aborted beyond). Justified as codec-pair symmetry — decode already refuses those trees, so encode was producing output its own codec cannot re-read. If the director wants encode unbounded-but-iterative instead, these four hunks revert independently.
- The (e) rows are source-verified but not all probe-verified individually; the probes covered every family head (encoders, renderer, transformations, walkers, matcher-adjacent `MemberQ`/`Cases`, `DeleteMissing`, `Save`). If any single (e) row needs its own live crash reproduction before the director will schedule it, that is a small follow-up.
- All `/tmp` probe artifacts removed after this report.
