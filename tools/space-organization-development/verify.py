"""Run real checks, retain full logs, and report compact failures without weakening gates."""

import json
import os
from pathlib import Path
import re
import subprocess

out = Path(os.environ["REPORT_DIRECTORY"])
out.mkdir(parents=True, exist_ok=True)
sha = subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip()
checks = [
    ("Types", ["pnpm", "typecheck"], 180),
    ("Architecture", ["pnpm", "architecture"], 120),
    ("Frontend tests", ["pnpm", "exec", "vitest", "run", "--maxWorkers=4", "--reporter=json", f"--outputFile={out / 'tests.json'}"], 420),
    ("Build", ["pnpm", "build"], 180),
]
results = []
for name, command, timeout in checks:
    log_path = out / (name.lower().replace(" ", "-") + ".log")
    with log_path.open("w") as log:
        try:
            completed = subprocess.run(command, stdout=log, stderr=subprocess.STDOUT, timeout=timeout, check=False)
            code = completed.returncode
        except subprocess.TimeoutExpired:
            log.write(f"\nCheck exceeded its {timeout}s timeout. This is a failure, not a pass.\n")
            code = 124
    text = re.sub(r"\x1b\[[0-?]*[ -/]*[@-~]", "", log_path.read_text(errors="replace"))
    results.append({"name": name, "exitCode": code, "log": text})
    print(f"{name}: exit {code}")

report = [f"## Organization development verification\n\nSource commit: `{sha}`. Actual runner results; not a complete product acceptance.\n", "| Check | Result |", "| --- | --- |"]
for item in results:
    report.append(f"| {item['name']} | {'PASS' if item['exitCode'] == 0 else 'FAIL (' + str(item['exitCode']) + ')'} |")
for item in results:
    if item["exitCode"] == 0:
        continue
    if item["name"] == "Frontend tests" and (out / "tests.json").exists():
        data = json.loads((out / "tests.json").read_text())
        lines = [f"{data.get('numPassedTests')} passed, {data.get('numFailedTests')} failed"]
        for test_file in data.get("testResults", []):
            for test in test_file.get("assertionResults", []):
                if test.get("status") == "failed":
                    lines.append(test_file.get("name", "") + " :: " + test.get("fullName", ""))
                    lines.extend(test.get("failureMessages", []))
            if test_file.get("message"):
                lines.append(test_file["message"])
        detail = "\n".join(lines)
    else:
        detail = item["log"]
    report.append(f"\n<details><summary>{item['name']} failures</summary>\n\n```text\n{detail[:18000]}\n```\n</details>")
report.append("\nComplete logs and test JSON are attached to this workflow run. No gates, tests, or lint rules were disabled.\n")
(out / "report.md").write_text("\n".join(report)[:62000])
(out / "results.json").write_text(json.dumps({"sha": sha, "checks": [{"name": row["name"], "exitCode": row["exitCode"]} for row in results]}, indent=2))
raise SystemExit(0 if all(row["exitCode"] == 0 for row in results) else 1)
