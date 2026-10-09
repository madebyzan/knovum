<script setup lang="ts">
import { Star } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import { spaceAssetSource } from "../../assets/node-preview-source";
import type { Space } from "../../model/space";

defineProps<{ space: Space; layout: "grid" | "list"; editing: boolean; tabIndex: number }>();
const draftTitle = defineModel<string>("draftTitle", { required: true });
const emit = defineEmits<{ open: []; menu: []; favorite: []; finish: []; cancel: [] }>();
const { d, t } = useI18n();
</script>
<template>
  <article
    class="group relative overflow-hidden bg-surface-muted"
    :class="layout === 'grid' ? 'rounded-panel border border-stroke-default' : 'flex items-center py-2'"
    @contextmenu.prevent="emit('menu')"
  >
    <button
      type="button"
      class="absolute inset-0 z-10 rounded-panel outline-none focus-visible:ring-2 focus-visible:ring-stroke-focus focus-visible:ring-inset"
      @click="emit('open')"
      :tabindex="tabIndex"
      :data-library-space-id="space.id"
      :aria-label="space.title"
    />
    <div
      class="shrink-0 overflow-hidden bg-surface-canvas"
      :class="layout === 'grid' ? 'aspect-video w-full' : 'h-16 w-28 rounded-md border border-stroke-default'"
    >
      <img
        v-if="space.thumbnailPath"
        :src="spaceAssetSource(space.thumbnailPath)"
        alt=""
        class="size-full object-cover"
      />
    </div>
    <div
      class="relative min-w-0 flex-1 px-3 py-2.5"
      :class="layout === 'grid' ? 'border-t border-stroke-divider' : ''"
    >
      <UiIconButton
        v-if="space.kind === 'space'"
        class="absolute top-1.5 right-1.5 z-20 opacity-0 group-focus-within:opacity-100 group-hover:opacity-100"
        size="sm"
        :active="space.favorite"
        :label="t(space.favorite ? 'spaces.space.unfavorite' : 'spaces.space.favorite')"
        @click="emit('favorite')"
      >
        <Star class="size-3.5" :fill="space.favorite ? 'currentColor' : 'none'" />
      </UiIconButton>
      <UiInput
        v-if="editing"
        v-model="draftTitle"
        data-library-title-editor
        size="sm"
        class="relative z-20 w-full"
        @click.stop
        @blur="emit('finish')"
        @keydown.enter.prevent="emit('finish')"
        @keydown.escape.prevent="emit('cancel')"
      />
      <h2 v-else class="truncate pr-7 text-label font-semibold text-fg-primary">
        {{ space.title }}
      </h2>
      <p class="mt-0.5 text-caption text-fg-tertiary">
        {{ t("spaces.space.nodeCount", { count: space.nodeCount }) }} ·
        {{ d(space.updatedAt, "medium") }}
      </p>
    </div>
  </article>
</template>
