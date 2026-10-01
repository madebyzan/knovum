import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  invoke: vi.fn<(command: string, args?: unknown) => Promise<unknown>>(),
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: mocks.invoke }));
vi.mock("../../../foundation/i18n/i18n", () => ({ i18n: { global: { t: (key: string) => key } } }));
vi.mock("../runs/resource-run-observation", () => ({
  observeResourceRun: vi.fn<(...args: unknown[]) => void>(),
}));

import { creationIntentDefinitions } from "../knowledge-derivations/catalog/creation-intent-definitions";
import { exportMarkdownDocument } from "../knowledge-derivations/document/export";

type ExportOptions = Parameters<typeof exportMarkdownDocument>[0];

const source: ExportOptions["source"] = {
  id: "source",
  type: "markdown",
  initialPosition: { x: 0, y: 0 },
  data: { kind: "markdown", title: "Report", knowledgeRevision: "revision-1", text: "Body" },
};

function store(activeSpaceId: string | undefined) {
  return {
    activeSpaceId,
  } as unknown as ExportOptions["store"];
}

describe("Markdown document export", () => {
  beforeEach(() => {
    mocks.invoke.mockReset().mockResolvedValue({ nodeId: "output", data: {} });
  });

  it.each(["docx", "pdf", "txt"] as const)("preserves source identity for %s", async (format) => {
    await exportMarkdownDocument({ store: store("space"), source, format });
    expect(mocks.invoke).toHaveBeenCalledTimes(1);
    expect(mocks.invoke).toHaveBeenCalledWith("plugin:spaces|start_space_action", {
      request: {
        requestId: expect.any(String),
        spaceId: "space",
        actionId: `export-document-as-${format}`,
        parameters: {},
        sources: [{ nodeId: "source", knowledgeRevision: "revision-1" }],
        target: null,
      },
    });
  });

  it("does not invoke Rust without an active Space", async () => {
    const inactive = store(undefined);
    await exportMarkdownDocument({ store: inactive, source, format: "txt" });
    expect(mocks.invoke).not.toHaveBeenCalled();
  });

  it("registers a localized TXT export action", () => {
    const action = creationIntentDefinitions.find((entry) => entry.id === "export-document-as-txt");
    expect(action?.labelKey).toBe("spaces.creation.document.export.txt");
  });
});
