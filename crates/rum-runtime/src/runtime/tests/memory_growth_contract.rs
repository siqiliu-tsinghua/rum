use super::*;

macro_rules! core_builtin_memory_growth_inventory {
    ($($name:literal => $contract:ident,)*) => {
        const CORE_BUILTIN_MEMORY_GROWTH_INVENTORY: &[(&str, MemoryGrowthContract)] = &[
            $(($name, MemoryGrowthContract::$contract),)*
        ];
    };
}

core_builtin_memory_growth_inventory! {
    "$Aborted" => NonAmplifying,
    "$Canceled" => NonAmplifying,
    "$Context" => NonAmplifying,
    "$ContextPath" => Incremental,
    "$Failed" => NonAmplifying,
    "$InputFileName" => NonAmplifying,
    "$MessageList" => Incremental,
    "$Packages" => Incremental,
    "$Path" => Incremental,
    "AbsoluteTime" => NonAmplifying,
    "AbsoluteTiming" => Delegated,
    "AddTo" => Incremental,
    "Alternatives" => NonAmplifying,
    "AppendTo" => Incremental,
    "ApplyTo" => Incremental,
    "ArgumentsOptions" => Incremental,
    "Attributes" => Preflight,
    "Automatic" => NonAmplifying,
    "Begin" => NonAmplifying,
    "BeginPackage" => NonAmplifying,
    "Blank" => NonAmplifying,
    "BlankNullSequence" => NonAmplifying,
    "BlankSequence" => NonAmplifying,
    "Block" => Delegated,
    "ByteArray" => Preflight,
    "ByteCount" => NonAmplifying,
    "Check" => Delegated,
    "CheckArguments" => NonAmplifying,
    "Clear" => NonAmplifying,
    "ClearAll" => NonAmplifying,
    "ClearAttributes" => NonAmplifying,
    "Context" => NonAmplifying,
    "Contexts" => Incremental,
    "Count" => NonAmplifying,
    "DeclarePackage" => NonAmplifying,
    "Decrement" => Incremental,
    "DefaultValues" => Incremental,
    "DefinePackage" => Incremental,
    "Definition" => Incremental,
    "Dialog" => Delegated,
    "DiffObject" => NonAmplifying,
    "DigitCharacter" => NonAmplifying,
    "DirectoryName" => NonAmplifying,
    "DivideBy" => Incremental,
    "DownValues" => Incremental,
    "DumpSave" => Incremental,
    "Echo" => Incremental,
    "EchoFunction" => NonAmplifying,
    "EchoLabel" => NonAmplifying,
    "End" => NonAmplifying,
    "EndOfLine" => NonAmplifying,
    "EndOfString" => NonAmplifying,
    "EndPackage" => NonAmplifying,
    "Evaluate" => Delegated,
    "EvaluationData" => Incremental,
    "Except" => NonAmplifying,
    "Extract" => Incremental,
    "File" => NonAmplifying,
    "FileNameJoin" => NonAmplifying,
    "FileNameTake" => NonAmplifying,
    "FileNames" => Incremental,
    "FindFile" => Delegated,
    "FreeQ" => NonAmplifying,
    "Function" => Delegated,
    "General" => NonAmplifying,
    "Get" => Delegated,
    "Hold" => NonAmplifying,
    "HoldAll" => NonAmplifying,
    "HoldAllComplete" => NonAmplifying,
    "HoldComplete" => NonAmplifying,
    "HoldFirst" => NonAmplifying,
    "HoldForm" => NonAmplifying,
    "HoldPattern" => NonAmplifying,
    "HoldRest" => NonAmplifying,
    "IgnoreCase" => NonAmplifying,
    "Increment" => Incremental,
    "Information" => Incremental,
    "Input" => Delegated,
    "InputForm" => NonAmplifying,
    "InputString" => Delegated,
    "LetterCharacter" => NonAmplifying,
    "Level" => Incremental,
    "Longest" => NonAmplifying,
    "MaxMemoryUsed" => NonAmplifying,
    "MemberQ" => NonAmplifying,
    "MemoryConstrained" => Delegated,
    "MemoryInUse" => NonAmplifying,
    "Message" => Incremental,
    "MessageDefinitions" => Incremental,
    "MessageName" => NonAmplifying,
    "MessageTags" => Incremental,
    "Messages" => Incremental,
    "MissingQ" => NonAmplifying,
    "Module" => Delegated,
    "Monitor" => Delegated,
    "NameQ" => NonAmplifying,
    "Names" => Incremental,
    "Needs" => Delegated,
    "NormalizeRules" => NonAmplifying,
    "Nothing" => NonAmplifying,
    "NumberString" => NonAmplifying,
    "Off" => Incremental,
    "On" => Incremental,
    "Once" => Incremental,
    "OptionValue" => Incremental,
    "Options" => Incremental,
    "OutputForm" => NonAmplifying,
    "Overlaps" => NonAmplifying,
    "OwnValues" => Incremental,
    "PackageScope" => Incremental,
    "ParentDirectory" => NonAmplifying,
    "Part" => Incremental,
    "Pattern" => NonAmplifying,
    "Pause" => NonAmplifying,
    "Position" => Incremental,
    "PreDecrement" => Incremental,
    "PreIncrement" => Incremental,
    "PrependTo" => Incremental,
    "Print" => Incremental,
    "Protect" => NonAmplifying,
    "Public" => Incremental,
    "PunctuationCharacter" => NonAmplifying,
    "Quiet" => Delegated,
    "Reap" => Incremental,
    "RegularExpression" => NonAmplifying,
    "ReleaseHold" => Delegated,
    "Remove" => NonAmplifying,
    "Repeated" => NonAmplifying,
    "RepeatedNull" => NonAmplifying,
    "RepeatedTiming" => Delegated,
    "ReplaceAt" => Incremental,
    "ReplacePart" => Incremental,
    "ResolveContextAliases" => NonAmplifying,
    "RestoreSealedReferences" => Delegated,
    "Rule" => Delegated,
    "RuleDelayed" => NonAmplifying,
    "RumProfileReset" => NonAmplifying,
    "RumProfileSnapshot" => Incremental,
    "Save" => Incremental,
    "Sequence" => NonAmplifying,
    "Set" => Incremental,
    "SetAttributes" => Incremental,
    "SetDelayed" => Incremental,
    "SetOptions" => Incremental,
    "Short" => Incremental,
    "Shortest" => NonAmplifying,
    "Sow" => Incremental,
    "SpellingCorrection" => Preflight,
    "Stack" => Incremental,
    "StandardForm" => NonAmplifying,
    "StartOfLine" => NonAmplifying,
    "StartOfString" => NonAmplifying,
    "StringExpression" => NonAmplifying,
    "SubValues" => Incremental,
    "SubtractFrom" => Incremental,
    "SymbolName" => NonAmplifying,
    "SyntaxInformation" => Incremental,
    "TagSet" => Incremental,
    "TagSetDelayed" => Incremental,
    "TagUnset" => NonAmplifying,
    "TemplateObject" => NonAmplifying,
    "TestCreate" => NonAmplifying,
    "TestReport" => Incremental,
    "Thread" => Delegated,
    "TimeConstrained" => Delegated,
    "TimesBy" => Incremental,
    "Timing" => Delegated,
    "Trace" => Incremental,
    "TraceDetail" => Incremental,
    "TracePrint" => Incremental,
    "TraditionalForm" => NonAmplifying,
    "Unevaluated" => NonAmplifying,
    "Unique" => Incremental,
    "Unprotect" => NonAmplifying,
    "Unset" => NonAmplifying,
    "UpSet" => Incremental,
    "UpSetDelayed" => Incremental,
    "UpTo" => NonAmplifying,
    "UpValues" => Incremental,
    "ValueQ" => Delegated,
    "Whitespace" => NonAmplifying,
    "WhitespaceCharacter" => NonAmplifying,
    "With" => Delegated,
    "WordBoundary" => NonAmplifying,
    "WordCharacter" => NonAmplifying,
}

