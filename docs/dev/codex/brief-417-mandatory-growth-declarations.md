# brief-417 — every builtin declares how it can grow, and a wrong declaration fails

Shape: **implementation**. Worker: **cdx (Codex)**.

Gap B, step 1. Rewritten from your own brief-414 draft; read that report's
§3 inventory before starting, because it is the authority for the
classification and this brief does not restate it.

## Goal

Two things must be true when you are done, and the second is the one that
matters:

1. Every builtin declaration carries a memory-growth contract, and **omitting
   it does not compile**.
2. **A wrong `NonAmplifying` declaration fails a test.** A classification that
   nothing can falsify is a convention wearing a type's clothing, and this
   project has replaced enough of those to know the difference.

No new prechecks, no behaviour change, no limit change. This brief builds the
structure that briefs 418 and 419 fill in.

## Why the second requirement exists

Under compile pressure, the cheapest way to make 709 construction sites build
again is to write `NonAmplifying` at every one of them. That would produce a
large diff, a green gate, and no information. Your own brief-414 report said it
plainly: "`NonAmplifying` remains a reviewed assertion." I am not accepting it
as one.

## Context pointers

- `AGENTS.md` first.
- **`docs/dev/codex/brief-414-memory-accounting-gaps.md`, the appended report's §3** (line 354 onward, "Gap B: inventory, structural fix, and stride") — your inventory
  of every multiplicative site, with `path:line`, growth factor, and current
  precheck. This is the source of truth for classification. Do not re-derive it
  from scratch and do not silently disagree with it: if a row is wrong, say so
  in the report as a correction, with evidence.
- `crates/rum-runtime/src/runtime/metadata.rs:296-460` — `BuiltinSpec`, its
  seven public fields and its nine `const` constructors.
- `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:133-220` — dispatch is
  already generic over `CallBuiltinSpec: Copy`, implemented for `BuiltinSpec`
  (`:198`) and `SuspendedBuiltinSpec` (`:212`). It reads only `handler()`,
  `capability()` and `name()`.
- `crates/rum-runtime/src/runtime/kernel_api.rs:69` — `BuiltinSpec` is public
  API.
- `crates/rum-runtime/src/runtime/tests/session_memory.rs` — the enforcement
  shape that just landed (brief-415): a macro emitting both a checked-in
  inventory and an exhaustive pattern, so an omission stops compiling. Copy
  the idea, not the code.
- `crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:269-333` —
  `expr_memory_proxy_bytes`, which you will need for requirement 2.

## Director-measured, so you do not have to re-derive it

- Construction sites: **697 constructor calls + 12 struct literals = 709**,
  across `BuiltinSpec::{new, with_hold, with_attributes, special,
  special_with_hold, special_with_attributes, ordinary, ordinary_with_hold,
  ordinary_with_attributes}`. `ordinary` alone is 335.
- No crate outside `rum-runtime` constructs a `BuiltinSpec` today.

Verify both if you like; report it if I am wrong.

## Deliverable

### 1. A mandatory contract on the declaration type

A `MemoryGrowthContract` with no `Default`, no setter, and no builder method —
a **required parameter of every `BuiltinSpec` constructor**, so a new builtin
cannot be declared without choosing. A builder method would be forgettable,
which defeats the point.

Start from your drafted variants (`NonAmplifying`, `Preflight`, `Incremental`,
`Delegated`) but treat them as a proposal, not a specification: name and shape
them for what the inventory actually contains. If the inventory needs a variant
you did not draft, add it and say why. Briefs 418-419 will attach projector
functions to `Preflight`; leave room for that without building it now.

### 2. The classification, from the inventory

Every installed builtin classified, driven by brief-414 §3. Any installed
builtin the inventory does not cover is a **finding to report**, not a blank to
fill with `NonAmplifying`. List them.

### 3. A sorted, checked-in invariant

Covering every declaration and its contract, so that a future change to a
classification shows up as a reviewable diff line rather than a silent edit —
the `frame_root_trace` and `session_memory` shape.

### 4. The falsifier — the load-bearing deliverable

Under a test or armed mode only, a check that catches a handler declared
`NonAmplifying` whose result is materially larger than its inputs.

