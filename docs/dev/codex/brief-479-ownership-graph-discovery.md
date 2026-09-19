# brief-479 — the ownership graph, discovery half

## Goal

Build the scanner that rum does not have: one that reads **type definitions**
and reports every type whose ownership graph can reach itself, plus every place
the graph leaves rum's source and cannot be closed. Produce an **exact, checked
inventory** of both.

**Discovery only. No dispositions, no obligations, no remediation.** Those are
step two. This step exists to answer a question the director and user agreed to
settle before any larger decision: **how many opaque ownership edges does rum
actually have, and where?**

## Why this is split from the ledger

brief-476 designed the whole thing — scanner, disposition vocabulary, discharge
obligations, falsifiers — as one step across eight new modules. It is split here
for two reasons, neither of them size.

- **The purpose of this step is a number.** The open question is whether rum
  should change its embedding API so host payloads declare their ownership
  (`set_print_sink` takes `Box<dyn FnMut(&str) + Send>`;
  `FileIoProvider::from_arc` takes `Arc<dyn FileIo>`). That decision was
  deliberately deferred until the scale is measured rather than argued. A
  discovery inventory delivers that; a full ledger delivers it later.
- **It is independently gateable.** An exact row set either matches or does not.
  Dispositions need judgement per row and a separate falsifier each.

## What must be discovered

### 1. Self-reaching types

The edge relation, from brief-476 §2 — verify it rather than assume it:

- **Owning**: `Vec<T>`, `Option<T>`, `Box<T>`, tuples, arrays, `HashMap`/
  `BTreeMap` key and value, enum variant payloads, struct fields.
- **Conditionally owning**: `Arc<T>`/`Rc<T>` — only the last strong holder
  destroys the payload. Record the edge with that condition attached; "usually
  shared" is not a reason to drop the edge.
- **Not owning**: `&T`, `&mut T`, `Weak<T>`. These must not produce edges.

Resolution must be **structural, never by name**. Two known cases prove it, and
both must be found without being told about them:

| hazard | why a naive scan misses it |
|---|---|
| `LookupOutcome` | says `List(Vec<Self>)` — `Self`, not its own name (`basic.rs:297-300`) |
| `NestedPathNodes` / `NestedPathNode` | a **mutual** pair whose edge runs through `Vec<(Expr, NestedPathNode)>` — a tuple inside a Vec (`key_reshape/traversal.rs:24-31`) |

A third property that matters: `LookupOutcome` is **declared in `basic.rs` while
its `Drop` lives in `basic/lookup.rs`**. Resolution must cross files.

### 2. Unknowns, which are the point of this step

Where the graph leaves rum's source, the scanner must say so precisely rather
than skip. At minimum:

- **`UnknownHostPayload`** — an opaque host-supplied value whose concrete type
  rum never sees: `Box<dyn FnMut(…)>` at
  `crates/rum-runtime/src/runtime/model/session.rs:137`,
  `Arc<dyn FileIo>` at `crates/rum-runtime/src/runtime/file_io.rs:105`.
  **Director ruling: this is fatal by default and waivable only with a named
  owner and the specific boundary cited** — not a silent skip, and not a hard
  block that would keep the check permanently red and therefore unused. In this
  step, enumerate them; the waiver mechanism is step two.
- **`UnknownConstruct`** — a trait object, function pointer, closure capture,
  macro-generated definition or foreign type the scanner cannot decide. Report
  it with its file, line and the construct. **Never skip silently.** brief-476
  is explicit that an unknown is an analysis failure, not a classification.

**Report both counts separately and prominently.** The `UnknownHostPayload`
count is the number this step exists to produce.

## Deliverable

1. **A scanner** under `crates/rum-stdlib/src/tests/ownership_graph/`, split by
   responsibility. brief-476 §7 proposed source / resolve / effects / graph /
   evidence / fixtures modules; follow that shape or justify a different one.
   `syn` and `proc-macro2` are already workspace dependencies
   (`crates/rum-derive/Cargo.toml`); adding them as `rum-stdlib` dev-dependencies
   is in scope, upgrading anything is not.

