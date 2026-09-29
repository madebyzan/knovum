import type { Editor } from "@tiptap/vue-3";
import { onBeforeUnmount, onMounted, ref, type Ref } from "vue";
import { toast } from "vue-sonner";

import { isFileDropInside, listenForFileDrops } from "../../../foundation/file-drop/file-drop";
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
          active.value = hovering && isFileDropInside(target.value, position);
        },
        drop: async (payload) => {
          active.value = false;
          const current = editor.value;
          if (!current || !isFileDropInside(target.value, payload.position) || !payload.paths.length) return;
          const position = current.view.posAtCoords({
            left: payload.position.x,
            top: payload.position.y,
          })?.pos;
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
