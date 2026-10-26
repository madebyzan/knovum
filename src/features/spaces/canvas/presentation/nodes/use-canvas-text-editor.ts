import { Editor } from "@tiptap/vue-3";
import { nextTick, onBeforeUnmount, shallowRef, watch, type Ref } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../../foundation/errors/error-format";
import { createCanvasTextExtensions } from "../../../document-editing/tiptap-extensions";
import type { SpaceCanvasTextPayload } from "../../../model/nodes/capabilities";
import { tiptapContentSchema } from "../../../model/nodes/schema/content";
import { truncateUtf16 } from "../../../model/text";
import { registerSpaceSessionParticipant } from "../../../workspace/session-participants";
import { activateCanvasTextEditor, deactivateCanvasTextEditor } from "../../editor-session";
import { isWithinCanvasTextSurface } from "./canvas-text-surface";

interface CanvasTextEditorOptions {
  id: string;
  data: Readonly<Ref<SpaceCanvasTextPayload>>;
  interactive: Readonly<Ref<boolean>>;
  persist: (nodeId: string, payload: SpaceCanvasTextPayload) => Promise<void>;
}

export function useCanvasTextEditor(options: CanvasTextEditorOptions) {
  const { t } = useI18n();
  const dirty = shallowRef(false);
  const editing = shallowRef(false);
  const editor = shallowRef<Editor>();
  let detachCommit: (() => void) | undefined;
  let detachPresence: (() => void) | undefined;
  let savePromise: Promise<void> | undefined;

  watch(options.interactive, (interactive) => (interactive ? void beginEditing() : void leaveEditing()));
  onBeforeUnmount(() => {
    deactivateCanvasTextEditor(options.id);
    editor.value?.destroy();
    editor.value = undefined;
    detachCommit?.();
    detachPresence?.();
  });

  async function beginEditing() {
    if (editing.value || !options.interactive.value) return;
    editing.value = true;
    activateCanvasTextEditor(options.id);
    editor.value = new Editor({
      content: options.data.value.tiptapContent,
      editable: true,
      extensions: createCanvasTextExtensions(
        options.data.value.kind === "note" ? t("spaces.editor.note.placeholder") : "",
        options.data.value.kind,
      ),
      editorProps: {
        attributes: { class: "canvas-text-prose w-full outline-none", spellcheck: "true" },
      },
      onUpdate: () => {
        dirty.value = true;
      },
    });
    detachCommit = registerSpaceSessionParticipant({ commit: save });
    detachPresence = watchPresence();
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
    savePromise = persist(instance);
    try {
      await savePromise;
      finishEditing();
    } catch (error) {
      // The caller is still inside the editor, so a rejected save hands the draft back for a
      // corrected attempt instead of discarding what was typed.
      if (editor.value === instance) instance.setEditable(true);
      throw error;
    } finally {
      savePromise = undefined;
    }
  }

  /**
   * Leaving the editor is not a save decision. The session owns canvas-wide state — the node
   * operation gate and a contenteditable laid over the scene — so it ends with the attempt whatever
   * the attempt reported. A session that outlives the user's presence leaves the canvas looking
   * normal while selection, structure edits and the canvas context menu are all refused in silence.
   */
  async function leaveEditing() {
    try {
      await save();
    } catch {
      // persist() reported it; the session still ends here.
    } finally {
      finishEditing();
    }
  }

  /**
   * The session lasts while the user is on its surface. Editor blur cannot tell: the selection bar
   * and the menus it opens take focus while they are used, and switching to another app blurs the
   * editor without the user leaving it. A press or a focus that lands off the surface does.
   */
  function leaveWhenOutside(event: Event) {
    if (!isWithinCanvasTextSurface(options.id, event.target)) void leaveEditing();
  }

  function watchPresence() {
    document.addEventListener("pointerdown", leaveWhenOutside, true);
    document.addEventListener("focusin", leaveWhenOutside);
    return () => {
      document.removeEventListener("pointerdown", leaveWhenOutside, true);
      document.removeEventListener("focusin", leaveWhenOutside);
    };
  }

  async function persist(instance: Editor) {
    try {
      const plainText = instance.state.doc.textBetween(0, instance.state.doc.content.size, "\n", "\n");
      const tiptapContent = tiptapContentSchema.parse(instance.getJSON());
      const data: SpaceCanvasTextPayload = {
        ...options.data.value,
        title:
          truncateUtf16(plainText.trim().split(/\r?\n/u, 1)[0] ?? "", 80) ||
          t(`spaces.editor.${options.data.value.kind}.new`),
        plainText,
        tiptapContent,
      };
      await options.persist(options.id, data);
    } catch (error) {
      toast.error(getErrorMessage(error));
      throw error;
    }
  }

  function finishEditing() {
    const instance = editor.value;
    editing.value = false;
    dirty.value = false;
    deactivateCanvasTextEditor(options.id);
    editor.value = undefined;
    detachCommit?.();
    detachCommit = undefined;
    detachPresence?.();
    detachPresence = undefined;
    instance?.destroy();
  }

  /** Escape discards the draft; the session ends the same way every other exit ends. */
  return { beginEditing, cancelEditing: finishEditing, editing, editor, save };
}
