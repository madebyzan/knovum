import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { ref } from "vue";

import { useAiSettingsStore } from "../../../../foundation/ai/ai-settings-store";
import {
  DEFAULT_AI_MODEL_SETTINGS,
  type AiModelSettings,
} from "../../../../foundation/ai/schema/model-settings";
import { useTaskStore } from "../../../../foundation/tasks/task-store";
import type { TaskTarget } from "../../../../shared/contracts/generated/TaskTarget";
import { transcriptFixture } from "../../__tests__/support/media-transcript-fixture";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceNodePayload } from "../../model/nodes/node-payload";
import { createSpaceMediaActions, type SpaceMediaAnalysisLabels } from "../actions";
import type { SpaceMediaAnalysisPreview } from "../analysis-preview";
import type { SpaceMediaPlaybackState, SpaceMediaSeekRequest, SpaceMediaToggleRequest } from "../playback";

const mocks = vi.hoisted(() => ({
  invoke: vi.fn<(command: string, args?: unknown) => Promise<unknown>>(),
  loadSettings: vi.fn<() => Promise<AiModelSettings>>(),
}));

vi.mock("../../../../foundation/ai/integration-settings", () => ({
  loadAiModelSettings: mocks.loadSettings,
  saveAiModelSettings: vi.fn<(value: AiModelSettings) => Promise<void>>(),
  resetAiModelSettings: vi.fn<() => Promise<void>>(),
}));
vi.mock("../../../../foundation/ipc/invoke-command", () => ({ invokeCommand: mocks.invoke }));
vi.mock("@tauri-apps/api/core", () => ({
  Channel: class {
    onmessage: unknown;
  },
}));

const TARGET_A: TaskTarget = { type: "model", connectionId: "connection-1", modelId: "model-a" };
const TARGET_B: TaskTarget = { type: "model", connectionId: "connection-1", modelId: "model-b" };
const AGENT_TARGET: TaskTarget = { type: "agent", runtimeKind: "codex" };
const labels: SpaceMediaAnalysisLabels = {
  title: "Organize media",
  preparing: "Preparing",
  transcoding: "Transcoding",
  transcribing: "Transcribing",
  organizing: "Organizing",
};
const node: SpaceCanvasNode = {
  id: "media-1",
  type: "audio",
  initialPosition: { x: 0, y: 0 },
  data: {
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
  },
};

function configuredSettings() {
  const settings = structuredClone(DEFAULT_AI_MODEL_SETTINGS);
  settings.connections = [
    {
      id: "connection-1",
      name: "Provider",
      provider: "openai",
      baseUrl: "https://api.example.com/v1/",
      models: ["model-a", "model-b"].map((modelId) => ({
        id: modelId,
        name: modelId,
        model: modelId,
        capabilities: ["text-generation"],
        builtinAgent: false,
      })),
    },
  ];
  settings.desktopAgents = [{ kind: "codex", enabled: true }];
  settings.defaults.knowledge = TARGET_A;
  return settings;
}

function actionsContext(): Parameters<typeof createSpaceMediaActions>[0] {
  return {
    nodes: ref([node]),
    activeSpaceId: ref("space-1"),
    applyNodeContentForSpace: vi.fn<(spaceId: string, nodeId: string, payload: SpaceNodePayload) => void>(),
    analysisIds: ref<string[]>([]),
    analysisPreviews: ref<Record<string, SpaceMediaAnalysisPreview>>({}),
    seekRequest: ref<SpaceMediaSeekRequest | undefined>(undefined),
    toggleRequest: ref<SpaceMediaToggleRequest | undefined>(undefined),
    playback: ref<{ id: string; state: SpaceMediaPlaybackState } | undefined>(undefined),
  };
}

beforeEach(() => {
  setActivePinia(createPinia());
  vi.clearAllMocks();
  mocks.loadSettings.mockImplementation(async () => configuredSettings());
  mocks.invoke.mockResolvedValue(node.data);
});

describe("media insights execution target", () => {
  it("sends the selected model and preserves it across Task Center retry after Settings changes", async () => {
    const actions = createSpaceMediaActions(actionsContext());
    mocks.invoke.mockRejectedValueOnce(new Error("Insights failed"));

    await expect(actions.generateMediaInsights(node.id, labels, TARGET_B)).rejects.toThrow("Insights failed");
    const tasks = useTaskStore();
    const jobId = tasks.tasks[0]!.id;
    expect(mocks.invoke).toHaveBeenLastCalledWith(
      "plugin:spaces|generate_media_insights",
      expect.objectContaining({
        spaceId: "space-1",
        nodeId: node.id,
        jobId,
        expectedRevision: "knowledge-1",
        target: { type: "model", connectionId: "connection-1", modelId: "model-b", temperature: 0.2 },
      }),
    );

    useAiSettingsStore().settings.defaults.knowledge = AGENT_TARGET;
    mocks.invoke.mockResolvedValueOnce(node.data);
    await tasks.retry(jobId);

    expect(mocks.invoke).toHaveBeenCalledTimes(2);
    expect(mocks.invoke).toHaveBeenLastCalledWith(
      "plugin:spaces|generate_media_insights",
      expect.objectContaining({
        jobId,
        target: { type: "model", connectionId: "connection-1", modelId: "model-b", temperature: 0.2 },
      }),
    );
    expect(tasks.tasks[0]?.status).toBe("succeeded");
    tasks.dismiss(jobId);
  });

  it("sends a desktop Agent target without model temperature", async () => {
    const actions = createSpaceMediaActions(actionsContext());

    await actions.generateMediaInsights(node.id, labels, AGENT_TARGET);

    expect(mocks.invoke).toHaveBeenLastCalledWith(
      "plugin:spaces|generate_media_insights",
      expect.objectContaining({ target: { type: "agent", runtimeKind: "codex" } }),
    );
    useTaskStore().dismiss(useTaskStore().tasks[0]!.id);
  });
});
