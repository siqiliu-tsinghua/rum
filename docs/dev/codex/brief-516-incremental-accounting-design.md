# brief-516 — design: what `Incremental` actually has to be

`Scope: NONE — no source change.` Design-only. Survey, reason, return a plan
and drafted implementation briefs. Write no code.

## Goal

`d1e5de5c` made every armed checkpoint walk the session proxy, which fixed a
ceiling that a loop walked straight through. Armed sessions now pay **2.26x
to 43.34x** instructions, and `eval_memory_limit` defaults to `None`, so the
cost lands entirely on hosts that asked for safety. The user ruled
(2026-09-22) to build `Incremental` rather than leave it.

Decide **what `Incremental` is**, and return the briefs to build it.

## Why this is a design brief and not an implementation

Three things are unsettled, and each changes the shape of the work.

### 1. Which design is meant — they are not the same size

- **(a) Per-builtin charging.** What the declaration literally says:
  "retained growth must be charged as data-dependent construction proceeds"
  (`crates/rum-runtime/src/runtime/metadata.rs:346-348`). **165 builtins
  already declare `Incremental`**
  (`crates/rum-stdlib/src/catalog/memory_growth_contract.rs`), so this is a
  165-site obligation that nothing currently discharges.
- **(b) Incremental session total.** Maintain the session's proxy total as
  session state changes, so `current_memory_proxy_bytes()` becomes O(1) and
  a checkpoint costs O(expr) instead of O(session). Far fewer sites, and it
  is the thing that actually removes the 43x.

They are not alternatives in the obvious way: (b) may remove the *cost* while
leaving the 165 declarations undischarged, and (a) may discharge them without
making a checkpoint cheap. Say which the line is, whether both are needed,
and in what order. **Cost of implementation is not an argument** — this
project does not score an option down for its size — but scope still has to
be the problem's shape.

### 2. The models disagree, and this is the crux

The existing walker charges repeated shared subtrees **by occurrence**. A
counter of unique allocations does not reproduce that. So an incremental
counter is not a faster way to compute the same number — **it computes a
different number**, and the ceiling's meaning changes with it.

Settle it explicitly: does the ceiling mean "bytes this session would occupy
if nothing were shared" (today's model, conservative, occurrence-based) or
"bytes actually allocated" (physical, unique-based)? Say which the
documented contract promises today
(`docs/guide/resource-limits-and-sandbox.md`), which one an incremental
design can maintain cheaply, and what breaks for each existing test and
documented guarantee if the answer changes. **If the two models cannot be
reconciled, that is a director decision — surface it, do not pick silently.**

### 3. What it must survive

Briefs 415-416 made the session walk complete: 52 of `Session`'s 93 fields
walked, the other 41 carrying recorded exemptions, with a macro generating
both the inventory and an exhaustive `let Session { .. }` pattern so a new
field stops compiling until classified. An incremental total must **stay
consistent with that inventory or replace it** — and if it replaces it, it
must keep the property that a new field cannot be added silently.

Enumerate, with `path:line`, every place the total would have to change:
capacity growth, expression import and sharing, retention in definitions and
caches, prints, traces, reaps, suspension, abandonment, and release on every
exit including error and panic paths. A missed release is a monotonically
drifting counter, which is worse than a slow walk because it fails closed on
a session that did nothing wrong.

## What I want back

1. The answer to §1, argued: which design, both or one, and the order.
2. The answer to §2, with the model question named as a ruling if it cannot
   be settled from the existing documented contract.
3. The §3 inventory, produced rather than estimated, with the release paths
   called out separately from the charge paths.
4. **The falsifier design.** A drifting or under-counting total must be
   detectable, and the check must be one that can be watched failing. The
   obvious shape — assert the incremental total equals the walk — is a
   *good* falsifier only if the walk stays; say whether it does.
5. **The cost model**: what a checkpoint costs afterward, what the charge
   paths cost, and which of those numbers are measured, inferred, or guessed.
   The A/B this line will need is the instruction-count recipe, with both
   armed and unarmed workloads, since the unarmed path must not move at all.
6. Drafted follow-up briefs in `TEMPLATE.md` shape, as raw material.

## Settled — do not reopen

- The unstrided armed checkpoint (`d1e5de5c`) stays. This line makes it
  cheap; it does not restore sampling. **No stride, no adaptive policy** —
  brief-513 established last-sample adaptive stride has no useful factor
  bound.
- `Symbol`'s representation and lifetime-free public type are frozen (user
  ruling 2026-09-21).
- The interner quota line is **parked, not cancelled**, and is not this
  line's concern. The user ruled the snapshot format changes instead; that
  is a separate line to be discussed after this one.
- **No default-path cost.** `eval_memory_limit` defaults to `None` and the
  unarmed path must stay exactly as fast.

## Method requirements

- Read `AGENTS.md`, `TODO.md` §User rulings of 2026-09-22 and the
  memory-accounting entries, and briefs 513-515's reports.
- **Re-probe rather than inherit.** The audit's mechanism for the strided
  finding was wrong, and the director has been wrong twice in this line
  already. Verify anything you lean on, including the claims in this brief,
  and say how.
- Where you assert a cost, say whether it is measured, inferred or a guess.
- Name what would falsify your recommendation. **A refutation is an
  acceptable answer**: if an incremental total cannot be maintained without
  either changing the ceiling's meaning or risking silent drift, say so, and
  the armed cost becomes a documented property instead.

## Hard rules

- **No source change.** No `.rs`, no script, no fixture, no doc except
  appending your report to this file. Scratch under `target/brief-516/`.
- Do NOT `git commit`, branch, switch, merge, push, stash, `git checkout` or
  `git restore`, or touch git config or the index.
- **No new `unsafe`.**
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only; no oracle harnesses, no comparisons against other vendors'
  engines.
- No network. Touch nothing outside this repo. You may read everything under
  `target/brief-5*/`; overwrite nothing there.
- Bound anything long with `scripts/run-bounded.py --timeout SECONDS`, sized
  from a measured run of the same shape.
- **Stop-loss, keyed on progress.** If a probe fails twice and the second
  attempt produced no more information, stop and report what you have.
- Paste verbatim evidence; never quote a number you could not take.

## Report format

1. Which design, and why.
2. The occurrence-versus-unique model answer, or the ruling it needs.
3. The charge-and-release inventory.
4. The falsifier design.
5. The cost model, labelled measured / inferred / guessed.
6. Drafted follow-up briefs, and any director decisions you hit.

---

## Codex report — 2026-09-22

Design only. No implementation, fixture, script, or other document was changed.
The report below is appended to the original brief. Scratch evidence is under
`target/brief-516/`. Follow-up drafts are raw material for the director to
review and rewrite, as required by `TEMPLATE.md`; they are not dispatched.

Source anchors below refer to HEAD
`eb1fd7219161eff7e5a138a8d5c2667105318145`. For readable inventories, these
prefixes expand to repository paths:

- `R/` = `crates/rum-runtime/src/runtime/`
- `S/` = `crates/rum-stdlib/src/`
- `C/` = `crates/rum-core/src/`
- `H/` = `crates/rum-host/src/`

### 1. Which design, and why

**Build both: (b), an incremental occurrence-based session sum, first; (a),
charging builtin-local retained construction, second. Keep the independent
walker.** The first removes repeated traversal of unchanged session state.
The second covers retained values the session walker cannot see. Neither
substitutes for the other. Do not remove the unstrided check at any stage.

The count of 165 `Incremental` declarations is correct. The stronger claim
that no existing mechanism discharges any part of them is not: for example,
Table already projects its generated list shell in
`R/kernel_data/control_data/branching_loops/iteration/spec.rs:338`.
Nevertheless, the `Incremental` arm of the growth validator is literally
empty (`R/eval_dispatch/mod.rs:265`, `:276`). It neither checks progress nor
proves that a builtin has charged its retained builder. Table's local
`collected` values (`iteration/driver.rs:269`, `:291` in the same directory)
are absent from `Session` until publication or suspension. Walking the whole
session at each evaluation inside that loop does not count that accumulator.
The same distinction applies to NestList/FoldList histories, grouping buckets,
and StringReplace's growing output. The declarations are obligations requiring
a per-declaration disposition, not evidence of an implemented accounting system.

The proposed implementation has three separate quantities:

1. `W(x)`: today's structural expression weight, including repeated shared
   occurrences and the capacities the current expression walker counts.
2. `S`: the maintained sum of exactly the session terms in the independent
   session walker, plus explicitly classified accounting storage introduced by
   the implementation. During the first cutover, existing checks remain
   `saturating_add(S, W(expr)) > limit` and
   `saturating_add(S, projected_bytes) > limit`.
3. `R`: subsequently, the weights/reservations of live, accounted local
   builders which are not already published as session roots. Adding this is
   a documented extension of coverage, not a claim that the old walker
   already included those values. Its independent oracle is a builder-payload
   traversal, not `current_memory_proxy_bytes()`.

For (b), maintain contributions at mutation boundaries. Charge a root each
place the existing walker reaches it, even if an identical root is already in
another field. Weigh an entering/leaving expression when necessary; update
known collection-shell and capacity deltas separately. A mutation of one rule
must not re-walk every definition. A Sow must not re-walk every previous value
in the Reap. Cached weights attached to retained **occurrences/owners** can
make a later release constant work. The first implementation does not require
a byte field on every `Expr`, a global pointer memo, allocation interception,
or a change to `Symbol`.

This does not promise O(1) for every mutation. Importing an arbitrary existing
expression with no valid weight costs a traversal of that expression. Removing
one without a saved weight can also cost that traversal. The required
improvement is to stop re-traversing unrelated, unchanged session roots on
every checkpoint. Accounting must not turn an append sequence into repeated
prefix traversals. The currently unarmed-only in-place Append/Part/association
paths remain unarmed-only in this line (`R/definitions/incremental_mutation.rs:12`,
`R/kernel_data/association_lvalue.rs:223`). Enabling them while armed is a
separate optimization with an alias/invalidation proof.

Initialize the sum with one complete traversal on an unarmed-to-armed
transition. Keep it valid through nested minimum limits, suspension, resume,
and abandonment. Discard/invalidate it when leaving the armed interval; reseed
after any intervening unarmed work. A nested limit that remains armed must not
reseed. Repeated short top-level `MemoryConstrained` intervals can still pay
repeated seed costs: that must be measured separately. Unarmed `MemoryInUse[]`
and `MaxMemoryUsed[]` can continue to walk on explicit invocation. Claiming
that every invocation is O(1), while maintaining nothing during arbitrary
unarmed mutations, would be false.

**Zero default-path cost is the first feasibility gate.** A newly added
`if accounting.is_some()` in every mutator does not meet that requirement.
Prototype mode-specialized internal evaluation/mutation paths: an unaccounted
instantiation with the accounting operations erased, and an accounted one
selected at the already-existing memory-limit/request boundary. Recursive
evaluation must propagate that mode; installing a local MemoryConstrained
frame must explicitly cross it. The existing `BuiltinFn` function pointer
(`R/metadata.rs:332`) and erased continuation interfaces
(`R/suspension/core.rs:74`) are real boundaries that need a mode-aware adapter
or equivalent design. Merely making an outer function generic does not solve
them. Host mutation during a pending armed request also needs a valid entry
adapter. Do not implement hundreds of hooks before this representative
prototype has passed instruction and microbench gates.

There is no measured proof yet that this plumbing can preserve default cost.
If its unarmed instruction slope rises, if it changes sharing to save weights,
or if production can still bypass the ownership API, stop before cutover.
Correct unstrided walking remains the production behavior. That is a
falsifiable recommendation, not permission to weaken the settled rule.

### 2. The occurrence-versus-unique model

**Preserve the current structural occurrence model. It can be maintained by
incremental deltas; incremental does not imply unique-allocation counting.**
The two concepts were conflated in the premise. Sharing prevents a physical
allocation counter from reproducing `W`, but does not prevent addition and
subtraction of root-occurrence weights from reproducing it.

The normative guide says deterministic session/expression structures and
explicitly says this is neither an allocator meter nor RSS
(`docs/guide/resource-limits-and-sandbox.md:292`, `:307`). It promises a check
at every existing per-expression checkpoint while armed and every projected
precheck, with transients/omitted memory excluded (`:300`). It does not promise
unique allocated bytes. Nor does it literally promise the number of bytes a
fully unshared process would allocate. The current proxy mixes live-entry
counts, capacity counts, omitted implementation storage, and deliberate
conservative terms. Preserve that documented proxy without upgrading it to a
physical claim.

Source and fresh probe agree:

- `C/expr.rs:574` and `:638` use shared `Arc` head/argument storage; expression
  cloning shares call storage. `R/debug_tools/resources/memory.rs:391` walks
  every call occurrence, then every argument at `:399`, without an identity
  set. Symbol context/name bytes are added at each occurrence (`:363`).
- Fresh execution of the preserved unstrided binary:

  ```text
  target/brief-514/rum-final --eval 'gRoot = Range[1000]; List[ByteCount[gRoot], ByteCount[List[gRoot, gRoot]], ByteCount[List[]]]'
  List[32075, 64289, 75]
  ```

  Exit 0, empty stderr. The pair counts both retained occurrences. The extra
  64 bytes beyond `2 * 32075 + 75` are consistent with the outer argument
  vector's spare slots and the 32-byte expression size; this capacity
  explanation is inferred from the formula, not an allocator observation.
- `memory.rs:51` includes `size_of::<Session>()`; `:333` separately includes
  ResourceLimits, and `:307` conditionally adds PendingEvalRequest. The field
  inventory explicitly calls the latter conservative terms. Do not remove
  them as an accounting cleanup.
- The proxy counts encoded String capacity for valid ByteArray (`:366`),
  serialized BigInt magnitude storage (`:795`), and concrete provider/closure
  object size, not arbitrary allocations hidden inside host providers or
  closures (`:254`, `:565`). Reap bucket spare capacity/tag-position indexes
  are not all included (`:724`); frame tracing intentionally ignores String
  payloads (`R/suspension/trace.rs:355`). “Complete field inventory” does not
  imply complete physical heap accounting.

A unique-allocation model would require a new contract and ownership policy:
which session owns a cross-session shared allocation, what happens when the
host retains it, whether weak handles retain charge, what the interned global
Symbol text means, and which allocator headers/capacities count. A reachable
unique-object graph still is not process allocated bytes. Constructors have
no Session (`C/expr.rs:577`, `:641`); retrofitting a physical counter is not a
local replacement of the walker. Strong-reference memo entries also change
retention/COW; weak-reference entries can change `Arc::make_mut` behavior and
need lifetime-safe identity validation (`R/model/cache.rs:237`). Neither is
free bookkeeping.

No occurrence-versus-unique ruling is required to pursue the recommended
model: it agrees with the guide and can use the existing oracle. **Choosing
unique/physical instead would require an explicit director ruling and a new
design, rather than silently changing thresholds in these follow-ups.**

Existing contract/test migration consequences were checked as follows:

