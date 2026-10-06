import { z } from "zod";

import { externalAgentKinds } from "../../../shared/contracts/generated/contract-values";
import type { DesktopAgentSettings } from "../../../shared/contracts/generated/DesktopAgentSettings";
import type { ExternalAgentDescriptor } from "../../../shared/contracts/generated/ExternalAgentDescriptor";
import type { ExternalAgentInstallation } from "../../../shared/contracts/generated/ExternalAgentInstallation";

export const externalAgentKindSchema = z.enum(externalAgentKinds);

export const externalAgentInstallationSchema: z.ZodType<ExternalAgentInstallation> = z
  .object({
    kind: externalAgentKindSchema,
    detected: z.boolean(),
    executablePath: z.string().nullable(),
  })
  .strict();

export const externalAgentDescriptorSchema: z.ZodType<ExternalAgentDescriptor> = z
  .object({
    kind: externalAgentKindSchema,
    displayName: z.string().min(1),
    executableNames: z.array(z.string().min(1)).min(1),
    help: z
      .object({
        connection: z.enum(["executable_cli", "node_adapter", "standalone_server"]),
        documentationUrl: z.string().url().startsWith("https://"),
        setupUrl: z.string().url().startsWith("https://"),
      })
      .strict(),
  })
  .strict();

export const desktopAgentSettingsSchema: z.ZodType<DesktopAgentSettings> = z
  .object({
    kind: externalAgentKindSchema,
    enabled: z.literal(true),
  })
  .strict();
