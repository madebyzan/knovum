import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { afterEach, expect, it, vi } from "vitest";

import { createSpaceNode } from "../canvas/node";
import type { SpaceVideoPayload } from "../model/nodes/node-payload";
import { transcriptFixture } from "./support/media-transcript-fixture";

const context = vi.hoisted(() => ({
  store: { activeSpaceId: "space" },
  observe: vi.fn<(...args: unknown[]) => void>(),
  admit: vi.fn<(...args: unknown[]) => Promise<boolean>>(),
}));
vi.mock("../../../foundation/ai/ai-readiness-admission", () => ({ admitAiRequirements: context.admit }));
vi.mock("../session/space-store", () => ({ useSpaceStore: () => context.store }));
vi.mock("../runs/resource-run-observation", () => ({ observeResourceRun: context.observe }));
vi.mock("../../../foundation/i18n/instance", () => ({ i18n: { global: { t: (key: string) => key } } }));
import { addVideoSubtitles } from "../knowledge-derivations/subtitled-video";

afterEach(() => {
  clearMocks();
  vi.clearAllMocks();
});
function video(transcribed = false) {
  return createSpaceNode<SpaceVideoPayload>(
    {
      kind: "video",
      title: "Interview",
      fileName: "interview.mp4",
      assetPath: "/owned/source.mp4",
      sourceRevision: "asset:1",
      knowledgeRevision: "knowledge:1",
      ...(transcribed
        ? {
            mediaKnowledge: {
              schemaVersion: 1,
              sourceRevision: "asset:1",
              transcript: transcriptFixture("asset:1", [{ startMs: 0, endMs: 1000, text: "Hello" }]),
            },
          }
        : {}),
    },
    { x: 0, y: 0 },
    "source",
  );
}
it("starts one local video Action, with no Workflow, transcript resource or model target", async () => {
  context.admit.mockResolvedValue(true);
  const source = video();
  const data = structuredClone(source.data);
  const store = context.store;
  const calls: Array<[string, unknown]> = [];
  mockIPC((command, args) => {
    calls.push([command, args]);
    store.activeSpaceId = "other-space";
    return { id: "run", scope: { kind: "space", spaceId: "space" } };
  });
  await addVideoSubtitles(source);
  expect(calls).toHaveLength(1);
  expect(calls[0]![0]).toBe("plugin:spaces|start_space_action");
  expect(calls[0]![1]).toMatchObject({
    request: {
      spaceId: "space",
      actionId: "add-video-subtitles",
      parameters: {},
      sources: [{ nodeId: "source", sourceRevision: "knowledge:1", revisionBasis: "knowledge" }],
      target: null,
    },
  });
  expect(source.data).toEqual(data);
  expect(context.observe).toHaveBeenCalledWith(
    { id: "run", scope: { kind: "space", spaceId: "space" } },
    "spaces.creation.video.subtitles.task",
    undefined,
  );
  expect(context.admit).toHaveBeenCalledWith([{ kind: "localCapability", capability: "transcription" }]);
});

it("starts nothing when the transcription model it needs is not admitted", async () => {
  context.admit.mockResolvedValue(false);
  const calls: string[] = [];
  mockIPC((command) => void calls.push(command));
  expect(await addVideoSubtitles(video())).toBeUndefined();
  expect(calls).toEqual([]);
});

it("needs no local model when the video already has a transcript to subtitle from", async () => {
  context.admit.mockResolvedValue(true);
  mockIPC(() => ({ id: "run", scope: { kind: "space", spaceId: "space" } }));
  await addVideoSubtitles(video(true));
  expect(context.admit).toHaveBeenCalledWith([]);
});
