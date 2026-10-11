import { spawnSync } from "node:child_process";
// Consume Cargo's documented JSON artifacts instead of recreating its cache or guessing OUT_DIR.
export function buildDesktopDependencies(repository, options) {
  const args = [
    "build",
    "--locked",
    "--manifest-path",
    "src-tauri/Cargo.toml",
    "-p",
    "knovum-inference-worker",
    "-p",
    "knovum-media-libraries",
    "--message-format=json-render-diagnostics",
  ];
  if (options.profile !== "dev") args.push("--profile", options.profile);
  if (options.explicitTarget) args.push("--target", options.triple);
  const buildOutput = spawnSync("cargo", args, {
    cwd: repository,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "inherit"],
    maxBuffer: 32 * 1024 ** 2,
  });
  if (buildOutput.error) throw buildOutput.error;
  if (buildOutput.status !== 0) throw new Error(`Cargo dependency build failed (${buildOutput.status})`);
  const events = buildOutput.stdout
    .split("\n")
    .filter(Boolean)
    .map((line) => JSON.parse(line));
  return desktopArtifacts(events);
}
function desktopArtifacts(events) {
  const output = {};
  for (const event of events) {
    if (event.reason === "compiler-artifact" && event.target.name === "knovum-inference-worker")
      output.worker = event.executable;
    if (event.reason === "build-script-executed") Object.assign(output, libraryPaths(event.env));
  }
  if (!output.worker || !output.inference || !output.media)
    throw new Error("Cargo did not produce the required desktop dependency artifacts");
  return output;
}
function libraryPaths(environment) {
  const output = {};
  for (const [key, value] of environment) {
    if (key === "KNOVUM_INFERENCE_LIBRARIES_DIRECTORY") output.inference = value;
    if (key === "KNOVUM_MEDIA_LIBRARIES_DIRECTORY") output.media = value;
  }
  return output;
}
