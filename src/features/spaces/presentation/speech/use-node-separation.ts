import { computed, watch, type Ref } from "vue";

import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { useSpaceStore } from "../../session/space-store";
import { useSourceSeparationCandidate } from "../../speech/use-source-separation-candidate";
import { useSpeechMediaSources } from "../../speech/use-speech-media-sources";

export function useNodeSeparation(node: Readonly<Ref<SpaceCanvasNode | undefined>>) {
  const store = useSpaceStore();
  const { videos, backgrounds } = useSpeechMediaSources();
  const separation = useSourceSeparationCandidate();
  const source = computed(
    () => [...videos.value, ...backgrounds.value].find(({ id }) => id === node.value?.id)?.source,
  );
  watch(
    [() => store.activeSpaceId, () => source.value?.assetPath, () => source.value?.sourceRevision],
    () => {
      if (store.activeSpaceId && source.value) void separation.restore(store.activeSpaceId, source.value);
      else separation.reset();
    },
    { immediate: true },
  );
  async function invoke() {
    if (separation.activeRunId.value) await separation.cancel();
    else if (store.activeSpaceId && source.value && source.value.durationMs <= 600_000)
      await separation.start(store.activeSpaceId, source.value);
  }
  return { separation, source, invoke };
}
