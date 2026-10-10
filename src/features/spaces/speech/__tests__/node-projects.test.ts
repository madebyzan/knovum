import { afterEach, expect, it, vi } from "vitest";
import { effectScope, shallowRef } from "vue";

import type { SaveSpeechProjectRevisionRequest } from "../../contracts/generated/SaveSpeechProjectRevisionRequest";
import type { SpeechProjectSnapshot } from "../../contracts/generated/SpeechProjectSnapshot";
import { useSpeechWorkspace } from "../use-speech-workspace";

const ipc = vi.hoisted(() => ({
  listSpeechProjects: vi.fn<(spaceId: string, nodeId: string) => Promise<unknown[]>>(),
  getSpeechProject: vi.fn<(projectId: string) => Promise<unknown>>(),
  saveSpeechProjectRevision:
    vi.fn<(input: SaveSpeechProjectRevisionRequest) => Promise<SpeechProjectSnapshot>>(),
}));
vi.mock("../../session/space-store", () => ({ useSpaceStore: () => ({ activeSpaceId: "space" }) }));
vi.mock("../ipc", () => ipc);
vi.mock("../use-speech-voices", () => ({
  useSpeechVoices: () => ({
    voices: shallowRef([]),
    loading: shallowRef(false),
    creating: shallowRef(false),
    errors: shallowRef([]),
    addPreset: vi.fn<() => void>(),
  }),
}));
vi.mock("../use-speech-generation", () => ({
  useSpeechGeneration: () => ({
    errors: shallowRef([]),
    reset: vi.fn<() => void>(),
    clearPreview: vi.fn<() => void>(),
  }),
}));
afterEach(() => vi.clearAllMocks());

function projectSnapshot(revision: number): SpeechProjectSnapshot {
  return {
    project: {
      id: "project",
      spaceId: "space",
      sourceNodeId: "source",
      projectType: "short_dubbing",
      title: "Speech",
      activeRevisionId: `revision-${revision}`,
      activeRevision: revision,
      segmentCount: 0,
      updatedAt: revision,
    },
    revision: {
      id: `revision-${revision}`,
      projectId: "project",
      revision,
      sourceRevision: "source-revision",
      pronunciationLexicon: {},
      outputConfig: { format: "wav", wavPrecision: "pcm16" },
      createdAt: revision,
    },
    segments: [],
  };
}

it.each([true, false])("shares an admitted revision write with the leave barrier (%s)", async (succeeded) => {
  ipc.listSpeechProjects.mockResolvedValue([
    { ...projectSnapshot(1).project, id: "older", updatedAt: 0 },
    projectSnapshot(1).project,
  ]);
  ipc.getSpeechProject.mockResolvedValue(projectSnapshot(1));
  let resolveWrite!: (snapshot: SpeechProjectSnapshot) => void;
  let rejectWrite!: (error: Error) => void;
  ipc.saveSpeechProjectRevision.mockImplementation(
    () =>
      new Promise((resolve, reject) => {
        resolveWrite = resolve;
        rejectWrite = reject;
      }),
  );
  const scope = effectScope();
  const workspace = scope.run(() =>
    useSpeechWorkspace(
      () => "source",
      () => "short_dubbing",
    ),
  )!;
  await vi.waitFor(() => expect(workspace.loading.value).toBe(false));
  expect(workspace.selectedProject.value?.project.id).toBe("project");
  expect(ipc.getSpeechProject).toHaveBeenCalledExactlyOnceWith("project");
  const input = {
    sourceRevision: "source-revision",
    pronunciationLexicon: {},
    outputConfig: {},
    segments: [],
  };
  const saving = workspace.saveRevision(input);
  expect(workspace.saveRevision(input)).toBe(saving);
  const settled = vi.fn<() => void>();
  const leaving = workspace.settleRevisionSave().then(settled);
  await Promise.resolve();
  expect(workspace.savingRevision.value).toBe(true);
  expect(settled).not.toHaveBeenCalled();
  expect(ipc.saveSpeechProjectRevision).toHaveBeenCalledOnce();
  expect(ipc.saveSpeechProjectRevision).toHaveBeenCalledWith({
    ...input,
    projectId: "project",
    expectedRevision: 1,
  });
  if (succeeded) resolveWrite(projectSnapshot(2));
  else rejectWrite(new Error("revision write failed"));
  expect(await saving).toBe(succeeded);
  await leaving;
  expect(settled).toHaveBeenCalledOnce();
  expect(workspace.savingRevision.value).toBe(false);
  expect(workspace.selectedProject.value?.project.activeRevision).toBe(succeeded ? 2 : 1);
  expect(workspace.errors.value).toEqual(succeeded ? [] : ["revision write failed"]);
  scope.stop();
});

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
