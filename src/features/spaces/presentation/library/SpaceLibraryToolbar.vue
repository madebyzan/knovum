<script setup lang="ts">
import { FileInput, Grid2X2, List, Plus } from "@lucide/vue";
import { storeToRefs } from "pinia";
import { useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiSearchInput from "../../../../shared/ui/UiSearchInput.vue";
import { useSpacePackageTransfer } from "../../package/use-space-package-transfer";
import { useSpaceStore } from "../../session/space-store";

const store = useSpaceStore();
const packages = useSpacePackageTransfer();
const { libraryLayout, libraryQuery } = storeToRefs(store);
const { t } = useI18n();
const search = useTemplateRef<InstanceType<typeof UiSearchInput>>("search");
const emit = defineEmits<{ create: [] }>();

watch(
  () => store.librarySearchNonce,
  () => search.value?.focus(),
);
</script>

<template>
  <div class="relative flex min-w-0 flex-1 items-center justify-end gap-1">
    <UiIconButton size="sm" :label="t('spaces.space.newSpace')" shortcut="$mod+T" @click="emit('create')">
      <Plus class="size-(--knovum-icon-size-sm)" :stroke-width="1.6" />
    </UiIconButton>
    <UiIconButton size="sm" :label="t('spaces.transfer.importSpace')" @click="packages.importPackage">
      <FileInput class="size-(--knovum-icon-size-sm)" :stroke-width="1.6" />
    </UiIconButton>
    <UiIconButton
      size="sm"
      :active="libraryLayout === 'grid'"
      :label="t('spaces.space.gridView')"
      @click="libraryLayout = 'grid'"
    >
      <Grid2X2 class="size-(--knovum-icon-size-sm)" :stroke-width="1.6" />
    </UiIconButton>
    <UiIconButton
      size="sm"
      :active="libraryLayout === 'list'"
      :label="t('spaces.space.listView')"
      @click="libraryLayout = 'list'"
    >
      <List class="size-(--knovum-icon-size-sm)" :stroke-width="1.6" />
    </UiIconButton>
    <UiSearchInput
      ref="search"
      v-model="libraryQuery"
      size="sm"
      class="w-52"
      :placeholder="t('spaces.space.searchSpaces')"
    />
  </div>
</template>
