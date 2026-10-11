import { mount, shallowMount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";

import type { AppTask } from "../../../foundation/tasks/task-model";
import { useTaskStore } from "../../../foundation/tasks/task-store";
import TaskCenter from "../TaskCenter.vue";
import TaskCenterActivity from "../TaskCenterActivity.vue";

vi.mock("vue-i18n", () => ({ useI18n: () => ({ t: (key: string) => key }) }));

const baseTask: AppTask = {
  id: "job-1",
  title: "Visual analysis",
  status: "failed",
  startedAt: 1,
  scope: { kind: "space", id: "space-1", label: "Research" },
  subject: { kind: "node", id: "node-1", label: "Market map", typeLabel: "Image" },
};

function mountActivity(tasks: AppTask[]) {
  return mount(TaskCenterActivity, {
    props: { tasks },
    global: {
      stubs: {
        UiIconButton: {
          props: ["label"],
          emits: ["click"],
          template: '<button :aria-label="label" @click="$emit(\'click\')"><slot /></button>',
        },
        UiProgress: true,
        UiSpinner: true,
      },
    },
  });
}

describe("TaskCenter", () => {
  beforeEach(() => setActivePinia(createPinia()));

  it("keeps a stable entry point and communicates the empty state", () => {
    const wrapper = shallowMount(TaskCenter, {
      global: {
        stubs: {
          UiPopover: { template: "<div><slot name='trigger' /><slot /></div>" },
        },
      },
    });

    expect(wrapper.get('button[aria-label="tasks.summary"]').text()).toContain("0");
    expect(wrapper.text()).toContain("tasks.empty");
  });

  it("filters tasks by the current workspace scope", async () => {
    const store = useTaskStore();
    store.tasks.push(baseTask, {
      ...baseTask,
      id: "job-2",
      scope: { kind: "space", id: "space-2", label: "Archive" },
    });
    const wrapper = shallowMount(TaskCenter, {
      props: { currentScope: baseTask.scope },
      global: {
        stubs: {
          UiPopover: { template: "<div><slot name='trigger' /><slot /></div>" },
        },
      },
    });

    const currentSpaceButton = wrapper.findAll("button").find((button) => button.text() === "Research");
    await currentSpaceButton?.trigger("click");

    expect(wrapper.getComponent(TaskCenterActivity).props("tasks")).toEqual([baseTask]);
  });

  it("shows task ownership and keeps durable failures actionable", () => {
    const wrapper = mountActivity([{ ...baseTask, durable: true, retryable: true }]);

    expect(wrapper.text()).toContain("Visual analysis");
    expect(wrapper.text()).toContain("Research");
    expect(wrapper.text()).toContain("Image");
    expect(wrapper.text()).toContain("Market map");
    expect(wrapper.text()).toContain("tasks.status.failed");
    expect(wrapper.find('[aria-label="common.retry"]').exists()).toBe(true);
    expect(wrapper.find('[aria-label="tasks.dismiss"]').exists()).toBe(false);
  });

  it("preserves valid controls for durable queued and terminal tasks", () => {
    const wrapper = mountActivity([
      { ...baseTask, status: "queued", durable: true, cancellable: true },
      { ...baseTask, id: "job-2", status: "succeeded", durable: true },
    ]);

    expect(wrapper.findAll('[aria-label="tasks.cancel"]')).toHaveLength(1);
    expect(wrapper.findAll('[aria-label="tasks.dismiss"]')).toHaveLength(1);
  });

  it("keeps observation recovery and cancellation available for attention tasks", () => {
    const wrapper = mountActivity([
      {
        ...baseTask,
        status: "attention",
        durable: true,
        openable: true,
        cancellable: true,
        retryable: true,
      },
    ]);

    expect(wrapper.find('[aria-label="tasks.openSubject"]').exists()).toBe(true);
    expect(wrapper.find('[aria-label="tasks.cancel"]').exists()).toBe(true);
    expect(wrapper.find('[aria-label="common.retry"]').exists()).toBe(true);
    expect(wrapper.find('[aria-label="tasks.dismiss"]').exists()).toBe(false);
  });
});
