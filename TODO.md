# TODO

This file is for active or near-term work only. Completed history and
operational lessons live in [PROGRESS.md](./PROGRESS.md) (append-only log),
and release-facing changes live in [CHANGELOG.md](./CHANGELOG.md).

Status checkpoint: 2026-09-05, committed tree through `bd172345`. Uncommitted
performance candidates do not count as completed until their correctness,
timing, counter, and memory gates have all been reviewed.

## Active — post-v0.1 algorithmic audit

The P0 tranche and all of P1 are done. Two independent re-surveys (one by each
worker) then produced a worklist of about thirty items, grouped into batches;
the 2026-08-26 entry of [PROGRESS.md](./PROGRESS.md) records what each batch
found and closed.

**Closed since the last checkpoint**, in order:

- Loop and `Module`-initializer drivers made iterative. A 20 000-iteration
  `While` terminated the process; ordinary loops are ordinary code, so this was
  a correctness defect before it was anything else.
- YAML and TOML decoders bounded, matching `FromJson`. All three codecs now
  accept exactly 256 nested collections and refuse the 257th with a message
  naming the limit and the byte offset.
- Left-associative chain lowering, five steps (plan-330). All sixteen affected
  operators overflowed the stack at 20 000 links; they now fold iteratively,
  nine of them into n-ary calls, with a 4 096-link ceiling proved on a 2 MiB
  embedding stack through the complete round trip.
- Resource budgets made symmetric (plan in briefs 336-338). Padding builtins,
  `Fibonacci`, and multi-iterator products went uncharged while their siblings
  charged the same work. A regression this introduced — sequential repetition
  charged as simultaneous residency — was found and fixed in the same batch.
- User-callback ordering (plan-339). `Sort`, `SortBy` and `ReverseSort` with a
  comparator were quadratic on every input; `KeySort` and `KeySortBy`
  overflowed the stack at 110 and 100 keys. All five now run on one iterative
  resumable merge engine.
- The microbench itself never closed stdin, so a run either finished in four
  seconds or burned its timeout and produced a header-only timings file. Every
  A/B before `dd4bdefc` was exposed to that.

**Not briefed, and why:** the surveyed "Association 4-to-5" and "65 536-key"
cliffs no longer exist. Measured across eight operations, every 4-to-5 ratio
falls between 1.02x and 1.41x and the 65 535/65 536/65 537 boundary is flat.
They were closed by `c8d34876` and `dc5f6698` earlier in this sweep; the
worklist was simply stale. Association key lookup remains a linear scan, which
is a standing decision, not a defect.

**Re-probed 2026-08-26, all still present.** Numbers are from the release CLI
at `c18b01f9`, each probe carrying a liveness assertion:

1. **`ValueQ` clones the whole session per call.** 4.6 us with no definitions,
   90.6 us at 2 000, 333 us at 8 000, and 1.94 ms at 32 000 — a cost that
   depends on session history, not on the question asked. Confirmed against
   the wall clock: 100 calls in a 32 000-definition session move the process
   from 0.10 s to 0.30 s. A 420x spread that ends at two milliseconds is worth
   fixing cheaply, not worth a redesign.
2. **`Part` / `Extract` materialized every association entry per call — FIXED
   `4e045135`** (briefs 357-358). Per read: positional 12.61 ms -> 1.00 us at
   32 000 keys and exactly flat across width; keyed 12.27 ms -> 10.36 us.
   Allocations per read are now width-invariant (14.03 at both 4 000 and
   32 000 keys, against 12 083 and 96 101 before).

   **Leftover, deliberate:** keyed reads are sublinear rather than flat —
   3.7x for 8x width — because a wide association does not always hold the
   read-side index under cache admission pressure. That policy (`MAX_KEYS`,
   the retained-entry budget) is a standing decision and was explicitly out of
   scope. Revisit it only with the memory tradeoff stated, since the surveyed
   "65 536-key cliff" was already judged a fossil.
3. **`StringTrim` with a literal *list* leaves the fast path.** At 8 000
   characters: 342 us for `{"1", "2"}` against 28 us for the single literal
   `"1"` and 21 us for whitespace, and 1.45 ms against 77 us at 32 000. Linear,
   not quadratic — the defect is a 12x-to-18x constant, so the surveyed
   "quadratic suffix helper" reading is wrong even though the target is real.
4. **Symbol ordering allocates two `String`s per comparison**
   (`ordering_sets.rs:875`), 6.7x/7.0x/14.2x slower than integer ordering at
   4k/8k/16k. Not a mechanical swap: `Symbol::cmp` compares context then name,
   which is not the lexicographic order of the joined qualified name, so a
   repair has to argue output equivalence.
5. REPL re-parsing, the Jupyter busy-spin, and the gensym sweep — **all FIXED**
   (`21569ddb`, `3732615a`). The REPL examines only appended text and runs the
   full check once per form rather than once per line; the kernel sleeps its
   launcher's 10 ms idle interval instead of yielding; the sweep rearms at a
   watermark, so per-batch cost stops climbing (base +106% by the third batch,
   candidate -11%).

**Batch 7 is closed.** Every surveyed item was re-probed, and each was either
fixed or recorded with its reason. Two defects found along the way outrank
what the batch was looking for, and both have their own entries above:
`Do[f[i] = i, …]` installing one raw definition, and rule right-hand-side
`Condition` guards never being evaluated.

## Next up — scheduled 2026-08-28

Ordered after re-probing everything below. Silent defects outrank loud ones,
and the instrument outranks nothing here because the instrument turned out to
be fixed already.

1. **The assignment-target normalization family.** Planned by brief-376,
   which re-probed all three defects, widened each, and returned two repair
   families rather than one or three. **Dispatch order: brief-377 (family A)
   first, then family B in three steps.**

   - **Family A — evaluated `Part` selectors — COMPLETE 2026-08-28**
     (brief-377, `636e01ee`). D2 plus the `Part` branches of
     `AssociateTo`/`KeyDropFrom`: thirteen mutation heads, not eleven. The
     repair is a typed `EvaluatedPartSelectors` that assignment-target parsers
     require, so a future head cannot pass raw selectors without a compile
     error; the only constructors are exactly-once evaluation and a
     stable-positive-literal proof that keeps `AppendTo`'s incremental path
     zero-callback. Plain `Set` and association `Set` were routed through the
     same driver, removing their duplicated accumulators. The callback and
     return-order contract is now in `docs/guide/evaluation.md`.

     Gate note for family B, which touches the same files: the A/B run carried
     a **+1.81% whole-run offset** measured on the 102 untouched control
     labels, against +1.02% median on the ten affected ones. The raw split was
     86-of-112 labels slower, which reads as a regression until the controls
     are consulted. Name the controls before judging.

     Watch, not yet actionable: `association_data.rs` is now 32.2k and
     `definition_updates.rs` 26.3k, both under the 35k architecture-debt line
     but rising, and family B touches both. Evaluate a split when family B
     closes — not mid-repair, where it would contaminate a semantic change.
   - **Family B — definition-lhs lookup identity.** Planned by brief-378,
     which re-probed all four recorded defects and **found three more of the
     same family**, all reproduced by the director at `dfedb317`:

     - `Unset[f[i]]` with `i = 2` does not remove the definition `Set` just
       stored under `f[2]` — removal compares the raw lhs. The mirror image of
       B3/B4, and proof the invariant has to be enforced on the removal side
       too.
     - Value-table assignment (`DownValues[f] = {…}`) stores raw rule
       left-hand sides, so active positions become unreachable.
     - A held ground nested `Flat` call stores `fHF[hFlat[1, hFlat[2, 3]]]`,
       which **neither** `fHF[hFlat[1, 2, 3]]` nor the source form reaches.
       Candidate matching normalizes `Flat`/`Orderless` structurally while the
       ground index does not, so store identity needs match normalization on
       top of hold-aware evaluation.

     Seven defects, one preparation. Brief-378 also settled, with evidence:
     `SequenceHold` is not a hold attribute (it only prevents splicing);
     `HoldAllComplete` additionally blocks `Evaluate` and splicing;
     `Unevaluated` contributes its stripped inner expression to identity; and
     `HoldAllComplete`'s upvalue suppression is documented behavior, not a
     defect. **No existing test encodes any of the wrong outcomes**, so nothing
     has to be deleted or weakened.

     **Dispatch order (director, splitting plan-378's oversized step 1):**
     1a foundation — **COMPLETE 2026-08-28** (brief-379, `5698e8e5`);
     1b preparation plus `Set`/`SetDelayed` — **COMPLETE 2026-08-28** (brief-380, `aa441ff0`); 1c `Unset`
     and value tables — **COMPLETE 2026-08-28** (brief-381, `45c5aa33`);
     2 the association fallback — **COMPLETE 2026-08-28** (brief-382,
     `e90fcc62`); 3 tagged definitions,
     `TagUnset`, and final sealing — **COMPLETE 2026-08-28** (brief-383,
     `37a12479`). **The whole family is closed.** **Four steps, not the six first sketched**:
     plan-378's specification makes `Flat`/`Orderless` identity normalization
     step 7 of the preparation pipeline rather than a separate transform, so
     the held-ground defect closes in 1b alongside `Set`. The split is for
     gateability, not size: each step has to be independently verifiable and
     revertible, and 1a's criterion — an empty rum-vs-rum differential diff —
     only works while no consumer has moved.

     **Close-out.** Seven defects, four dispatches, and the repair is
     enforced rather than maintained: store and removal travel as tokens whose
     constructors are private to one child module, so a raw lhs from
     source-language evaluation is not expressible. Two non-preparing paths
     remain by design — static catalog installation and whole-state
     restoration. Both guarantees were calibrated, not assumed: forging a
     token from another module fails to compile (E0603/E0624), and adding one
     call to the trusted path makes the inventory test fail with the exact
     count. `definition_tags.rs` and its neighbours lost 232 lines net.

     **Open observation, no structural explanation, deliberately not fixed.**
     Step 3 moved `downvalue-guarded-decline-fallback` +7.47% and
     `downvalue-guarded-hit` +6.01%, above the 0.4-4.1% spread measured
     between same-semantics binaries differing only in layout — while the
     three hot lookup functions' disassembly is instruction-identical with
     addresses shifted `+0xa0`. The layout estimate is itself incomplete: the
     inert arms add code where this change removes 232 lines net, so they
     perturb in the opposite direction. Overall the run is clean
     (pure-computation controls +1.33%, everything else +0.74%). Revisit only
     with a profile that explains the mechanism; do not tune blind.

     **Two things 1a established for the steps that follow.**

     - *A differential diff must be calibrated before it is trusted.* Run
       against a pre-family-A binary the same probe reports a non-empty diff,
       messages included; an empty result therefore means something. Every
       later step reusing this gate must re-run that calibration, because a
       scanner that reports clean has to be shown capable of finding something
       — the frame-roots line lost a round to a regex silently truncated by a
       comma.
     - *To tell a layout effect from an algorithmic regression, build a third
       binary that changes only the layout.* Step 2 left two labels looking
       like regressions with non-overlapping three-run ranges
       (`downvalue-exact-miss` +4.94%, `downvalue-pattern-miss` +7.63%), while
       the affected function's disassembly was instruction-identical between
       binaries. An arm carrying nothing but an inert `#[inline(never)]` dead
       function in an unrelated file moved those same labels +2.60% and
       +5.10% — further than the real change did — which settles it: those
       rows are layout-sensitive and carry no information about the change.
       One extra build, and far more conclusive than arguing from medians.
     - *Best-of-2 measures a spread four to five times too small.* 1c's worker
       left two labels open as possible regressions —
       `matchq-integer-success`/`fail` at +6.34%/+6.72% against best-of-2
       spreads of 2.01%/2.96%. At best-of-3 those same labels' spreads are
       10.6% and 11.3%, and both movements fall inside. Nothing in 1c reaches
       `MatchQ`. Run three, report the range, and treat a two-run spread as a
       single random sample rather than a noise floor.
     - *Whether valid controls exist depends on the change's reach — do not
       generalize either way.* 1a sat on the common call path, where nothing
       is untouched. 1b sat on the assignment path, where the 28
       pure-computation labels are a genuine control group — and they decided
       the gate: they moved +3.82% at the median against the definition
       path's +2.10%, so the affected labels moved *less* than the controls.
       Judging 1b by 1a's rule (each label against its own spread, no
       controls) would have left three labels looking like regressions.
       Decide per step which of the two applies, and say which in the brief.
     - *Named control labels stop working once the change is on the common
       call path.* 1a's four controls were all fast labels from one tier and
       moved as a group (median -7.16% against a suite median of -1.83%),
       which would read as "targets are 4.5% slower" if taken as the baseline.
       `sameq-integer` and `string-repeat-small` themselves run through call
       preparation. For steps touching the shared path, judge on each label's
       own same-build spread and the suite distribution instead, and say so in
       the brief rather than naming controls that cannot be untouched.

     **Flake, first occurrence, recorded not explained away.** One workspace
     run during brief-379 failed
     `runtime::tests::association_write_index::repeated_replacement_writes_stay_indexed`
     (`association_write_index.rs:146`); a focused rerun and both later full
     runs passed, and the director's two full runs were green. The worker did
     not state whether it had changed code between the failure and the rerun,
     so flake versus fixed-in-passing is unresolved. Worth noting: this test
     asserts pointer identity (`own_args_ptr_key`) and lookup counters, and
     the other open flake — `has_valid_call_stability_proof_for_key`,
     `evaluation_cache/tests.rs:374` — asserts a stability proof in the same
     association write path. Two identity-shaped assertions in one area may be
     one cause. Same standing agreement: brief on a third occurrence.

   **The contract, settled 2026-08-28.** A definition's lhs arguments are
   prepared according to the head's hold attributes: non-held positions are
   fully evaluated, including a delayed own-value; held positions keep their
   raw symbols. **The invariant is bidirectional — the identity a definition is
   stored under and the identity a lookup constructs must come from the same
   preparation.** rum currently violates it in both directions, which is why
   family B covers both:

   - Not normalizing where it should: `AssociateTo[af[ii], …]` storing
     `af[ii]`, `TagSet`/`UpSet` storing a raw lhs, and `fD[iD] = 8` with
     `iD := 1 + 1` storing `HoldPattern[fD[1 + 1]]` so `fD[2]` cannot reach it.
   - Normalizing where it should not: with `SetAttributes[fH, HoldFirst]` and
     `xH = 5`, `fH[xH] = 9` stores `HoldPattern[fH[5]]`, so the natural
     `fH[xH]` lookup misses it. This one is the opposite direction, and
     covering only the first direction would fix three defects and hide it —
     the same shape as the `Block` frame's bare `Symbol` in the frame-roots
     line.

   `TagUnset` is in family B's ripple: if tagged definitions normalize, removal
   must use the same preparation.

   **Ruling (user, 2026-08-28): keep rum's `UpSet::nosym`.** Once `m[ww] ^= 22`
   with `ww = 5` normalizes to `m[5]`, no symbol in the lhs can serve as a tag,
   and rum already reports exactly that for `m2[7] ^= 33`. The wider WL family
   reaches a differently-shaped diagnostic there; rum keeps the more direct one
   and stays internally consistent, as it did when it kept its precise
   `ValueQ[f[2]]` answer. Design input only — no engine comparison enters the
   repository.

   **Retired by evidence: the "raw rules become reachable after `Clear`"
   argument.** Plan-376 recorded it as a reason a repair must diagnose rather
   than refuse. It describes rum's own raw-store behavior, not a semantics
   worth keeping: under the settled contract the rule is stored ready for
   lookup and is reachable immediately. The "preserve raw plus a warning"
   option, and the diagnostic question that depended on it, are both closed.
2. **The `rum-jupyter` ZeroMQ ordering flake — CLOSED 2026-08-28** (brief-384,
   `5f9a44ab`). Baseline 21 of 30 release runs failing on the director's host,
   0 of 30 after. Diagnosis follows. It reproduces on demand in **release** mode: thirty runs of
   `cargo test -p rum-jupyter zeromq::tests::native --release` fail 22 times,
   always the same single test
   (`native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages`,
   `control.rs:166`, `left: Channel(Shell)` against `right: Channel(Control)`).

   The test sends a shutdown on control, sleeps, sends an execute on shell, and
   asserts the *next* event is `Channel(Control)`. The poller's priority is
   correct — `PriorityKernelSocketPoller` lists `Control` first in every branch
   (`socket.rs:191-229`) — but it chooses among channels **already pending**,
   and native transport buffers on background workers, so "sent first" is not
   "buffered first". The test asserts a timing property while the
   implementation promises a priority one. It is the only place with this
   shape: `wait_for_next_socket_event` is used exactly once in the suite; the
   other twenty-one waits loop until their expected event and are
   order-insensitive.

   **Two things to carry forward.**

   *The defect was environment-dependent, and a stop-loss demanding a
   reproduction made the work unsatisfiable.* The first dispatch of brief-384
   correctly stopped: 0 failures in 30 runs in the sandbox, against ~70% on the
   director's host, with `--test-threads` ruled out as the variable (6/10,
   3/10, 7/10, 4/10 at 1, 2, 4, 9 threads — serial still fails, so the race is
   inside one test). The fix was to make acceptance a **structural argument** —
   why the assertion cannot fail on either arrival order — with run counts
   demoted to corroboration and the director verifying where it reproduces.
   When a defect may not reproduce for the worker, do not gate the work on
   reproducing it; gate it on an argument, and verify separately.

   *The test gate runs debug, and this defect is nearly invisible there.* Three years of sightings were all
   incidental workspace runs; this line's own six-plus workspace runs never hit
   it. Where a defect is timing-shaped, a release-mode repetition run is a
   different instrument from the debug gate, not a slower version of it.
