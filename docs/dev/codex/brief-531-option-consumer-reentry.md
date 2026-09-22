# brief-531 — close the remaining option consumers, by construction

Implementation brief. **Do not run `git` in any form — not `status`, not
`diff`, not `log` — at any point, including while first loading this
repository.** Everything you need about the tree's state is in this brief.

## Goal

brief-530 repaired eleven option consumers and left nine call sites covering
22 builtins broken — and left them in a worse shape than it found them. Their
option loop now completes and dispatch returns its encoded list, so

```
StringWrap[…, PreserveNewlines :> <suspending>]
  →  InternalContinuationVec[Rule[PreserveNewlines, False], …]
```

where the queued-input path returns `"a\nb"`. An internal continuation
encoding is reaching user level.

Close them. **But not by writing nine more frames** — the mechanism that
repaired the first eleven is itself the shape this project keeps repairing:

```rust
// crates/rum-stdlib/src/strings/patterns/search/option_resume.rs:24-65
match self.symbol {
    "StringPosition" => …,
    …
    _ => unreachable!("only string search consumers install this frame"),
}
```

A tenth string consumer added later either lands in that `unreachable!()` or,
more likely, never installs the frame at all and silently leaks the encoding
again. The public wrapper's rustdoc already states the obligation — *"The
caller must install a consumer frame to use that list and finish its own
operation"* (`crates/rum-runtime/src/runtime/options/core.rs:147-150`) — and a
documented obligation is exactly a convention someone must remember.

**Make it structurally impossible to consume merged options without a
resumption.** A call site that does not supply one must not compile, and a
supplied resumption with no arm must fail a required test.

## The standard this is held to

*A fix the compiler or a derive enforces always beats one that relies on
people remembering, even an order of magnitude more expensive.* Nine bespoke
frames would close today's nine sites and leave the tenth to chance. Cost is
not an argument here; an unenforced boundary is.

## What I want

1. **The wrapper demands a re-entry.** `merged_option_exprs_for_system_symbol`
   — and whatever internal path it shares with
   `merged_option_exprs_for_symbol` — must not hand back a usable option list
   to a caller that has not said how to resume. Prefer a **typed re-entry
   descriptor** over a boxed closure: a closure in a suspension frame owns
   captures nothing traverses, which is why brief-530's three resume closures
   are sitting in `unknown_captures.tsv` as unresolved. A descriptor is data
   a frame can hold and a derive can traverse.

2. **The descriptor set is exhaustive and enforced.** Model it on what this
   repository already uses for exactly this problem: the `Session` field
   schema with no `..`
   (`crates/rum-runtime/src/runtime/debug_tools/resources/occurrence/schema.rs:19-42`),
   and the frame inventory at
   `crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs`.
   A new option-consuming builtin must be unable to reach the list without
   appearing in the set, and a set entry with no live arm must fail a required
   test. **`unreachable!()` on a symbol-name match is not enforcement** — it
   is the thing being removed.

3. **All 22 remaining builtins resume correctly.** The nine sites are
   `strings.rs:78` (Uncompress, HtmlUnescape, TemplateApply),
   `structure_ops/ordering_sets.rs:528` (Union, Intersection, Complement),
   `predicates/collections.rs:353` (DuplicateFreeQ, IntersectingQ, DisjointQ,
   SubsetQ, ContainsAll, ContainsAny, ContainsExactly, ContainsNone,
   ContainsOnly), `strings/scripting/args.rs:153` (ParseArgs),
   `strings/scripting/url.rs:140` (UrlEncode, UrlDecode), `url.rs:168`
   (UrlParse), and `strings/scripting/text_shape.rs:163,190,215` (StringWrap,
   StringIndent, AlignColumns). Note that several of them take options inside
   a **parser helper** (`parse_string_wrap_options`, `effective_same_test`),
   so the resumption has to re-enter the handler, not just the parser.

4. **The eleven already repaired move onto the same mechanism.** Two
   mechanisms for one obligation is one mechanism plus a trap. The existing
   certificates must keep passing unchanged.

5. **The leak must be impossible by construction.** After this, there must be
   no path on which the encoded continuation list can become a builtin's
   result. Say how that is prevented, not merely that no test shows it.

6. **A certificate per builtin**, in the shape brief-530 established: a real
   suspension asserted (`expect_err`, prompt checked), the queued answer and
   the resumed answer compared, and — where the builtin returns a boolean —
   an observable that an inner-value substitution cannot satisfy. brief-530
   used lists for exactly this reason; keep that trick.

7. **Re-measure the checker's frontier and report it.** brief-530 dropped
   `OPTION_FRONTIER` from 17 resolved bodies / 5 files / 50 rejected edges to
   **4 / 3 / 15**, because C02's Rule construction moved inside
   `resume_or_suspend!` and the restricted checker sees an unexpanded macro
   (`RET003` at `options/core.rs:219-227`). You are not required to fix that
   here. You **are** required to re-run the frontier and report the new
   numbers, and to say whether your restructuring moved construction into or
   out of macro boundaries. D2b cannot convert what the checker cannot see.

## Falsifiers — watched, not asserted

- **A new option-consuming call site without a re-entry does not compile.**
  Add one in a scratch fixture, watch the compiler reject it, and keep a
  permanent `compile_fail` proof with a compile-*pass* control built first.
  Pin the expected error where the compiler gives a stable code.
