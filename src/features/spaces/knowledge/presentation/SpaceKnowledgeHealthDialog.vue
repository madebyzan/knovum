<script setup lang="ts">
import { useI18n } from "vue-i18n";

import UiDialog from "../../../../shared/ui/UiDialog.vue";
import SpaceKnowledgeHealth from "./SpaceKnowledgeHealth.vue";
import SpaceRetrievalIndex from "./SpaceRetrievalIndex.vue";

const open = defineModel<boolean>("open", { required: true });
const { t } = useI18n();
</script>

<template>
  <UiDialog
    v-model:open="open"
    :title="t('spaces.knowledge.health.title')"
    :description="t('spaces.knowledge.health.description')"
  >
    <div class="space-y-5">
      <SpaceRetrievalIndex />
      <div class="border-t border-stroke-divider pt-4">
        <SpaceKnowledgeHealth />
      </div>
    </div>
  </UiDialog>
</template>
