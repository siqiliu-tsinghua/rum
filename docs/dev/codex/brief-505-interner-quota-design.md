# brief-505 — design: a bounded, loud interner under the 2026-09-21 ruling

`Scope: NONE — no source change.` Design-only. You survey, reason, and return
a plan plus drafted follow-up briefs. You write no code.

## Goal

The user ruled on 2026-09-21, after brief-504's option survey: **`Symbol`
keeps its lifetime-free public type**, on top of its 24-byte `Copy`
representation and id-based fast-accept equality. brief-504 established that
under those combined constraints **no in-process reclaiming fix exists**, so
the interner finding is not closed — it is converted from unbounded invisible
growth into a **bounded, loud** limit.

Design that conversion, completely, and return the briefs to execute it.

## What is ruled, and therefore not open

Do not re-propose any of these. If your plan requires one, say so and stop
rather than working around the ruling.

- **`Symbol` does not change.** Not its size, not `Copy`, not the id-based
  fast-accept equality, and **not its lifetime-free `'static`-compatible
  public type**. A generative brand, a lifetime parameter, an `Arc`, or a
  session-scoped id are all excluded. The measured floor for any owning
  representation is +10.93% instructions on ordinary evaluation
  (briefs 501-503).
- **No reclamation.** A reclaimable global interner cannot exclude escaped
  handles: a reissued dense id makes a `Symbol` that escaped into a host's
  `Expr` silently denote different text. `reset()` will not free interner
  bytes and the documentation will say so.
- **No per-session attribution of interner bytes.** They are shared; charging
  them to one session makes one tenant pay for another. The quota is
  process-wide and must not pretend otherwise.

## What the mitigation must be

- A process-wide interner **byte** quota. Count-only is insufficient because
  names vary in length; the budget must cover arbitrary text, the
  `SymbolText` pairs, and the hash-table growth that holds them.
- A **recoverable refusal** replacing the `u32::try_from(pairs.len()).expect(...)`
  panic at `crates/rum-core/src/expr.rs:88`.
- **Preflight before anything is leaked.** `intern_text` (`:57-70`) and
  `intern_symbol_text` (`:80-96`) must decide admission *before* the
  `Box::leak`, not after.
- **Existing names stay usable at the cap.** Refusal applies to inserting a
  new name, never to looking up one already interned.
- **Refusal coverage made compiler-enforced**, not remembered: the only API
  that can insert an arbitrary name returns `Result`, and every constructor
  and serde path is converted to it.
- **Reserved bootstrap and error symbols**, interned before the quota can
  bind, so a refusal can always be reported — a refusal that cannot construct
  its own message is a second failure.

## The questions that decide the plan

### 1. Which constructors become fallible, and what a refusal means at each site

`Symbol::new`, `Symbol::with_identity`, `Symbol::from_name` and
`Symbol::system` (`crates/rum-core/src/expr.rs:108-138`) plus
`Expr::symbol`, `Expr::contextual_symbol` and `Expr::system_symbol`
(`:801-812`) are the public constructors. A director regex over the four
`Symbol::` names finds **179 call sites**; brief-500 showed that such a count
is a lower bound on the real ripple, so **produce the inventory, do not trust
that number.**

Settle, with evidence:

- Which of these can stay **infallible** because they cannot insert arbitrary
  text — `Symbol::system` over `CANONICAL_CORE_HEADS`
  (`crates/rum-core/src/heads.rs:112`) is the obvious candidate, but
  `Symbol::system` also falls through to `Symbol::new` for a non-canonical
  name (`:132-136`), so say exactly what the safe subset is and how the type
  system keeps a caller inside it.
- For each fallible call site **class**, what a refusal becomes: an
  `EvalError` in the evaluator, a message, a parse diagnostic, a host error.
  `rum-core` has no `EvalError`, so name the error type that lives there and
  where it is mapped.
- The sites with **no error channel at all**. There will be some. For each,
  say whether the channel can be created, or the site can be proven unable to
  insert, or it must keep a panic — and if it must, why that is not simply
  moving the panic.

### 2. Serde, which cannot be waved through

`Deserialize for Symbol` (`crates/rum-core/src/expr.rs:303-320`) interns
attacker-supplied text from a snapshot. It is the one path where refusal is
most needed and where the error channel is fixed by the trait. Say how it
refuses, and what that means for `restore_from_bytes`, whose own audit
finding (`SECURITY.md`, the 2026-09-02 LOW) already says restoring untrusted
bytes runs them under the session's policy.

### 3. The quota's shape

- Who sets it, when, and can it change after the first intern? A process-wide
  knob configured once at startup behaves very differently from one a session
  can raise.
- What is the default, and is a default that never fires in practice the same
  fail-open mistake as the 6 MiB native stack guard on a 2 MiB thread
  (`TODO.md` §The native stack budget)? Argue the number, do not pick one.
- How does a host **observe** current usage? It cannot be in
  `current_memory_proxy_bytes`
  (`crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:50`),
  which is per-session, so name where it goes and what reads it.
- Is the accounting exact or an estimate? If an estimate, state the direction
  of its error; an under-estimate is a fail-open.

### 4. The falsifier

Any check this project builds ships with a checked-in proof that it can fire
(`TODO.md` §memory-accounting line; the rule is that an instrument never seen
to fail is not known to work). Design the falsifier: what test drives the
interner to the cap and asserts the refusal, how it avoids leaking a real
process-global quota into every other test in the binary, and how a future
change that silently stops enforcing is caught.

**This is the hardest part of the design and you should treat it as such**: a
process-global, never-reclaimed quota is exactly the kind of state that makes
tests order-dependent. If the honest answer is that the quota must be
injectable for tests, say so and design that seam.

### 5. Documentation obligations

The mitigation is only honest if the limits are stated. Draft the exact
claims that must appear, and where: that interner bytes are process-global
and outside the per-session ceiling; that `reset()` does not reclaim them;
that one tenant can permanently exhaust the shared quota; and that host
process isolation remains the pattern for hard multi-tenant isolation.
Candidate homes: `SECURITY.md`, `docs/host-embedding-and-hardening.md`,
`docs/guide/resource-limits-and-sandbox.md`.

Note the adjacent audit criticism while you are there — **no document says
which state is per-request and which is per-session** — and say whether this
work should answer it or whether it is a separate brief. Do not answer it
here.

## What I want back

1. The option-free plan: ordered steps, files and functions per step, the
   gate test per step, and how the ruled invariants stay intact.
2. The **call-site inventory**, produced rather than estimated, classified by
   what a refusal becomes there.
3. The falsifier design, per §4.
4. Risks and open questions, each one named as a decision for the director
   rather than resolved silently — brief-504 did this correctly and it is why
   there is a ruling to work under.
5. Drafted follow-up briefs in `TEMPLATE.md` shape, usable as drafts. The
   director rewrites them; they are raw material.

Every claim carries `path:line` evidence.

## Method requirements

- Read `AGENTS.md`, then `TODO.md` §The interner ruling of 2026-09-21, then
  brief-504's report in this directory.
- **Re-probe rather than inherit.** Where you rely on a claim from brief-414
  or brief-504, say you verified it and how. brief-414's Gap A analysis has
  already been found stale in places.
- Where you assert a cost, say whether it is measured, inferred, or a guess.
- Name what would falsify your plan.

## Hard rules

- **No source change.** No `.rs` file, no script, no fixture, no doc except
  appending your report to this file. Scratch under `target/brief-505/`.
- Do NOT `git commit`, branch, switch, merge, push, stash, `git checkout`, or
  touch git config or the index.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only; no oracle harnesses, no comparisons against other vendors'
  engines.
- No network. Touch nothing outside this repo.
- Paste verbatim evidence for anything you ran. Never quote a number you
  could not take.

## Report format

1. The plan.
2. The call-site inventory.
3. The falsifier design.
4. Risks and open questions, as director decisions.
5. The drafted follow-up briefs.


---

## Execution report — brief-505

### 1. The plan

**Implement a process-wide, non-reclaiming interner with checked byte
admission and fallible arbitrary-name construction. Keep the finding open as
a mitigated shared-process exhaustion risk.** This report proposes one
implementation, subject to the explicit director decisions in section 4;
none of brief-504's reclaiming alternatives is reopened. The `Symbol`
fields, `Copy`, lifetime-free public type, dense text IDs, equality, hashing
and ordering stay intact. No session owns or is charged for the interner.
(`TODO.md:1188-1219`; `crates/rum-core/src/expr.rs:35-40,164-168,250-285`.)

This is source analysis, not an implemented or benchmarked candidate. I read
`AGENTS.md`, the current TODO ruling, brief-504's report, the template and
the relevant contracts. No Rust compilation, executable rum workload, test
suite, allocator measurement or timing panel ran. All cost expectations below
are **inferences**, unless explicitly called a proposed policy number;
there is no new measured performance percentage. Source-free work has an
evidence gate, not a production test gate.
(`AGENTS.md:53-71`; `docs/dev/codex/TEMPLATE.md:19-25,102-105`.)

#### Fresh verification of the premises

| Premise inherited from 414/504 | Re-probe and result in this tree |
|---|---|
| Global text and pair retention | Read the symbol implementation and searched both private insertion functions. The only two constructor callers reach the same global mutex. Both `Box::leak`s remain, and no removal operation exists in this implementation. `crates/rum-core/src/expr.rs:42-138`. |
| An ID-overflow change alone would suffice | **False.** Context and name have already been interned at lines 85-86 when the ID conversion runs at 88. Moreover the conversion precedes the existing-pair lookup at 89, so ID exhaustion currently also rejects hits by panicking. Admission must cover the entire pair transaction and must follow a hit check. `crates/rum-core/src/expr.rs:80-96`. |
| Existing canonical heads cannot fail | **False with the current API.** Their lazy map calls the arbitrary constructor, and `system` falls through for other names. Even `intern_text`'s static-context branch inserts into a growing hash set. `crates/rum-core/src/expr.rs:57-60,98-105,132-138`. |
| Public lowering takes a resolver closure | Rechecked 504's correction: the public free function takes only `&SyntaxExpr`; the closure-bearing helper is `pub(super)`. The closure itself still returns `Option<Symbol>` and masks any future refusal unless changed. `crates/rum-runtime/src/runtime/lowering.rs:33-40,93-103,408-416`. |
| Session memory includes the leak | Read the **whole** current walk, not just a search. It totals session structures and does not access the interner. Referenced symbol text is separately charged per expression occurrence. `crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:50-338,363-365`. |
| Reset releases the interner | Reset rebuilds the inner session only. The global interner has no connection to that replacement. `crates/rum-host/src/session.rs:147-175`; `crates/rum-core/src/expr.rs:47-55`. |
| Host restore goes through `Deserialize for Symbol` | **It does not.** Core serde re-interns context/name, but the host snapshot API decodes UTF-8, resets and evaluates source. Both ingresses need coverage. The audit's LOW statement is currently in **TODO**, not in SECURITY's short threat model. `crates/rum-core/src/expr.rs:303-319`; `crates/rum-host/src/session.rs:192-220`; `TODO.md:941-944`; `SECURITY.md:68-98`. |
| Prior constructor counts establish the ripple | Replaced by this report's census: 180 direct `Symbol::` occurrences on 179 source lines, 157 direct `Expr::` occurrences, and two internal `Self::new` wrappers. The public `system_expr` layer adds 1,557 call expressions, overlapping the direct inventory at its implementation. This is a source inventory, **not** a claimed compiler-error count. Exact locations and verbatim output are in section 2. `crates/rum-runtime/src/runtime/options/core.rs:12`; `crates/rum-core/src/expr.rs:129,137`; `crates/rum-runtime/src/runtime/expr_helpers.rs:9-10`. |

#### Admission, storage and accounting

Use a new private `rum-core` interner module, split from symbol operations
and expression storage. The implementation anchors are the present
`SymbolInterner`, `intern_text` and `intern_symbol_text`; the public value
representation does not move into a session. A coherent split is required
rather than adding another large subsystem to `expr.rs`.
(`crates/rum-core/src/expr.rs:19-138,749-817`;
`crates/rum-core/src/lib.rs:7-15`;
`docs/code-organization-guidelines.md:66-95`.)

**The byte unit is charged Rust storage, not RSS.** Count all retained text
payloads, complete `SymbolText` allocations, complete index backing stores
including empty slots/control data/padding, the fixed interner state and
reserved-pool storage. Include an outstanding reservation for the overlap
of old and replacement tables during growth. Do not use
`HashMap::capacity() * size_of::<entry>()` as a claimed upper bound: the
current indexes are opaque standard-library hash tables, and their
capacity does not describe all backing storage. Replace these two private
indexes with safe open-addressed indexes over explicitly sized boxed slot
slices, using `RandomState` hashing, full equality checks and a maximum
one-half load before growth. Slot layouts include their occupied/empty
representation. This is a proposed implementation, **not a measured
optimization**.
(`crates/rum-core/src/expr.rs:2,42-45,57-96`;
`AGENTS.md:109-122`; `docs/host-embedding-and-hardening.md:54-59`.)

For that representation, the dynamic retained charge is exact in allocation
layout bytes: sum the string lengths, `size_of::<SymbolText>()` per pair,
and each allocated slot slice's full `Layout` size. Charge slice/pointer
owners in the fixed state once, rather than forgetting them or counting
logical lengths instead of allocated lengths. Charge the reserved pool
conservatively by its declared storage even if compiler/linker pooling
shares literal bytes; that error is **upward**. Allocator size-class rounding,
allocator metadata, executable mappings and caller/serde input buffers are
outside this unit. No portable upper bound on those is established here.
**Director decision D2 must ratify this precise byte contract; this report
does not disguise a lower estimate of RSS as a hard heap cap.**
(`crates/rum-core/src/expr.rs:23-45,66,90,305-309`;
`docs/host-embedding-and-hardening.md:54-59,130-132`.)

A private admission plan describes both missing strings, whether they are
the same string, the missing pair, the next ID and every index allocation.
Only its checked permit can reach the leak/publication operation. All sums,
products, layout calculations, capacity doublings and ID increments use
checked arithmetic; overflow refuses. Implement the transaction in this
order, under the existing process mutex:

1. Validate constructor arguments, including a nonzero generated identity.
   Probe the reserved pool and the dynamic indexes by borrowed text. Return
   an existing **pair** before testing quota or next-ID capacity. A new
   identity over an existing pair consumes no interner bytes or text ID.
2. On a missing pair, find existing context/name strings without insertion.
   Account a new context and a new name together, deduplicating them when
   equal. A new pair made from two old strings still needs a pair record and
   pair-index space. Compute final storage and simultaneous growth storage.
3. Refuse before leaking or publishing anything if either projected charge
   exceeds the process limit or the next dense ID does not fit `u32`.
   Keep the ID cursor wide enough to represent the exhausted state; permit
   `u32::MAX` itself as the last valid ID. Reserved IDs form its initial
   prefix. The quota does not replace the independent ID check.
4. Stage new owned boxed text, pair storage and replacement indexes. The
   safe boxed-slice construction path must expose and account any temporary
   vector capacity and conversion overlap before publication; an unexpected
   capacity or `try_reserve` failure drops staged owners and refuses. Do
   not call `reserve` on a live index and discover its charge afterward.
   The storage proof in Draft A must enumerate these allocations; an
   unaccounted construction temporary fails that proof.
5. Once staging is complete, leak only the admitted text/pair owners and
   commit the index slots, ledger and next ID with no remaining allocation,
   fallible conversion or callback. Drop replaced index buffers and release
   only their overlap reservation. Refusal leaves pair/text counts, IDs and
   retained charge unchanged. Names admitted by **earlier** successful
   constructor calls in a larger failed request remain charged permanently.

These are changes to the ordering at
`crates/rum-core/src/expr.rs:57-96,118-125`. The permit is private and not
cloneable; callers cannot mint one or bypass it. Local test planners never
mint public `Symbol`s. No arbitrary-name fast path is exempt, including an
arbitrary `System` name, a long context, or a string equal to a reserved
name in a different context. Compare complete string identity/content,
including lengths; a pointer address alone must not conflate distinct
static string slices in the expanded reserved pool. The existing pointer
key and limited static-string case are at
`crates/rum-core/src/expr.rs:72-87`.

The cap prevents quota/ID exhaustion from reaching a panic. It does not
make every allocator failure recoverable: ordinary Rust allocation can
still abort when a host has provisioned insufficient process memory. Use
fallible reservation where available, but do not claim that `Box::new`
has acquired a portable recoverable OOM contract. A poisoned interner lock
returns a core `Unavailable` error for fallible operations; do not replace
it with a fresh interner or recycled IDs. Reserved symbols and formatting
of the scalar error remain independent of that lock.
(`crates/rum-core/src/expr.rs:81-95`; `TODO.md:911-914`;
`docs/host-embedding-and-hardening.md:76-82,130-132`.)

#### Public API and the closed bootstrap set

Introduce `InternerError` in `rum-core`, with scalar variants for
`QuotaExceeded { limit_bytes, used_bytes, required_bytes }`,
`IdExhausted`, `SizeOverflow`, `InvalidIdentity`, allocation-reservation
failure and `Unavailable`. Its `Display`/`Error` implementations do not
construct an `Expr`, intern text, or embed the refused attacker-supplied
name. Configuration errors use a separate `InternerConfigError`.
The core crate cannot depend on `EvalError`; the conversion belongs in the
runtime error module.
(`crates/rum-core/src/lib.rs:1-15`; `crates/rum-core/Cargo.toml:12-17`;
`crates/rum-runtime/src/runtime/session/session_state.rs:498-574`.)

All seven existing arbitrary-text constructors keep their purpose but
return `Result<_, InternerError>`:
`Symbol::{new,with_identity,from_name,system}` and
`Expr::{symbol,contextual_symbol,system_symbol}`. Keep `with_identity`'s
`u64` input and return `InvalidIdentity` for zero **before** admission;
this also fixes its serde-triggered panic without changing the serialized
fields. No infallible compatibility wrapper, default symbol on refusal,
`unwrap_or($Failed)`, or quota-catching `expect` is retained.
(`crates/rum-core/src/expr.rs:109-138,303-319,801-811`.)

