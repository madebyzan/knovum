import { describe, expect, it } from "vitest";

import { importJsonCanvasDocument } from "../canvas-exchange/json-canvas-mapping";
import { mergeImportedJsonCanvasStructure } from "../canvas-exchange/json-canvas-structure-import";

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
      const snapshot = mergeImportedJsonCanvasStructure(
        { editRevision: 0, structure: [], layouts: [], relationStyles: [] },
        plan,
        {
          contentIds: new Map(),
          sectionIds: new Map([["group", "section"]]),
          backgroundPaths: new Map([["group", "/managed/background.png"]]),
          defaultSectionTitle: "Section",
        },
      );

      expect(snapshot.layouts[0]).toMatchObject({
        kind: "section",
        appearance: { background: "/managed/background.png", backgroundStyle },
      });
    },
  );
});
