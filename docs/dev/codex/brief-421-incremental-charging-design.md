# brief-421 — design: what `Incremental` would have to mean

Shape: **design-only**. `Scope: NONE — no source change, not one file.`
Worker: **cdx (Codex)**.

Gap B, step 4. This is design-only because the thing to falsify does not exist
yet, and because the change is cross-cutting enough that plan-first is the
project default.

## The state, measured

`MemoryGrowthContract::Incremental` says "retained growth must be charged as
data-dependent construction proceeds" (`metadata.rs:303-304`).

Director-measured:

- **233 declarations** carry `Incremental` across the two checked-in
  inventories.
- **`enforce_active_resource_limits_for_expr` has five callers**, and only two
  are a builtin charging its own construction: `factorial.rs:27` and `:64`. The
  other three are evaluator internals (`resolution.rs:411,490`,
  `eval_core/mod.rs:59`).
- There is **no dedicated incremental-charging primitive** anywhere in the tree.
- Worse, the one thing that looks like incremental charging is itself
  **strided**: `enforce_active_resource_limits_for_expr` does its full walk only
  on every 64th armed check (`constraints.rs:9,145-159`). So even `Factorial`'s
  in-loop polling does not bound what one step can add.

So building a falsifier now would report that 233 declarations are unbacked.
That is true and worth nothing: there is nothing for them to be backed by.

Eight builtins are already waiting on this. brief-420 retained the six `From*`
codecs plus `RumTokens` and `TokenizeWith`, each with the same recorded reason
in its own words — the output shape is known only after parsing, a per-byte
bound is sound but refuses ordinary long scalars, and what they need is an
incremental, parser-integrated check. Design for them specifically; they are
the hardest case and the one that will expose a design that only works for
loops.

## What I want back

A reviewable plan, plus drafted implementation briefs in `TEMPLATE.md` shape.
**Write no code.** Every claim carries `path:line`.

### 1. The primitive

`IntegerWorkBudget` (`crates/rum-stdlib/src/number_theory.rs:304-333`) is the
existing precedent: a small struct with `limit` and `used`, a `charge(units)`
that fails closed on overflow and on exceeding, threaded by hand through
`number_theory.rs` and `combinatorics.rs`. Start there and say whether a byte
budget should copy it, or whether a different shape — an RAII guard charging on
drop, a session method taking deltas, a builder wrapper around `Vec<Expr>` —
is better, and why. Name what you reject.

### 2. The baseline problem, which I think is the hard one

An integer-work budget is per-call and starts at zero. A **memory** budget
cannot: it must be relative to what the session already holds, and
`current_memory_proxy_bytes` is a full walk — the cost the stride exists to
avoid. Sampling once at handler entry and then charging deltas is the obvious
answer. Say whether it is sound, specifically:

- when nested evaluation inside the handler grows the session by other means;
- when the handler suspends and resumes, so construction spans two requests
  with a different session in between — brief-401 established that saved frames
  are not an unwind, and briefs 397-411 closed three defects of exactly this
  shape;
- when the handler fails partway and its partial construction is dropped.

If sampling at entry is not sound, say what is.

### 3. Parser integration — the eight codecs

`eval_from_json` (`crates/rum-stdlib/src/json.rs:11-15`) has
`session: &mut Session`; its decoder lives in `crates/rum-stdlib/src/json/decode.rs`
as recursive descent. Inventory, for all eight retained builtins, what the
decoder entry point takes today and what it would have to take. Threading a
`&mut Session` through recursive descent is one answer and probably a bad one;
say what you would do instead.

The eight already have depth guards (`MAX_JSON_DEPTH`, `next_depth`,
`MAX_TOML_DEPTH`) — director-verified, they are all guarded for nesting. Is the
byte budget the same shape of thing, threaded the same way, or different?

### 4. Cost

The unarmed path must stay free. A per-node charge is a per-node branch; say
what it costs when no ceiling is armed, and which microbench labels would prove
or disprove a regression. If your design puts a branch in `Vec<Expr>` pushes on
the ordinary path, say so plainly in the risks — that is a stop, not a detail.

### 5. The falsifier

What makes an `Incremental` declaration checkable, once the primitive exists?

Note the attribution problem before you answer: brief-419's falsifier snapshots
a counter around the handler, so charges made by *nested* builtin calls count
toward the outer handler. That was acceptable there because it makes the
instrument conservative — it under-reports rather than raising false alarms.
Say whether the same trade is acceptable here, and whether a stronger
attribution is worth building.

Also say what the rule should be. "Charged at least once" is weak: the promise
is that *un-charged growth between charges is bounded*. Is that checkable at
reasonable cost, or is "at least once" the honest first step? I would rather
have a weak rule you are honest about than a strong one you cannot enforce.

### 6. The stride

brief-414's own answer was to keep `ACTIVE_MEMORY_LIMIT_CHECK_STRIDE = 64`
"only after the structural inventory is enforced and every multiplicative site
is preflighted or incrementally reserved". This work is when that condition
comes due. Does landing incremental charging change the stride answer? Back it
with the recorded A/B (`docs/dev/host-embedding-line.md:107`: armed workload
3.825 s to 0.667 s), and say plainly if you did not remeasure.

### 7. Sequencing and drafted briefs

How many implementation steps, in what order, and which of the 233 declarations
does each cover. **Do not plan to reclassify anything**: brief-420's standing
rule is that a move into an unenforced contract is a deferral wearing a
repair's clothes, and it applies to moves *out* of `Incremental` too.

Draft the briefs as usable briefs, not step descriptions. A MAY list must
include a module root whenever a type it exports changes; that omission has
cost this project eight stop-losses.

## Method requirements

- **Every claim carries `path:line`.**
- **Cross-check any script-derived count against live evidence.** A regex over
  this repository has produced a confidently wrong count three times in its
  history, and I have made the same mistake five times in this line alone —
  most recently by taking the first `grep -rl` hit as a definition site when it
  was only a reference.
- **State first, verbatim, what your environment can actually run.**
- Say plainly where you reason from source rather than observation.
- **A confident wrong answer is worse than "I could not establish this."**

## Hard rules

- **`Scope: NONE`. Change no file** — not source, not tests, not docs. You may
  read anything, run `cargo` and the release `rum-cli`, and build throwaway
  crates under `/tmp` that you remove. If you believe a file must change, that
  is a finding for the report.
- Do NOT `git commit`, branch, push, checkout, stash, or touch git config. Do
  not touch `stash@{0}`.
- Touch nothing outside this repo except `/tmp` scratch you remove afterwards.
  No network; pass `--offline` to every `cargo` command.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **No new `unsafe`** in any proposal for `rum-core` or `rum-runtime`.
- **Append your report to this brief file** before finishing. brief-414 and
  brief-418 both left theirs in the dispatch log only; the first cost a wasted
  dispatch and nearly lost the inventory this line has depended on since.

## Report format

1. Environment capability, verbatim.
2. The primitive: options, what you reject, and your recommendation.
3. The baseline problem, with your answer for each of the three cases.
4. Parser integration: the eight-decoder inventory and your design.
5. Cost, with the labels that would prove it.
6. The falsifier and the rule you propose, including its honest limits.
7. The stride answer.
8. Sequencing and the drafted implementation briefs.
9. Risks and open questions — what you could not establish, and why.

---

## Codex report

### 1. Environment capability, verbatim

The capability probe and one release-binary execution produced the following
verbatim output; these are observations, not conclusions inferred from source
(`docs/dev/codex/brief-421-incremental-charging-design.md:128-149`).

```text
$ pwd
/home/remi/dev/rum
$ rustc --version
rustc 1.85.0 (4d91de4e4 2025-02-17)
$ cargo --offline --version
cargo 1.85.0 (d73d2caf9 2024-12-31)
$ python3 --version
Python 3.14.7
$ target/release/rum-cli --version
error: unknown flag `--version`
$ target/release/rum-cli --eval '1 + 1'
2
```

Therefore this workspace can run the pinned Rust/Cargo toolchain, Python, the
focused Cargo tests, and the existing release `rum-cli`; the binary does not
offer a `--version` flag (`crates/rum-cli/Cargo.toml:1-3`). Except for blocks
explicitly labelled **observed**, the rest of this report is reasoning from the
checked-in source and documentation, not a measurement of a proposed
implementation (`docs/dev/codex/brief-421-incremental-charging-design.md:130-138`).
I did not run a timing A/B because this brief forbids an implementation change
and asks the stride answer to say plainly whether it was remeasured
(`docs/dev/codex/brief-421-incremental-charging-design.md:108-115`).

**Observed inventory cross-check.** A Python parser for the exact inventory-row
syntax found 68 `Incremental` rows in the core inventory and 165 in the installed
inventory; an independent anchored `rg -c` found the same 68 and 165. Set
comparison found all 68 core names in the installed inventory, leaving 97
stdlib-only names, so the requested count is 68 + 165 = **233 inventory
declarations**, representing 165 distinct installed heads
(`crates/rum-runtime/src/runtime/tests/memory_growth_contract.rs:11-197`,
`crates/rum-stdlib/src/catalog/memory_growth_contract.rs:15-665`). The two live
invariant tests also passed; the core test equates its inventory to
`core_builtin_specs`, while the stdlib test checks every raw family declaration
and the effective installed session, so the regex count is not the sole evidence
(`crates/rum-runtime/src/runtime/tests/memory_growth_contract.rs:199-215`,
`crates/rum-stdlib/src/catalog/memory_growth_contract.rs:667-749`).

An exact-symbol `rg` call-site search also reproduced the stated five callers:
two factorial polls and three evaluator checkpoints. Direct inspection separates
those five invocations from the single method definition
(`crates/rum-runtime/src/runtime/arithmetic/factorial.rs:27-64`,
`crates/rum-runtime/src/runtime/eval_core/mod.rs:59-59`,
`crates/rum-runtime/src/runtime/resolution.rs:411-490`,
`crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:139-162`).

```text
running 1 test
test runtime::tests::memory_growth_contract::core_builtin_memory_growth_inventory_is_sorted_and_exhaustive ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 945 filtered out; finished in 0.00s

running 1 test
test catalog::memory_growth_contract::installed_builtin_memory_growth_inventory_is_sorted_and_exhaustive ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 832 filtered out; finished in 0.01s
```

### 2. The primitive

#### Recommendation

Keep three properties of `IntegerWorkBudget`: explicit hand-threading, checked
addition that fails closed, and an error at the construction site. Do **not**
copy its ownership model: it starts at zero and is local to one call, whereas an
active memory ceiling is session-relative and can be dynamically min-combined
(`crates/rum-stdlib/src/number_theory.rs:303-333`,
`crates/rum-runtime/src/runtime/model/config.rs:3-17`).

The primitive should be a safe, request-scoped **shared admission ledger** plus
short-lived **charge scopes**. `Session` should hold an
`Option<Arc<Mutex<MemoryLedger>>>`; `None` is the ordinary mode and allocates or
locks nothing, while the active `Arc<Mutex<_>>` is permitted to live in a
continuation frame because every frame is `Send + Sync + 'static`
(`crates/rum-runtime/src/runtime/model/session.rs:29-127`,
`crates/rum-runtime/src/runtime/suspension/core.rs:68-81`). The proposed runtime
types must be implemented entirely in safe Rust, as required by this brief
(`docs/dev/codex/brief-421-incremental-charging-design.md:140-151`).

The ledger should hold, under one active-only lock, the effective ceiling, the
last reconciled full-proxy sample, committed-but-not-yet-reconciled bytes, live
reservation bytes, a generation, and diagnostic owner/event data only when the
falsifier is armed. A charge scope should hold the shared ledger, its generation,
its exact outstanding credit, and an audit owner; saturating or checked
arithmetic must turn overflow into `MemoryConstraintExceeded`, never wrap
(`crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:174-188`,
`crates/rum-runtime/src/runtime/session/session_state.rs:517-536`).

The primitive's operations should be these semantic operations, irrespective of
the eventual Rust names:

1. **Reserve before logical retention.** Add a positive conservative proxy-byte
   delta to the shared live total only if `sample + committed + live + delta`
   stays at or below the active ceiling; the scope may spend that already
   admitted credit without another ledger lock (`crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:346-405`,
   `crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:181-186`).
2. **Commit exact ownership.** When a successfully built value escapes as a
   result or is installed in session state, move its charge from the scope's
   rollback-owned total into the ledger's unreconciled committed total; a
   persistent session mutation must commit at the mutation point rather than at
   handler return because assignments can retain state independently of the
   returned value (`docs/runtime-evaluation-audit.md:137-148`,
   `crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:108-242`).
3. **Release exact ownership.** Dropping an uncommitted scope or temporary child
   scope subtracts only its own live reservations; this makes parse failure and
   suspension abandonment safe without guessing how much unrelated session
   memory was freed (`crates/rum-runtime/src/runtime/suspension/core.rs:534-594`,
   `crates/rum-runtime/src/runtime/session/session_helpers.rs:189-235`).
4. **Reconcile only at a quiescent full walk.** When there is no live charge
   scope at an outer evaluation boundary, replace the sample with the actual
   `current_memory_proxy_bytes + candidate-expression proxy`, clear committed
   bytes, and advance the generation; while a scope is live, compare the full
   sample and ledger estimate but do not erase reservations
   (`crates/rum-runtime/src/runtime/eval_core/mod.rs:45-63`,
   `crates/rum-runtime/src/runtime/resolution.rs:398-467`).

`MemoryChargeScope` is the primitive; checked `Vec`, `String`, map, expression,
and session-mutation adapters are consumers of it, not substitutes for it. A
scope may reserve a chunk once and let an active-only builder spend local credit
per node, so a ledger lock need not occur for each `push`; unused credit is
released with the scope (`crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:346-405`,
`docs/dev/codex/TEMPLATE.md:86-95`).

Capacity needs an explicit rule. An active checked builder should reserve a
conservative requested capacity, perform a fallible `try_reserve_exact`, inspect
the actual new capacity, and top up the charge **before inserting or exposing a
new retained item**; if the top-up fails, it must drop that local builder. The
brief may promise modeled retained-memory admission, but not absence of a
transient allocator overshoot, because the documented ceiling is a proxy rather
than an allocator/RSS meter (`crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:385-400`,
`docs/host-embedding-and-hardening.md:112-121`). A session-owned container must
not be grown in place before that admission: grow a local replacement and swap
only after charging, or use already-admitted spare capacity, because a failed
top-up cannot drop the `Session` (`crates/rum-runtime/src/runtime/model/session.rs:29-127`,
`docs/runtime-evaluation-audit.md:145-148`).

