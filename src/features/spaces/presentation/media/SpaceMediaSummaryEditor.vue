<script setup lang="ts">
import { Check, Pencil, X } from "@lucide/vue";
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
import { useNodeDraft } from "../inspector/use-node-draft";

const props = defineProps<{ nodeId: string; summary: string; disabled?: boolean }>();
const emit = defineEmits<{ save: [summary: string] }>();
const { t } = useI18n();
const editing = ref(false);
const draft = ref(props.summary);
const nodeDraft = useNodeDraft(computed(() => props.nodeId));
watch(
  () => props.nodeId,
  () => (editing.value = false),
);
watch(
  () => props.summary,
  (value) => {
    if (!editing.value) draft.value = value;
  },
);
function beginSummaryEdit() {
  if (!nodeDraft.begin()) return;
  draft.value = props.summary;
  editing.value = true;
}
function cancelSummaryEdit() {
  nodeDraft.end();
  draft.value = props.summary;
  editing.value = false;
}
function saveMediaSummary() {
  nodeDraft.end();
  emit("save", draft.value);
  editing.value = false;
}
</script>

<template>
  <div>
    <div class="flex items-center gap-2">
      <h4 class="flex-1 text-caption-sm font-semibold tracking-wide text-fg-tertiary uppercase">
        {{ t("spaces.media.analysis.summary") }}
      </h4>
      <template v-if="editing">
        <UiIconButton :label="t('common.cancel')" size="sm" @click="cancelSummaryEdit"
          ><X class="size-3"
        /></UiIconButton>
        <UiIconButton :label="t('common.save')" size="sm" @click="saveMediaSummary"
          ><Check class="size-3"
        /></UiIconButton>
      </template>
      <UiIconButton
        v-else
        :label="t('common.edit')"
        size="sm"
        :disabled="disabled || !nodeDraft.available.value"
        @click="beginSummaryEdit"
        ><Pencil class="size-3"
      /></UiIconButton>
    </div>
    <UiTextarea v-if="editing" v-model="draft" class="mt-1.5" :rows="5" />
    <p v-else class="mt-1.5 text-caption leading-relaxed text-fg-secondary">{{ summary }}</p>
  </div>
</template>
