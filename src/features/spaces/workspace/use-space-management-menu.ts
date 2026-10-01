import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import type { ContextMenuItem } from "../../../foundation/context-menu/context-menu";
import type { CanvasExchangePort } from "../canvas-exchange/canvas-exchange-port";
import { useCanvasExchangeSession } from "../canvas-exchange/canvas-exchange-session";
import { exportJsonCanvasFile } from "../canvas-exchange/transfer";
import { useContextDock } from "../canvas/use-context-dock";
import type { Space } from "../model/space";
import { useSpacePackageTransfer } from "../package/use-space-package-transfer";
import { useSpaceStore } from "../session/space-store";
import { popupSpaceContextMenu, spaceMenuItem, spaceSubmenu } from "./space-menu";

async function reportExchange(operation: () => Promise<boolean>, success: string) {
  try {
    if (await operation()) toast.success(success);
  } catch (error) {
    toast.error(String(error));
  }
}

export function useSpaceManagementMenu() {
  const store = useSpaceStore();
  const dock = useContextDock();
  const packages = useSpacePackageTransfer();
  const { activePort } = useCanvasExchangeSession();
  const { t } = useI18n();

  async function exportCanvas(space: Space, port: CanvasExchangePort) {
    await store.commitActiveSession();
    if (store.activeSpaceId !== space.id || activePort.value !== port) {
      throw new Error("The Space selected for JSON Canvas export is no longer active");
    }
    return exportJsonCanvasFile(port, space.title);
  }

  function groups(space: Space): readonly ContextMenuItem[][] {
    const spaceId = space.id;
    // Capture the port with its Space identity, never look up a different active canvas later.
    const port = store.activeSpaceId === spaceId ? activePort.value : undefined;
    return [
      [
        spaceMenuItem(t("spaces.knowledge.health.title"), async () => {
          await store.openSpace(spaceId);
          dock.openKnowledgeHealth();
        }),
      ],
      [
        spaceSubmenu(t("spaces.importExport.export"), [
          spaceMenuItem(t("spaces.importExport.space.export"), () => packages.exportPackage(space)),
          spaceMenuItem(
            t("spaces.importExport.jsonCanvas.export"),
            () =>
              reportExchange(() => exportCanvas(space, port!), t("spaces.importExport.jsonCanvas.exported")),
            { enabled: Boolean(port) },
          ),
        ]),
      ],
    ];
  }

  function show(space: Space, preceding: readonly ContextMenuItem[][] = []) {
    return popupSpaceContextMenu([...preceding, ...groups(space)]);
  }

  return { groups, show };
}
