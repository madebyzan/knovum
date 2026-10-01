import { mount } from "@vue/test-utils";
import { describe, expect, it, vi } from "vitest";

import { i18n } from "../../../foundation/i18n/i18n";
import SpaceHtmlPreview from "../presentation/SpaceHtmlPreview.vue";

vi.mock("@tauri-apps/api/core", () => ({ convertFileSrc: (path: string) => `asset:${path}` }));

describe("imported HTML preview", () => {
  it("shows the imported page in an iframe without granting script execution", () => {
    const wrapper = mount(SpaceHtmlPreview, {
      props: {
        source: '<h1>Page</h1><script>window.parent.postMessage("unsafe", "*")</script>',
        previewPath: "/spaces/page.html",
        standalone: true,
        title: "Page preview",
      },
      global: { plugins: [i18n] },
    });

    const frame = wrapper.find("iframe");
    expect(frame.attributes("src")).toBe("asset:/spaces/page.html");
    expect(frame.attributes("sandbox")).toBe("");
  });
});
