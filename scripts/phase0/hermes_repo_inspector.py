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