| Existing contract or test anchor | Occurrence-preserving plan | If unique/physical is chosen |
| --- | --- | --- |
| `R/tests/session_memory.rs:129` field classification and `:165` exact exception String delta | Keep; classify every new field and accounting allocation. | Field coverage remains necessary, but live-entry estimates and exact deltas need a new formula. |
| `R/tests/session_memory.rs:186` ByteArray encoded-capacity/malformed fallback | Keep exact expression formula. | Decoded-size or allocator-size counting changes this contract; cannot weaken the test to pass. |
| `S/tests/incidental_debug_tools_resources.rs:13` exact small ByteCount values, relationships for big integers/contexts/options; `:174` removed-symbol growth; `:195` peak | Keep expression values and relationships. New Session metadata can change session totals explicitly. Preserve peak observation semantics. | Shared occurrences, interned text, BigInt representation, and high-water interpretation differ; new expectations and alias tests required. |
| `S/tests/debug_tools_resources.rs:101`, `:121` repeated growth and projected StringRepeat ceilings | Must still pass; the first fails with the old stride (observed below). | Single owned-string cases may still pass; that would not prove equivalent shared-expression ceilings. |
| `R/tests/session_requests/api.rs:300`, `:333`, `:360`; `H/tests/eval_contract.rs:210` | Preserve host termination, messages, local fallback, nested minimum and recovery. | Control-flow guarantee can stay; workloads crossing a numeric limit may change, especially sharing-heavy Table. Requires documented migration, not retuned limits. |
| `R/tests/scoping_gensym_gc/suspension_frames.rs:311` large continuation root and midpoint ceiling | Keep full frame roots and the same model-derived construction of the limit. | Root sharing changes the relevant delta; a recalculated test must still catch a missing owner/root. |
| `crates/rum-host/tests/suspension_abandonment_contract.rs:275`, `:489` completed/abandoned/cancelled memory comparisons | Keep matched warm-cache comparisons and existing 1902-byte allowance; no widening to hide leaks. | New cache/allocation ownership can change deltas; must explicitly derive a new contract before changing expectations. |
| `R/tests/arithmetic.rs:10`, `:55`; `S/tests/preflight_memory.rs:12`, `:236`, `:244` | Preserve pre-allocation rejection and realistic allowed cases. | BigInt/string/container projections must be rebuilt consistently with the selected physical model. Successful old tests do not establish that consistency. |
| `S/tests/debug_tools_reports.rs:386`, `S/tests/valueq_effect_suppression.rs:233` | Preserve TestReport classification and rollback observability. Compare restored fields, not just a saved total. | Threshold and rollback byte observations may change; effect suppression must not. |
| `H/tests/session_lifecycle.rs:242`; `H/tests/capability_profiles.rs:560`, `:579` | Preserve resource configuration through reset and diagnostic capability denial. | These behavioral guarantees should remain unchanged under either model. |
| Guide `:300`, builtin reference entries ByteCount/MemoryInUse/MaxMemoryUsed/MemoryConstrained (`docs/reference/builtins.md:599`, `:3141`, `:3066`, `:3131`) | State occurrence counting explicitly; document builder coverage when added. No RSS promise. | Replace model explanations deliberately, explain threshold migration and cross-session ownership. Still cannot promise RSS from a unique graph walk. |

Two arithmetic/measurement choices must be settled before implementation:

- Saturation cannot be undone by ordinary subtraction. After an overflowing
  sum, `saturating_total - released` is not the correct remaining total.
  Use exact wider component sums with checked overflow, or an overflow state
  whose removal path reconstructs a valid total before admitting work. The
  old outer sum saturates, but several inner `.sum::<usize>()` operations
  (`memory.rs:81`, `:114`, `:241`) do not. Recommend specifying conservative
  saturation throughout, with synthetic near-maximum tests. This is an
  explicit correction at an arithmetic boundary, not proven byte-for-byte
  equivalence to current overflow/wrap/panic behavior.
- New ledger storage counts. Inline fields change `size_of::<Session>()`;
  heap sidecars need Accounted terms and their own capacity deltas. Do not
  label a heap-owning field “fixed-size exempt.” Keep that bookkeeping term
  independently computable without recursively accounting its accounting.
  Recommend accepting and documenting the measured overhead, not exempting
  it silently. Also preserve `peak_memory_proxy_bytes` updates at existing
  observations (`memory.rs:335`); changing it into every-allocation high-water
  measurement is a different semantic change.

### 3. Produced charge-and-release inventory

I enumerated the macro entries, searched their references, then followed
mutating accessors and nested owners (notably definition indexes, association
indexes, and suspension slots). The scratch outputs are
`accounted-field-references.tsv` (broad token references), `field-accesses.tsv`
(member accesses), and `incremental-declarations-resolved.tsv` (all catalog
obligations). The first two deliberately include reads and similarly named
non-Session fields; they are search aids, not a compiler-generated mutation
proof. For example, `R/debug_tools/basic.rs:102` reads the evaluation stack to
build Stack's result; `R/metadata.rs:556` mutates a BuiltinSpec, not Session
attributes. Conversely, a search for `self.definitions.insert` alone misses
`entry`, borrowed SymbolDefinition mutation, and `OnceLock::get_or_init`.
Those indirect writes are included below.

The current inventory is **52 Accounted plus 42 Exempt = 94 in test builds**.
Production has 93 fields: `gensym_gc_sweep_count` is test-only. The exhaustive
pattern is generated at `R/tests/session_memory.rs:20` and checked by the test
at `:129`; its compilation barrier operates when compiling the test module,
not every ordinary release build. Keep this barrier and make the eventual
production ownership schema equally exhaustive. A new field must fail the
required gate until classified. A field table alone cannot establish that
all mutations are accounted; the mutation API must also prevent unrestricted
mutable access to classified owners.

The following table is transcribed from the actual macro, not estimated from
field names. Each anchor is in `R/tests/session_memory.rs`.

| Accounted field | Source line | Existing contribution |
| --- | --- | --- |
| `abandonment_worklist` | `:33` | suspended owners plus Vec spare capacity |
| `association_lookup_cache` | `:38` | AssociationLookupCache::memory_proxy_bytes |
| `attribute_set_cache` | `:40` | hash-table capacity and live Symbol text |
| `attributes` | `:41` | live attribute-key Symbol text |
| `builtin_dispatch_spec_cache` | `:42` | hash-table capacity and live Symbol text |
| `builtins` | `:44` | live BuiltinSpecMap entry storage |
| `call_stability_cache` | `:45` | hash-table capacity and retained SharedExpr/SharedArgs values |
| `context_path` | `:48` | String capacities and Vec capacity |
| `context_stack` | `:49` | ContextFrame contents and Vec spare capacity |
| `current_context` | `:50` | String capacity |
| `declared_stubs` | `:55` | live Symbol values and text |
| `definitions` | `:56` | definition keys, values, rules, guards, and dispatch indexes |
| `disabled_message_selectors` | `:57` | selector strings and Vec spare capacity |
| `dynamic_own_values` | `:58` | DynamicOwnValueFrame symbols, expressions, and Vec spare capacity |
| `entropy_source` | `:59` | Arc allocation header and concrete EntropySource object |
| `evaluation_stack` | `:64` | EvaluationStackEntry expressions and Vec spare capacity |
| `exception_types` | `:65` | BTreeMap entries and owned String capacities |
| `file_io` | `:66` | Arc allocation header and concrete FileIo object |
| `gc_candidate_gensyms` | `:67` | hash-table capacity and live Symbol text |
| `handled_non_loader_eval_errors` | `:70` | RuntimeMessage strings and Vec spare capacity |
| `installed_system_down_values` | `:73` | hash-table storage, Symbol text, rule expressions, and nested Vec spare capacity |
| `installed_system_own_values` | `:74` | hash-table capacity and live Symbol text |
| `last_message_list` | `:78` | RuntimeMessage strings and Vec spare capacity |
| `load_stack` | `:79` | PathBuf payloads and Vec capacity |
| `loaded_packages` | `:80` | String capacities and Vec capacity |
| `message_definitions` | `:82` | message keys, tags, and definition expressions |
| `message_symbol_counts` | `:83` | hash-table capacity and live Symbol text |
| `messages` | `:84` | RuntimeMessage strings and Vec spare capacity |
| `module_activations` | `:85` | ModuleActivation gensym vectors and outer Vec spare capacity |
| `monitor_scopes` | `:87` | MonitorScope expressions, display String, and Vec spare capacity |
| `once_values` | `:94` | owned String keys and Expr values |
| `option_value_scopes` | `:95` | OptionValueScope symbols, options, and Vec spare capacity |
| `options` | `:96` | Symbol keys and SymbolOptions entries |
| `package_search_paths` | `:97` | PathBuf payloads and Vec capacity |
| `pending_eval_request` | `:100` | existing conditional PendingEvalRequest conservative term |
| `pending_input_request` | `:101` | RuntimeInputRequest prompt capacity |
| `pending_suspension` | `:102` | owned SuspendedEvaluation stack, frames, and roots |
| `print_sink` | `:103` | concrete boxed FnMut object |
| `prints` | `:104` | String capacities and Vec capacity |
| `queued_input_responses` | `:106` | String capacities and VecDeque capacity |
| `reap_scopes` | `:107` | ReapScope patterns, handlers, buckets, and Vec spare capacity |
| `removed_symbols` | `:108` | live Symbol values and text |
| `request_interrupt_control` | `:109` | Arc allocation header and fixed RequestInterruptState object |
| `resource_limits` | `:110` | existing ResourceLimits conservative term |
| `resuming_suspension` | `:112` | owned SuspendedEvaluation stack, frames, and roots |
| `sealed_units` | `:113` | SourceUnit strings, expressions, symbol vectors, private index, and outer Vec spare capacity |
| `stable_symbol_cache` | `:117` | hash-table capacity and live Symbol text |
| `symbol_call_effect_cache` | `:119` | hash-table capacity and live key Symbol text |
| `symbol_generations` | `:120` | owned context/name String capacities and generation entries |
| `syntax_information` | `:121` | Symbol keys and Expr values |
| `trace_event_depths` | `:122` | Vec capacity |
| `trace_events` | `:123` | TraceEvent strings, expressions, and Vec spare capacity |

The 42 recorded exemptions (same source; reasons remain authoritative there)
are all fixed-size inline values, or accepted-request wrappers whose shared
control allocation is charged through `request_interrupt_control`:

```text
32: abandonment_active — fixed-size value already covered by size_of::<Session>()
34: abort_protect_depth — fixed-size value already covered by size_of::<Session>()
35: abort_protect_pending — fixed-size value already covered by size_of::<Session>()
36: active_accepted_request — fixed-size wrapper already covered by size_of::<Session>(); Arc contents are already counted through request_interrupt_control
37: asserts_enabled — fixed-size value already covered by size_of::<Session>()
39: attribute_revision — fixed-size value already covered by size_of::<Session>()
43: builtin_memory_projection_count — fixed-size value already covered by size_of::<Session>()
46: call_stability_generation — fixed-size value already covered by size_of::<Session>()
47: capability_policy — fixed-size value already covered by size_of::<Session>()
51: current_memory_limit — fixed-size value already covered by size_of::<Session>()
52: current_message_list_checkpoint — fixed-size value already covered by size_of::<Session>()
53: current_message_list_dirty — fixed-size value already covered by size_of::<Session>()
54: current_time_deadline — fixed-size value already covered by size_of::<Session>()
60: eval_deadline — fixed-size value already covered by size_of::<Session>()
61: eval_timeout — fixed-size value already covered by size_of::<Session>()
62: evaluation_depth — fixed-size value already covered by size_of::<Session>()
63: evaluation_epoch — fixed-size value already covered by size_of::<Session>()
68: gensym_gc_sweep_count — fixed-size test-only value already covered by size_of::<Session>()
69: gensym_gc_watermark — fixed-size value already covered by size_of::<Session>()
71: has_up_value_definitions — fixed-size value already covered by size_of::<Session>()
72: in_gensym_sweep — fixed-size value already covered by size_of::<Session>()
75: interrupt_armed — fixed-size value already covered by size_of::<Session>()
76: interrupt_poll_counter — fixed-size value already covered by size_of::<Session>()
77: iteration_product_budget — fixed-size value already covered by size_of::<Session>()
81: match_depth — fixed-size value already covered by size_of::<Session>()
86: monitor_callbacks_enabled — fixed-size value already covered by size_of::<Session>()
88: native_stack_base — fixed-size value already covered by size_of::<Session>()
89: native_stack_next_check_depth — fixed-size value already covered by size_of::<Session>()
90: next_generated_symbol_id — fixed-size value already covered by size_of::<Session>()
91: next_request_generation — fixed-size value already covered by size_of::<Session>()
92: next_sealed_symbol_id — fixed-size value already covered by size_of::<Session>()
93: next_source_unit_epoch — fixed-size value already covered by size_of::<Session>()
98: peak_memory_proxy_bytes — fixed-size value already covered by size_of::<Session>()
99: pending_accepted_request — fixed-size wrapper already covered by size_of::<Session>(); Arc contents are already counted through request_interrupt_control
105: prng_state — fixed-size value already covered by size_of::<Session>()
111: restoring_snapshot — fixed-size value already covered by size_of::<Session>()
114: stability_dirty_bloom — fixed-size two-u128 value already covered by size_of::<Session>()
115: stability_native_stack_poll_counter — fixed-size value already covered by size_of::<Session>()
116: stability_rewritten_dirty_bloom — fixed-size two-u128 value already covered by size_of::<Session>()
118: symbol_cache_generation — fixed-size value already covered by size_of::<Session>()
124: tracing_enabled — fixed-size value already covered by size_of::<Session>()
125: value_q_effect_suppression_active — fixed-size value already covered by size_of::<Session>()
```

#### 3.1 Charging, replacement, and capacity changes

Each replacement is an old-contribution removal plus a new-contribution
addition, performed before the next guest reentry/checkpoint. The anchors
below are existing writes/owner boundaries, not proposed line numbers. A
transaction may not hold `&mut Session` across guest evaluation. Compute or
reserve the incoming weight, complete the owner mutation, reconcile actual
capacity, then permit reentry. A failed reserve leaves the old contribution
intact. Capacity is the **actual** resulting capacity, not the requested one.

