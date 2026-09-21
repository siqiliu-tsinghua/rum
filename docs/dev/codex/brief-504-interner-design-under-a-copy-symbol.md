# brief-504 — design: close the interner finding without touching `Symbol`

`Scope: NONE — no source change.` This is a design-only brief. You survey,
reason, and return a plan plus drafted follow-up briefs. You write no code.

## Goal

The 2026-09-02 security audit's HIGH finding is still open: the symbol
interner is process-global, leaks by construction, and is invisible to the
per-session memory ceiling. The remedy that was ruled — session-scoped
ownership, option A1 — has now been **refuted by measurement**, and its
ruled fallback A2 does not close the finding. Design what does, under a
constraint that did not exist when A1 was chosen.

Return a plan, its open questions, and the follow-up briefs to execute it.

## The finding, unchanged

- `symbol_interner()` (`crates/rum-core/src/expr.rs:47-55`) is a process-wide
  `OnceLock<Mutex<SymbolInterner>>`; `intern_text` (`:57-70`) `Box::leak`s
  every new name and `intern_symbol_text` (`:80-96`) leaks a `SymbolText` per
  `(context, name)` pair. Neither `strings` nor `pairs` has a removal path.
- `current_memory_proxy_bytes`
  (`crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:50`) walks
  session-owned structures and never mentions the interner, so
  `eval_memory_limit` cannot see the growth.
- `HostSession::reset()` (`crates/rum-host/src/session.rs:161-174`) rebuilds
  the session and does not touch it.
- Director-reproduced: under a `{Pure}`-only policy, eight requests each
  interning 20 000 fresh names took RSS from 5.6 MB to 57.5 MB monotonically,
  and `reset()` returned only to 47.0 MB. For a long-lived multi-tenant host
  this is an exhaustion vector no documented kernel mechanism can close.

## The constraint the spike established — treat these as facts, not as things to re-derive

Briefs 499-503 built A1's representation and measured it. The reports are in
this directory; the numbers you need:

- **Any owning `Symbol` costs at least +10.93% instructions on ordinary
  evaluation.** That is `V-arc` — `Arc<SymbolText>` with owned strings but
  stage1's dense-`u32` equality retained — and it is the *cheapest owning
  variant that can exist*, because it keeps the fast-accept equality that an
  owning representation normally has to give up.
- The full A1 candidate is **+14.33%**: +10.93% from the owning
  representation and +3.42% from the equality demotion, adding with a
  residual under 0.1 percentage points across twelve workloads.
- `V-arc` is **not a valid A1**. Its dense-id equality is sound only because
  the interner remains process-global and assigns one id per distinct text.
  Session-scoped ownership destroys that guarantee, so a real A1 pays both
  halves.
- This project has rejected representation changes at 2-7%, 5-14% and 30%,
  and accepted +0.714% and +0.606% only to remove a process kill.

**Therefore: `Symbol` stays a 24-byte `Copy` handle with a dense id and an
id-based fast-accept equality. The ownership and the accounting must live
somewhere else.** A proposal that changes `Symbol`'s size, its `Copy`-ness,
or its equality fast path is refuted before it is written; say so and move
on rather than re-litigating it.

Two things the spike did **not** refute, and you may use: the migration to a
non-`Copy` `Symbol` is nearly free in itself (+0.06% to +0.84%), and the
depth contract survives an `Arc` (release slope 3.36% *smaller*, margin
44.84% over 4 097). Neither rescues A1, but do not cite them as obstacles.

## What I want back

### 1. The option space, scored

At least these, plus any you find. For each: what the per-session ceiling can
see, what `reset()` reclaims, what a multi-tenant host actually gets, the
hot-path cost measured against the +10.93% floor above, **enforceability —
compiler-enforced or convention-enforced**, and what it breaks.

- **A2 as ruled** — account the global table and refuse, replacing the `u32`
  `expect` at `:88` with a recoverable refusal. Score it honestly, including
  brief-414's own objection that charging the whole table makes one tenant
  pay for another while charging only insertion deltas leaves permanent
  uncharged bytes after `reset()`.
- **Reclaimable global interner** — `Weak` entries, or epoch/generation
  invalidation, with `Symbol` unchanged. **The hazard you must analyse
  head-on: a `Symbol` that has escaped into a host's `Expr` outlives the
  session, and a reclaimed dense id that is later reissued makes that
  `Symbol` silently denote different text.** A design that cannot exclude
  that is not a candidate; say so plainly rather than hedging.
- **Session-scoped dense ids with a mandatory ingress import** — `Symbol`
  keeps its layout and `Copy`, each session owns its table, and every symbol
  crossing into a session is imported. The question that decides it is
  whether that boundary can be made a compile error rather than a rule people
  remember; `lower_syntax_expr`
  (`crates/rum-runtime/src/runtime/lowering.rs:7-18`) is public and takes a
  resolver closure rather than a `Session`, which is where to start looking.
- **Bound and refuse without reclaiming** — cap the interner, fail closed
  with a message naming the limit, and leave the bytes unreclaimed. This does
  not close the finding but converts unbounded growth into a documented
  bounded limit; score what it does and does not buy.
- **Kernel does nothing; document and mitigate in the host.** Score it
  honestly rather than as a strawman — if it is genuinely the best available,
  say so and say what the documentation must then state.

### 2. The escaped-symbol lifetime question, answered once

Every reclaiming option collides with it, so answer it as its own section
with evidence: where can a `Symbol` outlive the session that interned it?
Start from request responses
(`crates/rum-runtime/src/runtime/model/requests.rs:89-134`), suspension
frames (`crates/rum-runtime/src/runtime/suspension/core.rs:74-87`), snapshot
and restore, and the jupyter worker thread
(`crates/rum-jupyter/src/adapter/worker.rs:312-341`). Say which of these can
hold a `Symbol` past a `reset()` today.

### 3. A recommendation, with the reasoning that produced it

One option, argued. If your recommendation is that no option closes the
finding at acceptable cost, recommend that — a refutation is a result, and
this line has already accepted one. The standard here is correctness,
semantic loss, whether the fix is structurally unavoidable, enforceability
and measurability. **Cost of implementation is not an argument**: this
project does not score an option down for its size, and "398 call sites" is a
description of the work, not an objection to it. A fix the compiler enforces
beats one that relies on people remembering, even an order of magnitude more
expensive.

### 4. Drafted follow-up briefs

In `TEMPLATE.md` shape, usable as drafts — not step descriptions. The
director rewrites them before dispatch; they are raw material.

Every claim in all four sections carries `path:line` evidence.

## Method requirements

- Read `AGENTS.md` first, and the three reports in this directory
  (`brief-501`, `brief-502`, `brief-503`) before proposing anything.
- **Re-probe before you rely on a stale statement.** brief-414's Gap A
  analysis is a year of this codebase old in places, and its microbench line
  citations were already stale once. Where you rely on one of its claims, say
  you verified it and how.
- Where you assert a cost, say whether it is measured, inferred from the
  spike's numbers, or a guess. Do not present the third as the second.
- Name what would falsify your recommendation.

## Hard rules

- **No source change.** No `.rs` file, no script, no fixture. The deliverable
  is the plan.
- Do NOT `git commit`, branch, switch, merge, push, stash, `git checkout`, or
  touch git config or the index.
- You MAY append your report to this file and write scratch under
  `target/brief-504/`. Nothing else.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only; no oracle harnesses, no comparisons against other vendors'
  engines.
- No network. Touch nothing outside this repo.
- Paste verbatim evidence for anything you measured or ran. Never fake a
  number, and never quote one you could not take.

## Report format

1. The option table, scored on the axes named above.
2. The escaped-symbol lifetime answer.
3. The recommendation, its reasoning, and what would falsify it.
4. The drafted follow-up briefs.
5. Open questions and anything you could not settle from the source.

---

## Execution report — brief-504

### 1. Option table

This is a source-based design, not a measured implementation. I read
`AGENTS.md` and the reports for briefs 501, 502 and 503. I accept the ruled
24-byte, `Copy`, dense-ID fast-accept constraint. The owning representation
is rejected; neither migration effort nor release stack depth is an argument
against a different ownership boundary. The reference cost is the recorded
ordinary payload **+10.9310% over stage1** for V-arc; the complete candidate's
**+14.3294% over base** is a different comparison. These are supplied
measurements, not new measurements by this brief.
(`docs/dev/codex/brief-504-interner-design-under-a-copy-symbol.md:39-63`;
`docs/dev/codex/brief-501-a1-representation-and-cost.md:304-344`;
`docs/dev/codex/brief-502-stack-slope-and-a1-cost.md:248-256`;
`docs/dev/codex/brief-503-split-the-fourteen-percent.md:232-259`.)

Scores below mean **reject** (violates correctness or the representation
constraint), **mitigation** (useful but does not close the kernel finding),
or **candidate** (a coherent design still requiring compiler and performance
evidence). No unimplemented option has a measured percentage. “No per-symbol
refcount” describes a proposed mechanism, not a prediction of zero total
cost. Closure here requires both accountable arbitrary-name storage and
reclamation without invalidating escaped values, rather than only a finite
process quota. These criteria come from this brief's finding and required
axes. (`docs/dev/codex/brief-504-interner-design-under-a-copy-symbol.md:8-32,54-98`.)

