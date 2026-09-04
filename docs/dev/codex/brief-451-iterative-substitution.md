# brief-451 — iterative lexical and rule-binding substitution

## Goal

Substitution is the last fail-open family in the deep-rebuild class. Remove the
depth-proportional Rust recursion from the lexical engine
(`lexical_substitution.rs`, 3,385 levels) and the rule-binding engine
(`binding_substitution.rs`, 4,430 levels) so both complete at physical height
4,097 on a 2 MiB stack and stop failing by terminating the process. Every
documented scoping, shadowing, alpha-renaming, splicing and sharing property
stays exactly as it is.

This is the second half of brief-439's draft 6, scoped by measurement — see
"What the probes changed about the scope".

## What I measured — treat as given, do not re-run

Release `rum-cli` at `e69a98be`, `ulimit -s 2048`, 1 MiB evaluator budget. Each
probe builds a deep body and asserts the exact substituted value.

| probe | engine | last ok / first fail | mode |
|---|---|---|---|
| `With @@ {{q -> 7}, deepBody}` | lexical | **3,385 / 3,393** | **process kill** |
| `Module @@ {{q}, deepBody}` | lexical | **3,385 / 3,393** | **process kill** |
| `Function @@ {q, deepBody}` applied, named parameter | lexical | **3,385 / 3,393** | **process kill** |
| `Replace[7, RuleDelayed @@ {Pattern[x, Blank[]], deepTemplate}]` | rule binding | **4,430 / 4,438** | **process kill** |
| `Function @@ {deepBodyContainingSlot[1]}` applied | slot | 5,867 / 5,874 | in band |

Both target files contain **zero** occurrences of `poll_native_stack_limit` and
`evaluation_depth`, which is why they end the process rather than raising an
error. All three lexical consumers land on the same bracket, which is the
shared entry `substitute_lexical_bindings_changed`.

Do not re-measure any of this, and do not write a probe that drives either
recursion past its ceiling.

## What the probes changed about the scope

Two corrections to what the record said before this re-probe, both of which move
the scope:

1. **Slot substitution is not short and is out of scope.** The earlier
   "`Function` slot substitution 3,685" entry was measured with a probe that
   used a *named parameter*, which routes through the lexical engine, not
   `substitute_slots`. A true `Slot[1]` probe clears 4,097 and fails closed at
   5,867. `slot_substitution.rs` is therefore not touched by this brief.
2. **Rule-binding substitution is in scope after all.** It clears the ruled
   height, but by **1.08x** — and brackets on this path are not stable across
   unrelated work: the lexical bracket moved from 3,685 to 3,385, about 8%, over
   the fourteen commits since it was last measured, with nothing touching
   substitution. A margin thinner than the drift is not a margin, so it is
   repaired now rather than left to fall below the contract from an unrelated
   change.

## The recursion edges to remove

**`crates/rum-runtime/src/runtime/substitution/lexical_substitution.rs`** — one
mutual recursion group around the entry `substitute_lexical_bindings_changed`
(`:386`). The arms re-enter it at `:82`, `:83` (pattern call), `:107`, `:108`,
`:111` (assignment-like), `:136`, `:137`, `:141` (`Condition`), `:166`, `:169`
(`With`), `:194` (scoped), `:226`, `:229`, `:258` (`Function`), and `:294`
(generic); `substitute_arg_list_changed` (`:314`) is the fan-out and is called
at `:198`, `:231`, `:260` and `:295`.

**`crates/rum-runtime/src/runtime/substitution/binding_substitution.rs`** — two
groups: `substitute_bindings_changed` (`:51`), and
`substitute_rule_bindings_changed` (`:125`) with its arms
`substitute_rule_scoped_call_changed` (`:163`),
`substitute_rule_with_call_changed` (`:193`),
`substitute_rule_function_call_changed` (`:243`),
`substitute_rule_call_changed_with_arg_alphas` (`:280`),
`substitute_rule_with_binding_list_changed` (`:331`),
`substitute_rule_with_binding_item_changed` (`:399`) and `rebuild_rule_call`
(`:446`).