| Owner / charge obligation | Existing write or ownership anchors |
| --- | --- |
| Session seed, fixed inline size, initial builtin installation | `R/session/session_state.rs:577` initializes the fields; `R/session/session_api.rs:98` applies config, `:271` registers a builtin, `:279` inserts it, `:283` installs attributes. Seed after existing unarmed construction/installation when a limit becomes active, not on every unarmed constructor. `R/session/session_helpers.rs:30` starts the request budget; `R/debug_tools/resources/constraints.rs:113` enters a local limit. |
| Host expression import, ordinary evaluated roots, shared head/args | `C/expr.rs:577`, `:641`, `:820`, `:824` construct shared calls without a Session; `:583`, `:647` can unwrap/clone; `:660`, `:664` permit mutable/COW args. These do not charge S on allocation. Charge when a value enters a modeled owner; `R/eval_core/mod.rs:46` retains evaluation-stack entries, `:59` checks returned expressions; `R/resolution.rs:321`, `:400` check evaluation passes. W(expr) is still separately counted at those checks even if the same expression occurs in S. |
| Builtin registration and installed-definition markers | `R/session/session_api.rs:293`, `:295`, `:304` store system own/down values and retained rule pairs. Builtin map uses live entries, installed markers use hash capacity and symbol text, installed down-values also use rule roots and nested Vec spare capacity (`memory.rs:53`, `:116`, `:482`). |
| Own/down/sub/up definitions and guards | `R/definitions/definitions.rs:478`, `:482`; `R/definitions/definition_lhs_preparation/prepared_store.rs:263`, `:268`, `:293`, `:330`, `:357`, `:382`, `:504`; `R/definitions/definition_updates.rs:544`, `:566`, `:601`, `:624`; `R/model/definitions.rs:105`. Include replacement of a matching rule as well as append, and the independently retained installed marker/rule occurrences. |
| Part, AppendTo, and association lvalues | `R/definitions/part_assignment.rs:149`, `:150`, `:187`, `:204`, `:210`, `:213`; `R/definitions/incremental_append.rs:53`, `:66`, `:67`, `:70`; `R/kernel_data/association_data.rs:869`, `:912`; `R/kernel_data/association_lvalue.rs:90`, `:102`, `:103`. Preserve current armed eligibility guards; COW/new-vector capacity can differ from the replaced vector. |
| Value-table replacement and Unique registration | `R/definitions/value_table_assignment.rs:248`, `:276`, `:283`; `R/scoping/unique.rs:173`. An empty new definition still adds its currently modeled key/structure terms. |
| Definition dispatch metadata, including read-side allocation | `R/model/definitions/down_value_dispatch.rs:74`, `:128`, `:150`, `:231`, `:248`, `:345`, `:359`, `:405`, `:428`, `:464`; `R/model/definitions/tagged_value_store.rs:52`, `:75`, `:98`, `:129`, `:160`, `:169`, `:188`, `:202`, `:228`, `:280`. `OnceLock::get_or_init` can allocate through a shared reference; route its successful installation through the accounted owner too. The current oracle includes the dispatch indexes' own formulas (`down_value_dispatch.rs:420`, `:522`; `tagged_value_store.rs:220`, `:289`), not every compiler/matcher allocation. |
| Attribute map and dispatch/attribute caches | `R/resolution.rs:95`, `:190`, `:194`, `:208`, `:210`, `:216`; `R/session/session_api.rs:280`. Attributes count live Symbol key text; cache maps count capacity plus live key text. Replacing an attribute value has no extra dynamic term unless key/capacity changes. |
| Stable-symbol and symbol-call-effect caches | `R/eval_core/stability.rs:360`, `:367`, `:508`, `:515`; `R/evaluation_cache/stability.rs:113`, `:114`, `:118`, `:232`, `:242`, `:243`. Count insertions, capacity growth, per-symbol invalidation, and generation-wide clears. |
| Call-stability cache | `R/evaluation_cache/stability.rs:343`, `:399`; `R/evaluation_cache/mutation_proofs.rs:39`, `:62`, `:84`, `:108`, `:120`. Keys retain head/args occurrences; the map shell and the SharedArgs spare slots are additional terms (`R/debug_tools/resources/memory.rs:453`, `:471`). A stable hit can be weight-neutral; remove/reinsert must not leak charges. |
| Association lookup indexes | Callers `R/kernel_data/association_data.rs:438`, `:447`, `:471`, `:488`, `:497`; write transfer `R/kernel_data/association_lvalue.rs:137`, `:186`, `:196`. Owner `R/model/cache.rs:134`, `:169`, `:209`, `:243`, `:274`, `:300`, `:344`; inner index construction `:399`, append `:443`. Account entries Vec capacity, buckets HashMap capacity, collision Vec capacity (`:332`, `:458`). Weak args are not an extra expression root here. Probe can evict stale entries even on a read. |
| Options and syntax metadata | `R/options/core.rs:36`, `:38`; `R/namespace_tools/syntax.rs:38`, `:40`. Options include per-option expression roots and entry spare capacity (`memory.rs:663`); syntax metadata adds a root per value. |
| Message definitions and symbol counts | `R/messages/message_catalog.rs:89`, `:91` inserts/replaces immediate/delayed definitions; `R/resolution.rs:160`, `:163`, `:169`, `:174` adjusts/rebuilds the derived count map. Count tag/key text and expression occurrences separately from count-map capacity. |
| Stubs, removed identity state, generations, Once values | `R/package_tools.rs:70`, `:75`, `:141`; `R/package_tools/resume_frames.rs:21`; `R/definitions/definition_clear.rs:207`, `:213`, `:228`. Once replaces String-key/Expr contributions. Removed/generation bookkeeping can grow when the user appears to be deleting symbols. |
| Current context, context path, package search paths, loaded packages | `R/session/session_api.rs:553`, `:561`, `:652`, `:656`, `:723`; `R/definitions/definitions.rs:258`, `:275`, `:292`; `R/builtins_context_loader.rs:349`, `:442`. The guide's names are not the storage formula: String uses capacity; PathBuf uses the current path-length proxy plus Vec capacity (`memory.rs:890`, `:902`). |
| Context/package frames | `R/session/session_api.rs:761`, `:767`, `:768`, `:789`, `:795`, `:796`; loader restoration `R/builtins_context_loader.rs:390`. Charge saved frame contents and current context/path independently while both are roots. |
| Load stack | `R/session/session_eval.rs:377`. Charge pushed PathBuf and capacity delta, including error-path restoration. |
| Providers, print sink, request control, limits | `R/session/session_api.rs:572`, `:584`, `:145`, `:151`, `:200`; `R/model/session.rs:140`; sink temporarily taken/restored at `R/messages/builtins_messages_inspect.rs:95`, `:109`. Provider concrete object/header size and closure concrete size are the existing terms; changing internals of a provider does not automatically enter today's proxy. Replacing request control must not charge the same Arc through accepted-ticket wrappers. ResourceLimits is a constant-size conservative term. |
| Exception declarations | `R/kernel_data/control_data/exceptions.rs:79`: new/replaced name and optional parent String capacities plus live entry shell, not BTree allocator capacity. |
| Dynamic own-value frames | `R/eval_core/symbol.rs:161`, `:167`: root symbol and expression plus outer spare capacity. Iteration bindings can use this path; it is not limited to ordinary OwnValue reads. |
| Module activations and GC candidates | `R/scoping/lexical/module.rs:60`, `:301`, `:336`; `R/scoping/gensym_gc.rs:111`. Module activation symbols have a nested Vec contribution; candidate hash set has capacity/text terms. Generated Symbol intern allocations remain outside this line. |
| Block and iterator-local symbol state | `R/scoping/lexical/block.rs:63`, `:169`, `:176`, `:179`, `:180`, `:189`, `:299`, `:340`, `:363`; `R/kernel_data/control_data/branching_loops/iteration/driver.rs:470`, `:478`, `:481`, `:482`, `:485`. Saved local snapshots are not S roots merely because they exist in a Rust local. They become counted roots if retained in a traced suspension frame. Restoration adds their actual restored contributions, including changed clone capacities/index state. |
| OptionValue scopes | `R/options/patterns.rs:20`. Include symbol, options, nested entry spare capacity, and outer scope Vec spare capacity; restoration and error paths listed below. |
| Sealed units | `R/scoping/packages.rs:468`, `:470`. Existing SourceUnit formula (`memory.rs:536`) includes context text, wrapper root, public/package symbol vectors, and private-index contents. Replacement has both release and charge. |
| Reap scopes and buckets | `R/debug_tools/reap.rs:47`, `:63`, `:336`, `:343`, `:347`, `:355`. Charge individual sown occurrences and bucket tag/handler/pattern terms instead of recomputing the whole scope after each Sow. Preserve current omissions: tag-position map mutation (`:338`, `:353`, `:361`) is not a term in the old oracle. Local rendering after pop (`:225`, `:227`, `:236`, `:255`, `:271`, `:279`) belongs to builder accounting. |
| Monitor scopes and retained displays | `R/debug_tools/monitor.rs:43`, `:144`; display String replacement can grow or shrink. Callback expressions are roots; printing at `:141` follows normal sink/buffer ownership. |
| Disabled-message selectors | `R/debug_tools/message_switch.rs:50`, `:55`: insertion/retention changes owned selector text; the Vec shell can remain after On removes selectors. |
| Message/error buffers | `R/session/session_helpers.rs:276`, `:294`, `:412`, `:414`; `R/builtins_context_loader.rs:284`. Each RuntimeMessage owns several Strings; count capacity rather than string length. Error suppression can remove and reappend (`session_helpers.rs:599`, `:603`). |
| Print buffer versus callback | `R/session/session_helpers.rs:303`, `:306`; `R/builtins_context_loader.rs:285`. A delivered callback string is not also retained in `prints`. A closure can retain arbitrary outside memory beyond the documented concrete-object proxy. |
| Last message list | `R/session/session_eval.rs:256`, `:269`, `:344`, `:356`; `R/session/session_requests.rs:251`, `:349`, `:461`, `:482`, `:567`, `:590`, `:613`; `R/debug_tools/test_report/evaluation.rs:291`, `:307`. Cloning messages creates another modeled occurrence; replacement drops the old list but may alter capacity. |
| Trace events and depth buffer | `R/definitions/application/mod.rs:94`, `:95`. Events include expr and String terms (`memory.rs:774`); depth storage uses capacity. A trace event referring to an existing definition still counts its own root occurrences. |
| Queued responses, input prompt, pending request | `R/session/session_api.rs:267`, `:268`; `R/session/session_requests.rs:234`, `:275`, `:299`, `:329`; `R/suspension/core.rs:255`, `:261`, `:510`, `:644`. Queue capacity and response String capacities differ from RuntimeInputRequest's prompt term. Prompt cloning during nested suspension adds an occurrence. |
| Pending/resuming suspension and abandonment worklist | `R/suspension/core.rs:227`, `:237`, `:241`, `:245`, `:250`, `:436`, `:451`, `:458`, `:463`, `:492`, `:544`, `:569`, `:575`, `:635`, `:640`, `:643`, `:644`, `:645`. See the transfer rules and separate release inventory below. |
| Bulk restore | `R/session/session_helpers.rs:127` replaces attributes/options/syntax/stubs/removed/generations/Once/GC candidates/context/path/stack/search paths/packages/definitions/installed markers/message definitions/exceptions/sealed units/trace/depth/selectors/messages/last-message list and rebuilds message counts. `:188` also restores option/module/load/evaluation scopes, print/error buffers, Reap/Monitor, queue/prompt/request state. Treat these as explicit multi-owner transactions; restoring one saved scalar is insufficient. |

Collection rules shared by these sites:

- `R/debug_tools/resources/memory.rs:883`, `:890`, `:902`, `:914` define Vec
  spare, String Vec, PathBuf Vec, and String VecDeque terms. A pop/truncate
  usually releases payload, not the reserved shell. `mem::take` generally
  replaces the shell with capacity zero. `split_off`, clone, COW, and
  replacement can leave two different capacities. Measure each side.
- Hash-capacity terms occur in `memory.rs:407`, `:419`, `:433`, `:453`,
  `:482`, plus association/dispatch helpers. Clear/remove usually retains
  table capacity. Several other maps deliberately use only live-entry
  contributions; do not charge their spare allocation in the compatibility
  ledger merely because it exists physically.
- Nested dispatch Vecs, expression args, ContextFrame strings/vectors,
  SymbolOptions entries, SourceUnit private data, and suspension queues need
  their own deltas. A field-level before/after full traversal is an acceptable
  cold bulk-replacement fallback, not a fast implementation for appending
  one child to a hot owner.
- No weight cache may outlive the immutable data/version it describes. Do not
  retain extra strong/weak Expr handles merely to key accounting. Importing
  the same shared value into two owners adds twice; releasing one subtracts
  once. Mutation through an alias is either prohibited while accounted or
  routed through an API that updates every affected owner; never silently
  assume Arc identity implies an immutable weight.

#### 3.2 Release, rollback, transfer, error, and panic inventory

These are distinct from charges. “The container is dropped eventually” is not
an implementation of release for a session that must keep evaluating.

| Release/exit | Existing path and required accounting action |
| --- | --- |
| Normal expression return and ordinary EvalError | `R/eval_core/mod.rs:52`, `:55`, `:59`: evaluate, pop stack, then check result. Remove the popped root while preserving stack capacity. Ordinary errors pass through the pop; a panic can bypass it. |
| Definition overwrite/unset and empty pruning | Old own/guard/rule value release at the stores in 3.1; `R/model/definitions.rs:121`; `R/definitions/definition_lhs_preparation/prepared_store.rs:420`, `:425`, `:434`, `:442`, `:463`, `:488`; matching rules removed by retain at `:449`, `:470`, `:495`, then indexes rebuilt. `R/definitions/definition_clear.rs:120`, `:142`, `:156`, `:234`, `:239`, `:245`, `:247`; `value_table_assignment.rs:276`. Remove old index contributions when rebuilding, including lazy OnceLock caches. |
| Clear/ClearAll/Remove and generated-symbol reclaim | `R/definitions/definition_clear.rs:108`, `:113`, `:116`, `:163`, `:169`, `:174`, `:178`, `:207`; `R/eval_core/symbol.rs:204`, `:209`, `:216`, `:219`, `:220`, `:229`, `:231`; `R/scoping/gensym_gc.rs:82`, `:83`, `:92`, `:100`. Removing payload may also add removed-symbol/generation records; account both. |
| Cache invalidation and bounded eviction | `R/resolution.rs:216`; `R/session/session_api.rs:280`; `R/eval_core/stability.rs:360`, `:508`; `R/evaluation_cache/stability.rs:113`, `:114`, `:118`, `:242`, `:243`, `:244`, `:399`; `R/evaluation_cache/mutation_proofs.rs:39`, `:62`, `:108`, `:120`. Association stale/bounded evictions at `R/model/cache.rs:224`, `:260`, `:323`, `:351`. Subtract roots/live entries/index allocation; retain whatever outer capacity remains. |
| Definition index invalidation | `R/model/definitions/down_value_dispatch.rs:152`, `:384`, `:394`, `:405`; `R/model/definitions/tagged_value_store.rs:124`, `:155`, `:180`, `:184`. Their clones can omit lazy indexes (`down_value_dispatch.rs:47`, `tagged_value_store.rs:16`), so clone/restore is not necessarily byte-neutral. |
| Dynamic frames and iteration cleanup | `R/eval_core/symbol.rs:172`, `:177`; `R/kernel_data/control_data/branching_loops/iteration/driver.rs:440`, `:441`. Truncate actual frames; do not subtract outer retained capacity. |
| Block normal/error restoration | `R/scoping/lexical/block.rs:227`, `:230`, `:238`, `:242`, `:248`, `:251`, `:257`, `:260`, `:265`, `:270`; single-symbol path `:435`, `:438`, `:451`, `:454`, `:462`, `:465`, `:471`, `:474`, `:484`, `:487`. Release temporary live bindings and add saved definitions/metadata; duplicate roots in a still-retained continuation remain charged until the frame leaves. |
| Iterator symbol restoration | `R/kernel_data/control_data/branching_loops/iteration/driver.rs:509`, `:512`, `:525`, `:528`, `:536`, `:539`, `:545`, `:548`, `:553`, `:558`. Restore all definition/attribute/option/syntax/message owners together before reentry. |
| Module exit and abandonment | `R/scoping/lexical/module.rs:330`, `:340`, `:345`; `R/suspension/core.rs:300`. Truncate activation, retain legitimate reachable gensym definitions/candidates, then apply the GC deltas. A local's scope ending is not proof that its escaped value is gone. |
| Option scope normal/error/resume/abandon | `R/options/patterns.rs:24`, `:32`, `:179`, `:191`; `R/suspension/core.rs:324`. Each successful push gets exactly one pop across these alternatives. |
| Reap and Monitor normal/error/abandon | `R/debug_tools/reap.rs:89`; `R/debug_tools/monitor.rs:83`, `:149`; `R/suspension/core.rs:331`, `:338`. Popped Reap buckets can move into a local output builder, so release S and establish R for the surviving occurrence without an accounting gap across evaluation. |
| Package/context unwind | `R/session/session_api.rs:773`, `:776`, `:781`, `:782`, `:802`, `:805`, `:809`, `:810`; `R/builtins_context_loader.rs:390`, `:396`, `:397`, `:400`; `R/session/session_eval.rs:379`. A wrong-kind frame is pushed back; a failed package truncates loaded packages. Compute actual before/after capacities rather than assuming every pop discards allocation. |
| Sealed-unit failure/replacement | `R/scoping/packages.rs:468`, `:491`, `:500`; `R/suspension/core.rs:420`. Replaced unit and failed minted definitions are released; bulk loader restoration uses its full multi-owner delta. |
| Messages/errors consumed or suppressed | `R/kernel_data/control_data/confirmation.rs:84`; `R/kernel_data/control_data/check_quiet.rs:106`, `:150`, `:154`; `R/session/session_helpers.rs:599`, `:603`; `R/debug_tools/evaluation_data.rs:74`, `:83`; `R/debug_tools/test_report/evaluation.rs:289`, `:292`, `:308`. Trace/print/last-message copies may survive even after the original message is consumed. |
| Trace and output extraction | `R/debug_tools/trace.rs:225`, `:226`; `R/session/session_api.rs:238`, `:239`, `:247`, `:255`; `R/debug_tools/evaluation_data.rs:75`; `R/debug_tools/test_report/evaluation.rs:290`. `take` releases the source allocation from S; truncate does not. Ownership transferred to a host response leaves S, even if the host retains the returned values. |
| Request completion, error, response split | `R/session/session_eval.rs:254`, `:255`, `:257`, `:270`; `R/session/session_requests.rs:248`, `:249`, `:252`, `:254`, `:346`, `:347`, `:350`, `:458`, `:462`, `:483`, `:610`, `:611`, `:614`, `:616`. Account split capacity, new last-message clone, truncation and pending-request removal separately. Cancellation completion also replaces last-message state (`:567`, `:590`). |
| Queue consumption and prompt extraction | `R/builtins_input.rs:236` pops a queued String; `R/session/session_api.rs:263`, `:268` takes/clears prompt; `R/suspension/core.rs:265`, `:270`, `:479`. Pop_front generally retains deque capacity. |
| Suspension resume and completion | `R/suspension/core.rs:469`, `:472`, `:479`, `:492`, `:497`, `:504`, `:515`, `:519`, `:527`, `:614`, `:619`, `:620`. Transfer pending to resuming; take/replace active stack; restore caller stack. Keep the active continuation counted during its callback; pop/drop it only after callback return, as current code does. Both Ok and Err finish paths release the owner. |
| Nested suspension suffix transfer | `R/suspension/core.rs:625`, `:635`, `:640`, `:643`, `:645`. The original suffix, new nested owner, and next frame have distinct lifetimes. Move charges, do not release/recreate live suffixes by rewalking them after every resume. Account destination queue capacity changes and prompt clone. |
| Abandon and cleanup that suspends again | `R/suspension/core.rs:534`, `:539`, `:544`, `:550`, `:553`, `:555`, `:558`, `:559`, `:565`, `:568`, `:569`, `:575`, `:577`, `:580`, `:587`, `:591`. Worklist ownership survives guest cleanup and possible nested suspension. Finishing one owner does not release all worklist owners. Stack restoration and discarded nested input have explicit deltas. |
| Per-frame abandonment actions | `R/suspension/core.rs:282`, `:290` Block; `:300` Module; `:304` loop; `:313` dynamic frames; `:324` options; `:331` Reap; `:338` Monitor; `:347` memory limit; `:351` trace; `:369`, `:370`, `:372`, `:373`, `:376` evaluation-data cleanup; `:385`, `:387` messages/errors; `:397`, `:398`, `:400` stream cleanup; `:420` package rollback. These reuse the owner APIs, rather than implementing a second set of arithmetic. |
| Limit restoration and local fallback | `R/debug_tools/resources/constraints.rs:254`, `:258`, `:262`, `:424`, `:429`. Restore prior limit on normal/error/abandon paths; nested armed limits share the ledger. A fallback can evaluate further work, so ledger validity must already reflect the completed unwind. Preserve current local-vs-request control flow; do not change message behavior as an incidental accounting fix. |
| ValueQ/effect rollback and loader rollback | `R/session/session_helpers.rs:127`, `:188`, `:214` through `:234`; `R/messages/builtins_messages_inspect.rs:95`, `:109`. Abandon a trial suspension before restoration. Restore actual contributions, not the pre-trial scalar: caches may have warmed/cleared and cloned containers may have different capacity. Sink transfer is a separate term. |
| Panic, poison, reset, Session drop | `H/session.rs:695`, `:700`, `:711`, `:712` catches panic and poisons; it does not restore every partially changed runtime field. `H/session.rs:175` replaces the inner Session on reset. An unwind guard must mark the ledger invalid before it can be reused; the next allowed runtime entry rebuilds or reset discards it. RAII builder tokens release once on unwind, without running guest cleanup. Retained evaluation-stack/state after a caught panic is real state, not a debit to subtract speculatively. Plain Session users can catch a panic outside HostSession too. Dropping the Session drops its owners and ledger together; no cross-session/global decrement may run against a new generation. |

