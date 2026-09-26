<script setup lang="ts">
import { Boxes, ChevronDown, ChevronRight, LayoutPanelTop, LockKeyhole } from "@lucide/vue";
import { error as logError } from "@tauri-apps/plugin-log";
import { nextTick, shallowRef, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import type { StructureOutlineRow } from "../model/structure/outline-projection";
import SpaceOutlineTitle from "./SpaceOutlineTitle.vue";

const props = defineProps<{
  row: StructureOutlineRow;
  selected: boolean;
  expanded: boolean;
  editing: boolean;
  tabIndex: number;
  saveName: (title: string) => Promise<unknown>;
}>();
const emit = defineEmits<{
  select: [];
  open: [];
  menu: [];
  toggle: [];
  finish: [];
  focus: [];
  navigate: [event: KeyboardEvent];
}>();
const { t } = useI18n();
const input = useTemplateRef<HTMLInputElement>("input");
const draft = shallowRef("");
const saving = shallowRef(false);
const error = shallowRef("");

watch(
  () => props.editing,
  async (editing) => {
    if (!editing) return;
    draft.value = props.row.title;
    error.value = "";
    await nextTick();
    input.value?.focus({ preventScroll: true });
    input.value?.select();
  },
  { immediate: true },
);

async function save() {
  if (saving.value) return;
  const title = draft.value.trim();
  if (!title) {
    error.value = t("spaces.sidebar.rename.nameRequired");
    return;
  }
  saving.value = true;
  try {
    await props.saveName(title);
    emit("finish");
  } catch (cause) {
    error.value = t("spaces.sidebar.rename.failed");
    void logError(`Structure node rename failed: ${String(cause)}`);
    await nextTick();
    input.value?.focus({ preventScroll: true });
  } finally {
    saving.value = false;
  }
}

function keydown(event: KeyboardEvent) {
  event.stopPropagation();
  if (event.isComposing) return;
  if (event.key === "Enter") {
    event.preventDefault();
    void save();
  } else if (event.key === "Escape") {
    event.preventDefault();
    emit("finish");
  }
}
</script>

<template>
  <div :data-active="selected" :style="{ paddingLeft: `${row.depth * 16}px` }">
    <div class="flex h-(--knovum-compact-item-height) items-center rounded-control hover:bg-control-hover">
      <button
        v-if="row.childCount"
        type="button"
        tabindex="-1"
        class="grid size-6 shrink-0 place-items-center text-foreground-tertiary"
        :aria-label="t(expanded ? 'common.collapse' : 'common.expand')"
        :aria-expanded="expanded"
        @click="emit('toggle')"
      >
        <ChevronDown v-if="expanded" class="size-(--knovum-icon-size-xs)" />
        <ChevronRight v-else class="size-(--knovum-icon-size-xs)" />
      </button>
      <span v-else class="w-6 shrink-0" />
      <template v-if="editing">
        <input
          ref="input"
          v-model="draft"
          maxlength="512"
          :readonly="saving"
          class="h-6 min-w-0 flex-1 rounded-control bg-control px-1 text-caption ring-1 ring-focus-ring outline-none"
          @keydown="keydown"
          @blur="emit('finish')"
        />
      </template>
      <button
        v-else
        type="button"
        role="treeitem"
        :aria-level="row.depth + 1"
        :aria-expanded="row.childCount ? expanded : undefined"
        :aria-selected="selected"
        :data-outline-tree-item="row.id"
        :tabindex="tabIndex"
        class="flex min-w-0 flex-1 items-center gap-2 pr-2 text-left text-caption text-foreground-secondary data-[active=true]:text-foreground"
        :data-active="selected"
        @click="emit('select')"
        @dblclick="emit('open')"
        @contextmenu.prevent="emit('menu')"
        @focus="emit('focus')"
        @keydown="emit('navigate', $event)"
      >
        <Boxes v-if="row.kind === 'group'" class="size-(--knovum-icon-size-xs) shrink-0" />
        <LayoutPanelTop v-else-if="row.kind === 'section'" class="size-(--knovum-icon-size-xs) shrink-0" />
        <SpaceOutlineTitle :title="row.title" />
        <LockKeyhole
          v-if="row.locked"
          aria-hidden="true"
          class="size-(--knovum-icon-size-xs) shrink-0 text-foreground-tertiary"
        />
        <span v-if="row.locked" class="sr-only">{{ t("spaces.canvas.locked") }}</span>
        <span v-if="row.childCount" class="text-micro text-foreground-tertiary tabular-nums">{{
          row.childCount
        }}</span>
      </button>
    </div>
    <p v-if="editing && error" role="alert" class="pb-1 pl-6 text-micro text-danger">{{ error }}</p>
  </div>
</template>
