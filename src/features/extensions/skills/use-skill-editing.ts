import { useMutation, useQuery } from "@tanstack/vue-query";
import { computed, onScopeDispose, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import { skillQuery } from "../../../foundation/extensions/extension-query";
import {
  readSkill,
  createSkill,
  saveSkillRevision,
  deleteSkill,
} from "../../../foundation/extensions/skill-operations";
import { useSettingsLeaveContext } from "../../../foundation/lifecycle/use-settings-leave-context";
import type { ExtensionChange } from "../../../shared/contracts/generated/ExtensionChange";
import type { SkillContent } from "../../../shared/contracts/generated/SkillContent";
import { confirmAction } from "../../../shared/ui/confirm-action";
import { useExtensionChanges } from "../use-extension-changes";

export function useSkillEditing() {
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
    () => editing.value && (markdown.value !== initialMarkdown.value || name.value !== initialName.value),
  );
  let pendingMutation: Promise<boolean> | undefined;
  function create() {
    selected.value = undefined;
    name.value = "";
    markdown.value = "---\nname: \ndescription: \n---\n\n";
    initialMarkdown.value = markdown.value;
    initialName.value = name.value;
    editing.value = true;
  }
  function mutate(operation: () => Promise<ExtensionChange<unknown> | void>) {
    pendingMutation = trackMutation(operation);
  }
  async function save() {
    pendingMutation = trackMutation(async () => {
      const change = selected.value
        ? await saveSkillRevision(selected.value.skill.id, selected.value.skill.digest, markdown.value)
        : await createSkill(name.value, markdown.value);
      editing.value = false;
      return change;
    });
    await pendingMutation;
  }
  const leaveParticipant = {
    async prepare() {
      if (pendingMutation && !(await pendingMutation)) return false;
      return !dirty.value || confirmDiscard();
    },
    finalize() {
      editing.value = false;
    },
  };
  onScopeDispose(useSettingsLeaveContext().register(leaveParticipant));

  async function closeEditor() {
    if (!(await leaveParticipant.prepare())) return false;
    leaveParticipant.finalize();
    return true;
  }
  async function deletePersonalSkill(skillId: string, skillName: string) {
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
    const completion: Promise<boolean> = changes
      .mutateAsync(operation)
      .then(
        () => true,
        () => false,
      )
      .finally(() => {
        if (pendingMutation === completion) pendingMutation = undefined;
      });
    return completion;
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
    save,
    closeEditor,
    deletePersonalSkill,
  };
}
