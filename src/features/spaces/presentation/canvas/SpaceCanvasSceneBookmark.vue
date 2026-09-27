<script setup lang="ts">
import { Check, Copy, Ellipsis, Pencil, RefreshCw, Trash2, TriangleAlert, X } from "@lucide/vue";
import { computed, nextTick, shallowRef, useTemplateRef } from "vue";
import { useI18n } from "vue-i18n";

import UiDropdownMenu from "../../../../shared/ui/UiDropdownMenu.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import { spaceAssetSource } from "../../assets/node-preview-source";
import type { SpaceScene } from "../../scenes/scene";

const { active, invalid, recent, scene } = defineProps<{
  active: boolean;
  invalid: boolean;
  recent: boolean;
  scene: SpaceScene;
}>();
const emit = defineEmits<{
  duplicate: [scene: SpaceScene];
  open: [scene: SpaceScene];
  remove: [scene: SpaceScene];
  rename: [scene: SpaceScene, title: string];
  update: [scene: SpaceScene];
}>();
const { t } = useI18n();
const editing = shallowRef(false);
const title = shallowRef("");
const titleInput = useTemplateRef<InstanceType<typeof UiInput>>("titleInput");

async function beginRename() {
  title.value = scene.title;
  editing.value = true;
  await nextTick();
  titleInput.value?.focus();
}

function finishRename() {
  const value = title.value.trim();
  if (value) emit("rename", scene, value);
  editing.value = false;
}

const menuItems = computed(() => [
  { label: t("spaces.scenes.update"), icon: RefreshCw, onSelect: () => emit("update", scene) },
  ...(invalid
    ? []
    : [
        {
          label: t("spaces.scenes.duplicate"),
          icon: Copy,
          onSelect: () => emit("duplicate", scene),
        },
        { label: t("spaces.scenes.rename"), icon: Pencil, onSelect: beginRename },
      ]),
  { separator: true },
  {
    label: t("spaces.scenes.delete"),
    icon: Trash2,
    destructive: true,
    onSelect: () => emit("remove", scene),
  },
]);
</script>

<template>
  <div
    :data-scene-id="scene.id"
    class="group relative mb-0.5 min-h-14 rounded-md transition-colors duration-100 ease-standard last:mb-0"
    :class="[
      active ? 'bg-surface-control' : 'hover:bg-surface-control-hover',
      recent ? 'scene-row-recent' : '',
    ]"
  >
    <div
      v-if="editing"
      class="grid min-h-14 grid-cols-[4.75rem_minmax(0,1fr)_3rem] items-center gap-2 px-1.5 py-1"
    >
      <img
        v-if="scene.thumbnailPath"
        :src="spaceAssetSource(scene.thumbnailPath)"
        alt=""
        class="h-10 w-18 rounded-sm border border-stroke-strong bg-surface-canvas object-cover"
        draggable="false"
      />
      <div v-else class="h-10 w-18 rounded-sm border border-stroke-strong bg-surface-canvas" />
      <UiInput
        ref="titleInput"
        v-model="title"
        class="h-7 w-full min-w-0"
        @keydown.enter="finishRename"
        @keydown.escape="editing = false"
      />
      <div class="flex items-center justify-self-end">
        <UiIconButton
          :label="t('common.save')"
          :tooltip="false"
          size="sm"
          class="size-6"
          @click="finishRename"
        >
          <Check class="size-3" />
        </UiIconButton>
        <UiIconButton
          :label="t('common.cancel')"
          :tooltip="false"
          size="sm"
          class="size-6"
          @click="editing = false"
        >
          <X class="size-3" />
        </UiIconButton>
      </div>
    </div>

    <button
      v-else
      type="button"
      class="grid min-h-14 w-full grid-cols-[4.75rem_minmax(0,1fr)] items-center gap-2 rounded-md px-1.5 py-1 pr-8 text-left outline-none focus-visible:ring-2 focus-visible:ring-stroke-focus focus-visible:ring-inset"
      :aria-current="active ? 'true' : undefined"
      :aria-disabled="invalid"
      :disabled="invalid"
      :title="invalid ? t('spaces.scenes.focus.unavailable') : undefined"
      @click="emit('open', scene)"
      @dblclick.stop="beginRename"
    >
      <img
        v-if="scene.thumbnailPath"
        :src="spaceAssetSource(scene.thumbnailPath)"
        alt=""
        class="h-10 w-18 rounded-sm border border-stroke-strong bg-surface-canvas object-cover"
        draggable="false"
      />
      <span v-else class="h-10 w-18 rounded-sm border border-stroke-strong bg-surface-canvas" />
      <span class="flex min-w-0 items-center gap-1.5">
        <span class="min-w-0 flex-1 truncate text-label-md font-medium text-fg-primary">{{
          scene.title
        }}</span>
        <TriangleAlert
          v-if="invalid"
          class="size-3 shrink-0 text-fg-warning"
          :aria-label="t('spaces.scenes.focus.unavailable')"
        />
      </span>
    </button>

    <UiDropdownMenu v-if="!editing" side="right" align="start" :items="menuItems">
      <template #trigger>
        <UiIconButton
          :label="t('common.more')"
          :tooltip="false"
          size="sm"
          class="absolute top-1/2 right-1 z-10 size-6 -translate-y-1/2 rounded-sm opacity-0 group-focus-within:opacity-100 group-hover:opacity-100 data-[state=open]:bg-surface-control data-[state=open]:opacity-100"
          @click.stop
        >
          <Ellipsis class="size-3" />
        </UiIconButton>
      </template>
    </UiDropdownMenu>
  </div>
</template>

<style scoped>
.scene-row-recent {
  animation: scene-row-arrive 420ms var(--ease-standard);
}

@keyframes scene-row-arrive {
  0% {
    opacity: 0;
    transform: translateY(-5px) scale(0.985);
    background: var(--color-surface-control-hover);
  }
  55% {
    opacity: 1;
    transform: translateY(0) scale(1);
    background: var(--color-surface-control-hover);
  }
}

@media (prefers-reduced-motion: reduce) {
  .scene-row-recent {
    animation: none;
  }
}
</style>
