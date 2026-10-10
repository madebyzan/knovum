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
  async function toggleSeparation() {
    if (separation.activeRunId.value) await separation.cancel();
    else if (store.activeSpaceId && node.value && source.value && source.value.durationMs <= 600_000)
      await separation.start(store.activeSpaceId, node.value.id, source.value);
  }
  return { separation, source, toggleSeparation };
}
