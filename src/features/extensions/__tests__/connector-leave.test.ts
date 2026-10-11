import { QueryClient, VueQueryPlugin } from "@tanstack/vue-query";
import { flushPromises, mount } from "@vue/test-utils";
import { expect, it, onTestFinished } from "vitest";

import { i18n } from "../../../foundation/i18n/instance";
import {
  settingsLeaveCoordinatorKey,
  type SettingsLeaveParticipant,
} from "../../../shared/contracts/settings-leave";
import { resolveConfirmation } from "../../../shared/ui/confirm-action";
import ConnectorConnections from "../connectors/components/ConnectorConnections.vue";
import { connectorFixture, mcpConnectionFixture } from "./support/connector-fixtures";

const connection = mcpConnectionFixture("work", "Work");

it("keeps parent and nested connector drafts when the nested discard is cancelled", async () => {
  let outerParticipant!: SettingsLeaveParticipant;
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const wrapper = mount(ConnectorConnections, {
    props: { connector: connectorFixture, connections: [connection], busy: false },
    global: {
      plugins: [i18n, [VueQueryPlugin, { queryClient }]],
      mocks: { $t: (key: string) => key },
      provide: {
        [settingsLeaveCoordinatorKey as symbol]: {
          register(participant: SettingsLeaveParticipant) {
            outerParticipant = participant;
            return () => {};
          },
          requestLeave: async () => {},
        },
      },
    },
  });
  onTestFinished(() => {
    resolveConfirmation(false);
    wrapper.unmount();
    queryClient.clear();
  });
  await wrapper.get('[data-testid="add-mcp-connection"]').trigger("click");
  await wrapper.get('[data-testid="mcp-connection-name"]').setValue("Research");
  await wrapper.get('[data-testid="mcp-connection-work"] button[aria-expanded="false"]').trigger("click");
  await flushPromises();
  const connectionName = wrapper.findAll("form").at(-1)!.get("input");
  await connectionName.setValue("Renamed Work");

  const leaving = outerParticipant.prepare();
  resolveConfirmation(true);
  await flushPromises();
  resolveConfirmation(false);

  await expect(leaving).resolves.toBe(false);
  expect(wrapper.get('[data-testid="mcp-connection-name"]').element).toHaveProperty("value", "Research");
  expect(connectionName.element).toHaveProperty("value", "Renamed Work");
});
