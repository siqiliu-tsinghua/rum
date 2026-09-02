# brief-414 — design: two accounting gaps in the modelled memory ceiling

Shape: **design-only**. `Scope: NONE — no source change, not one file.`
Worker: **cdx (Codex)**.

## Goal

`eval_memory_limit` installs a modelled memory ceiling per request, and
`docs/guide/resource-limits-and-sandbox.md` states that a breach returns
`TerminatedEvaluation["MemoryLimit"]` in band. Two categories of allocated
bytes escape that ceiling: one the model never walks at all, one it walks too
late. Produce a reviewable plan that closes both — options with a
recommendation for the first, a complete inventory plus a structural fix for
the second — and draft the follow-up implementation briefs. **Write no code.**

## Context pointers

- Read `AGENTS.md` (operational notes, Testing Policy hard lines) first.
- `docs/guide/resource-limits-and-sandbox.md` — what the ceiling promises.
- `crates/rum-core/src/expr.rs:42-96` — the symbol interner.
- `crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:50` —
  `current_memory_proxy_bytes`, the model itself.
- `crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:9,139-186`
  — the stride, the strided check, the projected-bytes helper.
- `docs/dev/sealed-scoping-design.md` — prior art on symbol identity lifetime.

## Gap A — the symbol interner is outside the model and never shrinks

`symbol_interner()` (`expr.rs:47-55`) is a process-wide
`OnceLock<Mutex<SymbolInterner>>`. `intern_text` (`:57-70`) `Box::leak`s every
name that is not one of the two canonical statics, and `intern_symbol_text`
(`:80-96`) `Box::leak`s a `SymbolText` per (context, name) pair. Neither
`strings` nor `pairs` has a removal or clear path — grep the file.

`current_memory_proxy_bytes` (`memory.rs:50`) sums session-owned structures:
builtins, the association cache, contexts, the evaluation stack, definitions,
attributes and so on. It never mentions the interner, so those bytes are
invisible to `enforce_active_resource_limits_for_expr`
(`constraints.rs:145-159`), which is the only thing the ceiling consults.
`Session::reset` rebuilds the session and does not touch it either.

**Director-measured, so you do not have to re-derive it** (verify if you want
to, do not assume): under a `{Pure}`-only policy with a memory ceiling armed,
eight consecutive requests each interning 20 000 fresh symbol names took RSS
from 5.6 MB to 57.5 MB monotonically, and `reset()` returned only to 47.0 MB.
Every request stayed in band; no ceiling fired.

Note `intern_symbol_text:88` also `expect`s the pair count to fit in `u32`.

## Gap B — the ceiling is sampled every 64 evaluation points

`ACTIVE_MEMORY_LIMIT_CHECK_STRIDE = 64` (`constraints.rs:9`). The strided
branch (`:145-159`) does a full walk on every 64th armed check and only
decrements a counter in between. The comment above the constant claims the
interval is safe because "overshoot is bounded by the existing per-step
element, integer, and iteration budgets". **Those budgets bound counts, not
the bytes one step can add**, and a step whose output size is a multiple of
its input size is not bounded by any of them.

The discipline for this already exists and is applied in **18 call sites**:
`enforce_active_memory_limit_for_projected_bytes` (`constraints.rs:175`) is
called from `random.rs`, `combinatorial_lists.rs`, `list_plumbing.rs`,
`sequence/range.rs`, `arrays/generators.rs`, `sequence/select.rs`,
`iteration/spec.rs` and `strings/text/builtins.rs`. `StringRepeat`
(`strings/text/builtins.rs:664-668`) prechecks projected bytes *and* the
generated-element count. `Power` has a bit-length precheck of its own
(`rum-runtime/src/runtime/arithmetic/canonical.rs:99-136`), and
`IntegerWorkBudget` threads a budget through `combinatorics.rs` and
`number_theory.rs`.

Two operations that grow bytes multiplicatively in one step and pass through
none of it:

- `eval_string_join` (`crates/rum-stdlib/src/strings/text/builtins.rs:13-42`)
  takes no `session` at all — its first parameter is `_session` — and ends in
  a bare `pieces.concat()`. `s = s <> s <> s` triples per step.
- Big-integer `Times`. `x = x*x` squares per step; the `Power` precheck above
  does not cover it.

Reported (**not** director-reproduced): an 8 MB ceiling reached a 2.04 GB peak
RSS over 20 tripling steps before terminating in band, and a 64 MB ceiling
reached 1.19 GB by repeated squaring. Treat the mechanism as established and
the magnitudes as unverified.

## What I want back

### 1. Gap A — options, then a recommendation

Enumerate the ways to bring interned bytes under the ceiling. At least these,
and **say plainly if there is a fourth I have not named** — that is the most
valuable thing you can add here:

- **A1 — session-scoped interner.** Symbol text lives with the session and
  dies with it.
- **A2 — account and refuse.** Keep the interner process-global, but make its
  bytes visible to `current_memory_proxy_bytes` and refuse to intern past the
  ceiling.
- **A3 — reclaimable interner.** Reference-counted or epoch-based entries that
  a session teardown can release.

For each, answer concretely and with `path:line`:

- What the `Symbol` identity contract actually requires. Is `&'static str`
  load-bearing beyond convenience — where does the `'static` escape into a
  signature that cannot take a lifetime? How many such signatures?
- Size and `Copy`-ness of `Symbol` today, and whether the option changes them.
  This type is in every `Expr`; a size change is a hot-path change.
- Interaction with sealed scoping's bit-62 identities and `symbol_generations`
  (`docs/dev/sealed-scoping-design.md`). **Precedent: that line already ruled
  symbol identities session-scoped and numeric reuse across sessions sound.**
  Does A1 follow from that ruling or conflict with it?
- Interaction with snapshot / `restore_from_bytes`, and with `Send`/`Sync`
  across the `rum-jupyter` threads.
- Cross-crate reach: how many call sites, in which crates, per option.
- Hot-path cost. `Expr` construction is the hottest path in the tree. State
  what the implementation's microbench A/B would have to show, and which
  labels are the right ones.

Then **recommend one and argue for it**. Cost is not an argument here — this
project's standing ruling is that the best and most correct option wins, and
"398 call sites" describes work rather than counting against it. What does
count: correctness, semantic loss, whether the fix is structurally
unavoidable, and enforceability.

### 2. Gap B — the class, not the two instances

I do not want "add a precheck to `StringJoin` and to bigint `Times`". That is
the failure brief-388 made: it closed the two instances that had been *seen*
rather than the class, because diagnosis and repair both looked only at
recorded sightings. Deliver instead:

- **A complete inventory** of every operation reachable from source-language
  evaluation whose output byte count can be a multiple of its input byte count
  in a single evaluation step, and which does not pass a projected-bytes,
  element-count, or integer-work precheck. Strings, big integers, lists,
  associations, byte arrays — wherever it goes. Give the inventory as a table:
  `path:line`, operation, growth factor per step, which precheck (if any) it
  currently passes.
- **A structural proposal** that makes an omission from that inventory fail
  loudly rather than rely on someone remembering. The standard here is
  `FrameRootTrace` (`docs/dev/plan-364-frame-roots.md`): a missing traversal
  is a *compile error*, and an invariant test records every handwritten impl.
  A fix the compiler or a derive enforces beats one that relies on people
  remembering, even an order of magnitude more expensive. Say honestly if you
  think no such structure is available for this shape, and why.
- A separate answer on the stride itself: is 64 defensible once the
  multiplicative sites are guarded, should it be adaptive, or should the
  comment simply be corrected to state what it really guarantees? Back it with
  the microbench cost of a smaller stride if you argue for keeping it.

### 3. Sequencing

Are A and B one line of work or two? Which order, and does either change the
other's design? Say so explicitly.

### 4. Drafted implementation briefs

In `TEMPLATE.md` shape, one per step, each with its own Scope (MAY-modify file
list), Deliverable, Verification and Hard rules. These are raw material — I
rewrite them before dispatch — but draft them as usable briefs, not step
descriptions. **A MAY list must include a module root whenever a type it
exports changes**; that omission has cost this project eight stop-losses.

## Method requirements

- **Every claim carries `path:line`.** A claim without one is a guess and I
  will treat it as one.
- **Cross-check any script-derived count against live probes before reporting
  it.** A regex over this repository has produced a confidently wrong count
  three separate times (attributes counted 527 where the truth was 483; two
  extraction passes that silently missed multi-line and `pub(super)` entries).
  If you report "N call sites", say how you counted and what you checked it
  against.
- **State first, verbatim, what your environment can actually run.** If a test
  or build cannot run here, say so rather than reporting a result you did not
  observe.
- Say plainly where you reason from source rather than from observation.
- **A confident wrong answer is worse than "I could not establish this."**

## Hard rules

- **`Scope: NONE`. Change no file** — not source, not tests, not docs. You may
  read anything, run `cargo` and the release `rum-cli`, and build throwaway
  crates under `/tmp` that you delete afterwards. If you believe a file must
  change, that is a finding for the report, not an edit.
- Do NOT `git commit`, branch, push, checkout, stash, or touch git config. Do
  not touch `stash@{0}`. Reading history with `git log` / `git show` /
  `git diff` is expected.
- Touch nothing outside this repo except `/tmp` scratch you remove afterwards.
  No network. Pass `--offline` to every `cargo` command.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only. No vendor engine, oracle harness, or compatibility matrix.
- **No new `unsafe`** in any proposal for `rum-core` or `rum-runtime`. If you
  are convinced a design needs it, that is an open question for me, not a
  decision for you.
- **No default-path cost.** Any proposal that adds an unconditional per-call
  branch, allocation, or lock to `Expr` construction or the evaluation loop
  must say so in its own risks section, with the microbench labels that would
  prove or disprove it.
