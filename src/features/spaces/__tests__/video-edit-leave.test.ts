import { mount } from "@vue/test-utils";
import { expect, it, onTestFinished, vi } from "vitest";
import { defineComponent, nextTick, shallowRef } from "vue";

import { useVideoEditLeave } from "../presentation/video-editor/use-video-edit-leave";
import { prepareEditorSessionClose } from "../workspace/session-participants";

it("prepares a video discard and applies it only after close succeeds", async () => {
  const discard = vi.fn<() => void>();
  const close = vi.fn<() => Promise<void>>().mockImplementation(async () => {
    const finalize = await prepareEditorSessionClose({ tabIds: ["video-tab"] });
    finalize();
  });
  let leave!: ReturnType<typeof useVideoEditLeave>;
  const Harness = defineComponent({
    setup() {
      leave = useVideoEditLeave({
        tab: { id: "video-tab", kind: "video", spaceId: "space", nodeId: "video" },
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
  expect(discard).not.toHaveBeenCalled();
  leave.discard();
  await vi.waitFor(() => expect(discard).toHaveBeenCalledOnce());
  expect(close).toHaveBeenCalledOnce();
});
