<script setup lang="ts">
import { Plus } from "@lucide/vue";
import { nextTick, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import type { SpaceScene } from "../../scenes/scene";
import SpaceCanvasSceneItem from "./SpaceCanvasSceneItem.vue";

const { activeSceneId, recentSceneId, scenes } = defineProps<{
  activeSceneId?: string;
  recentSceneId?: string;
  scenes: readonly SpaceScene[];
}>();
const emit = defineEmits<{
  create: [];
  duplicate: [scene: SpaceScene];
  open: [scene: SpaceScene];
  remove: [scene: SpaceScene];
  rename: [scene: SpaceScene, title: string];
  update: [scene: SpaceScene];
}>();
const { t } = useI18n();
const sceneList = useTemplateRef<HTMLElement>("sceneList");

watch(
  () => recentSceneId,
  async (sceneId) => {
    if (!sceneId) return;
    await nextTick();
    sceneList.value?.querySelector<HTMLElement>(`[data-scene-id="${CSS.escape(sceneId)}"]`)?.scrollIntoView({
      block: "nearest",
    });
  },
);
</script>

<template>
  <section class="w-72 select-none">
    <header class="flex items-center justify-between gap-3 px-1.5 pt-0.5 pb-2">
      <p class="text-label font-semibold text-foreground">{{ t("spaces.navigation.scenes") }}</p>
      <UiIconButton
        :label="t('spaces.navigation.saveScene')"
        size="sm"
        class="size-6 rounded-grid-cell border border-border-strong"
        @click="emit('create')"
      >
        <Plus class="size-(--knovum-icon-size-xs)" :stroke-width="1.55" />
      </UiIconButton>
    </header>
    <div
      v-if="scenes.length"
      ref="sceneList"
      class="scene-list max-h-72 divide-y divide-border overflow-y-auto rounded-control border border-border-strong bg-control"
    >
      <SpaceCanvasSceneItem
        v-for="scene in scenes"
        :key="scene.id"
        :scene="scene"
        :active="activeSceneId === scene.id"
        :recent="recentSceneId === scene.id"
        @duplicate="emit('duplicate', $event)"
        @open="emit('open', $event)"
        @remove="emit('remove', $event)"
        @rename="(item, title) => emit('rename', item, title)"
        @update="emit('update', $event)"
      />
    </div>
    <p v-else class="px-2 py-5 text-center text-caption text-foreground-tertiary">
      {{ t("spaces.navigation.scenesEmpty") }}
    </p>
  </section>
</template>

<style scoped>
.scene-list {
  scrollbar-color: var(--knovum-color-border-hover) transparent;
  scrollbar-width: thin;
}

.scene-list::-webkit-scrollbar {
  width: 0.25rem;
}

.scene-list::-webkit-scrollbar-thumb {
  border-radius: var(--knovum-control-radius);
  background: var(--knovum-color-border-hover);
}
</style>
