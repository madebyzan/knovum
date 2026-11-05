<script setup lang="ts">
import { computed } from "vue";

import { spaceAssetSource } from "../../../assets/node-preview-source";
import type { SpaceWebsitePayload } from "../../../model/nodes/node-payload";
import type { CanvasNodePresentationContext } from "../../preview/policy";
import SpaceCanvasGitHubRepository from "./SpaceCanvasGitHubRepository.vue";
import SpaceCanvasTwitterPost from "./SpaceCanvasTwitterPost.vue";
import SpaceCanvasWebsiteBookmark from "./SpaceCanvasWebsiteBookmark.vue";
import SpaceCanvasWebsiteHero from "./SpaceCanvasWebsiteHero.vue";

const props = defineProps<{
  id: string;
  data: SpaceWebsitePayload;
  frame?: { width: number; height: number } | undefined;
  presentationContext: CanvasNodePresentationContext;
}>();

const host = computed(() => {
  try {
    return new URL(props.data.url).hostname.replace(/^www\./u, "");
  } catch {
    return props.data.url;
  }
});

const isTwitter = computed(() => host.value === "x.com" || host.value === "twitter.com");
const isGitHub = computed(() => host.value === "github.com");

const previewSource = computed(() =>
  props.data.previewImagePath ? spaceAssetSource(props.data.previewImagePath) : undefined,
);

const showsPreview = computed(
  () =>
    props.presentationContext.mode === "interactive" ||
    props.presentationContext.viewport.previewItemLimit >= 4,
);

const hasHeroPreview = computed(() => Boolean(showsPreview.value && previewSource.value));
</script>

<template>
  <SpaceCanvasTwitterPost
    v-if="isTwitter"
    :id="id"
    :data="data"
    :host="host"
    :frame="frame"
    :presentation-context="presentationContext"
  />
  <SpaceCanvasGitHubRepository
    v-else-if="isGitHub"
    :id="id"
    :data="data"
    :host="host"
    :frame="frame"
    :presentation-context="presentationContext"
  />
  <SpaceCanvasWebsiteHero
    v-else-if="hasHeroPreview"
    :id="id"
    :data="data"
    :host="host"
    :frame="frame"
    :presentation-context="presentationContext"
  />
  <SpaceCanvasWebsiteBookmark
    v-else
    :id="id"
    :data="data"
    :host="host"
    :frame="frame"
    :presentation-context="presentationContext"
  />
</template>
