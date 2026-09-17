<script setup lang="ts">
import { Check, Copy, Ellipsis, Pencil, RefreshCw, Trash2, X } from "@lucide/vue";
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import UiDropdownMenu from "../../../../shared/ui/UiDropdownMenu.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import { spaceAssetSource } from "../../assets/node-preview-source";
import type { SpaceScene } from "../../scenes/scene";

const { active, recent, scene } = defineProps<{
  active: boolean;
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

function beginRename() {
  title.value = scene.title;
  editing.value = true;
}

function finishRename() {
  const value = title.value.trim();
  if (value) emit("rename", scene, value);
  editing.value = false;
}

const menuItems = computed(() => [
  { label: t("spaces.navigation.updateScene"), icon: RefreshCw, onSelect: () => emit("update", scene) },
  { label: t("spaces.navigation.duplicateScene"), icon: Copy, onSelect: () => emit("duplicate", scene) },
  { label: t("spaces.navigation.renameScene"), icon: Pencil, onSelect: beginRename },
  { separator: true },
  {
    label: t("spaces.navigation.deleteScene"),
    icon: Trash2,
    destructive: true,
    onSelect: () => emit("remove", scene),
  },
]);
</script>

<template>
  <div
    :data-scene-id="scene.id"
    class="group grid min-h-13 items-center px-2 transition-colors duration-100 ease-standard"
    :class="[
      editing ? 'grid-cols-[4rem_minmax(0,1fr)_3.5rem]' : 'grid-cols-[4rem_minmax(0,1fr)_1.75rem]',
      active ? 'bg-control-pressed text-foreground' : 'text-foreground-secondary',
      recent
        ? 'animate-in duration-150 ease-standard fade-in-0 slide-in-from-top-1 motion-reduce:animate-none'
        : '',
    ]"
  >
    <img
      v-if="scene.thumbnailPath"
      :src="spaceAssetSource(scene.thumbnailPath)"
      alt=""
      class="h-9 w-14 rounded-grid-cell border border-border-strong bg-canvas object-cover"
      draggable="false"
    />
    <div v-else class="h-9 w-14 rounded-grid-cell border border-border-strong bg-canvas" />
    <div v-if="editing" class="min-w-0 pr-2">
      <UiInput
        v-model="title"
        class="h-(--knovum-control-height-sm) w-full min-w-0"
        @keydown.enter="finishRename"
        @keydown.escape="editing = false"
      />
    </div>
    <button
      v-else
      type="button"
      class="min-w-0 self-stretch text-left outline-none focus-visible:text-foreground"
      @click="emit('open', scene)"
    >
      <span class="flex h-full items-center truncate text-label font-medium text-foreground">
        {{ scene.title }}
      </span>
    </button>
    <div v-if="editing" class="flex items-center gap-1 justify-self-end pr-1">
      <UiIconButton :label="t('common.save')" :tooltip="false" size="sm" class="size-6" @click="finishRename">
        <Check class="size-(--knovum-icon-size-xs)" />
      </UiIconButton>
      <UiIconButton
        :label="t('common.cancel')"
        :tooltip="false"
        size="sm"
        class="size-6"
        @click="editing = false"
      >
        <X class="size-(--knovum-icon-size-xs)" />
      </UiIconButton>
    </div>
    <UiDropdownMenu v-else side="right" align="start" :items="menuItems">
      <template #trigger>
        <UiIconButton
          :label="t('common.more')"
          :tooltip="false"
          size="sm"
          class="size-6 rounded-grid-cell border border-transparent opacity-0 group-focus-within:opacity-100 group-hover:opacity-100 hover:border-border"
        >
          <Ellipsis class="size-(--knovum-icon-size-xs)" />
        </UiIconButton>
      </template>
    </UiDropdownMenu>
  </div>
</template>
