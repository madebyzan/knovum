import { QueryClient, VueQueryPlugin } from "@tanstack/vue-query";
import { flushPromises, mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { beforeEach, expect, it, onTestFinished, vi } from "vitest";
import { ref, shallowRef, type Ref } from "vue";

import {
  DEFAULT_AI_MODEL_SETTINGS,
  type AiModelSettings,
} from "../../../../../foundation/ai/schema/model-settings";
import { i18n } from "../../../../../foundation/i18n/instance";
import type { EnrichmentJobState } from "../../../contracts/generated/EnrichmentJobState";
import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import SpaceVisualAnalysis from "../SpaceVisualAnalysis.vue";

const mocks = vi.hoisted(() => ({
  invoke: vi.fn<(command: string, args?: unknown) => Promise<unknown>>(),
  loadSettings: vi.fn<() => Promise<AiModelSettings>>(),
  refresh: vi.fn<() => Promise<void>>(),
  jobs: null as null | Ref<EnrichmentJobState[]>,
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
vi.mock("../../../enrichment/use-node-enrichment-status", () => ({
  useNodeEnrichmentStatus: () => ({ jobs: mocks.jobs, loading: ref(false), refresh: mocks.refresh }),
}));
vi.mock("../../../enrichment/use-recognized-text", () => ({ useRecognizedText: () => () => undefined }));
vi.mock("../../../session/space-store", () => ({ useSpaceStore: () => ({ activeSpaceId: ref("space-1") }) }));

const node: SpaceCanvasNode = {
  id: "image-1",
  type: "image",
  initialPosition: { x: 0, y: 0 },
  data: {
    kind: "image",
    title: "Cover",
    assetPath: "/cover.png",
    fileName: "cover.png",
    sourceRevision: "sha256:source",
    intrinsicWidth: 800,
    intrinsicHeight: 600,
  },
};

function job(overrides: Partial<EnrichmentJobState>): EnrichmentJobState {
  return {
    id: "job-1",
    spaceId: "space-1",
    nodeId: node.id,
    jobType: "analyze_visual",
    status: "awaiting_configuration",
    sourceRevision: "sha256:source",
    attempts: 0,
    executionTarget: null,
    error: null,
    createdAt: 1,
    updatedAt: 1,
    ...overrides,
  };
}

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
        capabilities: ["vision-understanding"],
        builtinAgent: false,
      })),
    },
  ];
  settings.defaults.visualUnderstanding = {
    type: "model",
    connectionId: "connection-1",
    modelId: modelIds[0]!,
  };
  return settings;
}

function render() {
  const wrapper = mount(SpaceVisualAnalysis, {
    attachTo: document.body,
    props: { node },
    global: {
      plugins: [
        i18n,
        [VueQueryPlugin, { queryClient: new QueryClient({ defaultOptions: { queries: { retry: false } } }) }],
      ],
      stubs: { SpaceRecognizedText: true },
    },
  });
  onTestFinished(() => wrapper.unmount());
  return wrapper;
}

beforeEach(() => {
  setActivePinia(createPinia());
  vi.clearAllMocks();
  mocks.jobs = shallowRef([]);
  mocks.loadSettings.mockResolvedValue(settingsWith(["model-a", "model-b"]));
  mocks.invoke.mockResolvedValue("job-1");
  mocks.refresh.mockResolvedValue(undefined);
});

it("opens a one-click popover when multiple targets exist and sends the chosen target", async () => {
  const wrapper = render();
  await flushPromises();

  await wrapper.get('[aria-label="Reanalyze visual content"]').trigger("click");
  await flushPromises();
  expect(mocks.invoke).not.toHaveBeenCalled();

  const option = Array.from(document.body.querySelectorAll("button")).find((button) =>
    button.textContent?.includes("model-b"),
  );
  expect(option).toBeDefined();
  option!.click();
  await flushPromises();

  expect(mocks.invoke).toHaveBeenCalledWith("plugin:spaces|request_visual_analysis", {
    spaceId: "space-1",
    nodeId: "image-1",
    target: { type: "model", connectionId: "connection-1", modelId: "model-b", temperature: 0.2 },
  });
  expect(mocks.refresh).toHaveBeenCalledOnce();
});

it("runs directly with the Settings default when only one target is configured", async () => {
  mocks.loadSettings.mockResolvedValue(settingsWith(["model-a"]));
  mocks.jobs!.value = [job({ executionTarget: null })];
  const wrapper = render();
  await flushPromises();

  await wrapper.get('[aria-label="Reanalyze visual content"]').trigger("click");
  await flushPromises();

  expect(mocks.invoke).toHaveBeenCalledWith("plugin:spaces|request_visual_analysis", {
    spaceId: "space-1",
    nodeId: "image-1",
    target: { type: "model", connectionId: "connection-1", modelId: "model-a", temperature: 0.2 },
  });
});
