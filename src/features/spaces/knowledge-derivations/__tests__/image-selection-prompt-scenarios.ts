import { expect, it } from "vitest";

import { imageSourceNode } from "../../__tests__/support/image-operation-fixtures";
import { useImageSelectionSessionStore } from "../image-selection-session-store";

/** Runs under the session suite's mocks, where the preview is still drawing. */
export function imageSelectionPromptScenarios() {
  it("offers the actions as soon as the prompts select something, without waiting for the preview", async () => {
    const store = useImageSelectionSessionStore();
    await store.begin(imageSourceNode());
    expect(store.canSubmit).toBe(false);
    store.tool = "brush";
    store.addStroke([0.2, 0.3, 0.4, 0.5]);
    expect(store.canSubmit).toBe(false);
    store.tool = "exclude";
    store.addPoint(0.7, 0.8);
    expect(store.canSubmit).toBe(false);
    store.tool = "include";
    store.addPoint(0.3, 0.4);
    expect(store.busy).toBe(true);
    expect(store.canSubmit).toBe(true);
    expect(store.canRepair).toBe(true);
    store.undo();
    store.undo();
    store.undo();
    store.setBox({ x: 0.1, y: 0.1, width: 0.6, height: 0.6 });
    expect(store.canSubmit).toBe(true);
  });
}