- Paste genuine verbatim output for anything you ran.

## Report format

1. Environment capability, verbatim.
2. Gap A: the options table, the answers above, the recommendation and its
   argument.
3. Gap B: the inventory table, the structural proposal, the stride answer.
4. Sequencing.
5. Drafted implementation briefs.
6. Risks and open questions — what you could not establish, and why.

---

# Report — 2026-09-02

## 1. Environment capability, verbatim

```text
/home/remi/dev/rum
rustc 1.85.0 (4d91de4e4 2025-02-17)
cargo 1.85.0 (d73d2caf9 2024-12-31)
Python 3.14.7
```

Release CLI:

```text
rum [--parse <expr>]
rum [--format <expr> [--line-width <columns>]]
rum [--eval <expr> [-- <arg>...]]
rum [--file <path> [-- <arg>...]]
rum [- [-- <arg>...]]
rum [--help]
rum [<path> [-- <arg>...]]
rum

With non-TTY stdin and no program argument, stdin is evaluated as a script.
With a program argument and non-TTY stdin, stdin is bound as Global`$ScriptInput.
Arguments after -- are bound as Global`$ScriptArgs.
Exit codes: 0 on success, requested Exit[n] status for Exit, 1 on syntax/load/evaluation failure.
```

Throwaway offline Rust representation probe:

```text
size_of::<Symbol>()=24
align_of::<Symbol>()=8
size_of::<Atom>()=32
size_of::<Expr>()=32
```

The same probe compiled `assert_copy::<Symbol>()`; `Symbol` is therefore currently `Copy`, matching its derive at [crates/rum-core/src/expr.rs:35](/home/remi/dev/rum/crates/rum-core/src/expr.rs:35).

Compiler-parsed direct-constructor inventory:

```text
compiler_unique_direct_constructor_sites=260
crate rum-core=14
crate rum-host=2
crate rum-runtime=195
crate rum-stdlib=49
api Expr::symbol=77
api Expr::system=11
api Symbol::from_name=43
api Symbol::new=69
api Symbol::system=48
api Symbol::with_identity=12
```

The probe label `Expr::system` denotes `Expr::system_symbol`. I cross-checked that count with a boundary-qualified live source search; five `SyntaxExpr::symbol` false positives were excluded, and the two internal `Self::new` wrappers at [crates/rum-core/src/expr.rs:128-137](/home/remi/dev/rum/crates/rum-core/src/expr.rs:128) were added explicitly:

```text
crate rum-core=14
crate rum-host=2
crate rum-runtime=195
crate rum-stdlib=49
qualified_calls=258
internal_symbol_wrappers=2
total=260
```

Scratch cleanup and final repository status:

```text
removed /tmp/rum414-symbol.0UmZRl
removed /tmp/rum414-growth-hits.txt
removed /tmp/rum414-callprobe.pF7U7n
removed /tmp/rum414-ast
removed /tmp/rum414-growth-index.txt
removed /tmp/rum414-eval-fns.txt
removed /tmp/rum414-alloc-sites.txt
## main...origin/main [ahead 32]
```

No repository file was changed. No repository test suite was run because the brief expressly requires `Scope: NONE` and no source change ([brief-414:182-185](/home/remi/dev/rum/docs/dev/codex/brief-414-memory-accounting-gaps.md:182)); the only compilation was the deleted `/tmp` representation/counting probe, run offline.

## 2. Gap A: options and recommendation

### Established contract

The current interner contains process-global `strings` and `(context,name)` pairs, leaking both arbitrary strings and `SymbolText` allocations without a removal path ([expr.rs:42-55](/home/remi/dev/rum/crates/rum-core/src/expr.rs:42), [expr.rs:57-96](/home/remi/dev/rum/crates/rum-core/src/expr.rs:57)). The pair ID uses `pairs.len()` and panics at `u32` exhaustion ([expr.rs:88](/home/remi/dev/rum/crates/rum-core/src/expr.rs:88)). The session memory walk covers session structures but has no interner term ([memory.rs:50-260](/home/remi/dev/rum/crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:50)); reset only replaces the inner session ([rum-host/src/session.rs:161-174](/home/remi/dev/rum/crates/rum-host/src/session.rs:161)).

The semantic key is text plus optional generated identity. Today global interning lets equality and hashing use `id + identity`, while ordering already falls back to textual `(context,name,identity)` ordering ([expr.rs:250-285](/home/remi/dev/rum/crates/rum-core/src/expr.rs:250)). Serialization likewise persists text plus optional identity, not the interner ID ([expr.rs:288-319](/home/remi/dev/rum/crates/rum-core/src/expr.rs:288)).

`&'static` is not part of the public symbol API:

- There are five private function signatures involving interned `'static` storage: `symbol_interner`, `intern_text`, `canonical_static_str`, `intern_symbol_text`, and `canonical_system_symbols` ([expr.rs:47-105](/home/remi/dev/rum/crates/rum-core/src/expr.rs:47)).
- The current representation also has three `'static` fields and two interner collection element types ([expr.rs:23-44](/home/remi/dev/rum/crates/rum-core/src/expr.rs:23)).
- Public `context()` and `name()` return ordinary borrows tied to `&self`, not `&'static str` ([expr.rs:140-146](/home/remi/dev/rum/crates/rum-core/src/expr.rs:140)).

What is load-bearing is a lifetime-free owning `Symbol`, not static storage. Expressions leave sessions in request responses ([requests.rs:89-134](/home/remi/dev/rum/crates/rum-runtime/src/runtime/model/requests.rs:89)), suspension frames must be `Send + Sync + 'static` ([suspension/core.rs:74-87](/home/remi/dev/rum/crates/rum-runtime/src/runtime/suspension/core.rs:74)), `HostSession` is moved across threads ([lifecycle_contract.rs:373-387](/home/remi/dev/rum/crates/rum-host/tests/lifecycle_contract.rs:373)), and Jupyter moves a session through a worker thread ([worker.rs:312-341](/home/remi/dev/rum/crates/rum-jupyter/src/adapter/worker.rs:312)). Owned `Arc` storage satisfies those contracts without leaking.

One additional public constraint is `Symbol::interned_id() -> u32`, documented as a dense shared text ID ([expr.rs:164-168](/home/remi/dev/rum/crates/rum-core/src/expr.rs:164)). Its sole live production consumer found by the audit feeds the stability Bloom filter ([evaluation_cache/stability.rs:18-24](/home/remi/dev/rum/crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:18)). A1/A3/A4 cannot silently turn that into a session-local value; it needs an explicit compatibility decision.

### Options

| Option | Identity and representation | Reset, snapshot, threads | Reach and hot-path cost | Verdict |
|---|---|---|---|---|
| **A1 — session-scoped interner** | Use `Arc<SymbolText>` plus optional identity and a stable content fingerprint. Equality becomes fingerprint fast-reject followed by text+identity comparison on collisions. A session-local dense ID alone is invalid because current equality assumes equal text has the same ID ([expr.rs:250-263](/home/remi/dev/rum/crates/rum-core/src/expr.rs:250)). A thin `Arc`, `u64` identity and `u64` fingerprint should remain 24 bytes on this target, but loses `Copy`; this must be measured rather than assumed ([expr.rs:35-39](/home/remi/dev/rum/crates/rum-core/src/expr.rs:35)). | Reset drops the session table; escaped `Expr` values retain only their referenced entries until those values are dropped. Textual restore re-interns names and re-registers sealed packages with fresh identities ([rum-host/src/session.rs:176-208](/home/remi/dev/rum/crates/rum-host/src/session.rs:176), [sealed-scoping-design.md:189-232](/home/remi/dev/rum/docs/dev/sealed-scoping-design.md:189)). Immutable `Arc` contents preserve the existing thread contracts. | All 260 direct constructor expressions require review: core 14, host 2, runtime 195, stdlib 49. The exact edit count cannot be established before a non-`Copy` prototype because dereference, `.copied()`, map-key and frame fallout is separate; examples already exist in sealed collection and builtin lookup ([packages.rs:16-20](/home/remi/dev/rum/crates/rum-runtime/src/runtime/scoping/packages.rs:16), [builtin_catalog.rs:75-92](/home/remi/dev/rum/crates/rum-runtime/src/runtime/builtin_catalog.rs:75)). Arc clone/drop replaces integer copying on the hottest representation path. | **Recommended**, with detached core symbols and a bounded canonical pool as described below. |
| **A2 — account and refuse globally** | Retains the current 24-byte `Copy` layout and process-global text IDs ([expr.rs:35-55](/home/remi/dev/rum/crates/rum-core/src/expr.rs:35)). It must replace the `u32` `expect` with a recoverable refusal ([expr.rs:88](/home/remi/dev/rum/crates/rum-core/src/expr.rs:88)). | Snapshot and threading remain unchanged, but reset still cannot release bytes ([rum-host/src/session.rs:161-174](/home/remi/dev/rum/crates/rum-host/src/session.rs:161)). | Reading a global byte total from the memory walk is mechanically small, but it cannot attribute bytes to the request/session that owns the ceiling. Charging the whole process table makes one tenant pay for another; charging only insertion deltas leaves permanent uncharged allocations after reset. A correct refusal therefore needs budget context at every symbol-producing ingress, requiring review of the same 260 sites; hidden TLS would add an unconditional constructor-path lookup. | Not acceptable as the per-request ceiling fix. It is only a separate process-wide quota. |
| **A3 — reclaimable global interner** | Replace leaked references with `Arc` values and global `Weak` entries. A monotonic ID or content fingerprint must replace `pairs.len()`, and equality must compare text on fingerprint collision ([expr.rs:80-96](/home/remi/dev/rum/crates/rum-core/src/expr.rs:80)). Expected 24 bytes if the fingerprint is inline; non-`Copy`. | Escaped values keep individual entries alive. Restore remains textual. `Arc` and a synchronized weak map satisfy current thread transfer requirements ([worker.rs:312-341](/home/remi/dev/rum/crates/rum-jupyter/src/adapter/worker.rs:312)). Static symbol-keyed caches can defeat reclamation: the canonical map and builtin map pin symbols ([expr.rs:98-105](/home/remi/dev/rum/crates/rum-core/src/expr.rs:98), [builtin_catalog.rs:85-92](/home/remi/dev/rum/crates/rum-runtime/src/runtime/builtin_catalog.rs:85)); diagnostic profile maps can also pin arbitrary symbols when enabled ([evaluation_cache/profile/mod.rs:219-264](/home/remi/dev/rum/crates/rum-runtime/src/runtime/evaluation_cache/profile/mod.rs:219)). | Constructor signatures may remain unchanged, so the known mandatory direct-call signature edit count is zero, but all 260 sites still need review and compiler-driven non-`Copy` repair. Every clone/drop gains Arc atomics, and global interning retains its mutex. Most importantly, per-session memory attribution remains ambiguous. | Better than leaking, but does not structurally implement a per-session ceiling. |
| **A4 — no interner** | Yes, there is a fourth option: every newly constructed textual symbol owns a fresh `Arc<SymbolText>` and equality/hashing operate on text+identity. It removes global state but loses `Copy`; it can keep the 24-byte proposed layout. | Reset and snapshot are naturally safe, and escaped values own their text. | Constructor APIs can remain unchanged, but all 260 sites still require review. Repeated `System` and ordinary-symbol construction duplicates allocation and loses deduplication; the current constructors are used by every `Expr::symbol`/`system_symbol` creation ([expr.rs:108-138](/home/remi/dev/rum/crates/rum-core/src/expr.rs:108), [expr.rs:801-810](/home/remi/dev/rum/crates/rum-core/src/expr.rs:801)). Adding a session factory to recover deduplication turns this into A1. | Correct but needlessly loses deduplication and predictable accounting. |

