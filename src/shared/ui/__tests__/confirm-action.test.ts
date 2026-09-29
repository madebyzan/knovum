import { describe, expect, it } from "vitest";

import { activeConfirmation, confirmAction, resolveConfirmation } from "../confirm-action";

const action = (title: string) => ({ title, description: "Confirm this action", confirmLabel: "Continue" });

describe("confirmAction", () => {
  it("treats dismissal as cancellation and shows queued actions in order", async () => {
    const first = confirmAction(action("First"));
    const second = confirmAction(action("Second"));

    expect(activeConfirmation.value?.title).toBe("First");
    resolveConfirmation(false);
    await expect(first).resolves.toBe(false);
    expect(activeConfirmation.value?.title).toBe("Second");

    resolveConfirmation(true);
    await expect(second).resolves.toBe(true);
    expect(activeConfirmation.value).toBeUndefined();
  });
});
