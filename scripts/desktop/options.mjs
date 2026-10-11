import { parseArgs } from "node:util";

export const runtimeTargets = {
  "x86_64-apple-darwin": "macos-x64",
  "aarch64-apple-darwin": "macos-arm64",
  "x86_64-pc-windows-msvc": "windows-x64",
};
function hostTriple() {
  const pair = `${process.platform}-${process.arch}`;
  const triple = {
    "darwin-x64": "x86_64-apple-darwin",
    "darwin-arm64": "aarch64-apple-darwin",
    "win32-x64": "x86_64-pc-windows-msvc",
  }[pair];
  if (!triple) throw new Error(`Unsupported desktop host: ${pair}`);
  return triple;
}
function validateProfile(args, command, profile, production) {
  if (production && profile !== "release") throw new Error("Production requires the release profile");
  if (command === "dev" && (args.includes("--release") || profile === "release"))
    throw new Error("Use package for distribution profiles; desktop:dev owns the editable environment");
}
function vendorValues(args) {
  const { values } = parseArgs({
    args,
    strict: false,
    allowPositionals: true,
    options: {
      target: { type: "string", short: "t" },
      profile: { type: "string" },
      debug: { type: "boolean", short: "d" },
      bundles: { type: "string", short: "b" },
      "no-bundle": { type: "boolean" },
      release: { type: "boolean" },
    },
  });
  for (const name of ["target", "profile", "bundles"])
    if (values[name] !== undefined && (typeof values[name] !== "string" || !values[name]))
      throw new Error(`--${name} requires a value`);
  for (const name of ["debug", "no-bundle", "release"])
    if (values[name] !== undefined && typeof values[name] !== "boolean")
      throw new Error(`--${name} does not accept a value`);
  return values;
}
function profileForCommand(values, command) {
  return values.profile ?? (values.debug || command === "dev" ? "dev" : "release");
}
export function lockedRunnerArguments(args) {
  if (args.includes("--locked")) return args;
  return [...args, ...(args.includes("--") ? [] : ["--"]), "--locked"];
}
export function buildOptions(input, command, host = hostTriple()) {
  const args = [...input];
  const values = vendorValues(args);
  const production = command === "build" && values.release === true;
  if (production) args.splice(args.indexOf("--release"), 1);
  const profile = profileForCommand(values, command);
  validateProfile(args, command, profile, production);
  const explicit = values.target;
  const triple = explicit ?? host;
  if (triple !== host) throw new Error("Agent distributions must be built on their native target host");
  return {
    args,
    profile,
    production,
    triple,
    target: runtimeTargets[triple],
    explicitTarget: Boolean(explicit),
    bundle: !values["no-bundle"] && values.bundles !== "none",
    bundles: values.bundles?.split(","),
  };
}