Suspension has its own shell accounting. `R/suspension/core.rs:142`, `:146`,
`:154`, `:160`, `:169` mutate the continuation queue; `:179` counts its
capacity and `:184` its frame allocations. `memory.rs:840` adds active stack,
traceable roots, and suspension shell. Active-stack spare capacity is not
counted there the same way it is in Session's evaluation stack. Thus a move
between those owners need not be numerically zero even if every Expr is
shared. `R/suspension/trace.rs:23`, `:35`, `:41`, `:70`, `:106`, `:160`
defines root occurrence traversal; weak roots (`:49`) and String payloads
(`:355`) have different policy. A generic “charge frame sizeof” is insufficient.
The immutable-continuation contract (`R/suspension/core.rs:74`) must also
constrain custom frames: a frame with unreported interior-mutated roots cannot
safely use a cached weight. Require immutable reported payloads or an explicit
accounted update operation; do not silently trust arbitrary mutable captures.

#### 3.3 The 165 builtin obligations and local builders

The registration census below resolves every declaration to its catalog and
implementation entry point. A delegating runtime `builtin_*` adapter was
followed to its Session method; stdlib wrappers forwarding into kernel data show both
anchors. `$` variables use the symbol evaluator. This is why declaring the
obligation at ordinary builtin dispatch alone is insufficient. The final
scratch census is `incremental-entrypoints.tsv`. The earlier
`incremental-declarations.tsv` matched only 99 constructor shapes; it was not
used as the final inventory. `incremental-declarations-resolved.tsv` fixed
catalog coverage but initially labelled bare runtime adapters as having no
handler; the final entry-point census resolves those too. All intermediate
scratch evidence was retained.

Each row needs a final, test-backed disposition: accounted mutation API,
accounted builder, already sufficient projected construction with a stated
proof, or justified corrected classification. Merely reclassifying all 165,
or asserting that all call the evaluator, does not discharge the obligation.
The entry-point table is an exhaustive declaration/work-routing inventory,
not a claim that every implementation branch has already been instrumented.

Concrete builder mutation/release boundaries found in this survey:

| Builder family | Accumulation/publication anchors and required ownership |
| --- | --- |
| Table, Sum, Product | `R/kernel_data/control_data/branching_loops/iteration/driver.rs:244`, `:254`, `:260`, `:269`, `:291`, `:295`, `:303`, `:317`, `:333`, `:344`, `:357`; aggregate entry points in the table below. List capacity is projected, but data-dependent child weights still need cumulative charge. Final Expr publication, error/drop, frame freeze, and persistent-item append all require transfers. |
| Nest/FixedPoint histories and Fold variants | `S/functional_ops/iteration.rs:269`, `:277`, `:324`, `:338`, `:450`, `:465`, `:467`, `:617`; `S/functional_ops/sequence/fold/plain.rs:63`, `while_loop.rs:70`, `:278`, `sequence_fold.rs:85`. Count previous states and cycle/seen retention separately where both are live; on suspension transfer every retained history occurrence. |
| Map/Apply/traversal and selected outputs | `S/functional_ops/apply_map/traversal.rs:97`, `:116`, `:132` has work/completed/value accumulators. Recursive and resumable variants in the same owning module need the same protocol, including errors before finalization. Local work stacks are not made visible by a session-only sum. |
| Association/group/merge outputs | `S/associations/grouping.rs:30`, `:45`, `:60`, `:95`, `:143`; `S/associations/aggregate.rs:196`, `:289`, `:418`; `S/associations/resumable/map.rs:230`. Charge per-bucket additions and outer shells; evaluate callbacks outside owner borrows; move counters with suspended builders. |
| String replacement | `S/strings/patterns/search.rs:198`, `S/strings/patterns/search/rules.rs:447`, `S/strings/patterns/search/guarded/replace.rs:523`. String capacity grows even when replacement evaluation returns a small value. Reserve known byte growth before extending; reconcile capacity and release on error/drop. |
| Reap/trace/report/Stack/namespace rendering | Reap rendering anchors in 3.1; Stack `R/debug_tools/basic.rs:101`; `$` variable collects at `R/eval_core/symbol.rs:14`, `:23`, `:38`, `:48`; full entry points below. Removing the session source does not remove its surviving local output, and rendering retained definitions can duplicate occurrence weight. |
| Rewrite, structure, ordering, calculus, polynomials, schema, parser, lazy | All their entry points are enumerated below. Require branch-level builder/projection receipts in the corresponding follow-up, including pure Rust loops with no evaluator reentry. `Incremental` is not evidence that every function grows a Vec; a whole-operation exact projection or a non-growing path can be its justified disposition. |

For new builder admission, include **already live other builders** as well as
S. A new reservation must replace the builder's old reservation, not add the
entire prefix every iteration. If a callback result cannot be bounded before
evaluation, check/charge its actual retained weight before appending or
continuing to the next callback; retain the documented single-construction
limitation. Known string/list growth should be projected before allocation.
Allocator over-reservation must be reconciled afterward. These are structural
admission checks, not a promise to prevent every transient allocation.

Move an occurrence's token into a continuation or published session owner at
the same boundary that removes it from R. If there are two actual retained
occurrences (for example, both a frame copy and the installed definition),
charge both. A host-returned result leaves S/R after the existing result check.
Use generation-bound tokens with single ownership; no borrowed Session guard
may be held across callback evaluation or suspension. The prototype must show
that its safe ownership implementation satisfies this, including panic/drop.

In the following table, `contract` is the line in
`S/catalog/memory_growth_contract.rs`; all other anchors use the prefixes
above.

