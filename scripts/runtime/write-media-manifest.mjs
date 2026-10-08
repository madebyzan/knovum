import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { writeMediaManifest } from "./media-manifest.mjs";

const repository = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const [directory, target, producerPath] = process.argv.slice(2);
writeMediaManifest(repository, directory, target, JSON.parse(readFileSync(producerPath, "utf8")));
