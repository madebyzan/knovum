import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { effectScope, shallowRef } from "vue";

import { localModelAvailability } from "../../__tests__/support/image-operation-fixtures";
import { useVoiceCandidateForm } from "../use-voice-candidate-form";

const draft = vi.hoisted(() => vi.fn<(request: { seed: number }) => void>());
vi.mock("vue-i18n", () => ({ useI18n: () => ({ t: (key: string) => key }) }));
vi.mock("../use-voice-candidates", () => ({
  useVoiceCandidates: () => ({
    candidates: shallowRef([]),
    busy: shallowRef(false),
    draft,
  }),
}));
beforeEach(() => {
  mockIPC((command) => {
    if (command !== "plugin:local-resources|get_model_catalog")
      throw new Error(`Unexpected command: ${command}`);
    return [localModelAvailability("htDemucs", "speechSeparation", { group: "audio" })];
  });
});

const scopes: ReturnType<typeof effectScope>[] = [];
function setup() {
  const scope = effectScope();
  scopes.push(scope);
  return scope.run(() => useVoiceCandidateForm())!;
}
afterEach(() => {
  clearMocks();
  scopes.splice(0).forEach((scope) => scope.stop());
  vi.clearAllMocks();
  vi.restoreAllMocks();
});

it("generates distinct voice proposals while preserving each submitted seed", async () => {
  vi.spyOn(crypto, "getRandomValues")
    .mockReturnValueOnce(new Uint32Array([42]))
    .mockReturnValueOnce(new Uint32Array([43]));
  const form = setup();
  form.mode.value = "designed";
  form.model.value = "voxCpm2";
  form.description.value = "A clear adult voice";
  form.modelAvailable.value = true;
  await form.generate();
  await form.generate();
  expect(draft).toHaveBeenCalledTimes(2);
  expect(draft.mock.calls[0]?.[0].seed).toBe(42);
  expect(draft.mock.calls[1]?.[0].seed).toBe(43);
  expect(draft.mock.calls[0]?.[0]).not.toHaveProperty("spaceId");
  expect(draft.mock.calls[0]?.[0]).not.toHaveProperty("authorized");
});

it("submits confirmed reference text according to the selected model contract", async () => {
  const form = setup();
  form.model.value = "voxCpm2Q8";
  form.referenceTranscript.value = "optionalConfirmed";
  form.referenceDuration.value = { minMs: 5_000, maxMs: 15_000 };
  form.reference.value = {
    sourcePath: "/reference.wav",
    sourceRevision: "sha256:reference",
    startMs: 0,
    endMs: 20_000,
    transcript: "Sample text",
    model: "whisperSmall",
  };
  form.transcriptConfirmed.value = true;
  form.modelAvailable.value = true;
  expect(form.ready.value).toBe(false);
  form.reference.value = {
    ...form.reference.value!,
    endMs: 15_000,
  };
  form.transcriptConfirmed.value = true;
  expect(form.ready.value).toBe(true);
  await form.generate();
  expect(draft).toHaveBeenCalledWith(
    expect.objectContaining({
      model: "voxCpm2Q8",
      source: expect.objectContaining({
        kind: "reference",
        referenceText: "Sample text",
        referenceTextConfirmed: true,
      }),
    }),
  );
});

it("treats the adopted machine model as the pristine baseline", () => {
  const form = setup();
  form.model.value = "voxCpm2Q8";
  form.initialModel.value = "voxCpm2Q8";
  expect(form.dirty.value).toBe(false);
  form.model.value = "voxCpm2";
  expect(form.dirty.value).toBe(true);
});

it("submits reference audio with optional design prompt when supported", async () => {
  const form = setup();
  form.model.value = "voxCpm2";
  form.supportsDesign.value = true;
  form.description.value = "  沉稳富有磁性的男声  ";
  form.referenceDuration.value = { minMs: 5_000, maxMs: 15_000 };
  form.reference.value = {
    sourcePath: "/reference.wav",
    sourceRevision: "sha256:reference",
    startMs: 0,
    endMs: 8_000,
    transcript: "",
    model: "whisperSmall",
  };
  form.modelAvailable.value = true;
  expect(form.ready.value).toBe(true);
  await form.generate();
  expect(draft).toHaveBeenCalledWith(
    expect.objectContaining({
      model: "voxCpm2",
      source: expect.objectContaining({
        kind: "reference",
        designPrompt: "沉稳富有磁性的男声",
      }),
    }),
  );
});
