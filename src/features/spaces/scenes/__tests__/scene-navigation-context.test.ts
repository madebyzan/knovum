import { expect, it, vi } from "vitest";
import { shallowRef } from "vue";

import { createSceneNavigationActions, type SceneNavigationContext } from "../scene-navigation-actions";

const context = vi.hoisted(() => ({ store: undefined as unknown }));
vi.mock("../../session/space-store", () => ({ useSpaceStore: () => context.store }));

it("keeps callbacks bound to the store that owns their scene navigation state", async () => {
  const viewport = { x: 0, y: 0, zoom: 1 };
  const scene = { id: "scene", spaceId: "space" };
  const original = {
    createScene: vi.fn<() => Promise<typeof scene>>().mockResolvedValue(scene),
    announceScene: vi.fn<() => void>(),
    openSceneView: vi.fn<(id: string) => Promise<void>>(),
  };
  context.store = original;
  const state: SceneNavigationContext = {
    scenes: shallowRef([]),
    activeSpace: shallowRef({ id: "space" } as never),
    viewport: shallowRef(viewport),
    recentSceneId: shallowRef(),
    triggerRecentTimer: vi.fn<() => void>(),
    stopRecentTimer: vi.fn<() => void>(),
  };
  const actions = createSceneNavigationActions(state);
  const other = { createScene: vi.fn<() => Promise<void>>() };
  context.store = other;
  await actions.createScene("thumbnail.png");
  expect(original.createScene).toHaveBeenCalledWith(expect.any(String), viewport, "thumbnail.png");
  expect(other.createScene).not.toHaveBeenCalled();
  expect(state.recentSceneId.value).toBe("scene");
  expect(state.triggerRecentTimer).toHaveBeenCalledOnce();
});
