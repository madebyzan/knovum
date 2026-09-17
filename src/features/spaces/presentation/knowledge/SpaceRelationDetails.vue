<script setup lang="ts">
import { Check } from "@lucide/vue";
import { storeToRefs } from "pinia";
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
import type { SpaceRelation } from "../../model/relations/relation";
import { useSpaceStore } from "../../session/space-store";
import SpaceProvenanceSources from "./SpaceProvenanceSources.vue";

const props = defineProps<{ relation: SpaceRelation }>();
const store = useSpaceStore();
const { t } = useI18n();
const { nodes } = storeToRefs(store);
const rationale = ref(props.relation.rationale);
const attachNodeIds = computed(() =>
  nodes.value
    .filter(({ id }) => id === props.relation.sourceId || id === props.relation.targetId)
    .map(({ id }) => id),
);

watch(
  () => props.relation.rationale,
  (value) => {
    rationale.value = value;
  },
);
async function saveRationale() {
  await store.updateRelation(props.relation.id, { rationale: rationale.value.trim() });
}
</script>

<template>
  <div class="space-y-2 border-t border-divider px-2 py-2.5">
    <div>
      <label class="text-micro font-medium text-foreground-tertiary">{{
        t("spaces.knowledge.relationRationale")
      }}</label>
      <UiTextarea
        v-model="rationale"
        class="mt-1"
        :rows="3"
        :placeholder="t('spaces.knowledge.relationRationalePlaceholder')"
      />
      <div class="mt-1.5 flex justify-end">
        <UiButton size="sm" variant="secondary" @click="saveRationale">
          <Check class="mr-1 size-(--knovum-icon-size-xs)" />{{ t("common.save") }}
        </UiButton>
      </div>
    </div>
    <SpaceProvenanceSources
      target-kind="relation"
      :target-id="relation.id"
      :attach-node-ids="attachNodeIds"
      removable
    />
  </div>
</template>
