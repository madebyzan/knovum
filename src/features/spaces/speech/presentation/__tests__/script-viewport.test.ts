import { mount } from "@vue/test-utils";
import { expect, it, onTestFinished, vi } from "vitest";
import { computed, shallowRef } from "vue";

import { overlayScopeVisibleKey } from "../../../../../shared/contracts/overlay-scope-visibility";
import UiScrollArea from "../../../../../shared/ui/UiScrollArea.vue";
import { SPEECH_EDITOR_VIEWPORT_KEY } from "../speech-editor-viewport";
import VideoDubbingLines from "../VideoDubbingLines.vue";

const virtualize = vi.hoisted(() => vi.fn<(options: unknown) => void>());
vi.mock("@tanstack/vue-virtual", () => ({
  defaultRangeExtractor: () => [0],
  useVirtualizer: (options: unknown) => {
    virtualize(options);
    return shallowRef({ getVirtualItems: () => [], getTotalSize: () => 0, measureElement: () => {} });
  },
}));
it("virtualizes timed script rows against the project viewport without a second scroll area", () => {
  const viewport = document.createElement("div");
  const wrapper = mount(VideoDubbingLines, {
    props: {
      lines: [],
      voiceOptions: [],
      deliveryInstructionVoiceIds: new Set<string>(),
      lexiconValid: true,
      showSource: false,
      disabled: false,
    },
    global: {
      provide: {
        [SPEECH_EDITOR_VIEWPORT_KEY as symbol]: computed(() => viewport),
        [overlayScopeVisibleKey as symbol]: computed(() => true),
      },
    },
  });
  onTestFinished(() => wrapper.unmount());
  expect(wrapper.findComponent(UiScrollArea).exists()).toBe(false);
  const options = virtualize.mock.calls.at(-1)![0] as { value: { getScrollElement: () => HTMLElement } };
  expect(options.value.getScrollElement()).toBe(viewport);
});
