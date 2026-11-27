import type { Editor } from "@tiptap/vue-3";
import { useDebounceFn } from "@vueuse/core";
import { computed, inject, nextTick, onBeforeUnmount, shallowRef, watch, type Ref } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { workspaceRightSidebarKey } from "../../../../shared/contracts/workspace-right-sidebar";
import { exportSpaceMarkdown } from "../../assets/export/text";
import { cleanSerializedMarkdown } from "../../assets/markdown-document";
import { createContextDock } from "../../canvas/use-context-dock";
import { registerDocumentEditCommands } from "../../commands/document-edit-commands";
import { unescapeCitationMarkers } from "../../knowledge/model/citation-markers";
import { useSpaceStore } from "../../session/space-store";
import type { SpaceTiptapWritingTools } from "../use-space-tiptap-writing-tools";

const VIEW_PREFS_KEY = "knovum.editor.viewPreferences";

interface EditorViewPreferences {
  readonly showToolbar: boolean;
  readonly fullWidth: boolean;
}

function readViewPreferences(): EditorViewPreferences {
  try {
    const raw = globalThis.localStorage?.getItem(VIEW_PREFS_KEY);
    if (!raw) return { showToolbar: false, fullWidth: false };
    const parsed = JSON.parse(raw) as Partial<EditorViewPreferences>;
    return {
      showToolbar: parsed.showToolbar === true,
      fullWidth: parsed.fullWidth === true,
    };
  } catch {
    return { showToolbar: false, fullWidth: false };
  }
}

function writeViewPreferences(prefs: EditorViewPreferences) {
  try {
    globalThis.localStorage?.setItem(VIEW_PREFS_KEY, JSON.stringify(prefs));
  } catch {
    // Disposable UI preference; ignore storage quota or private-mode restrictions.
  }
}

interface SurfaceDocument {
  readonly editor: Readonly<Ref<Editor | undefined>>;
  readonly editorNode: Readonly<Ref<{ readonly id: string; readonly data: { readonly kind: string } }>>;
  readonly title: Readonly<Ref<string>>;
  readonly error: Readonly<Ref<unknown>>;
  readonly suspended: Readonly<Ref<boolean>>;
  readonly headings: Readonly<Ref<readonly unknown[]>>;
  readonly persist: () => Promise<void>;
  readonly isGroundedMarkdownDocument: () => Promise<boolean>;
}

interface SurfaceOptions {
  readonly active: Readonly<Ref<boolean>>;
  readonly doc: SurfaceDocument;
  readonly writing: SpaceTiptapWritingTools;
  readonly onFind: () => void;
  readonly onFocusOutline: () => void;
}

export function useSpaceTiptapSurface(options: SurfaceOptions) {
  const { doc } = options;
  const store = useSpaceStore();
  const { t } = useI18n();
  const rightSidebar = inject(workspaceRightSidebarKey, undefined);
  const dock = rightSidebar ? createContextDock(rightSidebar) : undefined;

  const initialPrefs = readViewPreferences();
  const showToolbar = shallowRef(initialPrefs.showToolbar);
  const fullWidth = shallowRef(initialPrefs.fullWidth);
  const sourceMode = shallowRef(false);
  const sourceDraft = shallowRef("");
  let sourceSnapshot = "";

  watch([showToolbar, fullWidth], ([toolbar, wide]) => {
    writeViewPreferences({ showToolbar: toolbar, fullWidth: wide });
  });

  function syncSourceToEditor() {
    const instance = doc.editor.value;
    if (!sourceMode.value || !instance || instance.isDestroyed || sourceDraft.value === sourceSnapshot)
      return;
    instance.commands.setContent(sourceDraft.value, { contentType: "markdown" });
    sourceSnapshot = sourceDraft.value;
  }

  const syncSourceDebounced = useDebounceFn(() => syncSourceToEditor(), 500);

  watch(sourceDraft, () => {
    if (sourceMode.value && sourceDraft.value !== sourceSnapshot) void syncSourceDebounced();
  });

  watch(options.active, (visible) => {
    if (visible || !sourceMode.value) return;
    syncSourceDebounced.cancel();
    syncSourceToEditor();
    void doc.persist();
  });

  onBeforeUnmount(() => {
    syncSourceDebounced.cancel();
    syncSourceToEditor();
  });

  function toggleSourceMode() {
    const instance = doc.editor.value;
    if (!instance || doc.editorNode.value.data.kind !== "markdown") return;
    if (!sourceMode.value) {
      options.writing.discard();
      sourceDraft.value = cleanSerializedMarkdown(unescapeCitationMarkers(instance.getMarkdown()));
      sourceSnapshot = sourceDraft.value;
      sourceMode.value = true;
      return;
    }
    syncSourceDebounced.cancel();
    syncSourceToEditor();
    sourceMode.value = false;
  }

  async function persistDocument() {
    syncSourceDebounced.cancel();
    syncSourceToEditor();
    await doc.persist();
  }

  async function askInChat(quote?: string) {
    if (!dock || !store.activeSpaceId) return;
    await persistDocument();
    await store.openSpace(store.activeSpaceId);
    await nextTick();
    const trimmed = quote?.trim();
    dock.openAsk("selection", {
      nodeIds: [doc.editorNode.value.id],
      ...(trimmed ? { quote: trimmed } : {}),
    });
  }

  async function exportMarkdown() {
    const instance = doc.editor.value;
    if (!instance) return;
    try {
      await persistDocument();
      const stripCitations = await doc.isGroundedMarkdownDocument();
      const markdown = cleanSerializedMarkdown(unescapeCitationMarkers(instance.getMarkdown()));
      if (await exportSpaceMarkdown(doc.title.value, markdown, { stripCitations }))
        toast.success(t("spaces.importExport.export.completed"));
    } catch (caught) {
      toast.error(getErrorMessage(caught));
    }
  }

  watch(
    [doc.editor, options.active, doc.error, doc.suspended],
    ([instance], _previous, cleanup) => {
      if (!instance || !options.active.value || doc.error.value || doc.suspended.value) return;
      cleanup(
        registerDocumentEditCommands({
          canUndo: () => instance.can().undo(),
          canRedo: () => instance.can().redo(),
          undo: () => void instance.commands.undo(),
          redo: () => void instance.commands.redo(),
          find: options.onFind,
          hasOutline: () => doc.headings.value.length > 0 && !sourceMode.value,
          focusOutline: options.onFocusOutline,
          save: persistDocument,
        }),
      );
    },
    { immediate: true },
  );

  const readOnly = computed(() => Boolean(doc.error.value) || doc.suspended.value);
  const titlePlaceholder = computed(() =>
    doc.editorNode.value.data.kind === "note"
      ? t("spaces.editor.note.new")
      : t("spaces.editor.markdown.untitled"),
  );

  return {
    showToolbar,
    fullWidth,
    sourceMode,
    sourceDraft,
    readOnly,
    titlePlaceholder,
    toggleSourceMode,
    persistDocument,
    askInChat,
    exportMarkdown,
  };
}
