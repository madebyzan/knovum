<script setup lang="ts">
import { RefreshCw } from "@lucide/vue";
import { camelCase } from "es-toolkit";
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import UiButton from "../../../../shared/ui/UiButton.vue";
import { formatByteSize } from "../../assets/format-byte-size";
import { useNodeOperationGate } from "../../canvas/node-operation-gate";
import { mediaProviderDisplayName } from "../../media/provider-display-name";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import {
  isSpaceManagedAssetPayload,
  isSpaceMediaPayload,
  isSpaceWebsitePayload,
} from "../../model/nodes/capabilities";
import type { SpaceNodePayload } from "../../model/nodes/node-payload";
import { useSpaceStore } from "../../session/space-store";
import { refreshSavedWebsite } from "../../website/capture";

interface MetadataField {
  label: string;
  value: string | undefined;
  multiline?: boolean;
}

const props = defineProps<{ node: SpaceCanvasNode }>();
const store = useSpaceStore();
const nodeOperations = useNodeOperationGate();
const { d, t } = useI18n();
const refreshing = ref(false);

const fields = computed(() => {
  const data = props.node.data;
  const asset = isSpaceManagedAssetPayload(data) ? data : undefined;
  const media = isSpaceMediaPayload(data) ? data : undefined;
  const source = sourceField(data);
  const allFields: MetadataField[] = [
    { label: t("spaces.inspector.fields.type"), value: t(`spaces.nodeTypes.${camelCase(data.kind)}`) },
    source,
    ...websiteFields(data),
    {
      label: t("spaces.inspector.fields.provider"),
      value: media?.origin ? mediaProviderDisplayName(media.origin.provider) : undefined,
    },
    { label: t("spaces.importExport.fileName"), value: asset?.fileName },
    {
      label: t("spaces.inspector.fields.storage"),
      value: asset ? t("spaces.importExport.localPrivateCopy") : undefined,
    },
    { label: t("spaces.inspector.fields.size"), value: formatByteSize(asset?.size) },
    {
      label: t("spaces.inspector.fields.modified"),
      value: modifiedAtLabel(asset?.modifiedAt),
    },
  ];
  return allFields.filter((row): row is MetadataField & { value: string } => Boolean(row.value));
});

function sourceField(data: SpaceNodePayload) {
  const originalUrl = isSpaceMediaPayload(data) && Boolean(data.origin);
  return {
    label: t(originalUrl ? "spaces.inspector.fields.originalUrl" : "spaces.inspector.fields.source"),
    value: sourceUrl(data),
  };
}

function websiteFields(data: SpaceNodePayload) {
  if (!isSpaceWebsitePayload(data)) return [];
  return [
    { label: t("spaces.inspector.fields.siteName"), value: data.siteName },
    { label: t("spaces.inspector.fields.description"), value: data.description, multiline: true },
    { label: t("spaces.inspector.fields.canonicalUrl"), value: data.canonicalUrl },
  ];
}

function modifiedAtLabel(value: number | undefined) {
  return value === undefined ? undefined : d(value, "short");
}

function sourceUrl(data: SpaceNodePayload) {
  if (isSpaceWebsitePayload(data)) return data.url;
  return isSpaceMediaPayload(data) ? data.origin?.url : undefined;
}

async function refreshWebsite() {
  if (
    !nodeOperations.allows("edit") ||
    props.node.data.kind !== "website" ||
    !props.node.data.url ||
    refreshing.value
  )
    return;
  const activeSpaceId = store.activeSpaceId;
  if (!activeSpaceId) return;
  refreshing.value = true;
  try {
    await refreshSavedWebsite(
      {
        activeSpaceId,
        nodes: store.nodes,
        addContent: store.addContent,
        saveNodeContentForSpace: store.saveNodeContentForSpace,
      },
      props.node.id,
      props.node.data.url,
    );
  } catch (error) {
    toast.error(t("spaces.importExport.website.refreshFailed"), { description: String(error) });
  } finally {
    refreshing.value = false;
  }
}
</script>

<template>
  <dl class="space-y-0.5">
    <div v-for="row in fields" :key="row.label" class="grid grid-cols-[4.5rem_minmax(0,1fr)] gap-2 py-1.5">
      <dt class="text-caption text-foreground-tertiary">{{ row.label }}</dt>
      <dd class="min-w-0 text-caption text-foreground-secondary">
        <span class="block" :class="row.multiline ? 'line-clamp-3' : 'truncate'" :title="row.value">{{
          row.value
        }}</span>
      </dd>
    </div>
  </dl>
  <div v-if="node.data.kind === 'website'" class="mt-2">
    <UiButton
      size="sm"
      variant="secondary"
      :disabled="refreshing || !nodeOperations.allows('edit')"
      @click="refreshWebsite"
    >
      <RefreshCw class="size-(--knovum-icon-size-xs)" :class="refreshing && 'animate-spin'" />
      {{
        node.data.metadataState === "failed" ? t("common.retry") : t("spaces.importExport.website.refresh")
      }}
    </UiButton>
  </div>
</template>
