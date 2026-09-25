<script setup lang="ts">
import { Check, GitBranch, MessageSquareText } from "@lucide/vue";
import { storeToRefs } from "pinia";
import { computed, shallowRef, watch, watchEffect } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiDropdownMenu from "../../../../shared/ui/UiDropdownMenu.vue";
import UiDropdownMenuItem from "../../../../shared/ui/UiDropdownMenuItem.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiPopover from "../../../../shared/ui/UiPopover.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
import { CANVAS_RELATION_COLORS, type CanvasRelationColor } from "../../canvas/contract/canvas-document";
import { useCanvasEditorSession } from "../../canvas/editor-session";
import { SPACE_RELATION_KINDS, type SpaceRelationKind } from "../../model/relations/relation";
import { useSpaceStore } from "../../session/space-store";
import { CANVAS_ACTION_BAR_CLASSES } from "./canvas-action-classes";
import SpaceCanvasColorSwatches from "./SpaceCanvasColorSwatches.vue";

const store = useSpaceStore();
const { relations } = storeToRefs(store);
const session = useCanvasEditorSession();
const { t } = useI18n();
const rationale = shallowRef("");
const rationaleOpen = shallowRef(false);
const mutationPending = shallowRef(false);
const relationStyles = shallowRef(session.activeEditor.value?.getDocument().relationStyles);
const selected = computed(() => {
  const ids = session.selection.value.relationIds;
  return ids.length === 1 ? relations.value.find(({ id }) => id === ids[0]) : undefined;
});
const selectedColor = computed<CanvasRelationColor>(
  () => relationStyles.value?.find(({ relationId }) => relationId === selected.value?.id)?.color ?? "neutral",
);
const colorValues: Record<CanvasRelationColor, string> = {
  neutral: "var(--knovum-color-foreground-secondary)",
  blue: "var(--knovum-canvas-relation-blue)",
  green: "var(--knovum-canvas-relation-green)",
  amber: "var(--knovum-canvas-relation-amber)",
  red: "var(--knovum-canvas-relation-red)",
  purple: "var(--knovum-canvas-relation-purple)",
};
const colorOptions = computed(() =>
  CANVAS_RELATION_COLORS.map((value) => ({
    value,
    color: colorValues[value],
    label: t(`spaces.knowledge.relationColors.${value}`),
  })),
);

watch(
  selected,
  (relation) => {
    rationale.value = relation?.rationale ?? "";
  },
  { immediate: true },
);

watchEffect((cleanup) => {
  const editor = session.activeEditor.value;
  relationStyles.value = editor?.getDocument().relationStyles;
  if (editor) {
    cleanup(
      editor.observeDocument((document) => {
        if (relationStyles.value !== document.relationStyles) relationStyles.value = document.relationStyles;
      }),
    );
  }
});
async function updateKind(kind: SpaceRelationKind) {
  if (!selected.value || selected.value.kind === kind || mutationPending.value) return;
  mutationPending.value = true;
  try {
    await store.updateRelationKind(selected.value.id, kind);
  } catch (error) {
    toast.error(String(error));
  } finally {
    mutationPending.value = false;
  }
}

async function saveRationale() {
  if (!selected.value || mutationPending.value) return;
  mutationPending.value = true;
  try {
    await store.updateRelation(selected.value.id, { rationale: rationale.value.trim() });
    rationaleOpen.value = false;
  } catch (error) {
    toast.error(String(error));
  } finally {
    mutationPending.value = false;
  }
}

function setColor(value: string) {
  if (!selected.value) return;
  const color = CANVAS_RELATION_COLORS.find((option) => option === value);
  if (!color) return;
  session.activeEditor.value?.setRelationColor(selected.value.id, color);
}
</script>

<template>
  <div
    v-if="selected"
    :class="CANVAS_ACTION_BAR_CLASSES"
    :aria-label="t('spaces.knowledge.relations')"
    :aria-busy="mutationPending"
  >
    <UiDropdownMenu side="top" align="center" :side-offset="6" class="min-w-52">
      <template #trigger>
        <UiIconButton :label="t('spaces.knowledge.relations')" size="sm">
          <GitBranch class="size-(--knovum-icon-size-sm)" :stroke-width="1.55" />
        </UiIconButton>
      </template>
      <UiDropdownMenuItem
        v-for="kind in SPACE_RELATION_KINDS"
        :key="kind"
        :disabled="mutationPending"
        @select="updateKind(kind)"
      >
        <Check
          class="size-(--knovum-icon-size-xs)"
          :class="selected.kind === kind ? 'opacity-100' : 'opacity-0'"
        />
        {{ t(`spaces.knowledge.relation.${kind}`) }}
      </UiDropdownMenuItem>
    </UiDropdownMenu>
    <UiPopover v-model:open="rationaleOpen" side="top" align="center" :side-offset="8" class="w-72">
      <template #trigger>
        <UiIconButton :label="t('spaces.knowledge.relationRationale')" size="sm">
          <MessageSquareText class="size-(--knovum-icon-size-sm)" :stroke-width="1.55" />
        </UiIconButton>
      </template>
      <label class="text-micro font-medium text-foreground-tertiary">
        {{ t("spaces.knowledge.relationRationale") }}
      </label>
      <UiTextarea
        v-model="rationale"
        class="mt-1.5"
        :rows="3"
        :placeholder="t('spaces.knowledge.relationRationalePlaceholder')"
      />
      <div class="mt-2 flex justify-end">
        <UiButton size="sm" variant="secondary" :disabled="mutationPending" @click="saveRationale">
          {{ t("common.save") }}
        </UiButton>
      </div>
    </UiPopover>
    <SpaceCanvasColorSwatches
      :label="t('spaces.knowledge.relationColor')"
      :options="colorOptions"
      :selected="selectedColor"
      @select="setColor"
    />
  </div>
</template>
