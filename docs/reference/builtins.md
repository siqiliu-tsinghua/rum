# Builtin Reference

Generated from the installed `rum` builtin registry. Do not edit by hand.

Builtin count: 643

Regenerate with `cargo run -p rum-stdlib --bin generate-builtin-reference`.

### $Aborted

Represents a computation aborted by Abort or an external interrupt.

- Attributes: `Protected`
- Capability: `Pure`

### $AssertFunction

Stores the optional assertion failure callback used by Assert in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### $Canceled

Represents a canceled operation for confirmation-flow compatibility.

- Attributes: `Protected`
- Capability: `Pure`

### $Context

Evaluates to the current default context name as a string.

- Attributes: `Protected`
- Capability: `Pure`

### $ContextPath

Evaluates to the active context search path as a list of strings.

- Attributes: `None`
- Capability: `Pure`

### $Failed

Represents a failed operation.

- Attributes: `Protected`
- Capability: `Pure`

### $InputFileName

Evaluates to the path of the file currently being loaded, or the empty string at top level.

- Attributes: `Protected`
- Capability: `Pure`

### $MessageList

Lists the message names emitted by the most recent top-level evaluation.

- Attributes: `Protected`
- Capability: `Pure`

### $Packages

Lists package contexts loaded into the current session.

- Attributes: `None`
- Capability: `Pure`

### $Path

Lists search roots used by the package loader.

- Attributes: `None`
- Capability: `Pure`

### Abort

Generates an abort that terminates the current evaluation unless caught.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Abort::argx — expected exactly `1` argument(s), got `2`

### AbortProtect

Evaluates an expression while delaying any abort until the protected region exits in the current first-pass subset.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - AbortProtect::argx — expected exactly `1` argument(s), got `2`

### Abs

Returns the absolute value of exact integers and rationals.

- Attributes: `Listable`, `NumericFunction`, `Protected`
- Capability: `Pure`
- Messages:
  - Abs::argx — expected exactly `1` argument(s), got `2`

### AbsoluteTime

Returns the current exact rational Unix timestamp (seconds since 1970-01-01T00:00:00Z); requires Capability::Clock.

- Attributes: `Protected`
- Capability: `Clock`
- Limits: the timestamp contract is exact rational seconds since the Unix epoch 1970-01-01T00:00:00Z.
- Messages:
  - AbsoluteTime::argx — expected exactly `1` argument(s), got `2`

### AbsoluteTiming

Evaluates an expression and returns {elapsed, value}, with elapsed exact rational seconds measured by a monotonic clock.

- Attributes: `HoldAll`, `Protected`
- Capability: `Clock`
- Messages:
  - AbsoluteTiming::argx — expected exactly `1` argument(s), got `2`

### Accumulate

Returns running totals in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Accumulate::argx — expected exactly `1` argument(s), got `2`

### AddTo

Adds an evaluated increment to a held symbol or exact Part target and stores the result.

- Attributes: `HoldFirst`, `SubValuesHoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - AddTo::argform — expected an exact Part target as the first argument
  - AddTo::argx — expected exactly `1` argument(s), got `2`
  - AddTo::rvalue — `1` is not a variable with a value, so its value cannot be changed.
  - AddTo::wrsym — Symbol `1` is Protected.

### AlgebraicIntegerQ

Tests whether an expression is an exact integer in the current algebraic-integer subset.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`
- Messages:
  - AlgebraicIntegerQ::argx — expected exactly `1` argument(s), got `2`

### AlignColumns

Formats a list of string rows as left-aligned columns separated by "Separator" -> "  " by default; any string separator can be supplied.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - AlignColumns::argtu — expected between `1` and `2` argument(s), got `3`

### AllTrue

Tests whether all selected structural parts satisfy a criterion in the current boolean subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - AllTrue::argform — expected a standard levelspec as the third argument
  - AllTrue::argtu — expected between `1` and `2` argument(s), got `3`
  - AllTrue::argx — `1` called with `2` arguments; 1 argument is expected.

### Alphabet

Returns the current English lowercase alphabet subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Alphabet::argtu — expected between `1` and `2` argument(s), got `3`

### AlphabeticOrder

Compares two strings alphabetically in the current string-order subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - AlphabeticOrder::nstr — argument `1` is not a string

### AlphabeticSort

Sorts strings using the current English alphabet subset with simple diacritic folding.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - AlphabeticSort::argtu — expected between `1` and `2` argument(s), got `3`

### Alphanumeric

Evaluates to the uppercase, lowercase, and digit character-class string.

- Attributes: `Protected`
- Capability: `Pure`

### Alternatives

Represents the current pattern alternative head.

- Attributes: `Protected`
- Capability: `Pure`

### And

Evaluates a conjunction with current short-circuit boolean semantics.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`

### AnyTrue

Tests whether any selected structural part satisfies a criterion in the current boolean subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - AnyTrue::argform — expected a standard levelspec as the third argument
  - AnyTrue::argtu — expected between `1` and `2` argument(s), got `3`
  - AnyTrue::argx — `1` called with `2` arguments; 1 argument is expected.

### Append

Appends an element to the end of a normal expression, with current operator-form support.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Append::argtu — expected between `1` and `2` argument(s), got `3`
  - Append::normal — Nonatomic expression expected at position 1 in `1`.

### AppendTo

Appends an evaluated item to a held symbol or exact Part target and stores the result.

- Attributes: `HoldFirst`, `SubValuesHoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - AppendTo::argform — expected an exact Part target as the first argument
  - AppendTo::argx — expected exactly `1` argument(s), got `2`
  - AppendTo::normal — Nonatomic expression expected at position 1 in `1`.
  - AppendTo::rvalue — `1` is not a variable with a value, so its value cannot be changed.
  - AppendTo::wrsym — Symbol `1` is Protected.

### Apply

Replaces the head of an expression in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Apply::argform — expected a standard levelspec or Heads -> True/False as the third argument
  - Apply::argtu — expected between `1` and `2` argument(s), got `3`

### ApplyTo

Applies an evaluated function to a held symbol or exact Part target and stores the result.

- Attributes: `HoldFirst`, `SubValuesHoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - ApplyTo::argform — expected an exact Part target as the first argument
  - ApplyTo::argx — expected exactly `1` argument(s), got `2`
  - ApplyTo::rvalue — `1` is not a variable with a value, so its value cannot be changed.
  - ApplyTo::wrsym — Symbol `1` is Protected.

### ArgumentsOptions

Partitions a held call into positional arguments and trailing option rules.

- Attributes: `HoldFirst`, `Protected`
- Capability: `Pure`
- Messages:
  - ArgumentsOptions::argform — Reports an invalid held call, argument-count specification, or result association.
  - ArgumentsOptions::argtu — expected between `1` and `2` argument(s), got `3`

### Array

Constructs a dense list-like array by applying a function to generated indices in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Array::argform — expected a non-negative integer dimension or a list of non-negative integer dimensions
  - Array::argtu — expected between `1` and `2` argument(s), got `3`

### ArrayFilter

Applies a function to local list-array neighborhoods in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - ArrayFilter::argform — expected a rectangular List array as the second argument
  - ArrayFilter::argtu — expected between `1` and `2` argument(s), got `3`

### ArrayQ

Tests whether an expression is a rectangular List array, optionally matching its depth against a pattern and requiring every leaf to satisfy a test.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - ArrayQ::argtu — expected between `1` and `2` argument(s), got `3`

### ArrayReduce

Reduces rectangular list arrays across selected dimensions in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - ArrayReduce::argform — expected a rectangular List array as the second argument
  - ArrayReduce::argtu — expected between `1` and `2` argument(s), got `3`

### Assert

Represents an assertion site and supports the current session flag plus $AssertFunction hook.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - Assert::asrte — Assertion `1` had no test.
  - Assert::asrtf — Assertion `1` failed.
  - Assert::asrtfe — Assertion `1` in `2` failed.
  - Assert::asrttf — Assertion test `1` evaluated to `2` that is neither True nor False.

### AssociateTo

Adds or replaces association entries, including held symbol mutation in the current first-pass subset.

- Attributes: `HoldFirst`, `SubValuesHoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - AssociateTo::argr — AssociateTo called with 1 argument; 2 arguments are expected.
  - AssociateTo::argrx — AssociateTo called with `1` arguments; 2 arguments are expected.
  - AssociateTo::argx — `1` called with `2` arguments; 1 argument is expected.
  - AssociateTo::blnoval — The symbol `1` at position `2` should have an immediate value defined.
  - AssociateTo::invak — The argument `1` is not a valid Association.
  - AssociateTo::invlb — The argument `1` is not a list, Rule or Association.
  - AssociateTo::keynx — Key `1` does not exist in `2`.
  - AssociateTo::partw — Part `1` of `2` does not exist.
  - AssociateTo::rvalue — `1` is not a variable with a value, so its value cannot be changed.
  - AssociateTo::wrsym — Symbol `1` is Protected.

### Association

Builds an association from Rule/RuleDelayed entries in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`

### AssociationKeyDeflatten

Builds nested associations from flat associations whose keys are path lists.

- Attributes: `Protected`
- Capability: `Pure`

### AssociationKeyFlatten

Flattens nested associations into associations whose keys are path lists.

- Attributes: `Protected`
- Capability: `Pure`

### AssociationMap

Builds an association by mapping a function over keys in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - AssociationMap::argt — AssociationMap called with `1` arguments; 1 or 2 arguments are expected.
  - AssociationMap::invrlf — Applying `1` to `2` yields `3`, which is not a valid rule, list of rules or association.
  - AssociationMap::invrp — The argument `1` is not a valid Association or a list.

### AssociationQ

Tests whether an expression is an Association with rule entries.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - AssociationQ::argx — AssociationQ called with `1` arguments; 1 argument is expected.

### AssociationThread

Threads rule or key/value inputs into an association in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - AssociationThread::argt — AssociationThread called with `1` arguments; 1 or 2 arguments are expected.
  - AssociationThread::idim — `1` and `2` must have the same length.
  - AssociationThread::invrk — The argument `1` is not a valid rule.

### AtomQ

Tests whether an expression is atomic rather than a normal expression.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - AtomQ::argx — expected exactly `1` argument(s), got `2`

### Attributes

Lists the currently active attributes for a symbol.

- Attributes: `HoldFirst`, `Protected`
- Capability: `Pure`
- Messages:
  - Attributes::argform — expected a symbol or symbol-name string argument
  - Attributes::argx — expected exactly `1` argument(s), got `2`

### Automatic

Represents the current default-specifier symbol shared by several first-pass string and tooling subsets.

- Attributes: `Protected`
- Capability: `Pure`

### Begin

Switches into a context and prepends it to the context path.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Begin::argform — expected a string context name
  - Begin::argx — expected exactly `1` argument(s), got `2`

### BeginPackage

Starts a package context with a package-oriented context path.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: the no-dependency form performs no I/O and stays Pure; the dependency-list form BeginPackage[ctx, {deps}] loads each dependency via Needs, which transitively requires Capability::FileRead and is denied (the load fails closed) under a policy that does not allow it.
- Messages:
  - BeginPackage::argform — Reports an invalid package context or dependency list.
  - BeginPackage::argx — expected `1` argument(s), got `2`

### BernoulliB

Computes exact Bernoulli numbers for non-negative integer indices.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`
- Messages:
  - BernoulliB::argx — expected exactly `1` argument(s), got `2`

### Between

Tests interval inclusion in the current exact numeric subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Between::argx — expected exactly `1` argument(s), got `2`

### BinarySearch

Binary-searches an ascending List under rum's canonical order without validating sortedness, returning a 1-based position or Missing["NotFound"].

- Attributes: `Protected`
- Capability: `Pure`
- Limits: assumes the input List is sorted ascending under rum's canonical order and does not validate sortedness; incomparable values under that order are treated as not found.
- Messages:
  - BinarySearch::argtu — expected between `1` and `2` argument(s), got `3`

### Binomial

Computes exact binomial coefficients for integer arguments.

- Attributes: `Listable`, `NumericFunction`, `Protected`
- Capability: `Pure`
- Messages:
  - Binomial::argx — expected exactly `1` argument(s), got `2`

### Blank

Represents the current single-element pattern atom; in string patterns it matches one character.

- Attributes: `Protected`
- Capability: `Pure`

### BlankNullSequence

Represents the current zero-or-more sequence pattern atom; in string patterns it matches zero or more characters.

- Attributes: `Protected`
- Capability: `Pure`

### BlankSequence

Represents the current one-or-more sequence pattern atom; in string patterns it matches one or more characters.

- Attributes: `Protected`
- Capability: `Pure`

### Block

Temporarily rebinds symbols dynamically.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - Block::argform — expected a list of symbols or symbol initializers as the first argument
  - Block::argx — expected exactly `1` argument(s), got `2`

### BlockMap

Maps a function over contiguous blocks in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - BlockMap::argform — expected a positive integer block size as the third argument
  - BlockMap::argtu — expected between `1` and `2` argument(s), got `3`

### BlockRandom

Evaluates an expression while restoring the deterministic session PRNG state afterward.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - BlockRandom::argx — expected exactly `1` argument(s), got `2`

### Boole

Converts exact True and False to 1 and 0, leaving non-boolean conditions unevaluated.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`
- Messages:
  - Boole::argx — expected exactly `1` argument(s), got `2`

### BooleanQ

Tests whether an expression evaluates exactly to True or False.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - BooleanQ::argx — expected exactly `1` argument(s), got `2`

### Break

Exits the nearest enclosing loop.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Break::argx — expected exactly `1` argument(s), got `2`
  - Break::nofwd — No enclosing For, While, Until or Do found for Break[].

### ByteArray

Represents raw byte payloads using base64 text in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`

### ByteArrayQ

Tests whether an expression is a ByteArray with a valid base64 payload.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - ByteArrayQ::argx — expected exactly `1` argument(s), got `2`

### ByteArrayToString

Converts a ByteArray to a string when its payload is valid UTF-8, failing closed otherwise.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - ByteArrayToString::argx — expected exactly `1` argument(s), got `2`
  - ByteArrayToString::bytearray — invalid ByteArray payload
  - ByteArrayToString::utf8 — ByteArray payload is not valid UTF-8

### ByteCount

Returns a deterministic structural memory-size proxy for an expression.

- Attributes: `Protected`
- Capability: `Pure`

### Capitalize

Uppercases the first character of a string or each string in a list.

- Attributes: `Listable`, `Protected`, `ReadProtected`
- Capability: `Pure`
- Messages:
  - Capitalize::argx — expected exactly `1` argument(s), got `2`

### Cases

Collects subexpressions matching a pattern or rule.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Cases::argform — Reports an invalid traversal argument or Heads option.
  - Cases::argtu — expected between `1` and `2` argument(s), got `3`
  - Cases::argx — `1` called with `2` arguments; 1 argument is expected.
  - Cases::innf — Non-negative integer or Infinity expected at position `1` in `2`.
  - Cases::level — Level specification `1` is not of the form n, {n} or {m, n}.
  - Cases::nonopt — Options expected (instead of `1`) beyond position `2` in `3`. An option must be a rule or a list of rules.
  - Cases::opttf — Value of option `1` -> `2` should be True or False.
  - Cases::optx — Unknown option `1` in `2`.

### CatalanNumber

Computes exact Catalan numbers for non-negative integer indices.

- Attributes: `Listable`, `NumericFunction`, `Protected`, `ReadProtected`
- Capability: `Pure`
- Messages:
  - CatalanNumber::argx — expected exactly `1` argument(s), got `2`

### Catch

Evaluates an expression and intercepts matching Throw values.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - Catch::argtu — expected between `1` and `2` argument(s), got `3`

### CatchExceptions

Catches typed exceptions and dispatches ordered ancestry-aware handler rules.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - CatchExceptions::argtu — expected between `1` and `2` argument(s), got `3`
  - CatchExceptions::argform — handlers must be a List of rules

### Catenate

Concatenates lists and association values into a list in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Catenate::argx — Catenate called with `1` arguments; 1 argument is expected.
  - Catenate::invrp — The argument `1` is not a valid Association or a list.

### Ceiling

Returns the least integer not less than an exact number.

- Attributes: `Listable`, `NumericFunction`, `Protected`
- Capability: `Pure`

### CharacterCounts

Counts code-point occurrences in a string and returns an association in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - CharacterCounts::argx — expected exactly `1` argument(s), got `2`

### CharacterRange

Builds an inclusive range of single-character strings between two endpoints.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - CharacterRange::argx — expected exactly `1` argument(s), got `2`

### Characters

Splits a string into a list of single-code-point strings.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Characters::argx — expected exactly `1` argument(s), got `2`

### Check

Evaluates an expression and substitutes a fallback if matching messages are emitted.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - Check::argform — expected the optional message selector to be a symbol, a "symbol::tag" string, or a list of those
  - Check::argx — expected 2 or 3 argument(s), got `1`

### CheckAbort

Evaluates an expression and substitutes a fallback if an Abort occurs.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - CheckAbort::argr — expected at least 2 argument(s), got `1`
  - CheckAbort::argrx — expected at least 2 argument(s), got `1`
  - CheckAbort::nonopt — Options expected (instead of `1`) beyond position 2 in `2`. An option must be a rule or a list of rules.
  - CheckAbort::optrs — Option specification `1` in `2` is not a rule for a symbol or string.
  - CheckAbort::opttfa — The value of the option PropagateAborts -> `1` should be True, False or Automatic.
  - CheckAbort::optx — Unknown option `1` in `2`.

### CheckArguments

Checks whether a held call has an allowed positional argument count and valid trailing options.

- Attributes: `HoldFirst`, `Protected`
- Capability: `Pure`
- Messages:
  - CheckArguments::argform — Reports an invalid held call, argument-count specification, or result association.
  - CheckArguments::argtu — expected between `1` and `2` argument(s), got `3`

### Clear

Clears values and rules from one or more symbols.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - Clear::argform — expected one or more symbols, symbol-name strings, or lists of them
  - Clear::wrsym — Symbol `1` is Protected.

### ClearAll

Clears values, rules, messages, and user-defined attributes from one or more symbols.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - ClearAll::argform — expected one or more symbols, symbol-name strings, or lists of them
  - ClearAll::wrsym — Symbol `1` is Protected.

### ClearAttributes

Removes attributes from one or more symbols.

- Attributes: `HoldFirst`, `Protected`
- Capability: `Pure`
- Messages:
  - ClearAttributes::argform — Reports an invalid target or attribute specification.
  - ClearAttributes::argx — expected exactly `1` argument(s), got `2`
  - ClearAttributes::wrsym — Symbol `1` is Protected.

### Coefficient

Extracts coefficients of exact monomial powers from rational-coefficient polynomial expressions.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Coefficient::argtu — expected between `1` and `2` argument(s), got `3`

### CoefficientList

Builds coefficient arrays for rational-coefficient polynomial expressions in specified indeterminates.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - CoefficientList::argx — expected exactly `1` argument(s), got `2`

### Comap

Applies selected parts of a function collection to a common expression in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Comap::argform — expected a standard levelspec or Heads -> True/False as the third argument
  - Comap::argtu — expected between `1` and `2` argument(s), got `3`

### ComapApply

Applies selected parts of a function collection as heads of a common expression in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - ComapApply::argform — expected a Heads -> True/False option as the third argument
  - ComapApply::argtu — expected between `1` and `2` argument(s), got `3`

### Commonest

