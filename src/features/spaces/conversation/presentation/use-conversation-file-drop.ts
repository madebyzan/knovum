import { onBeforeUnmount, onMounted, shallowRef, type Ref } from "vue";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { isFileDropInside, listenForFileDrops } from "../../../../foundation/file-drop/events";
import { createAsyncDisposer } from "../../../../foundation/lifecycle/async-disposer";

interface ConversationFileDropOptions {
  /** Files are being chosen in the picker: a drop would race it, so it is ignored. */
  readonly blocked: () => boolean;
  readonly drop: (paths: string[]) => void;
}

/**
 * Files dropped from the system onto the composer. `hovering` is whether a drag is over it now; the
 * listener lives as long as the composer.
 */
export function useConversationFileDrop(
  zone: Readonly<Ref<HTMLElement | null>>,
  options: ConversationFileDropOptions,
) {
  const hovering = shallowRef(false);
  const listener = createAsyncDisposer();
  onMounted(() => {
    void listener
      .attach(
        listenForFileDrops({
          hover(active, position) {
            hovering.value = active && !options.blocked() && isFileDropInside(zone.value, position);
          },
          drop({ paths, position }) {
            hovering.value = false;
            if (paths.length && !options.blocked() && isFileDropInside(zone.value, position))
              options.drop(paths);
          },
        }),
      )
      .catch((error) => toast.error(getErrorMessage(error)));
  });
  onBeforeUnmount(listener.dispose);
  return { hovering };
}
