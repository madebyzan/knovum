<script setup lang="ts">
import { Plus } from "@lucide/vue";
import { RouterLink } from "vue-router";

import UiButton from "../../../../shared/ui/UiButton.vue";

defineProps<{ empty: boolean; disabled: boolean }>();
const emit = defineEmits<{ add: [] }>();
</script>

<template>
  <section class="shrink-0 space-y-2 border-b border-stroke-divider px-3 py-2">
    <p v-if="empty" class="text-caption text-fg-secondary" role="status">
      {{ $t("spaces.speech.library.selection.empty") }}
    </p>
    <div class="flex items-center gap-2">
      <RouterLink :to="{ path: '/settings', query: { pane: 'voices' } }" custom v-slot="{ href, navigate }">
        <UiButton as="a" :href="href" size="sm" variant="ghost" data-testid="manage-voices" @click="navigate">
          {{ $t("spaces.speech.library.collection.title") }}
        </UiButton>
      </RouterLink>
      <UiButton
        size="sm"
        variant="ghost"
        :disabled="disabled"
        data-testid="add-reference-voice"
        @click="emit('add')"
      >
        <Plus class="size-3.5" />{{ $t("spaces.speech.library.add") }}
      </UiButton>
      <slot />
    </div>
  </section>
</template>
