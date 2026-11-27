import type { Editor } from "@tiptap/vue-3";
import { onBeforeUnmount, onMounted, shallowRef, type Ref, type ShallowRef } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { isFileDropInside, listenForFileDrops } from "../../../foundation/file-drop/events";
import { createAsyncDisposer } from "../../../foundation/lifecycle/async-disposer";
import { measureTopLevelBlocks, resolveBlockDropSlot } from "./tiptap-block-gutter";
import { insertSpaceTiptapImagePaths } from "./tiptap-images";

export interface ImageDropIndicator {
  readonly lineTop: number;
  readonly insertPos: number;
}

function resolveBlockInsertPos(editor: Editor, targetIndex: number): number {
  const { doc } = editor.state;
  let pos = 0;
  const clamped = Math.max(0, Math.min(targetIndex, doc.childCount));
  for (let i = 0; i < clamped; i += 1) pos += doc.child(i).nodeSize;
  return pos;
}

function computeDropIndicator(
  editor: Editor | undefined,
  proseHost: HTMLElement | null,
  clientY: number,
): ImageDropIndicator | null {
  if (!editor || editor.isDestroyed || !proseHost) return null;
  const yInHost = clientY - proseHost.getBoundingClientRect().top;
  const metrics = measureTopLevelBlocks(editor, proseHost);
  const slot = resolveBlockDropSlot(metrics, yInHost);
  if (!slot) return null;
  return { lineTop: slot.lineTop, insertPos: resolveBlockInsertPos(editor, slot.targetIndex) };
}

export function useSpaceTiptapImageDrop(
  editor: Ref<Editor | undefined>,
  target: Readonly<ShallowRef<HTMLElement | null>>,
  proseHost: Readonly<ShallowRef<HTMLElement | null>>,
  visible: Ref<boolean>,
) {
  const { t } = useI18n();
  const active = shallowRef(false);
  const dropIndicator = shallowRef<ImageDropIndicator | null>(null);
  const disposer = createAsyncDisposer();

  onMounted(async () => {
    await disposer.attach(
      listenForFileDrops({
        hover: (hovering, position) => {
          const inside = visible.value && hovering && isFileDropInside(target.value, position);
          active.value = inside;
          dropIndicator.value =
            inside && position ? computeDropIndicator(editor.value, proseHost.value, position.y) : null;
        },
        drop: async (payload) => {
          const indicator = dropIndicator.value;
          active.value = false;
          dropIndicator.value = null;
          const current = editor.value;
          if (
            !visible.value ||
            !current ||
            !isFileDropInside(target.value, payload.position) ||
            !payload.paths.length
          )
            return;
          const computed = indicator ?? computeDropIndicator(current, proseHost.value, payload.position.y);
          const position =
            computed?.insertPos ??
            current.view.posAtCoords({ left: payload.position.x, top: payload.position.y })?.pos;
          try {
            await insertSpaceTiptapImagePaths(current, payload.paths, position);
          } catch (error) {
            toast.error(t("spaces.richtext.image.uploadFailed"), { description: getErrorMessage(error) });
          }
        },
      }),
    );
  });
  onBeforeUnmount(disposer.dispose);
  return { active, dropIndicator };
}
