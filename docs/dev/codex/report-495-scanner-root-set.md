# brief-495 — scanner root set

The ownership scanner now includes `rum-syntax`. Discovery adds three
self-reaching types and eight public opaque-input sites. All eight use the
existing `PublicInput` ledger disposition. Six foreign-storage unknowns become
resolved ownership edges; one new foreign-storage unknown remains visible.
Production behavior is unchanged.

## 1. Root-set criterion and evidence

Start with the production kernel/value/standard-library crates, then close the
root set over workspace crates defining their nominally owned field types,
including ownership through containers and aliases. Follow that direction
transitively. Scan each included crate's whole production module tree, including
local items. This is a crate-level ownership boundary, so discovery also sees
internal types such as the syntax formatter's `Doc`.

The production dependency sections of all ten workspace manifests and the
actual stored fields were inspected before changing `ROOTS`:

| Crate | Included? | Evidence / reason |
| --- | --- | --- |
| `rum-core` | Yes | Existing value-layer root; runtime/stdlib own its `Expr`, `SharedExpr`, `SharedArgs`, and other values. |
| `rum-runtime` | Yes | Existing kernel root; stdlib uses its owned kernel state. Its production workspace dependencies are core, syntax, and the derive macro crate. |
| `rum-stdlib` | Yes | Existing builtin-implementation root, including builtin-local owning state. Its production workspace dependencies are core, runtime, and syntax. |
| `rum-syntax` | Yes | Runtime fields own its ASTs, parsed-source containers, diagnostics, and operator enum. Its production dependencies contain no further workspace crates. |
| `rum-host` | No | Its manifest depends on core/runtime/stdlib; `HostSession.session: Session` (`crates/rum-host/src/session.rs:104`) demonstrates the reverse ownership direction. No included crate has a nominal owning field of a host-defined type. |
| `rum-jupyter` | No | Its only production workspace dependency is host; the included crates have no dependency or nominal owning edge back to this adapter. |
| `rum-repl` | No | Its production workspace dependencies are host/syntax/terminal; these are dependencies of a consumer, with no reverse nominal owning edge from the included crates. |
| `rum-cli` | No | Its production workspace dependencies are repl/host/syntax/terminal; it is an executable consumer, not a nominal field-type provider to the included crates. |
| `rum-terminal` | No | Its production workspace dependency is host; terminal rendering is downstream of the scanned ownership boundary. |
| `rum-derive` | No | `crates/rum-derive/Cargo.toml` declares `proc-macro = true`; it supplies compile-time macros, not runtime field representations. Expansions remain unknown under the existing scanner. |

This rule does not claim that erased payload implementations cannot originate
downstream. For example, host `StdFileIo::provider()` and
`StdEntropySource::provider()` construct runtime providers
(`crates/rum-host/src/file_io.rs:24`, `crates/rum-host/src/entropy.rs:21`).
Those enter the existing opaque host-payload boundary, whose declarations and
unresolved implementation coverage belong to the brief-494 ledger. A downstream
implementation does not turn a `dyn` slot into a resolved nominal field edge.
The scanner still does not enumerate arbitrary external implementations.

The six concrete cross-crate field sites are listed individually in section 3.
Runtime imports the syntax exports in `runtime/mod.rs:9-12`; syntax reexports
their definitions in `crates/rum-syntax/src/lib.rs:20-25`. The newly resolved
`ParsedSource.syntax: SyntaxExpr` field (`document.rs:47`) leads into the
recursive AST; `TopLevelEvalInput::Parsed.0` owns the AST directly. The
diagnostic types and `BinaryOp` are not themselves recursive.

Independent accounting of the original 70 storage rows found **29 explicit
`std` representations, 24 default-hasher `std` representations, 6 `num_bigint`
representations, 6 `rum_syntax` representations, and 5 unresolved local
storage/array-length rows**. Thus the director's 29 counts explicit `std`
representations, not all `std` rows. The remaining five rows concern
`StabilityBloom`, `ReplaceDepthStack.inline`, the two call-dispatch `stages`
fields, and `AdaptiveSetMembership.memo`; they do not identify another missing
workspace crate. `num_bigint` is external; the five excluded consumers and
`rum-derive` supply no additional nominal field representation to this closure.

