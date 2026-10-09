<script setup lang="ts">
import { MessageCircle } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import UiDropdownMenuItem from "../../../../shared/ui/UiDropdownMenuItem.vue";
import type { ImageEditTool } from "../../image-editing/image-edit-session-store";
import { IMAGE_EDIT_TOOL_ORDER } from "./canvas-bar-density-policy";
import SpaceCanvasImageToolTrigger from "./SpaceCanvasImageToolTrigger.vue";

defineProps<{ dimensionsReady: boolean }>();
const emit = defineEmits<{ beginTool: [tool: ImageEditTool]; beginComment: [] }>();
const { t } = useI18n();
</script>

<template>
  <UiDropdownMenuItem @select="emit('beginComment')">
    <MessageCircle class="size-3.5" />{{ t("spaces.media.imageEditor.comments.title") }}
  </UiDropdownMenuItem>
  <SpaceCanvasImageToolTrigger
    v-for="mode in IMAGE_EDIT_TOOL_ORDER"
    :key="mode"
    :mode="mode"
    menu-item
    :disabled="mode === 'crop' && !dimensionsReady"
    @begin="emit('beginTool', $event)"
  />
</template>
