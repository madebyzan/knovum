import { ArrowUpRight, Eye, LocateFixed, MousePointerClick, Pencil } from "@lucide/vue";
import { openUrl } from "@tauri-apps/plugin-opener";
import { computed, toValue, type MaybeRefOrGetter } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { openSpaceAsset, openSpaceAssetAtLocation } from "../../assets/asset-operations";
import { hasSystemQuickLook, previewSpaceAsset } from "../../assets/asset-preview";
import { selectAndRevealSpaceNode } from "../../canvas/selection-actions";
import type { KnowledgeLocator } from "../../knowledge/model/schema/chunk";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { isSpaceManagedAssetPayload, isSpaceWebsitePayload } from "../../model/nodes/capabilities";
import { useSpaceStore } from "../../session/space-store";

interface ReaderHeaderMenuOptions {
  node: MaybeRefOrGetter<SpaceCanvasNode>;
  followsSelection: MaybeRefOrGetter<boolean>;
  locator: MaybeRefOrGetter<KnowledgeLocator | undefined>;
  onToggleFollow: () => void;
}

/** Builds the Source Reader overflow menu items and handles external open, Quick Look, and edit routing. */
export function useReaderHeaderMenu(options: ReaderHeaderMenuOptions) {
  const { t } = useI18n();
  const store = useSpaceStore();

  async function openExternally() {
    const payload = toValue(options.node).data;
    try {
      if (isSpaceWebsitePayload(payload) && payload.url) {
        await openUrl(payload.url);
        return;
      }
      if (!isSpaceManagedAssetPayload(payload)) return;
      const loc = toValue(options.locator);
      const supportsJump =
        loc &&
        (loc.location.kind === "word" || loc.location.kind === "slides" || loc.location.kind === "sheet");
      if (supportsJump) await openSpaceAssetAtLocation(payload.assetPath, loc);
      else await openSpaceAsset(payload.assetPath);
    } catch (error) {
      toast.error(getErrorMessage(error));
    }
  }

  async function openQuickLook() {
    const payload = toValue(options.node).data;
    if (!isSpaceManagedAssetPayload(payload)) return;
    try {
      await previewSpaceAsset(payload.assetPath);
    } catch (error) {
      toast.error(getErrorMessage(error));
    }
  }

  function openEditor() {
    const node = toValue(options.node);
    const locator = toValue(options.locator);
    if (!locator) {
      void store.openDocumentEditor(node.id);
      return;
    }
    void store.openEditorAtSource({ nodeId: node.id, nodeTitle: node.data.title, locator });
  }

  return computed(() => {
    const node = toValue(options.node);
    const payload = node.data;
    const isAsset = isSpaceManagedAssetPayload(payload);
    const isEditableDoc = payload.kind === "note" || payload.kind === "markdown";
    const canOpenExternal = isAsset || (isSpaceWebsitePayload(payload) && Boolean(payload.url));
    return [
      {
        id: "follow-selection",
        label: t("spaces.inspector.reader.followSelection"),
        icon: MousePointerClick,
        checked: toValue(options.followsSelection),
        onSelect: options.onToggleFollow,
      },
      { id: "sep-1", separator: true },
      {
        id: "reveal",
        label: t("spaces.canvas.revealInCanvas"),
        icon: LocateFixed,
        onSelect: () => selectAndRevealSpaceNode(node.id),
      },
      ...(isEditableDoc ? [{ id: "edit", label: t("common.edit"), icon: Pencil, onSelect: openEditor }] : []),
      ...(isAsset && hasSystemQuickLook()
        ? [
            {
              id: "quick-look",
              label: t("spaces.importExport.preview.quickLook"),
              icon: Eye,
              onSelect: () => void openQuickLook(),
            },
          ]
        : []),
      ...(canOpenExternal
        ? [
            {
              id: "open-external",
              label: isSpaceWebsitePayload(payload)
                ? t("spaces.canvas.nodeActions.openOriginal")
                : t("spaces.importExport.openExternally"),
              icon: ArrowUpRight,
              onSelect: () => void openExternally(),
            },
          ]
        : []),
    ];
  });
}
