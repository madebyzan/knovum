import { QueryClient, VueQueryPlugin } from "@tanstack/vue-query";
import { mount } from "@vue/test-utils";
import { expect, it, onTestFinished } from "vitest";

import { i18n } from "../../../../foundation/i18n/instance";
import type { AgentExtensionSelection } from "../../../../shared/contracts/generated/AgentExtensionSelection";
import type { McpConnection } from "../../../../shared/contracts/generated/McpConnection";
import ConversationMcpSelection from "../presentation/ConversationMcpSelection.vue";

it("locks the selected MCP tools and catalog controls while the conversation extensions are disabled", async () => {
  const client = new QueryClient();
  const connection: McpConnection = {
    id: "sources",
    connectorId: "sources-definition",
    name: "Sources",
    revision: "one",
    enabled: true,
    tools: [{ name: "search", schemaDigest: "one", readOnly: true, policy: "allow" }],
  };
  const selection: AgentExtensionSelection = {
    pluginIds: [],
    skillIds: [],
    connections: [
      {
        connectionId: connection.id,
        toolNames: ["search"],
        resourceUris: [],
        resourceTemplates: [],
        promptNames: [],
      },
    ],
  };
  const wrapper = mount(ConversationMcpSelection, {
    props: { modelValue: selection, connections: [connection], disabled: true },
    global: { plugins: [i18n, [VueQueryPlugin, { queryClient: client }]] },
  });
  onTestFinished(() => {
    wrapper.unmount();
    client.clear();
  });

  const controls = wrapper.get("fieldset").findAll("button, input, textarea");
  expect(controls.length).toBeGreaterThan(0);
  expect(controls.every((control) => control.element.matches(":disabled"))).toBe(true);
  await wrapper.setProps({ disabled: false });
  expect(controls.every((control) => !control.element.matches(":disabled"))).toBe(true);
  await wrapper.get('fieldset [role="checkbox"]').trigger("click");
  expect(wrapper.emitted("update:modelValue")).toEqual([
    [{ ...selection, connections: [{ ...selection.connections[0], toolNames: [] }] }],
  ]);
});
