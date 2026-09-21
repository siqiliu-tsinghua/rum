# brief-509 — design: can `Enclose`'s internal tag stop minting text?

`Scope: NONE — no source change.` Design-only. Survey, reason, return a plan
and a drafted implementation brief. Write no code.

## Goal

brief-507 measured that an untagged `Enclose` leaks — one lazy seed plus one
interned string and pair per call, about **117-127 proxy bytes**, never
reclaimed. The allocation is
`fresh_internal_enclose_tag_expr`
(`crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation.rs:118`),
which takes the seed `System\`InternalEncloseTag` and runs it through the
shared fresh-**text** generator `allocate_fresh_symbol`
(`crates/rum-runtime/src/runtime/scoping/unique.rs:81`).

Decide whether that tag can instead be distinguished by **identity on one
shared interned pair**, and return the implementation brief if it can.

## The crux, stated so it can be refuted

A `Module` local could not simply switch, because its printed name is a
**source-resolvable** name: printing `x` with identity 3 as `x$3` does not
tell the resolver how to recreate it, and host snapshots are source text
(brief-507; `crates/rum-host/src/session.rs:185`).

`InternalEncloseTag$n` looks different in kind: the user never writes it, it
is `Internal`-prefixed, and it exists to be *compared*, not *named*. So:

> Does this tag need distinct **text**, or only a distinct **value**?

If only a value, `Symbol::with_identity(SYSTEM_CONTEXT, "InternalEncloseTag", n)`
gives distinctness with one interned pair, because identity participates in
equality (`crates/rum-core/src/expr.rs:250-254`).

**Establish the answer; do not assume it.** The way it fails is if the tag
escapes somewhere that needs to render or re-resolve it by name.

## What you must establish

### 1. Every path the tag reaches

Enumerate creation, the untagged-confirmation rewrite
(`confirmation.rs:299`), matching and selection, nested-`Enclose` boundary
behavior, failure-payload construction, message rendering, and
suspension/resume/abandon. `path:line` plus **actual `.rum` probes**, not
reasoning alone. Note that the third argument suppresses lexical tag
generation (`confirmation/builtins.rs:7`), so both forms must be probed.

### 2. Whether it can escape into anything text-shaped

The disqualifying cases, each to be answered yes or no with evidence: does
the tag ever appear in a **snapshot**, in a **user-visible failure payload**
or message, in `Names`-style introspection, or anywhere a rendered form
could be parsed back into a symbol? A tag that only ever lives inside kernel
frames and is compared by value is safe for identity; one that is rendered
into a persisted or re-resolvable form is not.

### 3. Identity lifetime and reuse

A tag can outlive an ordinary return — it can sit in a suspension frame.
Say which identity kind it should use, whether a **new** kind is required,
and whether identities may be reused after the tag's lifetime ends. **Do not
assume reuse is safe**: a reissued identity that makes a stale tag match a
live one is the same aliasing hazard that killed the reclaimable-interner
option in brief-504. If you cannot prove a lifetime bound, say the
identities must be monotonic and never reused, and state what that costs.

Do **not** treat this tag as a `Module` GC candidate merely because both use
numeric identities.

### 4. Scope discipline

The fix must be at the internal-tag call site. **Do not propose changing
`allocate_fresh_symbol` itself** — that is the shared generator and changing
it would silently change `Unique[]` and `Module` too, one of which is
deliberately fresh-named and the other blocked on a user ruling.

### 5. The corrected consumer statement

After this repair, restate which legitimate constructs still grow the
interner and at what measured rate, so the eventual cap sizing has a current
list. `Module` stays on it until its ruling lands.

## Why this one, now

It is **unblocked** while the `Module` repair waits on a user ruling, it is
on the path to sizing the quota, and if it works it proves the identity
mechanism on a low-stakes internal case before the higher-stakes one. That
last point is a benefit, **not** a licence to broaden this brief into a
general mechanism.

## Settled — do not reopen

- `Symbol` does not change: size, `Copy`-ness, id-based fast-accept
  equality, lifetime-free `'static`-compatible public type (user ruling
  2026-09-21).
- No interner reclamation, no per-session attribution.
- `Unique[]` keeps minting fresh names; that is its documented behavior.

## What I want back

1. The path inventory from §1 and the escape answers from §2, evidence-led.
2. A verdict on the crux, with what would falsify it. **A refutation is an
   acceptable answer.**
3. The identity-lifetime decision from §3, with its reuse stance argued.
4. A drafted implementation brief in `TEMPLATE.md` shape: exact files and
   functions, the test gate, a repeated-`Enclose` census gate proving the
   leak is gone, and the instruction-count neighbours for the A/B.
5. The corrected consumer statement.

## Hard rules

- **No source change.** No `.rs`, no script, no fixture, no doc except
  appending your report to this file. Scratch and `.rum` probes under
  `target/brief-509/`.
- Do NOT `git commit`, branch, switch, merge, push, stash, `git checkout` or
  `git restore`, or touch git config or the index.
- **No new `unsafe`.**
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only; no oracle harnesses, no comparisons against other vendors'
  engines.
- No network. Touch nothing outside this repo. You may read and reuse the
  census helpers and binaries under `target/brief-506/` and
  `target/brief-507/`; do not overwrite them.
- Bound anything long with `scripts/run-bounded.py --timeout SECONDS`, sized
  from a measured run of the same shape.
- **Stop-loss, keyed on progress.** If a probe fails twice and the second
  attempt produced no more information, stop and report what you have.
- Paste verbatim evidence; never quote a number you could not take.

## Report format

1. Path inventory and escape answers.
2. Verdict on the crux, and what would falsify it.
3. Identity kind, lifetime and reuse stance.
4. The drafted implementation brief.
5. Corrected consumer statement, and any director decisions you hit.


## Execution report — brief-509 (2026-09-22)

Design only; no production source change. **The confinement premise is
refuted.** The lexical tag escapes into ordinary expressions, delayed
values, failure fields, messages, traces and saved source. Its fully
qualified name resolves back to the same value. A shared interned pair
plus identity would distinguish tags in memory, but replacing only the
internal allocation call cannot preserve the observed text/value and
persistence behavior. No repaired candidate or post-repair result is claimed.

### 1. Path inventory and escape answers

#### Evidence and execution scope

The starting worktree was clean. All new scratch is under
`target/brief-509/`; the original brief is saved in `brief-prefix.txt` and
tracked-file hashes in `start-hashes.json`. Existing binaries were reused
as expressly permitted, without rebuilding or modifying them:

- `target/brief-507/build/release/rum-cli`: ordinary language probes.
- `target/brief-506/probes/consumer`: throwaway instrumented census helper;
  its source checks the returned iteration count, no messages and no
  termination, with the supported iteration-work limit set to 1,000,000.
- The existing brief-506 `rum_stdlib` and `rum_runtime` unit-test binaries:
  narrowly selected existing resume and abandonment evidence.

`target/brief-509/logs/*.json` records each invocation, exit, monotonic
elapsed duration and complete combined stdout/stderr; sibling `.log` files
contain the actual output. The binary hashes are in `binary-hashes.json`.
No new Rust helper or script file was written. Existing-source observations
below are not claims that incidental tag spelling is a normative promise;
changing the surrounding public behavior needs a decision, not an inference
from the word `Internal`.

Commands were bounded with `scripts/run-bounded.py`. The prior matching
consumer runs in `target/brief-507/run-results.json` include the 10,000-call
Enclose run at 0.624200381 seconds; the new census bound was 5 seconds per
process. The initial short CLI calibration used that conservative 5-second
ceiling and completed in 0.06303024099997856 seconds; subsequent short CLI
probes used 2 seconds, and the one REPL attempt used 3 seconds. The existing
selected tests used 5 seconds; the prior full test-binary runs are recorded
in `target/brief-506/logs/workspace.log`. Python command orchestrations also
had outer bounds. Elapsed durations were used only to bound execution, not
as performance evidence. Profiling/census variables were removed for
semantic probes/tests; only the separate census runs enabled census output.

