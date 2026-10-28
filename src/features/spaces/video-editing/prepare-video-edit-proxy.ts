import * as log from "@tauri-apps/plugin-log";
import { z } from "zod";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import type { PrepareVideoEditProxyRequest } from "../contracts/generated/PrepareVideoEditProxyRequest";
import type { VideoEditMediaSource } from "../contracts/generated/VideoEditMediaSource";

const proxyPathSchema = z.string().min(1);

/** A decodable stand-in for a source the WebView cannot decode; the path of its file. */
export async function prepareVideoEditProxy(source: VideoEditMediaSource, signal: AbortSignal) {
  signal.throwIfAborted();
  const id = crypto.randomUUID();
  const cancel = () => {
    void invokeCommand<void>("plugin:spaces|cancel_video_edit_proxy", { jobId: id }).catch((cause) =>
      log.error(`Failed to cancel video editing preparation: ${getErrorMessage(cause)}`),
    );
  };
  signal.addEventListener("abort", cancel, { once: true });
  try {
    const path = await invokeCommand<string>("plugin:spaces|prepare_video_edit_proxy", {
      request: { id, source } satisfies PrepareVideoEditProxyRequest,
    });
    signal.throwIfAborted();
    return proxyPathSchema.parse(path);
  } finally {
    signal.removeEventListener("abort", cancel);
  }
}