### Recommendation: A1, as a hybrid ownership boundary

Use these concrete rules:

1. `SymbolText` owns `String` context/name data behind `Arc`. `Symbol` stores a stable content fingerprint, `Option<NonZeroU64>`, and the thin `Arc`. Preserve the observed 24-byte size if the compiler layout permits it; never use `unsafe` to force that layout. Equality uses pointer equality as the common fast path, fingerprint as a reject, then full text+identity comparison; hashing uses the content fingerprint plus identity ([expr.rs:250-285](/home/remi/dev/rum/crates/rum-core/src/expr.rs:250)).

2. Keep a bounded process-static canonical pool for the fixed canonical core heads, replacing the current arbitrary global interner while retaining the intent of `canonical_system_symbols` ([expr.rs:98-105](/home/remi/dev/rum/crates/rum-core/src/expr.rs:98)). Arbitrary source names must never enter that pool.

3. Give each runtime `Session` a strong `(context,name) -> Arc<SymbolText>` map. Reset drops that map; an escaped response retains only entries actually reachable from it, rather than pinning the whole session arena ([requests.rs:89-134](/home/remi/dev/rum/crates/rum-runtime/src/runtime/model/requests.rs:89)). A single `Arc<SessionArena>` per symbol would be wrong because one escaped symbol would pin every name.

4. Preserve detached construction for `rum-core` and standalone lowering, because `lower_syntax_expr` is public and intentionally accepts only a resolver closure rather than a `Session` ([lowering.rs:7-18](/home/remi/dev/rum/crates/rum-runtime/src/runtime/lowering.rs:7)). Runtime request ingress imports/canonicalizes detached symbols into its session interner.

5. Account table capacity, owned strings, `SymbolText`, and Arc allocations in the session proxy, and preflight a missing-entry allocation before inserting it. Existing expression accounting already charges symbol text per occurrence ([memory.rs:283-288](/home/remi/dev/rum/crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:283)); the implementation must explicitly choose whether that conservative per-occurrence charge remains in addition to the once-per-interner-entry ledger.

6. Keep sealed identity allocation unchanged. Bit 62 remains the sealed partition, and sealed identities remain excluded from `symbol_generations` ([sealed-scoping-design.md:63-71](/home/remi/dev/rum/docs/dev/sealed-scoping-design.md:63), [definition_clear.rs:212-229](/home/remi/dev/rum/crates/rum-runtime/src/runtime/definitions/definition_clear.rs:212)). A1 follows—not conflicts with—the existing ruling that identities need only be unique inside a session and may be numerically reused across independent sessions ([packages.rs:420-429](/home/remi/dev/rum/crates/rum-runtime/src/runtime/scoping/packages.rs:420)).

7. Keep `interned_id() -> u32` source-compatible only if it is explicitly redefined as a stable text fingerprint rather than a dense allocation index; add a wider internal fingerprint for equality/hash. Its current “dense id” rustdoc and the Bloom consumer must change together ([expr.rs:164-168](/home/remi/dev/rum/crates/rum-core/src/expr.rs:164), [evaluation_cache/stability.rs:18-24](/home/remi/dev/rum/crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:18)). Silently returning session-local IDs is not acceptable.

### Required hot-path gate

A1 must show no regression in the construction, symbol lookup, equality, ordering, definition-dispatch and matcher neighbors:

- `literal-integer`, `literal-string`, `symbol-ownvalue-read`, `set-ownvalue`, `list-literal-small`, `sameq-list` ([core-language-microbench.sh:463-478](/home/remi/dev/rum/scripts/core-language-microbench.sh:463)).
- `module-local`, `block-symbol-only-repeat`, and the Table iterator rows ([core-language-microbench.sh:481-499](/home/remi/dev/rum/scripts/core-language-microbench.sh:481)).
- Exact/pattern downvalue hits and misses ([core-language-microbench.sh:502-510](/home/remi/dev/rum/scripts/core-language-microbench.sh:502)).
- Matcher and `replaceall-simple` rows ([core-language-microbench.sh:538-558](/home/remi/dev/rum/scripts/core-language-microbench.sh:538)).
- Symbol sorting at widths 4,000 and 16,000 and undefined-symbol lookup at 2,000 and 8,000 definitions ([core-language-microbench.sh:902-950](/home/remi/dev/rum/scripts/core-language-microbench.sh:902)).

Add one exact cold-miss label that interns unique names and one hit label that reconstructs already-interned names. Any regression on a target or same-tier neighbor remains a defect under the repository’s hot-path rule.

## 3. Gap B: inventory, structural fix, and stride

### Boundary and current guard coverage

For this audit, “output” includes the returned `Expr` and request/session-owned accumulators already intended to be walked by the memory model: definitions, prints, messages, traces, reaps and suspension state ([memory.rs:75-256](/home/remi/dev/rum/crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:75)). “Multiplicative” means payload repetition, Cartesian/product growth, encoding expansion, recursive substitution, or state duplication—not merely a fixed-size wrapper or a one-for-one reorder.

The model recursively charges `BigInt` limbs, string capacity and complete call subtrees ([memory.rs:269-319](/home/remi/dev/rum/crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:269)). Therefore an `Expr`-node-only projection is not conservative when a source `String`, `BigInt`, or subtree is copied repeatedly.

Legend:

- `Pnode`: projected-byte call exists, but only projects `size_of::<Expr>()` slots.
- `P`: payload-aware projected bytes.
- `G`: generated-element bound.
- `I`: iteration-work bound.
- `W`: integer-work bound.
- `post-G`: count checked only after construction.
- `none`: no projected/element/integer precheck.

The existing projected-byte calls total 18:

| File | Calls |
|---|---:|
| [random/password_policy.rs:107](/home/remi/dev/rum/crates/rum-stdlib/src/random/password_policy.rs:107) | 1 |
| [random.rs:494,570](/home/remi/dev/rum/crates/rum-stdlib/src/random.rs:494) | 2 |
| [strings/text/builtins.rs:608,665](/home/remi/dev/rum/crates/rum-stdlib/src/strings/text/builtins.rs:608) | 2 |
| [combinatorial_lists.rs:22,71,172,183,221,268](/home/remi/dev/rum/crates/rum-stdlib/src/structure_ops/combinatorial_lists.rs:22) | 6 |
| [list_plumbing.rs:127,181](/home/remi/dev/rum/crates/rum-stdlib/src/structure_ops/list_plumbing.rs:127) | 2 |
| [range.rs:38](/home/remi/dev/rum/crates/rum-stdlib/src/structure_ops/sequence/range.rs:38) | 1 |
| [select.rs:242,264](/home/remi/dev/rum/crates/rum-stdlib/src/functional_ops/sequence/select.rs:242) | 2 |
| [arrays/generators.rs:209](/home/remi/dev/rum/crates/rum-stdlib/src/functional_ops/arrays/generators.rs:209) | 1 |
| [iteration/spec.rs:338](/home/remi/dev/rum/crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/spec.rs:338) | 1 |
| **Total** | **18** |

Thirteen of those use node-only formulas: the combinatorial six, list-plumbing two, Range, Select two, Array/Outer and Table. Their helpers multiply only by `size_of::<Expr>()` ([combinatorial_lists.rs:754-756](/home/remi/dev/rum/crates/rum-stdlib/src/structure_ops/combinatorial_lists.rs:754), [list_plumbing.rs:366-368](/home/remi/dev/rum/crates/rum-stdlib/src/structure_ops/list_plumbing.rs:366), [range.rs:127-128](/home/remi/dev/rum/crates/rum-stdlib/src/structure_ops/sequence/range.rs:127), [functional_ops/mod.rs:149-151](/home/remi/dev/rum/crates/rum-stdlib/src/functional_ops/mod.rs:149), [iteration/spec.rs:419-420](/home/remi/dev/rum/crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/spec.rs:419)).

