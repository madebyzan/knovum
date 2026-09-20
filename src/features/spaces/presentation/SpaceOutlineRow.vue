<script setup lang="ts">
import { Frame, Group, LockKeyhole } from "@lucide/vue";
import { nextTick, shallowRef, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { nodeIcon } from "./node-icon";
import SpaceOutlineTitle from "./SpaceOutlineTitle.vue";
import type { StructureOutlineItem } from "./structure-outline";

const props = defineProps<{
  item: StructureOutlineItem;
  selected: boolean;
  editing: boolean;
  saveName(title: string): Promise<void>;
}>();
const emit = defineEmits<{ open: []; finish: [] }>();
const { t } = useI18n();
const input = useTemplateRef<HTMLInputElement>("input");
const row = useTemplateRef<HTMLElement>("row");
const draft = shallowRef("");
const saving = shallowRef(false);
const error = shallowRef("");
watch(
  () => props.editing,
  async (editing) => {
    if (!editing) return;
    draft.value = props.item.title;
    error.value = "";
    await nextTick();
    if (!props.editing) return;
    input.value?.scrollIntoView({ block: "nearest" });
    input.value?.focus({ preventScroll: true });
    input.value?.select();
  },
  { immediate: true },
);
async function finish(restoreFocus: boolean) {
  emit("finish");
  if (restoreFocus) {
    await nextTick();
    row.value?.closest<HTMLElement>('[role="treeitem"]')?.focus({ preventScroll: true });
  }
}
async function save() {
  if (saving.value) return;
  const title = draft.value.trim();
  if (!title || title.length > 512) {
    error.value = t("spaces.structure.renameInvalid");
    return;
  }
  saving.value = true;
  error.value = "";
  try {
    await props.saveName(title);
    await finish(true);
  } catch {
    error.value = t("spaces.structure.saveFailed");
    await nextTick();
    input.value?.focus();
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
  }
  if (event.key === "Escape" && !saving.value) {
    event.preventDefault();
    void finish(true);
  }
}
</script>
<template>
  <div ref="row" class="min-w-0 flex-1" :data-active="selected">
    <div
      class="flex h-(--knovum-compact-item-height) min-w-0 items-center gap-2"
      @dblclick="!editing && emit('open')"
    >
      <component
        :is="item.kind === 'section' ? Frame : item.kind === 'group' ? Group : nodeIcon(item.node!.data.kind)"
        class="size-(--knovum-icon-size-xs) shrink-0"
      />
      <input
        v-if="editing"
        ref="input"
        v-model="draft"
        :maxlength="512"
        :readonly="saving"
        :aria-label="t('spaces.structure.rename')"
        :aria-invalid="Boolean(error)"
        :aria-busy="saving"
        class="h-6 min-w-0 flex-1 rounded-control bg-control px-1 text-caption text-foreground ring-1 ring-focus-ring outline-none"
        @keydown="keydown"
        @pointerdown.stop
        @contextmenu.stop
        @blur="!saving && !error && finish(false)"
      />
      <SpaceOutlineTitle v-else :title="item.title" />
      <LockKeyhole
        v-if="item.locked"
        class="size-(--knovum-icon-size-xs) shrink-0 text-foreground-tertiary"
      />
    </div>
    <p v-if="editing && error" role="alert" class="pb-1 text-micro text-danger">{{ error }}</p>
  </div>
</template>
