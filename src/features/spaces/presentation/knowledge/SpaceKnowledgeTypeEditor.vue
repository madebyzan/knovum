<script setup lang="ts">
import { Plus, Trash2 } from "@lucide/vue";
import { shallowRef, toRef } from "vue";
import { useI18n } from "vue-i18n";

import { confirmAction } from "../../../../shared/ui/confirm-action";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiDialog from "../../../../shared/ui/UiDialog.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import UiScrollArea from "../../../../shared/ui/UiScrollArea.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
import { useKnowledgeTypeEditor } from "../../knowledge/object-types/use-knowledge-type-editor";
import SpaceKnowledgeIconSelect from "./SpaceKnowledgeIconSelect.vue";
import SpaceKnowledgeTypeFields from "./SpaceKnowledgeTypeFields.vue";

const open = defineModel<boolean>("open", { required: true });
const { t } = useI18n();
const {
  types,
  loading,
  loadFailed,
  dirty,
  saving,
  error,
  editingId,
  name,
  icon,
  description,
  fields,
  fieldKinds,
  reset,
  selectType,
  addField,
  updateKind,
  updateOptions,
  updateRequired,
  save,
  remove,
} = useKnowledgeTypeEditor(toRef(() => open.value));
const confirming = shallowRef(false);
async function changeEditor(action: () => void) {
  if (saving.value || confirming.value) return;
  confirming.value = true;
  try {
    if (
      !dirty.value ||
      (await confirmAction({
        title: t("common.discardChanges"),
        description: t("spaces.knowledge.types.discardChanges"),
        confirmLabel: t("common.discardChanges"),
        danger: true,
      }))
    )
      action();
  } finally {
    confirming.value = false;
  }
}
</script>

<template>
  <UiDialog
    :open="open"
    @update:open="
      changeEditor(() => {
        open = $event;
      })
    "
    :title="t('spaces.knowledge.types.manage')"
    :description="t('spaces.knowledge.types.manageDescription')"
    size="wide"
    class="h-[min(40rem,calc(100dvh-5rem))] max-w-[840px] bg-surface-canvas"
    body-class="flex overflow-hidden"
  >
    <div class="grid min-h-0 flex-1 grid-cols-[10rem_minmax(0,1fr)] gap-5">
      <aside class="flex min-h-0 min-w-0 flex-col gap-0.5 rounded-md bg-surface-muted p-2">
        <p v-if="loadFailed" role="alert" class="px-2 py-1 text-caption text-fg-danger">
          {{ t("spaces.knowledge.types.loadFailed") }}
        </p>
        <p v-else-if="loading" class="px-2 py-1 text-caption text-fg-secondary">
          {{ t("common.loading") }}
        </p>
        <button
          type="button"
          :disabled="saving"
          class="flex h-(--knovum-control-height-md) w-full shrink-0 items-center gap-1 rounded-md px-2 text-left text-label text-fg-secondary outline-none hover:bg-surface-control-hover hover:text-fg-primary focus-visible:bg-surface-control-hover"
          @click="changeEditor(reset)"
        >
          <Plus class="mr-1 inline size-3" />{{ t("spaces.knowledge.types.create") }}
        </button>
        <UiScrollArea class="min-h-0 flex-1">
          <div class="space-y-0.5">
            <div
              v-for="type in types"
              :key="type.id"
              class="group flex items-center justify-between rounded-md px-2 text-label transition-colors duration-100 hover:bg-surface-control-hover"
              :class="
                editingId === type.id
                  ? 'bg-(--knovum-sidebar-item-bg-selected) text-fg-primary'
                  : 'text-fg-secondary'
              "
            >
              <button
                type="button"
                :disabled="saving"
                :aria-current="editingId === type.id ? 'true' : undefined"
                class="h-(--knovum-control-height-md) min-w-0 flex-1 truncate text-left outline-none focus-visible:text-fg-accent"
                @click="editingId !== type.id && changeEditor(() => selectType(type.id))"
              >
                {{ type.name }}
              </button>
            </div>
          </div>
        </UiScrollArea>
      </aside>
      <UiScrollArea class="min-h-0 min-w-0">
        <fieldset :disabled="saving" class="min-w-0 pr-3 pb-2">
          <div class="grid grid-cols-[minmax(0,1fr)_9rem] gap-3">
            <label class="grid min-w-0 content-start gap-1 text-caption text-fg-secondary">
              <span>{{ t("spaces.knowledge.types.name") }}</span>
              <UiInput v-model="name" maxlength="128" />
            </label>
            <label class="grid min-w-0 content-start gap-1 text-caption text-fg-secondary">
              <span>{{ t("spaces.knowledge.types.icon") }}</span>
              <SpaceKnowledgeIconSelect v-model="icon" :disabled="saving" />
            </label>
          </div>
          <label class="mt-3 grid gap-1 text-caption text-fg-secondary">
            <span>{{ t("spaces.knowledge.types.description") }}</span>
            <UiTextarea v-model="description" :rows="2" :maxlength="4096" class="resize-none" />
          </label>
          <SpaceKnowledgeTypeFields
            :fields="fields"
            :field-kinds="fieldKinds"
            :disabled="saving"
            @add="addField"
            @kind="updateKind"
            @options="updateOptions"
            @required="updateRequired"
            @label="(field, value) => (field.label = value)"
            @key="(field, value) => (field.key = value)"
            @remove="fields.splice($event, 1)"
          />
          <p v-if="error" role="alert" class="mt-3 text-caption text-fg-danger">{{ error }}</p>
          <UiButton v-if="editingId" class="mt-5 text-fg-danger" size="sm" variant="ghost" @click="remove">
            <Trash2 class="size-3.5" />{{ t("spaces.knowledge.types.delete") }}
          </UiButton>
        </fieldset>
      </UiScrollArea>
    </div>
    <template #footer>
      <UiButton
        variant="ghost"
        :disabled="saving"
        @click="
          changeEditor(() => {
            open = false;
          })
        "
        >{{ t("common.cancel") }}</UiButton
      >
      <UiButton variant="primary" :disabled="saving || !name.trim()" :aria-busy="saving" @click="save">{{
        t("common.save")
      }}</UiButton>
    </template>
  </UiDialog>
</template>
