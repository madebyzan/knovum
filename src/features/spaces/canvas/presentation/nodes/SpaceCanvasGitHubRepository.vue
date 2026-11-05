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

const repositoryTarget = computed(() => {
  try {
    const { pathname } = new URL(props.data.url);
    const match = pathname.match(/^\/([^/]+)\/([^/]+)/u);
    const owner = match?.[1];
    const repo = match?.[2];
    if (owner && repo) {
      return { owner, repo: repo.replace(/\.git$/u, "") };
    }
    return undefined;
  } catch {
    return undefined;
  }
});

function cleanGitHubBoilerplate(text: string): string {
  return text
    .replace(/^Contribute to \S+ development by creating an account on GitHub\.?$/iu, "")
    .replace(/\.?\s*Contribute to \S+ development by creating an account on GitHub\.?$/iu, "")
    .trim();
}

function stripRepoSuffix(text: string, target?: { owner: string; repo: string }): string {
  if (!target) return text;
  const suffix = ` - ${target.owner}/${target.repo}`;
  return text.endsWith(suffix) ? text.slice(0, -suffix.length).trim() : text;
}

const repositoryDescription = computed(() => {
  const fromDesc = cleanGitHubBoilerplate(props.data.description?.trim() ?? "");
  const fromTitle = cleanGitHubBoilerplate(props.data.title?.replace(/^GitHub\s*-\s*[^:]+:\s*/u, "") ?? "");
  const stripped = stripRepoSuffix(fromDesc || fromTitle, repositoryTarget.value);
  return stripped || props.data.contentPreview?.trim() || "";
});

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
  <div class="flex size-full min-h-0 flex-col overflow-hidden bg-surface-panel select-none">
    <div
      :class="hasHeroPreview ? 'h-[15.4%] min-h-7' : 'h-9'"
      class="flex shrink-0 items-center justify-between gap-2 border-b border-stroke-divider bg-surface-panel px-3"
    >
      <div class="flex min-w-0 flex-1 items-center gap-1.5 truncate">
        <svg viewBox="0 0 24 24" class="size-3.5 shrink-0 fill-current text-fg-primary" aria-label="GitHub">
          <path
            d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.53 1.032 1.53 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z"
          />
        </svg>
        <div v-if="repositoryTarget" class="truncate text-caption-sm text-fg-secondary">
          <span>{{ repositoryTarget.owner }}</span>
          <span class="mx-0.5 text-fg-tertiary">/</span>
          <strong class="font-semibold text-fg-primary">{{ repositoryTarget.repo }}</strong>
        </div>
        <span v-else class="truncate text-caption-sm font-medium text-fg-secondary">{{ host }}</span>
      </div>
      <div class="flex shrink-0 items-center gap-1.5">
        <SpaceCanvasNodeStatus :id="id" :data="data" />
        <span
          v-if="repositoryTarget"
          class="rounded-full border border-stroke-divider bg-surface-control/60 px-1.5 py-0.5 text-[10px] leading-none font-medium text-fg-tertiary"
        >
          Repo
        </span>
      </div>
    </div>

    <!-- Layout A: Exact 2:1 Social Card Stage + Quiet Summary Footer -->
    <template v-if="hasHeroPreview">
      <div class="relative h-[69.2%] w-full shrink-0 overflow-hidden bg-white">
        <img
          :src="previewSource"
          :alt="repositoryDescription"
          class="size-full object-contain object-center"
          draggable="false"
          loading="lazy"
          decoding="async"
        />
      </div>

      <div
        class="flex min-h-0 flex-1 items-center justify-between gap-2 border-t border-stroke-divider bg-surface-muted/45 px-3"
      >
        <p class="min-w-0 flex-1 truncate text-caption-sm text-fg-secondary">
          {{ repositoryDescription || data.title || host }}
        </p>
        <span class="shrink-0 font-mono text-[10px] text-fg-tertiary">github.com</span>
      </div>
    </template>

    <!-- Layout B: Text-first Repository Overview -->
    <template v-else>
      <div class="flex min-h-0 flex-1 flex-col justify-between overflow-hidden px-3.5 py-3">
        <div class="flex flex-col gap-1 overflow-hidden">
          <strong v-if="repositoryTarget" class="line-clamp-1 text-label font-semibold text-fg-primary">
            {{ repositoryTarget.repo }}
          </strong>
          <p
            v-if="repositoryDescription"
            class="line-clamp-4 text-caption leading-relaxed wrap-break-word text-fg-secondary"
          >
            {{ repositoryDescription }}
          </p>
        </div>

        <div class="flex shrink-0 items-center justify-between text-caption-sm text-fg-tertiary">
          <span class="font-mono text-[10px]">github.com</span>
          <span v-if="repositoryTarget" class="truncate text-[11px] text-fg-secondary">
            {{ repositoryTarget.owner }}/{{ repositoryTarget.repo }}
          </span>
        </div>
      </div>
    </template>
  </div>
</template>
