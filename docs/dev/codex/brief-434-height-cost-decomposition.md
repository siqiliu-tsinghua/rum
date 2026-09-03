# brief-434 — decompose the height cache's cost before redesigning it

**Shape: diagnostic. Nothing you build here is kept.** The deliverable is a
cost attribution and one measured alternative.

## Why this brief exists

brief-426 built a cached physical height, measured `set-part-fixed100-len800`
at −30.04% and repeated list append at 8.5-8.9%, fired the stop-loss, and
concluded "the height is not affordable in this layout". That conclusion is
sound as far as it goes, but **the spike changed three things at once and the
verdict attributed the cost to one of them without separating them**:

1. **Allocation growth.** Both private Arc allocations grew 8 bytes —
   `SharedExpr::new` 48→56, `SharedArgs::new` 40→48 (brief-426 §2).
2. **A constructor scan.** `SharedArgs::new` went from `Arc::new(args)` to
   `max_physical_height(&args)`, an O(arity) pass in which every non-atom child
   costs two Arc dereferences to read its cached height.
3. **A rescan on every mutation.** `mutate_shared_args` installs a
   `RefreshHeightOnDrop` guard that recomputes the full O(width) maximum after
   *any* mutable access, and `get_mut`/`make_mut` are the closure-passing API
   every in-place edit goes through.

The label that regressed 30% is a Part assignment into an 800-element list —
the in-place `make_mut` path that plan-297 made O(1) per assignment. Under the
spike each of those assignments ends in an 800-element rescan. **My hypothesis
is that (3) is nearly all of the cost and (1) and (2) are small**, which would
mean the design is affordable and only its maintenance strategy was wrong. I do
not want that hypothesis confirmed by agreement; I want it measured, and I want
to be told if it is wrong.

## Your starting point

The working tree already contains the complete brief-426 spike, restored from
the stash. That is deliberate, not a scope violation. It compiles and its own
tests pass.

## Deliverable — five binaries, one window

Build these and measure them against each other. Nothing here is committed;
work in the tree and report numbers.

| Variant | What it is |
|---|---|
| **V0** | the committed tree, no spike (you will need to set the spike aside to build this — see below) |
| **V1** | spike with the height field present but **never computed**: `SharedArgs::new` does not scan, the mutation guard does not rescan, and every height is a fixed constant. The accessor still returns it so nothing is dead-code eliminated. Isolates allocation growth. |
| **V2** | V1 **plus** the constructor scan in `SharedArgs::new`. Isolates the constructor scan. |
| **V3** | V2 **plus** the mutation rescan — i.e. the unmodified spike. Isolates the mutation rescan. |
| **V4** | V2 plus an **O(1) conservative mutation update**: instead of rescanning, raise the cached maximum to `max(old_max, height_of_what_was_written)`. |

V4 is the design question. A cached maximum that only ever rises is still a
sound admission bound — it can refuse slightly early after a value shrinks, but
it can never admit something too deep. If V4 lands near V2, the height is
affordable and brief-426's verdict was about its maintenance strategy, not the
representation.

For V4 you need the height of what was written, which is available at the write
site. Where an edit does not expose that cheaply, take the conservative route
(leave the maximum unchanged only if you can argue the write cannot raise it;
otherwise saturate to the limit). Say exactly what you did per call site.

**Building V0.** You may not run git commands. Instead: build V0 **first**,
before you touch anything — the tree as you received it minus the spike is not
available to you, so build the spike variants first and tell me you need V0,
**or** simply report V1-V4 against each other and I will supply V0 myself. Do
not guess at V0's numbers.

## The measurement protocol — this is the part that matters

brief-431's gate produced a stop-loss that was half noise because a single
base/candidate pair cannot resolve a few percent on this panel. Use all three
instruments:

1. **Two passes per variant, all ten passes in one window**, so every
   comparison is within-session. Report the same-binary spread per label from
   the two passes of the same variant, first.
2. **Within-run quotients.** For each affected label, divide it by a same-tier
   untouched neighbour *from the same run*, then compare quotients across
   variants. This is immune to the whole-run offset by construction, and it is
   what settled the brief-433 gate when control-median adjustment moved the
   whole panel one way. Same-tier neighbours to consider: `table-count-null`,
   `list-literal-small`, `string-repeat-small`, `keysort-callback-large`,
   `sortby-callback-small`, `table-iterator-plus`, `module-chain-escape-large`.
3. **Report a movement as real only if it exceeds that label's own same-binary
   spread**, and say so per row rather than reporting every delta as a finding.

**The panel that decides this brief**: `set-part-fixed100-len100`,
`set-part-fixed100-len800`, `appendto-list-incremental-small`,
`appendto-list-incremental-large`, `appendto-part-outer1000-small`,
`appendto-part-outer1000-large`, plus every list-construction row and the
`matcher-single-*` and `cases`/`count`/`position`-wide rows as neighbours.
Controls: `literal-integer`, `literal-string`, `symbol-ownvalue-read`,
`loop-empty`, `sameq-integer`, `head-list`, `length-list`, `part-list`,
`if-true`, `module-local`, `block-local`, `with-local`, `table-small`,
`do-small`.

