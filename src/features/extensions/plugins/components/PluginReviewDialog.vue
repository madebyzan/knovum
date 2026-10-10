<script setup lang="ts">
import { getErrorMessage } from "../../../../foundation/errors/error-format";
import UiDialog from "../../../../shared/ui/UiDialog.vue";
import type { PluginInstallationProposal } from "../use-plugin-installations";
import PluginInstallationReview from "./PluginInstallationReview.vue";
import PluginRevisionChanges from "./PluginRevisionChanges.vue";
const review = defineModel<PluginInstallationProposal | undefined>({ required: true });
defineProps<{ busy: boolean; error: unknown }>();
defineEmits<{ commit: [] }>();
</script>
<template>
  <UiDialog
    :open="Boolean(review)"
    :title="
      $t(review?.previous ? 'extensions.plugins.update.title' : 'extensions.plugins.installation.title')
    "
    size="wide"
    :closable="!busy"
    @update:open="!busy && (review = undefined)"
  >
    <template v-if="review">
      <PluginRevisionChanges
        v-if="review.previous"
        :before="review.previous.revision"
        :after="review.inspection"
      />
      <PluginInstallationReview
        :inspection="review.inspection"
        :busy="busy"
        :is-update="Boolean(review.previous)"
        @commit="$emit('commit')"
        @cancel="review = undefined"
      />
      <p v-if="error" role="alert" class="mt-3 text-body text-fg-danger">
        {{ getErrorMessage(error) }}
      </p>
    </template>
  </UiDialog>
</template>
