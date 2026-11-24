# Brief 791 — prepare the four render substitution postimages

## Goal

Deliver the byte-exact input package for the retention-line remnant's repair: four postimage
files and one manifest, reconstructed independently from the closed frame discovery and the
paired census, all under `.temp/brief-791/`. No source is installed, no cargo runs, and no
repository tree is modified. Brief 793 installs these bytes on the two arms; the director takes
the same bytes into the live render commit.

## Context pointers

- Read `AGENTS.md`; brief 790
  (`docs/dev/codex/brief-790-g15cm4-retention-remnant-inventory.md`, including Amendment 1) and
  ruling 799 (`.temp/director/ruling-799.md`); `docs/dev/codex/TEMPLATE.md`.
- Accepted evidence under `.temp/brief-790/`: `frame-arrays.tsv` (both parsed frame arrays with
  indices), `frame-delta.tsv` (the 21 moved names with log positions, list positions, both derive
  sites and each armed manual `FrameAbandon` line), `frame-lists.tsv` (the full old/new list
  contents), `census-pins.tsv` and `census-pin-block.txt` (the paired frontiers and every pin),
  `artifact-substitutions.diff` (the exact three-file unified diff), `artifact-postimages.tsv`
  (path, preimage/postimage SHA-256 and byte counts), `scene-pins.tsv`.
- The archive itself: `.temp/director/impl-789/logs/stage-d-workspace.log:4905–4908` carries the
  two parsed frame arrays as printed by the failing test; the two census dumps
  `.temp/director/impl-789/logs/census-frontier.tsv` (armed) and `census-frontier-live.tsv`
  (live) carry 1,229 data rows each.
- The frame partition rule: `crates/rum-runtime/src/runtime/tests/frame_root_trace.rs:250–297`
  and the assertions at `523–550`; the independent derived-FrameRootTrace array and its count in
  `crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs`. The census
  pinned-site assertion: `crates/rum-stdlib/src/tests/retention_closure/census/tests.rs:176–207`.
- The manifest: `crates/rum-stdlib/Cargo.toml` (`[features]` at 12–14, optional `sha2` at 23,
  `[dev-dependencies]` at 25–29). The no-default include chain that Q-build closed:
  `src/lib.rs:162–164` → `src/tests.rs:172–173` → `src/tests/ownership_graph/mod.rs:27` →
  `src/tests/ownership_graph/reconcile.rs:10`.
- The pinned trees `.temp/brief-784/images/{baseline,armed,disabled,real}/` are read-only inputs.

## Scope

- You MAY write only `.temp/brief-791/`. Deliver the postimages at
  `.temp/brief-791/postimages/<repository-relative-path>` and the manifest at
  `.temp/brief-791/manifest.tsv`.
- You MUST NOT touch anything under `.temp/brief-784/images/`, the live repository files
  (`crates/**` included), the historical writers and gates, the 23 inherited live entries, or
  `.git`. No cargo, no network, no version-control command, no source installation. This brief
  writes no covered source, so no standing-inventory replay is triggered here; the standing
  maintenance duty belongs to brief 793, which installs these bytes.
- The three Rust preimages are byte-identical on live and on the armed/disabled copies; verify
  that before writing anything.

## Deliverable

1. `.temp/brief-791/postimages/crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs`
   — SHA-256 `216ae720f117ec7124c157dd3e32cadd8403ef0c8a004cbcd6e8436fd793bb99`,
   173,656 bytes. Preimage `930333ee6c140ef54e177efb1acf058799c60424752ac108b47a3a594a47c2e3`,
   176,541 bytes. Remove exactly the 21 moved entries from `PURE_FRAME_ABANDON_TYPES`; the
   `DERIVED_FRAME_ROOT_TRACE_TYPES` array, the 568-count assertion and every other byte stay
   unchanged.
2. `.temp/brief-791/postimages/crates/rum-runtime/src/runtime/tests/frame_root_trace.rs` — SHA-256
   `b59e0ede39f0d881e22e8e01c04397d4f2ab9908ccff777f27bfcf3e1d5d0561`, 40,719 bytes. Preimage
   `210f9cd33c8c147afa89ae97494bc7b0474226553224a39f9ac880864660181b`, 37,834 bytes. Add the
   same 21 entries, sorted, to `STATEFUL_FRAME_ABANDON_TYPES`; every other byte stays unchanged.
3. `.temp/brief-791/postimages/crates/rum-stdlib/src/tests/retention_closure/census/tests.rs` —
   SHA-256 `4a76c2f5d2016b41b9fd563f37efd419e190d56765cabcb0be639455769d4c3c`, 21,829 bytes.
   Preimage `007a6a461755093e1352569214b18ae1641157f2912b3ecaac62948e9994d0c9`, 21,829 bytes.
   Substitute only `405` for `233` on the unique row `("options/core.rs", 233, "method syntax"),`
   at physical line 188.