2. **An exact inventory test** over production source in `rum-core`,
   `rum-runtime` and `rum-stdlib`: a sorted row per self-reaching type
   (stable id, defining file and line, and the field-path witness for its cycle)
   and a sorted row per unknown (kind, file, line, construct). Exact equality,
   in the shape of
   `crates/rum-stdlib/src/tests/association_depth_contract/recursion_inventory.rs`
   — whose narrower scope you must **not** copy.

3. **The rediscovery falsifier, which is this step's acceptance.** Run discovery
   against real production source **with no inventory input at all** and assert
   that all three type ids appear — `LookupOutcome`, `NestedPathNodes`,
   `NestedPathNode` — each with its witness. Then, in turn:
   - delete only **one** member of the mutual pair's rows and require a failure
     naming the missing one;
   - feed an in-memory fixture containing a freshly named recursive type, a
     mutual pair split across two modules, and a field whose recursion appears
     **after a comma** inside a map or tuple — the last one because this project
     already lost a round to a regex silently truncated by a comma.

   **If your scanner cannot find the two real hazards without being told their
   names, it does not work yet. Report that rather than adding a name list.**

## Scope

- Files you MAY modify: new modules under
  `crates/rum-stdlib/src/tests/ownership_graph/`;
  `crates/rum-stdlib/src/tests.rs` for the module wiring;
  `crates/rum-stdlib/Cargo.toml` for dev-dependencies only; `Cargo.lock` only
  for that bookkeeping.
- Files you MUST NOT touch: **all production source**, the existing depth
  inventory and registry, `crates/rum-derive/`, `metadata.rs`, `Expr`,
  `SECURITY.md`, `TODO.md`, `PROGRESS.md`, other `docs/**`.
- **Accept-vs-stop:** this step adds a test-only scanner and an inventory. Any
  production change, any name-based special case for the two known hazards, and
  any silent skip of an undecidable construct are each a **stop**.

## rum / Rust pitfalls (read twice)

1. **A scan that reports clean must be shown capable of finding something.** The
   rediscovery falsifier is that proof; this project has had a regex silently
   truncated by a comma and, in this very line, a director scan that missed both
   known hazards because it matched only literal self-names.
2. **Do not associate a `Drop` with a type by file.** `LookupOutcome`'s is two
   files away.
3. `Arc`/`Rc` edges are real but conditional; record the condition rather than
   dropping the edge or asserting the payload always dies.
4. A count derived by a script is a claim, not a fact. Cross-check the type count
   by hand on at least two files and say which.
5. Wall-clock timing is not evidence on this host, and nothing here needs it.

## Verification (MANDATORY — run it, iterate to green, paste verbatim)

```
cargo fmt --check
cargo test -p rum-stdlib
cargo test --workspace
```

State whether your sandbox required `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`.

Then paste:

- the **`UnknownHostPayload` count and every row**, which is this step's headline;
- the `UnknownConstruct` count and rows;
- the self-reaching type count, and the three required rows in full;
- the rediscovery falsifier's output, including each deliberate failure;
- `git diff --stat`, showing **no production file**.

**No performance gate**: no production code changes. Say so explicitly rather
than running the panel.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Do NOT modify files outside Scope. **No new `unsafe`.** No network.
- **Testing Policy is a hard line** (`AGENTS.md`).
- Use `scripts/run-bounded.py --timeout SECONDS --` for anything long; size the
  bound from a prior run of the same shape.
- **Stop-loss.** Stop and report if: the scanner cannot find both known hazards
  structurally; the unknown population cannot be enumerated without a wildcard
  exemption; or the same step fails twice for the same reason.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format

1. What you built, per module, with one line on each module's responsibility.
2. The edge relation as implemented, and your decision for each undecidable
   construct class.
3. **The two counts**, with the `UnknownHostPayload` rows in full.
4. The self-reaching inventory: count, and the three required rows.
5. The rediscovery falsifier, including the deliberate failures.
6. Verification output and `git diff --stat`.
7. Your hand cross-check of the type count, and on which files.
8. Stop-loss and open questions.
