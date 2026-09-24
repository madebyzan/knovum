import { platform } from "@tauri-apps/plugin-os";

export function systemSansFamily(): string {
  switch (platform()) {
    case "macos":
      return "PingFang SC";
    case "windows":
      return "Microsoft YaHei UI";
    default:
      throw new Error("System font selection is unsupported on this platform");
  }
}