**Do not do this with a full session walk per handler.** The walk is why the
stride exists; running it around every builtin call would be brutal even in
armed mode. The cheap local form is enough: compare
`expr_memory_proxy_bytes(result)` against the summed
`expr_memory_proxy_bytes` of the arguments, and fail a `NonAmplifying`
declaration whose result exceeds that sum by more than a small stated
allowance. That catches the class of mistake this deliverable exists for — a
builtin that multiplies its input and claims it does not.

State its limits honestly in the report: it does not see session-state growth,
and it cannot bound an arbitrary host callback. Both are fine. It only has to
make a wrong declaration falsifiable, and it will be exercised by the whole
existing 2 291-test suite, which is what makes deliverable 2 trustworthy
instead of decorative.

**Then run it and report what it caught.** If it fires on nothing at all,
suspect the check before believing the classification — write in the report how
you confirmed it can fire, for example by deliberately misdeclaring one builtin
and observing the failure, then reverting. A falsifier that has never been seen
to fail is not known to work. This project has been burned four separate times
by an instrument that reported success while measuring nothing.

### 5. Numbers for the hot path

`BuiltinSpec` flows through ordinary dispatch **by `Copy`**, so a bigger
declaration type is a bigger per-dispatch copy of a field dispatch never reads.
Report `std::mem::size_of::<BuiltinSpec>()` before and after, and run the
microbench A/B below.

**If the A/B regresses, STOP and report.** Do not refactor dispatch onto a lean
record in this brief to rescue the number — that is a second variable and a
separate decision. `SuspendedBuiltinSpec` already shows what a lean record
looks like; if the measurement says we need one on the hot path, I will brief
it.

## Scope

Files you MAY modify:

- `crates/rum-runtime/src/runtime/metadata.rs`
- `crates/rum-runtime/src/runtime/mod.rs`
- `crates/rum-runtime/src/runtime/kernel_api.rs`
- `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs`
- `crates/rum-runtime/src/runtime/builtin_catalog.rs` and every `.rs` beneath
  `crates/rum-runtime/src/runtime/builtin_catalog/` (18 files)
- `crates/rum-runtime/src/runtime/session/session_state.rs`
- `crates/rum-runtime/src/runtime/tests/mod.rs` and a new test module beneath
  `crates/rum-runtime/src/runtime/tests/`
- `crates/rum-stdlib/src/lib.rs`, `crates/rum-stdlib/src/catalog/mod.rs` and
  every `.rs` beneath `crates/rum-stdlib/src/catalog/` (32 files)
- any other file that only needs its `BuiltinSpec` construction updated to pass
  the new argument — that is mechanical and expected. **Anything beyond a
  construction-site update, STOP and report.**

Files you MUST NOT touch:

- `crates/rum-runtime/src/runtime/debug_tools/resources/` — the accounting is
  settled by brief-415; you read `expr_memory_proxy_bytes`, you do not change
  it or the stride or any `ResourceLimits` default.
- `crates/rum-core/src/expr.rs`'s `Expr`/`Symbol` representation.
- `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**` except appending your report
  to this brief, `.git`, git config, `Cargo.lock`.

## Hard rules

- **No behaviour change.** No new precheck actually refuses anything in this
  brief; no message changes; no limit, default or stride changes. If a
  classification makes you want to add a precheck now, that is brief 418.
- **No default-path cost.** The falsifier must not run when no ceiling is armed
  and no test mode is on, and it must not add an allocation, a lock, or a
  non-trivial branch to ordinary dispatch. The disabled path is a single cheap
  check that returns immediately.
- **Preserve suspension/resume shape.** Do not collapse a `suspend_with_frame*`
  into an eager call. If the contract must survive a suspension, note that
  `SuspendedBuiltinSpec` (`session/session_state.rs:139-153`) keeps three
  fields and drops the rest — decide deliberately and say what you decided.
- **No new `unsafe`.** `rum-core` and `rum-runtime` are zero-`unsafe`.
- **Do not weaken an existing test to make the build pass.**
- Do NOT `git commit`, branch, push, stash, checkout, or touch git config.
  Leave everything in the working tree. Do not touch `stash@{0}`.
