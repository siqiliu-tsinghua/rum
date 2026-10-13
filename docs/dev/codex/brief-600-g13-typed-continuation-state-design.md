# brief-600 — design: G13, typed continuation state crosses frames in-band

`Scope: NONE — no production change.` Design-only, with one declared scratch
test (see Scope). **Do not run `git` in any form**, including before you have
read this brief.

## Goal

The suspension-continuation branch closed G1-G12 and E1 (briefs 548-599): every
declared guest-effect entry of the installed catalog has an executed descriptor
in the rum-vs-rum oracle (resumed path == queued-input path, suspending in the
middle and twice, lifecycle at both pauses). brief-597's sweep found **G13**,
which is not a lost continuation but a flaw in the channel continuations use.
This brief designs its repair. The repair must be structural — the compiler
rejects a wrongly typed completion, and guest code cannot construct anything a
decoder accepts as typed state — and its relation to E2 (a may-suspend result
type with no `Try`) is part of the design.

## The problem (verified by the director at the 597 gate)

- Frames complete with `Result<Expr, EvalError>` (`FrameAction::Complete`,
  `crates/rum-runtime/src/runtime/suspension/core.rs:100-135`). A function
  returning `Result<T, EvalError>` that suspends inserts a frame whose
  completed `Expr` the caller's frame decodes with `T::from_continuation_expr`
  (`ContinuationValue`, `core.rs:137-141`; 25 impls) or with an ad-hoc decoder
  such as `prepared_call_arguments_from_continuation_expr`
  (`eval_call_prepare.rs:145-185`). Production code uses 206 distinct
  ``System`Internal*`` heads.
- The heads are ordinary ``System`` symbols matched by text
  (`is_system_symbol_name`, `expr_runtime_helpers.rs:420`): **guest source can
  write them.**
- The decoders are lenient — `Option<Expr>` reads any unmarked value as `Some`
  (`suspension/primitive.rs:36-66`), `Vec<Expr>` as a one-element vector
  (`:68-83`), `bool` anything but `True` as false (`:13-21`), `String` via
  `to_string` (`:23-34`) — or they `panic!`/`expect`/`assert!` on an
  unexpected shape (`primitive.rs:85-153`; `eval_call_prepare.rs:155-185`;
  `eval_core/call.rs:322-323`; many sites in `error.rs`,
  `pattern_matching/continuation.rs`,
  `scoping/lexical/block/continuation.rs`).
- Nothing checks that a slot is filled by its own type's encoder. brief-597
  found three frames completing an `Option<Expr>` slot with a raw value:
  `FinishCasesRuleMatchSuspensionFrame` (`cases/collect/resume_frames.rs:295`),
  `TryApplyFunctionSuspensionFrame` (`scoping/callable.rs:141`),
  `FinishPositionalPartSelectorEvaluationSuspensionFrame`
  (`part_assignment/resume_frames.rs:5`). The lenient decoder is why ordinary
  values never showed it.
- Director-reproduced witnesses (resumed against queued;
  `.temp/brief-597/sweep.rs`, `.temp/director/gate-597/g13-witness.txt`):
  a guest value ``System`InternalContinuationNone[]`` is decoded as `None` by
  `resume_suspended_call_dispatch_frame` (`eval_core/call.rs:291-338`), which
  moves to the next dispatch stage, and the resumed call asks for input a
  second time; ``Function[(InputString["sweep"];System`InternalContinuationSome[5])][0]``
  resumes to `5` instead of `InternalContinuationSome[5]`; with
  `InternalContinuationSome[1,2]` a debug build panics at
  `suspension/primitive.rs:51` (a `debug_assert`; release would return `1` —
  inferred, not run).

## What the design must settle

1. **Inventory.** Every encoder/decoder pair and every decode site, with its
   behaviour on (a) a raw ordinary value, (b) another type's marker, (c) a
   malformed marker of its own type: lenient / error / panic. For every
   decoded slot, every frame or finalizer that fills it, with a verdict:
   encodes with the slot's encoder / completes raw / completes with another
   type's encoding. Classify all ``System`Internal*`` heads: continuation-state
   markers versus other internal uses (which are out of scope unless they are
   decoded as typed state). Recount the `SuspensionFrame` and
   `PureFrameFinalizer` implementors, the `ContinuationValue` impls and the
   ad-hoc decoders; do not rely on the numbers above.
2. **Reachability, by source reading.** For each lenient or panicking decoder:
   can guest code deliver a forged shape to it, and by which path? Also every
   path by which an encoded value reaches guest-visible state (a result,
   `Sow`/`Reap`, a trace, a message, a definition), where it could be captured
   and replayed. **Executable witnesses only for value divergence** (resumed
   against queued, no panic), in the scratch test. For a decoder that would
   panic, give the guest program and the path; **do not run it** — the
   director reproduces those.
3. **The repair design**, meeting three requirements:
   - (i) a frame cannot complete a slot with the wrong type — **a compile
     error, not a convention**;
   - (ii) guest code cannot construct anything a decoder accepts as typed
     state — **out of band**, not a renamed or "sealed" symbol;
   - (iii) decoding typed state cannot panic or misread on guest input; any
     residual runtime check is an internal invariant that only a kernel bug
     can violate, stated with why guest input cannot reach it.

   Compare, with evidence, at least: a typed output bound at insertion (the
   frame names the type it completes; the insertion point in a function
   returning `Result<T, _>` accepts only frames completing `T`; the driver
   adapts at the type-erased queue); a typed out-of-band payload carried by
   `FrameAction::Complete` / the resume outcome; and sealed tag identities —
   stating what those do not guarantee (a raw value of the wrong type; replay
   of a leaked encoding). Say where guest values legitimately travel as `Expr`
   and must stay `Expr`.
4. **Relation to E2** (`.temp/brief-583/e2-outline.md`: a may-suspend result
   type with no `Try`, typed continue descriptors, explicit output codecs):
   is G13's repair a foundation of E2, independent of it, or subsumed by it?
   Can it be built on today's `Result<T, EvalError>`, or does it need E2's
   type first? Recommend the sequence with evidence. If one migration should
   carry both, say so and lay out its steps; E2's representation choice still
   requires its measured prototype, so do not decide that here.
5. **A class-wide falsifier**, rum against rum, so G13's class cannot return:
   for example an oracle variant in which a callback or body returns each
   continuation-state marker shape and the resumed path must equal the queued
   path. Say which descriptors it applies to, that it is red today on the
   three frames, and what it checks once markers no longer exist in guest
   space.
6. **Ripple and cost.** The full ripple — `SuspensionFrame` implementors,
   `FrameAction`, `PureFrameFinalizer`, the resume driver, `kernel_api`
   exports, `rum-derive`, the frame-root trace inventory, the ownership and
   retention inventories and fingerprints, the oracle, the existing
   continuation suites. Which ordinary-path functions change shape (the frame
   is built only on suspension; anything that touches a hot return path is
   named), and the one-codegen-unit instruction gate plan: labels, `N`/`2N`,
   build both arms at the same path.
7. **Drafted implementation briefs** in `TEMPLATE.md` shape, one per gated
   step, small enough that each step lands green; the three witnesses and any
   new value-divergence witnesses become reds; each step keeps the oracle
   green in both builds.

## What I want back

1. The inventory tables (`.temp/brief-600/*.tsv`) and their counts.
2. The reachability table, the executed value-divergence witnesses with
   output, and the not-run panic paths for the director.
3. The repair design with the comparison, the E2 relation and sequence, the
   falsifier, the ripple and the gate plan.
4. The drafted implementation briefs.
5. Open questions for the director.

## Context pointers

- brief-597's report and sweep (`docs/dev/codex/brief-597-*.md`,
  `.temp/brief-597/`), `.temp/director/gate-597/g13-witness.txt`.
- `crates/rum-runtime/src/runtime/suspension/`, the frames named above, the
  resume driver, `rum-derive`'s frame derives, `kernel_api`.
- `.temp/brief-583/e2-outline.md`, `.temp/brief-548/enforcement.md`.
- The oracle: `crates/rum-stdlib/tests/suspension_oracle/`.
- `docs/dev/codex/TEMPLATE.md`, `AGENTS.md`,
  `docs/code-organization-guidelines.md`.

## Scope

- Files you MAY modify: **NONE**, except: append your report to this file;
  scratch under `.temp/brief-600/`; **one** scratch integration test,
  `crates/rum-stdlib/tests/zz_brief600_probe.rs`, **deleted before the report
  is appended**, its final content copied to `.temp/brief-600/probe.rs`.
- Everything else is read-only.

## Hard rules

- **No `git`, in any form.**
- Do NOT modify any file outside the exceptions; STOP and report instead.
- No network; nothing outside this repository; all scratch and `TMPDIR` under
  `.temp/brief-600/`.
- **Run no probe that panics or aborts.** A probe that would reach a panic is
  reported, not run.
- Commands, serial, each through `python3 scripts/run-bounded.py`:
  `env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --test zz_brief600_probe`
  (300 s); `python3` scripts (300 s each). File reads, `rg`/`grep`.
- **Testing Policy is a hard line** (`AGENTS.md`): rum against rum only.
- **No production code.** No new `unsafe`.
- Checkpoint to `.temp/brief-600/PROGRESS.md` after each section with the
  exact next action.

## Report format (your final message, appended to this file)

1. What changed — "design only", scratch, the deleted test.
2. The inventory and the reachability.
3. The repair design, the E2 relation and sequence.
4. The falsifier, the ripple and the gate plan.
5. The drafted implementation briefs.
6. Open questions for the director.

---
