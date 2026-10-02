import { shallowRef, watch, type Ref } from "vue";

import { removeClip, type VideoEditProject } from "../../video-editing/project";

export function useVideoEditSelection(project: Ref<VideoEditProject | undefined>) {
  const selectedClipId = shallowRef<string>();
  const selectedTextId = shallowRef<string>();

  function selectClip(id: string) {
    selectedClipId.value = id;
    selectedTextId.value = undefined;
  }

  function selectText(id: string) {
    selectedTextId.value = id;
    selectedClipId.value = undefined;
  }

  function clear() {
    selectedClipId.value = undefined;
    selectedTextId.value = undefined;
  }

  function removeSelection() {
    const current = project.value;
    if (!current) return undefined;
    let next = selectedClipId.value ? removeClip(current, selectedClipId.value) : current;
    if (selectedTextId.value) {
      next = {
        ...next,
        textTrack: {
          items: next.textTrack.items.filter(({ id }) => id !== selectedTextId.value),
        },
      };
    }
    clear();
    return next;
  }

  watch(project, (current) => {
    if (!current) return;
    if (selectedClipId.value && !current.videoTrack.clips.some(({ id }) => id === selectedClipId.value))
      clear();
    if (selectedTextId.value && !current.textTrack.items.some(({ id }) => id === selectedTextId.value))
      clear();
  });

  return { selectedClipId, selectedTextId, selectClip, selectText, removeSelection, clear };
}