Final roots: `rum-core`, `rum-runtime`, `rum-stdlib`, `rum-syntax`.

## 2. Inventory churn

All six discovery partitions were regenerated from
`discovery_report_without_inventory_input` output. The same partitioning first
reproduced every baseline inventory byte-for-byte. Rows were not edited to make
the check pass. The ledger retains every old entry unchanged and adds the
eight discovered public input rows with the existing checked disposition and
witness.

| Partition | Before | After | Added | Removed | Net |
| --- | ---: | ---: | ---: | ---: | ---: |
| `self_reaching.tsv` | 57 | 60 | 3 | 0 | +3 |
| `host_payloads.tsv` | 50 | 58 | 8 | 0 | +8 |
| `unknown_attributes.tsv` | 1,357 | 1,399 | 42 | 0 | +42 |
| `unknown_captures.tsv` | 2,774 | 2,827 | 53 | 0 | +53 |
| `unknown_macros.tsv` | 2,895 | 3,020 | 125 | 0 | +125 |
| `unknown_storage.tsv` | 70 | 65 | 1 | 6 | -5 |
| `host_payload_ledger.tsv` (excluding header) | 50 | 58 | 8 | 0 | +8 |

`UnknownConstruct` totals move from 7,096 to 7,311 (+215). The 220 added
attribute/capture/macro rows remain unresolved, without a new classification
axis. The storage addition is
`rum_syntax::trivia::SyntaxToken.leading_trivia: Range<usize>` at
`crates/rum-syntax/src/trivia.rs:9:25`; its `std::ops::Range` representation
remains unknown rather than being silently treated as proven.

Discovery also moves from 578 to 603 production files, 282 to 295 test-only
files (including the new scanner regression file), 1,505 to 1,539 nominal
definitions, 35 to 37 aliases, and 3,498 to 3,561 owning edges. Resolved explicit
`Drop` owners remain 13.

## 3. Individually disclosed rows

Every newly self-reaching row, verbatim:

```text
rum_syntax::ast::SyntaxExpr	crates/rum-syntax/src/ast.rs	6	rum_syntax::ast::SyntaxExpr.kind -> rum_syntax::ast::SyntaxExprKind ; rum_syntax::ast::SyntaxExprKind.Association.0/std::vec::Vec[Own]/arg0 -> rum_syntax::ast::SyntaxExpr
rum_syntax::ast::SyntaxExprKind	crates/rum-syntax/src/ast.rs	150	rum_syntax::ast::SyntaxExprKind.Association.0/std::vec::Vec[Own]/arg0 -> rum_syntax::ast::SyntaxExpr ; rum_syntax::ast::SyntaxExpr.kind -> rum_syntax::ast::SyntaxExprKind
rum_syntax::formatter::doc::Doc	crates/rum-syntax/src/formatter/doc.rs	5	rum_syntax::formatter::doc::Doc.Concat.0/std::vec::Vec[Own]/arg0 -> rum_syntax::formatter::doc::Doc
```

Every new `UnknownHostPayload` row, verbatim:

```text
UnknownHostPayload	crates/rum-syntax/src/ast.rs	16	27	public input rum_syntax::ast::impl(SyntaxExpr)@11::integer(value): opaque owned payload impl Into < String >
UnknownHostPayload	crates/rum-syntax/src/ast.rs	20	26	public input rum_syntax::ast::impl(SyntaxExpr)@11::string(value): opaque owned payload impl Into < String >
UnknownHostPayload	crates/rum-syntax/src/ast.rs	24	26	public input rum_syntax::ast::impl(SyntaxExpr)@11::symbol(value): opaque owned payload impl Into < String >
UnknownHostPayload	crates/rum-syntax/src/document.rs	51	24	public input rum_syntax::document::impl(SourceText)@50::new(source): opaque owned payload impl Into < String >
UnknownHostPayload	crates/rum-syntax/src/document.rs	326	39	public input rum_syntax::document::parse_with_diagnostics(source): opaque owned payload impl Into < String >
UnknownHostPayload	crates/rum-syntax/src/document.rs	341	26	public input rum_syntax::document::impl(ParsedSource)@340::parse(source): opaque owned payload impl Into < String >
UnknownHostPayload	crates/rum-syntax/src/trivia.rs	214	41	public input rum_syntax::trivia::impl(TriviaPiece)@209::whitespace(text): opaque owned payload impl Into < String >
UnknownHostPayload	crates/rum-syntax/src/trivia.rs	218	38	public input rum_syntax::trivia::impl(TriviaPiece)@209::comment(text): opaque owned payload impl Into < String >
```

