# Hermes Fork Validation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a repeatable Phase 0 validation toolkit that fetches a manageable Hermes Agent checkout, inspects the runtime surface, and produces a written report before Joi product code begins.

**Architecture:** The plan creates small Phase 0 scripts under `scripts/phase0/` and tests under `tests/phase0/`. Temporary Hermes source lives under ignored `.external/hermes-agent`, while durable validation artifacts live under `docs/superpowers/reports/`.

**Tech Stack:** PowerShell, Git sparse checkout, Python 3.11+ standard library, `unittest`, JSON, Markdown.

---

## Scope Check

The approved Joi Agent spec spans multiple independent subsystems: Tauri desktop, domain backend, Hermes runtime fork, research, video analysis, prompt adapters, memory, import/export, and packaging. This plan intentionally covers only **Phase 0: Hermes Fork Validation**. Later phases should each get their own plan after this phase confirms the runtime boundary.

## File Structure

- Create `scripts/phase0/hermes_repo_inspector.py`: pure Python library that inspects a Hermes checkout and returns structured metadata.
- Create `tests/phase0/test_hermes_repo_inspector.py`: unit tests for the inspector using a synthetic Hermes-like fixture.
- Create `scripts/phase0/inspect_hermes.py`: CLI wrapper that writes inspection JSON.
- Create `scripts/phase0/fetch-hermes.ps1`: PowerShell script that fetches Hermes into `.external/hermes-agent` using sparse checkout.
- Create `scripts/phase0/write_phase0_report.py`: CLI that converts inspection JSON into a Markdown validation report.
- Create `tests/phase0/test_write_phase0_report.py`: unit tests for report rendering.
- Generate `docs/superpowers/reports/hermes-phase0-inspection.json`: durable machine-readable inspection result.
- Generate `docs/superpowers/reports/hermes-phase0-report.md`: durable human-readable validation report.

## Task 1: Add Hermes Checkout Inspector

**Files:**
- Create: `scripts/phase0/hermes_repo_inspector.py`
- Create: `tests/phase0/test_hermes_repo_inspector.py`

- [ ] **Step 1: Write failing tests for checkout inspection**

Create `tests/phase0/test_hermes_repo_inspector.py`:

```python
import tempfile
import unittest
from pathlib import Path

from scripts.phase0.hermes_repo_inspector import inspect_checkout


def write_file(root: Path, relative_path: str, content: str) -> None:
    target = root / relative_path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(content, encoding="utf-8")


class HermesRepoInspectorTests(unittest.TestCase):
    def test_valid_checkout_returns_runtime_metadata(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_file(root, "README.md", "# Hermes Agent\n")
            write_file(root, "LICENSE", "MIT License\n")
            write_file(
                root,
                "pyproject.toml",
                (
                    "[project]\n"
                    "name = \"hermes-agent\"\n"
                    "version = \"0.16.0\"\n"
                    "requires-python = \">=3.11,<3.14\"\n"
                ),
            )
            write_file(root, "run_agent.py", "def main():\n    return None\n")
            write_file(root, "model_tools.py", "def handle_function_call():\n    return None\n")
            write_file(root, "toolsets.py", "TOOLSETS = {\"vision\": {\"tools\": [\"vision_analyze\"]}}\n")
            write_file(root, "agent/__init__.py", "")
            write_file(root, "tools/registry.py", "class Registry:\n    pass\n")
            write_file(
                root,
                "tools/example_tool.py",
                "from tools.registry import registry\nregistry.register(name=\"example\", toolset=\"test\", schema={}, handler=lambda args: args)\n",
            )
            write_file(root, "skills/example/SKILL.md", "---\nname: example\n---\n# Example\n")
            write_file(root, "optional-skills/example/SKILL.md", "---\nname: optional\n---\n# Optional\n")
            write_file(root, "website/docs/developer-guide/architecture.md", "# Architecture\n")
            write_file(
                root,
                "apps/desktop/package.json",
                "{\"name\":\"hermes\",\"version\":\"0.15.1\",\"dependencies\":{\"react\":\"^19.0.0\"}}\n",
            )

            result = inspect_checkout(root)

            self.assertEqual(result["status"], "pass")
            self.assertEqual(result["project"]["name"], "hermes-agent")
            self.assertEqual(result["project"]["version"], "0.16.0")
            self.assertEqual(result["project"]["requires_python"], ">=3.11,<3.14")
            self.assertEqual(result["desktop"]["version"], "0.15.1")
            self.assertEqual(result["counts"]["registered_tool_files"], 1)
            self.assertEqual(result["counts"]["skills"], 1)
            self.assertEqual(result["counts"]["optional_skills"], 1)
            self.assertTrue(result["capabilities"]["has_toolsets"])
            self.assertTrue(result["capabilities"]["has_desktop_app"])

    def test_missing_required_paths_marks_checkout_failed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_file(root, "README.md", "# Partial\n")

            result = inspect_checkout(root)

            self.assertEqual(result["status"], "fail")
            self.assertIn("pyproject.toml", result["missing_required_paths"])
            self.assertIn("tools", result["missing_required_paths"])


if __name__ == "__main__":
    unittest.main()
```

