import { open } from "@tauri-apps/plugin-dialog";

import { inspectConversationUpload } from "./conversation-ipc";

export function inspectConversationUploads(paths: readonly string[]) {
  return Promise.all(paths.map((path) => inspectConversationUpload(path)));
}

export async function chooseConversationUploads() {
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
  return inspectConversationUploads(paths);
}
