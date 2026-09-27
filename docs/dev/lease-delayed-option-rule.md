# Private delayed-option rule boundary

Brief 543 converts `Session::evaluate_runtime_option_expr` (C02) while retaining
its `Result<Expr, EvalError>` signature. C01's defaults and merged accumulator
remain separate unresolved obligations. No production certificate is minted.

`accounting_mode/rule.rs` owns clone publication, the two-child builder, and
resume publication. The outward result is caller-owned W: leaving the scope
does not give its caller a lease. The local lint therefore treats both success
and exceptional outgoing payloads as unleased at a subsequent hazardous call.

The concrete contracts have these domains:

| Record | Preconditions and guarantee | Excluded claim |
| --- | --- | --- |
| B04 | Exact `Expr::clone`. Its actual result is OwnedUnleased until resized into a receipt. The private clone adapters charge the clone, including actual string capacity, boxed numeric payload, and shared expression occurrences. | Knowing the Rust type does not establish capacity; no allocator or interner bound. |
| B10 | Only the unchanged private `finish_rule` adapter: one adopted RHS, a borrowed name, charged cloned name and literal head, checked shell and actual vector capacity. It consumes those owners into a charged `RetainedResult`. | Bare `Expr::call` never acquires this guarantee. |
| B11 | Only literal `system_expr("Rule")` in that builder. Cold/warm observations pin System symbol identity and occurrence weight; the result is charged explicitly. | Arbitrary names and global interner initialization. |
| B12 | The existing private `RetentionFrame`, initialized pending input, a charged frozen prefix, traced roots and complement, exactly one transfer. | Legacy raw option frames, callee-name-only insertion, or invented/uninitialized input. |
| R01–R15 and finite mode suffixes | Exact private clone/evaluate/resume, adoption, builder and late-arm handoffs, separately recorded for the two sealed modes. | Arbitrary generic or trait admission and caller coverage. |

The RHS crosses the existing activation bridge. On success its receipt precedes
reserving the two child slots; the name is cloned and adopted before another
growth operation. The builder charges the Rule head and outer shell before
construction. Ordinary errors retain their original payload and do not capture
a name or frame. Panic unwinds the active bridge and all receipts.

Only a real pending `InputRequired` clones the name into a retained prefix.
`RuntimeOptionSuspensionFrame` is now a rootless descriptor implementing
`RetentionContinuation`; the kernel's private frame owns `FrozenValues`.
Resumption borrows the frozen name, adopts the callback result, and charges a
separate working clone even when another `Arc` owns the frame. Final-owner
destruction releases the frame complement. Abandonment and reset run no guest
code from this descriptor.

Unarmed receipts remain `()`. The activation bridge makes existing prefixes
visible to a limit that arms inside the guest. A guest can leave a limit active
on suspension; the private rule handoff reselects the armed mode before name
cloning or growth. Already frozen unarmed roots remain visible through the
existing S/root and deferred-capacity machinery. S and expression representation
are unchanged.

The macro source adapter pins the canonical definition, qualified invocation,
selected arm, Ok/Err hygiene and exact guarded insertion. It can describe the
original three-branch expansion without admitting its raw frame. Only the
unchanged guarded insert arm can use B12; this is not a general macro expansion
or admission facility. The production rule path uses typed bridge matches.

Runtime tests in `accounting_mode/tests/rule*.rs`, `tests/frozen.rs`, and
`options/tests/rule.rs` check the ownership and exit matrix. Host tests check
actual cancellation and reset. Local checker controls reject stale, missing or
Pending dependencies, raw clones/results, changed schemas/bodies, and bare
constructors or insertions. Indexing audits explicitly listed declarations and
direct records; it does not follow a call graph. These are local protocol
guarantees, not bounds on guest-private allocation, process RSS, stack depth,
or all option consumers.