| Option and score | What the session ceiling sees | What reset reclaims; what multiple tenants get | Hot-path cost relative to the owning floor | Enforceability | What it breaks / leaves open |
|---|---|---|---|---|---|
| **A2 as ruled: account the global table and refuse — mitigation** | A whole-table term can include leaked strings, pair records and table capacity, with an insertion precheck and recoverable ID exhaustion. Whole-table charging makes every session pay for other sessions. Delta charging loses the charge at reset while the allocation survives. A persistent process ledger fixes the lost charge only by becoming a process quota. | No interned text is reclaimed. One tenant can permanently consume another's headroom; even a fresh session inherits the depleted process. | **Unmeasured.** Copy/equality remain unchanged, avoiding the mechanism responsible for +10.93%; ledger checks, locks and miss prechecks still cost work. A global total must not require a mutex on each ordinary evaluation step. | A private, fallible insertion API can make refusal structurally mandatory. Attribution cannot be made correct by a type signature while ownership remains global. Leaving infallible constructors beside a checked wrapper makes coverage convention-enforced. | Does not supply reset reclamation or independent budgets. Changing the `u32` `expect` alone also leaves allocations before refusal and bypasses through constructors/serde. Verified against `crates/rum-core/src/expr.rs:80-138,312-319`, `crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:50-338`, `crates/rum-host/src/session.rs:172-175`; objection rechecked from `docs/dev/codex/brief-414-memory-accounting-gaps.md:320`. |
| **Global Weak entries, epochs, generation invalidation or tracing GC, retaining today's unrestricted Symbol — reject** | A live-entry ledger could see global live bytes, but attributing shared entries to one session is unresolved. Weak index keys and tombstones must also be bounded; weak values alone do not free owned keys. | Cannot safely reclaim text reachable from untracked host copies. Retaining it for those copies preserves the leak. Reissuing IDs additionally makes different texts equal. An idle session or an empty continuation queue is not a proof that external values are dead. | No valid unchanged-Symbol implementation to measure. Adding Arc ownership to every Symbol is the already-refuted floor. Generation validation in equality changes the ruled fast path. | **Not enforceable with the current public, lifetime-free Copy type.** Copying a Symbol does not acquire a lease. A list of registered roots is insufficient when safe callers can retain unregistered copies. | Reject plainly: potential dangling references and ID aliasing. Never-reused IDs avoid aliasing but do not protect text lifetime and eventually require refusal in the finite ID space. `crates/rum-core/src/expr.rs:35-44,88-95,140-146,250-285,536-542`; `crates/rum-runtime/src/runtime/model/requests.rs:89-134`; `crates/rum-runtime/src/runtime/suspension/trace.rs:12-31`. |
| **Put an arena lease only on exported Expr roots — reject as a fix on today's API** | Could charge a leased arena while retained, provided the old session's ledger also remains retained. Dropping the session ledger first recreates uncharged retained storage. | One escaped result can pin the entire arena. Worse, a caller can extract a bare Copy Symbol and drop the leased root; the supposed last lease is then false. | **Unmeasured.** A root Arc avoids per-Symbol atomics, but extra root fields, clone/drop work and retention require measurement; it is not a numerical improvement established by V-arc. | A wrapper with public bare-Symbol extraction is convention-enforced. Making extraction lifetime-bound and domain-branded changes this into the scoped option below. | Wrapping only request results misses direct constructors, inspection, traces, installed values and frame APIs. `crates/rum-core/src/expr.rs:109-138,536-542,573-585`; `crates/rum-runtime/src/runtime/model/definitions.rs:28-49`; `crates/rum-runtime/src/runtime/session/session_api.rs:291-307`. |
| **Session-local dense IDs plus an ordinary import function — reject without a type boundary** | A session table and insertion checks can account the right bytes. That alone does not make incoming IDs belong to that table. | Reset could free the table only after excluding escaped references. A wrapper on one request method excludes neither escape nor other entrypoints. | **Unmeasured.** Boundary traversal is proportional to the input graph and text imported. No basis to claim it costs less than +10.93% on a real workload. | **Convention-enforced as stated.** Same unbranded Expr type can enter `evaluate`, `eval_expr_request`, system-value installation, or a builtin/frame result. Two ordinary borrow lifetimes also need not identify distinct sessions. | Different sessions can assign one ID to different names and compare them equal even before either resets. Public standalone lowering remains an ingress. `crates/rum-runtime/src/runtime/lowering.rs:33-40,93-103`; `crates/rum-runtime/src/runtime/resolution.rs:292-305`; `crates/rum-runtime/src/runtime/session/session_requests.rs:174-214`; `crates/rum-runtime/src/runtime/session/session_api.rs:291-307`; `crates/rum-runtime/src/runtime/metadata.rs:332`. |
| **Generatively branded session epochs, with owned detached export and mandatory import — recommended candidate** | The epoch owns arbitrary text, pair storage and indexes; its ledger participates in the session proxy and every missing-entry allocation requires a permit. Imports, lowering and non-evaluation requests use the same factory. | Ending an epoch frees all its arbitrary-name storage. Escaped detached values own their text and retain no arena. Another session has a separate budget. One session can fill its own table until reset, but cannot leave an immortal global residue. | **Unmeasured.** A zero-size type brand preserves the intended 24-byte Copy handle and ID equality without per-Symbol ownership. Arena insertion, owned exports, imports, canonical-pool changes and host scheduling still require actual A/B evidence. The +10.93% floor does not establish their cost. | **Compiler-enforced domain/lifetime boundary if the proof below succeeds.** Private constructors plus a fresh invariant brand prevent bypasses. Budget arithmetic still needs tests; Rust does not prove byte projections. | Breaks the lifetime-free low-level Symbol/Expr/Session API, ambient constructors and unseeded Symbol deserialization. Requires scoped continuations and a new owner arrangement for persistent host sessions. This preserves the three ruled representation properties, not every old Rust signature. `crates/rum-core/src/expr.rs:35-40,250-285,312-319`; `crates/rum-runtime/src/runtime/model/session.rs:29-78`; `crates/rum-runtime/src/runtime/suspension/core.rs:74-98`; `crates/rum-host/src/session.rs:103-108`; design obligations from `docs/dev/codex/brief-504-interner-design-under-a-copy-symbol.md:54-58,85-91`. |
| **Bound and refuse without reclaiming — mitigation** | A distinct process interner byte/count quota sees total retained interner storage. A count-only quota is insufficient because names have variable length. It need not pretend to be the per-session ceiling. | No reclamation. Total interner growth becomes bounded, but a tenant can exhaust the shared quota permanently; reset does not restore capacity. Existing names must remain usable at the cap. | **Unmeasured.** Checks can be confined to interning, with an unchanged evaluator representation. This avoids the owning mechanism, not necessarily all regressions. | Compiler-enforced refusal coverage is possible if the only arbitrary-name insertion API returns Result and all old constructors/serde paths are converted. The quota size and byte model are policy/test obligations. | New process-wide availability limit, named explicitly in the refusal. Must preflight strings and hash-table growth before leaking anything, reserve fixed error/bootstrap symbols and replace the ID panic. `crates/rum-core/src/expr.rs:57-96,98-138,312-319`; `crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:174-189`; `docs/dev/codex/brief-504-interner-design-under-a-copy-symbol.md:92-95`. |
| **No kernel fix; isolated host workers with hard limits and recycling — mitigation of the kernel finding, potentially adequate host isolation** | The kernel ceiling still misses the table. An external worker/process limit bounds the process instead. | Process exit reclaims its interner; Session reset and cancellation do not. A worker dedicated to one tenant avoids sharing this failure domain; a pool must also bound total concurrent workers and retire contaminated workers. | No kernel code change means no newly introduced kernel mechanism, **not a new measurement**. IPC, serialization, process startup and lost warm state have **unknown** cost. | OS/process lifetime is a strong boundary when actually enforced. Documentation, caller-chosen quotas and correct recycling are host responsibilities; no compiler guarantee from the kernel. | Loses in-process isolation and may lose unsnapshotted state on forced termination. Honest and viable when the host already uses process isolation; not closure for a general embeddable in-process runtime. `docs/host-embedding-and-hardening.md:54-59,95-105,130-132`; `crates/rum-host/src/session.rs:172-208`; `crates/rum-core/src/expr.rs:47-96`; `SECURITY.md:68-88`. |

An owning Symbol, an inline extra domain field that grows it, or a
generation/text check added to its equality accept path is not an additional
candidate. The explicit representation ruling excludes each before any
implementation discussion. Ownership of a **separate detached boundary
value** is a different proposal, whose cost must still be measured at that
boundary. (`docs/dev/codex/brief-504-interner-design-under-a-copy-symbol.md:39-58`;
`crates/rum-core/src/expr.rs:35-40,250-285`.)

For the host-only option, documentation must state the actual omission, not
merely say that the memory proxy differs from RSS: arbitrary interned text is
shared across sessions and retained for process lifetime; Pure-only policy,
request memory limits, cancellation and reset do not reclaim it. Recycling
must mean terminating the owning process. An RSS-triggered cancel can stop
more work but cannot undo existing leaked allocations. These statements are
source deductions from the interner and reset, and are more specific than
the present external-limit advice.
(`crates/rum-core/src/expr.rs:47-96`;
`crates/rum-host/src/session.rs:110-124,172-175`;
`docs/guide/resource-limits-and-sandbox.md:292-308`;
`SECURITY.md:75-88`.)

### 2. Escaped-symbol lifetime answer

**A request finishing or its session resetting does not establish that every
Symbol it produced is dead.** Today's Symbol has no session borrow or owner;
it can be copied independently of an Expr. Its text is safe after reset
precisely because it was leaked. The following distinguishes actual raw-value
escape routes from interfaces that already copy into text.
(`crates/rum-core/src/expr.rs:19-40,66-68,88-95,536-542`;
`crates/rum-host/src/session.rs:172-175`.)

