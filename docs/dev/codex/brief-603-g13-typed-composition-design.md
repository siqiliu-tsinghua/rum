# brief-603 — design round 3: G13 as a typed composition boundary

`Scope: NONE — no production change.` Design-only, with one declared scratch
test (see Scope). **Do not run `git` in any form**, including before you have
read this brief.

## Goal

G13: typed continuation state crosses suspension frames in-band, as `Expr`
under ``System`InternalContinuation*`` heads guest source can write, decoded
leniently or by panic, and nothing checks that a frame completes the type its
consumer decodes (six frames complete a guest value raw into an `Option<Expr>`
slot; 21 executed divergences). Two design rounds produced a direction that is
accepted and a migration that is not:

- brief-600 (inventory, first design) and brief-601 (revision: out-of-band
  channel, typed frames, output bound at the enclosing return position).
- brief-602, an independent review by another model, **rejected the revision
  as ready for implementation** with nine findings (F1-F9) and seven required
  changes. The director verified its counterexamples in source and adopts the
  review. Read `docs/dev/codex/brief-602-g13-design-review.md` §1-§3 in full;
  it is the core input of this brief.
- The director's own verdict, written before the review:
  `.temp/director/gate-601/director-verdict.md`. Where it and brief-602
  overlap, brief-602's formulation governs.

This round designs G13's repair as **a typed composition boundary** — checked
adjacency between frames, not only a local binding at each insertion — and an
independently green migration to it.

## Settled (do not reopen)

- The out-of-band channel: `Expr` is the only type that travels as a guest
  value; typed state is a Rust value guest source cannot name. Sealed or renamed
  symbols are rejected.
- A slot mismatch is a classified error in every build, never a panic.
- One input and one output type per frame; the dispatch-plan frames split per
  stage.
- The rum-vs-rum `marker-corpus` oracle falsifier, without panic shapes.
- The end state deletes every continuation marker, the lenient decoders,
  `ContinuationValue`, and any unrestricted insertion escape.

## Director rulings

1. **Adopt brief-602's required changes 1-7.** Each is a section of this
   design (below).
2. **Typed composition belongs to G13, not to E2.** Whatever propagation G13's
   migration depends on to work at all (F2, F6) is designed and migrated here.
   E2 keeps only what remains after G13: making a bare `?` on a may-suspend
   result a compile error, and the result representation its measured
   prototype chooses. State that boundary exactly.
3. **brief-601's open questions:** (1) a loud classified error, through a
   **named** `EvalError` variant you specify with its cleanup and propagation
   policy (F9); (2) the styled-message frame gets the explicit typed body;
   (3) corpus positions are tested **each separately** and per shape, with
   explicit per-descriptor inclusion/exclusion; (4) splitting a migration by
   crate is allowed only with a temporary legacy path that is enumerated and
   scheduled for deletion; (5) the `()`-carrier edges are resolved here if
   G13's migration depends on them, otherwise handed to E2 with a reason.
4. **Input binding** uses a zero-sized invariant witness of the original
   `Result`'s Ok type where that `Result` is available (F7); synthetic-`Err`
   helpers carry the producer's type evidence in, or are redesigned.
5. **Performance:** any step that changes a hot evaluator function carries the
   per-label microbench A/B (`AGENTS.md:46-48`, `:117-121`), with one-codegen-unit
   instruction slopes as a supplement; profiling-record obligations compare
   record identity, count, classification and balanced stack depth, not
   elapsed values (F9).

## What the design must settle

1. **The typed composition boundary** (required change 1). An opaque typed
   frame/action layer whose public surface cannot complete a wrong type:
   checked `Input` and `Output`; checked adjacency between consecutive frames;
   Route E (`Evaluate`) preserving the parent's output (`next.Output ==
   parent.Output`); finalization and conversion that survive a repeated
   suspension; erased queue and action construction private to the driver and
   its adapters; a public typed trait separate from the erased driver trait.
   **Deferred completion obligations** (F2): distinguish a helper's ordinary
   result from the obligation its frame discharges — the prepared-call
   finalization (`eval_call_prepare.rs:231-436`,
   `eval_call_prepare/resume_frames.rs:26-79`, `eval_core/call.rs:301`) and the
   lazy `First`/`Normal` captures (`structure_ops/surface/lazy_completion.rs`,
   `associations/basic/lazy_completion.rs`) are the two verified examples; find
   the rest of their families and say, for each, where the obligation is
   created and how its type is checked there. **Chained inserts** (F4,
   `pattern_matching_interpreted.rs:92-104` and its siblings,
   `replace/once.rs:183`): composed, adjacent types checked, returned once.
2. **Sealed conversion and the slot population** (required change 2). Only
   `Expr` uses the value arm; every state carrier uses the state arm by exact
   Rust identity; implementors choose a type, never an arm. Settle the actual
   **crossing** payload types (not stored frame fields — F8) and compare a
   closed enum with controlled erasure on that set. State the downstream
   extension policy and the honest trust boundary for arbitrary `Expr -> Expr`
   Rust implementations.
3. **The inventory, repaired** (required change 3): nested macro tokens
   (`confirmation/builtins.rs:406`, `replace/once.rs:212` were missed),
   closure return scopes, owner-qualified method identities (the 51-edge table
   dropped every method), non-`Result` carriers
   (`evaluated_part_selectors.rs:133-149`), chained inserts, synthetic-`Err`
   helpers, and every producer → conversion → consumer edge with its
   resuspension behaviour and migration disposition. Unresolved edges are
   listed as unresolved. Witnesses must exercise the edge they are cited for
   (F6: the string-guard witness had no guard; the Prime witness was the wrong
   edge).
4. **The verified counterexamples resolved** (required change 4): the
   prepared-argument finalization and the lazy captures, plus their analogous
   families, before their payloads move out of band.
5. **An independently green step plan** (required change 5). A first step of
   representation scaffolding may keep legacy control flow and makes **no
   enforcement claim**; every later step migrates a complete connected
   producer/consumer group; any temporary adapter is enumerated with the step
   that deletes it; the final state has no legacy decoder and no unrestricted
   insertion. Show the producer/consumer groups and their order.
6. **Ownership, transit and the adapter** (required change 6): safe consuming
   downcast (no `unsafe`); the adapter's ownership model without a `Clone`
   requirement or a uniqueness assumption, preserving drop-before-finalizer;
   delegation of `FrameRootTrace`, abandonment **and `output_boundary`**
   (`debug_tools/trace/resume_frames.rs:12`, `suspension/output.rs:51`,
   `suspension/core.rs:671`); the outcome's real lifetime across driver
   iterations (`core.rs:723-747`) and its rooting, accounting and drop;
   mismatch cleanup; `Send`/`'static` bounds. Ownership and retention
   inventory consequences stated as predictions, with how each implementation
   step verifies them; the `Expr` destruction debt stays acknowledged
   (`host_payload_ledger.tsv:53-55`).