The normal and armed construction paths should be separate monomorphizations:
the normal handler keeps the current `BuiltinFn`, and an armed sibling uses a
generic `NoMemoryCharge`/`ActiveMemoryCharge` implementation whose no-op methods
inline away. `BuiltinSpec` can carry an optional armed handler pointer, and the
existing builtin-spec cache can be rewritten/invalidated only when a ceiling is
entered or left so cache hits on ordinary calls retain the current handler path
(`crates/rum-runtime/src/runtime/metadata.rs:289-304`,
`crates/rum-runtime/src/runtime/metadata.rs:331-470`,
`crates/rum-runtime/src/runtime/resolution.rs:161-188`). This changes an exported
catalog type, so every implementation brief that changes the type must include
`metadata.rs`, `kernel_api.rs`, and their module roots in Scope
(`crates/rum-runtime/src/runtime/kernel_api.rs:90-119`,
`docs/dev/codex/brief-421-incremental-charging-design.md:117-126`).

#### Rejected shapes

- A byte-for-byte copy of `IntegerWorkBudget` is rejected: independent
  zero-based counters let nested calls spend the same headroom twice and do not
  survive a continuation boundary (`crates/rum-stdlib/src/number_theory.rs:303-333`,
  `crates/rum-runtime/src/runtime/session/session_requests.rs:265-355`).
- An RAII guard that charges **on drop** is rejected because admission would
  occur after allocation and `Drop` cannot return `EvalError`; RAII is useful in
  the opposite direction, to release an already admitted reservation when a
  partial builder is dropped (`crates/rum-runtime/src/runtime/session/session_state.rs:517-536`,
  `crates/rum-runtime/src/runtime/suspension/core.rs:68-81`).
- A `Session::charge(delta)` method with no owned scope is rejected because it
  cannot distinguish committed growth from a failed temporary and has nothing
  to store in a suspension frame (`crates/rum-runtime/src/runtime/suspension/core.rs:482-531`,
  `crates/rum-runtime/src/runtime/session/session_helpers.rs:189-235`).
- A universal wrapper around `Vec<Expr>` is rejected as the primitive: the live
  proxy also counts strings, big integers/rationals, maps, session stores,
  messages, traces, pending frames, and other containers
  (`crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:50-337`,
  `crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:346-405`).
- Threading `&mut Session` through parsers is rejected because it couples pure
  decoder state to the evaluator, creates recursive borrow pressure, and is
  impossible for the `rum-syntax` lexer without reversing the documented crate
  boundary (`ARCHITECTURE.md:168-224`,
  `crates/rum-syntax/src/lib.rs:1-32`).
- Thread-local/global allocation interception is rejected: it introduces ambient
  attribution, cannot preserve the zero-cost ordinary path, and is unnecessary
  for a modeled retained-memory contract; no proposal here requires `unsafe`
  (`docs/host-embedding-and-hardening.md:112-121`,
  `docs/dev/codex/TEMPLATE.md:125-146`).

### 3. The baseline problem

Sampling once at each handler entry is **not sound**. An outer handler's partial
builder is a native local and therefore need not appear in the session walk;
after it consumes `x` bytes, a nested handler can sample the same session-only
baseline and independently consume `y`, allowing `baseline + x + y` to exceed
the ceiling even though each local check passed
(`crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:50-337`,
`crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:173-239`). The sound baseline
is one shared dynamic-limit ledger initialized by the first unstrided full check
after a request or `MemoryConstrained` limit is installed, then advanced only by
owned reservations and safe reconciliation
(`crates/rum-runtime/src/runtime/session/session_helpers.rs:29-37`,
`crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:104-159`).

The answer for each required case is:

1. **Nested evaluation.** Every outer live reservation remains in the shared
   ledger, so a nested incremental scope sees less headroom. Existing preflight
   checks must also compare against `sample + committed + live`, not just a new
   `current_memory_proxy_bytes()` walk, or a nested preflight allocation can
   ignore its parent's native-local builder
   (`crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:174-188`,
   `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:102-157`). Existing
   projected allocations need either an owned preflight reservation or a
   conservative committed reservation until the next quiescent sample; changing
   only `Incremental` call sites while leaving this composition rule untouched
   would remain unsound (`crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:174-188`).
2. **Suspend/resume.** Do not sample a new independent baseline on resume. The
   same ledger and charge-scope handles must be fields of the handler's typed
   continuation state; the dispatch audit frame must retain the owner and
   original generation until terminal completion. The runtime moves the saved
   evaluation stack/depth into the resumption and drives existing frames rather
   than replaying the handler, and frame values must be `Send + Sync + 'static`
   (`crates/rum-runtime/src/runtime/suspension/core.rs:74-98`,
   `crates/rum-runtime/src/runtime/suspension/core.rs:482-531`). A second
   evaluation is rejected while input is pending, but the session's pending and
   resuming state still changes across requests, so preserving the ledger is
   required even though the Rust `Session` object is the same
   (`crates/rum-runtime/src/runtime/session/session_requests.rs:70-84`,
   `crates/rum-runtime/src/runtime/session/session_requests.rs:265-355`).
3. **Partial failure.** Local parser/builder reservations remain uncommitted and
   release on scope drop; a successfully installed session mutation commits its
   exact charge immediately and is not incorrectly refunded merely because the
   handler later errors. A top-level terminal error performs a quiescent sample
   after scopes are gone so conservative preflight commitments do not poison the
   next request (`crates/rum-runtime/src/runtime/session/session_helpers.rs:161-235`,
   `crates/rum-runtime/src/runtime/session/session_requests.rs:216-263`).

Nested `MemoryConstrained` currently saves and restores only
`current_memory_limit`; the implementation must save/restore the effective
ledger-limit state and armed-dispatch mode through both completion and
abandonment, while continuing to min-combine the nested limit
(`crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:104-136`,
`crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:253-270`,
`crates/rum-runtime/src/runtime/suspension/core.rs:343-349`). Evaluation rollback
snapshots likewise need a ledger checkpoint restored only after any trial-created
suspension has been abandoned, matching the existing restore ordering
(`crates/rum-runtime/src/runtime/session/session_helpers.rs:5-27`,
`crates/rum-runtime/src/runtime/session/session_helpers.rs:189-235`).

The current outer-boundary hooks are suitable: `evaluate_once` restores
`evaluation_depth` and pops the evaluation-stack entry before its result check,
and the fixed-point loop checks both normal and resumed candidates
(`crates/rum-runtime/src/runtime/eval_core/mod.rs:45-63`,
`crates/rum-runtime/src/runtime/resolution.rs:398-467`,
`crates/rum-runtime/src/runtime/resolution.rs:470-524`). Reconciliation may clear
committed charges only when the restored depth is zero **and** the ledger has no
live scope; at any nested point it must retain the conservative estimate even if
a strided full walk occurs (`crates/rum-runtime/src/runtime/eval_core/mod.rs:19-59`,
`crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:139-159`).

This becomes a sound promise only after every live `Incremental` declaration is
removed from a mechanically checked uncharged rollout allowlist and all other
memory admission mechanisms compose with the ledger. Until then, each landed
brief improves coverage but must not claim that the class as a whole is enforced
(`crates/rum-runtime/src/runtime/metadata.rs:291-306`,
`docs/dev/codex/brief-421-incremental-charging-design.md:117-126`).

### 4. Parser integration: the eight retained builtins

The proposed `C` below is a generic charge sink held by the parser/builder. The
normal entry instantiates a zero-sized no-op sink; the armed sibling receives a
scope from the runtime. Decoder errors must distinguish format errors from
`EvalError`, because every current handler converts a format error to a message
and `$Failed`, while a memory breach must remain the existing resource-limit
error (`crates/rum-stdlib/src/json.rs:11-43`,
`crates/rum-runtime/src/runtime/session/session_state.rs:517-536`).

| Builtin | Decoder entry and growth state today | Proposed charged entry |
|---|---|---|
| `FromCsv` | `from_csv(source: &str, config: FromCsvConfig) -> Result<Expr, CsvParseError>`; `CsvParser` holds only source/position/delimiter, builds row/field `Vec`s and `String`s, then clones them into an expression (`crates/rum-stdlib/src/csv.rs:266-305`, `crates/rum-stdlib/src/csv.rs:308-399`, `crates/rum-stdlib/src/csv.rs:436-470`). | Keep `from_csv` as the normal wrapper and add an internal `from_csv_with<C>(source, config, &mut C) -> Result<Expr, DecodeFailure<CsvParseError>>`; put `C` in `CsvParser`, and use separate temporary-row and final-expression child scopes (`crates/rum-stdlib/src/csv.rs:266-305`, `crates/rum-stdlib/src/csv.rs:436-470`). |
| `FromIni` | `from_ini(source: &str) -> Result<Expr, IniParseError>`; `IniParser` owns section/entry vectors and optional indexes and renders them to an association (`crates/rum-stdlib/src/ini.rs:104-132`, `crates/rum-stdlib/src/ini.rs:180-285`, `crates/rum-stdlib/src/ini.rs:402-429`). | Add `from_ini_with<C>(source, &mut C)` and store `C` in `IniParser`; charge key/value copies, index construction, sections, and the final association under separate temporary/final scopes (`crates/rum-stdlib/src/ini.rs:108-132`, `crates/rum-stdlib/src/ini.rs:192-269`). |
| `FromJson` | `decode::from_json(source: &str) -> ParseResult<Expr>`; the recursive-descent `Parser` holds source/bytes/position and directly builds strings, arrays, rules, and associations (`crates/rum-stdlib/src/json/decode.rs:55-84`, `crates/rum-stdlib/src/json/decode.rs:110-223`, `crates/rum-stdlib/src/json/decode.rs:225-240`). | Add `from_json_with<C>(source, &mut C)` and a charger field on `Parser`; all recursive methods use the field, so no `Session` borrow is threaded through recursion (`crates/rum-stdlib/src/json/decode.rs:71-84`, `crates/rum-stdlib/src/json/decode.rs:110-223`). |
| `FromToml` | `decode::from_toml(source: &str) -> Result<Expr, TomlParseError>` first constructs a `TomlTable`, then renders it; `Parser` owns the root/path, and `TomlTable` owns vectors plus a lazily built `HashMap` index (`crates/rum-stdlib/src/toml/decode.rs:17-52`, `crates/rum-stdlib/src/toml/model.rs:17-38`, `crates/rum-stdlib/src/toml/model.rs:73-96`). | Add `from_toml_with<C>(source, &mut C)`; thread `C` through `Parser` and the model insertion/render helpers, using one releasable scope for `TomlTable`/indexes and another committed scope for the rendered `Expr` (`crates/rum-stdlib/src/toml/decode.rs:17-52`, `crates/rum-stdlib/src/toml/model.rs:99-220`). |
| `FromXml` | `decode::from_xml(source: &str) -> ParseResult<Expr>`; `Parser` holds source/bytes/position, and recursive `parse_element` builds attribute vectors, text strings, and child vectors (`crates/rum-stdlib/src/xml/decode.rs:33-53`, `crates/rum-stdlib/src/xml/decode.rs:195-303`, `crates/rum-stdlib/src/xml/decode.rs:506-560`). | Add `from_xml_with<C>(source, &mut C)` and keep `C` on `Parser`; reserve for names, decoded text/entities, attributes, children, and normalization's replacement vector before retaining them (`crates/rum-stdlib/src/xml/decode.rs:195-320`, `crates/rum-stdlib/src/xml/decode.rs:506-560`). |
| `FromYaml` | `decode::from_yaml(source: &str) -> Result<Expr, YamlParseError>` first builds `Vec<Line>`, then `Parser` owns it and produces lists/associations/strings (`crates/rum-stdlib/src/yaml/decode.rs:50-84`, `crates/rum-stdlib/src/yaml/decode/lines.rs:21-72`, `crates/rum-stdlib/src/yaml/decode.rs:112-241`). | Add `scan_lines_with<C>` and `from_yaml_with<C>`; store the sink on `Parser`, release the line/index scope after parsing, and commit only the returned expression scope (`crates/rum-stdlib/src/yaml/decode.rs:50-84`, `crates/rum-stdlib/src/yaml/decode/lines.rs:21-72`). |
| `RumTokens` | `rum_tokens_expr(source: &str) -> Expr` repeatedly calls public `rum_syntax::lex_with_trivia(&str) -> Result<LexedSource, SyntaxError>`, then builds byte spans and result expressions; the lexer itself creates token/trivia vectors and copied trivia text (`crates/rum-stdlib/src/strings/tokenize.rs:216-302`, `crates/rum-syntax/src/lexer/mod.rs:21-36`, `crates/rum-syntax/src/lexer/trivia.rs:3-35`). | Add a generic `lex_with_trivia_charged<E>(input, &mut impl Charge<Error = E>)` sibling in `rum-syntax`, preserving `lex_with_trivia` as the no-op wrapper, and thread the same sink through span/result construction. Because the new type/function is re-exported, `crates/rum-syntax/src/lib.rs` is mandatory Scope (`crates/rum-syntax/src/lib.rs:20-34`, `docs/dev/codex/brief-421-incremental-charging-design.md:124-126`). |
| `TokenizeWith` | `tokenize_with_expr(rules: &[TokenizeRule], source: &str) -> Expr` first collects all input chars, accumulates spans, and calls the string-pattern matcher at each position; rule parsing also allocates class/pattern storage (`crates/rum-stdlib/src/strings/tokenize.rs:52-80`, `crates/rum-stdlib/src/strings/tokenize.rs:107-175`, `crates/rum-stdlib/src/strings/tokenize.rs:177-207`). | Add `tokenize_with_expr_with<C>` and thread `C` through rule parsing, the char/span builders, and allocation-producing matcher state; merely charging the final span vector misses matcher temporaries and the full char copy (`crates/rum-stdlib/src/strings/patterns.rs:484-520`, `crates/rum-stdlib/src/strings/patterns/matcher/engine.rs:125-248`). |

This keeps runtime policy in `rum-runtime`, builtin implementation in
`rum-stdlib`, and the syntax crate runtime-neutral: the syntax callback reports a
generic error and the stdlib adapter maps it to `EvalError`
(`ARCHITECTURE.md:172-224`, `crates/rum-syntax/src/lib.rs:1-32`). A parser field is
similar to a depth counter only in being explicitly threaded; unlike depth, byte
credit is session-relative, shared across nested calls, rollback-owned, and able
to outlive a request in a continuation (`crates/rum-stdlib/src/json/decode.rs:110-137`,
`crates/rum-runtime/src/runtime/suspension/core.rs:74-98`).