| Surface inspected | Can it retain a raw Symbol after the originating session is replaced/dropped today? | Evidence and consequence |
|---|---|---|
| Runtime evaluation and lowering responses | **Yes.** `EvalOutcome.value`, `SessionLowerResponse.expr` and `SessionEvalResponse.value` are owned, lifetime-free Expr fields. A caller may retain the Expr or a copied Symbol independently. | `crates/rum-runtime/src/runtime/model/requests.rs:89-95,122-134`; responses are returned by value at `crates/rum-runtime/src/runtime/session/session_requests.rs:137-147,248-262`. Runtime Session has no HostSession-style reset method in these paths; replacing/dropping it is the corresponding lifetime event. |
| Runtime trace, failure and inspection results | **Yes.** Trace events contain Expr; request failures can contain those traces; definition/message inspection returns Expr fields. Runtime message text and input prompts themselves contain only strings. | `crates/rum-runtime/src/runtime/model/messages.rs:26-63`; `crates/rum-runtime/src/runtime/model/requests.rs:149-154,213-219`; `crates/rum-runtime/src/runtime/model/definitions.rs:28-49`. Exporting only the main successful value would miss these roots. |
| Ordinary `rum-host` outcomes, including input suspension | **No raw Symbol in these value/trace/message shapes.** The returned data can survive reset, but symbol names are String and unsupported values are rendered. | `crates/rum-host/src/model/outcome.rs:15-39,66-106,164-228,230-238,261-299`. This is useful existing separation, not evidence that the underlying runtime API is safe to reclaim. |
| Installed continuation frames | **Not through the ordinary input-suspension response.** The pending frame queue belongs to Session and is dropped when that old Session is dropped. But the public frame/finalizer traits impose `'static`, and downstream code may retain its own frame/Expr/Arc copies; that is not prohibited by the types. | Queue ownership: `crates/rum-runtime/src/runtime/suspension/core.rs:14-27`; trait/result shapes: `:74-125`; public reexports: `crates/rum-runtime/src/runtime/kernel_api.rs:120-133`; host suspension data: `crates/rum-host/src/model/outcome.rs:66-79`. A `'static` bound is not proof of an actual leak or of frame survival; it is a constraint a borrowed design must replace. No in-tree exported pending-owner handle was found. |
| Snapshot bytes and restore | **The bytes: no. Previously returned Expr values: yes.** Snapshot is UTF-8 source, not a live object graph or saved dense-ID table. Restore resets and evaluates that text; it cannot invalidate a separately owned Rust result. | `crates/rum-host/src/session.rs:177-221`; `crates/rum-runtime/src/runtime/session/session_api.rs:345-380,396-429`; raw result fields above. Core Symbol serde also stores context/name/optional identity and re-interns on decode, at `crates/rum-core/src/expr.rs:288-319`. |
| Jupyter worker transfer | **It transfers the owner, not an independent borrowed Symbol.** The worker owns HostSession, then returns it together with a host-shaped outcome. The session cannot simultaneously be reset elsewhere through this move. Once joined, retained outcome data survives reset as strings/typed data. | `crates/rum-jupyter/src/adapter/worker.rs:29-38,76-105,312-350`; host result shapes above. `thread::spawn` still requires an owned Send + `'static` worker payload, so moving a borrowed Session into the existing worker is not a solution. The Send contract is exercised at `crates/rum-host/tests/lifecycle_contract.rs:374-390`. |
| Host installer captures and direct Rust construction | **Yes.** An installer stored in cloned HostConfig can capture an Expr/Copy Symbol; it is retained and reapplied across reset. Core constructors and serde can also make symbols outside any Session. | `crates/rum-host/src/config.rs:12-38`; `crates/rum-host/src/session.rs:68-80,147-175`; `crates/rum-core/src/expr.rs:109-138,312-319,801-810`. An ingress rule limited to parsing cannot cover these. |
| Process-static symbol caches | **Yes.** The canonical and builtin symbol maps are static; enabled profiling maps can retain arbitrary Symbol keys beyond a session. The fixed catalog case is bounded, the arbitrary-name case needs ownership/accounting. | `crates/rum-core/src/expr.rs:98-105`; `crates/rum-runtime/src/runtime/builtin_catalog.rs:80-96`; `crates/rum-runtime/src/runtime/evaluation_cache/profile/mod.rs:218-264`. |

There are **two independent correctness failures** in premature reclamation.
First, freeing SymbolText or its strings invalidates the reference used by
`name()`, `context()`, formatting and serialization, even if IDs are never
reused. Second, if old Symbol A and new Symbol B have the same recycled ID
and identity, equality accepts them without looking at text, and Ord takes
the same shortcut. An old handle can then address a new definition or collide
with its key. Leaving A's text allocated avoids the dangling reference but
does not avoid the ID alias. A generation stored only in the table cannot
change already copied handles; an extra check on equality violates the
constraint. (`crates/rum-core/src/expr.rs:140-146,187-190,250-299`;
`crates/rum-runtime/src/runtime/model/session.rs:10-11,45-49,68-70`;
`docs/dev/codex/brief-504-interner-design-under-a-copy-symbol.md:54-58,79-84`.)

Consequently, **reclaiming today's unrestricted lifetime-free Symbol is not
a candidate**. Reclamation needs either owning handles (ruled out), an owner
whose borrow is enforced on every handle, or an address-space boundary.
Reset-time enumeration of session roots, a Weak map, an epoch counter, Pin,
or a root lease that still exposes a naked Copy Symbol does not establish
that borrow. This is a source/type argument, not a runtime probe.
(`crates/rum-core/src/expr.rs:35-40,536-542`;
`crates/rum-runtime/src/runtime/model/requests.rs:89-134`;
`crates/rum-runtime/src/runtime/suspension/trace.rs:12-31`.)

### 3. Recommendation, plan and falsification

**Pursue the branded session-epoch candidate, starting with a compiler proof.
Do not mark the finding closed or the cost acceptable yet.** This is the
session-local dense-ID/import option with an enforced lifetime and domain
boundary, rather than the rejected owning A1. It places the burden on an API
change and on boundary conversion, where the owning-symbol spike has not
measured the cost. The other surveyed in-process options either preserve
unreclaimed shared state or cannot exclude escaped-handle invalidation.
Implementation size is not part of this choice.
(`docs/dev/codex/brief-504-interner-design-under-a-copy-symbol.md:54-58,85-95,113-121`;
`crates/rum-core/src/expr.rs:35-40,250-285`;
`crates/rum-runtime/src/runtime/model/requests.rs:89-134`.)

This recommendation deliberately changes low-level Rust signatures. If
“Symbol unchanged” is intended to additionally freeze its lifetime-free
public type, beyond the three explicit representation constraints, this
candidate is excluded too. There is then **no surveyed in-process reclaiming
fix under those combined constraints**; the director must retain the finding
and choose a documented mitigation. This distinction needs a ruling before
dispatch, not an implicit type erasure inside the implementation.
(`docs/dev/codex/brief-504-interner-design-under-a-copy-symbol.md:54-58`;
`crates/rum-core/src/lib.rs:1-14`;
`crates/rum-core/src/expr.rs:35-40,109-138`.)

#### Proposed ownership and API contract

1. **One fresh, invariant type brand per session epoch.** Conceptually the
   hot type becomes `Symbol<'epoch>`: dense u32, the existing optional
   generated identity, a borrowed text pointer, and a zero-size invariant
   brand. It remains Copy and must measure 24 bytes on the supported target.
   Equality/hash/ordering retain their current ID/identity operations and
   lexical fallback. Expr, SharedExpr, SharedArgs, Session, symbol maps,
   cached expressions and frame payloads carry the same brand; no additional
   runtime domain comparison goes into equality. Proposed change anchored
   at `crates/rum-core/src/expr.rs:35-40,250-285,536-542,573-580` and
   `crates/rum-runtime/src/runtime/model/session.rs:10-78`.

2. **Generativity, not just a lifetime parameter.** A private scope runner
   invokes a callback universally quantified over a fresh epoch. Only that
   runner can create its session/interner witness; callers cannot construct
   two independent interners with one freely chosen brand. The brand is
   invariant so two sessions cannot be shortened/coerced to a common borrow
   lifetime for equality or insertion. No constructor accepts a caller-made
   ID, no public constructor manufactures a branded symbol without its
   factory, and no conversion erases the brand. A plain covariant
   `Symbol<'a>` plus `&'a Session` would not prove distinct domains. This is
   a proposed type proof, not a compiled result; the dangerous current
   ambient constructors are `crates/rum-core/src/expr.rs:109-138,801-810`
   and the unchecked equality is `:250-285`.

3. **Own the arena outside the borrowed session state.** The scope runner
   owns text storage; the session's maps and expressions borrow it for the
   epoch. Drop session state first, then storage. Do not build a
   self-referential lifetime-free HostSession or extend a borrow to
   `'static`. A fully safe prototype can use a bounded-depth tree of
   append-only pages whose cells are `OnceLock<Box<...>>`/`OnceLock` values;
   insertion returns a borrow through the external owner's shared reference.
   A mutable index in session state may retain those borrows because the
   storage owner is outside that state. Unlike a mutable Vec followed by a
   retained reference into it, published cells never move. Unlike a linked
   chain of one allocation per name, page-tree destruction has bounded
   structural depth. Start with owned boxed context/name strings in each
   pair; do not smuggle in another leaking string-deduplication table. This
   is a concrete **uncompiled safe-storage proposal**, not a claimed cost
   result. It replaces `crates/rum-core/src/expr.rs:23-27,42-96` and avoids
   the owner/borrow conflict in `crates/rum-host/src/session.rs:103-108`;
   the safe-Rust requirement is `AGENTS.md:109-115`.

4. **Separate session values from detached values.** A new owned, lossless
   detached expression type contains text and optional identity metadata,
   never a session text pointer or a lease to its whole arena. Runtime
   evaluation APIs accept only same-brand Expr; APIs accepting detached
   values must import through the session factory before calling them.
   Egress copies the reachable graph/text into detached ownership, preserving
   sharing where practical and using bounded traversal/destruction. A
   returned main value, trace, failure payload, definition or message
   template must all use this boundary. Core `Deserialize` creates detached
   values; constructing a scoped value from serialized data requires an
   explicit session seed/import. No `Deref` or public field may expose an
   unbranded internal Symbol. Current escape and serde anchors:
   `crates/rum-runtime/src/runtime/model/requests.rs:89-154,213-219`;
   `crates/rum-runtime/src/runtime/model/definitions.rs:28-49`;
   `crates/rum-core/src/expr.rs:312-319`.

