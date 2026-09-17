import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";

import SpaceCanvasVideoCaption from "../presentation/canvas/nodes/SpaceCanvasVideoCaption.vue";

describe("canvas video caption", () => {
  it("separates a long video title from platform and creator", () => {
    const wrapper = mount(SpaceCanvasVideoCaption, {
      props: {
        id: "video-1",
        data: {
          kind: "video",
          title: "GPT-6 Astra真的很好用！8個應用案例，從實用到幾乎不可能",
          assetPath: "/assets/video.mp4",
          fileName: "video.mp4",
          creator: "孔老師AI研習社",
          origin: {
            kind: "web",
            url: "https://www.youtube.com/watch?v=example",
            provider: "Youtube",
            externalId: "example",
          },
        },
      },
      global: { stubs: { SpaceCanvasNodeStatus: true } },
    });
    expect(wrapper.find("[data-video-title]").classes()).toContain("line-clamp-2");
    expect(wrapper.text()).toContain("YouTube");
    expect(wrapper.text()).toContain("孔老師AI研習社");
    expect(wrapper.find("[data-video-source]").text()).not.toContain("GPT-6 Astra");
  });
});