- [ ] **Step 2: Run tests and verify they fail**

Run:

```powershell
python -m unittest tests.phase0.test_hermes_repo_inspector -v
```

Expected: fail with `ModuleNotFoundError: No module named 'scripts.phase0.hermes_repo_inspector'`.

- [ ] **Step 3: Add package marker files**

Create `scripts/__init__.py`:

```python
"""Project scripts package."""
```

Create `scripts/phase0/__init__.py`:

```python
"""Phase 0 validation utilities."""
```

Create `tests/__init__.py`:

```python
"""Project tests package."""
```

Create `tests/phase0/__init__.py`:

```python
"""Phase 0 tests."""
```

- [ ] **Step 4: Implement the inspector library**

Create `scripts/phase0/hermes_repo_inspector.py`:

```python
from __future__ import annotations

import ast
import json
import tomllib
from pathlib import Path
from typing import Any


REQUIRED_PATHS = [
    "README.md",
    "LICENSE",
    "pyproject.toml",
    "run_agent.py",
    "model_tools.py",
    "toolsets.py",
    "agent",
    "tools",
    "skills",
    "optional-skills",
    "website/docs/developer-guide/architecture.md",
]


def _read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8", errors="replace")


def _load_pyproject(root: Path) -> dict[str, str]:
    path = root / "pyproject.toml"
    if not path.exists():
        return {"name": "", "version": "", "requires_python": ""}
    with path.open("rb") as handle:
        data = tomllib.load(handle)
    project = data.get("project", {})
    return {
        "name": str(project.get("name", "")),
        "version": str(project.get("version", "")),
        "requires_python": str(project.get("requires-python", "")),
    }


def _load_desktop_package(root: Path) -> dict[str, Any]:
    path = root / "apps/desktop/package.json"
    if not path.exists():
        return {"present": False, "name": "", "version": "", "dependency_count": 0}
    data = json.loads(_read_text(path))
    dependencies = data.get("dependencies", {})
    dev_dependencies = data.get("devDependencies", {})
    return {
        "present": True,
        "name": str(data.get("name", "")),
        "version": str(data.get("version", "")),
        "dependency_count": len(dependencies) + len(dev_dependencies),
    }


def _module_registers_tool(path: Path) -> bool:
    try:
        tree = ast.parse(_read_text(path))
    except SyntaxError:
        return False
    for node in tree.body:
        if not isinstance(node, ast.Expr):
            continue
        call = node.value
        if not isinstance(call, ast.Call):
            continue
        func = call.func
        if isinstance(func, ast.Attribute) and func.attr == "register":
            return True
    return False


def _count_registered_tool_files(root: Path) -> int:
    tools_dir = root / "tools"
    if not tools_dir.exists():
        return 0
    return sum(1 for path in tools_dir.glob("*.py") if _module_registers_tool(path))


def _count_skill_files(root: Path, directory_name: str) -> int:
    directory = root / directory_name
    if not directory.exists():
        return 0
    return sum(1 for _ in directory.rglob("SKILL.md"))


def _has_text(root: Path, relative_path: str, needle: str) -> bool:
    path = root / relative_path
    if not path.exists():
        return False
    return needle in _read_text(path)


def inspect_checkout(root: str | Path) -> dict[str, Any]:
    checkout = Path(root).resolve()
    missing = [relative for relative in REQUIRED_PATHS if not (checkout / relative).exists()]
    project = _load_pyproject(checkout)
    desktop = _load_desktop_package(checkout)
    counts = {
        "registered_tool_files": _count_registered_tool_files(checkout),
        "skills": _count_skill_files(checkout, "skills"),
        "optional_skills": _count_skill_files(checkout, "optional-skills"),
    }
    capabilities = {
        "has_toolsets": _has_text(checkout, "toolsets.py", "TOOLSETS ="),
        "has_memory_tool": (checkout / "tools/memory_tool.py").exists(),
        "has_vision_tool": (checkout / "tools/vision_tools.py").exists(),
        "has_image_generation_tool": (checkout / "tools/image_generation_tool.py").exists(),
        "has_video_generation_tool": (checkout / "tools/video_generation_tool.py").exists(),
        "has_mcp_server": (checkout / "mcp_serve.py").exists(),
        "has_desktop_app": desktop["present"],
    }
    status = "pass" if not missing and project["name"] == "hermes-agent" else "fail"
    return {
        "status": status,
        "checkout": str(checkout),
        "missing_required_paths": missing,
        "project": project,
        "desktop": desktop,
        "counts": counts,
        "capabilities": capabilities,
    }
```