**Batch 7's performance items are all landed** — `ValueQ` (`e67715f1`),
association `Part`/`Extract` (`4e045135`), `StringTrim` (`ada528c4`), symbol
ordering (`64f6200f`), the gensym sweep (`3732615a`), and the REPL/Jupyter
spin (`21569ddb`). An earlier revision of this section listed them as still
scheduled; that was a stale entry written without checking, the same failure
this line keeps finding elsewhere.

Not scheduled, and why: the call-stability-proof flake stands at one recorded
occurrence against the same three-occurrence agreement, and association keyed
reads are sublinear by a standing cache-admission decision that needs its
memory tradeoff stated before it can be reopened.

## Opened by batch 7, not yet scheduled

- **Gensyms held only by an opaque continuation frame — CLOSED 2026-08-28**
  (briefs 364-375, `d13aa27f` through `4e4c229f`). Four defects, not the two
  originally recorded: the design brief's re-probe added a matcher binding
  table losing its bindings, and a `Block` frame holding a bare `Symbol` that
  *leaked* on the next sweep — the opposite direction from the others, and
  proof that covering only `Expr` would have fixed three and hidden the
  fourth. All four are verified closed against plan-364's original probes,
  which assert the old buggy values and therefore now fail by asserting the
  defect.

  The fix is enforced rather than maintained. `SuspensionFrame: FrameRootTrace`
  makes a missing traversal a compile error, the frame queue is private so
  there is one insertion route, and a repository invariant test bans the
  deleted APIs and any handwritten traversal outside a two-entry allowlist
  (`RandomState`, `SyntaxError` — both structurally incapable of holding a
  root). Frames are `Arc`-rooted in two Session slots so the executing frame
  stays visible to a sweep, and one traversal now feeds both the root walk and
  the memory proxy, which used to charge a flat 16 bytes per frame.

  It also removed ordinary-path waste: a closure captured its state before the
  suspend/resume split, so a width-2003 `Switch` paid 63.8 KiB for a capture it
  never used. That width slope is now constant-size noise.
- **The `rum-jupyter` native ZeroMQ ordering flake has now appeared three
  times** (briefs 165, 242, and a workspace run during brief-362). Each time it
  passes in isolation — three isolated runs and two further workspace runs
  after the third occurrence were all green. The standing agreement was to
  brief it on a third occurrence; it is unrelated to the 10 ms idle sleep,
  which landed after the flake's first two appearances.
