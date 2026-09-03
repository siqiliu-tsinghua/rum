# Capability Classification And Limit Coverage Consistency Audit

Diagnostic only; no source or test change.

## 1. Capability-classification table

Extraction method: a balanced-delimiter Python scan of every `BuiltinSpec::<constructor>(...)` expression under `crates/rum-runtime/src/runtime/builtin_catalog/**` and `crates/rum-stdlib/src/catalog/**`, taking `.with_capability(...)` when present and the constructor default `Pure` otherwise. It found **660 declarations / 642 distinct names**; the 18 duplicate declarations are the intentionally mirrored inert string-pattern heads. The generated reference independently contains **642 distinct headings** with the same per-name capability labels: **617 Pure, 8 Clock, 4 FileRead, 4 Diagnostic, 3 Interactive, 3 Random, 2 FileWrite, 1 Process**. No source name is absent from the reference, no reference name is absent from source, and no capability differs.

The exact count was therefore reconciled against the generated catalog product rather than inferred from `Names` (the live `Names["System`*"]` result is 681 because it also includes non-builtin support symbols). Representative release-CLI handlers were then sampled:

```text
$ target/release/rum-cli --eval 'Length[Names["System`*"]]'
681
$ target/release/rum-cli --eval '{FileNameJoin[{"a", "b"}], AbsoluteTime[], SeedRandom[17]; {RandomInteger[100], RandomInteger[100]}, RandomBytes[2], MemoryInUse[]}'
List["a/b", 55768469722307/31250, List[21, 47], ByteArray["qtQ="], 67141]
```

Classification uses the documented capability boundary: deterministic session state (including the fixed-seed PRNG), evaluator/session mutation, and the session-mediated print stream are `Pure`; host entropy is `Random`; runtime-internal/global instrumentation disclosure or mutation is `Diagnostic`. Internal caches, symbol interning, and env-gated evaluator instrumentation common to dispatch are infrastructure, not a builtin-specific ambient effect.

| Builtin | Declared capability | Actual external/impure effect | Verdict |
|---|---|---|---|
| `$Aborted` | `Pure` | None; deterministic/session-local semantics only. | match |
| `$AssertFunction` | `Pure` | None; deterministic/session-local semantics only. | match |
| `$Canceled` | `Pure` | None; deterministic/session-local semantics only. | match |
| `$Context` | `Pure` | None; deterministic/session-local semantics only. | match |
| `$ContextPath` | `Pure` | None; deterministic/session-local semantics only. | match |
| `$Failed` | `Pure` | None; deterministic/session-local semantics only. | match |
| `$InputFileName` | `Pure` | None; deterministic/session-local semantics only. | match |
| `$MessageList` | `Pure` | None; deterministic/session-local semantics only. | match |
| `$Packages` | `Pure` | None; deterministic/session-local semantics only. | match |
| `$Path` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Abort` | `Pure` | None; deterministic/session-local semantics only. | match |
| `AbortProtect` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Abs` | `Pure` | None; deterministic/session-local semantics only. | match |
| `AbsoluteTime` | `Clock` | Reads wall-clock `SystemTime` (`crates/rum-runtime/src/runtime/debug_tools/timing.rs:11-20`). | match |
| `AbsoluteTiming` | `Clock` | Reads a monotonic clock around evaluation (`crates/rum-runtime/src/runtime/debug_tools/timing.rs:50-74`). | match |
| `Accumulate` | `Pure` | None; deterministic/session-local semantics only. | match |
| `AddTo` | `Pure` | None; deterministic/session-local semantics only. | match |
| `AlgebraicIntegerQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `AlignColumns` | `Pure` | None; deterministic/session-local semantics only. | match |
| `AllTrue` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Alphabet` | `Pure` | None; deterministic/session-local semantics only. | match |
| `AlphabeticOrder` | `Pure` | None; deterministic/session-local semantics only. | match |
| `AlphabeticSort` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Alphanumeric` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Alternatives` | `Pure` | None; deterministic/session-local semantics only. | match |
| `And` | `Pure` | None; deterministic/session-local semantics only. | match |
| `AnyTrue` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Append` | `Pure` | None; deterministic/session-local semantics only. | match |
| `AppendTo` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Apply` | `Pure` | None; deterministic/session-local semantics only. | match |
| `ApplyTo` | `Pure` | None; deterministic/session-local semantics only. | match |
| `ArgumentsOptions` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Array` | `Pure` | None; deterministic/session-local semantics only. | match |
| `ArrayFilter` | `Pure` | None; deterministic/session-local semantics only. | match |
| `ArrayQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `ArrayReduce` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Assert` | `Pure` | None; deterministic/session-local semantics only. | match |
| `AssociateTo` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Association` | `Pure` | None; deterministic/session-local semantics only. | match |
| `AssociationKeyDeflatten` | `Pure` | None; deterministic/session-local semantics only. | match |
| `AssociationKeyFlatten` | `Pure` | None; deterministic/session-local semantics only. | match |
| `AssociationMap` | `Pure` | None; deterministic/session-local semantics only. | match |
| `AssociationQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `AssociationThread` | `Pure` | None; deterministic/session-local semantics only. | match |
| `AtomQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Attributes` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Automatic` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Begin` | `Pure` | None; deterministic/session-local semantics only. | match |
| `BeginPackage` | `Pure` | **File read:** the dependency form directly calls `eval_needs`, which can probe/read through `FileIo` (`crates/rum-runtime/src/runtime/builtins_context_loader.rs:404-444`, especially `:430-432`). | **FLAG: label too weak** |
| `BernoulliB` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Between` | `Pure` | None; deterministic/session-local semantics only. | match |
| `BinarySearch` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Binomial` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Blank` | `Pure` | None; deterministic/session-local semantics only. | match |
| `BlankNullSequence` | `Pure` | None; deterministic/session-local semantics only. | match |
| `BlankSequence` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Block` | `Pure` | None; deterministic/session-local semantics only. | match |
| `BlockMap` | `Pure` | None; deterministic/session-local semantics only. | match |
| `BlockRandom` | `Pure` | Session-local deterministic PRNG state only; no wall clock or host entropy (`crates/rum-runtime/src/runtime/session/session_api.rs:505-540`). | match (documented Pure/entropy split) |
| `Boole` | `Pure` | None; deterministic/session-local semantics only. | match |
| `BooleanQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Break` | `Pure` | None; deterministic/session-local semantics only. | match |
| `ByteArray` | `Pure` | None; deterministic/session-local semantics only. | match |
| `ByteArrayQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `ByteArrayToString` | `Pure` | None; deterministic/session-local semantics only. | match |
| `ByteCount` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Capitalize` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Cases` | `Pure` | None; deterministic/session-local semantics only. | match |
| `CatalanNumber` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Catch` | `Pure` | None; deterministic/session-local semantics only. | match |
| `CatchExceptions` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Catenate` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Ceiling` | `Pure` | None; deterministic/session-local semantics only. | match |
| `CharacterCounts` | `Pure` | None; deterministic/session-local semantics only. | match |
| `CharacterRange` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Characters` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Check` | `Pure` | None; deterministic/session-local semantics only. | match |
| `CheckAbort` | `Pure` | None; deterministic/session-local semantics only. | match |
| `CheckArguments` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Clear` | `Pure` | None; deterministic/session-local semantics only. | match |
| `ClearAll` | `Pure` | None; deterministic/session-local semantics only. | match |
| `ClearAttributes` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Coefficient` | `Pure` | None; deterministic/session-local semantics only. | match |
| `CoefficientList` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Comap` | `Pure` | None; deterministic/session-local semantics only. | match |
| `ComapApply` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Commonest` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Complement` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Composition` | `Pure` | None; deterministic/session-local semantics only. | match |
| `CompoundExpression` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Compress` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Confirm` | `Pure` | None; deterministic/session-local semantics only. | match |
| `ConfirmAssert` | `Pure` | None; deterministic/session-local semantics only. | match |
| `ConfirmBy` | `Pure` | None; deterministic/session-local semantics only. | match |
| `ConfirmMatch` | `Pure` | None; deterministic/session-local semantics only. | match |
| `ConfirmQuiet` | `Pure` | None; deterministic/session-local semantics only. | match |
| `ConformSchema` | `Pure` | None; deterministic/session-local semantics only. | match |
| `ConstantArray` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Construct` | `Pure` | None; deterministic/session-local semantics only. | match |
| `ContainsAll` | `Pure` | None; deterministic/session-local semantics only. | match |
| `ContainsAny` | `Pure` | None; deterministic/session-local semantics only. | match |
| `ContainsExactly` | `Pure` | None; deterministic/session-local semantics only. | match |
| `ContainsNone` | `Pure` | None; deterministic/session-local semantics only. | match |
| `ContainsOnly` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Context` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Contexts` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Continue` | `Pure` | None; deterministic/session-local semantics only. | match |
| `CoprimeQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Count` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Counts` | `Pure` | None; deterministic/session-local semantics only. | match |
| `CountsBy` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Crc32` | `Pure` | None; deterministic/session-local semantics only. | match |
| `CurryApplied` | `Pure` | None; deterministic/session-local semantics only. | match |
| `D` | `Pure` | None; deterministic/session-local semantics only. | match |
| `DateDifference` | `Pure` | None; deterministic/session-local semantics only. | match |
| `DateList` | `Pure` | None; deterministic/session-local semantics only. | match |
| `DateObjectQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `DatePlus` | `Pure` | None; deterministic/session-local semantics only. | match |
| `DateString` | `Pure` | None; deterministic/session-local semantics only. | match |
| `DayOfWeek` | `Pure` | None; deterministic/session-local semantics only. | match |
| `DaysInMonth` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Decapitalize` | `Pure` | None; deterministic/session-local semantics only. | match |
| `DecimalString` | `Pure` | None; deterministic/session-local semantics only. | match |
| `DeclarePackage` | `Pure` | **Latent file read:** it installs declared stubs (`crates/rum-runtime/src/runtime/package_tools.rs:44-76`) whose later symbol evaluation directly calls `eval_needs` and can probe/read through `FileIo` (`crates/rum-runtime/src/runtime/eval_core/symbol.rs:50-63`). | **FLAG: label too weak/ownership decision** |
| `Decrement` | `Pure` | None; deterministic/session-local semantics only. | match |
| `DefaultValues` | `Pure` | None; deterministic/session-local semantics only. | match |
| `DefinePackage` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Definition` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Delete` | `Pure` | None; deterministic/session-local semantics only. | match |
| `DeleteCases` | `Pure` | None; deterministic/session-local semantics only. | match |
| `DeleteDuplicates` | `Pure` | None; deterministic/session-local semantics only. | match |
| `DeleteDuplicatesBy` | `Pure` | None; deterministic/session-local semantics only. | match |
| `DeleteMissing` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Denominator` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Depth` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Dialog` | `Interactive` | May suspend evaluation with a host-input request from its held body (`crates/rum-runtime/src/runtime/builtins_input.rs:40-70,224-232`). | match |
| `Diff` | `Pure` | None; deterministic/session-local semantics only. | match |
| `DiffObject` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Differences` | `Pure` | None; deterministic/session-local semantics only. | match |
| `DigitCharacter` | `Pure` | None; deterministic/session-local semantics only. | match |
| `DigitCharacters` | `Pure` | None; deterministic/session-local semantics only. | match |
| `DigitCount` | `Pure` | None; deterministic/session-local semantics only. | match |
| `DigitQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `DirectoryName` | `Pure` | None; deterministic/session-local semantics only. | match |
| `DisjointQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Divide` | `Pure` | None; deterministic/session-local semantics only. | match |
| `DivideBy` | `Pure` | None; deterministic/session-local semantics only. | match |
| `DivisorSigma` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Divisors` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Do` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Dot` | `Pure` | None; deterministic/session-local semantics only. | match |
| `DownValues` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Drop` | `Pure` | None; deterministic/session-local semantics only. | match |
| `DumpSave` | `FileWrite` | Replaces a serialized dump through `FileIo::write_string` (`crates/rum-runtime/src/runtime/package_tools.rs:83-85,597-612`). | match |
| `DuplicateFreeQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Duplicates` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Echo` | `Pure` | Session-mediated print/callback stream only; no direct stdout/stderr or ambient authority (`crates/rum-runtime/src/runtime/session/session_helpers.rs:227-233`). | match (session-state doctrine) |
| `EchoFunction` | `Pure` | None; deterministic/session-local semantics only. | match |
| `EchoLabel` | `Pure` | None; deterministic/session-local semantics only. | match |
| `EditDistance` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Enclose` | `Pure` | None; deterministic/session-local semantics only. | match |
| `End` | `Pure` | None; deterministic/session-local semantics only. | match |
| `EndOfLine` | `Pure` | None; deterministic/session-local semantics only. | match |
| `EndOfString` | `Pure` | None; deterministic/session-local semantics only. | match |
| `EndPackage` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Equal` | `Pure` | None; deterministic/session-local semantics only. | match |
| `EqualTo` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Equivalent` | `Pure` | None; deterministic/session-local semantics only. | match |
| `EulerPhi` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Evaluate` | `Pure` | None; deterministic/session-local semantics only. | match |
| `EvaluationData` | `Clock` | Reads a monotonic clock for elapsed-time metadata (`crates/rum-runtime/src/runtime/debug_tools/evaluation_data.rs:14-18,83`). | match |
| `EvenQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Except` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Exception` | `Pure` | None; deterministic/session-local semantics only. | match |
| `ExceptionTypes` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Exit` | `Process` | Returns an `EvalError::Exit` host-termination request (`crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs:47-75`). | match |
| `Expand` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Extract` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Factor` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Factorial` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Factorial2` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Failsafe` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Failure` | `Pure` | None; deterministic/session-local semantics only. | match |
| `FailureQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Fibonacci` | `Pure` | None; deterministic/session-local semantics only. | match |
| `File` | `Pure` | None; deterministic/session-local semantics only. | match |
| `FileBaseName` | `Pure` | None; deterministic/session-local semantics only. | match |
| `FileExtension` | `Pure` | None; deterministic/session-local semantics only. | match |
| `FileNameJoin` | `Pure` | None; deterministic/session-local semantics only. | match |
| `FileNameSplit` | `Pure` | None; deterministic/session-local semantics only. | match |
| `FileNameTake` | `Pure` | None; deterministic/session-local semantics only. | match |
| `FileNames` | `FileRead` | Lists/probes directories through the `FileIo` provider (`crates/rum-runtime/src/runtime/builtins_context_loader.rs:181-203`; `crates/rum-runtime/src/runtime/builtins_context_loader/file_names_glob.rs:84-133`). | match |
| `FindFile` | `FileRead` | Probes candidate paths through the `FileIo` provider (`crates/rum-runtime/src/runtime/builtins_context_loader.rs:210-244`; `crates/rum-runtime/src/runtime/resolution.rs:247-258`). | match |
| `First` | `Pure` | None; deterministic/session-local semantics only. | match |
| `FixedPoint` | `Pure` | None; deterministic/session-local semantics only. | match |
| `FixedPointList` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Flatten` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Floor` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Fold` | `Pure` | None; deterministic/session-local semantics only. | match |
| `FoldList` | `Pure` | None; deterministic/session-local semantics only. | match |
| `FoldPair` | `Pure` | None; deterministic/session-local semantics only. | match |
| `FoldWhile` | `Pure` | None; deterministic/session-local semantics only. | match |
| `For` | `Pure` | None; deterministic/session-local semantics only. | match |
| `FractionalPart` | `Pure` | None; deterministic/session-local semantics only. | match |
| `FreeQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `FromBase64` | `Pure` | None; deterministic/session-local semantics only. | match |
| `FromCsv` | `Pure` | None; deterministic/session-local semantics only. | match |
| `FromDateList` | `Pure` | None; deterministic/session-local semantics only. | match |
| `FromDateString` | `Pure` | None; deterministic/session-local semantics only. | match |
| `FromDigits` | `Pure` | None; deterministic/session-local semantics only. | match |
| `FromHexString` | `Pure` | None; deterministic/session-local semantics only. | match |
| `FromIni` | `Pure` | None; deterministic/session-local semantics only. | match |
| `FromJson` | `Pure` | None; deterministic/session-local semantics only. | match |
| `FromToml` | `Pure` | None; deterministic/session-local semantics only. | match |
| `FromXml` | `Pure` | None; deterministic/session-local semantics only. | match |
| `FromYaml` | `Pure` | None; deterministic/session-local semantics only. | match |
| `FullForm` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Function` | `Pure` | None; deterministic/session-local semantics only. | match |
| `GCD` | `Pure` | None; deterministic/session-local semantics only. | match |
| `GatherBy` | `Pure` | None; deterministic/session-local semantics only. | match |
| `General` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Get` | `FileRead` | Reads source through the `FileIo` provider and evaluates it (`crates/rum-runtime/src/runtime/builtins_context_loader.rs:247-294`; `crates/rum-runtime/src/runtime/session/session_eval.rs:299-312`). | match |
| `Greater` | `Pure` | None; deterministic/session-local semantics only. | match |
| `GreaterEqual` | `Pure` | None; deterministic/session-local semantics only. | match |
| `GreaterEqualThan` | `Pure` | None; deterministic/session-local semantics only. | match |
| `GreaterThan` | `Pure` | None; deterministic/session-local semantics only. | match |
| `GroupBy` | `Pure` | None; deterministic/session-local semantics only. | match |
| `HammingDistance` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Hash` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Head` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Hold` | `Pure` | None; deterministic/session-local semantics only. | match |
| `HoldAll` | `Pure` | None; deterministic/session-local semantics only. | match |
| `HoldAllComplete` | `Pure` | None; deterministic/session-local semantics only. | match |
| `HoldComplete` | `Pure` | None; deterministic/session-local semantics only. | match |
| `HoldFirst` | `Pure` | None; deterministic/session-local semantics only. | match |
| `HoldForm` | `Pure` | None; deterministic/session-local semantics only. | match |
| `HoldPattern` | `Pure` | None; deterministic/session-local semantics only. | match |
| `HoldRest` | `Pure` | None; deterministic/session-local semantics only. | match |
| `HtmlEscape` | `Pure` | None; deterministic/session-local semantics only. | match |
| `HtmlUnescape` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Identity` | `Pure` | None; deterministic/session-local semantics only. | match |
| `If` | `Pure` | None; deterministic/session-local semantics only. | match |
| `IgnoreCase` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Implies` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Increment` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Information` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Inner` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Input` | `Interactive` | Requests host input, then parses/evaluates it (`crates/rum-runtime/src/runtime/builtins_input.rs:83-95,224-232`). | match |
| `InputForm` | `Pure` | None; deterministic/session-local semantics only. | match |
| `InputString` | `Interactive` | Requests/suspends for one line of host input (`crates/rum-runtime/src/runtime/builtins_input.rs:74-81,224-232`). | match |
| `Insert` | `Pure` | None; deterministic/session-local semantics only. | match |
| `IntegerDigits` | `Pure` | None; deterministic/session-local semantics only. | match |
| `IntegerLength` | `Pure` | None; deterministic/session-local semantics only. | match |
| `IntegerPart` | `Pure` | None; deterministic/session-local semantics only. | match |
| `IntegerQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `IntegerReverse` | `Pure` | None; deterministic/session-local semantics only. | match |
| `IntersectingQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Intersection` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Join` | `Pure` | None; deterministic/session-local semantics only. | match |
| `JoinAcross` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Key` | `Pure` | None; deterministic/session-local semantics only. | match |
| `KeyComplement` | `Pure` | None; deterministic/session-local semantics only. | match |
| `KeyDrop` | `Pure` | None; deterministic/session-local semantics only. | match |
| `KeyDropFrom` | `Pure` | None; deterministic/session-local semantics only. | match |
| `KeyExistsQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `KeyFreeQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `KeyGroupBy` | `Pure` | None; deterministic/session-local semantics only. | match |
| `KeyIntersection` | `Pure` | None; deterministic/session-local semantics only. | match |
| `KeyMap` | `Pure` | None; deterministic/session-local semantics only. | match |
| `KeyMemberQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `KeySelect` | `Pure` | None; deterministic/session-local semantics only. | match |
| `KeySort` | `Pure` | None; deterministic/session-local semantics only. | match |
| `KeySortBy` | `Pure` | None; deterministic/session-local semantics only. | match |
| `KeyTake` | `Pure` | None; deterministic/session-local semantics only. | match |
| `KeyUnion` | `Pure` | None; deterministic/session-local semantics only. | match |
| `KeyValueMap` | `Pure` | None; deterministic/session-local semantics only. | match |
| `KeyValuePattern` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Keys` | `Pure` | None; deterministic/session-local semantics only. | match |
| `LCM` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Last` | `Pure` | None; deterministic/session-local semantics only. | match |
| `LazyDrop` | `Pure` | None; deterministic/session-local semantics only. | match |
| `LazyFoldList` | `Pure` | None; deterministic/session-local semantics only. | match |
| `LazyList` | `Pure` | None; deterministic/session-local semantics only. | match |
| `LazyMap` | `Pure` | None; deterministic/session-local semantics only. | match |
| `LazyNestList` | `Pure` | None; deterministic/session-local semantics only. | match |
| `LazyPermutations` | `Pure` | None; deterministic/session-local semantics only. | match |
| `LazyRange` | `Pure` | None; deterministic/session-local semantics only. | match |
| `LazySelect` | `Pure` | None; deterministic/session-local semantics only. | match |
| `LazySubsets` | `Pure` | None; deterministic/session-local semantics only. | match |
| `LazyTable` | `Pure` | None; deterministic/session-local semantics only. | match |
| `LazyTake` | `Pure` | None; deterministic/session-local semantics only. | match |
| `LazyTuples` | `Pure` | None; deterministic/session-local semantics only. | match |
| `LeafCount` | `Pure` | None; deterministic/session-local semantics only. | match |
| `LeapYearQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Length` | `Pure` | None; deterministic/session-local semantics only. | match |
| `LengthWhile` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Less` | `Pure` | None; deterministic/session-local semantics only. | match |
| `LessEqual` | `Pure` | None; deterministic/session-local semantics only. | match |
| `LessEqualThan` | `Pure` | None; deterministic/session-local semantics only. | match |
| `LessThan` | `Pure` | None; deterministic/session-local semantics only. | match |
| `LetterCharacter` | `Pure` | None; deterministic/session-local semantics only. | match |
| `LetterQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Level` | `Pure` | None; deterministic/session-local semantics only. | match |
| `LexicographicOrder` | `Pure` | None; deterministic/session-local semantics only. | match |
| `ListQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Longest` | `Pure` | None; deterministic/session-local semantics only. | match |
| `LongestCommonSubsequence` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Lookup` | `Pure` | None; deterministic/session-local semantics only. | match |
| `LowerCaseQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `LowercaseLetters` | `Pure` | None; deterministic/session-local semantics only. | match |
| `MachineNumberQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Map` | `Pure` | None; deterministic/session-local semantics only. | match |
| `MapAll` | `Pure` | None; deterministic/session-local semantics only. | match |
| `MapApply` | `Pure` | None; deterministic/session-local semantics only. | match |
| `MapAt` | `Pure` | None; deterministic/session-local semantics only. | match |
| `MapIndexed` | `Pure` | None; deterministic/session-local semantics only. | match |
| `MapThread` | `Pure` | None; deterministic/session-local semantics only. | match |
| `MatchQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `MatrixQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Max` | `Pure` | None; deterministic/session-local semantics only. | match |
| `MaxBy` | `Pure` | None; deterministic/session-local semantics only. | match |
| `MaxMemoryUsed` | `Diagnostic` | Samples/discloses the session memory-proxy high-water mark (`crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:33-46`). | match |
| `MaximalBy` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Md5` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Mean` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Median` | `Pure` | None; deterministic/session-local semantics only. | match |
| `MemberQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `MemoryConstrained` | `Pure` | None; deterministic/session-local semantics only. | match |
| `MemoryInUse` | `Diagnostic` | Discloses the current session memory-proxy count (`crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:19-30`). | match |
| `Merge` | `Pure` | None; deterministic/session-local semantics only. | match |
| `MergeNested` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Message` | `Pure` | None; deterministic/session-local semantics only. | match |
| `MessageDefinitions` | `Pure` | None; deterministic/session-local semantics only. | match |
| `MessageName` | `Pure` | None; deterministic/session-local semantics only. | match |
| `MessageTags` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Messages` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Min` | `Pure` | None; deterministic/session-local semantics only. | match |
| `MinBy` | `Pure` | None; deterministic/session-local semantics only. | match |
| `MinMax` | `Pure` | None; deterministic/session-local semantics only. | match |
| `MinimalBy` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Minus` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Missing` | `Pure` | None; deterministic/session-local semantics only. | match |
| `MissingQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Mod` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Module` | `Pure` | None; deterministic/session-local semantics only. | match |
| `MoebiusMu` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Monitor` | `Pure` | Session-mediated print/callback stream only; no direct stdout/stderr or ambient authority (`crates/rum-runtime/src/runtime/session/session_helpers.rs:227-233`). | match (session-state doctrine) |
| `Most` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Multinomial` | `Pure` | None; deterministic/session-local semantics only. | match |
| `NameQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Names` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Nand` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Nearest` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Needs` | `FileRead` | Conditionally reaches `Get`/`FileIo` package loading (`crates/rum-runtime/src/runtime/builtins_context_loader.rs:297-349`). | match |
| `Negative` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Nest` | `Pure` | None; deterministic/session-local semantics only. | match |
| `NestList` | `Pure` | None; deterministic/session-local semantics only. | match |
| `NestWhile` | `Pure` | None; deterministic/session-local semantics only. | match |
| `NestWhileList` | `Pure` | None; deterministic/session-local semantics only. | match |
| `NextPrime` | `Pure` | None; deterministic/session-local semantics only. | match |
| `NonNegative` | `Pure` | None; deterministic/session-local semantics only. | match |
| `NonPositive` | `Pure` | None; deterministic/session-local semantics only. | match |
| `NoneTrue` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Nor` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Normal` | `Pure` | None; deterministic/session-local semantics only. | match |
| `NormalizeRules` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Not` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Nothing` | `Pure` | None; deterministic/session-local semantics only. | match |
| `NumberQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `NumberString` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Numerator` | `Pure` | None; deterministic/session-local semantics only. | match |
| `NumericQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `NumericalOrder` | `Pure` | None; deterministic/session-local semantics only. | match |
| `OddQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Off` | `Pure` | None; deterministic/session-local semantics only. | match |
| `On` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Once` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Operate` | `Pure` | None; deterministic/session-local semantics only. | match |
| `OperatorApplied` | `Pure` | None; deterministic/session-local semantics only. | match |
| `OptionValue` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Options` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Or` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Order` | `Pure` | None; deterministic/session-local semantics only. | match |
| `OrderedQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Outer` | `Pure` | None; deterministic/session-local semantics only. | match |
| `OutputForm` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Overlaps` | `Pure` | None; deterministic/session-local semantics only. | match |
| `OwnValues` | `Pure` | None; deterministic/session-local semantics only. | match |
| `PackageScope` | `Pure` | None; deterministic/session-local semantics only. | match |
| `PadLeft` | `Pure` | None; deterministic/session-local semantics only. | match |
| `PadRight` | `Pure` | None; deterministic/session-local semantics only. | match |
| `ParentDirectory` | `Pure` | None; deterministic/session-local semantics only. | match |
| `ParseArgs` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Part` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Partition` | `Pure` | None; deterministic/session-local semantics only. | match |
| `PartitionsP` | `Pure` | None; deterministic/session-local semantics only. | match |
| `PartitionsQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Pattern` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Pause` | `Clock` | Sleeps the current host thread for the parsed duration (`crates/rum-runtime/src/runtime/builtins_input.rs:107-126`). | match |
| `Permutations` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Pluralize` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Plus` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Pochhammer` | `Pure` | None; deterministic/session-local semantics only. | match |
| `PolynomialExpressionQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `PolynomialQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Position` | `Pure` | None; deterministic/session-local semantics only. | match |
| `PositionIndex` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Positive` | `Pure` | None; deterministic/session-local semantics only. | match |
| `PossibleZeroQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Power` | `Pure` | None; deterministic/session-local semantics only. | match |
| `PowerMod` | `Pure` | None; deterministic/session-local semantics only. | match |
| `PreDecrement` | `Pure` | None; deterministic/session-local semantics only. | match |
| `PreIncrement` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Prepend` | `Pure` | None; deterministic/session-local semantics only. | match |
| `PrependTo` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Prime` | `Pure` | None; deterministic/session-local semantics only. | match |
| `PrimeNu` | `Pure` | None; deterministic/session-local semantics only. | match |
| `PrimeOmega` | `Pure` | None; deterministic/session-local semantics only. | match |
| `PrimePi` | `Pure` | None; deterministic/session-local semantics only. | match |
| `PrimeQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Print` | `Pure` | Session-mediated print/callback stream only; no direct stdout/stderr or ambient authority (`crates/rum-runtime/src/runtime/session/session_helpers.rs:227-233`). | match (session-state doctrine) |
| `Product` | `Pure` | None; deterministic/session-local semantics only. | match |
| `PropagateAborts` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Protect` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Public` | `Pure` | None; deterministic/session-local semantics only. | match |
| `PunctuationCharacter` | `Pure` | None; deterministic/session-local semantics only. | match |
| `PunctuationCharacters` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Quantile` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Quiet` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Quotient` | `Pure` | None; deterministic/session-local semantics only. | match |
| `RandomBytes` | `Random` | Fills a byte buffer through the host entropy provider (`crates/rum-stdlib/src/random.rs:344-363`; `crates/rum-runtime/src/runtime/session/session_api.rs:497-503`). | match |
| `RandomChoice` | `Pure` | Session-local deterministic PRNG state only; no wall clock or host entropy (`crates/rum-runtime/src/runtime/session/session_api.rs:505-540`). | match (documented Pure/entropy split) |
| `RandomInteger` | `Pure` | Session-local deterministic PRNG state only; no wall clock or host entropy (`crates/rum-runtime/src/runtime/session/session_api.rs:505-540`). | match (documented Pure/entropy split) |
| `RandomPassword` | `Random` | Draws password indices through the host entropy provider (`crates/rum-stdlib/src/random.rs:269-301,416-457`). | match |
| `RandomSample` | `Pure` | Session-local deterministic PRNG state only; no wall clock or host entropy (`crates/rum-runtime/src/runtime/session/session_api.rs:505-540`). | match (documented Pure/entropy split) |
| `RandomString` | `Pure` | Session-local deterministic PRNG state only; no wall clock or host entropy (`crates/rum-runtime/src/runtime/session/session_api.rs:505-540`). | match (documented Pure/entropy split) |
| `RandomUuid` | `Pure` | Session-local deterministic PRNG state only; no wall clock or host entropy (`crates/rum-runtime/src/runtime/session/session_api.rs:505-540`). | match (documented Pure/entropy split) |
| `Range` | `Pure` | None; deterministic/session-local semantics only. | match |
| `RationalExpressionQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `RealValuedNumberQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `RealValuedNumericQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Reap` | `Pure` | None; deterministic/session-local semantics only. | match |
| `RegisterExceptionType` | `Pure` | None; deterministic/session-local semantics only. | match |
| `RegularExpression` | `Pure` | None; deterministic/session-local semantics only. | match |
| `ReleaseHold` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Remove` | `Pure` | None; deterministic/session-local semantics only. | match |
| `RemoveDiacritics` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Repeated` | `Pure` | None; deterministic/session-local semantics only. | match |
| `RepeatedNull` | `Pure` | None; deterministic/session-local semantics only. | match |
| `RepeatedTiming` | `Clock` | Reads a monotonic clock for three samples (`crates/rum-runtime/src/runtime/debug_tools/timing.rs:77-120`). | match |
| `Replace` | `Pure` | None; deterministic/session-local semantics only. | match |
| `ReplaceAll` | `Pure` | None; deterministic/session-local semantics only. | match |
| `ReplaceAllUnheld` | `Pure` | None; deterministic/session-local semantics only. | match |
| `ReplaceAt` | `Pure` | None; deterministic/session-local semantics only. | match |
| `ReplaceList` | `Pure` | None; deterministic/session-local semantics only. | match |
| `ReplacePart` | `Pure` | None; deterministic/session-local semantics only. | match |
| `ReplaceRepeated` | `Pure` | None; deterministic/session-local semantics only. | match |
| `ReseedRandom` | `Random` | Reads host entropy and reseeds the session PRNG (`crates/rum-stdlib/src/random.rs:26-40`; `crates/rum-runtime/src/runtime/session/session_api.rs:513-517`). | match |
| `ResolveContextAliases` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Rest` | `Pure` | None; deterministic/session-local semantics only. | match |
| `RestoreSealedReferences` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Return` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Reverse` | `Pure` | None; deterministic/session-local semantics only. | match |
| `ReverseApplied` | `Pure` | None; deterministic/session-local semantics only. | match |
| `ReverseSort` | `Pure` | None; deterministic/session-local semantics only. | match |
| `ReverseSortBy` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Riffle` | `Pure` | None; deterministic/session-local semantics only. | match |
| `RightComposition` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Round` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Row` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Rule` | `Pure` | None; deterministic/session-local semantics only. | match |
| `RuleDelayed` | `Pure` | None; deterministic/session-local semantics only. | match |
| `RumProfileReset` | `Diagnostic` | Resets env-gated process/thread profiling counters (`crates/rum-runtime/src/runtime/debug_tools/basic.rs:111-121`; `crates/rum-runtime/src/runtime/profiling/reset.rs:3-10`). | match |
| `RumProfileSnapshot` | `Diagnostic` | Reads profiling counters and may write the diagnostic snapshot to stderr (`crates/rum-runtime/src/runtime/debug_tools/basic.rs:124-138`; `crates/rum-runtime/src/runtime/profiling/summary.rs:19-31`). | match |
| `RumTokens` | `Pure` | None; deterministic/session-local semantics only. | match |
| `SameAs` | `Pure` | None; deterministic/session-local semantics only. | match |
| `SameQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Save` | `FileWrite` | Appends serialized definitions through `FileIo::write_string` (`crates/rum-runtime/src/runtime/package_tools.rs:79-81,597-612`). | match |
| `Scan` | `Pure` | None; deterministic/session-local semantics only. | match |
| `SchemaErrors` | `Pure` | None; deterministic/session-local semantics only. | match |
| `SeedRandom` | `Pure` | Session-local deterministic PRNG state only; no wall clock or host entropy (`crates/rum-runtime/src/runtime/session/session_api.rs:505-540`). | match (documented Pure/entropy split) |
| `Select` | `Pure` | None; deterministic/session-local semantics only. | match |
| `SelectFirst` | `Pure` | None; deterministic/session-local semantics only. | match |
| `SelectPositions` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Sequence` | `Pure` | None; deterministic/session-local semantics only. | match |
| `SequenceAlignment` | `Pure` | None; deterministic/session-local semantics only. | match |
| `SequenceFold` | `Pure` | None; deterministic/session-local semantics only. | match |
| `SequenceFoldList` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Set` | `Pure` | None; deterministic/session-local semantics only. | match |
| `SetAttributes` | `Pure` | None; deterministic/session-local semantics only. | match |
| `SetDelayed` | `Pure` | None; deterministic/session-local semantics only. | match |
| `SetOptions` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Sha1` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Sha256` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Sha512` | `Pure` | None; deterministic/session-local semantics only. | match |
| `ShellQuote` | `Pure` | None; deterministic/session-local semantics only. | match |
| `ShellSplit` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Short` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Shortest` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Sign` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Sort` | `Pure` | None; deterministic/session-local semantics only. | match |
| `SortBy` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Sow` | `Pure` | None; deterministic/session-local semantics only. | match |
| `SpellingCorrection` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Stack` | `Pure` | None; deterministic/session-local semantics only. | match |
| `StandardForm` | `Pure` | None; deterministic/session-local semantics only. | match |
| `StartOfLine` | `Pure` | None; deterministic/session-local semantics only. | match |
| `StartOfString` | `Pure` | None; deterministic/session-local semantics only. | match |
| `StirlingS1` | `Pure` | None; deterministic/session-local semantics only. | match |
| `StirlingS2` | `Pure` | None; deterministic/session-local semantics only. | match |
| `StringApply` | `Pure` | None; deterministic/session-local semantics only. | match |
| `StringCases` | `Pure` | None; deterministic/session-local semantics only. | match |
| `StringContainsQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `StringCount` | `Pure` | None; deterministic/session-local semantics only. | match |
| `StringDedent` | `Pure` | None; deterministic/session-local semantics only. | match |
| `StringDelete` | `Pure` | None; deterministic/session-local semantics only. | match |
| `StringDrop` | `Pure` | None; deterministic/session-local semantics only. | match |
| `StringEndsQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `StringExpandTabs` | `Pure` | None; deterministic/session-local semantics only. | match |
| `StringExpression` | `Pure` | None; deterministic/session-local semantics only. | match |
| `StringForm` | `Pure` | None; deterministic/session-local semantics only. | match |
| `StringFreeQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `StringIndent` | `Pure` | None; deterministic/session-local semantics only. | match |
| `StringJoin` | `Pure` | None; deterministic/session-local semantics only. | match |
| `StringLength` | `Pure` | None; deterministic/session-local semantics only. | match |
| `StringLines` | `Pure` | None; deterministic/session-local semantics only. | match |
| `StringMatchQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `StringPadLeft` | `Pure` | None; deterministic/session-local semantics only. | match |
| `StringPadRight` | `Pure` | None; deterministic/session-local semantics only. | match |
| `StringPart` | `Pure` | None; deterministic/session-local semantics only. | match |
| `StringPosition` | `Pure` | None; deterministic/session-local semantics only. | match |
| `StringQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `StringRepeat` | `Pure` | None; deterministic/session-local semantics only. | match |
| `StringReplace` | `Pure` | None; deterministic/session-local semantics only. | match |
| `StringReverse` | `Pure` | None; deterministic/session-local semantics only. | match |
| `StringRiffle` | `Pure` | None; deterministic/session-local semantics only. | match |
| `StringSplit` | `Pure` | None; deterministic/session-local semantics only. | match |
| `StringStartsQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `StringTake` | `Pure` | None; deterministic/session-local semantics only. | match |
| `StringTakeDrop` | `Pure` | None; deterministic/session-local semantics only. | match |
| `StringTemplate` | `Pure` | None; deterministic/session-local semantics only. | match |
| `StringToByteArray` | `Pure` | None; deterministic/session-local semantics only. | match |
| `StringTrim` | `Pure` | None; deterministic/session-local semantics only. | match |
| `StringWrap` | `Pure` | None; deterministic/session-local semantics only. | match |
| `SubValues` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Subfactorial` | `Pure` | None; deterministic/session-local semantics only. | match |
| `SubsetMap` | `Pure` | None; deterministic/session-local semantics only. | match |
| `SubsetQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Subsets` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Subtract` | `Pure` | None; deterministic/session-local semantics only. | match |
| `SubtractFrom` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Success` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Sum` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Switch` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Symbol` | `Pure` | None; deterministic/session-local semantics only. | match |
| `SymbolName` | `Pure` | None; deterministic/session-local semantics only. | match |
| `SyntaxInformation` | `Pure` | None; deterministic/session-local semantics only. | match |
| `SyntaxQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Table` | `Pure` | None; deterministic/session-local semantics only. | match |
| `TagSet` | `Pure` | None; deterministic/session-local semantics only. | match |
| `TagSetDelayed` | `Pure` | None; deterministic/session-local semantics only. | match |
| `TagUnset` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Take` | `Pure` | None; deterministic/session-local semantics only. | match |
| `TakeDrop` | `Pure` | None; deterministic/session-local semantics only. | match |
| `TakeLargestBy` | `Pure` | None; deterministic/session-local semantics only. | match |
| `TakeList` | `Pure` | None; deterministic/session-local semantics only. | match |
| `TakeSmallestBy` | `Pure` | None; deterministic/session-local semantics only. | match |
| `TakeWhile` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Tally` | `Pure` | None; deterministic/session-local semantics only. | match |
| `TemplateApply` | `Pure` | None; deterministic/session-local semantics only. | match |
| `TemplateObject` | `Pure` | None; deterministic/session-local semantics only. | match |
| `TerminatedEvaluation` | `Pure` | None; deterministic/session-local semantics only. | match |
| `TestCreate` | `Pure` | None; deterministic/session-local semantics only. | match |
| `TestReport` | `Clock` | Reads a monotonic clock for per-test timing (`crates/rum-runtime/src/runtime/debug_tools/test_report/evaluation.rs:175-176,236-237`). | match |
| `TextString` | `Pure` | None; deterministic/session-local semantics only. | match |
| `TextWords` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Thread` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Through` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Throw` | `Pure` | None; deterministic/session-local semantics only. | match |
| `ThrowException` | `Pure` | None; deterministic/session-local semantics only. | match |
| `TimeConstrained` | `Clock` | Reads a monotonic clock to establish and poll a deadline (`crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:54-60,157-160,277-281`). | match |
| `TimeObjectQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Times` | `Pure` | None; deterministic/session-local semantics only. | match |
| `TimesBy` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Timing` | `Clock` | Reads a monotonic clock around evaluation (`crates/rum-runtime/src/runtime/debug_tools/timing.rs:55-74`). | match |
| `ToBase64` | `Pure` | None; deterministic/session-local semantics only. | match |
| `ToCharacterCode` | `Pure` | None; deterministic/session-local semantics only. | match |
| `ToCsv` | `Pure` | None; deterministic/session-local semantics only. | match |
| `ToExpression` | `Pure` | None; deterministic/session-local semantics only. | match |
| `ToHexString` | `Pure` | None; deterministic/session-local semantics only. | match |
| `ToIni` | `Pure` | None; deterministic/session-local semantics only. | match |
| `ToJson` | `Pure` | None; deterministic/session-local semantics only. | match |
| `ToLowerCase` | `Pure` | None; deterministic/session-local semantics only. | match |
| `ToString` | `Pure` | None; deterministic/session-local semantics only. | match |
| `ToToml` | `Pure` | None; deterministic/session-local semantics only. | match |
| `ToUpperCase` | `Pure` | None; deterministic/session-local semantics only. | match |
| `ToXml` | `Pure` | None; deterministic/session-local semantics only. | match |
| `ToYaml` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Together` | `Pure` | None; deterministic/session-local semantics only. | match |
| `TokenizeWith` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Total` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Trace` | `Pure` | None; deterministic/session-local semantics only. | match |
| `TraceDetail` | `Pure` | None; deterministic/session-local semantics only. | match |
| `TracePrint` | `Pure` | Session-mediated print/callback stream only; no direct stdout/stderr or ambient authority (`crates/rum-runtime/src/runtime/session/session_helpers.rs:227-233`). | match (session-state doctrine) |
| `TraditionalForm` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Transpose` | `Pure` | None; deterministic/session-local semantics only. | match |
| `TrueQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Tuples` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Uncompress` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Unequal` | `Pure` | None; deterministic/session-local semantics only. | match |
| `UnequalTo` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Unevaluated` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Union` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Unique` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Unprotect` | `Pure` | None; deterministic/session-local semantics only. | match |
| `UnsameQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Unset` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Until` | `Pure` | None; deterministic/session-local semantics only. | match |
| `UpSet` | `Pure` | None; deterministic/session-local semantics only. | match |
| `UpSetDelayed` | `Pure` | None; deterministic/session-local semantics only. | match |
| `UpTo` | `Pure` | None; deterministic/session-local semantics only. | match |
| `UpValues` | `Pure` | None; deterministic/session-local semantics only. | match |
| `UpperCaseQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `UppercaseLetters` | `Pure` | None; deterministic/session-local semantics only. | match |
| `UrlDecode` | `Pure` | None; deterministic/session-local semantics only. | match |
| `UrlEncode` | `Pure` | None; deterministic/session-local semantics only. | match |
| `UrlParse` | `Pure` | None; deterministic/session-local semantics only. | match |
| `UuidQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `ValueQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Values` | `Pure` | None; deterministic/session-local semantics only. | match |
| `VectorQ` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Which` | `Pure` | None; deterministic/session-local semantics only. | match |
| `While` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Whitespace` | `Pure` | None; deterministic/session-local semantics only. | match |
| `WhitespaceCharacter` | `Pure` | None; deterministic/session-local semantics only. | match |
| `WhitespaceCharacters` | `Pure` | None; deterministic/session-local semantics only. | match |
| `With` | `Pure` | None; deterministic/session-local semantics only. | match |
| `WithCleanup` | `Pure` | None; deterministic/session-local semantics only. | match |
| `WordBoundary` | `Pure` | None; deterministic/session-local semantics only. | match |
| `WordCharacter` | `Pure` | None; deterministic/session-local semantics only. | match |
| `XmlElement` | `Pure` | None; deterministic/session-local semantics only. | match |
| `Xor` | `Pure` | None; deterministic/session-local semantics only. | match |

**Flagged classification rows: 2.**

- `BeginPackage`: spec defaults to `Pure` at `crates/rum-runtime/src/runtime/builtin_catalog/namespace/contexts.rs:231-246`; the dependency loop reaches `eval_needs` at `crates/rum-runtime/src/runtime/builtins_context_loader.rs:430-432`, and `eval_needs` reaches `eval_get` at `:336`, whose provider read is at `crates/rum-runtime/src/runtime/session/session_eval.rs:299-308`. Proposed ruling: reclassify `BeginPackage` as `FileRead`, or make the reference explicitly express a conditional/transitive FileRead requirement if preserving a Pure label for the no-dependency form.
- `DeclarePackage`: spec defaults to `Pure` at `crates/rum-runtime/src/runtime/builtin_catalog/namespace/contexts.rs:260-272`; it installs the stub at `crates/rum-runtime/src/runtime/package_tools.rs:44-76`, and later evaluation reaches `eval_needs` at `crates/rum-runtime/src/runtime/eval_core/symbol.rs:50-63`, with the same provider-read sink at `crates/rum-runtime/src/runtime/session/session_eval.rs:299-308`. Proposed ruling: reclassify `DeclarePackage` as `FileRead` if the latent autoload is owned by this builtin; otherwise split/document registration versus the separately gated autoload effect so the generated reference does not imply a wholly Pure lifecycle.

All other rows match. Capability labels describe the effect a valid call may perform, not whether every arity/error/zero-size branch performs it.

## 2. Gate-coverage table

The brief's checkpoint pointer has drifted: `crates/rum-runtime/src/runtime/builtins_context_loader.rs:63-78` is a private loader guard that receives an explicit capability; it does **not** consult a `BuiltinSpec`. The single checkpoint that consults the declared `spec.capability` is `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:122-149`, specifically `:135-138`, and it returns the sandbox-denied value before `run_builtin_handler` at `:147`. `rg` finds only that call to `run_builtin_handler` (`:164-170`), so there is no sibling handler invocation that skips the declared-label check.

| Reachability / dispatch entry | Capability-bearing builtin(s) or effect | Checkpoint trace | Verdict |
|---|---|---|---|
| Direct ordinary call | All 25 non-`Pure` catalog entries (all are ordinary dispatch) | Call plan adds `OrdinaryBuiltin` at `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:54-56`; the stage calls `try_dispatch_builtin_call` at `:115-117`; declared capability is denied at `:135-138`; handler/effect begins only at `:147`. | covered |
| Direct special-form call | No current non-`Pure` catalog entry; applies to a host-registered capability-bearing special form | `SpecialForm` calls the same `try_dispatch_builtin_call` at `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:81-83`, before any handler. | covered |
| Runtime-constructed head (`Symbol["Get"]`, `ToExpression[...]`) | Any effecting builtin, demonstrated for `Get` | The constructed call is evaluated normally, its System symbol resolves to the registered spec at `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:4-12`, then follows the ordinary/special checkpoint above. Tests pin both forms at `crates/rum-host/src/tests/capability_profiles.rs:186-199`. | covered |
| Higher-order/synthesized call (`Apply`, functions, definitions, held evaluation, listable threading, loaded source) | Any nested effecting builtin | These mechanisms eventually call `Session::evaluate`; call heads are re-planned through `build_call_dispatch_plan` (`crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:4-69`). The outer Pure builtin does not invoke an effect handler directly; each nested capability-bearing head is checked on its own dispatch. | covered |
| Registered stdlib/fallback handler | All stdlib effects (`Exit`, entropy heads) and any host-registered effecting builtin | `register_builtin` preserves/installs the effective spec, including fallback capability, at `crates/rum-runtime/src/runtime/session/session_api.rs:171-180`; lookup copies that spec at `crates/rum-runtime/src/runtime/resolution.rs:71-97`; dispatch checks its capability before the handler. | covered |
| `FindFile` | Provider path probe | No internal direct caller exists; adapter `builtin_find_file` reaches `eval_find_file` only after ordinary dispatch (`crates/rum-runtime/src/runtime/builtin_adapters.rs:391-397`). The probe is downstream at `crates/rum-runtime/src/runtime/resolution.rs:247-258`. | covered |
| `FileNames` | Provider directory list/probes | Direct dispatch checks declared `FileRead`, then `eval_file_names` redundantly calls the explicit loader guard before acquiring the provider (`crates/rum-runtime/src/runtime/builtins_context_loader.rs:181-202`). | covered twice |
| Direct `Get` / `Needs` | Provider probe/read | Direct dispatch checks declared `FileRead`; each method then redundantly checks the explicit `FileRead` loader guard before provider presence/access (`crates/rum-runtime/src/runtime/builtins_context_loader.rs:247-250,297-300`). | covered twice |
| `Needs` internal call to `Get` | Provider probe/read | `eval_needs` passes its guard at `crates/rum-runtime/src/runtime/builtins_context_loader.rs:297-300`, then calls `eval_get` at `:336`; `eval_get` rechecks at `:247-250` before the read at `crates/rum-runtime/src/runtime/session/session_eval.rs:299-308`. | covered twice at effect seam |
| `BeginPackage[..., dependencies]` internal loader call | Conditional provider probe/read despite `BeginPackage` being declared `Pure` | The Pure outer handler bypasses declared-FileRead dispatch because it directly calls `eval_needs` at `crates/rum-runtime/src/runtime/builtins_context_loader.rs:430-432`; `eval_needs` checks explicit `FileRead` at `:297-300` before any provider access. Restricted+provider denial/rollback is pinned at `crates/rum-host/src/tests/capability_profiles.rs:132-168`. | effect covered; classification flagged in section 1 |
| Declared-stub autoload after `DeclarePackage` | Conditional provider probe/read despite `DeclarePackage` being declared `Pure` | Symbol evaluation calls `eval_needs` at `crates/rum-runtime/src/runtime/eval_core/symbol.rs:50-63`; the same explicit `FileRead` guard at `builtins_context_loader.rs:297-300` runs before provider access. Restricted+provider denial is pinned at `crates/rum-host/src/tests/capability_profiles.rs:170-183`. | effect covered; classification flagged in section 1 |
| Interactive suspension and resume | `Dialog`, `Input`, `InputString` | Initial call passes the declared `Interactive` check before `drive_input_request` can return `EvalError::InputRequired` (`crates/rum-runtime/src/runtime/builtins_input.rs:74-95,224-232`). Resume continues the already-authorized suspended evaluation; it cannot create an initial request on an unchecked path. | covered |
| Host-facing `Session::eval_file` / request APIs | Direct host-requested file read, not a language builtin dispatch | `Session::eval_file` reads the configured provider directly (`crates/rum-runtime/src/runtime/session/session_eval.rs:299-308`). This is explicit embedding authority exercised by Rust host code and is outside `BuiltinSpec` policy; every language path into it is covered above. | out of builtin-policy scope, not a bypass |
| Host-facing provider methods (`fill_entropy`, `file_io`, print sink) | Direct host integration, not a language builtin dispatch | Rust callers can invoke/install providers directly (`crates/rum-runtime/src/runtime/session/session_api.rs:471-503`). Catalog handlers reach them only after the dispatch traces above. | out of builtin-policy scope, not a bypass |

**Flagged gate-coverage gaps: none.** The only finding is the stale checkpoint location in the brief: declared-label enforcement is at `eval_dispatch/mod.rs:135-138`, while `builtins_context_loader.rs:68` is the additional internal-loader guard that closes the former `BeginPackage`/declared-stub bypass.

## 3. Limit-coverage table

Sections 3 and 4 were completed by the director. Codex's run was terminated by
the upstream content filter during this section (the resource-exhaustion probes
themselves — e.g. `IntegerLength[2^(10^9)]` — read as cybersecurity work), so the
live limit probing is director-run.

Defaults are `ResourceLimits::default()`
(`crates/rum-runtime/src/runtime/model/config.rs:25-38`) with values from
`crates/rum-runtime/src/runtime/metadata.rs:4-33`. Probes are the release CLI
under `scripts/run-bounded.py`; each `TerminatedEvaluation[...]` is a fail-closed
verdict.

| Resource limit | Default | On by default? | Enforcement point | Probe → verdict |
|---|---|---|---|---|
| Recursion depth | `recursion_limit = 1024` | yes | `metadata.rs:5`; config `config.rs:28` | see native-stack row (stack fires first for self-recursion) |
| Native stack | `native_stack_limit = 1 MiB` (derived for a declared stack of at least 2 MiB) | yes | `metadata.rs:6-29`; config `config.rs:29` | `f[n_]:=f[n+1]; f[0]` → `TerminatedEvaluation["StackLimit"]` (`General::reslim`, 1048576 bytes) |
| Iteration work | `iteration_work_limit = 100_000` | yes | `metadata.rs:33`; `debug_tools/resources/constraints.rs:206-210` | `Do[0,{50000000}]` → `TerminatedEvaluation["IterationWorkLimit"]` |
| Generated elements | `generated_element_limit = 100_000` | yes | `metadata.rs:31` | `Range[10^9]` → `TerminatedEvaluation["GeneratedElementLimit"]` |
| Parse nesting depth | `MAX_DELIMITER_NESTING_DEPTH = 128` (compile-time) | yes | `crates/rum-syntax/src/parser/front.rs:10,97-113` | 200 nested `(...)` → parse error `parse nesting exceeded limit` (fail-closed at parse time) |
| Operator-chain depth | `MAX_OPERATOR_CHAIN_DEPTH = 128` (compile-time) | yes | `crates/rum-syntax/src/parser/front.rs:20,115-131` | 16 000-link `^` chain → parse error `parse nesting exceeded limit 128` (fail-closed at parse time; sized to the native stack budget — a 2 MiB thread aborts at ~452 links, so the guard fires first) |
| Left-associative chain length | `MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS = 4_096` (compile-time) | yes | constant: `crates/rum-syntax/src/lib.rs:17-18`; parser: `crates/rum-syntax/src/parser/front.rs:223-226,257-292,802-812`; direct-AST lowering: `crates/rum-runtime/src/runtime/lowering/binary/left_chain.rs:130-162`, `comparison.rs:75-112` | 4 096 links → accepted; link 4 097 and 20 000-link `+`/comparison chains → syntax error `left-associative chain exceeded limit 4096` at the rejected operator (fail-closed; implicit `Times` included; nested/rhs spines count independently) |
| Statement-chain length | `MAX_STATEMENT_CHAIN_DEPTH = 256` (compile-time) | yes | `crates/rum-syntax/src/parser/front.rs:27,217-222,294-320,357-396` | 300-statement one-line `;` chain → parse error `statement chain exceeded limit 256 … split the statements across separate lines` (fail-closed at parse time; per-line script splitting unaffected) |
| Match depth | `MAX_MATCH_DEPTH = 512` (compile-time) | yes | `crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/match_choices.rs:4,17-19` | bounded by construction in the orderless matcher |
| ReplaceRepeated | `replace_repeated_limit = 256` | yes | `metadata.rs:4` | per-call runaway budget (brief-184) |
| Evaluation passes | `evaluation_pass_limit = 256` | yes | `metadata.rs:30` | fixed-point runaway budget |
| Integer work | `integer_work_limit = 100_000` | yes, but **Factorial only** | `metadata.rs:32`; enforced ONLY at `crates/rum-runtime/src/runtime/arithmetic/factorial.rs:16,52,74-81` | **`IntegerLength[2^(10^9)]` → timed out at 12s, NOT fail-closed** — see flagged finding |
| Request memory ceiling | `eval_memory_limit = None` | **NO — opt-in** | `config.rs:19,32`; `MemoryConstrained` frame + host setter | off in the default/scripting profile; host must set it |
| Wall-clock timeout | none | **NO — opt-in** | `crates/rum-runtime/src/runtime/session/session_api.rs:93` (`set_eval_timeout`) | off by default; host must set it |

**Flagged limit-coverage finding (1): `Power`/general big-integer producers are
not bounded by `integer_work_limit`.** The `integer_work_limit` guard is wired
only into `Factorial` (`arithmetic/factorial.rs`). Exponentiation reaches num-bigint
directly, so `2^(10^9)` (a ~10^9-bit exact integer) computes to completion, bounded
only by the two opt-in backstops. The always-on structural limits bound the SHAPE of
a computation (recursion/stack/iteration/generated-count/parse/match depth) but NOT
the COST of a single large exact-integer value. Proposed follow-up (director →
user): a fix brief extending `integer_work_limit` enforcement to `Power` (and any
other single-op big-integer producer — bit shifts, large `Times`), matching the
Factorial guard, so a single huge-integer op fails closed by default. Candidate for
the security-audit bucket; deferred as its own brief.

**Operational conclusion for the threat model (feeds SECURITY.md):** untrusted
evaluation REQUIRES the host to set both opt-in bounds — a request memory ceiling
(`eval_memory_limit`) and a wall-clock timeout (`set_eval_timeout`). The always-on
structural limits alone do not bound the CPU/memory of a single expensive arithmetic
operation. This is the intended architecture (the host owns the resource budget), but
it must be stated prominently for anyone running untrusted scripts.

## 4. Hard-line re-verification

Director-run greps, verbatim verdicts:

1. **Zero `unsafe` in the kernel crates.** `git grep -nw unsafe` over
   `crates/rum-core` and `crates/rum-runtime` (non-test): **0 matches.** The only
   `unsafe` in the repo is `crates/rum-cli/src/alloc_count.rs:9` — the documented,
   off-by-default alloc-count `GlobalAlloc`. Invariant holds.
2. **No direct `std::fs` / `std::net` in the kernel.** `git grep -nE 'std::fs|std::net'`
   (non-test): the kernel crates (`rum-core`, `rum-runtime`) have ZERO. `std::fs`
   appears only where ambient filesystem authority is deliberately located — the host
   provider implementations `crates/rum-host/src/file_io.rs:1` (FileIo provider) and
   `crates/rum-host/src/entropy.rs:1` (entropy source), plus the `crates/rum-repl`
   application layer. `std::net` appears NOWHERE in the workspace. Invariant holds
   (it is a kernel invariant; the host layer owning fs is the FS-purity design, not a
   violation).
3. **`Pure` = deterministic.** Section 1 classifies all 617 `Pure` builtins; the only
   non-`match` rows are `BeginPackage`/`DeclarePackage` (transitive `FileRead`, a label
   question — not nondeterminism). No `Pure` builtin reads wall-clock, host entropy,
   environment, or process-global mutable state: the fixed-seed session PRNG is `Random`,
   wall-clock readers are `Clock`, and instrumentation disclosure is `Diagnostic`.
   Invariant holds.

## Audit verdict (director, 2026-07-21)

- **No sandbox bypass.** Every capability-bearing effect is downstream of the declared-
  capability checkpoint (`eval_dispatch/mod.rs:135-138`); the loader paths are
  additionally guarded at the effect seam. The three hard lines hold.
- **Finding A (low, documentation): `BeginPackage`/`DeclarePackage` labeled `Pure` but
  transitively `FileRead`** via the dependency form. Verified sound (the transitive read
  is gated by `eval_needs`' own `FileRead` check). **User ruling 2026-07-21: keep `Pure`,
  add a transitive-`FileRead` note to the generated reference** (relabeling would
  over-restrict the pure no-dependency form). → small reference-note fix.
- **Finding B (medium, hardening): `Power`/big-integer producers unbounded by
  `integer_work_limit`.** → candidate fix brief (extend the guard beyond `Factorial`);
  and SECURITY.md must state the opt-in memory-ceiling + timeout requirement for
  untrusted use.
- Everything else: consistent.

**Resolution status (2026-07-21):** Finding A resolved — a transitive-`FileRead`
`Limits` note was added to `BeginPackage`/`DeclarePackage` in `docgen.rs` and the
reference regenerated (drift test green). Finding B resolved — brief-290 (`c76889e`)
bounds `Power` by `integer_work_limit`.

## 5. Dependency review (director, 2026-07-21)

`cargo audit` / `cargo deny` are NOT installed in this environment and need the
network (advisory DB) to run, so the authoritative RUSTSEC/license/bans scan is
recorded as a **pre-publication step for the maintainer to run locally**:

```
cargo install cargo-audit cargo-deny
cargo audit                 # RUSTSEC advisories against Cargo.lock
cargo deny check            # advisories + licenses + bans (multiple-versions) + sources
```

Manual `cargo tree` review of the full graph (done): every external crate is a
mainstream, actively-maintained dependency. The security-sensitive KERNEL deps are
just the `num` ecosystem (`num-bigint`/`num-integer`/`num-traits`, exact arithmetic)
and, behind the default `hash` feature, RustCrypto digests (`sha1`/`sha2`/`md-5`/
`hmac`/`digest`/`crypto-common`/`subtle`), all well-vetted. The heavy async/transport
stack (`tokio`, `zeromq`, `futures*`, `mio`, `parking_lot`, …) is pulled ONLY by
`rum-jupyter`; the kernel + `rum-host` embedding path does not depend on it. No
obscure, abandoned, or unexpected crate is present. Two non-security notes for the
`cargo deny` run: `thiserror` v1 AND v2, and `getrandom` v0.3 AND v0.4, are present as
transitive version skews (duplication, not vulnerabilities).
