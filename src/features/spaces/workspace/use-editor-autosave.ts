import { useDebounceFn } from "@vueuse/core";
import { computed, onBeforeUnmount, shallowRef, watch, type Ref, type WatchSource } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { registerSpaceSessionParticipant } from "./session-participants";
import type { SpaceEditorTab } from "./workbench-tab";

export type EditorSaveStatus = "saving" | "unsaved" | "saved";

interface EditorAutosaveOptions {
  tab: SpaceEditorTab;
  active: Readonly<Ref<boolean>>;
  /** Whether the session holds work its node does not store yet. */
  dirty: Readonly<Ref<boolean>>;
  /** Owned by the session, which ignores outside changes to its node while a write is in flight. */
  saving: Ref<boolean>;
  error: Readonly<Ref<"missing" | "conflict" | undefined>>;
  /** Edits that schedule the next write. */
  edits: WatchSource | readonly WatchSource[];
  /** Stores the session's current work once; rejects when it could not be stored. */
  write(): Promise<void>;
  suspend?(): void;
  resume?(): void;
}

/**
 * The save model of every tab editor: work is stored by itself a moment after the last edit, and
 * leaving the tab or its Space stores what remains before the transition continues. The person
 * never decides whether to keep an edit — undo is how an edit is taken back — so explicit actions
 * in an editor only make things (an export, a render), never persist the work itself.
 */
export function useEditorAutosave(options: EditorAutosaveOptions) {
  const { t } = useI18n();
  const failed = shallowRef(false);
  const suspended = shallowRef(false);
  let pending: Promise<void> | undefined;

  function persist(): Promise<void> {
    if (pending) return pending.then(() => (options.dirty.value ? persist() : undefined));
    if (!options.dirty.value) return Promise.resolve();
    if (options.error.value) return Promise.reject(new Error(t(`spaces.editor.tabs.${options.error.value}`)));
    options.saving.value = true;
    failed.value = false;
    pending = options
      .write()
      .catch((error: unknown) => {
        failed.value = true;
        throw error;
      })
      .finally(() => {
        options.saving.value = false;
        pending = undefined;
      });
    return pending;
  }

  const autosave = useDebounceFn(() => {
    void persist().catch((error: unknown) => toast.error(getErrorMessage(error)));
  }, 1000);
  watch(options.edits as WatchSource[], () => {
    if (options.active.value) autosave();
  });
  watch(options.active, (visible) => {
    if (!visible) autosave.cancel();
    else if (options.dirty.value) autosave();
  });

  const detach = registerSpaceSessionParticipant({
    editorOwner: { spaceId: options.tab.spaceId, tabId: options.tab.id, commitOnSwitch: true },
    commit: async () => {
      do {
        // oxlint-disable-next-line eslint/no-await-in-loop -- Drain edits admitted during the preceding write before leaving.
        await persist();
      } while (options.dirty.value);
    },
    suspend: () => {
      suspended.value = true;
      autosave.cancel();
      options.suspend?.();
    },
    resume: () => {
      suspended.value = false;
      options.resume?.();
    },
  });
  onBeforeUnmount(() => {
    detach();
    autosave.cancel();
  });

  return {
    persist,
    suspended,
    status: computed<EditorSaveStatus>(() =>
      options.saving.value ? "saving" : options.dirty.value || failed.value ? "unsaved" : "saved",
    ),
  };
}
