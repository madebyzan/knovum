import { expect, it, vi } from "vitest";
import { shallowRef } from "vue";

import {
  createSceneNavigationActions,
  type SceneNavigationContext,
} from "../scenes/scene-navigation-actions";

const context = vi.hoisted(() => ({ store: undefined as unknown }));
vi.mock("../session/space-store", () => ({ useSpaceStore: () => context.store }));
vi.mock("../workspace/space-menu", () => ({
  popupSpaceContextMenu: vi.fn<() => Promise<void>>(),
  spaceMenuItem: vi.fn<(label: string) => void>(),
}));

it("keeps callbacks bound to the store that owns their scene navigation state", () => {
  const original = {
    updateScene: vi.fn<(id: string, change: { title: string }) => Promise<void>>(),
    openSceneView: vi.fn<(id: string) => Promise<void>>(),
  };
  context.store = original;
  const state: SceneNavigationContext = {
    scenes: shallowRef([]),
    activeSpace: shallowRef(),
    viewport: shallowRef(),
    title: shallowRef(""),
    editingId: shallowRef("scene"),
    editingTitle: shallowRef(" Renamed "),
    recentSceneId: shallowRef(),
    root: shallowRef(),
    triggerRecentTimer: vi.fn<() => void>(),
    stopRecentTimer: vi.fn<() => void>(),
  };
  const actions = createSceneNavigationActions(state);
  const other = { updateScene: vi.fn<(id: string, change: { title: string }) => Promise<void>>() };
  context.store = other;
  actions.finishRename();
  expect(original.updateScene).toHaveBeenCalledWith("scene", { title: "Renamed" });
  expect(other.updateScene).not.toHaveBeenCalled();
  expect(state.editingId.value).toBeUndefined();
});
