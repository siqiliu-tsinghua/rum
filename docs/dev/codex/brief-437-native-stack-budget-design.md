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

## Appended design report

### 1. Complete per-level frame inventory

#### Method and result

I inspected the existing `target/release/rum-cli`; I did not rebuild it or run
an evaluator probe. The permitted artifact and prologue-counting convention are
the ones named by this brief
(`docs/dev/codex/brief-437-native-stack-budget-design.md:54-55,181-185`). The
artifact is an x86-64 ELF with build ID
`6aa8dd01d93376c33548efdf24dbac8472b4cf56`, has a symbol table, and has no
DWARF debug sections. These are the reproduction commands:

```text
file target/release/rum-cli
readelf -n target/release/rum-cli
readelf -S target/release/rum-cli | rg '\.(debug|symtab)'
nm -S -n -C target/release/rum-cli | rg 'evaluate_once_unstable|evaluate_inner_unstable|evaluate_atom_fast_path|resume_call_after_head|resume_call_with_dispatch_plan|prepare_arguments_for_call_with_unevaluated_restore|run_call_dispatch_stage|build_call_dispatch_plan|try_dispatch_builtin_call|resolution.*Session>::evaluate$'
objdump -d --demangle --start-address=0x45b6d0 --stop-address=0x45b6f0 target/release/rum-cli
# Repeat objdump at each release address in the inventory table.
objdump -R target/release/rum-cli | rg 'a1e448|4f75e0'
```

For each separate native function I counted every leading register `push` as
8 bytes and added the first `sub $N,%rsp`, exactly as the supplied inventory
did. I then inspected each recursive call site to check that it made no extra
`%rsp` adjustment outside the caller's fixed frame. The source edges are the
fixed-point call at `crates/rum-runtime/src/runtime/resolution.rs:339-343`, the
inner-evaluator call at
`crates/rum-runtime/src/runtime/eval_core/mod.rs:51-52`, the call dispatcher at
`crates/rum-runtime/src/runtime/eval_core/call.rs:68`, the plan driver at
`crates/rum-runtime/src/runtime/eval_core/call.rs:98-99`, argument preparation
at `crates/rum-runtime/src/runtime/eval_core/call.rs:153-169`, and recursive
argument evaluation at
`crates/rum-runtime/src/runtime/eval_call_prepare.rs:209-220` and
`crates/rum-runtime/src/runtime/eval_core/mod.rs:140-145`.

The simultaneously resident packet from one `Session::evaluate` entry to the
next nested `Session::evaluate` entry is:

| order | release address | resident function | source edge / source body | prologue calculation | bytes |
|---:|---:|---|---|---:|---:|
| 1 | `0x4f75e0` | `Session::evaluate` | Captures the base only at outer depth, then enters the fixed-point driver (`crates/rum-runtime/src/runtime/resolution.rs:288-303,305-375`). | 6 pushes + `0x2b8` | 744 |
| 2 | `0x45b6d0` | `evaluate_once_unstable_after_input_check_with_cycle_tracking` | Enforces depth/byte guards and calls the inner evaluator (`crates/rum-runtime/src/runtime/eval_core/mod.rs:14-64`). | 6 pushes + `0xd8` | 264 |
| 3 | `0x45bd10` | `evaluate_inner_unstable` | Its call arm invokes `evaluate_call`; that source function is inlined into this frame in the inspected release binary (`crates/rum-runtime/src/runtime/eval_core/mod.rs:98-134`; `crates/rum-runtime/src/runtime/eval_core/call.rs:21-74`). | 6 pushes + `0x3f8` | 1,064 |
| 4 | `0x457050` | `resume_call_after_head` | Builds the dispatch plan/state and calls the plan driver (`crates/rum-runtime/src/runtime/eval_core/call.rs:87-109`). | 6 pushes + `0x248` | 632 |
| 5 | `0x4575d0` | `resume_call_with_dispatch_plan` | The `PrepareArguments` stage calls the preparation driver while retaining plan state (`crates/rum-runtime/src/runtime/eval_core/call.rs:111-200`). | 6 pushes + `0x228` | 600 |
| 6 | `0x453bc0` | `prepare_arguments_for_call_with_unevaluated_restore` | The `Evaluate` argument mode calls the atom fast path (`crates/rum-runtime/src/runtime/eval_call_prepare.rs:190-220`). | 6 pushes + `0x388` | 952 |
| 7 | `0x45e2c0` | `evaluate_atom_fast_path` | A call-shaped argument re-enters `Session::evaluate` (`crates/rum-runtime/src/runtime/eval_core/mod.rs:136-155`). | 5 pushes + `0x180` | 424 |
|  |  | **resident prologue subtotal** | The seven source edges above form one recursive construction level. |  | **4,680** |

There are seven x86-64 `call` edges from one packet boundary to the next, hence
seven live 8-byte return addresses. The complete static packet is therefore
`4,680 + 7 * 8 = 4,736 B = 4.625 KiB`. That lands inside the supplied
approximately 4.7 KiB slope, which was independently derived from three stack
sizes and two heads (`docs/dev/codex/brief-437-native-stack-budget-design.md:41-52`).

`evaluate_profiled_fixed_point` and `evaluate_call` have no separate symbols
in this artifact: their source bodies are folded into `Session::evaluate` and
`evaluate_inner_unstable`, respectively. Counting them again would double-count
their locals (`crates/rum-runtime/src/runtime/resolution.rs:305-375`;
`crates/rum-runtime/src/runtime/eval_core/call.rs:21-74`). The indirect call at
`evaluate_atom_fast_path+0x2e5` uses GOT slot `0xa1e448`; the relocation resolves
that slot to `0x4f75e0`, closing the packet back to `Session::evaluate`; the
corresponding source recursion is explicit at
`crates/rum-runtime/src/runtime/eval_core/mod.rs:140-145`.

#### The missing 1.2 KiB

The earlier five-frame list included the 328-byte
`run_call_dispatch_stage`, but that frame is not resident during strict
argument descent: the plan driver selects argument preparation at
`crates/rum-runtime/src/runtime/eval_core/call.rs:153-169`, and only selects
`run_call_dispatch_stage` after the child result returns at
`crates/rum-runtime/src/runtime/eval_core/call.rs:169-177`. The earlier list also
omitted the resident 264-byte `evaluate_once...`, 632-byte
`resume_call_after_head`, and 600-byte `resume_call_with_dispatch_plan` frames
(`crates/rum-runtime/src/runtime/eval_core/mod.rs:14-64`;
`crates/rum-runtime/src/runtime/eval_core/call.rs:87-200`). The reconciliation is
exact at prologue granularity:

```text
previous subtotal                         3,512 B
- transient run_call_dispatch_stage        -328 B
+ evaluate_once...                          264 B
+ resume_call_after_head                    632 B
+ resume_call_with_dispatch_plan            600 B
                                           -------
resident prologue subtotal                4,680 B
+ seven return addresses                     56 B
                                           -------
complete recursive packet                 4,736 B
```

Thus the requested approximately 1.2 KiB is the exact net `1,496 - 328 =
1,168 B` in the three omitted resident frames; return addresses account for a
further 56 B not represented by any prologue (`docs/dev/codex/brief-437-native-stack-budget-design.md:57-67`).

#### Frames invoked per source level but not resident in the recursive packet

These are important for endpoint/intercept stack use, but adding them to every
level would be the same error as adding `run_call_dispatch_stage` to the slope.
For `Power`, the catalog supplies `Listable` plus ordinary dispatch, so the plan
contains preparation, listable dispatch, and ordinary builtin dispatch
(`crates/rum-stdlib/src/catalog/arithmetic.rs:37-46`;
`crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:64-86`).

| release address | function | prologue | why it is not in the per-level resident subtotal |
|---:|---|---:|---|
| `0x45e880` | `build_call_dispatch_plan` | 296 B | It returns before `resume_call_with_dispatch_plan` is entered (`crates/rum-runtime/src/runtime/eval_core/call.rs:92-99`; `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:42-100`). |
| `0x628320` | `plan_call_argument_preparation` | 56 B | It returns a `CallArgumentPlan` before `evaluate_atom_fast_path` descends (`crates/rum-runtime/src/runtime/call_preparation.rs:115-168`; `crates/rum-runtime/src/runtime/eval_call_prepare.rs:203-214`). |
| `0x45ef30` | `run_call_dispatch_stage` | 328 B | It runs on ascent after prepared arguments are available, not while the next construction level is being entered (`crates/rum-runtime/src/runtime/eval_core/call.rs:153-177`; `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:102-158`). |
| `0x45f4f0` | the `BuiltinSpec` monomorph of `try_dispatch_builtin_call` | 616 B | The ordinary arm restores/ends the stage frame and tail-jumps to this function in the inspected binary; source dispatch begins only after preparation (`crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:153-178`). |
| `0x24e990` | `rum_stdlib::arithmetic::eval_power` | 8 B | The handler is reached during ordinary dispatch on ascent; its source is only a thin data-dispatch wrapper (`crates/rum-stdlib/src/arithmetic.rs:46-52`). |

