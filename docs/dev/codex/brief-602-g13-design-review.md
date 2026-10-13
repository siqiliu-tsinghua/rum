# brief-602 — review: is the G13 repair design acceptable?

`Scope: NONE — review only.` **Do not run `git` in any form**, including
before you have read this brief. You write no code and run no probes: this is
a reading review of a design another worker produced.

## Background

G13 is a defect in rum's suspension machinery. When an evaluation suspends for
input, frames are queued and later resumed; a frame hands its completed value
to the next frame as an `Expr`. Typed continuation state (an `Option<Expr>`, a
`bool`, a match outcome, …) is encoded in-band as `Expr` under
``System`InternalContinuation*`` heads and decoded by the next frame. Those
heads are ordinary symbols matched by text, so guest source can write them;
the decoders are lenient or panic on unexpected shapes; and nothing checks
that a frame completes the type its consumer decodes. Six frames complete a
guest value raw into an `Option<Expr>` slot, so a guest value such as
``System`InternalContinuationSome[5]`` resumes to `5`, while the same program
with its input queued in advance returns `InternalContinuationSome[5]`
(21 executed divergences, reproduced by the director).

Two design briefs produced the repair under review:

- `docs/dev/codex/brief-600-g13-typed-continuation-state-design.md` — the
  inventory (§2-§3, verified) and a first design (§4), whose binding rule the
  director rejected.
- `docs/dev/codex/brief-601-g13-design-revision.md` — the director's
  corrections (top) and the revised design (its report, §1-§7): an
  out-of-band channel `FrameOutcome { Value(Expr), State(Box<dyn
  ErasedSlotValue>) }`, typed frames with `Input`/`Output`, the output bound by
  a `return suspend_typed(…)` in the macro's suspend arm, a per-stage split of
  the dispatch-plan frames, a no-panic mismatch error, a `marker-corpus`
  oracle falsifier, and six drafted implementation steps (602-607 in its
  numbering).

Supporting tables are under `.temp/brief-600/` and `.temp/brief-601/`
(`macro-use-classification.tsv`, `ast-macro-positions.tsv`,
`insertion-routes.tsv`, `propagation-edges.tsv`, …).

## The three requirements the repair must meet

1. A frame cannot complete a slot with the wrong type — **a compile error**,
   not a convention.
2. Guest code cannot construct anything a decoder accepts as typed state —
   **out of band**, not a renamed symbol.
3. Decoding cannot panic or misread on guest input.

Plus the project's standing rules: no ordinary-path cost (the hot evaluator
path must not slow down), no new `unsafe`, and a fix the compiler enforces is
preferred over one that relies on review.

## What to review

Judge the revised design (brief-601's report, read with brief-600's inventory)
and answer, each with `path:line` evidence from the source or the tables:

1. **Soundness of the binding.** Does `return suspend_typed(session, frame,
   request)` really force `F::Output` to equal the enclosing function's or
   closure's Ok type in every macro site? Consider closures, generic frames,
   `where` clauses, sites whose enclosing function does not return
   `Result<_, EvalError>`, and any way the binding can be satisfied while the
   frame still completes the wrong thing.
2. **The input side.** The report says a frame's `Input` cannot be bound at
   compile time without an ordinary-path cost, so it is declared and checked
   at runtime. Is that true?
3. **Coherence of the step plan.** Can each drafted step compile and land
   green on its own, in the order given? In particular, check the first step's
   claim that every frame becomes an identity `Expr → Expr` typed frame with no
   behaviour change while the return-form macro lands in the same step.
4. **Requirement 2 in practice.** Can any implementor — in the kernel, the
   standard library, or an external crate using the exported API — still put
   typed state through the `Value` arm, or otherwise reintroduce in-band
   encoding?
5. **Coverage.** Is every route by which a frame enters a queue or a
   completion reaches a frame accounted for, with a binding? Are there macro
   sites the classification misses?
6. **Propagation without a frame.** `propagation-edges.tsv` lists 51 edges
   where a may-suspend call is propagated across a change of Ok type. The
   report says none diverges today. What happens to those edges when typed
   state moves out of band?
7. **Representation and ownership.** Is `Box<dyn ErasedSlotValue>` the right
   choice over a closed enum, and is its stated ownership/destruction
   disposition honest?
8. **Relation to E2** (`.temp/brief-583/e2-outline.md`, a may-suspend result
   type with no `Try`): is the stated sequence right?
9. Anything else that is wrong, missing or over-scoped.

## Verdict

End with one of: **accept**, **accept with required changes** (list them), or
**reject** (say why and what instead). Rank findings by severity; separate
what you verified in source from what you infer.

## Scope

- Files you MAY modify: **NONE**, except: append your review to this file;
  scratch notes under `.temp/brief-602/`.
- No scratch test, no Cargo command, no probe of any kind.

## Hard rules

- **No `git`, in any form.**
- Do NOT modify any file outside the exceptions; STOP and report instead.
- No network; nothing outside this repository.
- Commands: file reads, `rg`/`grep`, `python3` over the `.temp` tables
  (through `python3 scripts/run-bounded.py --timeout 300 -- …`).
- **Testing Policy is a hard line** (`AGENTS.md`).

## Report format (append to this file)

1. Findings, ranked by severity, each with evidence and whether verified or
   inferred.
2. Answers to questions 1-9.
3. Verdict and the required changes.

---