- Touch nothing outside this repo except `/tmp` scratch you remove afterwards.
  No network; pass `--offline` to every `cargo` command.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only.
- **This host can bind loopback**; the nine native ZeroMQ tests in `rum-jupyter`
  genuinely run here — I observed 9 of 9 during the brief-416 gate. If your
  sandbox denies loopback and you need `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`, say so
  and mark every native result unobserved. Never report a rate for a test you
  did not watch run.
- Minimal diff outside the mechanical construction-site updates.

## Verification (MANDATORY — you run it, iterate to green)

```
cargo fmt --check
cargo test --offline -p rum-runtime
cargo test --offline -p rum-stdlib
cargo test --offline --workspace
```

Then the crate-edge proof, which a source scan cannot give: build a throwaway
crate under `/tmp` that depends on `rum-runtime` by path and tries to construct
a `BuiltinSpec` **without** the contract argument. **Paste the compiler error.**
A boundary that holds only inside this repository is not a boundary; the
brief-396 review established that a scan of in-repository callers cannot prove
a property about code that does not exist yet. Remove the crate afterwards.

Then the microbench A/B, base build against candidate build in one window:

```
bash scripts/core-language-microbench.sh --output-dir <base-dir>
bash scripts/core-language-microbench.sh --output-dir <cand-dir>
```

Report the per-label `rum_seconds` rows for the dispatch-sensitive labels and
their same-tier neighbours: the literal, symbol-ownvalue, set-ownvalue and
list-literal rows, the exact and pattern downvalue hit/miss rows, and the
matcher and `replaceall-simple` rows. **Name your control labels before
judging** — a whole-run offset moves every label together, and a lopsided
direction split is not evidence. Profiling env off for every timing number.

## Report format

1. What changed — per file, line ranges, one-line why. Group the mechanical
   construction-site updates; do not list 709 of them.
2. The contract type: its variants, and why they differ from your draft.
3. The classification: how you derived each from the inventory, and every
   installed builtin the inventory did not cover.
4. **The falsifier: how it works, what it caught, and your evidence that it
   can fire.**
5. `size_of::<BuiltinSpec>()` before and after, plus the microbench rows with
   named controls.
6. Verification — every command, verbatim output, including the crate-edge
   compiler error.
7. Stop-loss / open questions.

## Worker report

### 1. What changed

- `crates/rum-runtime/src/runtime/metadata.rs:291-543` — added the mandatory
  public `MemoryGrowthContract`, stored it on `BuiltinSpec`, required it in all
  nine constructors, and transferred it with a fallback handler/capability.
- `crates/rum-runtime/src/runtime/kernel_api.rs:67-70` — re-exported the
  contract beside the already-public `BuiltinSpec`.
- `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:1-7,141-231,255-383` —
  carried the contract through generic dispatch, added the test/ceiling-armed
  local result check, and preserved the check across input suspension/resume.
  The disabled path is one short-circuit test plus the env flag's relaxed
  `AtomicU8` load; initialization and all proxy work are cold, and there is no
  allocation or lock on the ordinary disabled path.
- `crates/rum-runtime/src/runtime/session/session_state.rs:139-155` — retained
  the contract as a one-byte code in `SuspendedBuiltinSpec`. I deliberately did
  not collapse or bypass suspension. The existing lean record remains the
  dispatch representation, with one byte fitting its existing padding.
- `crates/rum-runtime/src/runtime/tests/memory_growth_contract.rs:1-257` and
  `runtime/tests/mod.rs:155` — added the sorted 185-entry core invariant and
  immediate/resumed falsifier tests.
- `crates/rum-stdlib/src/catalog/memory_growth_contract.rs:1-750` and
  `catalog/mod.rs:17-18` — added the sorted exact installed-catalog invariant:
  643 entries with `hash`, 637 without its six conditional entries.
- Mechanical construction-site updates span the changed files under
  `crates/rum-runtime/src/runtime/builtin_catalog/`,
  `crates/rum-stdlib/src/catalog/`, `runtime/test_builtins.rs`, the affected
  runtime test modules, and `crates/rum-host/tests/installer.rs:1-90`. Those
  edits only import/select a contract argument; no handler behavior changed.

