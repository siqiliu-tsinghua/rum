# brief-487 — a resumed `Table` panics the kernel

**Shape: DIAGNOSIS + DESIGN. `Scope: NONE — no source change.`** The
deliverable is the mechanism, the repair options with a recommendation, and a
drafted implementation brief in `TEMPLATE.md` shape. You write no production
code.

Found incidentally while converting the iteration accumulator (brief-486). It
is **not** caused by that conversion — the director reproduced it on the
pristine tree at `a5c519ca` — and it outranks the conversion, which is parked
in `stash@{0}` until this is settled.

## The defect

An ordinary source-language program, driven through the public session API,
**panics the kernel**:

```rust
let mut session = session_with_test_control_builtins();
let failure = session
    .eval_request("Table[InputString[\"value\"], {16}]")
    .unwrap_err();
assert!(matches!(failure.error, SessionRequestError::InputRequired { .. }));
for value in 0..15 {
    let failure = session
        .resume_eval_request_with_input(&value.to_string())
        .unwrap_err();
    assert!(matches!(failure.error, SessionRequestError::InputRequired { .. }));
}
let response = session.resume_eval_request_with_input("15").unwrap();
```

The final resume panics at
`kernel_data/control_data/branching_loops/iteration/mod.rs:97` —
`panic!("completed iteration accumulator must be uniquely owned")`, the `Err`
arm of `Arc::try_unwrap` in `PersistentExprAccumulator::into_vec`.

Director-verified on the pristine tree: dropped into
`runtime/tests/iterative_loop_drivers.rs` verbatim at `a5c519ca`, it fails with
that panic and nothing else. The probe is given here so **you do not need to
write one**.

Nothing in `TODO.md` or `docs/` records this.

## Why it matters, stated plainly

A `Table` whose body suspends for input and is then resumed to completion is
ordinary use. The failure mode is a **panic in the kernel**, not an
`EvalError`, so it reaches the host as a poisoned session rather than a
diagnosable failure. rum's contract is that a host request fails in band; a
panic is the one outcome the embedding API cannot classify.

## The hypothesis to confirm or refute

`into_vec` requires the tail to be uniquely owned. After a suspend/resume
cycle, something else still holds a strong reference — most plausibly a
suspension frame that captured the accumulator and has not released it by the
time completion runs, so the `Arc` count is above one. `PersistentExprAccumulator`
derives `Clone` (`iteration/mod.rs:57`), and `appended` clones the tail into
each new accumulator (`:80-89`), so a retained frame copy is entirely possible.

**Confirm the actual holder from source and from the failing run**, and say
which frame or field it is with `path:line`. Do not accept the hypothesis
because it is plausible; a plausible mechanism that is not the real one has
cost this project several briefs.

Also establish:

- **Where the boundary is.** Does it need 16 iterations, or does a smaller
  count reproduce? Does it need `InputString` specifically, or any suspension
  inside a collecting iteration (`Table`, `Sum`, `Product`, `Do`)? Does a
  single suspension suffice, or does it take repeated ones? Report the
  smallest reproducer you can establish **without writing a new panic probe of
  your own** — vary the supplied one.
- **Whether the panic is reachable through `rum-host`**, i.e. from an embedder
  rather than only through the in-crate session API. That decides whether this
  is a crash-only contract violation or an internal invariant.

## The repair options to weigh

At least these, and say which you recommend and why:

1. **Make `into_vec` not require uniqueness** — clone the payload out of a
   shared node instead of panicking. Cheapest, but it converts a loud
   invariant violation into a silent copy, and the invariant may be load-
   bearing elsewhere.
2. **Release the retaining reference before completion** so uniqueness genuinely
   holds. Correct if the retention is accidental; the question is whether the
   frame legitimately needs the accumulator after completion.
3. **Stop requiring a single owner by construction** — a different
   representation for the collected values across a suspension.
4. **Turn the panic into an `EvalError`** — this is the fallback, not a repair:
   it makes the failure classifiable without fixing the defect, and on its own
   it would leave a correct program failing.

For each: what breaks, what test proves it, and whether it is enforced or
merely intended.

## Notes for the drafted implementation brief

- The certificate is the probe above, and it must fail on the pre-repair tree
  and pass after — state that explicitly, since it is one of the rare cases
  where a genuine before/after pair exists.
- The repair touches a suspension path. Preserve the suspend/resume
  continuation shape; do not collapse a `suspend_with_frame*` into an eager
  call.
- Note whether the repair interacts with brief-486's parked conversion of the
  same type, which replaces the `tail` field with `AccumulatorChain`. The
  conversion is transparent to refcounts, so it should be independent — say so
  if you agree, or name the interaction if not.

## Scope

- **Files you MAY modify: NONE.** The report is the deliverable, with the
  drafted brief inside it.
- You MAY add the supplied probe to a test file **to run it**, but the tree
  must be clean when you finish — remove it and say you did.
- **Do not write a new crash or panic probe of your own.** Vary the one
  supplied. Nothing here needs a stack-overflow or process-termination probe.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Touch nothing outside this repo; no network.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **No new `unsafe`** in any drafted brief.
- **Cost is not an argument** (user ruling 2026-08-27). Option 4 alone is a
  stopgap and stopgaps are not proposed as the recommendation.
- Paste verbatim output for everything you run.

## Report format

1. "Diagnosis and design, no source change", and the mechanism with
   `path:line` evidence naming the actual retaining holder.
2. Every command run, verbatim, including the probe's failure and the
   boundary-narrowing variants.
3. The options, the recommendation, and the drafted implementation brief.
4. Stop-loss / open questions.