Add a closed `ReservedSymbol` vocabulary and statically constructed
`SymbolText`/symbol tables. `Symbol::reserved(ReservedSymbol)` and
`Expr::reserved_symbol(ReservedSymbol)` are infallible because they only
copy entries. A raw `&str` or even `&'static str` is **not** that proof.
The vocabulary has no unchecked public integer/string constructor; any
string-to-token conversion is lookup-only and returns `Option`/`Result`.
Keep `system_expr` as an infallible **typed-token** helper, and require an
already constructed `Symbol`/head expression at open-ended helper APIs.
There is no fall-through from that helper into arbitrary insertion.
(`crates/rum-core/src/heads.rs:112-198`;
`crates/rum-core/src/expr.rs:98-138`;
`crates/rum-runtime/src/runtime/expr_helpers.rs:9-10`;
`crates/rum-runtime/src/runtime/session/session_api.rs:13-54`.)

The vocabulary covers the current 81 canonical heads **and the finite
production construction closure**: fixed builtin/catalog names, option
names and defaults, parser sugar, attribute/result heads, internal
continuation tags, error heads, calendar/YAML fixed values and the named
operator-form parameters. Explicit non-System reservations include
`Information`'s `InformationValueForm` and the existing `Global` parameters
`rum$keyedArg`, `rum$extremalByArg`, `rum$sortArg`, `rum$replaceListArg` and
`rum$replaceAtArg`. Include both the System error symbol and the Global
`General` pair currently reconstructed from a raw runtime-message string.
Tests' arbitrary `xN`/`factorN` names do not become bootstrap exemptions.
The exact final vocabulary is the compiler-forced closure of these typed
conversions, not the raw count of literal strings in section 2.
(`crates/rum-runtime/src/runtime/messages/builtins_messages_inspect.rs:483-498`;
`crates/rum-stdlib/src/associations/data.rs:346-365`;
`crates/rum-stdlib/src/structure_ops/collection_stats.rs:894-903`;
`crates/rum-stdlib/src/structure_ops/ordering_sets.rs:981-989`;
`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/list.rs:251`;
`crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/replace_at.rs:213`;
`crates/rum-runtime/src/runtime/messages/message_format.rs:3-8,23-30`;
`crates/rum-runtime/src/runtime/session/session_helpers.rs:338-342`;
`crates/rum-stdlib/src/polynomials/core/variables.rs:57-105`.)

Reserve `TerminatedEvaluation`, `General`, `MessageName`, `HoldForm`, `List`,
`$Failed`, `$Aborted`, and the new interner-error continuation tag before
quota enforcement. Error tags/reason names such as `internerlim` and
`InternerLimit` remain ordinary strings, not new symbol insertions.
The static pool is counted at initialization; its lookup never allocates
or lazily builds another map. A reserved pair constructed through any
fallible constructor or serde must have **the same ID and text** as its
typed token. Reserving storage does not add a name to the semantic builtin
catalog, give it attributes, declare a stub, or make `NameQ` true.
(`crates/rum-runtime/src/runtime/session/session_helpers.rs:311-394`;
`crates/rum-runtime/src/runtime/suspension/error.rs:3-5,268-294`;
`crates/rum-runtime/src/runtime/resolution.rs:141-157`.)

Keep actual builtin implementations/specifications in their owning crates.
The new core vocabulary contains spelling/identity data only. Fixed catalog
entries carry a typed reserved-name proof, with the `BuiltinSpec` name
**derived from the token**, not separately supplied and trusted to match.
They use an infallible registration path accepting that proof; arbitrary
public `register_builtin(BuiltinSpec)` and `install_system_own_value(&str, …)`
return `Result`. Existing installed-catalog enumeration uses the map's
`Symbol` key rather than re-interning its spec's string name. This keeps
fixed default/session bootstrap and standard-library installation usable
even when a different tenant has filled the quota.
(`ARCHITECTURE.md:184-220`;
`crates/rum-runtime/src/runtime/builtin_catalog.rs:20-28,80-96`;
`crates/rum-runtime/src/runtime/session/session_state.rs:681-692`;
`crates/rum-runtime/src/runtime/session/session_api.rs:271-307`;
`crates/rum-runtime/src/runtime/messages/message_catalog.rs:385-398`;
`crates/rum-stdlib/src/lib.rs:42-59`.)

Add pure operations `Symbol::without_identity(self)` and
`Symbol::with_existing_identity(self, NonZeroU64)`. They copy the existing
text reference/ID and change only the identity field. Use them for the
profiling text keys, rule-RHS alpha renaming and sealed-reference relinking.
These sites have an actual existing pair as their proof; calling a
fallible constructor and asserting it succeeds would be weaker.
(`crates/rum-runtime/src/runtime/profiling/records.rs:18,51,282,326`;
`crates/rum-runtime/src/runtime/profiling/timing.rs:79-81,113,192,314-316`;
`crates/rum-runtime/src/runtime/profiling/presence.rs:268`;
`crates/rum-runtime/src/runtime/evaluation_cache/profile/mod.rs:279`;
`crates/rum-runtime/src/runtime/substitution/binding_substitution.rs:669-675`;
`crates/rum-runtime/src/runtime/scoping/packages.rs:392-397`.)

#### Configuration, default and observation

Expose process functions from `rum-core` and re-export them through a
`rum-host` process-resources module:
`configure_process_interner(InternerConfig) -> Result<InternerStats,
InternerConfigError>` and
`process_interner_stats() -> Result<InternerStats, InternerError>`. There is
**one successful configuration**, chosen by the embedding application's startup
code. No `SessionConfig`, `HostResourceLimits`, builtin, installer callback
or reset operation can raise it. A second configuration, including an
attempt to raise the same running process's limit, returns
`AlreadyInitialized`. An invalid too-small/overflowing configuration does
not seal the configuration and can be corrected before use.
(`crates/rum-core/src/expr.rs:47-55`;
`crates/rum-host/src/config.rs:55-78`;
`crates/rum-host/src/session.rs:147-175`.)

The reserved tables exist independently of dynamic initialization. Typed
reserved lookups and an observational stats read do not seal the choice.
The first arbitrary-name constructor call, including one that turns out
to be a hit, or explicit host startup initialization seals the default if
no configuration was supplied. Configuration and first use share the same
synchronization/state transition, so racing callers cannot both win or
silently replace a limit. Reject a configured total below the measured
reserved/fixed baseline; never silently raise it to fit bootstrap.
(`crates/rum-core/src/expr.rs:47-55,98-138`;
`crates/rum-host/src/session.rs:84-95,147-159`.)

**Recommended initial default: 8 MiB total charged storage, including the
reserved baseline and growth reservations. This is a proposed policy
number, not a measured working-set requirement.** It is deliberately a
small, finite process allowance independent of tenant/request limits. A
sequence of distinct 64 KiB names must refuse within 129 attempts solely
from text bytes, even before pairs and indexes are charged. That is a cheap
checked-in default-cap test, and repeated bounded requests cannot postpone
it by resetting their sessions. The arithmetic is reproduced verbatim in
section 2; the existing generated-element and iteration defaults are
100,000, so these attempts need no enormous single evaluation or disabled
work limits. Arbitrarily long names still refuse on their length before
leaking, rather than relying on a name count.
(`SECURITY.md:48-58`; `crates/rum-core/src/expr.rs:57-96`;
`crates/rum-host/src/session.rs:172-175`.)

Why this is not an asserted universal memory guard: there is no declared
minimum process-memory envelope or legitimate interner working-set
measurement in those contracts. An 8 MiB setting can still be too high for
a constrained host or too low for a legitimate long-lived workload.
**D1 requires the director to approve it against Draft A's storage baseline
and Draft B's actual suite/panel usage, without silently enlarging it when
it trips.** The new default must pass its real default-cap refusal test,
not just a tiny test-only quota. If a supported embedding exhausts process
memory before this guard can fire, that configuration repeats the native
stack guard's fail-open mistake and must be reduced or its support claim
withdrawn. No default substitutes for a host's hard process-memory budget.
(`TODO.md:1304-1319`; `docs/host-embedding-and-hardening.md:54-68,130-132`.)

`InternerStats` reports configured/uninitialized status, effective limit,
reserved baseline, retained text bytes, pair bytes, index bytes, retained
charged total, outstanding growth reservation/peak, and text/pair counts.
Read an internally consistent O(1) snapshot on demand under the interner
lock; before initialization report the baseline/default without freezing
configuration. A poisoned lock is an observation error, not a fabricated
zero. Hosts use these process values for health/admission/recycling policy.
`MemoryInUse`, `MaxMemoryUsed`, `ByteCount` and
`current_memory_proxy_bytes` retain their current session/expression
meanings; do not add the process total or charge insertion deltas there.
No new per-evaluation polling/counters are introduced.
(`crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:6-56,335-365`;
`crates/rum-core/src/expr.rs:81-84`; `AGENTS.md:130-132`.)

#### Refusal propagation and persistence

Map core errors to `EvalError::Interner(InternerError)` in runtime, preserving
structured numeric data through `SessionRequestError` and the host failure
mapping. Byte/ID exhaustion yields a specific `InternerLimit` termination
and `General::internerlim`, never `MemoryLimit`, invalid syntax, a missing
symbol, or a successful arbitrary replacement value. An evaluator request,
whether the error arose during lowering or evaluation, finishes as
`TerminatedEvaluation["InternerLimit"]` with the structured termination;
a lower-only request or typed-host-input construction returns an interner
resource failure. Direct `Session::evaluate` returns the `EvalError`.
The new error passes through ordinary suspension/cleanup paths and is
encoded with pre-reserved continuation tags. `MemoryConstrained` cannot
consume it as a local memory-limit failure or restore its process budget.
(`crates/rum-runtime/src/runtime/session/session_state.rs:499-574`;
`crates/rum-runtime/src/runtime/model/requests.rs:157-219`;
`crates/rum-runtime/src/runtime/session/session_requests.rs:137-171,216-246`;
`crates/rum-runtime/src/runtime/session/session_eval.rs:37-60`;
`crates/rum-runtime/src/runtime/suspension/error.rs:268-294`;
`docs/guide/resource-limits-and-sandbox.md:294-315`.)

Core serde keeps its wire shape. Deserialize the fields, validate identity,
call the single fallible constructor, and convert the refusal with
`serde::de::Error::custom` into `D::Error`. Derived `Atom`, `Expr` and shared
containers therefore propagate failure through their existing trait
channel. Preserve context/name/identity serialization; do not serialize the
process ID or the process configuration. A later error in a composite
snapshot does not undo earlier admitted names, but the particular refused
pair leaks nothing. Serde's temporary input strings are not immortal
interner storage and are not falsely charged to a session.
(`crates/rum-core/src/expr.rs:288-319,536,573,637,749`.)

For host typed input, replace the internal `Result<Expr, String>` error with
a typed conversion error containing either the existing validation detail
or `InternerError`; do not stringify the refusal and later parse it back to
classify it. The host exposes a specific interner failure code which does
not require reset. Boolean/List/Association/Rule construction uses reserved
symbols. The actual arbitrary ingress is `HostTyped::Symbol`'s resolver,
not the two direct `Expr::system_symbol` calls in this file.
(`crates/rum-host/src/model/outcome.rs:110-160,473-565`.)

Downstream installer callbacks gain `Result<(), InternerError>`; host
builder `build`, configured construction and `reset` gain a host error
channel. Build a replacement session completely before swapping it into a
live host; a failed reset leaves the old session/poison state intact and
reports the failure. Previously admitted process names remain charged.
Plain `Session::default`/`new` and the fixed default `HostSession::new` can
remain infallible only by calling the separately typed fixed installer
path described above, never by `expect` on the configurable builder.
`HostSessionBuilder::default` remains an infallible description of a
configuration. The callback API change is D3, explicitly documented.
(`crates/rum-host/src/config.rs:12-38`;
`crates/rum-host/src/session.rs:30-41,76-95,127-175`;
`crates/rum-runtime/src/runtime/session/session_state.rs:681-692`.)

`restore_from_bytes` remains source execution under the target session's
capability policy. Handle a failed reset and an interner refusal during
source evaluation. **Inspect the structured termination of an `Ok`
request response**: the current `Ok(_) => Ok(())` must not turn the new
`InternerLimit` termination into a successful restore. Return a specific
`HostFailure` with messages/prints/trace preserved, without poisoning the
host. A restore that has begun evaluating can leave partially restored
session state and permanently admitted names; this work does not promise
transactional snapshot rollback. Invalid UTF-8 still fails before reset.
(`crates/rum-host/src/session.rs:201-220,717-738`;
`crates/rum-runtime/src/runtime/session/session_api.rs:74-83`;
`TODO.md:941-944`; `docs/host-embedding-and-hardening.md:95-101`.)

#### Ordered work and gates

The numbered steps below are one implementation sequence. Steps 2-5 belong
to one complete, compiling integration change; do not land a partially
fallible API backed by panic wrappers. Draft A proves the private storage
mechanism first; Draft B performs that integration; Draft C independently
checks it. Source anchors are the current edit points, not a promise that
line numbers survive migration.
(`docs/dev/codex/TEMPLATE.md:19-37`; `TODO.md:1201-1211`.)

| Step | Files / functions and resulting behavior | Gate before moving on |
|---|---|---|
| 1. Prove byte layout and bootstrap | Draft A scratch prototype corresponding to `crates/rum-core/src/expr.rs:23-138` and `crates/rum-core/src/heads.rs:112-194`; explicit slot layouts, growth/staging ledger, reserved vocabulary, scalar errors and pure admission seam. | Independent layout/charge reconstruction; exact-fit and one-byte-short refusals; failed pair with two new strings leaves no retained allocations; all construction temporaries listed; immutable reserved pairs unify with text lookup. No performance claim from a prototype. |
| 2. Close every infallible insertion path | Proposed core symbol/interner/reserved modules, core exports, runtime `expr_helpers`, fixed catalog/defaults and continuation modules, stdlib operator helpers and host typed fixed values. Migrate fixed catalog specifications to typed reserved-name proof; add identity-only copy helpers. Current anchors: `crates/rum-core/src/expr.rs:98-138`, `crates/rum-runtime/src/runtime/expr_helpers.rs:9`, `crates/rum-runtime/src/runtime/session/session_api.rs:13-54`, `crates/rum-runtime/src/runtime/suspension/error.rs:3-5`, `crates/rum-host/src/model/outcome.rs:508,565`. | `Copy + Send + Sync + 'static` and layout checks; unchanged Eq/Hash/Ord behavior; compile-fail tests reject strings as reserved tokens; reserve-set/catalog consistency; ordinary fixed bootstrap and error construction at a full cap. |
| 3. Make arbitrary construction and normalization fallible | The seven core constructors/serde; runtime lowering, resolution, Unique/Module/packages, pattern-equivalence preparation, message-key materialization; public builtin registration. `crates/rum-core/src/expr.rs:109-138,312-319,801-811`; `crates/rum-runtime/src/runtime/lowering.rs:93-103`; `crates/rum-runtime/src/runtime/scoping/unique.rs:36-138`; `crates/rum-runtime/src/runtime/definitions/definition_analysis_helpers.rs:393-429`; `crates/rum-runtime/src/runtime/messages/message_format.rs:3-30`; `crates/rum-runtime/src/runtime/session/session_api.rs:271-297`. | Compiler-forced inventory to a workspace fixpoint, both recursive/iterative lowering; no `Err -> None`, no quota `expect`; staged definition/package mutations remain coherent on failure. Dedicated core/runtime cap processes cover each error class in section 2. |
| 4. Complete host and request mapping | Runtime `session_state`, request models/helpers/evaluation, continuation error encoding; host config/session/outcome and affected adapters. `crates/rum-runtime/src/runtime/session/session_requests.rs:137-171`, `crates/rum-runtime/src/runtime/session/session_eval.rs:37-60`, `crates/rum-runtime/src/runtime/session/session_helpers.rs:311-394`; `crates/rum-host/src/config.rs:12-38`, `crates/rum-host/src/session.rs:76-95,147-220`, `crates/rum-host/src/model/outcome.rs:473-565`. | Same structured interner reason from lowering and evaluation; lower-only/typed/installer failures retain their own error channels; error while suspended unwinds correctly; restore rejects the terminated success response; reset/new session cannot replenish quota. |
| 5. Ship the falsifier and exact contracts | Dedicated process-isolated core/runtime/host integration tests and private planner unit tests; rustdoc plus `SECURITY.md`, `docs/host-embedding-and-hardening.md`, `docs/guide/resource-limits-and-sandbox.md`, affected embedding examples/reference catalog. Current documentation anchors: `SECURITY.md:68-88`, host hardening `:54-59,86-101,130-132`, resource guide `:292-315`. | Section 3's negative/positive controls and default-cap test; a copied-source guard-removal mutation makes the checked-in test fail for the intended reason; reference drift gate and offline workspace suite. |
| 6. Measure and independently review the finished candidate | Supplied saved base/candidate binaries, current microbench and cold/warm constructor probes in authorized scratch. No Git operations to manufacture a base. | `cargo fmt --check`, full offline workspace tests and required depth/host lifecycle gates; full `core-language-microbench.sh` base/new `timings.tsv`, all hot/same-tier neighbors. Report any regression, no tolerance borrowed from the rejected ownership design. |

Every proposed index/Result/lookup change has **unmeasured** cost. Expected
extra work occurs at insertion, resolution and the pattern-normalization
Result propagation; static tokens and identity copies can remove current
locks, but that is an inference, not evidence of zero net cost. Measure
literal/symbol/list/equality, Module/Block/Table, downvalue/matcher/rewrite,
symbol-sort and undefined-symbol neighbors. Include cold miss, warm hit,
long-text, growing-index and near-cap controls separately; do not time a
loop that merely terminates early and call it a speedup.
(`AGENTS.md:46-49,117-132`;
`scripts/core-language-microbench.sh:479-535,566-586,981-992,1021-1037`;
`docs/testing-strategy.md:74-94`.)

#### Exact documentation text to land with implementation

**`SECURITY.md`, alongside the memory settings and isolation text:**

> Symbol interning uses a separate process-wide byte quota. Interned text,
> symbol-pair records and their indexes are shared by all sessions and live
> for the process lifetime. They are outside the per-session modeled-memory
> ceiling. The interner quota bounds charged storage, including complete
> index storage and reserved bootstrap data; it is not an allocator or RSS
> limit. Existing interned pairs remain usable when admission of new pairs
> is refused. One tenant can permanently exhaust this shared allowance.
> `reset()` does not reclaim interned names or restore this allowance. Hard
> multi-tenant isolation requires host process isolation and process memory
> limits; recycling this storage means ending its owning process.

This replaces any implication that the optional session ceiling alone
bounds this shared storage, without marking the original finding closed.
(`SECURITY.md:70-88`; `TODO.md:1201-1211`.)