**Allocation counters before timings.** Counters are load-immune; if the
allocation counts do not move between two variants, say so and treat any timing
difference between them with suspicion.

## What I want concluded

1. **The attribution**: how much of brief-426's regression is allocation
   growth, how much the constructor scan, how much the mutation rescan. Numbers,
   per label, with the same-binary spread beside them.
2. **Whether V4 is affordable** — near V2, or not.
3. **Any call site where the conservative O(1) update cannot be applied**, with
   `path:line`, and what it would cost there.
4. If my hypothesis is wrong — if the constructor scan or the allocation growth
   carries the cost — **say so plainly**. A refuted hypothesis here is the most
   valuable result this brief can produce, because it closes the design rather
   than sending me around the loop again.

## Scope

- You may modify anything needed to build the five variants, **but nothing is
  kept**: the tree you leave behind should be V4 (the most promising variant),
  and I will decide what survives. Do not delete or rewrite tests to make a
  variant compile — if a test blocks a variant, say so and skip that variant.
- Do not touch `docs/**` except to append your report to
  `docs/dev/codex/brief-434-height-cost-decomposition.md`.
- **No enforcement, no limit constant, no refusal path, no new public API.**
  This brief measures maintenance cost only. The bound itself is a later
  decision.

## Hard rules

- Do NOT `git commit`, branch, push, stash, checkout, restore, or otherwise run
  git. The working tree's contents are deliberate.
- No network. Touch nothing outside this repository.
- **No new `unsafe`**, including `unsafe` attributes.
- **Write no probe that builds a deep value to see it fail.** This brief is
  timing and counters only; there is no stack-overflow reproduction anywhere in
  it, and none is wanted.
- Profiling env OFF for every timing number. Bound long commands with
  `scripts/run-bounded.py --timeout SECONDS`, sized from a measured run.
- Keep the probe count bounded and state the cap. If a number looks wrong,
  report it rather than silently re-measuring everything.
- Paste real verbatim output. If a variant did not build or a run did not
  complete, say so.
- **Append your report to this file** before you finish.

## Report format

1. The five variants — what each one changed, per file and line range.
2. Allocation counters per variant, for the deciding panel.
3. The timing table — same-binary spread column first, then per-variant
   within-run quotients.
4. The attribution: the three components, with numbers.
5. V4's verdict, and any call site where the O(1) update could not be applied.
6. Whether my hypothesis survived. Say it in one sentence before explaining.

---

# Report — 2026-09-03 — Codex

## 1. Variants built

No bound, limit constant, refusal path, or `unsafe` was added. V3 was built
before the first source edit. V0 was not built: producing the committed tree
without the restored spike requires one of the Git operations this brief
forbids, so I did not guess at either its binary or its numbers.

- **V0 — unavailable.** No files were changed and no binary was claimed. The
  director must supply this binary if the allocation-growth component is to be
  measured.
- **V1 — allocation-size shape only.** In
  `crates/rum-core/src/expr.rs:586-594`, `SharedExpr::new` stored the constant
  atom height `1`; at `:690-698`, `SharedArgs::new` stored constant maximum
  `0`; at the V4-equivalent `:852-875`, `mutate_shared_args` called the closure
  directly and did no maintenance. The fields and accessors remained present.
  No other source differed from the restored spike.
- **V2 — constructor computation.** Relative to V1,
  `crates/rum-core/src/expr.rs:586-594,690-698` restored the unmodified
  constructor bodies: the O(1) `SharedExpr` height computation and the
  O(arity) `SharedArgs` scan. Mutation still did no maintenance. Restoring the
  O(1) `SharedExpr` computation here is necessary both for correct child
  heights during the `SharedArgs` scan and for V3 to be exactly the unmodified
  spike; the V2/V1 attribution below therefore means constructor computation,
  dominated structurally by the `SharedArgs` scan.
- **V3 — restored brief-426 spike.** The binary was built from the received
  tree, before edits. Its constructors were V2 and its
  `crates/rum-core/src/expr.rs:840-854`-equivalent mutation helper installed
  `RefreshHeightOnDrop`, which scanned the complete argument vector after each
  mutable access.
- **V4 — constructor scan plus conservative O(1) maintenance; tree left in
  this state.** `crates/rum-core/src/expr.rs:713-736,852-875` makes the existing
  scoped `get_mut`/`make_mut` API take a caller-computed written-height upper
  bound and raises the cached maximum in a drop guard, including during unwind.
  `crates/rum-runtime/src/runtime/definitions/incremental_append.rs:35-79,217-245`,
  `crates/rum-runtime/src/runtime/definitions/part_assignment.rs:184-216,337-365`,
  and
  `crates/rum-runtime/src/runtime/kernel_data/association_lvalue.rs:40-42,103-122`
  supply the bounds. Two runtime test files plus
  `crates/rum-runtime/src/runtime/evaluation_cache/tests.rs:164-203` were
  mechanically updated for the changed scoped-mutation signature.

