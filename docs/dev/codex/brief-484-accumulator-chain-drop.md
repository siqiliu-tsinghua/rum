# brief-484 — design-only: bounding the accumulator chains' destruction

**Shape: DESIGN-ONLY. `Scope: NONE — no source change.`** The deliverable is a
plan plus drafted implementation briefs in `TEMPLATE.md` shape. You write no
production code and no test code.

This is **step D's prerequisite**, not step D. brief-480's plan refused to let
every suspension frame declare `BoundedDestruction` while some of them own
ownership chains whose destruction is unbounded. This brief is about repairing
those chains. Step D — the declaration on `SuspensionFrame` and
`PureFrameFinalizer` — cannot be dispatched until it lands.

## What is already settled — do not re-derive it

- **The hazard is measured and it is reachable.** Director probe, a faithful
  replica of `IndexChunk` (`Option<Arc<Self>>` + payload), release build,
  `ulimit -s 2048` (the ruled 2 MiB embedder stack), three runs per point:
  **64 000 nodes survive 3/3, 66 000 die 3/3** with
  `thread 'main' has overflowed its stack` — roughly **32 bytes of stack per
  level**. brief-477 established that such a tail grows **one node per
  element** on the string-replacement resume paths and reached length 1 000
  with the generated-element and iteration-work limits *both* set to 8, so a
  65 000-character string reaches the ceiling. The failure is a **process kill
  during destruction**, which the native stack guard cannot catch: the guard
  bounds the evaluator's call stack, not the destructor.
- **Calibration caveat, and the plan must close it.** That number measures the
  *shape*, not any real type. `WithBindingNode`
  (`runtime/scoping/lexical/with.rs:12-16`) carries a `Symbol` and an `Expr`;
  `PersistentStringTextNode` (`stdlib/strings/suspended_state.rs:101-104`)
  carries a `String`. A heavier node means a **lower** ceiling, and an `Expr`
  payload drags in `Expr`'s own standing destruction debt. Do not treat 65 000
  as any type's ceiling.
- **The family is 29 types**, every one `previous: Option<Arc<Self>>` with no
  compaction, spread across `rum-runtime` and `rum-stdlib`. Derive the list
  yourself from
  `crates/rum-stdlib/src/tests/ownership_graph/inventory/self_reaching.tsv`
  (the rows whose ownership path contains `Option[Own]/arg0/std::sync::Arc`)
  rather than from a name pattern — several do not start with `Persistent`.
  Report the count you get; if it is not 29, that is a finding.
- **None of the 29 has an `impl Drop` today.** The in-tree precedent for an
  iterative destructor is `LookupOutcome`
  (`stdlib/associations/basic/lookup.rs:24-45`): the `Drop` takes the
  contents and hands them to a `#[cold] #[inline(never)]` worklist loop. Its
  shape is a `Vec<Self>` tree, **not** an `Arc` chain, so it is a precedent for
  the idiom, not a template to copy.

## What the plan must decide

Each is a question I will rule on. Give a recommendation, the rejected
alternative, and `path:line` evidence.

1. **The unlink idiom for an `Arc` chain.** Unlinking requires being the last
   strong holder, so the loop has to take ownership node by node — around
   `Arc::into_inner` or an equivalent — and **stop when a node is still
   shared**, because another holder owns the rest of that tail. Argue that the
   loop terminates, that it drops exactly what a recursive drop would, and that
   stopping at a shared node is correct rather than a leak. Say what happens
   when two chains share a tail and the *second* holder is itself deep.
2. **Whether one mechanism can serve all 29.** A generic helper over a trait
   that exposes "take my `previous`", a macro, or 29 hand-written `Drop` impls.
   A shared mechanism that the compiler applies is worth more than 29 correct
   copies; a mechanism that silently does nothing for a type that forgets to
   opt in is worth less than none. Which is it here?
3. **Coexistence with the existing obligations.** These types derive
   `FrameRootTrace`, and several are reachable from `SuspensionFrame`
   implementors that also owe `FrameAbandon`. Confirm a manual `Drop` composes
   with those derives, and check whether a manual `Drop` blocks anything the
   current code relies on (destructuring moves out of a type with `Drop` do not
   compile — say which of the 29 are destructured today and what that costs).
4. **Which of the 29 genuinely need it.** Some may be short by construction.
   "Short by construction" is a claim requiring proof — an enforced cap at the
   only growth site, not an argument that no caller would go that far. Classify
   each: needs the repair, or bounded with the enforcing `path:line`. Anything
   you cannot prove bounded needs the repair.
5. **The certificate, and the premise problem it carries.** A test that builds
   a long chain and drops it passes after the repair and **terminates the test
   binary before it**, so it cannot be a before/after pair in one binary. The
   traversal-depth line solved the same problem with a shallow companion
   pinning the behavior and a forced-parameter falsifier. Design the analogue:
   what proves the drop is *iterative* rather than merely surviving one chosen
   length? A certificate that only asserts "did not crash at N" is weak — it
   passes on a recursive drop with a slightly larger stack.
6. **The falsifier.** Name the perturbation that makes each proposed check go
   red on the post-repair tree, and probe its premise: a check that would fail
   identically on the pre-repair build proves nothing.
7. **Per-type ceilings, without reproducing a crash.** The plan may need each
   type's per-node frame cost. Say how to obtain it **without** driving a
   process to termination — a size computation, a `/proc` VmStk slope, or a
   reasoned frame estimate from the node's fields. Past-ceiling probing is the
   director's and is not delegated.
8. **Step decomposition**, each step leaving a green tree, with the proving
   ground named and argued.

## Scope

- **Files you MAY modify: NONE.** Your report is the deliverable, and the
  drafted briefs go *inside* it — do not create files under `docs/dev/codex/`.
- **Do not run any workload that reproduces a crash, an overflow, or a process
  termination.** The ceiling is already measured and is quoted above; nothing
  in this brief needs another one. Evidence is source reading plus, at most,
  `cargo check` / `cargo test` to confirm what compiles today.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Do NOT modify any
  file; if a change seems needed to answer a question, describe it instead.
- Touch nothing outside this repo; no network.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only.
- **No new `unsafe`** may appear in any drafted brief. `rum-core` and
  `rum-runtime` are zero-`unsafe`, and an iterative destructor is not a reason
  to introduce the first — if you believe one is unavoidable, STOP and say so
  rather than drafting it.
- **No default-path cost.** Several of these types are constructed on hot
  resume paths. A per-push cost to make destruction cheaper is a trade that
  needs stating, not assuming; any drafted step touching a hot path carries the
  microbench A/B gate.
- **Cost is not an argument** (user ruling 2026-08-27). Twenty-nine types
  describe the work. A mechanism the compiler enforces beats one relying on
  people remembering, even at an order of magnitude more work. No stopgap.
- Scope restraint still applies: this is the accumulator-chain family, not a
  general `Drop` redesign, and explicitly **not** `Expr` — its iterative drop
  was rejected on a measured 20-50% hot-path regression and stays standing
  debt.
- Paste verbatim output for anything you run; if you did not run something, say
  so.

## Report format (your final message)

1. "Design-only, no source change", the derived type list with its count, and
   the eight answers.
2. Any command you ran, with verbatim output.
3. The plan, the per-type classification table, and the drafted briefs.
4. Stop-loss / open questions, separated from the recommendations.