Returns the most frequent elements of a list in first-occurrence order, including all ties.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Commonest::argx — expected exactly `1` argument(s), got `2`

### Complement

Returns sorted unique elements from the first expression that are absent from later same-head expressions; SameTest keeps the first representative in structural scan order.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Complement::argm — Complement called with 0 arguments; 1 or more arguments are expected.
  - Complement::heads — Heads `1` and `2` at positions `3` and 1 are expected to be the same.
  - Complement::normal — Nonatomic expression expected at position 1 in `1`.

### Composition

Represents a left composition of functions in the current first-pass subset.

- Attributes: `Flat`, `OneIdentity`, `Protected`
- Capability: `Pure`

### CompoundExpression

Evaluates expressions sequentially and returns the last result.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`

### Compress

Encodes expressions into the current project-local compressed string envelope.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Compress::argx — expected exactly `1` argument(s), got `2`

### Confirm

Confirms that an expression does not evaluate to a failure-like result in the current first-pass subset.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - Confirm::argtu — expected between `1` and `2` argument(s), got `3`

### ConfirmAssert

Confirms that a test evaluates to True in the current first-pass subset.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - ConfirmAssert::argtu — expected between `1` and `2` argument(s), got `3`

### ConfirmBy

Confirms that a test function returns True for an expression in the current first-pass subset.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - ConfirmBy::argtu — expected between `1` and `2` argument(s), got `3`

### ConfirmMatch

Confirms that an expression matches a held pattern in the current first-pass subset.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - ConfirmMatch::argtu — expected between `1` and `2` argument(s), got `3`

### ConfirmQuiet

Confirms that an evaluation emits no matching messages in the current first-pass subset.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - ConfirmQuiet::argform — Reports an invalid message-selector specification.
  - ConfirmQuiet::argtu — expected between `1` and `2` argument(s), got `3`

### ConformSchema

Validates a value against a symbolic schema and returns the normalized value with defaults filled.

- Attributes: `HoldRest`, `Protected`
- Capability: `Pure`
- Messages:
  - ConformSchema::schema — Reports a malformed schema or conformance failure.
  - ConformSchema::argx — `1` called with `2` arguments; 1 argument is expected.

### ConstantArray

Constructs a constant plain list for non-negative integer scalar and list dimensions.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - ConstantArray::argx — expected exactly `1` argument(s), got `2`

### Construct

Constructs an expression from a head and zero or more arguments.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Construct::argx — expected exactly `1` argument(s), got `2`

### ContainsAll

Tests whether a List contains every requested element, optionally using SameTest.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - ContainsAll::argx — expected exactly `1` argument(s), got `2`

### ContainsAny

Tests whether a List contains any requested element, optionally using SameTest.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - ContainsAny::argx — expected exactly `1` argument(s), got `2`

### ContainsExactly

Tests List multiset equality, optionally using SameTest.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - ContainsExactly::argx — expected exactly `1` argument(s), got `2`

### ContainsNone

Tests whether a List contains none of the requested elements, optionally using SameTest.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - ContainsNone::argx — expected exactly `1` argument(s), got `2`

### ContainsOnly

Tests whether every element of a List belongs to the allowed-element List, optionally using SameTest.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - ContainsOnly::argx — expected exactly `1` argument(s), got `2`

### Context

Returns the context of the current session or of a symbol.

- Attributes: `HoldFirst`, `Protected`
- Capability: `Pure`
- Messages:
  - Context::argform — expected a symbol or string argument
  - Context::argx — expected 0 or 1 argument(s), got `1`

### Contexts

Lists known contexts in the current session.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Contexts::argform — expected a string context-name pattern
  - Contexts::argx — expected 0 or 1 argument(s), got `1`

### Continue

Skips to the next iteration of the nearest enclosing loop.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Continue::argx — expected exactly `1` argument(s), got `2`
  - Continue::nofwd — No enclosing For, While, Until or Do found for Continue[].

### CoprimeQ

Tests whether exact integers are pairwise relatively prime in the current integer subset.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`

### Count

Counts subexpressions matching a pattern on levels specified by a standard levelspec.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: supports the Wolfram Language-documented operator form Count[patt][expr], equivalent to Count[expr, patt].
- Messages:
  - Count::argform — Reports an invalid traversal argument or Heads option.
  - Count::argtu — expected between `1` and `2` argument(s), got `3`
  - Count::argx — `1` called with `2` arguments; 1 argument is expected.
  - Count::level — Level specification `1` is not of the form n, {n} or {m, n}.
  - Count::nonopt — Options expected (instead of `1`) beyond position `2` in `3`. An option must be a rule or a list of rules.
  - Count::opttf — Value of option `1` -> `2` should be True or False.
  - Count::optx — Unknown option `1` in `2`.

### Counts

Counts repeated elements and returns an association in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Counts::argt — Counts called with `1` arguments; 1 or 2 arguments are expected.
  - Counts::invl — The argument `1` is not a list.
  - Counts::invrp — The argument `1` is not a valid Association or a list.

### CountsBy

Counts transformed elements and returns an association in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - CountsBy::argt — CountsBy called with `1` arguments; 1 or 2 arguments are expected.
  - CountsBy::list — List expected at position `1` in `2`.

### Crc32

Computes the IEEE CRC-32 checksum over a string's UTF-8 bytes or a ByteArray.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: computes the IEEE CRC-32 checksum for integrity and interoperability, not security; it is not collision-resistant.
- Messages:
  - Crc32::argx — expected exactly `1` argument(s), got `2`
  - Crc32::bytearray — invalid ByteArray payload

### CurryApplied

Represents a left-to-right curried operator wrapper in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - CurryApplied::argform — expected a non-negative integer, an index list, or k -> index-list as the specification argument
  - CurryApplied::argtu — expected between `1` and `2` argument(s), got `3`

### D

Differentiates exact arithmetic, rational-function, and abstract-function expressions with held differentiation variables.

- Attributes: `HoldRest`, `Protected`
- Capability: `Pure`
- Messages:
  - D::argx — expected at least 2 argument(s), got `1`
  - D::dvar — `1` is not a valid variable.
  - D::dvord — Derivative order `1` is not a non-negative integer.

### DateDifference

Returns the exact fixed-duration difference between two timestamps or date lists.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: supports only fixed-duration units Seconds, Minutes, Hours, Days, and Weeks; calendar Months and Years are outside the folk date/time subset.
- Messages:
  - DateDifference::argx — expected exactly `1` argument(s), got `2`
  - DateDifference::date — Reports invalid date values or date units.

### DateList

Converts an exact Unix timestamp to a UTC date list.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: uses UTC, the proleptic Gregorian calendar, and no leap seconds.
- Messages:
  - DateList::argx — expected exactly `1` argument(s), got `2`
  - DateList::date — Reports a date-list conversion failure.

### DateObjectQ

Tests whether an expression has a DateObject head in the current structural subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - DateObjectQ::argx — expected exactly `1` argument(s), got `2`

### DatePlus

Adds an exact fixed-duration offset to an exact timestamp or date list.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: supports only fixed-duration units Seconds, Minutes, Hours, Days, and Weeks; calendar Months and Years are outside the folk date/time subset.
- Messages:
  - DatePlus::argx — expected exactly `1` argument(s), got `2`
  - DatePlus::date — Reports an invalid date value or duration.

### DateString

Formats an exact timestamp or date list as an ISO 8601 UTC string.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: renders ISO 8601 UTC only; non-integer seconds are rendered only when their exact rational value has a terminating decimal expansion. The two-argument form DateString[date, fmt] accepts fmt "ISODateTime" (default, full UTC timestamp), "ISODate" (date only), or "ISOTime" (time only).
- Messages:
  - DateString::argtu — expected between `1` and `2` argument(s), got `3`
  - DateString::argx — expected between 1 and 2 argument(s), got 0
  - DateString::date — Reports a date-formatting or date-value failure.

### DayOfWeek

Returns the ISO weekday symbol for an exact timestamp or date list.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: the two-argument form DayOfWeek[date, sel] accepts sel "Name" (default weekday symbol) or "ISOInteger" (Monday=1 through Sunday=7).
- Messages:
  - DayOfWeek::argtu — expected between `1` and `2` argument(s), got `3`
  - DayOfWeek::date — Reports an invalid date value or weekday result.

### DaysInMonth

Returns the number of days in a proleptic Gregorian month.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - DaysInMonth::argx — expected exactly `1` argument(s), got `2`
  - DaysInMonth::date — Reports an invalid year or month value.

### Decapitalize

Lowercases the first character of a string or each string in a list.

- Attributes: `Listable`, `Protected`, `ReadProtected`
- Capability: `Pure`
- Messages:
  - Decapitalize::argx — expected exactly `1` argument(s), got `2`

### DecimalString

Renders an exact number as a plain decimal string with exactly n fractional digits.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: uses exact integer/rational arithmetic only, rounds half away from zero, and always emits exactly n fractional digits.
- Messages:
  - DecimalString::argx — expected exactly `1` argument(s), got `2`

### DeclarePackage

Declares stub names for a package context and prepends that context to the search path.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: registering a stub context performs no I/O and stays Pure; it installs a deferred autoload, so evaluating a declared symbol later loads the package via Needs, which transitively requires Capability::FileRead and is denied (fails closed) under a policy that does not allow it.
- Messages:
  - DeclarePackage::argform — Reports an invalid package context or symbol-name list.
  - DeclarePackage::argx — expected exactly `1` argument(s), got `2`

### Decrement

Decrements a held symbol or exact Part target and returns its previous value.

- Attributes: `HoldFirst`, `Protected`
- Capability: `Pure`
- Messages:
  - Decrement::argform — expected an exact Part target as the first argument
  - Decrement::argx — expected exactly `1` argument(s), got `2`
  - Decrement::rvalue — `1` is not a variable with a value, so its value cannot be changed.
  - Decrement::wrsym — Symbol `1` is Protected.

### DefaultValues

Lists stored Default[...] definitions for a symbol in the current optional-pattern subset.

- Attributes: `HoldFirst`, `Protected`
- Capability: `Pure`
- Messages:
  - DefaultValues::argform — expected a symbol or symbol-name string argument
  - DefaultValues::argx — expected exactly `1` argument(s), got `2`

### DefinePackage

Registers a held source unit with public, package-scope, and identity-sealed private symbols.

- Attributes: `HoldAllComplete`, `Protected`
- Capability: `Pure`
- Messages:
  - DefinePackage::argform — expected a non-empty valid context string with a trailing context separator as the first argument
  - DefinePackage::decl — expected at most one Public and PackageScope declaration containing only current-context symbols
  - DefinePackage::failed — package source-unit registration failed during body evaluation

### Definition

Returns a held first-pass expression containing visible definitions for a symbol.

- Attributes: `HoldFirst`, `Protected`
- Capability: `Pure`
- Messages:
  - Definition::argform — expected a symbol or symbol-name string argument
  - Definition::argx — expected exactly `1` argument(s), got `2`

### Delete

Deletes one or more top-level structural positions in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Delete::argx — expected exactly `1` argument(s), got `2`
  - Delete::partw — Part `1` of `2` does not exist.

### DeleteCases

Deletes subexpressions matching a pattern on levels specified by a standard levelspec.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - DeleteCases::argform — Reports an invalid traversal argument or Heads option.
  - DeleteCases::argtu — expected between `1` and `2` argument(s), got `3`
  - DeleteCases::argx — `1` called with `2` arguments; 1 argument is expected.
  - DeleteCases::innf — Non-negative integer or Infinity expected at position `1` in `2`.
  - DeleteCases::level — Level specification `1` is not of the form n, {n} or {m, n}.
  - DeleteCases::nonopt — Options expected (instead of `1`) beyond position `2` in `3`. An option must be a rule or a list of rules.
  - DeleteCases::opttf — Value of option `1` -> `2` should be True or False.
  - DeleteCases::optx — Unknown option `1` in `2`.

### DeleteDuplicates

Keeps the first occurrence of structurally equal top-level elements, or of elements equivalent under a custom binary test.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: the custom binary-test form performs pairwise O(n^2) representative comparisons and keeps the first representative.
- Messages:
  - DeleteDuplicates::normal — Nonatomic expression expected at position 1 in `1`.

### DeleteDuplicatesBy

Keeps the first top-level element for each evaluated key in the current unary-function subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - DeleteDuplicatesBy::argtu — expected between `1` and `2` argument(s), got `3`
  - DeleteDuplicatesBy::argx — `1` called with `2` arguments; 1 argument is expected.
  - DeleteDuplicatesBy::normal — Nonatomic expression expected at position 1 in `1`.

### DeleteMissing

Deletes Missing entries from lists, associations, and rule-list data in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - DeleteMissing::arg2 — The second argument `1` is expected to be a positive integer or Infinity.
  - DeleteMissing::arg3 — The third argument `1` is expected to be a non-negative integer or Infinity.
  - DeleteMissing::argb — DeleteMissing called with `1` arguments; between 1 and 3 arguments are expected.
  - DeleteMissing::invrp — The argument `1` is not a valid Association or a list.

### Denominator

Returns the denominator of exact integers and rationals.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`
- Messages:
  - Denominator::argx — expected exactly `1` argument(s), got `2`

### Depth

Returns the structural depth of an expression, optionally including heads.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Depth::argform — Reports an invalid level specification or Heads option for Depth.
  - Depth::argtu — expected between `1` and `2` argument(s), got `3`
  - Depth::nonopt — Options expected (instead of `1`) beyond position `2` in `3`. An option must be a rule or a list of rules.
  - Depth::opttf — Value of option `1` -> `2` should be True or False.
  - Depth::optx — Unknown option `1` in `2`.

### Dialog

Runs the current first-pass dialog subset with Return-sensitive evaluation and host-input suspension; requires Capability::Interactive.

- Attributes: `HoldAll`, `Protected`
- Capability: `Interactive`
- Messages:
  - Dialog::argform — Reports invalid Dialog option rules or DialogSymbols bindings.

### Diff

Builds a simple DiffObject for strings or lists in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Diff::argx — expected exactly `1` argument(s), got `2`

### DiffObject

Represents the current diff result object returned by Diff.

- Attributes: `Protected`
- Capability: `Pure`

### Differences

Returns successive differences for plain lists, including the current non-negative order subset.

- Attributes: `NonThreadable`, `Protected`
- Capability: `Pure`
- Messages:
  - Differences::argtu — expected between `1` and `2` argument(s), got `3`
  - Differences::ilsmn — Single or list of non-negative machine-sized integers expected at position 2 of `1`.

### DigitCharacter

Represents the current string-pattern atom for one ASCII decimal digit.

- Attributes: `Protected`
- Capability: `Pure`

### DigitCharacters

Evaluates to the ASCII digit character-class string.

- Attributes: `Protected`
- Capability: `Pure`

### DigitCount

Counts digits in the absolute value of an exact integer.

- Attributes: `Protected`, `ReadProtected`
- Capability: `Pure`
- Messages:
  - DigitCount::argtu — expected between `1` and `2` argument(s), got `3`
  - DigitCount::ibase — Base `1` is not an integer greater than 1 in `2`.

### DigitQ

Tests whether all characters in a string belong to the current digit subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - DigitQ::argx — expected exactly `1` argument(s), got `2`

### DirectoryName

Returns a parent directory path string for a file name.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - DirectoryName::argform — expected a string file name
  - DirectoryName::argtu — expected between `1` and `2` argument(s), got `3`

### DisjointQ

Tests whether two Lists share no element, optionally using SameTest.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - DisjointQ::argx — expected exactly `1` argument(s), got `2`

### Divide

Divides exact numbers, producing exact rationals when possible.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Divide::argx — expected exactly `1` argument(s), got `2`

### DivideBy

Divides a held symbol or exact Part target by an evaluated divisor and stores the result.

- Attributes: `HoldFirst`, `SubValuesHoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - DivideBy::argform — expected an exact Part target as the first argument
  - DivideBy::argx — expected exactly `1` argument(s), got `2`
  - DivideBy::rvalue — `1` is not a variable with a value, so its value cannot be changed.
  - DivideBy::wrsym — Symbol `1` is Protected.

### DivisorSigma

Computes exact divisor power sums for exact integer arguments.

- Attributes: `Listable`, `NHoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - DivisorSigma::argx — expected exactly `1` argument(s), got `2`

### Divisors

Returns the sorted positive divisors of an exact integer.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`
- Messages:
  - Divisors::argx — expected exactly `1` argument(s), got `2`

### Do

Evaluates an expression repeatedly in the current first-pass iterator subset.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - Do::argform — expected a supported repetition count or iterator specification
  - Do::argx — expected at least 2 argument(s), got `1`

### Dot

Contracts list tensors over the last axis of each left operand and the first axis of each right operand.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Dot::dot — Tensor contraction dimensions `1` and `2` do not match for `3` and `4`.

### DownValues

Lists stored downvalue definitions for a symbol.

- Attributes: `HoldFirst`, `Protected`
- Capability: `Pure`
- Messages:
  - DownValues::argform — expected a symbol or symbol-name string argument
  - DownValues::argx — expected exactly `1` argument(s), got `2`
  - DownValues::setval — Reports an invalid DownValues assignment.

### Drop

Drops structural elements using the current standard sequence-spec subset, including UpTo bounds.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Drop::argx — expected exactly `1` argument(s), got `2`
  - Drop::drop — Cannot apply the requested sequence specification in `1`.

### DumpSave

Writes a textual first-pass dump of selected symbols to a file and returns the saved symbol list.

- Attributes: `HoldRest`, `Protected`
- Capability: `FileWrite`
- Limits: writes a textual dump format; it is not a binary image.
- Messages:
  - DumpSave::argform — Reports an invalid path, object specification, context filter, or option list.
  - DumpSave::argtu — expected between `1` and `2` argument(s), got `3`
  - DumpSave::io — could not write `1`: `2`
  - DumpSave::optval — Unsupported value `1` for option CompressionLevel in DumpSave; only 1 is currently supported.
  - DumpSave::optx — Unknown option `1` in `2`.
  - DumpSave::sealedref — Cannot save a definition containing a sealed symbol reference.

### DuplicateFreeQ

Tests whether a List has no repeated elements, optionally using SameTest.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - DuplicateFreeQ::argx — expected exactly `1` argument(s), got `2`

### Duplicates

Returns structurally equal top-level List elements that occur two or more times, once each in first-appearance order.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Duplicates::argtu — expected between `1` and `2` argument(s), got `3`

### Echo

Prints a value through the runtime print stream and returns it.

- Attributes: `Protected`
- Capability: `Pure`

### EchoFunction

Returns a pure function that echoes its argument through the runtime print stream.

- Attributes: `Protected`
- Capability: `Pure`

### EchoLabel

Returns a pure function that echoes its argument with a label through the runtime print stream.

- Attributes: `Protected`
- Capability: `Pure`

### EditDistance

Computes unit-cost Levenshtein distance for strings or lists in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - EditDistance::argx — expected exactly `1` argument(s), got `2`

### Enclose

Evaluates an expression and catches confirmation failures in the current first-pass subset.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - Enclose::argtu — expected between `1` and `2` argument(s), got `3`

### End

Restores the previous context after a matching Begin.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - End::argx — expected exactly `1` argument(s), got `2`
  - End::context — no matching Begin[] frame is active

### EndOfLine

Represents the current string-pattern anchor for the end of a string or a position before LF.

- Attributes: `Protected`
- Capability: `Pure`

