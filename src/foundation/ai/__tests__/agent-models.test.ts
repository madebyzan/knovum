import { describe, expect, it } from "vitest";

import { agentKindSchema, agentInstallationSchema } from "../agent-models";

describe("agent-models schema", () => {
  it("validates supported agent kinds", () => {
    expect(agentKindSchema.parse("codex")).toBe("codex");
    expect(agentKindSchema.parse("claude_code")).toBe("claude_code");
    expect(agentKindSchema.parse("gemini_cli")).toBe("gemini_cli");
    expect(agentKindSchema.parse("qoder_cli")).toBe("qoder_cli");
    expect(agentKindSchema.parse("antigravity")).toBe("antigravity");
    expect(() => agentKindSchema.parse("unknown")).toThrow(/invalid/i);
  });

  it("validates agent installation payloads", () => {
    const valid = {
      kind: "claude_code",
      detected: true,
      executablePath: "/usr/local/bin/claude",
    };
    expect(agentInstallationSchema.parse(valid)).toEqual(valid);
  });

  it("represents a desktop installation without claiming an ACP executable exists", () => {
    const installed = { kind: "antigravity", detected: true, executablePath: null };
    expect(agentInstallationSchema.parse(installed)).toEqual(installed);
  });
});
