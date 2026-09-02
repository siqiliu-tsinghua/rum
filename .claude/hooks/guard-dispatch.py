#!/usr/bin/env python3
"""PreToolUse guard: Codex dispatches must be standalone background Bash calls.

Blocks any Bash command containing `dispatch-brief`, `cdx exec` or `codex exec`
when it is
(a) chained with other commands (&&, ||, ;, newline), (b) detached with a bare
`&`, or (c) not run with run_in_background:true. `DRY_RUN=1` calls are exempt
(they only resolve, never dispatch). Redirections like `2>&1` / `< /dev/null`
are fine. Born from brief-103: a foreground `git commit && dispatch` line hit
the Bash tool's 2-minute timeout and SIGTERM'd Codex mid-move.
"""
import json
import re
import sys

data = json.load(sys.stdin)
tool_input = data.get("tool_input") or {}
cmd = tool_input.get("command") or ""
bg = bool(tool_input.get("run_in_background"))

# The CLI is installed as `cdx` on some hosts and `codex` on others.
is_dispatch = (
    "dispatch-brief" in cmd or "cdx exec" in cmd or "codex exec" in cmd
) and "DRY_RUN=1" not in cmd

if is_dispatch:
    problems = []
    if re.search(r"&&|\|\||;|\n", cmd):
        problems.append("it is chained with other commands (&&, ||, ; or newline)")
    if re.search(r"(?<![>&])&(?!&)", cmd):
        problems.append("it is detached with a bare '&'")
    if not bg:
        problems.append("run_in_background is not true (foreground dispatch dies at the Bash tool timeout)")
    if problems:
        reason = (
            "Codex dispatch guard: "
            + "; ".join(problems)
            + ". A dispatch must be its own single Bash call with run_in_background:true. "
            "Commit in a separate command FIRST, then dispatch alone. (DRY_RUN=1 is exempt.)"
        )
        print(json.dumps({
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "permissionDecision": "deny",
                "permissionDecisionReason": reason,
            }
        }))

sys.exit(0)
