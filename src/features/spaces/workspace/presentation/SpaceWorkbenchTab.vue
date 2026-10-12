<script setup lang="ts">
import { FileText, Film, LayoutDashboard, X } from "@lucide/vue";
import { TabsTrigger } from "reka-ui";
import { useI18n } from "vue-i18n";

import type { WorkbenchTabView } from "./use-workbench-tab-views";

const props = defineProps<{ view: WorkbenchTabView }>();
const emit = defineEmits<{ close: []; contextMenu: [] }>();
const { t } = useI18n();
const ICONS = { space: LayoutDashboard, document: FileText, video: Film };
</script>

<template>
  <div
    :data-tab-id="props.view.id"
    :data-active="props.view.active"
    class="group flex h-(--knovum-control-height-sm) w-55 shrink-0 items-center rounded-md bg-(--knovum-tab-bg-inactive) text-label text-fg-secondary hover:bg-surface-raised hover:text-fg-primary hover:shadow-control data-[active=true]:bg-surface-raised data-[active=true]:font-medium data-[active=true]:text-fg-primary data-[active=true]:shadow-control"
    @auxclick.middle.prevent="props.view.canClose && emit('close')"
    @contextmenu.prevent="emit('contextMenu')"
    @keydown.shift.f10.prevent="emit('contextMenu')"
  >
    <TabsTrigger
      :value="props.view.id"
      class="flex h-full min-w-0 flex-1 items-center gap-1.5 px-3 text-left outline-none"
    >
      <component :is="ICONS[props.view.kind]" class="size-3.5 shrink-0" :stroke-width="1.6" />
      <span class="truncate">{{ props.view.title }}</span>
      <span v-if="props.view.dirty" class="sr-only">{{ t("common.unsaved") }}</span>
    </TabsTrigger>
    <!--
      The unsaved marker and the close button share one reserved slot, so the label keeps its
      width and never re-truncates while a document saves.
    -->
    <div class="relative mr-1.5 size-4 shrink-0">
      <span
        v-if="props.view.dirty"
        class="grid size-4 place-items-center leading-none group-hover:invisible"
        aria-hidden="true"
        >•</span
      >
      <button
        v-if="props.view.canClose"
        type="button"
        :aria-label="t('spaces.space.tabs.closeNamed', { title: props.view.title })"
        class="absolute inset-0 grid place-items-center rounded opacity-0 group-hover:opacity-100 hover:bg-surface-control-hover focus-visible:opacity-100"
        @click.stop="emit('close')"
      >
        <X class="size-3" :stroke-width="1.8" />
      </button>
    </div>
  </div>
</template>
