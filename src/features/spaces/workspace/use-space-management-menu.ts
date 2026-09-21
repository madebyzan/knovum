import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import type { CanvasExchangePort } from "../canvas-exchange/canvas-exchange-port";
import { useCanvasExchangeSession } from "../canvas-exchange/canvas-exchange-session";
import { exportJsonCanvasFile } from "../canvas-exchange/transfer";
import { useContextDock } from "../canvas/use-context-dock";
import type { Space } from "../model/space";
import { useSpacePackageTransfer } from "../package/use-space-package-transfer";
import { useSpaceStore } from "../session/space-store";
import { popupSpaceContextMenu, spaceMenuItem, spaceSubmenu, type SpaceMenuItem } from "./space-menu";

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

  function groups(space: Space): readonly SpaceMenuItem[][] {
    const spaceId = space.id;
    // Capture the port with its Space identity, never look up a different active canvas later.
    const port = store.activeSpaceId === spaceId ? activePort.value : undefined;
    return [
      [
        spaceMenuItem(t("spaces.knowledge.knowledgeHealth"), async () => {
          await store.openSpace(spaceId);
          dock.openKnowledgeHealth();
        }),
      ],
      [
        spaceSubmenu(t("spaces.transfer.export"), [
          spaceMenuItem(t("spaces.transfer.exportSpace"), () => packages.exportPackage(space)),
          spaceMenuItem(
            t("spaces.transfer.exportJsonCanvas"),
            () => reportExchange(() => exportCanvas(space, port!), t("spaces.transfer.jsonCanvasExported")),
            { enabled: Boolean(port) },
          ),
        ]),
      ],
    ];
  }

  function show(space: Space, preceding: readonly SpaceMenuItem[][] = []) {
    return popupSpaceContextMenu([...preceding, ...groups(space)]);
  }

  return { groups, show };
}