5. **Close all ingress, including standalone lowering.** The free lowering
   API can produce a detached expression, or require a scoped factory; it
   cannot keep producing the same unbranded hot Expr accepted by every
   session. Session lowering's resolver becomes fallible and returns only
   its brand. Missing-symbol refusal must not be represented as `None` and
   accidentally trigger another allocation/fallback. Direct evaluation,
   accepted-expression requests, installed system values, builtin function
   returns and continuation results carry that same brand. The compiler must
   reject omission of import at any of these surfaces. Current anchors:
   `crates/rum-runtime/src/runtime/lowering.rs:33-40,93-103`;
   `crates/rum-runtime/src/runtime/resolution.rs:65-69,101-138,292-305`;
   `crates/rum-runtime/src/runtime/session/session_requests.rs:174-214`;
   `crates/rum-runtime/src/runtime/session/session_api.rs:291-307`;
   `crates/rum-runtime/src/runtime/metadata.rs:332`;
   `crates/rum-runtime/src/runtime/suspension/core.rs:76-98`.

6. **Make persistence explicit at the host boundary.** Offer a synchronous
   scoped kernel API for an embedder already owning the session's lexical
   lifetime. For the movable, imperative HostSession facade, the concrete
   safe design to prove is an owning worker that runs the scope locally and
   exchanges only owned requests, host data, detached values and control
   handles. Reset ends that entire epoch and starts another before
   acknowledging completion; suspension stays inside the current epoch.
   This changes the host's execution arrangement, not the kernel into an
   OS-thread service. The existing Jupyter worker can move the owning handle,
   or later use its request protocol, but cannot move a borrowed Session
   out of its scope. Preserve cancellation generations, poison recovery,
   installers and configured stack guarantees. Extra worker resources,
   request latency, callback/reentrancy behavior and worker startup failure
   are real acceptance questions, not implementation-effort objections.
   `crates/rum-host/src/session.rs:98-118,147-175`;
   `crates/rum-host/src/config.rs:12-38`;
   `crates/rum-jupyter/src/adapter/worker.rs:312-350`;
   `docs/host-embedding-and-hardening.md:61-105`.

7. **No raw symbol crosses an epoch boundary.** In-scope frames become
   epoch-bound trait objects instead of `'static` objects; root-tracing and
   abandonment traits/macros follow the same brand. Outside callbacks and
   installer captures retain detached values and import them explicitly.
   Static canonical storage contains only a closed, fixed set of text/ID
   descriptors; the factory brands those for the current session. Arbitrary
   `System` names still use the charged local table. Static builtin maps
   use canonical descriptors or scoped maps, never lifetime-erased symbols.
   Enabled arbitrary-symbol diagnostic maps move into epoch ownership or
   use an explicitly bounded export; they must not become a new global
   owned-string leak. Disabled diagnostics retain their cold-gate discipline.
   `crates/rum-runtime/src/runtime/suspension/core.rs:74-98`;
   `crates/rum-runtime/src/runtime/suspension/trace.rs:12-43`;
   `crates/rum-runtime/src/runtime/kernel_api.rs:120-135`;
   `crates/rum-core/src/expr.rs:98-138`;
   `crates/rum-runtime/src/runtime/builtin_catalog.rs:80-96`;
   `crates/rum-runtime/src/runtime/evaluation_cache/profile/mod.rs:218-264`;
   `AGENTS.md:130-132`.

8. **Distinguish dense text IDs from generated identities.** Dense IDs are
   local to an epoch and are never issued twice for different text within
   it. An existing-name lookup succeeds even at capacity. A new-name
   overflow returns a recoverable refusal before allocating; there is no
   wrapping or tombstone-based reuse inside the epoch. Across epochs raw
   comparison is a compile error, so numeric reuse cannot alias old handles.
   `interned_id()` remains dense but its domain must be documented; detached
   values do not advertise that ID as portable identity. This replaces the
   prior fingerprint ruling explicitly, rather than silently changing a
   public promise. Current contract and consumer:
   `crates/rum-core/src/expr.rs:80-95,164-168,250-285`;
   `crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:65-72`;
   older ruling: `TODO.md:1174-1181`.

9. **Preserve sealed semantics; refuse ambiguous imported identities.**
   Ordinary detached symbols import by exact `(context, name)`, not by
   replaying another session's dense ID. Proposed default for generated
   identities: a same-epoch export may carry an opaque provenance token
   retaining only a small identity object, not the arena; reimport into that
   epoch preserves the identity. Foreign/reset-epoch generated symbols
   refuse ordinary import instead of accidentally binding to a numerically
   reused identity. Any trusted raw-identity constructor is an explicit
   scoped host API. Text serde without provenance must not silently grant
   current-epoch identity authority. Snapshot restore keeps its existing
   re-registration/re-link mechanism and orphan refusal, which is the
   supported path across epochs. This is an intentional low-level import
   restriction requiring a director ruling, not a claim about current
   behavior. It follows the actual session-scoped identity rule at
   `crates/rum-runtime/src/runtime/scoping/packages.rs:8-14,332-342,420-429`,
   the exclusion from `symbol_generations` at
   `crates/rum-runtime/src/runtime/definitions/definition_clear.rs:212-229`,
   and restore/orphan handling at
   `crates/rum-runtime/src/runtime/session/session_api.rs:432-449` and
   `crates/rum-runtime/src/runtime/tests/sealed_snapshot.rs:75-103,107-125`.

10. **Accounting is part of the factory, not an optional caller precheck.**
    Charge arbitrary text, pair records, page/index capacity and temporary
    growth reservations to the epoch. Add the retained ledger to the session
    proxy. A private insertion operation consumes a checked permit covering
    all projected growth before allocation/publication; checked arithmetic
    fails closed. There is no unchecked arbitrary-name constructor available
    to callers. Charge retained allocations even if the returned expression
    is discarded or the request fails. Only physically dropped transient
    allocations release their charge. An import failure may leave already
    admitted names in the table, but never uncharged names. Preflight page
    and index growth as well as strings; size estimates must account for
    capacity, not just logical entries. Current missing term and preflight
    hook: `crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:50-338`;
    `crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:174-189`;
    allocation ordering to replace: `crates/rum-core/src/expr.rs:80-95`.

11. **Cover pre-evaluation and export as well as evaluation.** Factory policy
    uses the configured session ceiling even when no evaluation request has
    installed `current_memory_limit`, and min-combines any active local
    constraint. Thus lower-only requests, lookup misses, restore, installers
    and direct scoped construction cannot allocate through an unarmed gap.
    Source requests already begin the request budget before lowering;
    lower-only requests do not. Reserve export/import temporary bytes while
    the request owns them, and release that temporary charge at actual
    transfer/drop. Host-retained detached outputs then belong to the host,
    like today's returned Strings, and cannot pin a previous session arena.
    Retain the existing conservative per-occurrence symbol-text proxy charge
    initially, in addition to the new retained-table term; document the
    overcount rather than quietly changing ByteCount semantics. Fixed
    canonical text may remain a documented bounded process baseline, but
    session index/handle storage is charged. Refusal formatting must use
    already available fixed symbols and return in band without trying to
    intern a new diagnostic name. `crates/rum-runtime/src/runtime/session/session_helpers.rs:30-39`;
    `crates/rum-runtime/src/runtime/session/session_requests.rs:137-169`;
    `crates/rum-runtime/src/runtime/resolution.rs:101-138`;
    `crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:363-365`;
    `crates/rum-host/src/model/outcome.rs:164-228`;
    `docs/guide/resource-limits-and-sandbox.md:294-315`.

The consequence is intentionally **epoch reclamation, not per-symbol GC**:
unused names remain charged until reset/drop; a session that continually
mints names eventually refuses under its own limit. Ending the epoch makes
all raw handles inaccessible and frees the table, while detached values
remain valid. A host can still accumulate its own exported values or disable
its optional memory limit; this is not a hidden process-global retention
mechanism. No claim is made that an allocator immediately returns every freed
page to RSS. The testable closure criterion is released ownership and a
bounded ledger, alongside repeated reset behavior.
(`crates/rum-runtime/src/runtime/model/config.rs:19-32`;
`crates/rum-host/src/model/outcome.rs:164-228`;
`docs/host-embedding-and-hardening.md:54-59`;
`docs/dev/codex/brief-504-interner-design-under-a-copy-symbol.md:19-32`.)

#### Re-probes and implementation ripple

I verified the following brief-414 claims against the **current** source with
line-numbered reads and targeted searches. These are fresh source probes,
not reruns of its RSS experiment or constructor-count compilation. No old
260/398-site total is promoted to a current count.
(`docs/dev/codex/brief-414-memory-accounting-gaps.md:254-280,301-352`;
current evidence capture: `target/brief-504/source-evidence.txt:1`.)