The director's 697 constructor-call count was correct, but the reported 12
struct literals were 12 textual `BuiltinSpec {` false positives: six function
return signatures, struct/impl declarations, and matches inside
`SuspendedBuiltinSpec`. There were no pre-change `BuiltinSpec` expression
literals. The final tree has 698 constructor calls and one real struct-update
literal, both additions belonging to the falsifier tests. The statement that
no outside crate constructed `BuiltinSpec` was also not correct: four
pre-existing calls were in `crates/rum-host/tests/installer.rs`; I made only
the expected mechanical argument updates there.

No resource-accounting file, `Expr`/`Symbol` representation, limit, default,
stride, message, or language behavior was changed. No `unsafe` was added.

### 2. The contract type

The four fieldless variants retain the draft names; no fifth inventory shape
was needed:

- `NonAmplifying` — a successful local result must not exceed the arguments'
  summed proxy bytes by more than the stated 1,024-byte fixed allowance.
- `Preflight` — deterministic/bounded construction for which brief 418 can
  attach a conservative projector before allocation.
- `Incremental` — data-dependent, callback-driven, traversal, or retained
  session growth that brief 419 must charge as construction proceeds.
- `Delegated` — the meaningful growth happens across another evaluator or host
  boundary rather than in this handler's local result construction.

The enum intentionally has no `Default`, setter, or builder. Every one of the
nine constructors takes it as a required final argument. It differs from the
payload-bearing sketch in brief 414 (`ProofId`, `ProjectorFn`, `BuilderKind`,
`CoreGrowthSite`) only by deferring those not-yet-built payload types; the
closed variant choice is mandatory now, and `Preflight` is the named attachment
point for the next brief.

### 3. Classification

I expanded every grouped operation in brief 414 section 3 into individual
installed heads. Deterministic product/shape/encoding/integer bounds are
`Preflight`; callback, repeated-retention, association, rewrite, diagnostic,
and session-store rows are `Incremental`; evaluator/control and arbitrary host
result rows are `Delegated`; an installed handler with no such growth site is
`NonAmplifying`. The final 643-entry distribution is:

```text
Delegated        55
Incremental     165
NonAmplifying   290
Preflight       133
```

The section-3 rows, shorthand expansions, and its named conservative/bounded
exclusions cover 250 installed names. `RandomSample` is not individually named,
but I count it with that boundary because it is a one-for-one reorder (the
section explicitly excludes one-for-one reorder) and it also enforces its
generated count. `IntegerExponent` and `JacobiSymbol`, encountered while
expanding the excluded number-theory family, are not installed names.

The remaining 393 installed names are the required inventory-gap finding. I
did not fill that gap blindly: 106 of them are non-`NonAmplifying`, and the
armed suite plus source review corrected wrong initial assertions. Grouped by
their final declaration, they are:

- `Delegated` (47): AbortProtect, AbsoluteTiming, AllTrue, And, AnyTrue,
  Assert, Block, BlockRandom, Catch, CatchExceptions, Check, CheckAbort,
  CompoundExpression, Confirm, ConfirmAssert, ConfirmBy, ConfirmMatch,
  ConfirmQuiet, Do, Enclose, Evaluate, Failsafe, FindFile, For, If,
  LengthWhile, MemoryConstrained, Module, Monitor, NoneTrue, Or,
  PropagateAborts, Quiet, ReleaseHold, RepeatedTiming,
  RestoreSealedReferences, Rule, Scan, SelectFirst, Switch, TimeConstrained,
  Timing, Until, ValueQ, Which, While, WithCleanup.
- `Incremental` (32): ArgumentsOptions, Association, Decrement,
  DeleteDuplicatesBy, ExceptionTypes, Factor, Increment, KeyDropFrom,
  KeySelect, KeySortBy, MaxBy, MaximalBy, Message, MinBy, MinimalBy, Off, On,
  OptionValue, PackageScope, PreDecrement, PreIncrement, Public,
  RegisterExceptionType, ReverseSortBy, RumProfileSnapshot, SelectPositions,
  SetAttributes, SortBy, TakeLargestBy, TakeSmallestBy, TakeWhile, Unique.
