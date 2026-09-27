<script setup lang="ts">
import { LoaderCircle, Trash2 } from "@lucide/vue";
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiDialog from "../../../../shared/ui/UiDialog.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import { useRecentlyDeletedActions } from "../../deletion/use-recently-deleted-actions";
import { useRecentlyDeletedQuery } from "../../deletion/use-recently-deleted-query";
import { useSpaceStore } from "../../session/space-store";
import SpaceDeletedStructureRecovery from "./SpaceDeletedStructureRecovery.vue";

const props = defineProps<{ spaceId: string | null }>();
const open = defineModel<boolean>("open", { required: true });
const store = useSpaceStore();
const { t } = useI18n();
const filter = shallowRef(props.spaceId ?? "all");
const scope = computed(() => (filter.value === "all" ? null : filter.value));
const options = computed(() => [
  { value: "all", label: t("spaces.space.sections.all") },
  ...store.spaces.map((space) => ({ value: space.id, label: space.title })),
]);
const scopeLabel = computed(() => options.value.find((option) => option.value === filter.value)!.label);
const deletedQuery = useRecentlyDeletedQuery(scope, open);
const deletedRoots = computed(() => deletedQuery.data.value ?? []);
const { busy, restore, remove } = useRecentlyDeletedActions();
const canDeleteAll = computed(
  () =>
    !busy.value &&
    !deletedQuery.isFetching.value &&
    !deletedQuery.isError.value &&
    deletedRoots.value.length > 0,
);
</script>

<template>
  <UiDialog
    v-model:open="open"
    :title="t('spaces.trash.recent.title')"
    :description="t('spaces.trash.recent.description')"
  >
    <UiSelect
      v-model="filter"
      :options="options"
      :label="t('spaces.trash.filters.space')"
      :disabled="busy"
      size="sm"
      class="mb-3"
    />
    <div v-if="deletedQuery.isPending.value" class="grid h-72 place-items-center">
      <LoaderCircle class="size-3.5 animate-spin text-fg-accent" />
    </div>
    <div v-else-if="deletedQuery.isError.value" class="grid h-72 content-center justify-items-center gap-3">
      <p class="text-caption-md text-fg-secondary">
        {{ t("spaces.trash.recent.loadFailed") }}
      </p>
      <UiButton size="sm" @click="deletedQuery.refetch()">{{ t("common.retry") }}</UiButton>
    </div>
    <SpaceDeletedStructureRecovery
      v-else-if="deletedRoots.length"
      :deleted-roots="deletedRoots"
      :busy="busy || deletedQuery.isFetching.value"
      @restore="restore"
      @restore-batch="restore($event, true)"
      @remove="remove([$event])"
    />
    <div v-else class="grid h-72 content-center justify-items-center gap-2 text-fg-tertiary">
      <Trash2 class="size-5" :stroke-width="1.5" />
      <p class="text-caption-md">{{ t("spaces.trash.recent.empty") }}</p>
    </div>
    <template #footer>
      <span role="status" class="mr-auto self-center text-caption-md text-fg-secondary">
        {{
          busy
            ? t("spaces.trash.structure.processing")
            : t("spaces.trash.structure.deletedCount", { count: deletedRoots.length })
        }}
      </span>
      <UiButton
        size="sm"
        variant="dangerSubtle"
        :disabled="!canDeleteAll"
        @click="remove(deletedRoots, scopeLabel)"
      >
        {{ t("spaces.trash.deleteAll.action") }}
      </UiButton>
    </template>
  </UiDialog>
</template>
