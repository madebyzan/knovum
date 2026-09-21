import { describe, expect, it } from "vitest";

import {
  importJsonCanvasDocument,
  mergeImportedJsonCanvasDocument,
} from "../json-canvas-mapping";

describe("JSON Canvas group backgrounds", () => {
  it.each(["cover", "ratio", "repeat"] as const)(
    "keeps the standard %s style while adopting its managed path",
    (backgroundStyle) => {
      const plan = importJsonCanvasDocument({
        nodes: [
          {
            id: "group",
            type: "group",
            background: "assets/background.png",
            backgroundStyle,
            x: 0,
            y: 0,
            width: 400,
            height: 300,
          },
        ],
      });
      const document = mergeImportedJsonCanvasDocument(
        { version: 1, viewport: { x: 0, y: 0, zoom: 1 }, nodes: [] },
        {
          plan,
          importedNodes: [{ sourceId: "group", nodeId: "section" }],
          backgroundPaths: new Map([["group", "/managed/background.png"]]),
        },
      );

      expect(document.nodes[0]?.appearance).toEqual({
        background: "/managed/background.png",
        backgroundStyle,
      });
    },
  );
});
