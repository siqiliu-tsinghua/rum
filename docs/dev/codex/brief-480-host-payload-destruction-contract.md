# brief-480 — design-only: a destruction contract on stored host payloads

**Shape: DESIGN-ONLY. `Scope: NONE — no source change.`** The deliverable is a
reviewable plan plus drafted follow-up implementation briefs in `TEMPLATE.md`
shape. You write no production code and no test code in this brief.

## Goal

Produce the plan for a **breaking embedding-API change**: every host-supplied
payload that rum *stores* — and therefore destroys — must declare a bounded
destruction contract, and an implementor that takes no stance must **fail to
compile**, including from a crate outside this workspace. The plan must settle
how that declaration is expressed for each of the stored payload kinds, what it
can honestly promise given the compiler cannot inspect a foreign `Drop`, how the
ownership ledger consumes it, how the boundary is certified at the crate edge,
and what the migration costs at every in-tree implementor. The user has ruled
the API break acceptable (2026-09-19: no downstream consumers, `v0.2.0` just
tagged, API deliberately not locked), so **do not propose a documentation-only
contract as the primary option** — a doc note is the fallback the plan must
argue against, not the recommendation.

## Context — why this exists, with evidence

rum has three enforcement layers for recursion and they are all **call**-shaped:
the recursion inventory models call cycles, the per-site registry models
per-builtin descent, and `runtime/native_stack_guard.rs` bounds native call
depth in bytes. **None of them models ownership.** The host destructor walks an
ownership graph, and both real drop hazards found this month (`NestedPathNodes`,
`LookupOutcome`) were found by accident rather than by a check.

brief-479 built the discovery half — a test-only `syn` scanner at
`crates/rum-stdlib/src/tests/ownership_graph/`, pinning its results in
`inventory/*.tsv`. Its `host_payloads.tsv` has **48 `UnknownHostPayload` rows**.
42 are public inputs taken by value; **6 are owned storage edges**, i.e. places
rum keeps the payload and runs its destructor. Verbatim from
`crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payloads.tsv`
(the rows whose witness does not begin `public input`):

| file:line | field / path | payload |
|---|---|---|
| `crates/rum-runtime/src/runtime/file_io.rs:95` | `FileIoProvider.inner` / `Arc[LastStrongHolder]` | `dyn FileIo` |
| `crates/rum-runtime/src/runtime/model/session.rs:95` | `Session.print_sink` / `Option[Own]/Box[Own]` | `dyn FnMut(&str) + Send` |
| `crates/rum-runtime/src/runtime/random.rs:147` | `EntropyProvider.inner` / `Arc[LastStrongHolder]` | `dyn EntropySource` |
| `crates/rum-runtime/src/runtime/suspension/core.rs:17` | `SuspendedEvaluation.frames` / `VecDeque[Own]/Arc[LastStrongHolder]` | `dyn SuspensionFrame` |
| `crates/rum-runtime/src/runtime/suspension/core.rs:96` | `FrameAction::Evaluate.next_frame` / `Arc[LastStrongHolder]` | `dyn SuspensionFrame` |
| `crates/rum-runtime/src/runtime/suspension/core.rs:98` | `FrameAction::Finalize.0` / `Box[Own]` | `dyn PureFrameFinalizer` |

**The director's reading of that table, which you should verify rather than
assume.** The six are two different problems wearing one label:

- Three are genuine **embedding boundaries**: `FileIoProvider`
  (`file_io.rs:94-113`, constructors `new` and `from_arc`), `EntropyProvider`
  (`random.rs:146-164`, same two constructors), and `Session::set_print_sink`
  (`model/session.rs:131-139`, re-exported through `rum-host/src/session.rs:255`
  which wraps an `impl FnMut(&str) + Send + 'static`).
- Three are rum's **own** suspension machinery behind `#[doc(hidden)] pub`
  traits (`SuspensionFrame`, `PureFrameFinalizer`, `suspension/core.rs:75-99`).
  They are flagged because they are externally implementable in principle, not
  because a host supplies them today.

The director also checked one adjacency: `FrameAction::Evaluate.next_frame` is
**not** a chained owning edge — `suspension/core.rs:624-628` pushes it onto the
`VecDeque` immediately, so the deque is flat. Confirm this from source; if it is
wrong, say so, because it changes whether the suspension rows are a depth hazard
in their own right.

## What the plan must decide

Each item below is a question I will rule on. Answer it with a recommendation
**and** the alternative you rejected, every claim carrying `path:line`.

1. **The declaration mechanism, per payload kind.** Candidates include a
   supertrait marker (`trait FileIo: BoundedDestruction`), a required
   associated item that cannot be defaulted, a `rum-derive` derive alongside the
   existing `FrameRootTrace` / `PureFrameAbandon`
   (`crates/rum-derive/src/lib.rs:45,55`), or a constructor that demands a proof
   token. Pick one shape per kind and say whether one shape can cover all of
   them. **The acceptance criterion is uniform: an implementor that says
   nothing must not compile.**

