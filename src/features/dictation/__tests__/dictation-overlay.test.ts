import { flushPromises, mount } from "@vue/test-utils";
import { expect, it, onTestFinished, vi } from "vitest";
import { readonly, shallowRef } from "vue";

import type { DictationSnapshot } from "../contracts/generated/DictationSnapshot";
import DictationOverlay from "../DictationOverlay.vue";
import type { useDictationSession } from "../use-dictation-session";

const mocks = vi.hoisted(() => ({ session: vi.fn<typeof useDictationSession>() }));
vi.mock("../use-dictation-session", () => ({ useDictationSession: mocks.session }));
vi.mock("../../../foundation/i18n/instance", () => ({ i18n: { global: { t: (key: string) => key } } }));
vi.mock("vue-i18n", () => ({ useI18n: () => ({ t: (key: string) => key }) }));
it("shows resource preparation and cancellation, then switches to transcription after preparation", async () => {
  const state = shallowRef<DictationSnapshot>({
    phase: "preparing_resources",
    resources: { stage: "downloading", downloaded: 1024, total: 2048, percent: 50 },
    sessionId: "recording",
    startedAt: 1,
    targetApplicationName: "Editor",
    text: null,
    language: null,
    insertion: null,
    errorCode: null,
    errorMessage: null,
  });
  type Session = ReturnType<typeof useDictationSession>;
  const cancel = vi.fn<Session["cancel"]>().mockResolvedValue(undefined);
  mocks.session.mockReturnValue({
    state: readonly(state),
    start: vi.fn<Session["start"]>().mockResolvedValue(undefined),
    stop: vi.fn<Session["stop"]>(),
    cancel,
    dismiss: vi.fn<Session["dismiss"]>(),
    copy: vi.fn<Session["copy"]>(),
  });
  const wrapper = mount(DictationOverlay, { global: { stubs: { UiDialog: true } } });
  onTestFinished(() => wrapper.unmount());
  await flushPromises();
  expect(wrapper.get('[role="status"]').text()).toContain("dictation.preparingResources");
  expect(wrapper.get('[data-testid="dictation-resource-progress"]').text()).toContain("1.0 KiB / 2.0 KiB");
  await wrapper.get("button").trigger("click");
  expect(cancel).toHaveBeenCalledOnce();
  state.value = { ...state.value, phase: "transcribing", resources: null };
  await flushPromises();
  expect(wrapper.get('[role="status"]').text()).toContain("dictation.transcribing");
  expect(wrapper.find('[data-testid="dictation-resource-progress"]').exists()).toBe(false);
});
