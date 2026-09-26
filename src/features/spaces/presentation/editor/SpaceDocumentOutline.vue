<script setup lang="ts">
import { useI18n } from "vue-i18n";

import { useSpaceStore } from "../../session/space-store";
import SpaceContextHeader from "../SpaceContextHeader.vue";

const store = useSpaceStore();
const { t } = useI18n();

function reveal(position: number) {
  store.documentOutlineRequest = {
    position,
    nonce: (store.documentOutlineRequest?.nonce ?? 0) + 1,
  };
}
</script>

<template>
  <section class="flex min-h-0 flex-col">
    <SpaceContextHeader :title="t('spaces.sidebar.editorOutline.title')" />
    <nav
      class="min-h-0 flex-1 overflow-y-auto overscroll-contain px-2 py-3"
      :aria-label="t('spaces.sidebar.editorOutline.title')"
    >
      <button
        v-for="heading in store.documentOutlineHeadings"
        :key="heading.id"
        type="button"
        class="flex min-h-(--knovum-compact-item-height) w-full items-center rounded-control pr-2 text-left text-caption text-foreground-secondary hover:bg-control-hover hover:text-foreground focus-visible:ring-2 focus-visible:ring-focus-ring data-[active=true]:bg-(--knovum-sidebar-item-background-selected) data-[active=true]:text-foreground"
        :data-active="heading.id === store.documentActiveHeadingId"
        :style="{ paddingLeft: `${8 + (heading.level - 1) * 14}px` }"
        @click="reveal(heading.position)"
      >
        <span class="truncate">{{ heading.text }}</span>
      </button>
      <p v-if="!store.documentOutlineHeadings.length" class="px-2 py-4 text-caption text-foreground-tertiary">
        {{ t("spaces.sidebar.editorOutline.empty") }}
      </p>
    </nav>
  </section>
</template>