Those endpoint frames contribute to the measured ceiling's non-slope overhead;
they must not be multiplied by depth. The supplied measurements bracket each
transition rather than identify its exact level, and they were not rerun
(`docs/dev/codex/brief-437-native-stack-budget-design.md:41-52`).

### 2. Where the bytes come from in source terms

The existing binary has no DWARF, so source plus optimized disassembly can
identify live values, branch unions, inlining, and return areas, but cannot
prove an exact byte partition among Rust locals. I do not assign invented byte
counts to individual locals. A non-crashing way to settle a disputed slot is a
release-equivalent build in a separate target directory with full debuginfo,
followed by DWARF location/disassembly inspection and one-source-variable
outline variants; comparing the same prologue after each variant gives the
marginal contribution without approaching stack exhaustion. If optimized DWARF
still coalesces the slots, shallow (depth at most 16) entry markers may be added
behind the existing env-gated `AtomicU8` cold-load pattern, never on the default
path (`crates/rum-runtime/src/runtime/profiling/state.rs:19-31`;
`docs/dev/codex/brief-437-native-stack-budget-design.md:193-197`).

Two types recur across the large frames. `Expr` is an owned enum whose call arm
contains shared head and argument handles (`crates/rum-core/src/expr.rs:749-753`).
`EvalError` is a wide public error union; its largest visible variant carries
four `Expr` values plus flags, while other variants carry vectors, strings, or
an input request (`crates/rum-runtime/src/runtime/session/session_state.rs:498-574`).
Consequently every `Result<Expr, EvalError>` return area must accommodate the
largest error variant even on the successful strict-call path. The disassembly
copies a `0x88`-byte result area at several call boundaries; the source reason
for that wide union is the `ThrownBypassingCatch` payload at
`crates/rum-runtime/src/runtime/session/session_state.rs:549-559`.

Profiling tokens are another cross-cutting contributor. Each
`RuntimePathTimingToken` owns an `Instant` and a full
`RuntimePathTimingKey`; the key contains a phase string and optional `Symbol`
(`crates/rum-runtime/src/runtime/profiling/mod.rs:411-415,469-472`). The disabled
path returns `None`, but the caller's optimized stack layout still reserves the
union for every token local created around a nested result
(`crates/rum-runtime/src/runtime/profiling/timing.rs:96-125,374-414`).

#### Resident frames above 300 B

| frame | source attribution, including cold branches paid by the frame union |
|---|---|
| `Session::evaluate` — 744 B | The inlined fixed-point driver retains `current`, `evaluation_passes`, and `fixed_point_seen`, then creates timing tokens and a wide `next_result` (`crates/rum-runtime/src/runtime/resolution.rs:305-348`). Its cold exits construct cycle/pass errors and, for input, a `SuspendedFixedPointState` containing an `Expr`, count, and frozen history (`crates/rum-runtime/src/runtime/resolution.rs:21-26,310-357`). |
| `evaluate_inner_unstable` — 1,064 B | The outer total token/result spans a match over atom, symbol, and call arms (`crates/rum-runtime/src/runtime/eval_core/mod.rs:98-134`). The inlined call arm adds four timing tokens around head evaluation, a wide `evaluated_head_result`, input suspension/error branches, and the final call result (`crates/rum-runtime/src/runtime/eval_core/call.rs:21-73`). LLVM also lays this function out for the symbol arm: `$ContextPath`, `$Packages`, and `$Path` collect `Vec<Expr>` values, and lazy package loading carries another result even though the strict `Power` path never selects that arm (`crates/rum-runtime/src/runtime/eval_core/symbol.rs:10-69`). |
| `resume_call_after_head` — 632 B | Three timing-token lifetimes surround a by-value `CallDispatchPlan`, `CallDispatchState`, and result (`crates/rum-runtime/src/runtime/eval_core/call.rs:87-108`). The plan embeds an optional `BuiltinSpec` and a fixed seven-stage array, while the state retains raw and optionally prepared shared arguments (`crates/rum-runtime/src/runtime/session/session_state.rs:36-44,176-180`). |
| `resume_call_with_dispatch_plan` — 600 B | By-value plan/state/restore data stay live across the selected stage (`crates/rum-runtime/src/runtime/eval_core/call.rs:111-119,138-181`). Cold union members include a mutable `Vec<Expr>` for restoring `Unevaluated` wrappers and the input-required conversion to a suspension frame carrying plan, state, index, and restore data (`crates/rum-runtime/src/runtime/eval_core/call.rs:120-135,192-196,383-391`). |
| `prepare_arguments_for_call_with_unevaluated_restore` — 952 B | The loop retains `head`, raw arguments, an owned `CallArgumentPlan`, the prepared argument/result, and optional `built` and `unevaluated_restores` vectors (`crates/rum-runtime/src/runtime/eval_call_prepare.rs:190-220,262-339`). Although ordinary `Power` does not strip `Unevaluated`, the union includes wrapper/splice bookkeeping (`crates/rum-runtime/src/runtime/eval_call_prepare.rs:239-259`). Its input branch builds the large suspended state containing head, args, cursor, a persistent prepared accumulator, and optional restore accumulator (`crates/rum-runtime/src/runtime/eval_call_prepare.rs:32-40,221-234`; `crates/rum-runtime/src/runtime/call_preparation.rs:26-50`). |
| `evaluate_atom_fast_path` — 424 B | Two timing tokens and the wide result live across the classification match; the call arm recursively owns the result of `Session::evaluate` (`crates/rum-runtime/src/runtime/eval_core/mod.rs:136-154`). The classifier itself is small and only distinguishes stable symbol, unstable symbol, other atom, and call (`crates/rum-runtime/src/runtime/eval_core/mod.rs:158-186`). |

The 264-byte `evaluate_once...` frame is below the requested approximately
300-byte attribution threshold. Its material state is the saved evaluation
depth, optional cycle-stack bookkeeping, the nested result, and two cold guard
errors (`crates/rum-runtime/src/runtime/eval_core/mod.rs:14-64`). The actual
native-stack poll is already `#[cold] #[inline(never)]` and has no material
fixed prologue in this artifact (`crates/rum-runtime/src/runtime/eval_core/mod.rs:66-78`).

#### Large transient frames

The 328-byte `run_call_dispatch_stage` union spans special form, argument
preparation, list threading, up/down/subvalues, builtin, and function branches.
The listable arm can own a threaded expression/result and its cold input
suspension even though this `Power` instance takes `NotApplicable`
(`crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:102-170`). The 616-byte
`try_dispatch_builtin_call` monomorph includes profiling state and the
env-gated memory-growth validation branch, including cloned arguments for a
suspension frame (`crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:173-239`).
They affect constant endpoint headroom, not the 4,736-byte recursive slope.

### 3. Reachable floor

The following are projections, not measurements. Each range is bounded by the
current 4,680-byte prologue subtotal and by how many simultaneously live
source objects the change can remove; optimized-stack-slot reuse means the
ranges overlap and must not be added as independent certainties
(`crates/rum-runtime/src/runtime/profiling/mod.rs:411-472`;
`crates/rum-runtime/src/runtime/session/session_state.rs:498-574`).

| lever | projected packet saving | reason for the projection |
|---|---:|---|
| Make `RuntimePathTimingToken` a one-word depth cookie and move its `Instant`/key into the already-existing TLS timing frame. | **Projection: 0.55-0.85 KiB** | There are multiple live full tokens in `evaluate`, the inlined call arm, `resume_call_after_head`, preparation, and the atom fast path; the TLS stack already stores a key and child time (`crates/rum-runtime/src/runtime/profiling/mod.rs:452-472`; `crates/rum-runtime/src/runtime/profiling/timing.rs:96-125,374-414`). A cookie preserves nesting validation without putting the full token in every evaluator frame. |
| Split the call/symbol/atom implementation so unrelated branch unions do not share `evaluate_inner_unstable`. | **Projection: 0.25-0.50 KiB** | Only one arm runs, but the current 1,064-byte frame covers call-head results plus symbol-only vector collections and package-load state (`crates/rum-runtime/src/runtime/eval_core/mod.rs:98-134`; `crates/rum-runtime/src/runtime/eval_core/symbol.rs:10-84`; `crates/rum-runtime/src/runtime/eval_core/call.rs:21-74`). |
| Outline input-suspension construction and final `Unevaluated` restoration behind `#[cold] #[inline(never)]`. | **Projection: 0.35-0.65 KiB** | The strict path never constructs the fixed-point, argument-preparation, or dispatch suspension states and never restores stripped wrappers, but their values share the large hot frames (`crates/rum-runtime/src/runtime/resolution.rs:350-357`; `crates/rum-runtime/src/runtime/eval_call_prepare.rs:221-234`; `crates/rum-runtime/src/runtime/eval_core/call.rs:120-135,192-196`). |
| Use a compact private evaluator-error carrier, boxing only on an error path, and convert to the existing public `EvalError` at the request boundary. | **Projection: 0.35-0.65 KiB** | This can shrink repeated success-path result areas, but savings overlap with outlined branches. Changing the public `EvalError` variant fields is rejected because `EvalError` is publicly re-exported (`crates/rum-runtime/src/runtime/mod.rs:70-73`) and its largest payload is part of that API (`crates/rum-runtime/src/runtime/session/session_state.rs:541-574`). This lever needs a separate design before implementation. |
| Remove the call-only atom wrapper or combine private plan/driver state after the preceding changes. | **Projection: 0.25-0.45 KiB** | The 424-byte wrapper exists principally to retain two profile tokens and a result around a recursive call, while two adjacent plan frames retain overlapping plan/state/result data (`crates/rum-runtime/src/runtime/eval_core/mod.rs:136-155`; `crates/rum-runtime/src/runtime/eval_core/call.rs:87-177`). |

