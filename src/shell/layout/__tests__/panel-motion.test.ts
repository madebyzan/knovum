import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { effectScope } from "vue";

import { usePanelMotion } from "../use-panel-motion";

beforeEach(() => vi.useFakeTimers());
afterEach(() => vi.useRealTimers());

it("holds revealed panel content back until the geometry transition has finished", async () => {
  const scope = effectScope();
  const motion = scope.run(() => usePanelMotion())!;
  expect({ active: motion.active.value, expanding: motion.expandingRightSidebar.value }).toEqual({
    active: false,
    expanding: false,
  });

  motion.beginRightSidebarExpansion();
  expect(motion.active.value).toBe(true);
  expect(motion.expandingRightSidebar.value).toBe(true);

  // An unrelated panel animation must not reset the pending expansion window.
  motion.begin();
  expect(motion.expandingRightSidebar.value).toBe(true);
  await vi.advanceTimersByTimeAsync(230);
  expect(motion.active.value).toBe(false);
  expect(motion.expandingRightSidebar.value).toBe(false);

  motion.beginRightSidebarExpansion();
  motion.cancel();
  expect(motion.active.value).toBe(false);
  expect(motion.expandingRightSidebar.value).toBe(false);
  scope.stop();
});
