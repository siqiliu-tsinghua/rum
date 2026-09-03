# brief-437 — the evaluator's native stack budget: guard calibration and frame cost

**Shape: design-only. `Scope: NONE — no source change.`**

## Goal

Two coupled facts have to be settled together before any repair is written.

1. The kernel's two resource guards are calibrated for a stack about four
   times larger than the ruled embedder default, so on the ruled configuration
   they never fire and the process dies instead of returning an error.
2. One evaluation level costs about 4.7 KiB of native stack. That number is
   what decides whether the guards can be calibrated at all while still
   honouring rum's own documented `$RecursionLimit` default, and whether the
   ruled physical-depth bound is reachable for a strict residual call.

I want a plan that settles both, with the second measured rather than
estimated, plus drafted implementation briefs. No source change, no
benchmark, and — stated up front — **no reproduction of a stack overflow**;
see Hard rules.

## What is already established — do not re-measure this

brief-436 (`docs/dev/codex/brief-436-construction-depth-class.md`, report and
director gate appended) diagnosed the mechanism and I verified it. In short:
a strict head whose call stays residual and never acquires a stability proof
is re-evaluated through eager argument preparation on every construction step,
one native evaluator level per spine level. The class is wide — it includes
`Power`, `Mod`, `Max`, `Min`, `Equal`, `Less`, `Not`, and every metadata-only
string head — and ordinary user-defined heads are outside it.

My gate measurements, which you should treat as given:

**The guards, on `N=3000` deep residual construction:**

```
default stack (8 MiB):  exit=1   "evaluation exceeded recursion limit (1024)"   in-band
ruled stack   (2 MiB):  exit=134 "thread 'main' has overflowed its stack"       process killed
```

**Ceilings, bisected, release build:**

| head | stack | last ok | first fail |
|---|---|---|---|
| `Power` | 1024 KiB | 202 | 218 |
| `Power` | 2048 KiB | 421 | 437 |
| `Power` | 4096 KiB | 875 | 890 |
| `StringExpression` | 2048 KiB | 421 | 437 |

Linear in stack size, and identical across two heads with different handlers,
so the cost is in the shared chain. Slope
`(4096-1024) KiB / (875-202) levels` = **≈4.7 KiB per evaluation level**.

**Frame prologues, read from `target/release/rum-cli` with `objdump -d` and
summing the leading `push` plus the first `sub $N,%rsp`:**

| bytes | function |
|---|---|
| 1064 | `eval_core::…::evaluate_inner_unstable` |
| 952 | `eval_call_prepare::…::prepare_arguments_for_call_with_unevaluated_restore` |
| 744 | `resolution::…::evaluate` |
| 424 | `eval_core::…::evaluate_atom_fast_path` |
| 328 | `eval_dispatch::…::run_call_dispatch_stage` |

Those five sum to 3,512 B against a measured 4,700 B, so roughly a quarter of
the per-level cost is in frames I did not enumerate. Closing that gap is part
of deliverable 1.

## Context pointers

- `AGENTS.md` — operational notes and the Testing Policy hard lines.
- `crates/rum-runtime/src/runtime/metadata.rs:4-11` — every resource-limit
  default, including `DEFAULT_RECURSION_LIMIT = 1024`,
  `DEFAULT_NATIVE_STACK_LIMIT = 6 * 1024 * 1024`, and
  `NATIVE_STACK_DEPTH_POLL_STRIDE = 4`.
- `crates/rum-runtime/src/runtime/eval_core/mod.rs:20-79` — the depth check,
  the strided poll, and `poll_native_stack_limit`, which measures bytes
  consumed from `native_stack_base`.
- `crates/rum-runtime/src/runtime/resolution.rs:285-300` — where
  `native_stack_base` is captured.
- `crates/rum-runtime/src/runtime/suspension/core.rs:480-512` — the same
  capture on the resume path.
- `crates/rum-runtime/src/runtime/model/config.rs:1-33` — `ResourceLimits`
  and its `Default`.
- `crates/rum-runtime/src/runtime/session/session_api.rs:108-120` —
  `set_native_stack_limit`, the host knob that already exists.
- `crates/rum-host/src/config.rs:55-115` — how the host profiles forward
  `recursion_limit` and `native_stack_limit`.
- `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:1-46` — the
  release-only 2 MiB certificate harness and `TARGET_PHYSICAL_HEIGHT = 4_097`.
- `crates/rum-runtime/src/runtime/tests/holding_and_control_part2.rs:466-505`
  — the existing native-stack-limit and bounded-recursion tests.
- `docs/dev/codex/brief-426-height-cache-spike.md:25-30` — the ruling that
  fixes the physical depth bound at `L >= 4097` on a 2 MiB stack.

## The problem, stated as I see it

`poll_native_stack_limit` is the right instrument: it measures bytes actually
consumed rather than a depth proxy, so it is stack-size-honest by
construction. What is wrong is the number it is compared against. A fixed
6 MiB constant is not a property of the evaluator; it is an assumption about
the host's stack, and it contradicts the ruled 2 MiB embedder default.

