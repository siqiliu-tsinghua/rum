# brief-450 — iterative `Take`/`Drop` specs and the `ReplacePart` plan

## Goal

Two exact-path consumers still end the process on a deep value: multi-spec
`Take`/`Drop` at 2,573 levels and `ReplacePart` along a deep path at 2,854.
Remove the depth-proportional Rust recursion from both so each completes at
physical height 4,097 on a 2 MiB stack and fails in band above it, with every
documented value, message, and sharing property unchanged.

This is brief-439's draft 5, cut down to what measurement says is actually
short — see "Why this brief is smaller than draft 5".

## What I measured — treat as given, do not re-run

Release `rum-cli` at `250e41fd`, `ulimit -s 2048`, 1 MiB evaluator budget.
Every probe carries a liveness assertion on the exact rebuilt value.

| site | last ok / first fail | mode |
|---|---|---|
| `Take @@ Prepend[Table[{1}, {N}], x]` | **2,573 / 2,581** | **process kill** |
| `ReplacePart[x, Table[1, {N}] -> 9]` | **2,854 / 2,862** | **process kill** |
| nested association part assignment `b[["k",…]] = 8` | 1,106 / 1,114 | in-band |
| `MapAt[ff, x, {p}]` | clears 4,097 | — |
| `ReplaceAt[x, 1 -> 5, {p}]` | clears 4,097 | — |
| `Insert[x, 9, p]` | clears 4,097 | — |
| `Delete[x, p]` | clears 4,097 | — |
| `Extract[x, p]` | clears 4,097 | — |
| list part assignment `y[[1,…,1]] = 7` | clears 4,097 | — |

Both target files contain **zero** occurrences of `poll_native_stack_limit` and
`evaluation_depth`, which is why they end the process instead of raising an
error.

Do not re-measure any of this, and do not write a probe that drives either
recursion past its ceiling.

## Why this brief is smaller than draft 5

Draft 5 proposed shared primitives for exact lookup, reverse rebuild, selector
selection, `ReplacePart` planning and sequence edit plans, then routing roughly
fifteen files through them. **Six of the nine named consumers already clear
4,097**, so that refactor would mostly rewrite working code. Two are short and
fail open; those are this brief.

The third short site — nested association part assignment at 1,106 — is
**out of scope**: it fails closed, so it is a contract shortfall rather than a
safety defect, and it brackets identically to `MergeNested` (1,114), which
suggests a shared association-side cause that deserves its own diagnosis rather
than a repair bolted onto this one.

## The recursion edges to remove

**`crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs`** — one native
level per spec:

- `apply_take_specs` (`:303`) consumes `specs.split_first()`, applies the first
  spec, then calls `apply_take_drop_rest_to_expr` (`:325`);
- `apply_drop_specs` (`:328`) does the same (`:349`);
- `apply_take_drop_rest_to_expr` (`:373`) maps the remaining specs over every
  argument through the `recurse` function pointer (`:382`), which is the
  re-entry.

These are pure functions over `&Expr` returning `Option<Expr>` — no `Session`,
no suspension. Keep them that way.

**`crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/replace_part.rs`**
— one native level per plan level:

- `apply_replace_plan_owned` (`:475`) recurses into the head plan at `:495` and
  `:503`, and into each child plan at `:509`.

It takes `&mut Session` and returns `Result`, but the only action is
`ReplacePlanAction::Immediate` (`:482-484`), so no suspension happens inside
plan application. Do not introduce one.

## Semantics that must not move

- **`None` is a message channel.** `apply_take_specs` returns `None` for an
  atom (`:309`), for an unresolvable component (`:312`) and when a nested
  application fails; the caller turns that into the documented `Take`/`Drop`
  message. An iterative driver must produce `None` in exactly the same cases,
  including the case where the failure is discovered several specs deep.
- **Spec order and fan-out.** Each spec applies to the whole current level, then
  the remaining specs apply to *every* argument of the selection (`:380-383`).
  That is a breadth fan-out, not a single path: the work stack has to carry the
  remaining spec slice per node.
- **Head plans come before argument plans** in `apply_replace_plan_owned`, and
  an atom with a head plan becomes `Expr::call(replaced_head, [])` (`:493-498`).
  Preserve both.
- **Plan children are removed as they are consumed** (`plan.children.remove`),
  so a plan node is visited once. Keep that.
- **Sharing.** `Expr`/`SharedExpr`/`SharedArgs` are `Arc`-shared and unchanged
  siblings must stay pointer-identical; `Expr::call_with_shared_args` at `:514`
  is what preserves that today.

## Deliverable

1. Both walkers driven by explicit work stacks behind a depth-16 recursive
   front, the pattern `Level`, `Map`, `Replace`
   (`.../replace/once/traversal.rs`), `D`
   (`crates/rum-stdlib/src/calculus/differentiation/iterative.rs`) and
   `DeleteCases` (`.../cases/delete/iterative.rs`) already use. A shallow
   `Take[{1,2,3}, 2]` or `ReplacePart[{0}, {1} -> 9]` must not pay for the deep
   machinery.
