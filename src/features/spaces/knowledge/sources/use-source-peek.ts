import { useQuery } from "@tanstack/vue-query";
import { platform } from "@tauri-apps/plugin-os";
import { whenever } from "@vueuse/core";
import { storeToRefs } from "pinia";
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { openSpaceAsset, openSpaceAssetAtLocation } from "../../assets/asset-operations";
import { useCanvasEditorSession } from "../../canvas/editor-session";
import { createSourceEvidence } from "../../editor/knowledge-capture";
import { isSpaceManagedAssetPayload } from "../../model/nodes/capabilities";
import { useSpaceStore } from "../../session/space-store";
import type { KnowledgeChunk } from "../model/knowledge-chunk";
import { listCurrentKnowledgeChunks } from "../persistence/chunk-operations";
import { knowledgeLocationLabel } from "../presentation/location-label";

const DOCUMENT_CONTEXT_KINDS = new Set<string>(["pdf", "word", "slides", "sheet"]);

type SourcePeekTarget = NonNullable<ReturnType<typeof useSpaceStore>["sourcePeek"]>;
type DocumentContextRequest = {
  spaceId: string;
  nodeId: string;
  location: SourcePeekTarget["locator"]["location"];
};

export function useSourcePeek() {
  const store = useSpaceStore();
  const { sourcePeek } = storeToRefs(store);
  const { t } = useI18n();
  const node = computed(() => store.nodes.find((candidate) => candidate.id === sourcePeek.value?.nodeId));
  const location = computed(() => sourcePeek.value?.locator.location);
  const preciseExternal = computed(
    () => platform() === "windows" && ["word", "slides", "sheet"].includes(location.value?.kind ?? ""),
  );
  const editableSource = computed(() => ["note", "markdown"].includes(node.value?.data.kind ?? ""));
  const hasAsset = computed(() => Boolean(node.value && isSpaceManagedAssetPayload(node.value.data)));
  const contextRequest = computed<DocumentContextRequest | undefined>(() => {
    const target = sourcePeek.value;
    const spaceId = store.activeSpaceId;
    if (!target || !spaceId) return undefined;
    const targetLocation = target.locator.location;
    return DOCUMENT_CONTEXT_KINDS.has(targetLocation.kind)
      ? { spaceId, nodeId: target.nodeId, location: targetLocation }
      : undefined;
  });
  const sourceContextQuery = useQuery(
    computed(() => {
      const request = contextRequest.value;
      return {
        queryKey: ["spaces", store.activeSpaceId ?? "", "source-peek", request] as const,
        queryFn: () => loadDocumentContext(request),
        enabled: request !== undefined,
      };
    }),
  );
  const sourceContext = computed(() => sourceContextQuery.data.value ?? []);
  whenever(
    () => sourceContextQuery.error.value,
    (error) => toast.error(String(error)),
  );
  function revealSource() {
    if (sourcePeek.value) useCanvasEditorSession().reveal([sourcePeek.value.nodeId], "focus");
  }
  async function extractSource() {
    const target = sourcePeek.value;
    if (!target) return;
    try {
      await createSourceEvidence({
        store,
        target,
        title: t("spaces.inspector.excerpts.title", { title: target.nodeTitle }),
        sourceLine: t("spaces.inspector.excerpts.line", {
          title: target.nodeTitle,
          location: knowledgeLocationLabel(target.locator),
        }),
        rationale: t("spaces.assistant.evidence.sourceRationale"),
      });
    } catch (error) {
      toast.error(String(error));
    }
  }
  async function openSourceExternally() {
    await openAsset(false);
  }
  async function openSourceAtLocation() {
    await openAsset(true);
  }
  async function openAsset(atLocation: boolean) {
    const data = node.value?.data;
    const target = sourcePeek.value;
    if (!data || !isSpaceManagedAssetPayload(data) || (atLocation && !target)) return;
    try {
      if (atLocation && target) await openSpaceAssetAtLocation(data.assetPath, target.locator);
      else await openSpaceAsset(data.assetPath);
    } catch (error) {
      toast.error(String(error));
    }
  }
  return {
    store,
    sourcePeek,
    sourceContext,
    node,
    location,
    preciseExternal,
    editableSource,
    hasAsset,
    revealSource,
    extractSource,
    openSourceExternally,
    openSourceAtLocation,
  };
}

async function loadDocumentContext(request: DocumentContextRequest | undefined) {
  if (!request) return [];
  const chunks = await listCurrentKnowledgeChunks(request.spaceId, [request.nodeId]);
  return documentContext(chunks, request.location);
}

function documentContext(chunks: KnowledgeChunk[], target: KnowledgeChunk["locator"]["location"]) {
  if (target.kind === "pdf")
    return chunks.filter(
      ({ locator }) => locator.location.kind === "pdf" && locator.location.page === target.page,
    );
  if (target.kind === "word")
    return chunks.filter(
      ({ locator }) =>
        locator.location.kind === "word" &&
        locator.location.section === target.section &&
        Math.abs(locator.location.element - target.element) <= 2,
    );
  if (target.kind === "slides")
    return chunks.filter(
      ({ locator }) => locator.location.kind === "slides" && locator.location.slide === target.slide,
    );
  if (target.kind === "sheet")
    return chunks.filter(({ locator }) => {
      const location = locator.location;
      return (
        location.kind === "sheet" &&
        location.sheet === target.sheet &&
        (target.element === undefined ||
          location.element === undefined ||
          Math.abs(location.element - target.element) <= 2)
      );
    });
  return [];
}
