import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";

import TaskActivityIndicator from "../TaskActivityIndicator.vue";

describe("TaskActivityIndicator", () => {
  it("renders waveform variant by default", () => {
    const wrapper = mount(TaskActivityIndicator);
    expect(wrapper.findAll(".activity-bar")).toHaveLength(3);
  });

  it("renders orbit variant with balanced nodes", () => {
    const wrapper = mount(TaskActivityIndicator, { props: { variant: "orbit" } });
    expect(wrapper.find(".orbit-spinner").exists()).toBe(true);
    expect(wrapper.findAll("circle")).toHaveLength(3);
  });

  it("renders pulse beacon variant", () => {
    const wrapper = mount(TaskActivityIndicator, { props: { variant: "pulse" } });
    expect(wrapper.find(".beacon-halo").exists()).toBe(true);
  });

  it("renders precision ring variant with progress", () => {
    const wrapper = mount(TaskActivityIndicator, {
      props: { variant: "ring", progress: 50 },
    });
    const circles = wrapper.findAll("circle");
    expect(circles).toHaveLength(2);
    expect(circles[1]?.attributes("stroke-dashoffset")).toBeDefined();
  });
});
