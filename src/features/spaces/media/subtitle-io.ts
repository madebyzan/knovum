import { invokeCommand } from "../../../foundation/ipc/invoke-command";

/** Reads an OS-dialog-authorized `.srt` or `.vtt` subtitle file through the spaces IPC boundary. */
export function readSubtitleFile(sourcePath: string) {
  return invokeCommand<string>("plugin:spaces|read_dubbing_subtitles", { sourcePath });
}
