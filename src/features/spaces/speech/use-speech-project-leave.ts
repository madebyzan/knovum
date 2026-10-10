import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import { isApplicationLeaveCancelled } from "../../../shared/contracts/application-leave";
import type { SaveSpeechProjectRevisionRequest } from "../contracts/generated/SaveSpeechProjectRevisionRequest";
import { useUnsavedSessionParticipant } from "../workspace/use-unsaved-session-participant";

type RevisionInput = Omit<SaveSpeechProjectRevisionRequest, "projectId" | "expectedRevision">;

export function useSpeechProjectLeave(
  saveRevision: (input: RevisionInput) => Promise<boolean>,
  openProject: (projectId: string) => Promise<void>,
  discardDraft: () => void,
  settleRevisionSave: () => Promise<void>,
) {
  const { t } = useI18n();
  const dirty = shallowRef(false);
  const draft = shallowRef<RevisionInput>();
  const dialogTitle = computed(() =>
    draft.value ? t("spaces.speech.project.unsavedTitle") : t("spaces.speech.project.incompleteTitle"),
  );
  const dialogDescription = computed(() =>
    draft.value
      ? t("spaces.speech.project.unsavedDescription")
      : t("spaces.speech.project.incompleteDescription"),
  );
  const leave = useUnsavedSessionParticipant({
    dirty,
    beforeLeave: settleRevisionSave,
    discard: () => {
      dirty.value = false;
      draft.value = undefined;
      discardDraft();
    },
    save: async () => (draft.value ? saveRevision(draft.value) : false),
  });
  async function selectProject(projectId: string) {
    try {
      await leave.requestLeave();
    } catch (error) {
      if (isApplicationLeaveCancelled(error)) return;
      throw error;
    }
    await openProject(projectId);
  }
  function updateDraft(value: boolean, input?: RevisionInput) {
    dirty.value = value;
    draft.value = input;
  }
  return { draft, dialogTitle, dialogDescription, leave, selectProject, updateDraft };
}
