# TODO

This file is for active or near-term work only. Completed history and
operational lessons live in [PROGRESS.md](./PROGRESS.md) (append-only log),
and release-facing changes live in [CHANGELOG.md](./CHANGELOG.md).

Status checkpoint: 2026-09-02, committed tree through `4c4dbfab`. Uncommitted
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

**What remains, and why it is not queued.** brief-427 classified brief-422
§3(e)'s tail: only the rewrite family shares the normalizer mechanism, and
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