- **The call-stability-proof flake — CLOSED 2026-08-31** (briefs 387-388,
  `232ff567`). **All three recorded sightings were one cause.** The repair is
  in the tests, and removes the collision by construction: `StabilityBloom`
  compares its regular and generated lanes independently, so an
  identity-bearing target paired with an Association of `System` symbols and
  integer keys makes `intersects` reduce to `(0 & X) || (Y & 0)` for any
  interner order. Semantic assertions stayed on ordinary symbols; only the
  optimization assertions moved. Verified by sweeping preintern counts 0..256
  at 257/257 for both tests, after first reproducing the failures at 36 and 142
  to calibrate the harness.

  The diagnosis follows, and is worth keeping for the method as much as the
  finding.

  `symbol_stability_bloom_bit` derives one of 128 bits from
  `symbol.interned_id()`, assigned by the process-global `SYMBOL_INTERNER` that
  parallel tests share. In both failures the target's rewritten-dirty bit and
  ``System`Rule``'s dependency bit are the same `0x2000`. A Bloom overlap
  yields `MissDirty`, the proof is not cached, the direct association update
  declines, and the generic renderer allocates new args — which is why one
  sighting shows a missing proof and the other a moved pointer key and rebuilt
  index. Same cause, two downstream symptoms.

  **The flake is now deterministic**, which is the finding worth keeping:
  interning 36 unrelated symbols before the direct test, or 142 before the
  indexed one, fails every time. Neutralizing `StabilityBloom::intersects`
  makes both pass; forcing it true makes both fail harder.

  **The contract, settled from the code: the stability predicate is one-sided.**
  `true`/`Some(proof)` is authoritative; `false`/`None` may be conservative. A
  Bloom collision is a fast-path miss, not an "unstable" verdict — every
  consumer re-does the work rather than trusting a negative, and
  `association_lvalue.rs:53` explicitly anticipates a lost proof and rebuilds.
  **The implementation is correct; both tests assert cache residency and
  pointer identity that the mechanism never promised.** Same shape as the
  ZeroMQ flake closed in brief-384.

  **Director's ruling: pointer and index continuity are an opportunistic
  optimization, not a guaranteed invariant.** The recovery path exists
  precisely because a proof can vanish under a size-cap clear; guaranteeing
  continuity would require exact dependency disambiguation on hot-path
  machinery, and buys nothing semantically since the fallback produces the same
  result. The repair is therefore in the tests — but by making the collision
  impossible by construction, not by relaxing the assertions: a target whose
  dirty lane is structurally disjoint from its proof dependencies. Widening the
  Bloom is rejected outright; it lowers a probability without establishing a
  guarantee.

- **Superseded by the entry above; kept for the sighting record.** One
  workspace run during brief-385 failed
  `association_lvalue_direct_update_keeps_args_identity_and_reinserts_proof`
  again, on `valid_call_stability_proof(&session, &key).is_some()`; it passed
  alone and on the rerun, and brief-385 touched neither `evaluation_cache` nor
  `association_lvalue`. Two of three under the standing agreement. Note also
  the `association_write_index` failure recorded during brief-379, which was
  never resolved as flake-or-fixed and asserts pointer identity in the same
  write path — if that one is the same cause, the count is already three.
  Decide that before treating the next sighting as the first of a new pair.

- **First occurrence of a call-stability-proof flake, recorded not explained
  away.** One `cargo test -p rum-runtime` run during brief-370 failed
  `association_lvalue_direct_update_keeps_args_identity_and_reinserts_proof`
  on its `has_valid_call_stability_proof_for_key` assertion
  (`evaluation_cache/tests.rs:374`). The worker attributed it to the 128-bit
  stability Bloom's sensitivity to parallel symbol-interning order. It did not
  reproduce in thirteen director attempts — eight of the test alone, five of
  the full 854-test parallel suite — so the attribution is neither confirmed
  nor refuted. brief-370 changed neither `evaluation_cache` nor
  `association_lvalue`, so it is pre-existing. Same standing agreement as the
  Jupyter flake: brief it on a third occurrence.

**Re-probe each before briefing it.** Two of this batch's items turned out to
be already fixed, and one survey finding — "`SameTest` never exits early" —
was an artifact of an invalid probe: rum's custom equivalence test is a
positional argument, not a `SameTest` option, so the option form silently
compared nothing. Findings age, and probes lie.

**Rule right-hand-side `Condition` guards — COMPLETE 2026-08-26 (briefs
347-354, `5ab6afc5` through `eb926bff`).** All eight candidate-selecting lanes
evaluate their guards: `DownValues`, `UpValues`, `SubValues`, `DefaultValues`,
generic rewrite rules, `ReplacePart`, string rules, and `OwnValues`. One
substrate, one accept rule (`True` accepts, anything else declines), and a
decline is indistinguishable from the rule not matching. Per-lane ordering
contracts are documented in `docs/guide/patterns-and-rules.md`; a single test
exercises a guarded rule in all eight lanes so a future change cannot break
one silently. The unguarded path takes no new branch, allocation or clone in
any lane.

Known limitation, deliberate: one symbol keeps a single own-value slot, so
multiple guarded own-values do not coexist — a later definition replaces the
earlier one.

**The line's own lesson, worth more than the feature.** After six green gates
a guarded `DownValue` still stopped enforcing its guard the moment anything
bumped the attribute revision — any `SetAttributes` in the session, or a
loader-snapshot restore such as `ValueQ`. Every gate had asserted the guard
*before* the trigger and never after one, and the whole-line regression test
had the same shape, so none of them could see it. Brief-354 fixed it
structurally, by putting the barrier in the single rematch API rather than
patching another call site, and the regression test now asserts after a bump.
**For a feature that caches or compiles anything derived from a definition,
the test has to act after the invalidation, not only before it.**

The original diagnosis follows.

**Rule right-hand-side `Condition` guards are not implemented (brief-347,
diagnosed 2026-08-26).** `f[x_] := rhs /; test` is a standard idiom and rum
silently does something else: the guard is never evaluated, a false guard never
rejects the rule, and the `Condition` leaks into the result — `b[x_] := x^2 /;
x > 0` makes `b[3]` return `9 /; True` and `b[-3]` return `9 /; False`. Dedup
compares left-hand sides only, so a rule differing solely by its guard is
treated as a redefinition of the unguarded one and disappears. `Rule` and
`RuleDelayed` carry the same hole through the replacement system.

Parsing and lowering are correct; `DownValue` and `RewriteRule` simply have no
guard field, so the whole `Condition` is stored as the body and substituted as
one. The guard in *pattern* position works correctly and is the model to
follow. No existing test encodes the wrong behavior.

Every candidate-selecting lane is affected: `DownValues`, `UpValues`,
`SubValues`, `DefaultValues`, `OwnValues`, `ReplaceList`, `ReplacePart`,
string rules, and the exception-handler scan. Director's rulings on the plan:
only exact `True` accepts and anything else declines (matching the pattern-side
guard); nested guards flatten to an ordered sequence evaluated left to right;
a guard shares its body's `OptionValue` scope; identity is syntactic after
alpha-canonicalization, so `Condition[body, True]` is a distinct candidate from
an unguarded rule; `Unset` removes all guard variants of an lhs; a bare
`Condition` expression stays inert; `NValues` and `Dispatch` are out of scope.
**Guarded `OwnValues` are deliberately narrowed**: no storage-representation
change and no `symbol.rs` hot-path cost — check the guard only when the stored
value is itself a `Condition`, and record multiple guarded own-values for one
symbol as unsupported. This work precedes the `ValueQ` line below, which
depends on it: `ValueQ[b[-3]]` cannot be right until `b[-3]` evaluates right.

**`ValueQ` — COMPLETE 2026-08-26 (briefs 345, 346, 355, 356; `b85569c2`
and `e67715f1`).** Both tiers landed. The trial no longer commits side
effects: PRNG state is restored, the host print sink is suppressed for its
duration, and `FileWrite`/`Process` are refused, with restoration on every
exit path including panic. Proven-stable targets — `ValueQ[undefinedSymbol]`
above all — answer without evaluating at all: 332.6 us to 0.76 us per call at
8 000 definitions, and no longer scaling with session history. rum's precise
answers are unchanged throughout (`f[1] = 3; ValueQ[f[2]]` is still `False`),
with one deliberate exception: `ValueQ[DumpSave[path, x]]` now answers `False`
because the refused capability fails the trial.

The reasoning that got there follows.

**`ValueQ`: precision and freedom from side effects are the hard
requirements; not evaluating is not — user ruling 2026-08-26.** The first
ruling was "answer by lookup, never evaluate", but brief-346 proved the three
cannot hold together: deciding whether a `Condition` or `PatternTest` rule
applies *requires* evaluating user code, so a non-evaluating answer must be
imprecise on the very common `f[x_?NumericQ]` and `f[x_] /; test` shapes.
Since not-evaluating was only ever a means to the other two, it is the one
that gives way.

The shape is therefore two-tiered:

1. **Provable shapes answer by lookup, without evaluating.** rum's precise
   answers are kept — `f[1] = 3; ValueQ[f[2]]` stays `False`. A coarser "does
   the head have any definition" rule was considered and rejected as too
   surprising.
2. **Undecidable shapes are evaluated in an effect-suppressed environment**,
   so the answer stays exact while nothing escapes the trial.

Tier 2 comes first: it closes the contract breaches below without changing any
answer. Tier 1 is then pure performance.

The three breaches are what the ruling closes (brief-345, all reproduced by
the director). The generated reference — normative under the testing policy —
says `ValueQ` tests "by trial evaluation without committing side effects".
Session-local state does roll back (`ValueQ[yy = 7]` leaves `yy` undefined),
but three effects escape the trial:

- **The deterministic PRNG advances.** `SeedRandom[123]; ValueQ[RandomInteger[
  1000000]]` consumes the first draw, so the next draw returns what should
  have been the second. `Capability::Pure` cannot serve as an effect proof
  here — `RandomInteger` and `SeedRandom` are deliberately classified `Pure`.
- **`Print` reaches the host sink.** Output streams immediately when a sink is
  installed; the existing rollback test only covers the buffered no-sink case.
- **`FileWrite` commits.** `ValueQ[DumpSave[path, x]]` writes the file, and no
  in-memory snapshot can undo it.

The open question is the accuracy of the lookup, not the contract. rum already
has a machine that answers "would evaluating this change it" — the evaluation
stability check the evaluator uses to skip stable expressions — but it is
deliberately conservative, and its conservative direction ("might change")
becomes a wrong `True` here. How precise it is, which shapes need more, and
what an undecidable shape should answer are the questions to settle.

**Found while re-probing, and more serious than the list above: iterator
variables were invisible on an assignment lhs.** `Do[f[i] = i, {i, 1, 3}]`
installed one `HoldPattern[f[i]] :> 3` and said nothing — the standard table
fill and memoization idiom. Fixed in `4f5b85f1` (briefs 342-343): a clean
iterator symbol lives in a dynamic own-value frame, ordinary evaluation
resolves definitions then that frame, and the held-lhs normalizer stopped
after definitions. It now takes the same third step in the same order. The
`Do[With[{k = i}, f[k] = k], …]` workaround in the test suite is gone.

**The same matrix exposed three defects that have nothing to do with
iterators** — all reproduce with a plain global symbol. **Re-probed
2026-08-28 against the release CLI at `65c03870`: all three are still
present**, each probe carrying a liveness assertion and a working control.
They are one family — the assignment target is not normalized before the
definition is installed — but by three different mechanisms, so whether
they are one repair or three is a question for the design brief.

- **`AssociateTo`/`KeyDropFrom` write back to the raw target.** `ii = 2;
  af[2] = <|"a" -> 1|>; AssociateTo[af[ii], "k" -> 7]` reads `af[2]`
  correctly, applies the update correctly, then installs the result under a
  second definition `HoldPattern[af[ii]] :> <|"a" -> 1, "k" -> 7|>` and
  leaves `af[2]` untouched. Silent — no message. `KeyDropFrom` behaves the
  same way. Mechanism: the fallback path evaluates `target_expr` to read
  the current value but hands the *unevaluated* `target_expr` to
  `store_association_lvalue_update`
  (`crates/rum-stdlib/src/associations/mutation.rs:196-225`).
- **The compound assignment heads do not evaluate a `Part` selector.**
  `kk = 1; lst[[kk]] += 5` reports `Part::pkspec1` and changes nothing,
  while `lst[[kk]] = 99` works and `lst[[1]] += 5` works — a clean
  three-point control that isolates the defect to compound-assignment ×
  symbol-selector. Confirmed on nine heads (`AddTo`, `SubtractFrom`,
  `TimesBy`, `DivideBy`, `Increment`, `Decrement`, `PreIncrement`,
  `PreDecrement`, `AppendTo`). Loud, so it ranks below the silent ones.
  **The repair model is one call site**: `part_assignment.rs:228` passes
  `evaluated_selectors` to
  `exact_position_paths_from_part_target_selector_exprs`, while
  `definition_updates.rs:389` passes the raw `selector_exprs` to the same
  function.
- **`TagSet`/`UpSet` never normalize lhs arguments, and the definition they
  install is unreachable.** Worse than recorded: `tg = 5; h /: h[tag1, tg] =
  7` stores `HoldPattern[h[tag1, tg]] :> 7`, so `h[tag1, 5]` does not match
  and neither does `h[tag1, tg]` — the lhs evaluates to `h[tag1, 5]` before
  lookup, so the definition can never fire from any input. Same for the true
  up-value form (`vv = 5; g /: k[g, vv] = 11` leaves `k[g, 5]` unevaluated)
  and the tagless form (`ww = 5; m[ww] ^= 22`). A clean symbol works
  (`m3[qq] ^= 44` then `m3[qq]` is 44), and `m2[7] ^= 33` correctly reports
  `UpSet::nosym`. Silent in every failing case. Whether the lhs *should*
  normalize is still the standing contract question — but "writes a
  definition that nothing can reach, without a message" is a defect under
  either answer, so decide the contract, then fix at least the silence.

**The microbench scale ceiling — FIXED `d9c713e1` (brief-344), verified
2026-08-28.** Both halves are closed: the harness has headroom, and it fails
loudly when it does not. `--scale 5` at the default `--reps 7` now completes
and writes 112 measured labels with an empty stderr, and the scale is proved
live — 100 of the 112 labels carry exactly 5x the scale-1 iteration count,
the other 12 being fixed-width labels that scale does not touch. The script
now exits non-zero on both failure shapes (`core-language-microbench.sh:1084`
for a non-zero engine exit, `:1090` for zero measurement rows), so the
header-only `timings.tsv` can no longer pass for a run. The record above of a
scale-1 ceiling was written before brief-344 and was never retired.

Note for whoever re-checks this: `--reps 1` is not a valid probe of the
ceiling. The repetition loop is the layer that nests around each label's own
loop, so lowering it hides the very defect being tested.

**Deferred, with evidence, from the second sweep:**

- **Error spans — FIXED 2026-08-31** (brief-385, `a0c6afe3`). Spans now leave
  the parser in file coordinates, so `--file`, stdin scripts, host script
  requests, `Get` and `Needs` all report the right line and column, and `Get`
  has a location at all. Re-probing found it wider than recorded: both the line
  and the column were wrong, not just the column, and `a == (b == c)`-style
  right-hand cases were affected too. The suite's blind spot was the cause —
  every pinned location was `1:x`, where form and file coordinates coincide.
- **A trailing-operator form still re-parses per line.** The linear splitter
  drives the parser form by form, but a form whose delimiters are balanced and
  whose expression is unfinished (`1 +\n2 +\n3`) cannot be predicted from
  balance alone. Such forms are short in practice; recorded rather than hidden.
- **Pattern- and key-driven `ReplacePart` rules stay sequential.** Merging them
  into one pass would reorder user-visible evaluation or change association-key
  semantics. Their outer walk is iterative, so wide rule lists no longer
  consume a native frame per rule.
- **Comparison chains — FIXED 2026-08-31** (brief-386, `8b93dffb`). This entry
  was wrong in both directions: `a == (b == c)` flattened too, so the
  right-hand cases were not already fine, and the shared AST was not a fact
  about chains but a consequence of parsing comparisons as associative
  binaries. **These operators have no associativity.** Chains are now collected
  in the parser, so a parenthesised operand is a boundary by construction:
  `(a==b)==c` is `Equal[Equal[a,b],c]` and `(1==1)==1` is `Equal[True, 1]`.

  The same repair closed a second defect in the same machinery: mixed chains
  lowered to `And`, repeating the middle operand, so it evaluated twice and the
  chain short-circuited. They now lower to `Inequality[e1, op1, e2, op2, …]`
  (user ruling 2026-08-30), which carries exactly `{Protected}` — no hold
  attribute, which is why each operand evaluates exactly once and nothing is
  skipped. The missing short-circuit is deliberate: undetermined comparisons
  are routine, and a chain retained symbolically needs every part in evaluated
  form, so early exit has no well-defined meaning. A short-circuiting variant,
  if ever wanted, belongs in an option or a rum-native sibling.
- **Product propagation is decided from construct nesting, not data flow.**
  `Table[Length[Table[...1000...]], {i,1,200}]` is refused although the outer
  `Table` retains only integers. Distinguishing that statically is not
  something the iteration seam can do, and the safe side of the line is the one
  that keeps `Table[Table[...]]` refused.

Every item remains subject to the performance gate in
[docs/testing-strategy.md](./docs/testing-strategy.md): safe Rust only, base
versus candidate timing plus counters and memory, and reject or report
regressions and space-for-time tradeoffs.

## The suspension and cancellation line — CLOSED (briefs 397-411)

Opened by the first release-profile full-test run on the Linux host, which
produced two `rum-jupyter` pump failures that the debug gate never showed.
brief-397 diagnosed them as one invalid test assumption and one real
host-contract defect; the repair of the second uncovered a third defect, and
that one turned out to be the largest.

- **Cancellation was erased by the evaluation it was meant to stop.**
  `begin_top_level_interrupt_budget` cleared a session-global flag as
  `eval_request`'s first act. Repaired by briefs 404 and 405 as request-scoped,
  generation-qualified cancellation: a non-`Clone` accepted-request ticket, a
  phase word encoding `(generation << 1) | cancelled`, compare-exchange
  cancellation that cannot reach a later generation, and completion as the
  linearization point. The legacy `AtomicBool` lane is deleted, so the
  suppress-or-synchronize conditional that existed only because two mechanisms
  coexisted is gone with it.

- **Abandoning a suspension leaked every scope its frames owned.** Found while
  writing brief-400's cancellation test, split out as a pre-existing defect,
  and confirmed reachable through public `rum-host` API alone. Repaired by
  briefs 402 and 403: a closed abandonment protocol every one of the 545 named
  frames must classify, enforced by a supertrait with no default body rather
  than by a source scan — a distinction that matters because `SuspensionFrame`
  is `pub` and a scan cannot reach code that does not exist yet.

- **Overlapping public requests are rejected** (user ruling). Superseding a
  suspended request had been an accident of consuming queued input; it is now
  refused, and remains unimplemented as an explicit operation.

- **The jupyter timing-assumption class**, closed by briefs 406-411 — see the
  entry below.

brief-400 is superseded rather than resumed: brief-401 refuted its repair
(driving saved frames with `Err(Cancelled)` is not an unwind, because ordinary
frames consume errors and run work), and briefs 402-403 had rewritten the
files its stash touched. `stash@{0}` is kept as reference material.

## The 2026-08-31 consistency review — CLOSED (briefs 389-396)

Four reviews of the work from briefs 364-388, run in independent parts so an
interrupted dispatch would lose at most one. Eight findings, three of them
high, all repaired by briefs 393-396. **Every repair replaced a convention with
a structure**: a list of protected pattern wrappers became one registry shared
with the matcher; assertions resting on interner order moved to a construction
where the two Bloom lanes cannot intersect; a public installer that trusted its
caller now demands a typed left-hand side; and a wrapper claiming two
constructors left the continuation protocol that offered a third.

**Two of the findings were corrections to the briefs themselves**, which was
worth more than some of what was asked for: the frame-root allowlist is an
inventory of handwritten traversals rather than a two-entry exemption list, and
the frame queue is the single insertion route for ordinary code rather than
literally. `CLAUDE.md` was wrong on both and has been corrected.

**One was a gate miss of mine.** brief-388 repaired the two assertions that had
been seen failing; five more rested on the same mechanism, because the
diagnosis and the repair both examined the recorded sightings and the recorded
sightings were two.

The findings and their repairs follow.

## Opened by the 2026-08-31 consistency review (briefs 389-392)

- **The assertion class — CLOSED 2026-08-31** (brief-394, `HEAD`). The
  review's five sites were the complete remaining set; each group now splits
  semantic coverage on ordinary symbols from optimization coverage on a
  lane-disjoint target, with no assertion deleted. Four groups swept 0..256 at
  257/257. Two assertions could not be made to fail under any preintern count
  and were reported rather than changed. The original finding follows.

- **HIGH — brief-388 fixed two instances, not the class.** Five further
  assertions still pair ordinary symbols with ordinary dependencies, so they sit
  on one Bloom lane and can be flipped by a legal false positive exactly as the
  two closed flakes were: `evaluation_cache/tests.rs:105`, `:472`, `:499`,
  `association_write_index.rs:84`, and `association_write_recovery.rs:40`.
  Director-confirmed. **This is a gate miss of mine**: I verified that the two
  known tests were repaired and that the lane argument held, and never asked how
  many other assertions rest on the same mechanism. The diagnosis brief had the
  same shape — it examined the two recorded sightings because those were the
  sightings. Sweep the class, then decide per assertion whether it wants the
  lane-disjoint construction or should stop asserting cache residency at all.

- **Request-scoped cancellation — CLOSED 2026-09-02** (brief-404). Cancellation
  is generation-qualified, survives its request prologue, and stale handles are
  inert after completion or reset. The lifecycle, crate-local host, and runtime
  tests use channel handshakes. Only the ignored stability loop at
  `crates/rum-host/tests/stress.rs:124-144` still uses a 25 ms sleep; replacing
  that test synchronization remains separate cleanup.

- **The rum-jupyter timing-assumption class — CLOSED 2026-09-02** (briefs
  406-411). The entry that stood here named four IOPub tests and a 50 ms delay;
  brief-406's inventory found the class was much larger and had three unrelated
  roots, and two of its own citations were already stale. What actually
  existed:

  - *In-process worker tests* asserted immediate readiness and turned a shared
    two-second helper into a semantic requirement, though the production API
    exposes only a nonblocking poll and promises no latency. Closed by
    brief-407 with instance-scoped start/finish signals; a probe had first
    established that a yield is not enough and only a real happens-before edge
    would do.
  - *The native IOPub subscription race* was real, but the repository's
    `acknowledge_iopub_subscription` was not a barrier for it — it did a
    heartbeat round trip over an independent Req/Rep socket. Closed by
    brief-411 with a sentinel that travels the backend's own PUB path, plus
    parent-scoped frame collection so "the next four frames" stopped being an
    assumption.
  - *The native `basic` tests* were a third family the old entry could not
    reach: they open Dealer/Router and Req/Rep only, with no `SubSocket`, so
    PUB/SUB was never their mechanism. Closed by brief-409 along with the two
    negative assertions that tried to prove absence by waiting 200 ms.
  - *An ephemeral-port TOCTOU* nobody was looking for: the helper reserved five
    ports, read them, dropped every listener, and only then let the backend
    bind. Closed by brief-408 by recovering the resolved endpoint the pinned
    library already returns and that the wrapper was discarding.

  **The sandbox note that stood here is now enforced rather than remembered.**
  A skipped native test used to be reported by libtest as `ok`, which is how
  brief-384 recorded a 0-of-30 result that measured nothing. Without
  `RUM_ALLOW_NATIVE_ZEROMQ_SKIP` a host that cannot bind loopback now fails all
  nine loudly. Verification of native behaviour still belongs on a
  loopback-capable host: brief-409 passed the worker's gate and broke all nine
  native tests on the director's, and brief-410 added socket-free harness
  coverage so that class is catchable in a sandbox that cannot bind.

- **Comparison-chain coverage — CLOSED 2026-08-31** (brief-396, `aa8644cf`).
  Both gaps are covered: `crates/rum-stdlib/src/tests/logic/inequality.rs:31-35`
  lowers a surface uniform-`Less` chain and keeps its n-ary head, and `:82-98`
  counts all three operands including the leftmost. The original finding
  follows.

  A uniform-relational lowering regression or a leftmost double evaluation
  would have escaped the suite: no test lowered a surface uniform-`Less` chain,
  and none counted the leftmost operand.

- **Pattern regions — FIXED 2026-08-31** (brief-393, `d423d2d3`). The list of
  protected wrappers is gone; a shared registry names every pattern head once
  and preparation, the matcher, `MatchQ`, options matching and substitution all
  consult it, so the sets cannot drift. It gathers heads that had been
  recognized in three separate files, which is why it carries `HoldPattern` and
  `OptionsPattern` beyond the seventeen the matcher file compared by name. The
  old-versus-new recognized-head diff lost nothing, and matcher behavior was
  checked by probe. Per-form treatment is now documented rather than implied.
  The original finding follows.

- **HIGH — preparation evaluates inside pattern regions it does not recognize.**
  The pattern-region policy enumerates the wrappers it protects and omits
  `Verbatim`, `PatternSequence`, `OrderlessPatternSequence`, `KeyValuePattern`,
  `Longest`, `Shortest`, and `OptionsPattern`, all of which the matcher
  supports. Director-reproduced: `typeAlias = Integer; fV[Verbatim[Blank[
  typeAlias]]] = 7` stores `fV[Verbatim[_Integer]]` — **`Verbatim` means
  verbatim, and its contents were evaluated** — and
  `fP[PatternSequence[sideCall[], b_]] = 8` runs `sideCall[]` once and stores
  its result. An ordinary binder `x_` is correctly left raw, so the policy's
  shape is right and only its list is short. **User-reachable, no external
  crate needed.** This is family B's own invariant failing on the forms nobody
  enumerated.

- **Published-surface boundaries — FIXED 2026-08-31** (brief-395). The
  installer now takes a typed lhs and the selector wrapper left the
  continuation protocol; an external crate attempting either construction fails
  to compile (E0061, E0277), reproduced independently by the director. Catalog
  installation measured -0.56%. The original findings follow.

- **HIGH — `install_system_down_value` stores a raw lhs from outside the
  crate.** It is `pub`, takes an arbitrary `Expr`, and mints the trusted token
  internally, so preparation never runs (`session_api.rs:225`,
  `prepared_store.rs:385`). `#[doc(hidden)]` does not restrain a downstream
  Rust caller. brief-391 compiled an external crate and installed
  `ExternalRaw391[i391]` with `i391 = 2`; the stored identity is unreachable by
  ordinary lookup — exactly the defect class family B closed, reintroduced
  through a public API. The sealing inventory test only scans in-repository
  callers (`definition_store_sealing.rs:130`) and structurally cannot see this.

- **`EvaluatedPartSelectors` can be forged through the kernel API.** brief-390
  compiled an external crate that built the wrapper from raw expressions via
  the exported continuation decoder (`kernel_api.rs:18` and `:100`,
  `evaluated_part_selectors.rs:68`) and reached the parser with it. The typed
  seam brief-377 introduced is therefore not the compiler-enforced
  two-constructor boundary it was described as. Impact today is limited — the
  forged value failed to parse, and only a kernel-API consumer could construct
  one — but the guarantee is the point of that design, so the export surface
  should be narrowed. Worth fixing; not urgent.

- **The selector order contract — CLOSED 2026-08-31** (brief-396, `aa8644cf`).
  `docs/guide/evaluation.md:172-179` now says the selectors evaluate exactly
  once *when a mutation reaches target selection*, and names the
  `AssociateTo`/`KeyDropFrom` validation branch explicitly. The original
  finding follows.

- **The selector order contract is stated unconditionally and is not.**
  `docs/guide/evaluation.md:168` says selectors evaluate exactly once, but
  `AssociateTo` returns before selector handling when its update specification
  is invalid (`mutation.rs:166`), so they evaluate zero times on that branch.
  **The code is right and the document is wrong**: the specification is an
  ordinary evaluated argument, and a call that fails validation should not
  produce partial selector side effects. Reword the contract to say the
  selectors evaluate once *when the mutation reaches target selection*.

- **The frame-queue guard reaches less far than claimed.** brief-389 found
  `insert_suspension_frame` public, the resume driver pushing directly, and the
  invariant exempting `suspension/core.rs` and test directories. No current
  path bypasses `SuspensionFrame`, so the traversal guarantee holds; a future
  non-macro insertion in either exempt area would not be caught. `CLAUDE.md`
  has been corrected; whether to narrow the exemption is open.

## Opened by the 2026-09-02 security audit (brief-413)

The first audit of this codebase permitted to say "attack". GLM ran it because
Codex's cybersecurity filter makes security review there impossible. Seven
findings; the two HIGH ones share a root — **bytes the memory model cannot
see**.

- **HIGH — the symbol interner is process-global, leaks by construction, and is
  invisible to the memory ceiling.** `symbol_interner()`
  (`crates/rum-core/src/expr.rs:47-55`) is a process-wide
  `OnceLock<Mutex<SymbolInterner>>`; `intern_text` (`:57-70`) `Box::leak`s every
  new name; there is no removal or clear API. `current_memory_proxy_bytes`
  (`runtime/debug_tools/resources/memory.rs:50`) walks session-owned structures
  only and never mentions the interner, so `eval_memory_limit` cannot see the
  growth. `HostSession::reset()` rebuilds the session and does not touch it.
  **Director-reproduced**: under a `{Pure}`-only policy, eight requests each
  interning 20 000 fresh symbols took RSS 5.6 MB → 57.5 MB monotonically, and
  `reset()` returned only to 47.0 MB. For a long-lived host — a jupyter kernel,
  an editor host, a per-request-session service — this is an exhaustion vector
  no documented kernel mechanism can close, shared across every tenant.

