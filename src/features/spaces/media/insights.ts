import { Channel } from "@tauri-apps/api/core";

import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import type { MediaTranscriptionProgress } from "../contracts/generated/MediaTranscriptionProgress";
import { validateSpaceNodePayload } from "../model/nodes/node-validation";

export async function generateMediaInsights(
  input: { spaceId: string; nodeId: string; jobId: string; expectedRevision: string },
  progress: (value: MediaTranscriptionProgress) => void,
) {
  const onProgress = new Channel<MediaTranscriptionProgress>();
  // oxlint-disable-next-line unicorn/prefer-add-event-listener -- Tauri Channels use onmessage.
  onProgress.onmessage = progress;
  return validateSpaceNodePayload(
    await invokeCommand<unknown>("plugin:spaces|generate_media_insights", { ...input, onProgress }),
  );
}
