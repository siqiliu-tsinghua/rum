# brief-567 — design: a resumable, depth-safe schema validator (group G3)

`Scope: NONE — no production change.` Design-only, with one declared scratch
test (see Scope). **Do not run `git` in any form**, including before you have
read this brief.

## Goal

Group G3 of the suspension-continuation branch: `ConformSchema` and
`SchemaErrors` (`crates/rum-stdlib/src/schema.rs`) resume **past** their work
when a predicate or default asks for input. brief-548 recorded seven sites:

| ID | Site | Edge |
| --- | --- | --- |
| P2056 | `schema.rs:99` | `finish_schema_errors` → `conform_value` |
| P2061 | `schema.rs:150` | `finish_conform_schema` → `conform_value` |
| P2065 | `schema.rs:233` | `conform_association` → `conform_value` |
| P2066 | `schema.rs:246` | `conform_association` → `evaluate` (a default) |
| P2067 | `schema.rs:247` | `conform_association` → `conform_value` |
| P2068 | `schema.rs:297` | `conform_list` → `conform_value` |
| P2069 | `schema.rs:309` | `conform_leaf` → `match_leaf` |

The director re-ran brief-548's probes on `2df7681b`: all nine G3 witnesses
still diverge (`ConformSchema-leaf`, `-association`, `-default`,
`-default-predicate`, `SchemaErrors-leaf`, `-association`, `-default`,
`Schema-twice`, `Schema-defaults-twice`; `.temp/director/brief548-probe-rerun-2df7681b.txt`).
Lost continuations today: 66; G3 closes seven, predicted 66 -> 59.

**The director's adjacent-boundary probe found the validator also aborts the
process.** On the ruled 2 MiB stack with a current one-codegen-unit release
build (`.temp/brief-566/candidate-06/rum-cli`, source identical to HEAD):

```
v = ConstantArray[1, Table[1, {n}]];   (* builds fine: Depth[v] == n + 1 *)
ConformSchema[v, v]
```

