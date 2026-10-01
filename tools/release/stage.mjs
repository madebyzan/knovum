import { resolve } from "node:path";

import { stageTarget } from "./artifacts.mjs";

try {
  const [version, target, bundleDirectory, outputDirectory, ...extra] = process.argv.slice(2);
  if (!version || !target || !bundleDirectory || !outputDirectory || extra.length) {
    throw new Error("Usage: node tools/release/stage.mjs VERSION TARGET BUNDLE_DIRECTORY OUTPUT_DIRECTORY");
  }
  stageTarget({ version, target, bundleDirectory: resolve(bundleDirectory), outputDirectory: resolve(outputDirectory) });
} catch (error) {
  console.error(error instanceof Error ? error.message : "Artifact staging failed");
  process.exitCode = 1;
}
