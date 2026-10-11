import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { importSpaceFromPicker } from "../../assets/import-flow";
import { useNodeOperationGate } from "../../canvas/use-node-operation-gate";
import { createSystemKnowledgeObjectPayload } from "../../knowledge/model/object-factory";
import { useSpaceStore } from "../../session/space-store";
import { addSavedWebsite } from "../../website/capture";
import { normalizeSpaceUrl } from "../../website/url";

export function useAddMenu() {
  const store = useSpaceStore();
  const operations = useNodeOperationGate();
  const { t } = useI18n();
  const websiteOpen = ref(false);
  const customObjectOpen = ref(false);
  const typeEditorOpen = ref(false);
  const portalOpen = ref(false);
  const agentCreationOpen = ref(false);
  const website = ref("");
  const normalizedWebsite = computed(() => normalizeSpaceUrl(website.value));
  const addMarkdown = () => {
    if (operations.allows("create"))
      void store.addContent({ kind: "markdown", title: t("spaces.editor.markdown.untitled"), text: "" });
  };
  const addSection = () => {
    if (!operations.allows("create")) return;
    const position = store.nextInsertionPoint();
    const rootOrder = store.editSnapshot.structure.filter(
      (structureNode) => structureNode.parentId === null,
    ).length;
    return store.executeSpaceEdit({
      kind: "section.create-empty",
      sectionId: crypto.randomUUID(),
      title: t("spaces.canvas.sections.untitled"),
      frame: { ...position, width: 640, height: 400 },
      rootOrder,
    });
  };
  const addKnowledgeObject = (
    type: "claim" | "evidence" | "question" | "decision" | "concept" | "experiment",
  ) => {
    if (operations.allows("create"))
      void store.addContent(
        createSystemKnowledgeObjectPayload(type, t(`spaces.knowledgeObjectTypes.${type}`)),
      );
  };
  function addCredential(kind: "api-key" | "account-password") {
    if (!operations.allows("create")) return;
    void store.addContent({
      kind: "credential",
      title: kind === "api-key" ? t("spaces.credentials.apiKey") : t("spaces.credentials.accountPassword"),
      credentialKind: kind,
      credentialConfigured: false,
    });
  }
  async function importFromPicker() {
    if (!store.activeSpaceId || !operations.allows("create")) return;
    try {
      await importSpaceFromPicker(store, {
        title: t("spaces.importExport.progress.resource"),
        preparing: t("spaces.importExport.progress.preparing"),
      });
    } catch (error) {
      toast.error(t("spaces.importExport.import.failed"), { description: getErrorMessage(error) });
    }
  }
  async function addWebsite() {
    const url = normalizedWebsite.value;
    const activeSpaceId = store.activeSpaceId;
    if (!url || !activeSpaceId || !operations.allows("create")) return;
    await addSavedWebsite(
      {
        activeSpaceId,
        nodes: store.nodes,
        addContent: store.addContent,
        saveNodeContentForSpace: store.saveNodeContentForSpace,
      },
      url,
    );
    website.value = "";
    websiteOpen.value = false;
  }
  return {
    websiteOpen,
    customObjectOpen,
    typeEditorOpen,
    portalOpen,
    agentCreationOpen,
    website,
    normalizedWebsite,
    addMarkdown,
    addSection,
    addKnowledgeObject,
    addCredential,
    importFromPicker,
    addWebsite,
  };
}