| Builtin | Contract | Catalog registration | Implementation entry point(s) |
| --- | --- | --- | --- |
| `$ContextPath` | `:20` | `R/builtin_catalog/namespace/variables.rs:9` | `R/eval_core/symbol.rs:14` |
| `$MessageList` | `:23` | `R/builtin_catalog/namespace/variables.rs:25` | `R/eval_core/symbol.rs:48` |
| `$Packages` | `:24` | `R/builtin_catalog/namespace/variables.rs:30` | `R/eval_core/symbol.rs:23` |
| `$Path` | `:25` | `R/builtin_catalog/namespace/variables.rs:40` | `R/eval_core/symbol.rs:40` |
| `Accumulate` | `:31` | `S/catalog/functional.rs:65` | `S/functional_ops/sequence/window.rs:128` |
| `AddTo` | `:32` | `R/builtin_catalog/rules.rs:126` | `R/definitions/definition_updates.rs:108` |
| `AppendTo` | `:44` | `R/builtin_catalog/rules.rs:211` | `R/definitions/definition_updates.rs:166` |
| `Apply` | `:45` | `S/catalog/functional.rs:340` | `S/functional_ops/apply_map/mod.rs:137` |
| `ApplyTo` | `:46` | `R/builtin_catalog/rules.rs:194` | `R/definitions/definition_updates.rs:155` |
| `ArgumentsOptions` | `:47` | `R/builtin_catalog/namespace/symbols.rs:287` | `R/options/arguments.rs:258` |
| `Array` | `:48` | `S/catalog/functional.rs:445` | `S/functional_ops/arrays/generators.rs:84` |
| `ArrayFilter` | `:49` | `S/catalog/functional.rs:458` | `S/functional_ops/arrays/mod.rs:59` |
| `ArrayReduce` | `:51` | `S/catalog/functional.rs:471` | `S/functional_ops/arrays/mod.rs:111` |
| `AssociateTo` | `:53` | `S/catalog/associations_transform.rs:135` | `S/associations/mutation.rs:148` |
| `Association` | `:54` | `S/catalog/associations.rs:9` | `S/associations/basic.rs:23` |
| `AssociationKeyDeflatten` | `:55` | `S/catalog/associations.rs:27` | `S/associations/key_reshape.rs:55` |
| `AssociationKeyFlatten` | `:56` | `S/catalog/associations.rs:33` | `S/associations/key_reshape.rs:76` |
| `AssociationMap` | `:57` | `S/catalog/associations_transform.rs:11` | `S/associations/resumable/map.rs:6` |
| `AssociationThread` | `:59` | `S/catalog/associations_transform.rs:31` | `S/associations/transforms.rs:127` |
| `BlockMap` | `:73` | `S/catalog/functional.rs:9` | `S/functional_ops/sequence/window.rs:73` |
| `Cases` | `:83` | `S/catalog/rewrite.rs:18` | `R/kernel_data/rewrite_data/cases/collect.rs:22`; `S/rewrite_builtins.rs:14` |
| `CoefficientList` | `:99` | `S/catalog/polynomials.rs:27` | `S/polynomials/mod.rs:84` |
| `Comap` | `:100` | `S/catalog/functional.rs:39` | `S/functional_ops/apply_map/mod.rs:14` |
| `ComapApply` | `:101` | `S/catalog/functional.rs:52` | `S/functional_ops/apply_map/mod.rs:36` |
| `ConformSchema` | `:112` | `S/catalog/schema.rs:9` | `S/schema.rs:106` |
| `Contexts` | `:121` | `R/builtin_catalog/namespace/contexts.rs:78` | `R/builtins_context_loader.rs:103` |
| `Counts` | `:125` | `S/catalog/associations_transform.rs:271` | `S/associations/aggregate.rs:55` |
| `CountsBy` | `:126` | `S/catalog/associations_transform.rs:291` | `S/associations/resumable/aggregate.rs:68` |
| `D` | `:130` | `S/catalog/calculus.rs:8` | `S/calculus.rs:21` |
| `Decrement` | `:141` | `R/builtin_catalog/rules.rs:270` | `R/definitions/definition_updates.rs:200` |
| `DefaultValues` | `:142` | `R/builtin_catalog/namespace/symbols.rs:128` | `R/messages/builtins_messages_inspect.rs:176` |
| `DefinePackage` | `:143` | `R/builtin_catalog/scoping.rs:35` | `R/scoping/packages.rs:235` |
| `Definition` | `:144` | `R/builtin_catalog/namespace/symbols.rs:102` | `R/messages/builtins_messages_inspect.rs:30` |
| `DeleteDuplicatesBy` | `:148` | `S/catalog/structure_ordering.rs:27` | `S/structure_ops/dedup.rs:122` |
| `Differences` | `:155` | `S/catalog/structure.rs:225` | `S/structure_ops/list_plumbing.rs:4` |
| `DivideBy` | `:163` | `R/builtin_catalog/rules.rs:177` | `R/definitions/definition_updates.rs:144` |
| `Dot` | `:167` | `S/catalog/functional.rs:184` | `S/functional_ops/arrays/mod.rs:33` |
| `DownValues` | `:168` | `R/builtin_catalog/namespace/symbols.rs:181` | `R/messages/builtins_messages_inspect.rs:149` |
| `DumpSave` | `:170` | `R/builtin_catalog/namespace/contexts.rs:321` | `R/package_tools.rs:86` |
| `Echo` | `:173` | `R/builtin_catalog/debug.rs:112` | `R/debug_tools/basic.rs:30` |
| `EvaluationData` | `:187` | `R/builtin_catalog/debug.rs:249` | `R/debug_tools/evaluation_data.rs:5` |
| `ExceptionTypes` | `:191` | `S/catalog/control.rs:393` | `R/kernel_data/control_data/exceptions.rs:87`; `S/control_flow.rs:457` |
| `Expand` | `:193` | `S/catalog/polynomials.rs:9` | `S/polynomials/mod.rs:22` |
| `Extract` | `:194` | `R/builtin_catalog/rules.rs:34` | `R/kernel_data/structure_data/selectors.rs:179` |
| `Factor` | `:195` | `S/catalog/polynomials.rs:45` | `S/polynomials/mod.rs:127` |
| `FileNames` | `:208` | `R/builtin_catalog/namespace/contexts.rs:156` | `R/builtins_context_loader.rs:182` |
| `FixedPoint` | `:211` | `S/catalog/functional.rs:314` | `S/functional_ops/iteration.rs:229` |
| `FixedPointList` | `:212` | `S/catalog/functional.rs:327` | `S/functional_ops/iteration.rs:205` |
| `Fold` | `:215` | `S/catalog/functional.rs:196` | `S/functional_ops/sequence/fold/mod.rs:21` |
| `FoldList` | `:216` | `S/catalog/functional.rs:205` | `S/functional_ops/sequence/fold/mod.rs:37` |
| `FoldPair` | `:217` | `S/catalog/functional.rs:214` | `S/functional_ops/sequence/fold/mod.rs:53` |
| `FoldWhile` | `:218` | `S/catalog/functional.rs:223` | `S/functional_ops/sequence/fold/mod.rs:85` |
| `GatherBy` | `:236` | `S/catalog/structure_ordering.rs:63` | `S/structure_ops/collection_stats.rs:375` |
| `GroupBy` | `:243` | `S/catalog/associations_transform.rs:307` | `S/associations/grouping.rs:143` |
| `Increment` | `:262` | `R/builtin_catalog/rules.rs:253` | `R/definitions/definition_updates.rs:192` |
| `Information` | `:264` | `R/builtin_catalog/namespace/symbols.rs:91` | `R/messages/builtins_messages_inspect.rs:48` |
| `Inner` | `:265` | `S/catalog/functional.rs:171` | `S/functional_ops/arrays/mod.rs:14` |
| `Insert` | `:269` | `S/catalog/structure.rs:183` | `S/structure_ops/sequence/insert.rs:13` |
| `JoinAcross` | `:278` | `S/catalog/associations_transform.rs:222` | `S/associations/join_across.rs:80` |
| `KeyDropFrom` | `:282` | `S/catalog/associations_transform.rs:172` | `S/associations/mutation.rs:280` |
| `KeyGroupBy` | `:285` | `S/catalog/associations.rs:248` | `S/associations/key_reshape.rs:12` |
| `KeyIntersection` | `:286` | `S/catalog/associations_transform.rs:71` | `S/associations/transforms/key_collections.rs:146` |
| `KeyMap` | `:287` | `S/catalog/associations.rs:160` | `S/associations/transforms.rs:10` |
| `KeySelect` | `:289` | `S/catalog/associations.rs:192` | `S/associations/resumable/key_select.rs:6` |
| `KeySortBy` | `:291` | `S/catalog/associations.rs:273` | `S/associations/resumable/key_sort.rs:77` |
| `KeyUnion` | `:293` | `S/catalog/associations_transform.rs:51` | `S/associations/transforms/key_collections.rs:81` |
| `KeyValueMap` | `:294` | `S/catalog/associations.rs:176` | `S/associations/transforms.rs:48` |
| `Keys` | `:296` | `S/catalog/associations.rs:72` | `S/associations/basic/keys.rs:18` |
| `LazyTuples` | `:310` | `S/catalog/lazy.rs:181` | `S/lazy.rs:489` |
| `Level` | `:321` | `R/builtin_catalog/rules.rs:511` | `R/tree_walkers/level.rs:4` |
| `Lookup` | `:326` | `S/catalog/associations.rs:39` | `S/associations/basic/lookup.rs:327` |
| `Map` | `:330` | `S/catalog/functional.rs:87` | `S/functional_ops/apply_map/mod.rs:78` |
| `MapAll` | `:331` | `S/catalog/functional.rs:100` | `S/functional_ops/apply_map/mod.rs:96` |
| `MapApply` | `:332` | `S/catalog/functional.rs:353` | `S/functional_ops/apply_map/mod.rs:153` |
| `MapAt` | `:333` | `S/catalog/functional.rs:113` | `S/functional_ops/targeted_map.rs:183` |
| `MapIndexed` | `:334` | `S/catalog/functional.rs:74` | `S/functional_ops/apply_map/mod.rs:60` |
| `MapThread` | `:335` | `S/catalog/functional.rs:140` | `S/functional_ops/targeted_map.rs:217` |
| `MaxBy` | `:339` | `S/catalog/structure_ordering.rs:137` | `S/structure_ops/collection_stats.rs:256` |
| `MaximalBy` | `:341` | `S/catalog/structure_ordering.rs:154` | `S/structure_ops/collection_stats.rs:274` |
| `Merge` | `:349` | `S/catalog/associations_transform.rs:206` | `S/associations/resumable/aggregate.rs:11` |
| `MergeNested` | `:350` | `S/catalog/associations_transform.rs:246` | `S/associations/resumable/aggregate/merge_nested.rs:15` |
| `Message` | `:351` | `R/builtin_catalog/namespace/symbols.rs:4` | `R/messages/builtins_messages.rs:5` |
| `MessageDefinitions` | `:352` | `R/builtin_catalog/namespace/symbols.rs:55` | `R/messages/builtins_messages.rs:89` |
| `MessageTags` | `:354` | `R/builtin_catalog/namespace/symbols.rs:73` | `R/messages/builtins_messages.rs:115` |
| `Messages` | `:355` | `R/builtin_catalog/namespace/symbols.rs:37` | `R/messages/builtins_messages.rs:59` |
| `MinBy` | `:357` | `S/catalog/structure_ordering.rs:167` | `S/structure_ops/collection_stats.rs:265` |
| `MinimalBy` | `:359` | `S/catalog/structure_ordering.rs:184` | `S/structure_ops/collection_stats.rs:283` |
| `Names` | `:370` | `R/builtin_catalog/namespace/contexts.rs:95` | `R/namespace_tools/names.rs:68` |
| `Nest` | `:375` | `S/catalog/functional.rs:262` | `S/functional_ops/iteration.rs:47` |
| `NestList` | `:376` | `S/catalog/functional.rs:288` | `S/functional_ops/iteration.rs:126` |
| `NestWhile` | `:377` | `S/catalog/functional.rs:275` | `S/functional_ops/iteration.rs:80` |
| `NestWhileList` | `:378` | `S/catalog/functional.rs:301` | `S/functional_ops/iteration.rs:159` |
| `Normal` | `:384` | `S/catalog/associations_transform.rs:123` | `S/associations/basic.rs:200` |
| `Off` | `:394` | `R/builtin_catalog/debug.rs:154` | `R/debug_tools/message_switch.rs:15` |
| `On` | `:395` | `R/builtin_catalog/debug.rs:136` | `R/debug_tools/message_switch.rs:4` |
| `Once` | `:396` | `R/builtin_catalog/namespace/contexts.rs:353` | `R/package_tools.rs:90` |
| `OptionValue` | `:399` | `R/builtin_catalog/namespace/symbols.rs:264` | `R/options/builtins.rs:68` |
| `Options` | `:400` | `R/builtin_catalog/namespace/symbols.rs:249` | `R/options/builtins.rs:12` |
| `Outer` | `:404` | `S/catalog/functional.rs:484` | `S/functional_ops/arrays/generators.rs:136` |
| `OwnValues` | `:407` | `R/builtin_catalog/namespace/symbols.rs:143` | `R/messages/builtins_messages_inspect.rs:125` |
| `PackageScope` | `:408` | `R/builtin_catalog/scoping.rs:87` | `R/builtin_adapters.rs:431` |
| `Part` | `:413` | `R/builtin_catalog/rules.rs:5` | `R/kernel_data/structure_data/selectors.rs:99` |
| `Position` | `:425` | `R/builtin_catalog/rules.rs:535` | `R/tree_walkers/position.rs:94` |
| `PositionIndex` | `:426` | `S/catalog/associations_transform.rs:323` | `S/associations/aggregate.rs:120` |
| `PreDecrement` | `:431` | `R/builtin_catalog/rules.rs:304` | `R/definitions/definition_updates.rs:217` |
| `PreIncrement` | `:432` | `R/builtin_catalog/rules.rs:287` | `R/definitions/definition_updates.rs:207` |
| `PrependTo` | `:434` | `R/builtin_catalog/rules.rs:232` | `R/definitions/definition_updates.rs:181` |
| `Print` | `:440` | `R/builtin_catalog/scoping.rs:282` | `R/builtins_input.rs:104` |
| `Product` | `:441` | `S/catalog/control.rs:65` | `R/kernel_data/control_data/branching_loops/iteration_aggregate.rs:48`; `S/control_flow.rs:57` |
| `Public` | `:444` | `R/builtin_catalog/scoping.rs:79` | `R/builtin_adapters.rs:422` |
| `Reap` | `:461` | `R/builtin_catalog/debug.rs:178` | `R/debug_tools/reap.rs:56` |
| `RegisterExceptionType` | `:462` | `S/catalog/control.rs:376` | `R/kernel_data/control_data/exceptions.rs:27`; `S/control_flow.rs:444` |
| `Replace` | `:470` | `S/catalog/rewrite.rs:68` | `R/kernel_data/rewrite_data/replace/once.rs:61`; `S/rewrite_builtins.rs:38` |
| `ReplaceAll` | `:471` | `S/catalog/rewrite.rs:95` | `R/kernel_data/rewrite_data/replace/all.rs:5`; `S/rewrite_builtins.rs:46` |
| `ReplaceAllUnheld` | `:472` | `S/catalog/rewrite.rs:108` | `R/kernel_data/rewrite_data/replace/all.rs:72`; `S/rewrite_builtins.rs:54` |
| `ReplaceAt` | `:473` | `R/builtin_catalog/rules.rs:84` | `R/kernel_data/structure_data/replace/replace_at.rs:13` |
| `ReplaceList` | `:474` | `S/catalog/rewrite.rs:82` | `R/kernel_data/rewrite_data/replace/list.rs:17`; `S/rewrite_builtins.rs:62` |
| `ReplacePart` | `:475` | `R/builtin_catalog/rules.rs:66` | `R/kernel_data/structure_data/replace/replace_part.rs:59` |
| `ReplaceRepeated` | `:476` | `S/catalog/rewrite.rs:121` | `R/kernel_data/rewrite_data/replace/repeated.rs:5`; `S/rewrite_builtins.rs:70` |
| `ReverseSortBy` | `:485` | `S/catalog/structure_ordering.rs:124` | `S/structure_ops/collection_stats.rs:341` |
| `Riffle` | `:486` | `S/catalog/structure_ordering.rs:223` | `S/structure_ops/collection_stats.rs:418` |
| `RumProfileSnapshot` | `:493` | `R/builtin_catalog/debug.rs:276` | `R/debug_tools/basic.rs:124` |
| `Save` | `:497` | `R/builtin_catalog/namespace/contexts.rs:293` | `R/package_tools.rs:82` |
| `SchemaErrors` | `:499` | `S/catalog/schema.rs:24` | `S/schema.rs:55` |
| `Select` | `:501` | `S/catalog/functional.rs:497` | `S/functional_ops/sequence/select.rs:52` |
| `SelectPositions` | `:503` | `S/catalog/structure_ordering.rs:241` | `S/structure_ops/list_lookup.rs:40` |
| `SequenceFold` | `:506` | `S/catalog/functional.rs:236` | `S/functional_ops/sequence/fold/mod.rs:137` |
| `SequenceFoldList` | `:507` | `S/catalog/functional.rs:249` | `S/functional_ops/sequence/fold/mod.rs:161` |
| `Set` | `:508` | `R/builtin_catalog/rules.rs:98` | `R/definitions/definitions.rs:22` |
| `SetAttributes` | `:509` | `R/builtin_catalog/namespace/symbols.rs:317` | `R/attributes.rs:47` |
| `SetDelayed` | `:510` | `R/builtin_catalog/rules.rs:321` | `R/definitions/definitions.rs:67` |
| `SetOptions` | `:511` | `R/builtin_catalog/namespace/symbols.rs:349` | `R/options/builtins.rs:29` |
| `Short` | `:520` | `R/builtin_catalog/debug.rs:99` | `R/debug_tools/basic.rs:6` |
| `SortBy` | `:524` | `S/catalog/structure_ordering.rs:89` | `S/structure_ops/ordering_sets.rs:165` |
| `Sow` | `:525` | `R/builtin_catalog/debug.rs:172` | `R/debug_tools/reap.rs:21` |
| `Stack` | `:527` | `R/builtin_catalog/debug.rs:308` | `R/debug_tools/basic.rs:96` |
| `StringApply` | `:533` | `S/catalog/strings_match.rs:160` | `S/strings/patterns/search.rs:455` |
| `StringCases` | `:534` | `S/catalog/strings_match.rs:150` | `S/strings/patterns/search.rs:272` |
| `StringReplace` | `:556` | `S/catalog/strings_match.rs:140` | `S/strings/patterns/search.rs:198` |
| `SubValues` | `:567` | `R/builtin_catalog/namespace/symbols.rs:200` | `R/messages/builtins_messages_inspect.rs:164` |
| `SubsetMap` | `:569` | `S/catalog/functional.rs:22` | `S/functional_ops/targeted_map.rs:149` |
| `SubtractFrom` | `:573` | `R/builtin_catalog/rules.rs:143` | `R/definitions/definition_updates.rs:119` |
| `Sum` | `:575` | `S/catalog/control.rs:78` | `R/kernel_data/control_data/branching_loops/iteration_aggregate.rs:55`; `S/control_flow.rs:66` |
| `SyntaxInformation` | `:579` | `R/builtin_catalog/namespace/symbols.rs:234` | `R/namespace_tools/syntax.rs:4` |
| `Table` | `:581` | `S/catalog/control.rs:121` | `R/kernel_data/control_data/branching_loops/iteration/driver.rs:53`; `S/control_flow.rs:99` |
| `TagSet` | `:582` | `R/builtin_catalog/rules.rs:356` | `R/definitions/definition_tags.rs:39` |
| `TagSetDelayed` | `:583` | `R/builtin_catalog/rules.rs:373` | `R/definitions/definition_tags.rs:71` |
| `TakeLargestBy` | `:587` | `S/catalog/structure_ordering.rs:197` | `S/structure_ops/collection_stats.rs:292` |
| `TakeList` | `:588` | `S/catalog/structure.rs:206` | `S/structure_ops/sequence/take_list.rs:4` |
| `TakeSmallestBy` | `:589` | `S/catalog/structure_ordering.rs:210` | `S/structure_ops/collection_stats.rs:301` |
| `TakeWhile` | `:590` | `S/catalog/functional.rs:153` | `S/functional_ops/sequence/window.rs:145` |
| `Tally` | `:591` | `S/catalog/structure_ordering.rs:50` | `S/structure_ops/collection_stats.rs:92` |
| `TestReport` | `:596` | `R/builtin_catalog/debug.rs:294` | `R/debug_tools/test_report/report.rs:186` |
| `Through` | `:600` | `S/catalog/functional.rs:423` | `S/functional_ops/callable/head_ops.rs:4` |
| `TimesBy` | `:606` | `R/builtin_catalog/rules.rs:160` | `R/definitions/definition_updates.rs:133` |
| `ToExpression` | `:611` | `S/catalog/strings_text.rs:87` | `S/strings/text/builtins.rs:438` |
| `Together` | `:621` | `S/catalog/polynomials.rs:36` | `S/polynomials/mod.rs:110` |
| `Trace` | `:624` | `R/builtin_catalog/debug.rs:5` | `R/debug_tools/trace.rs:28` |
| `TraceDetail` | `:625` | `R/builtin_catalog/debug.rs:20` | `R/debug_tools/trace.rs:32` |
| `TracePrint` | `:626` | `R/builtin_catalog/debug.rs:35` | `R/debug_tools/trace.rs:39` |
| `Unique` | `:636` | `R/builtin_catalog/scoping.rs:95` | `R/scoping/unique.rs:46` |
| `UpSet` | `:641` | `R/builtin_catalog/rules.rs:332` | `R/definitions/definition_tags.rs:8` |
| `UpSetDelayed` | `:642` | `R/builtin_catalog/rules.rs:344` | `R/definitions/definition_tags.rs:28` |
| `UpValues` | `:644` | `R/builtin_catalog/namespace/symbols.rs:162` | `R/messages/builtins_messages_inspect.rs:137` |
| `Values` | `:652` | `S/catalog/associations.rs:88` | `S/associations/basic/values.rs:18` |

### 4. Falsifier design

**Keep the walker permanently as an independent implementation.** Extract its
pure per-field observations from the current peak-updating entry point, so
verification itself neither updates MaxMemoryUsed nor mutates caches. Do not
implement the oracle as a sum of cached ledger values. It must traverse real
owners, capacities, expressions and frame roots using the documented formula.
Sharing a tested leaf expression formula is reasonable, but retain independent
known-value tests so that a common formula error cannot satisfy both sides.

There are three independent gates:

1. **Field/schema and mutation coverage.** Preserve the exhaustive Session
   pattern and explicit exemption reasons. Extend this to the production
   accounting schema and restrict mutable access to retained owners. No raw
   mutable map/Vec/Expr or `DerefMut` escape may bypass accounting in armed
   mode. Nested dispatch `OnceLock` fills and read-side cache evictions count
   as writes. Add a deliberately unclassified field in an isolated test
   fixture and observe the expected compile failure; add a direct mutation
   bypass in a test fixture and require it to be inaccessible or detected.
2. **Numeric equality after actual transitions.** For each completed mutation
   transaction and each existing resource check in test/verification builds,
   assert every maintained field contribution and their total equal the pure
   traversal. Check before/after nested evaluation, immediately before and
   after suspend/resume/abandon transfers, and after normal/error/cancel/
   timeout/rollback/reset boundaries. Keep both sides per field so a +x/-x
   cancellation between unrelated fields cannot hide corruption. Use bounded
   deterministic generated operation sequences on rum's own APIs: repeated
   shared imports, replace/unset/clear, capacity grow/shrink/take/split, lazy
   dispatch index warming, association eviction, messages/prints/traces,
   nested Reap/Monitor/options/Block/Module, package rollback, queued input,
   repeated abandon, cleanup that suspends again, and caught callback panic.
3. **Builtin retained-growth progress.** Independently enumerate live builder
   payloads and compare their modeled weights with R at each append/reserve/
   release/transfer boundary. Assert there is no unchecked retained growth
   between callback iterations. A dispatch-entry/exit count is insufficient:
   it misses a large builder which is freed or fails before the call returns.
   Raw special-form arguments are not an upper bound on their evaluated
   outputs; do not repeat the known raw-argument validator problem recorded
   in TODO's memory-contract notes.

Watch the checks fail using test-only fault injection that alters only the
candidate accounting path, never the oracle. At minimum:

- Omit one definition/root charge: the next consistency check must report
  that field's exact negative discrepancy.
- Omit one pop/release: repeated complete/reclaim cycles must fail with a
  positive discrepancy, rather than only eventually tripping a small limit.