- [ ] **Step 5: Run tests and verify they pass**

Run:

```powershell
python -m unittest tests.phase0.test_hermes_repo_inspector -v
```

Expected: two tests pass.

- [ ] **Step 6: Commit Task 1**

Run:

```powershell
git add scripts/__init__.py scripts/phase0/__init__.py scripts/phase0/hermes_repo_inspector.py tests/__init__.py tests/phase0/__init__.py tests/phase0/test_hermes_repo_inspector.py
git commit -m "test: add Hermes checkout inspector"
```

Expected: commit succeeds.

## Task 2: Add Inspection CLI

**Files:**
- Create: `scripts/phase0/inspect_hermes.py`
- Modify: `tests/phase0/test_hermes_repo_inspector.py`

- [ ] **Step 1: Add failing CLI test**

Append this test method inside `HermesRepoInspectorTests` in `tests/phase0/test_hermes_repo_inspector.py`:

```python
    def test_inspection_cli_writes_json_file(self) -> None:
        from scripts.phase0.inspect_hermes import main

        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp) / "checkout"
            output = Path(tmp) / "inspection.json"
            write_file(root, "README.md", "# Hermes Agent\n")
            write_file(root, "LICENSE", "MIT License\n")
            write_file(
                root,
                "pyproject.toml",
                "[project]\nname = \"hermes-agent\"\nversion = \"0.16.0\"\nrequires-python = \">=3.11,<3.14\"\n",
            )
            write_file(root, "run_agent.py", "")
            write_file(root, "model_tools.py", "")
            write_file(root, "toolsets.py", "TOOLSETS = {}\n")
            write_file(root, "agent/__init__.py", "")
            write_file(root, "tools/registry.py", "")
            write_file(root, "skills/example/SKILL.md", "# Example\n")
            write_file(root, "optional-skills/example/SKILL.md", "# Optional\n")
            write_file(root, "website/docs/developer-guide/architecture.md", "# Architecture\n")

            exit_code = main([str(root), "--output", str(output)])

            self.assertEqual(exit_code, 0)
            self.assertTrue(output.exists())
            self.assertIn("\"status\": \"pass\"", output.read_text(encoding="utf-8"))
```

- [ ] **Step 2: Run test and verify it fails**

Run:

```powershell
python -m unittest tests.phase0.test_hermes_repo_inspector.HermesRepoInspectorTests.test_inspection_cli_writes_json_file -v
```

Expected: fail with `ModuleNotFoundError: No module named 'scripts.phase0.inspect_hermes'`.

- [ ] **Step 3: Implement inspection CLI**

Create `scripts/phase0/inspect_hermes.py`:

```python
from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Sequence

from scripts.phase0.hermes_repo_inspector import inspect_checkout


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Inspect a Hermes Agent checkout.")
    parser.add_argument("checkout", help="Path to the Hermes Agent checkout")
    parser.add_argument("--output", help="Path to write inspection JSON")
    args = parser.parse_args(argv)

    result = inspect_checkout(Path(args.checkout))
    rendered = json.dumps(result, ensure_ascii=False, indent=2) + "\n"

    if args.output:
        output = Path(args.output)
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(rendered, encoding="utf-8")
    else:
        print(rendered, end="")

    return 0 if result["status"] == "pass" else 2


if __name__ == "__main__":
    raise SystemExit(main())
```

- [ ] **Step 4: Run CLI test and full inspector test suite**

Run:

```powershell
python -m unittest tests.phase0.test_hermes_repo_inspector -v
```

Expected: three tests pass.

- [ ] **Step 5: Commit Task 2**

Run:

