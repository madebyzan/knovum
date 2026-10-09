import { useMutation, useQuery } from "@tanstack/vue-query";
import { open } from "@tauri-apps/plugin-dialog";
import { computed, onScopeDispose, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import { skillQuery } from "../../../foundation/extensions/extension-query";
import {
  readSkill,
  createSkill,
  saveSkillRevision,
  importSkill,
  exportSkill,
  deleteSkill,
} from "../../../foundation/extensions/skill-operations";
import type { ExtensionChange } from "../../../shared/contracts/generated/ExtensionChange";
import type { SkillContent } from "../../../shared/contracts/generated/SkillContent";
import { useSettingsLeaveContext } from "../../../shared/contracts/settings-leave";
import { confirmAction } from "../../../shared/ui/confirm-action";
import { useExtensionChanges } from "../use-extension-changes";

async function importDirectory() {
  const path = await open({ directory: true, multiple: false });
  if (typeof path === "string") return importSkill(path);
}
async function exportDirectory(skillId: string) {
  const path = await open({ directory: true, multiple: false });
  if (typeof path === "string") await exportSkill(skillId, path);
}

export function useSkillManagement() {
  const { t } = useI18n();
  const query = useQuery(skillQuery);
  const changes = useExtensionChanges();
  const reading = useMutation({
    mutationFn: readSkill,
    onSuccess: (content) => {
      selected.value = content;
      name.value = content.skill.name;
      markdown.value = content.markdown;
      initialMarkdown.value = content.markdown;
      initialName.value = content.skill.name;
      editing.value = true;
    },
  });
  const selected = shallowRef<SkillContent>();
  const name = shallowRef("");
  const markdown = shallowRef("");
  const editing = shallowRef(false);
  const initialMarkdown = shallowRef("");
  const initialName = shallowRef("");
  const dirty = computed(
    () =>
      editing.value &&
      !selected.value?.skill.pluginId &&
      (markdown.value !== initialMarkdown.value || name.value !== initialName.value),
  );
  let admittedMutation: Promise<boolean> | undefined;
  function create() {
    selected.value = undefined;
    name.value = "";
    markdown.value = "---\nname: \ndescription: \n---\n\n";
    initialMarkdown.value = markdown.value;
    initialName.value = name.value;
    editing.value = true;
  }
  function mutate(operation: () => Promise<ExtensionChange<unknown> | void>) {
    admittedMutation = trackMutation(operation);
  }
  async function save() {
    admittedMutation = trackMutation(async () => {
      const change = selected.value
        ? await saveSkillRevision(selected.value.skill.id, selected.value.skill.digest, markdown.value)
        : await createSkill(name.value, markdown.value);
      editing.value = false;
      return change;
    });
    await admittedMutation;
  }
  async function closeEditor() {
    if (admittedMutation) {
      const succeeded = await admittedMutation;
      if (!succeeded) return false;
      if (!editing.value) return true;
    }
    if (dirty.value && !(await confirmDiscard())) return false;
    editing.value = false;
    return true;
  }
  const unregister = useSettingsLeaveContext().register({
    async prepare() {
      if (admittedMutation && !(await admittedMutation)) return false;
      return !dirty.value || confirmDiscard();
    },
    finalize() {
      editing.value = false;
    },
  });
  onScopeDispose(unregister);
  async function remove(skillId: string, skillName: string) {
    if (
      await confirmAction({
        title: t("extensions.skills.delete.title", { name: skillName }),
        description: t("extensions.skills.delete.description"),
        confirmLabel: t("common.delete"),
        danger: true,
      })
    )
      mutate(() => deleteSkill(skillId));
  }
  function confirmDiscard() {
    return confirmAction({
      title: t("extensions.skills.discard.title"),
      description: t("extensions.skills.discard.description"),
      confirmLabel: t("common.discardChanges"),
      danger: true,
    });
  }
  const busy = computed(() => changes.isPending.value || reading.isPending.value);

  function trackMutation(operation: () => Promise<ExtensionChange<unknown> | void>) {
    const admitted: Promise<boolean> = changes
      .mutateAsync(operation)
      .then(
        () => true,
        () => false,
      )
      .finally(() => {
        if (admittedMutation === admitted) admittedMutation = undefined;
      });
    return admitted;
  }

  return {
    query,
    changes,
    reading,
    selected,
    name,
    markdown,
    editing,
    busy,
    create,
    mutate,
    importDirectory,
    exportDirectory,
    save,
    closeEditor,
    remove,
  };
}