- Ignore one spare-capacity increase without changing length: must fail at
  that operation.
- Omit one read-side index insertion or stale-cache eviction: must fail on a
  lookup, proving the inventory is not assignment-only.
- Double-release or omit one pending/resuming/worklist transfer, including a
  nested cleanup suspension: must fail at the transfer, not after another
  allocation happens to compensate it.
- Leave a rollback ledger at its pre-trial scalar while restored containers
  have different capacities/index contents: must fail by field.
- Exercise synthetic saturated weights, release after saturation, and
  re-addition: must fail for naive `saturating_sub`, without allocating an
  impossible process-sized object.
- Skip one Table/history/String builder update: builder consistency or a
  low-budget retained-growth test must fail before the next iteration.

Run each injected bad variant once and paste its failure; remove/disable the
fault and run the same test green. Intentional faults belong to test-only
mechanisms, with no branch in production's unarmed path. The verifier should
be compiled out of normal release builds; any opt-in diagnostic must use an
existing cold entry/armed-mode boundary, not a new unconditional per-call
atomic load. No sampling in either the resource check or the test oracle.

Panic handling needs a separate assertion: the armed ledger cannot report
valid after a partially completed mutation unwinds. Rebuild it before any
permitted reuse (or discard it during reset), then compare to the actual
remaining state. Do not demand a return to pre-panic bytes when the runtime
still retains state. The host's poison rule and raw Session's possible outer
`catch_unwind` must both be exercised.

The new ledger/builder falsifiers were **not run here**: no such implementation
exists and this brief forbids code changes. The existing ceiling falsifier
was re-run against preserved good and bad test executables, showing the
negative-control mechanism really fails. Verbatim results:

```text
$ scripts/run-bounded.py --timeout 5 -- target/brief-515/candidate-build/debug/deps/rum_runtime-562b258c6eef9b02 session_memory --nocapture --test-threads=1

running 4 tests
test runtime::tests::session_memory::byte_array_proxy_tracks_encoded_string_capacity_and_preserves_invalid_fallback ... ok
test runtime::tests::session_memory::session_memory_field_decision_inventory_is_sorted_and_exhaustive ... ok
test runtime::tests::session_memory::session_memory_walk_counts_declared_exception_storage ... ok
test runtime::tests::session_memory::stability_blooms_are_fixed_size_inline_values ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 995 filtered out; finished in 0.00s

exit: 0

$ scripts/run-bounded.py --timeout 5 -- target/brief-515/candidate-build/debug/deps/rum_stdlib-c8c5a89643371dc7 tests::debug_tools_resources::memory_constraint_ --nocapture --test-threads=1

running 2 tests
test tests::debug_tools_resources::memory_constraint_checks_each_repeated_string_growth_step ... ok
test tests::debug_tools_resources::memory_constraint_keeps_projected_string_repeat_precheck ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 987 filtered out; finished in 0.01s

exit: 0

$ scripts/run-bounded.py --timeout 5 -- target/brief-515/stride-build/debug/deps/rum_stdlib-3a556b9db7fd59ac tests::debug_tools_resources::memory_constraint_ --nocapture --test-threads=1

running 2 tests
test tests::debug_tools_resources::memory_constraint_checks_each_repeated_string_growth_step ... FAILED
test tests::debug_tools_resources::memory_constraint_keeps_projected_string_repeat_precheck ... ok

failures:

failures:
    tests::debug_tools_resources::memory_constraint_checks_each_repeated_string_growth_step

test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 987 filtered out; finished in 0.01s


thread 'tests::debug_tools_resources::memory_constraint_checks_each_repeated_string_growth_step' panicked at crates/rum-stdlib/src/tests/debug_tools_resources.rs:110:5:
repeated string growth escaped its 200000-byte ceiling
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
exit: 101

```

These are executions of preserved brief-515 test binaries, not a fresh
workspace build or a test of an incremental implementation. The intentional
exit 101 is the bad-stride control; it was not retried.

### 5. Cost model and measured evidence

#### 5.1 What was measured here

I read the required project/testing documents, the memory-accounting rulings
in TODO, reports 513–515, the current implementation and its owning tests.
The runtime/script diff from the unstride commit is empty:

```text
$ git diff --stat d1e5de5c HEAD -- crates scripts
```

The preserved repaired source files also match current files byte-for-byte:

```text
PRESERVED_SOURCE_MATCH debug_tools/resources/constraints.rs True
PRESERVED_SOURCE_MATCH model/session.rs True
PRESERVED_SOURCE_MATCH session/session_helpers.rs True
PRESERVED_SOURCE_MATCH tests/session_memory.rs True
PRESERVED_SOURCE_MATCH session/session_state.rs True
```

Executable identities, freshly hashed:

```text
SHA256 target/brief-514/rum-base 9a0b0583f64ed6bdc77ee0819f152f3822786edf266aabb00f3dc5640bd50e43
SHA256 target/brief-514/rum-final 877e1cb308d533c3ab49fe2c348ba1ca5c7b76e0868705f3af41b9c8277675ac
```

These binaries were reused, not rebuilt. `base` below is the preserved old
stride and `final` is the preserved unstrided repair. **Neither is an
incremental-accounting candidate.** This evidence reproduces the cost to be
removed; it does not measure the proposed replacement.

Each fresh perf invocation used the following shape, with existing `.rum`
sources unchanged, affinity CPU 2, all inherited `RUM_*` variables removed,
repo-local TMPDIR, offline cargo settings, and stdin closed:

```text
scripts/run-bounded.py --timeout 5 -- taskset -c 2 perf stat -e instructions:u -x, -o target/brief-516/<arm>-<label>-<repeat>.perf.csv -- target/brief-514/rum-<arm> target/brief-514/<label>.rum
```

The two smoke sources are instead
`target/brief-501/smoke25000.rum` and `smoke50000.rum`. Source payloads were
read before execution. The armed controls retain the named root, print its
ByteCount outside the loop, and evaluate the existing
`MemoryConstrained[Do[Null, {n}], 1000000000]` statement; the output check
establishes normal completion. No timing statement or extra sink was inserted.
Only one CPU-heavy workload ran at a time. `cpu_core/instructions/u` supplied
one numeric row at 100% coverage on this hybrid PMU; the unsupported atom row
was not added to it.

The 5-second per-process bounds and 60-second batch bounds used the preceding
same-shape subsecond runs as sizing evidence; elapsed monotonic subprocess
durations below are timeout evidence only, never the speed metric. There
were 30 executable invocations: 24 perf runs, three CLI semantic probes, and
three test-executable runs. No build or full suite was run. Verbatim reducer
output from the saved JSON records:

```text
PROBE_PROCESSES 30
MAX_FRESH_MONOTONIC_SECONDS 0.25530989799881354
label arm repeat exit instructions coverage
armed-empty-16 base 1 0 3826908 100.00
armed-empty-16 final 1 0 8662295 100.00
armed-empty-16 final 2 0 8660195 100.00
armed-empty-16 base 2 0 3824883 100.00
armed-empty-16 base 3 0 3826854 100.00
armed-empty-16 final 3 0 8660247 100.00
armed-list10k-1024 base 1 0 18500375 100.00
armed-list10k-1024 final 1 0 801717046 100.00
armed-list10k-1024 final 2 0 801716693 100.00
armed-list10k-1024 base 2 0 18500053 100.00
armed-list10k-1024 base 3 0 18498488 100.00
armed-list10k-1024 final 3 0 801718261 100.00
armed-empty-16 base_median 3826854 final_median 8660247 ratio 2.2630199636568316
armed-list10k-1024 base_median 18500053 final_median 801717046 ratio 43.335932388950454
unarmed label base final
smoke25000 689833519 689830140
smoke50000 1376130970 1376128770
symbol-ownvalue-read-scale10-det 301954096 301956192
symbol-ownvalue-read-scale20-det 599435493 599437572
downvalue-pattern-hit-scale10-det 1023739501 1023788401
downvalue-pattern-hit-scale20-det 2043129562 2043088855
```

The two armed ratios reproduce the brief's 2.26x and 43.34x finding. The six
unarmed rows are one fresh paired observation per label, not a new statistical
gate or a proof of exact zero instructions. Their paired sizes also show the
intended work increases with iteration count. Source inspection confirms
`eval_memory_limit: None` by default (`R/model/config.rs:32`) and that the
repaired full walk is behind the existing active-limit branch
(`R/debug_tools/resources/constraints.rs:144`).

Fresh semantic stdout, preserved verbatim:

```text
$ target/brief-514/rum-final target/brief-514/reproduction.rum
PASS: repeated growth aborted
Null
$ target/brief-514/rum-final target/brief-514/control.rum
PASS: projected allocation aborted
Null
```

Both commands actually ran under the 5-second bounded runner, exited 0, and
had empty stderr. The occurrence probe and the good/bad test results are in
sections 2 and 4. Raw stdout/stderr/perf CSV plus full JSON are in
`target/brief-516/reprobe-records.json` and `verification-records.json` and
their named sibling logs. They are local scratch, not generated artifacts to
commit. No failed performance probe was repeated without new information.

#### 5.2 Predicted costs, explicitly not benchmark results

Let `T_S` be the work of traversing all current session terms, and `T_x` the
work of traversing an expression **by occurrence**, including byte work such
as BigInt serialization and ByteArray validation. Node count alone does not
bound either. Let `T_changed` describe only roots/containers entering,
leaving, or changing at a mutation. The following are inferred from the
proposed algorithm; there are no guessed throughput or speedup numbers.

| Operation | Today | Proposed cost and qualification | Status |
| --- | --- | --- | --- |
| Armed expression checkpoint | `T_S + T_x` plus time/limit handling | Constant sum load/add/compare and the same `T_x`; no whole-session traversal. Entry/exit root accounting elsewhere may also traverse x. | Inferred; instruction constant not measured. |
| Armed projected-byte precheck | `T_S` plus addition/comparison | Constant arithmetic if S/R are valid and projected contribution is known. Preserve existing overflow and error ordering as explicitly specified. | Inferred. |
| Arm after unarmed mutation | No maintained ledger; walk on each checkpoint | One `T_S` seed plus metadata setup. Short repeated armed intervals may remain expensive. | Inferred. |
| New/removed arbitrary expression occurrence | Paid repeatedly as part of future walks | `T_changed` once at owner mutation unless a valid weight travels with it; saved-weight release can be constant. No promise that importing an arbitrary tree is O(1). | Inferred. |
| Append a value to a live owner/builder | Future walks rescan retained prefixes, or local builder is invisible | Weigh new value plus shell/capacity delta, then constant accumulator update. A reallocated container still has its real copy/hash/allocation work. Never re-walk the old prefix solely for accounting. | Inferred. |
| Definition/cache index build or replacement | Walk includes whatever index fields its helper counts | Update the affected index/capacity contribution alongside its existing build. A wholesale rebuild costs its existing build plus changed-root work, not a Session scan. | Inferred. |
| Frame/stack transfer | Each checkpoint walks all traced owners | Initial frame weight/root traversal, then stored-owner transfer plus actual shell deltas. Adding a new frame can cost its roots; retaining an unchanged suffix must not repeatedly traverse it. | Inferred. |
| Bulk snapshot restore/reset/poison recovery | Full walk observes actual post-state later | Cold recomputation of affected fields or a full seed is allowed and reported. Snapshot capture/clone has existing real allocation work; no saved-total shortcut across changed capacities. | Inferred. |
| Explicit unarmed MemoryInUse/MaxMemoryUsed | `T_S` | Continue `T_S`; avoiding all default maintenance means no universal constant-time claim. ByteCount remains `T_x`. | Inferred. |
| Normal unarmed evaluation | Existing code | Accounting operations must compile away on that route; layout/cache/code-size effects still require A/B. “No extra branch in source” alone is not evidence. | Required gate, not established. |
| Bookkeeping space | None for this ledger | Fixed per-field totals plus whichever per-owner/frame/builder weights the measured design needs; report actual layout and heap capacities. No per-Expr representation change required. | Inferred shape; byte count not measured. |

The repeated unchanged-session term is what should disappear, not necessarily
all 43.34x of the process instruction count. Old stride is an incorrect lower
cost bound, never an acceptance baseline. (a) intentionally adds coverage and
work on armed paths; isolate its measurements from the earlier (b) cutover.

#### 5.3 Required implementation A/B recipe

Use the **correct unstrided HEAD before the change** as base. Preserve its
release binary, source identity and hashes under that implementation brief's
scratch before editing. Build the candidate with identical toolchain,
features, release profile and environment. Do not obtain a base by changing
the worktree with git. The director can supply a preserved correct artifact,
or the executor builds and copies the current source before its edits.

Fix and publish the probe panel before changing code. Minimum panel: six
existing unarmed smoke/ownvalue/downvalue scale-pair labels above; four armed
unchanged-root labels (`empty`/`list10k` crossed with 16/1024 checks); two
size-paired definition replace/reclaim labels; two builtin builders (a
history/list and a String output); two size-paired suspension/abandonment
labels; and two cold-arm/rearm labels. That is 18 labels. Add the actual
modified family's same-tier neighbors via the existing core microbench suite.
For an additional critical label, revise the declared cap before running it;
do not keep widening the panel after a disappointing result.

For instructions, run three alternating base/candidate pairs per label,
**108 measured process launches maximum** for that fixed panel. Record every
raw count, exit, stdout marker and event coverage; use the median of the three
only after publishing raw rows. Pin the same valid CPU, require one numeric
100%-coverage user-instruction event, strip profiling/contract-verification
env flags, and use the exact same `.rum` source bytes for each pair. Choose
limits that arm the normal workloads without aborting. Ceiling-rejection
correctness probes are separate, since an early abort can fake a speedup.
Check doubled work sizes and report instruction slope as well as whole-process
ratios, so startup does not conceal a per-step regression.

Also run the required timing suite, one base and one candidate initially,
maximum two complete pairs (best-of-2) only for a stated unresolved concern:

```text
scripts/run-bounded.py --timeout <measured-suite-bound> -- bash scripts/core-language-microbench.sh --rum-cli <base-binary> --output-dir target/<brief>/micro-base --timeout <measured-run-bound>
scripts/run-bounded.py --timeout <measured-suite-bound> -- bash scripts/core-language-microbench.sh --rum-cli <candidate-binary> --output-dir target/<brief>/micro-new --timeout <measured-run-bound>
```

Use the suite's in-kernel monotonic timings.tsv rows, no wall-clock
subtraction; print the target and every same-tier neighbor. Set all output and
temporary paths inside the repo, all environment values before either arm,
and timeouts from recorded same-shape runs/builds. The short binary bounds
used here are not transferable to a workspace build or whole microbench.
Slow defining modes run once. No concurrent CPU-heavy command.

Acceptance: no reproducible unarmed per-label or slope regression, no hot or
same-tier timing regression, unchanged correctness/ceiling checks, equality
falsifiers green, and no dependence of repeated armed checks on the size of an
**unchanged, unrelated** retained session root after the one-time seed. An
isolated negative counter fluctuation is not a performance win. If a stable
positive regression appears, report it and fix/revert the candidate within
scope; do not average it away or restore stride. Actual instruction/space
constants and the feasibility of static mode routing remain unmeasured until
these implementation gates run.

### 6. Draft follow-up briefs and director decisions

Dispatch order is A → B → C → D → E → F → G/H/I → J, one rewritten brief at
a time. G/H/I share a completed builder protocol but should still be gated
separately. The steps deliberately distinguish a measured plumbing decision,
shadow accounting, production cutover, and extension of coverage. No stage
may advertise all 165 declarations as discharged before J's census closes.

The following common envelope is part of **each** draft, to be copied into
its rewritten brief. Each draft below has the template's sections and its own
allowed scope, deliverable and gate.

- Read `AGENTS.md`, `docs/testing-strategy.md`,
  `docs/code-organization-guidelines.md`, this report, and the immediately
  preceding accepted report. Keep genuine module boundaries and the file-size
  thresholds; new accounting code belongs in small responsibility-based
  modules, not an enlarged `memory.rs` or Session implementation file.
- Only the named source areas, their named owning tests, the explicitly named
  documentation, the assigned brief's appended report, and scratch under
  `target/<assigned-brief>/` may change. Preserve existing user changes.
  Paths below use the report prefixes. A directory scope includes its Rust
  submodules, not unrelated files elsewhere in the crate.