The source does **not** support the premise that all eight have nesting guards.
JSON has `MAX_JSON_DEPTH` checks, TOML has `MAX_TOML_DEPTH`/`next_depth`, and YAML
has `MAX_YAML_DEPTH`/`next_depth`
(`crates/rum-stdlib/src/json/decode.rs:6-18`,
`crates/rum-stdlib/src/json/decode.rs:110-124`,
`crates/rum-stdlib/src/toml.rs:5-22`,
`crates/rum-stdlib/src/toml/decode.rs:765-770`,
`crates/rum-stdlib/src/yaml.rs:5-20`,
`crates/rum-stdlib/src/yaml/decode.rs:373-378`). By contrast, XML's recursive
`parse_element()`/`parse_children()` signatures carry no depth and the parser has
no depth field; I could not establish an XML nesting guard in the live source
(`crates/rum-stdlib/src/xml/decode.rs:35-53`,
`crates/rum-stdlib/src/xml/decode.rs:195-303`). CSV and INI are iterative rather
than recursively nested, while the rum lexer uses an unbounded iterative comment
depth counter rather than a nesting-limit check
(`crates/rum-stdlib/src/csv.rs:290-338`,
`crates/rum-stdlib/src/ini.rs:127-178`,
`crates/rum-syntax/src/lexer/trivia.rs:50-83`). This is an open source finding,
not a reason to invent a guard inside a memory-accounting brief
(`docs/dev/codex/brief-421-incremental-charging-design.md:128-138`).

### 5. Cost and the labels that would prove it

The ordinary path must not test an `Option`, lock, allocate, or call a dynamic
charger at every node. The proposed normal decoder/builder is a no-op generic
monomorphization, while selection of an armed handler occurs when the active
limit changes and on a cold builtin-cache miss; ordinary cache hits continue to
return the cached `BuiltinSpec` and invoke its selected function pointer
(`crates/rum-runtime/src/runtime/resolution.rs:176-188`,
`crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:308-315`). If implementation
leaves an `if active_memory_limit` in every normal `Vec::push`, string append, or
matcher transition, that brief must stop rather than ship
(`docs/dev/codex/brief-421-incremental-charging-design.md:85-90`,
`docs/dev/codex/TEMPLATE.md:65-69`).

The armed path does pay for one scope/ledger handle per charged handler, a local
credit check per retained unit, and a lock only when replenishing a chunk or
committing/releasing a scope. That cost is confined to an explicitly active
mode; a lock on each node is avoidable because a scope can spend already
reserved local credit (`crates/rum-runtime/src/runtime/model/config.rs:9-17`,
`docs/dev/codex/TEMPLATE.md:86-95`).

The unarmed base/candidate A/B must include existing exact statements that
exercise evaluator dispatch and the affected families: `literal-integer`,
`list-literal-small`, `set-ownvalue`, `downvalue-exact-hit`,
`matcher-single-tiny`, `table-small`, `appendto-list-incremental-small/large`,
`association-set-key-incremental-small/large`, and
`stringreplace-pattern-unguarded`
(`scripts/core-language-microbench.sh:456-505`,
`scripts/core-language-microbench.sh:538-558`,
`scripts/core-language-microbench.sh:615-710`,
`scripts/core-language-microbench.sh:713-735`). Each rollout brief must add
targeted labels, prepared outside the timed statement, in long-scalar versus
dense-output pairs:

| Family | New exact-statement labels |
|---|---|
| CSV/INI | `from-csv-long-field`, `from-csv-dense-fields`, `from-ini-long-value`, `from-ini-dense-keys` (`crates/rum-stdlib/src/csv.rs:266-399`, `crates/rum-stdlib/src/ini.rs:104-285`). |
| JSON/TOML | `from-json-long-scalar`, `from-json-dense-array`, `from-toml-long-scalar`, `from-toml-dense-table` (`crates/rum-stdlib/src/json/decode.rs:110-240`, `crates/rum-stdlib/src/toml/decode.rs:123-225`). |
| XML/YAML | `from-xml-long-text`, `from-xml-dense-children`, `from-yaml-long-block-scalar`, `from-yaml-dense-sequence` (`crates/rum-stdlib/src/xml/decode.rs:195-320`, `crates/rum-stdlib/src/yaml/decode.rs:212-241`). |
| Tokenizers | `rumtokens-long-identifier`, `rumtokens-dense-punctuation`, `tokenizewith-long-unmatched`, `tokenizewith-dense-matches` (`crates/rum-stdlib/src/strings/tokenize.rs:141-302`). |
| Runtime state/results | family-specific exact statements plus the existing set/append/association/table/matcher/replace labels above (`scripts/core-language-microbench.sh:471-500`, `scripts/core-language-microbench.sh:538-613`, `scripts/core-language-microbench.sh:674-759`). |

For active-mode throughput, add paired labels whose exact statement includes a
generous `MemoryConstrained[body, limit]`; do not silently wrap an existing label
and call it the same probe, because the benchmark rule requires the label's exact
statement (`docs/dev/codex/TEMPLATE.md:78-98`). Every A/B must compare the same
target rows and same-tier neighbors from `timings.tsv`, with profiling/audit
variables off and an otherwise idle machine
(`docs/testing-strategy.md:74-94`).

### 6. The falsifier

The existing preflight falsifier snapshots one session-global counter and saves
that snapshot in the dispatch suspension frame; any nested projection advances
it and therefore can discharge the outer call
(`crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:205-238`,
`crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:421-486`). That conservative
under-reporting trade is **not strong enough for `Incremental`**: callback,
traversal, and repeated-retention heads are a central part of this class, so a
nested callback will commonly charge even if the outer result container never
does (`docs/dev/codex/brief-417-mandatory-growth-declarations.md:297-310`,
`docs/runtime-evaluation-audit.md:119-130`).

The stronger attribution is worth building because the proposed charge scope
already needs an owner that survives suspension. In test/explicit-audit mode,
dispatch should allocate a monotonically unique owner for each handler; the
charger captures it, nested dispatch receives a different owner, and the
dispatch-stage frame retains the original owner until terminal completion.
Production budgeting remains one shared ledger across all owners, while the
owner map/counters exist only behind the same cold-load `AtomicU8` test gate as
the present falsifier (`crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:361-379`,
`crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:421-486`). Continuation
frames that construct after resume must carry the charger; obtaining a fresh
charger without the saved owner is a test failure
(`crates/rum-runtime/src/runtime/suspension/core.rs:74-98`,
`docs/dev/codex/TEMPLATE.md:90-95`).

The honest first handler rule is:

> For an `Incremental` handler whose terminal result is more than the existing
> 1,024-byte allowance above its arguments, **or whose session proxy grows by
> more than that allowance even on a terminal error**, its exact audit owner must
> have made at least one positive reservation, unless the name remains in the
> sorted rollout allowlist.

The 1,024-byte threshold and argument/result proxy machinery already exist in
the slow validator, but `Incremental` is currently an unchecked arm
(`crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:253-279`,
`crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:382-419`). The session-delta
side is necessary for `Set`, messages, traces, options, and other retained-state
heads whose returned value alone is not the retained growth; the full session
proxy already walks those stores (`crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:108-327`).

“At least once” is deliberately weak. A handler could reserve one byte and then
grow without another reservation; a generic postcondition cannot observe that
gap without allocator interception or forcing a full proxy walk at every growth
event, and neither is a reasonable default mechanism for this modeled ceiling
(`crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:50-337`,
`docs/host-embedding-and-hardening.md:112-121`). The stronger practical layers
are: active-only builder assertions that consumed credit never exceeds reserved
credit, low-ceiling tests that force refusal at the first unadmitted chunk, and
the exact owner-specific handler rule above; raw constructors outside those
builders remain review risk rather than something Rust can infer
(`docs/dev/codex/brief-414-memory-accounting-gaps.md:463-488`).

The substrate should start with a sorted allowlist of all 165 distinct installed
`Incremental` names, and each rollout removes only its completed group. Separate
tests must prove a deliberately uncharged direct handler, an uncharged resumed
handler, nested-owner isolation, a charged direct handler, and a charged resumed
handler; the existing preflight tests show the corresponding direct/resumed
shape (`crates/rum-runtime/src/runtime/tests/memory_growth_contract.rs:217-225`,
`docs/dev/codex/brief-419-preflight-falsifier.md:232-244`). Structural inventory
tests remain mandatory because dynamic tests exercise paths, not all catalog
declarations (`crates/rum-runtime/src/runtime/tests/memory_growth_contract.rs:199-215`,
`crates/rum-stdlib/src/catalog/memory_growth_contract.rs:667-749`).

### 7. The stride answer

Keep `ACTIVE_MEMORY_LIMIT_CHECK_STRIDE = 64`, but only after the final rollout
allowlist is empty, the eight parser/tokenizer paths use the shared admission
ledger, existing projected checks compose with live reservations, and the two
factorial loops stop relying on a strided full walk for their growing integer
(`crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:5-9`,
`crates/rum-runtime/src/runtime/arithmetic/factorial.rs:18-34`,
`crates/rum-runtime/src/runtime/arithmetic/factorial.rs:54-71`). `Factorial` and
`Factorial2` are `Preflight`, not part of the 233 `Incremental` records, so that
cleanup is a zero-record prerequisite rather than a reclassification
(`crates/rum-stdlib/src/catalog/memory_growth_contract.rs:196-197`).

After those conditions, the strided walk is a reconciliation/backstop for
cumulative unclassified drift; it is not the mechanism that makes a large
classified allocation safe. The comment should then say: “Full proxy walks
sample cumulative, unclassified session drift every 64 evaluation points.
Declared multiplicative growth is admitted unstrided; the stride itself does
not bound per-step byte growth.” The current comment instead attributes the
between-walk bound to element/integer/iteration budgets
(`crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:5-9`,
`docs/dev/codex/brief-414-memory-accounting-gaps.md:490-500`).

The recorded A/B is 3.825 s at every check versus 0.667 s at stride 64, a 5.73x
ratio (an 82.6% reduction), while the default-path candidate median was -3.4%
with zero labels beyond the recorded noise floor
(`docs/dev/host-embedding-line.md:96-110`). I **did not remeasure** that A/B in
this design-only brief (`docs/dev/codex/brief-421-incremental-charging-design.md:108-115`).
Lowering the stride to 1 would pay the known full-walk cost and still would not
prevent a single allocation before the next evaluator checkpoint; adaptive
stride is also premature because it needs a proven maximum unreserved delta
(`crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:139-188`,
`docs/dev/codex/brief-414-memory-accounting-gaps.md:490-500`).

### 8. Sequencing and drafted implementation briefs

Nine implementation briefs are the smallest reviewable sequence I recommend:
one substrate, one hard parser/tokenizer pilot, six non-overlapping declaration
rollouts, and one closure gate. The drafts below follow the required Goal,
Context, Scope, Deliverable, pitfalls, Verification, and Hard-rules shape; they
are raw material for director review rather than dispatch-ready authority
(`docs/dev/codex/TEMPLATE.md:19-37`, `docs/dev/codex/TEMPLATE.md:41-159`).

#### Coverage ledger

The 68 core names count twice because they are rows in both the core and
installed inventories; the 97 remaining installed names count once. The
executable installed-inventory test deliberately starts with core specs before
adding stdlib families, which corroborates that record model
(`crates/rum-stdlib/src/catalog/memory_growth_contract.rs:680-724`,
`crates/rum-runtime/src/runtime/tests/memory_growth_contract.rs:11-197`).

| Step | Newly backed `Incremental` declarations | Exact head set |
|---|---:|---|
| A — substrate and owner-scoped falsifier | 0 | No production declaration is credited merely for having a primitive (`crates/rum-runtime/src/runtime/metadata.rs:291-306`). |
| B — eight parser/tokenizer pilots | 0 | `FromCsv`, `FromIni`, `FromJson`, `FromToml`, `FromXml`, `FromYaml`, `RumTokens`, `TokenizeWith` remain `Preflight`; the live allowlist confirms all eight (`crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:7-38`, `crates/rum-stdlib/src/catalog/memory_growth_contract.rs:223-232`, `crates/rum-stdlib/src/catalog/memory_growth_contract.rs:494-494`, `crates/rum-stdlib/src/catalog/memory_growth_contract.rs:622-622`). |
| C — core retained state | 94 (47 names x two inventories) | `$ContextPath`, `$MessageList`, `$Packages`, `$Path`, `AddTo`, `AppendTo`, `ApplyTo`, `Contexts`, `Decrement`, `DefaultValues`, `DefinePackage`, `Definition`, `DivideBy`, `DownValues`, `DumpSave`, `FileNames`, `Increment`, `Information`, `MessageDefinitions`, `MessageTags`, `Messages`, `Names`, `Off`, `On`, `Once`, `Options`, `OwnValues`, `PackageScope`, `PreDecrement`, `PreIncrement`, `PrependTo`, `Public`, `Save`, `Set`, `SetAttributes`, `SetDelayed`, `SetOptions`, `SubValues`, `SubtractFrom`, `SyntaxInformation`, `TagSet`, `TagSetDelayed`, `TimesBy`, `Unique`, `UpSet`, `UpSetDelayed`, `UpValues` (`crates/rum-runtime/src/runtime/tests/memory_growth_contract.rs:15-190`, `crates/rum-stdlib/src/catalog/memory_growth_contract.rs:15-665`). |
| D — core materializers and diagnostics | 42 (21 names x two inventories) | `ArgumentsOptions`, `Echo`, `EvaluationData`, `Extract`, `Level`, `Message`, `OptionValue`, `Part`, `Position`, `Print`, `Reap`, `ReplaceAt`, `ReplacePart`, `RumProfileSnapshot`, `Short`, `Sow`, `Stack`, `TestReport`, `Trace`, `TraceDetail`, `TracePrint` (`crates/rum-runtime/src/runtime/tests/memory_growth_contract.rs:27-181`, `crates/rum-stdlib/src/catalog/memory_growth_contract.rs:15-665`). |
| E — associations | 28 | `AssociateTo`, `Association`, `AssociationKeyDeflatten`, `AssociationKeyFlatten`, `AssociationMap`, `AssociationThread`, `Counts`, `CountsBy`, `DeleteDuplicatesBy`, `GatherBy`, `GroupBy`, `JoinAcross`, `KeyDropFrom`, `KeyGroupBy`, `KeyIntersection`, `KeyMap`, `KeySelect`, `KeySortBy`, `KeyUnion`, `KeyValueMap`, `Keys`, `Lookup`, `Merge`, `MergeNested`, `Normal`, `PositionIndex`, `Tally`, `Values` (`crates/rum-stdlib/src/catalog/memory_growth_contract.rs:31-665`). |
| F — functional/traversal | 44 | `Accumulate`, `Apply`, `Array`, `ArrayFilter`, `ArrayReduce`, `BlockMap`, `Cases`, `Comap`, `ComapApply`, `FixedPoint`, `FixedPointList`, `Fold`, `FoldList`, `FoldPair`, `FoldWhile`, `Insert`, `Map`, `MapAll`, `MapApply`, `MapAt`, `MapIndexed`, `MapThread`, `MaxBy`, `MaximalBy`, `MinBy`, `MinimalBy`, `Nest`, `NestList`, `NestWhile`, `NestWhileList`, `Outer`, `ReverseSortBy`, `Riffle`, `Select`, `SelectPositions`, `SequenceFold`, `SequenceFoldList`, `SortBy`, `SubsetMap`, `TakeLargestBy`, `TakeList`, `TakeSmallestBy`, `TakeWhile`, `Through` (`crates/rum-stdlib/src/catalog/memory_growth_contract.rs:15-665`). |
| G — rewrite/string | 9 | `Replace`, `ReplaceAll`, `ReplaceAllUnheld`, `ReplaceList`, `ReplaceRepeated`, `StringApply`, `StringCases`, `StringReplace`, `ToExpression` (`crates/rum-stdlib/src/catalog/memory_growth_contract.rs:15-665`). |
| H — algebra/schema/control generators | 16 | `CoefficientList`, `ConformSchema`, `D`, `Differences`, `Dot`, `ExceptionTypes`, `Expand`, `Factor`, `Inner`, `LazyTuples`, `Product`, `RegisterExceptionType`, `SchemaErrors`, `Sum`, `Table`, `Together` (`crates/rum-stdlib/src/catalog/memory_growth_contract.rs:15-665`). |
| I — closure/stride ratification | 0 new; certifies all 233 | Empty the rollout allowlist, migrate the two non-`Incremental` factorial polling sites, and ratify the stride only after A-H are green (`crates/rum-runtime/src/runtime/arithmetic/factorial.rs:18-71`, `docs/dev/codex/brief-421-incremental-charging-design.md:108-126`). |

