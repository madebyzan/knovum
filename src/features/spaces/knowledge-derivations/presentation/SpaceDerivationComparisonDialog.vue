<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import UiDialog from "../../../../shared/ui/UiDialog.vue";
import { spaceAssetSource } from "../../assets/node-preview-source";
import type { DerivationComparisonInput } from "../../contracts/generated/DerivationComparisonInput";
import type { SpaceImagePayload } from "../../model/nodes/node-payload";
import SpaceImageComparisonViewer from "./SpaceImageComparisonViewer.vue";

const open = defineModel<boolean>("open", { required: true });
const props = defineProps<{
  output: SpaceImagePayload;
  input: DerivationComparisonInput | undefined;
  error: unknown;
}>();
const { t } = useI18n();
const before = computed(() => props.input && spaceAssetSource(props.input.path, props.input.revision));
const after = computed(() => spaceAssetSource(props.output.assetPath, props.output.sourceRevision));
</script>

<template>
  <UiDialog
    v-model:open="open"
    :title="t('spaces.media.imageComparison.title')"
    :description="t('spaces.media.imageComparison.hint')"
    size="wide"
    class="h-[calc(100vh-5rem)] max-w-[calc(100vw-5rem)]"
    body-class="relative overflow-hidden p-0"
  >
    <SpaceImageComparisonViewer v-if="before" :before="before" :after="after" />
    <p v-else class="grid size-full place-items-center text-caption text-fg-secondary">
      {{ error ? getErrorMessage(error) : t("spaces.media.imageComparison.preparing") }}
    </p>
  </UiDialog>
</template>
