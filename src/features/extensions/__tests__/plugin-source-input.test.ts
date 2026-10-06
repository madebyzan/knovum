import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, describe, expect, it, vi } from "vitest";
const { open } = vi.hoisted(() => ({ open: vi.fn<() => Promise<string | null>>() }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open }));
import PluginSourceInput from "../plugins/PluginSourceInput.vue";
function sourceForm() {
  return mount(PluginSourceInput, {
    props: { busy: false },
    global: { mocks: { $t: (key: string) => key } },
  });
}
describe("plugin source review", () => {
  beforeEach(() => vi.clearAllMocks());
  it("uses a native picker and only emits an explicit source inspection", async () => {
    open.mockResolvedValue("/reviewed/plugin");
    const form = sourceForm();
    await form.find("button[type=button]").trigger("click");
    await flushPromises();
    expect(open).toHaveBeenCalledWith({ directory: true, multiple: false });
    expect(form.emitted("inspect")).toBeUndefined();
    await form.find("form").trigger("submit");
    expect(form.emitted("inspect")).toEqual([[{ type: "directory", path: "/reviewed/plugin" }]]);
  });
  it("retains a selected Git revision and subdirectory without changing transport", async () => {
    const form = sourceForm();
    await form.get('[data-testid="plugin-source-type"]').setValue("git");
    const inputs = form.findAll("input");
    await inputs[0]!.setValue("https://example.com/repo.git");
    await inputs[1]!.setValue("refs/tags/reviewed");
    await inputs[2]!.setValue("plugins/example");
    await form.find("form").trigger("submit");
    expect(form.emitted("inspect")).toEqual([
      [
        {
          type: "git",
          url: "https://example.com/repo.git",
          reference: "refs/tags/reviewed",
          subdirectory: "plugins/example",
        },
      ],
    ]);
  });
  it("displays native picker failures without emitting a package request", async () => {
    open.mockRejectedValue(new Error("Directory access denied"));
    const form = sourceForm();
    await form.find("button[type=button]").trigger("click");
    await flushPromises();
    expect(form.get('[role="alert"]').text()).toContain("Directory access denied");
    expect(form.emitted("inspect")).toBeUndefined();
  });
});
