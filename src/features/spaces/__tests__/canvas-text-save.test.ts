import { mount } from "@vue/test-utils";
import { afterEach, expect, it, vi } from "vitest";
import { defineComponent, h, shallowRef, type VNode } from "vue";

import { i18n } from "../../../foundation/i18n/instance";
import UiDropdownMenu from "../../../shared/ui/UiDropdownMenu.vue";
import UiDropdownMenuItem from "../../../shared/ui/UiDropdownMenuItem.vue";
import { getCanvasEditorSession } from "../canvas/editor-session";
import { CANVAS_TEXT_SURFACE_ATTRIBUTE } from "../canvas/presentation/nodes/canvas-text-surface";
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

it("ends the editing session when leaving on a failed save instead of stranding the canvas", async () => {
  const { interactive, session, wrapper } = createTextEditorHarness(() =>
    Promise.reject(new Error("Space session is transitioning")),
  );
  try {
    await session.beginEditing();
    session.editor.value!.commands.setContent("<p>Leaving with an unsaved draft</p>");
    expect(getCanvasEditorSession().activeTextEditorNodeId.value).toBe("text-node");

    interactive.value = false;
    await vi.waitUntil(() => !session.editing.value);

    expect(session.editor.value).toBeUndefined();
    expect(getCanvasEditorSession().activeTextEditorNodeId.value).toBeUndefined();
    expect(notices.error).toHaveBeenCalledOnce();
  } finally {
    wrapper.unmount();
  }
});

it("keeps editing while the selection bar's menu is used and ends once the user presses elsewhere", async () => {
  const persisted: unknown[] = [];
  const { session, wrapper } = createTextEditorHarness(
    async (_id, data) => void persisted.push(data),
    () =>
      h("div", { [CANVAS_TEXT_SURFACE_ATTRIBUTE]: "text-node" }, [
        h(
          UiDropdownMenu,
          { open: true, modal: false },
          {
            trigger: () => h("button", { type: "button" }, "Size"),
            default: () => h(UiDropdownMenuItem, () => "18 pt"),
          },
        ),
      ]),
  );
  try {
    await session.beginEditing();
    session.editor.value!.commands.setContent("<p>Styled</p>");
    const item = await vi.waitUntil(() => document.querySelector("[role='menuitem']"));

    item.dispatchEvent(new PointerEvent("pointerdown", { bubbles: true }));
    item.dispatchEvent(new FocusEvent("focusin", { bubbles: true }));
    expect(session.editing.value).toBe(true);
    expect(persisted).toEqual([]);

    document.body.dispatchEvent(new PointerEvent("pointerdown", { bubbles: true }));
    await vi.waitUntil(() => !session.editing.value);
    expect(persisted).toEqual([expect.objectContaining({ plainText: "Styled" })]);
  } finally {
    wrapper.unmount();
  }
});

function createTextEditorHarness(
  persist: (nodeId: string, payload: SpaceCanvasTextPayload) => Promise<void>,
  surface: () => VNode = () => h("div"),
) {
  let session!: ReturnType<typeof useCanvasTextEditor>;
  const interactive = shallowRef(true);
  const wrapper = mount(
    defineComponent({
      setup() {
        session = useCanvasTextEditor({
          id: "text-node",
          data: shallowRef(createSpaceTextPayload("Draft", "Draft")),
          interactive,
          persist,
        });
        return surface;
      },
    }),
    { attachTo: document.body, global: { plugins: [i18n] } },
  );
  return { interactive, session, wrapper };
}
