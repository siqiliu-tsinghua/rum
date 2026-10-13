# brief-604 — review 2: is the G13 typed composition design ready to implement?

`Scope: NONE — review only.` **Do not run `git` in any form**, including
before you have read this brief. You write no code and run no probes: this is
a reading review of a design another worker produced.

## Background

G13 concerns rum's suspension frames. When an evaluation suspends for input,
frames are queued and later resumed; each frame hands its completed value to
the next. Typed continuation state (an `Option<Expr>`, a match outcome, …) is
currently encoded in-band as `Expr` under ``System`InternalContinuation*``
heads and decoded by the consumer; the heads are ordinary symbols that guest
source can also write, the decoders are lenient or panic on unexpected shapes,
and nothing checks that a frame completes the type its consumer decodes. Six
frames complete a guest value raw into an `Option<Expr>` slot (21 executed
divergences between the resumed path and the queued-input path).

You reviewed the second design round in brief-602 and rejected it as ready for
implementation (nine findings F1-F9, seven required changes). Your review was
adopted in full. The third round answers it:

- `docs/dev/codex/brief-603-g13-typed-composition-design.md` — the director's
  brief (top: rulings and what it had to settle) and GLM's report (§1-§7):
  a closed `ContinuationState` enum carried out of band, a sealed `SlotValue`,
  `TypedFrame` with `Input`/`Output`, a zero-sized witness binding `Input`, the
  return position binding `Output`, checked constructors for `Evaluate` and
  finalizers, a `FrameChain` builder for chained inserts, a `ContinueToken` for
  helper re-suspension, a named uncatchable mismatch variant, and an eleven-step
  plan (604-614 in its numbering; implementation numbering will shift).
- Its tables are under `.temp/brief-603/`; earlier rounds' under
  `.temp/brief-600/` and `.temp/brief-601/`; your own review is
  `docs/dev/codex/brief-602-g13-design-review.md`.

## What to review

Answer each with `path:line` evidence from the source or the tables, and say
whether it is verified in source or inferred.

1. **Your seven required changes.** For each, is it met? Where not, what is
   missing.
2. **`ContinueToken` and state.** The token stores `Arc<F>` and its `suspend`
   takes no frame value. The prepared-call frames carry the loop's progress
   (`state: SuspendedCallArgumentPreparation`,
   `crates/rum-runtime/src/runtime/eval_call_prepare/resume_frames.rs:4-8`,
   `:34-38`). Does re-suspension through the token resume from the right
   progress? If not, what form should the token take so the frame type (and so
   its `Output`) stays bound while the state is new?
3. **Visibility and sealing in Rust.** The erased layer becomes `pub(crate)`,
   `ContinuationState` wraps module-private runtime types, and `SlotValue` is
   public with methods mentioning `FrameOutcome`. Does the proposed surface
   compile under Rust's private-in-public rules and still let `rum-stdlib`
   frames name their `Input`/`Output` and implement `TypedFrame`? If not,
   what arrangement does (for example conversions on the private sealed
   supertrait)?
4. **The remaining declared bindings.** Three side-effect inserts keep a
   declared `Output` (`insert_typed_frame`), one of them
   `DirectDefaultFallbackFrame` with a non-trivial output. Can each be bound
   by the compiler instead?
5. **Transformed results.** 29 macro sites transform `$result` before the
   macro; the witness then binds the transformed type. Where the transform
   keeps the type (`Expr → Expr`), nothing fails to compile but the resumed
   path skips the transform. Is the design's handling (audit per family
   brief) adequate?
6. **The closed enum.** Is the crossing population (35 types, all
   runtime-owned or primitive) established? Is the enum's extension policy
   and its ownership/inventory consequence stated honestly?
7. **The step plan.** Can each step compile and land green on its own, in the
   order given, with every crossing type flipped SCC-complete? Are the
   performance gates placed on every step that changes a hot evaluator
   function?
8. Anything else that is wrong, missing or over-scoped.

## Verdict

End with one of: **accept**, **accept with required changes** (list them
precisely enough to fold into implementation briefs), or **reject** (say why
and what instead). Rank findings by severity.

## Scope

- Files you MAY modify: **NONE**, except: append your review to this file;
  scratch notes under `.temp/brief-604/`.
- No scratch test, no Cargo command, no probe of any kind.

## Hard rules

- **No `git`, in any form.**
- Do NOT modify any file outside the exceptions; STOP and report instead.
- No network; nothing outside this repository.
- Commands: file reads, `rg`/`grep`, `python3` over the `.temp` tables
  (through `python3 scripts/run-bounded.py --timeout 300 -- …`).
- **Testing Policy is a hard line** (`AGENTS.md`).

## Report format (append to this file)

1. Findings, ranked by severity, each with evidence and verified/inferred.
2. Answers to questions 1-8.
3. Verdict and the required changes.

---
