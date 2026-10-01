import { readFileSync, writeFileSync } from "node:fs";

import { relocateManifest } from "./manifest.mjs";

try {
  const [input, baseUrl, output, ...extra] = process.argv.slice(2);
  if (!input || !baseUrl || !output || extra.length) {
    throw new Error("Usage: node tools/release/relocate.mjs INPUT_MANIFEST HTTPS_ARTIFACT_BASE OUTPUT_MANIFEST");
  }
  const manifest = relocateManifest(JSON.parse(readFileSync(input, "utf8")), baseUrl);
  writeFileSync(output, JSON.stringify(manifest, null, 2) + "\n", { flag: "wx" });
  console.log("Relocated manifest written. Upload identical signed artifacts first; publish this manifest last.");
} catch (error) {
  console.error(error instanceof Error ? error.message : "Manifest relocation failed");
  process.exitCode = 1;
}
