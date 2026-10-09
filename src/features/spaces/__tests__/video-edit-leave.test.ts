import { mount } from "@vue/test-utils";
import { expect, it, onTestFinished, vi } from "vitest";
import { defineComponent, nextTick, shallowRef } from "vue";

import { useVideoEditLeave } from "../presentation/video-editor/use-video-edit-leave";

it("discards and closes a local video edit exactly once", async () => {
  const discard = vi.fn<() => void>();
  const close = vi.fn<() => void>();
  let leave!: ReturnType<typeof useVideoEditLeave>;
  const Harness = defineComponent({
    setup() {
      leave = useVideoEditLeave({
        dirty: shallowRef(true),
        saving: shallowRef(false),
        save: async () => true,
        discard,
        close,
      });
      return () => null;
    },
  });
  const wrapper = mount(Harness);
  onTestFinished(() => wrapper.unmount());

  leave.requestClose();
  await nextTick();
  expect(leave.closeDialogOpen.value).toBe(true);
  leave.discard();
  await vi.waitFor(() => expect(discard).toHaveBeenCalledOnce());

  expect(close).not.toHaveBeenCalled();
});