The arithmetic is **94 + 42 + 28 + 44 + 9 + 16 = 233**, and the six credited
sets are disjoint by live set comparison. No step changes a
`MemoryGrowthContract` row; removing a name from the rollout allowlist means its
existing declaration is now backed, not reclassified
(`crates/rum-runtime/src/runtime/tests/memory_growth_contract.rs:11-215`,
`crates/rum-stdlib/src/catalog/memory_growth_contract.rs:15-749`,
`docs/dev/codex/brief-421-incremental-charging-design.md:117-126`).

---

#### Draft A — shared incremental-admission substrate and falsifier

##### Goal

Add the safe active-only memory ledger, scoped reservation API, no-op/armed
handler seam, quiescent reconciliation, and owner-scoped `Incremental`
falsifier. This brief backs zero production declarations; it supplies the
mechanical substrate and begins with all 165 distinct `Incremental` heads on a
sorted rollout allowlist (`crates/rum-runtime/src/runtime/metadata.rs:291-306`,
`crates/rum-stdlib/src/catalog/memory_growth_contract.rs:15-665`).

##### Context pointers

- Read `AGENTS.md`, `docs/testing-strategy.md`,
  `docs/guide/resource-limits-and-sandbox.md`, and this report before editing;
  memory limits are modeled proxy bytes and tests use rum's own contract
  (`docs/testing-strategy.md:1-37`,
  `docs/guide/resource-limits-and-sandbox.md:13-30`).
- The full walk is `Session::current_memory_proxy_bytes`, the current projection
  choke point is `enforce_active_memory_limit_for_projected_bytes`, and full
  expression checks are strided 64 (`crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:50-405`,
  `crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:139-188`).
- Request entry, dynamic limit entry/exit, rollback, and suspension are one
  lifecycle; the current fields and restore sites are distributed across
  session, constraints, and suspension modules
  (`crates/rum-runtime/src/runtime/session/session_helpers.rs:5-37`,
  `crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:104-136`,
  `crates/rum-runtime/src/runtime/suspension/core.rs:343-349`).

##### Scope

Files MAY modify:

- new `crates/rum-runtime/src/runtime/debug_tools/resources/charging.rs`, plus
  module roots `crates/rum-runtime/src/runtime/debug_tools/resources.rs` and
  `crates/rum-runtime/src/runtime/debug_tools/mod.rs`
  (`crates/rum-runtime/src/runtime/debug_tools/resources.rs:1-4`).
- `crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs` and
  `memory.rs` (`crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:139-188`,
  `crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:50-405`).
- `crates/rum-runtime/src/runtime/model/session.rs`, `model/cache.rs`,
  `model/mod.rs`, `session/session_state.rs`, `session/session_helpers.rs`,
  `session/session_api.rs`, `session/session_requests.rs`, and module root
  `session/mod.rs` (`crates/rum-runtime/src/runtime/model/session.rs:29-127`,
  `crates/rum-runtime/src/runtime/session/session_state.rs:576-665`).
- `crates/rum-runtime/src/runtime/metadata.rs`, `kernel_api.rs`,
  `eval_dispatch/mod.rs`, `eval_core/mod.rs`, `resolution.rs`,
  `suspension/core.rs`, and roots
  `crates/rum-runtime/src/runtime/suspension.rs`,
  `crates/rum-runtime/src/runtime/mod.rs`, and
  `crates/rum-runtime/src/lib.rs`; all roots are included because the charged
  handler/scope types cross module or crate boundaries
  (`crates/rum-runtime/src/runtime/metadata.rs:289-304`,
  `crates/rum-runtime/src/runtime/kernel_api.rs:90-119`).
- `crates/rum-runtime/src/runtime/tests/memory_growth_contract.rs`,
  `session_memory.rs`, `session_config.rs`, `suspension_abandonment.rs`, relevant
  files beneath `runtime/tests/session_requests/`, and test roots
  `runtime/tests/mod.rs` and `runtime/tests/session_requests/mod.rs`
  (`docs/testing-strategy.md:18-37`).
- `scripts/core-language-microbench.sh` only for the named substrate labels, and
  `docs/guide/resource-limits-and-sandbox.md` only for the settled modeled-memory
  lifecycle (`scripts/core-language-microbench.sh:456-558`,
  `docs/guide/resource-limits-and-sandbox.md:13-30`).

Files MUST NOT modify: either checked-in memory-growth inventory, any builtin's
contract value, `crates/rum-core/src/expr.rs`, `Cargo.lock`, unrelated docs/tests,
or `.git`; this brief is infrastructure, not reclassification
(`docs/dev/codex/brief-421-incremental-charging-design.md:117-126`,
`docs/dev/codex/TEMPLATE.md:55-69`).

##### Deliverable

- An active-only shared ledger and suspension-safe scope with checked/saturating
  reserve, commit, rollback, nested-limit, and quiescent-reconcile operations;
  no new `unsafe` (`crates/rum-runtime/src/runtime/suspension/core.rs:74-98`,
  `docs/dev/codex/brief-421-incremental-charging-design.md:140-151`).
- A normal/armed handler seam in `BuiltinSpec` and cache transition logic that
  does not branch or lock on an ordinary cache hit; include every exporting
  module root listed above (`crates/rum-runtime/src/runtime/metadata.rs:331-470`,
  `crates/rum-runtime/src/runtime/resolution.rs:176-188`).
- Projected-byte checks composed with live ledger reservations, without changing
  their existing test recorder semantics or the stride
  (`crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:139-188`).
- Ledger state preserved across request resume, nested `MemoryConstrained`,
  abandonment, reset, and evaluation rollback; full walks occur outside the
  ledger lock (`crates/rum-runtime/src/runtime/suspension/core.rs:482-594`,
  `crates/rum-runtime/src/runtime/session/session_helpers.rs:161-235`).
- An owner-scoped, test/explicit-audit-only falsifier and sorted 165-name rollout
  allowlist, with direct/resumed positive and negative proofs plus nested-owner
  isolation (`crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:361-486`,
  `crates/rum-runtime/src/runtime/tests/memory_growth_contract.rs:217-225`).
- Exhaustive `Session` memory-field accounting updated for every new field and
  active allocation (`crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:50-337`).

##### rum / Rust pitfalls

- Never clear committed reservations at nested evaluation depth or while a live
  scope exists; a native-local builder is not necessarily in the session walk
  (`crates/rum-runtime/src/runtime/eval_core/mod.rs:45-63`,
  `crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:50-337`).
- The ledger must share headroom across nested handlers, but audit owner counts
  must not; budgeting and falsifier attribution are deliberately different
  (`crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:205-238`).
- Do not hold the ledger lock while walking session memory or invoking a handler;
  the full walk visits pending/resuming frames that can own charger handles
  (`crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:299-327`).
- Preserve the typed continuation protocol and `Send + Sync + 'static` frame
  bound; do not replay a suspended handler (`crates/rum-runtime/src/runtime/suspension/core.rs:74-98`,
  `crates/rum-runtime/src/runtime/suspension/core.rs:482-531`).
- Assess and report every touched file in the 20k+ split-plan range
  (`docs/code-organization-guidelines.md:64-97`).

##### Verification

Run, bounded and offline, and paste the exact result lines:

```text
scripts/run-bounded.py --timeout 300 -- cargo --offline fmt --check
scripts/run-bounded.py --timeout 300 -- cargo --offline test -p rum-runtime memory_growth_contract
scripts/run-bounded.py --timeout 300 -- cargo --offline test -p rum-runtime session_memory
scripts/run-bounded.py --timeout 300 -- cargo --offline test -p rum-runtime suspension
scripts/run-bounded.py --timeout 1200 -- cargo --offline test --workspace --no-fail-fast
```

Before editing, preserve a base release binary under an explicitly granted
`/tmp/rum-421a-*` directory; after editing build the candidate offline and run
`CARGO_NET_OFFLINE=true scripts/core-language-microbench.sh` against both. Paste
the per-label rows for `literal-integer`, `list-literal-small`,
`downvalue-exact-hit`, `matcher-single-tiny`, and their same-tier neighbors;
profiling/audit variables stay off and any hot-row regression is a stop
(`scripts/core-language-microbench.sh:456-558`,
`docs/testing-strategy.md:74-94`).

##### Hard rules

- No git mutation, network, vendor oracle, contract reclassification, new
  `unsafe`, or write outside Scope and the named `/tmp` A/B directory
  (`docs/dev/codex/TEMPLATE.md:125-159`).
- No `Mutex`, allocation, atomic audit update, or active-memory branch on an
  ordinary cache hit; if the A/B shows a default-path regression, stop and
  report (`docs/dev/codex/TEMPLATE.md:65-69`,
  `docs/dev/codex/TEMPLATE.md:143-150`).
- Do not lower the stride in this brief, and do not remove a production name
  from the rollout allowlist (`crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:5-9`).

---

#### Draft B — charged codec and tokenizer pilots

##### Goal

Use Draft A's primitive in the six `From*` decoders, `RumTokens`, and
`TokenizeWith`, preserving their existing `Preflight` declarations while
removing their known-unprojected falsifier exemptions only after each path
performs real pre-retention admission (`crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:7-38`,
`docs/dev/codex/brief-420-preflight-repairs.md:242-264`). This backs zero of the
233 `Incremental` rows but proves the design on the eight shape-dependent parsers
that motivated it (`docs/dev/codex/brief-421-incremental-charging-design.md:32-38`).

##### Context pointers

- Read Section 4 of this report and the current per-builtin dispositions; all
  eight reject a useful whole-input projection for recorded, distinct reasons
  (`docs/dev/codex/brief-420-preflight-repairs.md:242-264`).
- `rum-syntax` owns lexing and must not import runtime semantics; `rum-stdlib`
  owns the codec/tokenizer implementations and can adapt a generic lexer error
  to `EvalError` (`ARCHITECTURE.md:172-224`).
- XML is recursively nested but has no established depth guard in the live
  source; memory charging may proceed, but the report must preserve that open
  finding rather than claim it was fixed (`crates/rum-stdlib/src/xml/decode.rs:35-53`,
  `crates/rum-stdlib/src/xml/decode.rs:195-303`).

##### Scope

Files MAY modify:

- `crates/rum-stdlib/src/csv.rs`, `ini.rs`, `json.rs`, `json/decode.rs`,
  `toml.rs`, `toml/decode.rs`, `toml/model.rs`, `xml.rs`, `xml/decode.rs`,
  `yaml.rs`, `yaml/decode.rs`, and `yaml/decode/lines.rs`, plus any touched
  codec module root listed here (`crates/rum-stdlib/src/csv.rs:266-470`,
  `crates/rum-stdlib/src/toml.rs:5-11`,
  `crates/rum-stdlib/src/yaml/decode.rs:1-10`).
- `crates/rum-stdlib/src/strings/tokenize.rs`, `strings/patterns.rs`, existing
  allocation-producing `.rs` files under `strings/patterns/matcher/`, and roots
  `crates/rum-stdlib/src/strings.rs` and `crates/rum-stdlib/src/lib.rs`
  (`crates/rum-stdlib/src/strings/tokenize.rs:1-7`,
  `crates/rum-stdlib/src/strings/patterns.rs:484-520`).
- `crates/rum-syntax/src/lexer/mod.rs`, `lexer/trivia.rs`, `trivia.rs`, relevant
  lexer tests, and root `crates/rum-syntax/src/lib.rs`; the crate root is
  mandatory for any new public charged lexer entry/error
  (`crates/rum-syntax/src/lexer/mod.rs:5-35`,
  `crates/rum-syntax/src/lib.rs:20-34`).
- the relevant catalog family files `catalog/csv.rs`, `catalog/ini.rs`,
  `catalog/json.rs`, `catalog/toml.rs`, `catalog/xml.rs`, `catalog/yaml.rs`, and
  `catalog/strings_text.rs`, plus `catalog/mod.rs`, only to attach armed handler
  pointers; no `memory_growth` value may change
  (`crates/rum-stdlib/src/catalog/memory_growth_contract.rs:223-232`,
  `crates/rum-stdlib/src/catalog/memory_growth_contract.rs:494-494`,
  `crates/rum-stdlib/src/catalog/memory_growth_contract.rs:622-622`).
- `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs` and its focused growth
  tests only to remove the eight old Preflight exemptions after their charged
  paths pass (`crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:7-38`).
- `crates/rum-stdlib/src/tests/json.rs`, `codecs_text.rs`, `codecs_toml.rs`,
  `codecs_xml.rs`, `codecs_yaml.rs`, relevant tokenizer/stress tests and test
  roots; `scripts/core-language-microbench.sh`; and
  `docs/guide/resource-limits-and-sandbox.md`
  (`docs/testing-strategy.md:18-37`,
  `scripts/core-language-microbench.sh:615-673`).

Files MUST NOT modify: either memory-growth inventory, any contract variant,
unrelated encoders, `rum-core`, `Cargo.lock`, or unrelated docs/tests
(`docs/dev/codex/brief-421-incremental-charging-design.md:117-126`).

##### Deliverable

- Normal no-op and armed charged entries for all eight, with the charger stored
  in parser/matcher state rather than `&mut Session` passed through recursion
  (`crates/rum-stdlib/src/json/decode.rs:71-137`,
  `ARCHITECTURE.md:172-224`).