| Earlier claim / planning area | Current verification and implication |
|---|---|
| Global leaks, Copy layout, equality and reset | Read the complete symbol implementation: `crates/rum-core/src/expr.rs:19-319`. Copy and ID equality are present; layout size is accepted from the brief, not newly measured. Reset is now `crates/rum-host/src/session.rs:172-175`. The old mechanism still applies. |
| Memory walk omits the interner but charges referenced text | Read `current_memory_proxy_bytes` through its return at `crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:50-338`; a targeted `interner` search has no match. Per-expression symbol text is now at `:363-365`, not brief-414's `:283-288`. Absence of a retained-table term follows from reading the whole walk, not just the search. |
| Public lowering takes a resolver closure | **Stale.** Public free `lower_syntax_expr(&SyntaxExpr)` is at `crates/rum-runtime/src/runtime/lowering.rs:33-40`; the closure helper is `pub(super)` at `:93-103`. Public Session lowering is `crates/rum-runtime/src/runtime/resolution.rs:67-69`. The standalone ingress problem survives, but the old signature/line claim does not. Reexport verified at `crates/rum-runtime/src/runtime/mod.rs:73-76` and `crates/rum-runtime/src/lib.rs:7-9`. |
| Dense-ID production consumer | Fresh `rg -n 'interned_id\(' crates` found the declaration at `crates/rum-core/src/expr.rs:166` and the Bloom use at `crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:70`. This is a source-search result, not a claim about external downstream consumers. |
| Canonical and diagnostic static roots | Re-read `crates/rum-core/src/expr.rs:98-138`, `crates/rum-runtime/src/runtime/builtin_catalog.rs:80-96`, and `crates/rum-runtime/src/runtime/evaluation_cache/profile/mod.rs:218-264`. Fixed maps and arbitrary diagnostic Symbol keys are still present. |
| Sealed identity reuse and persistence | Re-read `crates/rum-runtime/src/runtime/scoping/packages.rs:420-429`, `crates/rum-runtime/src/runtime/definitions/definition_clear.rs:212-229`, and `crates/rum-runtime/src/runtime/tests/sealed_snapshot.rs:75-125`. Numeric identity reuse is a within-independent-sessions ruling; it does not authorize cross-domain dense-ID equality for escaped host values. |
| Timing targets | Re-probed actual statements in `scripts/core-language-microbench.sh:479-497,508-535,566-586,981-992,1021-1037`; old brief-414 line numbers are not used. Cold symbol construction in `:884-891` is fixture setup, not an existing standalone cold-intern timing label. Do not claim the sort timing isolates intern insertion. |

The cross-crate inventory is by responsibility, not a guessed call count:

| Boundary to migrate | Current source anchors; invariant that must survive |
|---|---|
| Core type family, constructors, serde and reexports | `crates/rum-core/src/expr.rs:35-40,288-319,536-542,573-585,801-817`; `crates/rum-core/src/lib.rs:7-15`. Preserve Copy/layout/ID operations for scoped symbols; detached values own their storage. |
| Lowering, resolution and dynamic Symbol construction | `crates/rum-runtime/src/runtime/lowering.rs:33-40,93-103`; `crates/rum-runtime/src/runtime/resolution.rs:101-138`; `crates/rum-stdlib/src/structure_ops/surface.rs:314-347`. All arbitrary names flow through the fallible factory, including failed existence probes. |
| Session stores, evaluator and requests | `crates/rum-runtime/src/runtime/model/session.rs:29-78`; `crates/rum-runtime/src/runtime/resolution.rs:292-305`; `crates/rum-runtime/src/runtime/session/session_requests.rs:137-262`. No unbranded Expr acceptance remains. |
| Builtin installation and public kernel bridge | `crates/rum-runtime/src/runtime/metadata.rs:332,377-385`; `crates/rum-runtime/src/runtime/session/session_api.rs:271-307`; `crates/rum-runtime/src/runtime/kernel_api.rs:79-107,120-135`; `crates/rum-host/src/config.rs:12-38`. Function pointers/installers quantify over the epoch; mutable data cannot capture another epoch. |
| Suspension, root tracing and derives | `crates/rum-runtime/src/runtime/suspension/core.rs:74-125`; `crates/rum-runtime/src/runtime/suspension/trace.rs:12-43`; `crates/rum-derive/src/lib.rs:81-118`. Preserve resume/abandon behavior and mandatory traversal while carrying lifetimes. |
| Snapshot, identity and diagnostics | `crates/rum-runtime/src/runtime/session/session_api.rs:352-449`; `crates/rum-runtime/src/runtime/scoping/packages.rs:332-429`; `crates/rum-runtime/src/runtime/evaluation_cache/profile/mod.rs:218-264`. Restore re-links identities; diagnostics cannot retain a scoped handle globally. |
| Memory admission and accounting | `crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:50-338,363-365`; `crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:139-189`; `crates/rum-runtime/src/runtime/session/session_helpers.rs:30-39`. Insertion checks are unstrided, retained bytes persist across requests, and local constraints min-combine. |
| Host/adapters and public docs | `crates/rum-host/src/session.rs:98-118,147-221`; `crates/rum-host/src/model/outcome.rs:164-228`; `crates/rum-jupyter/src/adapter/worker.rs:312-350`; `docs/host-embedding-and-hardening.md:12-26,84-105`. Preserve public lifecycle/data behavior and disclose changed low-level APIs and owner placement. |

#### Gates and falsifiers

The recommendation is **falsified as specified** if a safe caller can make a
raw symbol survive its epoch, mix two brands, invoke evaluation without
import, deserialize a scoped handle without the factory, or retain a scoped
handle in a static diagnostic/installer. The first follow-up must try these
as compile-fail cases with compiling controls, including overlapping/nested
scopes and covariance/coercion attempts. If it needs unsafe, a Symbol size
increase, loss of Copy, or a new equality check, stop. These tests target
the actual public escape and insertion surfaces, not a toy API missing
frames or direct evaluation.
(`crates/rum-core/src/expr.rs:35-40,109-138,250-285,312-319`;
`crates/rum-runtime/src/runtime/session/session_requests.rs:174-214`;
`crates/rum-runtime/src/runtime/suspension/core.rs:74-98`;
`crates/rum-host/src/config.rs:19-32`.)

The resource claim is falsified by any path admitting uncharged arbitrary
text, by ledger loss on failed requests/imports, by retained arena ownership
after reset with only detached exports held, or by one session consuming
another's symbol quota. Exercise many short requests, one long name, many
short names, new contexts, arbitrary System names, lower-only input, serde,
restore, nested limits, suspended/resumed failures and repeated resets. Test
ID exhaustion with an injected small capacity, so a unit test proves
refusal and existing-name lookup without allocating the real u32 space.
(`crates/rum-core/src/expr.rs:57-96,132-138,312-319`;
`crates/rum-runtime/src/runtime/resolution.rs:101-138`;
`crates/rum-runtime/src/runtime/session/session_requests.rs:137-169`;
`crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:119-135`.)

The performance claim has **no established result yet**. It must survive
the repository's base/new microbench gate, including the literal, symbol,
list, equality, Module/Block, Table, downvalue, matcher, replacement and
symbol-sort neighbors. A regression is a defect to fix or report; “less
than +10.93%” is not a new acceptance threshold. Supplement that gate with
the same instruction instrument and ordinary workload from the spike,
separate cold/warm intern measurements, and separately reported host
import/export/worker costs. The in-kernel panel alone cannot establish
top-level request latency or worker resource cost. Preserve the release
depth contract as a test; do not re-open the already-settled assertion that
Arc itself lost that contract.
(`AGENTS.md:46-49,117-127`;
`scripts/core-language-microbench.sh:479-535,566-586,981-992`;
`docs/dev/codex/brief-502-stack-slope-and-a1-cost.md:385-408,464-496`;
`docs/dev/codex/brief-503-split-the-fourteen-percent.md:232-259`;
`crates/rum-jupyter/src/adapter/worker.rs:312-350`.)

Finally, if the director requires the old lifetime-free low-level API or
caller-thread execution/callback behavior with no alternative ownership
mechanism, the scoped worker design does not meet that requirement. That
would be an explicit semantic/embedding constraint, not a reason to secretly
leak, retain entire arenas, or erase lifetimes. The present contracts to
resolve are the runtime-neutral core export, public builtin installers,
HostSession's Send/lifecycle API and Jupyter's worker ownership.
(`crates/rum-core/src/lib.rs:1-14`; `crates/rum-host/src/config.rs:12-38`;
`crates/rum-host/tests/lifecycle_contract.rs:374-390`;
`crates/rum-jupyter/src/adapter/worker.rs:312-350`.)

### 4. Drafted follow-up briefs

These are drafts for the director to rewrite, **not authority to execute
source work during brief-504**. A → B → C is the proposed sequence: prove
the boundary, implement it with accounting, then independently audit and
measure closure. Each has the template's Goal, Context pointers, Scope,
Deliverable, pitfalls, Verification, Hard rules and Report format. The
scoped-API and generated-identity rulings in section 5 precede dispatch.
(`docs/dev/codex/TEMPLATE.md:19-37,41-169`;
`docs/dev/codex/brief-504-interner-design-under-a-copy-symbol.md:123-155`.)

#### Draft A — prove a Copy symbol can be scoped and cannot cross domains

**Goal**

Build an isolated safe Rust prototype that proves the proposed ownership
and import/export boundary, including dynamic insertion, suspension and a
persistent host owner. Produce compiler evidence before changing production.
This future brief explicitly permits scratch Rust source; brief-504 does
not write any. The prototype must address the public escape shapes at
`crates/rum-runtime/src/runtime/model/requests.rs:89-154,213-219`, the
`'static` frame contract at `crates/rum-runtime/src/runtime/suspension/core.rs:74-98`,
and worker ownership at `crates/rum-jupyter/src/adapter/worker.rs:312-350`.

**Context pointers**

- Read `AGENTS.md`, this report's sections 1–3, and
  `docs/dev/codex/brief-503-split-the-fourteen-percent.md:232-259`.
- `crates/rum-core/src/expr.rs:19-40,250-319`: required fields, operations,
  serialization and currently ambient construction.
- `crates/rum-host/src/config.rs:12-38` and
  `crates/rum-host/src/session.rs:147-221`: installer/reset/snapshot duties.
- `crates/rum-derive/src/lib.rs:81-118`: existing compile-fail contract style.

**Scope**

- MAY write only `target/brief-504/draft-a/**` and append to the dispatched
  brief's report. Use a standalone offline scratch package with its own
  workspace/target/tmp directories inside that subtree, or direct rustc.
- MUST NOT change production crates, scripts, fixtures, dependency manifests,
  the lockfile, other docs or any Git state.
- No new dependency or unsafe implementation; the proposed append-only
  safe-storage mechanism must stand on its own. These restrictions isolate
  the type proof from production behavior and the measured representation
  (`AGENTS.md:109-122`; `docs/dev/codex/TEMPLATE.md:19-25,55-69`).

**Deliverable**

