import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { afterEach, expect, it, vi } from "vitest";

import type { useSpaceStore } from "../session/space-store";
const observed = vi.hoisted(() => vi.fn<(...args: unknown[]) => void>());
vi.mock("../runs/resource-run-observation", () => ({ observeResourceRun: observed }));
vi.mock("../../../foundation/i18n/instance", () => ({ i18n: { global: { t: (key: string) => key } } }));
import { generateVideoFromSources } from "../knowledge-derivations/video/generation";

afterEach(() => {
  clearMocks();
  vi.clearAllMocks();
});
it("generates video directly from selected knowledge with only its explicit executor", async () => {
  const invoke = vi.fn<(command: string, args?: unknown) => void>();
  mockIPC((command, args) => {
    invoke(command, args);
    return { id: "video-run" };
  });
  await generateVideoFromSources({
    store: { activeSpaceId: "space" } as ReturnType<typeof useSpaceStore>,
    sources: [{ nodeId: "note", role: "primary", sourceRevision: "knowledge:1", title: "Idea" }],
    draft: {
      title: "Video",
      prompt: "Camera movement",
      aspectRatio: "16:9",
      durationSeconds: 5,
      target: { type: "agent", runtimeKind: "codex" },
    },
  });
  expect(invoke).toHaveBeenCalledOnce();
  expect(invoke).toHaveBeenCalledWith(
    "plugin:spaces|start_space_action",
    expect.objectContaining({
      request: expect.objectContaining({
        actionId: "generate-video",
        parameters: { title: "Video", prompt: "Camera movement", aspectRatio: "16:9", durationSeconds: 5 },
        sources: [{ nodeId: "note", knowledgeRevision: "knowledge:1" }],
        target: { type: "agent", runtimeKind: "codex" },
      }),
    }),
  );
});
