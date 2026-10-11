import { execFileSync } from "node:child_process";

import { safeRelativePath } from "./files.mjs";

export function preflightArchive(archive) {
  const limits = { encoding: "utf8", maxBuffer: 64 * 1024 ** 2, timeout: 120_000 };
  const entries = execFileSync("tar", ["-tf", archive], limits).split(/\r?\n/u).filter(Boolean);
  if (!entries.length) throw new Error("Bundled runtime archive is empty");
  if (entries.length > 100_000) throw new Error("Bundled runtime archive contains too many entries");
  const normalized = entries.map(normalizeArchiveEntry);
  if (new Set(normalized).size !== normalized.length) {
    throw new Error("Bundled runtime archive contains duplicate entries");
  }
  const verbose = execFileSync("tar", ["-tvf", archive], limits).split(/\r?\n/u).filter(Boolean);
  if (verbose.length !== entries.length || verbose.some((line) => !["-", "d"].includes(line[0]))) {
    throw new Error("Bundled runtime archive may contain only regular files and directories");
  }
}

function normalizeArchiveEntry(entry) {
  const path = entry.endsWith("/") ? entry.slice(0, -1) : entry;
  const containsControlCharacter = entry.includes("\0") || entry.includes("\n") || entry.includes("\r");
  if (containsControlCharacter || !safeRelativePath(path)) {
    throw new Error(`Bundled runtime archive contains an unsafe path: ${entry}`);
  }
  return path;
}
