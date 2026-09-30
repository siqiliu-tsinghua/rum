# Getting Started

This chapter gets from a checkout to running source code.

All commands are run from the repository root.

The workspace uses the Rust toolchain declared by the repository. The default
developer checks are:

- `cargo fmt --check`
- `cargo test -p rum-runtime`
- `cargo test --workspace`

For guide examples, the important binary is `rum-cli`.

The CLI can run a single expression, run a file, print syntax information, or
start the REPL.

## One Expression

Use `--eval` when you want one expression evaluated in a fresh session.

```text
$ cargo run -q -p rum-cli -- --eval 'Total[{1, 2, 1/2}]'
7/2
```

The output is the final expression after evaluation.

`Total` is a builtin. Its attributes and capability class are listed in the
[`Total`](../reference/builtins.md#total) reference entry.

## REPL

Running `cargo run -p rum-cli` with no arguments starts the prompt.

The REPL keeps one session alive, so definitions entered on one line are
available on later lines.

```text
$ cargo run -q -p rum-cli
rum> square[x_] := x * x
Null
rum> square[6]
36
rum> :quit
```

The first line stores a delayed downvalue for `square`.

The second line calls it.

`SetDelayed` returns `Null` when it stores the definition.

When an evaluation calls `InputString` or `Input`, the REPL displays the guest's
prompt and reads the next line as its answer. An empty prompt displays `input> `.
Output produced before the request appears before that prompt. The REPL resumes
the same evaluation, including nested file loads and repeated input requests.
The `:trace` command also resumes, retaining its trace renderer and all options.

```text
rum> answer = InputString["name: "]
name: Ada
"Ada"
rum> answer
"Ada"
```

Answers do not enter history or go through shell-command handling: even `:quit`
is literal text for `InputString`. Empty answers and whitespace are preserved;
only the line ending is removed. `Input` sends the answer to the host's source
parser without adding a REPL reprompt on a parse error. During an input wait,
the REPL uses the input request's prompt; `...> ` is for unfinished source.

EOF at an input prompt cancels the waiting evaluation, runs guest cleanup once,
and exits. On a Unix terminal, Ctrl-C while waiting for an answer cancels and
returns to `rum> `. Earlier definitions survive, and the abandoned remainder
does not run. Ctrl-C at the primary prompt or during an ordinary evaluation
keeps its existing behavior. Plain readers can also report an interrupted read
to cancel an answer.
An I/O error also retires the request before it is returned to the caller.
Cancellation is reported once; automatic abandonment retains the input-required
reason. A borrowed host session is reusable after the REPL's request ends.

The CLI's Unix terminal reader hides answers requested with `FieldMasked -> True`:
there is no character echo or placeholder, and answers never enter history or
reader diagnostics. Terminal settings are restored on normal return, EOF,
Ctrl-C, I/O error and panic unwinding. Masking protects entry only; guest code
can still print the answer afterwards. Plain buffered readers and platforms
without this terminal reader report `password input unavailable` and abandon
masked requests without reading an answer or falling back to visible entry.

## Script Files

Use `--file` to evaluate a source file.

The file is evaluated in one session, and the CLI prints the final result.

```text
$ cat > /tmp/rum-guide-start.rum <<'EOF'
square[x_] := x * x
square[6]
EOF
$ cargo run -q -p rum-cli -- --file /tmp/rum-guide-start.rum
36
```

A positional path works too:

`cargo run -q -p rum-cli -- /tmp/rum-guide-start.rum`

The guide examples use `--file` because it makes the command form explicit.

## Pipelines And Filter Mode

When stdin is not a terminal and no program argument is given, `rum-cli` reads
all of stdin as one script and prints the final result. It does not print the
REPL prompt.

```text
$ echo 'Total[{1,2,3}]' | cargo run -q -p rum-cli
6
```

Multiple input forms share one session, the same as a file.

```text
$ printf 'x=1\nx+1\n' | cargo run -q -p rum-cli
2
```

A positional `-` explicitly selects the same stdin-as-program mode.

```text
$ printf '2 + 3\n' | cargo run -q -p rum-cli -
5
```

When a program is supplied and stdin is piped, the piped bytes are available as
the string ``Global`$ScriptInput``.

```text
$ printf 'a\nb\nc\n' | cargo run -q -p rum-cli -- --eval 'Length[StringLines[$ScriptInput]]'
3
```

Arguments after `--` are available as ``Global`$ScriptArgs``.

```text
$ cargo run -q -p rum-cli -- --eval '$ScriptArgs' -- alpha beta
List["alpha", "beta"]
```

## First Definitions

`Set` stores the evaluated right-hand side immediately.

`SetDelayed` stores the right-hand side unevaluated and evaluates it each time
the definition fires.

Use `=` for stable values and `:=` when the right-hand side should be evaluated
at call time.

Rules have a similar split:

- `lhs -> rhs` creates `Rule[lhs, rhs]`
- `lhs :> rhs` creates `RuleDelayed[lhs, rhs]`

Definitions are session state. A new `--eval` command starts a new CLI session,
so definitions from a previous command are not present.

Use the REPL or a file when you want multiple inputs to share definitions.

## Getting Help

The generated reference is the compact builtin catalog:

- one entry per builtin
- the builtin's summary
- its attributes
- its capability class
- implementation limit notes where the catalog declares one

For example, start with:

- [`Set`](../reference/builtins.md#set)
- [`SetDelayed`](../reference/builtins.md#setdelayed)
- [`Rule`](../reference/builtins.md#rule)
- [`RuleDelayed`](../reference/builtins.md#ruledelayed)
- [`Function`](../reference/builtins.md#function)
- [`List`](../reference/builtins.md#list)

The reference is not a tutorial. Use it to check exact names and metadata while
you use this guide for the working model.

## Parse And Format Tools

The CLI also exposes parser and formatter helpers.

Use `--parse` when you need to see the parsed syntax tree and tokens.

Use `--format` when you need the formatter's rendering for one expression.

These helpers are diagnostics for source text. They do not replace evaluation
examples in this guide.

## Fresh Sessions

Each `cargo run -q -p rum-cli -- --eval ...` command constructs a new CLI
session, installs the core builtins, evaluates the expression, prints the
result, and exits.

That makes `--eval` examples deterministic and isolated.

It also means stateful examples should be written as one compound expression,
as multiple REPL inputs, or as a script file.

## Messages

Runtime messages are printed after the value in CLI transcripts.

Do not assume that a message means the top-level evaluation failed. Many
message-producing expressions still return a value.

The evaluation chapter shows `Message`, `Quiet`, and `Check`.

## File Extensions

The CLI accepts ordinary text files. The runtime does not inspect the file
extension, so any suffix evaluates the same way.

The examples use `.rum`, which is `rum`'s own source-file suffix. Because `rum`
has diverged from the Wolfram Language in documented ways, prefer `.rum` over
`.wl` for `rum` source: it names the file as `rum` code and avoids implying
Wolfram-Language compatibility that `rum` does not claim. A `.wl` file still
evaluates identically if you have one — the choice of suffix is a labelling
convention, not a runtime requirement.

`Get`, `Needs`, `Save`, and `DumpSave` also operate on text paths subject to the
host capability policy.

The scoping and resource chapters explain those boundaries.
