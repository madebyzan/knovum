import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  createReadStream,
  createWriteStream,
  copyFileSync,
  chmodSync,
  mkdirSync,
  readFileSync,
  rmSync,
} from "node:fs";
import { dirname, join, resolve as resolvePath } from "node:path";
import { Readable } from "node:stream";
import { pipeline } from "node:stream/promises";
import { fileURLToPath } from "node:url";

const defaultRepository = resolvePath(dirname(fileURLToPath(import.meta.url)), "..");

export async function provisionMediaImportRuntime({ repository = defaultRepository, target }) {
  const runtimeRoot = join(repository, "src-tauri", "resources", "bundled-runtime");
  const manifest = JSON.parse(readFileSync(join(runtimeRoot, "media-import-manifest.json"), "utf8"));
  const descriptor = manifest.targets[target];
  if (!descriptor) fail(`Unsupported media import runtime target: ${target}`);
  const scratch = join(repository, ".media-import-runtime-stage", target);
  const destination = join(runtimeRoot, target);
  rmSync(scratch, { recursive: true, force: true });
  mkdirSync(scratch, { recursive: true });
  mkdirSync(destination, { recursive: true });

  try {
    const ytDlp = await download(
      `https://github.com/yt-dlp/yt-dlp/releases/download/${manifest.ytDlp.version}/${descriptor.ytDlp.asset}`,
      descriptor.ytDlp.sha256,
      join(scratch, descriptor.ytDlp.asset),
    );
    const denoArchive = await download(
      `https://github.com/denoland/deno/releases/download/v${manifest.deno.version}/${descriptor.deno.asset}`,
      descriptor.deno.sha256,
      join(scratch, descriptor.deno.asset),
    );
    const extracted = join(scratch, "deno");
    mkdirSync(extracted);
    execFileSync("tar", ["-xf", denoArchive, "-C", extracted]);

    const windows = target.startsWith("windows-");
    const ytDlpTarget = join(destination, windows ? "yt-dlp.exe" : "yt-dlp");
    const denoTarget = join(destination, windows ? "deno.exe" : "deno");
    copyFileSync(ytDlp, ytDlpTarget);
    const denoBinary = join(extracted, windows ? "deno.exe" : "deno");
    if ((await sha256(denoBinary)) !== descriptor.deno.binarySha256) {
      fail(`Extracted Deno binary SHA-256 mismatch for ${target}`);
    }
    copyFileSync(denoBinary, denoTarget);
    if (!windows) {
      chmodSync(ytDlpTarget, 0o755);
      chmodSync(denoTarget, 0o755);
    }
    console.log(
      `Provisioned yt-dlp ${manifest.ytDlp.version} and Deno ${manifest.deno.version} for ${target}`,
    );
  } finally {
    rmSync(scratch, { recursive: true, force: true });
  }
}

async function download(url, expectedSha256, destination) {
  const response = await fetch(url, { headers: { "User-Agent": "Knovum runtime provisioner" } });
  if (!response.ok || !response.body) fail(`Runtime download failed: HTTP ${response.status} ${url}`);
  await pipeline(Readable.fromWeb(response.body), createWriteStream(destination));
  const actual = await sha256(destination);
  if (actual !== expectedSha256) {
    fail(`Runtime SHA-256 mismatch for ${url}: expected ${expectedSha256}, got ${actual}`);
  }
  return destination;
}

function sha256(filePath) {
  return new Promise((resolve, reject) => {
    const hash = createHash("sha256");
    const stream = createReadStream(filePath);
    stream.on("data", (chunk) => hash.update(chunk));
    stream.on("error", reject);
    stream.on("end", () => resolve(hash.digest("hex")));
  });
}

function fail(message) {
  throw new Error(message);
}
