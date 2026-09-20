import { describe, expect, it, vi } from "vitest";

import type { CanvasExchangeCodec } from "../canvas-exchange-port";
import type {
  CanvasJsonImportCommitResult,
  CanvasJsonImportDraft,
} from "../json-canvas-import-contract";
import { createJsonCanvasExchangePort } from "../json-canvas-service";
import type { CanvasDocumentV1 } from "../../canvas/contract/canvas-document";
import type { CanvasHistoryEntry, CanvasPersistenceHistory } from "../../canvas/contract/canvas-editor";
import type { CanvasEditorPort } from "../../canvas/contract/canvas-editor-port";

describe("JSON Canvas application service", () => {
  it("exports file and group background resources and imports the standard paths unchanged", async () => {
    const document: CanvasDocumentV1 = {
      version: 1,
      viewport: { x: 0, y: 0, zoom: 1 },
      nodes: [
        {
          nodeId: "group",
          frame: { x: 0, y: 0, width: 400, height: 300, rotation: 0 },
          sizeAuthority: "frame",
          zIndex: 0,
          appearance: { background: "/managed/bg.png", backgroundStyle: "repeat" },
        },
        {
          nodeId: "file",
          frame: { x: 20, y: 20, width: 200, height: 120, rotation: 0 },
          sizeAuthority: "frame",
          zIndex: 1,
          appearance: { background: "/managed/not-a-group-background.png" },
        },
      ],
    };
    const codec = jsonCodec();
    const history: CanvasPersistenceHistory = {
      undo: async () => ({ revision: 2 }),
      redo: async () => ({ revision: 3 }),
    };
    const commitImport = vi.fn<(request: CanvasJsonImportDraft) => Promise<CanvasJsonImportCommitResult>>(
      async (request) => ({
        importedNodes: [],
        importedRelations: [],
        document: request.document,
        revision: 1,
        history,
      }),
    );
    const editor = {
      getDocument: () => document,
      observeDocument: (observer) => {
        observer(document);
        return () => undefined;
      },
      setRelationColor: vi.fn(),
      adoptDocument: vi.fn<(document: CanvasDocumentV1) => void>(),
    } as unknown as CanvasEditorPort;
    const port = createJsonCanvasExchangePort({
      codec,
      editor,
      nodes: () => [
        {
          id: "group",
          type: "section",
          data: { kind: "section", title: "Group" },
          initialPosition: { x: 0, y: 0 },
        },
        {
          id: "file",
          type: "file",
          data: { kind: "file", title: "Doc", fileName: "doc.pdf", assetPath: "/managed/doc.pdf" },
          initialPosition: { x: 20, y: 20 },
        },
      ],
      relations: () => [],
      commitImport,
      historyEntry: vi.fn<(entry: CanvasHistoryEntry) => void>(),
      historyTransition: async (nextDocument, transition) => ({
        ...(await transition({ baseRevision: 1, document: nextDocument })),
        document: nextDocument,
      }),
      historyTransaction: async (action) => action(),
    });

    const exported = await port.exportJsonCanvas();
    const standard = JSON.parse(exported.source) as { nodes: Record<string, unknown>[] };
    expect(exported.resources).toEqual([
      { sourcePath: "/managed/bg.png", relativePath: "assets/group-bg.png" },
      { sourcePath: "/managed/doc.pdf", relativePath: "assets/file-doc.pdf" },
    ]);
    expect(standard.nodes[0]).toMatchObject({
      type: "group",
      background: "assets/group-bg.png",
      backgroundStyle: "repeat",
    });
    expect(standard.nodes[1]).toMatchObject({ type: "file", file: "assets/file-doc.pdf" });

    await port.importJsonCanvas(exported.source, "/export");
    expect(commitImport).toHaveBeenCalledWith(
      expect.objectContaining({
        sourceDirectory: "/export",
        plan: expect.objectContaining({
          nodes: expect.arrayContaining([
            expect.objectContaining({
              sourceId: "group",
              appearance: { background: "assets/group-bg.png", backgroundStyle: "repeat" },
            }),
          ]),
        }),
      }),
    );
  });
});

function jsonCodec(): CanvasExchangeCodec {
  return {
    parse: async (source) => JSON.parse(source),
    serialize: async (document) => JSON.stringify(document),
  };
}
