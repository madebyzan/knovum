<script setup lang="ts">
import { LoaderCircle, Plus } from "@lucide/vue";
import { nextTick, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import type { SpaceScene } from "../../scenes/scene";
import SpaceCanvasSceneBookmark from "./SpaceCanvasSceneBookmark.vue";

const { activeSceneId, recentSceneId, saving, scenes } = defineProps<{
  activeSceneId: string | undefined;
  recentSceneId: string | undefined;
  saving: boolean;
  scenes: readonly SpaceScene[];
  availableStructureNodeIds: ReadonlySet<string>;
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
  <section class="w-68 overflow-hidden rounded-[inherit] select-none">
    <header class="flex h-9 items-center justify-between border-b border-stroke-divider px-2.5">
      <p class="text-label font-medium text-fg-primary">{{ t("spaces.scenes.title") }}</p>
      <UiIconButton
        :label="t('spaces.scenes.save')"
        size="sm"
        class="size-6 rounded-sm"
        :disabled="saving"
        :aria-busy="saving"
        @click="emit('create')"
      >
        <LoaderCircle v-if="saving" class="size-3.5 animate-spin" :stroke-width="1.6" />
        <Plus v-else class="size-3.5" :stroke-width="1.6" />
      </UiIconButton>
    </header>
    <div v-if="scenes.length" ref="sceneList" class="scene-list max-h-80 overflow-y-auto p-1.5">
      <SpaceCanvasSceneBookmark
        v-for="scene in scenes"
        :key="scene.id"
        :scene="scene"
        :invalid="scene.focusStructureNodeIds.some((id) => !availableStructureNodeIds.has(id))"
        :active="activeSceneId === scene.id"
        :recent="recentSceneId === scene.id"
        @duplicate="emit('duplicate', $event)"
        @open="emit('open', $event)"
        @remove="emit('remove', $event)"
        @rename="(scene, title) => emit('rename', scene, title)"
        @update="emit('update', $event)"
      />
    </div>
    <div v-else class="flex flex-col items-center px-5 py-5 text-center">
      <p class="text-caption leading-5 text-fg-tertiary">
        {{ t("spaces.scenes.empty") }}
      </p>
      <button
        type="button"
        class="mt-3 flex h-7 items-center gap-1.5 rounded-md border border-stroke-strong bg-surface-control px-3 text-caption font-medium text-fg-primary shadow-control hover:bg-surface-control-hover active:bg-surface-control-pressed disabled:pointer-events-none disabled:opacity-40"
        :disabled="saving"
        :aria-busy="saving"
        @click="emit('create')"
      >
        <LoaderCircle v-if="saving" class="size-3 animate-spin" />
        {{ t("spaces.scenes.save") }}
      </button>
    </div>
  </section>
</template>

<style scoped>
.scene-list {
  scrollbar-color: var(--color-stroke-default-hover) transparent;
  scrollbar-width: thin;
}

.scene-list::-webkit-scrollbar {
  width: 0.25rem;
}

.scene-list::-webkit-scrollbar-thumb {
  border-radius: var(--radius-md);
  background: var(--color-stroke-default-hover);
}
</style>