The exact measured binaries were distinct:

```text
d85a306514d9a8b0bd650017a1296125c7dc3e26653d639b3a9b8e58bb756965  target/brief434/bin/rum-cli-v1
afff55488497e1c290157cf0998ce895ac8a4a473ee8f7f22c78f3a2e14ebfc9  target/brief434/bin/rum-cli-v2
8cf92a9db3323c311225b7ca7cd96b56d830a29ceb3343ffc95de92b558036aa  target/brief434/bin/rum-cli-v3
b2b3682a09ed951c385b489cde459183b08f6ef91823ac3b69cb3a1f993fddf1  target/brief434/bin/rum-cli-v4
```

Release disassembly also verifies that the intended work survived in the
measured binaries. V1 stores zero immediately before the same 48-byte Arc
allocation; V2 has the 32-byte-stride child loop; and V3 contains the
full-vector refresh drop implementation. Verbatim distinguishing instructions
and symbol output:

```text
V1 SharedArgs::new:
  7a6a5a: 66 c7 44 24 30 00 00  movw   $0x0,0x30(%rsp)
  7a6a6b: bf 30 00 00 00        mov    $0x30,%edi

V2 SharedArgs::new loop:
  7a7010: 66 39 f1              cmp    %si,%cx
  7a7013: 0f 47 f1              cmova  %ecx,%esi
  7a7016: 48 83 c0 20           add    $0x20,%rax
  7a701c: 48 ff ca              dec    %rdx

00000000007a74c0 T <rum_core::expr::mutate_shared_args::RefreshHeightOnDrop as core::ops::drop::Drop>::drop
```

Every release build completed. V1's expected dead-code warning is useful
confirmation that its scan was absent; its verbatim build tail was:

```text
   Compiling rum-core v0.2.0 (/home/remi/dev/rum/crates/rum-core)
warning: function `max_physical_height` is never used
   --> crates/rum-core/src/expr.rs:836:4
    |
836 | fn max_physical_height(exprs: &[Expr]) -> PhysicalExprHeight {
    |    ^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` on by default

   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
warning: `rum-core` (lib) generated 1 warning
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
   Compiling rum-host v0.2.0 (/home/remi/dev/rum/crates/rum-host)
   Compiling rum-terminal v0.2.0 (/home/remi/dev/rum/crates/rum-terminal)
   Compiling rum-repl v0.2.0 (/home/remi/dev/rum/crates/rum-repl)
   Compiling rum-cli v0.2.0 (/home/remi/dev/rum/crates/rum-cli)
    Finished `release` profile [optimized] target(s) in 21.60s
