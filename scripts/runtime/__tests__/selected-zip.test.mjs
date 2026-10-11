import assert from "node:assert/strict";
import { existsSync, mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { extractSelectedZip } from "../selected-zip.mjs";

for (const kind of ["regular", "link", "duplicate", "missing"])
  test(`selected archive ${kind} policy`, async (context) => {
    const directory = mkdtempSync(join(tmpdir(), "knovum-zip-"));
    context.after(() => rmSync(directory, { recursive: true, force: true }));
    const output = join(directory, "nested", "binary");
    const archive = fileURLToPath(new URL(`./fixtures/selected-zip/${kind}.zip`, import.meta.url));
    const operation = extractSelectedZip(archive, new Map([["supplier/binary", output]]));
    if (kind !== "regular") {
      await assert.rejects(operation);
      return;
    }
    await operation;
    assert.equal(readFileSync(output, "utf8"), "controlled fixture bytes");
    assert.equal(existsSync(join(directory, "ignored")), false);
    await assert.rejects(extractSelectedZip(archive, new Map([["supplier/binary", output]])), /EEXIST/u);
  });
