<script setup lang="ts">
import { computed, defineAsyncComponent } from "vue";

import type { SpaceEditorTab } from "../../workspace/workbench-tab";
import { useDeferredEditorMount } from "../use-deferred-editor-mount";
import SpaceEditorLoading from "./SpaceEditorLoading.vue";

const props = defineProps<{ tab: SpaceEditorTab; active: boolean }>();
const mounted = useDeferredEditorMount(computed(() => props.tab.id));
const SpaceTiptapEditor = defineAsyncComponent(() => import("./SpaceTiptapEditor.vue"));
</script>

<template>
  <SpaceEditorLoading v-if="active && !mounted" />
  <SpaceTiptapEditor v-if="mounted" :tab="tab" :active="active" />
</template>
