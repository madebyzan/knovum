<script setup lang="ts">
import { Box } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import SpaceCanvasActionTrigger from "../SpaceCanvasActionTrigger.vue";

defineProps<{ compact?: boolean; menuItem?: boolean }>();
const emit = defineEmits<{ begin: [] }>();
const { t } = useI18n();
</script>

<template>
  <SpaceCanvasActionTrigger
    :label="t('spaces.media.imageEditor.angle.title')"
    :icon="Box"
    :compact="compact"
    :menu-item="menuItem"
    @invoke="emit('begin')"
  />
</template>
