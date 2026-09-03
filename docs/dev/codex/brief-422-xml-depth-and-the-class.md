# brief-422 — `FromXml` kills the process, and it is the fourth of four

Shape: **implementation**. Worker: **cdx (Codex)**.

Jumps the queue ahead of the incremental-charging line. A sandboxed `{Pure}`
script can abort the host process; unbacked memory declarations can not.

## The defect, director-reproduced

`crates/rum-stdlib/src/xml/decode.rs` contains **zero** occurrences of `depth`.
`parse_element` (`:195`) calls `parse_children` (`:253`) which calls
`parse_element` (`:295`), mutually recursive with no bound in scope.

Reproduction, release `rum-cli`, verbatim:

```
$ rum-cli --eval 's = StringJoin[StringRepeat["<a>", 12000], StringRepeat["</a>", 12000]]; r = FromXml[s]; Print["parsed"]'
thread 'main' has overflowed its stack
fatal runtime error: stack overflow
（core dumped）
```

Every part of that input is `Pure`. The abort is outside the panic boundary, so
`catch_unwind` cannot catch it and the crash-only contract does not contain it:
the process dies. `SECURITY.md`'s model is evaluating *untrusted scripts* safely
inside a trusted host, and this breaks it with three builtins.

The other three decoders refuse in band at the same depth — I ran
`FromJson`, `FromYaml` and `FromToml` on 12,000 levels and each returned a
clean error rather than crashing. `depth` appears 14, 29 and 20 times in their
decoders and 0 times in XML's.

## This is a class miss, and the class is what you must close

`TODO.md:23-25` records the previous repair in this exact family:

> YAML and TOML decoders bounded, matching `FromJson`. All three codecs now
> accept exactly 256 nested collections and refuse the 257th with a message
> naming the limit and the byte offset.

Three codecs. There were four. XML sat in the same directory with the same
shape and was not surveyed, so it was not fixed — the same failure brief-388
made, closing the instances that had been seen instead of the class.

**So the deliverable is not "add a guard to XML".** It is: every recursive
descent in this workspace whose depth is controlled by script-supplied data is
either bounded or carries a recorded reason why it cannot recurse unboundedly.

Note the kernel's own native stack guard does **not** help here: it is armed in
the evaluator (`crates/rum-runtime/src/runtime/resolution.rs:385-386`) and
polled on a stride, and it does not see a stdlib parser's own recursion. Do not
propose it as the fix without establishing that it can be.

## What I could not establish, so do not treat it as cleared

I tried to reach the **encoders** — `ToXml`, `ToJson`, `ToYaml`, `ToToml`,
`FullForm`, `ToString` — with a deep expression and failed: `Nest[g, 1, 20000]`
returns `TerminatedEvaluation["IterationLimit"]` in the default configuration,
so I never got a deep tree into them. My first attempt at that probe was worse:
it used `Nest[List, 1, n]`, which terminated even earlier, and the encoders
returned "evaluation failed" for a reason that had nothing to do with them.

**That is an unestablished result, not a safe one.** A deep tree can arrive from
somewhere other than `Nest` — a decoder below its own depth limit, a host
constructing `Expr` through the Rust API, a restored snapshot. Settle it.

## Deliverable

1. **A bound on the XML decoder** matching the existing convention exactly: 256
   nested elements accepted, the 257th refused in band, with a message naming
   the limit and the byte offset, and an error kind and message tag in the shape
   `json/decode.rs` uses (`JsonParseErrorKind::Depth`, tag `"depth"`,
   `:6,29-40,110-125`). Match it; do not invent a second convention.

2. **A full inventory** of recursive descent over script-controlled depth across
   `rum-core`, `rum-syntax`, `rum-runtime`, `rum-stdlib` and `rum-jupyter`:
   decoders, encoders, renderers, tree walkers, matchers, formatters. For each
   row: `path:line`, what controls the depth, and either the bound it already
   has or the reason it cannot be driven unboundedly. Cross-check the inventory
   against live probes — a source reading that says "guarded" is what produced
   this defect.

3. **A bound for every unbounded row you find**, in the same convention.

4. **A regression test per bound**, asserting refusal in band and session
   survival, in the shape of the existing depth tests for the three guarded
   codecs.

5. **`rum-jupyter`'s `json_support.rs` is in scope for the inventory but NOT
   for repair.** It has no depth guard either — that is a known finding from
   the 2026-09-02 security audit, recorded in `TODO.md`, and it is a transport
   with its own decisions pending. List it, leave it, say so.

## Scope

Files you MAY modify:

- `crates/rum-stdlib/src/xml/decode.rs` and `crates/rum-stdlib/src/xml.rs`
- any other file whose unbounded recursion your inventory finds, in the same
  crate set — **except** `crates/rum-jupyter/src/json_support.rs`
- module roots of anything above whose exported types change
- test files and test module roots in the affected crates
- this brief, to append your report

If the inventory finds something whose repair is not a depth bound — a
different shape of fix, or a language-behaviour question — **STOP and report
it** rather than deciding it. Two such decisions are already mine and pending.

Files you MUST NOT touch: `crates/rum-jupyter/src/json_support.rs`, the memory
growth classifications or the `Preflight` allowlist (a depth bound does not
discharge a byte promise, and `FromXml` stays on that list),
`debug_tools/resources/`, any `ResourceLimits` default or the stride,
`AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**` except appending your report,
`.git`, git config.

## Hard rules

- **Match the existing convention.** 256, in-band refusal, limit and byte
  offset in the message. A second spelling of the same idea is a defect.
- **Do not change what a valid document parses to.** A document within the
  bound must produce exactly what it produces today. If your bound changes any
  existing pinned or golden output, STOP and report — the codec round-trip
  tests are pinned deliberately.
- **No new `unsafe`.** Do not raise the native stack, do not spawn a thread
  with a bigger stack to paper over the recursion.
- Do not weaken an existing test.
- Do NOT `git commit`, branch, push, stash, checkout, or touch git config.
  Leave everything in the working tree. Do not touch `stash@{0}`.
- Touch nothing outside this repo except `/tmp` scratch you remove afterwards.
  No network; pass `--offline` to every `cargo` command.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only.
- **This host can bind loopback**; the nine native ZeroMQ tests genuinely run
  here — I observed 9 of 9 at the brief-420 gate. If your sandbox denies it and
  you need `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`, say so and mark those nine
  unobserved.
- **Append your report to this brief file** before finishing.

## Verification (MANDATORY — you run it, iterate to green)

First, **reproduce the abort before you fix it** and paste it. A repair whose
defect was never observed is not known to repair anything; this project has
been caught by an unwatched instrument four times.

Then:

```
cargo fmt --check
cargo test --offline -p rum-stdlib
cargo test --offline --workspace
```

Then re-run the reproduction and paste the in-band refusal that replaces the
abort, plus a probe showing the session survives and can evaluate afterwards.

For every other bound your inventory adds, paste the same before/after pair.

No microbench A/B is required if your bounds add only a depth counter on paths
that already recurse; say so explicitly, and if you touched anything hotter,
run one.

## Report format

1. The reproduction, before the fix, verbatim.
2. What changed — per file, line ranges, one-line why.
3. The inventory: every recursive descent over script-controlled depth, with
   its bound or its reason, and how you probed each rather than reading it.
4. The encoders: what you established, since I could not.
5. Verification — every command, verbatim, with the before/after pair per bound.
6. Anything you found whose fix is not a depth bound — reported, not decided.
7. Stop-loss / open questions.
