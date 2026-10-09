import { afterEach, expect, it, vi } from "vitest";
import { effectScope, shallowRef } from "vue";

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

const scopes: ReturnType<typeof effectScope>[] = [];
function setup() {
  const scope = effectScope();
  scopes.push(scope);
  return scope.run(() => useVoiceCandidateForm(vi.fn()))!;
}
afterEach(() => {
  scopes.splice(0).forEach((scope) => scope.stop());
  vi.clearAllMocks();
  vi.restoreAllMocks();
});

it("generates distinct voice proposals while preserving each submitted seed", () => {
  vi.spyOn(crypto, "getRandomValues")
    .mockReturnValueOnce(new Uint32Array([42]))
    .mockReturnValueOnce(new Uint32Array([43]));
  const form = setup();
  form.mode.value = "designed";
  form.model.value = "voxCpm2";
  form.description.value = "A clear adult voice";
  form.authorized.value = true;
  form.modelAvailable.value = true;
  form.generate();
  form.generate();
  expect(draft).toHaveBeenCalledTimes(2);
  expect(draft.mock.calls[0]?.[0].seed).toBe(42);
  expect(draft.mock.calls[1]?.[0].seed).toBe(43);
  expect(draft.mock.calls[0]?.[0]).not.toHaveProperty("spaceId");
});

it("submits confirmed reference text according to the selected model contract", () => {
  const form = setup();
  form.model.value = "voxCpm2Q8";
  form.referenceTranscript.value = "optionalConfirmed";
  form.reference.value = {
    sourcePath: "/reference.wav",
    sourceRevision: "sha256:reference",
    startMs: 0,
    endMs: 5_000,
    transcript: "Sample text",
  };
  form.transcriptConfirmed.value = true;
  form.authorized.value = true;
  form.modelAvailable.value = true;
  form.generate();
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