After overlap, the plausible local-optimization floor is **projection:
1.5-2.6 KiB per recursive level**. The lower end assumes the full-token and
wide-result duplication is removed and cold state is successfully outlined;
the upper end assumes LLVM must retain separate fixed-point, dispatch, and
argument-preparation state. Even at the lower end, each live level still needs
an owned current expression/history, call plan/state, preparation cursor and
accumulators, plus ABI spills (`crates/rum-runtime/src/runtime/resolution.rs:305-310`;
`crates/rum-runtime/src/runtime/eval_core/call.rs:111-177`;
`crates/rum-runtime/src/runtime/eval_call_prepare.rs:190-220`). Only static
remeasurement after each isolated change can narrow that range.

No local combination gets near 512 B. The absolute zero-reserve allowance for
height 4,097 on 2 MiB is `2,097,152 / 4,097 = 511.875 B/level`; any real guard
reserve makes the allowance smaller. The ruled height and stack are fixed at
`docs/dev/codex/brief-426-height-cache-spike.md:25-30`, and the 4,097 certificate
harness actually provisions 2 MiB at
`crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:3-12`.

Making strict residual height 4,097 reachable therefore requires eliminating
proportional native recursion, not polishing these frames: an explicit heap
task stack/trampoline must retain fixed-point, plan, preparation, and
suspension continuation state. The runtime already demonstrates iterative
frame driving for suspended evaluation and an owned work-stack traversal
(`crates/rum-runtime/src/runtime/suspension/core.rs:601-650`;
`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:70-106`). Turning
ordinary evaluation into that shape is an evaluator-architecture project, not
a safe mechanical follow-up to this brief; it must preserve the existing
suspend/resume frames at `crates/rum-runtime/src/runtime/eval_call_prepare.rs:32-45`
and `crates/rum-runtime/src/runtime/eval_core/call.rs:383-391`.

### 4. Guard calibration design

#### Named relationship

Adopt these public calibration constants in `rum-runtime` and re-export them
from `rum-host`, whose public resource configuration currently forwards the
runtime value (`crates/rum-runtime/src/runtime/model/config.rs:3-31`;
`crates/rum-host/src/config.rs:55-117`):

| name | value | role |
|---|---:|---|
| `MIN_SUPPORTED_NATIVE_STACK_BYTES` | `2 * 1024 * 1024 = 2,097,152` | Smallest declared host-thread stack supported by the default. The ruling names 2 MiB as the embedder floor (`TODO.md:980-995`). |
| `NATIVE_STACK_NON_EVALUATOR_RESERVE_BYTES` | `512 * 1024 = 524,288` | Capacity for caller/host frames below `native_stack_base`, the root-frame portion before the marker, the poll/error/drop path, and measurement intercept (`crates/rum-runtime/src/runtime/resolution.rs:288-294`; `crates/rum-runtime/src/runtime/suspension/core.rs:482-510`). |
| `NATIVE_STACK_UNPOLLED_LEVEL_RESERVE_BYTES` | `128 * 1024 = 131,072` | Conservative allowance for one level between byte polls. It is over 6x the historical approximately 20 KiB user-recursion cost and about 2x the historical approximately 65 KiB debug physical-level observation (`PROGRESS.md:621-628`; `docs/dev/codex/brief-422-xml-depth-and-the-class.md:348-352`). |
| `NATIVE_STACK_POLL_HEADROOM_BYTES` | `NATIVE_STACK_DEPTH_POLL_STRIDE * NATIVE_STACK_UNPOLLED_LEVEL_RESERVE_BYTES = 512 * 1024` | Covers all four levels between checks; the stride is four at `crates/rum-runtime/src/runtime/metadata.rs:5-7`, and the next check is advanced by that stride at `crates/rum-runtime/src/runtime/eval_core/mod.rs:39-42`. |
| `DEFAULT_NATIVE_STACK_LIMIT` | `MIN_SUPPORTED_NATIVE_STACK_BYTES - NATIVE_STACK_NON_EVALUATOR_RESERVE_BYTES - NATIVE_STACK_POLL_HEADROOM_BYTES = 1,048,576` | Exactly 1 MiB of evaluator-consumed stack on the minimum supported 2 MiB thread. It replaces the unrelated 6 MiB literal at `crates/rum-runtime/src/runtime/metadata.rs:5-7`. |

The relationship is therefore:

```text
1,048,576 default evaluator budget
+ 524,288 four-poll reserve
+ 524,288 host/base/emergency reserve
= 2,097,152 minimum supported declared stack
```

The 128 KiB per-level value is a conservative calibration allowance, not a
claim that Rust's type system proves a global maximum frame. On the measured
strict packet, four-level overshoot is only `4 * 4,736 = 18,944 B`; on the
historical approximately 20 KiB user-recursion path it is approximately 80
KiB, so the 512 KiB poll reserve is respectively about 27x and 6.4x those
observations (`docs/dev/codex/brief-437-native-stack-budget-design.md:41-67`;
`PROGRESS.md:621-628`). A future packet above 128 KiB is a calibration failure
to report, not a reason to silently increase the minimum-stack promise.

The supplied last-success ceilings give conservative residuals when the
4.625 KiB packet is subtracted: `1 MiB - 202 * 4.625 KiB = 89.75 KiB`,
`2 MiB - 421 * 4.625 KiB = 100.875 KiB`, and
`4 MiB - 875 * 4.625 KiB = 49.125 KiB`. The 512 KiB non-evaluator reserve is
more than five times the largest of those residuals; these are upper residuals,
not exact host-frame measurements, because the supplied ceiling intervals are
16 levels wide (`docs/dev/codex/brief-437-native-stack-budget-design.md:41-52`).

Add a public `const fn native_stack_limit_for_declared_stack(stack_bytes:
usize) -> Option<usize>`. It returns `None` below
`MIN_SUPPORTED_NATIVE_STACK_BYTES`; otherwise it subtracts the two named
reserves with checked arithmetic. `ResourceLimits::default()` uses the exact
derived 1 MiB value, while an embedder that deliberately provisions a larger
worker stack can derive a correspondingly larger limit instead of copying a
magic literal (`crates/rum-runtime/src/runtime/model/config.rs:20-31`;
`crates/rum-host/src/config.rs:85-117`).

This is a declared-stack contract, not stack discovery. `Session::evaluate`
captures an address only after entering the evaluator and cannot see capacity
already consumed by arbitrary host recursion (`crates/rum-runtime/src/runtime/resolution.rs:288-294`).
Safe Rust has no portable API for discovering the thread bounds, and this brief
forbids platform FFI/`unsafe` (`docs/dev/codex/brief-437-native-stack-budget-design.md:115-117,189-192`).
The documented guarantee must therefore be: the default is calibrated for a
thread with at least 2 MiB and an ordinary shallow entry; a host entering after
substantial native recursion must lower the budget, and a host raising the
budget must provision/declare the matching worker stack. The host guide already
recommends one session per worker thread (`docs/guide/embedding.md:453-462`).

The same derivation applies to resumed requests because resumption captures a
fresh base and schedules its first poll relative to the restored evaluation
depth (`crates/rum-runtime/src/runtime/suspension/core.rs:482-510`). Do not cache
a process-global base or attempt an OS query.

#### What happens to recursion depth

Keep `DEFAULT_RECURSION_LIMIT = 1024` unchanged. Choose the third option in the
brief: the logical limit is a ceiling, while the byte guard may correctly bind
first on a small declared stack (`crates/rum-runtime/src/runtime/metadata.rs:5-7`;
`docs/dev/codex/brief-437-native-stack-budget-design.md:104-113`). A byte-derived
logical limit would be dishonest because the known strict path is 4.625 KiB per
level while the historical user-recursion path is approximately 20 KiB per
level (`docs/dev/codex/brief-437-native-stack-budget-design.md:41-67`;
`PROGRESS.md:621-628`).

With the proposed 1 MiB evaluator budget, the present strict packet reaches the
raw quotient at about 221 levels and the strided check at about 224 levels.
After the projected 1.5-2.6 KiB local floor, the byte budget would cover only
about 394-683 levels. These are projections from the static packet/floor, not
promised semantic depths. Both are below 1024, so documentation must say
explicitly that 1024 is the maximum logical count, not a guarantee that every
evaluation shape reaches it (`crates/rum-runtime/src/runtime/eval_core/mod.rs:34-43`;
`docs/guide/evaluation.md:261-273`).