```powershell
git add scripts/phase0/inspect_hermes.py tests/phase0/test_hermes_repo_inspector.py
git commit -m "feat: add Hermes inspection CLI"
```

Expected: commit succeeds.

## Task 3: Add Hermes Sparse Fetch Script

**Files:**
- Create: `scripts/phase0/fetch-hermes.ps1`

- [ ] **Step 1: Create the PowerShell fetch script**

Create `scripts/phase0/fetch-hermes.ps1`:

```powershell
[CmdletBinding()]
param(
  [string]$RepoUrl = "https://github.com/NousResearch/hermes-agent.git",
  [string]$Destination = "",
  [string]$Ref = "main",
  [switch]$Force,
  [switch]$PlanOnly
)

$ErrorActionPreference = "Stop"

$repoRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot "..\.."))
$externalRoot = [System.IO.Path]::GetFullPath((Join-Path $repoRoot ".external"))
if ([string]::IsNullOrWhiteSpace($Destination)) {
  $Destination = Join-Path $externalRoot "hermes-agent"
}
$destinationFull = [System.IO.Path]::GetFullPath($Destination)

$sparsePaths = @(
  "README.md",
  "LICENSE",
  "pyproject.toml",
  "uv.lock",
  "run_agent.py",
  "model_tools.py",
  "toolsets.py",
  "mcp_serve.py",
  "hermes_state.py",
  "hermes_constants.py",
  "agent",
  "tools",
  "skills",
  "optional-skills",
  "plugins",
  "providers",
  "hermes_cli",
  "gateway",
  "website/docs",
  "apps/desktop/package.json",
  "apps/desktop/src",
  "apps/desktop/electron",
  "apps/desktop/DESIGN.md",
  "apps/desktop/README.md"
)

if ($PlanOnly) {
  [PSCustomObject]@{
    repo_url = $RepoUrl
    destination = $destinationFull
    ref = $Ref
    sparse_paths = $sparsePaths
  } | ConvertTo-Json -Depth 4
  exit 0
}

if (-not $destinationFull.StartsWith($externalRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
  throw "Destination must stay inside $externalRoot. Received: $destinationFull"
}

New-Item -ItemType Directory -Force -Path $externalRoot | Out-Null

if (Test-Path $destinationFull) {
  if (-not $Force) {
    throw "Destination already exists: $destinationFull. Re-run with -Force to replace it."
  }
  Remove-Item -LiteralPath $destinationFull -Recurse -Force
}

git --version | Out-Host
git clone --depth 1 --filter=blob:none --sparse --branch $Ref $RepoUrl $destinationFull
git -C $destinationFull sparse-checkout set --no-cone @sparsePaths
git -C $destinationFull status --short

[PSCustomObject]@{
  status = "fetched"
  destination = $destinationFull
  ref = $Ref
} | ConvertTo-Json -Depth 3
```

- [ ] **Step 2: Run plan-only mode**

Run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/phase0/fetch-hermes.ps1 -PlanOnly
```

Expected: JSON output contains `"destination"` and `"sparse_paths"`.

- [ ] **Step 3: Run script parser check**

Run:

```powershell
powershell -NoProfile -Command "$null = [System.Management.Automation.Language.Parser]::ParseFile('scripts/phase0/fetch-hermes.ps1', [ref]$null, [ref]$null); 'parse-ok'"
```

Expected: `parse-ok`.

- [ ] **Step 4: Commit Task 3**

Run:

```powershell
git add scripts/phase0/fetch-hermes.ps1
git commit -m "feat: add Hermes sparse fetch script"
```

Expected: commit succeeds.

## Task 4: Add Phase 0 Report Renderer

**Files:**
- Create: `scripts/phase0/write_phase0_report.py`
- Create: `tests/phase0/test_write_phase0_report.py`

- [ ] **Step 1: Write failing report renderer tests**

Create `tests/phase0/test_write_phase0_report.py`:

```python
import unittest

from scripts.phase0.write_phase0_report import render_report


