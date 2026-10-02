import { execFileSync } from "node:child_process";
import { existsSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const repository = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const args = process.argv.slice(2);
const environment = { ...process.env };

if (args[0] === "dev") configureDevelopmentRuntime(environment);

const cli = createRequire(import.meta.url).resolve("@tauri-apps/cli/tauri.js");
execFileSync(process.execPath, [cli, ...args], {
  cwd: repository,
  env: environment,
  stdio: "inherit",
});

function configureDevelopmentRuntime(targetEnvironment) {
  const target = hostTarget();
  const directory = join(repository, "src-tauri", "resources", "bundled-runtime", target);
  const windows = process.platform === "win32";
  const tools = {
    KNOVUM_FFMPEG_PATH: windows ? "ffmpeg.exe" : "ffmpeg",
    KNOVUM_WHISPER_PATH: windows ? "whisper-cli.exe" : "whisper-cli",
  };
  for (const [key, name] of Object.entries(tools)) {
    if (targetEnvironment[key]) continue;
    const path = join(directory, name);
    if (!existsSync(path)) throw new Error(`Development runtime is missing ${path}`);
    targetEnvironment[key] = path;
  }
}

function hostTarget() {
  const platform = { darwin: "macos", win32: "windows" }[process.platform];
  const architecture = { arm64: "arm64", x64: "x64" }[process.arch];
  if (!platform || !architecture) {
    throw new Error(`Unsupported desktop development host: ${process.platform}/${process.arch}`);
  }
  return `${platform}-${architecture}`;
}
