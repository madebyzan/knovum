import { z } from "zod";

import type { ExternalAgentDescriptor } from "../../shared/contracts/generated/ExternalAgentDescriptor";
import type { ExternalAgentInstallation } from "../../shared/contracts/generated/ExternalAgentInstallation";
import { invokeCommand } from "../ipc/invoke-command";
import { externalAgentDescriptorSchema, externalAgentInstallationSchema } from "./schema/external-agents";

export async function loadExternalAgentDescriptors(): Promise<ExternalAgentDescriptor[]> {
  const descriptors = await invokeCommand<unknown>("plugin:ai|list_external_agent_descriptors");
  return z.array(externalAgentDescriptorSchema).parse(descriptors);
}

export async function detectExternalAgentInstallations(): Promise<ExternalAgentInstallation[]> {
  const runtimes = await invokeCommand<unknown>("plugin:ai|detect_external_agent_installations");
  return z.array(externalAgentInstallationSchema).parse(runtimes);
}