- Never change `Symbol`, `Expr`, `SharedExpr`, or `SharedArgs` representation,
  public type/serde shape, `C/expr.rs`, the interner, snapshot format,
  `AGENTS.md`, `TODO.md`, `PROGRESS.md`, `Cargo.lock`, or other briefs. The
  narrowly named mode adapter API decision in B is the only proposed public
  signature exception and requires the director's rewritten scope first.
- No git mutation/index/config, network, new dependencies, new unsafe, or
  writes outside the repo. Do not overwrite earlier brief evidence. Use safe
  Rust 1.85, edition 2024. No vendor oracle/comparison fixtures. Do not weaken
  an existing contract/test to make accounting pass. Preserve suspension and
  cancellation behavior; no stride or adaptive checks.
- The allowed tools are scoped file reads/edits, read-only git status/diff,
  local Rust/cargo, repo-local Python reducers, existing rum-cli/bounded-runner/
  microbench tools, and local taskset/perf for the named instruction gate.
  Generated rum source always ends in `.rum`.
- Run every long/spawning command through `scripts/run-bounded.py`. Record
  same-shape prior duration and derive its timeout before running. Use the
  fixed environment, one CPU-heavy workload at a time, and panel/cap in 5.3:
  at most 108 instruction runs and at most two complete microbench A/B pairs
  for a performance gate, slow defining mode once. Revise a cap explicitly
  before adding required probes. Stop after two failures with no additional
  information; report instead of tuning indefinitely.
- No default-path accounting allocation, mutex, lookup, clone, new branch or
  unconditional counter load. Static erasure is a proposal to prove, not an
  exemption from measurement. Diagnostic consistency/fault injection must
  compile out or live behind the already-accounted cold route. Do not use an
  extra sink/Set/Print inside a microbench's exact timed statement.
- Verification commands below are the inner commands; wrap them with measured
  bounds. Run focused tests during iteration, then the named full suite and
  `cargo fmt --check`. Every hot-path change also runs both gates in 5.3.
  Save raw logs and print exact relevant results; a forecast is not a result.
- Each report uses TEMPLATE's four items: (1) per-file change/line ranges and
  why; (2) exact commands, outputs, red/green falsifier evidence and per-label
  A/B rows; (3) findings, including armed/unarmed work and storage costs;
  (4) stop-loss, missing coverage and unresolved questions. Leave uncommitted;
  the director runs the authoritative gate.

#### Draft A — specify the occurrence sum and preserve its oracle

**Goal.** Establish an independently traversed, per-field occurrence model
and exhaustive production/test schema before adding cached totals. Incorporate
only the director-approved overflow rule; leave production checkpoint
frequency and ownership behavior unchanged.

**Context pointers.** Section 2's contract/test matrix; section 3's 94-field
inventory; `R/debug_tools/resources/memory.rs`, `R/tests/session_memory.rs`,
`R/model/session.rs`, `R/model/definitions/down_value_dispatch.rs`,
`R/model/definitions/tagged_value_store.rs`, `R/model/cache.rs`;
`docs/guide/resource-limits-and-sandbox.md`.

**Scope.** MAY modify those runtime files, new pure accounting/oracle modules
under `R/debug_tools/resources/`, required module declarations, their tests
under `R/tests/session_memory*`, `R/builtin_catalog/debug.rs` memory-entry
documentation, the guide, and regenerated `docs/reference/builtins.md`.
MUST NOT change evaluator dispatch, ownership/retention behavior, builtin
builders or excluded common-envelope areas. No new maintained ledger yet.

**Deliverable.** A pure independent field breakdown plus existing sampled-peak
wrapper; an exhaustive schema preserving every exemption; exact tests for
sharing/capacity/ByteArray/exception/dispatch/frame formulas; synthetic overflow
and release-after-overflow expectations under the approved rule. Document
occurrence semantics and retained omissions explicitly. Record baseline sizes
for Session and relevant frame/index structures.

**rum / Rust pitfalls (read twice).** Do not replace traversal with cached
values, deduplicate Arc roots, make diagnostics clear caches, or silently
remove conservative inline terms. `sum::<usize>()` and saturated subtraction
need explicit reasoning. Changing a helper's arithmetic is still a runtime
change and must be gated.

**Verification (MANDATORY).** `cargo fmt --check`; focused `session_memory`,
then `cargo test --workspace` for reference and host contracts. Watch an
unclassified-field fixture fail compilation; restore it and show the normal
schema green. Run 5.3 if any production traversal/arithmetic changes; report
all current model-value changes rather than retuning limits.

**Hard rules.** Common envelope applies. Walk at every armed checkpoint stays.
Do not resolve an unapproved overflow/metadata policy by implementation.

**Report format.** Common four items, including exact preserved-model tests,
all arithmetic differences and the new field-classification barrier.

#### Draft B — prove mode routing without default-path cost

**Goal.** Produce the smallest working accounting-mode route which can carry
an armed operation through recursive evaluation, a builtin callback, a host
mutation, and a suspended/resumed continuation without adding default-path
work. This is a feasibility gate; do not switch ceilings to a partial ledger.

**Context pointers.** Sections 1, 3.1 and 5.3; `R/metadata.rs:332`,
`R/eval_dispatch/mod.rs`, `R/eval_core/mod.rs`, `R/resolution.rs`,
`R/session/session_helpers.rs`, `R/debug_tools/resources/constraints.rs`,
`R/suspension/core.rs`, existing builtin adapters and `H/session.rs`.

**Scope.** MAY modify these files and small new accounting-mode modules under
`R/`; the required `R/mod.rs`/`R/debug_tools/resources.rs` declarations;
`R/builtin_adapters.rs`, `R/session/session_api.rs`, `R/model/session.rs`,
`R/session/session_state.rs`; one representative Table wrapper in
`S/control_flow.rs` and its catalog registration in `S/catalog/control.rs`;
corresponding runtime/stdlib/host unit tests and
`crates/rum-host/tests/suspension_abandonment_contract.rs`.
MUST NOT migrate unrelated builtin families, cut over `current_memory_proxy_bytes`,
or change public handler/frame API shape unless the director explicitly
includes the selected adapter design in this rewritten brief. Existing
unarmed signatures must remain available if compatibility is required.

**Deliverable.** Mode route plus a small, representative shadow counter for
stack push/pop and one builder/continuation boundary, including transitions
into/out of nested local limits and suspended host entry. A concrete adapter
API/ripple inventory and compiled-code/instruction evidence that unused
accounting erases from unarmed operations. If the constraint fails, report a
refutation and leave correct walking in place; do not generalize the failed
prototype across the tree.

**rum / Rust pitfalls (read twice).** A generic outer evaluator that calls an
erased `fn(&mut Session, ...)` has lost the mode. Host entry while suspended
and MemoryConstrained inside an unarmed call are transitions too. Safe
accounting guards must not borrow Session across guest evaluation. Layout and
instruction-cache regressions count even if a source branch disappeared.

**Verification (MANDATORY).** `cargo fmt --check`; focused recursion/local-limit/
suspend/resume/host-callback tests; `cargo test --workspace`; both 5.3 A/B gates.
Include unarmed scaled slopes and cold-arm cost. The production walker still
decides every memory limit, so this prototype cannot create an unchecked gap.

**Hard rules.** Common envelope applies. No per-constructor global/TLS
tracking or pointer-identity cache as a shortcut. A default regression voids
this route. Stop for a scope/API decision before touching excluded files.

**Report format.** Common four items, plus the explicit proceed/refute verdict,
actual mode crossings, compatibility impact and remaining routing work.

#### Draft C — maintain persistent session owners in shadow mode

**Goal.** Maintain exact per-owner contributions for persistent definitions,
metadata, indexes, caches, namespace/package state and providers through every
write/release in section 3, while the independent full walk remains the
production ceiling implementation.

**Context pointers.** Accepted A/B reports; sections 3.1–3.2; the pure oracle;
`R/definitions/`, `R/model/definitions/`, `R/model/cache.rs`,
`R/evaluation_cache/`, `R/resolution.rs`, `R/session/`.

**Scope.** MAY modify accepted accounting modules; `R/model/session.rs`,
`R/session/session_state.rs`, `R/session/session_api.rs`,
`R/session/session_helpers.rs`; `R/definitions/`, `R/model/definitions.rs`,
`R/model/definitions/`, `R/model/cache.rs`, `R/attributes.rs`,
`R/resolution.rs`, `R/eval_core/stability.rs`, `R/eval_core/symbol.rs`,
`R/evaluation_cache/`, `R/options/core.rs`, `R/namespace_tools/syntax.rs`,
`R/messages/message_catalog.rs`, `R/package_tools.rs`,
`R/package_tools/resume_frames.rs`, `R/scoping/packages.rs`,
`R/scoping/unique.rs`, `R/scoping/gensym_gc.rs`,
`R/kernel_data/association_data.rs`, `R/kernel_data/association_lvalue.rs`,
`R/kernel_data/control_data/exceptions.rs`, `R/builtins_context_loader.rs`,
and owning runtime tests. MUST NOT cut over limits or enable armed in-place
mutation fast paths. Common exclusions apply.

**Deliverable.** Encapsulated accounted-owner APIs, exact capacity/root deltas,
read-side lazy-index coverage, install/clear/replace/evict/drop handling,
per-owner equality checks, and a table marking every covered field and every
remaining uncovered field. Include accounting metadata in the oracle. Bulk
loader restore recomputes actual restored contributions. Adding one rule must
not re-walk unrelated rules/definitions.

**rum / Rust pitfalls (read twice).** Handle entry/get_mut/retain and OnceLock
mutation, not only insert. Clear retains capacity. Index clones can be colder
than the source. Do not keep extra Arc/Weak handles to cache weights; do not
hold mutation transactions across evaluator reentry.

**Verification (MANDATORY).** `cargo fmt --check`; focused definition/cache/
association/options/package tests and per-field consistency sequences;
`cargo test -p rum-runtime`, followed by `cargo test --workspace` for cross-crate
API effects. Observe omitted-charge, omitted-eviction and capacity faults red,
then green. Run both 5.3 gates; highlight definition-size and cache-hit slopes.

**Hard rules.** Common envelope applies. Untouched/uncovered fields continue
to be covered by the production walk. Do not treat partial totals as an
admission bound. Preserve existing proxy omissions unless separately approved.

**Report format.** Common four items plus exact covered/uncovered schema rows,
mutation API escape audit, and cold bulk-restore costs.

#### Draft D — complete dynamic roots and every lifetime exit

**Goal.** Complete the shadow sum across all remaining Session fields,
suspension ownership, response transfer, scope restoration, panic invalidation
and reset. Equality must hold at every legitimate observation boundary before
production can use the sum.

**Context pointers.** Accepted A–C reports; section 3.2 in full;
`R/suspension/core.rs`, `R/suspension/trace.rs`, `R/session/`,
`R/debug_tools/`, `R/scoping/lexical/`, `H/session.rs` and host lifecycle tests.

**Scope.** MAY modify accepted accounting modules; `R/model/session.rs`,
`R/session/`, `R/eval_core/mod.rs`, `R/eval_core/symbol.rs`,
`R/definitions/application/mod.rs`, `R/debug_tools/`, `R/scoping/lexical/`,
`R/options/patterns.rs`, `R/kernel_data/control_data/branching_loops/iteration/`,
`R/kernel_data/control_data/check_quiet.rs`,
`R/kernel_data/control_data/confirmation.rs`, `R/builtins_input.rs`,
`R/builtins_context_loader.rs`, `R/messages/builtins_messages_inspect.rs`,
`R/suspension/`; owning runtime tests; `H/session.rs`, host tests under
`H/tests/`, and `crates/rum-host/tests/suspension_abandonment_contract.rs`.
MUST NOT use the shadow total for production limits or introduce unrelated
control-flow changes. Common exclusions apply.

**Deliverable.** All 52 original Accounted rows plus new bookkeeping rows
maintained with no uncovered production writer; capacity-correct stack/frame/
worklist moves; cancellation/timeout/normal/error release; actual-state bulk
rollback; generation-safe invalidate/reseed after panic; clear reset/drop
ownership. Keep pure frame tracing independent. Record rules for custom
continuations and test the supported extension boundary.

**rum / Rust pitfalls (read twice).** A pending frame remains a root during
its callback. Abandonment can evaluate cleanup and suspend again. A moved
stack changes shell policy; a cloned prompt is another occurrence. Panic
poison does not imply all runtime state was restored. RAII drop must not run
guest callbacks or subtract a new session's generation.

**Verification (MANDATORY).** `cargo fmt --check`; focused frame-root,
repeated abandon/rearm, effect-rollback, panic/reset and generation tests;
`cargo test --workspace`; both 5.3 gates. Watch missing release, double transfer,
wrong rollback and panic-validity faults fail and then pass. Keep existing
abandonment allowances and realistic memory-limit tests unchanged.

**Hard rules.** Common envelope applies. No “clear the ledger to zero” recovery
while owners survive. No sum cutover until exhaustive coverage and all
falsifiers are green. Any unaccounted custom mutation path blocks cutover.

**Report format.** Common four items plus a closed 52-field/added-field matrix,
exit-path evidence, and any extension/API limitation that remains.

#### Draft E — switch armed checks to the validated sum

**Goal.** Replace the repeated session traversal in armed expression/projected
checks with the validated occurrence sum. Preserve checkpoint frequency,
expression contribution, peak observation policy, nested minimums and existing
termination/fallback behavior.

**Context pointers.** Accepted A–D reports; sections 1, 4 and 5;
`R/debug_tools/resources/constraints.rs`, `memory.rs`,
`R/session/session_helpers.rs`, `R/resolution.rs`, `R/eval_core/mod.rs`.

**Scope.** MAY modify these files, the accepted accounting modules, required
module declarations, focused runtime/stdlib resource tests under
`R/tests/session_memory*`, `R/tests/session_requests/`,
`S/tests/debug_tools_resources.rs`, `S/tests/incidental_debug_tools_resources.rs`,
`S/tests/preflight_memory.rs`, host resource tests under `H/tests/`,
`docs/guide/resource-limits-and-sandbox.md`, the memory-entry documentation in
`R/builtin_catalog/debug.rs`, and regenerated `docs/reference/builtins.md`.
MUST NOT add builder coverage yet, remove the independent walker or weaken
existing ceilings. Common exclusions apply.

**Deliverable.** Correct seed/rearm lifecycle; O(1) valid armed session sum;
unchanged `S + W(expr)` and `S + projected` formulas; explicit unarmed
MemoryInUse/MaxMemoryUsed fallback to the walk; compile-out verifier; measured
metadata overhead and armed instruction scaling.

**rum / Rust pitfalls (read twice).** Do not double-count W(expr) accidentally
or remove its intentional overlap with session roots. Keep time check before
memory check. No reseed at every nested limit or checkpoint. Overflow-invalid
state may never wrap into admission. Preserve peak updates at observations.

**Verification (MANDATORY).** `cargo fmt --check`; the old stride falsifier
and projection control green; all field/owner/fault tests; `cargo test --workspace`;
both 5.3 gates against the correct pre-cutover walker build. Demonstrate that
repeated checks no longer scale with an unchanged retained list, while normal
completion and unarmed slopes remain intact. Include cold-arm/rearm results.

**Hard rules.** Common envelope applies. No stride, stale approximate sums,
unclassified accounting storage, or fallback to unchecked execution on ledger
failure. A failed gate keeps the walker in production.

**Report format.** Common four items plus exact checkpoint formulas before/
after, unarmed/armed deltas, and any remaining work before builder coverage.

#### Draft F — builder protocol, Table/history/String pilots

**Goal.** Add a safe accounted-builder protocol and prove it on Table, one
resumable history collector, and StringReplace. Track local retained growth
which S cannot see, with complete error/drop/suspend/publication transfers.
This deliberately extends modeled coverage and must be documented as such.

**Context pointers.** Accepted A–E reports; sections 1, 3.3 and 4;
`R/metadata.rs:346`, `R/eval_dispatch/mod.rs:253`;
`R/kernel_data/control_data/branching_loops/iteration/`,
`S/functional_ops/iteration.rs`, `S/strings/patterns/` and their continuations.