**`docs/host-embedding-and-hardening.md`, resource/lifecycle sections, and
matching `rum-host` process-resource/reset rustdoc:**

> Configure the process interner quota once during host startup, before an
> arbitrary-name constructor or session initialization seals the default.
> The default total is 8 MiB of charged interner storage. A limit below the
> reserved baseline is rejected. The configuration cannot be raised or reset
> by a session. Use `process_interner_stats()` to observe shared usage and
> remaining capacity; these values are not tenant attribution. A successful
> session reset preserves this usage. An interner refusal is recoverable
> without poisoning the session, but reset cannot make the refused new name
> admissible. A failed installer or reset returns a host error; a failed
> replacement leaves the prior session in place.
>
> Snapshot restore evaluates UTF-8 source under the target session's
> capability policy. Treat untrusted snapshot bytes as untrusted code.
> Restore can fail because the process interner quota is exhausted. A
> restore stopped by that quota is reported as an error, and names admitted
> before the failure remain charged. A restore that has started evaluation
> is not an atomic rollback transaction.

The default sentence is conditional on D1; replace the number only by an
explicit director ruling backed by the calibration evidence, not by
implementation-time tuning.
(`docs/host-embedding-and-hardening.md:54-59,86-105`;
`crates/rum-host/src/session.rs:162-220`; `TODO.md:941-944`.)

**`docs/guide/resource-limits-and-sandbox.md`, next to Memory Ceilings:**

> Creating a new symbol may reach the shared process interner limit. Source
> evaluation then returns `TerminatedEvaluation["InternerLimit"]`, reports
> the `InternerLimit` termination reason and emits `General::internerlim`
> with the configured byte limit and usage. `MemoryConstrained` does not
> handle or raise this process quota. Existing symbol pairs remain usable;
> new requests, session reset and a Pure-only policy do not replenish the
> allowance. All tenants in the process share it. Hosts requiring hard
> tenant isolation must isolate and bound their worker processes.

Core constructor/serde rustdoc must also document `Result`, the preserved
wire fields, zero-identity refusal, existing-pair behavior at byte/ID caps,
and the charged-storage definition. Update the generated reference through
its owning catalog, not by hand, if its Symbol/Unique/resource descriptions
change. The broader **per-request versus per-session state inventory is a
separate brief**; this report does not answer it.
(`crates/rum-core/src/expr.rs:109-138,164-168,288-319`;
`docs/guide/resource-limits-and-sandbox.md:292-315`;
`docs/testing-strategy.md:60-72`; `TODO.md:953-960`.)

### 2. The call-site inventory

#### Method and measured source counts

The census scanned all Rust files returned by repository-local `rg --files`
with `.git` and `target` excluded. A word-boundary-qualified multiline regex
finds the seven constructor names, including function references rather
than just one-line calls; it does not count `SyntaxExpr::symbol`. I then
inspected their enclosing code and the private `Self::new` wrappers, traced
`system_expr`, and separately searched aliases/private insertion paths and
higher-level producers. Test classification uses test-file ownership and
in-file `#[cfg(test)]` modules, not merely a filename containing “test”
(`debug_tools/test_report` is production). These are lexical occurrence
counts and source-reviewed dispositions; no compiler migration was run.
(`crates/rum-core/src/expr.rs:108-138,936`;
`crates/rum-runtime/src/runtime/expr_helpers.rs:3-10`;
`crates/rum-runtime/src/runtime/debug_tools/test_report/evaluation.rs:169`.)

Verbatim output of the bounded inline-Python source census and its
classification pass follows. Raw source text and columns are retained in
`target/brief-505/direct-inventory.tsv`; all locations are also pasted below,
so the reviewable inventory does not depend on retaining ignored scratch.

```text
Rust files scanned: 1007
Direct constructor references: 337
Files containing direct references: 78
Expr::symbol: 146
Expr::system_symbol: 11
Symbol::from_name: 49
Symbol::new: 70
Symbol::system: 49
Symbol::with_identity: 12
By crate:
crates/rum-core: 12
crates/rum-host: 2
crates/rum-runtime: 239
crates/rum-stdlib: 84
Direct Symbol constructor occurrences: 180
Distinct lines with direct Symbol constructors: 179
Direct Expr constructor occurrences: 157
Internal Symbol Self::new wrappers: 2
Total direct inventory including wrappers: 339
Class B: 52
Class E: 13
Class F: 5
Class H: 2
Class M: 2
Class P: 1
Class S: 2
Class T: 248
Class X: 14
Canonical core head entries: 81
Brief original lines: 181
```

The 179-line regex total therefore undercounts even its own four
constructors: `crates/rum-runtime/src/runtime/options/core.rs:12` has two calls. The internal wrappers and
the Expr/helper layers are further, separately identified edges. Do not sum
helper counts into a supposed distinct-error total: for example
`system_expr`'s implementation is already one of the 157 Expr references.
The checked-in compiler tests and the future workspace migration, not this
census, enforce the final closure.
(`crates/rum-runtime/src/runtime/options/core.rs:12`;
`crates/rum-core/src/expr.rs:129,137`;
`crates/rum-runtime/src/runtime/expr_helpers.rs:9-10`.)

#### Dispositions, including sites with no current error channel

| Code | What happens to refusal in this class | Current anchors / required channel |
|---|---|---|
| F | Core factory/wrapper returns `Result<_, InternerError>`; no hidden infallible wrapper. | `crates/rum-core/src/expr.rs:129,137,801-811`. The four public Symbol definitions are the root API changes, not extra call sites. |
| S | `D::Error` via `serde::de::Error::custom`; zero identity is rejected before any admission. | `crates/rum-core/src/expr.rs:312-319`. Derived containers propagate the trait error. |
| E | `EvalError::Interner`, with `Result<Option<T>, …>` where `None` currently means invalid/missing input. Never reinterpret refusal as `None`. | `crates/rum-runtime/src/runtime/resolution.rs:101-138`; `crates/rum-runtime/src/runtime/scoping/unique.rs:36-138`; `crates/rum-runtime/src/runtime/scoping/packages.rs:270-289,420-429`; `crates/rum-runtime/src/runtime/options/core.rs:5-17,101-106`; `crates/rum-runtime/src/runtime/package_tools.rs:48-79`. |
| P | Create a Result channel through pattern-name canonicalization, equivalence, hash preparation and definition-store preparation. | `crates/rum-runtime/src/runtime/definitions/definition_analysis_helpers.rs:393-429`; `crates/rum-runtime/src/runtime/definitions/rhs_guards.rs:52-76`; `crates/rum-runtime/src/runtime/model/definitions/tagged_value_store.rs:51-95,227-276`; `crates/rum-runtime/src/runtime/model/definitions/down_value_dispatch.rs:545-572`. These are helper functions, not the immutable Symbol Eq/Hash implementations. Replace `position`/`retain` callbacks by a fallible planning pass followed by mutation; never swallow failure as “not equivalent”. |
| M | Raw-name/message-key construction becomes fallible; evaluator callers propagate `EvalError`. Message-list/report builders return Result too. The refusal's own formatted runtime/host message uses reserved symbols/strings. | `crates/rum-runtime/src/runtime/messages/message_format.rs:3-30`; `crates/rum-runtime/src/runtime/messages/message_selector_parsing.rs:83-104`; `crates/rum-runtime/src/runtime/messages/message_catalog.rs:173-250`; `crates/rum-runtime/src/runtime/eval_core/symbol.rs:250-257`; `crates/rum-runtime/src/runtime/debug_tools/test_report/report.rs:288,363,585`. |
| H | Public arbitrary builtin registration / own-value installation returns a core interner error; installer/build/reset maps it to a typed host failure. | `crates/rum-runtime/src/runtime/session/session_api.rs:271-297`; `crates/rum-host/src/config.rs:12-38`; `crates/rum-host/src/session.rs:76-95,147-175`. Existing `()` callbacks cannot silently discard refusal. |
| B | Prove no insertion: a typed reserved name, an existing registered map key, or an existing head handle. The operation remains infallible **after** this conversion; its present string-taking API is not proof. | Fixed static maps: `crates/rum-runtime/src/runtime/builtin_catalog.rs:80-96`; fixed defaults: `crates/rum-runtime/src/runtime/options/defaults.rs:3-15`; registered custom entries: `crates/rum-runtime/src/runtime/messages/message_catalog.rs:385-398`; open head-bearing helpers must accept an existing Symbol/Expr, while finite operator selectors accept reserved tokens. |
| X | Pure identity removal/rebinding of an existing Symbol. No interner call and no new quota error channel. | Profiling sources and `crates/rum-runtime/src/runtime/substitution/binding_substitution.rs:669-675`; sealed relink `crates/rum-runtime/src/runtime/scoping/packages.rs:392-397`. |
| T | Test setup explicitly unwraps/asserts successful arbitrary construction in ordinary tests, or uses a reserved token for a genuine production reserved name. Tests of refusal assert the error in isolated binaries. | Representative in-file boundary: `crates/rum-core/src/expr.rs:936-990`; direct locations below are exhaustive. No normal test changes the real process quota. |

Additional no-channel edges are material, even though they are not seven-name
constructor hits:

- **Resolution/inspection/existence:** separate a no-insertion lookup from
  `resolve_symbol_name`. Context-path existence probes and `NameQ` must not
  create speculative names just to test existence. Pure symbol/definition
  inspection can use that lookup and preserve its not-found result.
  A message-catalog/query path that must materialize a new `Symbol` gains
  `Result<Option<_>, EvalError>` and a host inspection error channel; it may
  not convert refusal to “not found”. The debug-only check in `push_message`
  also uses lookup, keeping that string-output API infallible.
  (`crates/rum-runtime/src/runtime/resolution.rs:101-157`;
  `crates/rum-runtime/src/runtime/namespace_tools/names.rs:160,329`; `crates/rum-runtime/src/runtime/session/session_eval.rs:85-175`;
  `crates/rum-runtime/src/runtime/session/session_helpers.rs:249-281`.)
- **Pattern equivalence and mutation:** `Pattern`-context `pN` symbols cannot
  be placed in a finite reserve just because the spelling is predictable.
  Precompute fallible equivalence decisions before changing a definition
  table or replacing an index. Do the same for DeclarePackage's stub batch
  before changing the context path. Preserve the current canonicalization
  semantics; this is not authorization to substitute a different pattern
  identity scheme. (`crates/rum-runtime/src/runtime/definitions/definition_analysis_helpers.rs:404-429`;
  `crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation/prepared_store.rs:449,470,495,665`;
  `crates/rum-runtime/src/runtime/package_tools.rs:68-78`.)
- **Continuation traits and error handling:** `ContinuationValue::into_continuation_expr`
  returns `Expr`, so its tag constructors must be typed reserved lookups.
  It must not call a fallible string constructor and panic while encoding
  the original refusal. Existing assertions about malformed private
  continuation payloads are programmer-invariant checks; none may become
  a path for quota refusal. (`crates/rum-runtime/src/runtime/suspension/error.rs:3-15,268-294`;
  `crates/rum-runtime/src/runtime/pattern_matching/continuation.rs:4`;
  `crates/rum-runtime/src/runtime/scoping/lexical/block/continuation.rs:4`.)
- **Plain-Expr operator builders and value decoders:** preserve their exact
  parameter names through reserved Global pairs, and propagate typed
  reserved operators through the private helper signatures. YAML's
  `symbol_value` is closed to Null/True/False, not arbitrary decoded tags;
  the apparent dynamic polynomial variable constructors are in test
  modules. (`crates/rum-stdlib/src/associations/data.rs:346-365`;
  `crates/rum-stdlib/src/structure_ops/collection_stats.rs:894-903`;
  `crates/rum-stdlib/src/structure_ops/ordering_sets.rs:981-989`;
  `crates/rum-stdlib/src/yaml.rs:184-185`; `crates/rum-stdlib/src/yaml/decode/scalars.rs:109,127-129`;
  `crates/rum-stdlib/src/polynomials/core/variables.rs:57-105`.)

**No production site is assigned “keep a panic for quota exhaustion”.**
Infallible sites instead lose the ability to insert, and genuinely
inserting sites acquire a channel. Fixed `Default` implementations use the
typed fixed installer path; custom installers cannot enter that proof.
Test-only `expect`s assert their setup contract and are not a production
escape hatch. Public callers can of course deliberately unwrap a Result;
the library cannot forbid caller-written panic policy, but it no longer
provides one as its arbitrary-name constructor.
(`crates/rum-core/src/expr.rs:109-138`;
`crates/rum-runtime/src/runtime/session/session_state.rs:681-692`;
`crates/rum-host/src/session.rs:127-159`.)

#### Complete direct inventory

Abbreviations: N=`Symbol::new`, G=`Symbol::from_name`,
Y=`Symbol::system`, I=`Symbol::with_identity`, eG=`Expr::symbol`,
eY=`Expr::system_symbol`, SelfN=the internal Symbol wrapper. Each number
is a line in the file in that row; repeated numbers denote separate
occurrences. No direct `Expr::contextual_symbol` caller was found, but its
public definition still changes. Factual source locations below support
this inventory's dispositions; the dispositions describe the proposed
migration, not current fallibility.
(`crates/rum-core/src/expr.rs:805-807`.)

