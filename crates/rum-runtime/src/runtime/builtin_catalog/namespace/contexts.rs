use super::*;

pub(in super::super) const NAMESPACE_CONTEXT_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary(
        "FileNameJoin",
        "Joins path components into a platform path string.",
        builtin_file_name_join,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a list of string path components",
        },
        ARGX_EXACT,
    ])
    .protected(),
    BuiltinSpec::ordinary(
        "DirectoryName",
        "Returns a parent directory path string for a file name.",
        builtin_directory_name,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a string file name",
        },
        ARGTU,
    ])
    .protected(),
    BuiltinSpec::ordinary(
        "ParentDirectory",
        "Returns the parent directory for an existing directory path.",
        builtin_parent_directory,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a string directory name",
        },
        ARGX_EXACT,
    ])
    .protected(),
    BuiltinSpec::ordinary(
        "FileNameTake",
        "Takes file name components from a path string.",
        builtin_file_name_take,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a string file name",
        },
        ARGTU,
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "Context",
        "Returns the context of the current session or of a symbol.",
        HoldAttribute::HoldFirst,
        builtin_context,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a symbol or string argument",
        },
        BuiltinMessageSpec {
            tag: "argx",
            template: "expected 0 or 1 argument(s), got `1`",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary(
        "Contexts",
        "Lists known contexts in the current session.",
        builtin_contexts,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a string context-name pattern",
        },
        BuiltinMessageSpec {
            tag: "argx",
            template: "expected 0 or 1 argument(s), got `1`",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary(
        "Names",
        "Lists symbol names matching a current first-pass name pattern subset.",
        builtin_names,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a string pattern, RegularExpression, StringExpression, Alternatives, or a list of them",
        },
        BuiltinMessageSpec {
            tag: "optval",
            template: "Reports an unsupported Names option value.",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary_with_attributes(
        "SymbolName",
        "Returns the short name of a symbol without its context.",
        AttributeSet::from_attribute(Attribute::Protected),
        Session::eval_symbol_name,
        MemoryGrowthContract::NonAmplifying,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "NameQ",
        "Tests whether a string is the exact name of an existing symbol.",
        AttributeSet::from_attribute(Attribute::Protected),
        Session::eval_name_q,
        MemoryGrowthContract::NonAmplifying,
    ),
    BuiltinSpec::new(
        "ResolveContextAliases",
        "Represents the current Names option symbol for context-alias resolution.",
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::new(
        "SpellingCorrection",
        "Represents the current Names option symbol for spelling-correction control.",
        MemoryGrowthContract::Preflight,
    ).protected(),
    BuiltinSpec::new("File", "Represents an explicit file specification.", MemoryGrowthContract::NonAmplifying).protected(),
    BuiltinSpec::ordinary(
        "FindFile",
        "Finds the full path of a file or package using the current search path.",
        builtin_find_file,
        MemoryGrowthContract::Delegated,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a string or File[\"...\"] load target",
        },
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "notfound",
            template: "Reports that the requested file could not be located.",
        },
    ])
    .protected()
    .with_capability(Capability::FileRead),
    BuiltinSpec::ordinary(
        "FileNames",
        "Lists sorted file paths whose names match a folk wildcard pattern.",
        builtin_file_names,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        ARGTU,
    ])
    .protected()
    .with_capability(Capability::FileRead),
    BuiltinSpec::ordinary(
        "Get",
        "Reads a file or package and evaluates its forms in order. In a resumable evaluation request, host input suspends the load and resumes the current form without replaying earlier forms; the load shares the enclosing resource budget.",
        builtin_get,
        MemoryGrowthContract::Delegated,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a string or File[\"...\"] load target",
        },
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "io",
            template: "could not read `1`: `2`",
        },
        BuiltinMessageSpec {
            tag: "load",
            template: "Reports a loader failure while reading or evaluating a file.",
        },
        BuiltinMessageSpec {
            tag: "notfound",
            template: "Reports that the requested file could not be located.",
        },
        BuiltinMessageSpec {
            tag: "syntax",
            template: "Reports a syntax error while evaluating a loaded file.",
        },
    ])
    .protected()
    .with_capability(Capability::FileRead),
    BuiltinSpec::ordinary(
        "Needs",
        "Loads a package only if its context has not already been loaded. In a resumable evaluation request, host input suspends the load; resumption finishes the file before registering the context and returning Null.",
        builtin_needs,
        MemoryGrowthContract::Delegated,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "Reports an invalid context name or explicit load target.",
        },
        BuiltinMessageSpec {
            tag: "argx",
            template: "expected `1` argument(s), got `2`",
        },
        BuiltinMessageSpec {
            tag: "load",
            template: "context `1` could not be loaded",
        },
    ])
    .protected()
    .with_capability(Capability::FileRead),
    BuiltinSpec::ordinary(
        "Begin",
        "Switches into a context and prepends it to the context path.",
        builtin_begin,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a string context name",
        },
        ARGX_EXACT,
    ])
    .protected(),
    BuiltinSpec::ordinary(
        "End",
        "Restores the previous context after a matching Begin.",
        builtin_end,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "context",
            template: "no matching Begin[] frame is active",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary(
        "BeginPackage",
        "Starts a package context with a package-oriented context path before loading its dependencies. Dependency input can suspend a resumable evaluation request; resumption finishes that dependency and the remaining dependencies. A failed dependency or propagated Exit restores this package frame without undoing earlier successful dependency definitions.",
        builtin_begin_package,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "Reports an invalid package context or dependency list.",
        },
        BuiltinMessageSpec {
            tag: "argx",
            template: "expected `1` argument(s), got `2`",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary(
        "EndPackage",
        "Ends a package context and leaves it on the restored context path.",
        builtin_end_package,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "context",
            template: "no matching BeginPackage[] frame is active",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary(
        "DeclarePackage",
        "Declares stub names for a package context and prepends that context to the search path. Reading a stub autoloads its package; input during that load can suspend a resumable evaluation request, which resumes into the symbol read after loading.",
        builtin_declare_package,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "Reports an invalid package context or symbol-name list.",
        },
        ARGX_EXACT,
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "Save",
        "Appends a textual first-pass serialization of selected symbols to a file.",
        HoldAttribute::HoldRest,
        builtin_save,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "Reports an invalid path, object specification, context filter, or option list.",
        },
        ARGTU,
        BuiltinMessageSpec {
            tag: "io",
            template: "could not write `1`: `2`",
        },
        BuiltinMessageSpec {
            tag: "optx",
            template: "Unknown option `1` in `2`.",
        },
        BuiltinMessageSpec {
            tag: "sealedref",
            template: "Cannot save a definition containing a sealed symbol reference.",
        },
    ])
    .protected()
    .with_capability(Capability::FileWrite),
    BuiltinSpec::ordinary_with_hold(
        "DumpSave",
        "Writes a textual first-pass dump of selected symbols to a file and returns the saved symbol list.",
        HoldAttribute::HoldRest,
        builtin_dump_save,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "Reports an invalid path, object specification, context filter, or option list.",
        },
        ARGTU,
        BuiltinMessageSpec {
            tag: "io",
            template: "could not write `1`: `2`",
        },
        BuiltinMessageSpec {
            tag: "optval",
            template: "Unsupported value `1` for option CompressionLevel in DumpSave; only 1 is currently supported.",
        },
        BuiltinMessageSpec {
            tag: "optx",
            template: "Unknown option `1` in `2`.",
        },
        BuiltinMessageSpec {
            tag: "sealedref",
            template: "Cannot save a definition containing a sealed symbol reference.",
        },
    ])
    .protected()
    .with_capability(Capability::FileWrite),
    BuiltinSpec::ordinary_with_hold(
        "Once",
        "Evaluates an expression at most once per session-local cache key in the current first-pass subset.",
        HoldAttribute::HoldAll,
        builtin_once,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "Reports an invalid option specification.",
        },
        ARGTU,
        BuiltinMessageSpec {
            tag: "pers",
            template: "Unsupported value `1` for option `2` in Once; supported values are ExpirationDate -> None and PersistenceTime -> None or 0.",
        },
    ])
    .protected(),
];
