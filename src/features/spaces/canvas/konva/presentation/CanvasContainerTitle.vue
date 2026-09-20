<script setup lang="ts">
import { EditableArea, EditableInput, EditablePreview, EditableRoot } from "reka-ui";
import { nextTick, ref, useId, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";
const props = defineProps<{ title: string; editing: boolean; save(title: string): Promise<void> }>();
const emit = defineEmits<{ finish: [] }>();
const { t } = useI18n();
const field = useTemplateRef<InstanceType<typeof EditableRoot>>("field");
const draft = ref(props.title),
  saving = ref(false),
  error = ref("");
const errorId = useId();
watch(
  () => props.editing,
  async (active) => {
    if (!active) return;
    draft.value = props.title;
    error.value = "";
    await nextTick();
    field.value?.edit();
  },
  { immediate: true },
);
async function save() {
  if (saving.value) return;
  const title = draft.value.trim();
  if (!title || title.length > 512) {
    error.value = t("spaces.structure.renameInvalid");
    return;
  }
  saving.value = true;
  error.value = "";
  try {
    await props.save(title);
    field.value?.submit();
    emit("finish");
  } catch (cause) {
    error.value = `${t("spaces.structure.saveFailed")} ${String(cause)}`;
  } finally {
    saving.value = false;
  }
}
function key(event: KeyboardEvent) {
  event.stopPropagation();
  if (event.isComposing || saving.value) return;
  if (event.key === "Enter") {
    event.preventDefault();
    void save();
  }
}
function blur() {
  if (!saving.value && !error.value) field.value?.cancel();
}
</script>
<template>
  <EditableRoot
    v-if="editing"
    ref="field"
    v-model="draft"
    activation-mode="none"
    submit-mode="none"
    select-on-focus
    :readonly="saving"
    :max-length="512"
    class="pointer-events-auto min-w-0 flex-1"
    @pointerdown.stop
    @dblclick.stop
    @update:state="(state) => state === 'cancel' && emit('finish')"
  >
    <EditableArea class="w-full">
      <EditablePreview class="truncate text-label font-medium" />
      <EditableInput
        :aria-label="t('spaces.structure.rename')"
        :aria-invalid="Boolean(error)"
        :aria-describedby="error ? errorId : undefined"
        class="h-7 w-full min-w-0 rounded-control border border-accent bg-panel px-1 text-label text-foreground outline-none"
        @keydown="key"
        @blur="blur"
      />
    </EditableArea>
    <p
      v-if="error"
      :id="errorId"
      role="alert"
      class="absolute top-full left-0 mt-1 max-w-sm rounded-control border border-border bg-overlay p-2 text-caption text-danger"
    >
      {{ error }}
    </p>
  </EditableRoot>
  <strong v-else class="min-w-0 flex-1 truncate text-label font-medium text-foreground" :title="title">{{
    title
  }}</strong>
</template>