| Source file | Disposition | Constructor and line (repetition means distinct occurrences) |
|---|---|---|
| `crates/rum-core/src/expr.rs` | B | N:103 |
| `crates/rum-core/src/expr.rs` | F | SelfN:129, SelfN:137, G:802, N:806, Y:810 |
| `crates/rum-core/src/expr.rs` | S | I:316, N:317 |
| `crates/rum-core/src/expr.rs` | T | G:942, Y:948, eG:956, eG:957, eY:989, eG:990 |
| `crates/rum-host/src/model/outcome.rs` | B | eY:508, eY:565 |
| `crates/rum-runtime/src/runtime/builtin_catalog.rs` | B | Y:91 |
| `crates/rum-runtime/src/runtime/builtin_catalog.rs` | T | N:125, N:138 |
| `crates/rum-runtime/src/runtime/builtins_input.rs` | B | Y:197, Y:290 |
| `crates/rum-runtime/src/runtime/call_preparation.rs` | T | G:418, G:454 |
| `crates/rum-runtime/src/runtime/debug_tools/monitor.rs` | B | Y:72 |
| `crates/rum-runtime/src/runtime/debug_tools/test_report/evaluation.rs` | B | Y:169, Y:243 |
| `crates/rum-runtime/src/runtime/definitions/application/default_values.rs` | B | Y:55 |
| `crates/rum-runtime/src/runtime/definitions/application/replace_all/plan.rs` | T | eG:155, eG:156, eG:159, eG:160, eG:160 |
| `crates/rum-runtime/src/runtime/definitions/definition_analysis_helpers.rs` | P | N:414 |
| `crates/rum-runtime/src/runtime/definitions/definition_clear.rs` | B | Y:233 |
| `crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation.rs` | T | eG:743, eG:743, eG:744 |
| `crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation/syntax_policy.rs` | T | eG:176 |
| `crates/rum-runtime/src/runtime/definitions/definitions.rs` | B | Y:247, Y:264, Y:281 |
| `crates/rum-runtime/src/runtime/definitions/incremental_append.rs` | B | Y:155 |
| `crates/rum-runtime/src/runtime/definitions/unique_symbols.rs` | T | G:55, G:73 |
| `crates/rum-runtime/src/runtime/evaluation_cache/profile/mod.rs` | X | N:279 |
| `crates/rum-runtime/src/runtime/evaluation_cache/tests.rs` | T | N:76, I:98, I:108, N:140, N:250, N:327, eG:355, eG:356, eG:357, eG:358 |
| `crates/rum-runtime/src/runtime/evaluation_cache/tests/association_lvalue.rs` | T | I:44, N:164, N:175, eG:231, eG:251, eG:267, eG:282, eG:297, eG:328, N:351, N:366 |
| `crates/rum-runtime/src/runtime/evaluation_cache/tests/association_stability_tail.rs` | T | eG:48, eG:70, eG:97, eG:101, eG:126, eG:130, N:158, eG:160, N:197, eG:199, eG:203, eG:229 |
| `crates/rum-runtime/src/runtime/expr_helpers.rs` | B | eY:10 |
| `crates/rum-runtime/src/runtime/expr_helpers.rs` | E | N:5, Y:6 |
| `crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs` | B | Y:102, Y:103 |
| `crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/table_fast_path.rs` | B | Y:208 |
| `crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation.rs` | B | Y:119 |
| `crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs` | B | Y:435 |
| `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/list.rs` | B | eG:251 |
| `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/suspended_state.rs` | T | G:187 |
| `crates/rum-runtime/src/runtime/kernel_data/structure_data/exact_paths_tests.rs` | T | eG:5, eG:8, eG:12, eG:112, eG:121, eG:127, eG:135, eG:138, eG:142 |
| `crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/replace_at.rs` | B | eG:213 |
| `crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/suspended_state.rs` | T | G:357 |
| `crates/rum-runtime/src/runtime/lowering/tests.rs` | T | eG:14, eG:20, eG:21, eG:28, N:148, G:177, eG:189, eG:190, eG:190, eG:296 |
| `crates/rum-runtime/src/runtime/messages/builtins_messages_inspect.rs` | B | Y:192, Y:312, N:489 |
| `crates/rum-runtime/src/runtime/messages/message_catalog.rs` | B | Y:388 |
| `crates/rum-runtime/src/runtime/messages/message_format.rs` | M | N:5, G:7 |
| `crates/rum-runtime/src/runtime/namespace_tools/names.rs` | B | Y:75, Y:92, Y:270, Y:281, Y:285, Y:378 |
| `crates/rum-runtime/src/runtime/options/core.rs` | E | Y:12, Y:12, Y:106 |
| `crates/rum-runtime/src/runtime/options/defaults.rs` | B | Y:5 |
| `crates/rum-runtime/src/runtime/package_tools.rs` | B | Y:218, Y:641, Y:661, Y:680, Y:699, Y:703, Y:712, Y:715, Y:722 |
| `crates/rum-runtime/src/runtime/package_tools.rs` | E | N:72 |
| `crates/rum-runtime/src/runtime/pattern_syntax.rs` | T | eG:90 |
| `crates/rum-runtime/src/runtime/profiling/presence.rs` | X | N:268 |
| `crates/rum-runtime/src/runtime/profiling/records.rs` | X | N:18, N:51, N:282, N:326 |
| `crates/rum-runtime/src/runtime/profiling/timing.rs` | X | N:79, N:81, N:113, N:192, N:314, N:316 |
| `crates/rum-runtime/src/runtime/resolution.rs` | E | Y:131, I:137, N:138 |
| `crates/rum-runtime/src/runtime/scoping/packages.rs` | E | N:270, N:277, I:425 |
| `crates/rum-runtime/src/runtime/scoping/packages.rs` | X | N:395 |
| `crates/rum-runtime/src/runtime/scoping/unique.rs` | E | N:42 |
| `crates/rum-runtime/src/runtime/session/session_api.rs` | B | Y:27, eY:36, eY:37 |
| `crates/rum-runtime/src/runtime/session/session_api.rs` | H | Y:272, Y:292 |
| `crates/rum-runtime/src/runtime/substitution/binding_substitution.rs` | T | G:760, G:764, G:765, G:785, G:789, G:807, G:810, G:824, G:825 |
| `crates/rum-runtime/src/runtime/substitution/binding_substitution.rs` | X | I:671 |
| `crates/rum-runtime/src/runtime/substitution/lexical_substitution.rs` | T | G:616, G:617, G:661, G:662, G:706, G:707 |
| `crates/rum-runtime/src/runtime/suspension/primitive.rs` | T | G:208, G:209, G:213, eG:226, eG:242 |
| `crates/rum-runtime/src/runtime/suspension/rewrite_rules.rs` | T | eG:94, eG:95, eG:122 |
| `crates/rum-runtime/src/runtime/tests/arithmetic.rs` | T | eG:40, eG:45, eG:433 |
| `crates/rum-runtime/src/runtime/tests/association_part_cursor.rs` | T | eG:27, eG:28 |
| `crates/rum-runtime/src/runtime/tests/association_write_index.rs` | T | N:22, I:34 |
| `crates/rum-runtime/src/runtime/tests/association_write_recovery.rs` | T | N:51, I:88 |
| `crates/rum-runtime/src/runtime/tests/definitions_rhs_guards_attribute_revision.rs` | T | N:245 |
| `crates/rum-runtime/src/runtime/tests/definitions_rhs_guards_own_values.rs` | T | N:100 |
| `crates/rum-runtime/src/runtime/tests/frame_root_trace/root_visitation.rs` | T | eG:65, eG:65, G:66, eG:78, eG:79, eG:79, eG:99, eG:101, G:102, G:127, G:128, eG:130, eG:132, eG:134, G:135, eG:136, eG:137, eG:140, G:140, eG:154, G:155, eG:157, G:157, eG:161, eG:162, G:178, eG:178, G:192, eG:193, eG:196, eG:231, eG:232, G:249 |
| `crates/rum-runtime/src/runtime/tests/match_normalization_contract.rs` | T | G:29, G:33, G:37, Y:44, eG:53, eG:68, eG:99, eG:108, eG:736 |
| `crates/rum-runtime/src/runtime/tests/mutation_operator_forms.rs` | T | N:281, N:579 |
| `crates/rum-runtime/src/runtime/tests/part_assignment.rs` | T | N:120, N:333, N:341, N:354, N:363, N:371, N:383, N:424 |
| `crates/rum-runtime/src/runtime/tests/scoping_gensym_gc.rs` | T | G:250, G:478, G:484 |
| `crates/rum-runtime/src/runtime/tests/scoping_gensym_gc/reachability.rs` | T | G:227 |
| `crates/rum-runtime/src/runtime/tests/sealed_packages.rs` | T | I:275 |
| `crates/rum-runtime/src/runtime/tests/sealed_snapshot.rs` | T | N:126 |
| `crates/rum-runtime/src/runtime/tests/suspension_abandonment.rs` | T | N:246, N:247 |
| `crates/rum-stdlib/src/associations/data.rs` | B | eG:347, eG:358 |
| `crates/rum-stdlib/src/functional_ops/arrays/mod.rs` | B | Y:90, Y:91 |
| `crates/rum-stdlib/src/predicates/collections.rs` | T | eG:838, eG:839, eG:839 |
| `crates/rum-stdlib/src/strings/patterns/matcher/api.rs` | T | G:437 |
| `crates/rum-stdlib/src/strings/patterns/matcher/state_set.rs` | T | G:178 |
| `crates/rum-stdlib/src/structure_ops/collection_stats.rs` | B | eG:895 |
| `crates/rum-stdlib/src/structure_ops/ordering_sets.rs` | B | eG:982 |
| `crates/rum-stdlib/src/structure_ops/sequence/edit_plan.rs` | T | eG:227, eG:255, eG:256, eG:287, eG:288, eG:349, eG:352, eG:353, eG:353, eG:355, eG:355 |
| `crates/rum-stdlib/src/structure_ops/sequence/insert.rs` | T | eG:226, eG:234 |
| `crates/rum-stdlib/src/tests/association_lookup_depth.rs` | T | eG:171, eG:206 |
| `crates/rum-stdlib/src/tests/held_association_exact_path_depth.rs` | T | eG:211, eG:236, eG:256, eG:277, eG:322, eG:325, eG:328, eG:356, eG:359, eG:362, eG:375, eG:382, eG:391, eG:396, eG:399, eG:402, eG:405, eG:423, eG:426, eG:429, eG:443, eG:446, eG:453, eG:464, eG:466, eG:467, eG:485, eG:534 |
| `crates/rum-stdlib/src/tests/map_indexed_association_depth.rs` | T | eG:132, eG:184, eG:228, eG:230, eG:232 |
| `crates/rum-stdlib/src/tests/structure_ops/ordering_sets.rs` | T | eY:15, eY:17, N:22, N:23, N:24, N:30, N:31, N:37, N:38, N:44, N:45, N:46, N:52, N:53, N:54, N:86, N:87, I:97, I:98, eY:112, eG:138, eG:144, eY:150, G:251, eY:267 |

#### The `system_expr` layer and propagated producers

The broad helper scan found 1,348 literal calls mentioning 314 distinct
literal names, 209 nonliteral calls, and one declaration. All non-call
`system_expr` references found by a separate whole-token search were
imports/reexports. The 314-name figure includes tests and is **not** a
bootstrap manifest. Nonliteral finite selectors (boolean branches, head
enums, operators and continuation tags) are converted to typed reserved
selectors; open APIs receive existing head handles; test-only arbitrary
names use the fallible constructor. Thus refusal is handled where the
arbitrary name is admitted, rather than reintroduced in a supposedly
infallible `system_expr`.
(`crates/rum-runtime/src/runtime/expr_helpers.rs:9-10`;
`crates/rum-runtime/src/runtime/kernel_data/functional_data.rs:611-629`;
`crates/rum-stdlib/src/polynomials/core/variables.rs:57-105`.)

Verbatim final helper census (declarations included):

```text
LITERAL CALLS 1348 UNIQUE LITERAL NAMES 314 NONLITERAL/DECLARATION 210
allocate_fresh_symbol: 4 references in 3 files (declarations included)
allocate_generated_symbol: 3 references in 1 files (declarations included)
generated_symbol_candidate: 3 references in 1 files (declarations included)
allocate_sealed_symbol: 2 references in 1 files (declarations included)
canonicalize_pattern_names: 7 references in 3 files (declarations included)
collect_pattern_name_mapping: 6 references in 2 files (declarations included)
lhs_patterns_equivalent: 7 references in 3 files (declarations included)
down_value_patterns_equivalent: 5 references in 3 files (declarations included)
runtime_message_name_expr: 9 references in 6 files (declarations included)
held_runtime_message_name_expr: 3 references in 2 files (declarations included)
normalize_symbol_target_expr: 24 references in 11 files (declarations included)
register_builtin: 72 references in 17 files (declarations included)
install_system_own_value: 2 references in 2 files (declarations included)
Recommended default arithmetic: 8 * 1024 * 1024 = 8388608
Long-name falsifier bound: (8 * 1024 * 1024) // (64 * 1024) + 1 = 129
```

The following locations are the full `system_expr` call inventory. They all undergo the typed-head conversion just described; the helper declaration is excluded. These are not additional distinct arbitrary-insertion APIs.

<details>
<summary>All 1,557 system_expr calls</summary>