### EndOfString

Represents the current string-pattern anchor for the end of a string.

- Attributes: `Protected`
- Capability: `Pure`

### EndPackage

Ends a package context and leaves it on the restored context path.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - EndPackage::argx — expected exactly `1` argument(s), got `2`
  - EndPackage::context — no matching BeginPackage[] frame is active

### Equal

Tests equality in the current exact subset for numbers, strings, and obviously identical expressions.

- Attributes: `Protected`
- Capability: `Pure`

### EqualTo

Represents a unary comparison operator equivalent to Equal[#, bound] when applied.

- Attributes: `Protected`
- Capability: `Pure`

### Equivalent

Evaluates logical equivalence over exact boolean arguments and reduces duplicate symbolic terms in the current subset.

- Attributes: `Orderless`, `Protected`
- Capability: `Pure`

### EulerPhi

Computes Euler's totient function for an exact integer.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`
- Messages:
  - EulerPhi::argx — expected exactly `1` argument(s), got `2`

### Evaluate

Forces evaluation in held contexts.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`

### EvaluationData

Evaluates an expression and returns a first-pass association of result and diagnostics.

- Attributes: `HoldFirst`, `Protected`
- Capability: `Clock`
- Messages:
  - EvaluationData::argform — expected the optional property argument to be All, Properties, a property string or symbol, or a list of property strings or symbols
  - EvaluationData::argtu — expected between `1` and `2` argument(s), got `3`

### EvenQ

Tests whether an expression is currently an even exact integer.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`
- Messages:
  - EvenQ::argx — expected exactly `1` argument(s), got `2`

### Except

Represents the current pattern exclusion head.

- Attributes: `Protected`
- Capability: `Pure`

### Exception

Represents a typed exception with an Association payload.

- Attributes: `Protected`
- Capability: `Pure`

### ExceptionTypes

Returns registered exception types as sorted type-to-parent rules.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - ExceptionTypes::argx — expected exactly `1` argument(s), got `2`

### Exit

Requests host-level termination with an optional integer exit code.

- Attributes: `Protected`
- Capability: `Process`
- Messages:
  - Exit::argt — Exit called with `1` arguments; 0 or 1 arguments are expected.
  - Exit::intm — Machine-sized integer expected at position 1 in `1`.

### Expand

Expands rational-coefficient polynomial expressions over pattern-free indeterminates.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Expand::argx — expected exactly `1` argument(s), got `2`

### Extract

Extracts parts at structural positions, optionally wrapping each result before evaluation.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Extract::argt — Extract called with `1` arguments; 2 or 3 arguments are expected.
  - Extract::keyw — Key `1` does not exist in `2`.
  - Extract::partd — Part specification `1` is longer than depth of object.
  - Extract::partw — Part `1` of `2` does not exist.
  - Extract::psl1 — Position specification `1` in `2` is not applicable.
  - Extract::pspec1 — Part specification `1` is not applicable.
  - Extract::span — Reports an invalid Span part specification.
  - Extract::take — Reports positions that cannot be taken from an expression.

### Factor

Factors rational-coefficient polynomials over the rationals using the current polynomial kernel.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: handles rational-coefficient polynomial expressions; current factorization extracts monomial factors and recursively splits discovered linear factors.
- Messages:
  - Factor::argx — expected exactly `1` argument(s), got `2`

### Factorial

Computes factorials for non-negative exact integers.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`
- Messages:
  - Factorial::argx — expected exactly `1` argument(s), got `2`

### Factorial2

Computes double factorials for non-negative exact integers.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`
- Messages:
  - Factorial2::argx — expected exactly `1` argument(s), got `2`

### Failsafe

Creates a callable wrapper that preserves failure-like inputs in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`

### Failure

Represents a failure object with associated details in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`

### FailureQ

Tests whether an expression is a Failure object.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - FailureQ::argx — expected exactly `1` argument(s), got `2`

### Fibonacci

Computes exact Fibonacci numbers for integer indices.

- Attributes: `Listable`, `NumericFunction`, `Protected`, `ReadProtected`
- Capability: `Pure`
- Messages:
  - Fibonacci::argx — expected exactly `1` argument(s), got `2`

### File

Represents an explicit file specification.

- Attributes: `Protected`
- Capability: `Pure`

### FileBaseName

Returns the final path component without its extension.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - FileBaseName::argx — expected exactly `1` argument(s), got `2`

### FileExtension

Returns the final path component extension without the dot, or an empty string when no extension is present.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: dotfiles such as .bashrc have no extension; only the final slash-separated path component is inspected.
- Messages:
  - FileExtension::argx — expected exactly `1` argument(s), got `2`

### FileNameJoin

Joins path components into a platform path string.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - FileNameJoin::argform — expected a list of string path components
  - FileNameJoin::argx — expected exactly `1` argument(s), got `2`

### FileNameSplit

Splits a slash-separated path string into components, preserving an absolute leading slash as its own component.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: treats / as the documented separator, emits an absolute leading / as its own component, and skips repeated or trailing separators.
- Messages:
  - FileNameSplit::argx — expected exactly `1` argument(s), got `2`

### FileNameTake

Takes file name components from a path string.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - FileNameTake::argform — expected a string file name
  - FileNameTake::argtu — expected between `1` and `2` argument(s), got `3`

### FileNames

Lists sorted file paths whose names match a folk wildcard pattern.

- Attributes: `Protected`
- Capability: `FileRead`
- Limits: supports ASCII path strings in this folk subset and uses only provider-mediated directory enumeration; recursive walks do not detect symlink loops beyond the requested depth and the fixed 100000-entry traversal cap.
- Messages:
  - FileNames::argtu — expected between `1` and `2` argument(s), got `3`

### FindFile

Finds the full path of a file or package using the current search path.

- Attributes: `Protected`
- Capability: `FileRead`
- Messages:
  - FindFile::argform — expected a string or File["..."] load target
  - FindFile::argx — expected exactly `1` argument(s), got `2`
  - FindFile::notfound — Reports that the requested file could not be located.

### First

Returns the first structural element, with an optional held default.

- Attributes: `HoldRest`, `Protected`
- Capability: `Pure`
- Messages:
  - First::argtu — expected between `1` and `2` argument(s), got `3`
  - First::nofirst — `1` has zero length and no `2` element.
  - First::normal — Nonatomic expression expected at position 1 in `1`.

### FixedPoint

Repeatedly applies a function until the result stops changing in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - FixedPoint::argform — expected a non-negative integer iteration limit
  - FixedPoint::argtu — expected between `1` and `2` argument(s), got `3`

### FixedPointList

Returns successive repeated applications until a fixed point in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - FixedPointList::argform — expected a non-negative integer iteration limit
  - FixedPointList::argtu — expected between `1` and `2` argument(s), got `3`

### Flatten

Flattens nested expressions through the current non-negative level, single-head-spec, and level-list subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Flatten::argtu — expected between `1` and `2` argument(s), got `3`
  - Flatten::fldep — Level `1` exceeds the levels, `2`, which can be flattened together in `3`.
  - Flatten::flev — The level argument `1` in position 2 of `2` should be a non-negative integer or Infinity.
  - Flatten::flpi — Levels to be flattened together in `1` should be lists of positive integers.
  - Flatten::flrep — Level `1` should not be repeated.
  - Flatten::normal — Nonatomic expression expected at position 1 in `1`.

### Floor

Returns the greatest integer not greater than an exact number.

- Attributes: `Listable`, `NumericFunction`, `Protected`
- Capability: `Pure`

### Fold

Accumulates over elements in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Fold::argtu — expected between `1` and `2` argument(s), got `3`

### FoldList

Returns successive accumulator states in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - FoldList::argtu — expected between `1` and `2` argument(s), got `3`

### FoldPair

Accumulates pair-valued state transitions in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - FoldPair::argtu — expected between `1` and `2` argument(s), got `3`

### FoldWhile

Accumulates while a test stays True in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - FoldWhile::argform — expected a positive integer or All test arity, with an optional signed integer tail adjustment
  - FoldWhile::argtu — expected between `1` and `2` argument(s), got `3`

### For

Evaluates start, test, increment, and body in the current first-pass procedural subset.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - For::argtu — expected between `1` and `2` argument(s), got `3`

### FractionalPart

Returns the exact fractional part after truncating toward zero.

- Attributes: `Listable`, `NumericFunction`, `Protected`
- Capability: `Pure`

### FreeQ

Tests whether no subexpression matches a pattern on levels specified by a standard levelspec.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: supports the Wolfram Language-documented operator form FreeQ[patt][expr], equivalent to FreeQ[expr, patt].
- Messages:
  - FreeQ::argform — Reports an invalid traversal argument or Heads option.
  - FreeQ::argtu — expected between `1` and `2` argument(s), got `3`
  - FreeQ::argx — `1` called with `2` arguments; 1 argument is expected.
  - FreeQ::level — Level specification `1` is not of the form n, {n} or {m, n}.
  - FreeQ::nonopt — Options expected (instead of `1`) beyond position `2` in `3`. An option must be a rule or a list of rules.
  - FreeQ::opttf — Value of option `1` -> `2` should be True or False.
  - FreeQ::optx — Unknown option `1` in `2`.

### FromBase64

Decodes standard RFC 4648 base64 with padding into a ByteArray, failing closed on malformed input.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - FromBase64::argx — expected exactly `1` argument(s), got `2`
  - FromBase64::base64 — expected standard RFC 4648 base64 with valid padding

### FromCsv

Parses CSV text into Lists of Strings, optionally using the first row as Association headers.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: parses RFC 4180-style text into String fields only; plain rows may be ragged, while Headers -> True rejects duplicate headers and ragged data rows. Delimiter must be one non-quote, non-newline character.
- Messages:
  - FromCsv::csv — Reports a CSV decoding or argument failure.

### FromDateList

Converts a UTC date list {y, mo, d, h, mi, s} to an exact Unix timestamp; the inverse of DateList.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: uses UTC, the proleptic Gregorian calendar, and no leap seconds; validates the date list and is the inverse of DateList.
- Messages:
  - FromDateList::argx — expected exactly `1` argument(s), got `2`
  - FromDateList::date — Reports an invalid date list.

### FromDateString

Parses the documented ISO 8601 UTC/fixed-offset subset into an exact Unix timestamp.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: accepts only the documented ISO 8601 subset with optional Z or fixed +/-HH:MM offsets; there is no time-zone database and no leap-second support.
- Messages:
  - FromDateString::argx — expected exactly `1` argument(s), got `2`
  - FromDateString::date — Reports an invalid date string.

### FromDigits

Constructs an exact integer from a list or string of digits.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - FromDigits::argtu — expected between `1` and `2` argument(s), got `3`

### FromHexString

Decodes even-length hexadecimal text into a ByteArray, failing closed on malformed input.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - FromHexString::argx — expected exactly `1` argument(s), got `2`
  - FromHexString::hex — expected an even-length hexadecimal string

### FromIni

Parses rum's fail-closed INI dialect into an Association of section Associations.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: implements rum's documented INI dialect only: whole-line ;/# comments, [section] headers, key = value lines, trimmed keys/values, verbatim values, no nesting, no multiline values, and no inline comments.
- Messages:
  - FromIni::ini — Reports an INI decoding or argument failure.

### FromJson

Parses an RFC 8259 JSON text string into exact rum values, failing closed on invalid JSON.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - FromJson::depth — JSON nesting exceeds the fixed depth limit of 256 at byte offset `1`.
  - FromJson::json — Reports an invalid JSON input or parse failure.

### FromToml

Parses rum's documented fail-closed TOML subset into exact rum values.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: implements rum's documented TOML 1.0 folk subset only: tables, arrays-of-tables, inline tables, strings, exact integers/decimals, booleans, and date/time tokens; inf/nan and unsupported TOML surface fail closed. Offset date-times decode as exact Unix-second instants, while local dates/times decode as Lists.
- Messages:
  - FromToml::depth — TOML nesting exceeds the fixed depth limit of 256 at byte offset `1`.
  - FromToml::toml — Reports a TOML decoding or argument failure.

### FromXml

Parses XML text into a symbolic XmlElement tree, rejecting DTDs and unsupported XML surface.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: parses a value-oriented XML element tree only: declaration syntax is validated but not represented, comments are dropped, CDATA becomes text, namespaces remain verbatim, whitespace-only runs around element content are dropped, and DTD/DOCTYPE declarations are deliberately rejected.
- Messages:
  - FromXml::depth — XML nesting exceeds the fixed depth limit of 256 at byte offset `1`.
  - FromXml::xml — Reports an XML decoding or argument failure.

### FromYaml

Parses rum's documented fail-closed YAML block subset into exact rum values.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: implements rum's documented YAML sane block subset only: block mappings and sequences, plain and quoted scalars, literal | and |- blocks, comments, and empty flow {} / []; anchors, aliases, tags, directives, document markers, merge keys, folded blocks, non-empty flow collections, duplicate keys, tabs in indentation, and ambiguous same-indent collection mixes fail closed.
- Messages:
  - FromYaml::depth — YAML nesting exceeds the fixed depth limit of 256 at byte offset `1`.
  - FromYaml::yaml — Reports a YAML decoding or argument failure.

### FullForm

Renders the held argument as a canonical structural-form string in the current core expression subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - FullForm::argx — expected exactly `1` argument(s), got `2`

### Function

Creates a pure function.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - Function::argx — expected at least `1` argument(s), got `2`

### GCD

Computes the greatest common divisor of exact integers.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`

### GatherBy

Groups top-level elements by evaluated keys in first-occurrence key order, including hierarchical grouping for a list of functions.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - GatherBy::argx — expected exactly `1` argument(s), got `2`
  - GatherBy::normal — Nonatomic expression expected at position 1 in `1`.

### General

Carries general runtime message templates.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - General::cancel — Reports evaluation cancellation requested by the host.
  - General::iterlim — Reports that ReplaceRepeated exceeded its iteration limit.
  - General::matchreclim — Reports that evaluation exceeded the pattern-match recursion limit.
  - General::memlim — Reports that evaluation exceeded a MemoryConstrained limit.
  - General::noentropy — no entropy provider installed
  - General::noio — no file provider installed
  - General::normal — Nonatomic expression expected at position 1 in `1`.
  - General::passlim — Reports an evaluation cycle or fixed-point pass-limit termination.
  - General::optreentry — The option reentry type is not registered.
  - General::reclim — Reports that evaluation exceeded the recursion limit.
  - General::reslim — Reports a native-stack or configured resource work-limit termination.
  - General::sandbox — `1` is not permitted in this sandbox.
  - General::timelim — Reports that evaluation exceeded a TimeConstrained limit.

### Get

Reads a file or package and evaluates the expressions it contains.

- Attributes: `Protected`
- Capability: `FileRead`
- Messages:
  - Get::argform — expected a string or File["..."] load target
  - Get::argx — expected exactly `1` argument(s), got `2`
  - Get::io — could not read `1`: `2`
  - Get::load — Reports a loader failure while reading or evaluating a file.
  - Get::notfound — Reports that the requested file could not be located.
  - Get::syntax — Reports a syntax error while evaluating a loaded file.

### Greater

Tests strict decreasing order in the current exact comparable subset.

- Attributes: `Protected`
- Capability: `Pure`

### GreaterEqual

Tests nonincreasing order in the current exact comparable subset.

- Attributes: `Protected`
- Capability: `Pure`

### GreaterEqualThan

Represents a unary comparison operator equivalent to GreaterEqual[#, bound] when applied.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - GreaterEqualThan::argx — `1` called with `2` arguments; 1 argument is expected.

### GreaterThan

Represents a unary comparison operator equivalent to Greater[#, bound] when applied.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - GreaterThan::argx — `1` called with `2` arguments; 1 argument is expected.

### GroupBy

Groups list elements by transformed keys, including the current reducer form.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - GroupBy::argb — GroupBy called with `1` arguments; between 1 and 3 arguments are expected.
  - GroupBy::list1 — The argument `1` is not a valid list of Associations or rules or lists of rules.

### HammingDistance

Computes mismatch counts for equal-length strings or lists in the current first-pass subset; unequal lengths emit a message and leave the call inert.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - HammingDistance::argx — expected exactly `1` argument(s), got `2`
  - HammingDistance::len — HammingDistance expects inputs with the same length in `1`.

### Hash

Computes SHA256, SHA1, SHA512, MD5, or CRC32 hashes over the current string/expression subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Hash::argtu — expected between `1` and `2` argument(s), got `3`

### Head

Returns the structural head of an expression, optionally wrapped in a second head.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Head::argtu — expected between `1` and `2` argument(s), got `3`

### Hold

Prevents evaluation of its arguments.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`

### HoldAll

Prevents evaluation of all arguments.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`

### HoldAllComplete

Prevents evaluation of all arguments and disables Evaluate and Sequence escapes.

- Attributes: `HoldAllComplete`, `Protected`
- Capability: `Pure`

### HoldComplete

Prevents evaluation of all arguments and disables Evaluate and Sequence escapes.

- Attributes: `HoldAllComplete`, `Protected`
- Capability: `Pure`

### HoldFirst

Prevents evaluation of the first argument.

- Attributes: `HoldFirst`, `Protected`
- Capability: `Pure`

### HoldForm

Holds an expression while rendering transparently in terminal results and output-like forms.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`

### HoldPattern

Marks a pattern expression so it is matched without prior evaluation.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`

### HoldRest

Prevents evaluation of all but the first argument.

- Attributes: `HoldRest`, `Protected`
- Capability: `Pure`

### HtmlEscape

Escapes the five HTML text characters &, <, >, double quote, and single quote.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - HtmlEscape::argx — expected exactly `1` argument(s), got `2`

### HtmlUnescape

Unescapes the five named HTML entities and decimal or hexadecimal numeric character references, leaving unknown and malformed entities verbatim by default; with "Strict" -> True, malformed entities fail closed with $Failed.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - HtmlUnescape::argx — expected exactly `1` argument(s), got `2`
  - HtmlUnescape::entity — Reports an invalid HTML entity.

### Identity

Returns its argument unchanged.

- Attributes: `Protected`
- Capability: `Pure`

### If

Evaluates one of several branches conditionally.

- Attributes: `HoldRest`, `Protected`
- Capability: `Pure`
- Messages:
  - If::argtu — expected between `1` and `2` argument(s), got `3`

### IgnoreCase

Represents the current option symbol for case-insensitive string matching.

- Attributes: `Protected`
- Capability: `Pure`

### Implies

Evaluates binary logical implication over exact boolean arguments in the current subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Implies::argx — expected exactly `1` argument(s), got `2`

### Increment

Increments a held symbol or exact Part target and returns its previous value.

- Attributes: `HoldFirst`, `Protected`
- Capability: `Pure`
- Messages:
  - Increment::argform — expected an exact Part target as the first argument
  - Increment::argx — expected exactly `1` argument(s), got `2`
  - Increment::rvalue — `1` is not a variable with a value, so its value cannot be changed.
  - Increment::wrsym — Symbol `1` is Protected.

### Inequality

Evaluates alternating operand/operator comparison chains without short-circuiting; each operand evaluates once, binary chains degenerate to their comparison head, and undetermined multi-segment chains remain symbolic.

- Attributes: `Protected`
- Capability: `Pure`

### Information

Returns a first-pass InformationData summary for a symbol or symbol-name string.

- Attributes: `HoldFirst`, `Protected`
- Capability: `Pure`
- Messages:
  - Information::argtu — expected between `1` and `2` argument(s), got `3`

### Inner

Combines matching tensor dimensions in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Inner::argtu — expected between `1` and `2` argument(s), got `3`
  - Inner::dim — Tensor contraction dimensions `1` and `2` do not match for `3` and `4`.

### Input

Requests one line of host-provided text, parses it as rum source, and evaluates the resulting expression in the current first-pass subset; requires Capability::Interactive.

- Attributes: `Protected`
- Capability: `Interactive`
- Messages:
  - Input::nonopt — Options expected (instead of `1`) beyond position 1 in `2`. An option must be a rule or a list of rules.
  - Input::optrs — Option specification `1` in `2` is not a rule for a symbol or string.
  - Input::optx — Unknown option `1` in `2`.

### InputForm

Represents the current formatting symbol accepted by the current ToExpression and ToString subsets.

- Attributes: `Protected`
- Capability: `Pure`

### InputString

Requests one line of host-provided text and returns it as a string; requires Capability::Interactive.

- Attributes: `Protected`
- Capability: `Interactive`
- Messages:
  - InputString::nonopt — Options expected (instead of `1`) beyond position 1 in `2`. An option must be a rule or a list of rules.
  - InputString::optrs — Option specification `1` in `2` is not a rule for a symbol or string.
  - InputString::optx — Unknown option `1` in `2`.

### Insert

Inserts an element at a top-level structural position in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Insert::argx — expected exactly `1` argument(s), got `2`
  - Insert::ins — Cannot insert at position `1` in `2`

### IntegerDigits

Returns the digits of an exact integer in a given base.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`
- Messages:
  - IntegerDigits::argtu — expected between `1` and `2` argument(s), got `3`
  - IntegerDigits::ibase — Base `1` is not an integer greater than 1 in `2`.
  - IntegerDigits::intnn — Length `1` is not a non-negative integer in `2`.

### IntegerLength

Returns the number of digits in an exact integer.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`
- Messages:
  - IntegerLength::argtu — expected between `1` and `2` argument(s), got `3`
  - IntegerLength::ibase — Base `1` is not an integer greater than 1 in `2`.

### IntegerPart

Returns the integer part of an exact number by truncating toward zero.

- Attributes: `Listable`, `NumericFunction`, `Protected`
- Capability: `Pure`

### IntegerQ

Tests whether an expression is an exact integer atom.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - IntegerQ::argx — expected exactly `1` argument(s), got `2`

### IntegerReverse

Reverses the digits of an exact integer in a given base.

- Attributes: `Protected`, `ReadProtected`
- Capability: `Pure`
- Messages:
  - IntegerReverse::argtu — expected between `1` and `2` argument(s), got `3`
  - IntegerReverse::ibase — Base `1` is not an integer greater than 1 in `2`.

### IntersectingQ

Tests whether two Lists share any element, optionally using SameTest.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - IntersectingQ::argx — expected exactly `1` argument(s), got `2`

### Intersection

Returns sorted unique elements common to all same-head normal expressions; SameTest keeps the first representative in structural scan order.

- Attributes: `Flat`, `OneIdentity`, `Protected`, `ReadProtected`
- Capability: `Pure`
- Messages:
  - Intersection::argm — Intersection called with 0 arguments; 1 or more arguments are expected.
  - Intersection::heads — Heads `1` and `2` at positions `3` and 1 are expected to be the same.
  - Intersection::normal — Nonatomic expression expected at position 1 in `1`.

### Join

Concatenates same-head normal expressions and associations in the current top-level subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Join::heads — Heads `1` and `2` at positions 1 and `3` are expected to be the same.
  - Join::headsd — Expression `1` at position `2` is expected to have head `3` for all expressions at level `4`.
  - Join::incpt — Incompatible elements in `1` cannot be joined.
  - Join::normal — Nonatomic expression expected at position 1 in `1`.
  - Join::normal1 — Expression `1` at position 1 is expected to have nonatomic subexpression at level `2`.

### JoinAcross

Joins two lists of associations by shared key values.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - JoinAcross::argb — JoinAcross called with `1` arguments; 3 or 4 arguments are expected.
  - JoinAcross::lists — The argument at position `1` in `2` is not a valid list of Associations.
  - JoinAcross::keys — The argument `1` is not a valid key specification: a key, a rule of keys, or a non-empty list of these.
  - JoinAcross::jtype — The argument `1` is not a valid join type; "Inner", "Left", "Right", or "Outer" is expected.

### Key

Represents a keyed selector in the current first-pass association subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Key::argx — Key called with `1` arguments; 1 argument is expected.
  - Key::invak — The argument `1` is not a valid Association.

### KeyComplement

Builds a key-complement association in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - KeyComplement::argx — KeyComplement called with `1` arguments; 1 argument is expected.
  - KeyComplement::empt — Argument `1` should be a nonempty list.
  - KeyComplement::invar — The argument `1` is not a valid list of Associations or rules.
  - KeyComplement::invrl — The argument `1` is not a valid Association or a list of rules.
  - KeyComplement::nonopt — Options expected (instead of `1`) beyond position `2` in `3`. An option must be a rule or a list of rules.
  - KeyComplement::optx — Unknown option `1` in `2`.

### KeyDrop

Drops selected keys from an association in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - KeyDrop::argt — KeyDrop called with `1` arguments; 1 or 2 arguments are expected.
  - KeyDrop::invrl — The argument `1` is not a valid Association or a list of rules.
  - KeyDrop::invru — The argument `1` is not a valid Association or a list of rules.

### KeyDropFrom

Drops keys from an association, including held symbol mutation in the current first-pass subset.

- Attributes: `HoldFirst`, `SubValuesHoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - KeyDropFrom::argr — KeyDropFrom called with 1 argument; 2 arguments are expected.
  - KeyDropFrom::argrx — KeyDropFrom called with `1` arguments; 2 arguments are expected.
  - KeyDropFrom::argx — `1` called with `2` arguments; 1 argument is expected.
  - KeyDropFrom::blnoval — The symbol `1` at position `2` should have an immediate value defined.
  - KeyDropFrom::invak — The argument `1` is not a valid Association.
  - KeyDropFrom::rvalue — `1` is not a variable with a value, so its value cannot be changed.
  - KeyDropFrom::wrsym — Symbol `1` is Protected.

### KeyExistsQ

Tests whether an association contains a given key in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - KeyExistsQ::argt — KeyExistsQ called with `1` arguments; 1 or 2 arguments are expected.
  - KeyExistsQ::invrl — The argument `1` is not a valid Association or a list of rules.

### KeyFreeQ

Tests whether an association does not contain a key in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - KeyFreeQ::argrx — KeyFreeQ called with `1` arguments; 2 arguments are expected.
  - KeyFreeQ::invas — The argument `1` is not a valid Association or rule.
  - KeyFreeQ::invrk — The argument `1` is not a valid rule.

### KeyGroupBy

Groups association entries by transformed keys, including the current reducer form.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - KeyGroupBy::argx — `1` called with `2` arguments; 1 argument is expected.

### KeyIntersection

Builds a key-intersection association in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - KeyIntersection::argx — KeyIntersection called with `1` arguments; 1 argument is expected.
  - KeyIntersection::invar — The argument `1` is not a valid list of Associations or rules.
  - KeyIntersection::invas — The argument `1` is not a valid Association or rule.

### KeyMap

Maps a function over association keys in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - KeyMap::argt — KeyMap called with `1` arguments; 1 or 2 arguments are expected.
  - KeyMap::invak — The argument `1` is not a valid Association.

### KeyMemberQ

Tests whether an association contains a key in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - KeyMemberQ::argrx — KeyMemberQ called with `1` arguments; 2 arguments are expected.
  - KeyMemberQ::invas — The argument `1` is not a valid Association or rule.
  - KeyMemberQ::invrk — The argument `1` is not a valid rule.

### KeySelect

Selects association entries whose keys satisfy a criterion in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - KeySelect::argt — KeySelect called with `1` arguments; 1 or 2 arguments are expected.
  - KeySelect::invru — The argument `1` is not a valid Association or a list of rules.

### KeySort

Sorts association entries by key in the current canonical ordering subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - KeySort::argt — KeySort called with `1` arguments; 1 or 2 arguments are expected.
  - KeySort::invrl — The argument `1` is not a valid Association or a list of rules.

### KeySortBy

Sorts association entries by transformed keys, optionally using a supplied binary ordering predicate.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - KeySortBy::argt — KeySortBy called with `1` arguments; 1, 2, or 3 arguments are expected.
  - KeySortBy::invrl — The argument `1` is not a valid Association or a list of rules.

### KeyTake

Keeps selected keys from an association in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - KeyTake::argt — KeyTake called with `1` arguments; 1 or 2 arguments are expected.
  - KeyTake::invrl — The argument `1` is not a valid Association or a list of rules.
  - KeyTake::invru — The argument `1` is not a valid Association or a list of rules.

### KeyUnion

Builds keyed unions, including the current missing-key fill-function form.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - KeyUnion::argt — KeyUnion called with `1` arguments; 1 or 2 arguments are expected.
  - KeyUnion::invar — The argument `1` is not a valid list of Associations or rules.
  - KeyUnion::invas — The argument `1` is not a valid Association or rule.

### KeyValueMap

Maps a binary function over association key-value pairs in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - KeyValueMap::argt — KeyValueMap called with `1` arguments; 1 or 2 arguments are expected.
  - KeyValueMap::invak — The argument `1` is not a valid Association.

### KeyValuePattern

Represents an association key/value pattern specification in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`

### Keys

Returns association keys in structural order.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Keys::argt — Keys called with `1` arguments; 1 or 2 arguments are expected.
  - Keys::invrl — The argument `1` is not a valid Association or a list of rules.

### LCM

Computes the least common multiple of exact integers.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`

### Last

Returns the last structural element, with an optional held default.

- Attributes: `HoldRest`, `Protected`
- Capability: `Pure`
- Messages:
  - Last::argtu — expected between `1` and `2` argument(s), got `3`
  - Last::nolast — `1` has zero length and no `2` element.
  - Last::normal — Nonatomic expression expected at position 1 in `1`.

### LazyDrop

Builds a lazy sequence that skips a non-negative number of source elements.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - LazyDrop::argx — expected exactly 2 argument(s), got `1`
  - LazyDrop::argform — expected a LazyList and a non-negative integer

### LazyFoldList

Builds a lazy sequence of successive accumulated values, including the initial value.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - LazyFoldList::argx — expected exactly 3 argument(s), got `1`
  - LazyFoldList::argform — expected a LazyList as the third argument

### LazyList

Represents an opaque pure lazy sequence descriptor.

- Attributes: `HoldAllComplete`, `Protected`
- Capability: `Pure`

### LazyMap

Builds a lazy sequence that applies a function to each source element.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - LazyMap::argx — expected exactly 2 argument(s), got `1`
  - LazyMap::argform — expected a LazyList as the second argument

### LazyNestList

Builds an infinite lazy sequence by repeatedly applying a function to an initial value.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - LazyNestList::argx — expected exactly 2 argument(s), got `1`

### LazyPermutations

Enumerates distinct list permutations lazily in rum's canonical lexicographic (sorted-first, not input-first) order.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - LazyPermutations::argx — expected exactly 1 argument(s), got `1`
  - LazyPermutations::argform — expected a List

### LazyRange

Builds an infinite lazy arithmetic sequence from an optional start and step.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - LazyRange::argtu — expected between 0 and 2 argument(s), got `1`
  - LazyRange::argform — start and step must be Integer or Rational values

### LazySelect

Builds a lazy sequence retaining source elements whose predicate returns True.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - LazySelect::argx — expected exactly 2 argument(s), got `1`
  - LazySelect::argform — expected a LazyList as the second argument

### LazySubsets

Enumerates the subsets of a list lazily in size-major, index-lexicographic order.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - LazySubsets::argx — expected exactly 1 argument(s), got `1`
  - LazySubsets::argform — expected a List

### LazyTable

Builds a lazy arithmetic table whose held body evaluates once per realized element.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - LazyTable::argx — expected exactly 2 argument(s), got `1`
  - LazyTable::argform — expected an arithmetic iterator with a symbol, exact rational bounds, and a nonzero step

### LazyTake

Builds a finite lazy sequence capped by a non-negative element count.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - LazyTake::argx — expected exactly 2 argument(s), got `1`
  - LazyTake::argform — expected a LazyList and a non-negative integer or UpTo[n]

### LazyTuples

Enumerates repeated tuples or a list-of-lists cartesian product lazily with the last index varying fastest.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - LazyTuples::argx — expected 1 or 2 argument(s), got `1`
  - LazyTuples::argform — expected a list of lists, or a list and a non-negative integer

### LeafCount

Counts atomic leaves in an expression's full structural tree, including heads.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - LeafCount::argx — expected exactly `1` argument(s), got `2`

### LeapYearQ

Tests whether an exact integer year is a leap year in the proleptic Gregorian calendar.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - LeapYearQ::argx — expected exactly `1` argument(s), got `2`
  - LeapYearQ::date — Reports an invalid year value.

### Length

Returns the number of structural elements in an expression.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Length::argx — expected exactly `1` argument(s), got `2`

### LengthWhile

Counts the longest prefix satisfying a criterion in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - LengthWhile::argtu — expected between `1` and `2` argument(s), got `3`

### Less

Tests strict increasing order in the current exact comparable subset.

- Attributes: `Protected`
- Capability: `Pure`

### LessEqual

Tests nondecreasing order in the current exact comparable subset.

- Attributes: `Protected`
- Capability: `Pure`

### LessEqualThan

Represents a unary comparison operator equivalent to LessEqual[#, bound] when applied.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - LessEqualThan::argx — `1` called with `2` arguments; 1 argument is expected.

### LessThan

Represents a unary comparison operator equivalent to Less[#, bound] when applied.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - LessThan::argx — `1` called with `2` arguments; 1 argument is expected.

### LetterCharacter

Represents the current string-pattern atom for one alphabetic character.

- Attributes: `Protected`
- Capability: `Pure`

### LetterQ

Tests whether all characters in a string belong to the current letter subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - LetterQ::argx — expected exactly `1` argument(s), got `2`

### Level

Collects subexpressions found on levels specified by a standard levelspec.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Level::argform — Reports an invalid Heads option.
  - Level::argtu — expected between `1` and `2` argument(s), got `3`
  - Level::level — Level specification `1` is not of the form n, {n} or {m, n}.
  - Level::nonopt — Options expected (instead of `1`) beyond position `2` in `3`. An option must be a rule or a list of rules.
  - Level::opttf — Value of option `1` -> `2` should be True or False.
  - Level::optx — Unknown option `1` in `2`.

### LexicographicOrder

Compares two expressions lexicographically under the default order or a supplied binary predicate.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - LexicographicOrder::argtu — expected between `1` and `2` argument(s), got `3`

### ListQ

Tests whether an expression has head List.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - ListQ::argx — expected exactly `1` argument(s), got `2`

### Longest

Represents the current string-pattern wrapper that prefers the longest match.

- Attributes: `Protected`
- Capability: `Pure`

### LongestCommonSubsequence

Finds the longest common contiguous substring or sublist in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - LongestCommonSubsequence::argx — expected exactly `1` argument(s), got `2`

### Lookup

Looks up a key in an association in the current first-pass subset.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - Lookup::argb — Lookup called with `1` arguments; between 2 and 4 arguments are expected.
  - Lookup::invrl — The argument `1` is not a valid Association or a list of rules.

### LowerCaseQ

Tests whether every character in a string is lowercase.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - LowerCaseQ::argx — expected exactly `1` argument(s), got `2`

### LowercaseLetters

Evaluates to the lowercase ASCII letter character-class string.

- Attributes: `Protected`
- Capability: `Pure`

### MachineNumberQ

Tests whether an expression is a machine number; currently always False because only exact numbers exist.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - MachineNumberQ::argx — expected exactly `1` argument(s), got `2`

### Map

Applies a function over selected levels of an expression in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Map::argform — expected a standard levelspec or Heads -> True/False as the third argument
  - Map::argtu — expected between `1` and `2` argument(s), got `3`

### MapAll

Applies a function to every subexpression in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - MapAll::argform — expected a Heads -> True/False option as the third argument
  - MapAll::argtu — expected between `1` and `2` argument(s), got `3`

### MapApply

Applies head replacement one level down in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - MapApply::argform — expected a standard levelspec or Heads -> True/False as the third argument
  - MapApply::argtu — expected between `1` and `2` argument(s), got `3`

### MapAt

Applies a function at exact part specifications in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - MapAt::argform — expected an exact part specification or list of exact part specifications as the third argument
  - MapAt::argtu — expected between `1` and `2` argument(s), got `3`
  - MapAt::partw — Part `1` of `2` does not exist.

### MapIndexed

Applies a function to parts together with their positions in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - MapIndexed::argform — expected a standard levelspec or Heads -> True/False as the third argument
  - MapIndexed::argtu — expected between `1` and `2` argument(s), got `3`

### MapThread

Threads corresponding elements from multiple expressions in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - MapThread::argform — expected a positive integer level as the third argument
  - MapThread::argtu — expected between `1` and `2` argument(s), got `3`

### MatchQ

Tests whether an expression matches a pattern.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - MatchQ::argx — expected exactly `1` argument(s), got `2`

### MatrixQ

Tests whether an expression is a rectangular two-dimensional List array, optionally requiring every leaf to satisfy a test.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - MatrixQ::argtu — expected between `1` and `2` argument(s), got `3`

### Max

Returns the maximum exact integer or rational operand.

- Attributes: `NumericFunction`, `Protected`
- Capability: `Pure`

### MaxBy

Returns the first top-level element attaining the maximum evaluated key, or the top n in decreasing key order with input-order ties.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - MaxBy::argx — expected exactly `1` argument(s), got `2`
  - MaxBy::empty — MaxBy cannot select from an empty list.
  - MaxBy::normal — Nonatomic expression expected at position 1 in `1`.

### MaxMemoryUsed

Returns the peak session memory-proxy byte count high-water mark subset.

- Attributes: `Protected`
- Capability: `Diagnostic`

### MaximalBy

Returns all top-level elements attaining the maximum evaluated key, preserving input order, with an operator form.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - MaximalBy::argtu — expected between `1` and `2` argument(s), got `3`
  - MaximalBy::normal — Nonatomic expression expected at position 1 in `1`.

### Md5

Computes an MD5 digest over a string's UTF-8 bytes or a ByteArray, returning lowercase hex by default.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: intended only for interoperability and checksums; it is not collision-resistant.
- Messages:
  - Md5::argtu — expected between `1` and `2` argument(s), got `3`
  - Md5::bytearray — invalid ByteArray payload
  - Md5::format — unknown digest output format; use "Hex" (default) or "Bytes"

### Mean

Aggregates a non-empty list as Total[list] divided by Length[list] through exact arithmetic.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Mean::argx — expected exactly `1` argument(s), got `2`
  - Mean::empty — Mean cannot aggregate an empty list.

### Median

Returns the middle element of a canonically sorted non-empty list, averaging the two middle values exactly for even lengths.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Median::argx — expected exactly `1` argument(s), got `2`
  - Median::empty — Median cannot aggregate an empty list.

### MemberQ

Tests whether any subexpression matches a pattern on levels specified by a standard levelspec.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: supports the Wolfram Language-documented operator form MemberQ[patt][expr], equivalent to MemberQ[expr, patt].
- Messages:
  - MemberQ::argform — Reports an invalid traversal argument or Heads option.
  - MemberQ::argtu — expected between `1` and `2` argument(s), got `3`
  - MemberQ::argx — `1` called with `2` arguments; 1 argument is expected.
  - MemberQ::level — Level specification `1` is not of the form n, {n} or {m, n}.
  - MemberQ::nonopt — Options expected (instead of `1`) beyond position `2` in `3`. An option must be a rule or a list of rules.
  - MemberQ::opttf — Value of option `1` -> `2` should be True or False.
  - MemberQ::optx — Unknown option `1` in `2`.

### MemoryConstrained

Evaluates an expression under the current cooperative memory-proxy limit subset.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - MemoryConstrained::argform — expected the memory limit argument to be a non-negative exact integer
  - MemoryConstrained::argtu — expected between `1` and `2` argument(s), got `3`

### MemoryInUse

Returns the current session memory-proxy byte count subset.

- Attributes: `Protected`
- Capability: `Diagnostic`

### Merge

Merges associations by key using a combiner in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Merge::argt — Merge called with `1` arguments; 1 or 2 arguments are expected.
  - Merge::list1 — The argument `1` is not a valid list of Associations or rules or lists of rules.

### MergeNested

Recursively merges nested associations by key using a combiner at leaves.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - MergeNested::argx — `1` called with `2` arguments; 1 argument is expected.

### Message

Emits a message described by a MessageName expression.

- Attributes: `HoldFirst`, `Protected`
- Capability: `Pure`
- Messages:
  - Message::argform — expected the first argument to be a MessageName expression
  - Message::argx — expected at least 1 argument

### MessageDefinitions

Lists stored message template definitions for the whole session, a symbol, or one message query.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - MessageDefinitions::argform — expected no arguments, a symbol or symbol-name string, or a MessageName expression
  - MessageDefinitions::argx — expected 0 or 1 argument(s), got `1`

### MessageName

Represents a symbolic message name such as f::tag.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - MessageName::argform — expected a symbol or symbol-name string, plus a message tag string or symbol
  - MessageName::argx — expected exactly `1` argument(s), got `2`

### MessageTags

Lists message tags defined globally, for a symbol, or for one message query.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - MessageTags::argform — expected no arguments, a symbol or symbol-name string, or a MessageName expression
  - MessageTags::argx — expected 0 or 1 argument(s), got `1`

### Messages

Lists message names defined for the whole session, a symbol, or one message query.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - Messages::argform — expected no arguments, a symbol or symbol-name string, or a MessageName expression
  - Messages::argx — expected 0 or 1 argument(s), got `1`

### Min

Returns the minimum exact integer or rational operand.

- Attributes: `NumericFunction`, `Protected`
- Capability: `Pure`

### MinBy

Returns the first top-level element attaining the minimum evaluated key, or the bottom n in increasing key order with input-order ties.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - MinBy::argx — expected exactly `1` argument(s), got `2`
  - MinBy::empty — MinBy cannot select from an empty list.
  - MinBy::normal — Nonatomic expression expected at position 1 in `1`.

### MinMax

Returns the exact minimum and maximum for one scalar or list argument.

- Attributes: `Protected`
- Capability: `Pure`

### MinimalBy

Returns all top-level elements attaining the minimum evaluated key, preserving input order, with an operator form.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - MinimalBy::argtu — expected between `1` and `2` argument(s), got `3`
  - MinimalBy::normal — Nonatomic expression expected at position 1 in `1`.

### Minus

Negates one exact or symbolic argument.

- Attributes: `Listable`, `NumericFunction`, `Protected`
- Capability: `Pure`

### Missing

Represents missing-data markers in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`

### MissingQ

Tests whether an expression is a Missing object.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - MissingQ::argx — expected exactly `1` argument(s), got `2`

### Mod

Returns the modulus of one integer with respect to another.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`
- Messages:
  - Mod::argx — expected exactly `1` argument(s), got `2`

### Module

Creates lexical local symbols.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - Module::argform — expected a list of symbols or symbol initializers as the first argument
  - Module::argx — expected exactly `1` argument(s), got `2`

### MoebiusMu

Computes the Moebius mu function for an exact integer.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`
- Messages:
  - MoebiusMu::argx — expected exactly `1` argument(s), got `2`

### Monitor

Evaluates an expression while exposing the current first-pass monitor display and callback subset.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - Monitor::argform — Reports an invalid monitor option rule.
  - Monitor::argtu — expected between `1` and `2` argument(s), got `3`

### Most

Returns an expression with its last structural element removed.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Most::argx — expected exactly `1` argument(s), got `2`
  - Most::nomost — Cannot take `1` of expression `2` with length zero.
  - Most::normal — Nonatomic expression expected at position 1 in `1`.

### Multinomial

Computes exact multinomial coefficients for non-negative integer arguments.

- Attributes: `Listable`, `NumericFunction`, `Orderless`, `Protected`
- Capability: `Pure`

### NameQ

Tests whether a string is the exact name of an existing symbol.

- Attributes: `Protected`
- Capability: `Pure`

### Names

Lists symbol names matching a current first-pass name pattern subset.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: SpellingCorrection is currently supported only at its False default; any other effective value emits Names::optval and leaves the call inert.
- Messages:
  - Names::argform — expected a string pattern, RegularExpression, StringExpression, Alternatives, or a list of them
  - Names::optval — Reports an unsupported Names option value.

### Nand

Evaluates a negated conjunction with current short-circuit boolean semantics.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`

### Nearest

Finds nearest strings or exact numeric values in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Nearest::argtu — expected between `1` and `2` argument(s), got `3`

### Needs

Loads a package only if its context has not already been loaded.

- Attributes: `Protected`
- Capability: `FileRead`
- Messages:
  - Needs::argform — Reports an invalid context name or explicit load target.
  - Needs::argx — expected `1` argument(s), got `2`
  - Needs::load — context `1` could not be loaded

### Negative

Tests negativity in the current exact integer/rational subset.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`
- Messages:
  - Negative::argx — expected exactly `1` argument(s), got `2`

### Nest

Applies a function repeatedly a fixed number of times.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Nest::argform — expected a non-negative integer iteration count
  - Nest::argtu — expected between `1` and `2` argument(s), got `3`

### NestList

Returns repeated applications of a function in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - NestList::argform — expected a non-negative integer iteration count
  - NestList::argtu — expected between `1` and `2` argument(s), got `3`

### NestWhile

Repeatedly applies a function while a test stays True in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - NestWhile::argform — expected All or a positive integer test arity as the fourth argument
  - NestWhile::argtu — expected between `1` and `2` argument(s), got `3`

### NestWhileList

Returns repeated applications while a test stays True in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - NestWhileList::argform — expected All or a positive integer test arity as the fourth argument
  - NestWhileList::argtu — expected between `1` and `2` argument(s), got `3`

### NextPrime

Returns the next or previous prime in rum's signed-prime order.

- Attributes: `Protected`, `ReadProtected`
- Capability: `Pure`
- Messages:
  - NextPrime::argtu — expected between `1` and `2` argument(s), got `3`

### NonNegative

Tests non-negativity in the current exact integer/rational subset.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`
- Messages:
  - NonNegative::argx — expected exactly `1` argument(s), got `2`

### NonPositive

Tests non-positivity in the current exact integer/rational subset.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`
- Messages:
  - NonPositive::argx — expected exactly `1` argument(s), got `2`

### NoneTrue

Tests whether no selected structural parts satisfy a criterion in the current boolean subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - NoneTrue::argform — expected a standard levelspec as the third argument
  - NoneTrue::argtu — expected between `1` and `2` argument(s), got `3`
  - NoneTrue::argx — `1` called with `2` arguments; 1 argument is expected.

### Nor

Evaluates a negated disjunction with current short-circuit boolean semantics.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`

### Normal

Converts an association into a list of rules in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Normal::argb — Normal called with `1` arguments; between 1 and 3 arguments are expected.

### NormalizeRules

Normalizes a rule or rule list into an explicit canonical list form for debugging.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - NormalizeRules::argform — expected a rule, a rule list, or a symbol that evaluates to rules
  - NormalizeRules::argx — expected exactly `1` argument(s), got `2`

### Not

Negates a boolean value in the current exact boolean subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Not::argx — expected exactly `1` argument(s), got `2`

### Nothing

Represents a list element that is removed from evaluated Lists.

- Attributes: `Protected`
- Capability: `Pure`

### NumberQ

Tests whether an expression is an exact numeric atom in the current number subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - NumberQ::argx — expected exactly `1` argument(s), got `2`

### NumberString

Represents the current string-pattern atom for decimal-style numeric text.

- Attributes: `Protected`
- Capability: `Pure`

### Numerator

Returns the numerator of exact integers and rationals.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`
- Messages:
  - Numerator::argx — expected exactly `1` argument(s), got `2`

### NumericQ

Tests whether an expression is currently numeric in the exact-number subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - NumericQ::argx — expected exactly `1` argument(s), got `2`

### NumericalOrder

Compares two expressions using exact numeric order when possible, then canonical order.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - NumericalOrder::argx — expected exactly `1` argument(s), got `2`

### OddQ

Tests whether an expression is currently an odd exact integer.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`
- Messages:
  - OddQ::argx — expected exactly `1` argument(s), got `2`

### Off

Disables session-level assertion checking or emission of matching messages in the current selector subset.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - Off::argform — expected one or more symbols, MessageName expressions, "symbol::tag" strings, or lists of those
  - Off::argx — expected at least 1 argument

### On

Enables session-level assertion checking or emission of matching messages in the current selector subset.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - On::argform — expected one or more symbols, MessageName expressions, "symbol::tag" strings, or lists of those
  - On::argx — expected at least 1 argument

### Once

Evaluates an expression at most once per session-local cache key in the current first-pass subset.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`
- Limits: uses deterministic session-local caching and performs no clock reads. ExpirationDate accepts only None; PersistenceTime accepts None (cache) or integer 0 (no cache). Other effective values emit Once::pers and leave the held call inert.
- Messages:
  - Once::argform — Reports an invalid option specification.
  - Once::argtu — expected between `1` and `2` argument(s), got `3`
  - Once::pers — Unsupported value `1` for option `2` in Once; supported values are ExpirationDate -> None and PersistenceTime -> None or 0.

### Operate

Applies an operator to heads in the head chain of an expression in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Operate::argform — expected a non-negative integer as the third argument
  - Operate::argtu — expected between `1` and `2` argument(s), got `3`

### OperatorApplied

Represents a curried operator wrapper in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - OperatorApplied::argform — expected a non-negative integer, an index list, or k -> index-list as the specification argument
  - OperatorApplied::argtu — expected between `1` and `2` argument(s), got `3`

### OptionValue

Looks up an option value from explicit rules or a symbol's defaults.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - OptionValue::argform — Reports an invalid option target, option name, or explicit option rules.
  - OptionValue::argtu — expected between `1` and `2` argument(s), got `3`
  - OptionValue::nodef — Unknown option `1` for `2`.
  - OptionValue::optnf — Option name `1` not found in defaults for `2`.

### Options

Lists the current default option rules for a symbol.

- Attributes: `HoldFirst`, `Protected`
- Capability: `Pure`
- Messages:
  - Options::argform — expected a symbol or symbol-name string argument
  - Options::argx — expected exactly `1` argument(s), got `2`

### Or

Evaluates a disjunction with current short-circuit boolean semantics.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`

### Order

Orders two expressions using exact numeric order when possible, then the current canonical textual ordering subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Order::argx — expected exactly `1` argument(s), got `2`

### OrderedQ

Tests whether an expression's arguments are ordered under the default order or a supplied binary predicate.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - OrderedQ::argtu — expected between `1` and `2` argument(s), got `3`
  - OrderedQ::normal — Nonatomic expression expected at position 1 in `1`.

### Outer

Forms outer combinations of same-head expressions in the current dense-array subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Outer::argtu — expected between `1` and `2` argument(s), got `3`
  - Outer::heads — Heads `1` and `2` at positions `3` and 2 are expected to be the same.

### OutputForm

Represents the current formatting symbol accepted by the current ToString subset.

- Attributes: `Protected`
- Capability: `Pure`

### Overlaps

Represents the current option symbol for overlapping string-match counting.

- Attributes: `Protected`
- Capability: `Pure`

### OwnValues

Lists stored immediate or delayed own-value definitions for a symbol, reconstructing right-hand-side guards.

- Attributes: `HoldFirst`, `Protected`
- Capability: `Pure`
- Messages:
  - OwnValues::argform — expected a symbol or symbol-name string argument
  - OwnValues::argx — expected exactly `1` argument(s), got `2`
  - OwnValues::setval — Reports an invalid OwnValues assignment.

### PackageScope

Declares conventionally package-scoped symbols inside a held DefinePackage source unit.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`

### PadLeft

Pads or truncates a normal expression on the left using scalar lengths or rectangular list dimensions.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: does not support the margin argument; list dimensions apply to rectangular nested lists only.
- Messages:
  - PadLeft::argtu — expected between `1` and `2` argument(s), got `3`
  - PadLeft::ilsm — Non-negative machine-sized integer or non-empty list of non-negative machine-sized integers expected at position 2 in `1`.
  - PadLeft::normal — Nonatomic expression expected at position 1 in `1`.

### PadRight

Pads or truncates a normal expression on the right using scalar lengths or rectangular list dimensions.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: does not support the margin argument; list dimensions apply to rectangular nested lists only.
- Messages:
  - PadRight::argtu — expected between `1` and `2` argument(s), got `3`
  - PadRight::ilsm — Non-negative machine-sized integer or non-empty list of non-negative machine-sized integers expected at position 2 in `1`.
  - PadRight::normal — Nonatomic expression expected at position 1 in `1`.

### ParentDirectory

Returns the parent directory for an existing directory path.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - ParentDirectory::argform — expected a string directory name
  - ParentDirectory::argx — expected exactly `1` argument(s), got `2`

### ParseArgs

Parses script arguments into positional arguments, --key=value options, and --flag names; with "ShortOptions" -> True, single-dash clusters become single-letter flags, without attached-value syntax.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: recognizes --key=value options, bare --flag flags, and -- as the end of option parsing; "ShortOptions" -> True recognizes single-dash flag clusters but does not support attached values.
- Messages:
  - ParseArgs::argtu — expected between `1` and `2` argument(s), got `3`

### Part

Extracts structural parts using integer positions, All, Span, and exact nested part specifications.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Part::argm — Part called with 0 arguments; 1 or more arguments are expected.
  - Part::partd — Part specification `1` is longer than depth of object.
  - Part::partw — Part `1` of `2` does not exist.
  - Part::pkspec1 — The expression `1` cannot be used as a part specification.
  - Part::pmix — Integer and Key specifications cannot be mixed in the same part specification.
  - Part::pspec1 — Part specification `1` is not applicable.
  - Part::span — Reports an invalid Span part specification.
  - Part::take — Reports positions that cannot be taken from an expression.

### Partition

Partitions a flat normal expression into fixed-size blocks; aligned forms anchor the first element at kL and stop once the last element appears at or beyond kR.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: supports flat positive scalar size/offset forms; aligned forms use positive {kL, kR}, prepend kL - 1 fill positions, stop at the first d-stepped block where the final item appears at position >= kR, use cyclic list padding when padding is omitted, and use ragged no-padding output for {}.
- Messages:
  - Partition::align — Position 4 of `1` must be {kL, kR}, where kL and kR are positive machine-sized integers not greater than the block size `2`.
  - Partition::argtu — expected between `1` and `2` argument(s), got `3`
  - Partition::argx — `1` called with `2` arguments; 1 argument is expected.
  - Partition::ilsmp — Positive machine-sized integer expected at position `1` of `2`.
  - Partition::npart — The expression `1` cannot be partitioned.

### PartitionsP

Computes the number of integer partitions by Euler's pentagonal recurrence.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`
- Messages:
  - PartitionsP::argx — expected exactly `1` argument(s), got `2`

### PartitionsQ

Computes the number of partitions into distinct parts.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`
- Messages:
  - PartitionsQ::argx — expected exactly `1` argument(s), got `2`

### Pattern

Represents the current named pattern-binding head.

- Attributes: `Protected`
- Capability: `Pure`

### Pause

Pauses for at least a nonnegative exact number of seconds in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Clock`
- Messages:
  - Pause::argx — expected exactly `1` argument(s), got `2`
  - Pause::numnm — Non-negative machine-sized number expected at position 1 in `1`.

### Permutations

Returns distinct permutations of a list in rum's canonical lexicographic (sorted-first, not input-first) order; fixed-length forms are deferred.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Permutations::argx — expected exactly `1` argument(s), got `2`
  - Permutations::normal — List expected at position 1 in `1`.

### Pluralize

Applies a simple English pluralization subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Pluralize::argtu — expected between `1` and `2` argument(s), got `3`

### Plus

Adds exact numbers and canonically combines additive expressions.

- Attributes: `Listable`, `Orderless`, `Flat`, `OneIdentity`, `Protected`
- Capability: `Pure`

### Pochhammer

Computes finite rising factorial products for non-negative integer counts.

- Attributes: `Listable`, `NumericFunction`, `Protected`
- Capability: `Pure`
- Messages:
  - Pochhammer::argx — expected exactly `1` argument(s), got `2`

### PolynomialExpressionQ

Tests whether an expression is polynomial in the requested variable subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - PolynomialExpressionQ::argform — expected a variable or variable list
  - PolynomialExpressionQ::argtu — expected between `1` and `2` argument(s), got `3`

### PolynomialQ

Tests whether an expression is polynomial in the current exact symbolic subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - PolynomialQ::argform — expected a variable or variable list
  - PolynomialQ::argtu — expected between `1` and `2` argument(s), got `3`

### Position

Finds positions of subexpressions matching a pattern on levels specified by a standard levelspec.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: supports the Wolfram Language-documented operator form Position[patt][expr], equivalent to Position[expr, patt].
- Messages:
  - Position::argform — Reports an invalid traversal argument or Heads option.
  - Position::argtu — expected between `1` and `2` argument(s), got `3`
  - Position::argx — `1` called with `2` arguments; 1 argument is expected.
  - Position::innf — Non-negative integer or Infinity expected at position `1` in `2`.
  - Position::level — Level specification `1` is not of the form n, {n} or {m, n}.
  - Position::nonopt — Options expected (instead of `1`) beyond position `2` in `3`. An option must be a rule or a list of rules.
  - Position::opttf — Value of option `1` -> `2` should be True or False.
  - Position::optx — Unknown option `1` in `2`.

### PositionIndex

Builds an association from elements to 1-based position lists in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - PositionIndex::argx — PositionIndex called with `1` arguments; 1 argument is expected.
  - PositionIndex::invrp — The argument `1` is not a valid Association or a list.

### Positive

Tests positivity in the current exact integer/rational subset.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`
- Messages:
  - Positive::argx — expected exactly `1` argument(s), got `2`

### PossibleZeroQ

Tests exact zero in the current exact integer/rational subset.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`
- Messages:
  - PossibleZeroQ::argx — expected exactly `1` argument(s), got `2`

### Power

Raises an exact number to an integer power when possible.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`
- Messages:
  - Power::argx — expected exactly `1` argument(s), got `2`

### PowerMod

Computes modular exponentiation for exact integers.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`
- Messages:
  - PowerMod::argx — expected exactly `1` argument(s), got `2`
  - PowerMod::ninv — `1` is not invertible modulo `2`.

### PreDecrement

Decrements a held symbol or exact Part target and returns the updated value.

- Attributes: `HoldFirst`, `Protected`
- Capability: `Pure`
- Messages:
  - PreDecrement::argform — expected an exact Part target as the first argument
  - PreDecrement::argx — expected exactly `1` argument(s), got `2`
  - PreDecrement::rvalue — `1` is not a variable with a value, so its value cannot be changed.
  - PreDecrement::wrsym — Symbol `1` is Protected.

### PreIncrement

Increments a held symbol or exact Part target and returns the updated value.

- Attributes: `HoldFirst`, `Protected`
- Capability: `Pure`
- Messages:
  - PreIncrement::argform — expected an exact Part target as the first argument
  - PreIncrement::argx — expected exactly `1` argument(s), got `2`
  - PreIncrement::rvalue — `1` is not a variable with a value, so its value cannot be changed.
  - PreIncrement::wrsym — Symbol `1` is Protected.

### Prepend

Prepends an element to the front of a normal expression, with current operator-form support.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Prepend::argtu — expected between `1` and `2` argument(s), got `3`
  - Prepend::normal — Nonatomic expression expected at position 1 in `1`.

### PrependTo

Prepends an evaluated item to a held symbol or exact Part target and stores the result.

- Attributes: `HoldFirst`, `SubValuesHoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - PrependTo::argform — expected an exact Part target as the first argument
  - PrependTo::argx — expected exactly `1` argument(s), got `2`
  - PrependTo::normal — Nonatomic expression expected at position 1 in `1`.
  - PrependTo::rvalue — `1` is not a variable with a value, so its value cannot be changed.
  - PrependTo::wrsym — Symbol `1` is Protected.

### Prime

Returns the n-th positive prime for a positive integer n.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`
- Limits: uses a bounded integer-work budget and sieve-based prime generation for supported exact positive integer indices.
- Messages:
  - Prime::argx — expected exactly `1` argument(s), got `2`
  - Prime::intpp — Positive integer argument expected in `1`.

### PrimeNu

Counts distinct prime factors of an exact integer.

- Attributes: `Protected`, `ReadProtected`
- Capability: `Pure`
- Messages:
  - PrimeNu::argx — expected exactly `1` argument(s), got `2`

### PrimeOmega

Counts prime factors with multiplicity for an exact integer.

- Attributes: `Protected`, `ReadProtected`
- Capability: `Pure`
- Messages:
  - PrimeOmega::argx — expected exactly `1` argument(s), got `2`

### PrimePi

Counts primes not greater than an exact integer or rational argument.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`
- Limits: uses a bounded integer-work budget and sieve-based counting after flooring exact rational inputs.
- Messages:
  - PrimePi::argx — expected exactly `1` argument(s), got `2`

### PrimeQ

Tests primality in the current exact-integer subset.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`
- Limits: uses direct trial division over exact integers.
- Messages:
  - PrimeQ::argx — expected exactly `1` argument(s), got `2`

### Print

Emits a line of plain print output through the runtime print stream.

- Attributes: `Protected`
- Capability: `Pure`

### Product

Multiplies expression values over concrete iterator specifications.

- Attributes: `HoldAll`, `Protected`, `ReadProtected`
- Capability: `Pure`
- Messages:
  - Product::argform — expected a supported repetition count or iterator specification
  - Product::argx — expected at least 2 argument(s), got `1`

### PropagateAborts

Represents the current first-pass CheckAbort option for rethrowing caught aborts.

- Attributes: `Protected`
- Capability: `Pure`

### Protect

Adds the Protected attribute to one or more symbols.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - Protect::argform — expected one or more symbols, symbol-name strings, or lists of them

### Public

Declares public symbols inside a held DefinePackage source unit.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`

### PunctuationCharacter

Represents the current string-pattern atom for one ASCII punctuation character.

- Attributes: `Protected`
- Capability: `Pure`

### PunctuationCharacters

Evaluates to the ASCII punctuation character-class string.

- Attributes: `Protected`
- Capability: `Pure`

### Quantile

Returns the nearest-rank quantile of a non-empty list using Ceiling[q * n] clamped to 1..n, with q exact in [0, 1].

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Quantile::argx — expected exactly `1` argument(s), got `2`
  - Quantile::empty — Quantile cannot aggregate an empty list.
  - Quantile::range — Quantile rank parameter must be an exact number between 0 and 1.

### Quiet

Evaluates an expression while suppressing matching runtime messages it emits.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - Quiet::argform — expected the optional message selector to be a symbol, a "symbol::tag" string, or a list of those
  - Quiet::argx — expected 1 or 2 argument(s), got `1`

### Quotient

Returns the floor-division quotient of two integers.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`
- Messages:
  - Quotient::argx — expected exactly `1` argument(s), got `2`

### RandomBytes

Returns entropy bytes as a ByteArray using the host entropy provider.

- Attributes: `Protected`
- Capability: `Random`
- Limits: draws directly from the host entropy provider, not from the deterministic session PRNG.
- Messages:
  - RandomBytes::argx — expected exactly `1` argument(s), got `2`
  - RandomBytes::entropy — could not read entropy: `1`

### RandomChoice

Chooses elements from a list using the deterministic session PRNG.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: uses the deterministic session PRNG; it is not cryptographic. RandomChoice[list, {n1, n2, ...}] returns a nested array of those dimensions. RandomChoice[weights -> elements] draws each element with probability proportional to its exact non-negative weight.
- Messages:
  - RandomChoice::argtu — expected between `1` and `2` argument(s), got `3`
  - RandomChoice::empty — expected a non-empty list
  - RandomChoice::weights — Reports invalid RandomChoice weights.

### RandomInteger

Draws exact integers uniformly from the deterministic session PRNG.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: uses the deterministic session PRNG with rejection sampling for unbiased exact integers; it is not cryptographic. There is no RandomReal in the current exact-number boundary. RandomInteger[spec, {n1, n2, ...}] returns a nested array of those dimensions.
- Messages:
  - RandomInteger::argtu — expected between `1` and `2` argument(s), got `3`

### RandomPassword

Draws an entropy-backed password string from an alphabet, optionally enforcing policy constraints.

- Attributes: `Protected`
- Capability: `Random`
- Limits: uses the host entropy provider directly with rejection sampling over the alphabet size to avoid modulo bias; it is not affected by SeedRandom or BlockRandom. The default alphabet is the 62-character A-Za-z0-9 set. Alphabet strings are read as code points, and duplicate code points weight the draw. RandomPassword[n, opts...] and RandomPassword[alphabet, n, opts...] accept string-keyed policy rules "Require" -> {group, ...}, "NoAdjacentRepeats" -> True|False, "Exclude" -> {substr, ...}, and "IgnoreCase" -> True|False. Require groups are exact character sets and must be non-empty subsets of the alphabet; IgnoreCase affects only Exclude matching. NoAdjacentRepeats forbids identical adjacent characters only, not sequential runs. Policy sampling draws whole candidate passwords uniformly from alphabet^n and accepts only candidates satisfying every constraint, so accepted outputs are uniform over the satisfying set. Unsatisfied policies fail closed after 10000 attempts.
- Messages:
  - RandomPassword::argform — expected RandomPassword[n, opts...] or RandomPassword[alphabet, n, opts...]
  - RandomPassword::entropy — could not read entropy: `1`
  - RandomPassword::password — Reports an invalid or unsatisfiable random-password policy.

### RandomSample

Samples distinct positions from a list using the deterministic session PRNG.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: uses the deterministic session PRNG; it is not cryptographic.
- Messages:
  - RandomSample::argtu — expected between `1` and `2` argument(s), got `3`
  - RandomSample::ksamp — sample count cannot exceed list length

### RandomString

Draws a string of characters from an alphabet using the deterministic session PRNG.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: uses the deterministic session PRNG with rejection sampling over the alphabet size; it is not cryptographic. The default alphabet is the 62-character A-Za-z0-9 set. Alphabet strings are read as code points, and duplicate code points weight the draw.
- Messages:
  - RandomString::argtu — expected between `1` and `2` argument(s), got `3`

### RandomUuid

Returns a reproducible version-4-format UUID string from the deterministic session PRNG.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: uses the deterministic session PRNG and is reproducible under the same seed; use an entropy reseed first when global uniqueness matters.
- Messages:
  - RandomUuid::argx — expected exactly `1` argument(s), got `2`

### Range

Constructs a plain list of exact integers from the current one-, two-, and three-argument subset.

- Attributes: `Protected`
- Capability: `Pure`

### RationalExpressionQ

Tests whether an expression is a rational expression in the requested variable subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - RationalExpressionQ::argform — expected a variable or variable list
  - RationalExpressionQ::argtu — expected between `1` and `2` argument(s), got `3`

### RealValuedNumberQ

Tests whether an expression is a real-valued number in the current exact-number subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - RealValuedNumberQ::argx — expected exactly `1` argument(s), got `2`

### RealValuedNumericQ

Tests whether an expression is real-valued and numeric in the current exact-number subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - RealValuedNumericQ::argx — expected exactly `1` argument(s), got `2`

### Reap

Evaluates an expression and collects values emitted by Sow in the current first-pass grouping subset.

- Attributes: `HoldFirst`, `Protected`
- Capability: `Pure`
- Messages:
  - Reap::argtu — expected between `1` and `2` argument(s), got `3`

### RegisterExceptionType

Registers a string exception type with an optional parent type.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - RegisterExceptionType::argtu — expected between `1` and `2` argument(s), got `3`
  - RegisterExceptionType::argform — exception type and parent must be strings
  - RegisterExceptionType::reregister — The exception type is already registered with a different parent.

### RegularExpression

Represents the current regular-expression string-pattern wrapper.

- Attributes: `Protected`
- Capability: `Pure`

### ReleaseHold

Removes one outer layer of Hold, HoldForm, HoldComplete, HoldPattern, HoldFirst, HoldRest, HoldAll, or HoldAllComplete, then resumes evaluation.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - ReleaseHold::argx — expected exactly `1` argument(s), got `2`

### Remove

Removes the current stored state for one or more symbols in the first-pass namespace subset.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - Remove::argform — expected one or more symbols, symbol-name strings, or lists of them
  - Remove::rmptc — Symbol `1` is Protected and cannot be removed.

### RemoveDiacritics

Removes common Latin diacritics from strings in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - RemoveDiacritics::argx — expected exactly `1` argument(s), got `2`

### Repeated

Represents the current string-pattern repetition head for one or more matches.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: bounded forms accept n, {n}, and {min, max}, where counts are non-negative and max may be Infinity. Zero bounds are valid and can produce zero-width matches; malformed bounds emit Repeated::range and leave the consuming call inert.
- Messages:
  - Repeated::range — `1` is not a valid repetition bound specification; expected n, {n}, or {min, max} with 0 <= min <= max, where max may be Infinity.

### RepeatedNull

Represents the current string-pattern repetition head for zero or more matches.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: bounded forms accept n, {n}, and {min, max}, where counts are non-negative and max may be Infinity. Zero bounds are valid and can produce zero-width matches; malformed bounds emit RepeatedNull::range and leave the consuming call inert.
- Messages:
  - RepeatedNull::range — `1` is not a valid repetition bound specification; expected n, {n}, or {min, max} with 0 <= min <= max, where max may be Infinity.

### RepeatedTiming

Evaluates an expression over a fixed sample set and returns {elapsed, value}, using the minimum monotonic elapsed time.

- Attributes: `HoldFirst`, `SequenceHold`, `Protected`
- Capability: `Clock`
- Messages:
  - RepeatedTiming::argx — expected exactly `1` argument(s), got `2`

### Replace

Applies rules at the current first-pass Replace levelspec subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Replace::argform — Reports an invalid rule list, levelspec, or Heads option.
  - Replace::argtu — expected between `1` and `2` argument(s), got `3`
  - Replace::argx — `1` called with `2` arguments; 1 argument is expected.

### ReplaceAll

Applies rules everywhere in an expression.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: traverses held positions. RuleDelayed right-hand sides are binding-substituted and inserted unevaluated. During parent reconstruction, a changed argument whose whole replacement is Sequence splices unless the parent has SequenceHold or HoldAllComplete; an unchanged literal Sequence stays nested, and a top-level replacement has no parent to splice into.
- Messages:
  - ReplaceAll::argform — expected a rule or list of rules as the second argument
  - ReplaceAll::argx — expected exactly `1` argument(s), got `2`

### ReplaceAllUnheld

Applies rules throughout the parts of an expression that would evaluate.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - ReplaceAllUnheld::argform — expected a rule or list of rules as the second argument
  - ReplaceAllUnheld::argx — expected exactly `1` argument(s), got `2`

### ReplaceAt

Applies rules at exact structural positions, with the current operator-form and exact-position first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: RuleDelayed right-hand sides are binding-substituted and inserted unevaluated; only subsequent evaluation of the fully rebuilt result evaluates an inserted right-hand side in an unheld position.
- Messages:
  - ReplaceAt::argform — Reports an invalid rule list or exact part specification.
  - ReplaceAt::argtu — expected between `1` and `2` argument(s), got `3`

### ReplaceList

Returns the current first-pass root replacement results across a rule list, with optional max-result limiting.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - ReplaceList::argform — Reports an invalid rule list or maximum-result limit.
  - ReplaceList::argtu — expected between `1` and `2` argument(s), got `3`

### ReplacePart

Replaces structural parts via immediate or delayed rules over exact, Span, and pattern-based position specifications, with Heads-aware matching.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: RuleDelayed right-hand sides are binding-substituted and inserted unevaluated; only subsequent evaluation of the fully rebuilt result evaluates an inserted right-hand side in an unheld position. Span positions reuse Part's span selection semantics; structurally valid missing exact or Span positions are ignored, while malformed Span shapes fail closed.
- Messages:
  - ReplacePart::argform — expected a Heads -> True/False option as the third argument
  - ReplacePart::argtu — expected between `1` and `2` argument(s), got `3`
  - ReplacePart::span — `1` is not a valid Span specification for ReplacePart; expected a valid Part Span with integer, All, or UpTo endpoints and a nonzero integer step.

### ReplaceRepeated

Repeatedly applies rules until an expression reaches a fixed point.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - ReplaceRepeated::argform — Reports an invalid rule list or MaxIterations option.
  - ReplaceRepeated::argx — expected exactly `1` argument(s), got `2`

### ReseedRandom

Reseeds the deterministic session PRNG from host entropy. Requires Capability::Random; denied under sandboxed().

- Attributes: `Protected`
- Capability: `Random`
- Limits: reseeds the session PRNG from host entropy; requires Capability::Random and is denied under sandboxed().
- Messages:
  - ReseedRandom::argx — expected exactly `1` argument(s), got `2`
  - ReseedRandom::entropy — could not read entropy: `1`

### ResolveContextAliases

Represents the current Names option symbol for context-alias resolution.

- Attributes: `Protected`
- Capability: `Pure`

### Rest

Returns an expression with its first structural element removed.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Rest::argx — expected exactly `1` argument(s), got `2`
  - Rest::norest — Cannot take `1` of expression `2` with length zero.
  - Rest::normal — Nonatomic expression expected at position 1 in `1`.

### RestoreSealedReferences

Re-links sealed symbol references while evaluating trusted snapshot restore forms.

- Attributes: `HoldAllComplete`, `Protected`
- Capability: `Pure`
- Messages:
  - RestoreSealedReferences::argform — expected a unit context or list of unit contexts and HoldComplete[form]
  - RestoreSealedReferences::mode — only valid inside a snapshot restore
  - RestoreSealedReferences::unit — snapshot restore could not find a named current sealed unit

### Return

Exits the current function-style evaluation with an optional result.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Return::argtu — expected between `1` and `2` argument(s), got `3`

### Reverse

Reverses normal-expression arguments at positive levels or at every level with All.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Reverse::argtu — expected between `1` and `2` argument(s), got `3`
  - Reverse::ilsmp — All, a positive machine-sized integer, or a list of positive machine-sized integers expected at position 2 of `1`.
  - Reverse::normal — Nonatomic expression expected at position 1 in `1`.

### ReverseApplied

Represents a wrapper that reverses current call arguments before application in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - ReverseApplied::argform — expected a non-negative integer as the second argument
  - ReverseApplied::argtu — expected between `1` and `2` argument(s), got `3`

### ReverseSort

Sorts top-level elements using structural ordering or a binary predicate, then reverses the result.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - ReverseSort::argx — expected exactly `1` argument(s), got `2`
  - ReverseSort::normal — Nonatomic expression expected at position 1 in `1`.

### ReverseSortBy

Sorts top-level elements by an evaluated key, then reverses the result.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - ReverseSortBy::argx — expected exactly `1` argument(s), got `2`
  - ReverseSortBy::normal — Nonatomic expression expected at position 1 in `1`.

### Riffle

Inserts one separator between top-level elements of a normal expression, with optional interval or position specs.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: supports one separator with optional positive interval or {imin, imax, di} insertion-position specs; list-separator forms are outside the current contract.
- Messages:
  - Riffle::argtu — expected between `1` and `2` argument(s), got `3`
  - Riffle::argx — `1` called with `2` arguments; 1 argument is expected.
  - Riffle::normal — Nonatomic expression expected at position 1 in `1`.
  - Riffle::seqs — Position 3 of `1` must be a positive machine-sized integer or {imin, imax, di} with nonzero integer step.

### RightComposition

Represents a right composition of functions in the current first-pass subset.

- Attributes: `Flat`, `OneIdentity`, `Protected`
- Capability: `Pure`

### Round

Rounds exact numbers to the nearest integer or exact multiple in the current snapshot-helper subset.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`
- Messages:
  - Round::argtu — expected between `1` and `2` argument(s), got `3`

### Row

Renders a list of expressions as a single text string in the current harness subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Row::argtu — expected between `1` and `2` argument(s), got `3`

### Rule

Represents an immediate rewrite rule.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Rule::argx — expected exactly `1` argument(s), got `2`

### RuleDelayed

Represents a delayed rewrite rule.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - RuleDelayed::argx — expected exactly `1` argument(s), got `2`

### RumProfileReset

Internal diagnostic hook: resets rum runtime profile counters when profiling is enabled.

- Attributes: `Protected`
- Capability: `Diagnostic`
- Messages:
  - RumProfileReset::argx — expected exactly `1` argument(s), got `2`

### RumProfileSnapshot

Internal diagnostic hook: prints rum runtime profile counters when profiling is enabled.

- Attributes: `Protected`
- Capability: `Diagnostic`
- Messages:
  - RumProfileSnapshot::argtu — expected between `1` and `2` argument(s), got `3`

### RumTokens

Tokenizes rum source text into total, trivia-preserving lexer spans.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - RumTokens::tok — Reports a tokenizer failure.

### SameAs

Represents a unary comparison operator equivalent to SameQ[#, bound] when applied.

- Attributes: `Protected`
- Capability: `Pure`

### SameQ

Tests whether all arguments are exactly identical after evaluation.

- Attributes: `Protected`
- Capability: `Pure`

### Save

Appends a textual first-pass serialization of selected symbols to a file.

- Attributes: `HoldRest`, `Protected`
- Capability: `FileWrite`
- Messages:
  - Save::argform — Reports an invalid path, object specification, context filter, or option list.
  - Save::argtu — expected between `1` and `2` argument(s), got `3`
  - Save::io — could not write `1`: `2`
  - Save::optx — Unknown option `1` in `2`.
  - Save::sealedref — Cannot save a definition containing a sealed symbol reference.

### Scan

Evaluates a function across selected levels for side effects in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Scan::argform — expected a standard levelspec or Heads -> True/False as the third argument
  - Scan::argtu — expected between `1` and `2` argument(s), got `3`

### SchemaErrors

Returns path-keyed validation errors for a value checked against a symbolic schema.

- Attributes: `HoldRest`, `Protected`
- Capability: `Pure`
- Messages:
  - SchemaErrors::schema — Reports a malformed schema or conformance failure.
  - SchemaErrors::argx — `1` called with `2` arguments; 1 argument is expected.

### SeedRandom

Reseeds the deterministic session PRNG from an exact integer.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: uses xoshiro256** with splitmix64 seed expansion; exact integer seeds fold magnitude bytes through splitmix64, with a sign tag for negative seeds.
- Messages:
  - SeedRandom::argx — expected exactly `1` argument(s), got `2`

### Select

Selects elements satisfying a criterion in the current sequence subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Select::argform — expected a sequence expression or a non-negative integer or Infinity selection limit
  - Select::argtu — expected between `1` and `2` argument(s), got `3`

### SelectFirst

Returns the first element satisfying a criterion in the current sequence subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - SelectFirst::argtu — expected between `1` and `2` argument(s), got `3`

### SelectPositions

Returns one-element 1-based position paths for top-level List elements whose criterion evaluates to True, matching Position's path convention, with an optional first-n limit.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - SelectPositions::argform — expected a non-negative integer selection limit
  - SelectPositions::argtu — expected between `1` and `2` argument(s), got `3`

### Sequence

Splices a sequence of arguments into a call.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: during rule right-hand-side substitution, literal Sequence calls are preserved for subsequent evaluation, while a Sequence value bound directly to a pattern variable splices structurally at its argument site.

### SequenceAlignment

Builds a simple global alignment summary for strings or lists in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - SequenceAlignment::argx — expected exactly `1` argument(s), got `2`

### SequenceFold

Folds a sequence with a sliding history in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - SequenceFold::argform — expected a non-negative integer arity as the fourth argument
  - SequenceFold::argtu — expected between `1` and `2` argument(s), got `3`

### SequenceFoldList

Returns the running history of a sequence fold in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - SequenceFoldList::argform — expected a non-negative integer arity as the fourth argument
  - SequenceFoldList::argtu — expected between `1` and `2` argument(s), got `3`

### Set

Assigns an immediate value to a symbol or pattern.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Set::argform — Reports an invalid assignment target or assigned metadata value.
  - Set::argx — expected exactly `1` argument(s), got `2`
  - Set::kval — The value for the key `1` does not exist.
  - Set::pkspec1 — The expression `1` cannot be used as a part specification.
  - Set::shape — Lists `1` and `2` are not the same shape.
  - Set::write — Tag `1` in `2` is Protected.
  - Set::wrsym — Symbol `1` is Protected.

### SetAttributes

Adds attributes to one or more symbols.

- Attributes: `HoldFirst`, `Protected`
- Capability: `Pure`
- Messages:
  - SetAttributes::argform — Reports an invalid target or attribute specification.
  - SetAttributes::argx — expected exactly `1` argument(s), got `2`
  - SetAttributes::wrsym — Symbol `1` is Protected.

### SetDelayed

Assigns a delayed definition evaluated on use.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - SetDelayed::argx — expected exactly `1` argument(s), got `2`
  - SetDelayed::wrsym — Symbol `1` is Protected.

### SetOptions

Updates known default options for a symbol.

- Attributes: `HoldFirst`, `Protected`
- Capability: `Pure`
- Messages:
  - SetOptions::argform — Reports an invalid target or option-rule specification.
  - SetOptions::argtu — expected between `1` and `2` argument(s), got `3`
  - SetOptions::optnf — `1` is not a known option for `2`.
  - SetOptions::wrsym — Symbol `1` is Protected.

### Sha1

Computes a SHA-1 digest over a string's UTF-8 bytes or a ByteArray, returning lowercase hex by default.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: intended only for interoperability and checksums; it is not collision-resistant.
- Messages:
  - Sha1::argtu — expected between `1` and `2` argument(s), got `3`
  - Sha1::bytearray — invalid ByteArray payload
  - Sha1::format — unknown digest output format; use "Hex" (default) or "Bytes"

### Sha256

Computes a SHA-256 digest over a string's UTF-8 bytes or a ByteArray, returning lowercase hex by default.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Sha256::argtu — expected between `1` and `2` argument(s), got `3`
  - Sha256::bytearray — invalid ByteArray payload
  - Sha256::format — unknown digest output format; use "Hex" (default) or "Bytes"

### Sha512

Computes a SHA-512 digest over a string's UTF-8 bytes or a ByteArray, returning lowercase hex by default.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Sha512::argtu — expected between `1` and `2` argument(s), got `3`
  - Sha512::bytearray — invalid ByteArray payload
  - Sha512::format — unknown digest output format; use "Hex" (default) or "Bytes"

### ShellQuote

Single-quotes a string for POSIX-style shell input, escaping embedded single quotes.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - ShellQuote::argx — expected exactly `1` argument(s), got `2`

### ShellSplit

Splits a POSIX-style shell word string with whitespace, quotes, and backslash escapes, returning $Failed on malformed quoting.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: implements only whitespace splitting, single and double quotes, and backslash escapes outside single quotes; expansion, globbing, command substitution, and short-option parsing are outside this text utility.
- Messages:
  - ShellSplit::argx — expected exactly `1` argument(s), got `2`
  - ShellSplit::syntax — Reports invalid input syntax.

### Short

Returns the current canonical-text shortened display subset for an expression.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Short::argform — expected the optional second argument to be a positive integer

### Shortest

Represents the current string-pattern wrapper that prefers the shortest match.

- Attributes: `Protected`
- Capability: `Pure`

### Sign

Returns the sign of exact integers and rationals.

- Attributes: `Listable`, `NumericFunction`, `Protected`
- Capability: `Pure`

### Sort

Sorts top-level elements using the current structural ordering subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Sort::argtu — expected between `1` and `2` argument(s), got `3`
  - Sort::normal — Nonatomic expression expected at position 1 in `1`.

### SortBy

Sorts top-level elements by an evaluated key or a lexicographic tuple from a list of key functions, optionally ordering keys with a binary predicate and preserving input order for ties.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - SortBy::argtu — expected between `1` and `2` argument(s), got `3`
  - SortBy::normal — Nonatomic expression expected at position 1 in `1`.

### Sow

Collects a value for enclosing Reap in the current first-pass tag-and-group subset.

- Attributes: `Protected`
- Capability: `Pure`

### SpellingCorrection

Represents the current Names option symbol for spelling-correction control.

- Attributes: `Protected`
- Capability: `Pure`

### Stack

Returns the current evaluation stack in the first-pass held-expression subset.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - Stack::argx — expected exactly `1` argument(s), got `2`

### StandardForm

Represents the current formatting symbol accepted by the current ToExpression and ToString subsets.

- Attributes: `Protected`
- Capability: `Pure`

### StartOfLine

Represents the current string-pattern anchor for the start of a string or a position after LF.

- Attributes: `Protected`
- Capability: `Pure`

### StartOfString

Represents the current string-pattern anchor for the start of a string.

- Attributes: `Protected`
- Capability: `Pure`

### StirlingS1

Computes signed Stirling numbers of the first kind.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`
- Messages:
  - StirlingS1::argx — expected exactly `1` argument(s), got `2`

### StirlingS2

Computes Stirling numbers of the second kind.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`
- Messages:
  - StirlingS2::argx — expected exactly `1` argument(s), got `2`

### StringApply

Applies a list-oriented function to string code points and reassembles string output in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - StringApply::argtu — expected between `1` and `2` argument(s), got `3`

### StringCases

Extracts supported string matches or immediate, delayed, and right-hand-side-guarded rule outputs.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - StringCases::argtu — expected between `1` and `2` argument(s), got `3`
  - StringCases::argx — `1` called with `2` arguments; 1 argument is expected.

### StringContainsQ

Tests whether a string or list of strings contains an exact substring in the current first-pass string subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - StringContainsQ::argtu — expected between `1` and `2` argument(s), got `3`
  - StringContainsQ::argx — `1` called with `2` arguments; 1 argument is expected.

### StringCount

Counts exact or current string-pattern matches, including the current IgnoreCase and Overlaps option subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - StringCount::argtu — expected between `1` and `2` argument(s), got `3`

### StringDedent

Removes the longest common leading whitespace from the non-blank lines of a string.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: computes the common leading whitespace from non-blank lines; blank and whitespace-only lines do not determine the prefix.
- Messages:
  - StringDedent::argx — expected exactly `1` argument(s), got `2`

### StringDelete

Deletes exact or current string-pattern matches from strings in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - StringDelete::argtu — expected between `1` and `2` argument(s), got `3`
  - StringDelete::argx — `1` called with `2` arguments; 1 argument is expected.

### StringDrop

Drops a substring using integer counts or inclusive 1-based ranges.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - StringDrop::argx — expected exactly `1` argument(s), got `2`

### StringEndsQ

Tests whether a string or list of strings ends with an exact substring in the current first-pass string subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - StringEndsQ::argtu — expected between `1` and `2` argument(s), got `3`
  - StringEndsQ::argx — `1` called with `2` arguments; 1 argument is expected.

### StringExpandTabs

Expands tabs in a string to the next fixed-width column stop, defaulting to width 8.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - StringExpandTabs::argform — tab width must be a positive integer
  - StringExpandTabs::argtu — expected between `1` and `2` argument(s), got `3`

### StringExpression

Represents the current string-pattern sequence head produced by `~~` syntax.

- Attributes: `Protected`
- Capability: `Pure`

### StringForm

Formats a template string eagerly, replacing `` and `n` placeholders with ToString-rendered arguments.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: returns a String eagerly rather than a lazy display wrapper; a backslash before a backtick emits a literal backtick, and missing or malformed placeholders emit StringForm::slot and leave the call inert.
- Messages:
  - StringForm::argtu — expected between `1` and `2` argument(s), got `3`
  - StringForm::slot — Reports a missing or invalid template slot.

### StringFreeQ

Tests whether a string is free of exact or current string-pattern matches.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - StringFreeQ::argtu — expected between `1` and `2` argument(s), got `3`
  - StringFreeQ::argx — `1` called with `2` arguments; 1 argument is expected.

### StringIndent

Prefixes each non-blank line of a string with a string prefix or an exact number of spaces. With "BlankLines" -> True, blank and whitespace-only lines are prefixed too; the default False leaves them unchanged.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: does not prefix empty or whitespace-only lines.
- Messages:
  - StringIndent::argtu — expected between `1` and `2` argument(s), got `3`

### StringJoin

Concatenates strings or a list of strings into a single string.

- Attributes: `Protected`
- Capability: `Pure`

### StringLength

Returns the Unicode code-point length of a string.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`
- Messages:
  - StringLength::argx — expected exactly `1` argument(s), got `2`

### StringLines

Splits a string on LF or CRLF newlines, preserving interior empty lines.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: splits only on LF and CRLF; one trailing newline is discarded while interior empty lines are preserved.
- Messages:
  - StringLines::argx — expected exactly `1` argument(s), got `2`

### StringMatchQ

Tests whether a whole string matches an exact or current string-pattern spec.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - StringMatchQ::argtu — expected between `1` and `2` argument(s), got `3`
  - StringMatchQ::argx — `1` called with `2` arguments; 1 argument is expected.

### StringPadLeft

Pads strings on the left to a target width using a string pad or a non-empty list of non-empty string pads; list pads cycle from the first emitted pad character and the final pad piece is truncated to the target character width.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - StringPadLeft::argtu — expected between `1` and `2` argument(s), got `3`

### StringPadRight

Pads strings on the right to a target width using a string pad or a non-empty list of non-empty string pads; list pads cycle from the first emitted pad character and the final pad piece is truncated to the target character width. Values at or above width are unchanged.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - StringPadRight::argtu — expected between `1` and `2` argument(s), got `3`

### StringPart

Extracts individual characters by 1-based positions.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - StringPart::argx — expected exactly `1` argument(s), got `2`

### StringPosition

Returns 1-based inclusive positions of exact substring matches, including overlaps, in the current first-pass string subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - StringPosition::argtu — expected between `1` and `2` argument(s), got `3`

### StringQ

Tests whether an expression is a string atom.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - StringQ::argx — expected exactly `1` argument(s), got `2`

### StringRepeat

Repeats a string a non-negative number of times.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - StringRepeat::argx — expected exactly `1` argument(s), got `2`

### StringReplace

Replaces supported exact and pattern string matches using immediate or delayed rules, including right-hand-side guards.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - StringReplace::argtu — expected between `1` and `2` argument(s), got `3`
  - StringReplace::argx — `1` called with `2` arguments; 1 argument is expected.

### StringReverse

Reverses the Unicode code-point characters in a string.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`
- Messages:
  - StringReverse::argx — expected exactly `1` argument(s), got `2`

### StringRiffle

Riffles a flat string list with a string separator or {left, separator, right} wrapper, and riffles nested string rows with row and item separators.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - StringRiffle::argtu — expected between `1` and `2` argument(s), got `3`

### StringSplit

Splits strings using whitespace or exact delimiters in the current first-pass string subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - StringSplit::argtu — expected between `1` and `2` argument(s), got `3`

### StringStartsQ

Tests whether a string or list of strings starts with an exact substring in the current first-pass string subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - StringStartsQ::argtu — expected between `1` and `2` argument(s), got `3`
  - StringStartsQ::argx — `1` called with `2` arguments; 1 argument is expected.

### StringTake

Extracts a substring using integer counts or inclusive 1-based ranges.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - StringTake::argx — expected exactly `1` argument(s), got `2`

### StringTakeDrop

Returns both the taken and dropped substrings using the current StringTake/StringDrop spec subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - StringTakeDrop::argx — expected exactly `1` argument(s), got `2`

### StringTemplate

Builds a simple TemplateObject wrapper from a template string in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - StringTemplate::argx — expected exactly `1` argument(s), got `2`

### StringToByteArray

Converts strings to ByteArray objects using UTF8/UTF-8 encoding.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - StringToByteArray::argtu — expected between `1` and `2` argument(s), got `3`

### StringTrim

Trims whitespace or exact boundary substrings in the current first-pass string subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - StringTrim::argtu — expected between `1` and `2` argument(s), got `3`

### StringWrap

Greedily word-wraps a string to a positive character width, separating output lines with LF. With "PreserveNewlines" -> True, wraps each LF-delimited line independently and preserves blank or whitespace-only lines verbatim; the default False collapses all whitespace as word separators.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: normalizes input whitespace runs to single spaces while wrapping; it does not preserve original spacing inside wrapped text.
- Messages:
  - StringWrap::argform — width must be a positive integer
  - StringWrap::argtu — expected between `1` and `2` argument(s), got `3`

### SubValues

Lists stored subvalue definitions for a symbol.

- Attributes: `HoldFirst`, `Protected`
- Capability: `Pure`
- Messages:
  - SubValues::argform — expected a symbol or symbol-name string argument
  - SubValues::argx — expected exactly `1` argument(s), got `2`
  - SubValues::setval — Reports an invalid SubValues assignment.

### Subfactorial

Computes derangement numbers for non-negative integer indices.

- Attributes: `Listable`, `NumericFunction`, `Protected`, `ReadProtected`
- Capability: `Pure`
- Messages:
  - Subfactorial::argx — expected exactly `1` argument(s), got `2`

### SubsetMap

Replaces selected exact parts using a mapped subset result in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - SubsetMap::argform — expected an exact part specification or list of exact part specifications as the third argument
  - SubsetMap::argtu — expected between `1` and `2` argument(s), got `3`
  - SubsetMap::path — Duplicate subset path `1` is not allowed.

### SubsetQ

Tests subset inclusion for Lists, optionally using SameTest.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - SubsetQ::argx — expected exactly `1` argument(s), got `2`

### Subsets

Returns subsets of a normal expression in rum's size-major order for scalar, All, Infinity, and range size specs.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Subsets::argtu — expected between `1` and `2` argument(s), got `3`
  - Subsets::nninfseq — Position 2 of `1` must be All, Infinity, nmax, {nmin}, {nmin, nmax} or {nmin, nmax, dn}, where nmin is a non-negative integer, nmax is non-negative integer or Infinity and dn is a nonzero integer.
  - Subsets::normal — Nonatomic expression expected at position 1 in `1`.

### Subtract

Subtracts one exact number from another.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Subtract::argx — expected exactly `1` argument(s), got `2`

### SubtractFrom

Subtracts an evaluated decrement from a held symbol or exact Part target and stores the result.

- Attributes: `HoldFirst`, `SubValuesHoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - SubtractFrom::argform — expected an exact Part target as the first argument
  - SubtractFrom::argx — expected exactly `1` argument(s), got `2`
  - SubtractFrom::rvalue — `1` is not a variable with a value, so its value cannot be changed.
  - SubtractFrom::wrsym — Symbol `1` is Protected.

### Success

Represents a success object with associated details in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`

### Sum

Adds expression values over concrete iterator specifications.

- Attributes: `HoldAll`, `Protected`, `ReadProtected`
- Capability: `Pure`
- Messages:
  - Sum::argform — expected a supported repetition count or iterator specification
  - Sum::argx — expected at least 2 argument(s), got `1`

### Switch

Matches one evaluated expression against held form/value pairs.

- Attributes: `HoldRest`, `Protected`
- Capability: `Pure`
- Messages:
  - Switch::argform — expected an expression followed by one or more form/value pairs

### Symbol

Resolves a symbol from a string name using the current context path.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Symbol::argx — expected exactly `1` argument(s), got `2`
  - Symbol::string — String expected at position 1 in `1`.

### SymbolName

Returns the short name of a symbol without its context.

- Attributes: `Protected`
- Capability: `Pure`

### SyntaxInformation

Lists the current syntax-information metadata rules for a symbol.

- Attributes: `HoldFirst`, `Protected`
- Capability: `Pure`
- Messages:
  - SyntaxInformation::argform — expected a symbol or symbol-name string argument
  - SyntaxInformation::argx — expected exactly `1` argument(s), got `2`

### SyntaxQ

Tests whether a string parses successfully in the current syntax subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - SyntaxQ::argx — expected exactly `1` argument(s), got `2`

### Table

Generates lists using the current first-pass iterator subset.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - Table::argform — expected a supported repetition count or iterator specification
  - Table::argx — expected at least 2 argument(s), got `1`

### TagSet

Stores an immediate tagged definition, including upvalues when the tag appears in an argument position.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - TagSet::argform — expected a symbol or symbol-name string tag as the first argument
  - TagSet::argx — expected exactly `1` argument(s), got `2`
  - TagSet::tagnf — tag symbol does not occur in the assignment lhs
  - TagSet::tagpos — tag symbol must appear within the outer two levels of the assignment lhs
  - TagSet::wrsym — Symbol `1` is Protected.

### TagSetDelayed

Stores a delayed tagged definition, including upvalues when the tag appears in an argument position.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - TagSetDelayed::argform — expected a symbol or symbol-name string tag as the first argument
  - TagSetDelayed::argx — expected exactly `1` argument(s), got `2`
  - TagSetDelayed::tagnf — tag symbol does not occur in the assignment lhs
  - TagSetDelayed::tagpos — tag symbol must appear within the outer two levels of the assignment lhs
  - TagSetDelayed::wrsym — Symbol `1` is Protected.

### TagUnset

Removes a tagged definition using an explicit tag and lhs.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - TagUnset::argform — expected a symbol or symbol-name string tag as the first argument
  - TagUnset::argx — expected exactly `1` argument(s), got `2`
  - TagUnset::tagnf — tag symbol does not occur in the assignment lhs
  - TagUnset::tagpos — tag symbol must appear within the outer two levels of the assignment lhs
  - TagUnset::wrsym — Symbol `1` is Protected.

### Take

Takes structural elements using the current standard sequence-spec subset, including UpTo bounds.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Take::argx — expected exactly `1` argument(s), got `2`
  - Take::take — Cannot apply the requested sequence specification in `1`.

### TakeDrop

Returns both the taken and dropped structural elements using the current standard sequence-spec subset, including UpTo bounds.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - TakeDrop::argx — expected exactly `1` argument(s), got `2`
  - TakeDrop::iseq — Invalid sequence specification `1` for an expression of length `2`.
  - TakeDrop::take — Cannot apply the requested sequence specification in `1`.

### TakeLargestBy

Takes top-level elements with largest evaluated keys in descending key order, preserving tie order.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - TakeLargestBy::argx — expected exactly `1` argument(s), got `2`
  - TakeLargestBy::normal — Nonatomic expression expected at position 1 in `1`.

### TakeList

Splits a normal expression into sequential chunks by non-negative counts.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - TakeList::argx — expected exactly `1` argument(s), got `2`
  - TakeList::normal — Nonatomic expression expected at position 1 in `1`.

### TakeSmallestBy

Takes top-level elements with smallest evaluated keys in ascending key order, preserving tie order.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - TakeSmallestBy::argx — expected exactly `1` argument(s), got `2`
  - TakeSmallestBy::normal — Nonatomic expression expected at position 1 in `1`.

### TakeWhile

Takes the longest prefix satisfying a criterion in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - TakeWhile::argtu — expected between `1` and `2` argument(s), got `3`

### Tally

Tallies structurally same top-level elements, or elements equivalent under a custom binary test, with the first element as representative.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: the custom binary-test form performs pairwise O(n^2) representative comparisons and uses the first element of each class as representative.
- Messages:
  - Tally::argx — expected exactly `1` argument(s), got `2`
  - Tally::normal — Nonatomic expression expected at position 1 in `1`.

### TemplateApply

Applies a simple named or positional string template in the current first-pass subset; unfilled slots leave the call inert by default, while "Strict" -> True reports them and returns $Failed.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - TemplateApply::argtu — expected between `1` and `2` argument(s), got `3`
  - TemplateApply::slot — Reports a missing or invalid template slot.

### TemplateObject

Represents the current simple template object wrapper returned by StringTemplate.

- Attributes: `Protected`
- Capability: `Pure`

### TerminatedEvaluation

Represents a terminated evaluation object in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`

### TestCreate

Builds a first-pass symbolic test specification.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`

### TestReport

Runs one or more first-pass symbolic tests and returns an association summary.

- Attributes: `Protected`
- Capability: `Clock`
- Messages:
  - TestReport::argform — expected the optional property argument to be All, Properties, a property string or symbol, or a list of property strings or symbols

### TextString

Converts expressions to the current canonical text subset, leaving raw strings unquoted.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - TextString::argx — expected exactly `1` argument(s), got `2`

### TextWords

Splits text into alphanumeric word runs in the current first-pass subset.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`
- Messages:
  - TextWords::argx — expected exactly `1` argument(s), got `2`

### Thread

Threads an expression over arguments that share a common head.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Thread::argform — expected the third argument to be a standard sequence specification such as All, None, n, -n, {n}, {m, n}, or {m, n, s}
  - Thread::argx — expected 1, 2, or 3 argument(s), got `1`
  - Thread::tdlen — Reports incompatible argument lengths while threading an expression.

### Through

Threads arguments through a functional head in the current first-pass subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Through::argtu — expected between `1` and `2` argument(s), got `3`

### Throw

Propagates a nonlocal value, optionally tagged for Catch matching.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Throw::argtu — expected between `1` and `2` argument(s), got `3`
  - Throw::nocatch — Uncaught `1` returned to top level.

### ThrowException

Raises a registered typed exception with an optional Association payload.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - ThrowException::argtu — expected between `1` and `2` argument(s), got `3`
  - ThrowException::argform — exception type must be a string and payload must be an Association
  - ThrowException::unreg — The exception type is not registered.
  - ThrowException::nocatch — Uncaught exception returned to top level.

### TimeConstrained

Evaluates an expression under the current cooperative exact-numeric timeout subset.

- Attributes: `HoldAll`, `Protected`
- Capability: `Clock`
- Messages:
  - TimeConstrained::argform — expected the timeout argument to be a non-negative exact number in the current first-pass subset
  - TimeConstrained::argtu — expected between `1` and `2` argument(s), got `3`

### TimeObjectQ

Tests whether an expression has a TimeObject head in the current structural subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - TimeObjectQ::argx — expected exactly `1` argument(s), got `2`

### Times

Multiplies exact numbers and canonically combines multiplicative expressions.

- Attributes: `Listable`, `Orderless`, `Flat`, `OneIdentity`, `Protected`
- Capability: `Pure`

### TimesBy

Multiplies a held symbol or exact Part target by an evaluated factor and stores the result.

- Attributes: `HoldFirst`, `SubValuesHoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - TimesBy::argform — expected an exact Part target as the first argument
  - TimesBy::argx — expected exactly `1` argument(s), got `2`
  - TimesBy::rvalue — `1` is not a variable with a value, so its value cannot be changed.
  - TimesBy::wrsym — Symbol `1` is Protected.

### Timing

Evaluates an expression and returns {elapsed, value}, with elapsed exact rational seconds measured by a monotonic clock.

- Attributes: `HoldAll`, `Protected`
- Capability: `Clock`
- Messages:
  - Timing::argx — expected exactly `1` argument(s), got `2`

### ToBase64

Encodes a string's UTF-8 bytes or a ByteArray as standard RFC 4648 base64 with padding.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - ToBase64::argx — expected exactly `1` argument(s), got `2`
  - ToBase64::bytearray — invalid ByteArray payload

### ToCharacterCode

Returns Unicode scalar values for strings, or UTF-8 byte values when given the "UTF8" or "UTF-8" encoding argument.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`
- Messages:
  - ToCharacterCode::argtu — expected between `1` and `2` argument(s), got `3`

### ToCsv

Renders Lists of String rows or String-keyed Association rows as CSV text.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: renders only String fields from list rows or String-keyed Association rows; Association rows imply a header row and must share the same keys. Delimiter must be one non-quote, non-newline character.
- Messages:
  - ToCsv::csv — Reports a CSV encoding or argument failure.

### ToExpression

Parses and evaluates input strings using the current InputForm/StandardForm-compatible subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - ToExpression::argtu — expected between `1` and `2` argument(s), got `3`

### ToHexString

Encodes a string's UTF-8 bytes or a ByteArray as lowercase hexadecimal text.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - ToHexString::argx — expected exactly `1` argument(s), got `2`
  - ToHexString::bytearray — invalid ByteArray payload

### ToIni

Renders an Association of section Associations as rum's INI dialect.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: renders only Association-of-Associations with String section names, keys, and values; unrepresentable names, multiline values, whitespace-trimming value loss, and an empty default section fail closed.
- Messages:
  - ToIni::ini — Reports an INI encoding or argument failure.

### ToJson

Renders the JSON-representable subset of rum values as a JSON text string.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - ToJson::depth — JSON nesting exceeds the fixed depth limit of 256.
  - ToJson::json — Reports an invalid JSON encoding argument or value.

### ToLowerCase

Converts a string to lowercase using the current Unicode-aware subset.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`
- Messages:
  - ToLowerCase::argx — expected exactly `1` argument(s), got `2`

### ToString

Renders an evaluated expression into the current canonical kernel text form.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - ToString::argtu — expected between `1` and `2` argument(s), got `3`

### ToToml

Renders the TOML-representable subset of rum Associations as TOML text.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: renders only top-level Associations with String keys, exact integers, terminating rationals, booleans, strings, Lists, nested Associations, and Lists of Associations; nested Associations use sub-table headers unless inline form is required to preserve same-level key order. It never synthesizes TOML datetime syntax, so date Lists and timestamps encode as arrays or numbers.
- Messages:
  - ToToml::depth — TOML nesting exceeds the fixed depth limit of 256.
  - ToToml::toml — Reports a TOML encoding or argument failure.

### ToUpperCase

Converts a string or list of strings to uppercase using the current Unicode-aware subset.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`
- Messages:
  - ToUpperCase::argx — expected exactly `1` argument(s), got `2`

### ToXml

Renders a well-formed XmlElement tree as XML text.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: renders only well-formed XmlElement[tag_String, attrs_Association, children_List] trees with String attributes and String/XmlElement children; names use the documented ASCII XML-name subset, compact output is default, and Indent pretty-prints only element-only content.
- Messages:
  - ToXml::depth — XML nesting exceeds the fixed depth limit of 256.
  - ToXml::xml — Reports an XML encoding or argument failure.

### ToYaml

Renders the YAML-representable subset of rum values as YAML text.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: renders only Associations with String keys, Lists, exact integers, terminating rationals, True, False, Null, and Strings; safe plain strings use the documented Norway-safe rule, other strings are double-quoted or literal blocks, and unsupported YAML surface is never synthesized.
- Messages:
  - ToYaml::depth — YAML nesting exceeds the fixed depth limit of 256.
  - ToYaml::yaml — Reports a YAML encoding or argument failure.

### Together

Combines rational-function terms over a common polynomial denominator.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Together::argx — expected exactly `1` argument(s), got `2`

### TokenizeWith

Scans a string with string-pattern rules and returns total 1-based token spans.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - TokenizeWith::tok — Reports a tokenizer failure.

### Total

Totals list elements through the current exact Plus semantics.

- Attributes: `Protected`
- Capability: `Pure`

### Trace

Evaluates an expression and returns a first-pass structured trace of current evaluation events.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - Trace::argform — Reports an invalid level specification or Heads option for Trace.
  - Trace::argtu — expected between `1` and `2` argument(s), got `3`

### TraceDetail

Evaluates an expression and returns a lossless structured trace whose event associations preserve rule and definition details.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - TraceDetail::argform — Reports an invalid level specification or Heads option for TraceDetail.
  - TraceDetail::argtu — expected between `1` and `2` argument(s), got `3`

### TracePrint

Evaluates an expression, emits its current first-pass structured trace through the print stream, and returns the result.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - TracePrint::argform — Reports an invalid level specification or Heads option for TracePrint.
  - TracePrint::argtu — expected between `1` and `2` argument(s), got `3`

### TraditionalForm

Represents the current formatting symbol accepted by the current ToString subset.

- Attributes: `Protected`
- Capability: `Pure`

### Transpose

Transposes rectangular plain-list matrices and tensors using identity, first-two-swap, or rank-matching positive permutations.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Transpose::argtu — expected between `1` and `2` argument(s), got `3`
  - Transpose::nmtx — The first two levels of `1` cannot be transposed.
  - Transpose::permspec — Invalid permutation specification found at position 2 in `1`.

### TrueQ

Tests whether an expression evaluates exactly to True.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - TrueQ::argx — expected exactly `1` argument(s), got `2`

### Tuples

Returns repeated tuples or cartesian products for normal expressions in the current subset.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Tuples::argtu — expected between `1` and `2` argument(s), got `3`
  - Tuples::ilsmn — Single or list of non-negative machine-sized integers expected at position 2 of `1`.
  - Tuples::normal — Nonatomic expression expected at position `1` in `2`.

### Uncompress

Recovers values from the current project-local compressed string envelope; malformed envelopes stay inert by default, while "Strict" -> True reports them and returns $Failed.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Uncompress::argtu — expected between `1` and `2` argument(s), got `3`
  - Uncompress::env — Uncompress found a malformed or corrupt compressed string envelope.

### Unequal

Tests inequality in the current exact subset for numbers, strings, and obviously identical expressions.

- Attributes: `Protected`
- Capability: `Pure`

### UnequalTo

Represents a unary comparison operator equivalent to Unequal[#, bound] when applied.

- Attributes: `Protected`
- Capability: `Pure`

### Unevaluated

Temporarily leaves an argument unevaluated as a direct function argument.

- Attributes: `HoldAllComplete`, `Protected`
- Capability: `Pure`

### Union

Returns sorted unique elements across same-head normal expressions; SameTest keeps the first representative in structural scan order.

- Attributes: `Flat`, `OneIdentity`, `Protected`, `ReadProtected`
- Capability: `Pure`
- Messages:
  - Union::heads — Heads `1` and `2` at positions `3` and 1 are expected to be the same.
  - Union::normal — Nonatomic expression expected at position 1 in `1`.

### Unique

Creates fresh symbols using prefix, list, and attribute forms.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Unique::argform — expected an attribute symbol or list of attribute symbols as the second argument
  - Unique::argtu — expected between `1` and `2` argument(s), got `3`
  - Unique::usym — `1` is not a symbol or a valid symbol name.

### Unprotect

Removes the Protected attribute from one or more symbols.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - Unprotect::argform — expected one or more symbols, symbol-name strings, or lists of them

### UnsameQ

Tests whether no two arguments are exactly identical after evaluation.

- Attributes: `Protected`
- Capability: `Pure`

### Unset

Removes a specific stored definition or message template.

- Attributes: `HoldFirst`, `Protected`
- Capability: `Pure`
- Messages:
  - Unset::argform — expected a symbol, definition lhs, message name, or tagged definition target
  - Unset::argx — expected exactly `1` argument(s), got `2`
  - Unset::wrsym — Symbol `1` is Protected.

### Until

Evaluates a body until its test yields True.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - Until::argtu — expected between `1` and `2` argument(s), got `3`

### UpSet

Stores an immediate upvalue definition on one or more tag symbols inferred from the lhs arguments.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - UpSet::argx — expected exactly `1` argument(s), got `2`
  - UpSet::nosym — No valid tag symbol occurs in the assignment lhs `1`.
  - UpSet::wrsym — Symbol `1` is Protected.

### UpSetDelayed

Stores a delayed upvalue definition on one or more tag symbols inferred from the lhs arguments.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - UpSetDelayed::argx — expected exactly `1` argument(s), got `2`
  - UpSetDelayed::nosym — No valid tag symbol occurs in the assignment lhs `1`.
  - UpSetDelayed::wrsym — Symbol `1` is Protected.

### UpTo

Represents a non-negative upper bound in supported Span selector forms.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - UpTo::argx — UpTo called with `1` arguments; 1 argument is expected.
  - UpTo::innf — Non-negative integer or Infinity expected at position 1 in `1`.

### UpValues

Lists stored tagged upvalue definitions for a symbol.

- Attributes: `HoldFirst`, `Protected`
- Capability: `Pure`
- Messages:
  - UpValues::argform — expected a symbol or symbol-name string argument
  - UpValues::argx — expected exactly `1` argument(s), got `2`
  - UpValues::setval — Reports an invalid UpValues assignment.

### UpperCaseQ

Tests whether every character in a string is uppercase.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - UpperCaseQ::argx — expected exactly `1` argument(s), got `2`

### UppercaseLetters

Evaluates to the uppercase ASCII letter character-class string.

- Attributes: `Protected`
- Capability: `Pure`

### UrlDecode

Decodes percent escapes in a UTF-8 URL string, returning $Failed for invalid escapes or invalid UTF-8; with "FormEncoding" -> True, + decodes to a space.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - UrlDecode::argtu — expected between `1` and `2` argument(s), got `3`
  - UrlDecode::escape — Reports an invalid escape sequence.

### UrlEncode

Percent-encodes a string using UTF-8 bytes and the RFC 3986 unreserved character set; with "FormEncoding" -> True, spaces are encoded as + for form query data.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - UrlEncode::argtu — expected between `1` and `2` argument(s), got `3`

### UrlParse

Parses scheme://host[:port][/path][?query][#fragment] into an association of string fields; "DefaultScheme" can supply a scheme for schemeless inputs, and "DecodeQuery" decodes query text using form encoding.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: without "DefaultScheme", accepts only a strict scheme://host[:port][/path][?query][#fragment] shape with an ASCII scheme and decimal port; with "DecodeQuery" -> True, query text is decoded as application/x-www-form-urlencoded data.
- Messages:
  - UrlParse::argtu — expected between `1` and `2` argument(s), got `3`
  - UrlParse::escape — Reports an invalid escape sequence.
  - UrlParse::syntax — Reports invalid input syntax.

### UuidQ

Tests whether a string has version-4 UUID format and variant bits.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - UuidQ::argx — expected exactly `1` argument(s), got `2`

### ValueQ

Tests whether an expression has a value by trial evaluation without committing side effects.

- Attributes: `HoldFirst`, `Protected`
- Capability: `Pure`
- Messages:
  - ValueQ::argx — expected exactly `1` argument(s), got `2`

### Values

Returns association values in structural order.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - Values::argt — Values called with `1` arguments; 1 or 2 arguments are expected.
  - Values::invrl — The argument `1` is not a valid Association or a list of rules.

### VectorQ

Tests whether an expression is a one-dimensional List array, optionally requiring every leaf to satisfy a test.

- Attributes: `Protected`
- Capability: `Pure`
- Messages:
  - VectorQ::argtu — expected between `1` and `2` argument(s), got `3`

### Which

Evaluates conditions in order and returns the first matching branch.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - Which::argform — expected one or more condition/result pairs

### While

Evaluates a body while its test yields True.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - While::argtu — expected between `1` and `2` argument(s), got `3`

### Whitespace

Represents the current string-pattern atom for one or more whitespace characters.

- Attributes: `Protected`
- Capability: `Pure`

### WhitespaceCharacter

Represents the current string-pattern atom for one whitespace character.

- Attributes: `Protected`
- Capability: `Pure`

### WhitespaceCharacters

Evaluates to the ASCII whitespace character-class string.

- Attributes: `Protected`
- Capability: `Pure`

### With

Lexically substitutes local constants into a body, with each additional declaration list forming a nested stage.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - With::argform — expected a list of symbol assignments or rules as the first argument
  - With::argx — expected exactly `1` argument(s), got `2`

### WithCleanup

Evaluates cleanup code even when the main expression exits abnormally in the current first-pass subset.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`
- Messages:
  - WithCleanup::argtu — expected between `1` and `2` argument(s), got `3`

### WordBoundary

Represents the current string-pattern anchor at a transition between word and non-word text.

- Attributes: `Protected`
- Capability: `Pure`

### WordCharacter

Represents the current string-pattern atom for one alphabetic, ASCII digit, or underscore character.

- Attributes: `Protected`
- Capability: `Pure`

### XmlElement

Represents an inert XML element as XmlElement[tag, attrs, children].

- Attributes: `Protected`
- Capability: `Pure`

### Xor

Evaluates exclusive-or over exact boolean arguments and reduces duplicate symbolic terms in the current subset.

- Attributes: `Orderless`, `Flat`, `OneIdentity`, `Protected`
- Capability: `Pure`

