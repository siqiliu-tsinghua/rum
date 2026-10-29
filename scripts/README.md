# Scripts

Helper scripts for developing and testing `rum`. Run them from the repository
root. Each script also prints usage when run with `--help` or on a bad
invocation.

## Performance and profiling

- `core-language-microbench.sh` — the `rum`-vs-`rum` language-performance A/B.
  Runs a labeled workload and writes per-label `timings.tsv`; run it on two
  builds and compare. It measures with in-kernel `AbsoluteTiming` backed by
  Rust's monotonic `Instant`, not the wall clock. Run timing gates only on an
  otherwise idle machine and in the normal workspace sandbox.
- `core-memory-microbench.sh` — the `rum`-vs-`rum` memory-baseline A/B. Pass
  `--no-rss` in sandboxes that deny `peak-rss.py`'s process probes; Rum's
  memory proxies and any enabled allocation count/byte/peak-live profile are
  still recorded, while the RSS columns remain empty.
- `peak-rss.py` — run a command and report its peak RSS in bytes, reading
  `getrusage` rather than depending on which `time` the host ships.
- `sample-self-time.py` — compute per-function self-time from macOS `sample`
  call-graph output. macOS only: there is no Linux producer for this format.
- `sample-attribute-callers.py` — attribute a target leaf symbol's samples to its
  nearest interesting callers.

## Test and run safety

- `run-bounded.py` — run a command under a wall-time bound, killing its whole
  process group on timeout. Use it for anything that may run long or spawn
  children.
- `guarded-test-run.sh` — run a command under a resource (RSS) watchdog that kills
  it before it can threaten the machine.

## Jupyter adapter (optional)

- `rum-jupyter-home.sh` — manage a local Jupyter home for the development kernel.
- `write-rum-jupyter-kernelspec.sh` — write a kernelspec for the `rum` kernel.
- `smoke-rum-jupyter.sh` — a quick end-to-end Jupyter smoke check.
- `rum-jupyter-kernel-smoke.py` — `jupyter_client` smoke tests for the kernel.
- `rum-jupyter-browser-smoke.py` — a headless JupyterLab browser smoke check.

## Maintainer workflow

- `dispatch-brief.sh` — dispatch a Codex task brief for execution (the internal
  director/Codex workflow; see [`CLAUDE.md`](../CLAUDE.md) and
  [`docs/dev/codex/`](../docs/dev/codex/)).
- `report.py` — the structured worker report (`rum-report/1`): `check`,
  `summary`, `show`, `render`.
- `director-gate.py` — the director's gate in one command: the scope check,
  fmt, new warnings, the standing inventory replays (ownership, census
  pairing, compiler oracle, LP01, typed-slot census, fingerprints,
  certificates), the workspace, and, with `--perf`, `arms`, `structure` and
  `panels`. One table out; logs in `.temp/director/gate-NNN<stage>/`.
  `selftest` checks its pure parts. `rule` records a ruling on a FLAGGED step.
- `perf-panel.py` — the rum-vs-rum instruction panel: build both arms at one
  path (`arms`), path-calibrate, diff functions (`structure`), run the
  interleaved perf panel, summarize it, and attribute a label with callgrind.
