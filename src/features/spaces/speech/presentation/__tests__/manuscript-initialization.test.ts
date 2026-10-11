import { flushPromises, mount } from "@vue/test-utils";
import { afterEach, expect, it, onTestFinished, vi } from "vitest";

import { i18n } from "../../../../../foundation/i18n/instance";
import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import type { readNodeAudiobookScript } from "../../audiobook-script-import";
import AudiobookScriptImport from "../AudiobookScriptImport.vue";

const reader = vi.hoisted(() => vi.fn<typeof readNodeAudiobookScript>());
vi.mock("../../audiobook-script-import", () => ({ readNodeAudiobookScript: reader }));
afterEach(() => vi.clearAllMocks());
const node: SpaceCanvasNode = {
  id: "notes",
  type: "markdown",
  initialPosition: { x: 0, y: 0 },
  data: { kind: "markdown", title: "Notes", text: "# Chapter\n\nRead this." },
};
const script = {
  schemaVersion: 1 as const,
  chapters: [{ title: "Chapter", paragraphs: [{ role: "Narrator", text: "Read this." }] }],
};
function render(initialize: boolean) {
  const wrapper = mount(AudiobookScriptImport, {
    props: { node, disabled: false, initialize },
    global: { plugins: [i18n] },
  });
  onTestFinished(() => wrapper.unmount());
  return wrapper;
}

it("imports a first manuscript directly and exposes the busy state", async () => {
  reader.mockResolvedValue(script);
  const wrapper = render(true);
  await flushPromises();
  expect(reader).toHaveBeenCalledTimes(1);
  expect(wrapper.emitted("imported")).toEqual([[script, true]]);
  expect(wrapper.emitted("busy")).toEqual([[false], [true], [false]]);
});

it("does not replace a saved manuscript until the user explicitly imports again", async () => {
  reader.mockResolvedValue(script);
  const wrapper = render(false);
  await flushPromises();
  expect(reader).not.toHaveBeenCalled();
  await wrapper.get("button").trigger("click");
  await flushPromises();
  expect(wrapper.emitted("imported")).toEqual([[script, false]]);
});