n = 600 passes; **n = 1 500 and 3 000 end in `thread 'main' has overflowed its
stack` (exit 134)**; at n = 6 000 the construction itself fails closed with
`evaluation exceeded native stack limit`. A schema containing patterns cannot
be built past ~150-200 levels by evaluation (the evaluator's guard fires), but
a literal schema can, and the validator's own recursion
(`conform_value` → `conform_association` / `conform_list` → `conform_value`)
has no guard. Probe files: `.temp/director/gate-g3/lit-*.rum`, `bis-*.rum`.

This is the same situation G2 was in (briefs 561-563): the structure the
repair must make resumable is a native recursion that already aborts the
process. **G3 includes making the validator depth-safe.** Design both together.

## The director's preferred direction — evaluate it, do not assume it

G2 settled it for a unary chain: a flat representation under an explicit
cursor made every walk iterative, and the cursor *was* the suspension state.
The validator walks a **tree** (associations and lists have many children), so
the analogue is an explicit, heap-held work stack: one frame per open
container (its schema, its value, its cursor over keys or indices, its
conformed-output prefix, its path), plus a pending phase for the leaf or
default being evaluated. Suspension freezes that stack; resume applies the one
answer and continues the same loop. Compare against keeping the recursion
behind the native stack guard (fail closed with `StackLimitExceeded`) plus a
chain of per-level continuation frames, and choose on correctness and
enforceability (`CLAUDE.md`'s standard). G2's report records why the guarded
alternative lost there: `Clone`, root tracing and `Drop` are infallible and
cannot return the guard's error.

## What the design must settle

1. **The current structure, with `path:line`**: `SchemaValidator<'a>` (it
   borrows the `Session`), every recursive and evaluating edge, how errors
   accumulate (`push_error`), how `SchemaPath` grows per level (and what that
   costs at depth), `SchemaAbort` and how `InputRequired` travels through it,
   options (defaults, strictness), and the two finishers.
2. **The representation and loop**, in implementable detail: the work-stack
   frame for associations (key cursor, entry specs, missing-key defaults,
   extra keys under the strictness option) and for lists, the leaf phase
   (`match_leaf` through `MatchQ`), the default phase (evaluate the default,
   then validate it), and how a finished child is folded into its parent's
   output and errors. Every observable semantic — output shape, error order,
   error paths and messages, when defaults are evaluated, short-circuits —
   must be preserved; say how each is.
3. **Suspension and resume**: what one frame owns (the frozen stack, the
   errors so far, the pending phase and its saved value), how the answer is
   applied once, a second suspension, abandonment (no skipped validation or
   default runs), and how `SchemaErrors`' and `ConformSchema`'s finishers
   complete. Follow brief-563's settled protocol: immutable, Session-rooted
   frames with a cold working copy on an actual resume; capture only on
   `InputRequired`, on the error edge; frames derive `FrameRootTrace` and
   `PureFrameAbandon`; no `Session` borrow, observer or `NativeStackDepth` in a
   frame; rum-runtime's closed frame lists (`TEMPLATE.md` standing duty).
4. **Depth safety, enforced by structure.** No walk — validation, path
   building, error rendering, clone, trace, drop — may recurse natively in
   proportion to the value's or schema's depth. Say how this is **enforced**,
   not demonstrated: brief-563 added `lazy_depth_contract.rs`, which runs the
   recursion scanner (`recursive_edges_from_source`) over the lazy sources per
   file and as one unit, and checks no lazy type is self-reaching, each with a
   falsifier. Propose the equivalent for the schema module and say which
   helpers outside it (path rendering, `MatchQ`, `render_error`) are reached
   and whether they are bounded.
5. **The ordinary path.** A validation that never suspends must not allocate
   or clone for the continuation machinery; the work stack itself replaces the
   native stack and is measured. Name the labels (add schema labels: a small
   valid list, a small association with defaults, a deep literal schema at a
   few depths) and the plan. **Instruction gates build both arms with one
   codegen unit** (`TEMPLATE.md` pitfall 8).
6. **Inventory and layout consequences**: which ownership-graph and
   retention-contract rows the change adds, moves or re-identifies (list them
   so the director can authorize them by name), and the module layout under
   `docs/code-organization-guidelines.md`.
7. **Retention**: which brief-547 region the work stack, the conformed prefix
   and the error list fall in; no leases here.

## What I want back

1. The current structure and the recursion inventory, with evidence.
2. The design (points 2-7), the frame inventory (fields and roots), and the
   comparison with the guarded alternative.
3. Red tests, run in the scratch test and recorded: all nine brief-548 G3
   witnesses through the rum-vs-rum oracle (resumed equals queued in value,
   `log`, messages and prints; suspension asserted; exact prompts; no pending
   owner and a working `1 + 1` after), suspension at two positions, an
   ordinary error after a resume, cancellation at a suspended predicate and at
   a suspended default, a strictness/extra-key case, a nested association in a
   list in an association. **Depth cases:** the director's literal-schema
   abort, and a nested association schema, forcing every level and asserting
   the exact result, on a 2 MiB thread at a depth the descriptor-construction
   control survives in the debug test profile (measure that control first, as
   brief-563 did; do not lower n silently), plus release `rum-cli` evidence
   under `ulimit -s 2048` at a depth beyond the pre-fix ceiling. A native
   overflow aborts the test binary, so record pre-fix deep evidence by running
   each deep case alone or through the release CLI. Measure the pre-fix
   per-level cost and ceiling.
4. Drafted implementation brief(s) in `TEMPLATE.md` shape, each with its red
   tests, its predicted lost-continuation count, the named inventory rows it
   authorizes, the structural depth checks with falsifiers, the one-unit
   performance gate, the standing duty, and serial runs
   (`env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1`).
5. Open questions for the director.

## Context pointers

- brief-548's report, `.temp/brief-548/draft-G3.md`, `probes.json`,
  `lost.tsv`, `probe.rs`.
- briefs 561, 562 and 563 with their reports — the G2 design, its revision for
  depth, and the accepted implementation (`crates/rum-stdlib/src/lazy/`,
  `tests/association_depth_contract/lazy_depth_contract.rs`,
  `tests/lazy_pull_continuations.rs`); briefs 565-566 for the one-unit
  measurement and what the flat loop cost.
- `crates/rum-stdlib/src/schema.rs`; `docs/guide/` for `ConformSchema` and
  `SchemaErrors`; the generated reference.
- `crates/rum-runtime/src/runtime/suspension/core.rs`,
  `suspension/primitive.rs`, `native_stack_guard.rs`.
- `docs/dev/codex/TEMPLATE.md`, `AGENTS.md`,
  `docs/code-organization-guidelines.md`.

## Scope

- Files you MAY modify: **NONE**, except: append your report to this file;
  scratch under `.temp/brief-567/`; **one** scratch integration test,
  `crates/rum-stdlib/tests/zz_brief567_probe.rs`, **deleted before the report
  is appended**, its final content copied to `.temp/brief-567/probe.rs`.
- Everything else is read-only.

## Hard rules

- **No `git`, in any form.**
- Do NOT modify any file outside the exceptions; STOP and report instead.
- No network; nothing outside this repository; all scratch and `TMPDIR` under
  `.temp/brief-567/`.
- Commands, serial, each through `python3 scripts/run-bounded.py`:
  `env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --test zz_brief567_probe`
  (150 s); one-codegen-unit release builds of `rum-cli` from the unchanged
  tree into a target directory under `.temp/brief-567/`
  (`CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, 600 s; the director's took 98 s);
  that binary under `bash -c 'ulimit -s …; ulimit -c 0; …'` with stdin
  `/dev/null` (60 s); `valgrind --tool=callgrind` on it if attribution needs it
  (150 s). File reads, `rg`/`grep`, `python3`.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **No production code.** No new `unsafe`.

## Report format (your final message, appended to this file)

1. What changed — "design only", scratch, the deleted test, any build.
2. The current structure and recursion inventory.
3. The design, frame inventory, and the comparison.
4. The red and depth tests with their pre-fix values and the depth measurements.
5. The drafted implementation brief(s).
6. Open questions for the director.

---
