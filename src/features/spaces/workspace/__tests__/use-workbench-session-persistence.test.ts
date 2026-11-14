import { flushPromises } from "@vue/test-utils";
import { afterEach, describe, expect, it } from "vitest";
import { nextTick, ref } from "vue";

import { createTestSpace } from "../../__tests__/support/session-context";
import type { Space } from "../../model/space";
import { useWorkbenchSessionPersistence } from "../use-workbench-session-persistence";
import { createSpaceCanvasTab, createSpaceEditorTab, type SpaceWorkbenchTab } from "../workbench-tab";

const SESSION_KEY = "knovum.workspace.workbench";

describe("useWorkbenchSessionPersistence", () => {
  afterEach(() => localStorage.clear());

  it("saves the strip once ready and leaves out the tabs of an unsaved draft", async () => {
    const draft = createTestSpace({ id: "draft", kind: "space", title: "Draft" });
    const canvas = createSpaceCanvasTab("inbox");
    const editor = createSpaceEditorTab("inbox", "video", "clip");
    const source = {
      ready: ref(false),
      tabs: ref<readonly SpaceWorkbenchTab[]>([canvas, createSpaceCanvasTab("draft"), editor]),
      activeTabId: ref<string | undefined>(editor.id),
      draftSpace: ref<Space | undefined>(draft),
    };
    useWorkbenchSessionPersistence(source);

    source.activeTabId.value = canvas.id;
    await nextTick();
    expect(localStorage.getItem(SESSION_KEY)).toBeNull();

    source.ready.value = true;
    await flushPromises();
    expect(JSON.parse(localStorage.getItem(SESSION_KEY)!)).toEqual({
      tabs: [
        { kind: "space", spaceId: "inbox" },
        { kind: "video", spaceId: "inbox", nodeId: "clip" },
      ],
      activeIndex: 0,
    });
  });
});
