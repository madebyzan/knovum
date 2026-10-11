<script setup lang="ts">
import { MessageCircle, Pencil, ScanLine, X } from "@lucide/vue";
import { ToolbarButton, ToolbarRoot, ToolbarSeparator } from "reka-ui";

import UiIconButton from "../../../../../shared/ui/UiIconButton.vue";
import { useImageCommentSessionStore } from "../../../comments/image-comment-session-store";
import { CANVAS_ACTION_BAR_CLASSES } from "../canvas-action-classes";

const session = useImageCommentSessionStore();
const MODES = [
  { id: "point", icon: MessageCircle, label: "spaces.media.imageComments.point" },
  { id: "draw", icon: Pencil, label: "spaces.media.imageComments.draw" },
  { id: "region", icon: ScanLine, label: "spaces.media.imageComments.region" },
] as const;
</script>

<template>
  <ToolbarRoot
    :class="CANVAS_ACTION_BAR_CLASSES"
    orientation="horizontal"
    :loop="false"
    :aria-label="$t('spaces.media.imageComments.mode')"
  >
    <span class="px-2 text-label font-medium text-fg-primary">{{
      $t("spaces.media.imageComments.title")
    }}</span>
    <ToolbarSeparator class="bg-border-divider mx-0.5 h-4 w-px" />
    <ToolbarButton v-for="item in MODES" :key="item.id" as-child>
      <UiIconButton
        :label="$t(item.label)"
        :active="session.mode === item.id"
        tooltip
        size="sm"
        class="data-[active=true]:bg-surface-selected data-[active=true]:text-fg-accent"
        @click.stop="session.setMode(item.id)"
      >
        <component :is="item.icon" class="size-3.5" :stroke-width="1.55" />
      </UiIconButton>
    </ToolbarButton>
    <ToolbarSeparator class="bg-border-divider mx-0.5 h-4 w-px" />
    <ToolbarButton as-child>
      <UiIconButton :label="$t('common.cancel')" size="sm" @click.stop="session.cancel()">
        <X class="size-3.5" :stroke-width="1.55" />
      </UiIconButton>
    </ToolbarButton>
  </ToolbarRoot>
</template>
