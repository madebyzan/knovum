import { mount } from "@vue/test-utils";
import { expect, it, vi } from "vitest";
import { shallowRef } from "vue";
import { createI18n } from "vue-i18n";

import { useSpaceStructureCommands } from "../commands/use-space-structure-commands";
import type { useSpaceStore } from "../session/space-store";

const context = vi.hoisted(() => ({ store: undefined as unknown, resolve: vi.fn<() => void>() }));
vi.mock("../session/space-store", () => ({
  useSpaceStore: () => {
    context.resolve();
    return context.store;
  },
}));

it("captures the store in setup and translates commands and new groups in the active scope", async () => {
  const store = { executeSpaceEdit: vi.fn<ReturnType<typeof useSpaceStore>["executeSpaceEdit"]>() };
  store.executeSpaceEdit.mockResolvedValue({
    mutationId: "edit",
    appliedAtRevision: 1,
    selectionSuggestion: [],
  });
  context.store = store;
  context.resolve.mockClear();
  const locale = createI18n({
    legacy: false,
    locale: "en",
    messages: {
      en: {
        common: { rename: "Rename" },
        spaces: { canvas: { groups: { create: "Group", ungroup: "Ungroup", untitled: "Untitled" } } },
      },
      fr: {
        common: { rename: "Renommer" },
        spaces: { canvas: { groups: { create: "Grouper", ungroup: "Dégrouper", untitled: "Sans titre" } } },
      },
    },
  });
  const selection = shallowRef(["first"]);
  let commands!: ReturnType<typeof useSpaceStructureCommands>;
  const wrapper = mount(
    {
      setup() {
        commands = useSpaceStructureCommands({
          selectedStructureNodeIds: () => selection.value,
          sidebar: { reveal: vi.fn<() => void>() },
        });
        return () => null;
      },
    },
    { global: { plugins: [locale] } },
  );
  try {
    expect(commands.value[0]?.title).toBe("Group");
    locale.global.locale.value = "fr";
    selection.value = ["second"];
    expect(commands.value[0]?.title).toBe("Grouper");
    expect(context.resolve).toHaveBeenCalledOnce();
    context.store = { executeSpaceEdit: vi.fn<typeof store.executeSpaceEdit>() };
    await commands.value[0]?.run();
    expect(store.executeSpaceEdit).toHaveBeenCalledWith(
      expect.objectContaining({
        kind: "group.create",
        rootIds: ["second"],
        title: "Sans titre",
      }),
    );
  } finally {
    wrapper.unmount();
  }
});