```

## 2. Allocation counters before timings

The probe cap was one counter pass per available variant and two timing passes
per available variant: 12 benchmark invocations total, with no silent reruns.
All output stayed inside `target/brief434/`. Counter passes used the same full
112-row generated benchmark (which contains the deciding panel), scale 1 and
7 repetitions. These counters are process aggregates, not per-label counters.

All allocation and ownership counters were byte-for-byte identical. In
particular, neither requested allocation count moved between any pair. That
makes small timing differences suspect, as required; it does **not** measure
the 8-byte allocation-size change, for which V0 is still required.

| Variant | `shared_expr_alloc` | `shared_args_alloc` | `into_vec_total/clone` | `into_owned_total/clone` |
|---|---:|---:|---:|---:|
| V0 | unavailable | unavailable | unavailable | unavailable |
| V1 | 726850 | 855636 | 116273 / 115274 | 84054 / 84054 |
| V2 | 726850 | 855636 | 116273 / 115274 | 84054 / 84054 |
| V3 | 726850 | 855636 | 116273 / 115274 | 84054 / 84054 |
| V4 | 726850 | 855636 | 116273 / 115274 | 84054 / 84054 |

Verbatim counter output:

```text
v1 rum-profile repr into_vec_total=116273 into_vec_clone=115274 into_owned_total=84054 into_owned_clone=84054 shared_args_alloc=855636 shared_expr_alloc=726850
v2 rum-profile repr into_vec_total=116273 into_vec_clone=115274 into_owned_total=84054 into_owned_clone=84054 shared_args_alloc=855636 shared_expr_alloc=726850
v3 rum-profile repr into_vec_total=116273 into_vec_clone=115274 into_owned_total=84054 into_owned_clone=84054 shared_args_alloc=855636 shared_expr_alloc=726850
v4 rum-profile repr into_vec_total=116273 into_vec_clone=115274 into_owned_total=84054 into_owned_clone=84054 shared_args_alloc=855636 shared_expr_alloc=726850
```

## 3. Timing window, spreads, and quotients

The eight available timing passes ran consecutively in one window, ordered
V1, V2, V3, V4, V1, V2, V3, V4. No CPU-heavy command ran concurrently. Each
used scale 10, 21 repetitions, a 75-second inner process-group bound and a
90-second outer bound. `RUM_PROFILE_REPR=0` was explicit and the benchmark
script set every other profiling switch it knows to `0`.

The raw same-binary spread is
`100 * (max(pass1, pass2) / min(pass1, pass2) - 1)`. It is the first numeric
column below, in V1/V2/V3/V4 order. Each quotient cell is independently
`target_seconds / neighbour_seconds` from pass 1/pass 2; no quotient divides
measurements from different processes. The attribution section uses the
geometric center of each pair.

The list-construction set reported here is `table-count-null`, both other
`table-iterator-*` constructors, `list-literal-small`, `table-small`,
`table-iterator-plus`, `table-iterator-pair-list`, and
`character-range-small`. The primary neighbours are from the brief's supplied
list: `string-repeat-small` for construction rows,
`module-chain-escape-large` for the very short iterator-symbol row, and
`sortby-callback-small` for mutation and matcher/traversal rows.

| Label | raw spread V1/V2/V3/V4 | neighbour | V1 q1/q2 | V2 q1/q2 | V3 q1/q2 | V4 q1/q2 |
|---|---:|---|---:|---:|---:|---:|
| `table-count-null` | 3.7/4.7/2.4/0.8% | `string-repeat-small` | 0.8032/0.7731 | 0.7475/0.7378 | 0.8299/0.8099 | 0.6995/0.6937 |
| `table-iterator-null` | 0.1/5.4/2.4/1.0% | `string-repeat-small` | 0.1592/0.1591 | 0.1543/0.1532 | 0.1700/0.1659 | 0.1438/0.1430 |
| `table-iterator-symbol` | 5.9/20.9/1.3/3.9% | `module-chain-escape-large` | 0.5973/0.5703 | 0.6227/0.6748 | 0.6544/0.6399 | 0.6374/0.6758 |
| `list-literal-small` | 4.6/0.7/0.4/10.2% | `string-repeat-small` | 0.4229/0.4032 | 0.4129/0.3921 | 0.4122/0.4136 | 0.4080/0.4423 |
| `table-small` | 2.0/0.0/0.6/1.9% | `string-repeat-small` | 1.0119/1.0296 | 1.0295/0.9707 | 1.0247/1.0298 | 1.0306/0.9949 |
| `table-iterator-plus` | 0.4/3.0/8.6/1.5% | `string-repeat-small` | 0.1666/0.1668 | 0.1702/0.1652 | 0.1662/0.1529 | 0.1719/0.1718 |
| `table-iterator-pair-list` | 1.7/0.5/1.7/1.8% | `string-repeat-small` | 0.3890/0.3949 | 0.4064/0.3812 | 0.3942/0.4004 | 0.4086/0.3951 |
| `character-range-small` | 2.4/5.8/2.6/0.6% | `string-repeat-small` | 0.7071/0.7222 | 0.7326/0.7302 | 0.7346/0.7529 | 0.7479/0.7400 |
| `set-part-fixed100-len100` | 0.7/0.4/1.9/1.0% | `sortby-callback-small` | 0.0428/0.0436 | 0.0449/0.0455 | 0.0451/0.0458 | 0.0454/0.0438 |
| `set-part-fixed100-len800` | 1.4/0.0/0.6/0.6% | `sortby-callback-small` | 0.1589/0.1628 | 0.1783/0.1800 | 0.1770/0.1821 | 0.1796/0.1759 |
| `appendto-list-incremental-small` | 0.0/1.5/1.6/1.1% | `sortby-callback-small` | 9.0417/9.1330 | 9.8279/10.0690 | 9.8373/10.0169 | 10.1347/9.7664 |
| `appendto-list-incremental-large` | 0.0/0.3/0.3/1.0% | `sortby-callback-small` | 34.8803/35.2385 | 38.1061/38.6037 | 37.2924/38.4649 | 39.2429/37.8368 |
| `appendto-part-outer1000-small` | 1.7/1.0/4.3/1.2% | `sortby-callback-small` | 7.0442/7.2400 | 7.3466/7.4913 | 7.3882/7.9744 | 7.5160/7.2327 |
| `appendto-part-outer1000-large` | 1.2/1.0/1.9/1.5% | `sortby-callback-small` | 17.0565/17.0365 | 17.6406/17.9855 | 17.7657/18.0360 | 18.0799/17.3489 |
| `matcher-single-tiny` | 1.7/2.6/1.7/0.4% | `sortby-callback-small` | 2.3074/2.2927 | 2.2764/2.3586 | 2.2895/2.3296 | 2.3070/2.2570 |
| `matcher-single-wide-small` | 2.9/0.2/0.1/0.9% | `sortby-callback-small` | 0.0134/0.0132 | 0.0128/0.0129 | 0.0127/0.0131 | 0.0130/0.0126 |
| `matcher-single-wide-large` | 2.9/0.1/0.3/0.5% | `sortby-callback-small` | 0.0263/0.0259 | 0.0251/0.0254 | 0.0251/0.0259 | 0.0257/0.0249 |
| `cases-wide-no-match` | 11.4/1.4/0.3/0.5% | `sortby-callback-small` | 0.5222/0.5876 | 0.5241/0.5219 | 0.5142/0.5332 | 0.5233/0.5124 |
| `count-wide-no-match` | 1.8/0.2/0.7/1.2% | `sortby-callback-small` | 0.4926/0.5068 | 0.4906/0.4965 | 0.4874/0.5076 | 0.5023/0.4834 |
| `position-wide-no-match` | 2.3/2.2/1.2/1.2% | `sortby-callback-small` | 0.6046/0.6251 | 0.5949/0.6140 | 0.6000/0.6132 | 0.6083/0.5853 |

The required controls show why raw cross-process ratios alone are not usable.
The longer controls are mostly within a few percent, but V2 pass 2 and V4 pass
2 have large offsets on several sub-millisecond controls:

| Control | V1 s1/s2 (spread) | V2 s1/s2 (spread) | V3 s1/s2 (spread) | V4 s1/s2 (spread) |
|---|---:|---:|---:|---:|
| `literal-integer` | 0.000734414/0.000730823 (0.5%) | 0.000728660/0.000781609 (7.3%) | 0.000759036/0.000761970 (0.4%) | 0.000744826/0.000879341 (18.1%) |
| `literal-string` | 0.000771036/0.000779149 (1.1%) | 0.000790846/0.000818338 (3.5%) | 0.000806517/0.000789554 (2.1%) | 0.000790064/0.000925568 (17.2%) |
| `symbol-ownvalue-read` | 0.00204413/0.00204965 (0.3%) | 0.00204765/0.00232222 (13.4%) | 0.00226610/0.00207723 (9.1%) | 0.00207892/0.00225606 (8.5%) |
| `loop-empty` | 0.00236511/0.00231579 (2.1%) | 0.00229935/0.00229626 (0.1%) | 0.00237171/0.00235967 (0.5%) | 0.00221383/0.00257085 (16.1%) |
| `sameq-integer` | 0.00637516/0.00636998 (0.1%) | 0.00655634/0.00670322 (2.2%) | 0.00645437/0.00633576 (1.9%) | 0.00639718/0.00651577 (1.9%) |
| `head-list` | 0.0112773/0.0112045 (0.6%) | 0.0113851/0.0116081 (2.0%) | 0.0115340/0.0113962 (1.2%) | 0.0112736/0.0112950 (0.2%) |
| `length-list` | 0.0113117/0.0112419 (0.6%) | 0.0113307/0.0117102 (3.3%) | 0.0115197/0.0113841 (1.2%) | 0.0112085/0.0113649 (1.4%) |
| `part-list` | 0.0133362/0.0130993 (1.8%) | 0.0132271/0.0137586 (4.0%) | 0.0132246/0.0131664 (0.4%) | 0.0132488/0.0133541 (0.8%) |
| `if-true` | 0.00724071/0.00713414 (1.5%) | 0.00726486/0.00758380 (4.4%) | 0.00730181/0.00713669 (2.3%) | 0.00718882/0.00728703 (1.4%) |
| `module-local` | 0.00770350/0.00748775 (2.9%) | 0.00764365/0.00780083 (2.1%) | 0.00771167/0.00739286 (4.3%) | 0.00755701/0.00734571 (2.9%) |
| `block-local` | 0.00471742/0.00459767 (2.6%) | 0.00475150/0.00479216 (0.9%) | 0.00466953/0.00465200 (0.4%) | 0.00462198/0.00457942 (0.9%) |
| `with-local` | 0.00499582/0.00497652 (0.4%) | 0.00494802/0.00503306 (1.7%) | 0.00497622/0.00494332 (0.7%) | 0.00490319/0.00491565 (0.3%) |
| `table-small` | 0.00189270/0.00193003 (2.0%) | 0.00192091/0.00192184 (0.0%) | 0.00192689/0.00193825 (0.6%) | 0.00191510/0.00187894 (1.9%) |
| `do-small` | 0.00920308/0.00944797 (2.7%) | 0.00915679/0.00952540 (4.0%) | 0.00905723/0.00910230 (0.5%) | 0.00902610/0.00884051 (2.1%) |

Verbatim timing-run output, including whole-process elapsed time:

```text
core microbench output: target/brief434/timings/v1-pass1
engine	status	exit_code	elapsed_seconds	stdout	stderr
rum	ok	0	37.524986	target/brief434/timings/v1-pass1/raw/rum.stdout	target/brief434/timings/v1-pass1/raw/rum.stderr
core microbench output: target/brief434/timings/v2-pass1
engine	status	exit_code	elapsed_seconds	stdout	stderr
rum	ok	0	37.326642	target/brief434/timings/v2-pass1/raw/rum.stdout	target/brief434/timings/v2-pass1/raw/rum.stderr
core microbench output: target/brief434/timings/v3-pass1
engine	status	exit_code	elapsed_seconds	stdout	stderr
rum	ok	0	37.314128	target/brief434/timings/v3-pass1/raw/rum.stdout	target/brief434/timings/v3-pass1/raw/rum.stderr
core microbench output: target/brief434/timings/v4-pass1
engine	status	exit_code	elapsed_seconds	stdout	stderr
rum	ok	0	36.820106	target/brief434/timings/v4-pass1/raw/rum.stdout	target/brief434/timings/v4-pass1/raw/rum.stderr
core microbench output: target/brief434/timings/v1-pass2
engine	status	exit_code	elapsed_seconds	stdout	stderr
rum	ok	0	36.983400	target/brief434/timings/v1-pass2/raw/rum.stdout	target/brief434/timings/v1-pass2/raw/rum.stderr
core microbench output: target/brief434/timings/v2-pass2
engine	status	exit_code	elapsed_seconds	stdout	stderr
rum	ok	0	38.274352	target/brief434/timings/v2-pass2/raw/rum.stdout	target/brief434/timings/v2-pass2/raw/rum.stderr
core microbench output: target/brief434/timings/v3-pass2
engine	status	exit_code	elapsed_seconds	stdout	stderr
rum	ok	0	36.927509	target/brief434/timings/v3-pass2/raw/rum.stdout	target/brief434/timings/v3-pass2/raw/rum.stderr
core microbench output: target/brief434/timings/v4-pass2
engine	status	exit_code	elapsed_seconds	stdout	stderr
rum	ok	0	36.775663	target/brief434/timings/v4-pass2/raw/rum.stdout	target/brief434/timings/v4-pass2/raw/rum.stderr
```

## 4. Attribution

The allocation-size contribution is **unknown**, not zero: V1/V0 is
unavailable. The constructor column is V2/V1, the mutation-rescan column is
V3/V2, and the conservative-update check is V4/V2. A `*` means the magnitude
of that primary-neighbour quotient movement exceeds the larger raw
same-binary spread of that label in the two variants being compared. Unstarred
movements are not findings under the brief's rule.

| Label | allocation V1/V0 | constructor V2/V1 | rescan V3/V2 | O(1) V4/V2 |
|---|---:|---:|---:|---:|
| `table-count-null` | unknown | -5.8%* | +10.4%* | -6.2%* |
| `table-iterator-null` | unknown | -3.4% | +9.2%* | -6.8%* |
| `table-iterator-symbol` | unknown | +11.1% | -0.2% | +1.3% |
| `list-literal-small` | unknown | -2.6% | +2.6%* | +5.6% |
| `table-small` | unknown | -2.1%* | +2.8%* | +1.3% |
| `table-iterator-plus` | unknown | +0.6% | -4.9% | +2.5% |
| `table-iterator-pair-list` | unknown | +0.4% | +0.9% | +2.1%* |
| `character-range-small` | unknown | +2.4% | +1.7% | +1.7% |
| `set-part-fixed100-len100` | unknown | **+4.6%** * | +0.6% | -1.3%* |
| `set-part-fixed100-len800` | unknown | **+11.4%** * | +0.2% | -0.8%* |
| `appendto-list-incremental-small` | unknown | **+9.5%** * | -0.2% | +0.0% |
| `appendto-list-incremental-large` | unknown | **+9.4%** * | -1.3%* | +0.5% |
| `appendto-part-outer1000-small` | unknown | **+3.9%** * | +3.5% | -0.6% |
| `appendto-part-outer1000-large` | unknown | **+4.5%** * | +0.5% | -0.6% |
| `matcher-single-tiny` | unknown | +0.7% | -0.3% | -1.5% |
| `matcher-single-wide-small` | unknown | -3.5%* | +0.5%* | -0.2% |
| `matcher-single-wide-large` | unknown | -3.2%* | +0.8%* | +0.0% |
| `cases-wide-no-match` | unknown | -5.6% | +0.1% | -1.0% |
| `count-wide-no-match` | unknown | -1.2% | +0.8%* | -0.2% |
| `position-wide-no-match` | unknown | -1.7% | +0.4% | -1.3% |

The scattered starred movements outside the six mutation targets alternate in
sign on paths V3 cannot execute: for example, `table-count-null` is +10.4%
while `table-iterator-plus` is -4.9%. With identical allocation counts, this
is code-layout/panel noise, not a coherent mutation-rescan cost. I have
reported it rather than rerunning it.

The six mutation rows are coherent only in V2/V1. As a sensitivity check, the
same geometric quotient calculation using the separately prescribed
`keysort-callback-large` neighbour gives the following primary/secondary
numbers:

| Label | constructor (`sortby` / `keysort`) | rescan (`sortby` / `keysort`) | O(1) (`sortby` / `keysort`) |
|---|---:|---:|---:|
| `set-part-fixed100-len100` | +4.6% / +5.5% | +0.6% / -2.5% | -1.3% / -0.4% |
| `set-part-fixed100-len800` | +11.4% / +12.3% | +0.2% / -2.9% | -0.8% / +0.1% |
| `appendto-list-incremental-small` | +9.5% / +10.4% | -0.2% / -3.3% | +0.0% / +0.9% |
| `appendto-list-incremental-large` | +9.4% / +10.3% | -1.3% / -4.3% | +0.5% / +1.4% |
| `appendto-part-outer1000-small` | +3.9% / +4.8% | +3.5% / +0.2% | -0.6% / +0.3% |
| `appendto-part-outer1000-large` | +4.5% / +5.4% | +0.5% / -2.6% | -0.6% / +0.3% |

Thus the measured decomposition, pending V0, is:

- allocation growth: **unmeasured**;
- constructor computation/scan: **+3.9% to +11.4%** on all six deciding
  mutation rows with the primary neighbour, confirmed as **+4.8% to +12.3%**
  with the secondary neighbour;
- mutation rescan: **no coherent positive cost**. Five primary movements are
  within the label's spread; the lone movement outside it is a 1.3% apparent
  improvement, and the secondary neighbour also moves that row faster;
- V4 O(1) maintenance: **near V2**. The primary range is -1.3% to +0.5% and
  the secondary range is -0.4% to +1.4%, with no coherent regression.

## 5. V4 call sites and verdict

V4 is affordable **relative to V2**: the conservative maintenance itself is
not measurably more expensive than constructor-only caching. This does not yet
make the complete cache affordable relative to V0, because the constructor
cost is real and the allocation-size component remains missing.

Every repository mutation call site accepted an O(1) bound; none had to
saturate unconditionally:

- `crates/rum-runtime/src/runtime/definitions/incremental_append.rs:39,61-79`
  computes the appended item's exact cached height once and supplies it to the
  leaf vector write. At `:217-245`, each already-visited ancestor supplies
  `item_height + current_path_length`, saturating the `u16`; the path length is
  exactly the number of enclosing `List` calls in the child being rewritten.
- `crates/rum-runtime/src/runtime/definitions/part_assignment.rs:185-203`
  computes the replacement height once. At `:337-365`, each ancestor supplies
  `replacement_height + remaining_path_length`, saturating the `u16`. Existing
  siblings are covered by the old maximum.
- `crates/rum-runtime/src/runtime/kernel_data/association_lvalue.rs:40-42,103-122`
  supplies the exact height of the newly built `Rule` for either replacement
  or append.
- `crates/rum-runtime/src/runtime/evaluation_cache/tests.rs:164-203` passes
  zero only to closures that provably do not write, and passes exact atom
  heights to the two append checks.
- `crates/rum-runtime/src/runtime/tests/association_lookup_cache.rs:52-61` and
  `crates/rum-runtime/src/runtime/tests/association_part_cursor.rs:92-101`
  pass the exact replacement-`Rule` height.

The guard at `crates/rum-core/src/expr.rs:852-875` performs one `max` even on
unwind. If a rollback restores the previous value or a reserve fails before a
write, raising to the supplied bound is merely an overestimate. No path can
lower the cached maximum, and no path can admit an expression that is too
deep. The cost is one O(1) cached-height read per leaf write plus O(1) work at
each ancestor the existing path walk already visits; there is no O(width)
rescan.

## 6. Hypothesis

**The hypothesis did not survive: the constructor computation/scan, not the mutation rescan, is the only coherent measured cost in the six deciding mutation rows.**

On `set-part-fixed100-len800`, specifically, V2/V1 is +11.4% (+12.3% with
the secondary neighbour), V3/V2 is +0.2% (-2.9% secondary), and V4/V2 is
-0.8% (+0.1% secondary), against raw same-binary spreads of
1.4%/0.0%/0.6%/0.6% for V1/V2/V3/V4. Brief-426's 30.04% total cannot be
fully decomposed until V0 is supplied, but these measurements refute the claim
that the full-vector mutation rescan is nearly all of it.

## 7. Verification and raw command results

All Cargo commands were offline and all long commands used
`scripts/run-bounded.py`. The final V4 runtime suite passed:

```text
running 956 tests
test result: ok. 954 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 3.89s

