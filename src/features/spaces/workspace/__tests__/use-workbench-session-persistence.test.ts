import { flushPromises } from "@vue/test-utils";
import { afterEach, describe, expect, it } from "vitest";
import { nextTick, ref, shallowRef } from "vue";

import { createTestSpace } from "../../__tests__/support/session-context";
import type { Space } from "../../model/space";
import { SOURCE_READER_WIDTH_DEFAULT, type SourceReadingTarget } from "../../reading/source-reader-target";
import { defaultSpaceViewSession, type SpaceViewSession } from "../space-view-session";
import { useWorkbenchSessionPersistence } from "../use-workbench-session-persistence";
import { createSpaceCanvasTab, createSpaceEditorTab, type SpaceWorkbenchTab } from "../workbench-tab";

const SESSION_KEY = "knovum.workspace.workbench";

describe("useWorkbenchSessionPersistence", () => {
  afterEach(() => localStorage.clear());

  it("saves the workbench session once ready and omits draft Spaces", async () => {
    const draft = createTestSpace({ id: "draft", kind: "space", title: "Draft" });
    const canvas = createSpaceCanvasTab("inbox");
    const research = createSpaceCanvasTab("research");
    const editor = createSpaceEditorTab("inbox", "video", "clip");
    const retained = new Map<string, { view: SpaceViewSession }>([
      [
        "research",
        {
          view: {
            ...defaultSpaceViewSession(),
            groupScopePath: ["grp-1"],
            activeSceneId: "scene-1",
            outlineExpandedIds: ["sec-1"],
            readingTarget: { nodeId: "doc-1", pane: "read", nonce: 1 },
            readerWidth: 52,
            readerFocusMode: true,
            backgroundPattern: "grid",
          },
        },
      ],
    ]);
    const source = {
      ready: ref(false),
      tabs: ref<readonly SpaceWorkbenchTab[]>([canvas, research, createSpaceCanvasTab("draft"), editor]),
      activeTabId: ref<string | undefined>(editor.id),
      activeSpaceId: ref<string | undefined>("inbox"),
      draftSpace: ref<Space | undefined>(draft),
      groupScopePath: ref<string[]>([]),
      activeSceneId: ref<string | undefined>(),
      outlineExpandedIds: shallowRef<string[]>(["inbox-sec"]),
      readingTarget: shallowRef<SourceReadingTarget | undefined>(),
      readerWidth: shallowRef(SOURCE_READER_WIDTH_DEFAULT),
      readerFocusMode: shallowRef(false),
      backgroundPattern: shallowRef<"dots" | "grid" | "none" | undefined>(),
      runtime: { retained },
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
        { kind: "space", spaceId: "research" },
        { kind: "video", spaceId: "inbox", nodeId: "clip" },
      ],
      activeIndex: 0,
      spaceViews: {
        inbox: { outlineExpandedIds: ["inbox-sec"] },
        research: {
          groupScopePath: ["grp-1"],
          activeSceneId: "scene-1",
          outlineExpandedIds: ["sec-1"],
          readingTarget: { nodeId: "doc-1", pane: "read" },
          readerWidth: 52,
          readerFocusMode: true,
          backgroundPattern: "grid",
        },
      },
    });
  });
});
