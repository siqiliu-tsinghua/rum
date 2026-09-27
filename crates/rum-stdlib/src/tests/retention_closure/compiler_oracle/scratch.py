"""Scratch copying, bounded stable builds, and structured rustc diagnostics.

Diagnostic fields follow https://doc.rust-lang.org/rustc/json.html. Match error
primary spans on full relative file and exact starting line; neither an error
elsewhere nor a secondary definition-note span establishes a resolution claim.
"""

import csv
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess


def read_tsv(path):
    with path.open(newline="") as stream:
        return list(csv.DictReader(stream, delimiter="\t", quoting=csv.QUOTE_NONE))


def offset(text, line, column):
    lines = text.splitlines(keepends=True)
    line, column = int(line), int(column)
    assert 1 <= line <= len(lines) and 1 <= column <= len(lines[line - 1]) + 1
    return sum(map(len, lines[:line - 1])) + column - 1


def refresh_crates(workspace, root):
    """A compiler input snapshot must not retain deleted source modules."""
    workspace, root = workspace.resolve(), root.resolve()
    assert (root.is_relative_to(workspace / 'target/brief-535')
            or root.is_relative_to(workspace / 'target/brief-540')
            or root.is_relative_to(workspace / '.temp/brief-542')
            or root.is_relative_to(workspace / '.temp/brief-543/oracle-reads'))
    destination = root / 'crates'
    assert not destination.is_symlink()
    if destination.exists():
        shutil.rmtree(destination)
    shutil.copytree(workspace / 'crates', destination,
                    ignore=shutil.ignore_patterns('target', '.git', '__pycache__'))


class Scratch:
    def __init__(self, workspace, directory, features):
        self.original = workspace.resolve()
        self.directory = directory.resolve()
        self.root = self.directory / "workspace"
        assert self.root.is_relative_to(self.original / "target/brief-535")
        assert not self.root.is_symlink()
        self.root.mkdir(parents=True, exist_ok=True)
        self.features = features
        self.original_hashes = {}
        # Copies, never hardlinks/symlinks: mutation cannot alias real source.
        for file in ["Cargo.toml", "Cargo.lock", "rust-toolchain.toml"]:
            shutil.copy2(self.original / file, self.root / file)
        refresh_crates(self.original, self.root)
        for path in (self.original / "crates").rglob("*.rs"):
            self.original_hashes[path] = hashlib.sha256(path.read_bytes()).hexdigest()
        version = self.run("toolchain", ["rustc", "+1.85.0", "--version"])
        assert version.returncode == 0 and version.stdout.startswith(b"rustc 1.85.0 "), version.stdout
        self.originals = {}

    def run(self, label, command):
        env = os.environ.copy()
        env["CARGO_TARGET_DIR"] = str(self.directory / "build")
        full = ["python3", str(self.original / "scripts/run-bounded.py"),
                "--timeout", "180", "--", *command]
        result = subprocess.run(full, cwd=self.root, env=env, capture_output=True)
        (self.directory / f"{label}.command.json").write_text(json.dumps(full) + "\n")
        (self.directory / f"{label}.stdout").write_bytes(result.stdout)
        (self.directory / f"{label}.stderr").write_bytes(result.stderr)
        (self.directory / f"{label}.status").write_text(str(result.returncode) + "\n")
        assert result.returncode not in (124, -9, -15), f"bounded compiler failed: {label} ({result.returncode})"
        return result

    def compile(self, label):
        command = ["cargo", "+1.85.0", "check", "--offline", "--locked", "--lib",
                   "-p", "rum-stdlib", "--message-format=json"]
        if self.features == "no-default-features":
            command.append("--no-default-features")
        result = self.run(label, command)
        diagnostics = []
        for line in result.stdout.splitlines():
            if not line.strip():
                continue
            record = json.loads(line)
            if record.get("reason") == "compiler-message" and record["message"]["level"] == "error":
                diagnostics.append(record["message"])
        assert result.returncode == 0 or diagnostics, f"build failed without Rust diagnostics: {label}: {result.stderr.decode()}"
        return result.returncode, diagnostics

    def at(self, diagnostics, file, line, code=None):
        expected = (self.root / file).resolve()
        return [d for d in diagnostics
                if (code is None or (d.get("code") or {}).get("code") == code)
                and any(s["is_primary"] and (self.root / s["file_name"]).resolve() == expected
                        and s["line_start"] == int(line) for s in d["spans"])]

    def text(self, file):
        path = (self.root / file).resolve()
        assert path.is_relative_to(self.root / "crates"), "mutations stay in scratch crate copies"
        assert not path.is_symlink()
        original = (self.original / file).read_text()
        self.originals[path] = original
        assert path.read_text() == original, f"previous mutation not restored: {file}"
        return path, original

    def write(self, label, path, text):
        assert text.count("\n") == self.originals[path].count("\n"), "mutation shifted source lines"
        path.write_text(text)
        # Keep the exact mutated file as evidence, not just the compiler error.
        evidence = self.directory / "mutations" / label / path.relative_to(self.root)
        evidence.parent.mkdir(parents=True, exist_ok=True)
        evidence.write_text(text)

    def signature(self, label, row, prefix):
        path, text = self.text(row[f"{prefix}_file"])
        at = offset(text, row[f"{prefix}_line"], row[f"{prefix}_column"])
        assert text[at] == ")", "signature span must point at closing parenthesis"
        addition = row[f"{prefix}_prefix"] + "__brief_533_probe: ()"
        self.write(label, path, text[:at] + addition + text[at:])

    def expression(self, label, row, make):
        path, text = self.text(row["file"])
        start = offset(text, row["start_line"], row["start_column"])
        end = offset(text, row["end_line"], row["end_column"])
        self.write(label, path, text[:start] + make(text[start:end]) + text[end:])

    def restore(self):
        for path, original in self.originals.items():
            path.write_text(original)
        self.originals.clear()

    def verify_originals(self):
        for path, digest in self.original_hashes.items():
            assert hashlib.sha256(path.read_bytes()).hexdigest() == digest, f"real source changed: {path}"
