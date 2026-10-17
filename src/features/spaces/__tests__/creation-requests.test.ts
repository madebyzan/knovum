import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { afterEach, expect, it, vi } from "vitest";
vi.mock("../session/space-store", () => ({ useSpaceStore: () => ({ activeSpaceId: "space" }) }));
const observed = vi.hoisted(() => vi.fn<(...args: unknown[]) => void>());
vi.mock("../runs/resource-run-observation", () => ({ observeResourceRun: observed }));
vi.mock("../../../foundation/i18n/instance", () => ({ i18n: { global: { t: (key: string) => key } } }));
import { startCreation } from "../knowledge-derivations/creation";

afterEach(() => {
  clearMocks();
  vi.clearAllMocks();
});

const SOURCES = [
  {
    nodeId: "note",
    title: "Idea",
    sourceRevision: "knowledge:1",
    revisionBasis: "knowledge" as const,
  },
];

it("creates a video from the selection with only its explicit executor", async () => {
  const invoke = vi.fn<(command: string, args?: unknown) => void>();
  mockIPC((command, args) => {
    invoke(command, args);
    return { id: "video-run" };
  });
  await startCreation(SOURCES, {
    output: "video",
    title: "Video",
    requirement: "Camera movement",
    imageAspectRatio: "16:9",
    videoAspectRatio: "16:9",
    durationSeconds: 5,
    target: { type: "agent", runtimeKind: "codex" },
  });
  expect(invoke).toHaveBeenCalledOnce();
  expect(invoke).toHaveBeenCalledWith(
    "plugin:spaces|start_space_action",
    expect.objectContaining({
      request: expect.objectContaining({
        actionId: "generate-video",
        parameters: { title: "Video", prompt: "Camera movement", aspectRatio: "16:9", durationSeconds: 5 },
        sources: [{ nodeId: "note", sourceRevision: "knowledge:1", revisionBasis: "knowledge" }],
        target: { type: "agent", runtimeKind: "codex" },
      }),
    }),
  );
});

it("creates an image from the same selection, with the image options only", async () => {
  const invoke = vi.fn<(command: string, args?: unknown) => void>();
  mockIPC((command, args) => {
    invoke(command, args);
    return { id: "image-run" };
  });
  await startCreation(SOURCES, {
    output: "image",
    title: "Variant",
    requirement: "Make it snow",
    imageAspectRatio: "1:1",
    videoAspectRatio: "16:9",
    durationSeconds: 5,
    target: { type: "agent", runtimeKind: "codex" },
  });
  expect(invoke).toHaveBeenCalledWith(
    "plugin:spaces|start_space_action",
    expect.objectContaining({
      request: expect.objectContaining({
        actionId: "generate-visualization",
        parameters: { title: "Variant", intent: "Make it snow", aspectRatio: "1:1" },
      }),
    }),
  );
});
