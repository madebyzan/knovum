import { z } from "zod";

export const agentKindSchema = z.enum([
  "codex",
  "claude_code",
  "gemini_cli",
  "github_copilot",
  "open_code",
  "qwen_code",
  "qoder_cli",
  "kimi_cli",
  "auggie_cli",
  "cline",
  "cursor",
  "antigravity",
]);

export const agentInstallationSchema = z
  .object({
    kind: agentKindSchema,
    detected: z.boolean(),
    executablePath: z.string().nullable(),
  })
  .strict();

export const agentDescriptorSchema = z
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

export const desktopAgentSettingsSchema = z
  .object({
    kind: agentKindSchema,
    enabled: z.literal(true),
  })
  .strict();

export const agentTargetSchema = z.object({ type: z.literal("agent"), kind: agentKindSchema }).strict();

export type AgentKind = z.infer<typeof agentKindSchema>;
export type AgentInstallation = z.infer<typeof agentInstallationSchema>;
export type AgentDescriptor = z.infer<typeof agentDescriptorSchema>;
export type DesktopAgentSettings = z.infer<typeof desktopAgentSettingsSchema>;
export type AgentTarget = z.infer<typeof agentTargetSchema>;
