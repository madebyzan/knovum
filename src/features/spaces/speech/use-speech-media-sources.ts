import { computed } from "vue";

import { useSpaceStore } from "../session/space-store";
import type { SpeechMediaSource } from "./contracts";

export function useSpeechMediaSources() {
  const store = useSpaceStore();
  const sources = computed(() =>
    store.nodes.flatMap((node) => {
      const data = node.data;
      if (data.kind !== "video" && data.kind !== "audio") return [];
      if (
        !data.assetPath ||
        !data.sourceRevision ||
        !data.mediaAnalysis ||
        !isCurrentSpeechMedia(
          data.sourceRevision,
          data.mediaAnalysis.sourceRevision,
          data.mediaAnalysis.durationMs,
        )
      )
        return [];
      const source: SpeechMediaSource = {
        assetPath: data.assetPath,
        sourceRevision: data.sourceRevision,
        durationMs: data.mediaAnalysis.durationMs,
      };
      const transcript =
        data.mediaKnowledge?.sourceRevision === data.sourceRevision
          ? data.mediaKnowledge.transcript
          : undefined;
      return [
        {
          id: node.id,
          name: data.title,
          kind: data.kind,
          source,
          ...(transcript ? { transcript } : {}),
        },
      ];
    }),
  );
  const videos = computed(() => sources.value.filter((source) => source.kind === "video"));
  const backgrounds = computed(() => sources.value.filter((source) => source.kind === "audio"));
  return { videos, backgrounds };
}

function isCurrentSpeechMedia(revision: string, analysisRevision: string, durationMs: number) {
  return (
    revision === analysisRevision &&
    /^sha256:[a-f0-9]{64}$/u.test(revision) &&
    durationMs > 0 &&
    durationMs <= 43_200_000
  );
}
