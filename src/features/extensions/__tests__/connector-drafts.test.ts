import { QueryClient, VueQueryPlugin } from "@tanstack/vue-query";
import { flushPromises, mount } from "@vue/test-utils";
import { afterEach, beforeEach, expect, it } from "vitest";
import { defineComponent } from "vue";

import { i18n } from "../../../foundation/i18n/i18n";
import {
  settingsLeaveCoordinatorKey,
  type SettingsLeaveParticipant,
} from "../../../shared/contracts/settings-leave";
import { activeConfirmation, resolveConfirmation } from "../../../shared/ui/confirm-action";
import { useConnectorEditing } from "../connectors/use-connector-editing";
import { connectorFixture } from "./support/connector-fixtures";
const Editor = defineComponent({ setup: () => useConnectorEditing(), template: "<div />" });
let wrapper: ReturnType<typeof mount<typeof Editor>>;
let queryClient: QueryClient;
let participant: SettingsLeaveParticipant;
beforeEach(() => {
  queryClient = new QueryClient();
  wrapper = mount(Editor, {
    global: {
      plugins: [i18n, [VueQueryPlugin, { queryClient }]],
      provide: {
        [settingsLeaveCoordinatorKey as symbol]: {
          register: (registered: SettingsLeaveParticipant) => {
            participant = registered;
            return () => {};
          },
          requestLeave: async () => {},
        },
      },
    },
  });
  wrapper.vm.edit(connectorFixture);
});
afterEach(() => {
  resolveConfirmation(false);
  wrapper.unmount();
  queryClient.clear();
});
it("retains incomplete public-field edits when leaving is cancelled", async () => {
  wrapper.vm.publicFields = '{"X-Test":';
  const leaving = participant.prepare();
  expect(activeConfirmation.value).toBeDefined();
  resolveConfirmation(false);
  await expect(leaving).resolves.toBe(false);
  expect(wrapper.vm.publicFields).toBe('{"X-Test":');
  expect(wrapper.vm.editing).toBe(true);
});
it("preserves public-field drafts while changing another field of the same transport", async () => {
  await flushPromises();
  wrapper.vm.publicFields = '{"X-Test":"draft"}';
  wrapper.vm.draft.endpoint = {
    type: "streamable_http",
    url: "https://next.example.com/mcp",
    headers: {},
    allowLocalNetwork: false,
  };
  await flushPromises();
  expect(wrapper.vm.publicFields).toBe('{"X-Test":"draft"}');
});
