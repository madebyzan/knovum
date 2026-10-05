from __future__ import annotations

import subprocess
from pathlib import Path


def main() -> None:
    root = Path(__file__).resolve().parents[1]
    subprocess.run(
        [
            "datamodel-codegen",
            "--input",
            str(root / "schemas" / "runtime.schema.json"),
            "--input-file-type",
            "jsonschema",
            "--output",
            str(root / "src" / "knovum_inference" / "contracts.py"),
            "--output-model-type",
            "pydantic_v2.BaseModel",
            "--target-python-version",
            "3.11",
            "--disable-timestamp",
            "--base-class",
            "knovum_inference.wire_model.WireModel",
            "--snake-case-field",
            "--use-annotated",
            "--use-union-operator",
            "--use-standard-collections",
            "--use-default",
            "--use-double-quotes",
            "--formatters",
            "black",
            "isort",
        ],
        check=True,
    )


if __name__ == "__main__":
    main()
