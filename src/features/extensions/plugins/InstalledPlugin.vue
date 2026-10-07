<script setup lang="ts">
import { ChevronDown, MoreHorizontal, RefreshCw, Trash2 } from "@lucide/vue";

import type { PluginInstallation } from "../../../shared/contracts/generated/PluginInstallation";
import UiBadge from "../../../shared/ui/UiBadge.vue";
import UiDropdownMenu from "../../../shared/ui/UiDropdownMenu.vue";
import UiDropdownMenuItem from "../../../shared/ui/UiDropdownMenuItem.vue";
import UiIconButton from "../../../shared/ui/UiIconButton.vue";
import UiSwitch from "../../../shared/ui/UiSwitch.vue";

defineProps<{ plugin: PluginInstallation; busy: boolean }>();
const emit = defineEmits<{ toggle: []; update: []; uninstall: [] }>();
</script>

<template>
  <section :data-testid="`plugin-${plugin.id}`" class="min-h-(--knovum-setting-min-height) min-w-0 px-4 py-3">
    <div class="flex items-start gap-3">
      <div class="min-w-0 flex-1">
        <div class="flex flex-wrap items-center gap-x-2 gap-y-1">
          <h3 class="text-body-lg font-medium break-words text-fg-primary">
            {{ plugin.revision.manifest.name }}
          </h3>
          <span v-if="plugin.revision.manifest.version" class="text-caption text-fg-tertiary">{{
            plugin.revision.manifest.version
          }}</span>
          <UiBadge v-if="plugin.builtin" size="sm">{{ $t("extensions.builtin") }}</UiBadge>
        </div>
        <p
          v-if="plugin.revision.manifest.description"
          class="mt-0.5 text-caption leading-normal text-fg-secondary"
        >
          {{ plugin.revision.manifest.description }}
        </p>
        <p class="mt-1.5 text-caption text-fg-tertiary">
          {{ $t("extensions.componentCount", { count: plugin.revision.components.length }) }}
        </p>
      </div>
      <UiSwitch
        :model-value="plugin.enabled"
        :label="
          $t(plugin.enabled ? 'extensions.disableNamed' : 'extensions.enableNamed', {
            name: plugin.revision.manifest.name,
          })
        "
        :disabled="busy"
        :data-testid="`toggle-plugin-${plugin.id}`"
        class="mt-1"
        @update:model-value="emit('toggle')"
      />
      <div class="w-8 shrink-0">
        <UiDropdownMenu v-if="!plugin.builtin">
          <template #trigger>
            <UiIconButton
              :label="$t('extensions.pluginActions')"
              :tooltip="false"
              :disabled="busy"
              :data-testid="`plugin-actions-${plugin.id}`"
              ><MoreHorizontal class="size-4"
            /></UiIconButton>
          </template>
          <UiDropdownMenuItem
            :data-testid="`update-plugin-${plugin.id}`"
            :disabled="busy"
            @select="emit('update')"
            ><RefreshCw class="size-3.5" />{{ $t("extensions.checkUpdate") }}</UiDropdownMenuItem
          >
          <UiDropdownMenuItem
            destructive
            :data-testid="`uninstall-plugin-${plugin.id}`"
            :disabled="busy"
            @select="emit('uninstall')"
            ><Trash2 class="size-3.5" />{{ $t("extensions.uninstall") }}</UiDropdownMenuItem
          >
        </UiDropdownMenu>
      </div>
    </div>
    <details class="group mt-1 text-caption text-fg-secondary">
      <summary
        class="flex w-fit cursor-default list-none items-center gap-1 rounded-sm py-1 outline-none hover:text-fg-primary focus-visible:outline-2 focus-visible:outline-stroke-focus"
      >
        <ChevronDown
          class="size-3 transition-transform group-open:rotate-180 motion-reduce:transition-none"
        />{{ $t("extensions.details") }}
      </summary>
      <dl class="mt-2 grid min-w-0 gap-2 rounded-md bg-surface-muted p-3">
        <div>
          <dt class="text-fg-tertiary">{{ $t("extensions.source") }}</dt>
          <dd class="mt-0.5 break-all">
            {{
              plugin.source.type === "directory" || plugin.source.type === "zip"
                ? plugin.source.path
                : plugin.source.url
            }}
          </dd>
        </div>
        <div v-if="plugin.publication">
          <dt class="text-fg-tertiary">{{ $t("extensions.publisherUnverified") }}</dt>
          <dd class="mt-0.5 break-all">{{ plugin.publication.publisherId }}</dd>
        </div>
        <div>
          <dt class="text-fg-tertiary">{{ $t("extensions.sourceRevision") }}</dt>
          <dd class="mt-0.5 font-mono break-all">{{ plugin.revision.sourceRevision }}</dd>
        </div>
      </dl>
    </details>
  </section>
</template>
