import { useResizeObserver } from "@vueuse/core";
import { computed, onScopeDispose, shallowRef, watch, type ShallowRef } from "vue";

import { createStageEditing, type StageTarget } from "../stage/stage-editing";
import type { VideoEditor } from "./use-video-editor";

const MARGIN_PX = 24;
const ZOOM_RANGE = { min: 0.05, max: 4 };

/**
 * Stage viewport: how much of the output fits on screen, and the bridge between the Konva scene and
 * the editor's selection. The scene is created once per container and reused for every frame.
 */
interface VideoStageElements {
  viewport: Readonly<ShallowRef<HTMLElement | null>>;
  container: Readonly<ShallowRef<HTMLDivElement | null>>;
}

export function useVideoStage(editor: VideoEditor, { viewport, container }: VideoStageElements) {
  const { player, project, visibleProject, selection, commit } = editor;
  const bounds = shallowRef({ width: 0, height: 0 });
  const zoom = shallowRef(1);
  const fitted = shallowRef(true);
  const editing = createStageEditing({
    project: () => project.value,
    commit,
    onSelect: (target: StageTarget | undefined) => {
      if (target) selection.select({ kind: target.kind, id: target.id }, { reveal: false });
      else selection.clear();
    },
  });

  const output = computed(() => visibleProject.value?.output ?? project.value?.output);
  const fitScale = computed(() => {
    const frame = output.value;
    if (!frame || !bounds.value.width || !bounds.value.height) return 0;
    return Math.min(
      (bounds.value.width - MARGIN_PX) / frame.width,
      (bounds.value.height - MARGIN_PX) / frame.height,
    );
  });
  const scale = computed(() => Math.max(ZOOM_RANGE.min / 2, fitted.value ? fitScale.value : zoom.value));

  useResizeObserver(viewport, ([entry]) => {
    if (entry) bounds.value = { width: entry.contentRect.width, height: entry.contentRect.height };
  });
  watch(container, (element) => {
    if (element) editing.attach(player.attach(element));
  });
  watch(
    scale,
    (value) => {
      player.resize(value);
      editing.rescale(value);
    },
    { immediate: true },
  );
  function updateHighlight() {
    const clipId = selection.clip.value?.id;
    const textId = selection.text.value?.id;
    if (clipId) editing.highlight({ kind: "clip", id: clipId });
    else if (textId) editing.highlight({ kind: "text", id: textId });
    else editing.highlight(undefined);
  }

  watch([() => selection.clip.value?.id, () => selection.text.value?.id], updateHighlight, {
    immediate: true,
  });
  onScopeDispose(
    player.subscribe((_time, playing) => {
      if (playing) editing.highlight(undefined);
      else updateHighlight();
    }),
  );
  onScopeDispose(() => editing.dispose());

  return {
    output,
    scale,
    fitted,
    frameStyle: computed(() => ({
      width: `${(output.value?.width ?? 0) * scale.value}px`,
      height: `${(output.value?.height ?? 0) * scale.value}px`,
    })),
    zoomBy(factor: number) {
      const current = fitted.value ? fitScale.value : zoom.value;
      zoom.value = Math.min(ZOOM_RANGE.max, Math.max(ZOOM_RANGE.min, current * factor));
      fitted.value = false;
    },
    fit() {
      fitted.value = true;
    },
  };
}
