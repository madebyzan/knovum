import assert from "node:assert/strict";
import test from "node:test";
import { fileURLToPath } from "node:url";

import { packagingConfiguration } from "../package.mjs";

const repository = fileURLToPath(new URL("../../../", import.meta.url));

for (const target of ["macos-arm64", "macos-x64", "windows-x64"]) {
  test(`${target}: the default bundle excludes Office engines`, () => {
    const configuration = packagingConfiguration(repository, { target, production: false });
    assert.doesNotMatch(JSON.stringify(configuration.bundle), /office-runtime|LibreOffice/iu);
    assert.ok(configuration.bundle.resources["resources/inference-runtime/"]);
  });
}
