import { computed, watch, type Ref } from "vue";

import { admitAiRequirements } from "../../../../foundation/ai/ai-readiness-admission";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { useSpaceStore } from "../../session/space-store";
import { useSourceSeparation } from "../use-source-separation";
import { useSpeechMediaSources } from "../use-speech-media-sources";

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
    if (unavailableReason.value || !store.activeSpaceId || !node.value || !source.value) return;
    const [spaceId, nodeId, media] = [store.activeSpaceId, node.value.id, source.value];
    if (await admitAiRequirements([{ kind: "localCapability", capability: "speechSeparation" }]))
      await separation.start(spaceId, nodeId, media);
  }
  return { separation, unavailableReason, startSeparation };
}
