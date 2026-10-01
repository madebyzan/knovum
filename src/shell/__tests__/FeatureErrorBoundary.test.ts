import { error as logError } from "@tauri-apps/plugin-log";
import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { defineAsyncComponent, defineComponent, h, onUpdated, onUnmounted, ref, watch } from "vue";
import { createI18n } from "vue-i18n";

import FeatureErrorBoundary from "../FeatureErrorBoundary.vue";

vi.mock("@tauri-apps/plugin-log", () => ({ error: vi.fn<typeof logError>().mockResolvedValue(undefined) }));

beforeEach(() => vi.clearAllMocks());

function openRegion(content: ReturnType<typeof defineComponent>) {
  const reportError = vi.fn<() => void>();
  const wrapper = mount(FeatureErrorBoundary, {
    props: { resetKey: "first" },
    slots: { default: () => h(content) },
    global: {
      plugins: [createI18n({ legacy: false, locale: "en", messages: { en: {} }, missingWarn: false })],
      config: { errorHandler: reportError, warnHandler: vi.fn<() => void>() },
    },
  });
  return { wrapper, reportError };
}

describe("FeatureErrorBoundary", () => {
  it.each(["setup", "render", "loader"])(
    "isolates a failed %s without exposing diagnostics",
    async (stage) => {
      const failure = new Error("private technical details");
      const content =
        stage === "loader"
          ? defineAsyncComponent(async () => {
              throw failure;
            })
          : defineComponent({
              setup() {
                if (stage === "setup") throw failure;
                return () => {
                  throw failure;
                };
              },
            });
      const { wrapper, reportError } = openRegion(content);
      await flushPromises();

      expect(wrapper.find('[data-testid="content-render-error"]').exists()).toBe(true);
      expect(wrapper.find("pre").exists()).toBe(false);
      expect(wrapper.text()).not.toContain(failure.message);
      expect(reportError).not.toHaveBeenCalled();
      expect(logError).toHaveBeenCalledTimes(1);
      wrapper.unmount();
    },
  );

  it("keeps event failures, input and component state inside the mounted region", async () => {
    const failure = new Error("operation failed");
    const unmounted = vi.fn<() => void>();
    const { wrapper, reportError } = openRegion(
      defineComponent({
        setup() {
          onUnmounted(unmounted);
          return () =>
            h("div", [
              h("input"),
              h(
                "button",
                {
                  onClick: () => {
                    throw failure;
                  },
                },
                "Run",
              ),
            ]);
        },
      }),
    );
    await wrapper.get("input").setValue("unfinished work");
    await wrapper.get("button").trigger("click");

    expect(wrapper.get("input").element.value).toBe("unfinished work");
    expect(unmounted).not.toHaveBeenCalled();
    expect(reportError).toHaveBeenCalledWith(failure, expect.anything(), expect.any(String));
    expect(logError).not.toHaveBeenCalled();
    wrapper.unmount();
  });

  it("keeps watcher and lifecycle-effect failures from replacing content", async () => {
    const { wrapper, reportError } = openRegion(
      defineComponent({
        setup() {
          const value = ref(0);
          watch(value, () => {
            throw new Error("watch failed");
          });
          onUpdated(() => {
            throw new Error("update effect failed");
          });
          return () => h("button", { onClick: () => value.value++ }, `Change ${value.value}`);
        },
      }),
    );
    await wrapper.get("button").trigger("click");

    expect(wrapper.get("button").text()).toBe("Change 1");
    expect(reportError).toHaveBeenCalledTimes(2);
    expect(logError).not.toHaveBeenCalled();
    wrapper.unmount();
  });

  it.each(["retry", "navigation"])("remounts only the failed region on %s", async (action) => {
    let broken = true;
    const attempts = vi.fn<() => void>();
    const { wrapper } = openRegion(
      defineComponent({
        setup() {
          attempts();
          return () => {
            if (broken) throw new Error("render failed");
            return h("p", "Recovered content");
          };
        },
      }),
    );
    await flushPromises();
    expect(attempts).toHaveBeenCalledTimes(1);
    broken = false;
    if (action === "retry") await wrapper.get("button").trigger("click");
    else await wrapper.setProps({ resetKey: "second" });

    expect(wrapper.text()).toBe("Recovered content");
    expect(attempts).toHaveBeenCalledTimes(2);
    wrapper.unmount();
  });

  it("leaves navigation mounted while content fails and reopens", async () => {
    let broken = true;
    const navigationUnmounted = vi.fn<() => void>();
    const navigation = defineComponent({
      setup() {
        onUnmounted(navigationUnmounted);
        return () => h("input");
      },
    });
    const content = defineComponent({
      setup: () => () => {
        if (broken) throw new Error("broken content");
        return h("p", "Recovered");
      },
    });
    const { wrapper } = openRegion(
      defineComponent({
        setup: () => () =>
          h("div", [h(navigation), h(FeatureErrorBoundary, { resetKey: "nested" }, () => h(content))]),
      }),
    );
    await wrapper.get("input").setValue("navigation state");
    broken = false;
    await wrapper.get("button").trigger("click");

    expect(wrapper.get("input").element.value).toBe("navigation state");
    expect(navigationUnmounted).not.toHaveBeenCalled();
    expect(wrapper.get("p").text()).toBe("Recovered");
    wrapper.unmount();
  });
});