Each of those eight rows is a public by-value `impl Into<String>` signature
site, not opaque retained storage. Each receives `PublicInput`, with witness
`public by-value input; owning storage sites are separate rows`, `owed_by: -`,
and `external_implementation_coverage: -`. This does not assert bounded
destruction for the caller's implementation of `Into<String>`. The AST/lexical
constructors convert to owned `String`; discovery accounts for their concrete
storage independently. No new opaque storage slot was found.

Every removed row, verbatim (all six are from `unknown_storage.tsv`):

```text
UnknownConstruct	crates/rum-runtime/src/runtime/lowering/binary/left_chain.rs	5	9	rum_runtime::runtime::lowering::binary::left_chain::LeftChainLink.op: foreign representation rum_syntax::BinaryOp: BinaryOp
UnknownConstruct	crates/rum-runtime/src/runtime/model/requests.rs	99	17	rum_runtime::runtime::model::requests::SessionParseResponse.parsed: foreign representation rum_syntax::ParsedSource: ParsedSource
UnknownConstruct	crates/rum-runtime/src/runtime/model/requests.rs	119	21	rum_runtime::runtime::model::requests::SessionIsCompleteResponse.diagnostic/std::option::Option[Own]/arg0: foreign representation rum_syntax::StructuredSourceDiagnostic: StructuredSourceDiagnostic
UnknownConstruct	crates/rum-runtime/src/runtime/model/requests.rs	158	12	rum_runtime::runtime::model::requests::SessionRequestError.Syntax.0: foreign representation rum_syntax::StructuredSourceDiagnostic: StructuredSourceDiagnostic
UnknownConstruct	crates/rum-runtime/src/runtime/session/session_eval.rs	21	12	rum_runtime::runtime::session::session_eval::TopLevelEvalInput.Parsed.0: foreign representation rum_syntax::SyntaxExpr: SyntaxExpr
UnknownConstruct	crates/rum-runtime/src/runtime/session/session_state.rs	501	20	rum_runtime::runtime::session::session_state::EvalError.Syntax.0: foreign representation rum_syntax::SyntaxError: SyntaxError
```

These removals resolve respectively to `rum_syntax::ast::BinaryOp`,
`rum_syntax::document::ParsedSource`,
`rum_syntax::document::StructuredSourceDiagnostic` (two sites),
`rum_syntax::ast::SyntaxExpr`, and `rum_syntax::error::SyntaxError`. The new
regression test checks all six field-specific ownership edges. There are no
other removals, including from the disposition ledger.

## 4. Destruction classification and actual guards

All three new self-reaching types use ordinary recursive field destruction;
none has an explicit `Drop` implementation. These are existing structural
owning-tree shapes, not a new scanner class. They add no `Option<Arc<_>>`
accumulator chain and no opaque retained host payload. Accordingly, the
existing accumulator coverage gate applies unchanged, the public inputs use
the existing ledger, and the tree debt is recorded here rather than being
misattributed to an opaque slot.

| Newly self-reaching type | Classification |
| --- | --- |
| `rum_syntax::ast::SyntaxExpr` | **Unbounded destruction-depth debt.** Its `kind` owns `SyntaxExprKind`, which owns more expressions through `Vec`/`Box`. Public fields and constructors impose no depth limit. Parser guards constrain particular source constructions but do not prove a bound on every AST spine. |
| `rum_syntax::ast::SyntaxExprKind` | **The same unbounded AST debt**, as the other member of the mutual ownership cycle. `List`, `Association`, `Blank`, `Pattern`, `Prefix`, `Binary`, `ComparisonChain`, `Postfix`, and `Call` carry recursive ownership. The enum and its variants are public. |
| `rum_syntax::formatter::doc::Doc` | **Unbounded destruction-depth debt in an internal source-derived tree.** `Concat(Vec<Doc>)` and `Group { doc: Box<Doc>, .. }` recursively own documents. The type is private to formatter code, but its construction is not universally bounded by delimiter depth, as explained below. |