- `Preflight` (27): Alphabet, Append, Attributes, Commonest, Complement,
  Crc32, DateString, DeleteDuplicates, DeleteMissing, DigitCount, Duplicates,
  FromDateString, IntegerDigits, IntegerLength, IntegerReverse, Intersection,
  Md5, Quantile, RemoveDiacritics, Sha1, Sha256, Sha512, SpellingCorrection,
  StringPart, StringTakeDrop, Transpose, Union.
- `NonAmplifying` (287): $Aborted, $AssertFunction, $Canceled, $Context,
  $Failed, $InputFileName, Abort, Abs, AbsoluteTime, AlgebraicIntegerQ,
  AlphabeticOrder, AlphabeticSort, Alphanumeric, Alternatives, ArrayQ,
  AssociationQ, AtomQ, Automatic, Begin, BeginPackage, Between, BinarySearch,
  Blank, BlankNullSequence, BlankSequence, Boole, BooleanQ, Break, ByteArrayQ,
  ByteCount, Catenate, Ceiling, CheckArguments, Clear, ClearAll,
  ClearAttributes, Coefficient, Composition, Construct, ContainsAll,
  ContainsAny, ContainsExactly, ContainsNone, ContainsOnly, Context, Continue,
  Count, CurryApplied, DateDifference, DateList, DateObjectQ, DatePlus,
  DayOfWeek, DaysInMonth, DeclarePackage, Delete, DeleteCases, Denominator,
  Depth, DiffObject, DigitCharacter, DigitCharacters, DigitQ, DirectoryName,
  DisjointQ, Drop, DuplicateFreeQ, EchoFunction, EchoLabel, EditDistance, End,
  EndOfLine, EndOfString, EndPackage, Equal, EqualTo, Equivalent, EvenQ,
  Except, Exception, Exit, Failure, FailureQ, File, FileBaseName,
  FileExtension, FileNameJoin, FileNameTake, First, Flatten, Floor,
  FractionalPart, FreeQ, FromDateList, General, Greater, GreaterEqual,
  GreaterEqualThan, GreaterThan, HammingDistance, Hash, Head, Hold, HoldAll,
  HoldAllComplete, HoldComplete, HoldFirst, HoldForm, HoldPattern, HoldRest,
  HtmlUnescape, Identity, IgnoreCase, Implies, Inequality, InputForm,
  IntegerPart, IntegerQ, IntersectingQ, Join, Key, KeyComplement, KeyDrop,
  KeyExistsQ, KeyFreeQ, KeyMemberQ, KeySort, KeyTake, KeyValuePattern, Last,
  LazyDrop, LazyFoldList, LazyList, LazyMap, LazyNestList, LazyPermutations,
  LazyRange, LazySelect, LazySubsets, LazyTable, LazyTake, LeafCount,
  LeapYearQ, Length, Less, LessEqual, LessEqualThan, LessThan, LetterCharacter,
  LetterQ, LexicographicOrder, ListQ, Longest, LongestCommonSubsequence,
  LowerCaseQ, LowercaseLetters, MachineNumberQ, MatchQ, MatrixQ, Max,
  MaxMemoryUsed, Median, MemberQ, MemoryInUse, MessageName, Min, MinMax, Minus,
  Missing, MissingQ, Mod, Most, NameQ, Nand, Nearest, Negative, NonNegative,
  NonPositive, Nor, NormalizeRules, Not, Nothing, NumberQ, NumberString,
  Numerator, NumericQ, NumericalOrder, OddQ, Operate, OperatorApplied, Order,
  OrderedQ, OutputForm, Overlaps, ParentDirectory, Pattern, Pause, Pluralize,
  PolynomialExpressionQ, PolynomialQ, Positive, PossibleZeroQ, Prepend,
  Protect, PunctuationCharacter, PunctuationCharacters, Quotient, RandomUuid,
  RationalExpressionQ, RealValuedNumberQ, RealValuedNumericQ,
  RegularExpression, Remove, Repeated, RepeatedNull, ReseedRandom,
  ResolveContextAliases, Rest, Return, Reverse, ReverseApplied, ReverseSort,
  RightComposition, Round, RuleDelayed, RumProfileReset, SameAs, SameQ,
  SeedRandom, Sequence, Shortest, Sign, Sort, StandardForm, StartOfLine,
  StartOfString, StringContainsQ, StringCount, StringDedent, StringDelete,
  StringDrop, StringEndsQ, StringExpression, StringFreeQ, StringLength,
  StringMatchQ, StringQ, StringReverse, StringStartsQ, StringTake,
  StringTemplate, StringTrim, SubsetQ, Success, Symbol, SymbolName, SyntaxQ,
  TagUnset, Take, TakeDrop, TemplateObject, TerminatedEvaluation, TestCreate,
  Throw, ThrowException, TimeObjectQ, TraditionalForm, TrueQ, Unequal,
  UnequalTo, Unevaluated, Unprotect, UnsameQ, Unset, UpTo, UpperCaseQ,
  UppercaseLetters, UrlDecode, UuidQ, VectorQ, Whitespace,
  WhitespaceCharacter, WhitespaceCharacters, WordBoundary, WordCharacter,
  XmlElement, Xor.

