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
        f"- Desktop app surface present: {_yes_no(capabilities.get('has_desktop_app'))}",
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