4. `.temp/brief-791/postimages/crates/rum-stdlib/Cargo.toml` — SHA-256
   `67be96107619f11d2ebb03367138bc75f61f05d683663eb70c940e296ab331e9`, 1,010 bytes. Preimage
   `6dcfaa9f639ba5cf155a0cd6977333e1c33833085b73c0b3d421b8d23fed9bd7`, 982 bytes. Insert exactly
   one line, `sha2 = { workspace = true }`, into `[dev-dependencies]` between the `rum-runtime`
   line and the `syn` line (i.e. after the `rum-runtime` line, before `syn`); every other byte is
   unchanged. This is the Q-build ruling: it makes the plain
   `cargo test --offline --locked -p rum-stdlib --lib --no-default-features --no-run` build
   compile (the director watched the pre-fix E0432 `unresolved import sha2` at
   `ownership_graph/reconcile.rs:10:5` and the post-fix success). `--locked` stays valid: the
   lock already lists sha2 for rum-stdlib.
5. `.temp/brief-791/manifest.tsv` — header `path`, `preimage_sha256`, `postimage_sha256`,
   `preimage_bytes`, `postimage_bytes`, `roles`; four data rows; `roles` = `armed+disabled` for
   all four (baseline and real keep the preimages).

## Verification

Run every check yourself and quote the verdict lines.

1. **Preimages.** Recompute the four preimage SHA-256 on the live tree and on the armed and
   disabled copies; all four must equal the table above, and the three Rust files must be
   identical across live/armed/disabled. A preimage mismatch is a STOP.
2. **Frames, two independent routes.** Parse the two printed arrays from
   `stage-d-workspace.log:4905–4908` and cross-check them against `frame-arrays.tsv`. Require
   514 ordered unique discovered-pure entries, 535 ordered unique listed-pure entries, zero
   left-only, exactly the 21 right-only rows of `frame-delta.tsv`, and each of the 21 rendered
   declarations carrying a FrameRootTrace derive and its own manual `FrameAbandon` implementation.
   Reconstruct the two list postimages from the arrays (pure = intersection in listed order;
   stateful = sorted union), then independently apply `artifact-substitutions.diff` in memory to
   the preimages and require byte equality between the two routes. Third cross-check:
   `frame-lists.tsv`'s new contents. Verify the final lists are sorted, unique, disjoint, 568
   names total, split 514/54, and that the derived-FrameRootTrace array and the 568-count
   assertion are byte-identical to the preimage.
3. **Census.** Require both dumps to carry 1,229 data rows with identical
   `(kind, file, callee, reason)` multisets; the unique armed key at physical line 835 is
   `405:58–405:68`, the unique live key is `233:58–233:68`, and both are RET002/`options/core.rs`/
   callee `-`/the full `method syntax … is not searched` reason. Recompute the census postimage
   by applying the single substitution; require all other bytes equal the preimage; verify the
   preimage's line 188 is the pointed row and the postimage's line 188 is
   `("options/core.rs", 405, "method syntax"),`. The control arms: the negative-constructor pin,
   the RET001 pin and the four other pinned sites keep their exact preimage bytes.
4. **Manifest.** Require the postimage to differ from the preimage by exactly the one inserted
   line; parse it as TOML (Python `tomllib` is allowed) and require `[dev-dependencies].sha2`
   present, `[features]` and `[dependencies]` otherwise unchanged, and the resulting file to be
   valid. Require the SHA-256 `67be9610…`.
5. **Deliverable closure.** Exactly the four postimages plus `manifest.tsv` and the report exist
   under `.temp/brief-791/`; the manifest's four rows match the actual bytes and hashes; no fifth
   changed file exists anywhere.

## Checkpointing, STOP rules and disclosure

Run serially and bound every parsing command with `scripts/run-bounded.py --timeout 600 --`.
Checkpoint the four preimage hashes before writing any postimage, then the manifest, then the
four output hashes. STOP on: a preimage mismatch; a different discovered set (not zero
left-only / exactly 21 right-only); a missing manual `FrameAbandon`; a second changed byte; a
non-unique or mismatched census row; a different keyed pair; a manifest that is not exactly one
inserted line; any output hash mismatch; any file outside `.temp/brief-791/` touched; any git,
cargo or network attempt. Report every failed attempt and do not revert to copying a hash.

## Budget

1,200 s. The dispatcher's 16G scope is enough: this brief runs no cargo.

## Report format

Write `.temp/brief-791/report.json` in `rum-report/1`: verdict DONE or STOP; the four postimage
SHA-256 values and byte counts; the four preimage SHA-256 values; the frame reconstruction
verdicts (514/535, left-only 0, right-only 21, 568 total, 514/54, derived array byte-identical);
the census verdicts (1,229/1,229, the keyed pair, the single literal substitution, the control
pins); the manifest TOML verdict; the routing (two independent routes agreeing); disclosures of
any changed file; no untried claim. Run `python3 scripts/report.py check 791` and return its
summary. Quote the actual comparison lines verbatim.
