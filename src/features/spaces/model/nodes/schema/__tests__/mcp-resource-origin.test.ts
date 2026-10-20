import { describe, expect, it } from "vitest";

import { externalContentSourceSchema } from "../external-source";
describe("MCP resource origin", () => {
  it("keeps resource origin as closed non-secret provenance", () => {
    const source = {
      type: "mcp_resource",
      connectionId: "connection",
      connectionRevision: "revision",
      catalogDigest: "a".repeat(64),
      uri: "service://reviewed",
      contentDigest: "b".repeat(64),
      capturedAt: 1,
    };
    expect(externalContentSourceSchema.parse(source)).toEqual(source);
    expect(externalContentSourceSchema.safeParse({ ...source, token: "credential" }).success).toBe(false);
  });
});