2. No new suspension frame. If you think you need one, stop and report.
3. Certificates, following the rule this line settled in briefs 439-442:
   - release-only `#[ignore]` deep certificates on a 2 MiB declared thread at
     `TARGET_PHYSICAL_HEIGHT`, one for multi-spec `Take` and one for deep-path
     `ReplacePart`, each asserting the **exact rebuilt `InputForm`**, not a
     summary (`crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:43-49`
     has the `nested_input_form` helper);
   - a shallow companion for each, in the ordinary suite, pinning the spec /
     path semantics on a three-level fixture;
   - a shallow regression pinning the `None` path: a multi-spec `Take` whose
     failure occurs *below* the first spec, asserting the documented message.
4. Small microbench labels `take-multispec-small` and
   `replacepart-deep-path-small`, each with a sink liveness assertion following
   the convention at `scripts/core-language-microbench.sh:546,552,558`.

## The falsifier — required section

Before changing any source, add the two shallow companions and the `None`
regression, run them against unmodified HEAD, and paste the passes. Then break
each deliberately — a spec list one element short for the companions, an
expectation of success for the `None` test — and paste the failures. Restore
them before continuing.

Do **not** run the deep certificates against unmodified HEAD: at 4,097 the
pre-change code ends the process rather than failing a test.

## Scope

Files you MAY modify:

- `crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs`
- `crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/replace_part.rs`
- new files beside either, declared from their existing module roots
  (`crates/rum-stdlib/src/structure_ops/sequence.rs`,
  `crates/rum-runtime/src/runtime/kernel_data/structure_data/replace.rs`) — you
  MAY edit those two roots for the `mod` line only
- directly relevant runtime/stdlib tests for `Take`, `Drop`, `TakeDrop` and
  `ReplacePart`
- `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs`
- `scripts/core-language-microbench.sh` — only for the two new labels

Out of scope: `exact_paths.rs`, the selector modules, `part_assignment.rs`,
`targeted_map.rs`, `associations/mutation.rs`, `edit_plan.rs`, level-spec
parsing, `Expr` representation, and every consumer that already clears 4,097.
Do not build the shared primitives draft 5 proposed.

## Explicitly out of scope

Do not remove repeated work, cache selections, or change how many times a spec
is resolved. This brief bounds depth and nothing else.

## Measurement — instruction counts, not wall time (READ THIS)

**A timing percentage is not evidence on this host and will not be accepted.**
This machine is an i9-14900HX with 8 P-cores and 16 E-cores; a kernel module
caps every core at 5.2 GHz, and `scaling_max_freq` reads 5 200 000 against
4 100 000, so scheduling alone moves a wall-clock number by up to 27% — far more
than any effect this gate is judging. Two of the last three briefs in this line
ended in a stop-loss on a timing delta that turned out to be noise.

Use deterministic instruments instead:

1. **Allocation counts.** Build with `--features alloc-count` and run with
   `RUM_PROFILE_ALLOC=1`; the binary prints
   `rum-profile alloc count= bytes_total= peak_live_bytes=` on exit. Write an
   isolated probe per label — the fixture plus the exact statement in a `Do`
   loop — and compare base against candidate. **Calibrate first**: one extra
   loop iteration must move the count, or your probe is not measuring the
   statement.
2. **Instruction counts.** `perf stat -e instructions:u -x, -- <bin> --file
   <probe.rum>` works unprivileged here (`perf_event_paranoid` is 2). Three
   traps: the hybrid PMU reports `cpu_atom` and `cpu_core` separately, and a run
   that **migrates between core classes reads about 55% high — discard it**
   (both counters non-zero); `<not counted>` means that PMU never ran; clean
   runs cluster within 0.006%-0.08%, so take the minimum of the tight cluster.

Report both, base against candidate, for each new label. If both are unchanged
or move by a fraction of a percent, that is the result — **do not** then go
looking for a timing number, and do not open a stop-loss on one.

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

That panel takes about 460 s end to end because of the `D` certificate; size the
timeout from that, not from a guess.

Then the counter and instruction comparisons described above.

## Stop-loss

Stop and report if:

- the `None` / message contract cannot be preserved without changing a caller;
- a spec fan-out ordering difference shows up twice;
- allocation or instruction counts show a real increase on a shallow label that
  two isolated revisions do not remove;
- pointer identity for unchanged siblings regresses;
- a certificate can only pass by lowering `TARGET_PHYSICAL_HEIGHT`.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave the tree for the
  gate.
- Do NOT modify files outside Scope. Touch nothing outside this repository; no
  network.
- Testing Policy is a hard line (`AGENTS.md`): rum's own documented behavior
  only; no oracle harnesses, no third-party engine outputs.
- No new `unsafe` in `rum-core`/`rum-runtime`.
- Use `scripts/run-bounded.py --timeout SECONDS -- <command>` for anything that
  can run long or spawn children.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format

1. What changed — per file and line range, naming each removed recursive edge.
2. Verification — every command with verbatim output, including both falsifier
   passes and their deliberate failures, and both workspace runs.
3. Safety and contract as separate outcomes per site: no process termination,
   and the height reached.
4. Allocation counts and instruction counts, base against candidate, with the
   calibration run that shows each instrument is sensitive, and the discarded
   migrated runs named.
5. Stop-loss / open questions.
