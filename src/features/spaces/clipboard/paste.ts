import { info as logInfo, warn as logWarn } from "@tauri-apps/plugin-log";
import { hasFiles, readFiles } from "tauri-plugin-clipboard-api";

import { invokeCommand as invoke } from "../../../foundation/ipc/invoke-command";
import type { ImportedSpaceAsset } from "../assets/imported-asset";
import { importSpaceClipboardImage } from "./image-import";

interface SpaceClipboardPasteHandlers {
  imageName: string;
  pasteNodes(html: string | null): Promise<boolean>;
  importFiles(paths: readonly string[]): Promise<void>;
  addImage(image: ImportedSpaceAsset): Promise<void>;
  insertText(text: string): Promise<void>;
}

export async function pasteSpaceClipboard(handlers: SpaceClipboardPasteHandlers) {
  let stage = "read-content";
  try {
    await logInfo("[spaces.paste] reading clipboard content");
    const content = await invoke<{ html: string | null; text: string | null }>(
      "plugin:spaces|read_space_clipboard_content",
    );
    await logInfo(
      `[spaces.paste] content read; htmlLength=${content.html?.length ?? 0}; textLength=${content.text?.length ?? 0}`,
    );
    stage = "paste-nodes";
    if (await handlers.pasteNodes(content.html)) {
      await logInfo("[spaces.paste] imported canvas nodes");
      return true;
    }
    stage = "read-files";
    if (
      await pasteClipboardFiles(handlers, (current) => {
        stage = current;
      })
    )
      return true;
    return await pasteImageOrText(content.text, handlers, (current) => {
      stage = current;
    });
  } catch (error) {
    await logWarn(
      `[spaces.paste] failed at ${stage}; errorType=${error instanceof Error ? error.name : typeof error}`,
    );
    throw error;
  }
}

async function pasteClipboardFiles(handlers: SpaceClipboardPasteHandlers, setStage: (stage: string) => void) {
  if (!(await hasFiles())) return false;
  const paths = await readFiles();
  await logInfo(`[spaces.paste] clipboard files detected; count=${paths.length}`);
  if (!paths.length) return false;
  setStage("import-files");
  await handlers.importFiles(paths);
  await logInfo("[spaces.paste] imported clipboard files");
  return true;
}

async function pasteImageOrText(
  text: string | null,
  handlers: SpaceClipboardPasteHandlers,
  setStage: (stage: string) => void,
) {
  setStage("read-image");
  const image = await importSpaceClipboardImage(handlers.imageName);
  if (image) {
    setStage("add-image");
    await handlers.addImage(image);
    await logInfo("[spaces.paste] imported clipboard image");
    return true;
  }
  if (!text?.trim()) {
    await logInfo("[spaces.paste] no supported clipboard content");
    return false;
  }
  setStage("insert-text");
  await logInfo(`[spaces.paste] inserting clipboard text; length=${text.length}`);
  await handlers.insertText(text);
  await logInfo("[spaces.paste] clipboard text inserted");
  return true;
}
