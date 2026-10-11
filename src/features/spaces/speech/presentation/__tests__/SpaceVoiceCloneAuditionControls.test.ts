import { flushPromises, mount } from "@vue/test-utils";
import { ToolbarRoot } from "reka-ui";
import { expect, it, onTestFinished, vi } from "vitest";
import { computed, defineComponent, h, reactive, shallowRef } from "vue";

import { i18n } from "../../../../../foundation/i18n/instance";
import { useVoiceCandidateForm } from "../../use-voice-candidate-form";
import type { useVoiceCandidates } from "../../use-voice-candidates";
import type { PlayableVoiceCandidate } from "../../voice-candidate-selection";
import SpaceVoiceCloneAuditionControls from "../SpaceVoiceCloneAuditionControls.vue";
import type { useCanvasVoiceClone } from "../use-canvas-voice-clone";

const mocks = vi.hoisted(() => ({ candidates: vi.fn<typeof useVoiceCandidates>() }));
vi.mock("../../use-voice-candidates", () => ({ useVoiceCandidates: mocks.candidates }));
const candidate: PlayableVoiceCandidate = {
  model: "mossNano",
  candidateId: "candidate",
  runId: "run",
  sourceKind: "reference",
  language: "zh-CN",
  sampleText: "Hello",
  seed: 1,
  designDescription: null,
  referenceText: null,
  audio: { path: "/audition.wav", sha256: "hash", sampleRate: 48000, sampleCount: 48000 },
  status: "ready",
  auditionedAt: 1,
  selectedVoiceProfileId: null,
  selectedVoiceRevisionId: null,
  selectedAt: null,
  createdAt: 1,
  source: "/audition.wav",
};
function render() {
  const candidates = shallowRef<PlayableVoiceCandidate[]>([]);
  const activeRunId = shallowRef<string>();
  const pendingRequestId = shallowRef<string>();
  const cancelling = shallowRef(false);
  const cancel = vi.fn<ReturnType<typeof useVoiceCandidates>["cancel"]>().mockResolvedValue();
  mocks.candidates.mockReturnValue({
    candidates: computed(() => candidates.value),
    activeRunId,
    pendingRequestId: computed(() => pendingRequestId.value),
    isCancelling: cancelling,
    busy: computed(() => Boolean(activeRunId.value || pendingRequestId.value)),
    errors: computed(() => []),
    draft: vi.fn<ReturnType<typeof useVoiceCandidates>["draft"]>(() => {
      activeRunId.value = "run";
    }),
    save: vi.fn<ReturnType<typeof useVoiceCandidates>["save"]>().mockResolvedValue(undefined),
    cancel,
    discard: vi.fn<ReturnType<typeof useVoiceCandidates>["discard"]>(),
    audition: vi.fn<ReturnType<typeof useVoiceCandidates>["audition"]>(),
  });
  let form!: ReturnType<typeof useCanvasVoiceClone>["form"];
  const openPopover = shallowRef<"audition" | "candidates">();
  const Harness = defineComponent({
    setup() {
      form = reactive(useVoiceCandidateForm(() => "reference"));
      form.modelAvailable = true;
      form.reference = {
        sourcePath: "/reference.wav",
        sourceRevision: "sha256:source",
        startMs: 0,
        endMs: 10000,
        transcript: "Reference",
      };
      return () =>
        h(
          ToolbarRoot,
          {},
          {
            default: () =>
              h(SpaceVoiceCloneAuditionControls, {
                form,
                compact: false,
                openPopover: openPopover.value,
                "onUpdate:openPopover": (next) => {
                  openPopover.value = next;
                },
              }),
          },
        );
    },
  });
  const wrapper = mount(Harness, { attachTo: document.body, global: { plugins: [i18n] } });
  onTestFinished(() => {
    wrapper.unmount();
    document.body.innerHTML = "";
  });
  return { wrapper, form, candidates, cancelling, cancel, openPopover };
}

it("keeps the audition editor mounted, retains input and offers explicit cancellation while generating", async () => {
  const { wrapper, form, cancelling, cancel, openPopover } = render();
  await wrapper.get('[data-testid="clone-audition-options"]').trigger("click");
  await flushPromises();
  const textarea = document.querySelector<HTMLTextAreaElement>("textarea")!;
  textarea.value = "My audition text";
  textarea.dispatchEvent(new Event("input", { bubbles: true }));
  await flushPromises();
  const generate = document.querySelector<HTMLButtonElement>('[data-testid="clone-generate-audition"]')!;
  generate.focus();
  generate.click();
  await flushPromises();
  expect(form.activeRunId).toBe("run");
  expect(openPopover.value).toBe("audition");
  expect(document.querySelector("textarea")).toBe(textarea);
  expect(textarea.value).toBe("My audition text");
  expect(textarea.disabled).toBe(true);
  const popup = textarea.closest('[role="dialog"]')!;
  const cancelButton = popup.querySelector<HTMLButtonElement>('[data-testid="clone-cancel-audition"]')!;
  expect(cancelButton).toBe(generate);
  expect(document.activeElement).toBe(generate);
  cancelButton.click();
  expect(cancel).toHaveBeenCalledOnce();
  cancelling.value = true;
  await flushPromises();
  expect(cancelButton.disabled).toBe(true);
});

it("preserves candidate names across popover switches and returns focus after the last candidate is removed", async () => {
  const { wrapper, candidates, openPopover } = render();
  candidates.value = [candidate];
  await flushPromises();
  await wrapper.get('[data-testid="clone-auditions"]').trigger("click");
  await flushPromises();
  const name = document.querySelector<HTMLInputElement>("input")!;
  name.value = "Narrator";
  name.dispatchEvent(new Event("input", { bubbles: true }));
  await flushPromises();
  openPopover.value = "audition";
  await flushPromises();
  openPopover.value = "candidates";
  await flushPromises();
  expect(document.querySelector("input")).toBe(name);
  expect(name.value).toBe("Narrator");
  candidates.value = [];
  await flushPromises();
  expect(openPopover.value).toBeUndefined();
  expect(document.activeElement).toBe(wrapper.get('[data-testid="clone-audition-options"]').element);
});
