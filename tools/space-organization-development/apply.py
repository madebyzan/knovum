"""Format and test the reviewed structure command integration, without a runtime adapter."""

from pathlib import Path
import subprocess

root = Path("src/features/spaces")
contract = root / "organization/structure/commands/contract.ts"
text = contract.read_text().replace('ids: ids.min(0)', 'ids: z.array(itemIdSchema).max(MAX_STRUCTURE_ITEMS)')
contract.write_text(text)
test = root / "__tests__/structure-contract.test.ts"
text = test.read_text()
old = 'expect(() => parse(source)).toThrow(/bounds/u);'
if old in text:
    text = text.replace(old, 'source.layout.items[2]!.transform = { ...transform, scale: 2 };\n    expect(() => parse(source)).toThrow(/Composed world/u);')
test.write_text(text)
paths = [str(root / "organization/structure/commands"), str(root / "testing/structure-fixtures.ts"), str(root / "__tests__/structure-commands.test.ts"), str(test)]
subprocess.run(["pnpm", "exec", "oxfmt", *paths], check=True)
