<script setup lang="ts">
import { Tag } from "@lucide/vue";
import { ToolbarButton } from "reka-ui";
import { shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../../shared/ui/UiIconButton.vue";
import UiPopover from "../../../../../shared/ui/UiPopover.vue";
import SpaceTags from "../../../knowledge/presentation/SpaceTags.vue";

const open = shallowRef(false);
const { t } = useI18n();
</script>

<template>
  <UiPopover v-model:open="open" side="top" align="center" :side-offset="8" class="w-64">
    <template #trigger>
      <ToolbarButton as-child>
        <UiIconButton :label="t('spaces.tags.title')" tooltip>
          <Tag class="size-4" :stroke-width="1.6" />
        </UiIconButton>
      </ToolbarButton>
    </template>
    <p class="mb-2 text-caption-sm font-semibold text-fg-secondary">{{ t("spaces.tags.title") }}</p>
    <SpaceTags />
  </UiPopover>
</template>