This also makes strict residual height 4,097 intentionally unreachable through
the current native-recursive route: the proposed 1 MiB evaluator budget permits
only `1,048,576 / 4,097 = 255.94 B` per physical level. The result must be a
recoverable `StackLimit`/`General::reslim`, never an overflow; reaching 4,097
requires the explicit-task-stack redesign identified above
(`crates/rum-runtime/src/runtime/session/session_state.rs:519-524`;
`crates/rum-runtime/src/runtime/session/session_helpers.rs:322-355`).

### 5. Falsifier

Add a pure arithmetic unit test named
`default_native_stack_budget_is_derived_from_minimum_supported_stack` beside
the existing default-configuration test. It performs no recursive evaluation
and cannot overflow (`crates/rum-runtime/src/runtime/tests/session_config.rs:3-11`):

```rust
assert_eq!(
    NATIVE_STACK_POLL_HEADROOM_BYTES,
    NATIVE_STACK_DEPTH_POLL_STRIDE * NATIVE_STACK_UNPOLLED_LEVEL_RESERVE_BYTES,
);
assert_eq!(
    DEFAULT_NATIVE_STACK_LIMIT
        + NATIVE_STACK_POLL_HEADROOM_BYTES
        + NATIVE_STACK_NON_EVALUATOR_RESERVE_BYTES,
    MIN_SUPPORTED_NATIVE_STACK_BYTES,
);
assert_eq!(
    native_stack_limit_for_declared_stack(MIN_SUPPORTED_NATIVE_STACK_BYTES),
    Some(DEFAULT_NATIVE_STACK_LIMIT),
);
assert_eq!(
    native_stack_limit_for_declared_stack(MIN_SUPPORTED_NATIVE_STACK_BYTES - 1),
    None,
);
assert_eq!(
    ResourceLimits::default().native_stack_limit,
    DEFAULT_NATIVE_STACK_LIMIT,
);
```

To satisfy the standing falsifier rule, introduce the relationship/constants
while temporarily leaving the current 6 MiB default selected, run only this
test, and preserve the failure showing that `6 MiB + 1 MiB != 2 MiB`. Then make
`DEFAULT_NATIVE_STACK_LIMIT` use the derived 1 MiB value and rerun it green.
This catches the present defect because any default detached from the declared
stack and its two reserves violates the equality before evaluator code runs
(`crates/rum-runtime/src/runtime/metadata.rs:4-11`;
`docs/dev/codex/brief-437-native-stack-budget-design.md:154-160`).

Add a host-side propagation assertion that `HostResourceLimits::default()` and
all four profiles (`default`, `sandboxed`, `hermetic`, `scripting`) expose that
same derived value. The profiles currently all descend from `HostConfig::default`
(`crates/rum-host/src/config.rs:85-117,153-212`), while existing profile tests
only pin capabilities/providers (`crates/rum-host/src/tests/capability_profiles.rs:5-27`).

The two existing 512 KiB tests prove that the guard can return `reslim` and
leave the session usable, but neither can catch a bad default because both
override it (`crates/rum-runtime/src/runtime/tests/holding_and_control_part2.rs:466-485`;
`crates/rum-stdlib/src/tests/stress_corpus.rs:73-88`). They remain valuable
mechanism tests; they are not the calibration falsifier.

### 6. Ripple inventory

#### Constants, configuration, and host profiles

- The only current numeric definition is crate-private
  `DEFAULT_NATIVE_STACK_LIMIT`; `ResourceLimits::default()` makes it publicly
  observable. Change it to the derived public constant, expose the calibration
  constants/helper through the existing runtime re-export, and leave every
  other resource default unchanged
  (`crates/rum-runtime/src/runtime/metadata.rs:4-11`;
  `crates/rum-runtime/src/runtime/mod.rs:70-73`;
  `crates/rum-runtime/src/runtime/model/config.rs:3-31`).

- `Session::empty` installs `ResourceLimits::default`, and `Session::new` is
  built from that state. Both automatically inherit the new value
  (`crates/rum-runtime/src/runtime/session/session_state.rs:576-636,680-692`;
  `crates/rum-runtime/src/runtime/session/session_api.rs:63-71`). The existing
  setters stay source-compatible, but their rustdoc must state the declared
  stack precondition and point at the helper
  (`crates/rum-runtime/src/runtime/session/session_api.rs:98-115`).

- `HostResourceLimits::default` converts the runtime defaults in both
  directions. Its 6 MiB/8 MiB rustdoc is stale and must describe the 2 MiB
  minimum, derived 1 MiB budget, and opt-up helper
  (`crates/rum-host/src/config.rs:55-117`). `HostConfig::default`, `sandboxed`,
  `hermetic`, and `scripting` all inherit it; none should override the stack
  budget independently (`crates/rum-host/src/config.rs:153-212,310-319`).

- `HostSessionBuilder` and `HostSession::new` select `HostConfig::default`, so
  all embedding callers inherit the safe default
  (`crates/rum-host/src/session.rs:29-45,126-148`). `rum-cli`/the REPL use the
  scripting profile, and the Jupyter launcher uses `HostSession::new`; both
  therefore change without local constants
  (`crates/rum-cli/src/main.rs:208-218`;
  `crates/rum-jupyter/src/launcher.rs:67-76`).

- There is no current public numeric default constant: the constant is
  `pub(super)`, while the public API is the `ResourceLimits`/`HostResourceLimits`
  field and `Default` behavior (`crates/rum-runtime/src/runtime/metadata.rs:4-7`;
  `crates/rum-runtime/src/runtime/model/config.rs:3-31`;
  `crates/rum-host/src/config.rs:55-117`). The new public constants/helper are
  therefore an additive API, not a rename of an existing public item.

#### Limit-sensitive tests (exhaustive direct inventory)

- `default_session_configuration_matches_default_session_state` currently
  proves only two defaults agree and is tautological with respect to the 6 MiB
  number. Extend this file with the arithmetic falsifier; the other tests use
  explicit sentinel values 19, 15, and 102 and remain unchanged
  (`crates/rum-runtime/src/runtime/tests/session_config.rs:3-18,47-57,75-108,133-144`).

- `holding_and_control_failure_edges` pins the public recursion default at
  1024, which stays correct. Its three recursion-limit scenarios explicitly use
  32 and should continue to produce `reclim`; a different result is a
  regression (`crates/rum-stdlib/src/tests/holding_and_control_failure_edges.rs:189-205,224-238,257-280`).

- Runtime control tests use recursion 32, explicit native budget 512 KiB, and
  a bounded success depth 48. The first two classifications must remain
  `reclim` and `reslim`; the depth-48 success is a required guard-adjacent
  regression gate under the new default
  (`crates/rum-runtime/src/runtime/tests/holding_and_control_part2.rs:446-505`).

- Runtime request tests use recursion 32 or native budget 512 KiB and assert
  both structured termination and file-request error mapping. Their expected
  tags/reasons stay unchanged
  (`crates/rum-runtime/src/runtime/tests/session_requests/api.rs:259-295,734-752`).
  The loader rollback test also uses recursion 32 and deliberately hides the
  inner `reclim` behind `Get::load`; that behavior must remain unchanged
  (`crates/rum-runtime/src/runtime/tests/context_and_loader.rs:1044-1070`).

- The host recursion contract sets 32 and must keep returning structured
  `HostTerminationReason::RecursionLimit`
  (`crates/rum-host/src/tests/eval_contract.rs:165-183`). The lifecycle test's
  value 77 checks configuration persistence only and remains unchanged
  (`crates/rum-host/src/tests/session_lifecycle.rs:238-290`).

- The two guard-adjacent stdlib tests use recursion 96 at success depth 72 and
  failure depth 120. **Projection:** using the historical approximately 20 KiB
  user-recursion cost, the new 1 MiB byte budget can bind at roughly 52 levels,
  so both can flip to `reslim` before the intended `reclim`
  (`crates/rum-stdlib/src/tests/stress_corpus.rs:44-70`;
  `PROGRESS.md:621-628`). That flip is a regression in these tests' stated
  recursion-boundary coverage, not a reason to expect `reslim`: lower the
  explicit logical limit and adjacent depths (for example 32 with below/above
  depths 24/40), then require success followed by `reclim`.

- The stdlib and runtime 512 KiB native-budget tests deliberately bind the byte
  guard first and remain `reslim` mechanism tests
  (`crates/rum-stdlib/src/tests/stress_corpus.rs:73-88`;
  `crates/rum-runtime/src/runtime/tests/holding_and_control_part2.rs:466-485`).

- The ignored Peano test provisions an 8 MiB thread, raises recursion to
  10,000, but silently inherits the current 6 MiB native default. It must set
  the budget with `native_stack_limit_for_declared_stack(8 * 1024 * 1024)` so
  the declared stack and budget remain coupled
  (`crates/rum-stdlib/src/tests/stress_corpus.rs:187-207`). The ignored host
  battered-session test hard-codes 6 MiB twice on its 8 MiB thread and should
  use the same helper; its explicit 32/512 KiB `reclim`/`reslim` phases remain
  semantically unchanged (`crates/rum-host/tests/stress.rs:290-324`).

