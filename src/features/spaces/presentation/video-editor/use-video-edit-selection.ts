import { shallowRef, type Ref } from "vue";

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

  function reset() {
    selectedClipId.value = project.value?.videoTrack.clips[0]?.id;
    selectedTextId.value = undefined;
  }

  function removeSelection() {
    if (!project.value) return;
    if (selectedClipId.value) project.value = removeClip(project.value, selectedClipId.value);
    if (selectedTextId.value) {
      project.value = {
        ...project.value,
        textTrack: {
          items: project.value.textTrack.items.filter(({ id }) => id !== selectedTextId.value),
        },
      };
    }
    reset();
  }

  return { selectedClipId, selectedTextId, selectClip, selectText, removeSelection, reset };
}
