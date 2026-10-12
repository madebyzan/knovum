import { shallowRef, watch, type Ref } from "vue";

import { removeClip } from "../clip-edits";
import { keepReferencedVideoAssets } from "../material";
import { type VideoEditProject } from "../project";

export function useVideoEditSelection(
  project: Ref<VideoEditProject | undefined>,
  selectedClipId = shallowRef<string>(),
  selectedTextId = shallowRef<string>(),
  selectedAudioId = shallowRef<string>(),
) {
  function selectClip(clipId: string) {
    selectedClipId.value = clipId;
    selectedTextId.value = undefined;
    selectedAudioId.value = undefined;
  }

  function selectText(textId: string) {
    selectedTextId.value = textId;
    selectedClipId.value = undefined;
    selectedAudioId.value = undefined;
  }

  function selectAudio(id: string) {
    clear();
    selectedAudioId.value = id;
  }

  function clear() {
    selectedClipId.value = undefined;
    selectedTextId.value = undefined;
    selectedAudioId.value = undefined;
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
    if (selectedAudioId.value)
      next = {
        ...next,
        audioTrack: { clips: next.audioTrack.clips.filter(({ id }) => id !== selectedAudioId.value) },
      };
    clear();
    return keepReferencedVideoAssets(next);
  }

  watch(project, (current) => {
    if (!current) return;
    if (selectedClipId.value && !current.videoTrack.clips.some(({ id }) => id === selectedClipId.value))
      clear();
    if (selectedAudioId.value && !current.audioTrack.clips.some(({ id }) => id === selectedAudioId.value))
      clear();
    if (selectedTextId.value && !current.textTrack.items.some(({ id }) => id === selectedTextId.value))
      clear();
  });

  return {
    selectedClipId,
    selectedTextId,
    selectedAudioId,
    selectClip,
    selectText,
    selectAudio,
    removeSelection,
    clear,
  };
}