#### Source-led inventory

The following paths include arbitrary user expression storage: there is no
finite list of user containers once the rewritten tag is an ordinary
`Expr`. Held expressions and delayed definitions are concrete witnesses,
not an assumption that every sink must have special Enclose code.

| Stage | Exact source and mechanism | Executed witness |
|---|---|---|
| Entry and creation | `crates/rum-stdlib/src/catalog/control.rs:249` registers held Enclose; `crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation/builtins.rs:7` validates arity before allocation. At `:14`, absent third argument allocates once even if no confirmation occurs; present third argument allocates no lexical tag. `crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation.rs:118` creates the seed and calls the shared fresh-name allocator. `crates/rum-runtime/src/runtime/scoping/unique.rs:81`, `:149`, `:36` increment the session counter, format fresh text, probe collisions and construct an ordinary `Symbol::new` with no identity. | `escape.rum`, `paths.rum`; census of `Enclose[1]` versus `Enclose[1, Identity, _]`. |
| Lexical rewrite | `crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation.rs:299` traverses call heads and arguments recursively. It does not stop at Hold, a delayed definition, an information argument or a function body. It appends the tag to Confirm/ConfirmAssert at index 2 and ConfirmBy/ConfirmMatch/ConfirmQuiet at index 3, inserting automatic-information and All placeholders where needed. Already explicit tags are preserved. | `paths.rum` prints all five rewritten families, both Confirm arities and all explicit-tag controls; `retained.rum` prints the tagged delayed definition. |
| Nested boundary | `crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation.rs:301` returns an entire nested System Enclose call unchanged, before descending into its head/arguments. Each subsequently evaluated implicit inner Enclose supplies its own tag; an explicit third-form inner Enclose supplies none. | `paths.rum` has held nested, inner handler and third-form boundary controls; `retained.rum` shows different tags for outer and inner held values. |
| Confirmation value and metadata | `crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation.rs:97`, `:108` evaluate optional information but clone the tag without evaluating it. `confirmation/builtins.rs:273`, `:346`, `:386`, `:436`, `:491` select the appropriate tag after value/test/match/message processing. The five finish paths at `:295`, `:370`, `:420`, `:475`, `:513` construct `EvalError::Thrown { value: failure, tag, explicit_tag: true }`. | `paths.rum` success and handler results; `messages.rum` and `escape.rum` exercise failing assertions, matches and information. |
| Enclose selection | `crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation/builtins.rs:186` only handles thrown Failure values. At `:220` implicit selection compares complete Expr values for equality. A miss propagates the same throw. At `:240` explicit third-form selection uses the ordinary matcher on the tag. `:259` returns the failure, projects a string property or calls the handler with the failure alone. | `paths.rum`: matching and mismatching explicit forms, implicit mismatch, inner handler and ordinary success. `suspend-matcher.rum` reaches a suspending third-form predicate. |
| Other handlers can see the tag | There is no private throw channel. `crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:155`, `:183` perform Catch selection; `:320` invokes a two-argument handler with the failure and tag. Equality's identity-sensitive fast path is `crates/rum-core/src/expr.rs:250`; Catch's exact atom comparison is `nonlocal.rs:333`. | `escape.rum` returns the tag from Catch inside Enclose. `stale.rum` and `retained.rum` show old tags propagating past a later implicit Enclose. |
| Failure payloads | `crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation.rs:127`, `:137`, `:147`, `:164`, `:178`, `:203` construct the six payload shapes. None adds a dedicated lexical-tag field. However, Expression, Information, Function, Pattern, Test, HeldExpression, HeldTest and MessageParameters can contain already rewritten subexpressions. No sanitization removes their tags. | `escape.rum`: Information contains the tag. `messages.rum`: Expression and the computed Message contain it. The plain assertion failure is a negative control. |
| Rendering and request failures | `crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:134` renders Failure Message; `crates/rum-runtime/src/runtime/messages/message_format.rs:165` renders message arguments. `crates/rum-runtime/src/runtime/session/session_requests.rs:738` renders an uncaught throw's tag into the host request error. `session/session_helpers.rs:454`, `:623` also render explicit tags for the top-level Throw message path. `crates/rum-core/src/expr.rs:322` uses the symbol's qualified name. | `messages.rum` prints a Failure message and emits `probe::tag`; `stale.rum` produces a host error naming the old tag. These are distinct rendering paths; the latter output is not claimed to be a `Throw::nocatch` message. |
| Return, definitions and trace | `confirmation/builtins.rs:194` returns successful bodies without removing tags. The general expression/definition/trace machinery consequently receives tagged trees, rather than an opaque private token. Saved definitions are visited by `crates/rum-runtime/src/runtime/package_tools.rs:519`. | `escape.rum`: returned Hold. `retained.rum`: DownValues retains a tag after return; Trace exposes the rewritten Confirm even on success; the third-form trace has no generated tag. |
| Names and source lookup | `crates/rum-runtime/src/runtime/namespace_tools/names.rs:275` enumerates definition/attribute/options/message/stub keys, not all interned pairs. `:186` filters sealed-private visibility, not the `Internal` prefix. `:116` returns SymbolName text. `:152` resolves selected names back to symbols. `crates/rum-runtime/src/runtime/resolution.rs:101`, `:129` resolve fully qualified System names to ordinary System symbols. | `persist.rum`: Names is empty before assigning a definition to the escaped tag, then returns `System`InternalEncloseTag$3`; Symbol and ToExpression reach the same symbol, and a name-based call returns 41. |
| Persistence | `crates/rum-host/src/session.rs:185` calls `snapshot_definitions_source`; `:201` resets and evaluates the source image. `crates/rum-runtime/src/runtime/session/session_api.rs:352`, especially `:366`–`:382`, pass ordinary definitions through `render_saved_symbols`. `crates/rum-runtime/src/runtime/package_tools.rs:284`, `:410`, `:511`, `:519` use the same saved-statement/Display route for DumpSave. An escaped tag in a user's RHS has no exclusion. Suspensions themselves are not part of the host image. | `persist.rum` writes actual tagged source under `target/brief-509/probes/saved.rum`, reloads it, and preserves the distinction of the two held values. Direct `HostSession::snapshot_to_bytes()` on this specific state was not executable through the reused CLI; the host-snapshot inclusion answer follows from the shared renderer, not a fabricated host API run. |

#### Suspension, resume and abandonment inventory

All frame locations in this paragraph are in
`crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation/builtins/resume_frames.rs`.
`EvalEncloseSuspensionFrame` (`:5`) stores handler, form and lexical_tag;
its resume at `:23` uses that stored tag, without minting a replacement.
The five initial confirmation frames (`:29`, `:50`, `:71`, `:92`, `:124`)
retain the rewritten SharedArgs. Predicate resume retains args/value
(`:259`). Information and source-failure-message resumes retain the tag
explicitly (`:189`, `:222`, `:283`, `:367`, `:400`, `:433`); the initial
ConfirmMatch matcher continuation also retains it (`:316`). Explicit
Enclose selection stores failure/tag/handler in
`ResumeEncloseForFailureSuspensionFrame` (`:145`) and at `:166` finishes or
rethrows after the resumed match. A handler that has already received the
failure can suspend in normal evaluator frames too; the tag may remain in
that failure's nested expressions.

Each of those confirmation frames derives `FrameRootTrace` and
`PureFrameAbandon`. The latter expands to Discard with no continuation
execution (`crates/rum-derive/src/lib.rs:123`, `:143`). The abandonment
worklist at `crates/rum-runtime/src/runtime/suspension/core.rs:534` drops
pure frames and separately runs designated WithCleanup cleanups; it does
not recycle symbol identities. Cancellation reaches it from
`crates/rum-runtime/src/runtime/session/session_requests.rs:46`, `:63`.
Normal input resume goes through that file's `:265`, `:295`, preserving the
pending continuation. Do not equate Session drop with that protocol:
`crates/rum-runtime/src/runtime/session/session_state.rs:696` only prints a
profiling summary in its explicit Drop implementation.

Executed `.rum` probes reached input suspension in implicit metadata,
explicit-tag metadata and explicit tag-pattern selection. Six existing
Enclose tests were then run: among them actual resume through handler,
information, predicate/test, and the Confirm family, including a second
ConfirmQuiet suspension. One existing runtime test independently checked
pure-frame abandonment and monotonic suspension generations. That test's
generations are **request/suspension generations, not proposed tag ids**.

A direct Enclose abandonment attempt via REPL did **not** execute
abandonment: after the first Get suspended, the host rejected subsequent
evaluations, including the attempted explicit-form probe. It has no REPL
resume/reset command. The rejection was returned repeatedly for commands
already queued in that one attempt; it was not retried. The attempt is
preserved below. Thus Enclose-specific cancellation/abandonment and
explicit-third-form *completion after resume* remain source-audited but
not directly demonstrated here. The later gate must exercise them via the
Rust host API. This limit does not affect the already witnessed escapes.

#### Explicit answers to the disqualifying questions

| Question | Answer and evidence |
|---|---|
| Can the tag appear in a snapshot? | **Yes, through escaped definitions.** `persist.rum` and its actual saved source use the same ordinary saved-statement renderer that host snapshots call. Frame state itself is not snapshotted. Host API provenance and the absence of a direct tag-specific host snapshot run are stated above. |
| Can it appear in a user-visible failure payload? | **Yes.** Information in `escape.rum`, Expression in `messages.rum`. No special Tag field is needed. |
| Can it appear in a message? | **Yes.** The computed Failure Message, `probe::tag`, and the uncaught request error all show it. |
| Can it appear in Names-style introspection? | **Yes, after the escaped symbol acquires a definition.** An otherwise unused tag is absent; Names does not enumerate the whole interner. DownValues and Trace expose it without such a tag definition. |
| Can rendered text be parsed/resolved back into the symbol? | **Yes.** Fully qualified Symbol/ToExpression round-trips are True and a text-directed call reaches its definition. The first unqualified ToString round-trip was False because the unregistered System symbol's short spelling is resolved in Global; that negative control is not evidence of opacity. |

#### Verbatim semantic evidence

Each block below contains the complete executed `.rum` source followed by
its complete combined output. For outputs containing trailing prompt spaces,
the saved JSON execution record is pasted verbatim so the exact whitespace
is preserved without trailing Markdown whitespace. `stale` deliberately ends in an uncaught
throw; the three `suspend` probes deliberately end in InputRequired. Their
exit 1 results are observations, not successful completion assertions.
`messages.rum`'s initial ReleaseHold attempt returned the held expression;
only the subsequent `stale.rum` Part extraction demonstrated the uncaught
throw. No claim relies on ReleaseHold releasing it.

`target/brief-509/probes/escape.rum` (process exit 0):

```text
Print["held=", Enclose[Hold[Confirm[x]]]];
Print["third-held=", Enclose[Hold[Confirm[x]], Identity, _]];
escaped = Enclose[Catch[Confirm[Missing["bad"]], _, Function[{value, tag}, tag]]];
Print["catch-tag=", escaped];
Print["catch-roundtrip=", SameQ[escaped, ToExpression[ToString[escaped, InputForm]]]];
Print["names=", Names["System`InternalEncloseTag*"]];
f = Enclose[ConfirmAssert[False, Hold[Confirm[x]]]];
Print["failure=", f];
Print["failure-info=", f["Information"]];
```

```text
held=Hold[Confirm[x, InternalConfirmAutomaticInfo, InternalEncloseTag$1]]
third-held=Hold[Confirm[x]]
catch-tag=InternalEncloseTag$2
catch-roundtrip=False
names={}
failure=Failure["ConfirmationFailed", <|"MessageTemplate" -> "Assertion `` failed.", "MessageParameters" -> {False}, "ConfirmationType" -> "ConfirmAssert", "Expression" -> False, "Information" -> Hold[Confirm[x, InternalConfirmAutomaticInfo, InternalEncloseTag$3]], "Test" -> False, "HeldTest" -> Hold[False]|>]
failure-info=Hold[Confirm[x, InternalConfirmAutomaticInfo, InternalEncloseTag$3]]
Null
```

`target/brief-509/probes/paths.rum` (process exit 0):

```text
Print["success=", {Enclose[Confirm[7]], Enclose[ConfirmBy[2, IntegerQ]], Enclose[ConfirmMatch[2, _Integer]], Enclose[ConfirmAssert[True]]}];
Print["handlers=", {Enclose[Confirm[Missing["bad"]], "ConfirmationType"], Enclose[ConfirmAssert[False], Function[f, f["ConfirmationType"]]], Enclose[ConfirmQuiet[First[f[]]], "ConfirmationType"]}];
Print["all-held=", Enclose[Hold[Confirm[x], Confirm[x, info], ConfirmBy[x, p], ConfirmMatch[x, _Integer], ConfirmQuiet[x], ConfirmAssert[x]]]];
Print["explicit-preserved=", Enclose[Hold[Confirm[x, info, userTag], ConfirmBy[x, p, info, userTag], ConfirmMatch[x, p, info, userTag], ConfirmQuiet[x, All, info, userTag], ConfirmAssert[x, info, userTag]]]];
Print["nested-held=", Enclose[Hold[Confirm[x], Enclose[Hold[Confirm[y]]]]]];
Print["nested-selection=", Enclose[Enclose[ConfirmAssert[False], Function[f, "inner"]], Function[f, "outer"]]];
Print["explicit-selection=", Enclose[ConfirmAssert[False, info, userTag], "Information", userTag]];
Print["explicit-miss=", Catch[Enclose[ConfirmAssert[False, info, otherTag], Identity, userTag], _, Function[{value, tag}, tag]]];
Print["lexical-miss=", Catch[Enclose[ConfirmAssert[False, info, userTag]], _, Function[{value, tag}, tag]]];
Print["third-no-rewrite=", Enclose[ConfirmAssert[False], "Information", _]];
Print["third-default-tag=", Enclose[Catch[ConfirmAssert[False], _, Function[{value, tag}, tag]], Identity, _]];
Print["outer-around-third=", Enclose[Enclose[ConfirmAssert[False], Function[f, "third-inner"], _], Function[f, "outer"]]];
```

```text
success={7, 2, 2, Null}
handlers={"Confirm", "ConfirmAssert", "ConfirmQuiet"}
all-held=Hold[Confirm[x, InternalConfirmAutomaticInfo, InternalEncloseTag$8], Confirm[x, info, InternalEncloseTag$8], ConfirmBy[x, p, InternalConfirmAutomaticInfo, InternalEncloseTag$8], ConfirmMatch[x, _Integer, InternalConfirmAutomaticInfo, InternalEncloseTag$8], ConfirmQuiet[x, All, InternalConfirmAutomaticInfo, InternalEncloseTag$8], ConfirmAssert[x, InternalConfirmAutomaticInfo, InternalEncloseTag$8]]
explicit-preserved=Hold[Confirm[x, info, userTag], ConfirmBy[x, p, info, userTag], ConfirmMatch[x, p, info, userTag], ConfirmQuiet[x, All, info, userTag], ConfirmAssert[x, info, userTag]]
nested-held=Hold[Confirm[x, InternalConfirmAutomaticInfo, InternalEncloseTag$10], Enclose[Hold[Confirm[y]]]]
nested-selection=inner
explicit-selection=info
explicit-miss=otherTag
lexical-miss=userTag
third-no-rewrite=Null
third-default-tag=InternalConfirmTag
outer-around-third=third-inner
Null
```

`target/brief-509/probes/persist.rum` (process exit 0):

```text
first = Enclose[Hold[Confirm[x]]];
second = Enclose[Hold[Confirm[x]]];
tag = Enclose[Catch[Confirm[Missing["bad"]], _, Function[{value, tag}, tag]]];
Print["distinct-before=", SameQ[first, second]];
Print["fully-qualified-roundtrip=", SameQ[tag, ToExpression[StringJoin["System`", SymbolName[tag]]]]];
Print["symbol-roundtrip=", SameQ[tag, Symbol[StringJoin["System`", SymbolName[tag]]]]];
Print["names-before=", Names["System`InternalEncloseTag*"]];
With[{t = tag}, t[] := 41];
Print["names-after=", Names["System`InternalEncloseTag*"]];
Print["target-by-name=", ToExpression[StringJoin["System`", SymbolName[tag], "[]"]]];
Print["dump=", DumpSave[File["/home/remi/dev/rum/target/brief-509/probes/saved.rum"], {first, second, tag, "System`InternalEncloseTag*"}]];
Clear[first, second, tag];
Get["/home/remi/dev/rum/target/brief-509/probes/saved.rum"];
Print["restored=", {first, second, tag}];
Print["distinct-after=", SameQ[first, second]];
```

```text
distinct-before=False
fully-qualified-roundtrip=True
symbol-roundtrip=True
names-before={}
names-after={"System`InternalEncloseTag$3"}
target-by-name=41
dump={"Global`first", "Global`second", "Global`tag"}
restored={Hold[Confirm[x, InternalConfirmAutomaticInfo, InternalEncloseTag$1]], Hold[Confirm[x, InternalConfirmAutomaticInfo, InternalEncloseTag$2]], InternalEncloseTag$3}
distinct-after=False
Null
```

`target/brief-509/probes/messages.rum` (process exit 0):

```text
Print["plain-failure=", Enclose[ConfirmAssert[False]]];
Print["raw-expression=", Enclose[ConfirmMatch[Hold[Confirm[x]], _Integer], "Expression"]];
Print["failure-message=", Enclose[ConfirmMatch[Hold[Confirm[x]], _Integer], "Message"]];
probe::tag = "escaped ``";
Enclose[Message[probe::tag, Hold[Confirm[x]]]];
held = Enclose[Hold[ConfirmAssert[False]]];
ReleaseHold[held]
```

```text
plain-failure=Failure["ConfirmationFailed", <|"MessageTemplate" -> "Assertion `` failed.", "MessageParameters" -> {False}, "ConfirmationType" -> "ConfirmAssert", "Expression" -> False, "Information" -> Null, "Test" -> False, "HeldTest" -> Hold[False]|>]
raw-expression=Hold[Confirm[x, InternalConfirmAutomaticInfo, InternalEncloseTag$2]]
failure-message=Hold[Confirm[x, InternalConfirmAutomaticInfo, InternalEncloseTag$3]] does not match Blank[Integer].
! probe::tag: escaped Hold[Confirm[x, InternalConfirmAutomaticInfo, InternalEncloseTag$4]]
Hold[ConfirmAssert[False, InternalConfirmAutomaticInfo, InternalEncloseTag$5]]
```

`target/brief-509/probes/stale.rum` (process exit 1):

```text
held = Enclose[Hold[ConfirmAssert[False]]];
Print["stale-selection=", Catch[Enclose[held[[1]], Function[f, "wrong-live-handler"]], _, Function[{value, tag}, tag]]];
held[[1]]
```

```text
stale-selection=InternalEncloseTag$1
! uncaught throw: Failure["ConfirmationFailed", Association[Rule["MessageTemplate", "Assertion `` failed."], RuleDelayed["MessageParameters", List[HoldForm[False]]], Rule["ConfirmationType", "ConfirmAssert"], RuleDelayed["Expression", False], Rule["Information", Null], RuleDelayed["Test", False], Rule["HeldTest", Hold[False]]]] with tag InternalEncloseTag$1
error: uncaught throw: Failure["ConfirmationFailed", Association[Rule["MessageTemplate", "Assertion `` failed."], RuleDelayed["MessageParameters", List[HoldForm[False]]], Rule["ConfirmationType", "ConfirmAssert"], RuleDelayed["Expression", False], Rule["Information", Null], RuleDelayed["Test", False], Rule["HeldTest", Hold[False]]]] with tag InternalEncloseTag$1
```

`target/brief-509/probes/retained.rum` (process exit 0):

```text
Print["nested-values=", Enclose[{Hold[Confirm[x]], Enclose[Hold[Confirm[y]]]}]];
Enclose[saved[] := ConfirmAssert[False]];
Print["delayed-definition=", DownValues[saved]];
Print["delayed-stale=", Catch[Enclose[saved[], Function[f, "wrong-live-handler"]], _, Function[{value, tag}, tag]]];
Print["trace=", Trace[Enclose[Confirm[7]]]];
Print["third-trace=", Trace[Enclose[Confirm[7], Identity, _]]];
```

```text
nested-values={Hold[Confirm[x, InternalConfirmAutomaticInfo, InternalEncloseTag$1]], Hold[Confirm[y, InternalConfirmAutomaticInfo, InternalEncloseTag$2]]}
delayed-definition={HoldPattern[saved[]] :> ConfirmAssert[False, InternalConfirmAutomaticInfo, InternalEncloseTag$3]}
delayed-stale=InternalEncloseTag$3
trace={Hold[Enclose[Confirm[7]]], {{Hold[Confirm[7, InternalConfirmAutomaticInfo, InternalEncloseTag$5]], Hold[7]}, Hold[Enclose[Confirm[7]]], Hold[7]}}
third-trace={Hold[Enclose[Confirm[7], Identity, _]], {Hold[Confirm[7]], Hold[7]}, Hold[7]}
Null
```

`target/brief-509/probes/suspend.rum` (process exit 1):

```text
Enclose[Confirm[Missing["bad"], InputString["info: "]], "Information"]
```

Verbatim JSON execution record (preserves trailing prompt spaces):

```json
{
  "label": "suspend",
  "command": [
    "scripts/run-bounded.py",
    "--timeout",
    "2",
    "--",
    "target/brief-507/build/release/rum-cli",
    "--file",
    "target/brief-509/probes/suspend.rum"
  ],
  "exit": 1,
  "elapsed_seconds": 0.06684260400015773,
  "output": "! input required: info: \nerror: input required: info: \n"
}
```

`target/brief-509/probes/suspend-explicit.rum` (process exit 1):

```text
Enclose[Confirm[Missing["bad"], InputString["explicit info: "], userTag], "Information", userTag]
```

Verbatim JSON execution record (preserves trailing prompt spaces):

```json
{
  "label": "suspend-explicit",
  "command": [
    "scripts/run-bounded.py",
    "--timeout",
    "2",
    "--",
    "target/brief-507/build/release/rum-cli",
    "--file",
    "target/brief-509/probes/suspend-explicit.rum"
  ],
  "exit": 1,
  "elapsed_seconds": 0.06061020500055747,
  "output": "! input required: explicit info: \nerror: input required: explicit info: \n"
}
```

`target/brief-509/probes/suspend-matcher.rum` (process exit 1):

```text
Enclose[ConfirmAssert[False, info, userTag], "Information", _?(Function[tag, SameQ[InputString["select: "], "yes"]])]
```

Verbatim JSON execution record (preserves trailing prompt spaces):

```json
{
  "label": "suspend-matcher",
  "command": [
    "scripts/run-bounded.py",
    "--timeout",
    "2",
    "--",
    "target/brief-507/build/release/rum-cli",
    "--file",
    "target/brief-509/probes/suspend-matcher.rum"
  ],
  "exit": 1,
  "elapsed_seconds": 0.03084912499980419,
  "output": "! input required: select: \nerror: input required: select: \n"
}
```

Actual `target/brief-509/probes/saved.rum`:

```text
Set[first, Hold[Confirm[x, InternalConfirmAutomaticInfo, InternalEncloseTag$1]]]