### Complete source-derived inventory

This is a static inventory against all 643 generated catalog entries ([builtins.md:1-5](/home/remi/dev/rum/docs/reference/builtins.md:1)), all catalog families registered by `install_core` ([rum-stdlib/src/lib.rs:42-138](/home/remi/dev/rum/crates/rum-stdlib/src/lib.rs:42)), and the non-catalog evaluator seams below. Grouped rows share one implementation pattern; individual heads must be recorded separately in the implementation invariant.

| `path:line` | Operation(s) | One-step growth | Current precheck |
|---|---|---|---|
| [canonical.rs:8-18,138-180](/home/remi/dev/rum/crates/rum-runtime/src/runtime/arithmetic/canonical.rs:8), [expr.rs:489-510](/home/remi/dev/rum/crates/rum-core/src/expr.rs:489) | `Plus`, `Subtract`, `Total`, `Mean`, `AddTo`, `SubtractFrom` | Rational denominator cross-products; update forms also retain/store the result | none |
| [canonical.rs:16-18,256-333](/home/remi/dev/rum/crates/rum-runtime/src/runtime/arithmetic/canonical.rs:16), [expr.rs:501-510](/home/remi/dev/rum/crates/rum-core/src/expr.rs:501) | `Times`, `Divide`, `TimesBy`, `DivideBy` | `x*x` doubles limb length; rational numerators and denominators multiply | none |
| [control_flow.rs:57-73](/home/remi/dev/rum/crates/rum-stdlib/src/control_flow.rs:57), [iteration/spec.rs:341-357](/home/remi/dev/rum/crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/spec.rs:341) | `Product`, `Sum` | Repeated arithmetic over `n` iterator values; result bytes can grow each iteration | I only |
| [window.rs:128-143,299-416](/home/remi/dev/rum/crates/rum-stdlib/src/functional_ops/sequence/window.rs:128), [list_plumbing.rs:4-34](/home/remi/dev/rum/crates/rum-stdlib/src/structure_ops/list_plumbing.rs:4) | `Accumulate`, `Differences` | Retained arithmetic prefixes/differences; large rational or integer payloads | none / post-G |
| [integer_ops.rs:64-91](/home/remi/dev/rum/crates/rum-runtime/src/runtime/arithmetic/integer_ops.rs:64), [random.rs:138-180](/home/remi/dev/rum/crates/rum-stdlib/src/random.rs:138) | `LCM`; weighted `RandomChoice` normalization | Product after GCD; common-denominator LCM across weights | none |
| [integer_digits.rs:138-168](/home/remi/dev/rum/crates/rum-stdlib/src/integer_digits.rs:138) | `FromDigits` | Horner accumulation gives `O(n log base)` output bits | none |
| [calculus.rs:16-46,131-157,198-225](/home/remi/dev/rum/crates/rum-stdlib/src/calculus.rs:16) | `D` | Product/abstract-call rules can generate `Θ(n²)` terms/factors | none |
| [polynomials/mod.rs:22-37](/home/remi/dev/rum/crates/rum-stdlib/src/polynomials/mod.rs:22), [polynomials/core.rs:118-155](/home/remi/dev/rum/crates/rum-stdlib/src/polynomials/core.rs:118) | `Expand` | Term Cartesian products and exponentiation | none |
| [polynomials/mod.rs:84-108](/home/remi/dev/rum/crates/rum-stdlib/src/polynomials/mod.rs:84), [polynomials/core.rs:191-228](/home/remi/dev/rum/crates/rum-stdlib/src/polynomials/core.rs:191) | `CoefficientList` | Dense grid `∏(degreeᵢ+1)` | none |
| [polynomials/mod.rs:110-125](/home/remi/dev/rum/crates/rum-stdlib/src/polynomials/mod.rs:110), [rational_function.rs:59-100](/home/remi/dev/rum/crates/rum-stdlib/src/polynomials/rational_function.rs:59) | `Together` | Numerator/denominator cross-products | none |
| [listable_dispatch.rs:65-129](/home/remi/dev/rum/crates/rum-runtime/src/runtime/listable_dispatch.rs:65), [hold_thread.rs:34-80](/home/remi/dev/rum/crates/rum-runtime/src/runtime/hold_thread.rs:34) | Implicit `Listable`, explicit `Thread` | `n ×` every non-threaded argument/head | none |
| [slot_substitution.rs:3-66](/home/remi/dev/rum/crates/rum-runtime/src/runtime/substitution/slot_substitution.rs:3), [binding_substitution.rs:47-159](/home/remi/dev/rum/crates/rum-runtime/src/runtime/substitution/binding_substitution.rs:47), [lexical_substitution.rs:365-427](/home/remi/dev/rum/crates/rum-runtime/src/runtime/substitution/lexical_substitution.rs:365) | `Function` slots, rule RHS bindings, `With`/lexical substitution | Occurrence count × bound value; `SlotSequence` splicing | none |
| [rewrite_builtins.rs:14-70](/home/remi/dev/rum/crates/rum-stdlib/src/rewrite_builtins.rs:14), [replace_all.rs:125-181](/home/remi/dev/rum/crates/rum-runtime/src/runtime/definitions/application/replace_all.rs:125) | `Cases`, `Replace`, `ReplaceAll`, `ReplaceAllUnheld`, `ReplaceList`, `ReplaceRepeated` | Matches/rules × substituted RHS; `ReplaceList` retains all results | only repeated-pass limit where applicable |
| [replace_part.rs:59-103,475](/home/remi/dev/rum/crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/replace_part.rs:59), [replace_at.rs:13-125](/home/remi/dev/rum/crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/replace_at.rs:13) | `ReplacePart`, `ReplaceAt` | Replacement payload × selected paths plus rebuilt parents | none |
| [selectors.rs:99-220](/home/remi/dev/rum/crates/rum-runtime/src/runtime/kernel_data/structure_data/selectors.rs:99), [selectors/apply.rs:9-30](/home/remi/dev/rum/crates/rum-runtime/src/runtime/kernel_data/structure_data/selectors/apply.rs:9) | `Part`, `Extract` | Repeated selectors × selected payload | none |
| [tree_walkers/level.rs:4-68,77-160](/home/remi/dev/rum/crates/rum-runtime/src/runtime/tree_walkers/level.rs:4), [tree_walkers/position.rs:101-199,588-596](/home/remi/dev/rum/crates/rum-runtime/src/runtime/tree_walkers/position.rs:101) | `Level`, `Position` | All subtrees can be `Θ(n²)` in proxy bytes on a chain; paths are `nodes × depth` | none |
| [list_plumbing.rs:37-52,102-187](/home/remi/dev/rum/crates/rum-stdlib/src/structure_ops/list_plumbing.rs:37) | `ConstantArray`, `PadLeft`, `PadRight` | Shape/product or padding count × source/padding payload | Pnode + G |
| [combinatorial_lists.rs:4-285](/home/remi/dev/rum/crates/rum-stdlib/src/structure_ops/combinatorial_lists.rs:4) | `Permutations`, `Subsets`, `Tuples`, `Partition` | Factorial/power/Cartesian/window count × element payload | Pnode + G |
| [range.rs:4-38,92-103](/home/remi/dev/rum/crates/rum-stdlib/src/structure_ops/sequence/range.rs:4) | `Range` | Count × BigInt endpoint/step limb width | Pnode + G |
| [arrays/generators.rs:84-217](/home/remi/dev/rum/crates/rum-stdlib/src/functional_ops/arrays/generators.rs:84) | `Array`, `Outer` | `∏dimensions` callback results; Outer concatenates all input shapes | Pnode + G + I |
| [iteration/spec.rs:312-360,384-420](/home/remi/dev/rum/crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/spec.rs:312) | `Table` | Iterator product × evaluated body payload | Pnode + G + I |
| [select.rs:219-268](/home/remi/dev/rum/crates/rum-stdlib/src/functional_ops/sequence/select.rs:219) | `Select` | Source item payload retained both in source and selected accumulator | Pnode + G |
| [random.rs:46-136,524-557](/home/remi/dev/rum/crates/rum-stdlib/src/random.rs:46) | Array forms of `RandomInteger`, `RandomChoice` | Count/shape × BigInt or selected element payload | G |
| [callable/head_ops.rs:4-75](/home/remi/dev/rum/crates/rum-stdlib/src/functional_ops/callable/head_ops.rs:4) | `Through` | Function count × original call arguments | none |
| [apply_map/mod.rs:13-163](/home/remi/dev/rum/crates/rum-stdlib/src/functional_ops/apply_map/mod.rs:13) | `Apply`, `Map`, `MapAll`, `MapIndexed`, `MapApply`, `Comap`, `ComapApply` | Node count × operator/index wrappers or retained callback result | none |
| [targeted_map.rs:149-293](/home/remi/dev/rum/crates/rum-stdlib/src/functional_ops/targeted_map.rs:149) | `MapAt`, `SubsetMap`, `MapThread` | Selected positions/zip length × callback result | none |
| [fold/mod.rs:21-181](/home/remi/dev/rum/crates/rum-stdlib/src/functional_ops/sequence/fold/mod.rs:21) | `Fold`, `FoldList`, `FoldPair`, `FoldWhile`, `SequenceFold`, `SequenceFoldList` | Callback recurrence; list forms retain every intermediate | none |
| [functional_ops/iteration.rs:47-249](/home/remi/dev/rum/crates/rum-stdlib/src/functional_ops/iteration.rs:47) | `Nest`, `NestList`, `NestWhile`, `NestWhileList`, `FixedPoint`, `FixedPointList` | Recurrent callback output; list forms retain all values | evaluation-pass/iteration control only |
| [window.rs:73-143,382-452](/home/remi/dev/rum/crates/rum-stdlib/src/functional_ops/sequence/window.rs:73) | `BlockMap` | Window count × block payload/callback result | none |
| [arrays/mod.rs:14-136](/home/remi/dev/rum/crates/rum-stdlib/src/functional_ops/arrays/mod.rs:14), [arrays/tensor.rs:353-575](/home/remi/dev/rum/crates/rum-stdlib/src/functional_ops/arrays/tensor.rs:353) | `Inner`, `Dot`, `ArrayFilter`, `ArrayReduce` | Tensor output dimensions × contraction/window terms | none |
| [sequence/insert.rs:13-58,175-213](/home/remi/dev/rum/crates/rum-stdlib/src/structure_ops/sequence/insert.rs:13) | `Insert` | Inserted payload × path count plus rebuilt parents | none |
| [collection_stats.rs:418-459](/home/remi/dev/rum/crates/rum-stdlib/src/structure_ops/collection_stats.rs:418), [take_list.rs:4-87](/home/remi/dev/rum/crates/rum-stdlib/src/structure_ops/sequence/take_list.rs:4) | `Riffle`, `TakeList` | Repeated separator payload; wrapper count × chunks | G / none |
| [collection_stats.rs:92-110,375-415](/home/remi/dev/rum/crates/rum-stdlib/src/structure_ops/collection_stats.rs:92), [associations/aggregate.rs:55-145](/home/remi/dev/rum/crates/rum-stdlib/src/associations/aggregate.rs:55) | `Tally`, `GatherBy`, `Counts`, `PositionIndex` | Per-element pair/rule/group/path wrappers and callback keys | none |
| [lazy.rs:489-512,551-588](/home/remi/dev/rum/crates/rum-stdlib/src/lazy.rs:489), [associations/basic.rs:475-495](/home/remi/dev/rum/crates/rum-stdlib/src/associations/basic.rs:475) | `LazyTuples`, `Normal` | Descriptor repeats the list `count` times; `Normal` realizes all retained results | none |
| [associations/transforms.rs:10-84](/home/remi/dev/rum/crates/rum-stdlib/src/associations/transforms.rs:10), [resumable/map.rs:6-80](/home/remi/dev/rum/crates/rum-stdlib/src/associations/resumable/map.rs:6) | `KeyMap`, `KeyValueMap`, `AssociationMap` | Entry count × function/callback result and rule wrappers | none |
| [associations/transforms.rs:127-216](/home/remi/dev/rum/crates/rum-stdlib/src/associations/transforms.rs:127), [associations/basic.rs:139-190,332-380](/home/remi/dev/rum/crates/rum-stdlib/src/associations/basic.rs:139) | `AssociationThread`, `Lookup`, `Keys`, `Values` | Scalar value/key/wrapper repeated over key or query list | none |
| [associations/grouping.rs:143-210](/home/remi/dev/rum/crates/rum-stdlib/src/associations/grouping.rs:143), [key_reshape.rs:62-103](/home/remi/dev/rum/crates/rum-stdlib/src/associations/key_reshape.rs:62), [resumable/aggregate.rs:8-105](/home/remi/dev/rum/crates/rum-stdlib/src/associations/resumable/aggregate.rs:8) | `GroupBy`, `KeyGroupBy`, `Merge`, `MergeNested`, `CountsBy` | Groups × retained values, rules and reducer results | none |
| [join_across.rs:80-145,214-277](/home/remi/dev/rum/crates/rum-stdlib/src/associations/join_across.rs:80) | `JoinAcross` | Matching-row Cartesian product | none |
| [key_collections.rs:81-220](/home/remi/dev/rum/crates/rum-stdlib/src/associations/transforms/key_collections.rs:81) | `KeyUnion`, `KeyIntersection` | Collection count × union/intersection keys; fill callback results | none |
| [key_reshape.rs:105-143](/home/remi/dev/rum/crates/rum-stdlib/src/associations/key_reshape.rs:105) | `AssociationKeyFlatten`, `AssociationKeyDeflatten` | Path components/rules × nested key depth | none |
| [strings/text/builtins.rs:13-42](/home/remi/dev/rum/crates/rum-stdlib/src/strings/text/builtins.rs:13) | `StringJoin` | Sum/repetition of all string payloads; bare `concat()` | none |
| [strings/text/builtins.rs:83-149,177-206](/home/remi/dev/rum/crates/rum-stdlib/src/strings/text/builtins.rs:83) | `StringRiffle`, `Row` | Separator/rendered item repeated by item count | none |
| [strings/text/builtins.rs:61-77](/home/remi/dev/rum/crates/rum-stdlib/src/strings/text/builtins.rs:61), [strings/text/case.rs:21-89](/home/remi/dev/rum/crates/rum-stdlib/src/strings/text/case.rs:21) | `ToUpperCase`, `ToLowerCase`, `Capitalize`, `Decapitalize` | Unicode case mapping may expand each scalar | none |
| [strings/text/builtins.rs:209-294](/home/remi/dev/rum/crates/rum-stdlib/src/strings/text/builtins.rs:209), [text_shape.rs:35-74,92-140,362-475](/home/remi/dev/rum/crates/rum-stdlib/src/strings/scripting/text_shape.rs:35), [decimal.rs:4-44](/home/remi/dev/rum/crates/rum-stdlib/src/strings/text/decimal.rs:4) | `StringPadLeft/Right`, `StringIndent`, `StringExpandTabs`, `StringWrap`, `AlignColumns`, `DecimalString` | Target width/line count × padding/prefix; decimal precision × digits | G at selected paths only |
| [strings/text/builtins.rs:335-435,521-583](/home/remi/dev/rum/crates/rum-stdlib/src/strings/text/builtins.rs:335) | `StringLines`, `TextWords`, `ToCharacterCode`, `CharacterCounts`, `Characters` | Per-character/token wrapper and atom overhead | none |
| [structure_ops/surface.rs:78-89](/home/remi/dev/rum/crates/rum-stdlib/src/structure_ops/surface.rs:78), [strings/text/builtins.rs:501-519,624-645](/home/remi/dev/rum/crates/rum-stdlib/src/strings/text/builtins.rs:501) | `FullForm`, `TextString`, `ToString` | Rendered/structural representation of every node, including escaping | none |
| [strings/template.rs:11-105,304-353](/home/remi/dev/rum/crates/rum-stdlib/src/strings/template.rs:11) | `StringForm`, `TemplateApply` | Placeholder occurrence count × rendered value | none |
| [strings/patterns/search.rs:51-127,198-373,455-475](/home/remi/dev/rum/crates/rum-stdlib/src/strings/patterns/search.rs:51) | `StringPosition`, `StringSplit`, `StringReplace`, `StringCases`, `StringApply` | Match count × position/substrings/replacement/callback output | none |
| [html.rs:24-84](/home/remi/dev/rum/crates/rum-stdlib/src/strings/scripting/html.rs:24), [shell.rs:35-49](/home/remi/dev/rum/crates/rum-stdlib/src/strings/scripting/shell.rs:35), [url.rs:6-23](/home/remi/dev/rum/crates/rum-stdlib/src/strings/scripting/url.rs:6) | `HtmlEscape`, `ShellQuote`, `UrlEncode` | Entity/quote/percent escaping: fixed payload multiplier | none |
| [shell.rs:11-33](/home/remi/dev/rum/crates/rum-stdlib/src/strings/scripting/shell.rs:11), [url.rs:47-72](/home/remi/dev/rum/crates/rum-stdlib/src/strings/scripting/url.rs:47), [paths.rs:36](/home/remi/dev/rum/crates/rum-stdlib/src/strings/scripting/paths.rs:36), [tokenize.rs:52-105](/home/remi/dev/rum/crates/rum-stdlib/src/strings/tokenize.rs:52), [args.rs:4](/home/remi/dev/rum/crates/rum-stdlib/src/strings/scripting/args.rs:4) | `ShellSplit`, `UrlParse`, `FileNameSplit`, `RumTokens`, `TokenizeWith`, `ParseArgs` | Token/component count × `Expr`/association wrappers | none |
| [strings/text/builtins.rs:438-481](/home/remi/dev/rum/crates/rum-stdlib/src/strings/text/builtins.rs:438) | `ToExpression` | Compact text becomes a node-rich parsed/lowered tree | none |
| [strings/similarity/mod.rs:334-390](/home/remi/dev/rum/crates/rum-stdlib/src/strings/similarity/mod.rs:334) | `SequenceAlignment`, `Diff` | Aligned/diff wrappers and duplicated sequence spans | none |
| [bytes.rs:17-80](/home/remi/dev/rum/crates/rum-stdlib/src/bytes.rs:17), [bytes/codec.rs:65-140](/home/remi/dev/rum/crates/rum-stdlib/src/bytes/codec.rs:65), [strings/encoding.rs:15-47,88-99,256-265](/home/remi/dev/rum/crates/rum-stdlib/src/strings/encoding.rs:15) | `ToHexString`, `ToBase64`, `StringToByteArray`, `Compress` | 2× hex, 4/3× Base64, render-plus-envelope expansion | none |
| [bytes.rs:38-143](/home/remi/dev/rum/crates/rum-stdlib/src/bytes.rs:38), [strings/encoding.rs:101-135,268-286](/home/remi/dev/rum/crates/rum-stdlib/src/strings/encoding.rs:101) | `FromHexString`, `FromBase64`, `ByteArrayToString`, `Uncompress` | Decode buffers plus reconstructed `ByteArray`/`Expr`; compact transport can produce wrapper-heavy output | none |
| [memory.rs:289-305](/home/remi/dev/rum/crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:289), [bytes/codec.rs:44-48](/home/remi/dev/rum/crates/rum-stdlib/src/bytes/codec.rs:44), [random.rs:562-574,609-613](/home/remi/dev/rum/crates/rum-stdlib/src/random.rs:562) | `RandomBytes` and every `ByteArray` builder | Physical representation stores Base64 text, while the model substitutes decoded payload length | P + G under the current logical-byte model; physical representation remains undercounted |
| [csv.rs:91-122](/home/remi/dev/rum/crates/rum-stdlib/src/csv.rs:91), [json.rs:11-82](/home/remi/dev/rum/crates/rum-stdlib/src/json.rs:11), [ini.rs:56-104](/home/remi/dev/rum/crates/rum-stdlib/src/ini.rs:56), [toml.rs:77-112](/home/remi/dev/rum/crates/rum-stdlib/src/toml.rs:77), [xml.rs:11-76](/home/remi/dev/rum/crates/rum-stdlib/src/xml.rs:11), [yaml.rs:75-110](/home/remi/dev/rum/crates/rum-stdlib/src/yaml.rs:75) | `From/ToCsv`, `From/ToJson`, `From/ToIni`, `From/ToToml`, `From/ToXml`, `From/ToYaml` | Parse wrappers or encoded escaping/indentation per token/node | none |
| [schema.rs:55-126,516-557](/home/remi/dev/rum/crates/rum-stdlib/src/schema.rs:55) | `SchemaErrors`, `ConformSchema` | Error count/default insertion × paths, messages and wrappers | none |
| [names.rs:68-113,186-224](/home/remi/dev/rum/crates/rum-runtime/src/runtime/namespace_tools/names.rs:68), [builtins_context_loader.rs:103-208](/home/remi/dev/rum/crates/rum-runtime/src/runtime/builtins_context_loader.rs:103), [eval_core/symbol.rs:10-50](/home/remi/dev/rum/crates/rum-runtime/src/runtime/eval_core/symbol.rs:10) | `Names`, `Contexts`, `FileNames`, `$ContextPath`, `$Packages`, `$Path`, `$MessageList` | Tiny query/symbol returns all matching session or host entries | none |
| [builtins_messages_inspect.rs:30-195](/home/remi/dev/rum/crates/rum-runtime/src/runtime/messages/builtins_messages_inspect.rs:30), [builtins_messages.rs:59-137](/home/remi/dev/rum/crates/rum-runtime/src/runtime/messages/builtins_messages.rs:59), [options/builtins.rs:12-25](/home/remi/dev/rum/crates/rum-runtime/src/runtime/options/builtins.rs:12), [namespace_tools/syntax.rs:4-29](/home/remi/dev/rum/crates/rum-runtime/src/runtime/namespace_tools/syntax.rs:4) | `Definition`, `Information`, `OwnValues`, `DownValues`, `UpValues`, `SubValues`, `DefaultValues`, `Messages`, `MessageDefinitions`, `MessageTags`, `Options`, `SyntaxInformation`, `Stack` | One symbol/query reconstructs arbitrary stored state | none |
| [builtins_context_loader.rs:248-337](/home/remi/dev/rum/crates/rum-runtime/src/runtime/builtins_context_loader.rs:248), [builtins_input.rs:41-102](/home/remi/dev/rum/crates/rum-runtime/src/runtime/builtins_input.rs:41) | `Get`, `Needs`, `Dialog`, `Input`, `InputString` | Small path/prompt produces arbitrary host-provided source, state, expression or string | capability only |
| [package_tools.rs:82-87,284-300,607-622](/home/remi/dev/rum/crates/rum-runtime/src/runtime/package_tools.rs:82), [builtins_input.rs:104-111](/home/remi/dev/rum/crates/rum-runtime/src/runtime/builtins_input.rs:104), [debug_tools/basic.rs:6-49](/home/remi/dev/rum/crates/rum-runtime/src/runtime/debug_tools/basic.rs:6) | `Save`, `DumpSave`, `Print`, `Echo`, `Short` | Whole-state/rendered-value buffers; prints are retained in the session | capability where applicable; no memory precheck |
| [debug_tools/trace.rs:28-135](/home/remi/dev/rum/crates/rum-runtime/src/runtime/debug_tools/trace.rs:28), [memory.rs:517-535](/home/remi/dev/rum/crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:517) | `Trace`, `TraceDetail`, `TracePrint` | Evaluation count × before/after/rule/result payload | none |
| [debug_tools/reap.rs:21-68](/home/remi/dev/rum/crates/rum-runtime/src/runtime/debug_tools/reap.rs:21), [memory.rs:467-497](/home/remi/dev/rum/crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:467) | `Sow`, `Reap` | One value copied into every matching active bucket | none |
| [evaluation_data.rs:5-181](/home/remi/dev/rum/crates/rum-runtime/src/runtime/debug_tools/evaluation_data.rs:5), [test_report/report.rs:183-270](/home/remi/dev/rum/crates/rum-runtime/src/runtime/debug_tools/test_report/report.rs:183) | `EvaluationData`, `TestReport` | Input/result/messages/prints/trace duplication; retained result per test | none |
| [definitions.rs:22-77](/home/remi/dev/rum/crates/rum-runtime/src/runtime/definitions/definitions.rs:22), [definition_tags.rs:8-79](/home/remi/dev/rum/crates/rum-runtime/src/runtime/definitions/definition_tags.rs:8), [definition_updates.rs:108-189](/home/remi/dev/rum/crates/rum-runtime/src/runtime/definitions/definition_updates.rs:108), [options/builtins.rs:29-65](/home/remi/dev/rum/crates/rum-runtime/src/runtime/options/builtins.rs:29), [package_tools.rs:90-147](/home/remi/dev/rum/crates/rum-runtime/src/runtime/package_tools.rs:90), [packages.rs:235-270](/home/remi/dev/rum/crates/rum-runtime/src/runtime/scoping/packages.rs:235), [associations/mutation.rs:148-277](/home/remi/dev/rum/crates/rum-stdlib/src/associations/mutation.rs:148) | `Set`, delayed/tag/up assignments, `AddTo`/`TimesBy`/`ApplyTo`, `AppendTo`/`PrependTo`, `SetOptions`, `Once`, `DefinePackage`, `AssociateTo` | Result retained both as output and session state; multi-target operations repeat values; self-updates compound | none |

