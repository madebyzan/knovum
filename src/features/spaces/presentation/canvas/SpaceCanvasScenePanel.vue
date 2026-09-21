<script setup lang="ts">
import { LoaderCircle, Plus } from "@lucide/vue";
import { nextTick, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import type { SpaceScene } from "../../scenes/scene";
import SpaceCanvasSceneRow from "./SpaceCanvasSceneRow.vue";

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
    <header class="flex h-9 items-center justify-between border-b border-divider px-2.5">
      <p class="text-label font-medium text-foreground">{{ t("spaces.navigation.scenes") }}</p>
      <UiIconButton
        :label="t('spaces.navigation.saveScene')"
        size="sm"
        class="size-6 rounded-grid-cell"
        :disabled="saving"
        :aria-busy="saving"
        @click="emit('create')"
      >
        <LoaderCircle v-if="saving" class="size-(--knovum-icon-size-sm) animate-spin" :stroke-width="1.6" />
        <Plus v-else class="size-(--knovum-icon-size-sm)" :stroke-width="1.6" />
      </UiIconButton>
    </header>
    <div v-if="scenes.length" ref="sceneList" class="scene-list max-h-80 overflow-y-auto p-1.5">
      <SpaceCanvasSceneRow
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
      <p class="text-caption leading-5 text-foreground-tertiary">
        {{ t("spaces.navigation.scenesEmpty") }}
      </p>
      <button
        type="button"
        class="mt-3 flex h-7 items-center gap-1.5 rounded-control border border-border-strong bg-control px-3 text-caption font-medium text-foreground shadow-control hover:bg-control-hover active:bg-control-pressed disabled:pointer-events-none disabled:opacity-50"
        :disabled="saving"
        :aria-busy="saving"
        @click="emit('create')"
      >
        <LoaderCircle v-if="saving" class="size-(--knovum-icon-size-xs) animate-spin" />
        {{ t("spaces.navigation.saveScene") }}
      </button>
    </div>
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