#[test]
fn core_builtin_memory_growth_inventory_is_sorted_and_exhaustive() {
    assert!(
        CORE_BUILTIN_MEMORY_GROWTH_INVENTORY
            .windows(2)
            .all(|entries| entries[0].0 < entries[1].0),
        "the core builtin memory-growth inventory must stay sorted and unique"
    );

    let mut actual = core_builtin_specs()
        .iter()
        .map(|spec| (spec.name, spec.memory_growth))
        .collect::<Vec<_>>();
    actual.sort_by_key(|(name, _)| *name);

    assert_eq!(actual, CORE_BUILTIN_MEMORY_GROWTH_INVENTORY);
}

fn deliberately_amplifying_builtin(
    _session: &mut Session,
    _head: &Expr,
    _args: &SharedArgs,
) -> Result<Expr, EvalError> {
    Ok(Expr::string("x".repeat(4096)))
}

fn deliberately_projecting_builtin(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    session.enforce_active_memory_limit_for_projected_bytes(4096)?;
    deliberately_amplifying_builtin(session, head, args)
}

fn deliberately_projecting_then_suspending_builtin(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    session.enforce_active_memory_limit_for_projected_bytes(4096)?;
    session.eval_input_string(args)
}

#[test]
fn known_unprojected_preflight_builtin_allowlist_is_sorted_and_unique() {
    assert!(
        KNOWN_UNPROJECTED_PREFLIGHT_BUILTINS
            .windows(2)
            .all(|entries| entries[0] < entries[1]),
        "the known-unprojected Preflight builtin allowlist must stay sorted and unique"
    );
}