The parser's actual constants in `crates/rum-syntax/src/parser/front.rs` are
`MAX_DELIMITER_NESTING_DEPTH = 128` (line 10),
`MAX_OPERATOR_CHAIN_DEPTH = 128` (line 22), and
`MAX_STATEMENT_CHAIN_DEPTH = 256` (line 29).
`MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS = 4_096` is exported from syntax `lib.rs:18`.
`with_delimiter_depth` and `with_operator_chain_depth` test their current
counter against the corresponding 128 limit before incrementing it
(`front.rs:117-151`). These are parser counters, not a universal 128-node AST
height invariant.

There is a source-inspection qualification to the brief's source-bounded
premise. `parse_postfix_expr` (`front.rs:560-640`) repeatedly wraps its previous
expression. In the call branch, `with_delimiter_depth` guards parsing one
argument list, returns and decrements, and only then
`SyntaxExpr::call(expr, args, span)` boxes the prior head. A sequence shaped
like `f[a,b][a,b]...` therefore grows the owned head spine without accumulating
delimiter or operator depth. Repeated postfix wrapping also has no spine
counter at those sites. No deep input, ceiling experiment, or crash probe was
run; this is a control-flow/ownership finding, not a measured overflow claim.

The same call shape prevents a blanket source-bound claim for `Doc`.
`collect_container_group_plans` visits each call head and records a container
plan for a call with two arguments (`formatter/planning.rs:236-269,281-325`).
Successive calls have the same start and successively larger end positions.
`select_group_plan` chooses the largest eligible plan; `build_group_doc`
recurses into its range while suppressing that exact plan, allowing the next
smaller one (`formatter/trivia.rs:114-154`). Each level owns another boxed
`Doc::Group` within its document. There is no independent document depth cap
or iterative destruction. `ParsedSource` has private fields and is produced
through parsing (`document.rs:44-48,326-343`), so a host cannot directly inject
an arbitrary `Doc`; the source construction above is nevertheless available.

Host AST construction is a separate, direct path. Syntax `lib.rs:20` exports
`SyntaxExpr` and `SyntaxExprKind`; both have public construction surfaces
(`ast.rs:5-14,28-33,150-185`). Repeated use of `list`, `call`, or direct variant
construction can create any depth without invoking the parser. Runtime
exports `lower_syntax_expr(&SyntaxExpr)` (`runtime/mod.rs:74`,
`runtime/lowering.rs:7`) and accepts such a host-built tree **by reference**.
The host still owns and ultimately drops that AST. The runtime's
`TopLevelEvalInput::Parsed` is private and receives parser-produced
`ScriptForm` syntax (`session_eval.rs:310`); it is not a public by-value AST
injection API. This distinction does not remove either the public AST drop
debt or the runtime's ownership of source-produced ASTs.

The AST debt has the host-construction shape of the recorded `Expr` debt in
`TODO.md`'s Standing engineering debt section, with the additional source-spine
qualification above. That existing `Expr` entry is not claimed to already
document syntax/formatter debt. This report records the newly exposed debt;
`TODO.md` and the host documentation were not modified. No `Drop` design,
marker declaration, or production fix is introduced or proposed here.

## 5. Falsifier

`tests::ownership_graph::source::root_set::syntax_root_discovers_added_recursive_type_and_restores_source`
uses the actual production source collection and module loader. It adds this
text to an in-memory scratchpad copy of `crates/rum-syntax/src/ast.rs`:

```rust
struct ScannerRootSetProbe { next: Option<Box<ScannerRootSetProbe>> }
```

It scans the old three-root collection (the type is invisible), scans the new
four-root collection (the recursive type is found), and restores the AST source
from its saved string. Restoration checks that the probe disappears and every
other cycle and unknown is unchanged. This reconciles the requested mutation
with the hard prohibition on modifying production source. No recursive Rust
value is compiled or constructed by the probe, and no on-disk syntax file was
edited. No git restoration was used.

