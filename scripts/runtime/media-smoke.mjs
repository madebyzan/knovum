import { execFileSync } from "node:child_process";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";

import { mediaExecutable } from "./media-manifest.mjs";

export function mediaSmoke(root, target, version) {
  const executable = join(root, mediaExecutable(target));
  const run = (args, cwd = dirname(root)) =>
    execFileSync(executable, args, { cwd, encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] });
  const actual = run(["-version"]);
  if (!actual.startsWith(`ffmpeg version ${version} `) && !actual.startsWith(`ffmpeg version ${version}-`))
    throw new Error("Core FFmpeg version differs");
  if (run(["-hide_banner", "-buildconf"]).includes("--enable-nonfree"))
    throw new Error("Core FFmpeg must remain redistributable");
  const filters = run(["-hide_banner", "-filters"]);
  if (["subtitles", "astats", "loudnorm", "concat"].some((name) => !filters.includes(` ${name} `)))
    throw new Error("Core FFmpeg lacks required filters");
  const encoder = target === "windows-x64" ? "h264_mf" : "h264_videotoolbox";
  if (!run(["-hide_banner", "-encoders"]).includes(` ${encoder} `))
    throw new Error("Core FFmpeg lacks the native H.264 encoder");
  const common = ["-hide_banner", "-loglevel", "error", "-nostdin"];
  run([
    ...common,
    "-f",
    "lavfi",
    "-i",
    "color=c=black:s=64x64:r=10:d=0.2",
    "-f",
    "lavfi",
    "-i",
    "sine=frequency=440:sample_rate=48000:duration=0.2",
    "-af",
    "loudnorm=I=-18:TP=-1:LRA=11",
    "-c:v",
    "libx264",
    "-c:a",
    "aac",
    "-f",
    "null",
    "-",
  ]);
  const scratch = mkdtempSync(join(dirname(root), "media-smoke-中文-"));
  try {
    writeFileSync(
      join(scratch, "captions.ass"),
      "[Script Info]\nScriptType: v4.00+\nPlayResX: 64\nPlayResY: 64\n[V4+ Styles]\nFormat: Name, Fontname, Fontsize, PrimaryColour, Alignment\nStyle: Default,Arial,12,&H00FFFFFF,2\n[Events]\nFormat: Layer, Start, End, Style, Text\nDialogue: 0,0:00:00.00,0:00:00.20,Default,Qualification\n",
    );
    run(
      [
        ...common,
        "-f",
        "lavfi",
        "-i",
        "color=s=64x64:r=10:d=0.1",
        "-f",
        "lavfi",
        "-i",
        "color=s=64x64:r=10:d=0.1",
        "-filter_complex",
        "[0:v][1:v]concat=n=2:v=1:a=0,subtitles=captions.ass[v]",
        "-map",
        "[v]",
        "-c:v",
        "libx264",
        "-f",
        "null",
        "-",
      ],
      scratch,
    );
    run(
      [
        ...common,
        "-f",
        "lavfi",
        "-i",
        "sine=duration=0.1",
        "-af",
        "astats",
        "-c:a",
        "pcm_s16le",
        "-f",
        "null",
        "-",
      ],
      scratch,
    );
  } finally {
    rmSync(scratch, { recursive: true, force: true });
  }
}