| Source file | `system_expr` call lines |
|---|---|
| `crates/rum-runtime/src/runtime/arithmetic/canonical.rs` | 71, 72, 181, 199, 202, 221, 224, 285, 287, 469, 616, 664 |
| `crates/rum-runtime/src/runtime/arithmetic/extremum.rs` | 27, 28, 36, 37, 55, 70, 71, 74 |
| `crates/rum-runtime/src/runtime/arithmetic/parts.rs` | 83, 87, 101, 111, 136 |
| `crates/rum-runtime/src/runtime/arithmetic/rounding.rs` | 101 |
| `crates/rum-runtime/src/runtime/attributes.rs` | 143, 169 |
| `crates/rum-runtime/src/runtime/builtins_context_loader.rs` | 78, 88, 169, 189, 196, 206, 230, 243, 271, 278, 293, 329, 334, 345, 350 |
| `crates/rum-runtime/src/runtime/builtins_context_loader/file_names.rs` | 35, 100 |
| `crates/rum-runtime/src/runtime/builtins_input.rs` | 53, 59, 111, 120, 133, 146, 175, 221, 225, 254, 282, 416, 421 |
| `crates/rum-runtime/src/runtime/call_preparation.rs` | 402, 419, 436, 455 |
| `crates/rum-runtime/src/runtime/debug_tools/basic.rs` | 58, 62, 65, 75, 88, 90, 105, 108, 121, 138, 143 |
| `crates/rum-runtime/src/runtime/debug_tools/evaluation_data.rs` | 88, 100, 103, 108, 112, 113, 115, 118, 124, 130, 138, 149, 160, 170 |
| `crates/rum-runtime/src/runtime/debug_tools/message_switch.rs` | 8, 19, 60 |
| `crates/rum-runtime/src/runtime/debug_tools/monitor.rs` | 11 |
| `crates/rum-runtime/src/runtime/debug_tools/properties.rs` | 24, 52, 83 |
| `crates/rum-runtime/src/runtime/debug_tools/reap.rs` | 27, 136, 141, 198, 222, 225, 269, 305, 309, 320, 326, 375 |
| `crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs` | 247, 273 |
| `crates/rum-runtime/src/runtime/debug_tools/shared.rs` | 11, 13, 25, 27 |
| `crates/rum-runtime/src/runtime/debug_tools/test_report/create.rs` | 38, 42, 52 |
| `crates/rum-runtime/src/runtime/debug_tools/test_report/evaluation.rs` | 8, 13, 15, 20, 31, 62, 113, 174, 216, 219, 222, 224, 225, 226, 228, 232, 382, 389, 405, 415, 434, 441 |
| `crates/rum-runtime/src/runtime/debug_tools/test_report/report.rs` | 421, 422, 561, 601, 607, 611, 615, 619, 623, 627, 671 |
| `crates/rum-runtime/src/runtime/debug_tools/timing.rs` | 47, 78, 92 |
| `crates/rum-runtime/src/runtime/debug_tools/trace.rs` | 355, 378, 404, 411, 539, 546, 603, 610, 626, 644, 647, 691, 694, 700 |
| `crates/rum-runtime/src/runtime/debug_tools/trace/builder.rs` | 34, 61, 70 |
| `crates/rum-runtime/src/runtime/definitions/application/mod.rs` | 156 |
| `crates/rum-runtime/src/runtime/definitions/application/replace_all.rs` | 46, 49 |
| `crates/rum-runtime/src/runtime/definitions/definition_catalog/entries.rs` | 100, 102, 121, 132, 143, 154 |
| `crates/rum-runtime/src/runtime/definitions/definition_catalog/rewrite_rules.rs` | 260, 265, 275 |
| `crates/rum-runtime/src/runtime/definitions/definition_catalog/traversal_continuation.rs` | 11, 15, 66, 124 |
| `crates/rum-runtime/src/runtime/definitions/definition_clear.rs` | 36, 71, 102 |
| `crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation/prepared_store.rs` | 319, 346, 374 |
| `crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation/syntax_policy.rs` | 175 |
| `crates/rum-runtime/src/runtime/definitions/definition_tags.rs` | 115, 173, 176, 195, 216, 227 |
| `crates/rum-runtime/src/runtime/definitions/definition_updates.rs` | 273, 285, 297, 413, 417, 421, 425, 438, 480, 662, 663 |
| `crates/rum-runtime/src/runtime/definitions/definitions.rs` | 110, 160, 422, 425, 432, 434, 438, 446, 449 |
| `crates/rum-runtime/src/runtime/definitions/rhs_guards.rs` | 49 |
| `crates/rum-runtime/src/runtime/eval_call_prepare.rs` | 131, 137, 139, 140 |
| `crates/rum-runtime/src/runtime/eval_core/symbol.rs` | 21, 30, 46, 50 |
| `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs` | 290, 305 |
| `crates/rum-runtime/src/runtime/evaluation_cache/tests.rs` | 84, 88, 110, 237, 267, 268, 301, 320, 490, 515 |
| `crates/rum-runtime/src/runtime/evaluation_cache/tests/association_lvalue.rs` | 244 |
| `crates/rum-runtime/src/runtime/evaluation_cache/tests/association_stability_tail.rs` | 8, 12, 16, 125 |
| `crates/rum-runtime/src/runtime/expr_helpers.rs` | 15 |
| `crates/rum-runtime/src/runtime/expr_runtime_helpers.rs` | 75 |
| `crates/rum-runtime/src/runtime/hold_thread.rs` | 90 |
| `crates/rum-runtime/src/runtime/kernel_data/association_data.rs` | 131, 134, 164, 178, 313, 323, 565 |
| `crates/rum-runtime/src/runtime/kernel_data/association_lvalue.rs` | 40 |
| `crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs` | 45, 279, 318, 349 |
| `crates/rum-runtime/src/runtime/kernel_data/control_data/assertions.rs` | 8, 31, 44, 65, 76, 81, 109, 111, 119, 150, 217 |
| `crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs` | 47, 138, 143, 178, 223, 228, 241, 281, 286, 303, 337, 344, 355, 404, 433, 455 |
| `crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver.rs` | 116, 123, 134, 160, 173, 181, 191, 275, 282, 333, 344, 572 |
| `crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver/continuation.rs` | 13 |
| `crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/spec.rs` | 12, 14, 16, 19, 129 |
| `crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/table_fast_path.rs` | 28, 44, 53, 90, 109 |
| `crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration_aggregate.rs` | 244 |
| `crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation.rs` | 115, 220, 245, 249, 253, 257, 261, 263, 264, 268, 272, 275, 282, 286, 317, 330, 343, 355, 356, 365, 377 |
| `crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation/builtins.rs` | 497 |
| `crates/rum-runtime/src/runtime/kernel_data/control_data/exceptions.rs` | 506, 520, 526, 530, 534 |
| `crates/rum-runtime/src/runtime/kernel_data/control_data/failsafe.rs` | 41, 43 |
| `crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs` | 13, 36 |
| `crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs` | 32, 100, 346, 365, 394, 405, 429, 486 |
| `crates/rum-runtime/src/runtime/kernel_data/evaluated_part_selectors.rs` | 98, 197 |
| `crates/rum-runtime/src/runtime/kernel_data/functional_data.rs` | 40, 51, 232, 242, 397, 412, 456, 464, 467, 473, 486, 553, 628, 691, 739, 805, 871, 934 |
| `crates/rum-runtime/src/runtime/kernel_data/rewrite_data.rs` | 41, 45, 49, 50, 55, 60, 135, 139 |
| `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect.rs` | 179, 187, 200, 208 |
| `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect/resume_frames.rs` | 75, 85, 189 |
| `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs` | 163, 164, 196, 197 |
| `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete/iterative.rs` | 376, 425, 471, 497, 507, 541 |
| `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete/resume_frames.rs` | 112, 206, 207 |
| `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete/traversal.rs` | 210, 234, 377, 405, 417, 441, 547, 573, 643, 679, 691, 708 |
| `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs` | 21, 112, 120, 255 |
| `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/list.rs` | 141, 253, 256, 345, 432, 477 |
| `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/list/matching.rs` | 19 |
| `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/list/matching/resume_frames.rs` | 65, 149, 273, 371 |
| `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs` | 601, 817, 876 |
| `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once/resume_frames.rs` | 125 |
| `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once/traversal.rs` | 288 |
| `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs` | 255 |
| `crates/rum-runtime/src/runtime/kernel_data/structure_data/mod.rs` | 96, 97, 98, 99 |
| `crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/plan.rs` | 6, 10, 14, 210, 303 |
| `crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/plan_application.rs` | 227, 229, 239 |
| `crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/replace_at.rs` | 215, 219, 310 |
| `crates/rum-runtime/src/runtime/kernel_data/structure_data/selectors.rs` | 94, 124, 127, 232 |
| `crates/rum-runtime/src/runtime/kernel_data/structure_data/selectors/apply.rs` | 39, 211, 462 |
| `crates/rum-runtime/src/runtime/kernel_data/structure_data/selectors/diagnostics.rs` | 158, 162, 170, 177, 226, 230, 233 |
| `crates/rum-runtime/src/runtime/kernel_data/structure_data/selectors/extract.rs` | 106, 108, 110 |
| `crates/rum-runtime/src/runtime/listable_dispatch.rs` | 58, 59 |
| `crates/rum-runtime/src/runtime/lowering.rs` | 255 |
| `crates/rum-runtime/src/runtime/lowering/binary.rs` | 374 |
| `crates/rum-runtime/src/runtime/lowering/binary/comparison.rs` | 67 |
| `crates/rum-runtime/src/runtime/messages/builtins_messages.rs` | 31, 71, 161 |
| `crates/rum-runtime/src/runtime/messages/builtins_messages_inspect.rs` | 35, 71, 87, 201, 203, 212, 215, 223, 228, 246, 260, 333, 344, 398, 399, 430, 431, 458, 467, 474, 486, 494, 505 |
| `crates/rum-runtime/src/runtime/messages/message_catalog.rs` | 263, 276, 296, 298, 300, 300, 320, 360, 440, 449, 456, 465 |
| `crates/rum-runtime/src/runtime/messages/message_format.rs` | 18, 29 |
| `crates/rum-runtime/src/runtime/messages/parse_and_message_helpers.rs` | 249, 256, 260, 267, 271, 496, 515 |
| `crates/rum-runtime/src/runtime/metadata.rs` | 172, 282 |
| `crates/rum-runtime/src/runtime/mod.rs` | 88 |
| `crates/rum-runtime/src/runtime/namespace_tools.rs` | 26 |
| `crates/rum-runtime/src/runtime/namespace_tools/names.rs` | 87, 104, 145, 264 |
| `crates/rum-runtime/src/runtime/namespace_tools/syntax.rs` | 79, 83, 87, 91, 95, 99, 103, 108, 110, 111, 126, 178, 194, 217, 231, 237, 244, 252, 261, 272, 276, 281, 287, 299, 312, 316, 317, 325, 331, 345, 359, 375 |
| `crates/rum-runtime/src/runtime/options.rs` | 116, 127, 132, 137, 205 |
| `crates/rum-runtime/src/runtime/options/arguments.rs` | 55, 122, 126, 130, 163, 182, 207, 300, 302, 303, 352, 355, 423 |
| `crates/rum-runtime/src/runtime/options/builtins.rs` | 47, 184, 272 |
| `crates/rum-runtime/src/runtime/options/core.rs` | 76, 171 |
| `crates/rum-runtime/src/runtime/options/defaults.rs` | 12, 33, 35, 40, 50, 53, 54, 57, 58, 60, 62, 65, 67, 70, 71, 72, 75, 76, 83, 84, 93, 98, 124, 126, 127, 130 |
| `crates/rum-runtime/src/runtime/options/patterns.rs` | 76, 114 |
| `crates/rum-runtime/src/runtime/package_tools.rs` | 123, 210, 234, 242, 281, 291, 296, 303, 453, 457, 459, 461, 462, 467, 469, 471, 472, 531, 543, 545, 553, 555, 583, 602 |
| `crates/rum-runtime/src/runtime/pattern_matching/continuation.rs` | 4 |
| `crates/rum-runtime/src/runtime/pattern_matching/core/key_value.rs` | 165, 179, 189, 195, 199 |
| `crates/rum-runtime/src/runtime/pattern_matching/core/resume_frames.rs` | 124, 335, 454, 599 |
| `crates/rum-runtime/src/runtime/pattern_matching/outcome.rs` | 16, 23, 25, 26 |
| `crates/rum-runtime/src/runtime/pattern_matching/sequence.rs` | 65, 180 |
| `crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/choices.rs` | 54, 322, 402 |
| `crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/collect.rs` | 18, 26, 359 |
| `crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/optional.rs` | 49, 135 |
| `crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/optional/resume_frames.rs` | 35, 103, 211, 279 |
| `crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/pattern.rs` | 169, 265 |
| `crates/rum-runtime/src/runtime/pattern_matching/sequence/ordered.rs` | 171 |
| `crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/candidates.rs` | 13 |
| `crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/collect.rs` | 18, 24, 351 |
| `crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/match_choices.rs` | 23, 32, 37, 357, 437 |
| `crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/optional.rs` | 18 |
| `crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/optional/resume_frames.rs` | 53 |
| `crates/rum-runtime/src/runtime/pattern_matching/sequence/resume_frames.rs` | 52, 119, 190, 330, 407 |
| `crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs` | 405, 607, 730, 735, 846, 851 |
| `crates/rum-runtime/src/runtime/pattern_syntax.rs` | 86 |
| `crates/rum-runtime/src/runtime/scoping/lexical/block.rs` | 387 |
| `crates/rum-runtime/src/runtime/scoping/lexical/block/continuation.rs` | 4 |
| `crates/rum-runtime/src/runtime/scoping/lexical/with.rs` | 93 |
| `crates/rum-runtime/src/runtime/scoping/packages.rs` | 62, 68, 77, 185, 247, 342, 390, 406, 487, 506 |
| `crates/rum-runtime/src/runtime/scoping/packages/resume_frames.rs` | 16 |
| `crates/rum-runtime/src/runtime/scoping/unique.rs` | 100, 106, 115 |
| `crates/rum-runtime/src/runtime/session/session_api.rs` | 462, 473, 476, 479, 811 |
| `crates/rum-runtime/src/runtime/session/session_eval.rs` | 32, 51, 299, 406, 410, 413, 417 |
| `crates/rum-runtime/src/runtime/session/session_helpers.rs` | 391, 523 |
| `crates/rum-runtime/src/runtime/session/session_requests.rs` | 571 |
| `crates/rum-runtime/src/runtime/substitution/binding_substitution.rs` | 46, 753, 755, 762, 782, 783, 787, 805, 809, 826, 830, 832, 834 |
| `crates/rum-runtime/src/runtime/substitution/lexical_substitution.rs` | 623, 626, 627, 630, 644, 647, 648, 651, 668, 671, 675, 689, 692, 696, 713, 716, 718, 726, 737, 740, 742, 749 |
| `crates/rum-runtime/src/runtime/suspension/collections.rs` | 6, 8, 10, 52, 54, 56, 102, 106 |
| `crates/rum-runtime/src/runtime/suspension/error.rs` | 4 |
| `crates/rum-runtime/src/runtime/suspension/primitive.rs` | 15, 39, 40, 70, 91, 96, 105, 159, 160, 162, 214, 236 |
| `crates/rum-runtime/src/runtime/suspension/rewrite_rules.rs` | 5, 28, 38, 40, 44, 110, 125 |
| `crates/rum-runtime/src/runtime/suspension/traversal.rs` | 5, 10, 45, 48, 78, 83, 84, 126, 130 |
| `crates/rum-runtime/src/runtime/test_builtins.rs` | 494 |
| `crates/rum-runtime/src/runtime/tests/arithmetic.rs` | 442, 454, 462, 474 |
| `crates/rum-runtime/src/runtime/tests/association_lookup_cache.rs` | 56 |
| `crates/rum-runtime/src/runtime/tests/association_part_cursor.rs` | 4, 10, 96 |
| `crates/rum-runtime/src/runtime/tests/association_write_index.rs` | 5, 9 |
| `crates/rum-runtime/src/runtime/tests/association_write_recovery.rs` | 18, 22 |
| `crates/rum-runtime/src/runtime/tests/definitions_rhs_guards.rs` | 50 |
| `crates/rum-runtime/src/runtime/tests/file_names.rs` | 11 |
| `crates/rum-runtime/src/runtime/tests/holding_and_control_part2.rs` | 253, 256, 265, 275, 275 |
| `crates/rum-runtime/src/runtime/tests/lowering_golden.rs` | 225, 227 |
| `crates/rum-runtime/src/runtime/tests/match_normalization_contract.rs` | 129, 130, 786, 787 |
| `crates/rum-runtime/src/runtime/tests/part_assignment.rs` | 74, 346 |
| `crates/rum-runtime/src/runtime/tests/scoping_gensym_gc/suspension_frames.rs` | 258 |
| `crates/rum-runtime/src/runtime/tests/sealed_snapshot.rs` | 262 |
| `crates/rum-runtime/src/runtime/tests/sealed_visibility.rs` | 47 |
| `crates/rum-runtime/src/runtime/tests/session_memory.rs` | 189, 207 |
| `crates/rum-runtime/src/runtime/tests/session_requests/api.rs` | 17 |
| `crates/rum-runtime/src/runtime/tests/valueq_stable_shortcut.rs` | 40, 85, 102, 109, 140 |
| `crates/rum-runtime/src/runtime/tree_walkers/count_like.rs` | 27, 28 |
| `crates/rum-runtime/src/runtime/tree_walkers/level.rs` | 65 |
| `crates/rum-runtime/src/runtime/tree_walkers/position.rs` | 177, 185 |
| `crates/rum-runtime/src/runtime/tree_walkers/position/resume_frames.rs` | 91 |
| `crates/rum-runtime/src/runtime/tree_walkers/post_order.rs` | 214, 282 |
| `crates/rum-stdlib/src/arithmetic.rs` | 168, 180, 182 |
| `crates/rum-stdlib/src/arithmetic/stats.rs` | 24, 53, 97, 125, 142, 167, 179, 204, 234, 236 |
| `crates/rum-stdlib/src/associations/aggregate.rs` | 356, 376, 388, 442, 613 |
| `crates/rum-stdlib/src/associations/basic.rs` | 272 |
| `crates/rum-stdlib/src/associations/basic/key_take_drop.rs` | 126, 163, 250, 291, 307, 312, 318 |
| `crates/rum-stdlib/src/associations/basic/keys.rs` | 82, 92, 135, 151, 156, 162, 233 |
| `crates/rum-stdlib/src/associations/basic/lookup.rs` | 138, 192, 232, 276, 515, 551, 569, 599, 635, 698, 784, 811 |
| `crates/rum-stdlib/src/associations/basic/values.rs` | 82, 92, 138, 154, 159, 165, 236 |
| `crates/rum-stdlib/src/associations/data.rs` | 142, 156, 273, 279, 349, 352, 360, 363 |
| `crates/rum-stdlib/src/associations/grouping/list.rs` | 250, 284, 440, 480 |
| `crates/rum-stdlib/src/associations/join_across.rs` | 144, 350 |
| `crates/rum-stdlib/src/associations/key_reshape/traversal.rs` | 428, 477, 493, 499, 503 |
| `crates/rum-stdlib/src/associations/missing.rs` | 158, 217, 219, 223, 225, 226, 231, 234, 235, 237 |
| `crates/rum-stdlib/src/associations/missing/target.rs` | 161, 413 |
| `crates/rum-stdlib/src/associations/missing/traversal/counting.rs` | 376 |
| `crates/rum-stdlib/src/associations/missing/traversal/deletion.rs` | 68, 276, 395 |
| `crates/rum-stdlib/src/associations/missing/traversal/projection.rs` | 124 |
| `crates/rum-stdlib/src/associations/mutation.rs` | 111, 229, 458, 467, 572, 576, 595, 630, 655, 675, 683, 704 |
| `crates/rum-stdlib/src/associations/resumable/aggregate.rs` | 117, 158 |
| `crates/rum-stdlib/src/associations/resumable/aggregate/merge_nested.rs` | 111, 167 |
| `crates/rum-stdlib/src/associations/resumable/map.rs` | 144, 180 |
| `crates/rum-stdlib/src/associations/transforms.rs` | 70, 202 |
| `crates/rum-stdlib/src/associations/transforms/key_collections.rs` | 190, 213, 256, 462, 499 |
| `crates/rum-stdlib/src/bytes/codec.rs` | 46, 52, 62 |
| `crates/rum-stdlib/src/calculus/differentiation.rs` | 101, 122, 146, 176, 200, 220, 230, 234, 255, 303 |
| `crates/rum-stdlib/src/calculus/differentiation/iterative.rs` | 69, 96, 136, 177, 215, 237, 263 |
| `crates/rum-stdlib/src/combinatorics.rs` | 274, 275 |
| `crates/rum-stdlib/src/csv.rs` | 438, 442, 455, 504, 510, 521, 739 |
| `crates/rum-stdlib/src/datetime.rs` | 160, 175, 229, 233 |
| `crates/rum-stdlib/src/datetime/calendar.rs` | 87 |
| `crates/rum-stdlib/src/functional_ops/apply_map/mod.rs` | 51 |
| `crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs` | 461 |
| `crates/rum-stdlib/src/functional_ops/apply_map/traversal/map_indexed.rs` | 391, 396, 406, 422, 435, 447, 464, 476 |
| `crates/rum-stdlib/src/functional_ops/arrays/filter.rs` | 11, 85 |
| `crates/rum-stdlib/src/functional_ops/arrays/generators.rs` | 91, 92, 177, 484, 508, 557, 558, 559, 560 |
| `crates/rum-stdlib/src/functional_ops/arrays/mod.rs` | 21, 27, 46, 47 |
| `crates/rum-stdlib/src/functional_ops/arrays/spec.rs` | 75, 81, 88, 133, 144, 189, 196, 201, 243 |
| `crates/rum-stdlib/src/functional_ops/arrays/tensor.rs` | 89 |
| `crates/rum-stdlib/src/functional_ops/callable/composition.rs` | 30 |
| `crates/rum-stdlib/src/functional_ops/callable/curried.rs` | 52, 60, 97, 112 |
| `crates/rum-stdlib/src/functional_ops/iteration.rs` | 757, 769, 785, 793, 807 |
| `crates/rum-stdlib/src/functional_ops/mod.rs` | 41, 46, 53, 55, 63, 65, 73, 75, 85, 86, 93, 94, 106, 107, 112, 122, 124, 142 |
| `crates/rum-stdlib/src/functional_ops/sequence/fold/pair.rs` | 72, 145, 188 |
| `crates/rum-stdlib/src/functional_ops/sequence/fold/plain.rs` | 141, 159, 187 |
| `crates/rum-stdlib/src/functional_ops/sequence/fold/sequence_fold.rs` | 207, 218 |
| `crates/rum-stdlib/src/functional_ops/sequence/select.rs` | 72, 86, 112, 125, 381, 385, 424, 455, 505 |
| `crates/rum-stdlib/src/functional_ops/sequence/window.rs` | 139, 155, 174, 314, 352, 390, 427 |
| `crates/rum-stdlib/src/functional_ops/targeted_map.rs` | 274, 469, 470, 482, 513, 635, 677 |
| `crates/rum-stdlib/src/ini.rs` | 404, 409, 419, 424, 438 |
| `crates/rum-stdlib/src/integer_digits.rs` | 244, 245, 255, 269, 288 |
| `crates/rum-stdlib/src/json.rs` | 118 |
| `crates/rum-stdlib/src/json/decode.rs` | 125, 126, 127, 147, 170, 190, 200, 221 |
| `crates/rum-stdlib/src/json/encode.rs` | 316 |
| `crates/rum-stdlib/src/lazy.rs` | 122, 132, 148, 245, 268, 291, 307, 326, 359, 382, 401, 452, 455, 456, 458, 484, 511, 531, 546, 582, 585, 798 |
| `crates/rum-stdlib/src/logic/comparison.rs` | 368 |
| `crates/rum-stdlib/src/logic/shared.rs` | 18 |
| `crates/rum-stdlib/src/logic/truth.rs` | 141, 145, 195, 230, 247, 286, 321, 338, 378, 413, 434, 471, 506, 527, 622, 640, 644, 647, 654, 666 |
| `crates/rum-stdlib/src/number_theory.rs` | 37, 284, 285 |
| `crates/rum-stdlib/src/polynomials/core.rs` | 227, 415 |
| `crates/rum-stdlib/src/polynomials/core/construction.rs` | 98, 103, 115 |
| `crates/rum-stdlib/src/polynomials/core/factors.rs` | 126, 136 |
| `crates/rum-stdlib/src/polynomials/core/factors/tests.rs` | 79 |
| `crates/rum-stdlib/src/polynomials/core/term_order.rs` | 58, 80 |
| `crates/rum-stdlib/src/polynomials/core/variables.rs` | 63, 96 |
| `crates/rum-stdlib/src/polynomials/factor_collections.rs` | 154 |
| `crates/rum-stdlib/src/polynomials/factorization.rs` | 88, 94 |
| `crates/rum-stdlib/src/polynomials/rational_function.rs` | 141, 145 |
| `crates/rum-stdlib/src/predicates.rs` | 176, 393 |
| `crates/rum-stdlib/src/predicates/numeric.rs` | 267 |
| `crates/rum-stdlib/src/predicates/truth.rs` | 157, 158, 161, 199, 200, 203, 237, 238, 241, 287, 289 |
| `crates/rum-stdlib/src/random.rs` | 25, 38, 41, 305, 373, 611, 617, 621 |
| `crates/rum-stdlib/src/random/password_policy.rs` | 63, 276 |
| `crates/rum-stdlib/src/schema.rs` | 164, 300, 326, 518, 563 |
| `crates/rum-stdlib/src/strings.rs` | 62, 70, 139, 141, 395 |
| `crates/rum-stdlib/src/strings/encoding.rs` | 158, 166, 273, 274, 275, 283, 285, 289 |
| `crates/rum-stdlib/src/strings/patterns.rs` | 908 |
| `crates/rum-stdlib/src/strings/scripting/mod.rs` | 27 |
| `crates/rum-stdlib/src/strings/scripting/text_shape.rs` | 439, 442 |
| `crates/rum-stdlib/src/strings/scripting/url.rs` | 226, 233, 239 |
| `crates/rum-stdlib/src/strings/similarity/distance.rs` | 214, 240, 244, 255, 263 |
| `crates/rum-stdlib/src/strings/similarity/mod.rs` | 45, 47, 56, 58, 250, 251, 281, 282, 325, 326, 352, 353, 381, 382 |
| `crates/rum-stdlib/src/strings/similarity/nearest.rs` | 14, 65, 91, 92, 138 |
| `crates/rum-stdlib/src/strings/template.rs` | 56, 58, 96, 109, 254, 273 |
| `crates/rum-stdlib/src/strings/text/values.rs` | 4 |
| `crates/rum-stdlib/src/strings/tokenize.rs` | 349 |
| `crates/rum-stdlib/src/structure_ops/collection_stats.rs` | 408, 897, 900, 941, 962, 968 |
| `crates/rum-stdlib/src/structure_ops/combinatorial_lists.rs` | 31, 40, 89, 237, 284, 294, 306, 318, 330, 342, 367, 379 |
| `crates/rum-stdlib/src/structure_ops/dedup.rs` | 389 |
| `crates/rum-stdlib/src/structure_ops/flatten.rs` | 102, 143, 161 |
| `crates/rum-stdlib/src/structure_ops/join.rs` | 11, 120, 162 |
| `crates/rum-stdlib/src/structure_ops/list_lookup.rs` | 115, 122, 165, 208, 276, 298 |
| `crates/rum-stdlib/src/structure_ops/list_plumbing.rs` | 34, 91, 95, 121, 133, 144, 156, 168, 224, 344 |
| `crates/rum-stdlib/src/structure_ops/mod.rs` | 69, 72, 100, 101, 102, 103 |
| `crates/rum-stdlib/src/structure_ops/ordering_sets.rs` | 221, 958, 984, 987, 998 |
| `crates/rum-stdlib/src/structure_ops/reordering.rs` | 85, 97, 118, 207, 211, 224, 228, 317 |
| `crates/rum-stdlib/src/structure_ops/selectors/apply.rs` | 29, 194, 400 |
| `crates/rum-stdlib/src/structure_ops/sequence/delete.rs` | 66, 82, 246, 268 |
| `crates/rum-stdlib/src/structure_ops/sequence/edit_plan.rs` | 213, 340 |
| `crates/rum-stdlib/src/structure_ops/sequence/insert.rs` | 74, 242 |
| `crates/rum-stdlib/src/structure_ops/sequence/range.rs` | 23 |
| `crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs` | 43, 102, 132, 139, 149, 166, 169, 188, 201, 573, 580, 582, 582, 582, 583, 583, 583, 590, 592, 592, 593, 593 |
| `crates/rum-stdlib/src/structure_ops/sequence/take_list.rs` | 31, 44, 47 |
| `crates/rum-stdlib/src/structure_ops/surface.rs` | 100, 162, 419, 439, 456, 460, 462, 463 |
| `crates/rum-stdlib/src/tests/association_key_reshape_depth.rs` | 31, 37, 41, 45, 52, 231, 248, 338 |
| `crates/rum-stdlib/src/tests/association_key_take_drop_depth.rs` | 31, 36, 42, 59 |
| `crates/rum-stdlib/src/tests/association_keys_depth.rs` | 27, 32, 38, 49, 176, 188, 192, 218 |
| `crates/rum-stdlib/src/tests/association_lookup_depth.rs` | 28, 32, 36, 48, 133, 141, 248 |
| `crates/rum-stdlib/src/tests/association_values_depth.rs` | 27, 32, 38, 49, 176, 188, 192, 218 |
| `crates/rum-stdlib/src/tests/codecs_toml.rs` | 26 |
| `crates/rum-stdlib/src/tests/codecs_xml.rs` | 46, 65, 74, 171, 175, 231, 239 |
| `crates/rum-stdlib/src/tests/codecs_yaml.rs` | 26 |
| `crates/rum-stdlib/src/tests/debug_tools_evaluation.rs` | 19, 29, 39, 49, 63, 73, 87, 97, 111, 121, 143, 153, 199, 212, 228, 241 |
| `crates/rum-stdlib/src/tests/debug_tools_reports.rs` | 186, 196, 209, 222, 251, 264, 292, 302, 318, 331, 345, 368, 378, 394, 404 |
| `crates/rum-stdlib/src/tests/delete_missing_depth.rs` | 238 |
| `crates/rum-stdlib/src/tests/delete_missing_target_certificates.rs` | 33, 37, 41, 57, 69, 101, 167, 192 |
| `crates/rum-stdlib/src/tests/held_association_exact_path_depth.rs` | 31, 35, 40, 61, 82, 97, 222, 242, 259, 306, 336, 350, 370, 390, 413, 437, 460, 474, 508, 554 |
| `crates/rum-stdlib/src/tests/holding_and_control_failure_edges.rs` | 264, 274 |
| `crates/rum-stdlib/src/tests/incidental_debug_tools_resources.rs` | 41, 50, 58, 70, 81, 93, 127, 143, 164, 187, 227, 236 |
| `crates/rum-stdlib/src/tests/incidental_session_requests_input_resume.rs` | 361, 371, 381, 391 |
| `crates/rum-stdlib/src/tests/json.rs` | 30, 89 |
| `crates/rum-stdlib/src/tests/map_indexed_association_depth.rs` | 27, 31, 36, 58, 58, 60 |
| `crates/rum-stdlib/src/tests/merge_native_stack_guard.rs` | 21 |
| `crates/rum-stdlib/src/tests/preflight_memory.rs` | 76, 101, 109, 115, 127, 135, 146, 152, 154, 155, 158, 164, 172, 178, 181, 194, 212, 225, 229 |
| `crates/rum-stdlib/src/tests/scan_and_keypath_set_native_stack_guard.rs` | 102, 116, 233 |
| `crates/rum-stdlib/src/tests/strings/surface.rs` | 189, 191, 194, 195, 205, 206, 206, 211, 212, 212, 217, 218, 218, 226, 229, 230, 230, 232, 241, 243, 244, 244, 250, 252, 262, 264, 266, 268, 280, 282, 283, 283, 288, 736, 739, 742, 743, 743, 745, 749, 751, 752, 752 |
| `crates/rum-stdlib/src/tests/strings/tokenize.rs` | 224 |
| `crates/rum-stdlib/src/toml.rs` | 169 |
| `crates/rum-stdlib/src/toml/decode.rs` | 146, 150 |
| `crates/rum-stdlib/src/toml/encode.rs` | 390 |
| `crates/rum-stdlib/src/toml/model.rs` | 269, 275, 293 |
| `crates/rum-stdlib/src/xml.rs` | 115, 125, 127 |
| `crates/rum-stdlib/src/yaml.rs` | 167, 171, 176, 179, 185 |
| `crates/rum-stdlib/src/yaml/encode.rs` | 298 |

