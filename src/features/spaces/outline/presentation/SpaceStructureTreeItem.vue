<script setup lang="ts">
import { Boxes, ChevronDown, ChevronRight, LayoutPanelTop, LockKeyhole } from "@lucide/vue";
import { error as logError } from "@tauri-apps/plugin-log";
import { useResizeObserver } from "@vueuse/core";
import { omit } from "es-toolkit";
import { TreeItem, type TreeItemSelectEvent, type TreeItemToggleEvent } from "reka-ui";
import { nextTick, shallowRef, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { formatErrorDiagnostic } from "../../../../foundation/errors/error-format";
import UiInput from "../../../../shared/ui/UiInput.vue";
import type { OutlineTreeItem } from "./outline-tree-items";
import SpaceOutlineTitle from "./SpaceOutlineTitle.vue";

defineOptions({ inheritAttrs: false });
const props = defineProps<{
  editing: boolean;
  expanded: boolean;
  item: OutlineTreeItem;
  level: number;
  saveName: (title: string) => Promise<unknown>;
  selected: boolean;
}>();
const emit = defineEmits<{
  finish: [];
  height: [height: number];
  menu: [];
  open: [];
  select: [];
  toggle: [];
}>();
const { t } = useI18n();
const input = useTemplateRef<InstanceType<typeof UiInput>>("input");
const root = useTemplateRef<HTMLElement>("root");
useResizeObserver(root, ([entry]) => {
  if (entry) emit("height", entry.borderBoxSize[0]!.blockSize);
});
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
    input.value?.focus({ preventScroll: true });
    input.value?.select();
  },
  { immediate: true },
);

/** The canvas owns selection, so reka's own selection state stays inert and is only relayed. */
function selectFromTree(event: TreeItemSelectEvent<OutlineTreeItem>) {
  event.preventDefault();
  emit("select");
}

/** TreeItem toggles on click too; disclosure belongs to the chevron and the arrow keys alone. */
function suppressClickToggle(event: TreeItemToggleEvent<OutlineTreeItem>) {
  if (event.detail.originalEvent.type === "click") event.preventDefault();
}

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
    void logError(`Structure node rename failed: ${formatErrorDiagnostic(cause)}`);
    await nextTick();
    input.value?.focus({ preventScroll: true });
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <div
    ref="root"
    class="w-full min-w-0 pb-0.5"
    v-bind="omit($attrs, ['data-index'])"
    :data-active="selected"
    :style="{ paddingLeft: `${(level - 1) * 16}px` }"
  >
    <div
      :data-active="selected"
      class="flex h-interaction min-w-0 items-center rounded-md hover:bg-(--knovum-sidebar-item-bg-hover) data-[active=true]:bg-(--knovum-sidebar-item-bg-selected)"
    >
      <button
        v-if="item.children"
        type="button"
        tabindex="-1"
        class="grid size-6 shrink-0 place-items-center text-fg-tertiary"
        :aria-label="t(expanded ? 'common.collapse' : 'common.expand')"
        :aria-expanded="expanded"
        @click="emit('toggle')"
      >
        <ChevronDown v-if="expanded" class="size-3" />
        <ChevronRight v-else class="size-3" />
      </button>
      <span v-else class="w-6 shrink-0" />
      <TreeItem
        v-if="!editing"
        as-child
        :data-index="$attrs['data-index']"
        :value="item"
        :level="level"
        @select="selectFromTree"
        @toggle="suppressClickToggle"
      >
        <button
          type="button"
          :aria-selected="selected"
          :data-active="selected"
          class="flex h-full min-w-0 flex-1 items-center gap-2 pr-2 text-left text-caption text-fg-secondary data-[active=true]:text-fg-primary"
          @dblclick="emit('open')"
          @contextmenu.prevent="emit('menu')"
        >
          <Boxes v-if="item.kind === 'group'" class="size-3 shrink-0" />
          <LayoutPanelTop v-else-if="item.kind === 'section'" class="size-3 shrink-0" />
          <SpaceOutlineTitle :title="item.title" />
          <LockKeyhole v-if="item.locked" aria-hidden="true" class="size-3 shrink-0 text-fg-tertiary" />
          <span v-if="item.locked" class="sr-only">{{ t("spaces.canvas.locked") }}</span>
          <span v-if="item.children" class="text-caption-sm text-fg-tertiary tabular-nums">{{
            item.children.length
          }}</span>
        </button>
      </TreeItem>
      <UiInput
        v-else
        ref="input"
        v-model="draft"
        size="sm"
        :aria-label="t('common.rename')"
        maxlength="512"
        :readonly="saving"
        class="h-6 min-w-0 flex-1 px-1 text-caption"
        @keydown.stop
        @commit.prevent="save"
        @cancel.prevent="!saving && emit('finish')"
        @blur="!saving && emit('finish')"
      />
    </div>
    <p v-if="editing && error" role="alert" class="pb-1 pl-6 text-caption-sm text-fg-danger">
      {{ error }}
    </p>
  </div>
</template>