Already conservative or separately bounded families are intentionally excluded: `Power` has a pre-allocation bit projection ([canonical.rs:99-136](/home/remi/dev/rum/crates/rum-runtime/src/runtime/arithmetic/canonical.rs:99)); `Factorial`/`Factorial2` precheck integer work and poll during construction ([factorial.rs:5-86](/home/remi/dev/rum/crates/rum-runtime/src/runtime/arithmetic/factorial.rs:5)); number-theory and combinatorics thread explicit integer budgets ([number_theory.rs:15-33,304-346](/home/remi/dev/rum/crates/rum-stdlib/src/number_theory.rs:15), [combinatorics.rs:9-44,291-326](/home/remi/dev/rum/crates/rum-stdlib/src/combinatorics.rs:9)); `StringRepeat` and `CharacterRange` have payload-aware projections ([strings/text/builtins.rs:586-668,753-765](/home/remi/dev/rum/crates/rum-stdlib/src/strings/text/builtins.rs:586)); and random text has a maximum UTF-8 byte projection ([random.rs:487-507](/home/remi/dev/rum/crates/rum-stdlib/src/random.rs:487)).

### Structural proposal

A compiler can force classification and typed construction, but it cannot prove arbitrary Rust allocation mathematics. The enforceable design should therefore have three layers.

