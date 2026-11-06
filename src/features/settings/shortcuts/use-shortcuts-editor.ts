import { search } from "fast-fuzzy";
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { appCommands } from "../../../foundation/commands/run-app-command";
import { getErrorMessage } from "../../../foundation/errors/error-format";
import { useShortcutsStore } from "../../../foundation/shortcuts/shortcut-store";
import type { CommandCategory } from "../../../shared/contracts/app-command";

const CATEGORIES: readonly CommandCategory[] = ["general", "navigation", "canvas", "editing"];

const CATEGORY_LABEL_KEYS: Readonly<Record<CommandCategory, string>> = {
  general: "shell.shortcuts.categories.general",
  navigation: "shell.shortcuts.categories.navigation",
  canvas: "shell.shortcuts.categories.canvas",
  editing: "shell.shortcuts.categories.editing",
};

export function useShortcutsEditor() {
  const { t } = useI18n();
  const shortcutsStore = useShortcutsStore();

  const query = shallowRef("");
  const recorderOpen = shallowRef(false);
  const recordingTarget = shallowRef<{
    id: string;
    title: string;
    currentChord?: string | undefined;
    isGlobal?: boolean | undefined;
  } | null>(null);

  const allCommands = computed(() => appCommands());

  const bindableCommands = computed(() => {
    return allCommands.value.filter((cmd) => cmd.shortcut && !cmd.editingIntent);
  });

  const filteredCommands = computed(() => {
    const needle = query.value.trim();
    if (!needle) return bindableCommands.value;
    return search(needle, [...bindableCommands.value], {
      keySelector: (cmd) => `${cmd.title} ${shortcutsStore.getEffectiveShortcut(cmd.id, cmd.shortcut)}`,
      threshold: 0.6,
    });
  });

  const groupedCommands = computed(() => {
    return CATEGORIES.map((cat) => ({
      category: cat,
      title: t(CATEGORY_LABEL_KEYS[cat]),
      commands: filteredCommands.value.filter((cmd) => (cmd.category ?? "general") === cat),
    })).filter((group) => group.commands.length > 0);
  });

  function openRecorder(id: string, title: string, currentChord?: string, isGlobal = false) {
    recordingTarget.value = { id, title, currentChord, isGlobal };
    recorderOpen.value = true;
  }

  async function handleSaveChord(chord: string) {
    if (!recordingTarget.value) return;
    const { id, isGlobal } = recordingTarget.value;
    try {
      if (isGlobal) {
        await shortcutsStore.setDictationShortcut(chord);
        if (shortcutsStore.dictationStatus?.registered) {
          toast.success(t("settings.shortcuts.saved"));
        } else {
          toast.warning(t("settings.shortcuts.conflictWarning"));
        }
      } else {
        await shortcutsStore.setCustomShortcut(id, chord);
        toast.success(t("settings.shortcuts.saved"));
      }
    } catch (error) {
      toast.error(getErrorMessage(error));
    }
  }

  async function handleReset(id: string, isGlobal = false) {
    try {
      if (isGlobal) {
        await shortcutsStore.resetDictationShortcut();
      } else {
        await shortcutsStore.resetCustomShortcut(id);
      }
      toast.success(t("settings.shortcuts.resetSuccess"));
    } catch (error) {
      toast.error(getErrorMessage(error));
    }
  }

  async function handleResetAll() {
    try {
      await shortcutsStore.resetAll();
      toast.success(t("settings.shortcuts.resetAllSuccess"));
    } catch (error) {
      toast.error(getErrorMessage(error));
    }
  }

  return {
    query,
    recorderOpen,
    recordingTarget,
    shortcutsStore,
    allCommands,
    groupedCommands,
    openRecorder,
    handleSaveChord,
    handleReset,
    handleResetAll,
  };
}