- All 2 MiB iterative traversal certificates construct their session with the
  default and therefore must be rerun in release mode after calibration. They
  are designed to succeed at physical height 4,097 (two at 20,001), so an
  unexpected `StackLimit` is a regression/stop-loss, not a test expectation to
  rewrite (`crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:3-29,31-136`).
  The parser/lowering 2 MiB harness does not instantiate a session and is
  unaffected (`crates/rum-runtime/src/runtime/tests/left_chain_lowering.rs:375-393`).

- The debug XML acceptance test is the other known default-stack-sensitive
  case: it deliberately stops at physical depth 64 because the full semantic
  depth 256 costs too much evaluator stack on the approximately 2 MiB libtest
  thread (`crates/rum-stdlib/src/tests/codecs_xml.rs:297-314`). **Projection:**
  the derived 1 MiB evaluator budget may now return `StackLimit` before 64.
  That is correct option-three behavior for the default resource profile, but
  it would defeat this test's codec-acceptance purpose. If observed, preserve
  the codec contract by running the acceptance assertion on an explicitly
  provisioned larger thread with a budget from the declared-stack helper; do
  not lower the accepted depth or change the expected result to `reslim`. The
  one-past/far-past tests reject during codec parsing and retain their `FromXml::depth`
  contract (`crates/rum-stdlib/src/tests/codecs_xml.rs:316-334`).

- Two tokenizer stress tests start with `ResourceLimits::default()` and change
  unrelated generated/iteration budgets. They inherit the new stack default
  but do not assert or intentionally approach it; keep their results unchanged
  (`crates/rum-stdlib/src/tests/stress_corpus.rs:331-357`). Likewise the host and
  Jupyter long-running helpers start with `HostResourceLimits::default()` only
  to raise iteration work, so they require ordinary suite coverage but no
  expectation edit (`crates/rum-host/src/tests/common.rs:7-15`;
  `crates/rum-host/tests/stress.rs:45-51`;
  `crates/rum-jupyter/src/tests/mod.rs:48-57`).

No checked-in test expects the default 1024 recursion guard to win on a deep
evaluation. The only numeric default assertion is the 1024 field check above;
all `reclim` outcome tests explicitly set 32 or 96
(`crates/rum-stdlib/src/tests/holding_and_control_failure_edges.rs:189-205`;
`crates/rum-stdlib/src/tests/stress_corpus.rs:44-70`;
`crates/rum-runtime/src/runtime/tests/holding_and_control_part2.rs:446-463`;
`crates/rum-runtime/src/runtime/tests/session_requests/api.rs:259-295,734-743`;
`crates/rum-host/src/tests/eval_contract.rs:165-183`). The supplied uncommitted
default-stack depth probe would intentionally change from `reclim` to `reslim`
under option three; that is the public behavior being documented, not a
checked-in expectation (`docs/dev/codex/brief-437-native-stack-budget-design.md:32-39,104-113`).

An exhaustive tag search also finds many `General::reslim` assertions for
integer, generated-element, iteration-work, and memory budgets. They are not
stack-default dependencies because the mapper intentionally shares `reslim`
across those distinct error variants (`crates/rum-runtime/src/runtime/session/session_helpers.rs:322-335`).
Their configured budgets and termination reasons do not change.

#### Messages and structured outcomes

- Keep `EvalError::RecursionLimitExceeded` and `StackLimitExceeded`, their exact
  text, and `EvalTerminationReason::{RecursionLimit, StackLimit}` unchanged
  (`crates/rum-runtime/src/runtime/session/session_state.rs:517-524`;
  `crates/rum-runtime/src/runtime/model/requests.rs:56-85`).

- Keep `reclim` for logical recursion and `reslim` for native stack. The mapper
  and structured termination classifier already distinguish them correctly
  (`crates/rum-runtime/src/runtime/session/session_helpers.rs:322-355`). File
  request errors also retain separate variants
  (`crates/rum-runtime/src/runtime/session/session_requests.rs:702-713`).

- The host retains both structured termination variants and its existing
  conversion. A file-request `StackLimitExceeded` still maps to the coarse
  `HostFailureCode::Other`; this calibration brief does not redesign that
  separate host-failure taxonomy
  (`crates/rum-host/src/model/outcome.rs:41-64,338-351,661-700`).

- The `General` catalog descriptions, generated builtin reference, and message
  registry remain semantically correct and need no regeneration because no tag
  or template changes (`crates/rum-runtime/src/runtime/builtin_catalog/inert_heads.rs:38-49`;
  `docs/reference/builtins.md:2098-2114`;
  `docs/dev/message-registry-inventory.md:1201-1216`). The seeded-tag and selector
  tests likewise remain unchanged (`crates/rum-runtime/src/runtime/tests/messages.rs:248-259`;
  `crates/rum-runtime/src/runtime/tests/messages_part2.rs:287-310`).

#### Documentation

- Update the numeric table and the fail-closed promise in `SECURITY.md`; it
  currently says 6 MiB and promises all defaults terminate in-band
  (`SECURITY.md:43-60`).

- Update all three numeric/default occurrences and the transcript in the
  resource guide, and add the named equation, minimum-stack precondition, helper,
  and logical-ceiling explanation
  (`docs/guide/resource-limits-and-sandbox.md:13-30,74-94`). Its parser discussion
  already uses 2 MiB as a deployment floor and does not need a parser-limit
  change (`docs/guide/resource-limits-and-sandbox.md:111-130`).

- Strengthen the evaluation guide's already-correct statement that native
  stack may bind before recursion, making 1024 explicitly a ceiling
  (`docs/guide/evaluation.md:261-273`). Add the helper/precondition to the
  embedding configuration/threading sections
  (`docs/guide/embedding.md:49-76,453-462`).

- Add the minimum-stack/declared-budget contract to host hardening; its current
  warning correctly says panic catching cannot catch native overflow
  (`docs/host-embedding-and-hardening.md:52-71,92-94`). Update the publication
  audit's two default rows and safe transcript
  (`docs/dev/publication-audit/capability-consistency-findings.md:702-715`).

- Close or advance the calibration item in `TODO.md` without changing the
  separate ruling that physical height 4,097 remains the target
  (`TODO.md:980-999`). Do not rewrite `PROGRESS.md` or completed Codex briefs:
  their 6 MiB values are historical measurements, not current contract text
  (`PROGRESS.md:621-628`; `docs/dev/codex/brief-422-xml-depth-and-the-class.md:348-354`;
  `docs/dev/codex/brief-423-traversal-depth-design.md:13-14`;
  `docs/dev/codex/brief-424-traversal-height-brackets.md:30-35`;
  `docs/dev/codex/brief-436-construction-depth-class.md:852-861`).

### 7. Drafted implementation briefs

These drafts follow the required template shape; they are raw material for the
director, not authorization to implement them
(`docs/dev/codex/TEMPLATE.md:17-37,41-74`). The order is deliberate: make the
guard safe first, then take isolated frame reductions with static and timing
gates. The explicit-task-stack evaluator remains a new design-only project
because the current evidence is insufficient for a mechanical implementation
brief (`docs/dev/codex/TEMPLATE.md:19-30`;
`crates/rum-runtime/src/runtime/suspension/core.rs:601-650`).

Only the calibration, compact timing token, and cold-state outlining are
implementation-ready steps. Splitting the call/symbol union, introducing a
private compact error carrier, fusing plan drivers, or replacing recursive
evaluation with a task stack must be reconsidered against the new packet after
Drafts B/C; each changes broader ownership or control flow and therefore needs
a fresh design brief rather than a guessed implementation prescription
(`crates/rum-runtime/src/runtime/eval_core/mod.rs:98-155`;
`crates/rum-runtime/src/runtime/session/session_state.rs:498-574`;
`docs/dev/codex/TEMPLATE.md:19-37`).

#### Draft A — derive the native-stack default from the 2 MiB support floor

##### Goal

Make every default runtime/host profile use the named 2 MiB calibration
equation, keep logical recursion at 1024 as a ceiling, provide a safe helper for
hosts with declared larger stacks, preserve all message/termination semantics,
and watch the arithmetic falsifier fail on the old 6 MiB default before making
it pass (`crates/rum-runtime/src/runtime/metadata.rs:4-11`;
`crates/rum-runtime/src/runtime/session/session_helpers.rs:322-355`).

##### Context pointers

- Read `AGENTS.md:36-51,75-93,109-132`, this appended report, and the host/resource documentation
  before editing (`docs/guide/resource-limits-and-sandbox.md:13-94`;
  `docs/host-embedding-and-hardening.md:52-71`).
- Runtime defaults/guard: `crates/rum-runtime/src/runtime/metadata.rs:4-11`,
  `crates/rum-runtime/src/runtime/model/config.rs:3-31`, and
  `crates/rum-runtime/src/runtime/eval_core/mod.rs:34-78`.
- Host forwarding/profiles: `crates/rum-host/src/config.rs:55-117,153-212` and
  `crates/rum-host/src/session.rs:29-45,126-148`.
- Acceptance is the red/green arithmetic test, unchanged classification tests,
  and the existing release 2 MiB certificates
  (`crates/rum-runtime/src/runtime/tests/session_config.rs:3-11`;
  `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:3-136`).