</details>

Higher-level producers below show why the migration must reach a compiler
fixpoint instead of stopping at the direct table. Creation helpers use E,
pattern preparation P, message materialization M, registration H; existing
value traversal remains infallible where the B/X proof applies. The counts
include definitions, recursion and test callers, so they are never added
together as a distinct-site total.

| Helper / entrypoint | All current source references (declarations included) |
|---|---|
| `allocate_fresh_symbol` | `crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation.rs:120`; `crates/rum-runtime/src/runtime/scoping/lexical/module.rs:49`; `crates/rum-runtime/src/runtime/scoping/unique.rs:81,93` |
| `allocate_generated_symbol` | `crates/rum-runtime/src/runtime/scoping/unique.rs:82,87,149` |
| `generated_symbol_candidate` | `crates/rum-runtime/src/runtime/scoping/unique.rs:36,131,157` |
| `allocate_sealed_symbol` | `crates/rum-runtime/src/runtime/scoping/packages.rs:289,420` |
| `canonicalize_pattern_names` | `crates/rum-runtime/src/runtime/definitions/definition_analysis_helpers.rs:394,394,397`; `crates/rum-runtime/src/runtime/model/definitions/down_value_dispatch.rs:557`; `crates/rum-runtime/src/runtime/model/definitions/tagged_value_store.rs:233,251,262` |
| `collect_pattern_name_mapping` | `crates/rum-runtime/src/runtime/definitions/definition_analysis_helpers.rs:400,404,418,421,424`; `crates/rum-runtime/src/runtime/definitions/rhs_guards.rs:75` |
| `lhs_patterns_equivalent` | `crates/rum-runtime/src/runtime/definitions/definition_analysis_helpers.rs:393`; `crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation/prepared_store.rs:449,470,495,665`; `crates/rum-runtime/src/runtime/model/definitions/tagged_value_store.rs:61,84` |
| `down_value_patterns_equivalent` | `crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation/prepared_store.rs:581,592,668`; `crates/rum-runtime/src/runtime/definitions/rhs_guards.rs:52`; `crates/rum-runtime/src/runtime/model/definitions/down_value_dispatch.rs:571` |
| `runtime_message_name_expr` | `crates/rum-runtime/src/runtime/debug_tools/evaluation_data.rs:142`; `crates/rum-runtime/src/runtime/debug_tools/shared.rs:28`; `crates/rum-runtime/src/runtime/debug_tools/test_report/evaluation.rs:140`; `crates/rum-runtime/src/runtime/debug_tools/test_report/report.rs:288,363,585`; `crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation.rs:184`; `crates/rum-runtime/src/runtime/messages/message_format.rs:23,30` |
| `held_runtime_message_name_expr` | `crates/rum-runtime/src/runtime/eval_core/symbol.rs:250,257`; `crates/rum-runtime/src/runtime/messages/message_format.rs:27` |
| `normalize_symbol_target_expr` | `crates/rum-runtime/src/runtime/attributes.rs:33`; `crates/rum-runtime/src/runtime/definitions/definition_analysis_helpers.rs:90,165,180`; `crates/rum-runtime/src/runtime/definitions/definition_updates.rs:248`; `crates/rum-runtime/src/runtime/kernel_data/association_data.rs:752`; `crates/rum-runtime/src/runtime/messages/builtins_messages_inspect.rs:57`; `crates/rum-runtime/src/runtime/messages/message_catalog.rs:47,173,197,208,219`; `crates/rum-runtime/src/runtime/namespace_tools/syntax.rs:10`; `crates/rum-runtime/src/runtime/options/builtins.rs:15,92,113,325,343,394`; `crates/rum-runtime/src/runtime/options/patterns.rs:102`; `crates/rum-stdlib/src/associations/mutation.rs:175,298,518`; `crates/rum-stdlib/src/associations/transforms/key_collections.rs:537` |
| `register_builtin` | `crates/rum-host/tests/installer.rs:33,39,76,85`; `crates/rum-runtime/src/runtime/session/session_api.rs:271`; `crates/rum-runtime/src/runtime/session/session_state.rs:685`; `crates/rum-runtime/src/runtime/test_builtins.rs:306,311,321,331,340,349,453,469`; `crates/rum-runtime/src/runtime/tests/definitions_rhs_guards.rs:5,11`; `crates/rum-runtime/src/runtime/tests/definitions_rhs_guards_attribute_revision.rs:206`; `crates/rum-runtime/src/runtime/tests/definitions_rhs_guards_rewrite_rules.rs:13`; `crates/rum-runtime/src/runtime/tests/memory_growth_contract.rs:257,276,291,310,324,340`; `crates/rum-runtime/src/runtime/tests/messages.rs:363`; `crates/rum-runtime/src/runtime/tests/part_assignment.rs:563`; `crates/rum-runtime/src/runtime/tests/registered_builtin_resolution.rs:15`; `crates/rum-runtime/src/runtime/tests/replace_all_unheld.rs:15`; `crates/rum-runtime/src/runtime/tests/sealed_packages.rs:48,57,66,75`; `crates/rum-runtime/src/runtime/tests/session_requests/api.rs:669,706,816`; `crates/rum-runtime/src/runtime/tests/structured_exceptions.rs:31,36,42,48`; `crates/rum-stdlib/src/lib.rs:44,47,50,53,56,59,63,66,69,72,75,78,81,84,87,90,93,96,99,102,105,108,111,114,117,120,123,126,129,132,135,138`; `crates/rum-stdlib/src/tests/preflight_memory.rs:61` |
| `install_system_own_value` | `crates/rum-runtime/src/runtime/session/session_api.rs:291`; `crates/rum-stdlib/src/character_classes.rs:23` |

The only private text/pair insertion functions found are the ones in
`expr.rs`; their callers are the two core constructor implementations.
An alias/`FromStr` search found no alternate Symbol constructor surface in
this source tree. This is corroboration of the private-field boundary,
not a claim to have exhaustively enumerated external consumers or all
possible future macros. `Symbol` serialization and the public reexports
are separately included in the gates.
(`crates/rum-core/src/expr.rs:35-40,80-138,288-319`;
`crates/rum-core/src/lib.rs:11-15`.)

Verbatim mechanism-probe output follows. `$` lines display subprocess argv
joined for readability; regex alternations were passed as **one argument**,
not executed as shell pipes.

```text
$ rg -n Box::leak|OnceLock<Mutex<SymbolInterner>>|interned symbol count should fit crates/rum-core/src/expr.rs
48:    static SYMBOL_INTERNER: OnceLock<Mutex<SymbolInterner>> = OnceLock::new();
66:        let leaked: &'static str = Box::leak(value.to_owned().into_boxed_str());
88:    let next_id = u32::try_from(pairs.len()).expect("interned symbol count should fit in u32");
90:        Box::leak(Box::new(SymbolText {
[exit 0]

$ rg -n interner crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs
[exit 1]

$ rg -n \bintern_symbol_text\b|\bintern_text\b crates
crates/rum-core/src/expr.rs:57:fn intern_text(strings: &mut HashSet<&'static str>, value: &str) -> &'static str {
crates/rum-core/src/expr.rs:80:fn intern_symbol_text(context: &str, name: &str) -> &'static SymbolText {
crates/rum-core/src/expr.rs:85:    let context = intern_text(strings, context);
crates/rum-core/src/expr.rs:86:    let name = intern_text(strings, name);
crates/rum-core/src/expr.rs:110:        let text = intern_symbol_text(context.as_ref(), name.as_ref());
crates/rum-core/src/expr.rs:119:        let text = intern_symbol_text(context.as_ref(), name.as_ref());
[exit 0]

$ rg -n \bSymbol\s+as\s+|type\s+\w+\s*=\s*(?:\w+::)*Symbol\b|impl\s+FromStr\s+for\s+Symbol crates
[exit 1]

$ rg -n interned_id\( crates
crates/rum-core/src/expr.rs:166:    pub fn interned_id(&self) -> u32 {
crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:70:    let mix = u64::from(symbol.interned_id()).wrapping_mul(0x9e3779b97f4a7c15) ^ identity;
[exit 0]
```


### 3. The falsifier design

**Use real process isolation for public-API quota tests, and an injectable
private admission/storage-planning seam for arithmetic and ID boundaries.**
Do not install a temporary process-global quota in an ordinary unit-test
binary. Restoring its numeric limit cannot undo immortal text or IDs, and
a test mutex cannot exclude unrelated parallel tests. A thread-local quota
would test a different policy. A second local interner that returns real
unbranded `Symbol`s is also unacceptable: reusing dense IDs across those
instances would violate equality before either instance was dropped.
(`crates/rum-core/src/expr.rs:35-55,80-96,250-254`;
`TODO.md:1213-1219`; `docs/testing-strategy.md:18-36`.)

#### Private seam, without another Symbol domain

Factor `AdmissionState` and `InsertionPlan` from the production factory.
Their inputs include the charged limit, existing byte totals, index slot
layouts/capacities, which strings/pair are missing, and the next ID. Unit
tests instantiate these states locally and can inject a tiny ID ceiling,
integer-overflow inputs and deterministic reserve failures. They return a
plan/permit or error, **never a public Symbol or escaped static reference**.
The real factory consumes exactly this plan; there is no alternative
production bypass selected by a test flag. A test storage builder can
observe staged owners' drops and requested layouts without implementing
another public interner. The actual factory/public-API tests below are
required in addition to that seam.
(`crates/rum-core/src/expr.rs:42-45,57-96,109-125`;
`docs/testing-strategy.md:18-23`.)

Use an independently calculated charge in the assertions: literal byte
lengths, `size_of`/`Layout` for the actual slot types, full allocated slot
counts, the fixed baseline and simultaneous old/new storage. Do not make
the expected result a call to the production projection helper. Exercise
exact fit, one byte short, UTF-8 byte lengths, empty strings, equal
context/name, one/two new strings, a new pair with no new strings, and both
indexes crossing their growth threshold together. Failure injection before
publication must drop all staged owners. Byte overflow, capacity overflow,
`u32::MAX` admission and the following refusal need finite tests. An ID-cap
hit must still return the already interned pair before asking for a new
permit. Put that hit-versus-admission decision in the shared private seam
used by the real factory; local tests use an opaque existing-pair marker,
not a second domain of public Symbols. Otherwise moving the real hit check
behind the ID check would escape a planner-only test. These cases target the ordering currently visible at
`crates/rum-core/src/expr.rs:57-96`, not a fabricated billion-entry fixture.

#### Public tests, each in its own process

Add dedicated integration-test executables with **one orchestrating test
per executable**. They call the public startup configuration before any
arbitrary-name construction; stats may be read first to obtain the fixed
baseline. Cargo's separate test executable boundary is the isolation seam.
Do not combine these scenarios as ordinary parallel tests in the runtime's
large unit-test executable. A small configured allowance is added to the
reported reserved baseline; it is never installed in another test's
process. If a scenario needs a different limit or a fresh first-use state,
it gets a separate executable. No quota-reset API, environment-variable
mutation, singleton replacement or globally enabled “ignore quota” feature
is introduced. The proposed tests belong in their owning crate's `tests/`
directory, following `AGENTS.md:136-138`.

| Executable / scope | Positive control and condition that must go red if enforcement disappears |
|---|---|
| `crates/rum-core/tests/interner_quota_bootstrap.rs` | Configure exactly the reserved baseline. Every reserved token and fallible/serde spelling of its pair succeeds with identical ID/text; the first non-reserved pair refuses. A configuration below the baseline returns an error and does not silently enlarge the allowance. Exercise error construction/formatting with no dynamic headroom. |
| `crates/rum-core/tests/interner_quota_bytes.rs` | Configure a small finite allowance, admit a non-reserved seed, then make bounded distinct-name insertions until the next same-shaped insertion refuses with `QuotaExceeded` and numeric data. The seed still compares/formats/serializes correctly and constructs again through all applicable constructors, including a new nonzero identity. Repeated rejected pairs leave retained bytes/counts/ID state unchanged. Include a fresh context and name in the rejected pair. |
| `crates/rum-core/tests/interner_quota_growth.rs` | Use the independently derived layouts to hit a strings-index transition, a pairs-index transition and simultaneous growth. A request whose text would fit but whose complete table/overlap storage would not fit refuses. Reconstruct actual retained storage independently of the projection. A failure must not retain a grown live index or consume an ID. Exact-fit arithmetic is additionally covered by the local seam. |
| `crates/rum-core/tests/interner_quota_serde.rs` | Use serde's existing value deserializer support to avoid a new JSON dependency (`crates/rum-core/Cargo.toml:12-17`). Decode valid Symbol fields successfully, then decode arbitrary long fields at the cap and assert a normal `D::Error`, not a panic. Test `Some(0)` separately before admission. Include a composite derived Expr/shared-container decode that fails after a successful earlier symbol, and check that only earlier admission remains charged. |
| `crates/rum-core/tests/interner_quota_concurrency.rs` | Race duplicate and distinct names using a barrier under one preconfigured budget. Duplicates share IDs and are charged once; only admissible distinct transactions commit. Total charge never exceeds the limit. This test does not assume thread scheduling selects a particular winner. |
| `crates/rum-core/tests/interner_quota_first_use.rs` | In its own fresh process, race explicit configuration against first arbitrary construction: exactly one initialization policy wins, the loser receives the documented result, and subsequent changes cannot raise the limit. Separate below-baseline and stats-before-configuration controls from the already-sealed scenario. |
| `crates/rum-core/tests/interner_quota_default.rs` | Do **not** configure a small test limit. Use the real default and unique names each at least 64 KiB long. Require a successful first non-reserved control and a quota refusal within the calculated finite bound of 129 attempts for the proposed default. If enforcement is removed, terminate the loop and fail explicitly; do not loop to OOM. Pair/index overhead may make refusal earlier and is not an exact expected name count. |
| `crates/rum-runtime/tests/interner_quota_requests.rs` | Pre-intern control inputs, fill the process quota, then cover standalone/session lowering, lower-only versus source evaluation, `Symbol`, `ToExpression`, Unique/Module, package declarations and new contexts. Assert resource failure/`InternerLimit` rather than invalid syntax, `None` fallback or poisoned state. Parse-only and existing-name evaluation still work. Probe pattern equivalence with a previously unused `Pattern` pN name so the P channel is actually exercised. |
| `crates/rum-runtime/tests/interner_quota_messages.rs` | At the cap exercise `General::internerlim`, message-list materialization, report generation and the new error's continuation encode/decode. Include an older arbitrary string-named message whose conversion itself refuses: formatting the interner error must still succeed without recursively interning another error name. Reserved tag construction must also work when that particular continuation has never previously run. |
| `crates/rum-host/tests/interner_quota_lifecycle.rs` | Two sandboxed hosts observe the same process usage. One consumes the allowance; the other refuses a fresh typed Symbol/source name. Existing names and stored definitions work in both. A successful reset and a new default host do not replenish usage. Keep a raw runtime Expr/Symbol alive across reset and verify unchanged text, ID and equality. |
| `crates/rum-host/tests/interner_quota_restore.rs` | Test the **two distinct paths**: source snapshot restore through host APIs and core serde in its owning test. Fill the quota before restoring source that requires a new pair; assert HostFailure even when the runtime response is a terminated success. Preserve messages and the non-poisoned state; verify retained admitted names after partial restore and invalid UTF-8 before reset. |
| `crates/rum-host/tests/interner_quota_install_suspend.rs` | Make a custom installer attempt a fresh name at the cap; build/reset fails through the new Result channel and an existing host is not half-replaced. Suspend evaluation, fill the quota from another host, then resume into fresh-name creation. Assert the specific error and correct cleanup/restoration of the active frames; a following existing-name request works. |