class Phase0ReportTests(unittest.TestCase):
    def test_render_report_includes_status_and_runtime_counts(self) -> None:
        inspection = {
            "status": "pass",
            "checkout": "D:/Software Project/Joi-agent/.external/hermes-agent",
            "missing_required_paths": [],
            "project": {
                "name": "hermes-agent",
                "version": "0.16.0",
                "requires_python": ">=3.11,<3.14",
            },
            "desktop": {
                "present": True,
                "name": "hermes",
                "version": "0.15.1",
                "dependency_count": 84,
            },
            "counts": {
                "registered_tool_files": 72,
                "skills": 60,
                "optional_skills": 80,
            },
            "capabilities": {
                "has_toolsets": True,
                "has_memory_tool": True,
                "has_vision_tool": True,
                "has_image_generation_tool": True,
                "has_video_generation_tool": True,
                "has_mcp_server": True,
                "has_desktop_app": True,
            },
        }

        report = render_report(inspection)

        self.assertIn("# Hermes Phase 0 Validation Report", report)
        self.assertIn("Status: pass", report)
        self.assertIn("Hermes version: 0.16.0", report)
        self.assertIn("Registered tool files: 72", report)
        self.assertIn("Joi Runtime initial decision: keep most Hermes Core", report)

    def test_render_report_lists_missing_paths(self) -> None:
        inspection = {
            "status": "fail",
            "checkout": "missing",
            "missing_required_paths": ["pyproject.toml", "tools"],
            "project": {"name": "", "version": "", "requires_python": ""},
            "desktop": {"present": False, "name": "", "version": "", "dependency_count": 0},
            "counts": {"registered_tool_files": 0, "skills": 0, "optional_skills": 0},
            "capabilities": {},
        }

        report = render_report(inspection)

        self.assertIn("Status: fail", report)
        self.assertIn("- pyproject.toml", report)
        self.assertIn("- tools", report)


if __name__ == "__main__":
    unittest.main()
```

- [ ] **Step 2: Run tests and verify they fail**

Run:

```powershell
python -m unittest tests.phase0.test_write_phase0_report -v
```

Expected: fail with `ModuleNotFoundError: No module named 'scripts.phase0.write_phase0_report'`.

- [ ] **Step 3: Implement report renderer**

Create `scripts/phase0/write_phase0_report.py`:

```python
from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any, Sequence


def _yes_no(value: Any) -> str:
    return "yes" if bool(value) else "no"