- A generative, invariant epoch witness with private construction; scoped
  Symbol/Expr/Session, a lossless detached type and mandatory import/export.
  Dynamic storage must use owned text and stable, safely borrowed cells.
  Report measured Symbol/Atom/Expr/SharedExpr sizes and `needs_drop` for the
  scoped types; Symbol must be 24 bytes and Copy. No toy result is a kernel
  performance result (`crates/rum-core/src/expr.rs:35-40,536-542,573-580`).
- Negative compilation cases: Symbol and Expr escaping the scope; a copied
  Symbol outliving its detached-root wrapper; two overlapping/nested sessions
  comparing or sharing raw symbols; coercing both brands to a common
  lifetime; evaluating a detached/foreign Expr without import; unseeded
  scoped deserialization; static frame/installer/profile capture; raw access
  through a wrapper; constructing independent tables with one brand. Include
  positive controls and verify the diagnostic addresses the intended rule,
  not a missing import or unrelated error (`crates/rum-core/src/expr.rs:109-138,312-319`;
  `crates/rum-runtime/src/runtime/session/session_requests.rs:174-214`;
  `crates/rum-host/src/config.rs:19-32`).
- Positive runtime cases: append new names while old borrows exist; duplicate
  names; context/name differences; same-brand Eq/Hash/Ord; freeze/export,
  reset and new-ID reuse; old detached output unchanged; dropping the epoch
  with exported data retained frees all arbitrary arena storage. Include
  failed imports and a frame retained across suspend/resume inside the epoch
  (`crates/rum-core/src/expr.rs:250-285`;
  `crates/rum-runtime/src/runtime/suspension/core.rs:74-125`).
- A minimal persistent owner exchanges owned commands/results, resets the
  epoch, and remains Send. Demonstrate the lifetime arrangement without
  self-reference or Pin/unsafe lifetime extension. Record unresolved callback,
  cancellation, startup failure and shutdown semantics; do not assert that
  the worker preserves all host contracts from this prototype
  (`crates/rum-host/tests/lifecycle_contract.rs:374-390`;
  `crates/rum-host/src/session.rs:110-124,162-175`).

**rum / Rust pitfalls**

Copy handles cannot own leases; plain covariance is not domain identity;
`'static` trait objects do not accept borrowed session text; a safe function
cannot return a lifetime-extended reference after releasing a mutable
container guard. The prototype must exhibit the owner graph and compile
its negative cases, not merely state these intentions. Keep arena Drop
bounded and distinguish text ID from generated identity
(`crates/rum-core/src/expr.rs:35-40`; `crates/rum-runtime/src/runtime/suspension/core.rs:76-87`;
`crates/rum-runtime/src/runtime/scoping/packages.rs:420-429`).

**Verification**

Use the repository's declared Rust 1.85 toolchain, offline. The director sets
`BRIEF504A_TEST_TIMEOUT` from a recorded same-shape scratch build/check
before dispatch; it is a required variable, not a fabricated duration.
Run the scratch package's positive tests, its checked compile-fail tests and
its layout reporter through `scripts/run-bounded.py`; record complete argv,
exit status and real diagnostics. For a Cargo-based prototype the gate is:

```sh
CARGO_NET_OFFLINE=true scripts/run-bounded.py --timeout "${BRIEF504A_TEST_TIMEOUT:?required from baseline capture}" -- cargo test --offline --manifest-path target/brief-504/draft-a/Cargo.toml
```

The scratch package must execute the named negative cases (doctests or a
compiler driver) and layout reporter as part of this gate. Use only local
dependencies; do not run the production workspace suite or a toy timing
comparison. Source-free production work has no production test gate
(`docs/dev/codex/TEMPLATE.md:102-105,151-159`; `AGENTS.md:77-93`).

**Hard rules**

- No source change outside the scratch subtree, no network/out-of-repo
  writes, no git commit/branch/switch/merge/push/stash/checkout/restore,
  index/config changes or new unsafe.
- Reject a non-Copy/grown Symbol or changed ID accept path. Reject any safe
  negative case that compiles; do not weaken the case to make the report pass.
- No vendor engines or oracle harnesses. No performance claims from toy
  compilation. If the same step fails twice without new information, stop
  and return the evidence (`docs/dev/codex/TEMPLATE.md:99-105,125-159`;
  `docs/dev/codex/brief-504-interner-design-under-a-copy-symbol.md:54-58`).

**Report format**

1. Prototype owner/type boundary with production `path:line` correspondence.
2. Measured layouts/Copy checks and verbatim positive/negative compiler output.
3. Escape, reset, frame and worker evidence; every unproved host contract.
4. Pass/refutation of the proposed boundary and stop-loss/open questions.

#### Draft B — migrate to scoped interning with mandatory budget admission

**Goal**

After Draft A passes and the director rules on the API breaks, replace the
arbitrary global leaking interner with scoped text ownership and an enforced
detached-value boundary. Include accounting/refusal in this migration; do
not land an intermediate shape that frees text while an old unbranded API
survives. Preserve the ruled Symbol representation and documented language,
snapshot, host and suspension semantics except for explicitly ruled
low-level API changes (`crates/rum-core/src/expr.rs:19-96,250-285`;
`docs/dev/codex/brief-504-interner-design-under-a-copy-symbol.md:54-58`;
`docs/host-embedding-and-hardening.md:84-121`).

**Context pointers**

- Read `AGENTS.md`, Draft A's accepted proof and sections 2–3 of this report.
- Core/export roots: `crates/rum-core/src/lib.rs:7-15`;
  `crates/rum-runtime/src/runtime/mod.rs:73-76`;
  `crates/rum-runtime/src/runtime/kernel_api.rs:79-135`.
- Admission/accounting: `crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:50-338,363-365`;
  `crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:139-189`;
  `crates/rum-runtime/src/runtime/session/session_helpers.rs:30-39`.
- Input/escape: `crates/rum-runtime/src/runtime/lowering.rs:33-40,93-103`;
  `crates/rum-runtime/src/runtime/session/session_requests.rs:137-262`;
  `crates/rum-runtime/src/runtime/model/definitions.rs:28-49`.
- Host lifetime/threading: `crates/rum-host/src/session.rs:103-118,147-221`;
  `crates/rum-jupyter/src/adapter/worker.rs:312-350`.

**Scope**

MAY modify the following explicit areas for the type migration, accounting,
owner boundary and their tests only. Every compiler-forced file outside the
primary mechanisms must be listed in the report with the forcing signature.

- `crates/rum-core/src/**`, including `lib.rs`, `expr.rs`, `heads.rs` and
  `profiling.rs`; new coherent symbol-storage/detached-value modules and tests.
- `crates/rum-runtime/src/**`, including `lib.rs`, `runtime/mod.rs`,
  `runtime/kernel_api.rs`, all affected subsystem/module roots, request/model
  types, lowering, session, metadata, caches, diagnostics, identity, suspension
  and resource accounting.
- `crates/rum-stdlib/src/**`, including `lib.rs`, affected builtin/catalog
  signatures, frame implementations, test roots and ownership-graph fixtures.
  Inventory/ledger updates must explain each changed row; do not bless
  unexplained new storage or weaken a gate.
- `crates/rum-derive/src/**`, including `lib.rs`, only for lifetime propagation
  required by the existing derive contracts; corresponding derive tests.
- `crates/rum-host/src/**`, `crates/rum-host/tests/**`,
  `crates/rum-jupyter/src/**`, `crates/rum-jupyter/tests/**`,
  `crates/rum-terminal/src/**`, `crates/rum-repl/src/**`,
  `crates/rum-cli/src/**`, plus each affected crate's tests/examples and
  module roots, only for the new public types/owner arrangement.
- `docs/guide/embedding.md`, `docs/guide/resource-limits-and-sandbox.md`,
  `docs/host-embedding-and-hardening.md`, `docs/lowering-canonical-forms.md`,
  `docs/reference/builtins.md`, `SECURITY.md` and the dispatched brief's
  report; update API rustdoc alongside implementation. Regenerate only the
  reference changes required by the ruled resource/refusal contract; do not
  change unrelated builtin semantics.
- `target/brief-504/draft-b/**` for captures, base/candidate artifacts and
  `.rum` probes.

MUST NOT modify syntax-language semantics, unrelated algorithms, `AGENTS.md`,
`TODO.md`, `PROGRESS.md`, other briefs, Cargo manifests/lockfile, `.git` or
the benchmark scripts. Any required dependency or extra scope is a stop.
All exporting module roots are expressly included because these types cross
the core/kernel/host boundary (`crates/rum-core/src/lib.rs:1-14`;
`crates/rum-runtime/src/runtime/kernel_api.rs:1-6,79-135`). Perform a split
assessment when touching large files; symbol storage vs expression storage
is a meaningful boundary, not an arbitrary cut (`docs/code-organization-guidelines.md:69-94`;
`crates/rum-core/src/expr.rs:19-319,536-585`).

**Deliverable**

- Complete migration of section 3's owner graph and API boundaries, retaining
  measured 24-byte Copy Symbol, ID/identity equality and existing hot Expr
  layout. No legacy ambient interner, raw unbranded alias, unchecked
  constructor or unseeded scoped Deserialize remains
  (`crates/rum-core/src/expr.rs:35-40,109-138,250-319`).
- A private admission API requiring a checked permit before arbitrary text,
  pair/page or index growth. Retained-table accounting includes discarded
  results and failed requests. The configured limit covers lower-only and
  import paths; active nested limits min-combine. Bound/check ID issuance,
  look up existing names before exhaustion checks, and make refusal recoverable
  without interning an error symbol. Fixed canonical storage is closed and
  bounded; arbitrary System names are local
  (`crates/rum-core/src/expr.rs:57-138`;
  `crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:119-135,174-189`;
  `crates/rum-runtime/src/runtime/session/session_requests.rs:137-169`).
- Detached successful, failure, trace, inspection and serde values; mandatory
  import; explicit foreign-generated-identity refusal under the director's
  ruling; unchanged sealed snapshot re-link/orphan behavior
  (`crates/rum-runtime/src/runtime/model/requests.rs:89-154,213-219`;
  `crates/rum-runtime/src/runtime/model/definitions.rs:28-49`;
  `crates/rum-runtime/src/runtime/session/session_api.rs:432-449`).