- Charge every retained string, node, vector/map capacity, lexer token/trivia
  payload, matcher work buffer that survives a transition, and intermediate-to-
  result overlap; release temporary parse-model scopes precisely
  (`crates/rum-stdlib/src/toml/model.rs:17-96`,
  `crates/rum-syntax/src/lexer/mod.rs:21-35`,
  `crates/rum-stdlib/src/strings/tokenize.rs:141-302`).
- Preserve each format error/message, while propagating
  `MemoryConstraintExceeded` as a resource error rather than converting it to a
  codec `$Failed` (`crates/rum-stdlib/src/json.rs:35-43`,
  `crates/rum-runtime/src/runtime/session/session_state.rs:527-536`).
- Keep `rum_syntax::lex_with_trivia` source/API behavior unchanged and add a
  generic charged sibling with no runtime dependency
  (`crates/rum-syntax/src/lexer/mod.rs:21-36`, `ARCHITECTURE.md:172-195`).
- Low-ceiling proofs for both dense output refusal and realistic long-scalar
  acceptance, plus ordinary-output equivalence; only then remove the exact eight
  Preflight exemptions (`docs/dev/codex/brief-420-preflight-repairs.md:242-264`,
  `docs/dev/codex/brief-420-preflight-repairs.md:326-344`).

##### rum / Rust pitfalls

- Do not reserve `input.len() * worst_case_node_size`; that is the over-refusal
  this brief exists to avoid (`docs/dev/codex/brief-420-preflight-repairs.md:249-259`).
- CSV, INI, TOML, YAML, and tokenization hold substantial intermediate storage;
  charging both intermediate and final memory as permanently committed would
  create false refusals, so use distinct releasable scopes
  (`crates/rum-stdlib/src/csv.rs:266-470`,
  `crates/rum-stdlib/src/ini.rs:104-132`,
  `crates/rum-stdlib/src/toml/decode.rs:17-52`,
  `crates/rum-stdlib/src/yaml/decode.rs:50-84`).
- Preserve the normal allocation shape through no-op monomorphization; a normal
  per-node branch is a stop-loss (`docs/dev/codex/brief-421-incremental-charging-design.md:85-90`).
- Do not silently add an XML language limit or claim one exists; report the
  separate depth finding (`crates/rum-stdlib/src/xml/decode.rs:35-53`,
  `crates/rum-stdlib/src/xml/decode.rs:195-303`).
- Include and assess every touched module root/export and every 20k+ file
  (`docs/dev/codex/brief-421-incremental-charging-design.md:124-126`,
  `docs/code-organization-guidelines.md:64-97`).

##### Verification

Run and paste exact output:

```text
scripts/run-bounded.py --timeout 300 -- cargo --offline fmt --check
scripts/run-bounded.py --timeout 300 -- cargo --offline test -p rum-syntax
scripts/run-bounded.py --timeout 600 -- cargo --offline test -p rum-stdlib codecs
scripts/run-bounded.py --timeout 600 -- cargo --offline test -p rum-stdlib tokenize
scripts/run-bounded.py --timeout 1200 -- cargo --offline test --workspace --no-fail-fast
```

Run a base/candidate A/B with the 16 long/dense labels named in Section 5 plus
`literal-string`, `stringreplace-pattern-unguarded`, `matcher-single-tiny`, and
same-tier neighbors; use separate `/tmp/rum-421b-*` outputs and
`CARGO_NET_OFFLINE=true`, and paste every target/neighbor `timings.tsv` row
(`scripts/core-language-microbench.sh:463-464`,
`scripts/core-language-microbench.sh:538-673`,
`docs/dev/codex/TEMPLATE.md:78-123`).

##### Hard rules

- No git/network/vendor oracle/new dependency/new `unsafe`, and no writes beyond
  Scope plus named `/tmp` benchmark output (`docs/dev/codex/TEMPLATE.md:125-159`).
- No contract reclassification; all eight remain `Preflight` even though their
  armed implementation uses the common reservation primitive
  (`docs/dev/codex/brief-421-incremental-charging-design.md:117-126`).
- Do not remove an exemption until its direct low-ceiling and ordinary-output
  tests pass; stop on any default-path or same-tier timing regression
  (`docs/dev/codex/TEMPLATE.md:96-100`,
  `docs/dev/codex/brief-420-preflight-repairs.md:326-344`).

---

#### Draft C — core retained-state charging

##### Goal

Back the 47 core retained-state heads in the coverage ledger, removing exactly
those 47 names from the `Incremental` rollout allowlist after every retained
mutation/materialization path is charged. Because each head has a core and an
installed inventory row, this brief covers 94 of the 233 declarations without
changing either row (`crates/rum-runtime/src/runtime/tests/memory_growth_contract.rs:15-190`,
`crates/rum-stdlib/src/catalog/memory_growth_contract.rs:15-665`).

##### Context pointers

- The target set is `$ContextPath`, `$MessageList`, `$Packages`, `$Path`,
  `AddTo`, `AppendTo`, `ApplyTo`, `Contexts`, `Decrement`, `DefaultValues`,
  `DefinePackage`, `Definition`, `DivideBy`, `DownValues`, `DumpSave`,
  `FileNames`, `Increment`, `Information`, `MessageDefinitions`, `MessageTags`,
  `Messages`, `Names`, `Off`, `On`, `Once`, `Options`, `OwnValues`,
  `PackageScope`, `PreDecrement`, `PreIncrement`, `PrependTo`, `Public`, `Save`,
  `Set`, `SetAttributes`, `SetDelayed`, `SetOptions`, `SubValues`,
  `SubtractFrom`, `SyntaxInformation`, `TagSet`, `TagSetDelayed`, `TimesBy`,
  `Unique`, `UpSet`, `UpSetDelayed`, and `UpValues`
  (`crates/rum-runtime/src/runtime/tests/memory_growth_contract.rs:15-190`).
- These paths reconstruct arbitrary stored state or retain results in definitions,
  options, messages, namespaces, package state, and update operators
  (`docs/dev/codex/brief-414-memory-accounting-gaps.md:452-459`).
- Session memory accounting walks the corresponding stores, so charge formulas
  must use the same proxy components and must include capacity growth
  (`crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:75-253`).

##### Scope

Files MAY modify:

- `crates/rum-runtime/src/runtime/attributes.rs`, `builtins_context_loader.rs`,
  existing `.rs` files under `builtins_context_loader/`, `context_and_load_helpers.rs`,
  `namespace_tools.rs`, and existing `.rs` files under `namespace_tools/`
  (`docs/dev/codex/brief-414-memory-accounting-gaps.md:452-455`).
- `crates/rum-runtime/src/runtime/definitions/mod.rs` and existing `.rs` files
  under `definitions/`; `options.rs` and existing `.rs` files
  under `options/`; `messages/mod.rs` and existing `.rs` files under `messages/`
  (`docs/runtime-evaluation-audit.md:42-59`).
- `crates/rum-runtime/src/runtime/scoping.rs` and existing
  `.rs` files under `scoping/`; `package_tools.rs` and existing `.rs` files under
  `package_tools/`; `debug_tools/message_switch.rs`; `eval_core/symbol.rs`; and
  any touched directory module root
  (`docs/runtime-evaluation-audit.md:137-148`,
  `docs/dev/codex/brief-414-memory-accounting-gaps.md:452-459`).
- `crates/rum-runtime/src/runtime/builtin_catalog.rs`,
  `builtin_catalog/namespace/mod.rs`, existing namespace catalog files,
  `builtin_catalog/rules.rs`, `scoping.rs`, `debug.rs`, and the catalog root only
  to attach the armed handler pointers for the exact target set
  (`crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:41-99`).
- `crates/rum-runtime/src/runtime/model/session.rs` and any exact memory-ledger
  helper from Draft A needed for staged session-container commits; relevant
  suspension-frame files and their `mod.rs` roots when a changed state type is
  exported (`crates/rum-runtime/src/runtime/model/session.rs:29-127`,
  `docs/dev/codex/brief-421-incremental-charging-design.md:124-126`).
- corresponding files under `crates/rum-runtime/src/runtime/tests/`, their
  module roots, `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs` solely for
  this group's allowlist removals, `scripts/core-language-microbench.sh`, and
  `docs/guide/resource-limits-and-sandbox.md`
  (`docs/testing-strategy.md:18-37`,
  `scripts/core-language-microbench.sh:456-500`).

Files MUST NOT modify: either inventory, any `memory_growth` classification,
`rum-core`, stdlib implementations, `Cargo.lock`, or unrelated docs/tests
(`docs/dev/codex/brief-421-incremental-charging-design.md:117-126`).

##### Deliverable

- Armed charged siblings for all 47 names and unchanged ordinary handlers;
  remove a name from the rollout allowlist only after every material-growth path
  for it is covered (`crates/rum-runtime/src/runtime/metadata.rs:289-304`).
- Stage capacity-growing map/vector/string updates off-session, charge their
  observed proxy delta, then commit/swap; charge inserts into existing capacity
  before cloning the retained payload (`crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:108-253`,
  `docs/runtime-evaluation-audit.md:145-148`).
- Store charger scopes in every affected resume frame and commit exact mutations
  before a later callback can suspend or fail (`docs/runtime-evaluation-audit.md:119-130`,
  `crates/rum-runtime/src/runtime/suspension/core.rs:74-98`).
- Focused low-ceiling tests for definition replacement/growth, update operators,
  message/option accumulation, namespace listing, package save/definition
  rendering, and direct/resumed mutation; keep ordinary values/messages pinned
  (`docs/testing-strategy.md:39-58`).
- The owner-scoped falsifier green with exactly this 47-name removal, producing
  cumulative coverage of 94/233 declarations
  (`crates/rum-runtime/src/runtime/tests/memory_growth_contract.rs:199-215`).

##### rum / Rust pitfalls

- A handler-level rollback must not refund a session mutation that remains
  visible after error; commit at the actual state-store boundary
  (`crates/rum-runtime/src/runtime/session/session_helpers.rs:161-235`).
- `Expr` and `SharedArgs` are Arc-backed; do not add deep clones to obtain a size
  or stage an update when borrowed proxy traversal suffices
  (`crates/rum-core/src/expr.rs:637-715`,
  `docs/dev/codex/TEMPLATE.md:90-95`).
- Preserve definition precedence, delayed/immediate distinctions, sealed-package
  identity, and mutation cache invalidation; accounting cannot become a semantic
  shortcut (`docs/runtime-evaluation-audit.md:42-59`,
  `docs/runtime-evaluation-audit.md:137-148`).
- Include every changed resume-state/module root and assess 20k+ files
  (`docs/dev/codex/brief-421-incremental-charging-design.md:124-126`,
  `docs/code-organization-guidelines.md:64-97`).

##### Verification

Run bounded/offline formatting, focused runtime definition/options/messages/
namespace/package/mutation/session-memory tests, the owner falsifier, and the
workspace suite; paste exact results (`docs/testing-strategy.md:18-37`,
`docs/dev/codex/TEMPLATE.md:102-123`). At minimum:

```text
scripts/run-bounded.py --timeout 300 -- cargo --offline fmt --check
scripts/run-bounded.py --timeout 600 -- cargo --offline test -p rum-runtime definitions
scripts/run-bounded.py --timeout 600 -- cargo --offline test -p rum-runtime options
scripts/run-bounded.py --timeout 600 -- cargo --offline test -p rum-runtime session_memory
scripts/run-bounded.py --timeout 1200 -- cargo --offline test --workspace --no-fail-fast
```

Run a base/candidate A/B from named `/tmp/rum-421c-*` outputs for
`set-ownvalue`, `symbol-ownvalue-read`, `downvalue-exact-hit`, `module-local`,
`appendto-list-incremental-small/large`, and same-tier neighbors; paste all
named rows and stop on any regression
(`scripts/core-language-microbench.sh:465-506`,
`scripts/core-language-microbench.sh:674-710`,
`docs/testing-strategy.md:74-94`).

##### Hard rules

- No reclassification, git/network/vendor oracle/new `unsafe`, write outside
  Scope, or weakening of documented state semantics
  (`docs/dev/codex/TEMPLATE.md:125-159`).
- No ordinary-handler charge branch and no allowlist removal based only on one
  happy-path test (`docs/dev/codex/brief-421-incremental-charging-design.md:85-106`).
- Stop on a suspension semantic difference, unaccounted persistent mutation, or
  hot-label regression (`docs/dev/codex/TEMPLATE.md:90-100`).

---

#### Draft D — core materializers and diagnostic accumulation

##### Goal

Back the 21 core materializer/diagnostic heads in the coverage ledger and remove
exactly those names from the rollout allowlist. Their duplicate core/installed
rows make this brief cover 42 more declarations, bringing cumulative coverage to
136/233 without reclassification
(`crates/rum-runtime/src/runtime/tests/memory_growth_contract.rs:27-181`,
`crates/rum-stdlib/src/catalog/memory_growth_contract.rs:15-665`).

##### Context pointers

- The target set is `ArgumentsOptions`, `Echo`, `EvaluationData`, `Extract`,
  `Level`, `Message`, `OptionValue`, `Part`, `Position`, `Print`, `Reap`,
  `ReplaceAt`, `ReplacePart`, `RumProfileSnapshot`, `Short`, `Sow`, `Stack`,
  `TestReport`, `Trace`, `TraceDetail`, and `TracePrint`
  (`crates/rum-runtime/src/runtime/tests/memory_growth_contract.rs:27-181`).
- These heads build traversal results, rendered buffers, trace/reap buckets, or
  reports that can duplicate inputs and accumulated session data
  (`docs/dev/codex/brief-414-memory-accounting-gaps.md:453-458`).
- Tracing, test reports, folds, windows, and matching/replacement can suspend,
  so their typed states—not a handler-entry local—must own charge scopes
  (`docs/runtime-evaluation-audit.md:119-130`).

##### Scope

Files MAY modify:

- `crates/rum-runtime/src/runtime/debug_tools/mod.rs` and existing `.rs` files
  beneath `debug_tools/`, including `trace/` and `test_report/` plus every touched
  module root (`docs/dev/codex/brief-414-memory-accounting-gaps.md:455-458`).
- `crates/rum-runtime/src/runtime/tree_walkers.rs`, existing `.rs` files under
  `tree_walkers/`, `kernel_data/structure_data/mod.rs`, and existing `.rs` files
  under `kernel_data/structure_data/`
  (`docs/runtime-evaluation-audit.md:83-95`).
- `crates/rum-runtime/src/runtime/options/arguments.rs`, relevant message builtin
  files under `messages/`, and their module roots
  (`docs/dev/codex/brief-414-memory-accounting-gaps.md:453-454`).
- exact relevant core catalog files under `builtin_catalog/` plus
  `builtin_catalog.rs`, only for armed handler pointers; affected suspension
  state/resume files plus their module roots
  (`crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:41-99`,
  `crates/rum-runtime/src/runtime/suspension/core.rs:74-98`).