The armed run itself corrected 11 initially wrong `NonAmplifying`
declarations (section 4). I then reviewed their sibling implementations and
proactively moved `LengthWhile` to `Delegated`; `DeleteDuplicatesBy`,
`KeySelect`, `KeySortBy`, `MaxBy`, `MaximalBy`, `MinBy`, `MinimalBy`,
`ReverseSortBy`, `TakeLargestBy`, `TakeSmallestBy`, and `TakeWhile` to
`Incremental`; and `Complement`, `DigitCount`, `Duplicates`, `Intersection`,
`Quantile`, and `Transpose` to `Preflight`.

The core and installed invariant tests sort and compare exact `(name,
contract)` pairs. The stdlib test checks every raw family declaration first,
including the deliberate core/stdlib fallback duplicates, requires each
duplicate to agree with the one inventory row, then separately checks the
effective installed session. A missing/extra name or classification change
therefore fails or produces a reviewable table line.

### 4. The falsifier

`Session::validate_builtin_memory_growth` runs only when the runtime crate is
under `cfg(test)`, `RUM_TEST_BUILTIN_MEMORY_GROWTH` is armed, or a memory
ceiling is active. For a successful `NonAmplifying` result it saturating-sums
the arguments' proxy bytes, obtains the result proxy through the existing
`ByteCount` path (which calls the settled local `expr_memory_proxy_bytes`), and
panics if the result exceeds arguments plus 1,024 bytes. Other contracts and
error results return immediately.

Handler suspension stores the builtin name, one-byte contract, and shared
arguments in the existing dispatch-stage frame; the resumed result is checked
before frame completion. Threaded-call suspension uses the same frame with no
builtin payload. Thus neither suspension shape nor eager behavior changed.

The first armed passes did fire. They found these real classification errors;
the numbers are `result/arguments` proxy bytes, all with the 1,024-byte
allowance:

```text
ArgumentsOptions   1870/355
Rule              64289/424
KeyDropFrom        4526/1399
DeleteMissing      2004/925
Commonest        327755/160075
Factor            16190/13118
Association       38994/36427 and 16521/13111
DeleteDuplicates   8267/6023
Union              8267/6641
SortBy           295051/132229
Append           320075/160107
```

Those were reclassified to the appropriate `Delegated`, `Incremental`, or
`Preflight` strategy; the armed whole workspace then passed. Separately, two
checked-in deliberately wrong declarations prove both paths can fire. Their
observed messages were:

```text
builtin DeliberatelyAmplifyingBuiltin declared NonAmplifying but returned 4128 proxy bytes from 0 argument proxy bytes (allowance: 1024 bytes)
builtin InputString declared NonAmplifying but returned 4128 proxy bytes from 39 argument proxy bytes (allowance: 1024 bytes)
```

Limits: this is a logical expression proxy, not allocator interception; it
does not observe session-state growth, temporary peak allocation, or identity
sharing, and cannot bound an arbitrary host/evaluator callback. It deliberately
does not yet validate projector math or incremental reservations. The fixed
allowance can also hide small absolute amplification. It is a diagnostic
postcondition, not pre-allocation prevention.