1. **Mandatory catalog classification.** Add a non-`Default` `MemoryGrowthContract` to every public builtin declaration. Every `BuiltinSpec` constructor must take it as a required argument; no setter or implicit default is allowed. `BuiltinSpec` is currently exported through `kernel_api` ([kernel_api.rs:67-70](/home/remi/dev/rum/crates/rum-runtime/src/runtime/kernel_api.rs:67)) and all constructors currently synthesize an implicit default specification ([metadata.rs:297-330](/home/remi/dev/rum/crates/rum-runtime/src/runtime/metadata.rs:297)).

   Suggested closed variants:

   ```rust
   NonAmplifying(ProofId)
   Preflight(ProjectorFn)
   Incremental(BuilderKind)
   Delegated(CoreGrowthSite)
   ```

   Adding a builtin without selecting one becomes a compile error. A sorted invariant test must compare all 643 installed declarations with the inventory and record every explicit `NonAmplifying` proof/exemption.

2. **Closed core-site enum plus checked builders.** Non-catalog evaluator transformations use a closed `CoreGrowthSite` enum. `String`, `Vec<Expr>`, BigInt/rational construction and session-state insertion at classified sites proceed through `ProjectedAllocation`, `CheckedExprVec`, `CheckedString`, `CheckedIntegerResult`, or `SessionGrowthReservation`. Known-size operations preflight the full proxy contribution; callback/rewrite/trace/Table-like operations reserve incrementally before each retained push. Raw amplifying constructors should be private where practical; unavoidable raw sites belong to a sorted, named `UncheckedGrowth` allowlist.