Verbatim output from the focused scanner run:

```text
test tests::ownership_graph::source::root_set::syntax_root_discovers_added_recursive_type_and_restores_source ... OLD ROOTS: rum_syntax::ast::ScannerRootSetProbe invisible (no definition or self-reaching row)
NEW ROOTS: rum_syntax::ast::ScannerRootSetProbe	crates/rum-syntax/src/ast.rs	986	rum_syntax::ast::ScannerRootSetProbe.next/std::option::Option[Own]/arg0/std::boxed::Box[Own]/arg0 -> rum_syntax::ast::ScannerRootSetProbe
RESTORED: rum_syntax::ast::ScannerRootSetProbe absent; all other cycles and unknowns unchanged
RESOLVED: all six former rum-syntax foreign storage rows are owning edges
ok
```

The probe is absent from the final inventories. The permanent regression test
repeats the add/see/remove experiment on source text only.

## 6. Verification

All Cargo commands used offline mode; all long-running commands used
`scripts/run-bounded.py`. `TMPDIR` was kept inside the repository. The two
standalone discovery runs initially used the allowed scanner scratch
directory; subsequent tests used the ignored repository `target` directory so
temporary test crates do not enter source inventories. Scratch copies/logs
were removed after their evidence was copied into this report.

Focused command, exit 0:

```sh
CARGO_NET_OFFLINE=true CARGO_TERM_COLOR=never TMPDIR="$PWD/target" python3 scripts/run-bounded.py --timeout 600 -- cargo test -p rum-stdlib --lib --offline --locked tests::ownership_graph:: -- --nocapture --test-threads=1
```

Verbatim excerpts:

```text
test tests::ownership_graph::accumulator_coverage::every_self_reaching_arc_chain_participates ... uncovered accumulator ownership paths: 0
ok

test tests::ownership_graph::host_payload_ledger::production_host_payload_ledger_is_complete_and_attributed ... HOST PAYLOAD LEDGER: 58 rows; 6 storage edges (3 Declared, 3 Unbounded); 52 PublicInput
checks 1/2/3: complete, no stale entries, field-specific expression attribution verified
external_implementation_coverage: unresolved for every Unbounded entry
ok

test tests::ownership_graph::production_inventory_is_exact ... EXACT INVENTORY self-reaching types: 60 rows
EXACT INVENTORY UnknownHostPayload: 58 rows
EXACT INVENTORY UnknownConstruct: 7311 rows
ok

test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 964 filtered out; finished in 52.27s
```

Mandatory formatting command, exit 0 with empty stdout/stderr:

```sh
CARGO_NET_OFFLINE=true CARGO_TERM_COLOR=never TMPDIR="$PWD/target" python3 scripts/run-bounded.py --timeout 180 -- cargo fmt --check
```

The first formatting check exited 1 with two layout differences in the new
`root_set.rs` test. Those differences were corrected; the mandatory final
check above passed. Neither discovery run nor the focused scanner suite
failed.

Mandatory workspace command:

```sh
CARGO_NET_OFFLINE=true CARGO_TERM_COLOR=never TMPDIR="$PWD/target" RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 python3 scripts/run-bounded.py --timeout 2400 -- cargo test --workspace --offline --locked
```

Exit **0**. Summing the 32 target summaries gives **2,465 passed, 0 failed, 57 ignored** (2,455 unit/integration tests and 10 doctests passed). The 57 ignored tests are pre-existing: 2 host, 3 runtime, and 52 stdlib.

The compiler emitted three dead-code warnings in the unchanged runtime tests:
`register_test_if_builtin`, `register_test_return_builtin`, and
`register_test_vm_template_builtins` are unused. There were no compilation or
test failures in this workspace run.

The loopback capability probe returned, verbatim:

```text
Loopback bind unavailable: [Errno 1] Operation not permitted
```

The skip is `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`, checked by
`crates/rum-jupyter/src/zeromq_native_test_support.rs::loopback_bind_available`.
It permits existing native tests to return early specifically when loopback
binding is denied. These early returns are separate from Rust's ignored count.
No external network or elevated execution was used.

