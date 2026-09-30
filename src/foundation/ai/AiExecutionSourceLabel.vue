<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import type { ExecutionSource } from "../../shared/contracts/generated/ExecutionSource";
import { useExecutionCatalogQuery } from "./execution-catalog-query";

const props = defineProps<{ source: ExecutionSource }>();
const { t } = useI18n();
const catalog = useExecutionCatalogQuery({ detectInstallations: false });
const label = computed(() => {
  const source = props.source;
  if (source.type === "model") return t("ai.executedWithModel", source);
  const name =
    catalog.data.value?.agents.find(({ kind }) => kind === source.runtimeKind)?.displayName ??
    source.runtimeKind;
  return t("ai.executedWithAgent", { name });
});
</script>

<template>
  <span>{{ label }}</span>
</template>
