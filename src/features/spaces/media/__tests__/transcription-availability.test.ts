import { defineStore, setActivePinia, createPinia } from "pinia";
import { beforeEach, expect, it, vi } from "vitest";
import { effectScope, ref, shallowRef } from "vue";

import type { NodeEnrichmentStatus } from "../../enrichment/use-node-enrichment-status";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { useMediaKnowledge } from "../use-media-knowledge";

const mocks = vi.hoisted(() => ({
  analyze: vi.fn<(nodeId: string) => Promise<string>>(),
  refresh: vi.fn<() => Promise<void>>(),
  error: vi.fn<(message: string) => void>(),
}));
vi.mock("../../session/space-store", () => ({ useSpaceStore: () => testStore() }));
vi.mock("../../canvas/use-node-operation-gate", () => ({
  useNodeOperationGate: () => ({ allows: () => allowed.value }),
}));
vi.mock("../../enrichment/use-node-enrichment-status", () => ({
  useNodeEnrichmentStatus: () => ({ status }),
}));
vi.mock("../../enrichment/enrichment-jobs-query", () => ({ refreshEnrichmentJobs: mocks.refresh }));
vi.mock("../../../../foundation/i18n/instance", () => ({ i18n: { global: { t: (key: string) => key } } }));
vi.mock("vue-sonner", () => ({ toast: { error: mocks.error } }));
vi.mock("vue-i18n", () => ({ useI18n: () => ({ t: (key: string) => key }) }));
const allowed = shallowRef(true);
const status = shallowRef<NodeEnrichmentStatus>({ pending: 0, running: 0, failed: 0, waiting: 0 });
const testStore = defineStore("transcription-availability", () => ({
  activeSpaceId: ref("space"),
  mediaAnalysisIds: ref<string[]>([]),
  mediaAnalysisPreviews: ref({}),
  mediaPlayback: ref(),
  analyzeMedia: mocks.analyze,
}));
const revision = `sha256:${"a".repeat(64)}`;
function mediaNode(): SpaceCanvasNode {
  return {
    id: "audio",
    type: "audio",
    initialPosition: { x: 0, y: 0 },
    data: {
      kind: "audio",
      title: "Recording",
      fileName: "recording.wav",
      assetPath: "/recording.wav",
      sourceRevision: revision,
      mediaAnalysis: { schemaVersion: 1, sourceRevision: revision, durationMs: 1000, waveform: [0.5] },
    },
  };
}
beforeEach(() => {
  setActivePinia(createPinia());
  vi.clearAllMocks();
  allowed.value = true;
  status.value = { pending: 0, running: 0, failed: 0, waiting: 0 };
  mocks.analyze.mockResolvedValue("job");
  mocks.refresh.mockResolvedValue(undefined);
});
it("starts an analyzed source and makes its Rust task visible immediately", async () => {
  const scope = effectScope();
  try {
    const operation = scope.run(() => useMediaKnowledge(ref(mediaNode())))!;
    expect(operation.canTranscribe.value).toBe(true);
    await operation.analyze();
    expect(mocks.analyze).toHaveBeenCalledWith("audio");
    expect(mocks.refresh).toHaveBeenCalledOnce();
  } finally {
    scope.stop();
  }
});
it("does not dispatch against pending media analysis or an editing gate", async () => {
  const scope = effectScope();
  const node = ref(mediaNode());
  try {
    const operation = scope.run(() => useMediaKnowledge(node))!;
    allowed.value = false;
    await operation.analyze();
    allowed.value = true;
    if (node.value.data.kind === "audio") node.value.data.mediaAnalysis = undefined;
    await operation.analyze();
    expect(mocks.analyze).not.toHaveBeenCalled();
  } finally {
    scope.stop();
  }
});
it.each(["pending", "running", "waiting"] as const)(
  "does not dispatch while a Rust %s job owns the node",
  async (field) => {
    const scope = effectScope();
    try {
      status.value = { pending: 0, running: 0, failed: 0, waiting: 0, [field]: 1 };
      const operation = scope.run(() => useMediaKnowledge(ref(mediaNode())))!;
      await operation.analyze();
      expect(operation.canTranscribe.value).toBe(false);
      expect(mocks.analyze).not.toHaveBeenCalled();
    } finally {
      scope.stop();
    }
  },
);
it("preserves a current transcript instead of attempting to overwrite its edits", async () => {
  const node = mediaNode();
  if (node.data.kind !== "audio") throw new Error("Audio fixture required");
  node.data.mediaKnowledge = {
    schemaVersion: 1,
    sourceRevision: revision,
    transcript: {
      id: "b".repeat(64),
      sourceRevision: revision,
      revision: 2,
      language: "en",
      durationMs: 1000,
      text: "Edited transcript",
      segments: [
        {
          id: "c".repeat(64),
          revision: 2,
          startMs: 0,
          endMs: 1000,
          startSample: 0,
          endSample: 16000,
          speakerId: null,
          text: "Edited transcript",
          units: [
            {
              id: "d".repeat(64),
              text: "Edited transcript",
              startCharacter: 0,
              endCharacter: 17,
              startMs: 0,
              endMs: 1000,
              score: null,
              unalignedReason: null,
              unitKind: "segment",
            },
          ],
        },
      ],
      subtitleCues: [],
      speakerTracks: [],
      provenance: {
        engineId: "whisper-cpp",
        modelSpecId: "whisper-small",
        modelRevision: "fixed",
        timing: "engine_words",
      },
    },
  };
  const scope = effectScope();
  try {
    const operation = scope.run(() => useMediaKnowledge(ref(node)))!;
    expect(operation.transcribed.value).toBe(true);
    await operation.analyze();
    expect(mocks.analyze).not.toHaveBeenCalled();
    expect(node.data.mediaKnowledge.transcript.text).toBe("Edited transcript");
  } finally {
    scope.stop();
  }
});
