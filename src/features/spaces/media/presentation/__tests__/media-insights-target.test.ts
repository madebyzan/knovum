import { QueryClient, VueQueryPlugin } from "@tanstack/vue-query";
import { flushPromises, mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { beforeEach, expect, it, onTestFinished, vi } from "vitest";
import { ref } from "vue";

import { useAiSettingsStore } from "../../../../../foundation/ai/ai-settings-store";
import {
  DEFAULT_AI_MODEL_SETTINGS,
  type AiModelSettings,
} from "../../../../../foundation/ai/schema/model-settings";
import { i18n } from "../../../../../foundation/i18n/instance";
import type { TaskTarget } from "../../../../../shared/contracts/generated/TaskTarget";
import UiSelect from "../../../../../shared/ui/UiSelect.vue";
import { transcriptFixture } from "../../../__tests__/support/media-transcript-fixture";
import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import type { SpaceAudioPayload } from "../../../model/nodes/node-payload";
import type { SpaceMediaAnalysisLabels } from "../../actions";
import SpaceMediaKnowledge from "../SpaceMediaKnowledge.vue";

const mocks = vi.hoisted(() => ({
  generate: vi.fn<(nodeId: string, labels: SpaceMediaAnalysisLabels, target: TaskTarget) => Promise<void>>(),
  loadSettings: vi.fn<() => Promise<AiModelSettings>>(),
  invoke: vi.fn<(command: string, args?: unknown) => Promise<unknown>>(),
}));

vi.mock("../../../../../foundation/ai/integration-settings", () => ({
  loadAiModelSettings: mocks.loadSettings,
  saveAiModelSettings: vi.fn<(value: AiModelSettings) => Promise<void>>(),
  resetAiModelSettings: vi.fn<() => Promise<void>>(),
}));
vi.mock("../../../../../foundation/ipc/invoke-command", () => ({ invokeCommand: mocks.invoke }));
vi.mock("../../../../../foundation/ai/use-execution-catalog-query", () => ({
  useExecutionCatalogQuery: () => ({ data: ref({ agents: [] }) }),
}));
vi.mock("../../../session/space-store", () => ({
  useSpaceStore: () => ({
    activeSpaceId: ref("space-1"),
    mediaAnalysisIds: ref<string[]>([]),
    mediaAnalysisPreviews: ref({}),
    mediaPlayback: ref(undefined),
    generateMediaInsights: mocks.generate,
    analyzeMedia: vi.fn<() => Promise<string>>(),
    saveNodeContent: vi.fn<() => Promise<void>>(),
    requestMediaSeek: vi.fn<() => void>(),
  }),
}));
vi.mock("../../../canvas/use-node-operation-gate", () => ({
  useNodeOperationGate: () => ({ allows: () => true }),
}));
vi.mock("../../../canvas/editor-session", () => ({
  getCanvasEditorSession: () => ({ reveal: vi.fn<() => void>() }),
}));

const TARGET_A: TaskTarget = { type: "model", connectionId: "connection-1", modelId: "model-a" };
const TARGET_B: TaskTarget = { type: "model", connectionId: "connection-1", modelId: "model-b" };
const data: SpaceAudioPayload = {
  kind: "audio",
  title: "Interview",
  assetPath: "/interview.mp3",
  fileName: "interview.mp3",
  sourceRevision: "sha256:source",
  knowledgeRevision: "knowledge-1",
  mediaKnowledge: {
    transcript: transcriptFixture("sha256:source", [{ startMs: 0, endMs: 3_000, text: "hello" }]),
    sourceRevision: "sha256:source",
    schemaVersion: 1,
  },
};
const node: SpaceCanvasNode = { id: "media-1", type: "audio", initialPosition: { x: 0, y: 0 }, data };
const insightsNode: SpaceCanvasNode = {
  ...node,
  data: {
    ...data,
    mediaKnowledge: {
      ...data.mediaKnowledge!,
      insights: {
        summary: "Summary",
        chapters: [{ startMs: 0, endMs: 3_000, title: "Chapter", summary: "Chapter summary" }],
        promptVersion: 1,
        generatedAt: 1,
        executionSource: { type: "model", provider: "provider", model: "model" },
      },
    },
  },
};

function settingsWith(modelIds: string[]) {
  const settings = structuredClone(DEFAULT_AI_MODEL_SETTINGS);
  settings.connections = [
    {
      id: "connection-1",
      name: "Provider",
      provider: "openai",
      baseUrl: "https://api.example.com/v1/",
      models: modelIds.map((modelId) => ({
        id: modelId,
        name: modelId,
        model: modelId,
        capabilities: ["text-generation"],
        builtinAgent: false,
      })),
    },
  ];
  settings.defaults.knowledge = { type: "model", connectionId: "connection-1", modelId: modelIds[0]! };
  return settings;
}

function render(propsNode: SpaceCanvasNode) {
  const wrapper = mount(SpaceMediaKnowledge, {
    props: { node: propsNode },
    global: {
      plugins: [
        i18n,
        [VueQueryPlugin, { queryClient: new QueryClient({ defaultOptions: { queries: { retry: false } } }) }],
      ],
      stubs: {
        SpaceNodeEnrichmentStatus: true,
        SpaceMediaLiveTranscript: true,
        SpaceMediaSummaryEditor: true,
        SpaceMediaChapterEditor: true,
        SpaceTranscriptNavigator: true,
        SpaceDiarizationControl: true,
      },
    },
  });
  onTestFinished(() => wrapper.unmount());
  return wrapper;
}

beforeEach(() => {
  setActivePinia(createPinia());
  vi.clearAllMocks();
  mocks.loadSettings.mockResolvedValue(settingsWith(["model-a"]));
  mocks.invoke.mockResolvedValue([]);
  mocks.generate.mockResolvedValue(undefined);
});

it("hides the single configured target but still generates with the synchronized default", async () => {
  const wrapper = render(node);
  await flushPromises();

  expect(wrapper.findComponent(UiSelect).exists()).toBe(false);
  await wrapper.get("button").trigger("click");
  await flushPromises();

  expect(mocks.generate).toHaveBeenCalledWith("media-1", expect.any(Object), TARGET_A);
});

it("passes an inline model choice through summary regeneration without changing Settings", async () => {
  mocks.loadSettings.mockResolvedValue(settingsWith(["model-a", "model-b"]));
  const wrapper = render(insightsNode);
  await flushPromises();

  const editor = wrapper.findComponent({ name: "SpaceMediaSummaryEditor" });
  expect(editor.props("regenerateOptions")).toHaveLength(2);
  editor.vm.$emit("regenerate", JSON.stringify(TARGET_B));
  await flushPromises();

  expect(mocks.generate).toHaveBeenCalledWith("media-1", expect.any(Object), TARGET_B);
  expect(useAiSettingsStore().settings.defaults.knowledge).toEqual(TARGET_A);
});
