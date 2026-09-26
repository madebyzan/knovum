import { z } from "zod";

import type { AgentDescriptor } from "../../shared/contracts/generated/AgentDescriptor";
import type { AgentInstallation } from "../../shared/contracts/generated/AgentInstallation";
import { agentKinds } from "../../shared/contracts/generated/contract-values";
import type { DesktopAgentSettings } from "../../shared/contracts/generated/DesktopAgentSettings";

export const agentKindSchema = z.enum(agentKinds);

export const agentInstallationSchema: z.ZodType<AgentInstallation> = z
  .object({
    kind: agentKindSchema,
    detected: z.boolean(),
    executablePath: z.string().nullable(),
  })
  .strict();

export const agentDescriptorSchema: z.ZodType<AgentDescriptor> = z
  .object({
    kind: agentKindSchema,
    displayName: z.string().min(1),
    executableNames: z.array(z.string().min(1)).min(1),
    help: z
      .object({
        connection: z.enum(["native_cli", "node_adapter", "standalone_server"]),
        documentationUrl: z.string().url().startsWith("https://"),
        setupUrl: z.string().url().startsWith("https://"),
      })
      .strict(),
  })
  .strict();

export const desktopAgentSettingsSchema: z.ZodType<DesktopAgentSettings> = z
  .object({
    kind: agentKindSchema,
    enabled: z.literal(true),
  })
  .strict();

export type { AgentKind } from "../../shared/contracts/generated/AgentKind";
export type { AgentInstallation } from "../../shared/contracts/generated/AgentInstallation";
export type { AgentDescriptor } from "../../shared/contracts/generated/AgentDescriptor";
export type { DesktopAgentSettings } from "../../shared/contracts/generated/DesktopAgentSettings";