Both files are pure functions over `&Expr` — no `Session`, no suspension. Keep
them that way; if you think you need a suspension frame, stop and report.

## Semantics that must not move

- **The shadow set is per subtree, not per walk.** Every `_changed` function
  takes `shadowed: &BTreeSet<Symbol>` and several arms build a *different* set
  for one child than for its siblings — `rhs_shadowed` (`:109-111`),
  `condition_shadowed` (`:139-141`), `body_shadowed` (`:167-169`, `:196-198`,
  `:256-260`). An explicit stack must carry the shadow set **in the work item**.
  Sharing one set across siblings is the defect this conversion will produce if
  it is going to produce one, and it is silent: it changes which symbols a
  nested `With`/`Module`/`Function` shadows, not whether the walk terminates.
- **`None` means unchanged, and that is the sharing contract.** The `_changed`
  suffix is the convention: returning `None` lets the caller keep the original
  `Arc` instead of rebuilding. An iterative driver must still report "nothing
  below me changed" for an untouched subtree, or every substitution starts
  rebuilding trees it used to share.
- **`may_contain_binding_symbol` (`:36`) is the empty/absent-binding fast
  reject.** Keep it, and keep it *before* the descent.
- Preserve the special handling for `Pattern`, assignment-like constructs,
  `Condition`, nested `With`/`Module`/`Block`, and explicit `Function`
  parameter shadowing; preserve alpha-renames
  (`substitute_rule_call_changed_with_arg_alphas`); splice a `Sequence` only
  when it came from a directly bound pattern symbol.
- **`record_lexical_rebuild` (`:58`) and `record_rule_rhs_rebuild`
  (`binding_substitution.rs:40`) are off-by-default profiling.** Do not carry
  profiling state in a work item or a frame — brief-444 removed exactly that
  from the stability walk because 64 bytes of disabled state in a hot recursive
  frame is a hard-line violation. Derive it inside the enabled path.

## Deliverable

1. Both engines driven by explicit work stacks behind a depth-16 recursive
   front — the pattern `Level`, `Map`, `Replace`, `D`, `DeleteCases` and now
   `Take`/`ReplacePart` already use. A shallow `With[{a = 1}, a + 1]` must not
   pay for the deep machinery.
2. No caller changes. Both entries keep their current signatures; if a signature
   must change, that is a stop, not a scope extension.
3. Certificates, following the rule this line settled in briefs 439-442:
   - release-only `#[ignore]` deep certificates on a 2 MiB declared thread at
     `TARGET_PHYSICAL_HEIGHT`, one for a lexical consumer (`With` over a deep
     body) and one for a deep rule RHS, each asserting the **exact substituted
     `InputForm`**, not a summary;
   - a shallow companion for each, in the ordinary suite, pinning the
     substitution on a three-level fixture;
   - **a shallow shadowing regression**: a nested `With`/`Module` inside the
     deep-ish body where an inner binding shadows the outer one, asserting the
     exact result. Nothing else in the suite pins the per-subtree shadow set,
     and it is the semantic this conversion is most likely to break.
4. Two microbench labels, `with-deep-body-small` and `rule-rhs-deep-small`,
   each with a sink liveness assertion following the convention at
   `scripts/core-language-microbench.sh:546,552,558`.

## The falsifier — required section

Before changing any source, add the two shallow companions and the shadowing
regression, run all three against unmodified HEAD, and paste the passes. Then
break each deliberately — a wrong substituted value for the companions, and for
the shadowing test an expectation that the inner binding does *not* shadow — and
paste the failures. Restore all three before continuing.

Do **not** run the deep certificates against unmodified HEAD: at 4,097 the
pre-change code ends the process rather than failing a test.

## Scope

Files you MAY modify:

- `crates/rum-runtime/src/runtime/substitution/lexical_substitution.rs`
- `crates/rum-runtime/src/runtime/substitution/binding_substitution.rs`
- new files under `crates/rum-runtime/src/runtime/substitution/`, declared from
  `crates/rum-runtime/src/runtime/substitution/mod.rs` — you MAY edit that root
  for the `mod` lines only
