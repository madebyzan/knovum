import { useQueryClient } from "@tanstack/vue-query";
import { compact, isEqual } from "es-toolkit";
import { storeToRefs } from "pinia";
import { computed, ref, shallowRef, watch, type Ref } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { confirmAction } from "../../../../shared/ui/confirm-action";
import { useSpaceStore } from "../../session/space-store";
import type { SpaceObjectFieldDefinition, SpaceObjectFieldKind } from "../model/object-types";
import { createObjectType, deleteObjectType, updateObjectType } from "../persistence/knowledge-operations";
import { spaceKnowledgeQueryKeys } from "../queries/query-options";
import { useSpaceObjectTypesQuery } from "../queries/use-knowledge-queries";

export function useKnowledgeTypeEditor(open: Ref<boolean>) {
  const queryClient = useQueryClient();
  const { activeSpaceId } = storeToRefs(useSpaceStore());
  const { t } = useI18n();
  const typesQuery = useSpaceObjectTypesQuery(activeSpaceId);
  const types = computed(() => (typesQuery.data.value ?? []).filter((type) => !type.system));
  const editingId = ref<string>();
  const name = ref("");
  const icon = ref("Shapes");
  const description = ref("");
  const fields = ref<SpaceObjectFieldDefinition[]>([]);
  const saving = shallowRef(false);
  const error = shallowRef("");
  const fingerprint = () => ({
    name: name.value.trim(),
    icon: icon.value,
    description: description.value.trim(),
    fields: snapshotFields(fields.value),
  });
  const baseline = shallowRef<ReturnType<typeof fingerprint>>();
  const dirty = computed(() => baseline.value !== undefined && !isEqual(baseline.value, fingerprint()));
  const fieldKinds: SpaceObjectFieldKind[] = ["text", "number", "boolean", "date", "node-ref", "enum"];
  function refreshTypes(spaceId: string) {
    return queryClient.invalidateQueries({ queryKey: spaceKnowledgeQueryKeys.objectTypes(spaceId) });
  }
  watch(activeSpaceId, reset);
  watch(open, (value) => {
    if (value) {
      reset();
      void typesQuery.refetch();
    }
  });
  function reset() {
    if (saving.value) return;
    error.value = "";
    editingId.value = undefined;
    name.value = "";
    icon.value = "Shapes";
    description.value = "";
    fields.value = [];
    baseline.value = fingerprint();
  }
  function selectType(objectTypeId: string) {
    if (saving.value) return;
    error.value = "";
    const type = types.value.find((knowledgeType) => knowledgeType.id === objectTypeId);
    if (!type) return reset();
    editingId.value = type.id;
    name.value = type.name;
    icon.value = type.icon;
    description.value = type.description;
    fields.value = snapshotFields(type.fields);
    baseline.value = fingerprint();
  }
  function addField() {
    let index = 1;
    while (fields.value.some((field) => field.key === `field${index}`)) index++;
    fields.value.push({
      key: `field${index}`,
      label: `${t("spaces.knowledge.fields.title")} ${index}`,
      kind: "text",
    });
  }
  async function save() {
    if (saving.value) return;
    const spaceId = activeSpaceId.value;
    if (!spaceId || !name.value.trim()) return;
    error.value = "";
    const keys = fields.value.map((field) => field.key.trim());
    if (
      fields.value.some((field) => !field.label.trim() || !field.key.trim()) ||
      new Set(keys).size !== keys.length
    ) {
      error.value = t("spaces.knowledge.fields.invalid");
      return;
    }
    const input = {
      id: editingId.value ?? crypto.randomUUID(),
      spaceId,
      name: name.value.trim(),
      icon: icon.value,
      description: description.value.trim(),
      fields: snapshotFields(fields.value),
    };
    saving.value = true;
    try {
      if (editingId.value) await updateObjectType(input);
      else await createObjectType(input);
      if (activeSpaceId.value === spaceId) {
        editingId.value = input.id;
        baseline.value = fingerprint();
      }
      await refreshTypes(spaceId);
    } catch (cause) {
      error.value = getErrorMessage(cause);
    } finally {
      saving.value = false;
    }
  }
  async function remove() {
    const spaceId = activeSpaceId.value;
    if (!spaceId || !editingId.value || saving.value) return;
    const id = editingId.value;
    saving.value = true;
    try {
      if (
        !(await confirmAction({
          title: t("common.delete"),
          description: t("spaces.knowledge.types.deleteConfirmation", { name: name.value }),
          confirmLabel: t("common.delete"),
          danger: true,
        }))
      )
        return;
      await deleteObjectType(id, spaceId);
      await refreshTypes(spaceId);
      saving.value = false;
      if (activeSpaceId.value === spaceId) reset();
    } catch (cause) {
      toast.error(getErrorMessage(cause));
    } finally {
      saving.value = false;
    }
  }
  return {
    types,
    loading: typesQuery.isPending,
    loadFailed: typesQuery.isError,
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
  };
}

function updateKind(field: SpaceObjectFieldDefinition, kind: SpaceObjectFieldKind) {
  field.kind = kind;
  if (kind === "enum") field.options = [];
  else delete field.options;
}

function updateOptions(field: SpaceObjectFieldDefinition, value: string) {
  field.options = compact(value.split(",").map((option) => option.trim()));
}

function updateRequired(field: SpaceObjectFieldDefinition, required: boolean) {
  if (required) field.required = true;
  else delete field.required;
}

function snapshotFields(fields: readonly SpaceObjectFieldDefinition[]) {
  return fields.map((field) => ({
    key: field.key.trim(),
    label: field.label.trim(),
    kind: field.kind,
    ...(field.required ? { required: true } : {}),
    ...(field.options ? { options: [...field.options] } : {}),
  }));
}
