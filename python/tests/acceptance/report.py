from __future__ import annotations

import argparse
import json
import sys
from datetime import UTC, datetime
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Evaluate real native inference evidence against a fixed corpus")
    parser.add_argument("--corpus", type=Path, required=True)
    parser.add_argument("--results", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    return parser.parse_args()


def main() -> int:
    # -I excludes the script directory; add only this repository-owned acceptance package.
    sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
    from acceptance.corpus import Corpus
    from acceptance.evaluator import evaluate
    from acceptance.evidence import EvidenceError, EvidenceUnavailable, load_evidence
    from acceptance.results import Results

    args = parse_args()
    missing = [str(path) for path in (args.corpus, args.results) if not path.is_file()]
    if missing:
        report = _terminal_report("unavailable", f"required acceptance input is unavailable: {', '.join(missing)}")
        exit_code = 2
    else:
        try:
            report = evaluate(
                load_evidence(args.corpus, Corpus), load_evidence(args.results, Results), args.corpus, args.results
            )
            exit_code = 0 if report["status"] == "passed" else 2 if report["status"] == "unavailable" else 1
        except EvidenceUnavailable as error:
            report = _terminal_report("unavailable", str(error))
            exit_code = 2
        except EvidenceError as error:
            report = _terminal_report("failed", str(error))
            exit_code = 1
    args.output.parent.mkdir(parents=True, exist_ok=True)
    temporary = args.output.with_name(f".{args.output.name}.tmp")
    temporary.write_text(
        json.dumps(report, ensure_ascii=False, indent=2, sort_keys=True, allow_nan=False) + "\n", encoding="utf-8"
    )
    temporary.replace(args.output)
    return exit_code


def _terminal_report(status: str, reason: str) -> dict[str, object]:
    return {
        "schemaVersion": 1,
        "status": status,
        "generatedAt": datetime.now(UTC).isoformat(),
        "reason": reason,
    }


if __name__ == "__main__":
    sys.exit(main())