##### Scope

- Files MAY modify:
  `crates/rum-runtime/src/runtime/metadata.rs`,
  `crates/rum-runtime/src/runtime/session/session_api.rs`,
  `crates/rum-runtime/src/runtime/tests/session_config.rs`,
  `crates/rum-host/src/lib.rs`, `crates/rum-host/src/config.rs`,
  `crates/rum-host/src/tests/capability_profiles.rs`,
  `crates/rum-stdlib/src/tests/stress_corpus.rs`,
  `crates/rum-stdlib/src/tests/codecs_xml.rs`,
  `crates/rum-host/tests/stress.rs`, `SECURITY.md`,
  `docs/guide/resource-limits-and-sandbox.md`,
  `docs/guide/evaluation.md`, `docs/guide/embedding.md`,
  `docs/host-embedding-and-hardening.md`,
  `docs/dev/publication-audit/capability-consistency-findings.md`, `TODO.md`,
  and this brief's own appended implementation report. These are precisely the
  direct ripple sites inventoried above
  (`crates/rum-runtime/src/runtime/metadata.rs:4-7`;
  `crates/rum-host/src/config.rs:55-117`; `SECURITY.md:43-60`).
- MUST NOT modify evaluator control flow, message tags/templates, generated
  reference files, `PROGRESS.md`, historical briefs, `Expr`, public serde
  shapes, `.git`, or `Cargo.lock`
  (`crates/rum-core/src/expr.rs:749-753`;
  `crates/rum-runtime/src/runtime/builtin_catalog/inert_heads.rs:38-49`).
- Do not add stack discovery or `unsafe`; the stack is caller-declared
  (`docs/dev/codex/brief-437-native-stack-budget-design.md:115-117,189-192`).

##### Deliverable

- Add the five named constants and
  `native_stack_limit_for_declared_stack`, with exactly the arithmetic and
  values specified in section 4; make values public through runtime and host
  without changing `ResourceLimits` field layout
  (`crates/rum-runtime/src/runtime/model/config.rs:3-31`;
  `crates/rum-host/src/lib.rs:44-69`).
- Add the arithmetic falsifier first, capture its failure while 6 MiB is still
  selected, then derive the 1 MiB default and capture the pass
  (`crates/rum-runtime/src/runtime/tests/session_config.rs:3-11`).
- Keep `DEFAULT_RECURSION_LIMIT` at 1024; adjust the 96-depth adjacency tests to
  a safely lower explicit logical boundary while retaining `reclim`, couple the
  two explicit 8 MiB stress threads through the helper, and do not weaken the
  2 MiB traversal certificates
  (`crates/rum-stdlib/src/tests/stress_corpus.rs:44-88,187-207`;
  `crates/rum-host/tests/stress.rs:290-324`;
  `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:3-136`).
- Rerun the debug XML depth-64 acceptance test. If the new default byte guard
  binds first, move that semantic acceptance assertion to an explicitly
  provisioned larger thread and derive its budget with the helper; do not lower
  the accepted depth or weaken `FromXml::depth` coverage
  (`crates/rum-stdlib/src/tests/codecs_xml.rs:297-334`).
- Update only the current contract documents enumerated in section 6; obtain
  the transcript from a safely guarded run or existing test, never from an
  overflow (`SECURITY.md:43-60`;
  `docs/guide/resource-limits-and-sandbox.md:74-94`).

##### rum / Rust pitfalls (read twice)

1. `native_stack_limit` measures bytes consumed from a base captured inside
   evaluation; it is not total thread capacity
   (`crates/rum-runtime/src/runtime/resolution.rs:288-294`;
   `crates/rum-runtime/src/runtime/eval_core/mod.rs:66-78`).
2. Resume captures a new base and must inherit the same budget without a second
   formula (`crates/rum-runtime/src/runtime/suspension/core.rs:482-510`).
3. `reclim` and `reslim` identify different limits; do not make tests pass by
   swapping expected tags (`crates/rum-runtime/src/runtime/session/session_helpers.rs:322-355`).
4. The helper must reject stacks below 2 MiB and use checked subtraction; no OS
   query and no `unsafe`
   (`docs/dev/codex/brief-437-native-stack-budget-design.md:115-117,189-192`).

##### A/B panel and stop-loss

A/B timing: **none**. This brief changes constants, public forwarding, tests,
and docs, not evaluator hot-path instructions. If implementation requires a
change in `eval_core`, dispatch, matcher, `Expr`, or construction/drop, stop and
return for a new hot-path brief (`AGENTS.md:117-132`).

Stop if the arithmetic test does not fail against 6 MiB, if any explicit
recursion-32 test changes from `reclim`, if an existing 2 MiB release certificate
returns `StackLimit`, or if the API cannot be expressed without platform FFI.
Do not raise the default ad hoc to make a certificate pass
(`crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:3-136`;
`docs/dev/codex/brief-437-native-stack-budget-design.md:189-197`).

##### Verification (mandatory)

Use `scripts/run-bounded.py` for every cargo/release command, deriving each
timeout from the most recent recorded run of the same shape as required by the
repository (`AGENTS.md:75-93`). Run and paste verbatim:

```text
# Red phase, while DEFAULT_NATIVE_STACK_LIMIT is still 6 MiB:
scripts/run-bounded.py --timeout <derived> -- cargo test -p rum-runtime \
  runtime::tests::session_config::default_native_stack_budget_is_derived_from_minimum_supported_stack -- --exact

# Green phase and suites:
scripts/run-bounded.py --timeout <derived> -- cargo fmt --check
scripts/run-bounded.py --timeout <derived> -- cargo test -p rum-runtime
scripts/run-bounded.py --timeout <derived> -- cargo test -p rum-stdlib
scripts/run-bounded.py --timeout <derived> -- cargo test -p rum-stdlib \
  from_xml_depth_limit_accepts_deep_element_nesting -- --exact
scripts/run-bounded.py --timeout <derived> -- cargo test -p rum-host
scripts/run-bounded.py --timeout <derived> -- cargo test --workspace
scripts/run-bounded.py --timeout <derived> -- cargo test -p rum-stdlib --release \
  iterative_traversal_depth -- --ignored
scripts/run-bounded.py --timeout <derived> -- cargo test -p rum-stdlib --release \
  peano_arithmetic_heavy_multiplication -- --ignored --exact
scripts/run-bounded.py --timeout <derived> -- cargo test -p rum-host --release \
  battered_session_recovers_after_repeated_recoverable_limit_trips -- --ignored --exact
```

No test may intentionally exhaust the native stack; every deep test above is
an existing success/guard certificate (`crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:3-136`;
`crates/rum-host/tests/stress.rs:290-324`).

##### Hard rules

No git mutation, network, third-party engine, `unsafe`, out-of-scope edit,
unbounded command, or weakened behavior contract. Preserve user changes and
leave the tree uncommitted (`docs/dev/codex/TEMPLATE.md:125-159`).

##### Report format

Append: per-file changes with line ranges; the verbatim red falsifier and all
green command summaries; the exact derived values/profile propagation; and any
stop-loss/open question (`docs/dev/codex/TEMPLATE.md:161-170`).

#### Draft B — compact runtime-path profiling tokens, then remeasure the packet

##### Goal

Move the runtime-path timer's `Instant` and key into its already-existing TLS
frame, reduce the evaluator-held token to a one-word nesting cookie, preserve
profile output/nesting semantics, and demonstrate both a smaller seven-frame
packet and no hot-label regression
(`crates/rum-runtime/src/runtime/profiling/mod.rs:452-472`;
`crates/rum-runtime/src/runtime/profiling/timing.rs:96-125,374-414`).

##### Context pointers

- Read `AGENTS.md:36-51,75-93,109-132`, this report sections 1-3, and the timing implementation
  (`crates/rum-runtime/src/runtime/profiling/mod.rs:411-472`;
  `crates/rum-runtime/src/runtime/profiling/timing.rs:96-125,374-414`).
- The env cache is the required disabled-path gate
  (`crates/rum-runtime/src/runtime/profiling/state.rs:19-31`;
  `crates/rum-runtime/src/runtime/profiling/config.rs:3-24`).
- Acceptance is profile equivalence, a freshly pasted seven-frame table, and
  the exact microbench target/neighbors (`docs/dev/codex/TEMPLATE.md:78-100,114-123`).

##### Scope

- Files MAY modify:
  `crates/rum-runtime/src/runtime/profiling/mod.rs`,
  `crates/rum-runtime/src/runtime/profiling/timing.rs`, focused profiling tests
  in that module or `crates/rum-runtime/src/runtime/tests/debug_tools.rs`,
  `scripts/core-language-microbench.sh`, and this brief's own appended report
  (`crates/rum-runtime/src/runtime/debug_tools/basic.rs:132-170`;
  `scripts/core-language-microbench.sh:1-25`).
- MUST NOT modify evaluator semantics/call structure, `Expr`, `EvalError`,
  suspension frames, resource defaults, docs outside the report, `.git`, or
  `Cargo.lock` (`crates/rum-runtime/src/runtime/eval_core/mod.rs:14-155`;
  `crates/rum-runtime/src/runtime/session/session_state.rs:498-574`).