- corresponding runtime tests/test roots,
  `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs` only for these 21
  allowlist removals, `scripts/core-language-microbench.sh`, and the resource
  guide if public limit wording changes (`docs/testing-strategy.md:18-37`).

Files MUST NOT modify: either inventory/classification, stdlib families,
`rum-core`, `Cargo.lock`, or unrelated docs/tests
(`docs/dev/codex/brief-421-incremental-charging-design.md:117-126`).

##### Deliverable

- Charge result-vector nodes/capacity and copied payload in traversal/part/
  replacement materializers; use a retained scope in every continuation that
  can resume construction (`crates/rum-runtime/src/runtime/suspension/core.rs:74-98`,
  `crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:346-405`).
- Charge strings, message/print buffers, trace events/depths, reap buckets,
  profile/report models, and final rendered expressions without counting a
  temporary as permanently retained (`crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:198-297`).
- Test success, error, fallback, nested callback, and input-resume/abandon paths
  under low limits, asserting both the termination reason and post-failure
  session usability (`docs/testing-strategy.md:39-58`,
  `docs/guide/resource-limits-and-sandbox.md:47-57`).
- Remove exactly 21 names from the rollout allowlist only after the exact-owner
  falsifier and all focused tests pass; cumulative coverage is 136/233
  (`crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:253-279`).

##### rum / Rust pitfalls

- A diagnostic can retain the same value in a result, trace, message, and report;
  charge each proxy-visible ownership represented by the current full walker
  (`crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:198-327`).
- `Reap`/`Sow`, trace, and test-report cleanup must preserve abandonment ordering;
  dropping a charge scope cannot replace semantic cleanup
  (`crates/rum-runtime/src/runtime/suspension/core.rs:534-594`).
- Avoid a full session walk on each trace/traversal append; active local credit is
  the point of the primitive (`crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:139-188`).
- Include changed module roots and assess split-plan files
  (`docs/dev/codex/brief-421-incremental-charging-design.md:124-126`,
  `docs/code-organization-guidelines.md:64-97`).

##### Verification

Run bounded/offline formatting, focused debug-tools/tree-walker/part/reap/trace/
request-resume tests, the owner falsifier, and the workspace suite; paste exact
results (`docs/testing-strategy.md:18-37`). At minimum:

```text
scripts/run-bounded.py --timeout 300 -- cargo --offline fmt --check
scripts/run-bounded.py --timeout 600 -- cargo --offline test -p rum-runtime debug_tools
scripts/run-bounded.py --timeout 600 -- cargo --offline test -p rum-runtime traversal
scripts/run-bounded.py --timeout 600 -- cargo --offline test -p rum-runtime session_requests
scripts/run-bounded.py --timeout 1200 -- cargo --offline test --workspace --no-fail-fast
```

Run a base/candidate A/B under `/tmp/rum-421d-*` for `part-list`,
`cases-wide-no-match`, `replace-wide-no-match`,
`replacepart-wide-rule-list`, plus new exact trace/reap/report labels and their
same-tier neighbors; paste each named row
(`scripts/core-language-microbench.sh:473-478`,
`scripts/core-language-microbench.sh:569-613`,
`docs/testing-strategy.md:74-94`).

##### Hard rules

- No reclassification, git/network/vendor oracle/new `unsafe`, or writes beyond
  Scope and named `/tmp` output (`docs/dev/codex/TEMPLATE.md:125-159`).
- No eager replacement of a resumable flow, no default-path full walk/lock/
  branch, and no semantic change to messages/traces/reap results
  (`docs/dev/codex/TEMPLATE.md:65-95`).
- Stop on any direct/resumed mismatch, uncharged target path, or target/neighbor
  regression (`docs/dev/codex/TEMPLATE.md:96-100`).

---

#### Draft E — association-family charging

##### Goal

Back the 28 stdlib-only association heads in the coverage ledger and remove
exactly those names from the rollout allowlist, bringing cumulative coverage to
164/233 declarations without changing their `Incremental` classifications
(`crates/rum-stdlib/src/catalog/memory_growth_contract.rs:15-665`).

##### Context pointers

- The target set is `AssociateTo`, `Association`, `AssociationKeyDeflatten`,
  `AssociationKeyFlatten`, `AssociationMap`, `AssociationThread`, `Counts`,
  `CountsBy`, `DeleteDuplicatesBy`, `GatherBy`, `GroupBy`, `JoinAcross`,
  `KeyDropFrom`, `KeyGroupBy`, `KeyIntersection`, `KeyMap`, `KeySelect`,
  `KeySortBy`, `KeyUnion`, `KeyValueMap`, `Keys`, `Lookup`, `Merge`,
  `MergeNested`, `Normal`, `PositionIndex`, `Tally`, and `Values`
  (`crates/rum-stdlib/src/catalog/memory_growth_contract.rs:15-665`).
- The association root fans these heads across aggregate/basic/grouping/join/
  reshape/mutation/resumable/transform modules, and several callback-driven
  implementations already have persistent suspended state
  (`crates/rum-stdlib/src/associations/mod.rs:13-52`,
  `crates/rum-stdlib/src/associations/resumable.rs:1-13`).
- `AssociateTo` and `KeyDropFrom` are session mutations with held-target
  preparation and suspension, so local result accounting alone is insufficient
  (`crates/rum-stdlib/src/associations/mutation.rs:148-277`).

##### Scope

Files MAY modify:

- `crates/rum-stdlib/src/associations/mod.rs` and existing `.rs` files under
  `associations/`, including all changed resume/suspended-state roots
  (`crates/rum-stdlib/src/associations/mod.rs:13-52`).
- `crates/rum-stdlib/src/adaptive_set.rs` only where association indexes allocate,
  plus `crates/rum-runtime/src/runtime/kernel_data/association_data.rs`, existing
  `.rs` files beneath `kernel_data/association_data/`,
  `kernel_data/association_lvalue.rs`, roots `kernel_data/mod.rs` and
  `kernel_data/association_data.rs`, and `kernel_api.rs` if an exported helper
  signature changes (`crates/rum-runtime/src/runtime/kernel_data/association_data.rs:1-100`,
  `docs/dev/codex/brief-421-incremental-charging-design.md:124-126`).
- `crates/rum-stdlib/src/catalog/associations.rs`,
  `catalog/associations_transform.rs`, and root `catalog/mod.rs` only for armed
  handler pointers; `crates/rum-stdlib/src/lib.rs` if a module export changes
  (`ARCHITECTURE.md:215-224`).
- corresponding stdlib/runtime association and request-resume tests plus their
  module roots; `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs` only for
  these 28 allowlist removals; `scripts/core-language-microbench.sh`; and the
  resource guide if the public boundary needs clarification
  (`docs/testing-strategy.md:18-37`).

Files MUST NOT modify: either inventory or any contract value, unrelated
families, `rum-core`, `Cargo.lock`, or unrelated docs/tests
(`docs/dev/codex/brief-421-incremental-charging-design.md:117-126`).

##### Deliverable

- Armed charged paths for every result vector, association rule/key/value copy,
  grouping/index map, merge/sort buffer, and transformed association in the 28
  heads; ordinary paths remain no-op monomorphizations
  (`crates/rum-stdlib/src/associations/mod.rs:31-52`,
  `crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:346-405`).
- Charger ownership in persistent association sequences and every callback
  resume frame, with exact release on abandonment
  (`crates/rum-stdlib/src/associations/resumable.rs:1-13`,
  `crates/rum-runtime/src/runtime/suspension/core.rs:534-594`).
- Staged and charged `AssociateTo`/`KeyDropFrom` updates before the lvalue store;
  an error after a committed store does not refund it
  (`crates/rum-stdlib/src/associations/mutation.rs:215-277`).
- Focused low-ceiling direct/resumed/nested-callback tests spanning tiny, wide,
  duplicate-key, delayed-rule, mutation, grouping, merge, and sort cases
  (`docs/testing-strategy.md:39-58`).
- Exact-owner falsifier green after exactly 28 names leave the rollout allowlist;
  cumulative coverage is 164/233 (`crates/rum-stdlib/src/catalog/memory_growth_contract.rs:15-665`).

##### rum / Rust pitfalls

- The proxy logically revisits shared expression payloads, so formulas must
  match the existing association/result proxy rather than assume an Arc clone is
  free (`crates/rum-core/src/expr.rs:637-715`,
  `crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:346-405`).
- Lazy index creation and map-capacity changes need charging even when output
  cardinality is unchanged (`crates/rum-runtime/src/runtime/kernel_data/association_data.rs:21-100`).
- Preserve first-applicable/order semantics, delayed rule heads, stable sorting,
  and exact mutation messages; accounting is not permission to reorder work
  (`docs/runtime-evaluation-audit.md:49-59`,
  `crates/rum-stdlib/src/associations/mutation.rs:148-277`).
- Include changed module roots and assess split-plan files
  (`docs/dev/codex/brief-421-incremental-charging-design.md:124-126`,
  `docs/code-organization-guidelines.md:64-97`).

##### Verification

Run bounded/offline formatting, all association-focused runtime/stdlib tests,
request-resume tests, the owner falsifier, and the workspace suite; paste exact
results (`docs/testing-strategy.md:18-37`). At minimum:

```text
scripts/run-bounded.py --timeout 300 -- cargo --offline fmt --check
scripts/run-bounded.py --timeout 600 -- cargo --offline test -p rum-stdlib associations
scripts/run-bounded.py --timeout 600 -- cargo --offline test -p rum-runtime association
scripts/run-bounded.py --timeout 600 -- cargo --offline test -p rum-stdlib session_requests_transforms_resume_associations
scripts/run-bounded.py --timeout 1200 -- cargo --offline test --workspace --no-fail-fast
```

Run the base/candidate A/B under `/tmp/rum-421e-*` for
`association-small-literal-repeat`, both
`association-set-key-incremental-*` rows, the association Part read rows,
`keysortby-default-wide`, new exact callback/group/merge labels, and same-tier
neighbors; paste every named row
(`scripts/core-language-microbench.sh:423-454`,
`scripts/core-language-microbench.sh:713-735`,
`scripts/core-language-microbench.sh:925-936`).

##### Hard rules

- No reclassification, git/network/vendor oracle/new `unsafe`, or writes beyond
  Scope and named `/tmp` output (`docs/dev/codex/TEMPLATE.md:125-159`).
- No ordinary-path charge branch, full walk per association entry, semantic
  reorder, or eager replacement of continuation state
  (`docs/dev/codex/TEMPLATE.md:65-95`).
- Stop on an uncharged index/mutation path, direct/resumed mismatch, or any
  target/neighbor regression (`docs/dev/codex/TEMPLATE.md:96-100`).

---

#### Draft F — functional and structural traversal charging

##### Goal

Back the 44 stdlib-only functional/traversal heads in the coverage ledger and
remove exactly those names from the rollout allowlist, bringing cumulative
coverage to 208/233 declarations without reclassification
(`crates/rum-stdlib/src/catalog/memory_growth_contract.rs:15-665`).

##### Context pointers

- The target set is `Accumulate`, `Apply`, `Array`, `ArrayFilter`, `ArrayReduce`,
  `BlockMap`, `Cases`, `Comap`, `ComapApply`, `FixedPoint`, `FixedPointList`,
  `Fold`, `FoldList`, `FoldPair`, `FoldWhile`, `Insert`, `Map`, `MapAll`,
  `MapApply`, `MapAt`, `MapIndexed`, `MapThread`, `MaxBy`, `MaximalBy`, `MinBy`,
  `MinimalBy`, `Nest`, `NestList`, `NestWhile`, `NestWhileList`, `Outer`,
  `ReverseSortBy`, `Riffle`, `Select`, `SelectPositions`, `SequenceFold`,
  `SequenceFoldList`, `SortBy`, `SubsetMap`, `TakeLargestBy`, `TakeList`,
  `TakeSmallestBy`, `TakeWhile`, and `Through`
  (`crates/rum-stdlib/src/catalog/memory_growth_contract.rs:15-665`).
- The functional module distributes these across apply/map, arrays, callable,
  iteration, sequence, and targeted-map code; its public internal exports show
  the callback-heavy surface (`crates/rum-stdlib/src/functional_ops/mod.rs:11-38`).
- Structure operations supply insert/select/sort/riffle/take materializers and
  persistent state; the shared matcher/rewrite machinery also owns `Cases`
  semantics (`crates/rum-stdlib/src/structure_ops/mod.rs:12-59`,
  `docs/runtime-evaluation-audit.md:83-95`).

##### Scope

Files MAY modify:

- `crates/rum-stdlib/src/functional_ops/mod.rs` and every existing `.rs` file
  beneath `functional_ops/`, including all apply-map/array/iteration/sequence/
  targeted-map resume and suspended-state module roots
  (`crates/rum-stdlib/src/functional_ops/mod.rs:11-38`).
- `crates/rum-stdlib/src/structure_ops/mod.rs` and the exact implementation/
  state/resume files under `structure_ops/` used by the target set, including
  `collection_stats`, `list_lookup`, `list_plumbing`, `ordering_sets`,
  `resumable_merge_sort`, `selectors`, `sequence`, and their module roots
  (`crates/rum-stdlib/src/structure_ops/mod.rs:12-59`).
- target-relevant files and roots under
  `crates/rum-runtime/src/runtime/kernel_data/functional_data.rs`,
  `kernel_data/rewrite_data/`, `tree_walkers/`, and `pattern_matching/`, plus
  `kernel_api.rs` whenever an exported continuation/helper type changes
  (`docs/runtime-evaluation-audit.md:83-95`,
  `docs/dev/codex/brief-421-incremental-charging-design.md:124-126`).
- `crates/rum-stdlib/src/catalog/functional.rs`, `catalog/structure.rs`,
  `catalog/structure_ordering.rs`, and root `catalog/mod.rs` only for armed
  handler pointers; `crates/rum-stdlib/src/lib.rs` for changed module exports
  (`ARCHITECTURE.md:215-224`).
- corresponding stdlib/runtime functional, traversal, structure, matcher, and
  request-resume tests plus every touched test root;
  `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs` only for the 44 allowlist
  removals; `scripts/core-language-microbench.sh`; and the resource guide if
  needed (`docs/testing-strategy.md:18-37`).

Files MUST NOT modify: either contract inventory/value, non-target stdlib
families, `rum-core`, `Cargo.lock`, or unrelated docs/tests
(`docs/dev/codex/brief-421-incremental-charging-design.md:117-126`).

##### Deliverable

- Charged result and work-buffer builders for all 44 targets, reserving before
  each retained callback result/container growth and releasing discarded
  candidates (`crates/rum-stdlib/src/functional_ops/mod.rs:21-38`,
  `crates/rum-stdlib/src/structure_ops/mod.rs:33-59`).
- Charger scopes carried in every persistent functional/structure sequence,
  sorting continuation, and callback resume frame; nested callback charges keep
  separate audit owners but share production headroom
  (`docs/runtime-evaluation-audit.md:119-130`,
  `crates/rum-runtime/src/runtime/suspension/core.rs:74-98`).
