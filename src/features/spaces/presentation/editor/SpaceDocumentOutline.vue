<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import { useSpaceStore } from "../../session/space-store";
import SpaceContextHeader from "../SpaceContextHeader.vue";
import SpaceRightSidebarViewPicker from "../SpaceRightSidebarViewPicker.vue";

const store = useSpaceStore();
const { t } = useI18n();
const title = computed(
  () =>
    store.nodes.find((node) => node.id === store.focusedEditorId)?.data.title ??
    t("spaces.sidebar.editorOutline.title"),
);

function reveal(position: number) {
  store.documentOutlineRequest = {
    position,
    nonce: (store.documentOutlineRequest?.nonce ?? 0) + 1,
  };
}
</script>

<template>
  <section class="flex min-h-0 flex-col">
    <SpaceContextHeader :title="title">
      <template #leading><SpaceRightSidebarViewPicker /></template>
    </SpaceContextHeader>
    <nav
      class="min-h-0 flex-1 overflow-y-auto overscroll-contain px-2 py-3"
      :aria-label="t('spaces.sidebar.editorOutline.title')"
    >
      <button
        v-for="heading in store.documentOutlineHeadings"
        :key="heading.id"
        type="button"
        class="flex min-h-interaction w-full items-center rounded-md pr-2 text-left text-caption text-fg-secondary hover:bg-surface-control-hover hover:text-fg-primary focus-visible:ring-2 focus-visible:ring-stroke-focus data-[active=true]:bg-(--knovum-sidebar-item-bg-selected) data-[active=true]:text-fg-primary"
        :data-active="heading.id === store.documentActiveHeadingId"
        :style="{ paddingLeft: `${8 + (heading.level - 1) * 14}px` }"
        @click="reveal(heading.position)"
      >
        <span class="truncate">{{ heading.text }}</span>
      </button>
      <p v-if="!store.documentOutlineHeadings.length" class="px-2 py-4 text-caption text-fg-tertiary">
        {{ t("spaces.sidebar.editorOutline.empty") }}
      </p>
    </nav>
  </section>
</template>