- No diagnostic `Mutex`, default-path allocation, or additional per-call gate;
  retain the existing relaxed `AtomicU8` early return
  (`crates/rum-runtime/src/runtime/profiling/config.rs:3-24`;
  `docs/dev/codex/TEMPLATE.md:65-69,86-89`).

##### Deliverable

- Put `start: Instant`, key, and child time together in
  `RuntimePathTimingFrame`; make `RuntimePathTimingToken` a one-word depth/index
  cookie used to validate LIFO completion. Preserve safe behavior on a mismatch
  and every existing outcome counter
  (`crates/rum-runtime/src/runtime/profiling/mod.rs:411-472`;
  `crates/rum-runtime/src/runtime/profiling/timing.rs:390-430`).
- Add focused tests for nested timing-frame push/pop, outcome accounting, and a
  mismatched cookie without relying on process-global env ordering. Also run one
  shallow profiling-enabled CLI snapshot to prove externally visible reporting
  is populated (`crates/rum-runtime/src/runtime/profiling/state.rs:121-126`;
  `crates/rum-runtime/src/runtime/debug_tools/basic.rs:132-170`).
- Before source change, save the release base binary. After source change,
  rebuild and reproduce the exact section-1 prologue/call-edge table. Target
  **projection:** at least 512 B reduction from the 4,736 B packet; if the
  reduction is smaller, report rather than expanding scope
  (`crates/rum-runtime/src/runtime/profiling/mod.rs:469-472`;
  `docs/dev/codex/brief-437-native-stack-budget-design.md:121-143`).
- Add two safe target rows to the common microbench using the exact statements
  `Module[{s = "a"}, Do[s = Power[s, "a"], {64}]]` and
  `Module[{s = "a"}, Do[s = Power[s, "a"], {192}]]`. The depths are below the
  supplied non-crashing 1 MiB last-success ceiling, and the script must give
  them stable labels without adding a result sink
  (`docs/dev/codex/brief-437-native-stack-budget-design.md:41-47`;
  `docs/dev/codex/TEMPLATE.md:82-85`).

##### rum / Rust pitfalls (read twice)

1. Timing frames are nested; compute elapsed/self time from the popped frame and
   retain parent child-time propagation (`crates/rum-runtime/src/runtime/profiling/timing.rs:390-414`).
2. The disabled path remains one cached relaxed-load gate and must not touch TLS
   (`crates/rum-runtime/src/runtime/profiling/config.rs:3-24`;
   `crates/rum-runtime/src/runtime/profiling/timing.rs:96-105`).
3. Do not remove source timing calls or change phase names/outcomes merely to
   shrink frames (`crates/rum-runtime/src/runtime/eval_core/mod.rs:98-154`).
4. The benchmark statement is exact; do not wrap it in assignment/print/sink
   (`docs/dev/codex/TEMPLATE.md:82-85`).

##### A/B panel and stop-loss

Build/save BASE before the token edit and compare it to NEW with profiling env
vars off. Cap the wall runs at four total: best-of-2 BASE and best-of-2 NEW.
Paste `timings.tsv` rows for the two new residual-construction targets and these
same-tier neighbors: `do-count-null`, `do-iterator-null`,
`symbol-ownvalue-read`, `set-ownvalue`, `list-literal-small`,
`downvalue-exact-hit`, `downvalue-exact-miss`, `downvalue-pattern-hit`, and
`downvalue-pattern-miss` (`scripts/core-language-microbench.sh:457-473,502-507`).

Stop after the same failure twice; stop immediately if profile-on results
change, the packet does not shrink by the named target, any target/neighbor
regresses, or the change needs evaluator-source edits. Do not tune unrelated
labels (`docs/dev/codex/TEMPLATE.md:96-100,143-150`).

##### Verification (mandatory)

Run bounded `cargo fmt --check`, focused profiling tests,
`cargo test -p rum-runtime`, and `cargo test --workspace`; build release BASE
and NEW in separate target paths, paste the exact `nm`/`objdump` frame table,
run one shallow profile-on snapshot, then run the four capped microbench walls
and paste every named row. Derive command timeouts from prior same-shape runs
(`docs/dev/codex/TEMPLATE.md:102-123,147-155`). Do not run any stack-exhaustion
or ceiling probe (`docs/dev/codex/brief-437-native-stack-budget-design.md:179-185`).

##### Hard rules

No git mutation, network, third-party engine, `unsafe`, out-of-scope edit,
unbounded command, fake output, or unconditional diagnostic cost. Preserve
suspension behavior and leave the tree uncommitted
(`docs/dev/codex/TEMPLATE.md:125-159`).

##### Report format

Append: per-file changes; verbatim tests/profile snapshot; BASE/NEW frame table
and packet arithmetic; all A/B rows; verdict against the 512 B projected saving;
and stop-loss/open questions (`docs/dev/codex/TEMPLATE.md:161-170`).

#### Draft C — outline cold suspension/restore state one source cluster at a time

##### Goal

After Draft B establishes a new static baseline, remove suspension-frame
construction and `Unevaluated` restoration locals from the ordinary strict-call
frames through cold, non-inlined helpers, without changing evaluation,
suspend/resume, root tracing, or public error semantics; retain only variants
that reduce the seven-frame packet and pass A/B
(`crates/rum-runtime/src/runtime/resolution.rs:350-357`;
`crates/rum-runtime/src/runtime/eval_call_prepare.rs:221-234`;
`crates/rum-runtime/src/runtime/eval_core/call.rs:120-135,192-196`).

##### Context pointers

- Read `AGENTS.md:36-51,75-93,109-132`, this report sections 1-3, and every affected continuation
  definition before editing
  (`crates/rum-runtime/src/runtime/eval_call_prepare.rs:1-45`;
  `crates/rum-runtime/src/runtime/eval_core/call.rs:360-402`).
- The existing `suspend_evaluate_call` and
  `suspend_threaded_call_dispatch` are the cold-helper shape to copy
  (`crates/rum-runtime/src/runtime/eval_core/call.rs:76-85`;
  `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:160-171`).
- Acceptance is semantic/suspension tests, a smaller static packet after each
  isolated cluster, and the frozen Draft-B A/B panel
  (`docs/dev/codex/TEMPLATE.md:90-100,114-123`).

##### Scope

- Files MAY modify:
  `crates/rum-runtime/src/runtime/resolution.rs` and its `resume_frames` module,
  `crates/rum-runtime/src/runtime/eval_call_prepare.rs` and its `resume_frames`
  module, `crates/rum-runtime/src/runtime/eval_core/call.rs` and its continuation
  definitions, focused runtime/host suspension tests, and this brief's own
  appended report (`crates/rum-runtime/src/runtime/resolution.rs:21-26,305-375`;
  `crates/rum-runtime/src/runtime/eval_call_prepare.rs:1-45,190-346`;
  `crates/rum-runtime/src/runtime/eval_core/call.rs:5-18,111-200,360-402`).
- MUST NOT modify `Expr`, public `EvalError` shape, resource defaults, message
  tags, builtin dispatch order, profiling representation, microbench statements,
  docs outside the report, `.git`, or `Cargo.lock`
  (`crates/rum-core/src/expr.rs:749-753`;
  `crates/rum-runtime/src/runtime/session/session_state.rs:498-574`).
- No new heap allocation on a successful/default path. Allocation is allowed
  only inside a cold branch that already constructs owned suspension/restore
  state (`crates/rum-runtime/src/runtime/eval_call_prepare.rs:221-234,328-339`).

##### Deliverable

- Treat each cluster as one isolated variant: (1) fixed-point input suspension,
  (2) argument-preparation input suspension, (3) plan-driver input suspension,
  and (4) final `Unevaluated` restoration. Move construction/handling behind
  `#[cold] #[inline(never)]` helpers while preserving exact frame payloads and
  `resume_or_suspend!` ordering
  (`crates/rum-runtime/src/runtime/resolution.rs:350-357`;
  `crates/rum-runtime/src/runtime/eval_call_prepare.rs:221-234`;
  `crates/rum-runtime/src/runtime/eval_core/call.rs:120-135,192-196`).
- After each cluster, rebuild release and record all seven prologues plus packet
  sum. Keep a cluster only if the sum decreases; revert that cluster alone if
  LLVM retains/grows the union. Do not merge clusters before measuring
  (`docs/dev/codex/TEMPLATE.md:96-100`).
- Add or extend focused tests that suspend during head evaluation, nested
  argument evaluation, list threading, and fixed-point continuation, then
  resume and compare value/messages/trace/root liveness with the pre-change
  contract (`crates/rum-runtime/src/runtime/eval_core/call.rs:51-57,192-196`;
  `crates/rum-runtime/src/runtime/eval_call_prepare.rs:221-234`;
  `crates/rum-runtime/src/runtime/suspension/core.rs:601-650`).
- Do not introduce the private boxed-error carrier in this brief; the public
  error union and its propagation surface need a separate design
  (`crates/rum-runtime/src/runtime/session/session_state.rs:498-574`;
  `crates/rum-runtime/src/runtime/session/session_requests.rs:695-718`).