- **HIGH — the modelled memory ceiling is sampled every 64 evaluation points and
  can be overshot multiplicatively in between.**
  `ACTIVE_MEMORY_LIMIT_CHECK_STRIDE = 64`
  (`runtime/debug_tools/resources/constraints.rs:9`). Its own comment claims the
  interval is "bounded by the existing per-step element, integer, and iteration
  budgets" — but those bound *counts*, not the bytes of one multiplicative step,
  and `eval_string_join` (`rum-stdlib/src/strings/text/builtins.rs:13-42`) has no
  projected-byte precheck at all, unlike `StringRepeat` (`:664-668`) or `Power`.
  Reported: an 8 MB ceiling reached a 2.04 GB peak before terminating in band.
  A failed allocation is `handle_alloc_error`, which aborts outside the panic
  boundary. **Mechanism director-verified in source; the 2 GB overshoot is not
  director-reproduced.**

- **MEDIUM-HIGH — `rum-jupyter`'s hand-written JSON parser has no depth guard.**
  `json_support.rs` contains zero occurrences of depth/limit/MAX_, and
  `parse_value`/`parse_object`/`parse_array` (`:61,86,126`) are mutually
  recursive. Reported: 50 000 levels of nesting aborts the kernel process with a
  stack overflow, outside the panic boundary. Reachable unauthenticated when the
  signature policy is `Disabled` — see the fail-open finding below. The kernel's
  own parser has a four-level depth guard; the discipline did not reach the
  adapter. **Not director-reproduced** (it aborts a process).

- **MEDIUM — `eval_with_timeout`'s deadline is silently dropped across an input
  suspension.** `crates/rum-host/src/session.rs:518-524` clears the timeout
  unconditionally after the guarded call, and its own rustdoc says the timeout
  "is cleared after the request completes or fails" — a suspension is neither.
  `resume_eval_request_with_input` never re-arms it. **Director-reproduced**: a
  500 ms deadline, suspended on `Input[]`, resumed and ran 1.39 s to completion
  with no termination reason. The host cannot re-arm it; there is no
  `set_eval_timeout` on `HostSession` and no timeout variant of
  `resume_with_input`.

- **LOW — a terminated request leaves `$Context` where the script put it**, and
  the next request inherits it. Same family as the control-state leaks briefs
  397-411 closed; `Block` bindings and `TimeConstrained` deadlines were verified
  to restore correctly, so this is the survivor. No document says whether
  context is per-request or per-session state.

- **LOW — `restore_from_bytes` is arbitrary code execution by design**, and the
  documentation states the mechanism without stating the consequence. Restoring
  untrusted bytes runs them under that session's policy; a sandboxed session
  refuses the capability-gated parts, an unrestricted one does not.

- **LOW — the jupyter signature policy fails open.** An unrecognised
  `signature_scheme`, or an empty key, silently yields
  `KernelSignaturePolicy::Disabled` (`connection.rs:123-131`) rather than
  refusing to start. HMAC comparison is also not constant-time
  (`transport/codec/validation.rs:45,50`), which is near-unexploitable on local
  IPC but differs from the reference implementation.

**Three criticisms of the threat model itself**, which are worth more than some
of the findings: the shipped jupyter kernel is itself a host and chooses full
authority with no per-cell timeout or memory ceiling, which no document states;
the two-stage memory defence has a gap between the modelled ceiling and the
host's own RSS watchdog, which the docs describe as merely "one pattern"; and
**no document says which state is per-request and which is per-session**, which
is the same question briefs 397-411, this audit's `$Context` finding, and the
timeout gap are all instances of.

### Closed by brief-415/416 (`1427bb71`) — the walk itself

A third gap, found reviewing brief-414's plan rather than by the audit.
`current_memory_proxy_bytes` is the baseline the strided check and all 18
projected-byte prechecks measure against, and it walked 34 of `Session`'s 93
fields. It now walks 52; the other 41 carry a recorded exemption, and a macro
generates both the inventory and an exhaustive `let Session { .. }` pattern, so
a new field stops compiling until it is classified. `ByteArray` also stopped
being charged its decoded length instead of the base64 text it holds.

### Opened by that work, not yet briefed

- **The call-stability cache is bounded by entry count, not by bytes.**
  `CALL_STABILITY_CACHE_MAX_ENTRIES = 1 << 14`
  (`runtime/evaluation_cache/stability.rs:12`) caps entries at 16 384, but each
  entry pins a `SharedExpr` head and `SharedArgs` — one entry was measured
  holding 64 KB. This is the same error as the stride comment: a budget that
  bounds counts while the quantity at risk is bytes. It was invisible until
  brief-415 made the cache part of the walk; now a session can approach its
  ceiling on cache alone. The repair touches insertion paths, so it needs its
  own brief and the mandated hot-path A/B. Fold it into the Gap B line.
- **Abandonment retains 1 668 bytes more than completing the same work**, and
  nobody has explained why. Deterministic: identical across five consecutive
  runs of `suspension_abandonment_contract`. It is two orders of magnitude
  below the ~64 KB a genuinely retained `Reap` bucket would cost, so the
  contract tests absorb it in a stated margin rather than failing — but the
  constant is unexplained, and an unexplained constant in a test becomes a
  fossil. Worth one probe before the Gap B line closes.

### The traversal-depth line (briefs 422-433) — the per-node bound is in; the construction-side bound is not

**What is fixed.** brief-422 bounded four unbounded recursive descents that
ended the process: `FromXml`, `RegularExpression` group nesting, `rum-syntax`'s
postfix-blank chain, and the four codec encoders. brief-425 then converted ten
traversals to iterative walks — `ToString`, `Map`, `Apply`, `ReplaceAll`,
`Cases`, `Level`, `MemberQ`, `Count`, `Position`, `FreeQ`.

**Measured on a 2 MiB stack** — Rust's default for a spawned thread, so what an
embedder gets without setting `stack_size`
(`docs/guide/resource-limits-and-sandbox.md:122`). After brief-425:
`ToString` 43,000, `Level` >20,000, `Map`/`Apply` 20,000, and six others at
4,500 completing / 4,700 not. `Expr::Display` clears 4,097 with headroom to
5,000-8,000; `Depth`, `LeafCount` and `ByteCount` clear it.

**Rulings, all 2026-09-03.** `MAX_PHYSICAL_EXPR_DEPTH >= 4097`, preserving
plan-330's documented 4,096-link contract; the bound is on **physical
expression height**, not the `Depth` builtin, which omits heads and treats
associations specially and so certifies a different tree; traversals get **no
new knob** — rum already pairs a fixed portable limit with a measured backstop
(`recursion_limit` beside `poll_native_stack_limit`) and the design extends
that pair; and `DEFAULT_NATIVE_STACK_LIMIT` at 6 MiB fails open on a 2 MiB
thread where it can never fire, so it becomes safe-by-default on the smallest
supported stack, raised by hosts that have more.

**The construction-side bound is CLOSED as not affordable (user ruling,
2026-09-03, after four spikes).** Admission is O(1) only if the physical height
is cached, and every way of knowing it costs more than the bound is worth. The
decomposition is complete and each term was measured separately:

| Term | Where measured | Cost |
|---|---|---|
| 8 bytes per `Arc` allocation, height never computed | brief-434 V1 vs V0 (director) | **+5% to +14%** |
| `SharedArgs::new` scan with an Arc dereference per child | brief-434 V2/V1 | +3.9% to +11.4% |
| full rescan after every mutation | brief-434 V3/V2 | free |
| O(1) conservative maintenance (`max(old, written)`) | brief-434 V4/V2 | free |
| `max` over args at construction, inline in `Expr::Call`, no allocation growth | brief-435 (director-confirmed) | **+6% to +9.5%** |

Two of those results are worth keeping. **The allocation term was the hidden
one**: `stringtrim-literal-list-32000` builds no call spines and never reads a
height, and it cost +14.21% from a field that was never computed —
`SharedArgsInner` 40→48 bytes moves it into the next allocator chunk class.
`Expr` and `Atom` are both 32 bytes while `Expr::Call` carries 16, so an inline
height grows no allocation, and brief-435 proved that removes the term
completely (those same rows return to −0.40%…+0.93%). **What no design removes
is the pass itself**: computing `max` over the arguments at construction costs
+6% to +9.5% on the construction and mutation families.

**The one untried shape, and why it was not tried.** The caller could supply the
height, fused into a loop it already runs — a `Table` builder tracks a running
max, a `Part` assignment knows `max(old, replacement)`, a permutation knows the
max is unchanged. That is the only remaining place the cost can go. It was
rejected on a semantic argument rather than a measurement: a supplied height can
only be checked by a debug assertion, and the construction-side bound was worth
its price precisely because it turned an omitted traversal check into a *type*
invariant. A height that callers must get right is the "relies on people
remembering" shape this project replaces with structure, so the design no longer
buys what made it attractive.

All four spikes are preserved: brief-426 and brief-434 V4 in the stash,
brief-435 in history at `74082418` (reverted by `c8031ee1`).

**Consequence for this line: the tail is now the work.** Without a construction
bound, each recursion in brief-422 §3(e) has to be measured and, if it falls
below 4,097, bounded on its own. That is the next item.

**The 4,500 ceiling was candidate normalization, and it is now fixed**
(briefs 427-433, `0f679699`). `match_expr_owned` recursively normalized the
whole candidate before dispatch (`pattern_matching/core.rs:23-32`), so the
ceiling belonged to one normalization and not to the traversal at all: a single
`MatchQ[x, _?IntegerQ]` — which defeats `fast_match_q_resolved_pattern` — died
in the same bracket as an all-level `Count` over the same value. The normalizer
now runs a depth-16 recursive front path and hands an over-deep subtree to a
cold explicit-stack engine in place, following `Level` and `Map`/`Apply`.

Measured on 2 MiB, director-run: `MatchQ[x, _?IntegerQ]` went from 4,601
completing / 4,651 overflowing to clearing **43,001**, and
`Count[x, _, Infinity]` from 4,501 / 4,701 to clearing **20,001** — 10.5x and
4.9x the ruled 4,097 minimum. Above 43,000 the limiter is `Depth`'s own
recursion, not the normalizer; the checked-in certificate therefore excludes
`Depth` and a separate liveness test carries the height assertion.

**It removes stack growth, not repeated work.** The general entry still
normalizes the entire candidate on every invocation, so `Cases` over a deep
tree is still quadratic. Two deferred work-removal items sit here, priced by
brief-428 §6 and deliberately kept out of that A/B: `Cases` normalizes each
candidate twice, and the pattern is recompiled per selected node.

**The cost took three shapes to get to zero, and that is the lesson.**
Propagating exhaustion as a `Result` cost 6-15% on wide traversal normalization
because every atom returned through it; handling exhaustion locally fixed that
but left ~8% on `matcher-single-*` because atoms still entered a function
carrying a depth parameter their arm ignored. The recursive worker now takes a
decomposed call, so an atom cannot reach it.

**What remains, and why it is not queued** — *superseded 2026-09-04 by briefs
436-446; see "The deep-rebuild class" and "The native stack budget" below. Most
of the deep failures listed here turned out to share one defect downstream of
the traversals, the evaluation-stability walk, and the per-site conversions that
survive that repair are brief-439's steps 2-8. The paragraph is kept because its
inventory is still the map, and because its warning — measure before briefing —
is exactly what re-probing under the new budget vindicated.*

brief-427 classified brief-422 §3(e)'s tail: only the rewrite family shares the normalizer mechanism, and
everything else is its own independent recursion — string-pattern machinery,
calculus, polynomials, schema conformance, `Flatten[_, Infinity]`, tensor rank,
`Merge` nested, substitution walks, fingerprints, `MapIndexed` and `Scan`
(which brief-425 did not cover), `Expr`'s `Display`, `PartialEq`, `Hash` and
serde, and `lowering.rs` for host-constructed ASTs. Suspension continuation
encoding is "unclear — needs measurement". **None of these is known to fall
below 4,097; they are unbounded, which is not the same thing.** Measure them
against the bound and order by real ceiling before opening briefs, rather than
scheduling a dozen repairs on source reading alone.

**They now have numbers (director-measured, 2026-09-03).** Release CLI,
`ulimit -s 2048`, values built with `x = 1; Do[x = {x}, {i, N}]` or the shape
each probe needs; every failure below was re-run and confirmed to be a genuine
`thread 'main' has overflowed its stack`, not an in-band error and not a
timeout.

| Probe | completes / overflows | against 4,097 |
|---|---|---|
| **`StringExpression` nesting (construction)** | 375 / 550 | **10.9x short** |
| **`Replace[_, _, {0, Infinity}]`** | 1,250 / 1,425 | **3.3x short** |
| **`MapIndexed[_, _, Infinity]`** | 1,600 / 1,775 | **2.6x short** |
| **`D` over a nested-`Sin` spine** | 1,600 / 1,775 | **2.6x short** |
| `FullForm` (`Expr::Display`) | 5,800 / 6,150 | 1.4x |
| `Save` (session serialization) | 5,800 / 6,150 | 1.4x |
| `SameQ` (`PartialEq` logical path) | 7,900 / 8,600 | 1.9x |
| deep `Association` construction | 10,700 / 11,400 | 2.6x |
| `Merge` over deep associations | 10,700 / 11,400 | masked, see below |
| `Scan[_, _, Infinity]` | 11,400 / 12,100 | 2.8x |
| `ArrayQ` | 15,600 / 17,000 | 3.8x |
| `Flatten[_, Infinity]` | 25,400 / 26,800 | 6.2x |
| deep `Association` key (`Hash`) | clears 45,000 | — |
| `Depth` | clears 20,001, fails by 43,001 | — |

**Four fall short, and that is the queue**, in shortfall order:
`StringExpression` construction, `Replace` at all levels, `MapIndexed`, and the
calculus walker. brief-427 predicted `MapIndexed` and `Scan` as a pair; `Scan`
clears by 2.8x and `MapIndexed` does not, so the pair was not a pair.

**Two rows share a bracket and a mechanism.** `FullForm` and `Save` both give
out at 5,800/6,150 because a session snapshot is a UTF-8 source image, so
`Save` renders through the same `Expr::Display`. Fixing one fixes both, and it
is the same renderer brief-424 found at 5,000-8,000.

**`Merge`'s own ceiling is unmeasured**: building the deep association it needs
gives out at exactly the same 10,700/11,400, so the construction masks it. Any
probe of an association-consuming traversal has to bracket the construction
first — this one did, which is the only reason the masking is visible.

**Probes that measured nothing, recorded so they are not repeated.** `D` and
`Expand` do not descend into lists at all (`D[{{{q}}}, q]` returns a chain-rule
expression over `List`, `Expand` returns its argument unchanged), so a
list-shaped probe of either measures the construction; the calculus number above
comes from a nested-`Sin` spine instead. `ArrayDepth` and `Dimensions` are not
rum builtins and return unevaluated — which still prints, so they would have
reported a false ceiling. `Nest[List, 1, N]` fails in band at the fixed-point
pass limit (256), not on the stack. `MergeNested` with one argument returns an
operator form.

**Not probed, and why.** String-pattern *matching* machinery (the construction
probe dies at 400, so matching cannot be reached with a deep pattern this way),
suspension continuation encoding, `lowering.rs` for host-constructed ASTs (not
reachable from the source language), profiling fingerprints (diagnostics-gated),
and polynomial walkers (no cheap deep probe — a deep polynomial expands
exponentially).

