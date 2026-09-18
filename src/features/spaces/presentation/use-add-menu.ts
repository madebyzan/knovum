import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { importSpaceFromPicker } from "../assets/import-flow";
import { createSystemKnowledgeObjectPayload } from "../knowledge/model/object-factory";
import { useSpaceStore } from "../session/space-store";
import { addSavedWebsite } from "../website/capture";
import { normalizeSpaceUrl } from "../website/url";

export function useAddMenu() {
  const store = useSpaceStore();
  const { t } = useI18n();
  const websiteOpen = ref(false);
  const customObjectOpen = ref(false);
  const typeManagerOpen = ref(false);
  const portalOpen = ref(false);
  const agentCreationOpen = ref(false);
  const website = ref("");
  const normalizedWebsite = computed(() => normalizeSpaceUrl(website.value));
  const addMarkdown = () =>
    void store.addItem({ kind: "markdown", title: t("spaces.editor.untitledMarkdown"), text: "" });
  const addKnowledgeObject = (
    type: "claim" | "evidence" | "question" | "decision" | "concept" | "experiment",
  ) => void store.addItem(createSystemKnowledgeObjectPayload(type, t(`spaces.knowledgeObjectTypes.${type}`)));
  function addCredential(kind: "api-key" | "account-password") {
    void store.addItem({
      kind: "credential",
      title: kind === "api-key" ? t("spaces.credentials.apiKey") : t("spaces.credentials.accountPassword"),
      credentialKind: kind,
      credentialConfigured: false,
    });
  }
  async function importFromPicker() {
    if (!store.activeSpaceId) return;
    try {
      await importSpaceFromPicker(store, {
        title: t("spaces.transfer.importingItem"),
        preparing: t("spaces.transfer.importPreparing"),
      });
    } catch (error) {
      toast.error(t("spaces.transfer.importFailed"), { description: String(error) });
    }
  }
  async function addWebsite() {
    const url = normalizedWebsite.value;
    const activeSpaceId = store.activeSpaceId;
    if (!url || !activeSpaceId) return;
    await addSavedWebsite(
      {
        activeSpaceId,
        nodes: store.nodes,
        addItem: store.addItem,
        saveNodeDataForSpace: store.saveNodeDataForSpace,
      },
      url,
    );
    website.value = "";
    websiteOpen.value = false;
  }
  return {
    store,
    websiteOpen,
    customObjectOpen,
    typeManagerOpen,
    portalOpen,
    agentCreationOpen,
    website,
    normalizedWebsite,
    addMarkdown,
    addKnowledgeObject,
    addCredential,
    importFromPicker,
    addWebsite,
  };
}
