import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  invoke: vi.fn<(command: string, args?: unknown) => Promise<unknown>>(),
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: mocks.invoke }));
vi.mock("../../../foundation/i18n/i18n", () => ({ i18n: { global: { t: (key: string) => key } } }));
vi.mock("../canvas/editor-session", () => ({
  useCanvasEditorSession: () => ({ selection: { value: { nodePlacements: [] } } }),
}));
vi.mock("../knowledge-derivations/placement", () => ({ derivedNodePosition: () => ({ x: 0, y: 0 }) }));
vi.mock("../knowledge-derivations/run-derivation", () => ({
  runDerivation: (operation: { execute(): Promise<unknown> }) => operation.execute(),
}));

import { creationIntentDefinitions } from "../knowledge-derivations/catalog/creation-intent-definitions";
import { exportMarkdownDocument } from "../knowledge-derivations/document/export";

const commands = {
  docx: {
    export: "plugin:spaces|export_document_as_docx",
  },
  pdf: {
    export: "plugin:spaces|export_document_as_pdf",
  },
  txt: {
    export: "plugin:spaces|export_document_as_txt",
  },
} as const;

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
    editSnapshot: { editRevision: 7 },
    groupScopePath: [],
    addContent: vi.fn<() => Promise<Array<{ id: string }>>>(async () => [{ id: "output" }]),
  } as unknown as ExportOptions["store"];
}

describe("Markdown document export", () => {
  beforeEach(() => {
    mocks.invoke.mockReset().mockResolvedValue({ nodeId: "output", data: {} });
  });

  it.each(["docx", "pdf", "txt"] as const)("preserves source identity for %s", async (format) => {
    await exportMarkdownDocument({ store: store("space"), source, format });
    expect(mocks.invoke).toHaveBeenCalledTimes(1);
    expect(mocks.invoke).toHaveBeenNthCalledWith(1, commands[format].export, {
      request: {
        id: expect.any(String),
        spaceId: "space",
        sourceNodeId: "source",
        sourceRevision: "revision-1",
        outputNodeId: expect.any(String),
        expectedEditRevision: 7,
        targetParentId: null,
        position: { x: 0, y: 0 },
        size: { width: 256, height: 192 },
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
    expect(action?.effect).toBe("export");
  });
});
