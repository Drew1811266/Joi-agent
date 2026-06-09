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
