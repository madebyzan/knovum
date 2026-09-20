"""Run the repository's real frontend gates and retain complete, line-addressable evidence."""
import json
import os
from pathlib import Path
import re
import subprocess

out = Path(os.environ['REPORT_DIRECTORY'])
out.mkdir(parents=True, exist_ok=True)
sha = subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip()
checks = [
    ('Types', ['pnpm', 'typecheck'], 180),
    ('Desktop types', ['pnpm', 'typecheck:e2e'], 120),
    ('Command authority', ['pnpm', 'check:authority'], 120),
    ('Lint', ['pnpm', 'lint'], 180),
    ('Architecture', ['pnpm', 'architecture'], 150),
    ('Frontend tests', ['pnpm', 'exec', 'vitest', 'run', '--maxWorkers=4', '--reporter=default', '--reporter=json', f'--outputFile.json={out / "tests.json"}'], 600),
    ('Unused code', ['pnpm', 'deadcode'], 180),
    ('Duplication', ['pnpm', 'duplicates'], 180),
    ('Formatting', ['pnpm', 'format:check'], 180),
    ('Build', ['pnpm', 'build'], 240),
]
results = []
ansi = re.compile(r'\x1b\[[0-?]*[ -/]*[@-~]')
for name, command, timeout in checks:
    stem = name.lower().replace(' ', '-')
    path = out / f'{stem}.log'
    with path.open('w') as stream:
        try:
            status = subprocess.run(command, stdout=stream, stderr=subprocess.STDOUT, timeout=timeout, check=False).returncode
        except subprocess.TimeoutExpired:
            stream.write(f'\nCheck exceeded its {timeout}s timeout. This is a failure, not a pass.\n')
            status = 124
    text = ansi.sub('', path.read_text(errors='replace'))
    results.append({'name': name, 'exitCode': status})
    (out / f'{stem}.md').write_text(f'# {name}\n\nSource: `{sha}`. Exit code: {status}.\n\n```text\n{text}\n```\n')
    if name == 'Types':
        diagnostic = '\n'.join(line for line in text.splitlines() if ': error TS' in line and '/__tests__/' not in line)
        (out / 'production-type-errors.md').write_text(f'# Production type diagnostics\n\nSource: `{sha}`.\n\n```text\n{diagnostic}\n```\n')
    print(f'{name}: exit {status}', flush=True)

summary = [f'## Organization development verification\n\nSource commit: `{sha}`. Actual execution, not a product acceptance claim.\n', '| Check | Result |', '| --- | --- |']
for result in results:
    summary.append(f'| {result["name"]} | {"PASS" if result["exitCode"] == 0 else "FAIL (" + str(result["exitCode"]) + ")"} |')
if (out / 'tests.json').exists():
    data = json.loads((out / 'tests.json').read_text())
    summary.append(f'\nFrontend tests: {data.get("numPassedTests")} passed, {data.get("numFailedTests")} failed; {data.get("numPendingTests")} pending.\n')
    failures = []
    for file in data.get('testResults', []):
        for test in file.get('assertionResults', []):
            if test.get('status') == 'failed':
                failures.append(file.get('name', '') + ' :: ' + test.get('fullName', ''))
                failures.extend(test.get('failureMessages', []))
        if file.get('message'): failures.append(file['message'])
    (out / 'test-failures.md').write_text('# Complete failing test diagnostics\n\n```text\n' + '\n'.join(failures) + '\n```\n')
summary.append('\nEvery check has a complete companion Markdown/log artifact. Rust validation is a separate job. No gate, test, or lint rule is disabled.\n')
(out / 'report.md').write_text('\n'.join(summary))
(out / 'results.json').write_text(json.dumps({'sha': sha, 'checks': results}, indent=2))
raise SystemExit(0 if all(result['exitCode'] == 0 for result in results) else 1)
