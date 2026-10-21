import { beforeEach, describe, expect, it } from "vitest";

import { hasActionableTextSelection, prefersNativeContextMenu } from "../text-selection";

function select(node: Node) {
  const range = document.createRange();
  range.selectNodeContents(node);
  document.getSelection()!.removeAllRanges();
  document.getSelection()!.addRange(range);
}

beforeEach(() => {
  document.getSelection()?.removeAllRanges();
  document.body.innerHTML = `
    <p id="bubble">Selectable reply</p>
    <p id="other">Unselected</p>
    <div id="canvas" tabindex="0" data-editing-surface><span id="card">Card</span></div>`;
});

const byId = (id: string) => document.getElementById(id)!;

describe("text selection ownership", () => {
  it("ignores a collapsed selection", () => {
    const range = document.createRange();
    range.setStart(byId("bubble").firstChild!, 2);
    document.getSelection()!.addRange(range);
    expect(hasActionableTextSelection(document.body)).toBe(false);
  });

  it("acts on a selection from wherever focus is, unless an editing surface holds focus", () => {
    select(byId("bubble"));
    expect(hasActionableTextSelection(document.body)).toBe(true);
    expect(hasActionableTextSelection(byId("canvas"))).toBe(false);

    select(byId("card"));
    expect(hasActionableTextSelection(byId("canvas"))).toBe(true);
  });

  it("keeps the native context menu for selected text, so it can be copied", () => {
    expect(prefersNativeContextMenu(byId("bubble"))).toBe(false);
    select(byId("bubble"));
    expect(prefersNativeContextMenu(byId("bubble"))).toBe(true);
    expect(prefersNativeContextMenu(byId("other"))).toBe(false);
  });
});
