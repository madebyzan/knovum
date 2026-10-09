import { mount } from "@vue/test-utils";
import { afterEach, expect, it, vi } from "vitest";
import { defineComponent, h, shallowRef } from "vue";

import { createSpaceTextPayload } from "../editor/tiptap-content";
import { useCanvasTextEditor } from "../presentation/canvas/nodes/use-canvas-text-editor";

const notices = vi.hoisted(() => ({ error: vi.fn<(message: string) => void>() }));
vi.mock("vue-sonner", () => ({ toast: notices }));

afterEach(() => notices.error.mockReset());

it("preserves an editable draft when content validation fails and permits a corrected save", async () => {
  const persisted: unknown[] = [];
  let session!: ReturnType<typeof useCanvasTextEditor>;
  const wrapper = mount(
    defineComponent({
      setup() {
        session = useCanvasTextEditor({
          id: "text-node",
          data: shallowRef(createSpaceTextPayload("Draft", "Draft")),
          interactive: shallowRef(true),
          placeholder: "",
          defaultTitle: "Text",
          persist: async (_id, data) => void persisted.push(data),
        });
        return () => h("div");
      },
    }),
  );
  try {
    await session.beginEditing();
    const instance = session.editor.value!;
    const oversized = "x".repeat(4 * 1024 * 1024 + 1);
    instance.commands.setContent({
      type: "doc",
      content: [{ type: "paragraph", content: [{ type: "text", text: oversized }] }],
    });

    await expect(session.save()).rejects.toThrow(/Too big/u);

    expect(session.editing.value).toBe(true);
    expect(instance.isEditable).toBe(true);
    expect(instance.getText()).toBe(oversized);
    expect(persisted).toEqual([]);
    expect(notices.error).toHaveBeenCalledOnce();

    instance.commands.setContent("<p>Corrected draft</p>");
    await session.save();
    expect(persisted).toEqual([
      expect.objectContaining({ title: "Corrected draft", plainText: "Corrected draft" }),
    ]);
    expect(session.editing.value).toBe(false);
    expect(session.editor.value).toBeUndefined();
  } finally {
    wrapper.unmount();
  }
});
