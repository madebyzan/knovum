import { z } from "zod";

import type { AgentDescriptor } from "../../shared/contracts/generated/AgentDescriptor";
import type { AgentInstallation } from "../../shared/contracts/generated/AgentInstallation";
import { invokeCommand } from "../ipc/invoke-command";
import { agentDescriptorSchema, agentInstallationSchema } from "./schema/agents";

export async function loadAgentDescriptors(): Promise<AgentDescriptor[]> {
  const descriptors = await invokeCommand<unknown>("plugin:ai|list_agent_descriptors");
  return z.array(agentDescriptorSchema).parse(descriptors);
}

export async function detectAgentInstallations(): Promise<AgentInstallation[]> {
  const runtimes = await invokeCommand<unknown>("plugin:ai|detect_agent_installations");
  return z.array(agentInstallationSchema).parse(runtimes);
}
