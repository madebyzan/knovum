<script setup lang="ts">
import { toRef } from "vue";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { useImageEditSession } from "../../image-editing/image-edit-session";
import { useUnsavedSessionParticipant } from "../../workspace/use-unsaved-session-participant";
import SpaceUnsavedChangesDialog from "../workspace/SpaceUnsavedChangesDialog.vue";

const session = useImageEditSession();
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
