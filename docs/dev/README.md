# rum Development Record

This directory is `rum`'s **internal working history**, kept public as part of
the project's transparent development. **You do not need anything here to use or
embed `rum`** — for that, read the [guide](../guide/) and
[reference](../reference/builtins.md).

It is a record, not a contract: documents here describe how a piece of work was
designed and executed at a point in time, and may lag the current code.

## What's here

- **[codex/](./codex/)** — the Codex task briefs. `rum` is developed by a
  supervising director who writes a detailed brief per unit of work and an
  executing agent who implements exactly that brief. `TEMPLATE.md` is the brief
  format; `brief-NNN-<slug>.md` are the packages. The workflow is described in
  [`CLAUDE.md`](../../CLAUDE.md).
- **Design docs, contracts, and surveys** (this directory's top level) — per-line
  design documents and behavioral contracts (for example
  `lazy-sequences-design.md`, `sealed-scoping-design.md`,
  `structured-exceptions-contract.md`, `matcher-flat-optional-contract.md`) and
  the line records (`*-line.md`).
- **[director-method.md](./director-method.md)** and
  **[director-log.md](./director-log.md)** — the director's full method and
  the line-by-line direction log, moved out of `CLAUDE.md` when it was cut to
  200 lines.
- **[usability-pass/](./usability-pass/)** — per-unit findings ledgers from the
  builtin usability sweep.
- **[publication-audit/](./publication-audit/)** — the pre-publication security
  and capability-consistency audit findings.

The active roadmap and the append-only history live at the repository root in
[`TODO.md`](../../TODO.md) and [`PROGRESS.md`](../../PROGRESS.md).
