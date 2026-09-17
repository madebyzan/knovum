<script setup lang="ts">
import { Plus, Trash2 } from "@lucide/vue";
import { ask } from "@tauri-apps/plugin-dialog";
import { shallowRef, toRef } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiDialog from "../../../../shared/ui/UiDialog.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import UiScrollArea from "../../../../shared/ui/UiScrollArea.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
import type { SpaceObjectFieldDefinition } from "../../knowledge/model/object-types";
import { useKnowledgeTypeManager } from "../../knowledge/types/use-knowledge-type-manager";
import SpaceKnowledgeIconSelect from "./SpaceKnowledgeIconSelect.vue";
import SpaceKnowledgeTypeFieldRow from "./SpaceKnowledgeTypeFieldRow.vue";

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
} = useKnowledgeTypeManager(toRef(() => open.value));
const fieldIds = new WeakMap<SpaceObjectFieldDefinition, string>();
function fieldIdentity(field: SpaceObjectFieldDefinition) {
  let id = fieldIds.get(field);
  if (!id) {
    id = crypto.randomUUID();
    fieldIds.set(field, id);
  }
  return id;
}
const confirming = shallowRef(false);
async function changeEditor(action: () => void) {
  if (saving.value || confirming.value) return;
  confirming.value = true;
  try {
    if (
      !dirty.value ||
      (await ask(t("spaces.knowledge.discardTypeChanges"), {
        title: t("spaces.knowledge.manageKnowledgeTypes"),
        kind: "warning",
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
    :title="t('spaces.knowledge.manageKnowledgeTypes')"
    :description="t('spaces.knowledge.manageKnowledgeTypesDescription')"
    size="wide"
    class="h-[min(40rem,calc(100dvh-5rem))] max-w-[840px] bg-canvas"
    body-class="flex overflow-hidden"
  >
    <div class="grid min-h-0 flex-1 grid-cols-[10rem_minmax(0,1fr)] gap-5">
      <aside class="flex min-h-0 min-w-0 flex-col gap-0.5 rounded-control bg-surface p-2">
        <p v-if="loadFailed" role="alert" class="px-2 py-1 text-caption text-danger">
          {{ t("spaces.knowledge.knowledgeTypesLoadFailed") }}
        </p>
        <p v-else-if="loading" class="px-2 py-1 text-caption text-foreground-secondary">
          {{ t("common.loading") }}
        </p>
        <button
          type="button"
          :disabled="saving"
          class="flex h-(--knovum-control-height-md) w-full shrink-0 items-center gap-1 rounded-control px-2 text-left text-label text-foreground-secondary outline-none hover:bg-control-hover hover:text-foreground focus-visible:bg-control-hover"
          @click="changeEditor(reset)"
        >
          <Plus class="mr-1 inline size-(--knovum-icon-size-xs)" />{{
            t("spaces.knowledge.newKnowledgeType")
          }}
        </button>
        <UiScrollArea class="min-h-0 flex-1">
          <div class="space-y-0.5">
            <div
              v-for="type in types"
              :key="type.id"
              class="group flex items-center justify-between rounded-control px-2 text-label transition-colors duration-100 hover:bg-control-hover"
              :class="
                editingId === type.id
                  ? 'bg-(--knovum-sidebar-item-background-selected) text-foreground'
                  : 'text-foreground-secondary'
              "
            >
              <button
                type="button"
                :disabled="saving"
                :aria-current="editingId === type.id ? 'true' : undefined"
                class="h-(--knovum-control-height-md) min-w-0 flex-1 truncate text-left outline-none focus-visible:text-accent"
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
            <label class="grid min-w-0 content-start gap-1 text-caption text-foreground-secondary">
              <span>{{ t("spaces.knowledge.knowledgeTypeName") }}</span>
              <UiInput v-model="name" maxlength="128" />
            </label>
            <label class="grid min-w-0 content-start gap-1 text-caption text-foreground-secondary">
              <span>{{ t("spaces.knowledge.knowledgeTypeIcon") }}</span>
              <SpaceKnowledgeIconSelect v-model="icon" :disabled="saving" />
            </label>
          </div>
          <label class="mt-3 grid gap-1 text-caption text-foreground-secondary">
            <span>{{ t("spaces.knowledge.knowledgeTypeDescription") }}</span>
            <UiTextarea v-model="description" :rows="2" maxlength="4096" class="resize-none" />
          </label>
          <div class="mt-4">
            <h3 class="mb-2 text-label font-medium">{{ t("spaces.knowledge.knowledgeField") }}</h3>
            <p v-if="!fields.length" class="py-3 text-caption text-foreground-secondary">
              {{ t("spaces.knowledge.emptyTypeFields") }}
            </p>
            <div class="overflow-x-auto">
              <div
                class="grid min-w-[30rem] grid-cols-[minmax(0,1fr)_minmax(0,0.8fr)_8rem_3rem_var(--knovum-control-height-md)] items-center gap-2"
              >
                <template v-if="fields.length">
                  <span class="text-caption text-foreground-secondary">{{
                    t("spaces.knowledge.knowledgeFieldLabel")
                  }}</span>
                  <span class="text-caption text-foreground-secondary">{{
                    t("spaces.knowledge.knowledgeFieldKey")
                  }}</span>
                  <span class="text-caption text-foreground-secondary">{{
                    t("spaces.knowledge.knowledgeFieldKind")
                  }}</span>
                  <span class="text-center text-caption text-foreground-secondary">{{
                    t("spaces.knowledge.knowledgeFieldRequired")
                  }}</span>
                  <span aria-hidden="true"></span>
                </template>
                <SpaceKnowledgeTypeFieldRow
                  v-for="(field, index) in fields"
                  :key="fieldIdentity(field)"
                  :field="field"
                  :disabled="saving"
                  :field-kinds="fieldKinds"
                  @kind="updateKind(field, $event)"
                  @options="updateOptions(field, $event)"
                  @required="updateRequired(field, $event)"
                  @label="field.label = $event"
                  @key="field.key = $event"
                  @remove="fields.splice(index, 1)"
                />
              </div>
            </div>
            <UiButton
              class="mt-2"
              :disabled="fields.length >= 64"
              size="sm"
              variant="ghost"
              @click="addField"
            >
              <Plus class="size-(--knovum-icon-size-xs)" />{{ t("spaces.knowledge.addKnowledgeField") }}
            </UiButton>
          </div>
          <p v-if="error" role="alert" class="mt-3 text-caption text-danger">{{ error }}</p>
          <UiButton v-if="editingId" class="mt-5 text-danger" size="sm" variant="ghost" @click="remove">
            <Trash2 class="size-(--knovum-icon-size-sm)" />{{ t("spaces.knowledge.deleteKnowledgeType") }}
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