**Adjacent, still open:** `rum-jupyter`'s `json_support.rs:61` has no depth
guard on wire-message JSON (the 2026-09-02 audit's F3), deliberately untouched.

### Rulings on the repair line (user, 2026-09-02)

Settled after brief-414's design pass and the director review of it.

- **The interner becomes session-scoped (option A1), but brief-414's drafted
  step 4 opens as a spike, not an implementation.** A1 is the only option that
  both reclaims on `reset()` and attributes bytes to the session that made
  them; A3 and A4 pay A1's price (`Symbol` stops being `Copy`) and buy less,
  and A2 keeps `Copy` while leaving the growth permanent. The reason for the
  spike is that `Symbol`'s `PartialEq`, `Hash` and `Ord` are all built on `id`
  being a perfect collision-free hash of the text — `eq` is a fast *accept* on
  `id` (`crates/rum-core/src/expr.rs:250-254`), `Ord` uses it as a fast path
  (`:275`), `Hash` hashes it alone (`:258`). A content fingerprint collides, so
  equality must demote to fingerprint-rejects-then-compare-text, and `Symbol`
  is the key type of `definitions`, `attributes` and six caches. The spike
  makes `Symbol` non-`Copy`, reports the true compiler-error count, and
  measures the new equality and clone/drop paths. A1 lands or falls on those
  numbers; the fallback is A2.
- **`Symbol::interned_id` is redefined as a stable content fingerprint**, with
  its rustdoc rewritten to say so. Its documented promise ("symbols with the
  same text share the id") becomes *stronger* — true across sessions rather
  than only within one process — and its sole consumer, the call-stability
  bloom (`evaluation_cache/stability.rs:23`), already multiplies by the
  golden-ratio constant, which is Fibonacci hashing designed to spread dense
  ids and is unharmed by receiving a hash instead. This also deletes the
  `u32::try_from(pairs.len()).expect(...)` panic path at `expr.rs:88`. The
  density and uniqueness of `id` are what equality relies on today, which is
  why this ruling and the A1 ruling are one decision, not two.
- **`ByteArray` is charged its physical representation.** It holds base64 text
  and was charged its decoded payload length, about 25% under what it occupies
  — the only shape in `expr_memory_proxy_bytes` charged less than it holds.
  Folded into brief-415.

## Decision-gated or deferred

- **Custom `Sort` / `KeySort` comparisons — settled 2026-08-26.** The ordering
  callback schedule is unspecified: a valid strict weak order determines the
  result, while comparison count, order and direction are not promised.
  Stability is still promised, and key evaluation stays exactly once per entry
  in input order — the contract relaxed comparisons only. Anything that would
  change what a comparator's *return value* means, or how many times a *key*
  function runs, remains a separate semantic decision.
- **Full polynomial factorization.** Deferred by user decision. The documented
  `Factor` contract remains the current rational-polynomial subset: monomial
  extraction plus recursively discovered linear factors. A substantially more
  complete algorithm requires finite-field factorization and its supporting
  polynomial machinery; do not grow the current linear-factor code piecemeal.
- **Formatter binary-chain planning (P2).** Some long binary chains still
  rebuild temporary vectors. Revisit after the P1 runtime/stdlib work, with
  formatter output frozen before changing the planner.

## Standing engineering debt

- Deep `Expr` trees constructed directly through the Rust API can overflow the
  native stack on drop. Source-created trees are depth-guarded; three iterative
  `Drop` designs regressed the hot path by 20-50% and were rejected. Keep the
  host limitation documented in
  [docs/host-embedding-and-hardening.md](./docs/host-embedding-and-hardening.md).
- Session snapshots are UTF-8 source images. A direct binary snapshot format is
  deferred until substantially larger session images justify the extra format
  and compatibility surface.
- Keep suspension-safe helper state on typed continuation values. `Monitor`
  refresh and `ValueQ` rollback remain best-effort unless their documented
  contract is deliberately promoted.
- The same-expression evaluation-cycle guard can reject recursive definitions
  whose visible call shape repeats while other session state progresses. Relax
  it only with an explicit progress model that preserves genuine-cycle
  detection.
- Continue splitting formatter planner/builder/render/test modules by
  responsibility as they grow; do not create a separate string crate without a
  real ownership boundary.

### The native stack budget — CLOSED (2026-09-04, briefs 436-438)

The 2026-09-03 ruling above ("`DEFAULT_NATIVE_STACK_LIMIT` at 6 MiB fails open
on a 2 MiB thread where it can never fire") is executed. The route to it was
not the one expected.

brief-436 diagnosed why a deep `StringExpression` construction died at 375
levels: a strict head whose call stays residual and never acquires a stability
proof is re-evaluated through eager argument preparation on every construction
step, one native evaluator level per spine level. The class is wide — `Power`,
`Mod`, `Max`, `Min`, `Equal`, `Less`, `Not` and every metadata-only string head
— and ordinary user-defined heads are outside it.

The gate then found the guard. `poll_native_stack_limit` measures bytes
actually consumed and is the correct instrument; its default was three times
the ruled stack, so the same construction failed in band on 8 MiB and killed
the process on 2 MiB. `SECURITY.md` published the opposite promise, and
`codecs_xml.rs` already carried a workaround comment blaming the 2 MiB debug
test thread — this defect observed in the wild and patched at the call site.

brief-437 measured the frame. One evaluation level is a 4,736 B static packet
across seven resident frames, matching a 4,675 B slope bisected over three
stack sizes. **Three per-level costs, and a budget argument has to say which it
means**: 4.7 KiB release strict, 7.5 KiB release user recursion, 20.6 KiB debug
user recursion. The projected floor from local frame optimization is 1.5-2.6
KiB, against the 511.875 B that height 4,097 on 2 MiB would allow — so the
ruled height for a strict residual call needs an explicit task stack, not frame
polishing. That is open and unscheduled.

brief-438 landed the calibration: `2 MiB - 512 KiB poll headroom - 512 KiB
non-evaluator reserve = 1 MiB`, computed rather than restated, with
`native_stack_limit_for_declared_stack` for hosts that provision more. Four
tests broke and all four were repaired by declaring a thread and deriving the
budget from the same size — depths and `reclim` assertions untouched — which
also retired the `codecs_xml` workaround and restored its acceptance coverage
from 64 back to the full 256.

**The accepted price**: on the default profile a 2 MiB host gets roughly 48
debug / 140 release levels of user recursion, not the nominal 1024.
`recursion_limit` is now documented as a ceiling.

**Known limit, recorded not fixed**: the byte guard polls only at
evaluation-depth increments, so a recursion running entirely inside one builtin
advances no depth and is invisible to it for that whole descent. Converting
such walkers to explicit stacks is what closes that, which is what briefs
422-433 did for ten traversals.

### The deep-rebuild class — stability walk CLOSED (2026-09-04, briefs 439-442)

Re-probing the three remaining traversal items under the new 1 MiB budget found
one shared defect instead of three separate ones.

**The mechanism.** A traversal that rebuilds a deep result hands the evaluator a
tree with fresh `Arc` identities at every level. Every stability cache key
misses, and `call_stability_uncached_with_bloom` walks the whole tree through
`cached_expr_stability_with_bloom` — mutual recursion in a file where
`evaluation_depth` never appears, so the byte guard has no poll point for the
entire descent. That is why `ReplaceAll` on a match, `Apply` at a descending
spec and `Replace` at all levels died the same way, and why the same operations
with a rule that never matches survived: the traversal returns
`Expr::call_shared` with the original pointers and the root's proof still hits.
brief-425's conversions were not wrong; the shared post-result check downstream
of them was never converted.

**The certificate panel had a matching hole.** None of the twelve exercised a
rebuild. `Map` and `Apply` used `{-4097, 0}`, which in rum selects level 0 only
— intended, test-pinned semantics, not a defect — and `ReplaceAll` replaced an
absent symbol. Repaired under a three-part rule: a unique deepest target that
must propagate `changed`, an assertion on the exact rebuilt value rather than a
summary, and a selector pinned separately on a shallow fixture.

**Two repairs were rejected before the third worked.** briefs 440 and 441 made
the walk iterative behind a depth-bounded front. Both were functionally
complete — every invariant covered by differential tests including an exact
cache-insertion-order trace, all twelve certificates green, counters identical
— and both cost. Reproduced in my own four-pass window with within-run
quotients: `downvalue-exact-miss-repeat` +5.6%/+6.9% against a 0.62% base
spread, `downvalue-pattern` +4.4%/+7.1%, `cases-wide` +5.7%/+3.8%,
`matcher-single-tiny` +2.4%/+2.2%. A depth parameter and an `Option` return on
the most-consulted function in the system is not free. Preserved as stashes.

**What worked was moving locals, not control flow.** The walk cost 811 B per
level and its dominant frame was 584 B — not because the function is large but
because the whole chain is inlined into it, making the frame the union of
thirteen data-constructor arms, cache lookup and insertion, head-effect
computation and profiling tokens. A deep descent executes almost none of that.
Outlining them behind `#[cold] #[inline(never)]` gave **255.6 B/level measured**
(bisected at 1024 and 2048 KiB: 3,905 and 8,008 levels), a 3.17x reduction, with
counters byte-identical and the previously-regressed families neutral to
faster.

**brief-443 measured three fail-closed shapes and none fit** — a per-level floor
check grows the frame to 272 B and bails at 3,797; the same check on a stride
holds 256 B but bails at 4,040; a bare depth counter keeps the certificates but
is not stack-size-honest and regresses `do-count-null`. All three would have
turned nine certificates into in-band failures, so a well-measured no was the
outcome. It also found the route out: about 64 bytes of off-by-default
profiling state resident in the hot recursive frame, which is a hard-line
violation independent of any poll. **brief-444 removed it** — the key is derived
inside the cold enabled path and the start instant moved to a thread-local
stack — taking the walk to **192 B/level** (184 B prologue plus the return
address, `/proc` slope 191.1). Height 9,500 now completes on 2 MiB where 8,008
was the ceiling, and the previously untested per-head profiling surface gained
an env-gated certificate that was run against HEAD before the change.

**The walk now fails closed (briefs 445-446).** The strided poll fits once the
frame is 192 B/level: the check sits on the call-cache miss path, immediately
before the recursion, and on budget exhaustion returns a conservative "not
stable" — always safe, because stability only decides whether evaluation may be
skipped, and the value then goes through the evaluator's own guarded path.
Measured at my gate: bail-out bisected at 5,339 ok / 5,341, a margin of 1,242
levels (30%) over the ruled 4,097; height 20,001 on a 2 MiB stack moves from
exit 134 to an in-band `General::reslim`.

brief-445 put the counter at the walk's entry, where it also ran for atoms and
cache hits — neither of which can grow the stack — and cost `list-literal-small`
+7.99%/+7.98%. Moving it to the miss path removed that and is the more honest
meter, since the stride's overshoot arithmetic already assumes it counts
recursive descents. Two codec depth tests surfaced at the same time: they were
passing by consuming more stack than the declared budget permits, and were given
declared threads like their XML sibling rather than having their accepted depth
lowered. The ruled
4,097 contract is met with roughly 2x headroom, but the fail-closed promise is
not. A strided poll inside the walk would convert that into an in-band error at
some hot-path cost that has not been measured. That is the next decision, and it
needs its own measurement rather than an assumption.

**Also still open:** brief-439's steps 2-8, the per-site native conversions
(`Replace`, `MapIndexed`, `D`, exact-path engines, substitution engines, tensor
rebuilders). Their projections rested on brackets measured under the old 6 MiB
budget; they have now been re-probed, and the results, the reordered dispatch
queue, and a correction to the organization-line note are in the last section of
this file.

### The tail re-probed under the 1 MiB budget (2026-09-04, director)

brief-439's gate ruled that steps 2-8 would be re-derived against post-step-1
measurements rather than dispatched from projections
(`docs/dev/codex/brief-439-deep-rebuild-class-design.md:1300-1308`). They now
have those measurements, and they reorder the queue.

Release `rum-cli` at `59410b75`, `ulimit -s 2048`, 1 MiB evaluator budget. Every
probe carries a liveness assertion on the exact rebuilt value — the replaced
leaf, the deleted atom, the factor count, the swapped pair — and was calibrated
at height 3 before any bracket was believed.

| Site | last ok / first fail | mode | against 4,097 | draft |
|---|---|---|---|---|
| `StringExpression` construction | **216 / 224** | in-band | **19x short** | none |
| `Alternatives` construction | 210 / 218 | in-band | 19x short | none |
| `MergeNested` over deep associations | 1,114 / 1,122 | in-band | 3.7x short | 8 |
| `Replace[_, _, {0, Infinity}]` | 1,308 / 1,316 | **process kill** | 3.1x short | 2 |
| `D` over a nested-`Sin` spine | 1,738 / 1,746 | **process kill** | 2.4x short | 4 |
| `DeleteCases[_, _, Infinity]` | 1,808 / 1,816 | **process kill** | 2.3x short | 6 |
| `MapIndexed[ff, _, {0, Infinity}]` | 2,694 / 2,702 | in-band | 1.5x short | 3 |
| `ReplacePart` at a full deep path | 2,854 / 2,862 | **process kill** | 1.4x short | 5 |
| `Reverse[_, All]` | — | — | **clears** | 7 |
| deep association construction | — | — | clears | — |
| `FullForm` (`Expr::Display`) | 5,846 / 5,851 | **process kill** | clears, 1.4x | none |

**Controls, run in the same window.** `ReplaceAll[x, 1 -> 2]` and
`Apply[gg, x, {0, Infinity}]` clear 4,097 with a real rebuild (depth 4,098,
`MemberQ` hit), and `Cases`/`Count`/`Position`/`ToString`/`Level` all clear.
Step 1 and brief-425 are therefore verified independently of their own briefs.

**The instrument was calibrated before any result was believed.** The
classifier was shown printing each of its four verdicts on a known case:
`fRec[n_] := fRec[n + 1]` for in-band, deep `FullForm` for process kill,
`ToString` for pass, and a 200 000-iteration build for the invalid-probe class.
Its first version scored two known failures as `UNKNOWN`.

**Two probes measured nothing and were replaced.** `D[Nest[Sin, q, 4097], q]`
fails on the **fixed-point pass limit (256)**, not the stack — the same trap
`Nest[List, 1, N]` set earlier in this line; building the spine with
`Do[e = Sin[e], {i, N}]` reaches it. `Reverse[x, {1, Infinity}]` is not a legal
rum call (`Reverse` takes `All` or a list of positive levels), so it returned
unevaluated and would have certified a false ceiling. A third guard held: deep
association construction was bracketed first and clears 4,097, so `MergeNested`'s
number is its own and not its fixture's.

**Three findings reorder the queue.**

1. **`Replace`'s ceiling did not move.** 1,308 is identical to brief-439's
   pre-step-1 bracket, which is both the proof that step 1 never touched these
   sites and a consistency check on the harness. Confirmed in source: the
   strings `poll_native_stack_limit` and `evaluation_depth` occur **zero** times
   under `kernel_data/rewrite_data/replace/`, in `structure_data/exact_paths.rs`,
   under `runtime/substitution/`, and in `rum-stdlib/src/calculus.rs`. No poll
   point is why these four end the process instead of raising an error.
2. **Order by failure direction, not by shortfall.** Drafts 2, 4, 6 and 5 fail
   open; drafts 3 and 8 already fail closed and are short only on the contract.
   Draft 7's representative clears 4,097 outright, which matches brief-439
   scoping 7 and 8 as budget-gated rather than unconditional.
3. **The worst shortfall is not in steps 2-8 at all.** The strict-residual
   construction class is at 216, worse than the 375 recorded under the old
   guard, because the budget now fires: the failure direction improved and the
   height fell. `Alternatives` at 210 confirms a class rather than a
   `StringExpression` quirk, and 1 MiB / 213 ≈ 4.9 KiB per level matches
   brief-437's measured 4,736 B evaluation-level packet. brief-437 already
   priced frame polishing out of this (floor 1.5-2.6 KiB against the 511.875 B
   that 4,097 allows), so this is the evaluator task stack — its own design
   line, not a per-site conversion.

**Dispatch order for the tail**: draft 2 (`Replace`), then 4 (calculus), 6
(`DeleteCases` and the substitution engines), 5 (exact paths) — the fail-open
four, in shortfall order — then 3 and 8, with 7 reduced to a confirmation probe.
The evaluator task stack is tracked separately and opens with its own
design-only brief.

**Draft 2 is CLOSED (brief-447, `c71d808a`).** `Replace` keeps a depth-16
recursive front and hands the tail to an explicit work/completed stack; both
depth helpers get a single-chain fast path plus an explicit stack for branching
shapes. Repeated depth work was held out of scope so the bound could be gated
alone. Director-measured: 4,097 completes with the exact rebuilt value, the
ceiling moves 1,308 -> 5,843 (1.43x margin), and above it the failure is an
in-band native stack limit instead of a dead process — the same bracket and the
same direction as `ReplaceAll`, which is the endstate this repair was aiming at.

Two gate notes worth keeping. The worker reported its own two-revision A/B
stop-loss on a residual +0.61% on `replacepart-wide-rule-list`; four passes with
within-run quotients read -3.26% raw and -1.50% quotient on that label, with a
same-binary spread of 1.80%. **The stop-loss was noise, and reporting it rather
than tuning through it was the correct call.** Second, the ceiling is now the
shared stability walk rather than `Replace` itself, so the remaining fail-open
sites should be expected to land in the same 5,800-level bracket once
converted.

**Draft 4 is CLOSED (brief-448, `bd3820c1`), and it refuted its own
projection.** Both of `D`'s walks — the containment prepass that runs once per
visited node, and the four-way differentiation group — keep a depth-16 recursive
front and hand the tail to explicit stacks. `calculus.rs` split into containment
and differentiation modules declared from it, `lib.rs` untouched.

brief-439's draft had written that 4,097 must end in a recoverable in-band
error, and its stop-loss forbade demanding successful evaluation. **It
completes**: `Length[D[e, q]]` is 4,096 with no messages and a usable session
afterwards. The projection predated the stability-walk repair, which is why the
brief asked for the outcome to be measured and asserted rather than assumed —
dispatching the draft as written would have produced a certificate asserting an
error that no longer happens.

**Two costs recorded, not fixed.** Repeated containment work was held out of
scope, so `D` is near-cubic on a deep spine — director-measured 0.146 s at 256,
0.970 s at 512, 7.26 s at 1,024, about 440 s at 4,097 — and the release-only
certificate panel now takes about 460 s end to end. Budget that into any future
gate that runs the panel. Removing the repeated prepass is the work-removal item
this leaves behind, and it sits beside the `Cases` double-normalization entry
above.

**A denominator lesson for the A/B protocol.** The within-run quotient read
+0.96% against the 3 nearest-magnitude neighbors and +2.43% against 12, both
outside their same-binary spreads — while the target's raw movement was -0.06%
and the median over all 46 same-tier labels was -0.18%. A three-label
denominator has no stable median of its own; take the tier median over tens of
labels, or the quotient invents a regression.

**Draft 6 is SPLIT on measurement, and its `DeleteCases` half is CLOSED
(brief-449, `d83929fe`).** Probing the substitution engines before briefing them
found that draft 6's bundle was three different problems:

| site | last ok / first fail | against 4,097 |
|---|---|---|
| `DeleteCases[_, _, Infinity]` | 1,808 / 1,816 | 2.3x short |
| `Function` slot substitution over a deep body | 3,685 / 3,693 | 1.11x short |
| `Module` lexical substitution over a deep body | 3,677 / 3,685 | 1.11x short |
| `With` lexical substitution over a deep body | 3,677 / 3,685 | 1.11x short |
| rule-RHS binding substitution over a deep template | clears 4,097, kills by 8,001 | **not short** |

All are process kills; `substitution/` and `cases/delete/` both have zero poll
points. **The engine draft 6 named first is not short of the contract at all** —
it is merely fail-open above roughly 6,000, which is the lesser problem in this
line's own taxonomy. So `DeleteCases` went alone (its own subtree, no caller
ripple), and the substitution engines get their own brief with their own
numbers and their ten-file consumer surface.

`DeleteCases` now completes at 4,097 and fails closed above 5,408 — the same
bracket as `Replace`, confirming the prediction above that converted sites land
where the shared stability walk binds.

**The second worker A/B stop-loss in a row was noise, and this one needed the
layout control to settle.** The worker reported +7.14% on the new
`deletecases-leaf-delete-small` label after two isolated revisions and stopped.
At my gate, over a six-pass base/candidate/layout window: the target moved
+0.96% against a **+0.87% whole-panel median**, and a layout-control binary —
HEAD with one function relocated inside its `impl` block, semantically identical
— moved the same label's within-run quotient **+1.57%** against the candidate's
+1.51%. The candidate is inside the layout floor. The label also measures a
depth-9 value, which never reaches the iterative tail at all, so the only change
it can see is one integer comparison.

**Consequence for briefing**: best-of-2 without a whole-panel median is not
enough to declare a regression on this panel, and the next brief in this line
must require the worker to report the whole-panel median beside each label.

**Settled by counters instead, and the timing instrument is now known to be
weak on this host (user ruling, 2026-09-04).** This machine is an i9-14900HX
with 8 P-cores and 16 E-cores. One core is unhealthy at full turbo, so a kernel
module caps every core at 5.2 GHz; `scaling_max_freq` reads 5 200 000 on the
P-cores and 4 100 000 on the E-cores, so the clock ratio is **1.27x**, not the
1.41x the spec sheet implies. Nothing pins a benchmark to a core class. Any wall-time percentage here can be a scheduling artifact larger than
the effect being judged, so **a timing A/B is an indication, never proof**.
`CLAUDE.md`'s standing rule "counters beat timings, report counters first" was
not applied at the briefs 447-449 gates; it is reinstated.

Re-measured with `RUM_PROFILE_ALLOC=1` on a `--features alloc-count` build,
pre-447 (`b9cdb56d`) against `a0432c43`, each label as an isolated probe
(fixture plus the exact statement in a `Do` loop):

| probe | pre-447 | HEAD |
|---|---|---|
| empty script (baseline) | 1 572 | 1 572 |
| `Replace[Nest[List,1,8], 1 -> 2, {0, Infinity}]` x500 | 25 466 | 25 466 |
| `DeleteCases[Nest[List,1,8], 1, Infinity]` x500 | 24 915 | 24 915 |
| `D[Nest[Sin, v, 8], v]` x200 | 92 895 | 92 895 |
| `ReplacePart[{0}, rules]` x20 | 379 678 | 379 678 |

Allocation counts identical to the byte, `bytes_total` and `peak_live_bytes`
too. **The disputed +7.14% is settled: the three conversions do no extra
allocation on their labels' shallow paths.** The counter was calibrated for
sensitivity first — one extra `Do` iteration moves it +46 allocations.

**One caveat found while doing it: a counter is load-immune only if the
workload is deterministic.** The whole-panel total is not — two runs of the
identical binary read 9 999 973 and 10 000 216 — because the harness has
time-adaptive parts (the retained-module GC batches keyed off
`AbsoluteTiming`). The base-vs-candidate panel total differed by +530 in 10
million, inside that same-binary spread. Use isolated per-label probes and
calibrate the same-binary spread before believing any counter delta.

**Nothing was overturned by the re-measurement.** The three A/B accepts it
re-checked (briefs 447, 448, 449) are all confirmed; what the timing numbers
were weak evidence *for* turned out to be true.

**The gap the allocation counters left is now closed with the PMU.**
`perf_event_paranoid` is 2 on this host, so `perf stat -e instructions:u`
works unprivileged and sees the pure CPU cost that allocation counts cannot.
Minimum of the tight cluster, migrated runs discarded:

| probe | pre-447 | HEAD | delta |
|---|---|---|---|
| `Replace[Nest[List,1,8], 1 -> 2, {0, Infinity}]` x500 | 43 442 724 | 43 381 754 | **-0.140%** |
| `DeleteCases[Nest[List,1,8], 1, Infinity]` x500 | 37 991 580 | 38 019 336 | **+0.073%** |
| `D[Nest[Sin, v, 8], v]` x200 | 96 361 697 | 96 587 958 | **+0.235%** |

All three are real signals — the within-cluster spread is 0.006% to 0.08%, so
these are 3x to 40x the noise — and all three are negligible. `DeleteCases`
costs about 50 extra instructions per call over a nine-level value, which is
the depth-16 front check running once per node and nothing else. **The disputed
figure was +7.14% from the worker and +0.96% from my own timing window; the
true cost is +0.073%.** `Replace` is faster, not slower.

Three PMU traps, met on first use and worth reusing: the hybrid PMU reports
`cpu_atom` and `cpu_core` separately and a run that **migrates between core
classes reads about 55% high** and must be discarded (both counters non-zero);
`<not counted>` means that PMU never ran; and instruction counts are
core-class independent, so an all-E-core run is directly comparable with an
all-P-core one.

**Draft 5 is CUT DOWN on measurement and its fail-open half is CLOSED
(brief-450, `b7f390ac`).** Probing all nine consumers draft 5 named found that
**six already clear 4,097** — `MapAt`, `ReplaceAt`, `Insert`, `Delete`,
`Extract` and list part assignment — so its shared-primitive refactor across
about fifteen files would have mostly rewritten working code. Two were short and
fail open, and they are what shipped: multi-spec `Take` 2,573 and deep-path
`ReplacePart` 2,854, both now completing at 4,097 with the exact rebuilt value.

The third short site is **left open deliberately**: nested association part
assignment at 1,106 / 1,114, in band. It fails closed, and it brackets
identically to `MergeNested` (1,114 / 1,122), which points at a shared
association-side cause worth diagnosing once rather than patching twice.

**This was the first brief to forbid a wall-time A/B**, and the protocol worked
on both sides. The worker reported identical allocation counts with a
sensitivity calibration, instruction counts from clean clusters, and the
migrated runs it discarded — no stop-loss, no revisions chasing noise. At my
gate the instruments agreed: allocation counts identical, instructions **-1.65%
on `Take` and -0.94% on `ReplacePart`** against a 0.006% within-cluster spread,
so both operations are slightly *faster*.

**Draft 6's substitution half is CLOSED (briefs 451-453, `7acff251`), and it is
the first change this line accepted over its own stated bar.** The lexical
engine ended the process at 3,385 — `With`, `Module` and a named-parameter
`Function` share that bracket through `substitute_lexical_bindings_changed` —
and rule binding at 4,430. Both now complete at 4,097 with the exact substituted
value and fail closed above 5,867 and 5,843, the same bracket as every other
converted site.

Two corrections the re-probe forced, both of which moved the scope:
**slot substitution was never short** — the earlier "3,685" figure came from a
probe using a *named parameter*, which routes through the lexical engine; a true
`Slot[1]` probe clears 4,097 and fails closed at 5,867 — and **rule binding was
in scope after all**, because its 1.08x margin is thinner than the ~8% the
lexical bracket drifted over fourteen unrelated commits. A margin thinner than
the drift is not a margin.

**The cost, and why it was accepted.** The ordinary `With[{x = 1}, x + 1]` path
costs **+0.714% instructions**; `Module` is -0.083% and rule RHS -0.159%, and
allocation counts are unchanged. That is over the ±0.15% bar brief-452 set, and
it is accepted on measurement rather than waived: two shapes were built and
measured, and the alternative — a thread-local depth — was **slower** at +1.085%
and carried five manual `enter`/`leave` pairs with no `Drop` guard, a shape
whose breakage would be a silently shrinking recursion budget for the rest of a
thread rather than a failure. The accepted shape puts the depth in a
`LexicalWalk` context struct beside the bindings and the shadow set, so the arms
take *fewer* arguments than before the bound existed.

**Two method notes.** The instrument is what made this decision possible at all:
+0.714% is invisible under this host's 27% scheduling spread, so on wall time
this would have shipped unexamined. And the semantics at risk was the per-subtree
shadow set — several arms build a different set for one child than for its
siblings — which is now pinned by a regression at depth 20, past the recursive
front, asserting that an inner `With` shadows only inside its own subtree.

**Recorded as an unexplained sighting, not a diagnosis: two dispatches of
brief-452 were terminated mid-run**, each after its edits had landed and before
any verification. What is established is only what it was *not*: the three
recorded content-filter kills (briefs 422, 424, 439) ended with `exit 1` and a
policy message in the log tail, while both of these ended with **no `codex
exit=` line at all**, and neither log contains any filter text. It was not a
size or duration ceiling either — brief-451 succeeded with a 15.3 MB log over
18m40s, while these died at 623 KB. The only error present is
`codex_models_manager: failed to refresh available models: timeout waiting for
child process to exit`. That is suggestive of the connection, but this failure
has not been seen before on this project and the cause is **open** (user, first
occurrence). Record the next sighting rather than explaining this one away.

**Third sighting, 2026-09-05, brief-456.** Same signature: the log stops mid
output at 193 KB with no error text and no `codex exit=` line, and the process
exits some time later having applied only its first edit. It ran 27 minutes and
died about 25 minutes *after* an unrelated SIGTERM of mine had killed the
foreground wrapper, so it is not that either. The three failures are all
**early** — 193 KB to 636 KB of log — while every completed dispatch in this
line ran to between 745 KB and 15.3 MB, so it is not a size or duration
ceiling. What it costs is bounded: the tree held only unvalidated falsifier
tests, which were discarded rather than kept.

The recovery that worked was to shrink the brief — brief-453 changed one file
and ran only `fmt` plus two crate suites and two instruction probes, with the
workspace run and the release panel kept on the director's side.

**Re-examined 2026-09-05 after the user reported seeing, in other Codex
sessions, `Falling back from WebSockets to HTTPS transport` and
`stream disconnected before completion: websocket closed by server before
response.completed`. That is now the leading hypothesis, and one line above is
retracted.**

- **Retracted.** "The only error present is `codex_models_manager: … timeout` …
  That is suggestive of the connection" is wrong. That error appears in
  *successful* runs too — five times in brief-455, which completed, and also in
  briefs 445, 446, 447, 449, 451, 454 and 457 — while briefs 448, 450, 453 and
  456 have none. In the failed brief-452 log it sits at line 7,509 of 12,110,
  38% short of the truncation. It is background noise.
- **What Codex's own session record shows.** The rollout for the failed run
  (`~/.codex/sessions/2026/09/04/rollout-2026-09-04T20-26-46-*.jsonl`) has 243
  records and its **last one is a `reasoning` response_item, mid-turn**. No
  error record, no abort record, no completion record. That is the shape a
  stream cut leaves.
- **Why the absence of the message proved nothing.** codex logs through
  `tracing` and at its default emits **only ERROR**: across fourteen dispatch
  logs and about 1.5 million captured lines, the entire tracing output is 25
  ERROR lines from two targets, with no WARN, INFO or DEBUG at all. A transport
  fallback notice is WARN or INFO and could never have appeared in what we
  capture. **The earlier "cause is open" was declared on an instrument that can
  only see ERROR** — the same failure this line keeps making, believing a
  negative from an uncalibrated probe.
- **The instrument is fixed rather than the sighting explained.**
  `scripts/dispatch-brief.sh` now exports `RUST_LOG` (default
  `warn,codex_core=info`, overridable with `BRIEF_RUST_LOG`), verified to reach
  the worker through the interactive fish shell and the `with_proxy` wrapper.
  It is exported rather than prefixed because the worker is a shell function:
  `env VAR=v cdx …` cannot exec a function, and fish has no prefix-assignment
  syntax. The same latent bug was fixed in the `cgl` branch, where
  `CGL_MAX_OUTPUT_TOKENS` had been passed as a prefix assignment that fish
  would not have applied.

The cause remains formally **open** — the specific message has not been seen in
our own logs — but the next occurrence should now be explicable rather than
merely recorded.

**Draft A of brief-454 is CLOSED (brief-455, `9f8e89b2`).** The association arm
of the stability walk is bounded: an association-rooted chain takes over the
whole subtree once the walk meets an `Association`, keeps a depth-16 borrowed
front, and carries its depth through intervening ordinary calls — which is what
the `Map` result needs, since it produces `ff[Association[Rule[…]]]` at every
level. The generic chain is untouched.

`Map` over a deep association and a read after a nested key-path assignment now
complete at raw physical height 4,097 (`N=2048` wrappers), up from 966 and
1,145. `Map` runs to the construction ceiling at 5,457 wrappers, so the
operation is no longer the limit — the fixture is.

**A measurement limit learned here — and my first reading of it was wrong,
corrected at the brief-456 gate.** My generic probes read -7% to -8.7% on paths
this change cannot touch, while the worker's read within ±0.15%. I attributed
that to codegen from the module growing 444 KB. **It was the build path.** I
compared a binary built in a `git worktree` under `/tmp/…` against one built in
`/home/remi/dev/rum`; rebuilding the candidate under a same-shaped `/tmp/…` path
brings the unrelated probe from -8.42% to **+0.005%**. The tell was already in
the data and I missed it: briefs 455 and 456 are unrelated changes and produced
*the same* -8.4% on *the same* untouched probes.

**The rule: build base and candidate at the same filesystem path, or
instruction counts are not comparable.** Allocation counts are path-immune —
identical to the byte across the boundary in both gates — so pair every
instruction number with them. The worker's numbers were right throughout,
because it built both arms in one path.

**Draft B is CLOSED (brief-456, `d0e80b1b`), and the re-probe that opened it
found the failure mode had flipped.** After brief-455, `MergeNested` went from
1,153 wrappers failing **in band** to 2,027 **terminating the process** — the
association stability arm used to exhaust the byte budget first, and that arm
polls; once it was cheap, `MergeNested`'s own unpolled recursion reached the raw
stack instead. That moved it from 3.6x short and fail-closed to 1.01x short and
fail-open. **Removing a guard-adjacent cost can convert a fail-closed site into
a fail-open one** — worth carrying into the remaining drafts.

The walk is now explicit enter/finish work items behind a bounded front, with
suspension freezing the whole stack. It completes at raw 4,097 and runs to the
construction ceiling at 5,457 wrappers; instructions are neutral (-0.119% on the
merge probe, +0.005% on an unrelated control) with allocation counts identical.

**Drafts C-J were re-derived and re-measured (2026-09-05, brief-457 plus the
director's brackets), and six of the eight drafts are DROPPED.** brief-455 was
the shared prerequisite every one of them was priced against, so removing it
re-priced the whole class. Codex re-derived the plan from source; the brackets
below are the director's, measured in parallel because a past-ceiling probe is
not delegable in this line.

Release CLI at `80c8acce`, `ulimit -s 2048`, 1 MiB evaluator budget, heights in
**physical levels**. A verdict is 3/3 runs; see the measurement note below.

| draft | site | at 4,097 | ok <= | fail >= | verdict |
|---|---|---|---|---|---|
| **C** | `ConformSchema` / `SchemaErrors` | in band | 208 | 224 | **19.7x short**, fail-closed |
| **E** | multi-spec `GroupBy` | kill | 2,016 | 2,032 | **2.03x short, fail-open** |
| **J'** | `KeyTake` over nested lists | kill | 2,856 | 2,864 | **1.43x short, fail-open** |
| 439-3 | `MapIndexed` generic list shape | in band | 2,944 | 2,952 | 1.39x short, fail-closed |
| **J'** | `Keys` over nested lists | kill | 3,056 | 3,080 | **1.34x short, fail-open** |
| D | `DeleteMissing` | pass | 6,272 | 6,304 | clears 1.53x, **kills above** |
| H | `AssociationKeyDeflatten` | pass | 7,344 | 7,376 | clears 1.79x, kills above |
| I | `MapIndexed` association shape | pass | 7,344 | 7,376 | clears 1.79x, kills above |
| G | `AssociateTo` / `KeyDropFrom` Part target | pass | 8,048 | 8,080 | clears 1.96x, kills above |
| H | `AssociationKeyFlatten` | pass | 8,292 | 8,334 | clears 2.02x, kills above |
| F | nested key-path assignment | pass | >= 10,800 | — | fixture-bound |
| I | `Scan` association shape | pass | >= 10,800 | — | fixture-bound |
| J | `Merge` collection parser | pass | 16,072 | 16,128 | clears 3.92x |
| ctl | `Map`, `MergeNested` | pass | >= 10,800 | — | confirms 455/456 independently |

**Four findings, in the order they change the queue.**

1. **The direction flip is the whole class, not `MergeNested`.** Every site that
   now clears the contract **terminates the process** above it; not one fails in
   band. brief-455 lifted them over 4,097 and removed the polling frame they
   were borrowing in the same stroke. The contract is met with 1.5x-2x margin
   and the fail-closed promise is not, which is the same shape as the stability
   walk before briefs 445-446 — and it is now the largest open item in the
   class, ahead of any individual site.
2. **Draft C's axis is schema depth, not value depth, and its packet is
   ten times what was reasoned.** Decomposed: a deep value against a shallow
   schema (`_`, or one level) clears `N=2049` outright; only a nested *schema*
   descends. 1 MiB / 104 pairs is roughly 10 KiB per pair, the scale of an
   evaluation-level packet (brief-437: 4.7-7.5 KiB), not of the 1,056 B native
   frame brief-454 priced. So the validator crosses the evaluator at **every
   nested pair**, not only at the leaf as both brief-454 and brief-457 reasoned
   — which is also why it fails closed. That mechanism should be confirmed in
   source before the repair is briefed.
3. **`Keys` and `KeyTake` over nested lists are short and fail-open**, and they
   are in the one lane brief-454 deliberately deferred ("should be briefed in
   its generic collection-shape lane rather than silently bundled here").
   brief-454 reasoned `KeyTake` at about 2,900 (right) and `Keys` at about 6,200
   (**2x optimistic, and the verdict was wrong**). Draft J as scoped — the
   parsers — is correctly dropped; the deferred sibling lane is not a note any
   more, it is two fail-open shortfalls.
4. **`GroupBy`'s failure direction does not depend on the callback**, which was
   brief-457's own largest stated risk. A callback consuming evaluator depth
   (`Function[u, deepf[u]]` over a `Module` + `Nest`) stays a process kill and
   *lowers* the ceiling below 1,008 specs rather than converting it to an
   in-band error.

**The re-derivation's accuracy, since it decides how much to trust the next
one.** Ten of twelve reasoned projections landed within 2-25% of measurement,
four of them within 2% (`Merge` parser 16,384 vs 16,072; exact-path replacement
8,192 vs 8,048; `AssociationKeyFlatten` 4,228 vs 4,146; association `MapIndexed`
3,745 vs 3,672), and it correctly picked the binding mode inside `DeleteMissing`
before measuring it. The two failures are both in the same direction — an
evaluator crossing priced as a native frame: schema (19x, and the direction
wrong) and `Keys` (2x, and the verdict wrong). **Reasoned packet arithmetic is
reliable for an operation-local native walk and unreliable the moment the
descent crosses the evaluator.**

**The re-ordered queue**, fail-open before fail-closed, then by shortfall:

1. The class-wide fail-open direction above the contract (new; supersedes
   ranking the individual passing sites).
2. Draft E, multi-spec `GroupBy` — 2.03x short, fail-open.
3. The deferred collection-shape lane — `Keys` and `KeyTake` over nested lists,
   1.34x and 1.43x short, fail-open.
4. brief-439 draft 3, generic-shape `MapIndexed` — 1.39x short, fail-closed.
   One source walker with draft I, but the surviving defect is the generic
   post-result path, so only an evaluator-side task stack lifts it.
5. Draft C, the schema validator — 19.7x short but fail-closed, and on its own
   axis; re-diagnose the per-pair evaluator crossing before briefing a repair.

**DROPPED as drafted**: D, F, G, H, I, J. Each one's priced target now clears,
and rewriting a passing site is what brief-450 established not to do.

**A measurement note that applies to every bracket in this line.** The ceiling
is **not deterministic**: inside a band a few tens of levels wide the same `N`
flips run to run (`DeleteMissing` at 3,144 read 3 PASS / 2 KILL over five runs),
because the stack base moves between processes. A single-run bisection therefore
reports a bracket 8 wide when the real uncertainty is nearer 50. Every number
above is the highest `N` passing 3/3 and the lowest failing 3/3, with the
failure direction read well inside the stable-fail region, never from the band.

**Two probes that measured nothing, recorded so they are not repeated.** A
literal deep association source (`<|"k" -> <|"k" -> ... |>|>`) cannot reach the
association parser at all — rum refuses source nesting past 128
(`parse nesting exceeded limit 128`) — so draft J's parser site is reachable only
through construction, which clears. And `KeyDropFrom` on an association `Part`
target requires `Key["k"]` selectors; with bare string selectors it returns
`KeyDropFrom::invak`, which reads exactly like a depth failure and is not one.
`AssociateTo` accepts both, so the two heads differ in strictness — worth a
look, but not a defect on this evidence.

### The class-wide fail-open repair — step 1 is IN (2026-09-05, brief-459)

The guard substrate is installed and has one real consumer. `Merge` over a
collection nested past the budget now returns an in-band `StackLimitExceeded`
and leaves the session usable, where it previously terminated the process, and
it still completes at raw physical height 4,097 with the exact merged value.

**Two rulings were made at the design gate and are now implemented.**

- **The operation boundary is `native_stack_limit`, 1 MiB on the default
  profile.** brief-458 proposed `native_stack_limit +
  NATIVE_STACK_NON_EVALUATOR_RESERVE_BYTES` = 1.5 MiB. Rejected: consumption is
  measured from `native_stack_base`, set at the request base
  (`resolution.rs:292`), so the 512 KiB non-evaluator reserve covers frames
  *below* that base and spending it above the base counts the same bytes twice.
  A host that actually used its reserve would overflow — the same shape as the
  6 MiB default that could never fire on a 2 MiB stack.
- **`Merge` is guarded; the other five sites are converted, not guarded.**
  Director-measured under a 1 MiB budget at the contract height, three runs
  each: `DeleteMissing`, `AssociationKeyDeflatten`, association `MapIndexed` and
  `AssociateTo` cannot do the work at all, and `AssociationKeyFlatten` sits on
  the line. Only `Merge`'s collection parser passes 3/3. **A guard at the honest
  budget would have converted four contract-meeting sites into contract-failing
  ones** — trading a fail-open above the contract for a fail-closed below it,
  which is strictly worse. That is what makes conversion the repair for the
  other five, and it is also why `Merge` was the right first consumer: it proves
  the mechanism and the boundary together.

**The shape of the substrate.** A scoped runner captures base and limit once and
owns a sticky exhaustion flag; a branded `NativeStackDepth<'scope>` yields one
child token per recursive edge, polling actual bytes every fourth edge through a
`#[cold] #[inline(never)]` check that returns before the recursion. Logical
depth is a stride only, never a refusal criterion — the bare-counter shape
brief-443 measured and rejected. `kernel_api` exports exactly two
`#[doc(hidden)]` items and no raw poll, base, limit, address, or constructor.
Exhaustion travels through the parser's existing `Result`, so no per-node
`Result` was introduced — the shape that cost 6-15% in brief-431.

**Falsifiers, all four verified at the gate rather than reported.** Two
`compile_fail` doctests (the token cannot enter a `FrameRootTrace` frame, and
cannot escape the HRTB scope) pass in the workspace run; the zero-boundary test
returns `TerminatedEvaluation["StackLimit"]` in band with the session usable;
and the **red/green was reproduced by the director** — removing the guard from
`data.rs:186` turns the focused test red, restoring it turns it green.

**Cost, counters first.** Allocation counts are identical to the byte on all
three probes with a same-binary spread of zero, and the counter was calibrated
first: one extra `Do` iteration moves it +35. Instructions: the untouched
control is -0.020% against a 0.13-0.20% cluster spread, and the `Merge` probe is
**+0.381%** against a 0.098% cluster spread — a real signal, about 15
instructions per guarded edge, on a probe built so guarded edges dominate
(depth 8, 500 iterations, nothing else). The microbench panel was **not** run,
and the reason is that it cannot see this change: `Merge[` occurs zero times in
`scripts/core-language-microbench.sh`, and its one merge label exercises
`MergeNested`, a different operation whose parser is untouched.

**Step 2 is IN (2026-09-05, brief-460): the class is now a closed
declaration.** Two enforcement layers, tests only, no behavior change.

- **The surface registry** — **44 rows** covering both association catalogs
  plus the cross-family axes (`Map`, `Apply`, `MapIndexed`, `Scan`, `Set`,
  keyed `Part`). *(Corrected: this entry first said 52, from a grep that counted
  the enum declaration and the vocabulary list alongside the rows — the
  "a script-derived count is a claim, not a fact" rule, violated in the same
  session that wrote it into a brief.)* The catalog comparison
  is **bidirectional and live** — director-verified by deleting the
  `DeleteMissing` row, which fails with `catalog entries without rows:
  ["DeleteMissing"]`.
- **The recursion inventory** — a `syn` scan over `associations/**`,
  `functional_ops/apply_map/**`, `association_data.rs` and `exact_paths.rs`,
  exhaustive within those roots with no per-function allowlist, comparing exact
  `(path, caller, callee, occurrence count)` rows. Blind spots are documented in
  the test itself: trait-object and function-pointer dispatch, macro-generated
  recursion, and anything outside those roots.

**Each row carries two orthogonal fields and neither may borrow from the
other** — mechanism today, and measured contract status with failure direction
and the brief that owes the work. brief-458's draft had proposed `GuardedNative`
for `Scan` and nested key-path `Set`, which are not guarded; the registry
records them as `UnguardedNative + MeetsContract + Unmeasured` with an owed
queue item. `Unmeasured` is the honest entry: those two clear the association
fixture's own cap, so their direction above it has never been observed, and the
brief made "this cell is unmeasured" an acceptable deliverable rather than a
gap to fill with a guess.

**Both falsifiers were reproduced at the gate, not accepted on report.** The
registry's checked-in `#[should_panic]` misdeclares the real catalog entry
`AssociationKeyFlatten` and must name its real edge
(`key_reshape.rs`, `flatten_association_entries`, count 1). For the inventory
the director added one extra syntactic recursive call to a real function and
captured the red — `("crates/rum-stdlib/src/associations/data.rs",
"parse_merge_keyed_entries_collection_with_diagnostics", …, 2)` against the
declared 1 — then restored it and captured green.

**A gate note worth keeping.** The worker's own workspace run needed
`RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`, so its green did not cover the nine native
ZeroMQ tests. The director's run, on this loopback-capable host and without
that variable, is 2 209 tests green. This is the standing rule that a worker's
gate is worth exactly what its sandbox can execute — and it was **visible only
because the dispatcher now captures WARN**: the run that first hit it logged
`WARN codex_sandboxing::violation: … operation_not_permitted` and exited 101,
a line that would have been invisible under the previous ERROR-only capture.

**Deferred, with a re-open condition**: the `rum-derive` compile-time group
validator that would make an unwrapped recursive edge a build error rather than
a test failure. It is stronger, and it is a separate correctness surface whose
gate should not be shared with the registry's; meanwhile the inventory covers
the same class, because an edge added inside a guarded function changes that
function's occurrence count. Re-open it when the guarded population stops being
small — today `Merge`, with `Scan` and nested key-path `Set` measured
guard-eligible — or when a conversion is rejected and a site stays natively
recursive by decision rather than by schedule.

**The two remaining guards are IN (2026-09-05, brief-461).** Association `Scan`
and nested key-path `Set` now return an in-band `StackLimitExceeded` with a
usable session where they terminated the process, and both still complete at
raw physical height 4,097 — which on a 2 MiB thread exercises the guard at its
real 1 MiB boundary, so the certificate is the direct proof that guarding costs
no contract height. Registry rows flipped to `GuardedNative` with
`FailureDirection::FailsClosed` and `owed_by` cleared; the distribution is now
23 `NoProportionalDescent`, 15 `UnguardedNative`, 3 `BoundedHeap`, 3
`GuardedNative`. **Every remaining `UnguardedNative` row is a conversion, not a
guard.**

Both recursive workers return a plain value rather than a `Result`, so they use
the substrate's throwaway-fallback path — `ScanCollection { parts: vec![],
depth: 1 }` and `AssociationSetUpdate::Unchanged` — with the sticky bit giving
the error precedence. No per-node `Result` was introduced anywhere in this line.

**Scope closure is the property that mattered, and it is visible in the diff
rather than asserted.** `Scan`'s guard closes at the `?` before
`evaluate_scan_parts` runs the user callbacks, so no scope is open across a
possible suspension; `Set`'s closes before the `match update` that writes
`definitions`, so an exhausted path cannot half-write.

**Four edges, four reds, reproduced at the gate.** Removing the wrapper from
each real edge in turn — `traversal.rs` association values, head and arguments,
and `association_data.rs`'s key-path edge — turns its focused test red; all four
did. An edge whose removal leaves the suite green has no coverage, and that is
the check that finds it.

**Cost.** Allocation counts are identical to the byte on all four probes,
including `bytes_total` and `peak_live_bytes`. Instructions: `Scan` +0.082%,
key-path `Set` +0.175%, untouched control -0.013%, and — the check this brief
existed to force — the **`Map` control, which shares the `apply_map` machinery
three of the four edges live in, moves +0.002%**. Clusters were tight on every
row.

**The first conversion is HALF IN (2026-09-05, briefs 462-463).**
`DeleteMissing`'s recursive-mode walkers, both pairs of delayed-wrapper
counters, the memory preflight projection and the missing scan are now bounded
by a depth-16 front plus explicit work stacks. **The finite-target/resume family
is untouched, so the registry row stays `UnguardedNative`** with `owed_by`
naming brief-464 — a row that said `BoundedHeap` while a reachable path still
recursed would be the reclassification failure this project has already paid
for.

**Measured before briefing, and it corrected three earlier claims.**
`DeleteMissing[expr, level]` is the *recursive* mode — `Target` is the
**three-argument** form — so brief-454's and brief-457's per-mode reasoning
rested on a distinction that a two-argument probe silently gets wrong. The
memory preflight is unreachable unless a limit is armed
(`missing.rs:89-91`), and armed it costs essentially nothing extra (6,240
against 6,272 physical), the opposite of brief-457's risk 3. And the target mode
is 1.27x costlier per level, not the 5x its recorded packets implied. Brackets,
all fail-open: recursive 6,272, armed 6,240, target 4,944, target with scan
`All` 4,928.

**brief-462 stop-lossed correctly and the gate found two things it had not.**

- *Its failing certificate was a bad probe.* It used an absolute
  `MemoryConstrained[…, 1000000000]` while `MemoryInUse[]` at height 4,097 is
  **2 810 469 079 bytes**, so the call aborted before running and returned
  `$Aborted`. It had copied the two-argument form from its own shallow
  companion, where 1e9 is above usage. The allowance had to be relative.
- *A default-path allocation regression of +1 per association level*, invisible
  to the worker because it never reached the cost gate. The base's
  `entries.into_iter().filter_map(…).collect()` can be satisfied **in place,
  reusing the source buffer**; the rewrite rebuilt into a fresh
  `Vec::with_capacity` and lost that on every level. The sibling list walk was
  not affected — its base iterated a borrowed slice and always allocated.
- *The forced-front falsifier the brief required was never delivered*, which
  mattered because every certificate fixture is a single chain: branch handling
  in the heap tail was untested.

**brief-463 closed all three**, and its falsifier is better than the shape I
specified: instead of a test-only cargo feature it calls the private
`*_with_front_depth` directly with a front of zero, asserts that front zero
really reports `Exhausted` and front 16 reports `Complete`, then asserts
**tail == front == expected** on a three-entry-per-level fixture with a
per-level witness. Director-verified that it fires: reversing the tail's entry
order fails it with `keep` and `k` transposed.

**Gate.** 2 379 tests green on a loopback-capable host with no ZeroMQ skip;
three release certificates at raw physical height 4,097 including the armed one;
allocation counts back to base **byte-identical** on all three probes;
instructions `-0.639%` on the recursive probe, `+0.003%` armed, `+0.001%`
control. The split landed by responsibility — `counting.rs` 14 493,
`deletion.rs` 16 929, `projection.rs` 7 348 — after brief-462's single
`traversal.rs` came in at 35 131, over the architecture-debt line.

**brief-464 planned the finite-target/resume family (design-only), and the
measurement that preceded it found a second defect.**

**A flat list of 3,000 `Missing[]` entries at the target level terminates the
process** — `DeleteMissing[Table[Missing[], {N}], 1, 0]`, stable ok <= 2,921
siblings and fail >= 2,968, same bracket for the association shape. That value
has **physical height 2**. It is not deep, not adversarial, and not near any
documented limit. brief-454 had written these edges off — *"the many `resume_*`
calls that advance `index + 1` are loops or tail transitions over siblings, not
tree height"* — and Rust does not guarantee tail-call elimination, so they are
not loops. **A reasoned claim about what the compiler will do is a hypothesis.**

The recursive mode, converted in the same file by briefs 462-463, now clears
**64,000** siblings as well as its depth certificate: one conversion closed both
of its axes.

**Three things the plan established that change how the repair must be built.**

- *The bounded-front-then-restart shape this whole line has used does not work
  here.* Every previous conversion lets the cold tail redo the traversal after
  the front exhausts. Here a target-level delayed evaluation runs user code and
  can mutate state before a later child exhausts, so a restart would re-run it.
  **The exhausted handoff must carry the live continuation and the completed
  prefixes.** That is materially harder than every conversion before it.
- *One conversion can close both axes, but only if it does both jobs.* A
  depth-only bounded front leaves the measured width edges at `:256` and `:463`
  native; converting siblings alone leaves the measured child descent at `:562`
  native. Sibling advance becomes an explicit `loop`, not a tail-call assumption.
- *The observable contract is eight items*, each cited in the plan: visit order,
  the rule-like short circuit ahead of the scan, syntactic-`Missing` drop before
  any delayed evaluation, **each reached `RuleDelayed` RHS evaluated exactly
  once**, the delayed-wrapper rule above target level, leaf and head
  preservation, what a suspension resumes into, and pure frame abandonment.

**My axis reading was corrected in both directions.** The depth edges I named
were right but incomplete (add `:383`, `:393`, `:722`, `:732`); so were the
sibling edges (eleven more). The classification rule is checkable from the
argument lists: a sibling edge keeps `level` and passes `index + 1`, a depth edge
passes `level - 1` and resets index and accumulator.

**No other registry row is dishonest, and that negative was calibrated.** Ten
other association sites share the textual `index + 1` idiom, but each
continuation lands in a function containing an explicit `loop`, established from
control flow rather than from assuming tail-call elimination. Director
spot-check at width 10,000, where `DeleteMissing`'s target mode is a stable
process kill: `Counts`, `AssociationMap` and `KeySelect` all pass 3/3. The probe
could have contradicted the survey and did not.

**Open, and needing measurement rather than argument**: the five
post-suspension functions and eight heap frames have no bracket at all, because
no supplied probe requests input. Their correctness has to come from forced-tail
multi-suspension tests, not from a number.

**Step A is IN (2026-09-05, brief-465): the semantic net is under the
conversion.** Ten tests, test-only, written against the untouched
implementation — exact value and order for both parent shapes and both `scan`
settings, `RuleDelayed` reached exactly once, the `level - 1` delayed wrapper,
resume without replay at the first, middle and last sibling, two suspensions in
different branches, non-input error propagation, and cancellation discarding
frames without callbacks.

**The falsifier method was corrected from the plan's draft, and the correction
is the point.** brief-464 proposed perturbing the tests' *expected values* and
checking they fail. That is not a falsifier: a test whose expectation you
corrupt always fails, which demonstrates the assertion is wired up and says
nothing about detection power. The brief instead named a minimal perturbation to
the **production source** per contract item, each reverted afterwards.

**Director-reproduced, and the selectivity is what matters.** Perturbing the
post-suspension resume to restart its level at index 0 turns **all four**
resume/replay tests red while the six non-suspension tests stay green. Prefix
replay on resume is the exact failure the next step's shape risks — a handoff
that carries a live continuation instead of restarting — so a net that catches
it precisely, and nothing else, is the thing this step existed to build.

Item 8 got the honest answer the brief asked for: the error-propagation half has
a behavioural perturbation, while the pure-discard half is structural, supplied
by `PureFrameAbandon`, with a cancellation test asserting no continuation
callback runs when those frames are discarded.

Gate: 2 389 tests green, `git diff` is test files and wiring only, no production
edit survived.

**Step B is IN, and `DeleteMissing` is `BoundedHeap` (2026-09-05, briefs
466-467).** Sibling advance is an explicit loop, child descent has a depth-16
front over a cold work stack, and the exhausted handoff carries live cursors and
completed prefixes. The target machine moved to
`crates/rum-stdlib/src/associations/missing/target.rs`; `missing.rs` is down to
8 113 bytes and `resume_frames.rs` to 5 573, since the eight frames collapse to
one that freezes the whole work vector.

**Both axes are closed, verified at the CLI.** Width 3,000 — which terminated
the process before — and width 64,000 both complete; depth 4,097 and 8,000 both
complete, against a previous ceiling of 4,944. Four release certificates cover
depth at both `scan` settings and 64,000 siblings in both container shapes.

**The ten semantic locks passed unmodified**, which was the acceptance criterion
and the whole reason step A existed. Allocation counts are byte-identical to
base on a shallow probe, a 2,000-sibling wide probe and an untouched control.

**The crux property turned out to be unobservable, and that is the finding.**
brief-464 identified the carried handoff as this conversion's crux: unlike every
previous conversion, a cold tail that redid the traversal could re-run user code.
At the gate, perturbing the entry point to discard its carried work and restart
from the root left **890 rum-stdlib tests green** — locks, certificates and both
in-module handoff tests. brief-467 was dispatched to close that, and stop-lossed
with a proof instead: **delayed right-hand sides are evaluated only at
`level == 1`, while front exhaustion happens only at remaining front depth zero,
which is still `level > 1`.** On the only path that has a front, exhaustion
strictly precedes any user-code evaluation. The director confirmed the other
half: `resume_delete_missing_target_work` drives entirely on the heap with no
front at all, including after a suspension resumes into it.

So a restarting handoff would redo pure structural work and be invisible. The
carried shape is still the right one — it is also cheaper — but its correctness
is **structural, not test-enforced**, and the invariant is now recorded beside
the handoff so that a change which lets user code run before an exhaustion knows
it has re-opened the question. The in-module test that claimed to cover this was
renamed to
`resume_target_work_given_a_carried_prefix_does_not_replay_it`, which is what it
actually establishes.

Gate: 2 392 tests green without the ZeroMQ skip.

**The key-reshape conversions are IN (2026-09-05, brief-468).**
`AssociationKeyFlatten` and `AssociationKeyDeflatten` are `BoundedHeap`: three
walks converted — the flatten walk, deflatten's insertion phase, and its
mutually recursive render pair — each behind its own depth-16 front over a heap
tail. Certificates at raw physical height 4,097 for both operations plus a
round trip that exercises all three walks in one value; three forced-front
falsifiers, one of which the director reproduced red by reversing the render
tail's assembly order. Allocation counts byte-identical to base on shallow
flatten, shallow deflatten, a 4,000-entry wide probe, and an untouched control.

**Width was measured clean before briefing.** All four remaining conversion
sites pass 3/3 at 1,000 through 64,000 entries, so `DeleteMissing`'s target
family was the only site with the width defect. This brief was depth-only, and
the negative was calibrated — the same harness caught `DeleteMissing` at 3,000.

**The brief told the worker to use the simple shape and not to build
`DeleteMissing`'s carried-continuation handoff**, on a fact checkable from the
signatures: none of the three walkers takes a `Session`, so none can run user
code mid-walk and a cold tail may redo the traversal. Scope is set by the
problem; the harder shape would have bought nothing.

### What the cross-comparison found (2026-09-05)

The first dispatch was interrupted mid-self-verification. Its work was kept
outside the repository and outside `/tmp` — both inside the worker's fence — and
the brief was re-dispatched from a clean tree, giving two independent
implementations of the same brief.

**They converged on the architecture**: both chose three separate front
constants rather than one shared, both produced the same three loops and the
same render work enum, within 7% on file size. That is evidence the brief was
specific enough to determine the design.

**They diverged on one thing, and it is a real defect the second caught.**
`NestedPathNode::Branch(NestedPathNodes)` is recursive, so its derived `Drop`
recurses. On the deflatten error path a partially built tree is dropped, and the
first attempt let that drop recursively while the second added
`drop_nested_path_nodes_iterative`. Director-measured on the same calibrated
probe — a deep good path followed by an empty-list key, which is the **only**
input `path_from_key` rejects: **attempt 1 terminates the process at depth
60,000 and attempt 2 completes.** The second implementation is also cleaner on a
second axis, generalising `BoundedTraversal<T>` with `Exhausted(T)` where the
first added a separate `InsertPathTraversal` enum.

**The gap is closed (director, same day).** That iterative drop was
unrequested hardening the worker added on its own; it closed a real fail-open
and had no test, so a future refactor could have reverted it silently. The
certificate now exists — a 60 000-element good path followed by an empty-list
key, on a declared 2 MiB thread, asserting the call returns unevaluated with its
argument intact. **Verified to fire**: replacing both
`drop_nested_path_nodes_iterative` calls with ordinary drops overflows the
stack in the named certificate thread.

The depth is measured rather than chosen — a recursive drop survives 20 000 and
dies at 60 000 on that thread — and the comment records the trigger, because my
own first probe measured nothing: `path_from_key` treats a non-list key as a
one-element path and rejects **only** `{}`.

**Association `MapIndexed` is `BoundedHeap` (2026-09-05, brief-469).** Three
edges — association values, the head, and arguments — behind a depth-16 front
over a heap tail, in a new `apply_map/traversal/map_indexed.rs`. The plain
certificate completes at raw physical height 4,097 with the exact rebuilt value;
the forced-front falsifier runs with `include_heads: true`, so the head edge is
covered by a tail-versus-front comparison, and the director reproduced it red by
reversing the tail's assembly order. Allocation counts byte-identical to base on
a `MapIndexed` probe and on the **`Map` and `Scan` controls that share this
file**.

**Two traps the brief named, both avoided.** `Scan` sits beside this walker and
was *guarded* by brief-461 rather than converted, because `Scan` fits the 1 MiB
boundary at contract height and `MapIndexed` does not; its three guard edges are
untouched. And the shared walker serves the generic shape too, so only the
`MapIndexed (association axis)` row flipped — the generic shortfall is
downstream on the post-result stability path and is still owed.

**One certificate I demanded could not exist, and it is the second time.** I
required a deep certificate with `Heads -> True` so the head edge would be
covered by a certificate and not only by a falsifier. It fails — and fails
**identically on the pre-conversion build**, so it is not a regression. The
reason is not the walker: the head-bearing centre is a call of undefined
symbols, which the evaluator descends as genuine evaluation levels, unlike an
`Association`, which is a data constructor and costs none. That shape refuses in
band from physical height 515 upward, long before the walker matters. The
certificate was rewritten to assert what is true and worth pinning — the
head-bearing shape must fail **closed**, never terminate the process — with the
reason recorded in the test.

brief-467 was the first instance: a test specified without checking it was
achievable. **Both times the shape was impossible for a structural reason that a
five-minute probe would have shown**, and both times the worker or the gate
found it rather than the brief. Probe a certificate's premise before demanding
it.

**The exact-path walks are converted and the conversion phase is closed
(2026-09-05, brief-470).** `AssociateTo` and `KeyDropFrom` on a `Part` target are
`BoundedHeap`: the runtime's `exact_expr_at_path`,
`collect_exact_paths_for_position_spec` and `replace_exact_path` each run behind
a depth-16 front over a heap tail. Two release certificates at raw physical
height 4,097 — one per head, since the two are not interchangeable — a shallow
companion pinning both selector spellings, and a **shared-consumer behaviour
test** the worker added on its own. Three forced-front falsifiers, one per walk;
the director reproduced the `replace_exact_path` one red by unwinding its
ancestor stack FIFO instead of LIFO, which reddens that test and only that test.

`mutation.rs` needed no change — it has no depth recursion of its own, which is
the useful negative the brief asked for. `targeted_map.rs` is untouched, and
allocation counts are byte-identical to base on an `AssociateTo` probe and on
the **`ReplacePart` and `MapAt` shared-consumer controls**.

**The axis here was path length, not value depth** — `exact_expr_at_path`
recurses once per path component and a key selector expands to more than one raw
component — so a fixture built around nesting would have measured a different
walk.

### Registry endstate after the conversion phase

44 rows: **23 `NoProportionalDescent`, 9 `BoundedHeap`, 3 `GuardedNative`, 9
`UnguardedNative`.** Gate note: 2 404 tests green.

The nine unguarded rows are **not** all the short-of-contract queue, which is
what I had assumed before reading them:

- **Four are short of the contract** and have their own queue items: `GroupBy`
  (multi-spec), and `Keys`, `KeyTake`, `KeyDrop` in the deferred
  collection-shape lane. That lane has **three** members, not the two I had been
  saying.
- **Five meet the contract but are still unbounded**: `Association`,
  `KeyComplement`, `Lookup`, `Normal`, `Values`. These are brief-454's second
  table — recursion on collection, selector and wrapper axes rather than
  association value nesting — and they are **reasoned** to clear 4,097, not
  measured. They fail open above whatever their real ceiling is, and nobody has
  bracketed them.

So the conversion phase closed every **measured** fail-open site. The residual
five are the honest remainder, and the registry enforces that each names what
owes it: a row that is `UnguardedNative` with no `owed_by` fails the test.

**Next, in the order the line's own ranking gives**: bracket those five residual
shapes — they are unmeasured, and every time this line has measured something it
had only reasoned about, the scope moved. Then the four short sites, of which
the collection-shape lane is fail-open and therefore first. Then the remaining four conversions:
`AssociationKeyFlatten`, `AssociationKeyDeflatten`, association `MapIndexed`,
and the `AssociateTo`/`KeyDropFrom` Part target. None fits the 1 MiB budget at
contract height, which is why a guard is not an option for any of them.

**Opened by brief-450 — a second recursion above the contract.** Above roughly
11,000 levels both operations return to terminating the process (in band at
10,000, killed at 12,000), while `Replace`, `DeleteCases` and `ReplaceAll` stay
in band at 12,000 and the fixture alone clears 20,001. So a different recursion
in the same two operations reaches that far. For `ReplacePart` the named
candidate is plan *construction* — `merge_replace_plans`
(`.../structure_data/replace/plan.rs:113`) recurses over a plan whose depth is
the path length, and brief-450 bounded only plan *application*. The `Take`-side
cause is unidentified. It is 2.9x above the ruled height and fails open, so it
belongs in the same class as binding substitution: a safety gap above the
contract, needing its own diagnosis, not a patch bolted onto this repair.

**Recorded for the memory-accounting line, found by accident here.** A release
test spelling `DeleteCases[x, 1, Infinity]` literally trips the `NonAmplifying`
growth assertion: `DeleteCases` is a special form, so the validator measures the
raw argument — the symbol `x`, 298 proxy bytes — while the result is the
evaluated 700 KB tree. The declaration is right and the measurement is wrong for
any special form whose argument is a symbol bound to a large value. It is
**test-only** (the check lives behind the `test-builtin-memory-growth` cargo
feature that `rum-stdlib` enables through a dev-dependency), so it is an
instrument gap rather than a user-facing defect. The certificate works around it
with `Apply`; fixing the model belongs to the growth-contract line.

**Correction to the organization-line note above.** Re-measured at `59410b75`,
only one of the three files named there is over 35k: `evaluation_cache/tests.rs`
at 38 294. `debug_tools/resources/memory.rs` is 32 019 and both `stability.rs`
files are 12 476 and 17 066, all under the line. Three files that were not named
are over it: `tests/context_and_loader.rs` 38 873, `tests/frame_root_trace.rs`
37 971, and `tests/scoping_gensym_gc.rs` 36 121, beside the generated
`frame_root_trace/derived_inventory.rs` at 162 506.
