import { expect, it, vi } from "vitest";

import { imageSourceNode } from "../../__tests__/support/image-operation-fixtures";
import { getImageOperationCapabilities } from "../image-operation";
import { deriveImageSelection } from "../image-selection-derivation";
import { useImageSelectionSessionStore } from "../image-selection-session-store";

export function imageSelectionModelScenarios() {
  it("keeps the Settings models pinned when defaults change during the selection session", async () => {
    const store = useImageSelectionSessionStore();
    const node = imageSourceNode();
    await store.begin(node);
    const changed = structuredClone(await getImageOperationCapabilities());
    changed.targets[0]!.execution = { type: "local", model: "efficientSamS", specId: "changed" };
    changed.targets[1]!.selected = false;
    changed.targets[2]!.selected = true;
    vi.mocked(getImageOperationCapabilities).mockResolvedValueOnce(changed);
    await getImageOperationCapabilities();
    store.addPoint(0.3, 0.4);
    vi.mocked(deriveImageSelection).mockResolvedValueOnce(undefined);
    await store.submit("repair");
    expect(deriveImageSelection).toHaveBeenLastCalledWith(
      node,
      expect.objectContaining({
        selectionModel: { type: "local", model: "efficientSamTi", specId: "spec" },
        repairModel: { type: "local", model: "migan", specId: "migan-spec" },
      }),
    );
  });

  it("can extract without a repair model, but cannot start repair", async () => {
    const store = useImageSelectionSessionStore();
    const node = imageSourceNode();
    const capabilities = structuredClone(await getImageOperationCapabilities());
    capabilities.targets = capabilities.targets.filter((target) => target.capability !== "imageInpainting");
    vi.mocked(getImageOperationCapabilities).mockResolvedValueOnce(capabilities);
    await store.begin(node);
    expect(store.canSubmit).toBe(true);
    expect(store.canRepair).toBe(false);
    const before = vi.mocked(deriveImageSelection).mock.calls.length;
    await store.submit("repair");
    expect(deriveImageSelection).toHaveBeenCalledTimes(before);
    vi.mocked(deriveImageSelection).mockResolvedValueOnce(undefined);
    await store.submit("extract");
    expect(deriveImageSelection).toHaveBeenLastCalledWith(
      node,
      expect.objectContaining({
        effect: "extract",
        selectionModel: { type: "local", model: "efficientSamTi", specId: "spec" },
      }),
    );
    expect(vi.mocked(deriveImageSelection).mock.lastCall?.[1]).not.toHaveProperty("repairModel");
  });
}