The current entrypoints underlying these proposed tests are
`crates/rum-core/src/expr.rs:109-138,312-319,801-811`;
`crates/rum-runtime/src/runtime/session/session_requests.rs:129-181,265-268`;
`crates/rum-runtime/src/runtime/session/session_api.rs:74-83,271-307`;
`crates/rum-runtime/src/runtime/definitions/definition_analysis_helpers.rs:393-429`;
`crates/rum-runtime/src/runtime/messages/message_format.rs:23-30`;
`crates/rum-host/src/session.rs:147-220`;
`crates/rum-host/src/model/outcome.rs:473-565`.
All names/source fragments belong to rum's own tests and documentation;
generated source files use `.rum` (`AGENTS.md:38-51,139-147`).

The per-scenario executables can share test **source helpers** for input
construction and assertions, but cannot share a long-lived child process.
Run the ordinary parallel workspace suite with them enabled, then explicitly
repeat the quota executables with a different harness thread count to prove
that isolation does not depend on `--test-threads=1`. A timeout, skip,
bootstrap failure or unrelated validation error is not evidence that the
quota fired. Log the error variant, effective configured limit and actual
retained charge. Avoid text-byte tests that accidentally hit the tiny ID
seam, and ID tests that first run out of byte budget.
(`docs/testing-strategy.md:18-55`; `AGENTS.md:86-93`;
`TODO.md:3133-3137`.)

#### Prove the checked-in tests detect loss of enforcement

Draft C uses an isolated source copy under its authorized scratch, with the
candidate source kept unchanged. Make one minimal mutation at a time:
remove the **production** byte-admission comparison; omit string bytes from
the projection; omit table-growth bytes; move the hit check behind the
ID refusal; and remove the ID-admission check. Run the specifically named
checked-in test for that condition. Each must fail because the forbidden
admission/refusal/charge occurred, not because the mutant did not compile,
ran out of time, changed its expected value, or failed an unrelated case.
Restore the scratch copy byte-for-byte between mutations and run its
unmodified control. Record each exact diff and failure output. This is a
finite mutation manifest, not an automatic tuning campaign.
(`TODO.md:2188-2193,3133-3137`;
`docs/dev/codex/TEMPLATE.md:99-105,147-159`.)

Compiler tests use the actual public API: assigning each arbitrary-name
constructor's result to `Symbol`/`Expr` without handling Result fails;
passing a dynamic string to a reserved-token constructor fails; a positive
control using `?` compiles. Private fields prevent manufacturing a Symbol or
admission permit outside the factory. Test serde's actual error channel,
not a wrapper which never invokes `Symbol::deserialize`. The constructor
signature checks do not prove accounting arithmetic; both the runtime
falsifiers and the independent charge checks remain mandatory.
(`crates/rum-core/src/expr.rs:35-40,109-138,312-319,801-811`;
`crates/rum-core/src/lib.rs:11-15`.)

### 4. Risks and open questions — director decisions

These decisions are prerequisites for dispatching implementation, **not**
permission questions left to the executing agent. The report and drafts
are complete design deliverables; the mitigation is not implemented or
accepted on the strength of this report.
(`docs/dev/codex/TEMPLATE.md:19-37`; `TODO.md:1196-1211`.)

1. **D1 — default and supported memory envelope.** Approve the proposed
   8 MiB total default, subject to the measured bootstrap footprint, the
   default-cap falsifier and actual ordinary suite/panel headroom. The
   source provides no measured legitimate process working-set requirement.
   Decide the supported embedding envelope; do not claim any positive
   default is safe under an arbitrarily small OS memory limit. A default
   that aborts a supported workload before refusal fails the same principle
   as the stack guard. If calibration rejects 8 MiB, the director chooses
   and explains the replacement; the agent must not silently tune it upward.
   (`TODO.md:1304-1319`; `SECURITY.md:48-58`;
   `docs/host-embedding-and-hardening.md:54-68,130-132`.)
2. **D2 — exact charged-storage contract and index representation.** Ratify
   complete Rust allocation-layout bytes plus conservative fixed-reserve
   storage, explicitly excluding allocator metadata/rounding and RSS.
   Approve replacing opaque HashMap/HashSet backing stores with private
   safe, explicitly sized indexes. A rough entries-times-size formula is
   not the proposed implementation. If the requirement instead means a
   portable physical allocator/RSS bound, this design does not establish
   it; the director must resolve that requirement before dispatch, rather
   than declaring an underestimate conservative. Draft A must fail if its
   safe staging path has any unaccounted allocation or needs unsafe.
   (`crates/rum-core/src/expr.rs:42-96`; `AGENTS.md:109-122`;
   `docs/host-embedding-and-hardening.md:54-59`.)
3. **D3 — public fallibility and closed-name API changes.** Approve Result
   returns for the seven constructors, arbitrary registration, installer
   callbacks and configured host build/reset, plus any genuinely creating
   inspection paths. Approve changing `system_expr` and fixed catalog
   construction to typed names and open head helpers to existing handles.
   Keep fixed default constructors infallible only through their separate
   closed installer proof. There is no quota-panic compatibility layer.
   Confirm zero identity becomes a constructor/serde error rather than a
   panic. (`crates/rum-core/src/expr.rs:109-138,312-319,801-811`;
   `crates/rum-runtime/src/runtime/expr_helpers.rs:9-10`;
   `crates/rum-host/src/config.rs:19-38`;
   `crates/rum-host/src/session.rs:76-95,127-175`.)
4. **D4 — ownership of the finite reserved vocabulary.** Approve a core
   spelling/identity table covering the fixed construction closure while
   builtin behavior/specs remain owned by runtime/stdlib. The central table
   must include non-System helper pairs and continuation tags, cannot grow
   from caller-provided static strings, and must not confer builtin status.
   A missing name forces an explicit typed-vocabulary change at compile
   time; a consistency test also cross-checks catalog names. This is a
   deliberate core data dependency to review, not an accidental move of
   builtin implementations into the kernel.
   (`ARCHITECTURE.md:184-220`;
   `crates/rum-core/src/heads.rs:112-198`;
   `crates/rum-runtime/src/runtime/resolution.rs:141-157`;
   `crates/rum-runtime/src/runtime/suspension/error.rs:268-294`.)
5. **D5 — externally observable refusal contract.** Approve the distinct
   `InternerLimit` termination, `General::internerlim`, typed host failure
   and explicit reset/restore failure semantics. The current restore
   `Ok(_)` discards termination; handling the new interner termination is
   required here. Whether to change its handling of **other existing**
   termination kinds is a separate director scope decision. Do not silently
   make the entire restore transactional or change other failure contracts.
   (`crates/rum-host/src/session.rs:201-220`;
   `crates/rum-runtime/src/runtime/session/session_eval.rs:37-60`;
   `docs/guide/resource-limits-and-sandbox.md:294-315`.)
6. **D6 — performance acceptance.** No cost was measured here. The private
   index replacement, static pool lookup, Result propagation through
   normalization and resolver changes may regress hot paths. Accept only
   after the prescribed rum-vs-rum gate; a regression is a defect to fix or
   report even though Symbol's representation is preserved. If the private
   index proof or performance fails, return to the director with evidence;
   do not substitute unbounded tables, an approximate undercharge or one of
   the ruled-out Symbol representations.
   (`AGENTS.md:46-49,109-132`;
   `scripts/core-language-microbench.sh:479-535,566-586,981-992`.)
7. **D7 — separate state-lifetime audit.** Commission a separate brief for
   the audit criticism that no document distinguishes per-request and
   per-session state. This work documents only the interner's process
   lifetime and affected APIs, and leaves that broader question unanswered
   as requested. (`TODO.md:935-944,953-960`.)

The plan is falsified by a surviving arbitrary-name infallible path, an
unreserved error/bootstrap insertion, admitted storage missing from the
ledger, a refused pair leaking storage, a cap-dependent failure of an
existing-pair lookup, a Result converted to missing/invalid input, a reset
replenishing the quota, serde panicking, or a terminated restore reported
successful. It also fails its stated constraints if Symbol changes its
representation/lifetime/ID equality, if local test interners manufacture
colliding real Symbols, or if any required storage proof needs unsafe.
These are concrete rejection conditions attached to the current paths and
the tests in section 3, not risk disclaimers substituting for a gate.
(`crates/rum-core/src/expr.rs:35-138,250-319`;
`crates/rum-host/src/session.rs:172-220`; `TODO.md:1188-1219`.)

### 5. The drafted follow-up briefs

