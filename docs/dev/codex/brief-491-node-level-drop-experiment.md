# brief-491 — is the wrapper's `Drop` the cost, and can the node carry it instead?

**Shape: EXPERIMENT + DESIGN.** One prototype, one measurement, one
recommendation. brief-489's sixteen conversions are in the working tree,
uncommitted and correct; this decides whether they ship as they are.

## What the director measured — the premise, not a question to redo

brief-489's conversions cost instructions on three ordinary paths. Three drain
shapes were then measured against the same base (`HEAD` without the
conversions), same calibrated recipe, three runs per arm, arm spreads ~0.019%:

| shape | prepared-arguments | lhs-parents | value-table-rules |
|---|---:|---:|---:|
| `drop { drain(take) }`, drain `#[cold] #[inline(never)]` | +0.567% | +1.044% | +0.774% |
| fast path inlined into `drop`, `#[inline(always)]` (brief-490) | +2.243% | +1.941% | +3.572% |
| drain with **no** `#[cold]`/`#[inline(never)]` (director) | +0.492% | +0.994% | +0.689% |

**The conclusion, which this brief starts from:** the cost does not live in the
drain's body. Tuning it made things worse or made no difference. It is
intrinsic to `AccumulatorChain<N>` **having a `Drop` at all** — a holder whose
field carries a destructor gets different drop glue, loses field-wise moves,
and gains drop flags at every drop site.

## The question

Can the destructor live on the **node** instead of on the link wrapper, so a
holder's field goes back to a plain `Option<Arc<N>>` and pays nothing beyond
the refcount decrement it already paid?

```rust
impl Drop for SomeNode {
    fn drop(&mut self) {
        let mut link = self.previous.take();
        while let Some(shared) = link {
            let Some(mut node) = Arc::into_inner(shared) else { break };
            link = node.previous.take();
            // node drops here with an empty link, so no recursion
        }
    }
}
```

The reason brief-484 put `Drop` on the wrapper was that **13 types move fields
out of an unwrapped node across 14 sites**, and a type with `Drop` cannot be
destructured. That is an *effort* argument, and effort is not an argument on
this project (user ruling 2026-08-27). Whether it is also a *soundness*
argument is exactly what this brief must establish.

## What to do — one member, not sixteen

1. **Prototype on `PreparedArgumentNode` only**
   (`crates/rum-runtime/src/runtime/call_preparation.rs`): give the node its
   own iterative `Drop`, return its holder's field to a plain
   `Option<Arc<PreparedArgumentNode>>`, and remove the wrapper from that one
   type. Leave the other fifteen conversions exactly as they are.
2. **Solve the move-out sites for that type honestly.** `into_vec` and
   `into_vec_allow_shared` (`call_preparation.rs:77-100` and onward) move
   `value` out of an owned node. With a `Drop` on the node that no longer
   compiles. Report precisely how you resolved it and what it costs:
   `mem::replace` with a cheap dummy, an `Option` field, a different
   consumption shape — **no `unsafe`, no `ManuallyDrop`**. If no safe
   resolution exists that keeps the payload's semantics, **that is the answer**
   — report it and stop.
3. **Measure the same workload the same way**: base is `HEAD` without
   brief-489's conversions; candidate is the tree with the other fifteen
   conversions plus this prototype.

   ```
   taskset -c 2 perf stat -e instructions:u -x, -- <rum-cli> target/brief-489/prepared-arguments.rum < /dev/null
   ```

   Read `cpu_core/instructions/u`, require `100.00` multiplex coverage, build
   both arms at the same path, release profile, profiling env OFF, three runs
   per arm. Report both arms' counts, spreads and the delta against the
   **+0.567%** the wrapper shape costs on this workload.

   Note: run the CLI with stdin closed (`< /dev/null`). The director had three
   probe processes hang without it.
4. **Keep the guarantee.** Whatever shape you prototype must still: acquire
   with `Arc::into_inner`, stop at a shared node leaving its surviving holder
   responsible, release immediately in O(1) auxiliary space, and never recurse.
   Prove it with a depth observer in the prototype's own test module, asserting
   destruction nesting of exactly 1 at lengths 16 and 64 — the same
   discriminator as brief-485's helper — plus a falsifier that makes it read
   the chain length.

## One adjacent question, cheap while you are in the file

`call_preparation.rs:81` panics with `completed call argument state must be
unique` when `Arc::try_unwrap` fails, and `into_vec_allow_shared` right below
it handles the shared case instead. That is **the same shape as the defect
brief-487 diagnosed and brief-488 fixed**, where a suspension frame legitimately
retained the state and the uniqueness precondition was simply wrong.

Say whether a suspension can leave this one shared — i.e. whether the panicking
`into_vec` is reachable with a retained frame. **Do not fix it here** and do not
write a panic probe; source reasoning plus the existing tests is what is wanted,
and a real finding becomes its own brief.

## Scope

Files you MAY modify: `crates/rum-runtime/src/runtime/call_preparation.rs` and,
only if the field type change forces it, its immediate call sites. Nothing else.

MUST NOT touch: `accumulator_chain.rs`, `crates/rum-derive/**`, the other
fifteen converted members, the ownership scanner and its inventories — **if an
inventory row moves, report it rather than editing it; this is a prototype and
may well be reverted**; `host_payload.rs`; the suspension traits; `rum-core`;
`SECURITY.md`, `AGENTS.md`, `PROGRESS.md`, `TODO.md`, other `docs/**`,
`docs/dev/codex/**` except your own report; `.git`, git config, `Cargo.lock`.

## Verification

```
python3 scripts/run-bounded.py --timeout 180  -- cargo fmt --check
python3 scripts/run-bounded.py --timeout 2400 -- cargo test --workspace --offline --locked
```

The workspace must stay green with the prototype in. If your sandbox cannot
bind loopback, say so and name the skip you used.

## Hard rules

- Do NOT `git commit`, branch, push, touch git config, or run any `git stash`
  operation. The tree carries brief-489's uncommitted work — **do not revert,
  stash or "clean" it**.
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- Touch nothing outside this repo; no network (`--offline` everywhere).
- **No new `unsafe`**, and no `ManuallyDrop` shortcut. If the only way through
  is unsafe, that is a finding: stop and say so.
- **No crash, overflow, past-ceiling, or fatal-process probe.**
- **No `#[cfg(test)]` divergence in production code.**
- **Stop-loss: one prototype.** If it does not compile or does not measure
  cleanly after one honest attempt, report what you learned rather than trying
  a second design — the director is choosing between two known options, not
  looking for a third.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format

1. The prototype: what changed, with line ranges, and exactly how the move-out
   sites were resolved.
2. The measurement: both arms, spreads, delta, against +0.567%.
3. The depth certificate and its falsifier, verbatim.
4. `cargo fmt --check` and the workspace total.
5. Your answer on the `call_preparation.rs:81` uniqueness panic.
6. Your recommendation: node-level `Drop` for the family, or keep the wrapper
   and accept the measured cost — with the reason, not the preference.
