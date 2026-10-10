import { flushPromises } from "@vue/test-utils";
import { afterEach, expect, it, vi } from "vitest";
import { effectScope, shallowRef } from "vue";

import type { SpeechProjectSnapshot } from "../../contracts/generated/SpeechProjectSnapshot";
import { useSpeechWorkspace } from "../use-speech-workspace";

const ipc = vi.hoisted(() => ({
  list: vi.fn<() => Promise<unknown[]>>(),
  get: vi.fn<() => Promise<SpeechProjectSnapshot>>(),
  save: vi.fn<() => Promise<SpeechProjectSnapshot>>(),
  generate: vi.fn<(revision: number | undefined) => void>(),
}));
vi.mock("../ipc", () => ({
  listSpeechProjects: ipc.list,
  getSpeechProject: ipc.get,
  saveSpeechProjectRevision: ipc.save,
}));
vi.mock("../../session/space-store", () => ({ useSpaceStore: () => ({ activeSpaceId: "space" }) }));
vi.mock("../use-speech-voices", () => ({
  useSpeechVoices: () => ({ voices: shallowRef([]), loading: shallowRef(false), errors: shallowRef([]) }),
}));
vi.mock("../use-speech-generation", () => ({
  useSpeechGeneration: (owner: { project: () => SpeechProjectSnapshot | undefined }) => ({
    errors: shallowRef([]),
    hasActiveExecution: shallowRef(false),
    submitting: shallowRef(false),
    reset: vi.fn<() => void>(),
    clearPreview: vi.fn<() => void>(),
    generateProject: () => ipc.generate(owner.project()?.project.activeRevision),
  }),
}));
afterEach(() => vi.clearAllMocks());

function snapshot(revision: number): SpeechProjectSnapshot {
  return {
    project: {
      id: "project",
      spaceId: "space",
      sourceNodeId: "source",
      projectType: "short_dubbing",
      title: "Narration",
      activeRevisionId: `r${revision}`,
      activeRevision: revision,
      segmentCount: 0,
      updatedAt: revision,
    },
    revision: {
      id: `r${revision}`,
      projectId: "project",
      revision,
      sourceRevision: "source",
      pronunciationLexicon: {},
      outputConfig: { format: "wav", wavPrecision: "pcm16" },
      createdAt: revision,
    },
    segments: [],
  };
}

it.each([true, false])(
  "generates the saved draft only when its revision write succeeds (%s)",
  async (succeeded) => {
    ipc.list.mockResolvedValue([snapshot(1).project]);
    ipc.get.mockResolvedValue(snapshot(1));
    let resolveSave!: (saved: SpeechProjectSnapshot) => void;
    let rejectSave!: (error: Error) => void;
    ipc.save.mockImplementation(
      () =>
        new Promise((resolve, reject) => {
          resolveSave = resolve;
          rejectSave = reject;
        }),
    );
    const scope = effectScope();
    try {
      const workspace = scope.run(() =>
        useSpeechWorkspace(
          () => "source",
          () => "short_dubbing",
        ),
      )!;
      await flushPromises();
      const input = {
        sourceRevision: "edited",
        outputConfig: { format: "wav", wavPrecision: "pcm16" },
        pronunciationLexicon: {},
        segments: [],
      };
      const submission = workspace.startGeneration(input);
      expect(ipc.generate).not.toHaveBeenCalled();
      await workspace.startGeneration(input);
      expect(ipc.save).toHaveBeenCalledTimes(1);
      if (succeeded) resolveSave(snapshot(2));
      else rejectSave(new Error("Disk write failed"));
      await submission;
      expect(ipc.generate).toHaveBeenCalledTimes(succeeded ? 1 : 0);
      expect(ipc.generate.mock.calls).toEqual(succeeded ? [[2]] : []);
      expect(workspace.errors.value).toEqual(succeeded ? [] : ["Disk write failed"]);
    } finally {
      scope.stop();
    }
  },
);

it("does not start delayed generation after the editor is disposed during a save", async () => {
  ipc.list.mockResolvedValue([snapshot(1).project]);
  ipc.get.mockResolvedValue(snapshot(1));
  let resolveSave!: (saved: SpeechProjectSnapshot) => void;
  ipc.save.mockImplementation(
    () =>
      new Promise((resolve) => {
        resolveSave = resolve;
      }),
  );
  const scope = effectScope();
  const workspace = scope.run(() =>
    useSpeechWorkspace(
      () => "source",
      () => "short_dubbing",
    ),
  )!;
  await flushPromises();
  const submission = workspace.startGeneration({
    sourceRevision: "edited",
    outputConfig: {},
    pronunciationLexicon: {},
    segments: [],
  });
  scope.stop();
  resolveSave(snapshot(2));
  await submission;
  expect(ipc.generate).not.toHaveBeenCalled();
});
