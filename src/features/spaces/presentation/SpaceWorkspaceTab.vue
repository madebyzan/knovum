<script setup lang="ts">
import { X } from "@lucide/vue";
import { TabsTrigger } from "reka-ui";
import { useI18n } from "vue-i18n";

defineProps<{ id: string; title: string; active: boolean; canClose: boolean }>();
const emit = defineEmits<{ close: []; contextMenu: [] }>();
const { t } = useI18n();
</script>

<template>
  <div
    :data-space-id="id"
    :data-active="active"
    class="group flex h-(--knovum-control-height-sm) w-55 shrink-0 items-center rounded-control bg-tab-inactive text-label text-foreground-secondary hover:bg-raised hover:text-foreground hover:shadow-control data-[active=true]:bg-raised data-[active=true]:font-medium data-[active=true]:text-foreground data-[active=true]:shadow-control"
    @auxclick.middle.prevent="canClose && emit('close')"
    @contextmenu.prevent="emit('contextMenu')"
    @keydown.shift.f10.prevent="emit('contextMenu')"
  >
    <TabsTrigger
      :value="id"
      class="flex h-full min-w-0 flex-1 items-center truncate px-3 text-left outline-none"
    >
      <span class="truncate">{{ title }}</span>
    </TabsTrigger>
    <button
      v-if="canClose"
      type="button"
      :aria-label="t('spaces.space.tabs.close')"
      class="mr-1.5 grid size-4 shrink-0 place-items-center rounded opacity-0 group-hover:opacity-100 hover:bg-control-pressed focus-visible:opacity-100"
      @click.stop="emit('close')"
    >
      <X class="size-(--knovum-icon-size-xs)" :stroke-width="1.8" />
    </button>
  </div>
</template>
