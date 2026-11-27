import { mount } from "@vue/test-utils";
import { expect, it, vi } from "vitest";
import { nextTick } from "vue";

import { i18n } from "../../../../../foundation/i18n/instance";
import SpaceTiptapImageView from "../SpaceTiptapImageView.vue";

function createEditorStub(initialEditable: boolean) {
  const listeners = new Set<() => void>();
  const editor = {
    isEditable: initialEditable,
    on(event: string, listener: () => void) {
      if (event === "update") listeners.add(listener);
    },
    off(event: string, listener: () => void) {
      if (event === "update") listeners.delete(listener);
    },
    setEditable(next: boolean) {
      editor.isEditable = next;
      for (const listener of listeners) listener();
    },
  };
  return editor;
}

it("reveals the alt caption input after an initially inactive editor becomes editable and selected", async () => {
  const editor = createEditorStub(false);
  const updateAttributes = vi.fn<(attrs: Record<string, unknown>) => void>();
  const wrapper = mount(SpaceTiptapImageView, {
    props: {
      editor: editor as never,
      node: { attrs: { src: "https://example.com/a.png", alt: "", width: null, assetPath: null } } as never,
      decorations: [] as never,
      innerDecorations: {} as never,
      view: {} as never,
      selected: false,
      extension: {} as never,
      HTMLAttributes: {},
      getPos: () => 0,
      updateAttributes,
      deleteNode: vi.fn<() => void>(),
    },
    global: {
      plugins: [i18n],
      stubs: { NodeViewWrapper: { template: "<figure><slot /></figure>" } },
    },
  });

  expect(wrapper.find('[data-testid="image-caption-input"]').exists()).toBe(false);

  editor.setEditable(true);
  await wrapper.setProps({ selected: true });
  await nextTick();

  const input = wrapper.find('[data-testid="image-caption-input"]');
  expect(input.exists()).toBe(true);
  expect((input.element as HTMLTextAreaElement).disabled).toBe(false);

  await input.setValue("Architecture diagram");
  expect(updateAttributes).toHaveBeenCalledWith({ alt: "Architecture diagram" });

  wrapper.unmount();
});