Doc-tests rum_runtime

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

The supplied tree is missing a file declared by the restored spike:
`crates/rum-core/src/expr/height_cache_tests.rs`. I did not create, delete, or
rewrite a test to work around that. Consequently core and workspace test-mode
compilation cannot start. Verbatim focused output:

```text
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
   Compiling rum-core v0.2.0 (/home/remi/dev/rum/crates/rum-core)
error: couldn't read crates/rum-core/src/expr/height_cache_tests.rs: No such file or directory (os error 2)
    --> crates/rum-core/src/expr.rs:1077:1
     |
1077 | mod height_cache_tests;
     | ^^^^^^^^^^^^^^^^^^^^^^^

error: could not compile `rum-core` (lib test) due to 1 previous error
```

The bounded workspace command stopped at the same pre-existing missing file:

```text
   Compiling rum-core v0.2.0 (/home/remi/dev/rum/crates/rum-core)
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
   Compiling rum-derive v0.2.0 (/home/remi/dev/rum/crates/rum-derive)
   Compiling rum-syntax v0.2.0 (/home/remi/dev/rum/crates/rum-syntax)
error: couldn't read crates/rum-core/src/expr/height_cache_tests.rs: No such file or directory (os error 2)
    --> crates/rum-core/src/expr.rs:1077:1
     |
1077 | mod height_cache_tests;
     | ^^^^^^^^^^^^^^^^^^^^^^^

error: could not compile `rum-core` (lib test) due to 1 previous error
warning: build failed, waiting for other jobs to finish...
```