- Migrate current repeated total-size projected checks inside `Select` to real
  delta reservations so the ledger does not accumulate triangular conservative
  charges (`crates/rum-stdlib/src/functional_ops/sequence/select.rs:233-268`).
- Low-ceiling tests spanning empty/singleton/wide/nested input, callback success/
  decline/error/input-resume, limit-short-circuit, ordering stability, and
  post-abandon session usability (`docs/testing-strategy.md:39-58`).
- Exact-owner falsifier green after exactly 44 removals; cumulative coverage is
  208/233 (`crates/rum-stdlib/src/catalog/memory_growth_contract.rs:15-665`).

##### rum / Rust pitfalls

- A callback's own charge cannot satisfy its caller's container growth; the
  owner-scoped falsifier must see an outer reservation for the outer retained
  slot/payload (`crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:205-238`).
- Limit/short-circuit variants must release unused reserved credit and must not
  pre-reserve the generated-element maximum when actual output stops early
  (`crates/rum-stdlib/src/functional_ops/sequence/select.rs:233-282`).
- Preserve suspension order and avoid owned-tree clones or per-node full walks
  on evaluator/matcher hot paths (`docs/dev/codex/TEMPLATE.md:90-98`).
- Include changed exported-type roots and assess split-plan files
  (`docs/dev/codex/brief-421-incremental-charging-design.md:124-126`,
  `docs/code-organization-guidelines.md:64-97`).

##### Verification

Run bounded/offline formatting, all focused functional/structure/traversal/
matcher/request-resume tests, the owner falsifier, and the workspace suite;
paste exact results (`docs/testing-strategy.md:18-37`). At minimum:

```text
scripts/run-bounded.py --timeout 300 -- cargo --offline fmt --check
scripts/run-bounded.py --timeout 600 -- cargo --offline test -p rum-stdlib functional_ops
scripts/run-bounded.py --timeout 600 -- cargo --offline test -p rum-stdlib traversal
scripts/run-bounded.py --timeout 600 -- cargo --offline test -p rum-runtime matcher
scripts/run-bounded.py --timeout 1200 -- cargo --offline test --workspace --no-fail-fast
```

Run a base/candidate A/B under `/tmp/rum-421f-*` for `table-small`,
`table-iterator-plus`, `cases-wide-no-match`, the matcher-wide rows,
`keysortby-default-wide`, new exact map/fold/select/sort rows, and all same-tier
neighbors; paste every named row
(`scripts/core-language-microbench.sh:456-500`,
`scripts/core-language-microbench.sh:538-598`,
`scripts/core-language-microbench.sh:925-936`).

##### Hard rules

- No reclassification, git/network/vendor oracle/new `unsafe`, or writes beyond
  Scope and named `/tmp` output (`docs/dev/codex/TEMPLATE.md:125-159`).
- No ordinary per-node charge branch, callback-result deep clone, per-step full
  session walk, or continuation collapse (`docs/dev/codex/TEMPLATE.md:65-98`).
- Stop on any missed outer-owner charge, direct/resumed semantic mismatch, or
  hot-label regression (`docs/dev/codex/TEMPLATE.md:96-100`).

---

#### Draft G — rewrite and string-rewrite charging

##### Goal

Back the nine stdlib-only rewrite/string heads in the coverage ledger and
remove exactly those names from the rollout allowlist, bringing cumulative
coverage to 217/233 declarations without reclassification
(`crates/rum-stdlib/src/catalog/memory_growth_contract.rs:15-665`).

##### Context pointers

- The target set is `Replace`, `ReplaceAll`, `ReplaceAllUnheld`, `ReplaceList`,
  `ReplaceRepeated`, `StringApply`, `StringCases`, `StringReplace`, and
  `ToExpression` (`crates/rum-stdlib/src/catalog/memory_growth_contract.rs:15-665`).
- The first five stdlib handlers delegate to runtime rewrite data; the three
  string heads build match/replacement results in pattern-search code
  (`crates/rum-stdlib/src/rewrite_builtins.rs:38-75`,
  `docs/dev/codex/brief-414-memory-accounting-gaps.md:440-445`).
- `ToExpression` parses, lowers, evaluates, and can suspend twice, so its scope
  must cover syntax/lowering temporaries and survive both continuation frames
  (`crates/rum-stdlib/src/strings/text/builtins.rs:438-481`).

##### Scope

Files MAY modify:

- `crates/rum-stdlib/src/rewrite_builtins.rs` and roots
  `crates/rum-stdlib/src/lib.rs`; `catalog/rewrite.rs`,
  `catalog/strings_match.rs`, `catalog/strings_text.rs`, and `catalog/mod.rs`
  only for the nine armed handler pointers (`crates/rum-stdlib/src/rewrite_builtins.rs:1-75`,
  `ARCHITECTURE.md:215-224`).
- `crates/rum-runtime/src/runtime/kernel_data/rewrite_data.rs`, existing `.rs`
  files under `kernel_data/rewrite_data/`, relevant `pattern_matching/` files,
  and roots `kernel_data/mod.rs`, `kernel_data/rewrite_data.rs`,
  `pattern_matching/mod.rs`, and `kernel_api.rs` whenever an exported
  `RewriteDataBuiltin`/continuation signature changes
  (`docs/runtime-evaluation-audit.md:83-95`,
  `docs/dev/codex/brief-421-incremental-charging-design.md:124-126`).
- `crates/rum-stdlib/src/strings.rs`, `strings/patterns.rs`, exact files and
  module roots under `strings/patterns/search/` and `strings/patterns/matcher/`,
  `strings/text.rs`, `strings/text/builtins.rs`, and its resume-frame root
  (`crates/rum-stdlib/src/strings/patterns/search.rs:198-475`,
  `crates/rum-stdlib/src/strings/text/builtins.rs:438-481`).
- For `ToExpression` only: charged siblings in the exact lexer/parser files
  needed under `crates/rum-syntax/src/`, root `crates/rum-syntax/src/lib.rs`,
  `crates/rum-runtime/src/runtime/lowering.rs`, exact lowering child files, root
  `crates/rum-runtime/src/runtime/mod.rs`, and `kernel_api.rs` if the charged
  lowering entry is exported (`crates/rum-stdlib/src/strings/text/builtins.rs:460-462`,
  `ARCHITECTURE.md:172-213`).
- corresponding syntax/runtime/stdlib rewrite, string, lowering, and resume
  tests plus every touched test root;
  `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs` only for nine allowlist
  removals; `scripts/core-language-microbench.sh`; and the resource guide if
  needed (`docs/testing-strategy.md:18-37`).

Files MUST NOT modify: either inventory/contract value, unrelated string
families, `rum-core`, `Cargo.lock`, or unrelated docs/tests
(`docs/dev/codex/brief-421-incremental-charging-design.md:117-126`).

##### Deliverable

- Charged runtime rewrite builders for candidate paths, substitutions, rule
  materialization, result lists, and repeated replacements, carrying scope state
  through every existing suspension (`docs/runtime-evaluation-audit.md:83-95`,
  `docs/runtime-evaluation-audit.md:119-130`).
- Charged string matcher/search state and output strings/vectors for all three
  string heads; callback results have their own nested owners while the outer
  output receives an outer-owner reservation
  (`crates/rum-stdlib/src/strings/patterns/search.rs:198-475`).
- A charged syntax-parse/lower path for `ToExpression`, with generic syntax-side
  callbacks and runtime-side error mapping, and charger ownership in both
  `ToExpression` resume frames (`crates/rum-stdlib/src/strings/text/builtins.rs:438-481`,
  `ARCHITECTURE.md:172-213`).
- Low-ceiling tests covering direct/resumed guarded and unguarded rules,
  replacement limits, no-match cases, dense string matches, long unmatched
  text, and `ToExpression` compact-node-rich input; ordinary semantics stay
  pinned (`docs/testing-strategy.md:39-58`).
- Exact-owner falsifier green after exactly nine removals; cumulative coverage
  is 217/233 (`crates/rum-stdlib/src/catalog/memory_growth_contract.rs:15-665`).

##### rum / Rust pitfalls

- Matcher/callback charges from a nested call must not discharge the outer
  rewrite owner; charge the outer retained result separately
  (`crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:205-238`).
- `ReplaceRepeated` must preserve its independent iteration limit and fixed-point
  semantics; memory admission is an additional error path, not a replacement
  (`crates/rum-runtime/src/runtime/session/session_state.rs:537-540`,
  `docs/runtime-evaluation-audit.md:28-36`).
- The syntax crate stays runtime-neutral and normal `parse_expr` remains a no-op
  wrapper; do not put `Session` or `EvalError` in `rum-syntax`
  (`ARCHITECTURE.md:172-195`).
- Include changed exported-type/module roots and assess split-plan files
  (`docs/dev/codex/brief-421-incremental-charging-design.md:124-126`,
  `docs/code-organization-guidelines.md:64-97`).

##### Verification

Run bounded/offline syntax, lowering, runtime rewrite/matcher, stdlib string/
rewrite, request-resume, owner-falsifier, and workspace suites; paste exact
results (`docs/testing-strategy.md:18-37`). At minimum:

```text
scripts/run-bounded.py --timeout 300 -- cargo --offline fmt --check
scripts/run-bounded.py --timeout 600 -- cargo --offline test -p rum-syntax
scripts/run-bounded.py --timeout 600 -- cargo --offline test -p rum-runtime replace
scripts/run-bounded.py --timeout 600 -- cargo --offline test -p rum-stdlib string
scripts/run-bounded.py --timeout 1200 -- cargo --offline test --workspace --no-fail-fast
```

Run a base/candidate A/B under `/tmp/rum-421g-*` for `replaceall-simple`, all
guarded/wide replacement rows, all string-replace rows, matcher-wide rows, and
new exact `ToExpression` scalar/dense rows with same-tier neighbors; paste every
named row (`scripts/core-language-microbench.sh:538-673`,
`docs/testing-strategy.md:74-94`).

##### Hard rules

- No reclassification, git/network/vendor oracle/new `unsafe`, or writes beyond
  Scope and named `/tmp` output (`docs/dev/codex/TEMPLATE.md:125-159`).
- No ordinary matcher/node branch, per-transition full walk, deep-tree clone, or
  suspension collapse (`docs/dev/codex/TEMPLATE.md:65-98`).
- Stop on a nested-owner false discharge, replacement semantic difference, or
  target/neighbor regression (`docs/dev/codex/TEMPLATE.md:96-100`).

---

#### Draft H — algebra, schema, and control-generator charging

##### Goal

Back the final 16 stdlib-only `Incremental` heads in the coverage ledger and
remove exactly those names from the rollout allowlist, bringing cumulative
coverage to all 233 declarations without changing a classification
(`crates/rum-stdlib/src/catalog/memory_growth_contract.rs:15-665`).

##### Context pointers

- The target set is `CoefficientList`, `ConformSchema`, `D`, `Differences`,
  `Dot`, `ExceptionTypes`, `Expand`, `Factor`, `Inner`, `LazyTuples`, `Product`,
  `RegisterExceptionType`, `SchemaErrors`, `Sum`, `Table`, and `Together`
  (`crates/rum-stdlib/src/catalog/memory_growth_contract.rs:15-665`).
- The implementations span polynomials/calculus, structure differences,
  functional tensors, lazy tuples, control generators/exception state, and
  schema validation; schema alone retains an error vector and renders a capped
  result list (`crates/rum-stdlib/src/schema.rs:45-104`,
  `crates/rum-stdlib/src/schema.rs:516-540`).
- `Product`, `Sum`, `Table`, exception registration, schema validation, and
  tensor callbacks include evaluator or state boundaries, so scopes must follow
  their existing continuation/state shapes (`crates/rum-stdlib/src/control_flow.rs:57-99`,
  `crates/rum-stdlib/src/schema.rs:55-129`).

##### Scope

Files MAY modify:

- `crates/rum-stdlib/src/calculus.rs`, `polynomials/mod.rs`, and exact existing
  `.rs` files under `polynomials/`; `arithmetic.rs` and exact arithmetic child
  files used by `Dot`/algebra; include every touched module root
  (`docs/dev/codex/brief-414-memory-accounting-gaps.md:435-439`).
- `crates/rum-stdlib/src/structure_ops/list_plumbing.rs` and root
  `structure_ops/mod.rs`; exact `functional_ops/arrays/` files and roots; and
  `functional_ops/mod.rs` (`crates/rum-stdlib/src/structure_ops/mod.rs:33-59`,
  `crates/rum-stdlib/src/functional_ops/mod.rs:21-38`).
- `crates/rum-stdlib/src/lazy.rs`, `control_flow.rs`, `schema.rs`, any exact
  resume-frame/suspended-state child files, and root `crates/rum-stdlib/src/lib.rs`
  (`crates/rum-stdlib/src/schema.rs:45-129`).
- the exact relevant files `catalog/arithmetic.rs`, `catalog/calculus.rs`,
  `catalog/polynomials.rs`, `catalog/functional.rs`, `catalog/structure.rs`,
  `catalog/lazy.rs`, `catalog/control.rs`, `catalog/schema.rs`, and root
  `catalog/mod.rs`, only for armed handler pointers
  (`ARCHITECTURE.md:215-224`).
- target-relevant runtime kernel helpers and their module roots only where a
  charger must cross an exported evaluator/continuation boundary, including
  `kernel_api.rs` whenever its exported type changes
  (`docs/dev/codex/brief-421-incremental-charging-design.md:124-126`).
- corresponding arithmetic/polynomial/functional/structure/lazy/control/schema/
  resume tests and roots; `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs`
  only for the 16 allowlist removals; `scripts/core-language-microbench.sh`; and
  the resource guide if needed (`docs/testing-strategy.md:18-37`).

Files MUST NOT modify: either inventory/contract value, non-target families,
`rum-core`, `Cargo.lock`, or unrelated docs/tests
(`docs/dev/codex/brief-421-incremental-charging-design.md:117-126`).

##### Deliverable

- Charged coefficient/term/factor/rational-expression builders; differences,
  tensor and lazy-tuple storage; and `Product`/`Sum`/`Table` result accumulation,
  with existing integer/generated/iteration limits still enforced first where
  applicable (`docs/runtime-evaluation-audit.md:28-36`,
  `docs/guide/resource-limits-and-sandbox.md:32-72`).
- Charged schema path/error/default/result storage and exact committed charging
  for exception-type registration/session listing
  (`crates/rum-stdlib/src/schema.rs:45-104`,
  `crates/rum-stdlib/src/schema.rs:516-563`).
- Charger state in every callback/resume frame, with partial candidate release
  on error/abandon and state-retention commit at mutation boundaries
  (`crates/rum-runtime/src/runtime/suspension/core.rs:74-98`,
  `crates/rum-runtime/src/runtime/suspension/core.rs:534-594`).
