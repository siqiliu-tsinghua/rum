# brief-459 — the native-descent guard substrate, with `Merge` as its first consumer

## Goal

Install the least-authority, stack-size-honest guard that lets an
operation-local native recursion in `rum-stdlib` fail closed, and put it to work
on exactly one real site: the ordinary `Merge` nested-collection parser. When
this is done, a `Merge` over a collection nested past the budget returns an
in-band `StackLimitExceeded` and leaves the session usable, instead of
terminating the process, **and it still completes at the ruled physical height
4,097.**

This is step 1 of the class-wide fail-open repair. It deliberately ships the
mechanism plus one consumer and nothing else. The enforcement layers — the
`rum-derive` group validator, the recursion inventory, and the association
surface registry — are step 2, and the per-site conversions follow them.

## Two rulings you are implementing, not deciding

**1. The operation boundary is `native_stack_limit`, unchanged at 1 MiB on the
default profile.** brief-458 proposed `native_stack_limit +
NATIVE_STACK_NON_EVALUATOR_RESERVE_BYTES` = 1.5 MiB. That is **rejected**.
Consumption is measured from `native_stack_base`, which is set at the request
base (`crates/rum-runtime/src/runtime/resolution.rs:292`), so the 512 KiB
non-evaluator reserve covers frames *below* that base; spending it on frames
above the base counts the same bytes twice, and a host that actually uses its
reserve would overflow. Do not introduce a new constant, do not change
`DEFAULT_NATIVE_STACK_LIMIT` or either 512 KiB reserve, and do not change
`SECURITY.md`.

**2. `Merge` is guarded in this step; the other five sites are not.** Under a
1 MiB budget, director-measured at the contract height with three runs each,
`DeleteMissing`, `AssociationKeyDeflatten`, association `MapIndexed` and
`AssociateTo` cannot do the work at all, and `AssociationKeyFlatten` sits on the
line. `Merge`'s collection parser passes 3/3, and its 2 MiB ceiling of 16,072
physical levels projects to roughly 8,036 under 1 MiB — still 1.96x over the
contract. **It is the only site where a 1 MiB guard converts a fail-open into a
fail-closed without costing the height contract**, which is exactly why it is
the first consumer: it proves the mechanism and the boundary together. The other
five are being converted to bounded-front/explicit-stack walks in later briefs,
not guarded. Do not touch them here.

## What is given — do not re-measure, and do not probe past a ceiling

- `Merge` collection parser: clears the contract, ok <= 16,072 / fails >= 16,128
  physical levels on a 2 MiB thread, and **terminates the process** above that.
  Fixture `c = {"k" -> 7}; Do[c = {c}, {i, N}]; Merge[c, Identity]`, one physical
  level per wrapper.
- A ceiling is not deterministic: inside a band tens of levels wide the same
  height flips run to run. Treat no number above as exact to better than about
  50 levels, and never build a test that depends on one.
- The recursion is a single self-edge at
  `crates/rum-stdlib/src/associations/data.rs:186`, inside
  `parse_merge_keyed_entries_collection_with_diagnostics`
  (`crates/rum-stdlib/src/associations/data.rs:173-189`).
- Its sibling `parse_keyed_entry_forms_list_expr_with_diagnostics`
  (`crates/rum-stdlib/src/associations/data.rs:190-201`) does **not** recurse;
  it is a flat `map`. Do not guard it.

## Design constraints — verified, treat as given

1. **The poll is private to the evaluator.** `poll_native_stack_limit` is
   `fn` (private) at `crates/rum-runtime/src/runtime/eval_core/mod.rs:68`, and
   the identifier appears nowhere outside
   `crates/rum-runtime/src/runtime/eval_core/`. `crates/rum-stdlib/` has zero
   occurrences of it or of `evaluation_depth`. A seam is therefore required.
2. **`native_stack_base` and `resource_limits.native_stack_limit` both live on
   `Session`** (`crates/rum-runtime/src/runtime/model/session.rs:84`;
   `crates/rum-runtime/src/runtime/eval_core/mod.rs:69-76`), and `eval_merge`
   already holds `session: &mut Session`
   (`crates/rum-stdlib/src/associations/resumable/aggregate.rs:11-15`).