#[test]
#[should_panic(expected = "declared NonAmplifying")]
fn non_amplifying_contract_is_falsifiable_through_dispatch() {
    let mut session = Session::new();
    session.register_builtin(BuiltinSpec::ordinary(
        "DeliberatelyAmplifyingBuiltin",
        "Exercises the test-only memory-growth contract falsifier.",
        deliberately_amplifying_builtin,
        MemoryGrowthContract::NonAmplifying,
    ));

    let _ = session.eval_text("DeliberatelyAmplifyingBuiltin[]");
}

#[test]
#[should_panic(expected = "declared NonAmplifying")]
fn non_amplifying_contract_is_falsifiable_after_input_resume() {
    let mut session = Session::new();
    let input_string = core_builtin_specs()
        .iter()
        .copied()
        .find(|spec| spec.name == "InputString")
        .expect("InputString should be in the core catalog");
    session.register_builtin(BuiltinSpec {
        memory_growth: MemoryGrowthContract::NonAmplifying,
        ..input_string
    });

    session
        .eval_request("InputString[\"value: \"]")
        .expect_err("InputString should suspend for host input");
    let _ = session.resume_eval_request_with_input("x".repeat(4096));
}

#[test]
#[should_panic(expected = "declared Preflight")]
fn preflight_contract_is_falsifiable_through_dispatch() {
    let mut session = Session::new();
    session.register_builtin(BuiltinSpec::ordinary(
        "DeliberatelyUnprojectedPreflightBuiltin",
        "Exercises the test-only Preflight memory-growth contract falsifier.",
        deliberately_amplifying_builtin,
        MemoryGrowthContract::Preflight,
    ));

    let _ = session.eval_text("DeliberatelyUnprojectedPreflightBuiltin[]");
}

#[test]
#[should_panic(expected = "declared Preflight")]
fn preflight_contract_is_falsifiable_after_input_resume() {
    let mut session = Session::new();
    let input_string = core_builtin_specs()
        .iter()
        .copied()
        .find(|spec| spec.name == "InputString")
        .expect("InputString should be in the core catalog");
    session.register_builtin(BuiltinSpec {
        memory_growth: MemoryGrowthContract::Preflight,
        ..input_string
    });

    session
        .eval_request("InputString[\"value: \"]")
        .expect_err("InputString should suspend for host input");
    let _ = session.resume_eval_request_with_input("x".repeat(4096));
}

#[test]
fn preflight_contract_accepts_recorded_projection() {
    let mut session = Session::new();
    session.register_builtin(BuiltinSpec::ordinary(
        "DeliberatelyProjectedPreflightBuiltin",
        "Exercises the positive path of the test-only Preflight falsifier.",
        deliberately_projecting_builtin,
        MemoryGrowthContract::Preflight,
    ));

    let result = session
        .eval_text("DeliberatelyProjectedPreflightBuiltin[]")
        .expect("a recorded projection should discharge the Preflight postcondition");
    assert_eq!(result.value, Expr::string("x".repeat(4096)));
}

#[test]
fn preflight_contract_remembers_projection_across_input_resume() {
    let mut session = Session::new();
    session.register_builtin(BuiltinSpec::ordinary(
        "DeliberatelyProjectedSuspendingPreflightBuiltin",
        "Exercises projected Preflight validation across suspension.",
        deliberately_projecting_then_suspending_builtin,
        MemoryGrowthContract::Preflight,
    ));

    session
        .eval_request("DeliberatelyProjectedSuspendingPreflightBuiltin[\"value: \"]")
        .expect_err("the projected test builtin should suspend for host input");
    let response = session
        .resume_eval_request_with_input("x".repeat(4096))
        .expect("the pre-suspension projection should remain visible after resume");
    assert_eq!(response.value, Expr::string("x".repeat(4096)));
}
