import { platform } from "@tauri-apps/plugin-os";
import { storeToRefs } from "pinia";
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { openSpaceAsset, openSpaceAssetAtLocation } from "../../assets/asset-operations";
import { useCanvasEditorSession } from "../../canvas/editor-session";
import { createSourceEvidence } from "../../editor/knowledge-capture";
import { isSpaceManagedAssetPayload } from "../../model/nodes/capabilities";
import { useSpaceStore } from "../../session/space-store";
import type { KnowledgeChunk } from "../model/knowledge-chunk";
import { knowledgeLocationLabel } from "../model/location";
import { listCurrentKnowledgeChunks } from "../persistence/chunk-repository";

export function useSourcePeek() {
  const store = useSpaceStore();
  const { sourcePeek } = storeToRefs(store);
  const { t } = useI18n();
  const sourceContext = ref<KnowledgeChunk[]>([]);
  let generation = 0;
  const node = computed(() => store.nodes.find((item) => item.id === sourcePeek.value?.nodeId));
  const location = computed(() => sourcePeek.value?.locator.location);
  const preciseExternal = computed(
    () => platform() === "windows" && ["word", "slides", "sheet"].includes(location.value?.kind ?? ""),
  );
  const editableSource = computed(() => ["note", "markdown"].includes(node.value?.data.kind ?? ""));
  const hasAsset = computed(() => Boolean(node.value && isSpaceManagedAssetPayload(node.value.data)));
  watch([sourcePeek, node], () => void refresh(++generation), { immediate: true });

  async function refresh(current: number) {
    sourceContext.value = [];
    const target = sourcePeek.value;
    const data = node.value?.data;
    if (!target || !data) return;
    try {
      if (["pdf", "word", "slides", "sheet"].includes(target.locator.location.kind)) {
        const context = await loadDocumentContext(store.activeSpaceId, target);
        if (current === generation) sourceContext.value = context;
      }
    } catch (error) {
      toast.error(String(error));
    }
  }
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
        title: t("spaces.inspector.sourceExcerptTitle", { title: target.nodeTitle }),
        sourceLine: t("spaces.inspector.sourceExcerptLine", {
          title: target.nodeTitle,
          location: knowledgeLocationLabel(target.locator),
        }),
        rationale: t("spaces.assistant.groundedSourceRationale"),
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

async function loadDocumentContext(
  spaceId: string | undefined,
  target: NonNullable<ReturnType<typeof useSpaceStore>["sourcePeek"]>,
) {
  if (!spaceId) throw new Error("Source Peek requires an active Space");
  const chunks = await listCurrentKnowledgeChunks(spaceId, [target.nodeId]);
  return documentContext(chunks, target.locator.location);
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
