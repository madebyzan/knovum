import { describe, expect, it } from "vitest";

import { externalAgentKindSchema, externalAgentInstallationSchema } from "../external-agents";

describe("agent-models schema", () => {
  it("validates supported agent kinds", () => {
    expect(externalAgentKindSchema.parse("codex")).toBe("codex");
    expect(externalAgentKindSchema.parse("claude_code")).toBe("claude_code");
    expect(externalAgentKindSchema.parse("gemini_cli")).toBe("gemini_cli");
    expect(externalAgentKindSchema.parse("qoder_cli")).toBe("qoder_cli");
    expect(externalAgentKindSchema.parse("antigravity")).toBe("antigravity");
    expect(() => externalAgentKindSchema.parse("unknown")).toThrow(/invalid/i);
  });

  it("validates agent installation payloads", () => {
    const valid = {
      kind: "claude_code",
      detected: true,
      executablePath: "/usr/local/bin/claude",
    };
    expect(externalAgentInstallationSchema.parse(valid)).toEqual(valid);
  });

  it("represents a desktop installation without claiming an ACP executable exists", () => {
    const installed = { kind: "antigravity", detected: true, executablePath: null };
    expect(externalAgentInstallationSchema.parse(installed)).toEqual(installed);
  });
});