### 5. Size and hot-path A/B

`std::mem::size_of::<BuiltinSpec>()` was **64 bytes before and 64 bytes after**.
The enum occupies existing layout padding. The suspended spec likewise keeps
its contract as `u8`, not a payload-bearing catalog enum.

Before looking at candidate timings, I named these controls:
`literal-integer`, `literal-string`, `symbol-ownvalue-read`,
`symbol-ownvalue-guarded-read`, `set-ownvalue`, and `list-literal-small`.
Profiling and the armed falsifier env were explicitly off for both runs. The
required target and same-tier-neighbor rows are:

| label | iterations | base `rum_seconds` | candidate `rum_seconds` | delta |
|---|---:|---:|---:|---:|
| literal-integer | 2000 | 0.000076941 | 0.000081534 | +5.97% |
| literal-string | 2000 | 0.000085548 | 0.000084530 | -1.19% |
| symbol-ownvalue-read | 2000 | 0.000205438 | 0.000205506 | +0.03% |
| symbol-ownvalue-guarded-read | 1000 | 0.000175748 | 0.000169507 | -3.55% |
| set-ownvalue | 1000 | 0.000253960 | 0.000253836 | -0.05% |
| list-literal-small | 1000 | 0.000082207 | 0.000080670 | -1.87% |
| downvalue-exact-hit | 2000 | 0.000785528 | 0.000792482 | +0.89% |
| downvalue-exact-miss | 2000 | 0.000550828 | 0.000564718 | +2.52% |
| downvalue-pattern-hit | 1000 | 0.000965716 | 0.000984045 | +1.90% |
| downvalue-pattern-miss | 1000 | 0.000286363 | 0.000290997 | +1.62% |
| downvalue-exact-miss-repeat | 5000 | 0.001241020 | 0.001237320 | -0.30% |
| downvalue-pattern-miss-repeat | 5000 | 0.001294900 | 0.001291990 | -0.22% |
| matcher-single-tiny | 1000 | 0.000675531 | 0.000674394 | -0.17% |
| matcher-single-wide-small | 800 | 0.000041343 | 0.000037761 | -8.66% |
| matcher-single-wide-large | 1600 | 0.000082477 | 0.000074569 | -9.59% |
| replaceall-simple | 500 | 0.000359788 | 0.000351534 | -2.29% |

The controls split directionally and have a median delta of approximately
-0.01%; the target/neighbor median is approximately -0.20%, with four positive
and six negative rows. The isolated `literal-integer` movement is 4.593
microseconds over 2,000 iterations while its literal neighbor improves. There
is no coherent whole-run or dispatch-sensitive regression, so the stop-loss
did not fire.

### 6. Verification

All cargo test/build invocations were offline and long-running ones used
`scripts/run-bounded.py`. Final commands and verbatim relevant output follow.

`cargo --offline fmt --check`

No output; exit status 0.

`cargo test --offline -p rum-runtime`

```text
test result: ok. 937 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 3.71s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

`cargo test --offline -p rum-stdlib`

```text
test result: ok. 829 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 2.62s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

The conditional 637-entry catalog was also checked with
`cargo test --offline -p rum-stdlib --no-default-features
catalog::memory_growth_contract::installed_builtin_memory_growth_inventory_is_sorted_and_exhaustive`:

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 825 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

The first `cargo test --offline --workspace` reached `rum-jupyter`, but this
sandbox rejected the loopback bind with `Operation not permitted (os error
1)`. Its exact failing result line was:

```text
test result: FAILED. 97 passed; 9 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
```

As permitted by the brief for that condition, I reran
`RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test --offline --workspace`. Its verbatim
result lines were:

```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 87 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.11s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
test result: ok. 106 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
test result: ok. 937 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 3.73s
test result: ok. 829 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 2.53s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 189 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.88s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

All nine native ZeroMQ subjects are **unobserved**, not passes: basic execute,
heartbeat, signed execute/IOPub, tampered-signature rejection, frontend
execute, frontend stdin, control interrupt, shutdown with pending shell work,
and multi-message session flow.

The whole-suite armed run was
`RUM_TEST_BUILTIN_MEMORY_GROWTH=1 RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test
--offline --workspace`. Its verbatim result lines were:

```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 87 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.11s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
test result: ok. 106 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
test result: ok. 937 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 3.74s
test result: ok. 829 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 2.90s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 189 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.87s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

