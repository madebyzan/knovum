import { open } from "@tauri-apps/plugin-dialog";

import { inspectAssistantUpload } from "./conversation-ipc";

export function inspectAssistantUploads(paths: readonly string[]) {
  return Promise.all(paths.map((path) => inspectAssistantUpload(path)));
}

export async function chooseAssistantUploads() {
  const selected = await open({
    directory: false,
    multiple: true,
    filters: [
      {
        name: "Knovum",
        extensions: [
          "png",
          "jpg",
          "jpeg",
          "webp",
          "txt",
          "md",
          "markdown",
          "csv",
          "json",
          "log",
          "pdf",
          "doc",
          "docx",
          "xls",
          "xlsx",
          "ppt",
          "pptx",
        ],
      },
    ],
  });
  if (!selected) return [];
  const paths = Array.isArray(selected) ? selected : [selected];
  return inspectAssistantUploads(paths);
}