2. **`print_sink` is the crux and probably cannot take the same shape.** It is a
   bare `Box<dyn FnMut(&str) + Send>`; a trait cannot be implemented on the
   host's closure. So either the sink becomes a named trait, or the setter takes
   a wrapper constructed through a stance-taking constructor, or the sink is
   argued out of scope on the grounds that a closure's captured state is the
   host's and already bounded. Take a position; this is the single design
   decision most likely to change the shape of the whole change.

3. **What the declaration can honestly promise.** The compiler cannot verify a
   foreign `Drop`. State plainly what the mechanism buys — a forced stance, a
   named owner, a ledger row that stops being unknown — and what it does not.
   If your analysis is that a declaration is worth less than it costs for one of
   the six, say so and argue it; an honest negative is a valid answer for a
   subset, but not for the whole change, which the user has already ruled.

4. **Scope of the three kernel_api rows.** The director's position is that
   `SuspensionFrame` and `PureFrameFinalizer` take the same declaration, since
   they are `pub` and externally implementable (14 and 5 in-tree impls
   respectively, per `grep -rn "impl SuspensionFrame for"`). Argue for or
   against, noting that `SuspensionFrame` already carries the `FrameRootTrace`
   obligation, so a second obligation may compose with the first rather than
   duplicate it.

5. **Whether any of the 42 `public input` rows is actually stored.** The scanner
   classified at the signature. Check a representative sample and report whether
   a by-value public input is retained anywhere, which would mean the storage
   set is larger than six. Give the sample size and the method.

6. **How the ownership ledger consumes the declaration.** After the change, what
   does a `host_payloads.tsv` row for a declared site look like, and what makes
   a *newly added* undeclared storage site red? This is step two of the ledger
   seen from one side; do not design the ledger's `UnknownConstruct` second axis
   here (that is its own brief), but do say what interface this change owes it.

7. **The certificate, and it must hold at the crate edge.** The precedent is the
   two `compile_fail` doctests at `runtime/native_stack_guard.rs:19` and `:79`.
   The standing lesson is that a boundary can hold inside the repository and
   evaporate outside it: a source scan cannot establish a property about code
   that does not exist yet, and the gate that settled that question compiled an
   **external** crate and required the compiler errors. Say per mechanism
   whether a `compile_fail` doctest suffices or an out-of-workspace compile test
   is required, and where such a test would live.

8. **The falsifier.** A check nobody has watched fail is not known to work.
   Every follow-up brief you draft must contain the proof that its enforcement
   *can* fire, and name the perturbation that makes it red against the
   post-change tree. Probe the premise: if a proposed certificate would fail
   identically on the pre-change build, it proves nothing and must be redesigned.

9. **The full ripple inventory**, `path:line`, every implementor and call site
   that must change: `FileIo` impls, `EntropySource` impls, `set_print_sink`
   callers (kernel and the `rum-host` wrapper), `SuspensionFrame` and
   `PureFrameFinalizer` impls, plus any doc that states the current contract
   (`docs/dev/embedding-api-line.md`, `docs/dev/host-embedding-line.md`,
   `SECURITY.md` if it makes a claim here). Tests count as call sites.

10. **Step decomposition.** Order the steps so each one leaves a green tree, and
    say which step is the irreversible API break. If one mechanism can land
    ahead of the others as a proving ground, name it and say why it is the
    right first one.

## Scope

- **Files you MAY modify: NONE.** No source change, no test change, no doc
  change. Your report is the deliverable, plus the drafted briefs *inside* that
  report (do not create files under `docs/dev/codex/`).
- Do not run any workload that reproduces a crash, an overflow, or a process
  termination. This brief needs no executable reproduction of any failure: the
  evidence is source reading plus, at most, `cargo check` / `cargo test` to
  confirm a claim about what compiles today.

## Deliverable

1. A step-by-step plan: per step, the files and functions, the invariant that
   stays intact, the gate that decides it, and the falsifier for any check it
   builds.
2. Answers to all ten questions above, each with a recommendation, the rejected
   alternative, and `path:line` evidence.
3. The full ripple inventory as a table.
4. **Drafted follow-up implementation briefs in `TEMPLATE.md` shape** — usable
   drafts, not step summaries. These are raw material; the director rewrites
   them before dispatch.
5. Risks and open questions, explicitly separated from the recommendations.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Do NOT modify any file. If you believe a change is needed to answer a
  question, describe it in the report instead.
- Touch nothing outside this repo; no network.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only; no differential testing against other engines.
- **No new `unsafe`** may appear in any drafted brief for `rum-core` or
  `rum-runtime`; both are zero-`unsafe` today.
- **Cost is not an argument** (user ruling 2026-08-27). A count of call sites
  describes the work; it does not rank an option down. Pick the most correct
  mechanism. A fix the compiler enforces beats one that relies on people
  remembering, even at an order of magnitude more work. Do not propose
  stopgap-now / real-fix-later.
- Scope restraint still applies: this change covers stored host payloads, not a
  general ownership redesign, and not the ledger's second axis.
- Paste verbatim output for any command you do run; if you did not run
  something, say so rather than implying you did.

## Report format (your final message)

1. "Design-only, no source change" plus the ten answers.
2. Any command you ran, with verbatim output.
3. The plan, the ripple table, and the drafted briefs.
4. Stop-loss / open questions — where you stopped and why, anything unsure.
