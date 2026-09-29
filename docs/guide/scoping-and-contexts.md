# Scoping And Contexts

`rum` has lexical scoping, dynamic rebinding, and context-qualified names.

Scoping controls symbol meaning inside an expression.

Contexts control which symbol an unqualified name resolves to.

The important scoping builtins are
[`With`](../reference/builtins.md#with),
[`Module`](../reference/builtins.md#module), and
[`Block`](../reference/builtins.md#block).

## With

`With` performs lexical substitution.

It is useful when a value should be inserted into held or delayed code before
that code is evaluated later.

The binding list is held and interpreted by the scoping builtin.

## Block

`Block` dynamically rebinds the current symbol-state slice for the duration of
the body.

It is useful when code should temporarily see another value for an existing
symbol.

The implementation localizes the symbol-state categories covered by `Block`'s
documented contract. New categories are not implicitly localized; they require
an explicit contract and regression tests.

```text
$ cargo run -q -p rum-cli -- --eval 'x = 10; {With[{x = 1}, x + 1], Block[{x = 1}, x + 1], x}'
List[2, 2, 10]
```

Both scoped bodies see `x` as `1`.

After the scoped forms finish, the outer ownvalue of `x` is still `10`.

## Module

`Module` creates fresh local symbols.

Those locals can be captured by closures.

```text
$ cargo run -q -p rum-cli -- --eval 'make[] := Module[{x = 0}, Function[{}, x = x + 1]]; c = make[]; {c[], c[]}'
List[1, 2]
```

The closure returned by `make[]` keeps access to its module local.

Calling it twice increments the same captured local.

`$ModuleNumber` is not part of the current public surface.

## Context Names

A context name ends in a backtick:

`` Global` ``, `` System` ``, and `` Demo` `` are context names.

A fully qualified symbol includes a context and symbol name:

`` Demo`value `` is a fully qualified symbol.

Unqualified symbols are resolved through `$Context` and `$ContextPath`.

The default session starts in `` Global` `` and has `` System` `` and
`` Global` `` on the context path.

## Top-Level Resolution

Names in one input are parsed and lowered before that input evaluates.

That means changing context inside a compound `--eval` input does not retro-
actively change names that were already resolved earlier in the same input.

When you need context changes to affect later unqualified names, use later REPL
inputs, a package file, or explicit fully qualified names.

## Begin And End

`` Begin["Ctx`"] `` changes the current context.

`End[]` restores the previous context and returns the ended context name.

These forms are useful for implementation sections inside package files.

## Packages

`` BeginPackage["Ctx`"] `` starts a package context.

`EndPackage[]` closes it.

`Get[path]` reads and evaluates a source file.

`Needs[context, path]` loads a package context if needed from an explicit path.

Both `Get` and `Needs` have `FileRead` capability, so a sandboxed host can
deny them at dispatch.

From a resumable evaluation request, input in a file loaded by `Get` or
`Needs`, a `BeginPackage` dependency, or a `DeclarePackage` stub suspends the
same request. Resuming continues the current form, then the remaining forms
and the caller's completion work. Earlier forms and their output are not
replayed; nested loads retain the correct `$InputFileName`. Each form is
lowered when reached, using the context established by earlier forms.

Guest loads share the enclosing request's timeout and memory budget across
forms and input responses. An input request is not a load failure.
Cancellation rolls back active file transactions without running their
remaining forms or registering a successful package load. Ordinary load
errors restore the file's definitions, contexts, packages, and messages;
printed output remains. `BeginPackage` restores its own context frame on a
failed dependency or propagated `Exit`, preserving definitions from earlier
successful dependencies. Public host script/file batch APIs retain their
separate, non-resumable contract described in the embedding chapter.

```text
$ mkdir -p /tmp/rum-guide-pkg
$ cat > /tmp/rum-guide-pkg/GuidePkg.rum <<'EOF'
BeginPackage["GuidePkg`"]
answer = 42
EndPackage[]
EOF
$ cargo run -q -p rum-cli -- --eval 'Needs["GuidePkg`", "/tmp/rum-guide-pkg/GuidePkg.rum"]; GuidePkg`answer'
42
```

The explicit path avoids relying on package search paths.

The final expression uses the fully qualified symbol `` GuidePkg`answer ``.

## Sealed Packages

`DefinePackage` registers a source unit with three symbol tiers.

Declared `Public` symbols live in the package context. Declared
`PackageScope` symbols live in a `PackageScope\`` subcontext. Both are
ordinary name-resolvable symbols — honest conventions, reachable by fully
qualified name.

Everything else the unit binds is private, and private is a real boundary:
each private symbol is minted with a fresh internal identity, the unit body
is rewritten to reference it, and the name is never entered in the
resolution tables. Code outside the unit that spells the name — directly,
via `Symbol`, or via `ToExpression` — gets a distinct ordinary symbol and
cannot reach the sealed binding.

```text
$ cargo run -q -p rum-cli -- --eval 'DefinePackage["Counter`", Public[{Bump, Peek}], PackageScope[{limit}], limit = 10; total = 0; Bump[n_] := If[total + n > limit, $Failed, total = total + n]; Peek[] := total]; {Counter`Bump[3], Counter`Bump[4], Counter`Peek[], Counter`PackageScope`limit}'
List[3, 7, 7, 10]
```

Registration returns the package context string. A failing body form
rolls the sealed installs back, reports `DefinePackage::failed`, and
returns `$Failed`.

Assigning to the spelled private name creates the caller's own symbol; the
sealed state is untouched:

```text
$ cargo run -q -p rum-cli -- --eval 'DefinePackage["Counter`", Public[{Bump, Peek}], PackageScope[{limit}], limit = 10; total = 0; Bump[n_] := If[total + n > limit, $Failed, total = total + n]; Peek[] := total]; Counter`Bump[3]; Counter`Private`total = 999; {Counter`Peek[], Counter`Private`total}'
List[3, 999]
```

Introspection of sealed privates is capability-gated. A session whose
policy allows `Diagnostic` — the trusted CLI default — sees them in
`Names`, `Contexts`, `Definition`, and `Information` (the owner/debug
view); a sandboxed policy without `Diagnostic` sees no trace of them, and
`Save`/`DumpSave` refuse to write sealed definitions into files.

```text
$ cargo run -q -p rum-cli -- --eval 'DefinePackage["Counter`", Public[{Bump, Peek}], PackageScope[{limit}], limit = 10; total = 0; Bump[n_] := If[total + n > limit, $Failed, total = total + n]; Peek[] := total]; Names["Counter`Private`*"]'
List["Counter`Private`total"]
```

Snapshots persist a sealed package as its registration form: restore
re-runs `DefinePackage`, minting fresh identities, so the seal survives
every round trip. Private state mutated after registration resets to its
as-registered values; public and package-scope state persists as usual.
Sealed values that leaked into ordinary variables re-link to the fresh
identities on restore, and a snapshot fails loudly rather than write a
sealed reference it cannot re-link.

## Context Variables

`$Context` is the current context name.

`$ContextPath` is the list of contexts used for resolving unqualified names.

`$Packages` records loaded package contexts.

`$Path` records package and file search paths.

Hosts can also configure the initial current context, context path, and package
search paths through `HostConfig`.

## Persistence

`DumpSave` writes a textual session image for selected symbols.

The format is source text, not a binary image.

`Get` restores it by reading and evaluating that source.

```text
$ mkdir -p /tmp/rum-guide-dump
$ cargo run -q -p rum-cli -- --eval 'saved[x_] := x + 3; DumpSave["/tmp/rum-guide-dump/state.rum", saved]'
List[saved]
$ sed -n '1,80p' /tmp/rum-guide-dump/state.rum
SetDelayed[saved[Pattern[x, Blank[]]], Plus[x, 3]]
$ cargo run -q -p rum-cli -- --eval 'Get["/tmp/rum-guide-dump/state.rum"]; saved[4]'
7
```

The file records a canonical definition.

Loading it into a fresh session restores that definition.

## Host Snapshots

`rum-host` also provides `snapshot_to_bytes()` and `restore_from_bytes()`.

Those APIs avoid the filesystem and use a UTF-8 source image of user
definitions and attributes for defined symbols.

They do not capture every session detail: attribute-only symbols, `$`
variables, options, messages, the gensym counter, and unrelated context state
are outside the current snapshot image.

Sealed packages are captured as their `DefinePackage` registration forms
(restore re-registers them; see Sealed Packages above), so `snapshot_to_bytes`
is fallible: it refuses, with a descriptive failure, a session whose state
holds a sealed reference that no current unit can re-link.

## Practical Guidance

Use fully qualified names in package examples and saved-session tests.

Use `With` for lexical substitution.

Use `Module` for fresh locals and closures.

Use `Block` for temporary dynamic rebinding.

Use `DefinePackage` when a package needs a genuinely private tier rather
than a naming convention.

Use `DumpSave` or host snapshots for persistence, depending on whether the
caller wants filesystem I/O or an in-memory byte buffer.