3. **The error channel already exists.** The recursive function returns
   `Result<_, KeyedEntriesListError>`
   (`crates/rum-stdlib/src/associations/data.rs:16-19`), so exhaustion needs no
   new `Result` wrapper around every node — the shape that cost 6-15% in
   brief-431. Adding a variant is compiler-forced at its consumers: one in
   `resumable/aggregate.rs:21-40` and three in
   `crates/rum-stdlib/src/associations/transforms/key_collections.rs:100,153,257`.
   The three key-collection sites use the non-recursive parser and can never
   observe exhaustion; give them a defined in-band message anyway. **No
   `unreachable!`, no `panic!`, no `unwrap`.**

## Deliverable

### 1. The guard substrate, in `rum-runtime`

A new focused module, exported through `rum_runtime::kernel_api` under
`#[doc(hidden)]`, offering **only**:

- a scoped runner that borrows the session, snapshots base and limit once, owns
  a sticky exhaustion flag, and returns the operation's result or
  `StackLimitExceeded`;
- a branded `NativeStackDepth<'scope>` child token obtained per recursive edge.

It must **not** export `poll_native_stack_limit`, `native_stack_base`, a raw
limit, a raw address, a public constructor, or any method that can raise the
boundary. The seam is semver-relevant
(`crates/rum-runtime/src/runtime/kernel_api.rs:1-6`), so its narrowness is an
acceptance item: say in the report what a caller holding these types can and
cannot do.

Mechanics that are not negotiable:

- The byte check is `#[cold] #[inline(never)]`, takes the address of a local,
  compares `base - current` against the limit, and **returns before the
  recursive call** so its own frame is not resident — the shape at
  `crates/rum-runtime/src/runtime/eval_core/mod.rs:66-78`.
- Logical depth is used **only as a stride**, never as the refusal criterion.
  A depth counter that refuses by count is not stack-size-honest and was
  measured and rejected in brief-443.
- No thread-local, no `unsafe`, no allocation, no lock, and no new field on the
  normal-build `Session`.
- The branded lifetime must not escape the runner. The token must not be
  `Clone`, must not implement `FrameRootTrace`, and must not be storable in a
  suspension frame.

### 2. The one consumer

Guard the self-edge at `crates/rum-stdlib/src/associations/data.rs:186`. Thread
the token through `parse_merge_keyed_entries_collection_with_diagnostics` and
its non-recursive entry `parse_keyed_entries_list_expr_with_diagnostics`, and
open the scope in `eval_merge`.

**Confirm from source, and state in the report, that the scope cannot span a
suspension point** — the parse runs before grouping in `eval_merge`, but say so
with `path:line` rather than assuming it.

### 3. The certificate

Release-only, on a declared 2 MiB thread, following this line's three-part rule:

- **A unique deepest target that must propagate.** Nest the collection N deep
  with a distinguishing innermost entry, e.g. `c = {"deep" -> 7}` then
  `Do[c = {c}, {i, N}]`, at raw physical height 4,097. Assert the **exact**
  result, `Merge[c, Identity] === <|"deep" -> {7}|>` — a summary such as
  `Head[...]` or `Length[...]` is correct even when the parser never descended,
  and twelve certificates in this line were green for exactly that reason.
- **A shallow companion**, three levels, asserting the same exact value, which
  pins the flattening semantics independently of the deep case and is the
  falsifier if the deep case ever aborts the test binary.
- No test may approach the real ceiling.

### 4. The falsifiers — all three, none optional

A check nobody has watched fail is not known to work.

- **Boundary override.** A test-only feature supplies an effective boundary of
  zero, with no normal-build `Session` field and no normal-build branch. At a
  modest depth the guarded `Merge` must return `StackLimitExceeded` in band and
  leave the session usable.
- **Red/green on the real edge.** Remove the guard wrapper from
  `data.rs:186`, run the focused override test, **capture it red verbatim**,
  restore the wrapper, capture it green. If removal does not turn it red, the
  guard is not doing anything and the step is rejected.
- **Compile-fail.** A proof that the token cannot be stored in a suspension
  frame or otherwise escape its scope, in the style the derive crate already
  uses (`crates/rum-derive/src/lib.rs:13-44`).

## Scope

