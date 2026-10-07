import { mount } from "@vue/test-utils";
import { afterEach, expect, it, vi } from "vitest";

import { i18n } from "../../../foundation/i18n/i18n";
import SpaceGlobalFind from "../presentation/SpaceGlobalFind.vue";
import type { GlobalSearchMatch } from "../search/search-match";

const { select } = vi.hoisted(() => ({ select: vi.fn<(match: GlobalSearchMatch) => void>() }));

vi.mock("@tauri-apps/plugin-os", () => ({ platform: () => "macos" }));

vi.mock("../search/use-global-find", async () => {
  const { shallowRef } = await import("vue");
  return {
    useGlobalFind: () => ({
      query: shallowRef("Section"),
      matches: shallowRef<GlobalSearchMatch[]>([
        {
          kind: "structure",
          structureNodeId: "section-id",
          nodeKind: "section",
          title: "Section",
          spaceId: "space-id",
          spaceTitle: "Space",
        },
      ]),
      searching: shallowRef(false),
      failed: shallowRef(false),
      retry: vi.fn<() => void>(),
      select,
    }),
  };
});

let wrapper: ReturnType<typeof mount> | undefined;

afterEach(() => {
  wrapper?.unmount();
  document.body.replaceChildren();
  select.mockClear();
});

it.each(["click", "keyboard"])("opens the matched node through %s selection", async (input) => {
  wrapper = mount(SpaceGlobalFind, {
    props: { open: true },
    attachTo: document.body,
    global: { plugins: [i18n], stubs: { DialogPortal: { template: "<slot />" } } },
  });
  const option = wrapper.get("[role='option']");
  if (input === "click") await option.trigger("click");
  else {
    const filter = wrapper.get("input");
    await filter.trigger("keydown", { key: "ArrowDown" });
    await filter.trigger("keydown", { key: "Enter" });
  }
  expect(select).toHaveBeenCalledExactlyOnceWith(
    expect.objectContaining({ spaceId: "space-id", structureNodeId: "section-id" }),
  );
});
