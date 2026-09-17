import type { Editor } from "@tiptap/vue-3";
import { onBeforeUnmount, onMounted, ref, type Ref } from "vue";
import { toast } from "vue-sonner";

import { listenForFileDrops, type DragDropPosition } from "../../../foundation/file-drop/file-drop";
import { createAsyncDisposer } from "../../../foundation/lifecycle/async-disposer";
import { insertSpaceTiptapImagePaths } from "./tiptap-images";

export function useSpaceTiptapImageDrop(
  editor: Ref<Editor | undefined>,
  target: Ref<HTMLElement | undefined>,
  errorLabel: () => string,
) {
  const active = ref(false);
  const disposer = createAsyncDisposer();

  onMounted(async () => {
    await disposer.attach(
      listenForFileDrops({
        hover: (hovering, position) => {
          active.value = hovering && dropPoint(target.value, position) !== undefined;
        },
        drop: async (payload) => {
          active.value = false;
          const current = editor.value;
          const point = dropPoint(target.value, payload.position);
          if (!current || !point || !payload.paths.length) return;
          const position = current.view.posAtCoords({ left: point.x, top: point.y })?.pos;
          try {
            await insertSpaceTiptapImagePaths(current, payload.paths, position);
          } catch (error) {
            toast.error(errorLabel(), { description: String(error) });
          }
        },
      }),
    );
  });
  onBeforeUnmount(disposer.dispose);
  return active;
}

function dropPoint(element: HTMLElement | undefined, position: DragDropPosition | undefined) {
  if (!element || !position) return undefined;
  const rect = element.getBoundingClientRect();
  const inside =
    position.x >= rect.left &&
    position.x <= rect.right &&
    position.y >= rect.top &&
    position.y <= rect.bottom;
  return inside ? position : undefined;
}