- Files you MAY modify:
  - a new focused guard module under `crates/rum-runtime/src/runtime/`;
  - `crates/rum-runtime/src/runtime/mod.rs` (module declaration only);
  - `crates/rum-runtime/src/runtime/kernel_api.rs` (the seam);
  - `crates/rum-runtime/Cargo.toml` (the test-only feature only);
  - focused new runtime tests plus their test-module wiring;
  - `crates/rum-stdlib/src/associations/data.rs`;
  - `crates/rum-stdlib/src/associations/resumable/aggregate.rs`;
  - `crates/rum-stdlib/src/associations/transforms/key_collections.rs`
    (forced by the error variant, three sites);
  - `crates/rum-stdlib/src/associations/mod.rs` and any test-module root whose
    exported items change — a module root belongs in this list whenever a type
    it exports changes;
  - a new focused `rum-stdlib` test file plus its module wiring.
- Files you MUST NOT touch: the five deferred sites (`missing.rs`,
  `key_reshape.rs`, `functional_ops/apply_map/`, `mutation.rs`,
  `kernel_data/association_data.rs`, `kernel_data/structure_data/exact_paths.rs`),
  `metadata.rs` constants, `SECURITY.md`, `TODO.md`, `PROGRESS.md`, other
  `docs/**`, `Expr`, `Cargo.lock`, `.git`.
- **The accept-vs-stop criterion, general so that a file I failed to list still
  cannot admit a regression:** the ONLY intended behavior change is that a
  `Merge` collection nested past the budget returns an in-band
  `StackLimitExceeded` where it previously terminated the process. Any change to
  which inputs `Merge` accepts, to the values it returns, to its existing
  diagnostics, or to any other builtin's behavior is a **stop** — report it
  instead of shipping it.

## rum / Rust pitfalls (read twice)

1. **No default-path cost.** The guard runs on a path that must stay cheap; the
   disabled/normal case is a strided cold call and nothing else.
2. **Wall-clock timing is not evidence on this host** (P/E cores 1.27x apart,
   nothing pins a core class). Do not run a timing A/B and do not quote a
   percentage from one.
3. **Build both arms at the same filesystem path.** A `/tmp` worktree against a
   home-directory build read 8% apart on untouched probes; allocation counts are
   path-immune, instruction counts are not.
4. **Assert deep structure iteratively**, never through `PartialEq`/`Display` on
   a deep value, which have their own recursion.
5. **Logical wrappers are not physical height.** This fixture is one physical
   level per wrapper, so raw 4,097 is `N=4097`.
6. **Suspension safety**: `eval_merge` lives under `resumable/`. Preserve the
   suspend/resume continuation shape; the guard scope must not span a suspend.

## Verification (MANDATORY — you run it, iterate to green, paste verbatim)

```
cargo fmt --check
cargo test -p rum-runtime
cargo test -p rum-stdlib
cargo test --workspace
```

Then the certificate on its declared 2 MiB thread, release, and the three
falsifiers, with the red capture and the green capture both pasted.

Then the cost gate, **counters first**:

- Allocation counts with `RUM_PROFILE_ALLOC=1` on a `--features alloc-count`
  build, base against candidate, for a `Merge` probe and for one untouched
  control. Calibrate the counter's sensitivity first (show that one extra loop
  iteration moves it) and state the same-binary spread before believing a delta.
- `perf stat -e instructions:u`, minimum of the tight clean cluster, runs that
  migrate between `cpu_atom` and `cpu_core` discarded, both arms built at the
  same path.

Report the whole-panel median beside any per-label number you quote.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree.
- Do NOT modify files outside Scope. If you believe you must, STOP and write
  that in the report.
- **No new `unsafe`.** `rum-core` and `rum-runtime` are zero-`unsafe`.
- Touch nothing outside this repo; no network.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **Do not run any probe that drives a recursion to or past its ceiling**, and
  do not write a reproduction whose purpose is to terminate a process. Every
  falsifier here works at modest depth through the boundary override.
- **Stop-loss.** Stop and report if: the 4,097 certificate does not complete
  with the exact value; removing the wrapper does not turn the focused test red;
  the token can be stored in a suspension frame; allocation counts move on the
  untouched control; or the same step fails twice for the same reason.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. What changed, per file, with line ranges and a one-line why.
2. The seam's authority argument: what a holder of the exported types can and
   cannot do, and why the boundary cannot be raised through it.
3. The suspension argument, with `path:line`.
4. Verification: every command with verbatim output.
5. The three falsifiers, including the red capture and the green capture.
6. The cost gate: allocation rows first with their sensitivity calibration and
   same-binary spread, then instruction counts.
7. Stop-loss and open questions.
