import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from scripts.phase0.hermes_repo_inspector import inspect_checkout


def write_file(root: Path, relative_path: str, content: str) -> None:
    target = root / relative_path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(content, encoding="utf-8")


def write_valid_checkout(root: Path) -> None:
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


class HermesRepoInspectorTests(unittest.TestCase):
    def test_valid_checkout_returns_runtime_metadata(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_valid_checkout(root)

            result = inspect_checkout(root)

            self.assertEqual(result["status"], "pass")
            self.assertEqual(result["errors"], [])
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

    def test_client_register_file_is_not_counted_as_hermes_tool(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_valid_checkout(root)
            write_file(
                root,
                "tools/client_tool.py",
                "from external import client\nclient.register(name=\"not-hermes\")\n",
            )

            result = inspect_checkout(root)

            self.assertEqual(result["status"], "pass")
            self.assertEqual(result["counts"]["registered_tool_files"], 1)

    def test_malformed_metadata_returns_errors_without_raising(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_valid_checkout(root)
            write_file(root, "pyproject.toml", "[project\n")
            write_file(root, "apps/desktop/package.json", "{\"name\": \"hermes\"\n")

            result = inspect_checkout(root)

            self.assertEqual(result["status"], "fail")
            self.assertGreaterEqual(len(result["errors"]), 2)
            self.assertTrue(any("pyproject.toml" in error for error in result["errors"]))
            self.assertTrue(any("apps/desktop/package.json" in error for error in result["errors"]))

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

    def test_inspection_cli_supports_file_path_invocation(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]

        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp) / "checkout"
            output = Path(tmp) / "inspection.json"
            write_valid_checkout(root)

            completed = subprocess.run(
                [
                    sys.executable,
                    "scripts/phase0/inspect_hermes.py",
                    str(root),
                    "--output",
                    str(output),
                ],
                cwd=repo_root,
                capture_output=True,
                text=True,
            )

            self.assertEqual(completed.returncode, 0, msg=completed.stderr)
            self.assertTrue(output.exists())
            self.assertEqual(json.loads(output.read_text(encoding="utf-8"))["status"], "pass")


if __name__ == "__main__":
    unittest.main()