3. **Armed-mode enforcement and audit postcondition.** Preserve the ordinary handler path unchanged. At registration, compile declarations into a lean normal dispatch map and an armed-memory dispatch sidecar; swap/select the armed map when entering a request ceiling or `MemoryConstrained`. The ordinary dispatcher currently invokes the handler directly ([eval_dispatch/mod.rs:133-189](/home/remi/dev/rum/crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:133)); it must not gain a per-call memory branch. Suspended dispatch metadata must preserve the selected growth contract and mode rather than dropping it as the current three-field `SuspendedBuiltinSpec` does ([session_state.rs:139-153](/home/remi/dev/rum/crates/rum-runtime/src/runtime/session/session_state.rs:139)).

   Under an armed ceiling, add an unstrided after-handler invariant check. It catches misclassified `NonAmplifying` handlers and bad projector math, but is explicitly diagnostic/backstop—not prevention, because allocation has already happened. Prevention comes from preflight or incremental reservations.

This mirrors the useful properties of `FrameRootTrace`: the derive makes an untraceable field a compile failure ([rum-derive/src/lib.rs:13-49](/home/remi/dev/rum/crates/rum-derive/src/lib.rs:13)), while repository tests maintain sorted derived/manual inventories and require every suspension frame to derive the trait ([frame_root_trace.rs:313-367](/home/remi/dev/rum/crates/rum-runtime/src/runtime/tests/frame_root_trace.rs:313), [frame_root_trace.rs:500-525](/home/remi/dev/rum/crates/rum-runtime/src/runtime/tests/frame_root_trace.rs:500)).

The honest limitation is that `NonAmplifying` remains a reviewed assertion and Rust cannot infer a bound for an arbitrary host callback. Mandatory classification, closed enums, checked construction APIs, an allowlist test and an armed postcondition are the strongest available structure without allocator interception or `unsafe`.

### Stride decision

Keep 64 only after the structural inventory is enforced and every multiplicative site is preflighted or incrementally reserved. Correct the comment to say:

> Full proxy walks sample cumulative, non-classified session drift every 64 evaluation points. Declared multiplicative growth is guarded unstrided; the stride itself does not bound per-step byte growth.

The present comment instead attributes safety to element/integer/iteration budgets ([constraints.rs:5-9](/home/remi/dev/rum/crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:5)). The actual implementation performs a full walk only at counter zero ([constraints.rs:139-159](/home/remi/dev/rum/crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:139)), while projected-byte checks are unstrided ([constraints.rs:174-186](/home/remi/dev/rum/crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:174)).

The existing recorded A/B makes stride 1 unattractive: armed execution went from 3.825 seconds at the every-check shape to 0.667 seconds with stride 64, a 5.73× difference; the default candidate median was −3.4% with zero rows above its noise floor ([host-embedding-line.md:96-110](/home/remi/dev/rum/docs/dev/host-embedding-line.md:96)). I did not remeasure that result.

Do not make the stride adaptive yet. An adaptive rule is defensible only when every transition has a proven maximum unreserved delta and the last full walk supplies known remaining headroom. Otherwise adaptation changes sampling frequency without establishing a bound.

## 4. Sequencing

A and B are two distinct lines of work:

1. Land the Gap B classification and checked-reservation substrate first.
2. Convert deterministic scalar/string/integer/codec sites.
3. Convert collection, callback, rewrite, session-state and diagnostic accumulators.
4. Implement A1 using the same session-growth reservation primitive for interner insertion.
5. Re-run the final stride and hot-path gates after the `Symbol` representation change.

B does not determine symbol identity semantics. A changes `Symbol` clone/drop behavior and potentially `Expr` size, so B’s projections must use central proxy helpers rather than baked-in sizes. Conversely, B’s reservation API gives A1 an enforceable refusal point before interner allocation.

## 5. Drafted implementation briefs

### Brief 1 — mandatory memory-growth declarations

#### Goal

Make omission of a builtin/core memory-growth classification a compile failure or repository-invariant failure, without changing language behavior or adding a branch to ordinary dispatch.

#### Context pointers

- `BuiltinSpec`: [metadata.rs:283-330](/home/remi/dev/rum/crates/rum-runtime/src/runtime/metadata.rs:283)
- Dispatch: [eval_dispatch/mod.rs:133-189](/home/remi/dev/rum/crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:133)
- Suspended spec: [session_state.rs:139-153](/home/remi/dev/rum/crates/rum-runtime/src/runtime/session/session_state.rs:139)
- Catalog installation: [rum-stdlib/src/lib.rs:42-138](/home/remi/dev/rum/crates/rum-stdlib/src/lib.rs:42)
- Frame precedent: [frame_root_trace.rs:313-367](/home/remi/dev/rum/crates/rum-runtime/src/runtime/tests/frame_root_trace.rs:313)

#### Scope

Files MAY modify:

- `crates/rum-runtime/src/runtime/metadata.rs`
- `crates/rum-runtime/src/runtime/mod.rs`
- `crates/rum-runtime/src/runtime/kernel_api.rs`
- `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs`
- `crates/rum-runtime/src/runtime/session/mod.rs`
- `crates/rum-runtime/src/runtime/session/session_state.rs`
- `crates/rum-runtime/src/runtime/builtin_catalog.rs`
- all existing `.rs` declarations beneath `crates/rum-runtime/src/runtime/builtin_catalog/`
- `crates/rum-runtime/src/runtime/tests/mod.rs`
- new `crates/rum-runtime/src/runtime/tests/memory_growth_contract.rs`
- `crates/rum-stdlib/src/catalog/mod.rs`
- every existing builtin declaration file beneath `crates/rum-stdlib/src/catalog/`
- `crates/rum-stdlib/src/lib.rs`
- `scripts/core-language-microbench.sh`

The runtime module roots and `kernel_api.rs` are explicitly in scope because exported `BuiltinSpec` changes.

#### Deliverable

- Non-`Default`, mandatory growth contract on every builtin declaration.
- Private lean runtime dispatch representation so default dispatch does not carry an extra branch.
- Closed `CoreGrowthSite`.
- Sorted invariant covering all 643 catalog entries and all handwritten core sites.
- Compile-fail coverage proving a missing declaration cannot compile.
- No behavior changes and no actual new prechecks in this step.

#### Verification

```text
cargo --offline fmt --check
scripts/run-bounded.py --timeout 1200 -- cargo test --offline -p rum-runtime
scripts/run-bounded.py --timeout 1200 -- cargo test --offline -p rum-stdlib
scripts/run-bounded.py --timeout 1800 -- cargo test --offline --workspace
```

Run base/new `core-language-microbench.sh` with `CARGO_NET_OFFLINE=true`; report the literal/symbol/list/downvalue/matcher/replace/Table labels and same-tier neighbors from [core-language-microbench.sh:456-558](/home/remi/dev/rum/scripts/core-language-microbench.sh:456).

#### Hard rules

- No git mutation, network, vendor oracle, `unsafe`, weakened contract or drive-by edit.
- Do not add a default-path dispatch branch, allocation or lock.
- Preserve suspension/resume behavior.
- A default-path regression or enlarged lean runtime dispatch record is a stop-loss.
- Paste exact test and timing output.

---

### Brief 2 — deterministic scalar, string and codec preflights

#### Goal

Preflight every inventory site whose maximum output bytes can be computed before construction, including exact arithmetic, strings, byte encodings and fixed-shape generation.

#### Scope

Files MAY modify:

- `crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs`
- `crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs`
- `crates/rum-runtime/src/runtime/debug_tools/resources.rs`
- `crates/rum-runtime/src/runtime/kernel_api.rs`
- `crates/rum-runtime/src/runtime/arithmetic/mod.rs`
- `crates/rum-runtime/src/runtime/arithmetic/canonical.rs`
- `crates/rum-runtime/src/runtime/arithmetic/integer_ops.rs`
- `crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/spec.rs`
- corresponding runtime memory-limit tests and their `mod.rs`
- `crates/rum-stdlib/src/arithmetic.rs`
- `crates/rum-stdlib/src/integer_digits.rs`
- `crates/rum-stdlib/src/random.rs`
- `crates/rum-stdlib/src/random/password_policy.rs`
- `crates/rum-stdlib/src/strings.rs`
- existing `.rs` files beneath `crates/rum-stdlib/src/strings/`
- `crates/rum-stdlib/src/bytes.rs`
- existing `.rs` files beneath `crates/rum-stdlib/src/bytes/`
- `crates/rum-stdlib/src/csv.rs`, `json.rs`, `ini.rs`, `toml.rs`, `xml.rs`, `yaml.rs`
- their existing encoder/decoder submodules
- `crates/rum-stdlib/src/schema.rs`
- relevant stdlib test files and test module roots
- `scripts/core-language-microbench.sh`
- `docs/guide/resource-limits-and-sandbox.md`
- `docs/guide/structured-data.md`

`kernel_api.rs` and both string/module roots are included because projection helpers cross crate/module boundaries.

#### Deliverable

- Overflow-safe projected sizes for `StringJoin`, arithmetic products/rationals, LCM, FromDigits, escaping, formatting and codec outputs.
- Replace node-only projections for fixed-shape generators with payload-aware proxy projections.
- Decide and document whether `ByteArray` accounts physical Base64 storage or logical decoded bytes; tests must match that decision.
- Low-ceiling regressions proving refusal occurs before large output allocation.
- No ordinary-path projection work: the checked handler path created by Brief 1 is active only while a memory ceiling is armed.

#### Verification

Run offline formatting, focused runtime/stdlib suites, then the workspace suite. Add and A/B exact `string-join-small`, BigInt `times-small`, `string-repeat-small`, `character-range-small`, Range, random and codec neighbors. Paste exact output and per-label rows.

#### Hard rules

