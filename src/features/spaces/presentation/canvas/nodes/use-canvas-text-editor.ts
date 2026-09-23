import { Editor } from "@tiptap/vue-3";
import { nextTick, onBeforeUnmount, shallowRef, watch, type Ref } from "vue";
import { toast } from "vue-sonner";

import {
  registerSpaceSessionParticipant,
  updateCanvasTextSelectionProjection,
} from "../../../canvas/editor-session";
import { createCanvasTextExtensions } from "../../../editor/tiptap-extensions";
import type { SpaceCanvasTextPayload } from "../../../model/nodes/capabilities";
import { tiptapContentSchema } from "../../../model/nodes/schema/content";
import { truncateUtf16 } from "../../../model/text";

interface CanvasTextEditorOptions {
  id: string;
  data: Readonly<Ref<SpaceCanvasTextPayload>>;
  interactive: Readonly<Ref<boolean>>;
  placeholder: string;
  defaultTitle: string;
  persist: (id: string, data: SpaceCanvasTextPayload) => Promise<void>;
}

export function useCanvasTextEditor(options: CanvasTextEditorOptions) {
  const dirty = shallowRef(false);
  const editing = shallowRef(false);
  const saving = shallowRef(false);
  const editor = shallowRef<Editor>();
  let detachCommit: (() => void) | undefined;
  let savePromise: Promise<void> | undefined;

  watch(options.interactive, (interactive) => (interactive ? void beginEditing() : void saveQuietly()));
  onBeforeUnmount(() => {
    updateCanvasTextSelectionProjection(options.id, false);
    editor.value?.destroy();
    editor.value = undefined;
    detachCommit?.();
  });

  async function beginEditing() {
    if (editing.value || !options.interactive.value) return;
    editing.value = true;
    editor.value = new Editor({
      content: options.data.value.tiptapContent,
      editable: true,
      extensions: createCanvasTextExtensions(options.placeholder),
      editorProps: {
        attributes: { class: "canvas-text-prose w-full outline-none", spellcheck: "true" },
      },
      onSelectionUpdate: ({ editor: instance }) => {
        const { from, to } = instance.state.selection;
        updateCanvasTextSelectionProjection(options.id, from !== to);
      },
      onUpdate: () => {
        dirty.value = true;
      },
      onBlur: () => {
        void saveQuietly();
      },
    });
    detachCommit = registerSpaceSessionParticipant({ commit: save });
    await nextTick();
    editor.value?.commands.focus("end");
  }

  async function save() {
    const instance = editor.value;
    if (savePromise) return savePromise;
    if (!instance || !editing.value) return;
    if (!dirty.value) {
      finishEditing();
      return;
    }
    instance.setEditable(false);
    savePromise = persist(instance, true);
    try {
      await savePromise;
    } finally {
      savePromise = undefined;
    }
  }

  async function saveQuietly() {
    await save().catch(() => undefined);
  }

  async function persist(instance: Editor, close: boolean) {
    const plainText = instance.state.doc.textBetween(0, instance.state.doc.content.size, "\n", "\n");
    const tiptapContent = tiptapContentSchema.parse(instance.getJSON());
    saving.value = true;
    try {
      const data: SpaceCanvasTextPayload = {
        ...options.data.value,
        title: truncateUtf16(plainText.trim().split(/\r?\n/u, 1)[0] ?? "", 80) || options.defaultTitle,
        plainText,
        tiptapContent,
      };
      await options.persist(options.id, data);
      dirty.value = false;
      if (close) finishEditing();
    } catch (error) {
      if (close && editor.value === instance) instance.setEditable(true);
      toast.error(String(error));
      throw error;
    } finally {
      saving.value = false;
    }
  }

  function finishEditing() {
    const instance = editor.value;
    editing.value = false;
    editor.value = undefined;
    updateCanvasTextSelectionProjection(options.id, false);
    detachCommit?.();
    detachCommit = undefined;
    instance?.destroy();
  }

  function cancelEditing() {
    dirty.value = false;
    finishEditing();
  }

  return { beginEditing, cancelEditing, editing, editor, save };
}
