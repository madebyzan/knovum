import { expect, it, onTestFinished, vi } from "vitest";

import { registerTiptapSessionParticipant } from "../editor/tiptap-session-participant";
import { createEditorSurfaceActions } from "../workspace/editor-surface-actions";
import { createTestSessionContext } from "./support/session-context";

it.each(["saved", "failed"])("closes the document only after its session is %s", async (outcome) => {
  const context = createTestSessionContext();
  context.surface.value = { kind: "document", nodeId: "note" };
  const failure = new Error("disk full");
  const persist = vi.fn<() => Promise<void>>();
  if (outcome === "saved") persist.mockResolvedValue();
  else persist.mockRejectedValue(failure);
  onTestFinished(
    registerTiptapSessionParticipant({
      cancelAutoSave: () => {},
      editor: () => undefined,
      persist,
    }),
  );
  const result = await createEditorSurfaceActions(context)
    .returnToCanvas()
    .then(
      () => undefined,
      (error: unknown) => error,
    );
  expect(result).toBe(outcome === "saved" ? undefined : failure);
  expect(context.surface.value.kind).toBe(outcome === "saved" ? "canvas" : "document");
  expect(persist).toHaveBeenCalledOnce();
});