- Epoch-bound suspension traits, generic builtin/install callbacks, safe
  persistent HostSession owner, reset/drop/shutdown without retained epochs,
  cancellation/poison behavior and Jupyter worker transfer; controlled worker
  startup failure and no reentrant-command deadlock. Preserve existing public
  host data shapes where they already contain only text/typed data
  (`crates/rum-runtime/src/runtime/suspension/core.rs:74-125`;
  `crates/rum-host/src/config.rs:12-38`;
  `crates/rum-host/src/model/outcome.rs:164-228`;
  `crates/rum-host/tests/lifecycle_contract.rs:374-390`).
- Tests that prove the compile-time negatives on **production** APIs, plus
  independent session quotas, discarded/failed-name retention, repeated
  reset, exported-value survival, exhaustion-at-existing-name, long names,
  contexts, lower-only requests, dynamic Symbol/ToExpression, restore,
  same/different-context equality, lexicographic ordering, generated identity,
  suspension and diagnostics-enabled ownership. Use rum's documented contract
  as the oracle (`docs/testing-strategy.md:3-14,18-38`;
  `crates/rum-stdlib/src/structure_ops/surface.rs:314-347`;
  `crates/rum-runtime/src/runtime/tests/sealed_snapshot.rs:55-125`).
- A per-file compiler-ripple inventory and measured before/after evidence;
  no blanket invocation count borrowed from brief-414. Identify all remaining
  trusted host allocations or intentionally bounded static pools, without
  presenting them as unbounded guest-name exemptions
  (`docs/dev/codex/brief-414-memory-accounting-gaps.md:254-280,793-799`).

**rum / Rust pitfalls**

Do not solve self-reference with unsafe or an immortal owner. Do not demote
equality, add per-symbol leases, or let a detached wrapper expose a naked
handle. Preserve suspension/abandonment, sealed bit-62 identities, canonical
ordering and bounded destruction. A byte permit is mandatory but does not
prove its own projection formula; test capacity growth, overflow, failure
and bootstrap/error paths. Enabled diagnostics must not create another
unbounded global text pool. No new per-evaluation diagnostic lock or
allocation is permitted (`crates/rum-core/src/expr.rs:250-285`;
`crates/rum-runtime/src/runtime/scoping/packages.rs:8-14,420-429`;
`crates/rum-runtime/src/runtime/evaluation_cache/profile/mod.rs:248-264`;
`docs/dev/codex/TEMPLATE.md:76-100`).

**Verification**

Before editing, preserve/build the base artifact at this same path without
Git operations. Set offline Cargo, TMPDIR and build/output paths within the
authorized scratch subtree. The director must assign `BRIEF504B_FMT_TIMEOUT`,
`BRIEF504B_TEST_TIMEOUT`, `BRIEF504B_BUILD_TIMEOUT` and panel timeouts from
recorded same-shape baseline runs before dispatch; no timeout in this draft
is a claimed measurement (`docs/dev/codex/TEMPLATE.md:151-159`). Run focused
suites while iterating, then:

```sh
CARGO_NET_OFFLINE=true scripts/run-bounded.py --timeout "${BRIEF504B_FMT_TIMEOUT:?required from baseline capture}" -- cargo fmt --check
CARGO_NET_OFFLINE=true scripts/run-bounded.py --timeout "${BRIEF504B_TEST_TIMEOUT:?required from baseline capture}" -- cargo test --offline --locked --workspace --no-fail-fast
CARGO_NET_OFFLINE=true scripts/run-bounded.py --timeout "${BRIEF504B_BUILD_TIMEOUT:?required from baseline capture}" -- cargo build --offline --locked --release -p rum-cli
```

No old spike failure exemption carries over automatically. Report skipped
native transport tests as skipped, not executed passes. Preserve all failures
and stop on unexplained inventory drift (`docs/testing-strategy.md:18-38`;
`docs/dev/codex/brief-501-a1-representation-and-cost.md:233-247`).

Use `scripts/core-language-microbench.sh --rum-cli <saved-arm> --output-dir
<in-scope-dir> --timeout <baseline-derived-bound>` for both arms. The script
already delegates its engine run to the bounded runner and closes stdin.
Run on an idle machine, profiling OFF, at most two panel runs per arm;
report every target and same-tier `timings.tsv` row, retaining both raw runs.
The targets are the current rows at `scripts/core-language-microbench.sh:479-535,566-586,981-992,1021-1037`.
Do not use `runs.tsv` process wall-time for timing decisions; its wrapper
timing is separate from the in-kernel rows (`:20-25,1097-1125`;
`docs/testing-strategy.md:83-98`).

**Hard rules**

- No git mutations, network, out-of-repo writes, new unsafe, outside-scope
  edits, vendor comparison or weakened tests. Generated source uses `.rum`.
- Stop for any changed ruled Symbol property, semantic regression, unresolved
  hot-label regression or surviving unbranded entrypoint. A smaller penalty
  than the owning spike does not make a regression acceptable.
- No silent tuning/rerun loop; two identical failures without new evidence
  trigger stop-loss. No diagnostic default-path cost. Preserve and quote
  actual output, including failed runs (`AGENTS.md:38-51,109-132,139-147`;
  `docs/dev/codex/TEMPLATE.md:99-100,125-159`).

**Report format**

1. Changed files/exports, owner graph and compiler-forced ripple.
2. Layout, compile-fail, semantic/lifecycle/accounting evidence verbatim.
3. Base/new per-label timing rows and every regression/skip.
4. Remaining escape/admission risks, approved API breaks and stop-loss.

#### Draft C — independent closure, cost and lifetime gate

**Goal**

Audit Draft B's finished candidate against the original finding and the
three Symbol constraints. Produce an independent closure recommendation,
separating ownership correctness, quota behavior, kernel cost and host
boundary cost. Do not repair a failing candidate in this diagnostic
(`docs/dev/codex/brief-504-interner-design-under-a-copy-symbol.md:19-32,54-58`;
`docs/dev/codex/TEMPLATE.md:26-30,102-105`).

**Context pointers**

- Read `AGENTS.md`, this report and Draft A/B evidence.
- Re-probe the current versions of `crates/rum-core/src/expr.rs:19-138,250-319`,
  `crates/rum-runtime/src/runtime/model/requests.rs:89-154,213-219`,
  `crates/rum-runtime/src/runtime/suspension/core.rs:74-125`,
  `crates/rum-host/src/session.rs:162-221` and
  `crates/rum-jupyter/src/adapter/worker.rs:312-350`; these are pre-migration
  coordinates, not assumed candidate coordinates.
- Use `docs/dev/codex/brief-502-stack-slope-and-a1-cost.md:464-511,559-565,601-628`
  and `docs/dev/codex/brief-503-split-the-fourteen-percent.md:232-259,416-453`
  for the instruction instrument, controls and accepted prior evidence.

**Scope**

- No tracked source changes. MAY write `.rum` workloads, offline external
  Rust consumer probes, captures and analyses under
  `target/brief-504/draft-c/**`, and append the diagnostic's report.
- MUST NOT modify candidate/base source, scripts, fixtures, manifests,
  lockfile, other docs or Git state. Baseline and candidate binaries must be
  supplied by the director/Draft B; no branch operations to manufacture them
  (`docs/dev/codex/TEMPLATE.md:26-30,125-159`).

**Deliverable**

- Fresh external-consumer compilation attempts covering section 3's
  falsifiers. Audit core reexports, standalone lowering, direct evaluation,
  installers, all response payloads, serde and frame/callback paths.
  A passing unit test of the importer does not establish that these other
  interfaces require it (`crates/rum-core/src/lib.rs:7-14`;
  `crates/rum-runtime/src/runtime/kernel_api.rs:79-135`;
  `crates/rum-runtime/src/runtime/session/session_requests.rs:174-214`).
- A repeated-request/reset ownership and quota report with detached values
  retained, both profiling off and enabled, and two independently budgeted
  sessions. Reproduce the finding's fresh-name request shape under Pure;
  report live-allocation/ledger evidence separately from RSS. Prove that
  reset frees arbitrary arena storage, not merely that one RSS reading fell.
  Include a failed request/import and a pending suspension reset
  (`docs/dev/codex/brief-504-interner-design-under-a-copy-symbol.md:29-32`;
  `crates/rum-runtime/src/runtime/evaluation_cache/profile/mod.rs:218-264`;
  `crates/rum-host/src/session.rs:162-175`).
- The full rum-vs-rum timing gate, plus spike-comparable ordinary, cold/warm
  and symbol-hot instruction rows with setup/scaling controls. Report total
  and payload costs, observed spreads, successful liveness and limit refusals
  separately. No numerical cost attribution to the brand/arena/export/worker
  without a controlled contrast (`scripts/core-language-microbench.sh:479-535,566-586,981-992`;
  `docs/dev/codex/brief-502-stack-slope-and-a1-cost.md:484-511,559-565`).
- Host-boundary evidence: repeated small calls and large detached imports/
  exports, worker startup/reset/shutdown, cancellation while active/suspended,
  panic recovery, thread transfer and snapshot re-link. Measure request
  latency with Rust's monotonic Instant in a bounded host probe, outside
  the in-kernel timing decision. Report worker memory/resource observations
  separately. Do not infer those costs from the ordinary kernel loop
  (`docs/host-embedding-and-hardening.md:61-105`;
  `crates/rum-host/tests/lifecycle_contract.rs:374-406`;
  `crates/rum-runtime/src/runtime/tests/sealed_snapshot.rs:55-125`).

**rum / Rust pitfalls**

