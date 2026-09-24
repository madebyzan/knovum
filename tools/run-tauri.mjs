import { execFileSync } from "node:child_process";
import { existsSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const repository = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const args = process.argv.slice(2);
const environment = { ...process.env };

if (args[0] === "dev") {
  configureDevelopmentRuntime(environment);
  if (process.platform === "darwin") configureMacOsDevelopmentSigning(environment);
}

const pnpm = process.platform === "win32" ? "pnpm.cmd" : "pnpm";
execFileSync(pnpm, ["exec", "tauri", ...args], {
  cwd: repository,
  env: environment,
  stdio: "inherit",
});

function configureDevelopmentRuntime(targetEnvironment) {
  const target = hostTarget();
  const directory = join(repository, "src-tauri", "resources", "bundled-runtime", target);
  const windows = process.platform === "win32";
  const tools = {
    KNOVUM_DENO_PATH: windows ? "deno.exe" : "deno",
    KNOVUM_FFMPEG_PATH: windows ? "ffmpeg.exe" : "ffmpeg",
    KNOVUM_WHISPER_PATH: windows ? "whisper-cli.exe" : "whisper-cli",
    KNOVUM_YT_DLP_PATH: windows ? "yt-dlp.exe" : "yt-dlp",
  };
  for (const [key, name] of Object.entries(tools)) {
    if (targetEnvironment[key]) continue;
    const path = join(directory, name);
    if (!existsSync(path)) throw new Error(`Development runtime is missing ${path}`);
    targetEnvironment[key] = path;
  }
}

function configureMacOsDevelopmentSigning(targetEnvironment) {
  const identity = targetEnvironment.KNOVUM_DEV_SIGNING_IDENTITY?.trim();
  if (!identity || identity === "-") {
    throw new Error(
      "macOS desktop development requires KNOVUM_DEV_SIGNING_IDENTITY with a stable code-signing certificate; ad-hoc signing cannot preserve Keychain authorization",
    );
  }
  const identities = execFileSync("security", ["find-identity", "-v", "-p", "codesigning"], {
    encoding: "utf8",
  });
  if (!identities.includes(identity)) {
    throw new Error(`Code-signing identity ${identity} is unavailable in the macOS keychain`);
  }
  const architecture = process.arch === "arm64" ? "AARCH64" : "X86_64";
  targetEnvironment[`CARGO_TARGET_${architecture}_APPLE_DARWIN_RUNNER`] = join(
    repository,
    "tools",
    "run-signed-tauri-dev.mjs",
  );
}

function hostTarget() {
  const platform = { darwin: "macos", win32: "windows" }[process.platform];
  const architecture = { arm64: "arm64", x64: "x64" }[process.arch];
  if (!platform || !architecture) {
    throw new Error(`Unsupported desktop development host: ${process.platform}/${process.arch}`);
  }
  return `${platform}-${architecture}`;
}