7. **Gates** (required change 7): compile-fail and compile-pass controls for a
   wrong `Input`, a wrong `Output`, a wrong `Evaluate` output, a wrong
   finalizer output, generic and closure sites, direct insertion and an
   external codec implementation (this repository proves `compile_fail` with
   doctests — use the existing pattern); both feature builds' oracle and
   lifecycle; justified corpus coverage; an explicit assertion that no slot
   mismatch occurred, including where guest error handling would consume it;
   microbench A/B and instruction slopes per ruling 5.

## What I want back

1. The repaired inventory tables (`.temp/brief-603/*.tsv`), with counts that
   reconcile against a raw source scan, and witnesses tied to their edges.
2. The typed composition design (sections 1, 2, 4, 6) with Rust signatures,
   the sealing mechanism, and why each bound is sound — including the cases
   brief-602 broke the last design with.
3. The E2 boundary (ruling 2).
4. The step plan as producer/consumer groups, and redrafted implementation
   briefs in `TEMPLATE.md` shape, one per gated step, each with its gates.
5. Open questions for the director.

## Context pointers

- brief-600, brief-601 and brief-602 with their reports; `.temp/brief-600/`,
  `.temp/brief-601/` (tables, scripts, `probe.rs`);
  `.temp/director/gate-600/`, `.temp/director/gate-601/`.
- `crates/rum-runtime/src/runtime/suspension/`, `kernel_api.rs`,
  `rum-derive`, `runtime/tests/frame_root_trace*`.
- `crates/rum-stdlib/src/tests/ownership_graph/` and its ledger; the retention
  inventories; the oracle under `crates/rum-stdlib/tests/suspension_oracle/`.
- `.temp/brief-583/` (`e2-outline.md`, census tooling,
  `may-functions.json`), `.temp/brief-548/enforcement.md`.
- `docs/dev/codex/TEMPLATE.md`, `AGENTS.md`,
  `docs/code-organization-guidelines.md`.

## Scope

- Files you MAY modify: **NONE**, except: append your report to this file;
  scratch under `.temp/brief-603/`; **one** scratch integration test,
  `crates/rum-stdlib/tests/zz_brief603_probe.rs`, **deleted before the report
  is appended**, its final content copied to `.temp/brief-603/probe.rs`.
- Everything else is read-only — including `.temp/brief-600/` and
  `.temp/brief-601/`, whose files you copy rather than edit.
- This worker has no OS-level write fence: these exceptions are the whole
  fence. Write nothing outside this repository.

## Hard rules

- **No `git`, in any form.**
- Do NOT modify any file outside the exceptions; STOP and report instead.
- No network; nothing outside this repository; all scratch and `TMPDIR` under
  `.temp/brief-603/`.
- **Run no probe that panics or aborts.** A probe that would reach a panic is
  reported, not run.
- Commands, serial, each through `python3 scripts/run-bounded.py`:
  `env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --test zz_brief603_probe`
  (900 s); `cargo test -p rum-stdlib --lib ownership_graph` (900 s) if you
  need its current output; `python3` scripts (300 s each). File reads,
  `rg`/`grep`.
- **Testing Policy is a hard line** (`AGENTS.md`): rum against rum only.
- **No production code.** No new `unsafe`.
- Checkpoint to `.temp/brief-603/PROGRESS.md` after each section with the
  exact next action.

## Report format (your final message, appended to this file)

1. What changed — "design only", scratch, the deleted test.
2. The repaired inventory and witnesses.
3. The typed composition design and the resolved counterexamples.
4. Sealing, the slot population and the trust boundary; ownership and transit.
5. The E2 boundary; the gates.
6. The step plan and the redrafted implementation briefs.
7. Open questions for the director.

---