**Scope.** MAY modify accepted accounting modules and their runtime public
facade; `R/debug_tools/resources/`, `R/metadata.rs`,
`R/eval_dispatch/mod.rs`, `R/kernel_data/control_data/branching_loops/iteration/`,
`R/suspension/` transfer adapters, `S/control_flow.rs`,
`S/functional_ops/iteration.rs` and its resume modules,
`S/strings/patterns/search.rs`, `S/strings/patterns/search/`,
`S/strings/patterns/search/guarded/`; `S/catalog/memory_growth_contract.rs`, owning
runtime/stdlib tests, the resource-limit guide, matching builtin catalog
summaries and regenerated builtin reference. MUST NOT migrate unrelated
families, count unique allocations or remove the S oracle. Common exclusions
apply; director names any additional facade file in the rewritten scope.

**Deliverable.** Generation-bound builder ownership and separate R total;
reserve/check/reconcile/append/release/transfer operations; independent builder
payload verifier; nested-builder and callback-safe handling; fully closed
pilot obligations including fast, slow, invalid-input, error and resume paths.
If a current exact precheck already suffices, prove and record that disposition
rather than adding a redundant token. Document admission timing and the
single-construction/transient limitation.

**rum / Rust pitfalls (read twice).** Reserve the next known contribution,
not the cumulative prefix again. Include live sibling builders. Unknown
callback-result size cannot be prechecked honestly; check before retaining it
and starting the next iteration. Release R only when the actual occurrence
leaves or transfers, and do not erase real duplicate roots. Allocator spare
capacity must be reconciled after reserve. A token cannot borrow Session
through evaluation/suspension.

**Verification (MANDATORY).** `cargo fmt --check`; constrained long Table,
history and replacement outputs where previous local results stay live;
nested builder, throw/error, panic, suspension, abandonment and final
publication tests; omitted-progress and transfer-fault red/green evidence;
`cargo test --workspace`; both 5.3 gates. Verify unarmed wrappers erase and
armed work grows with added children rather than repeated prefix length.

**Hard rules.** Common envelope applies. Do not call ordinary evaluator
checkpoints proof of local-builder coverage. No release-token double-counting
of a single published occurrence. Stop on an unresolved transfer API rather
than leaking/deleting its charge.

**Report format.** Common four items plus S/R ownership diagrams or transition
tables, pilot branch receipts, exact fail-before-next-iteration evidence and
documented coverage changes.

#### Draft G — functional, association, structure, ordering and lazy builders

**Goal.** Discharge every remaining `Incremental` declaration in the functional,
association, structure/ordering and lazy catalog families using F's protocol,
with one explicit disposition and branch inventory per declaration.

**Context pointers.** Accepted F report; section 3.3's entry-point table;
`S/catalog/functional.rs`, `associations.rs`, `associations_transform.rs`,
`structure.rs`, `structure_ordering.rs`, `lazy.rs`; corresponding implementations.

**Scope.** MAY modify `S/functional_ops/`, `S/associations/`,
`S/structure_ops/`, `S/lazy.rs`; `S/catalog/functional.rs`,
`S/catalog/associations.rs`, `S/catalog/associations_transform.rs`,
`S/catalog/structure.rs`, `S/catalog/structure_ordering.rs`, `S/catalog/lazy.rs`,
and `S/catalog/memory_growth_contract.rs`; accepted accounted-builder modules
only for a demonstrated missing operation; owning stdlib tests under `S/tests/`
and runtime transfer tests under `R/tests/suspension*`.
Matching builtin documentation and regenerated reference are allowed
only where the contract changes. MUST NOT edit unrelated families, expression
representation or evaluator behavior. Common exclusions apply.

**Deliverable.** Per-declaration table linking every output/history/group/
work-list accumulator, helper, fast path and continuation to a charged API,
precheck proof or justified classification; no residual bare retained-growth
mutation on an armed path. Update vector/hash/string capacities incrementally;
release local values on every exit and transfer suspended builders exactly
once. Retain the declaration census as a required test artifact rather than
an unchecked prose claim.

**rum / Rust pitfalls (read twice).** Shared children remain multiple modeled
occurrences. Callback group keys/results, seen/cycle sets and output histories
can be simultaneously live. Selection/non-amplification of input structure
does not prove that auxiliary builder storage is absent. Treat recursive and
iterative/resumable implementations separately.

**Verification (MANDATORY).** `cargo fmt --check`; representative low and
realistic budgets for every distinct builder shape, malformed/error/callback/
resume/abandon exits and consistency sequences; `cargo test --workspace`;
both 5.3 gates with the modified families' hot/same-tier neighbors. Observe at
least one omitted progress and one missed release fault red, then green.

**Hard rules.** Common envelope applies. No blanket reclassification, post hoc
end-of-call-only check or test threshold weakening. Stop if a needed file is
absent from the rewritten scope; do not guess a sibling file's authorization.

**Report format.** Common four items plus closed declaration names and exact
remaining obligations. Include per-added-element instruction scaling.

#### Draft H — rewrite, strings, algebra, schema and parser builders

**Goal.** Discharge `Incremental` in rewrite, string matching/text, calculus,
polynomial, schema and ToExpression construction, including pure Rust loops
which do not reenter the evaluator between allocations.

**Context pointers.** Accepted F report; section 3.3 entry points;
`S/catalog/rewrite.rs`, `strings_match.rs`, `strings_text.rs`, `calculus.rs`,
`polynomials.rs`, `schema.rs`; `R/kernel_data/rewrite_data/`.

**Scope.** MAY modify `S/rewrite_builtins.rs`, `R/kernel_data/rewrite_data/`,
`S/strings/patterns/`, `S/strings/text/`, `S/calculus.rs`, `S/polynomials/`,
`S/schema.rs`; `S/catalog/rewrite.rs`, `S/catalog/strings_match.rs`,
`S/catalog/strings_text.rs`, `S/catalog/calculus.rs`, `S/catalog/polynomials.rs`,
`S/catalog/schema.rs`, `S/catalog/memory_growth_contract.rs`; owning tests under
`R/tests/` and `S/tests/`; accepted accounted-builder modules only for an API
extension with a concrete need. Documentation scope is
the matching builtin entries and regenerated reference. MUST NOT change
syntax/lowering semantics, dependencies, matcher semantics, Symbol/interner,
or snapshot format. Common exclusions apply.

**Deliverable.** Per-declaration and per-loop construction/release receipts;
accounted intermediate/output containers or sufficient exact projection;
consistent replacement-string reservation; preserved parser/rewrite error and
suspension behavior. A whole-result calculation with a known bound can use
preflight with a proof; a data-dependent collector must show progress checks.
Retain syntax trees outside the modeled contract unless a separately named
coverage extension is approved.

**rum / Rust pitfalls (read twice).** A small AST/rule input can build a much
larger output. No callback does not mean no growth. Shared rewrite subtrees
are occurrences; temporary replacement/history structures can coexist.
Unknown parse result size is not a reason to invent a physical bound or
silently change parser limits.

**Verification (MANDATORY).** `cargo fmt --check`; owning focused tests and
builder oracle sequences across success, error, malformed pattern/schema/
source and supported suspension paths; `cargo test --workspace`; both 5.3
A/B gates including hot matcher/rewrite neighbors. Watch an omitted string
capacity charge and one retained intermediate/result charge fail, then pass.

**Hard rules.** Common envelope applies. No vendor comparison or mathematical
compatibility oracle. Keep rum's documented behavior and preflight tests;
do not drop a large intermediate early solely to fool the accounting test.

**Report format.** Common four items plus complete declaration coverage,
projection assumptions and any intentional model-coverage extension.

#### Draft I — namespace, state, diagnostics, reports and kernel collectors

**Goal.** Close the remaining runtime-owned declaration obligations: value
and metadata readers/writers, namespace/package outputs, diagnostics/trace/
reports, Reap rendering, control aggregates, Print and symbol variables.
Use already-accounted session mutations where sufficient; cover local output
builders separately.

**Context pointers.** Accepted C–F reports; all `R/builtin_catalog/` entries in
section 3.3; `R/debug_tools/`, `R/namespace_tools/`, `R/definitions/`,
`R/package_tools*`, `R/eval_core/symbol.rs`, control aggregate and exception
entry points. These declarations often use raw held arguments or adapters.

**Scope.** MAY modify `R/attributes.rs`, `R/builtin_adapters.rs`,
`R/builtins_context_loader.rs`, `R/builtins_input.rs`, `R/debug_tools/`,
`R/definitions/`, `R/eval_core/symbol.rs`,
`R/kernel_data/control_data/branching_loops/iteration/`,
`R/kernel_data/control_data/branching_loops/iteration_aggregate.rs`,
`R/kernel_data/control_data/exceptions.rs`, `R/kernel_data/structure_data/`,
`R/messages/builtins_messages.rs`, `R/messages/builtins_messages_inspect.rs`,
`R/namespace_tools/`, `R/options/`, `R/package_tools.rs`, `R/package_tools/`,
`R/scoping/packages.rs`, `R/scoping/unique.rs`, `R/tree_walkers/`,
`R/builtin_catalog/debug.rs`, `R/builtin_catalog/namespace/`,
`R/builtin_catalog/rules.rs`, `R/builtin_catalog/scoping.rs`, `S/control_flow.rs`,
`S/catalog/control.rs`, `S/catalog/memory_growth_contract.rs`, and accepted
accounted-builder/transfer modules. Owning tests under `R/tests/`, `S/tests/`,
`H/tests/`, matching builtin documentation and generated reference are allowed.
MUST NOT change
package/sandbox/effect semantics, snapshot format, or excluded common areas.

**Deliverable.** A closed disposition for every remaining name, including the
four `$` variables; accounted diagnostic/report/definition-rendering outputs
and local Reap results; correct split/take/clone transfers into responses.
Demonstrate which stateful operations are fully covered by S and which still
need R. Generated namespaces/global intern storage remain under the existing
model and the parked quota ruling.

**rum / Rust pitfalls (read twice).** Do not validate held forms against their
raw arguments as though those were evaluated output bounds. Print sinks and
buffered prints have different retention. Rendering definitions/messages can
duplicate roots. Clear/Remove may add generation state. ValueQ rollback must
restore effects without hiding real changed cache capacity.

**Verification (MANDATORY).** `cargo fmt --check`; exact ByteCount/MemoryInUse/
MaxMemoryUsed and resource tests; scoped diagnostics/trace/report/Reap/package
success/error/suspend/abandon tests; `cargo test --workspace`; both 5.3 gates
with the affected definition/diagnostic neighbors. Observe missed diagnostic
builder and release faults fail, then pass. Preserve sandbox capability tests.

**Hard rules.** Common envelope applies. No orphan declarations or reclassify-
to-Delegated escape without an identified accountable callee on every branch.
Do not use arbitrary host-closure allocation as an unreviewed expansion of
this structural model.

**Report format.** Common four items plus the full remaining-name closure
matrix and evidence for S-only versus S-plus-R dispositions.

#### Draft J — close the inventory and run the line's acceptance gate

**Goal.** Establish that the incremental session line preserves the approved
proxy and that every current Incremental declaration has a checked
implementation. Produce the final armed/unarmed evidence and documented limits;
do not treat green unit tests alone as completion.

**Context pointers.** All accepted reports A–I; this report's field and
165-declaration censuses, test matrix, falsifier design and cost recipe;
`docs/testing-strategy.md`, `docs/guide/resource-limits-and-sandbox.md`,
`docs/host-embedding-and-hardening.md`.

**Scope.** MAY modify only accounting/builtin-coverage and lifecycle tests in
`R/tests/`, `S/tests/`, `H/tests/` and
`crates/rum-host/tests/suspension_abandonment_contract.rs`; approved
accounting documentation in the two named contract docs, memory/catalog
summaries and regenerated builtin reference; append the assigned report.
MUST NOT silently fix implementation outside this verification scope. Report
any implementation defect for a narrowly scoped follow-up before acceptance.
Common exclusions apply.

**Deliverable.** Regenerated source-derived field/declaration inventory with
no missing or duplicate disposition; compile-failure evidence for an
unclassified field and a failing coverage gate for a new, undisposed
Incremental declaration; independent equality,
progress, fault-injection and lifecycle results; actual bookkeeping sizes;
final per-label performance tables; a statement of which limits remain
structural and where single-construction/transient allocation is unbounded.
Keep the independent walker and builder payload oracle in the repository.

**rum / Rust pitfalls (read twice).** Coverage tests must fail when a new
obligation is added. An exempt field needs a real fixed-size/ownership reason,
not “not currently instrumented.” A warm-cache delta is not a leak test unless
compared with matched execution. A faster early-aborted run is not a speedup.

**Verification (MANDATORY).** `cargo fmt --check`; `cargo test --workspace`;
all watched-bad-then-good falsifiers; both 5.3 performance gates against the
correct unstrided pre-line base and per-step baselines where needed. Publish
all hot/same-tier regressions and cold-arm costs. No acceptance if any mutation
can silently bypass accounting, any declaration lacks a proof/test, or an
unarmed regression is reproducible.

**Hard rules.** Common envelope applies. No sampling, relaxed margin,
changed model or removed oracle to obtain green. No release/tag/commit action.
The director runs the authoritative acceptance gate afterward.

**Report format.** Common four items, followed by a concrete achieved/not-
achieved verdict against every requirement, remaining limitations, and the
location of all reproducible evidence.

#### Decisions for the director before rewriting/dispatch

1. **Confirm the proposed split/order:** occurrence-based session sum first,
   then local retained builders. This follows the existing contract; changing
   to unique/physical counting is a separate explicit model ruling and
   invalidates this implementation plan.
2. **Set overflow semantics in A:** recommend conservative saturation of all
   structural sums with a non-lossy or explicitly invalid overflow state for
   releases. Current inner usize sums do not consistently implement that.
   No impossible-size allocation is needed to test the arithmetic.
3. **Approve counted bookkeeping:** recommend that all new inline/heap ledger
   storage be represented in the proxy, with measured byte overhead and
   documented finite threshold effects. Do not silently exempt it.
4. **Set B's API compatibility boundary:** decide which mode-aware builtin/
   continuation/host-mutation adapter changes may be made. Require immutable
   reported continuation payloads or a routed mutation protocol. If a route
   cannot preserve public extension behavior and zero default cost, stop the
   prototype and keep correct walking; do not accept unobserved drift.
5. **Approve F's coverage extension:** include local retained builder debt in
   admissions and publish when transfer removes it. Preserve the existing S
   oracle separately; do not pretend the extension is numerically identical
   to the old S-plus-expression-only observation.

Separate existing contract discrepancy to review: the guide's local
MemoryConstrained paragraph (`docs/guide/resource-limits-and-sandbox.md:318`)
says the consumed constraint returns with `General::memlim`, whereas the local
handler at `R/debug_tools/resources/constraints.rs:262` visibly returns the
fallback or `$Aborted` without emitting that message. The enclosing behavior
deserves a focused contract check before changing it; this source observation
is not a new message-behavior probe. Do not weaken the guide or add/remove
messages incidentally in the ledger cutover. It is separate from the measured
ceiling and occurrence-model findings.

These decisions do not reopen Symbol, stride/adaptive policy, the parked
interner quota, or the later snapshot-format line. No user confirmation or
implementation authorization was needed to complete this design-only brief.
The recommendation is rejected if the prototype shows unavoidable default
cost, if meaningful mutation/extension paths cannot be covered without silent
drift, or if repeated unchanged-root checks still scale with session size.
In that event the correct armed walk remains, with its measured cost stated
as a property. No incremental speedup is claimed by this report.

#### Design-only completion check

Final report validation (before appending this evidence block):

```text
ORIGINAL_BRIEF_PREFIX_UNCHANGED d1e4b393f448be2314e436c5dca4cc6ada3af441274bf4c68d7fbf95279137a4
GIT_INDEX_UNCHANGED 9e13d3008c2e828bbbb8961948171a0fb3e1de6362931235bd416f0bde0b8630
REPORT_SECTIONS 6; TEMPLATE_SHAPED_DRAFTS 10; DECLARATIONS 165
CITED_PREFIXED_PATHS_EXIST; CITED_LINES_WITHIN_FILES; CODE_FENCES_BALANCED
PROBE_RECORDS 30; UNEXPECTED_NONZERO_EXITS 0
```

`git diff --check` exited 0 with no output. The only tracked modification was:

```text
 M docs/dev/codex/brief-516-incremental-accounting-design.md
```

No implementation was started, no full build/test suite was claimed, no git
state was changed, and no decision was silently made on behalf of the director.
The original brief prefix is byte-for-byte intact. Scratch stayed in
`target/brief-516/`; earlier artifacts were only read/executed.
