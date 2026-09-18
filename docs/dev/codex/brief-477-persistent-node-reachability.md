# brief-477 — is the `Persistent*Node` chain reachable from the source language?

## Goal

Settle one question with evidence: **can a `Persistent…Sequence`'s linked tail
be made long from the source language, and if so, how long?**

The answer decides one row in the ownership inventory brief-476 is designing,
and it is currently `Unbounded(owed_by = this brief)` because nobody knows.

**This is a measurement brief.** No production change. The deliverable is a
reproducible construction plus a number, or a source-backed argument that no
such construction exists.

## What the structure is

Eleven files define the same shape — for example
`crates/rum-stdlib/src/associations/suspended_state.rs:50-60`:

```rust
struct PersistentAssociationSequence<T> { prefix: Arc<[T]>, tail: Option<Arc<Node<T>>>, len: usize }
struct PersistentAssociationSequenceNode<T> { previous: Option<Arc<Node<T>>>, value: T }
```

`appended` prepends one node and clones the previous tail
(`:72-81`); there is **no compaction** and **no length cap at the append site**.
A derived `Drop` on that chain recurses once per node, which is the class that
terminated the process at depth 60,000 in brief-468 with a different type.

The eleven files: `associations/suspended_state.rs`,
`functional_ops/suspended_state.rs`, `logic/suspended_state.rs`,
`predicates/suspended_state.rs`, `strings/suspended_state.rs`,
`structure_ops/suspended_state.rs`,
`runtime/kernel_data/rewrite_data/suspended_state.rs`,
`runtime/kernel_data/structure_data/replace/suspended_state.rs`,
`runtime/kernel_data/control_data/branching_loops/iteration/mod.rs`,
`runtime/pattern_matching/suspended_state.rs`,
`runtime/tree_walkers/position.rs`.

## What the director already established — do not redo this

Measured 2026-09-18, all of it negative, all of it costing a probe:

1. **Every one of the 152 `.appended(` call sites is reachable only from a
   `resume_suspended_*` function.** The ordinary (first-resume) paths use a plain
   `Vec` sibling — for example `extend_flat_logical_residual` at
   `crates/rum-stdlib/src/logic/truth.rs:549` against the linked-list
   `append_flat_logical_residual` at `:533`.
2. An earlier classification that reported "3 ordinary-path sites" was **wrong**:
   it classified by the function *containing* the `.appended(` call, which for
   two of them was the helper's own definition, not a caller.

**Five probes failed to reach the linked-list builder**, each verified by a
`panic!` planted in `append_flat_logical_residual` as a liveness assertion:

| probe | outcome |
|---|---|
| `And @@ Table[xx[i], {i, N}]`, no suspension | Vec path; panic never fired |
| the same at N=99,999 across 2048/1024/512/256 KiB stacks | all completed; measuring the Vec path |
| `And[InputString[…], xx[1], …, xx[N]]`, one queued input | panic never fired |
| the same with two `InputString` arguments and two queued inputs | panic never fired |
| `SelectPositions[…, (# === 1)&]` | hung; probe defect, not a finding |

So: **a single suspension and a resume are not enough.** The working hypothesis
is that `resume_*` handles the first resume and `resume_suspended_*` is entered
only when a resume itself suspends, but that is a hypothesis and the probes
above did not confirm it.

## What to deliver

1. **From source**, for at least three of the eleven families: the exact entry
   condition under which a `resume_suspended_*` function runs, and whether the
   tail it builds grows per **element** or per **suspension**. Cite `path:line`.
   Per-element growth after one suspension is the dangerous shape; per-suspension
   growth is self-limiting because each suspension costs a host round trip.
   `resume_suspended_replace_string_value_from_index`
   (`crates/rum-stdlib/src/strings/patterns/search/rules.rs:258`, appending per
   character at `:348`) is the strongest candidate for per-element growth.
2. **A reproducible construction** that drives one family's tail to length
   `N >= 1000`, written as a `#[test]` using the host request API —
   `eval_request` plus `queue_input_response` / `resume_eval_request_with_input`;
   `crates/rum-stdlib/src/tests/session_requests/transforms_resume_associations.rs:11-47`
   is the idiom. **Prove the tail is real** by planting a temporary counter or
   assertion in the `appended` path and reporting its value, then removing it —
   a test that merely produces the right answer proves nothing here, as five
   probes above demonstrate.
3. **The achieved length** and what bounds it: a generated-element limit
   (`DEFAULT_GENERATED_ELEMENT_LIMIT` is 100 000), an iteration limit, or
   nothing.
4. If you conclude **no source-language construction can make the tail long**,
   say so with the source argument that makes it true, and name what would have
   to change for it to become false. That is a complete and useful answer.

**Do not drive the drop to a crash.** Report the construction and the length you
reached; the director brackets the ceiling, as in every other measurement in
this line.

## Scope

- Files you MAY modify: a **temporary** test and a **temporary** instrumentation
  line in the `appended` path, both of which you remove before finishing.
  `git diff --stat` at the end must be empty.
- Files you MUST NOT leave modified: anything.
- If the measurement requires a production change to be observable, STOP and
  report that — it is a finding about observability, not a licence to edit.

## rum / Rust pitfalls (read twice)

1. **A probe that produces the right answer has not necessarily run the code you
   named.** Five did not. Plant a liveness assertion inside the target and check
   that it fires, every time, before believing a result.
2. `resume_*` and `resume_suspended_*` are different families with different
   data structures. Confirm which one your fixture enters.
3. The CLI cannot drive `InputString`: in `--file` mode it reports
   `input required` and exits. Use the host request API from a test.
4. Wall-clock timing is not evidence on this host.

## Verification

```
cargo fmt --check
cargo test -p rum-stdlib
```

Plus the verbatim output of your instrumented run showing the achieved tail
length, and `git diff --stat` showing an empty tree at the end.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Leave no modification in the tree.
- **No new `unsafe`.** No network. **Testing Policy is a hard line**
  (`AGENTS.md`).
- **Do not drive a recursion or a drop to a crash.**
- **Stop-loss.** Stop and report if: no construction reaches a
  `resume_suspended_*` path after three genuinely different shapes; or the
  instrumentation cannot be made to fire.
- Paste real verbatim output. If something was not established, say so.

## Report format

1. Measurement only, tree left clean — state it, with `git diff --stat`.
2. Per family examined: the entry condition for `resume_suspended_*`, and
   whether the tail grows per element or per suspension, with `path:line`.
3. The construction, the liveness evidence that it reached the target, and the
   achieved tail length.
4. What bounds the length.
5. If no construction exists: the source argument, and what would falsify it.
6. Stop-loss and open questions.
