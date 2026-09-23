<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
import { useWorkflowEditor } from "../../workflows/use-workflow-editor";
import SpaceWorkflowLimits from "./SpaceWorkflowLimits.vue";
import SpaceWorkflowVersionDiff from "./SpaceWorkflowVersionDiff.vue";

const props = defineProps<{ editor: ReturnType<typeof useWorkflowEditor> }>();
const { t } = useI18n();
const testNodeOptions = computed(
  () =>
    props.editor.draft.value?.definition.nodes.map(({ id, kind }) => ({
      value: id,
      label: `${id} · ${kind}`,
    })) ?? [],
);
</script>

<template>
  <p v-if="editor.error.value" class="text-meta px-3 py-2 text-danger">{{ editor.error.value }}</p>
  <ul v-if="editor.diagnostics.value.length" class="text-meta border-t border-divider px-3 py-2 text-danger">
    <li v-for="message in editor.diagnostics.value" :key="message">{{ message }}</li>
  </ul>
  <details v-if="editor.versions.value.length" class="border-t border-divider px-3 py-2">
    <summary class="cursor-pointer text-label">
      {{ t("spaces.assistant.versionHistory") }} · {{ editor.versions.value.length }}
    </summary>
    <ol class="mt-2 max-h-40 space-y-1 overflow-auto">
      <li
        v-for="version in editor.versions.value"
        :key="`${version.workflowId}:${version.version}`"
        class="text-meta flex items-center gap-2"
      >
        <span class="min-w-0 flex-1 truncate"
          >{{ version.definition.title }} · v{{ version.version }} ·
          {{ version.definitionDigest.slice(0, 10) }}</span
        >
        <UiButton size="sm" variant="ghost" @click="editor.copyVersion(version)">{{
          t("spaces.assistant.copyAsDraft")
        }}</UiButton>
      </li>
    </ol>
    <SpaceWorkflowVersionDiff
      v-if="editor.versions.value[0] && editor.versions.value[1]"
      :previous="editor.versions.value[1]"
      :current="editor.versions.value[0]"
    />
  </details>
  <UiTextarea
    v-model="editor.sampleInput.value"
    :rows="2"
    class="m-2 font-mono text-micro"
    :placeholder="t('spaces.assistant.sampleInput')"
  />
  <details v-if="editor.draft.value" class="border-t border-divider px-3 py-2">
    <summary class="cursor-pointer text-label">
      {{ t("spaces.assistant.breakpoints") }} · {{ editor.breakpointNodeIds.value.length }}
    </summary>
    <div class="mt-2 grid grid-cols-2 gap-1">
      <label
        v-for="node in editor.draft.value.definition.nodes"
        :key="node.id"
        class="text-meta flex items-center gap-2 rounded-control px-2 py-1 hover:bg-control"
      >
        <input
          type="checkbox"
          :checked="editor.breakpointNodeIds.value.includes(node.id)"
          @change="editor.toggleBreakpoint(node.id)"
        />
        <span class="truncate">{{ node.id }} · {{ node.kind }}</span>
      </label>
    </div>
  </details>
  <details v-if="editor.draft.value" class="border-t border-divider">
    <summary class="cursor-pointer px-3 py-2 text-label">{{ t("spaces.assistant.runLimits") }}</summary>
    <SpaceWorkflowLimits v-model="editor.draft.value.definition.limits" />
  </details>
  <footer class="flex justify-end gap-1.5 border-t border-divider p-2.5">
    <UiSelect
      v-model="editor.selectedTestNodeId.value"
      class="mr-auto w-48"
      size="sm"
      :label="t('spaces.assistant.testStep')"
      :options="testNodeOptions"
    />
    <UiButton size="sm" :disabled="!editor.draft.value || editor.busy.value" @click="editor.validate">{{
      t("common.validate")
    }}</UiButton>
    <UiButton size="sm" :disabled="!editor.draft.value || editor.busy.value" @click="editor.save">{{
      t("common.save")
    }}</UiButton>
    <UiButton size="sm" :disabled="!editor.draft.value || editor.busy.value" @click="editor.publish">{{
      t("spaces.assistant.publish")
    }}</UiButton>
    <UiButton
      size="sm"
      variant="ghost"
      :disabled="!editor.draft.value || editor.busy.value"
      @click="editor.test"
      >{{ t("spaces.assistant.testWorkflow") }}</UiButton
    >
    <UiButton
      size="sm"
      variant="ghost"
      :disabled="!editor.draft.value || !editor.selectedTestNodeId.value || editor.busy.value"
      @click="editor.testStep"
      >{{ t("spaces.assistant.testStep") }}</UiButton
    >
    <UiButton
      size="sm"
      variant="primary"
      :disabled="!editor.draft.value || editor.busy.value"
      @click="editor.run"
      >{{ t("spaces.assistant.run") }}</UiButton
    >
  </footer>
</template>