- Low-ceiling tests spanning scalar/empty/wide/nested/callback/resumed/error
  cases and exact preservation of existing resource-limit precedence
  (`docs/testing-strategy.md:39-58`).
- Exact-owner falsifier green after exactly 16 removals; the `Incremental`
  rollout allowlist is now empty and coverage is 233/233
  (`crates/rum-runtime/src/runtime/metadata.rs:291-306`).

##### rum / Rust pitfalls

- BigInt/rational/string payload bytes matter in addition to node count; use the
  settled proxy helpers and fail closed on multiplication/addition overflow
  (`crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:346-405`).
- Do not weaken integer-work, generated-element, iteration, schema error-cap, or
  evaluator-pass limits to make a memory test pass
  (`crates/rum-runtime/src/runtime/model/config.rs:3-17`,
  `crates/rum-stdlib/src/schema.rs:530-540`).
- A schema/callback child charge does not cover the outer error/result container;
  preserve exact audit attribution (`crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:205-238`).
- Include changed roots and assess split-plan files
  (`docs/dev/codex/brief-421-incremental-charging-design.md:124-126`,
  `docs/code-organization-guidelines.md:64-97`).

##### Verification

Run bounded/offline formatting, focused arithmetic/polynomial/functional/
structure/lazy/control/schema/resume tests, owner falsifier, and workspace suite;
paste exact output (`docs/testing-strategy.md:18-37`). At minimum:

```text
scripts/run-bounded.py --timeout 300 -- cargo --offline fmt --check
scripts/run-bounded.py --timeout 600 -- cargo --offline test -p rum-stdlib polynomials
scripts/run-bounded.py --timeout 600 -- cargo --offline test -p rum-stdlib schema
scripts/run-bounded.py --timeout 600 -- cargo --offline test -p rum-stdlib holding_and_control
scripts/run-bounded.py --timeout 1200 -- cargo --offline test --workspace --no-fail-fast
```

Run a base/candidate A/B under `/tmp/rum-421h-*` for every table row,
`literal-integer`, arithmetic exact-neighbor rows, new exact dot/inner/
coefficient/schema/lazy-tuple rows, and same-tier neighbors; paste every named
row (`scripts/core-language-microbench.sh:456-500`,
`docs/testing-strategy.md:74-94`).

##### Hard rules

- No reclassification, git/network/vendor oracle/new `unsafe`, or writes beyond
  Scope and named `/tmp` output (`docs/dev/codex/TEMPLATE.md:125-159`).
- No weakened non-memory limit, ordinary per-node branch, full walk per item, or
  continuation collapse (`docs/dev/codex/TEMPLATE.md:65-98`).
- Stop on an overflow that does not fail closed, an uncharged target path,
  direct/resumed mismatch, or target/neighbor regression
  (`docs/dev/codex/TEMPLATE.md:96-100`).

---

#### Draft I — closure gate and stride ratification

##### Goal

Ratify that all 233 existing `Incremental` declaration rows are backed, close
the two non-`Incremental` factorial polling gaps, remove temporary rollout
scaffolding, and keep stride 64 only if the complete post-rollout invariant and
fresh gates pass. This brief credits no new declaration; it certifies the total
established by Drafts C-H (`crates/rum-runtime/src/runtime/metadata.rs:291-306`,
`crates/rum-runtime/src/runtime/arithmetic/factorial.rs:18-71`).

##### Context pointers

- The core and installed tests already compare exact sorted declaration tables
  to raw/effective specs; extend that structure rather than inventing a second
  count source (`crates/rum-runtime/src/runtime/tests/memory_growth_contract.rs:199-215`,
  `crates/rum-stdlib/src/catalog/memory_growth_contract.rs:667-749`).
- The only direct builtin callers of the strided expression check are the two
  factorial loops, and both heads remain `Preflight`
  (`crates/rum-runtime/src/runtime/arithmetic/factorial.rs:18-71`,
  `crates/rum-stdlib/src/catalog/memory_growth_contract.rs:196-197`).
- The original stride gate used the same armed
  `MemoryConstrained[Do[Length[Table[...]]...]]` workload at K=1 and K=64; use
  that exact shape for the fresh comparison
  (`docs/dev/codex/brief-232-request-memory-ceiling.md:81-106`).

##### Scope

Files MAY modify:

- `crates/rum-runtime/src/runtime/arithmetic/factorial.rs` and root
  `arithmetic/mod.rs` for direct charged BigInt growth; no catalog contract
  value changes (`crates/rum-runtime/src/runtime/arithmetic/factorial.rs:1-78`).
- `crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs`, the
  Draft-A charging module, and root `debug_tools/resources.rs` for final
  reconciliation/stride comments and removal of temporary fallback paths
  (`crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:5-188`).
- `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs`, `metadata.rs`,
  `kernel_api.rs`, `model/cache.rs`, `resolution.rs`, and their exporting module
  roots solely for an empty rollout allowlist, exact armed-handler invariant,
  and cleanup of temporary transition machinery
  (`crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:173-279`,
  `crates/rum-runtime/src/runtime/metadata.rs:331-470`).
- `crates/rum-runtime/src/runtime/tests/memory_growth_contract.rs`,
  `session_memory.rs`, `session_config.rs`, relevant resource/suspension/request
  tests and their module roots; the test portion of
  `crates/rum-stdlib/src/catalog/memory_growth_contract.rs` may change only to
  assert armed-handler completeness—the inventory rows themselves are forbidden
  (`crates/rum-stdlib/src/catalog/memory_growth_contract.rs:667-749`).
- `scripts/core-language-microbench.sh`,
  `docs/guide/resource-limits-and-sandbox.md`, and this final implementation
  brief's own appended report (`docs/testing-strategy.md:60-94`,
  `docs/dev/codex/TEMPLATE.md:55-74`).

Files MUST NOT modify: any inventory row/contract value, non-factorial builtin
implementation, `rum-core`, `Cargo.lock`, unrelated docs/tests, or `.git`
(`docs/dev/codex/brief-421-incremental-charging-design.md:117-126`).

##### Deliverable

- A mechanical invariant that every effective `Incremental` spec has an armed
  charged handler and that the sorted rollout allowlist is empty; report both
  68/165 source-table counts and the 233-record coverage arithmetic
  (`crates/rum-runtime/src/runtime/tests/memory_growth_contract.rs:199-215`,
  `crates/rum-stdlib/src/catalog/memory_growth_contract.rs:667-749`).
- Replace factorial's every-256-iteration call into the strided full walker with
  unstrided charged BigInt-capacity deltas/chunks, retaining the existing
  integer-work check and `Preflight` classifications
  (`crates/rum-runtime/src/runtime/arithmetic/factorial.rs:16-34`,
  `crates/rum-runtime/src/runtime/arithmetic/factorial.rs:52-71`).
- Final nested-evaluation, direct/resumed/abandoned, dynamic-limit,
  partial-failure, and next-request recovery tests under tight ceilings
  (`crates/rum-runtime/src/runtime/suspension/core.rs:482-594`,
  `docs/testing-strategy.md:33-58`).
- Correct the stride comment to describe reconciliation/unclassified drift and
  retain K=64 only after both the fresh armed K=1/K=64 result and the default
  microbench A/B satisfy their gates
  (`crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:5-9`,
  `docs/dev/codex/brief-232-request-memory-ceiling.md:81-106`).
- Update public modeled-memory documentation to distinguish admission of retained
  proxy growth from a hard allocator/RSS limit
  (`docs/host-embedding-and-hardening.md:112-121`,
  `docs/guide/resource-limits-and-sandbox.md:13-30`).

##### rum / Rust pitfalls

- Empty allowlists and attached armed handlers prove structure, not arithmetic;
  the owner falsifier, low-ceiling family tests, and workspace tests must all
  remain green (`docs/dev/codex/brief-414-memory-accounting-gaps.md:463-488`).
- Do not hold a BigInt temporary beyond its admitted scope or replace the
  integer-work limit with the memory limit (`crates/rum-runtime/src/runtime/arithmetic/factorial.rs:16-34`).
- A temporary K=1 build for measurement must be restored with a scoped patch,
  not git checkout/reset, and the final diff must show K=64 only if the decision
  gate passes (`docs/dev/codex/TEMPLATE.md:125-159`,
  `docs/dev/codex/brief-232-request-memory-ceiling.md:92-102`).
- Include all changed roots and assess split-plan files
  (`docs/dev/codex/brief-421-incremental-charging-design.md:124-126`,
  `docs/code-organization-guidelines.md:64-97`).

##### Verification

Run all commands bounded and offline and paste exact output; the full workspace
and armed owner-falsifier lanes are mandatory because this is the closure gate
(`docs/testing-strategy.md:1-37`, `docs/dev/codex/TEMPLATE.md:102-123`):

```text
scripts/run-bounded.py --timeout 300 -- cargo --offline fmt --check
scripts/run-bounded.py --timeout 600 -- cargo --offline test -p rum-runtime memory_growth_contract
scripts/run-bounded.py --timeout 600 -- cargo --offline test -p rum-runtime session_memory
scripts/run-bounded.py --timeout 600 -- cargo --offline test -p rum-stdlib memory_growth_contract
scripts/run-bounded.py --timeout 1200 -- cargo --offline test --workspace --no-fail-fast
```

Run a default-off base/candidate A/B under `/tmp/rum-421i-default-*` for every
existing/new target label named in Sections 5 and Drafts C-H plus same-tier
neighbors; profiling/audit variables remain off and every row is reported
(`scripts/core-language-microbench.sh:456-989`,
`docs/testing-strategy.md:74-94`). Separately build K=1 and K=64 from the same
candidate, run the exact armed workload from brief 232 interleaved best-of-2,
and paste all four raw times and the decision; K=64 requires greater than 20%
improvement and no correctness difference
(`docs/dev/codex/brief-232-request-memory-ceiling.md:92-106`,
`docs/dev/codex/TEMPLATE.md:147-150`).

##### Hard rules

- No reclassification, git/network/vendor oracle/new `unsafe`, or writes beyond
  Scope and named `/tmp` output (`docs/dev/codex/TEMPLATE.md:125-159`).
- Do not certify 233/233 unless both inventory invariants, the empty rollout
  allowlist, all owner-falsifier tests, all focused low-ceiling tests, and the
  workspace suite are green (`crates/rum-runtime/src/runtime/tests/memory_growth_contract.rs:199-215`,
  `crates/rum-stdlib/src/catalog/memory_growth_contract.rs:667-749`).
- Do not keep K=64 from historical numbers alone; do not keep K=1 if the fresh
  gate favors 64; stop on any default target/neighbor regression
  (`docs/dev/codex/brief-232-request-memory-ceiling.md:92-106`,
  `docs/dev/codex/TEMPLATE.md:143-150`).

### 9. Risks, open questions, and verification

1. **The XML depth premise is contradicted by source.** XML recursively calls
   `parse_element` from `parse_children`, but neither the parser fields nor the
   recursive signatures carry a depth or limit; I could not establish the stated
   guard (`crates/rum-stdlib/src/xml/decode.rs:35-53`,
   `crates/rum-stdlib/src/xml/decode.rs:195-303`). The director should decide a
   separate XML-depth brief; this design does not silently widen a memory brief
   to change language behavior (`docs/dev/codex/brief-421-incremental-charging-design.md:128-138`).
2. **The eight pilots and factorial expose a metadata wording tension.** Their
   rows must not be reclassified by this plan, yet the recommended safety is a
   sequence of local reservations rather than one whole-result projection; the
   current `Preflight` doc says a conservative result-size projection can run
   before construction (`crates/rum-runtime/src/runtime/metadata.rs:299-304`,
   `docs/dev/codex/brief-421-incremental-charging-design.md:117-126`). Before
   Draft B, the director must decide whether a sequence of unstrided
   before-chunk projections satisfies that wording; otherwise the eight should
   remain on the Preflight falsifier allowlist even after production hardening,
   because this brief forbids the alternative classification change
   (`crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:7-38`).
3. **Allocator capacity is not known before allocation.** The existing proxy
   charges actual `Vec`/`String` capacity, so an active builder can pre-admit a
   requested chunk and inspect/top-up actual capacity before logical retention,
   but cannot promise that the allocator never transiently reserves more bytes
   (`crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:360-400`).
   That is consistent only because the public ceiling is explicitly modeled
   memory rather than RSS; staging is mandatory for session-owned containers
   that cannot be dropped after a failed top-up
   (`docs/host-embedding-and-hardening.md:112-121`,
   `crates/rum-runtime/src/runtime/model/session.rs:29-127`).
4. **The no-cost selection seam is a measured hypothesis.** Adding an armed
   handler field and rewriting/invalidation of the builtin cache on mode changes
   appears capable of keeping ordinary cache hits unchanged, but I did not
   compile or time that design; Draft A's A/B is a stop-loss, not ceremony
   (`crates/rum-runtime/src/runtime/resolution.rs:176-188`,
   `docs/dev/codex/TEMPLATE.md:143-150`).
5. **A full handler-wide bound is not mechanically observable.** Owner-specific
   “at least one positive charge” avoids nested false discharge, while builder
   credit assertions prove only code that actually uses those builders; raw
   allocations can still evade both until review/low-ceiling tests expose them
   (`docs/dev/codex/brief-414-memory-accounting-gaps.md:463-488`). This is why the
   plan does not claim that the falsifier proves bounded spacing by itself
   (`docs/dev/codex/brief-421-incremental-charging-design.md:92-106`).
6. **Intermediate rollout is not full enforcement.** Until Draft H empties the
   165-name rollout allowlist, an armed request can still enter an old handler;
   each brief must report only its exact credited subset, and Draft I alone may
   certify all 233 (`crates/rum-runtime/src/runtime/metadata.rs:291-306`,
   `docs/dev/codex/brief-421-incremental-charging-design.md:117-126`).
7. **The memory model itself remains conservative and non-physical.** It walks
   expression payloads recursively and can count shared Arc-backed structures
   more than once; the ledger must match that settled proxy rather than describe
   the result as allocator truth (`crates/rum-core/src/expr.rs:637-715`,
   `crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:346-405`,
   `docs/host-embedding-and-hardening.md:112-121`).

**Observed verification.** My first attempts used `-- --exact` with an
unqualified test name and selected zero tests; I did not treat those runs as
evidence. I reran both filters without `--exact`, producing the two one-test
passes quoted in Section 1; their source definitions are the exact exhaustive
checks (`crates/rum-runtime/src/runtime/tests/memory_growth_contract.rs:199-215`,
`crates/rum-stdlib/src/catalog/memory_growth_contract.rs:667-749`). No workspace
suite or performance gate was run because this is a design-only brief whose only
permitted write is this appended report
(`docs/dev/codex/brief-421-incremental-charging-design.md:140-154`).