The focused can-fire command was
`cargo test --offline -p rum-runtime memory_growth_contract:: -- --nocapture`:

```text
running 3 tests
test runtime::tests::memory_growth_contract::core_builtin_memory_growth_inventory_is_sorted_and_exhaustive ... ok
builtin InputString declared NonAmplifying but returned 4128 proxy bytes from 39 argument proxy bytes (allowance: 1024 bytes)
test runtime::tests::memory_growth_contract::non_amplifying_contract_is_falsifiable_after_input_resume - should panic ... ok
builtin DeliberatelyAmplifyingBuiltin declared NonAmplifying but returned 4128 proxy bytes from 0 argument proxy bytes (allowance: 1024 bytes)
test runtime::tests::memory_growth_contract::non_amplifying_contract_is_falsifiable_through_dispatch - should panic ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 936 filtered out; finished in 0.00s
```

For the crate-edge proof, `/tmp/rum-brief417-crate-edge` depended on
`rum-runtime` by path and called `BuiltinSpec::new(name, summary)`. The command
was `cargo check --offline`; the required compiler error was:

```text
error[E0061]: this function takes 3 arguments but 2 arguments were supplied
   --> src/main.rs:4:13
    |
  4 |       let _ = BuiltinSpec::new(
    |  _____________^^^^^^^^^^^^^^^^-
  5 | |         "ExternalMissingContract",
  6 | |         "This construction intentionally omits the required contract.",
  7 | |     );
    | |_____- argument #3 of type `MemoryGrowthContract` is missing
    |
note: associated function defined here
   --> /home/remi/dev/rum/crates/rum-runtime/src/runtime/metadata.rs:346:18
    |
346 |     pub const fn new(
    |                  ^^^
help: provide the argument
    |
  4 |     let _ = BuiltinSpec::new(
  5 |         "ExternalMissingContract",
  6 |         "This construction intentionally omits the required contract.",
  7 ~         /* MemoryGrowthContract */,
  8 ~     );
    |

For more information about this error, try `rustc --explain E0061`.
error: could not compile `rum-brief417-crate-edge` (bin "rum-brief417-crate-edge") due to 1 previous error
```

The size probes were path-dependent throwaway crates run with
`cargo run --offline`; both printed:

```text
64
```

The candidate release command was `cargo build --offline --release -p
rum-cli` with every `RUM_PROFILE_*` variable and the armed checker unset:

```text
Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
Compiling rum-host v0.2.0 (/home/remi/dev/rum/crates/rum-host)
Compiling rum-terminal v0.2.0 (/home/remi/dev/rum/crates/rum-terminal)
Compiling rum-repl v0.2.0 (/home/remi/dev/rum/crates/rum-repl)
Compiling rum-cli v0.2.0 (/home/remi/dev/rum/crates/rum-cli)
Finished `release` profile [optimized] target(s) in 20.96s
```

The base and candidate commands were respectively:

```text
CARGO_NET_OFFLINE=true bash scripts/core-language-microbench.sh --output-dir /tmp/rum-brief417-base
core microbench output: /tmp/rum-brief417-base

CARGO_NET_OFFLINE=true bash scripts/core-language-microbench.sh --output-dir /tmp/rum-brief417-cand
core microbench output: /tmp/rum-brief417-cand
```

The exact selected timing rows are in section 5. `git diff --check` also had no
output and exited 0.

### 7. Stop-loss / open questions

The size and A/B stop-losses did not fire. All available tests pass both
normally and with the checker armed; no precheck or second dispatch
representation was introduced.

The only external gate item is the sandbox-blocked native ZeroMQ set: all nine
subjects remain unobserved and must be run in an environment that permits
loopback. For briefs 418-419, the open design work is exactly the deferred
payload: attach conservative projectors to `Preflight` and checked retained
builders/reservations to `Incremental`; the current local postcondition cannot
substitute for either.
