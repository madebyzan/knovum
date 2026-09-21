<script setup lang="ts">
import { error as logError } from "@tauri-apps/plugin-log";
import { nextTick, shallowRef, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { nodeIcon } from "../node-icon";
import SpaceOutlineTitle from "./SpaceOutlineTitle.vue";

const props = defineProps<{
  node: SpaceCanvasNode;
  selected: boolean;
  editing: boolean;
  saveName: (title: string) => Promise<void>;
}>();
const emit = defineEmits<{ reveal: []; open: []; menu: []; finish: [] }>();
const { t } = useI18n();
const input = useTemplateRef<HTMLInputElement>("input");
const button = useTemplateRef<HTMLButtonElement>("button");
const draft = shallowRef("");
const saving = shallowRef(false);
const error = shallowRef("");

watch(
  () => props.editing,
  async (editing) => {
    if (!editing) return;
    draft.value = props.node.data.title;
    error.value = "";
    await nextTick();
    if (editing !== props.editing) return;
    input.value?.scrollIntoView({ block: "nearest" });
    input.value?.focus({ preventScroll: true });
    input.value?.select();
  },
  { immediate: true },
);

async function save() {
  if (saving.value) return;
  const name = draft.value.trim();
  if (!name) {
    error.value = t("spaces.navigation.nodeNameRequired");
    return;
  }
  saving.value = true;
  error.value = "";
  try {
    await props.saveName(name);
    await finish(true);
  } catch (cause) {
    error.value = t("spaces.navigation.nodeRenameFailed");
    void logError(`Space node rename failed: ${String(cause)}`);
    await nextTick();
    input.value?.focus({ preventScroll: true });
  } finally {
    saving.value = false;
  }
}

async function finish(restoreFocus: boolean) {
  if (!props.editing) return;
  emit("finish");
  if (restoreFocus) {
    await nextTick();
    button.value?.focus({ preventScroll: true });
  }
}

function cancel(restoreFocus = false) {
  if (!saving.value) void finish(restoreFocus);
}

function keydown(event: KeyboardEvent) {
  event.stopPropagation();
  if (event.isComposing) return;
  if (event.key === "Enter") {
    event.preventDefault();
    void save();
  } else if (event.key === "Escape") {
    event.preventDefault();
    cancel(true);
  }
}
</script>

<template>
  <div class="min-w-0" :data-active="selected">
    <div v-if="editing" class="flex h-(--knovum-compact-item-height) items-center gap-2">
      <component
        v-if="node.data.kind !== 'section'"
        :is="nodeIcon(node.data.kind)"
        class="size-(--knovum-icon-size-xs) shrink-0"
      />
      <input
        ref="input"
        v-model="draft"
        type="text"
        maxlength="512"
        :readonly="saving"
        :aria-label="t('spaces.navigation.renameNode')"
        :aria-invalid="Boolean(error)"
        :aria-busy="saving"
        class="h-6 min-w-0 flex-1 rounded-control bg-control px-1 text-caption text-foreground ring-1 ring-focus-ring outline-none"
        @keydown="keydown"
        @blur="cancel()"
        @pointerdown.stop
      />
    </div>
    <button
      v-else
      ref="button"
      type="button"
      class="flex h-(--knovum-compact-item-height) w-full min-w-0 items-center gap-2 text-left"
      :aria-pressed="selected"
      @click="emit('reveal')"
      @dblclick="emit('open')"
      @contextmenu.prevent="emit('menu')"
    >
      <component
        v-if="node.data.kind !== 'section'"
        :is="nodeIcon(node.data.kind)"
        class="size-(--knovum-icon-size-xs) shrink-0"
      />
      <SpaceOutlineTitle :title="node.data.title" />
      <slot />
    </button>
    <p v-if="editing && error" role="alert" class="pb-1 text-micro text-danger">{{ error }}</p>
  </div>
</template>
