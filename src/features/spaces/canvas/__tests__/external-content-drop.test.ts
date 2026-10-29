import { describe, expect, it, vi } from "vitest";
import { effectScope, shallowRef } from "vue";

import { writePassageTransfer } from "../contracts/passage-transfer";
import { useExternalContentDrop } from "../konva/interaction/use-external-content-drop";

function transfer() {
  const data = new Map<string, string>();
  return {
    get types() {
      return [...data.keys()];
    },
    setData: (type: string, value: string) => void data.set(type, value),
    getData: (type: string) => data.get(type) ?? "",
    effectAllowed: "all",
  } as unknown as DataTransfer;
}

function drop(dataTransfer: DataTransfer) {
  const host = document.createElement("div");
  host.getBoundingClientRect = () => ({ left: 10, top: 20 }) as DOMRect;
  const handlers = {
    insertText: vi.fn<(text: string, point: { x: number; y: number }) => Promise<void>>(async () => {}),
    insertPassage: vi.fn<(passage: unknown, point: { x: number; y: number }) => Promise<void>>(
      async () => {},
    ),
    insertUrl: vi.fn<(url: string, point: { x: number; y: number }) => Promise<void>>(async () => {}),
  };
  const scope = effectScope();
  scope.run(() =>
    useExternalContentDrop({
      host: shallowRef(host),
      viewport: () => ({ x: 0, y: 0, zoom: 1 }),
      ...handlers,
      reportError: vi.fn<(error: unknown) => void>(),
    }),
  );
  const event = Object.assign(new Event("drop", { cancelable: true }), {
    dataTransfer,
    clientX: 110,
    clientY: 220,
  });
  host.dispatchEvent(event);
  scope.stop();
  return handlers;
}

describe("dropping text on the canvas", () => {
  it("lands a passage from an answer as a passage, at the drop point", () => {
    const data = transfer();
    writePassageTransfer(data, { text: "Words", sourceNodeIds: ["source"] });
    const handlers = drop(data);
    expect(handlers.insertPassage).toHaveBeenCalledWith(
      { text: "Words", sourceNodeIds: ["source"] },
      { x: 100, y: 200 },
    );
    expect(handlers.insertText).not.toHaveBeenCalled();
  });

  it("still lands plain text from elsewhere as text", () => {
    const data = transfer();
    data.setData("text/plain", "Elsewhere");
    const handlers = drop(data);
    expect(handlers.insertText).toHaveBeenCalledWith("Elsewhere", { x: 100, y: 200 });
    expect(handlers.insertPassage).not.toHaveBeenCalled();
  });
});
