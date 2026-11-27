<script setup lang="ts">
import { MessageCircle } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import UiSeparator from "../../../../../shared/ui/UiSeparator.vue";
import type { ImageEditTool } from "../../../image-editing/image-edit-session-store";
import SpaceCanvasActionTrigger from "../SpaceCanvasActionTrigger.vue";
import { PRIMARY_IMAGE_EDIT_TOOLS } from "./image-edit-tool-order";
import SpaceCanvasImageToolTrigger from "./SpaceCanvasImageToolTrigger.vue";

defineProps<{ compact: boolean; dimensionsReady: boolean }>();
const emit = defineEmits<{ beginTool: [tool: ImageEditTool]; beginComment: [] }>();
const { t } = useI18n();
</script>

<template>
  <SpaceCanvasActionTrigger
    :label="t('spaces.media.imageComments.title')"
    :icon="MessageCircle"
    :compact="compact"
    @invoke="emit('beginComment')"
  />
  <SpaceCanvasImageToolTrigger
    v-for="mode in PRIMARY_IMAGE_EDIT_TOOLS"
    :key="mode"
    :mode="mode"
    :compact="compact"
    :disabled="mode === 'crop' && !dimensionsReady"
    @begin="emit('beginTool', $event)"
  />
  <UiSeparator orientation="vertical" class="mx-0.5 h-4" />
</template>
