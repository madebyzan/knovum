import { Channel } from "@tauri-apps/api/core";

import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import type { SpeechSignalReport } from "../contracts/generated/SpeechSignalReport";

export function inspectSpeechSignal(
  request: { projectId: string; revisionId: string; jobId: string },
  progress: (milliseconds: number) => void,
) {
  return invokeCommand<SpeechSignalReport>("plugin:spaces|inspect_speech_audio", {
    ...request,
    onProgress: new Channel<number>(progress),
  });
}
