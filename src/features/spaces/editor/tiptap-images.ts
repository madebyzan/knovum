import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type { Editor } from "@tiptap/vue-3";
import { basename } from "pathe";

import { removeSpaceAssets } from "../assets/asset-operations";

const TIPTAP_IMAGE_EXTENSIONS = {
  "image/gif": "gif",
  "image/jpeg": "jpg",
  "image/png": "png",
  "image/webp": "webp",
} as const;

export const TIPTAP_IMAGE_MIME_TYPES = Object.keys(TIPTAP_IMAGE_EXTENSIONS);
const TIPTAP_IMAGE_DIALOG_EXTENSIONS = ["png", "jpg", "jpeg", "gif", "webp"];
const MAX_EDITOR_IMAGE_BYTES = 20 * 1024 * 1024;

type ImageContent = { type: "image"; attrs: { src: string; alt: string; assetPath: string } };
export type SpaceTiptapImageEditor = Pick<Editor, "chain" | "isDestroyed">;

export async function chooseSpaceTiptapImages(editor: SpaceTiptapImageEditor) {
  const selected = await open({
    multiple: true,
    directory: false,
    filters: [{ name: "Images", extensions: TIPTAP_IMAGE_DIALOG_EXTENSIONS }],
  });
  if (!selected || editor.isDestroyed) return false;
  const paths = Array.isArray(selected) ? selected : [selected];
  await insertSpaceTiptapImagePaths(editor, paths);
  return true;
}

export async function insertSpaceTiptapImagePaths(
  editor: SpaceTiptapImageEditor,
  paths: readonly string[],
  position?: number,
) {
  const content: ImageContent[] = [];
  const importedPaths: string[] = [];
  try {
    assertEditorAlive(editor);
    for (const path of paths) {
      // oxlint-disable-next-line eslint/no-await-in-loop -- Imports stay ordered for deterministic editor insertion and rollback.
      const imported = await invoke<string>("plugin:spaces|import_editor_image", { sourcePath: path });
      importedPaths.push(imported);
      assertEditorAlive(editor);
      content.push({
        type: "image",
        attrs: { src: convertFileSrc(imported), alt: imageAltName(path), assetPath: imported },
      });
    }
    assertEditorAlive(editor);
    insertImages(editor, content, position);
  } catch (error) {
    try {
      await removeSpaceAssets(importedPaths);
    } catch (cleanupError) {
      throw new AggregateError(
        [error, cleanupError],
        "Image import failed and staged images could not be removed",
        { cause: cleanupError },
      );
    }
    throw error;
  }
}

export async function insertSpaceTiptapImages(
  editor: SpaceTiptapImageEditor,
  files: File[],
  position?: number,
) {
  const content: ImageContent[] = [];
  const savedPaths: string[] = [];
  try {
    assertEditorAlive(editor);
    for (const file of files) {
      // oxlint-disable-next-line eslint/no-await-in-loop -- Saves stay ordered for deterministic editor insertion and rollback.
      const image = await saveSpaceTiptapImage(file);
      savedPaths.push(image.assetPath);
      assertEditorAlive(editor);
      content.push({ type: "image", attrs: { ...image, alt: file.name } });
    }
    assertEditorAlive(editor);
    insertImages(editor, content, position);
  } catch (error) {
    await removeSpaceAssets(savedPaths);
    throw error;
  }
}

async function saveSpaceTiptapImage(file: File) {
  if (!TIPTAP_IMAGE_EXTENSIONS[file.type as keyof typeof TIPTAP_IMAGE_EXTENSIONS]) {
    throw new Error(`Unsupported image type: ${file.type || "unknown"}`);
  }
  if (file.size <= 0 || file.size > MAX_EDITOR_IMAGE_BYTES) {
    throw new Error("Editor images must be between 1 byte and 20 MiB");
  }
  const assetPath = await invoke<string>("plugin:spaces|persist_editor_image", {
    bytes: Array.from(new Uint8Array(await file.arrayBuffer())),
    mimeType: file.type,
  });
  return { assetPath, src: convertFileSrc(assetPath) };
}

function insertImages(editor: SpaceTiptapImageEditor, content: ImageContent[], position?: number) {
  if (!content.length) return;
  const chain = editor.chain().focus();
  const inserted =
    position === undefined
      ? chain.insertContent(content).run()
      : chain.insertContentAt(position, content).run();
  if (!inserted) throw new Error("The editor rejected the image insertion");
}

function imageAltName(path: string) {
  return basename(path) || "image";
}

function assertEditorAlive(editor: SpaceTiptapImageEditor) {
  if (editor.isDestroyed) throw new Error("The editor was closed before image import completed");
}