Set[second, Hold[Confirm[x, InternalConfirmAutomaticInfo, InternalEncloseTag$2]]]

Set[tag, InternalEncloseTag$3]
```

REPL abandonment attempt sources (the explicit file was queued but did not execute):

`target/brief-509/probes/abandon-lexical.rum`:

```text
cleanup = 0; later = 0; handled = 0;
Enclose[WithCleanup[heldBefore = Hold[Confirm[x]]; InputString["lexical pause: "]; later = 1, cleanup = cleanup + 1], Function[f, handled = 1]]
```

`target/brief-509/probes/abandon-explicit.rum`:

```text
cleanup = 0; later = 0; handled = 0;
Enclose[WithCleanup[heldBefore = Hold[Confirm[x]]; InputString["explicit pause: "]; later = 1, cleanup = cleanup + 1], Function[f, handled = 1], userTag]
```

Verbatim REPL JSON execution record, exit 0 (terminal CRLF and prompt spaces preserved by JSON escaping):

```json
{
  "label": "abandon-repl",
  "command": [
    "scripts/run-bounded.py",
    "--timeout",
    "3",
    "--",
    "target/brief-507/build/release/rum-cli"
  ],
  "input": "Get[\"/home/remi/dev/rum/target/brief-509/probes/abandon-lexical.rum\"]\nPrint[\"lexical-abandoned=\", {cleanup, later, handled, heldBefore}]\nGet[\"/home/remi/dev/rum/target/brief-509/probes/abandon-explicit.rum\"]\nPrint[\"explicit-abandoned=\", {cleanup, later, handled, heldBefore}]\n:quit\n",
  "exit": 0,
  "elapsed_seconds": 0.06151918999967165,
  "output": "Get[\"/home/remi/dev/rum/target/brief-509/probes/abandon-lexical.rum\"]\r\nPrint[\"lexical-abandoned=\", {cleanup, later, handled, heldBefore}]\r\nGet[\"/home/remi/dev/rum/target/brief-509/probes/abandon-explicit.rum\"]\r\nPrint[\"explicit-abandoned=\", {cleanup, later, handled, heldBefore}]\r\n:quit\r\nrum> ! Get::load: input required: lexical pause: \r\n$Failed\r\nrum> ! cannot start a new evaluation while the session is waiting for host input; call resume_with_input or reset first\r\nrum> ! cannot start a new evaluation while the session is waiting for host input; call resume_with_input or reset first\r\nrum> ! cannot start a new evaluation while the session is waiting for host input; call resume_with_input or reset first\r\nrum> "
}
```

Exact existing-binary invocation:

```text
scripts/run-bounded.py --timeout 5 -- target/brief-506/build/debug/deps/rum_stdlib-c8c5a89643371dc7 enclose --nocapture --test-threads=1
```

Verbatim output:

```text

