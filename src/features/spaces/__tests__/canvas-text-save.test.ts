import { mount } from "@vue/test-utils";
import { afterEach, expect, it, vi } from "vitest";
import { defineComponent, h, shallowRef } from "vue";

import { i18n } from "../../../foundation/i18n/instance";
import { useCanvasTextEditor } from "../canvas/presentation/nodes/use-canvas-text-editor";
import { createSpaceTextPayload } from "../document-editing/tiptap-content";
import type { SpaceCanvasTextPayload } from "../model/nodes/capabilities";

const notices = vi.hoisted(() => ({ error: vi.fn<(message: string) => void>() }));
vi.mock("vue-sonner", () => ({ toast: notices }));

afterEach(() => notices.error.mockReset());

it("preserves an editable draft when content validation fails and permits a corrected save", async () => {
  const persisted: unknown[] = [];
  const { session, wrapper } = createTextEditorHarness(async (_id, data) => void persisted.push(data));
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

it("uses the current locale for an empty title when saving after a language change", async () => {
  const previousLocale = i18n.global.locale.value;
  i18n.global.locale.value = "en";
  const persisted: unknown[] = [];
  const { session, wrapper } = createTextEditorHarness(async (_id, data) => void persisted.push(data));
  try {
    await session.beginEditing();
    session.editor.value!.commands.clearContent();
    i18n.global.locale.value = "zh-CN";
    await session.save();
    expect(persisted).toEqual([
      expect.objectContaining({ title: i18n.global.t("spaces.editor.text.new"), plainText: "" }),
    ]);
  } finally {
    wrapper.unmount();
    i18n.global.locale.value = previousLocale;
  }
});

function createTextEditorHarness(
  persist: (nodeId: string, payload: SpaceCanvasTextPayload) => Promise<void>,
) {
  let session!: ReturnType<typeof useCanvasTextEditor>;
  const wrapper = mount(
    defineComponent({
      setup() {
        session = useCanvasTextEditor({
          id: "text-node",
          data: shallowRef(createSpaceTextPayload("Draft", "Draft")),
          interactive: shallowRef(true),
          persist,
        });
        return () => h("div");
      },
    }),
    { global: { plugins: [i18n] } },
  );
  return { session, wrapper };
}
