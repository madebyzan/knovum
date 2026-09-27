<script setup lang="ts">
import { CheckCircle2, CircleAlert, CircleX, Square, Trash2 } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import { formatBytes } from "../../../foundation/format/bytes";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiIconButton from "../../../shared/ui/UiIconButton.vue";
import UiProgress from "../../../shared/ui/UiProgress.vue";
import UiSpinner from "../../../shared/ui/UiSpinner.vue";
import type { ModelProgressInfo } from "../runtime/use-runtime-settings";

defineProps<{
  title: string;
  description?: string;
  installed: boolean;
  installedCount?: number;
  totalCount?: number;
  size?: number;
  progress: ModelProgressInfo;
  busy?: boolean;
}>();

const emit = defineEmits<{
  download: [];
  cancel: [];
  remove: [];
}>();

const { t } = useI18n();
</script>

<template>
  <div
    class="flex min-h-(--knovum-preference-item-height) flex-col justify-center border-b border-divider px-4 py-3 last:border-b-0"
  >
    <div class="flex items-center justify-between gap-6">
      <div class="min-w-0 flex-1">
        <div class="flex items-center gap-2">
          <UiSpinner
            v-if="progress.state === 'downloading' || progress.state === 'verifying'"
            class="text-accent"
          />
          <CircleX v-else-if="progress.state === 'error'" class="size-(--knovum-icon-size-sm) text-danger" />
          <component
            :is="installed ? CheckCircle2 : CircleAlert"
            v-else
            class="size-(--knovum-icon-size-sm)"
            :class="installed ? 'text-success' : 'text-warning'"
          />
          <span class="text-body-lg font-medium text-foreground">{{ title }}</span>
        </div>

        <div class="mt-0.5 text-caption text-foreground-secondary">
          <template v-if="progress.state === 'downloading'">
            {{
              t("settings.offlineModels.download.downloading", {
                downloaded: formatBytes(progress.downloaded),
                total: formatBytes(progress.total),
              })
            }}
          </template>
          <template v-else-if="progress.state === 'verifying'">
            {{ t("settings.offlineModels.download.verifying") }}
          </template>
          <template v-else-if="progress.state === 'error'">
            <span class="text-danger">{{
              progress.error || t("settings.offlineModels.download.failed")
            }}</span>
          </template>
          <template v-else>
            {{
              installedCount !== undefined && totalCount !== undefined
                ? t("settings.offlineModels.models.installed", {
                    installed: installedCount,
                    total: totalCount,
                  })
                : description
            }}
            <template v-if="size"> · {{ formatBytes(size) }}</template>
          </template>
        </div>
      </div>

      <div class="flex shrink-0 items-center gap-3">
        <span
          v-if="progress.state === 'downloading'"
          class="text-caption font-medium text-accent tabular-nums"
        >
          {{ progress.progress }}%
        </span>
        <UiIconButton
          v-if="progress.state === 'downloading' || progress.state === 'verifying'"
          :label="t('settings.offlineModels.download.cancel')"
          size="sm"
          @click="emit('cancel')"
        >
          <Square class="size-(--knovum-icon-size-xs)" />
        </UiIconButton>
        <UiButton
          v-else-if="progress.state === 'error'"
          size="sm"
          variant="danger"
          :disabled="busy"
          @click="emit('download')"
        >
          {{ t("settings.offlineModels.download.retry") }}
        </UiButton>
        <UiButton v-else-if="!installed" :disabled="busy" size="sm" @click="emit('download')">
          {{ t("settings.offlineModels.download.action") }}
        </UiButton>
        <UiIconButton
          v-else
          :label="t('settings.offlineModels.models.remove')"
          :disabled="busy"
          size="sm"
          @click="emit('remove')"
        >
          <Trash2 class="size-(--knovum-icon-size-sm)" />
        </UiIconButton>
      </div>
    </div>

    <UiProgress
      v-if="progress.state === 'downloading' || progress.state === 'verifying'"
      :value="progress.state === 'verifying' ? 100 : progress.progress"
      :label="title"
      class="mt-2.5"
    />
  </div>
</template>