- No git/network/vendor oracle/`unsafe`.
- Saturating or checked arithmetic only; overflow fails closed.
- No weakening generated-element or integer-work contracts.
- Default unarmed handlers must remain on the lean dispatch path.
- Any hot-label regression is a stop-loss.

---

### Brief 3 — incremental collection and evaluator accounting

#### Goal

Convert every callback-, traversal-, rewrite-, association-, session-state- and diagnostic-accumulation row to checked incremental retention, with suspension-safe accounting.

#### Scope

Files MAY modify:

- `crates/rum-runtime/src/runtime/listable_dispatch.rs`
- `crates/rum-runtime/src/runtime/hold_thread.rs`
- existing `.rs` files beneath `crates/rum-runtime/src/runtime/substitution/`
- `crates/rum-runtime/src/runtime/substitution.rs` or its module root
- existing `.rs` files beneath `runtime/kernel_data/structure_data/`
- existing `.rs` files beneath `runtime/kernel_data/rewrite_data/`
- existing `.rs` files beneath `runtime/tree_walkers/`
- existing `.rs` files beneath `runtime/definitions/`
- existing `.rs` files beneath `runtime/debug_tools/`
- `runtime/builtins_input.rs`, `runtime/builtins_context_loader.rs`
- existing `.rs` files beneath `runtime/namespace_tools/`, `runtime/messages/`, `runtime/options/`, `runtime/scoping/`
- `runtime/package_tools.rs` and its resume-frame module
- every touched directory’s `mod.rs`
- existing `.rs` files beneath `crates/rum-stdlib/src/functional_ops/`
- existing `.rs` files beneath `crates/rum-stdlib/src/structure_ops/`
- existing `.rs` files beneath `crates/rum-stdlib/src/associations/`
- `crates/rum-stdlib/src/lazy.rs`, `rewrite_builtins.rs`, `control_flow.rs`, `polynomials/**`, `calculus.rs`
- corresponding test files and module roots
- `scripts/core-language-microbench.sh`
- `docs/guide/resource-limits-and-sandbox.md`

All module roots exporting changed continuation/builder types are explicitly included.

#### Deliverable

- Checked incremental builders retained through every suspension frame.
- Full-payload reservations before each `push`, state insertion, trace/reap event, rewrite result and callback result.
- Complete individual-head inventory matched mechanically against Brief 1’s declarations.
- Armed postcondition tests that fail on a deliberately wrong `NonAmplifying` classification.
- Adversarial low-ceiling tests for Table, substitutions, Cartesian generators, JoinAcross, Lookup, Trace, Sow/Reap, EvaluationData and session stores.

#### Verification

Run offline runtime, stdlib, host and workspace suites. A/B Table, map/fold, matcher, replace, association, part/selector and string neighbors. Verify suspended and uninterrupted evaluations produce identical in-band outcomes.

#### Hard rules

- No `unsafe`, vendor comparison, network or git mutation.
- Do not collapse or bypass suspension frames.
- No owned deep-tree clone on the default hot path.
- Every raw unchecked allocation requires a named sorted allowlist entry.
- Stop on any semantic or same-tier performance regression.

---

### Brief 4 — session-owned symbol interning

#### Goal

Replace the arbitrary process-global leaking interner with the recommended session-owned/escaped-value-owned design, preserving symbol semantics, snapshots and thread safety while bringing symbol allocations under the memory ceiling.

#### Scope

Files MAY modify:

- `crates/rum-core/src/expr.rs`
- `crates/rum-core/src/lib.rs`
- `crates/rum-core/src/heads.rs`
- relevant `rum-core` tests
- `crates/rum-runtime/src/lib.rs`
- `crates/rum-runtime/src/runtime/mod.rs`
- `crates/rum-runtime/src/runtime/kernel_api.rs`
- all affected `.rs` files beneath `crates/rum-runtime/src/`
- `crates/rum-stdlib/src/lib.rs`
- all affected `.rs` files beneath `crates/rum-stdlib/src/`
- `crates/rum-host/src/lib.rs`
- all affected `.rs` files beneath `crates/rum-host/src/`
- `crates/rum-syntax/src/lib.rs` and affected syntax files
- `crates/rum-repl/src/lib.rs` and affected REPL files
- `crates/rum-jupyter/src/lib.rs` and affected Jupyter files
- relevant tests in those crates
- `scripts/core-language-microbench.sh`
- public symbol/scoping/embedding/resource-limit documentation

Every affected crate root is explicitly in scope because `Symbol` is exported by `rum-core` and crosses crate APIs ([rum-core/src/lib.rs:1-14](/home/remi/dev/rum/crates/rum-core/src/lib.rs:1)).

#### Deliverable

- Safe owned `SymbolText`, session interner, bounded canonical pool and detached-symbol import.
- No arbitrary process-global strong symbol storage.
- Recoverable allocation refusal; no `u32` panic.
- Explicit `interned_id` compatibility resolution.
- Session memory ledger and pre-insertion reservation.
- Cross-session equality/hash/order, serde, sealed snapshot/re-link, reset reclamation, escaped-value lifetime and host/Jupyter thread tests.
- Compile-time size assertions for the intended 24-byte `Symbol` and 32-byte `Expr`, if those sizes remain achievable without `unsafe`.
- Updated 260-site inventory with the exact edited subset and compiler-reported non-`Copy` fallout.

#### Verification

Run offline core, runtime, stdlib, host, Jupyter and workspace suites. Run repeated unique-name requests under a low ceiling, reset cycles, escaped-result-after-drop tests and the full A1 hot-path A/B label set from section 2.

#### Hard rules

- No `unsafe`, leaks, global arbitrary-name pool, TLS budget context or lifetime-bearing public `Expr`.
- Preserve bit-62 sealing and fresh-identity restore behavior.
- Do not accept an `Expr`/`Symbol` size increase or hot-label regression without stopping and reporting.
- Do not weaken equality to fingerprint-only comparison.
- No git/network/vendor oracle; paste exact results.

---

### Brief 5 — final stride proof and documentation

#### Goal

Validate the completed accounting model, retain or change stride 64 based on measured armed/default behavior, and make the documented guarantee exact.

#### Scope

Files MAY modify:

- `crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs`
- relevant runtime resource-limit tests and module roots
- `docs/guide/resource-limits-and-sandbox.md`
- `docs/host-embedding-and-hardening.md`
- `scripts/core-language-microbench.sh`

#### Deliverable

- A test demonstrating that every classified multiplicative site is checked unstrided.
- A forced-counter test showing sampled cumulative drift behaves exactly as documented.
- Interleaved stride-1/stride-64 armed A/B plus default-path A/B.
- Retain 64 only if the inventory invariant is complete and the measurement supports it; otherwise report and stop.
- Correct the current comment so it no longer claims element/integer/iteration budgets bound bytes.

#### Verification

Run offline runtime/workspace tests and bounded interleaved microbench runs. Paste all target and same-tier rows, including the existing calibration labels and any new memory-armed labels.

#### Hard rules

- One variable per measurement; profiling disabled.
- Exact benchmark statements only.
- No silent reruns of anomalous data.
- No git/network/vendor oracle/`unsafe`.
- A default regression or unbounded unclassified site blocks completion.

## 6. Risks and open questions

1. **`interned_id` is a public-contract decision.** Keeping its signature but changing “dense ID” to stable fingerprint is my recommendation; removing/changing it is a larger public API break. Its current rustdoc and live Bloom use are at [expr.rs:164-168](/home/remi/dev/rum/crates/rum-core/src/expr.rs:164) and [evaluation_cache/stability.rs:18-24](/home/remi/dev/rum/crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:18).

2. **The proposed 24-byte Arc-backed `Symbol` is a target-specific layout expectation, not an observed implementation result.** Only the current 24/32-byte layout was observed; the implementation probe must establish the candidate size before landing ([expr.rs:35-39](/home/remi/dev/rum/crates/rum-core/src/expr.rs:35), [expr.rs:536-542](/home/remi/dev/rum/crates/rum-core/src/expr.rs:536)).

3. **The exact non-`Copy` edit count is not established.** The verified 260 count covers direct constructor expressions only. Map `.copied()`, dereference moves, frame fields and downstream trait bounds require a compiling prototype; claiming their count now would be guesswork.

4. **ByteArray needs a model ruling.** It is physically Base64 text ([structured-data.md:14-15](/home/remi/dev/rum/docs/guide/structured-data.md:14)), while the memory proxy substitutes decoded byte length ([memory.rs:289-305](/home/remi/dev/rum/crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:289)). Either logical-byte accounting must be documented as deliberate, or physical capacity must be charged.

5. **Compiler enforcement cannot prove projector correctness or arbitrary host handlers.** It can require a declaration and typed builder, close internal enums, and reject omitted fields; the armed invariant remains necessary. `BuiltinSpec` is a public host-facing registration surface ([kernel_api.rs:67-70](/home/remi/dev/rum/crates/rum-runtime/src/runtime/kernel_api.rs:67)).

6. **This inventory is source-reasoned, not dynamically proven.** I did not reproduce the reported 2.04 GB/1.19 GB peaks or the director’s interner RSS sequence. The documented ceiling is a deterministic modeled-memory bound rather than a hard RSS meter, and embedders still need an external hard-process limit when required ([resource-limits-and-sandbox.md:268-284](/home/remi/dev/rum/docs/guide/resource-limits-and-sandbox.md:268)).

7. **The prior-art path in the brief is stale.** Exact probe output was:

   ```text
   absent docs/dev/plan-364-frame-roots.md
   ```

   The actual document is [docs/dev/codex/plan-364-frame-roots.md:1](/home/remi/dev/rum/docs/dev/codex/plan-364-frame-roots.md:1); the implemented derive and invariant tests cited above were used as the authoritative precedent.
