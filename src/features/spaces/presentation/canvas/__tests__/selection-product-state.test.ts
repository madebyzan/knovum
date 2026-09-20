import { describe, expect, it } from "vitest";

import { createSpaceNode } from "../../../canvas/node";
import { canvasSelectionProductState } from "../selection-product-state";

const note = (id: string, locked = false) =>
  createSpaceNode(
    {
      kind: "note",
      title: id,
      tiptapContent: { type: "doc" },
      plainText: "",
      ...(locked ? { locked: true } : {}),
    },
    { x: 0, y: 0 },
    id,
  );

describe("canvasSelectionProductState", () => {
  it("exposes semantic product actions without duplicating engine arrangement actions", () => {
    const state = canvasSelectionProductState([note("a"), note("b")], ["a", "b"]);

    expect(state).toMatchObject({
      allLocked: false,
      askScope: "selection",
      canConnect: true,
      canOpen: false,
    });
    expect(Object.keys(state)).not.toContain("align");
    expect(Object.keys(state)).not.toContain("group");
  });

  it("uses section context and preserves the selected lock state", () => {
    const section = createSpaceNode(
      { kind: "section", title: "Research", locked: true },
      { x: 0, y: 0 },
      "section",
    );
    const state = canvasSelectionProductState([section], ["section"]);

    expect(state.askScope).toBe("section");
    expect(state.allLocked).toBe(true);
    expect(state.section?.id).toBe("section");
  });

  it("connects only an exact two-node semantic selection", () => {
    const section = createSpaceNode({ kind: "section", title: "Research" }, { x: 0, y: 0 }, "section");

    expect(canvasSelectionProductState([note("a"), note("b")], ["a", "b"]).canConnect).toBe(true);
    expect(canvasSelectionProductState([note("a"), note("b"), note("c")], ["a", "b", "c"]).canConnect).toBe(
      false,
    );
    expect(
      canvasSelectionProductState([section, note("a"), note("b")], ["section", "a", "b"]).canConnect,
    ).toBe(false);
  });

  it("identifies managed assets as system-preview targets", () => {
    const file = createSpaceNode(
      { kind: "file", title: "Report", fileName: "report.pdf", assetPath: "/spaces/report.pdf" },
      { x: 0, y: 0 },
      "file",
    );

    expect(canvasSelectionProductState([file], [file.id]).opensSystemPreview).toBe(true);
    expect(canvasSelectionProductState([note("note")], ["note"]).opensSystemPreview).toBe(false);
  });
});