- directly relevant runtime tests for scoping, substitution and rule
  application
- `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs`
- `scripts/core-language-microbench.sh` — only for the two new labels

Out of scope: `slot_substitution.rs`, `lexical_presence.rs`,
`function_and_subst_helpers.rs`, every consumer (`down_values.rs`,
`up_values.rs`, `sub_values.rs`, `rhs_guards.rs`, `callable.rs`, `with.rs`,
`module.rs`, `packages.rs`, `exceptions.rs`, the string-rule path), matcher
semantics, and `Expr` representation.

## Explicitly out of scope

Do not remove repeated work, cache shadow sets, or change how often
`may_contain_binding_symbol` runs. This brief bounds depth and nothing else.

## Measurement — instruction counts, not wall time (READ THIS)

**A timing percentage is not evidence on this host and will not be accepted.**
This machine is an i9-14900HX with 8 P-cores and 16 E-cores; a kernel module
caps every core at 5.2 GHz and `scaling_max_freq` reads 5 200 000 against
4 100 000, so scheduling alone can move a wall-clock number by up to 27% — far
more than any effect this gate is judging.

1. **Allocation counts.** Build with `--features alloc-count`, run with
   `RUM_PROFILE_ALLOC=1`; the binary prints
   `rum-profile alloc count= bytes_total= peak_live_bytes=` on exit. Use an
   isolated probe per label — fixture plus the exact statement in a `Do` loop.
   **Calibrate first**: one extra loop iteration must move the count. Keep the
   base and candidate executable paths the **same length**; argv size changes
   the count.
2. **Instruction counts.** `perf stat -e instructions:u -x, -- <bin> --file
   <probe.rum>` works unprivileged here. The hybrid PMU reports `cpu_atom` and
   `cpu_core` separately: a run that **migrates between core classes reads about
   55% high — discard it** (both counters non-zero); `<not counted>` means that
   PMU never ran; clean runs cluster within 0.006%-0.08%, so take the minimum of
   the tight cluster and name the runs you discarded.

Report both, base against candidate, for each new label. If both are unchanged
or move by a fraction of a percent, that is the result — do not then go looking
for a timing number, and do not open a stop-loss on one.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test -p rum-runtime
cargo test -p rum-stdlib
cargo test --workspace
```

`cargo test --workspace` may fail nine `rum-jupyter` native ZeroMQ tests if your
sandbox cannot bind loopback. That is an environment limit, not a result: say so
explicitly and also run `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test --workspace`,
pasting both.

Then, after the shallow suite is green:

```
cargo test -p rum-stdlib --release -- --ignored
```

That panel takes about 465 s end to end because of the `D` certificate; size the
timeout from that, not from a guess.

Then the counter and instruction comparisons described above.

## Stop-loss

Stop and report if:

- a shadowing or alpha-renaming difference appears twice;
- the unchanged/`None` sharing contract cannot be preserved;
- an entry signature would have to change, or a consumer file would have to be
  edited;
- allocation or instruction counts show a real increase on a shallow label that
  two isolated revisions do not remove;
- a certificate can only pass by lowering `TARGET_PHYSICAL_HEIGHT`.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave the tree for the
  gate.
- Do NOT modify files outside Scope. Touch nothing outside this repository; no
  network.
- Testing Policy is a hard line (`AGENTS.md`): rum's own documented behavior
  only; no oracle harnesses, no third-party engine outputs.
- No new `unsafe` in `rum-core`/`rum-runtime`.
- No default-path cost, and no profiling state carried in a work item or frame.
- Use `scripts/run-bounded.py --timeout SECONDS -- <command>` for anything that
  can run long or spawn children.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format

1. What changed — per file and line range, naming each removed recursive edge
   and how the per-subtree shadow set travels in the new driver.
2. Verification — every command with verbatim output, including all three
   falsifier passes and their deliberate failures, and both workspace runs.
3. Safety and contract as separate outcomes per engine: no process termination,
   and the height reached.
4. Allocation counts and instruction counts, base against candidate, with the
   calibration run for each instrument and the discarded migrated runs named.
5. Stop-loss / open questions.