Workspace target/result lines, verbatim with intervening individual test lines
omitted:

```text
     Running unittests src/main.rs (target/debug/deps/rum_cli-071a386fb6152ac7)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/cli.rs (target/debug/deps/cli-527e9f2d0cca7f28)
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
     Running unittests src/lib.rs (target/debug/deps/rum_core-49fc2a810a19f202)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_derive-b50628eb478e3209)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/accumulator_chain.rs (target/debug/deps/accumulator_chain-f4b1762065518378)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/derive.rs (target/debug/deps/derive-b38e62f915ec5c7d)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_host-a119f062096badf7)
test result: ok. 89 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s
     Running tests/adapter_contract.rs (target/debug/deps/adapter_contract-36e2ac93dcf516de)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/host_payload_destruction_contract.rs (target/debug/deps/host_payload_destruction_contract-1be17d3af592a343)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 35.15s
     Running tests/installer.rs (target/debug/deps/installer-8ebb99c00d41e78e)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/lifecycle_buildout.rs (target/debug/deps/lifecycle_buildout-5e85c48ede3e9593)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/lifecycle_contract.rs (target/debug/deps/lifecycle_contract-a2c563982853655a)
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
     Running tests/random_password_policy.rs (target/debug/deps/random_password_policy-ddd2fec076417963)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/stress.rs (target/debug/deps/stress-5ec2a75152f1305e)
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.19s
     Running tests/suspension_abandonment_contract.rs (target/debug/deps/suspension_abandonment_contract-ac93b53b9ae0d1d1)
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
     Running unittests src/lib.rs (target/debug/deps/rum_jupyter-edb9ee3aea58135e)
test result: ok. 106 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
     Running unittests src/bin/rum-jupyter-kernel.rs (target/debug/deps/rum_jupyter_kernel-965588c9ac9ffa23)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_repl-78a8fab085fc98d8)
test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s
     Running unittests src/lib.rs (target/debug/deps/rum_runtime-562b258c6eef9b02)
test result: ok. 990 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 4.65s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-c8c5a89643371dc7)
test result: ok. 932 passed; 0 failed; 52 ignored; 0 measured; 0 filtered out; finished in 19.79s
     Running unittests src/bin/generate-builtin-reference.rs (target/debug/deps/generate_builtin_reference-7e0be655aec7b381)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_syntax-e24f301251f42f42)
test result: ok. 190 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
     Running unittests src/lib.rs (target/debug/deps/rum_terminal-bae91ae0a1529dd6)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_core
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_derive
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
   Doc-tests rum_host
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.99s
   Doc-tests rum_jupyter
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_repl
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_runtime
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
   Doc-tests rum_stdlib
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_syntax
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_terminal
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

No A/B was run: inspection of the final diff confirms that all Rust changes
are under the stdlib's test-only ownership scanner. The remaining changes are
its regenerated inventories/ledger and this report. No evaluator, matcher,
value construction/destruction, or other production code changed. Syntax
source hashes match the initial scratchpad hashes.

## 7. Stop-loss / open questions

- No scope stop-loss was required to complete the scanner change. Every new
  row fits the existing discovery/ledger machinery; unresolved constructs
  remain unresolved.
- The three newly exposed recursive trees do not have a universal bounded
  destruction guarantee. Public AST construction and the uncapped source
  spines above need director disposition. This brief does not add a production
  fix, a new destruction declaration, a new classification axis, or overflow
  experiments.
- The new `Range<usize>` foreign representation, existing foreign storage
  unknowns, and opaque external implementations retain their analysis limits.
- The scope prohibits editing standing-debt/host documentation. The new syntax
  and formatter debt is disclosed in this report for the director; it is not
  silently folded into the existing `Expr` entry.
- Organization assessment: `source.rs` remains one coherent module-loading
  responsibility (about 10.6k characters, below the test-file warning range).
  The falsifier and root-boundary regression live separately in `root_set.rs`;
  the directory remains below the approximately 20-child review threshold.
- The initial worktree was clean. Changes remain uncommitted. No production
  source, protected document, derive crate, accumulator helper, host-payload
  trait, suspension trait, lockfile, or git configuration was modified; no
  unsafe code was added.