running 6 tests
test tests::holding_and_control_part8::confirm_information_can_suspend_and_resume_inside_enclose ... ok
test tests::holding_and_control_part8::confirm_predicate_and_test_can_suspend_and_resume_inside_enclose ... ok
test tests::holding_and_control_part8::enclose_handler_can_suspend_and_resume_through_host_input ... ok
test tests::incidental_holding_and_control_part8::confirm_family_info_can_suspend_and_resume_inside_enclose ... ok
test tests::strings::incidental_runtime::failure_messages::failure_and_enclose_confirm_family_cover_first_pass_error_flow ... ok
test tests::structured_exceptions::cleanup_runs_and_enclose_does_not_intercept_exceptions ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 978 filtered out; finished in 0.03s

```

Exact existing-binary invocation:

```text
scripts/run-bounded.py --timeout 5 -- target/brief-506/build/debug/deps/rum_runtime-562b258c6eef9b02 abandonment_generations_are_monotonic_and_pure_frames_run_no_continuation --nocapture --test-threads=1
```

Verbatim output:

```text

running 1 test
test runtime::tests::suspension_abandonment::abandonment_generations_are_monotonic_and_pure_frames_run_no_continuation ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 998 filtered out; finished in 0.00s

```

### 2. Verdict on the crux, and falsifiers

**Distinct value is sufficient for selection; it is insufficient for the
whole present behavior.** Equality already includes identity, and the
confirmation selection code does not intrinsically need fresh text.
Nevertheless, this implementation embeds the tag into user expressions
before evaluation. Hold, Catch, delayed definitions and Trace refute the
claim that it stays in kernel frames. No guessed name, malformed call,
third-party oracle or private Rust API was needed to extract it.

A literal call-site switch to
`Symbol::with_identity(SYSTEM_CONTEXT, "InternalEncloseTag", n)` makes
successive live tags unequal while all of their printed names are
`InternalEncloseTag`: `qualified_name` ignores identity
(`crates/rum-core/src/expr.rs:174`) and Display delegates to it (`:322`).
Serializing the two distinct held values from `persist.rum` through today's
source renderer would lose their distinction. Fully qualified resolution
returns `Symbol::system(name)` with no identity
(`crates/rum-runtime/src/runtime/resolution.rs:129`). Even adding a rendered
`$n` suffix would not teach this resolver to reconstruct that identity.
Raw Symbol serde at `crates/rum-core/src/expr.rs:288` does preserve identity,
but is not the host snapshot format.

Therefore **do not dispatch a call-site-only behavior-preserving repair on
this evidence**. The desired constant-space representation is viable; the
proposed justification for enabling it without a naming/persistence ruling
is not. This is a refutation permitted by the brief, not a claim that the
larger problem has no possible solution.

What would overturn that conclusion: demonstrate, on the approved
semantics, a complete barrier preventing these same held/Catch/definition/
failure/trace escapes, or an approved, lossless way to encode and recover
escaped tag identity that does not retain a growing alias registry. The
present call site has neither. A ruling that intentionally changes the
observed surfaces changes the problem's contract; it does not retroactively
make these probes safe or prove snapshot correctness. Merely filtering
Names, hiding a printed suffix, skipping one held head, or testing only
`Enclose[1]` does not address the other escapes. Do not broaden this report
into that redesign or alter `allocate_fresh_symbol`.

### 3. Identity kind, lifetime and reuse stance

No new allocator is enabled by this report. For a future approved same-pair
implementation, the smallest representation choice is a **dedicated
Enclose allocation domain on the fixed System pair**, using the existing
nonzero Symbol identity field. It is not a Module gensym, sealed private
or rule-RHS alpha-renamed symbol.

A new public Symbol type, enum variant or global high-bit marker is **not
required just to distinguish these values**. The pair plus nonzero id is
already a complete key. A private Enclose counter with a checked nonzero
payload below bit 61 would keep bits 61–63 clear: this avoids sealed bit 62
(`crates/rum-runtime/src/runtime/scoping/packages.rs:8`), alpha bit 63
(`crates/rum-runtime/src/runtime/substitution/binding_substitution.rs:29`),
and does not commandeer brief-507's proposed, still unimplemented Module
bit-61 region. Name-removal generations do not occupy this System pair:
`crates/rum-runtime/src/runtime/definitions/definition_clear.rs:217`
returns early for System, and `resolution.rs:129` resolves System names
without consulting generations. Ordinary lookalikes have identity None.
Thus it needs a separate **allocator domain**, not a representation change
or reuse of another family's allocator. If the eventual persistence design
needs global kind classification, that is an additional decision outside
this rejected call-site patch, not a reason to label these tags Module.

**Reuse: forbidden.** There is no proved lifetime bound. The tag survives
an ordinary return in Hold and DownValues, can survive a pending input
frame, and can be saved as source text. `stale.rum` and `retained.rum`
actually execute an old tagged confirmation under a later Enclose: the
later handler must not select it. Reissuing that identity would make the
old throw compare equal and invoke the wrong handler. Dropping a frame,
successful completion, cancellation, rollback, gensym sweep and session
reset cannot establish that every external copy is gone. Symbol is Copy
and the runtime accepts host expressions; an Enclose-local free list or a
resettable per-session counter is not a lifetime proof.

For allocator-issued ids, use a **process-wide monotonic, checked counter**,
never reset/restore/recycle it, and fail closed before exhaustion rather
than wrapping. A counter on the Enclose path adds fixed state and one
checked allocation operation per implicit Enclose; the existing identity
field adds no Symbol bytes. There is one retained interned pair/name after
warmup, not a retained string/pair per call. `with_identity` still enters
the interner mutex (`crates/rum-core/src/expr.rs:80`, `:118`); no speed or
instruction-count improvement is asserted. These are structural cost
claims, not measured candidate results. The proposed finite id range
consumes numbers permanently, not heap entries; its recoverable exhaustion
outcome must be specified before implementation.

Process monotonicity does not solve persistence/import by itself. A text
snapshot across a process restart must preserve internal sharing and
separation while relinking to newly allocated ids, or use another explicitly
approved representation. Raw host `with_identity` construction and serde
can supply arbitrary ids (`crates/rum-core/src/expr.rs:118`, `:312`); an
allocator cannot promise freshness against unchecked imported future ids.
Define validation/reservation/relinking at any accepted import boundary
before making that stronger claim. No such mechanism is implemented here.

Do not register these tags in Module GC. That GC uses activation membership
and full-Symbol candidate sets (`crates/rum-runtime/src/runtime/scoping/lexical/module.rs:54`,
`:329`; `crates/rum-runtime/src/runtime/scoping/gensym_gc.rs:9`), not the
mere presence of an identity. Its cleanup cannot reclaim process-global
interner text or prove that an exported tag has died.

### 4. Drafted implementation brief

#### Draft — same-pair Enclose tags, blocked at the escape gate

**Status: NOT DISPATCHABLE as a behavior-preserving call-site-only repair.**
This TEMPLATE-shaped draft records the exact proposed edit and acceptance
gates rather than presenting a knowingly incomplete change as ready. Its
first prerequisite fails on the current tree. The director must settle the
escape/persistence contract and rewrite the brief with any newly required
files before dispatch. No such ruling is assumed here.

##### Goal

Remove fresh interned text from implicit Enclose's internal-tag allocation
while preserving approved confirmation selection, nested boundaries,
externalized tag behavior and host lifecycle/persistence. After fixed
setup, repeated implicit Enclose must add no strings, pairs or retained
interner bytes per call. Explicit third-form Enclose must remain allocation
free with respect to lexical tags. No change to Unique or Module allocation.

##### Context pointers

- `AGENTS.md`, `docs/testing-strategy.md`,
  `docs/host-embedding-and-hardening.md:95`, this report's escape witnesses.
- `crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation.rs`:
  `fresh_internal_enclose_tag_expr`, `rewrite_untagged_confirms`, failure builders.
- `crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation/builtins.rs`:
  `eval_enclose`, selection, metadata and failure continuations.
- `crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation/builtins/resume_frames.rs`:
  retained tags and PureFrameAbandon contract; read-only unless a rewritten
  approved brief names a required continuation change.
- `crates/rum-core/src/expr.rs`: representation, equality, Display and serde,
  read-only. `scoping/unique.rs`: shared fresh-text allocator, read-only.
- Host snapshot and name-resolution call sites inventoried in section 1.
  These are blockers, not implied permission to edit them.

##### Scope

If the prerequisite has been independently resolved and this scope is still
sufficient, proposed writable production files are exactly:

1. `crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation.rs`:
   replace only `fresh_internal_enclose_tag_expr`'s allocation, adding the
   private checked Enclose-domain allocator here or in the explicitly named
   new sibling `confirmation/tag_identity.rs` if a real module boundary is
   needed. Keep a single fixed pair; no formatted candidate names or
   text-based collision probes.
2. `crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation/builtins.rs`:
   `eval_enclose` only, for propagating the approved recoverable identity
   exhaustion error if the helper becomes fallible. Allocate exactly once,
   only when form is absent, before the existing rewrite. Do not change
   matching, handlers or continuation shape to make a gate pass.
3. `crates/rum-stdlib/src/tests.rs` and new
   `crates/rum-stdlib/src/tests/enclose_tag_identity.rs`: register and own
   focused language/identity/suspension regression tests.
4. `crates/rum-host/src/tests/session_lifecycle.rs`: actual host snapshot,
   restore, reset and cancellation tests for escaped tags.

May append only to the dispatched brief; use its own `target/brief-NNN/`
for census instrumentation copies, builds, `.rum` sources and evidence.
Do not modify Symbol representation/equality, the shared allocator,
Unique, Module, GC provenance, parser/resolver, serializer, interner,
quota/defaults, `.git`, Cargo.lock, other docs or scripts. **If the approved
contract needs any of those files, this scope is insufficient: stop and
have the director rewrite the brief.** In particular the current snapshot
and resolver defects of the proposed replacement cannot be fixed within
this list. A new public error variant or documentation change also requires
an explicit scope revision, not a silent expansion.

##### Deliverable

- Before editing production code, record the approved disposition of the
  held value, Catch tag, delayed definition, Names, Failure Message,
  Trace and source-image witnesses. With current semantics and current
  serializers, report the failed prerequisite and stop.
- If a revised brief resolves that prerequisite, implement the exact
  internal call-site replacement, checked lifetime domain and specified
  exhaustion behavior. Never modify `allocate_fresh_symbol`.
- Regression coverage and complete raw verification output, fresh-process
  census endpoints, same-process warm-repeat evidence, and instruction
  A/B target/neighbor rows. No performance or leak-free claim on tests alone.

##### rum / Rust pitfalls (read twice)

1. Third argument means no lexical allocation or rewrite. An explicit-form
   loop alone cannot prove the automatic leak was fixed.
2. Return and cancellation do not bound tag lifetime. Old held/delayed
   confirmations must miss a new implicit handler, including after reset
   or round-tripping under the approved host contract.
3. Failure builders need no explicit tag field for a tag to escape: raw
   subexpressions and information already carry it. Catch receives it
   directly, and Trace can expose it on success.
4. No fresh text in a display cache, collision probe or permanent alias map;
   that would reinstate or relocate the growth. Do not disguise persistence
   failure by switching a host test to Symbol serde.
5. Preserve every existing suspension frame and resume path. Do not mint a
   second tag on resume or run a handler/body suffix on pure abandonment.
6. No unsafe, representation change or per-call diagnostic work. Census
   instrumentation belongs only in a separate throwaway build.

##### Verification (MANDATORY — run it; iterate to green / evidence)

**Semantic gate.** Convert the section-1 witnesses into focused assertions
against the approved contract, keeping their actual distinctions rather
than asserting suffix numbers. Cover every confirmation family, normal and
handler returns, explicit form match/miss, explicit Confirm tags under
implicit Enclose, nested lexical boundaries, caught/exported tags, stale
held/delayed confirmations, failure fields/messages, Names targeting and
Trace. Add direct allocator tests for shared interned id but unequal full
Symbols, same id copied unchanged through frames, no third-form allocation,
checked exhaustion and no reuse across allocation lifetime boundaries.

Drive `HostSession::eval` and `resume_with_input` directly for both forms:
body, information, predicate, source-Failure Message, Enclose handler and
third-form matcher; include two successive suspensions, matching and
nonmatching resumed patterns. While suspended, cancel/reset/drop as
applicable to the documented lifecycle and prove no Enclose handler/body
suffix runs during pure abandonment. Check a saved user value with the old
tag after completion/cancellation. Do not substitute the REPL attempt in
this report for those assertions.

Use actual `snapshot_to_bytes`/`restore_from_bytes` for two distinct escaped
tags plus two references sharing one tag, tags nested in failure fields,
held/delayed values and definitions keyed by an escaped tag. Verify identity
relationships after restore, stale-versus-fresh selection and another new
Enclose after restore. Current Display alone cannot pass this gate. The
suite must expose that failure, not weaken it.

Run bounded focused tests, then bounded `cargo fmt --check` and
`cargo test --workspace`; paste real outputs. This crosses runtime, stdlib
and host behavior. Pick command bounds from recorded same-shape runs;
no network, no native-transport escalation. Missing environmental coverage
is reported as missing.

**Census gate.** Use fresh processes for the exact fixed-source workloads
`Do[Enclose[1], {censusI, 1, N}]; N` at N=0, 1,000 and 10,000, and the
parallel `Enclose[1, Identity, _]` control. Use the brief-506 diagnostic
method and a newly built candidate helper, not the old base helper labeled
as candidate. Record completion, strings, pairs, payload bytes, capacities,
retained proxy lower/upper and growth-overlap peak. Measure warm checkpoints
after the first Enclose and successive batches in one session as well:
string/pair/retained-byte totals must remain exactly flat after fixed setup.
One lazy shared pair at first use is allowed and must be reported; a
per-call slope, a large fixed preallocated name pool masking the short run,
or a growing side table fails. Check both successful calls and caught
confirmation failures/handlers so the allocator's useful path is covered.
Reproduce base rows rather than inferring them from Names. The exact raw
base rows are in section 5. No candidate savings are measured in this brief.

**Instruction-count A/B and neighbors.** Use the existing rum-vs-rum
instrument from `docs/dev/codex/brief-503-split-the-fourteen-percent.md:102`:

```text
scripts/run-bounded.py --timeout <measured-bound> -- taskset -c 2 perf stat -e instructions:u -x, -- <plain-release-cli> --file <workload>.rum < /dev/null
```

Base and candidate use the same workspace build path, Rust toolchain,
profile and setup; copy aside the base executable before edits, without
switching branches. Census and allocation/profiling instrumentation are
absent. Require the supported `cpu_core/instructions/u` event at 100.00
coverage, correct output, zero exit and no resource-limit truncation. Take
three repeats per arm of full, half and matching setup controls; publish
raw counts, median, within-arm spread and both gross and setup-subtracted
counts. Lack of perf access is a missing gate, not permission to substitute
an estimated instruction result.

Seven target/control statement families (names below are proposed new
probe labels, not claims that the existing panel contains them):

| Proposed label | Exact body |
|---|---|
| enclose-empty | `Enclose[1]` |
| enclose-confirm-success | `Enclose[Confirm[1]]` |
| enclose-confirm-failure | `Enclose[ConfirmAssert[False]]` |
| enclose-property-handler | `Enclose[ConfirmAssert[False], "ConfirmationType"]` |
| enclose-nested | `Enclose[Enclose[ConfirmAssert[False]]]` |
| enclose-held-escape | `Enclose[Hold[Confirm[x]]]` |
| enclose-explicit-control | `Enclose[ConfirmAssert[False, Null, stableTag], Identity, stableTag]` |

Eight unchanged panel neighbors from
`scripts/core-language-microbench.sh`: `module-local`, `block-local`,
`with-local`, `if-true`, `sameq-list`, `literal-integer`,
`symbol-ownvalue-read`, `replaceall-simple`. Reuse their exact setup and
bodies, especially `Module[{x = 1}, x + 1]`,
`Block[{xOwn = 1}, xOwn + 1]` and `With[{x = 1}, x + 1]` at `:497`, `:508`,
`:512`. These check the untouched fresh-name consumer, nearby control/
scoping work, equality and ordinary evaluation/rewriting. Calibrate loop
sizes before the matrix; do not add a sink assignment or Print around a
measured body. Liveness output/checks belong outside that body. Fifteen
families times at most four full/half/control shapes, three repeats and two
arms caps this matrix at 360 processes; share identical controls.

Also retain AGENTS.md's required base/candidate
`scripts/core-language-microbench.sh` per-label `timings.tsv` gate, with
explicit output directories under the assigned scratch root, no concurrent
CPU-heavy work and in-kernel AbsoluteTiming. It supplements the named
instruction gate; neither an average win nor a timing result can erase an
instruction neighbor regression or a missing instruction gate. The director
may explicitly select the interner lane's previously ruled instruction-only
policy when rewriting the blocked draft; this report executes no candidate
performance gate and makes no new ruling on that policy.

##### Hard rules

- Never modify the shared fresh-text allocator, Unique or Module to fix
  this tag. Preserve Symbol layout, Copy, equality and public shape.
- No source changes until the escape/persistence prerequisite is actually
  resolved in a rewritten brief. Do not assume `Internal` is an opacity rule.
- No git mutation: no commit, branch, switch, merge, push, stash,
  checkout/restore, index or git-config writes. Leave changes uncommitted.
- No unsafe, network, vendor oracle, interner reclamation, per-session
  attribution, quota/default selection or outside-repo writes.
- Scope is closed. Exact source, script and documentation additions need
  explicit listing. Scratch belongs under the new brief's target directory.
- Bound long/child-spawning commands from measured same-shape data; stop
  after two failures if the second adds no information. Preserve failures.
- Report any target/neighbor regression, failed contract or missing gate;
  no weakening assertions to call the candidate complete.

##### Report format

1. Per-file change and the approved escape/lifetime/persistence contract,
   or the failed prerequisite before editing.
2. Verbatim semantic/host tests, stale/exhaustion cases, census checkpoints,
   instruction target/neighbor rows and any required panel rows.
3. Whether the automatic per-call interner slope is actually zero, with
   side-table growth excluded and fixed setup explicitly charged.
4. Stops, regressions, coverage limitations and remaining director decisions.

### 5. Corrected consumer statement and director decisions

**No repair was made. Enclose remains a legitimate automatic interner
consumer.** In freshly rerun census processes, the third-form control is
flat; implicit Enclose still adds one lazy seed plus one string/pair per
call. The 10,000-call run adds 10,001 strings/pairs and 228,912 payload bytes;
its retained incremental proxy is 1,166,552–1,274,072 bytes, or
116.655200–127.407200 bytes per call averaged over this workload, including
the seed. Capacity steps explain why the 1,000-call average differs.
These proxies are not RSS or portable allocator byte bounds.

Every following CONSUMER line and raw final census row is verbatim from
this execution. The common row header is:

```text
strings	pairs	text_bytes	pair_bytes	strings_capacity	pairs_capacity	canonical_capacity	strings_lower	strings_upper	pairs_lower	pairs_upper	canonical_lower	canonical_upper	reserve	lower	upper	peak_upper	context	name
```

```text
CONSUMER source=target/brief-509/probes/enclose-0.rum calls=0 complete=true messages=0 termination=None
664	663	6155	26520	896	896	112	14336	17439	21504	25631	4480	5279	4096	77091	85120	87299	""	""
```

```text
CONSUMER source=target/brief-509/probes/enclose-1000.rum calls=1000 complete=true messages=0 termination=None
1665	1664	28066	66560	1792	1792	112	28672	34847	43008	51231	4480	5279	4096	174882	190079	190079	""	""
```

```text
CONSUMER source=target/brief-509/probes/enclose-10000.rum calls=10000 complete=true messages=0 termination=None
10665	10664	235067	426560	14336	14336	112	229376	278559	344064	409631	4480	5279	4096	1243643	1359192	1359192	""	""
```

```text
CONSUMER source=target/brief-509/probes/explicit-0.rum calls=0 complete=true messages=0 termination=None
664	663	6155	26520	896	896	112	14336	17439	21504	25631	4480	5279	4096	77091	85120	87299	""	""
```

```text
CONSUMER source=target/brief-509/probes/explicit-1000.rum calls=1000 complete=true messages=0 termination=None
664	663	6155	26520	896	896	112	14336	17439	21504	25631	4480	5279	4096	77091	85120	87299	""	""
```

```text
CONSUMER source=target/brief-509/probes/explicit-10000.rum calls=10000 complete=true messages=0 termination=None
664	663	6155	26520	896	896	112	14336	17439	21504	25631	4480	5279	4096	77091	85120	87299	""	""
```

Computed deltas from those rows (`target/brief-509/census-deltas.txt`):

```text
enclose calls=1000 delta_strings=1001 delta_pairs=1001 delta_text_bytes=21911 delta_proxy_lower=97791 delta_proxy_upper=104959 proxy_per_call_lower=97.791000 proxy_per_call_upper=104.959000
enclose calls=10000 delta_strings=10001 delta_pairs=10001 delta_text_bytes=228912 delta_proxy_lower=1166552 delta_proxy_upper=1274072 proxy_per_call_lower=116.655200 proxy_per_call_upper=127.407200
explicit calls=1000 delta_strings=0 delta_pairs=0 delta_text_bytes=0 delta_proxy_lower=0 delta_proxy_upper=0 proxy_per_call_lower=0.000000 proxy_per_call_upper=0.000000
explicit calls=10000 delta_strings=0 delta_pairs=0 delta_text_bytes=0 delta_proxy_lower=0 delta_proxy_upper=0 proxy_per_call_lower=0.000000 proxy_per_call_upper=0.000000
```

Remaining consumer inputs for cap sizing:

| Construct | Current observed growth and status |
|---|---|
| Implicit Enclose, one or two arguments | Automatic fresh tag text; current rerun above. Remains on the list because the simple repair is refuted. |
| Module | Automatic fresh text per local allocation, even after runtime gensym cleanup. The prior 10,000-activation one-local census measured 109.649400–120.401400 retained proxy bytes per activation; do not generalize it to an arbitrary number of locals. Still blocked on its own ruling. |
| Unique[] | Intentionally fresh names; prior 240,000-call census measured 122.846396–138.108262 retained proxy bytes/call. This behavior remains settled. Other prefixes change text payload length. |
| Dynamic Symbol of fresh unqualified text | Prior `Symbol[StringJoin[…]]` 240,000-call census measured 219.721596–243.721596 retained proxy bytes/call: one fresh name string but two context-probe pairs per call in that setup. A dynamic ToExpression/source/package workload introducing new names has the same resolution ingress, with a workload-dependent length/context cost, not a universal rate. |
| Explicit third-form Enclose with fixed names | Measured zero additional strings/pairs/payload/proxy through 10,000 calls. It does not remove growth caused by the body asking for new names. |

Module, Unique and dynamic-Symbol rows are **reused prior measurements**,
not newly executed large runs or candidate results. The exact selected
rows from `target/brief-506/consumer-summary.tsv` were read and saved in
`target/brief-509/prior-consumers.tsv`:

```text
mode	calls	strings	pairs	text_bytes	lower	upper	peak_upper	bytes_per_call_lower	bytes_per_call_upper
unique	0	664	663	6155	77091	85120	87299	-	-
unique	240000	240664	240663	1575050	29560226	33231103	39254292	122.846396	138.108262
symbol	0	664	663	6155	77091	85120	87299	-	-
symbol	240000	240664	480663	4215050	52810274	58578303	70611944	219.721596	243.721596
module	0	665	665	6166	77182	85211	87299	-	-
module	10000	10665	10665	165060	1173676	1289225	1298279	109.649400	120.401400
```

The proxy definition is unchanged from
`docs/dev/codex/brief-506-interner-census.md:185`: allocated text bytes plus
pair payloads and capacity-dependent table estimates, with the stated fixed
reserve; upper is implementation-conditioned and excludes allocator/RSS
costs. Peak overlap and retained bytes are distinct columns. No quota or
frequency-based claim of a dominant consumer is selected here.

**Conditional post-repair statement:** only after an approved Enclose
implementation passes the flat census gate may its automatic per-call
line be removed and replaced with measured fixed setup. Module, Unique
and genuinely new source/dynamic names still grow the interner. Module
stays listed until its separate ruling and implementation pass their own
gates. There is no factual “after this repair” census yet, and the current
list must continue to include Enclose.

Director decisions exposed by this report:

- E1: the proposed internal-only exemption is false. Decide the contract
  for the witnessed held/Catch/definition/failure/trace/name escapes and
  source persistence before dispatching a representation change.
- E2: if proceeding, approve the tag's allocator domain, exhaustion outcome
  and import/restart treatment; keep it separate from Module GC. No reuse
  can be justified by return, suspension abandonment or GC.
- E3: either keep this item's measured demand in cap sizing or commission a
  separately scoped escape/persistence design. A call-site-only replacement
  cannot silently undertake that larger repair. Do not defer the unavoidable
  quota merely on the assumption that every ordinary consumer is now flat.

No user decision is needed to complete this design report. The unresolved
choices block dispatch of a behavior-changing implementation, not the
requested refutation/report.

Coverage and stop-loss: there was no twice-uninformative probe retry.
The unqualified parse round-trip and ineffective ReleaseHold supplied
information and were followed by distinct qualified/Part probes. The
single queued REPL abandonment attempt could not enter the host operation;
its rejection is preserved and no second REPL attempt was made. Direct
host tag-snapshot, explicit-form resume completion and tag-specific
abandonment remain unexecuted gates, plainly separated from the source
proof and existing tests. Exploratory reads included nonexistent guessed
paths; those are not verification successes. No source change means no
workspace-build, fmt or implementation-performance claim is required or
made. Only the existing focused binaries were executed as evidence.

#### Handoff verification

The only tracked-file change is this appended report; the original brief
prefix is byte-identical. All 18 process-result records are preserved,
including the expected InputRequired/uncaught exits and the inconclusive
REPL attempt. No production Rust, script, fixture or other document was
changed; no sub-agents, network, git mutation or outside-repo writes were
used. Existing helper/test binaries retain their recorded hashes.

The first diff check identified trailing prompt spaces inside verbatim
output. Those four output blocks now quote their complete saved JSON
records verbatim, preserving whitespace through JSON escaping; the final
diff check is clean. Exact verification output:

```text
Changed tracked files:
docs/dev/codex/brief-509-enclose-tag-leak.md
Original brief prefix intact: True
Every execution output or its complete JSON record reproduced verbatim: True
Recorded process results: 18
Original executable hashes still match: True
git diff --check exit: 0
git diff --check output: <empty>
git status --short:
 M docs/dev/codex/brief-509-enclose-tag-leak.md
```