Run exact benchmark statements; a failed/limited loop is not a fast loop.
Validate liveness and scaling before accepting an instruction row. Keep
allocator counters out of instruction/timing runs. RSS is not the ownership
ledger. A `'static` detached value must own its data, not conceal a leaked
arena. The fixed canonical pool must not grow with arbitrary System names
(`docs/dev/codex/brief-502-stack-slope-and-a1-cost.md:127-145,484-496,601-628`;
`crates/rum-core/src/expr.rs:132-138`;
`docs/host-embedding-and-hardening.md:54-59`).

**Verification**

Run the production gates once on the unchanged candidate to verify Draft B's
reported state; run external-consumer positive/negative probes offline with
the bounded runner. The director supplies baseline-derived timeouts and the
binary/workload manifest before dispatch. For each gate record argv,
environment, binary/workload hashes, exit status and verbatim output.

- Workspace: offline bounded `cargo fmt --check` and
  `cargo test --offline --locked --workspace --no-fail-fast`.
- Timing: at most two complete `core-language-microbench.sh` runs per arm,
  fixed environment, explicit in-repo output directory and supplied
  `--rum-cli`; compare all hot rows and same-tier neighbors.
- Instructions: reuse the original workload/control bytes when available;
  otherwise reconstruct from the recorded statements and label the new
  baseline as a fresh comparison. Three runs per arm per workload/control,
  `taskset -c 2 perf stat -e instructions:u -x, -- <binary> <workload>.rum
  < /dev/null`, full core-event coverage required. Set a finite run manifest
  before measurement, covering only base and this candidate; no old-arm
  rebuilds or automatic repeated panels.
- Allocation and host-boundary probes: a separate finite manifest, with
  counter/timing separation and successful-work assertions. Repeat a cell
  only for a named unresolved concern, within the declared budget.

These are future commands/gates, not runs claimed in brief-504. Their
discipline follows `docs/dev/codex/TEMPLATE.md:76-100,102-123,143-159` and
`docs/dev/codex/brief-503-split-the-fourteen-percent.md:416-453`.

**Hard rules**

- No production repairs, Git mutations, network, out-of-repo writes, unsafe,
  vendor engines or weakened cases. Use only the supplied artifacts and
  authorized scratch. Keep generated rum code suffixed `.rum`.
- No CPU-heavy concurrency during measurement; no perf coverage substitution,
  false “green” from skipped tests, or hidden reruns of anomalous data.
- Stop on a lifetime/domain counterexample or two failed attempts without
  progress. A semantic or hot-neighbor regression prevents acceptance;
  the +10.93% floor is a comparison, not a tolerance
  (`AGENTS.md:38-51,109-132`; `docs/dev/codex/TEMPLATE.md:99-100,125-159`).

**Report format**

1. Lifetime/admission counterexamples sought and actual compiler outcomes.
2. Retention/reset/tenant-isolation evidence, including failed paths.
3. Kernel timing/instruction tables and separate boundary/resource costs.
4. Closure verdict against the original finding; all failures, unsupported
   claims, skipped tests and open questions. If the candidate fails, return
   to the director; do not silently substitute A2 and call the finding closed.

### 5. Open questions and limits of this report

1. **API ruling.** The proposed brand changes the public type signatures of
   Symbol/Expr/Session while preserving the three mandated representation
   properties. Confirm that this is an allowed dimension, and explicitly
   supersede brief-414's old “no lifetime-bearing public Expr” restriction
   and the older stable-fingerprint ruling before dispatch. Those belonged
   to the owning proposal and are incompatible with this one
   (`docs/dev/codex/brief-414-memory-accounting-gaps.md:743-746`;
   `TODO.md:1174-1181`;
   `docs/dev/codex/brief-504-interner-design-under-a-copy-symbol.md:54-58`).
2. **No compiled lifetime proof or new cost result.** The safe append-only
   arena, invariant brand, generic frame traits and persistent owner have
   been designed, not implemented. In particular, no assertion of a
   percentage saving, unchanged code generation, final layout, worker
   latency or RSS reclamation has been established here. Draft A is needed
   before the cross-crate migration; Draft B/C gate the actual runtime
   against `AGENTS.md:117-127` and
   `docs/dev/codex/brief-504-interner-design-under-a-copy-symbol.md:138-140`.
3. **Host execution contract.** Decide whether moving execution/providers/
   installers to a persistent owner worker is acceptable, including callback
   thread affinity, reentrancy, thread creation failure, stack provisioning,
   shutdown and per-session worker resources. The existing public surface
   guarantees Send and lifecycle behavior; it does not itself prove all
   downstream integrations tolerate this arrangement
   (`crates/rum-host/src/config.rs:12-38`;
   `docs/host-embedding-and-hardening.md:61-105`;
   `crates/rum-jupyter/src/adapter/worker.rs:312-350`).
4. **Generated-symbol import contract.** Rule on the proposed default refusal
   for foreign/reset-epoch generated identities and the explicit trusted
   alternative. Preserving ordinary text imports is straightforward in the
   design; preserving arbitrary old Rust numeric-identity behavior while
   claiming new isolation is not. Snapshots already have a distinct
   re-registration contract that must remain intact
   (`crates/rum-core/src/expr.rs:118-125,288-319`;
   `crates/rum-runtime/src/runtime/scoping/packages.rs:420-429`;
   `docs/dev/sealed-scoping-design.md:219-246`).
5. **Quota/bootstrap semantics.** Choose the exact deterministic page/index
   charge, treatment of fixed canonical storage, and recoverable behavior
   when configured limits cannot cover initial session storage. New
   insertion must reserve growth before allocation; a later sampled walk is
   insufficient. The current host builder is infallible, so a fallible build
   API or a documented initialization-failure result is an explicit API
   decision, not an `expect` to add during implementation
   (`crates/rum-host/src/session.rs:84-95,147-159`;
   `crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:139-189`;
   `crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:50-56`).
6. **Prior-report residual wording.** The brief's “under 0.1 percentage points
   across twelve workloads” sentence is broader than the supplied report's
   table: the ordinary payload residual is -0.0840 pp, while the report
   explicitly discusses larger non-ordinary residuals. I use the ordinary
   row for the quoted attribution and accept the brief's representation
   ruling; I did not rerun or re-litigate the spike. The director should
   reconcile that sentence before reusing it as a claim about every workload
   (`docs/dev/codex/brief-504-interner-design-under-a-copy-symbol.md:44-46`;
   `docs/dev/codex/brief-503-split-the-fourteen-percent.md:235,271-280`).
7. **Unmeasured insertion and ordering costs.** The initial owned-string
   storage proposal may duplicate context bytes and lose current shared-context
   pointer hits in Ord. The existing projected-byte helper also performs a
   full session walk, so naively invoking it for every miss can scale badly
   with retained session state. Measure these effects explicitly, including
   same-context sorting and insertion with large pre-existing definitions;
   do not infer a cheap fast path from the zero-size brand
   (`crates/rum-core/src/expr.rs:57-70,277-284`;
   `crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:174-189`;
   `crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:108-115`;
   `AGENTS.md:117-122`).
8. **Current scope is satisfied by evidence and drafts, not a landed fix.**
   No source, script file, fixture, benchmark workload or compiler probe was
   written. No compilation, test suite, executable workload or performance
   measurement ran. Existing `scripts/run-bounded.py` wrapped inline evidence
   processing. The finding remains open until an implementation passes the
   gates. Only this report and evidence scratch under `target/brief-504/` are
   authorized outputs
   (`docs/dev/codex/brief-504-interner-design-under-a-copy-symbol.md:142-155`;
   `docs/dev/codex/TEMPLATE.md:102-105`).

#### Verbatim source-probe evidence

The initial worktree status was empty. Source reads used `cat`, `nl`, `sed`
and `rg`; exploratory searches for obsolete `request_api`/`evaluation`/
`session.rs`/host `value.rs` paths reported missing files and were replaced
by the current paths cited above. Those failed path guesses support no
finding. No compilation, binary execution, RSS sampling or benchmarking
occurred. A bounded inline Python capture retained the exact source excerpts
and a SHA-256 manifest in `target/brief-504/source-evidence.txt` and
`target/brief-504/source-manifest.json`; these are evidence data, not scripts.

The capture used subprocess argument arrays, with `GIT_OPTIONAL_LOCKS=0`.
Verbatim selected command results follow (the `$` lines are the capture's
space-joined argv display, not shell commands to paste without quoting):

```text
$ git status --porcelain=v1
[exit 0]
$ rg -n interned_id\( crates
crates/rum-core/src/expr.rs:166:    pub fn interned_id(&self) -> u32 {
crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:70:    let mix = u64::from(symbol.interned_id()).wrapping_mul(0x9e3779b97f4a7c15) ^ identity;
[exit 0]
$ rg -n Box::leak|OnceLock<Mutex<SymbolInterner>>|interned symbol count should fit crates/rum-core/src/expr.rs
48:    static SYMBOL_INTERNER: OnceLock<Mutex<SymbolInterner>> = OnceLock::new();
66:        let leaked: &'static str = Box::leak(value.to_owned().into_boxed_str());
88:    let next_id = u32::try_from(pairs.len()).expect("interned symbol count should fit in u32");
90:        Box::leak(Box::new(SymbolText {
[exit 0]
$ rg -n interner crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs
[exit 1]
```

The no-match result is accompanied by the complete memory-walk source in
the evidence capture. This report's findings are deductions from that
source, not a claim to have repeated the director's measurements.

Final read-only verification, captured in
`target/brief-504/verification.txt:1` (citation validation checks existence
and line bounds, not the truth of the design):

```text
Original brief prefix preserved: True
Changed inspected reference/source files: []
Missing or out-of-range cited anchors: []
Markdown code fences balanced: True
$ GIT_OPTIONAL_LOCKS=0 git diff --check
[exit 0]
$ GIT_OPTIONAL_LOCKS=0 git diff --name-only
docs/dev/codex/brief-504-interner-design-under-a-copy-symbol.md
[exit 0]
$ GIT_OPTIONAL_LOCKS=0 git status --porcelain=v1
 M docs/dev/codex/brief-504-interner-design-under-a-copy-symbol.md
[exit 0]
```
