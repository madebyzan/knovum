<script setup lang="ts">
import { computed } from "vue";

import { spaceAssetSource } from "../../../assets/node-preview-source";
import type { SpaceWebsitePayload } from "../../../model/nodes/node-payload";
import type { CanvasNodePresentationContext } from "../../preview/policy";
import SpaceCanvasNodeStatus from "./SpaceCanvasNodeStatus.vue";

const props = defineProps<{
  id: string;
  data: SpaceWebsitePayload;
  host: string;
  frame?: { width: number; height: number } | undefined;
  presentationContext: CanvasNodePresentationContext;
}>();

const handle = computed(() => {
  try {
    const { pathname } = new URL(props.data.url);
    const match = pathname.match(/^\/([^/]+)\/status\/\d+/u);
    return match ? `@${match[1]}` : undefined;
  } catch {
    return undefined;
  }
});

function extractAuthorName(title: string, fallbackHandle?: string): string {
  const match = title.match(/^(?<name>.+?)(?:\s*\(@[^)]+\))?\s+on\s+X(?:[:/]|$)/u);
  if (match?.groups?.name) return match.groups.name.trim();
  if (fallbackHandle) return fallbackHandle.replace(/^@/u, "");
  return title || "X";
}

const authorName = computed(() => extractAuthorName(props.data.title || "", handle.value));

const authorInitial = computed(() => {
  const name = authorName.value.trim();
  return name.charAt(0).toUpperCase() || "X";
});

function cleanTweetContent(text: string): string {
  return text
    .replace(/(?:(?:👉|🔗|⬇️|↓)\s*)?https:\/\/t\.co\/\w+/gu, "")
    .replace(/\n{3,}/gu, "\n\n")
    .trim();
}

const tweetText = computed(() => {
  const desc = props.data.description?.trim();
  if (desc) return cleanTweetContent(desc);
  const quote = props.data.title?.match(/on\s+X:\s*["“](?<text>.+?)["”]/u);
  const raw = quote?.groups?.text?.trim() ?? props.data.contentPreview?.trim() ?? "";
  return cleanTweetContent(raw);
});

const previewSource = computed(() =>
  props.data.previewImagePath ? spaceAssetSource(props.data.previewImagePath) : undefined,
);

const showsPreview = computed(
  () =>
    props.presentationContext.mode === "interactive" ||
    props.presentationContext.viewport.previewItemLimit >= 4,
);

const hasImage = computed(() => Boolean(showsPreview.value && previewSource.value));
</script>

<template>
  <div
    class="flex size-full min-h-0 flex-col justify-between gap-2 overflow-hidden bg-surface-panel p-3 select-none"
  >
    <!-- Header: Author Profile & X Platform Identity -->
    <div class="flex shrink-0 items-center justify-between gap-2">
      <div class="flex min-w-0 items-center gap-2">
        <div
          class="flex size-7 shrink-0 items-center justify-center rounded-full border border-stroke-divider bg-surface-control text-caption font-semibold text-fg-primary shadow-2xs"
        >
          {{ authorInitial }}
        </div>
        <div class="min-w-0 flex-1 leading-tight">
          <p class="truncate text-caption font-semibold text-fg-primary">
            {{ authorName }}
          </p>
          <p v-if="handle" class="truncate text-[11px] text-fg-tertiary">
            {{ handle }}
          </p>
        </div>
      </div>

      <div class="flex shrink-0 items-center gap-1.5">
        <SpaceCanvasNodeStatus :id="id" :data="data" />
        <svg viewBox="0 0 24 24" class="size-3.5 fill-current text-fg-primary opacity-75" aria-label="X">
          <path
            d="M18.244 2.25h3.308l-7.227 8.26 8.502 11.24H16.17l-5.214-6.817L4.99 24.75H1.68l7.73-8.835L1.254 2.25H8.08l4.713 6.231zm-1.161 17.52h1.833L7.084 4.126H5.117z"
          />
        </svg>
      </div>
    </div>

    <!-- Body: Tweet text -->
    <p
      v-if="tweetText"
      :class="
        hasImage
          ? 'line-clamp-2 shrink-0 text-caption leading-normal'
          : 'my-auto line-clamp-5 text-body leading-relaxed'
      "
      class="wrap-break-word whitespace-pre-line text-fg-primary"
    >
      {{ tweetText }}
    </p>

    <!-- Attached Media: Fills rounded media frame cleanly and scales smoothly across resize -->
    <div
      v-if="hasImage"
      class="relative min-h-0 w-full flex-1 overflow-hidden rounded-lg border border-stroke-divider bg-surface-control/20"
    >
      <img
        :src="previewSource"
        alt=""
        class="size-full object-cover object-center"
        draggable="false"
        loading="lazy"
        decoding="async"
      />
    </div>

    <!-- Footer -->
    <div class="flex shrink-0 items-center justify-between text-[11px] leading-none text-fg-tertiary">
      <span class="truncate">x.com</span>
      <span v-if="handle" class="truncate font-medium text-fg-secondary">{{ handle }}</span>
    </div>
  </div>
</template>
