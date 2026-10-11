import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  invoke: vi.fn<(command: string, args?: unknown) => Promise<unknown>>(),
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: mocks.invoke }));
vi.mock("../../../foundation/i18n/instance", () => ({ i18n: { global: { t: (key: string) => key } } }));
vi.mock("../runs/resource-run-observation", () => ({
  observeResourceRun: vi.fn<(...args: unknown[]) => void>(),
}));

import { CREATION_INTENT_DEFINITIONS } from "../knowledge-derivations/catalog/creation-intent-definitions";
import { convertDocument } from "../knowledge-derivations/document/conversion";

type DocumentConversionOptions = Parameters<typeof convertDocument>[0];

const source: DocumentConversionOptions["source"] = {
  id: "source",
  type: "markdown",
  initialPosition: { x: 0, y: 0 },
  data: { kind: "markdown", title: "Report", knowledgeRevision: "revision-1", text: "Body" },
};

function store(activeSpaceId: string | undefined) {
  return {
    activeSpaceId,
  } as unknown as DocumentConversionOptions["store"];
}

describe("document conversion", () => {
  beforeEach(() => {
    mocks.invoke.mockReset().mockResolvedValue({ nodeId: "output", data: {} });
  });

  it.each(["docx", "pdf", "txt", "html"] as const)("preserves source identity for %s", async (format) => {
    await convertDocument({ store: store("space"), source, format });
    expect(mocks.invoke).toHaveBeenCalledTimes(1);
    expect(mocks.invoke).toHaveBeenCalledWith("plugin:spaces|start_space_action", {
      request: {
        requestId: expect.any(String),
        spaceId: "space",
        actionId: `convert-document-to-${format}`,
        parameters: {},
        sources: [{ nodeId: "source", knowledgeRevision: "revision-1" }],
        target: null,
      },
    });
  });

  it("does not invoke Rust without an active Space", async () => {
    const inactive = store(undefined);
    await convertDocument({ store: inactive, source, format: "txt" });
    expect(mocks.invoke).not.toHaveBeenCalled();
  });

  it("submits a pinned PDF identity without sending paths or cached content to execution", async () => {
    const file: DocumentConversionOptions["source"] = {
      id: "pdf",
      type: "file",
      initialPosition: { x: 0, y: 0 },
      data: {
        kind: "file",
        title: "Report",
        assetPath: "/owned/report.pdf",
        fileName: "report.pdf",
        knowledgeRevision: "knowledge:pdf",
        sourceRevision: "sha256:pdf",
        extractedText: "Cached partial text",
      },
    };
    await convertDocument({ store: store("space"), source: file, format: "docx" });
    expect(mocks.invoke).toHaveBeenCalledWith("plugin:spaces|start_space_action", {
      request: {
        requestId: expect.any(String),
        spaceId: "space",
        actionId: "convert-document-to-docx",
        parameters: {},
        sources: [{ nodeId: "pdf", knowledgeRevision: "knowledge:pdf" }],
        target: null,
      },
    });
  });

  it("registers a localized TXT conversion action", () => {
    const action = CREATION_INTENT_DEFINITIONS.find(
      (definition) => definition.id === "convert-document-to-txt",
    );
    expect(action?.labelKey).toBe("spaces.creation.document.conversion.txt");
  });
});
