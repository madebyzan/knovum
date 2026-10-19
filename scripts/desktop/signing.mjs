import { execFileSync } from "node:child_process";
import { openSync, readSync, closeSync } from "node:fs";
import { join, relative, resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { parseArgs } from "node:util";

import { regularFiles as listRuntimeFiles } from "./resources.mjs";

const mach = new Set([
  "cefaedfe",
  "cffaedfe",
  "feedface",
  "feedfacf",
  "cafebabe",
  "bebafeca",
  "cafebabf",
  "bfbafeca",
]);

function compiledFiles(directory, target) {
  return listRuntimeFiles(directory, { allowLinks: target.startsWith("macos-") })
    .filter((name) => {
      const fd = openSync(join(directory, name), "r");
      const header = Buffer.alloc(4);
      try {
        readSync(fd, header, 0, 4, 0);
      } finally {
        closeSync(fd);
      }
      return target.startsWith("macos-")
        ? mach.has(header.toString("hex"))
        : header.subarray(0, 2).toString() === "MZ";
    })
    .map((name) => join(directory, name));
}

function verifyCompiledSignature(path, target) {
  if (target.startsWith("macos-")) {
    execFileSync("codesign", ["--verify", "--strict", path]);
  }
}

export function verifyCompiledFiles(directory, target) {
  for (const path of compiledFiles(directory, target)) verifyCompiledSignature(path, target);
}

export function signCompiledFiles(directory, target, mode = "adhoc") {
  const files = compiledFiles(directory, target);
  let identity = "unsigned";
  if (target.startsWith("macos-")) {
    identity = "-";
    for (const path of files) {
      execFileSync("codesign", ["--force", "--sign", "-", path]);
      verifyCompiledSignature(path, target);
    }
  }
  return { mode, identity, compiledBinaryCount: files.length };
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const { values } = parseArgs({
    options: {
      directory: { type: "string" },
      target: { type: "string" },
      mode: { type: "string" },
      list: { type: "boolean" },
    },
  });
  if (!values.directory || !["macos-arm64", "macos-x64", "windows-x64"].includes(values.target))
    throw new Error("Compiled resource inspection requires a directory and supported target");
  const compiledFileReport = values.list
    ? compiledFiles(values.directory, values.target).map((file) =>
        relative(values.directory, file).replaceAll("\\", "/"),
      )
    : signCompiledFiles(values.directory, values.target, values.mode);
  if (!values.list && compiledFileReport.compiledBinaryCount === 0)
    throw new Error("Runtime contains no compiled executable files");
  process.stdout.write(JSON.stringify(compiledFileReport));
}