For the same reason, workspace `cargo fmt --check` cannot resolve the module:

```text
error: couldn't read /home/remi/dev/rum/crates/rum-core/src/expr/height_cache_tests.rs: No such file or directory (os error 2)
    --> /home/remi/dev/rum/crates/rum-core/src/expr.rs:1077:1
     |
1077 | mod height_cache_tests;
     | ^^^^^^^^^^^^^^^^^^^^^^^

Error writing files: failed to resolve mod `height_cache_tests`: /home/remi/dev/rum/crates/rum-core/src/expr/height_cache_tests.rs does not exist
```

The seven files changed for V4 were formatted and then checked directly with
Rustfmt using edition 2024 and `skip_children=true`; the check exited 0 with
verbatim empty output. Release builds for all four available variants and the
full V4 runtime suite demonstrate that the missing test-only module did not
affect the measured binaries.

## Director's gate (2026-09-03)

**The hypothesis in this brief was wrong, the report's attribution is right as
far as it goes, and the component nobody measured turns out to be the dominant
one.** I supplied the two missing comparisons.

### V0 vs V4 — the whole design against no height at all

112 labels, whole-panel median offset **+0.06%** (a clean window), same-binary
spreads under 1% on every row below except one:

| Label | V4 vs V0 | same-binary |
|---|---:|---:|
| `set-part-fixed100-len800` | **+12.21%** | +0.29% |
| `sort-default-wide` | **+11.79%** | +3.28% |
| `reversesort-default-wide` | **+10.67%** | +0.26% |
| `set-part-fill-large` | **+10.14%** | −0.45% |
| `set-part-fill-small` | +6.89% | −0.39% |
| `appendto-part-outer1000-large` | +4.39% | −0.02% |
| `appendto-part-outer1000-small` | +4.26% | −0.06% |
| `set-part-fixed100-len100` | +3.96% | −0.65% |
| `appendto-list-incremental-large` | +3.83% | −0.01% |
| `keysort-default-wide` | +2.91% | +0.11% |

