import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { afterEach, expect, it, vi } from "vitest";

import { createSpaceNode } from "../canvas/node";

const context = vi.hoisted(() => ({
  store: { activeSpaceId: "space" },
  observe: vi.fn<(...args: unknown[]) => void>(),
}));
vi.mock("../session/space-store", () => ({ useSpaceStore: () => context.store }));
vi.mock("../runs/resource-run-observation", () => ({ observeResourceRun: context.observe }));
vi.mock("../../../foundation/i18n/instance", () => ({ i18n: { global: { t: (key: string) => key } } }));
import { addVideoSubtitles } from "../knowledge-derivations/subtitled-video";

afterEach(() => {
  clearMocks();
  vi.clearAllMocks();
});
it("starts one local video Action, with no Workflow, transcript resource or model target", async () => {
  const source = createSpaceNode(
    {
      kind: "video",
      title: "Interview",
      fileName: "interview.mp4",
      assetPath: "/owned/source.mp4",
      sourceRevision: "asset:1",
      knowledgeRevision: "knowledge:1",
    },
    { x: 0, y: 0 },
    "source",
  );
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
});
