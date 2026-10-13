import { computed, watch, type Ref } from "vue";

import { removeClip } from "../edits/arrange";
import { removeAudioClip } from "../edits/audio";
import { removeTextOverlay } from "../edits/text";
import { removeTransition } from "../edits/transition";
import type { VideoEditProject } from "../project";
import {
  retainSelection,
  selectedAudio,
  selectedClip,
  selectedText,
  selectedTransition,
  selectionSpan,
  type VideoSelection,
} from "../selection";

interface SelectionOptions {
  project: Ref<VideoEditProject | undefined>;
  selection: Ref<VideoSelection | undefined>;
  seek: (timeMs: number) => void;
  pause: () => void;
  playheadMs: () => number;
}

/** Selection state plus the navigation and deletion rules that belong to it. */
export function useVideoSelection(options: SelectionOptions) {
  const clip = computed(() =>
    options.project.value ? selectedClip(options.project.value, options.selection.value) : undefined,
  );
  const text = computed(() =>
    options.project.value ? selectedText(options.project.value, options.selection.value) : undefined,
  );
  const audio = computed(() =>
    options.project.value ? selectedAudio(options.project.value, options.selection.value) : undefined,
  );
  const transition = computed(() =>
    options.project.value ? selectedTransition(options.project.value, options.selection.value) : undefined,
  );

  watch(options.project, (project) => {
    if (project) options.selection.value = retainSelection(project, options.selection.value);
  });

  /** Selecting reveals its subject: the playhead moves inside it when it sits elsewhere. */
  function select(selection: VideoSelection, options_: { reveal?: boolean } = { reveal: true }) {
    options.selection.value = selection;
    if (!options_.reveal) return;
    const span = options.project.value && selectionSpan(options.project.value, selection);
    if (!span) return;
    const at = options.playheadMs();
    if (at < span.startMs || at >= span.endMs) {
      options.pause();
      options.seek(span.startMs);
    }
  }

  const canDelete = computed(() => {
    if (!options.selection.value || !options.project.value) return false;
    if (options.selection.value.kind !== "clip") return true;
    return options.project.value.videoTrack.clips.length > 1;
  });

  function removeSelected(): VideoEditProject | undefined {
    const project = options.project.value;
    const selection = options.selection.value;
    if (!project || !selection || !canDelete.value) return undefined;
    const removals = {
      clip: () => removeClip(project, selection.id),
      text: () => removeTextOverlay(project, selection.id),
      audio: () => removeAudioClip(project, selection.id),
      transition: () => removeTransition(project, selection.id),
    };
    const next = removals[selection.kind]();
    options.selection.value = undefined;
    return next;
  }

  return {
    selection: options.selection,
    clip,
    text,
    audio,
    transition,
    canDelete,
    select,
    clear: () => (options.selection.value = undefined),
    removeSelected,
  };
}

export type VideoSelectionController = ReturnType<typeof useVideoSelection>;