**The affected family is wider than the panel this brief named.**
`sort-default-wide`, `reversesort-default-wide`, `set-part-fill-*` and
`keysort-default-wide` were not in it and are among the worst rows. A panel
chosen from where the cost was expected missed where it actually landed.

### V0 vs V1 — the field present, never computed

I built V1 myself: `SharedExpr::new` stores a constant, `SharedArgs::new` does
not scan, mutation does no maintenance. The field and accessors remain, so
nothing is dead-code eliminated. **This is the cost of 8 bytes per Arc
allocation and nothing else.**

| Label | V1 vs V0 | same-binary |
|---|---:|---:|
| `stringtrim-literal-list-32000` | **+14.21%** | −0.05% |
| `set-part-fixed100-len800` | **+9.58%** | +0.32% |
| `table-iterator-pair-list` | +8.70% | +0.49% |
| `stringtrim-literal-list-4000` | +8.15% | +0.46% |
| `stringtrim-single-literal-32000` | +7.41% | −0.07% |
| `appendto-part-outer1000-small` | +6.13% | +0.27% |
| `keysort-default-wide` | +5.16% | +0.15% |

`stringtrim` does not build call spines and never reads a height, and it moves
14% from a field that is never computed. `set-part-fixed100-len800` is +9.58%
of its +12.21% total. **The dominant term is the allocation growth, not any
computation** — `SharedExprInner` 48→56 and `SharedArgsInner` 40→48 bytes,
which under a 16-byte-granular allocator moves the args inner from a 48-byte
chunk to a 64-byte one and costs cache density everywhere that allocates.

### What this means

brief-426's verdict — "the height is not affordable **in this layout**" — was
right, and its emphasis was on exactly the right word. Neither the constructor
scan nor the mutation rescan is the floor: **the floor is putting the height
behind the `Arc`.** No maintenance strategy can recover it, which retires both
the `SharedArgs::new` redesign and the O(1) conservative update as answers.

### The representation that is left

Verified by compile-time assertion on this toolchain: `Expr` is 32 bytes,
`Atom` is 32 bytes, and both shared wrappers are 8. `Expr::Call { head, args }`
therefore carries 16 bytes of payload in a 32-byte enum — **about 16 spare
bytes inside the value that already exists**. A height stored there grows no
allocation at all, and reading a child's height during any scan becomes a field
read at a fixed offset in contiguous memory rather than an Arc dereference.

That is the only representation the measurements leave standing, and it is a
different and larger change than this line has attempted: roughly 73 struct
literal construction sites and ~900 `Expr::Call` mentions, most of them
patterns that need `..`. brief-435 spikes it for cost before anything is
designed around it.

**V4 is preserved in `stash@{0}`.**
