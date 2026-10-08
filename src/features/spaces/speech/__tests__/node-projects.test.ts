import { afterEach, expect, it, vi } from "vitest";
import { effectScope, shallowRef } from "vue";

import { useSpeechWorkspace } from "../use-speech-workspace";

const ipc = vi.hoisted(() => ({
  listSpeechProjects: vi.fn<(spaceId: string, nodeId: string) => Promise<unknown[]>>(),
  getSpeechProject: vi.fn<(id: string) => Promise<unknown>>(),
}));
vi.mock("../../session/space-store", () => ({ useSpaceStore: () => ({ activeSpaceId: "space" }) }));
vi.mock("../ipc", () => ipc);
vi.mock("../use-speech-voices", () => ({
  useSpeechVoices: () => ({ load: async () => [], replace: vi.fn<() => void>() }),
}));
vi.mock("../use-speech-generation", () => ({
  useSpeechGeneration: () => ({ reset: vi.fn<() => void>() }),
}));
afterEach(() => vi.clearAllMocks());

it("queries projects for the source node and discards a late response after node switching", async () => {
  let resolveFirst!: (value: unknown[]) => void;
  ipc.listSpeechProjects.mockImplementationOnce(
    () =>
      new Promise((resolve) => {
        resolveFirst = resolve;
      }),
  );
  ipc.listSpeechProjects.mockResolvedValueOnce([
    { id: "b-project", sourceNodeId: "b", projectType: "short_dubbing" },
  ]);
  const nodeId = shallowRef("a");
  const scope = effectScope();
  const workspace = scope.run(() =>
    useSpeechWorkspace(
      () => nodeId.value,
      () => "short_dubbing",
    ),
  )!;
  expect(ipc.listSpeechProjects).toHaveBeenCalledWith("space", "a");
  nodeId.value = "b";
  await vi.waitFor(() => expect(workspace.projects.value[0]?.id).toBe("b-project"));
  resolveFirst([{ id: "a-project", sourceNodeId: "a" }]);
  await Promise.resolve();
  expect(workspace.projects.value[0]?.id).toBe("b-project");
  scope.stop();
});

it("refuses to open a project owned by another source node", async () => {
  ipc.listSpeechProjects.mockResolvedValue([]);
  ipc.getSpeechProject.mockResolvedValue({
    project: { id: "foreign", spaceId: "space", sourceNodeId: "other" },
  });
  const scope = effectScope();
  const workspace = scope.run(() =>
    useSpeechWorkspace(
      () => "source",
      () => "short_dubbing",
    ),
  )!;
  await workspace.openProject("foreign");
  expect(workspace.selectedProject.value).toBeUndefined();
  scope.stop();
});

it("keeps another operation's projects out of the selected node editor", async () => {
  ipc.listSpeechProjects.mockResolvedValue([
    { id: "speech", sourceNodeId: "source", projectType: "short_dubbing" },
    { id: "book", sourceNodeId: "source", projectType: "audiobook" },
  ]);
  ipc.getSpeechProject.mockResolvedValue({
    project: { id: "book", spaceId: "space", sourceNodeId: "source", projectType: "audiobook" },
  });
  const scope = effectScope();
  const workspace = scope.run(() =>
    useSpeechWorkspace(
      () => "source",
      () => "short_dubbing",
    ),
  )!;
  await vi.waitFor(() => expect(workspace.projects.value.map((project) => project.id)).toEqual(["speech"]));
  await workspace.openProject("book");
  expect(workspace.selectedProject.value).toBeUndefined();
  scope.stop();
});
