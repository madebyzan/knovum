import { readFile } from "@tauri-apps/plugin-fs";
import { shallowRef, watch, type Ref } from "vue";

import type { ConversationAttachment } from "../contracts/generated/ConversationAttachment";

type FileAttachment = Extract<ConversationAttachment, { kind: "file" }>;

export function isConversationImagePath(path: string) {
  return /\.(png|jpe?g|webp)$/i.test(path);
}
function previewKey({ path, sourceRevision }: FileAttachment) {
  return `${path}\0${sourceRevision}`;
}

async function createPreview(path: string) {
  const bytes = await readFile(path);
  const type = /\.png$/i.test(path) ? "image/png" : /\.webp$/i.test(path) ? "image/webp" : "image/jpeg";
  const source = URL.createObjectURL(new Blob([bytes], { type }));
  try {
    const image = new Image();
    image.src = source;
    await image.decode();
    const scale = Math.min(1, 144 / Math.max(image.naturalWidth, image.naturalHeight));
    const canvas = document.createElement("canvas");
    canvas.width = Math.max(1, Math.round(image.naturalWidth * scale));
    canvas.height = Math.max(1, Math.round(image.naturalHeight * scale));
    const context = canvas.getContext("2d");
    if (!context) throw new Error("Unable to render attachment preview");
    context.drawImage(image, 0, 0, canvas.width, canvas.height);
    return canvas.toDataURL("image/png");
  } finally {
    URL.revokeObjectURL(source);
  }
}

export function useConversationImagePreviews(attachments: Readonly<Ref<ConversationAttachment[]>>) {
  const previewUrlByAttachmentKey = shallowRef(new Map<string, string>());

  watch(
    attachments,
    (items, _, onCleanup) => {
      let active = true;
      onCleanup(() => (active = false));
      const images = items.filter(
        (item): item is FileAttachment => item.kind === "file" && isConversationImagePath(item.path),
      );
      const keys = new Set(images.map(previewKey));
      previewUrlByAttachmentKey.value = new Map(
        [...previewUrlByAttachmentKey.value].filter(([key]) => keys.has(key)),
      );
      void Promise.all(
        images.map(async (image) => {
          const key = previewKey(image);
          if (!active || previewUrlByAttachmentKey.value.has(key)) return;
          try {
            const source = await createPreview(image.path);
            if (active)
              previewUrlByAttachmentKey.value = new Map(previewUrlByAttachmentKey.value).set(key, source);
          } catch {
            // The attachment remains removable if its preview cannot be read.
          }
        }),
      );
    },
    { immediate: true },
  );

  return (file: FileAttachment) => previewUrlByAttachmentKey.value.get(previewKey(file)) ?? "";
}
