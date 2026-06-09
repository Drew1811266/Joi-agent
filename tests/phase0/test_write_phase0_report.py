import json
import tempfile
import unittest
from pathlib import Path

from scripts.phase0.write_phase0_report import main, render_report


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

    def test_report_cli_writes_markdown_file(self) -> None:
        inspection = {
            "status": "pass",
            "checkout": "checkout",
            "missing_required_paths": [],
            "project": {"name": "hermes-agent", "version": "0.16.0", "requires_python": ">=3.11"},
            "desktop": {"present": True, "name": "hermes", "version": "0.15.1", "dependency_count": 1},
            "counts": {"registered_tool_files": 72, "skills": 60, "optional_skills": 80},
            "capabilities": {"has_toolsets": True},
        }

        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            input_path = root / "inspection.json"
            output_path = root / "nested" / "report.md"
            input_path.write_text(json.dumps(inspection), encoding="utf-8")

            exit_code = main([str(input_path), "--output", str(output_path)])

            self.assertEqual(exit_code, 0)
            self.assertTrue(output_path.exists())
            self.assertIn("Status: pass", output_path.read_text(encoding="utf-8"))


if __name__ == "__main__":
    unittest.main()
