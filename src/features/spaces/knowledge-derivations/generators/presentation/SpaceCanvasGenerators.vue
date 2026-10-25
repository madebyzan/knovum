<script setup lang="ts">
import { useCanvasGeneratorStore } from "../generator-store";
import SpaceCanvasGeneratorComposer from "./SpaceCanvasGeneratorComposer.vue";
import SpaceCanvasGeneratorFrame from "./SpaceCanvasGeneratorFrame.vue";

/** The generators of the open Space, drawn in canvas space; the active one takes requests below its frame. */
const store = useCanvasGeneratorStore();
</script>

<template>
  <SpaceCanvasGeneratorFrame
    v-for="generator in store.generators"
    :key="generator.id"
    :generator="generator"
    :active="store.active?.id === generator.id"
  />
  <SpaceCanvasGeneratorComposer v-if="store.active" :key="store.active.id" :generator="store.active" />
</template>
