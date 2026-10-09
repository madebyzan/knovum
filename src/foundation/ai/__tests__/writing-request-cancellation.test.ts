import { beforeEach, describe, expect, it, vi } from "vitest";

import type * as AiExecution from "../ai-execution";
import { transformEditorTextStream } from "../editor-transform";
import type { ResolvedAiTaskTarget } from "../schema/models";
import { translateTextStream } from "../translation";

const requests = vi.hoisted(() => ({
  target: vi.fn<() => Promise<ResolvedAiTaskTarget>>(),
  invoke: vi.fn<() => Promise<{ text: string }>>(),
}));
vi.mock("../ai-execution", async (original) => ({
  ...(await original<typeof AiExecution>()),
  loadAiExecutionTarget: requests.target,
}));
vi.mock("../../ipc/invoke-command", () => ({ invokeCommand: requests.invoke }));
vi.mock("@tauri-apps/api/core", () => ({
  Channel: class {
    onmessage: ((event: unknown) => void) | undefined;
  },
}));
beforeEach(() => {
  requests.invoke.mockReset().mockResolvedValue({ text: "generated" });
  requests.target.mockReset();
});

describe("writing request cancellation before IPC admission", () => {
  it.each(["transform", "translation"] as const)(
    "does not start %s after cancellation during target loading",
    async (kind) => {
      let release!: (target: ResolvedAiTaskTarget) => void;
      requests.target.mockReturnValue(new Promise((resolve) => (release = resolve)));
      const cancellation = new AbortController();
      const request =
        kind === "transform"
          ? transformEditorTextStream({
              action: "improve",
              text: "source",
              jobId: "job",
              onEvent: () => {},
              signal: cancellation.signal,
            })
          : translateTextStream({ text: "source", target: "en" }, "job", () => {}, cancellation.signal);
      cancellation.abort();
      release({ type: "agent", runtimeKind: "codex" });
      await expect(request).rejects.toMatchObject({ name: "AbortError" });
      expect(requests.invoke).not.toHaveBeenCalled();
    },
  );
});
