from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any, Sequence

_REQUIRED_RUNTIME_CAPABILITIES = (
    ("has_toolsets", "toolsets"),
    ("has_memory_tool", "memory"),
    ("has_vision_tool", "vision"),
    ("has_image_generation_tool", "image generation"),
    ("has_video_generation_tool", "video generation"),
    ("has_mcp_server", "MCP server"),
    ("has_desktop_app", "desktop app"),
)


def _yes_no(value: Any) -> str:
    return "yes" if bool(value) else "no"


def _phase0_blockers(inspection: dict[str, Any]) -> list[str]:
    status = str(inspection.get("status", "fail"))
    missing = inspection.get("missing_required_paths", [])
    errors = inspection.get("errors", [])
    capabilities = inspection.get("capabilities", {})
    blockers: list[str] = []

    if status != "pass":
        blockers.append(f"inspection status is {status}")
    if missing:
        blockers.append("required paths are missing")
    if errors:
        blockers.append("inspector errors are present")

    missing_capabilities = [
        label for key, label in _REQUIRED_RUNTIME_CAPABILITIES if not capabilities.get(key)
    ]
    if missing_capabilities:
        blockers.append("runtime capabilities missing: " + ", ".join(missing_capabilities))

    return blockers


def _decision_lines(inspection: dict[str, Any]) -> list[str]:
    blockers = _phase0_blockers(inspection)
    if not blockers:
        return [
            "Joi Runtime initial decision: keep most Hermes Core.",
            "",
            "This validation phase confirms whether Hermes provides the runtime surface needed by Joi before Tauri desktop and domain workflow work begins.",
        ]

    return [
        "Joi Runtime initial decision: blocked pending Phase 0 fixes.",
        "",
        "Phase 0 blockers:",
        *(f"- {blocker}" for blocker in blockers),
    ]


def render_report(inspection: dict[str, Any]) -> str:
    missing = inspection.get("missing_required_paths", [])
    errors = inspection.get("errors", [])
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
        f"- Desktop app surface present: {_yes_no(capabilities.get('has_desktop_app'))}",
        "",
        "## Missing Required Paths",
        "",
    ]

    if missing:
        lines.extend(f"- {item}" for item in missing)
    else:
        lines.append("- none")

    lines.extend(["", "## Inspector Errors", ""])
    if errors:
        lines.extend(f"- {item}" for item in errors)
    else:
        lines.append("- none")

    lines.extend(["", "## Phase 0 Decision", "", *_decision_lines(inspection)])
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
