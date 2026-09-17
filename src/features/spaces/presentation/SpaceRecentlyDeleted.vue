<script setup lang="ts">
import { LoaderCircle, Trash2 } from "@lucide/vue";
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../shared/ui/UiButton.vue";
import UiDialog from "../../../shared/ui/UiDialog.vue";
import UiSelect from "../../../shared/ui/UiSelect.vue";
import { useRecentlyDeletedActions } from "../deletion/use-recently-deleted-actions";
import { useRecentlyDeletedQuery } from "../knowledge/queries/queries";
import { useSpaceStore } from "../session/space-store";
import SpaceDeletedNodeList from "./SpaceDeletedNodeList.vue";

const props = defineProps<{ spaceId: string | null }>();
const open = defineModel<boolean>("open", { required: true });
const store = useSpaceStore();
const { t } = useI18n();
const filter = shallowRef(props.spaceId ?? "all");
const scope = computed(() => (filter.value === "all" ? null : filter.value));
const options = computed(() => [
  { value: "all", label: t("spaces.space.allSpaces") },
  ...store.spaces.map((space) => ({ value: space.id, label: space.title })),
]);
const scopeLabel = computed(() => options.value.find((option) => option.value === filter.value)!.label);
const deletedQuery = useRecentlyDeletedQuery(scope, open);
const items = computed(() => deletedQuery.data.value ?? []);
const { busy, restore, remove } = useRecentlyDeletedActions();
const canDeleteAll = computed(
  () =>
    !busy.value && !deletedQuery.isFetching.value && !deletedQuery.isError.value && items.value.length > 0,
);
</script>

<template>
  <UiDialog
    v-model:open="open"
    :title="t('spaces.navigation.recentlyDeleted')"
    :description="t('spaces.navigation.recentlyDeletedDescription')"
  >
    <UiSelect
      v-model="filter"
      :options="options"
      :label="t('spaces.navigation.deletedSpaceFilter')"
      :disabled="busy"
      size="sm"
      class="mb-3"
    />
    <div v-if="deletedQuery.isPending.value" class="grid h-72 place-items-center">
      <LoaderCircle class="size-(--knovum-icon-size-sm) animate-spin text-accent" />
    </div>
    <div v-else-if="deletedQuery.isError.value" class="grid h-72 content-center justify-items-center gap-3">
      <p class="text-caption text-foreground-secondary">
        {{ t("spaces.navigation.recentlyDeletedLoadFailed") }}
      </p>
      <UiButton size="sm" @click="deletedQuery.refetch()">{{ t("common.retry") }}</UiButton>
    </div>
    <SpaceDeletedNodeList
      v-else-if="items.length"
      :items="items"
      :busy="busy || deletedQuery.isFetching.value"
      @restore="restore"
      @remove="remove([$event])"
    />
    <div v-else class="grid h-72 content-center justify-items-center gap-2 text-foreground-tertiary">
      <Trash2 class="size-(--knovum-icon-size-lg)" :stroke-width="1.5" />
      <p class="text-caption">{{ t("spaces.navigation.recentlyDeletedEmpty") }}</p>
    </div>
    <template #footer>
      <span role="status" class="mr-auto self-center text-caption text-foreground-secondary">
        {{
          busy
            ? t("spaces.navigation.processingDeletedItems")
            : t("spaces.navigation.deletedItemCount", { count: items.length })
        }}
      </span>
      <UiButton size="sm" variant="dangerSubtle" :disabled="!canDeleteAll" @click="remove(items, scopeLabel)">
        {{ t("spaces.navigation.deleteAll") }}
      </UiButton>
    </template>
  </UiDialog>
</template>
