<script setup lang="ts">
import { Link2, Pencil } from "@lucide/vue";
import { isString } from "es-toolkit";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiBadge from "../../../../shared/ui/UiBadge.vue";
import UiButton from "../../../../shared/ui/UiButton.vue";
import { selectAndRevealSpaceNode } from "../../canvas/selection-actions";
import { knowledgeObjectTypeKey } from "../../knowledge/model/object-presentation";
import { useSpaceObjectTypesQuery } from "../../knowledge/queries/queries";
import type { SpaceKnowledgeObjectPayload } from "../../model/nodes/node-payload";
import { useSpaceStore } from "../../session/space-store";

const props = withDefaults(
  defineProps<{ data: SpaceKnowledgeObjectPayload; editable?: boolean; editDisabled?: boolean }>(),
  {
    editable: true,
  },
);
const emit = defineEmits<{ edit: [] }>();
const { t } = useI18n();
const store = useSpaceStore();
const typesQuery = useSpaceObjectTypesQuery(() => store.activeSpaceId);
const objectType = computed(() => typesQuery.data.value?.find((type) => type.id === props.data.objectTypeId));
const typeLabel = computed(() => {
  const key = knowledgeObjectTypeKey(props.data.objectTypeId);
  return key
    ? t(`spaces.knowledgeObjectTypes.${key}`)
    : (objectType.value?.name ?? t("spaces.knowledge.objects.custom"));
});
const fields = computed(() =>
  (objectType.value?.fields ?? []).flatMap((field) => {
    const raw = props.data.properties[field.key];
    if (!["string", "number", "boolean"].includes(typeof raw) || raw === props.data.summary) return [];
    let displayValue = String(raw);
    let targetNodeId: string | undefined;
    if (field.kind === "node-ref" && isString(raw)) {
      const targetNode = store.nodes.find((n) => n.id === raw);
      if (targetNode) {
        displayValue = targetNode.data.title;
        targetNodeId = targetNode.id;
      }
    }
    return [{ key: field.key, label: field.label, value: displayValue, targetNodeId }];
  }),
);

function reveal(nodeId: string) {
  selectAndRevealSpaceNode(nodeId);
}
</script>

<template>
  <div class="space-y-3">
    <div class="flex items-center gap-2">
      <UiBadge size="sm">{{ typeLabel }}</UiBadge>
      <UiBadge size="sm" variant="accent">
        {{ t(`spaces.knowledge.status.labels.${data.status}`) }}
      </UiBadge>
      <UiButton
        v-if="editable"
        size="sm"
        variant="ghost"
        class="ml-auto shrink-0"
        :disabled="editDisabled"
        @click="emit('edit')"
      >
        <Pencil class="size-(--knovum-icon-size-xs)" />{{ t("common.edit") }}
      </UiButton>
    </div>

    <section v-if="data.summary">
      <h4 class="text-micro font-semibold tracking-wide text-foreground-tertiary uppercase">
        {{ t("spaces.knowledge.summary") }}
      </h4>
      <p class="mt-1.5 text-caption leading-relaxed whitespace-pre-wrap text-foreground">
        {{ data.summary }}
      </p>
    </section>

    <dl v-if="fields.length" class="space-y-2">
      <div v-for="field in fields" :key="field.key" class="grid grid-cols-[5.5rem_minmax(0,1fr)] gap-2">
        <dt class="text-caption text-foreground-tertiary">{{ field.label }}</dt>
        <dd class="min-w-0 text-caption text-foreground-secondary">
          <button
            v-if="field.targetNodeId"
            type="button"
            class="inline-flex max-w-full items-center gap-1.5 rounded-control bg-control px-2 py-1 text-caption text-foreground-secondary hover:bg-control-hover hover:text-foreground"
            @click="reveal(field.targetNodeId)"
          >
            <Link2 class="size-(--knovum-icon-size-xs) shrink-0 text-foreground-tertiary" />
            <span class="truncate">{{ field.value }}</span>
          </button>
          <span v-else class="block whitespace-pre-wrap">{{ field.value }}</span>
        </dd>
      </div>
    </dl>
  </div>
</template>