##### rum / Rust pitfalls (read twice)

1. Continuation frames are session roots; moved helpers must preserve
   `FrameRootTrace` fields and ownership exactly
   (`crates/rum-runtime/src/runtime/eval_call_prepare.rs:13-45`;
   `crates/rum-runtime/src/runtime/eval_core/call.rs:383-402`).
2. `Unevaluated` wrappers are restored only after all stages; do not move the
   behavior earlier or omit normalization
   (`crates/rum-runtime/src/runtime/eval_core/call.rs:120-135`).
3. Error and input branches must record the same runtime-profile outcome before
   returning (`crates/rum-runtime/src/runtime/resolution.rs:339-359`;
   `crates/rum-runtime/src/runtime/eval_call_prepare.rs:212-237`).
4. No successful-path allocation/clone may be added; shared expression/argument
   ownership stays unchanged (`crates/rum-core/src/expr.rs:649-747`).

##### A/B panel and stop-loss

Use the exact BASE/NEW procedure and four-run cap from Draft B. For every
retained cluster, paste the two residual-construction rows and the same neighbor
set from `scripts/core-language-microbench.sh:457-473,502-507`. Profiling is off
for timings (`docs/dev/codex/TEMPLATE.md:78-100,147-150`).

Stop after the same failure twice. Stop/revert the isolated cluster if the
seven-frame packet does not decrease, any hot label regresses, any suspension
value/message/trace differs, a root-liveness test fails, or the change requires
public-type/unsafe work. If the final packet remains above 512 B, report that
expected finding; do not expand into an evaluator rewrite
(`docs/dev/codex/brief-437-native-stack-budget-design.md:136-143,189-197`).

##### Verification (mandatory)

Run bounded formatting, focused runtime suspension/request tests,
`cargo test -p rum-runtime`, `cargo test -p rum-host`, and
`cargo test --workspace`. After each isolated cluster, run release
`nm`/`objdump` only; after the retained final candidate, run the capped A/B and
paste every named row. Derive all timeouts from prior same-shape measurements
and never execute a depth/overflow probe
(`docs/dev/codex/TEMPLATE.md:102-123,147-159`;
`docs/dev/codex/brief-437-native-stack-budget-design.md:179-185`).

##### Hard rules

No git mutation, network, third-party engine, `unsafe`, out-of-scope edit,
unbounded command, semantic weakening, unconditional instrumentation, or
unverified claim. Preserve user changes and leave the tree uncommitted
(`docs/dev/codex/TEMPLATE.md:125-159`).

##### Report format

Append: per-cluster source diff summary; verbatim tests; the before/after
seven-frame table after every attempted cluster; the retained final A/B rows;
the packet verdict versus the 1.5-2.6 KiB projection and 512 B physical target;
and every reverted variant/open question (`docs/dev/codex/TEMPLATE.md:161-170`).

---

## Director gate — 2026-09-04

**Scope**: clean. This file only, 1006 insertions, no source change, no test,
no benchmark, and no stack-exhaustion run, as the brief required. The report
is complete through deliverable 7 and was not truncated.

### What I verified

**The frame inventory is right, including its correction of mine.** I had
counted `run_call_dispatch_stage` as resident; it is not. The plan driver
selects argument preparation in the `if` arm at
`crates/rum-runtime/src/runtime/eval_core/call.rs:153-169` and reaches
`run_call_dispatch_stage` only in the `else`, so it runs on ascent and is
never live while the next level is entered. The three frames I omitted
(`evaluate_once…`, `resume_call_after_head`,
`resume_call_with_dispatch_plan`) are resident. Its 4,736 B static packet
against my 4,675 B measured slope agrees to 1.3%, which is as close as two
independent methods get here.

**`SECURITY.md` makes a promise the kernel cannot keep.**
`SECURITY.md:43-60` publishes "Native stack | 6 MiB" and then "Exceeding one
yields an in-band `TerminatedEvaluation[...]` value, not a crash." On the
ruled 2 MiB embedder stack that sentence is false: the process takes SIGABRT.
This is the headline, and it makes the repair a documentation-accuracy defect
as well as a calibration one.

**The scar tissue is already in the tree.** `codecs_xml.rs:305-313` lowers an
acceptance test from 256 to 64 with the comment that a 256-deep `XmlElement`
tree "exceeds the default ~2 MiB debug test-thread stack". That is this defect
observed in the wild and worked around at the call site, because the guard
that should have returned `reslim` could not fire.

### Where the report is wrong, and the measurement that settles it

The report prices its whole ripple inventory on "the historical approximately
20 KiB user-recursion cost" from `PROGRESS.md:621-628`. That figure is real
but unqualified: it is a **debug** number, from a 2026-07-06 macOS build. The
report neither states the profile nor notices that its own 4.625 KiB packet
came from a **release** binary — so it mixes two profiles in one budget
argument.

I measured both on this host, bisecting `peel[wrap[x_]] := peel[x]` over a
`Nest`-built argument. My first attempt used a literal nested source and was
invalid: the ceiling was 125 levels at *every* stack size, because it was
hitting the parser's 128 nesting guard, not the stack. Building the argument
with `Nest` fixes it. A second invalid region: at 2 MiB and above the ceiling
pins at 252 for a different reason — the fixed-point pass limit (256) — so
only points below that are stack-bound.

| profile | 512 KiB | 768 KiB | 1024 KiB | 1536 KiB | 2048 KiB | 4096 KiB | slope |
|---|---|---|---|---|---|---|---|
| release, user recursion | 63 | 97 | 132 | 202 | — | — | **7.5 KiB/level** |
| debug, user recursion | — | — | 45 | — | 94 | 194 | **20.6 KiB/level** |
| release, strict residual | — | — | 202 | — | 421 | 875 | **4.7 KiB/level** |

So the 20 KiB figure is the debug cost and is still accurate today; release is
7.5 KiB. Three distinct per-level costs, and a budget argument has to say
which it means.

**The consequence reverses my own first reading.** I initially took the
release 7.5 KiB and concluded the report's projected test breakage would not
happen. That was wrong: `cargo test` runs debug on a 2 MiB libtest thread.
At 20.6 KiB/level, `guard_adjacent_user_recursion_below_limit_succeeds`
(`crates/rum-stdlib/src/tests/stress_corpus.rs:44-55`, depth 72) consumes
about 1.48 MiB — it passes today on 74% of a 2 MiB stack with no guard
between it and the edge, and a 1 MiB budget trips it at about 48 levels. The
report's projection was right; its stated reason was under-qualified.

### The gap neither of us flagged

The byte guard polls only at evaluation-depth increments
(`crates/rum-runtime/src/runtime/eval_core/mod.rs:39-42`). A recursion that
runs entirely inside one builtin — a codec parser, `parse_association_item_forms_deep`
from brief-436 — advances no evaluation depth and is therefore invisible to
the guard for its whole descent. Calibrating the budget does not close that;
converting such walkers to explicit stacks does, which is what briefs 422-433
were for. Record it as a known limit, do not widen this repair to chase it.

### Verdict

Accepted, with three corrections and one deletion.

1. **The calibration equation is adopted** — named minimum-stack constant,
   two named reserves, a derived default, a `const fn` for hosts that declare
   a larger stack, and the arithmetic falsifier watched to fail on the old
   6 MiB value before it is made to pass. This is the right shape: the
   relationship is checkable before any evaluator runs.
2. **`DEFAULT_RECURSION_LIMIT` stays 1024 as a ceiling** (its option three).
   With three per-level costs spanning 4.7 to 20.6 KiB, a byte-derived depth
   limit would be a fiction. The documentation must stop presenting 1024 as a
   reachable guarantee.
3. **The ripple inventory is run, not projected.** Every per-level number in
   the report's test predictions is now superseded by the table above, and
   the predictions must be re-derived from it — but no test expectation is
   edited before its failure has been produced verbatim.
4. **Its proposed repair for the failing tests is rejected.** Lowering
   `guard_adjacent_user_recursion_*` from 96/72/120 to 32/24/40 would shrink
   the coverage to fit the budget. These tests are not correct behaviour that
   the fix breaks; they are code passing on 74% of a 2 MiB stack with nothing
   between them and the edge. A test that needs depth declares a thread and
   derives its budget from the same declared size — the shape
   `iterative_traversal_depth.rs:5-12` and the Peano test at
   `stress_corpus.rs:191-197` already use. That also retires the
   `codecs_xml.rs` workaround instead of adding a second one.

The honest price, stated plainly because it is user-visible: on the default
profile a 2 MiB host gets roughly 48 debug / 140 release levels of user
recursion, not 1024. That is what fail-closed on the ruled stack costs, and
it is strictly better than the process kill it replaces.

Deliverables 1-3 (the frame inventory, the source attribution, and the
1.5-2.6 KiB floor showing 512 B is unreachable) stand as the record that
height 4,097 for a strict residual call needs an explicit task stack, not
frame polishing. That is a separate project and is not opened here.

Drafts B and C (compact timing token, cold-state outlining) are not dispatched
yet: they are performance work whose gate is a microbench panel, and they
belong after the safety repair lands. Draft A is rewritten as brief-438.