- **A descriptor with no live arm fails a required test**, watched red then
  green.
- **The leak itself**: a probe that produced `InternalContinuationVec` before
  the change and the documented answer after, for at least `StringWrap` and
  one non-string consumer.
- Save complete output under `target/brief-531/` before quoting it.

## Scope

**This section states an objective and a fence, not an enumeration.** The
enumeration is guidance; it is not the boundary.

**The objective.** Inside `crates/rum-runtime/` and `crates/rum-stdlib/`,
change what the Goal requires — including files not named here.

**The disclosure duty, which replaces asking permission.** Every file you
touch that the guidance does not name must appear in your report's first
section with its `path:line` ranges and one line on why the objective required
it. **An undisclosed edit is the violation; a disclosed necessary edit is
not.** The director diffs the tree against your disclosure list.

**Where the work is expected to live** (guidance):

- `crates/rum-runtime/src/runtime/options/` — the wrapper's contract, the
  descriptor type, the option-loop frames.
- `crates/rum-runtime/src/runtime/suspension/` and the frame inventories,
  plus `crates/rum-derive/` if the enforcement wants a derive.
- The nine call sites and their handlers in `crates/rum-stdlib/src/`, and
  `crates/rum-stdlib/src/strings/patterns/search/option_resume.rs`, which
  should stop being a symbol-name match.
- Owning tests in both crates, including
  `crates/rum-stdlib/src/tests/options_suspension.rs`.
- The ownership and frame inventories that exact-equality tests require.

**The fence — STOP conditions:**

- **`crates/rum-core/`**, in particular `expr.rs`. Representation is fixed.
- All `Cargo.toml` files and `Cargo.lock`. No new dependency; the pin is
  stable 1.85.0.
- The interner, gensym, and the snapshot format.
- **No leases, no `R` term, no admission or accounting change.** That is D2b.
- `AGENTS.md`, `PROGRESS.md`, `TODO.md`.

## rum / Rust pitfalls (read twice)

1. **A closure in a frame owns captures nothing traverses.** That is why a
   descriptor beats a boxed closure here, and why brief-530's resume closures
   are recorded unresolved.
2. **A resumed frame must not assume unique ownership.** `Arc::try_unwrap`
   demanding uniqueness is a shape that has panicked this kernel.
3. **A certificate must force the transformation it names.** Assert the
   suspension happened and the exact resumed value; a boolean that is correct
   when nothing ran is the trap this project has hit twelve times.
4. **Queued input does not suspend.** You need both paths per builtin.
5. **`compile_fail` passes when the code fails for the wrong reason.** Build
   the compile-pass control first and pin the error.
6. **Do not collapse a `suspend_with_frame*` into an eager call.**
7. **Eight `retention_closure` tests run in the gate.** If your change makes a
   resolved edge unresolved, report it — do not silence it.

## Verification (MANDATORY — you run it; iterate to green)

Director measurements on this host, warm tree, immediately before dispatch:
`cargo test --workspace` green in **76 s**, no failures; eight checker tests
green. The tree is green at your starting point; any red is yours.

```
python3 scripts/run-bounded.py --timeout 600  -- cargo fmt --check
python3 scripts/run-bounded.py --timeout 1200 -- cargo test -p rum-runtime
python3 scripts/run-bounded.py --timeout 900  -- cargo test -p rum-stdlib retention_closure -- --nocapture
python3 scripts/run-bounded.py --timeout 2400 -- cargo test --workspace
```

Nine native ZeroMQ tests cannot bind loopback in your sandbox. Report them as
skipped-by-environment; the director settles them. **Do not** set
`RUM_ALLOW_NATIVE_ZEROMQ_SKIP`.

**Measurement.** You are changing a signature consulted by option-taking
builtins. **Allocation counts first**, via
`scripts/core-memory-microbench.sh` — this host's cores are 1.27x apart and
one timing pair cannot gate the panel. If the counters do not move, do not
quote a time. If they do, run the language microbench base and candidate at
the same binary path, four passes each, profiling env off, normalized against
**named untouched labels**, never the suite median.

## Hard rules

- **No `git`, in any form.**
- Do NOT commit, branch, push, or touch git config.
- Respect the fence. Outside it, the rule is the disclosure duty.
- **No new `unsafe`.**
- No `#[ignore]`, no feature gate on a required test, no test weakened to get
  green, no documented answer changed to match an implementation.
- **Testing Policy is a hard line** (`AGENTS.md`).
- Scratch goes under `target/brief-531/`.
- **Stop-loss, keyed on progress.** Report two numbers per attempt: builtins
  whose resumed answer matches their queued answer, and required gates
  passing. Stop only when **two consecutive** attempts move neither. A compile
  error you are actively reducing is not a wall.

## Report format (your final message)

1. What changed — per file with line ranges, including every file the
   guidance did not name, with why.
2. The mechanism: what the wrapper now demands, how the descriptor set is
   made exhaustive, and what a new consumer physically cannot do.
3. The falsifiers, each watched red then green, with the `compile_fail`
   control.
4. Coverage: all 22 builtins with their queued and resumed answers, plus the
   eleven moved onto the shared mechanism.
5. Verification — every command with verbatim output.
6. The re-measured `OPTION_FRONTIER`, and whether construction moved into or
   out of macro boundaries.
7. Measurement — allocation counts first; timing only if the counters moved.
8. Stop-loss / open questions.
