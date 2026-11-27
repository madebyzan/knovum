<script setup lang="ts">
import { MessageCircle } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import UiDropdownMenuItem from "../../../../../shared/ui/UiDropdownMenuItem.vue";
import type { ImageEditTool } from "../../../image-editing/image-edit-session-store";
import { PRIMARY_IMAGE_EDIT_TOOLS, SECONDARY_IMAGE_EDIT_TOOLS } from "./image-edit-tool-order";
import SpaceCanvasImageToolTrigger from "./SpaceCanvasImageToolTrigger.vue";

defineProps<{ dimensionsReady: boolean; includePrimary?: boolean }>();
const emit = defineEmits<{ beginTool: [tool: ImageEditTool]; beginComment: [] }>();
const { t } = useI18n();
</script>

<template>
  <template v-if="includePrimary">
    <UiDropdownMenuItem @select="emit('beginComment')">
      <MessageCircle class="size-3.5" />{{ t("spaces.media.imageComments.title") }}
    </UiDropdownMenuItem>
    <SpaceCanvasImageToolTrigger
      v-for="mode in PRIMARY_IMAGE_EDIT_TOOLS"
      :key="mode"
      :mode="mode"
      menu-item
      :disabled="mode === 'crop' && !dimensionsReady"
      @begin="emit('beginTool', $event)"
    />
  </template>
  <SpaceCanvasImageToolTrigger
    v-for="mode in SECONDARY_IMAGE_EDIT_TOOLS"
    :key="mode"
    :mode="mode"
    menu-item
    @begin="emit('beginTool', $event)"
  />
</template>
