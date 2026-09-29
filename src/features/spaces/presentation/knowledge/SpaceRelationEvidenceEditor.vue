<script setup lang="ts">
import { Pencil } from "@lucide/vue";
import { useId } from "reka-ui";
import { computed, nextTick, shallowRef, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
import type { SpaceRelation } from "../../model/relations/relation";
import { useSpaceStore } from "../../session/space-store";
import SpaceProvenanceSources from "./SpaceProvenanceSources.vue";

const props = defineProps<{ relation: SpaceRelation }>();
const store = useSpaceStore();
const { t } = useI18n();
const rationaleId = useId();
const rationaleInput = useTemplateRef<InstanceType<typeof UiTextarea>>("rationaleInput");
const draft = shallowRef(props.relation.rationale);
const editing = shallowRef(false);
const saving = shallowRef(false);
const changed = computed(() => draft.value.trim() !== props.relation.rationale.trim());
const attachNodeIds = computed(() => [props.relation.sourceId, props.relation.targetId]);

watch(
  () => props.relation.rationale,
  (value) => {
    if (!editing.value) draft.value = value;
  },
);
function beginEdit() {
  draft.value = props.relation.rationale;
  editing.value = true;
  void nextTick(() => rationaleInput.value?.focus());
}
function cancelEdit() {
  if (saving.value) return;
  draft.value = props.relation.rationale;
  editing.value = false;
}
async function saveRationale() {
  if (!changed.value || saving.value) return;
  saving.value = true;
  try {
    const rationale = draft.value.trim();
    await store.updateRelation(props.relation.id, { rationale });
    draft.value = rationale;
    editing.value = false;
  } catch (error) {
    toast.error(String(error));
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <div class="space-y-2.5">
    <div>
      <template v-if="editing">
        <label :for="rationaleId" class="sr-only">{{ t("spaces.knowledge.relations.rationale") }}</label>
        <UiTextarea
          ref="rationaleInput"
          v-model="draft"
          :id="rationaleId"
          :rows="2"
          :disabled="saving"
          :placeholder="t('spaces.knowledge.relations.rationalePlaceholder')"
          @keydown.esc.stop.prevent="cancelEdit"
          @keydown.meta.enter.stop.prevent="saveRationale"
          @keydown.ctrl.enter.stop.prevent="saveRationale"
        />
        <div class="mt-2 flex justify-end gap-1">
          <UiButton size="sm" variant="ghost" :disabled="saving" @click="cancelEdit">{{ t("common.cancel") }}</UiButton>
          <UiButton size="sm" variant="primary" :disabled="!changed || saving" @click="saveRationale">
            {{ t("common.save") }}
          </UiButton>
        </div>
      </template>
      <button
        v-else
        type="button"
        class="flex min-h-16 w-full items-start gap-2 rounded-md border border-stroke-default bg-surface-muted px-2.5 py-2 text-left hover:border-stroke-default-hover hover:bg-surface-control-hover focus-visible:outline-2 focus-visible:outline-stroke-focus"
        :aria-label="`${t(relation.rationale.trim() ? 'common.edit' : 'spaces.knowledge.relations.addRationale')} ${t('spaces.knowledge.relations.rationale')}`"
        @click="beginEdit"
      >
        <span
          class="min-w-0 flex-1 text-body leading-relaxed whitespace-pre-wrap"
          :class="relation.rationale.trim() ? 'text-fg-primary' : 'text-fg-tertiary'"
        >
          {{ relation.rationale.trim() || t("spaces.knowledge.relations.rationalePlaceholder") }}
        </span>
        <Pencil class="mt-0.5 size-3.5 shrink-0 text-fg-tertiary" :stroke-width="1.6" />
      </button>
    </div>
    <SpaceProvenanceSources
      target-kind="relation"
      :target-id="relation.id"
      :attach-node-ids="attachNodeIds"
      removable
    />
  </div>
</template>