These are raw drafts for the director to rewrite and number. They do not
authorize source work in brief-505. Their scopes are deliberately explicit:
A proves storage in scratch; B is the complete cross-crate implementation
with its tests/docs; C is the independent falsification/performance review.
D1-D6 are settled before the applicable implementation dispatch. D7 is a
separate future brief and is not answered or implemented by these drafts.
(`docs/dev/codex/TEMPLATE.md:19-37`; this brief's Hard rules above.)

#### Draft A — prove bounded interner storage and the admission instrument

**Goal**

Produce a safe, scratch-only storage/admission proof for the process-global
quota design, preserving the production Symbol contract. Establish actual
slot/pair/reserved layout costs and show that the planned byte and ID
refusals can fire before publication. This is not a production interner
replacement or a performance claim.
(`crates/rum-core/src/expr.rs:23-138,250-285`; `TODO.md:1188-1211`.)

**Context pointers**

- Read `AGENTS.md`, the 2026-09-21 ruling and brief-505 sections 1-4.
- `crates/rum-core/src/expr.rs:42-105`: current allocation/order defect.
- `crates/rum-core/src/heads.rs:112-194`: starting fixed vocabulary, not its
  complete required closure.
- `crates/rum-runtime/src/runtime/suspension/error.rs:3-5,268-294` and
  `crates/rum-stdlib/src/associations/data.rs:346-365`: tags/non-System helpers.
- `docs/dev/codex/TEMPLATE.md:102-105,151-159`: evidence and timeout policy.

**Scope**

MAY write a standalone offline scratch package, evidence, source manifests
and `.rum` inputs only under `target/brief-505/draft-a/**`, and append to the
dispatched brief. This future draft expressly permits scratch Rust; the
current design brief did not. MUST NOT edit production source, manifests,
Cargo.lock, scripts, fixtures, other documentation or Git state. Use no new
external dependency, network access or unsafe. All temporary/build output
must remain in that scratch subtree.
(`AGENTS.md:109-115,139-147`; `docs/dev/codex/TEMPLATE.md:55-69,125-159`.)

**Deliverable**

- Safe explicitly sized indexes with full slot/control/padding storage,
  a staged insertion transaction and an independently checked byte ledger.
  Enumerate all staging/conversion allocations, old/new table overlap,
  fixed state, static pool data and the accounting unit. Use checked
  arithmetic and no live-index growth before admission.
  (`crates/rum-core/src/expr.rs:42-96`.)
- Private injectable admission state and failure injection with no public
  Symbol creation in an independent domain. Show ownership/drop evidence
  for rejected staged allocations; exercise same-string context/name,
  new-pair/old-strings, long text, all growth transitions, arithmetic and
  ID exhaustion, exact fit and one byte short. Test the real factory
  decision ordering through the same private seam: an existing-pair hit
  is returned before asking for a new-ID permit. Local tests can supply
  an opaque existing-pair marker; they cannot construct another domain
  of public Symbols. The permit is consumed only by a no-fail publication
  step.
  (`crates/rum-core/src/expr.rs:80-96,250-254`.)
- A finite reserved-vocabulary draft listing its source origins, including
  fixed runtime/stdlib names, option defaults, error/continuation tags and
  the Global/Information helpers. Measure its declared layout separately
  from process RSS; distinguish static upper charging from exact dynamic
  allocation layouts. Report whether the proposed 8 MiB default covers it
  and how much dynamic allowance remains. Do not infer a legitimate
  workload working set from that subtraction.
  (`crates/rum-core/src/heads.rs:112-194`;
  `crates/rum-runtime/src/runtime/messages/builtins_messages_inspect.rs:483-498`;
  `crates/rum-runtime/src/runtime/options/defaults.rs:3-15`.)
- Safe compilation checks for 24-byte/Copy/`'static` production Symbol and
  the proposed unchanged handle arrangement; scalar error formatting that
  never interns. An isolated prototype's layout is not proof of final
  production codegen or runtime behavior.
  (`crates/rum-core/src/expr.rs:35-40,250-319`; `TODO.md:1188-1193`.)

**rum / Rust pitfalls (read twice)**

A Rust lifetime-free Copy handle cannot lease a local arena. A prototype
must not export a real core Symbol backed by its own ID sequence. Static
`&str` is not a closed vocabulary. `capacity * entry_size` is not evidence
of opaque hash-table backing bytes. `Box::new` does not promise recoverable
OOM. A test calling the same projection for its expected answer certifies
nothing about omitted storage. Do not use new unsafe allocation code to
obtain measurements.
(`crates/rum-core/src/expr.rs:35-96`; `AGENTS.md:109-115`.)

**Verification (MANDATORY)**

Before dispatch the director supplies `BRIEF505A_TEST_TIMEOUT` from a
recorded same-shape build/test run. Configure `CARGO_NET_OFFLINE=true`,
`TMPDIR` and `CARGO_TARGET_DIR` inside the allowed subtree; use the local
workspace-declared Rust toolchain. Execute the scratch package tests,
including its negative/positive controls and layout reporter, with:

```sh
CARGO_NET_OFFLINE=true scripts/run-bounded.py --timeout "${BRIEF505A_TEST_TIMEOUT:?baseline-derived bound required}" -- cargo test --offline --manifest-path target/brief-505/draft-a/Cargo.toml
```

Its tests must contain the named boundary/drop/accounting cases and print
actual layout rows. Capture exact argv/environment, exits and output.
Do not run a production suite or timing panel for an unchanged production
tree. Failure to bound every relevant allocation is an explicit failed
proof, not permission to replace exact accounting with an estimate.
(`docs/dev/codex/TEMPLATE.md:102-105,151-159`.)

**Hard rules**

- No Git mutation, network, out-of-repo writes, production edits, new unsafe,
  vendor engines/oracles, or changed Symbol invariants.
- Touch only the granted scratch subtree and the dispatched report. Do not
  leave generated files or temporary probes elsewhere; rum files use `.rum`.
- Stop if the same proof step fails twice without new evidence, if the
  ledger undercharges, or if the storage needs a forbidden representation.
- Paste the real output, including failures. No cost claims from source
  intent or a toy prototype. (`AGENTS.md:38-51,109-147`;
  `docs/dev/codex/TEMPLATE.md:99-105,125-159`.)

**Report format**

1. Storage/permit graph, every allocation and independent charge calculation.
2. Actual layout, boundary, drop and refusal evidence verbatim.
3. Reserved-set origins and default-baseline arithmetic; unmeasured costs.
4. Proof verdict and remaining director decisions; no production closure claim.

#### Draft B — integrate the bounded, recoverable process interner

**Goal**

Implement the approved brief-505 design in one complete workspace migration:
process byte admission, closed bootstrap/error symbols, Result-returning
arbitrary constructors and serde, all error channels, global observation,
process-isolated falsifiers and matching documentation. Preserve Symbol's
24-byte Copy lifetime-free type and ID equality. The result mitigates
permanent shared-process growth; it does not reclaim or close the original
multi-tenant finding.
(`TODO.md:1188-1219`; `crates/rum-core/src/expr.rs:35-138,250-319`.)

**Context pointers**

- Read `AGENTS.md`, the director's D1-D6 rulings, Draft A's actual proof and
  brief-505's complete constructor/helper inventories.
- Core roots: `crates/rum-core/src/expr.rs:19-138,288-319,801-811`,
  `crates/rum-core/src/lib.rs:7-15`, `crates/rum-core/src/heads.rs:112-194`.
- Nontrivial channels: runtime lowering/resolution, pattern equivalence,
  message materialization, suspension error encoding and installer/restore
  sites enumerated in sections 1-3 of the design report.
- Normative policy: `docs/testing-strategy.md:3-14,60-94` and
  `docs/host-embedding-and-hardening.md:54-105,130-132`.

**Scope**

MAY modify the following areas **only for this migration and its tests**:

- `crates/rum-core/src/**` and `crates/rum-core/tests/**`: new coherent
  symbol/interner/reserved/admission modules, constructors, exports, serde
  and dedicated integration tests. Preserve value representation and wire
  fields; split symbol storage from expression storage.
- `crates/rum-runtime/src/**` and `crates/rum-runtime/tests/**`: typed fixed
  catalogs, lookup/resolution/lowering, arbitrary producers, fallible
  normalization/store preparation, message/query helpers, errors/requests,
  continuations, registration and tests. Only change the per-session memory
  walk if compilation mechanically requires it; add **no interner charge**.
- `crates/rum-stdlib/src/**`, its tests/examples, and module roots: typed
  catalog proofs and fixed helpers, Result propagation, default installation,
  catalog contract text and regression tests.
- `crates/rum-host/src/**`, `crates/rum-host/tests/**` and examples: process
  API reexports, typed-input/failure mapping, installers, build/reset/restore
  channels and tests. `crates/rum-cli`, `rum-repl`, `rum-terminal` and
  `rum-jupyter` source/tests/examples only for forced public-API adaptation
  and its existing contract tests.
- `SECURITY.md`, `docs/host-embedding-and-hardening.md`,
  `docs/guide/resource-limits-and-sandbox.md`, `docs/guide/embedding.md`,
  affected crate rustdoc/README examples, `docs/reference/builtins.md`
  through its generator, and the dispatched report.
- `target/brief-505/draft-b/**`: saved base/candidate artifacts, compile
  probes, measured usage, logs and generated `.rum` workloads.

MUST NOT modify AGENTS/CLAUDE/TODO/PROGRESS, unrelated algorithms or language
contracts, existing benchmark statements, dependency manifests/lockfile,
other briefs or Git state. No unsafe. New integration-test files use Cargo's
existing automatic discovery; serde tests use already available serde
facilities. Report any genuinely required out-of-scope dependency/file to
the director instead of silently extending scope. Fixed-name data in core
must not move builtin implementations out of stdlib.
(`ARCHITECTURE.md:184-224`; `crates/rum-core/Cargo.toml:12-17`;
`docs/dev/codex/TEMPLATE.md:55-69,125-159`.)

**Deliverable**

- The approved byte/accounting/storage implementation, one process startup
  policy and on-demand global stats. Preflight both strings, pair and all
  index growth as one transaction; hits precede byte/ID checks; all
  arithmetic refuses on overflow. Keep reserved IDs and dynamic IDs in one
  non-reused global domain. (`crates/rum-core/src/expr.rs:42-138,250-285`.)
- All seven constructor Results, serde `D::Error`, scalar error mapping,
  typed reserved-token/registered-name proofs and identity-only copies.
  Compiler errors are followed to a workspace fixpoint with a per-file
  forcing-signature inventory. No quota `expect`, `Err -> None`, silent
  default, dynamically extensible reserve or infallible string wrapper
  survives. (`crates/rum-core/src/expr.rs:109-138,312-319,801-811`.)
- The complete E/P/M/H channels from the design. Prepare fallible mutation
  decisions before publishing definition/package changes. Ensure all
  reserve-dependent continuation constructors work at a cold full cap.
  Keep the default diagnostic path free of new polling/locks/allocations.
  (`crates/rum-runtime/src/runtime/definitions/definition_analysis_helpers.rs:393-429`;
  `crates/rum-runtime/src/runtime/messages/message_format.rs:3-30`;
  `crates/rum-runtime/src/runtime/suspension/error.rs:268-294`;
  `AGENTS.md:130-132`.)
- Host installer/build/reset errors and explicit interner-limited restore
  failure, preserved partial-request outputs, non-poisoned recovery using
  existing names, and no replenishment through reset. Keep configured
  replacement transactional at the session-swap boundary, not at the
  global interner or whole-source restore boundary.
  (`crates/rum-host/src/config.rs:12-38`;
  `crates/rum-host/src/session.rs:147-220`.)
- Section 3's private seam, public dedicated-process tests, constructor
  compile-fail/positive controls and real-default falsifier. Report actual
  reserved baseline and process usage of ordinary suite/panel workloads.
  Apply the approved default verbatim; a legitimate workload hitting it
  is a director decision, not authority to disable or increase it.
  (`docs/testing-strategy.md:18-55`; `TODO.md:1304-1319`.)
- The exact approved limits/reset/tenant-isolation/restore wording and
  public API migration examples in the same change. Preserve the original
  finding as mitigated. Do not answer the separate request/session state
  audit or fix unrelated restore termination kinds without a ruling.
  (`TODO.md:941-960,1196-1211`; `docs/testing-strategy.md:60-72`.)

**rum / Rust pitfalls (read twice)**

Old IDs and text may escape any session. Never reclaim, recycle, brand or
own them per Symbol. Bootstrap must be closed by types, not by calling an
arbitrary constructor and expecting it to succeed. Result propagation must
not collapse suspend/resume flow or make quota refusal look like invalid
user syntax. Snapshot source is execution; core serde is a separate path.
`restore_from_bytes` must inspect structured termination. A process quota
is never a term in the session memory proxy. Stage values/indexes before
mutating published state; take special care with `.retain` and comparison
callbacks. No ad-hoc wall-clock timing, ownership-scale benchmark claims
or testing against another engine.
(`crates/rum-core/src/expr.rs:35-40,80-138`;
`crates/rum-host/src/session.rs:201-220`;
`docs/dev/codex/TEMPLATE.md:76-100`; `AGENTS.md:38-51,117-147`.)

**Verification (MANDATORY)**

Before editing, build/save the unchanged base and record artifact hashes
under the authorized subtree, without Git operations. Use offline Cargo
and in-repo TMPDIR/target/output paths. The director supplies
`BRIEF505B_FMT_TIMEOUT`, `BRIEF505B_TEST_TIMEOUT`,
`BRIEF505B_BUILD_TIMEOUT` and `BRIEF505B_PANEL_TIMEOUT` from recorded
same-shape bounds before dispatch. Run focused tests while iterating,
then these gates and paste actual output:

```sh
CARGO_NET_OFFLINE=true scripts/run-bounded.py --timeout "${BRIEF505B_FMT_TIMEOUT:?required}" -- cargo fmt --check
CARGO_NET_OFFLINE=true scripts/run-bounded.py --timeout "${BRIEF505B_TEST_TIMEOUT:?required}" -- cargo test --offline --locked --workspace --no-fail-fast
CARGO_NET_OFFLINE=true scripts/run-bounded.py --timeout "${BRIEF505B_BUILD_TIMEOUT:?required}" -- cargo build --offline --locked --release -p rum-cli
```

Execute the dedicated quota test binaries in the normal workspace run and
also as an explicit isolated manifest at a different harness thread count;
include release runs of the quota/default/suspension cases so the
`push_message` debug-only probe does not hide release behavior. Capture
layout/Copy/`Send + Sync + 'static` checks and relevant existing depth,
sealed-package, host lifecycle and adapter tests. Report native transport
skips as skips, never as exercised passes.
(`crates/rum-runtime/src/runtime/session/session_helpers.rs:255-274`;
`AGENTS.md:86-93,124-128`; `docs/testing-strategy.md:24-36`.)

Run the current full panel once per saved arm, profiling OFF, on an idle
machine, using explicit in-repo paths and the script's bounded engine run:

```sh
bash scripts/core-language-microbench.sh --rum-cli target/brief-505/draft-b/base/rum-cli --output-dir target/brief-505/draft-b/timing-base --timeout "${BRIEF505B_PANEL_TIMEOUT:?required}"
bash scripts/core-language-microbench.sh --rum-cli target/brief-505/draft-b/new/rum-cli --output-dir target/brief-505/draft-b/timing-new --timeout "${BRIEF505B_PANEL_TIMEOUT:?required}"
```

Budget at most two complete runs per arm if a named uncertainty requires
the second. Compare every hot label and same-tier neighbor in `timings.tsv`,
especially the current literal/symbol/list/equality, Module/Block/Table,
downvalue/matcher/replacement, symbol-sort and undefined-symbol rows.
Separate cold/warm interner and quota accounting probes from timing; run
no CPU-heavy workload concurrently. Verify successful work and expected
refusal separately. A reduced amount of completed work is not a speedup.
(`scripts/core-language-microbench.sh:479-535,566-586,981-992,1021-1037`;
`docs/testing-strategy.md:74-94`; `docs/dev/codex/TEMPLATE.md:147-159`.)

**Hard rules**

- No git commit/branch/switch/merge/push/stash/checkout, config/index changes,
  network, out-of-repo writes, new unsafe or outside-scope edits.
- Preserve the ruled Symbol type, size, Copy and ID equality; no reclamation
  or session attribution. No infallible arbitrary-name compatibility API.
- No vendor engines/oracles, weakened documented tests, hidden quota tuning,
  diagnostic default-path cost or acceptance of an unresolved hot regression.
- All long/child-spawning commands are bounded; generated rum files use
  `.rum`. Do not check in build artifacts/probes.
- Two identical failures without new evidence trigger stop-loss. Paste real
  successful and failed output; unsupported or skipped gates are reported
  explicitly. (`AGENTS.md:38-51,86-93,109-147`;
  `docs/dev/codex/TEMPLATE.md:99-100,125-159`.)

**Report format**

1. Files, compiler-forced signature changes and the final insertion boundary.
2. Exact accounting/bootstrapping/constructor/serde/falsifier evidence.
3. Host/request/reset/restore/suspension behavior and published contracts.
4. Per-label base/new timing rows, actual process usage and every regression.
5. Remaining risks, skips, director decisions and mitigation-only verdict.

#### Draft C — independently falsify and gate the integrated mitigation

**Goal**

Audit the finished Draft B candidate without repairing its production tree.
Demonstrate that the checked-in tests catch missing enforcement, that real
public entrypoints reach the refusal, and that the default/cost claims are
supported. Report bounded mitigation, not reclamation or independent
in-process tenant budgets.
(`TODO.md:1196-1219`; `docs/dev/codex/TEMPLATE.md:26-30,102-105`.)

**Context pointers**

- Read `AGENTS.md`, the director's decisions, brief-505 and Draft A/B evidence.
- Re-probe the candidate versions of core constructors/serde, runtime
  lowering/normalization/messages and host installer/reset/restore sites;
  design-report line numbers describe the base, not assumed candidate lines.
- Use `docs/testing-strategy.md:74-94` and the actual current
  `scripts/core-language-microbench.sh:479-535,566-586,981-992,1021-1037` rows.
- Use the production-source perturbation rule at `TODO.md:2188-2193` and
  the failure-attribution requirement at `TODO.md:3133-3137`.

**Scope**

No tracked production source changes. MAY write source copies, compiler
probes, a finite mutation runner, captures and generated `.rum` workloads
only under `target/brief-505/draft-c/**`, and append the dispatched report.
Source mutations are permitted **only in those copies** and only for the
five named falsifier mutations in section 3. MUST NOT modify the live
candidate, base artifacts, scripts, fixtures, manifests/lockfile, other docs
or Git state. The director supplies accepted base/candidate binaries and
same-shape timeout bounds; do not branch/switch/rebuild other historical
variants to manufacture an A/B.
(`docs/dev/codex/TEMPLATE.md:26-30,125-159`.)

**Deliverable**

- Fresh source/API inventory confirming every arbitrary-name constructor,
  wrapper and serde path reaches the one fallible factory, and every
  infallible path has a typed no-insertion proof. Include actual external
  consumer compile failures and compiling controls, not just source regex
  counts. Check static maps, option helpers, error continuations and
  profiling identity copies at a cold full cap.
  (`crates/rum-core/src/expr.rs:98-138,312-319,801-811`;
  `crates/rum-runtime/src/runtime/expr_helpers.rs:9-10`.)
- Reproduced process-isolated byte/default/ID-seam/growth/serde/concurrency
  and host lifecycle tests, with actual numeric charges and failures.
  Demonstrate shared exhaustion across two Pure-only hosts, unchanged
  escaped Symbols after reset, no quota replenishment and no false restore
  success. Distinguish charged allocation layouts from RSS observations.
  (`crates/rum-host/src/session.rs:172-220`;
  `docs/host-embedding-and-hardening.md:54-59,95-105`.)
- Five source-mutation results, one mutant at a time: byte comparison
  removed; text charge omitted; table-growth charge omitted; hit lookup
  moved behind exhausted-ID refusal; ID comparison removed. Each named
  checked-in case must go red for its intended cause. Also record the
  unmodified copied-tree positive control and byte-for-byte restoration.
  Never mutate test expectations to obtain a red result.
  (`TODO.md:2188-2193,3133-3137`.)
- Independently reproduced workspace and per-label timing gates, all
  relevant failures/skips, actual reserved/default headroom and separate
  cold/hit/near-cap behavior. No cost attribution to unchanged Symbol
  layout without measuring the whole candidate.
  (`AGENTS.md:46-49,117-132`; `docs/testing-strategy.md:74-94`.)

**rum / Rust pitfalls (read twice)**

Do not let a test-only allocator/interner policy replace production
admission. Global state isolation is a process boundary, not an assertion
that tests happen to run first. A mutation failing to compile is not the
intended runtime falsifier. Tests must stop at a finite attempt count when
limits are removed. Do not report a discarded termination or an early
failed workload as successful restore/evaluation/performance. Keep
allocation probes and timing separate and preserve Symbol's ruled type.
(`crates/rum-core/src/expr.rs:35-40,80-96`;
`crates/rum-host/src/session.rs:208-217`;
`docs/testing-strategy.md:84-94`; `TODO.md:3133-3137`.)

**Verification (MANDATORY)**

The director supplies `BRIEF505C_TEST_TIMEOUT`, `BRIEF505C_PROBE_TIMEOUT`,
`BRIEF505C_FMT_TIMEOUT` and `BRIEF505C_PANEL_TIMEOUT` before dispatch from
recorded same-shape runs. Run the unmodified candidate offline with bounded
`cargo fmt --check` and
`cargo test --offline --locked --workspace --no-fail-fast`, capturing
verbatim output. Run the named quota binaries and external-consumer
positive/negative cases with `scripts/run-bounded.py`. All Cargo/TMPDIR
output is inside authorized scratch. Retain the compiler reasons, quota
variants, numeric limits/charges and exit statuses.
(`docs/dev/codex/TEMPLATE.md:102-123,151-159`.)

Execute the five-mutant manifest once, serially. Stop an unexplained case
and report it instead of extending the campaign. Each mutant has its own
copied source/build/output directory and bounded named test command;
record its source diff/hash. After the unmodified control, perform one full
microbench run per supplied base/candidate arm with explicit `--rum-cli`,
`--output-dir` and `--timeout`. At most a second run per arm is allowed for
a named unresolved timing concern. Use only `timings.tsv`'s in-kernel
monotonic measurements for timing decisions, with profiling disabled and
no CPU-heavy concurrency. No bare background processes or unbounded waits.
(`AGENTS.md:86-93,124-132`; `docs/testing-strategy.md:74-94`;
`docs/dev/codex/TEMPLATE.md:147-159`.)

**Hard rules**

- No live production repair, Git mutations, network, outside-repo writes,
  new unsafe, vendor comparisons/oracles, weakened tests or changed scope.
- Do not accept a missing guard, undercharged allocation, unreserved error
  path, quota panic, quota replenishment, changed Symbol invariant or an
  unresolved hot-label regression.
- No unchecked test-domain Symbols, disabled production quota feature,
  hidden repeated measurements, or false pass from a skip/timeout/unrelated
  error. Keep generated source suffixed `.rum`.
- Stop-loss applies after two failures without new evidence; preserve and
  quote actual results. Return failure to the director rather than
  substituting a different design. (`AGENTS.md:38-51,109-147`;
  `docs/dev/codex/TEMPLATE.md:99-105,125-159`.)

**Report format**

1. Actual API/accounting review and any surviving bypass or counterexample.
2. Unmodified controls and all five intended mutation failures verbatim.
3. Default/process-shared exhaustion, serde/reset/restore/continuation evidence.
4. Workspace results, per-label A/B, usage observations, failures and skips.
5. Accept/reject against the bounded mitigation contract; no closure claim.

#### Execution evidence and scope audit for this design report

The source census and evidence reducers ran as bounded inline Python via
`scripts/run-bounded.py --timeout 60 -- python3 -`; they wrote data only
under `target/brief-505/`. Source reads used `nl`, `sed`, `cat` and `rg`.
The first worktree status was empty. The early non-boundary exploratory
search matched `SyntaxExpr` substrings; the counted inventory uses the
correct word-boundary regex and does not include those matches. Exploratory
queries for `crates/rum-stdlib/src/expr_support.rs`,
`crates/rum-runtime/src/runtime/model/errors.rs` and
`crates/rum-host/src/model/failure.rs` returned “No such file or directory”
(exit 2); current paths were discovered before making the cited claims.
Those failed path guesses support no finding. No network, compiler,
executable rum workload, test suite or benchmark was run.

Key verbatim probe output is pasted in section 2; the full direct/helper
locations are part of the report. Scratch data also contains the original
brief, source hashes and inventories. The original brief prefix is preserved;
only this appended report is a tracked-file change. Final read-only
verification output is appended below. These checks establish scope and
report consistency, not the correctness or cost of the unimplemented design.
(`docs/dev/codex/TEMPLATE.md:102-105`; this brief's Hard rules above.)

The first attempt at the final inline verification wrapper had a missing
closing parenthesis. Python rejected it before executing any checks or
writes (exit 1). The actual error was:

```text
  File "<stdin>", line 59
    emit('Report details tags paired: '+str(report.count('<details>')==report.count('</details>'))
        ^
SyntaxError: '(' was never closed
```

After correcting that wrapper, the final bounded verification re-scanned
the unchanged source, compared all saved direct coordinates, checked the
complete pasted tables, validated source-anchor ranges and template
sections, and checked file hashes and Git scope. It also makes the zero-use
contextual constructor explicit. Verbatim output follows; the code-fence
check describes the report before these paired evidence blocks were
appended. No production tests were needed or run.

```text
$ scripts/run-bounded.py --timeout 60 -- python3 -  [inline source/inventory/scope verification]
Original 181-line brief prefix preserved: True
Changed manifest files excluding brief: []
Fresh Rust files scanned: 1007
Fresh direct inventory matches saved coordinates: True
Symbol::new: 70
Symbol::with_identity: 12
Symbol::from_name: 49
Symbol::system: 49
Expr::symbol: 146
Expr::contextual_symbol: 0
Expr::system_symbol: 11
Fresh direct constructor occurrences: 337
Fresh system_expr calls plus declaration: 1558
Fresh literal system_expr calls: 1348
Fresh unique literal system_expr names: 314
Classified direct inventory including Self wrappers: 339
Class B: 52
Class E: 13
Class F: 5
Class H: 2
Class M: 2
Class P: 1
Class S: 2
Class T: 248
Class X: 14
Complete pasted direct-table.md: True
Complete pasted system-expr-table.md: True
Complete pasted propagation-table.md: True
Report source anchors checked: 374
Invalid report source anchors: []
Draft A template sections complete: True
Draft B template sections complete: True
Draft C template sections complete: True
Report code fences paired: True
Report details tags paired: True
$ git --no-optional-locks diff --check
[exit 0]
$ git --no-optional-locks diff --name-only
docs/dev/codex/brief-505-interner-quota-design.md
[exit 0]
$ git --no-optional-locks status --porcelain=v1 --untracked-files=all
 M docs/dev/codex/brief-505-interner-quota-design.md
[exit 0]
```
