import { useMutation, useQuery } from "@tanstack/vue-query";
import { open } from "@tauri-apps/plugin-dialog";
import { computed, shallowRef } from "vue";
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
  function create() {
    selected.value = undefined;
    name.value = "";
    markdown.value = "---\nname: \ndescription: \n---\n\n";
    initialMarkdown.value = markdown.value;
    initialName.value = name.value;
    editing.value = true;
  }
  function mutate(operation: () => Promise<ExtensionChange<unknown> | void>) {
    void changes.mutateAsync(operation).catch(() => {});
  }
  async function save() {
    await changes
      .mutateAsync(async () => {
        const change = selected.value
          ? await saveSkillRevision(selected.value.skill.id, selected.value.skill.digest, markdown.value)
          : await createSkill(name.value, markdown.value);
        editing.value = false;
        return change;
      })
      .catch(() => {});
  }
  async function closeEditor() {
    if (changes.isPending.value) return;
    const dirty =
      !selected.value?.skill.pluginId &&
      (markdown.value !== initialMarkdown.value || name.value !== initialName.value);
    if (
      dirty &&
      !(await confirmAction({
        title: t("extensions.skills.discard.title"),
        description: t("extensions.skills.discard.description"),
        confirmLabel: t("common.discardChanges"),
        danger: true,
      }))
    )
      return;
    editing.value = false;
  }
  async function remove(id: string, skillName: string) {
    if (
      await confirmAction({
        title: t("extensions.skills.delete.title", { name: skillName }),
        description: t("extensions.skills.delete.description"),
        confirmLabel: t("common.delete"),
        danger: true,
      })
    )
      mutate(() => deleteSkill(id));
  }
  const busy = computed(() => changes.isPending.value || reading.isPending.value);

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
