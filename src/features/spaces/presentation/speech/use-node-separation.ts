import { computed, watch, type Ref } from "vue";

import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { useSpaceStore } from "../../session/space-store";
import { useSourceSeparation } from "../../speech/use-source-separation";
import { useSpeechMediaSources } from "../../speech/use-speech-media-sources";

export function useNodeSeparation(node: Readonly<Ref<SpaceCanvasNode | undefined>>) {
  const store = useSpaceStore();
  const { videos, backgrounds } = useSpeechMediaSources();
  const separation = useSourceSeparation();
  const source = computed(
    () => [...videos.value, ...backgrounds.value].find(({ id }) => id === node.value?.id)?.source,
  );
  const unavailableReason = computed(() => {
    if (!store.activeSpaceId || !source.value) return "spaces.speech.separation.sourceUnavailable";
    if (source.value.durationMs > 600_000) return "spaces.speech.separation.tooLong";
    return undefined;
  });
  watch(
    [
      () => store.activeSpaceId,
      () => node.value?.id,
      () => source.value?.assetPath,
      () => source.value?.sourceRevision,
      () => source.value?.durationMs,
    ],
    () => {
      if (store.activeSpaceId && node.value && source.value)
        void separation.restore(store.activeSpaceId, node.value.id, source.value);
      else separation.reset();
    },
    { immediate: true },
  );
  async function startSeparation() {
    if (!unavailableReason.value && store.activeSpaceId && node.value && source.value)
      await separation.start(store.activeSpaceId, node.value.id, source.value);
  }
  return { separation, unavailableReason, startSeparation };
}
