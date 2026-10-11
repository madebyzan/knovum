import { mount } from "@vue/test-utils";
import { expect, it, vi } from "vitest";
import { defineComponent, nextTick, shallowRef } from "vue";

import type { Space } from "../../model/space";
import { useLibraryViewport } from "../presentation/library/use-library-viewport";

it("mounts a newly focused or renamed row before the scroll event arrives", async () => {
  const element = document.createElement("div");
  Object.defineProperties(element, {
    offsetHeight: { value: 400 },
    offsetWidth: { value: 500 },
  });
  document.body.append(element);
  const editingId = shallowRef<string>();
  const spaces = shallowRef(Array.from({ length: 5000 }, (_, index) => ({ id: `space${index}` }) as Space));
  let viewport!: ReturnType<typeof useLibraryViewport>;
  const wrapper = mount(
    defineComponent({
      setup() {
        viewport = useLibraryViewport({
          spaces,
          layout: shallowRef("list" as const),
          root: shallowRef(element),
          host: shallowRef(element),
          scroll: () => element,
          editingId,
        });
        return () => null;
      },
    }),
  );
  await vi.waitFor(() => expect(viewport.rows.value.length).toBeGreaterThan(0));
  expect(viewport.rows.value.some(({ index }) => index === 4999)).toBe(false);
  await viewport.reveal("space4999");
  expect(viewport.rows.value.some(({ index }) => index === 4999)).toBe(true);
  editingId.value = "space2500";
  await nextTick();
  expect(viewport.rows.value.some(({ index }) => index === 2500)).toBe(true);
  expect(viewport.rows.value.length).toBeLessThan(20);
  wrapper.unmount();
  element.remove();
});