The coupling that makes this a design question rather than a one-line change:
at 4.7 KiB per level, `$RecursionLimit = 1024` needs ≈4.8 MiB of stack. So on
the ruled 2 MiB configuration rum cannot honour its own documented default
depth limit no matter how the byte guard is set. Three ways out, and I want
the plan to choose between them on evidence rather than assert one:

- the frame shrinks enough that 1024 levels fit in the ruled stack;
- the default depth limit becomes a function of the declared stack budget;
- the byte guard binds first on small stacks and the documented depth limit is
  restated as a ceiling rather than a promise.

Note also that safe Rust cannot ask the operating system how much stack the
current thread has. So the budget is necessarily declared, not discovered, and
the design question is what a *safe default* is and what enforces it.

## Deliverables

1. **The complete per-level frame inventory.** Enumerate every function
   entered once per evaluation level for a strict residual call such as
   `Power[deep, "a"]`, with the frame bytes of each read from the release
   binary, and reconcile the sum against the measured 4.7 KiB. Say where the
   ~1.2 KiB I did not account for lives. Give the method you used so I can
   reproduce it.

2. **Where the bytes come from, in source terms.** For each frame above about
   300 bytes, attribute the allocation to source constructs: which locals,
   which branches, which large enum or struct temporaries. LLVM allocates the
   union over branches, so name the cold branches whose locals are being paid
   for on the hot recursive path — suspension frame construction, error
   construction, unevaluated-restore accumulators, and anything else you find.
   `path:line` for each.

3. **The reachable floor, with a projected number.** Which of those frames can
   be reduced, by what technique (outlining a cold branch behind
   `#[cold] #[inline(never)]`, boxing a large temporary, splitting a function,
   moving state into `Session`), and what the per-level cost would plausibly
   become. I want a range with reasoning, not a single confident figure. State
   plainly if the answer is that no combination gets near 512 B per level,
   because that is the number that would make height 4,097 reachable on 2 MiB
   and it is a legitimate finding that it is out of reach.

4. **The guard calibration design.** Given your floor from (3), propose the
   defaults and the structure that ties them to the ruled stack. I want the
   relationship to be named and testable, not a constant someone picked: a
   declared minimum-supported-stack constant, the derived byte budget, the
   headroom argument covering the poll stride (up to four levels of overshoot
   between polls) and the non-evaluator frames below the base. Say explicitly
   what happens to `DEFAULT_RECURSION_LIMIT` and which of the three ways out
   above you are choosing, and why.

5. **The falsifier.** Per the standing rule, a check nobody has watched fail is
   not known to work. Say exactly which test would have caught the present
   defect, and what makes it fire. A checked-in test that asserts the
   arithmetic relation between the declared stack, the derived budget, and the
   poll-stride headroom is the shape I expect; if you propose something
   better, argue for it. A `#[should_panic]` proof that runs an actual
   overflow is **not** acceptable here — it would take the test binary with
   it.

6. **The ripple inventory.** Every test, doc, message tag, host profile, and
   public constant whose behaviour depends on the current defaults. In
   particular: which tests today reach the depth limit before the byte limit
   and would flip from the `reclim` message to `reslim` under your proposal,
   and whether that flip is correct or is a behaviour regression that has to
   be avoided. Enumerate them; do not sample.

7. **Drafted implementation briefs** in `docs/dev/codex/TEMPLATE.md` shape,
   one per step, each with its own Scope, gate, A/B panel where a hot path is
   touched, and stop-loss. These are drafts; I rewrite them before dispatch.

## Scope

- Files you MAY modify: **NONE.** Append your report to this brief only.
- Write no Rust, add no test, run no benchmark.

## Hard rules

- **Do not reproduce a stack overflow.** Do not write, generate, or run any
  program whose purpose is to exhaust the stack, and do not run the deep
  constructions quoted above. Every number you need from such a run is already
  in this brief. Reading the compiled binary with `objdump`/`nm`, reading
  source, and reading existing test code are the tools for this brief.
- Do NOT `git commit`, branch, push, or touch git config.
- No network. No third-party engine, oracle, or comparison corpus of any kind
  (`AGENTS.md` Testing Policy).
- Do not propose adding `unsafe` to `rum-core` or `rum-runtime`; both are
  `unsafe`-free today and that is a hard line. This rules out reading the
  thread's real stack bounds via platform FFI, which is why the budget has to
  be declared.
- Do not propose a default-path cost for diagnostics: any instrumentation must
  be behind the env-gated cold-load `AtomicU8` pattern used elsewhere.
- If you conclude a deliverable cannot be established from source and static
  binary reading, say so and name the non-crashing measurement that would
  settle it. Do not guess and do not pad.

## Report format

Append to this file, under a `## Appended design report` heading, in the order
of the deliverables above. Every claim carries `path:line`. Where you give a
projected number, mark it as a projection and give the reasoning that bounds
it.
