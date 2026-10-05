import { open } from "@tauri-apps/plugin-dialog";

import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import type { AudiobookSource } from "../contracts/generated/AudiobookSource";
import { audiobookDocumentScript } from "./audiobook-document-script";
import { audiobookScript, type ImportedAudiobookScript } from "./audiobook-script";

export async function chooseAudiobookScript(
  role: string,
  jobId: string,
  isCurrent: () => boolean,
): Promise<ImportedAudiobookScript | undefined> {
  const path = await open({
    multiple: false,
    directory: false,
    filters: [
      { name: "Audiobook manuscript", extensions: ["json", "txt", "md", "doc", "docx", "pdf", "epub"] },
    ],
  });
  if (typeof path !== "string" || !isCurrent()) return undefined;
  const source = await invokeCommand<AudiobookSource>("plugin:spaces|read_audiobook_source", {
    jobId,
    sourcePath: path,
  });
  if (path.toLowerCase().endsWith(".json")) {
    if (source.kind !== "text") throw new Error("Structured audiobook source must be text");
    return audiobookScript.parse(JSON.parse(source.text));
  }
  const title =
    path
      .split(/[\\/]/u)
      .at(-1)
      ?.replace(/\.[^.]+$/u, "") ?? "Audiobook";
  return audiobookScript.parse(audiobookDocumentScript(source, title, role));
}