def render_report(inspection: dict[str, Any]) -> str:
    missing = inspection.get("missing_required_paths", [])
    project = inspection.get("project", {})
    desktop = inspection.get("desktop", {})
    counts = inspection.get("counts", {})
    capabilities = inspection.get("capabilities", {})

    lines = [
        "# Hermes Phase 0 Validation Report",
        "",
        f"Status: {inspection.get('status', 'fail')}",
        f"Checkout: {inspection.get('checkout', '')}",
        "",
        "## Runtime Identity",
        "",
        f"- Project name: {project.get('name', '')}",
        f"- Hermes version: {project.get('version', '')}",
        f"- Python requirement: {project.get('requires_python', '')}",
        f"- Desktop app present: {_yes_no(desktop.get('present'))}",
        f"- Desktop version: {desktop.get('version', '')}",
        f"- Desktop dependency count: {desktop.get('dependency_count', 0)}",
        "",
        "## Runtime Surface",
        "",
        f"- Registered tool files: {counts.get('registered_tool_files', 0)}",
        f"- Bundled skills: {counts.get('skills', 0)}",
        f"- Optional skills: {counts.get('optional_skills', 0)}",
        f"- Toolsets present: {_yes_no(capabilities.get('has_toolsets'))}",
        f"- Memory tool present: {_yes_no(capabilities.get('has_memory_tool'))}",
        f"- Vision tool present: {_yes_no(capabilities.get('has_vision_tool'))}",
        f"- Image generation tool present: {_yes_no(capabilities.get('has_image_generation_tool'))}",
        f"- Video generation tool present: {_yes_no(capabilities.get('has_video_generation_tool'))}",
        f"- MCP server present: {_yes_no(capabilities.get('has_mcp_server'))}",
        "",
        "## Missing Required Paths",
        "",
    ]

    if missing:
        lines.extend(f"- {item}" for item in missing)
    else:
        lines.append("- none")

    lines.extend(
        [
            "",
            "## Phase 0 Decision",
            "",
            "Joi Runtime initial decision: keep most Hermes Core.",
            "",
            "This validation phase confirms whether Hermes provides the runtime surface needed by Joi before Tauri desktop and domain workflow work begins.",
        ]
    )
    return "\n".join(lines) + "\n"


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Write the Hermes Phase 0 report.")
    parser.add_argument("inspection_json", help="Path to inspection JSON")
    parser.add_argument("--output", required=True, help="Path to write Markdown report")
    args = parser.parse_args(argv)

    inspection = json.loads(Path(args.inspection_json).read_text(encoding="utf-8"))
    report = render_report(inspection)
    output = Path(args.output)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(report, encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
```

- [ ] **Step 4: Run report tests**

Run:

```powershell
python -m unittest tests.phase0.test_write_phase0_report -v
```

Expected: two tests pass.

- [ ] **Step 5: Run all Phase 0 unit tests**

Run:

```powershell
python -m unittest discover tests/phase0 -v
```

Expected: five tests pass.

- [ ] **Step 6: Commit Task 4**

Run:

```powershell
git add scripts/phase0/write_phase0_report.py tests/phase0/test_write_phase0_report.py
git commit -m "feat: add Hermes validation report renderer"
```

Expected: commit succeeds.

## Task 5: Run Hermes Phase 0 Validation

**Files:**
- Generate: `docs/superpowers/reports/hermes-phase0-inspection.json`
- Generate: `docs/superpowers/reports/hermes-phase0-report.md`

- [ ] **Step 1: Confirm Python version**

Run:

```powershell
python --version
```

Expected: Python version is 3.11 or newer.

- [ ] **Step 2: Fetch Hermes sparse checkout**

Run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/phase0/fetch-hermes.ps1 -Force
```

Expected: JSON output ends with `"status": "fetched"` and destination `.external\hermes-agent`.

- [ ] **Step 3: Inspect Hermes checkout**

Run:

```powershell
python scripts/phase0/inspect_hermes.py .external/hermes-agent --output docs/superpowers/reports/hermes-phase0-inspection.json
```

Expected: exit code `0`; `docs/superpowers/reports/hermes-phase0-inspection.json` contains `"status": "pass"`.

- [ ] **Step 4: Generate Markdown report**

Run:

```powershell
python scripts/phase0/write_phase0_report.py docs/superpowers/reports/hermes-phase0-inspection.json --output docs/superpowers/reports/hermes-phase0-report.md
```

Expected: `docs/superpowers/reports/hermes-phase0-report.md` exists and contains `Joi Runtime initial decision: keep most Hermes Core`.

- [ ] **Step 5: Run all Phase 0 tests again**

Run:

```powershell
python -m unittest discover tests/phase0 -v
```

Expected: five tests pass.

- [ ] **Step 6: Commit Task 5**

Run:

```powershell
git add docs/superpowers/reports/hermes-phase0-inspection.json docs/superpowers/reports/hermes-phase0-report.md
git commit -m "docs: add Hermes Phase 0 validation report"
```

Expected: commit succeeds.

## Task 6: Phase 0 Completion Review

**Files:**
- Review: `docs/superpowers/specs/2026-06-09-joi-agent-design.md`
- Review: `docs/superpowers/reports/hermes-phase0-report.md`
- Review: `docs/superpowers/reports/hermes-phase0-inspection.json`

- [ ] **Step 1: Verify spec-to-report coverage**

Run:

```powershell
Select-String -Path docs/superpowers/reports/hermes-phase0-report.md -Pattern "Memory tool present|Vision tool present|MCP server present|Video generation tool present|Joi Runtime initial decision"
```

Expected: output includes all five searched phrases.

- [ ] **Step 2: Verify ignored Hermes checkout is not staged**

Run:

```powershell
git status --short
```

Expected: no `.external/` files appear in staged or unstaged output.

- [ ] **Step 3: Record Phase 0 completion commit if review changed files**

If Step 1 or Step 2 causes a report correction, run:

```powershell
git add docs/superpowers/reports/hermes-phase0-report.md docs/superpowers/reports/hermes-phase0-inspection.json
git commit -m "docs: finalize Hermes Phase 0 validation"
```

Expected: commit succeeds when files changed; if no files changed, skip this command.

- [ ] **Step 4: Handoff to Phase 1 planning**

Write a short final note that includes:

```text
Phase 0 complete. Hermes runtime validation passed. Next recommended plan: Phase 1 Joi data model and local project store.
```

Expected: the user can decide whether to start the Phase 1 plan.

## Self-Review

- Spec coverage: this plan covers Phase 0 from the approved design spec: fork validation, runtime entry surface, dependency boundary, and report generation. Desktop, domain workflow, prompt adapters, and packaging are intentionally covered by later phase plans.
- Red-flag scan: the plan uses concrete files, code, commands, and expected results.
- Type consistency: Python functions are consistently named `inspect_checkout`, `render_report`, and `main`; report and JSON paths match across tasks.
