<script setup lang="ts">
import { toRef } from "vue";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import SpaceUnsavedChangesDialog from "../../workspace/presentation/SpaceUnsavedChangesDialog.vue";
import { useUnsavedSessionParticipant } from "../../workspace/use-unsaved-session-participant";
import { useImageEditSessionStore } from "../image-edit-session-store";

const session = useImageEditSessionStore();
const leave = useUnsavedSessionParticipant({
  dirty: toRef(session, "dirty"),
  beforeLeave: session.settleApply,
  discard: session.cancel,
  leaveClean: session.cancel,
  save: async () => {
    try {
      await session.apply();
      return !session.dirty;
    } catch (error) {
      toast.error(getErrorMessage(error));
      return false;
    }
  },
});
</script>

<template>
  <SpaceUnsavedChangesDialog
    :open="leave.dialogOpen.value"
    :saving="leave.saving.value || session.applying"
    :title="$t('spaces.media.imageEditor.unsaved.title')"
    :description="$t('spaces.media.imageEditor.unsaved.description')"
    @cancel="leave.cancel"
    @discard="leave.discard"
    @save="leave.save"
  />
</template>
