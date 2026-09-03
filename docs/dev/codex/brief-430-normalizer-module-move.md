# brief-430 — move the candidate normalizer into its own module, no logic change

Step 2 of three. Step 1 (brief-429, `66faf671`) locked the copy-on-write
contract in tests. Step 3 (brief-431) replaces the recursion. **This step moves
code and changes nothing else**, so that step 3's diff shows only the algorithm.

The repository's own precedent is the reason for the separation: a file split
folded into a semantic change makes both unreviewable, which is why the
assignment-target line deferred its split until the repair closed
(`TODO.md`, family A's "evaluate a split when family B closes — not
mid-repair" note).

## Goal

Move the candidate-normalization cluster out of
`crates/rum-runtime/src/runtime/resolution.rs` into a new
`crates/rum-runtime/src/runtime/resolution/match_normalization.rs`, preserving
behavior exactly. `resolution.rs` is 20,037 bytes and the repository requires a
split assessment when a file at that size is touched
(`docs/code-organization-guidelines.md:64-79,124-140`); normalization is a real
responsibility boundary, and step 3 will add an explicit-stack engine to it.

When you are done, every moved function body is **byte-identical** to what it
was, `cargo test --workspace` is green, and the only semantic difference in the
tree is which file the code lives in.

## What moves

From `crates/rum-runtime/src/runtime/resolution.rs`:

- `Session::normalize_match_expr` (`:62-65`)
- `Session::normalize_definition_identity_for_matching` (`:67-96`)
- `Session::normalize_match_expr_changed` (`:99-118`)
- `Session::normalize_match_expr_changed_for_test` (`:120-127`, `#[cfg(test)]`)
- `Session::normalize_match_arg_list_changed` (`:128-153`)

Nothing else moves. `resolution.rs` keeps everything above and below that
cluster, including `mod resume_frames;`.

## The one thing that will break if you are not careful

`pub(super)` **changes meaning when the code moves down a level.** Inside
`resolution.rs` it means visible throughout `crate::runtime`; inside
`resolution::match_normalization` the same token would mean visible only inside
`resolution`, and the twelve callers across the runtime would stop compiling.

`normalize_match_expr` and `normalize_definition_identity_for_matching` must
therefore become `pub(in crate::runtime)`, which is exactly their current reach
— not `pub`, not `pub(crate)`, not `pub(super)`. The two private workers stay
private to the new module (their only callers move with them). The
`#[cfg(test)]` accessor is already `pub(in crate::runtime)` and keeps that.

Widening visibility beyond `pub(in crate::runtime)` is a stop condition, not a
convenience: brief-395 had to repair a published surface that had been widened
without anyone deciding to publish it.

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/resolution.rs` — remove the moved cluster,
    add the `mod match_normalization;` declaration.
  - `crates/rum-runtime/src/runtime/resolution/match_normalization.rs` (new).
  - Appending your report to
    `docs/dev/codex/brief-430-normalizer-module-move.md`.
- Files/areas you MUST NOT touch: everything else. In particular **no caller
  changes anywhere**. These are inherent methods on `Session`, so no call site
  needs an import change; if you find yourself editing a caller, you have
  changed visibility wrongly — stop and report instead.
- No logic change, no renaming, no reformatting of the moved code, no doc
  comment edits, no reordering of the functions relative to each other.
- The test module from step 1
  (`crates/rum-runtime/src/runtime/tests/match_normalization_contract.rs`) must
  keep passing untouched. It is not in scope; if it needs an edit to compile,
  that means the move changed reach — stop and report.

## Deliverable

- The new module, with whatever `use` items it needs to compile on its own.
  Prefer explicit imports over a blanket glob if a glob would pull in more than
  the moved code uses.
- `resolution.rs` with the cluster removed and the module declared.
- **A mechanical byte-identity proof** in your report: extract each moved
  function's body from `git show HEAD:crates/rum-runtime/src/runtime/resolution.rs`
  and from the new file, and diff them. Paste the command and its empty output.
  Prose asserting the bodies are unchanged is not the deliverable; the diff is.
  Signature lines legitimately differ for the two visibility changes named
  above — show those two lines explicitly and separately, and let the diff cover
  the bodies.

## Verification (MANDATORY — you run it; paste verbatim)

```
cargo fmt --check
cargo test --workspace
```

The workspace gate is right for this step even though the change is inside one
crate: a visibility mistake is exactly the failure mode here, and it surfaces
as a compile error in dependent crates, not as a test failure in `rum-runtime`.

**Do NOT run `scripts/core-language-microbench.sh`.** A pure move that the
compiler inlines identically cannot be measured against run noise, and a green
A/B here would be mistaken for evidence about step 3. State that reasoning in
the report in place of numbers. If you believe the move could change codegen,
say why in the report — do not measure it.

## Stop-loss

- If any caller needs an edit, stop and report: the reach changed.
- If `match_normalization_contract.rs` needs an edit, stop and report.
- If the byte-identity diff is non-empty for any body, either restore the
  original text or stop and report what forced the difference.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; I run the authoritative gate and commit.
- Do NOT modify files outside Scope.
- No network. Touch nothing outside this repository.
- No new `unsafe`.
- Bound long commands with `scripts/run-bounded.py --timeout SECONDS`, sized
  from a measured run of the same shape.
- Paste real verbatim output; if a run did not reach green, say so.
- **Append your report to this file** before you finish.

## Report format

1. What changed — per file, with line ranges.
2. The byte-identity proof — commands and their output.
3. The two visibility lines, before and after.
4. Verification — `cargo fmt --check` and `cargo test --workspace`, verbatim,
   with the test totals.
5. Why no microbench was run.
6. Stop-loss / open questions.
