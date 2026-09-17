<script setup lang="ts">
import { AlertTriangle, RotateCcw } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import UiButton from "../shared/ui/UiButton.vue";

defineProps<{ failure?: Error | undefined }>();
const { t } = useI18n();

function reload() {
  globalThis.location.reload();
}
</script>

<template>
  <main data-tauri-drag-region class="grid h-full place-items-center bg-chrome p-8 select-none">
    <section
      v-if="failure"
      class="w-full max-w-(--knovum-error-max-width) rounded-dialog border border-border-strong bg-raised p-5 shadow-dialog"
    >
      <AlertTriangle class="mb-3 size-(--knovum-icon-size-lg) text-danger" />
      <h1 class="m-0 text-title-sm font-semibold text-foreground">{{ t("errors.bootstrapTitle") }}</h1>
      <p class="mt-1.5 text-body text-foreground-secondary">{{ t("errors.bootstrapDescription") }}</p>
      <pre
        class="mt-4 max-h-40 overflow-auto rounded-control bg-chrome p-3 font-mono text-micro text-foreground-secondary select-text"
        >{{ failure.message }}</pre>
      <div class="mt-4">
        <UiButton @click="reload"
          ><RotateCcw class="size-(--knovum-icon-size-sm)" /> {{ t("common.reload") }}</UiButton
        >
      </div>
    </section>

    <div v-else class="flex flex-col items-center justify-center">
      <div class="pointer-events-none relative size-20 select-none">
        <img
          src="/app-icon.png"
          alt=""
          width="80"
          height="80"
          class="size-full opacity-90 select-none"
          draggable="false"
        />
        <div class="shimmer-mask absolute inset-0">
          <div class="shimmer-beam" />
        </div>
      </div>
      <p class="mt-4 text-label font-medium tracking-tight text-foreground-secondary">
        {{ t("common.starting") }}
      </p>
    </div>
  </main>
</template>

<style scoped>
.shimmer-mask {
  mask-image: url("/app-icon.png");
  -webkit-mask-image: url("/app-icon.png");
  mask-size: 100% 100%;
  -webkit-mask-size: 100% 100%;
  overflow: hidden;
}

.shimmer-beam {
  position: absolute;
  top: -50%;
  left: -50%;
  width: 200%;
  height: 200%;
  background: linear-gradient(
    115deg,
    transparent 35%,
    oklch(1 0 0 / 0.25) 45%,
    oklch(1 0 0 / 0.7) 50%,
    oklch(1 0 0 / 0.25) 55%,
    transparent 65%
  );
  animation: beam-sweep 2.4s cubic-bezier(0.4, 0, 0.2, 1) infinite;
}

@keyframes beam-sweep {
  0% {
    transform: translateX(-100%);
  }
  60%,
  100% {
    transform: translateX(100%);
  }
}

@media (prefers-reduced-motion: reduce) {
  .shimmer-beam {
    animation: none;
    display: none;
  }
}
</style>
