import { describe, expect, it } from "vitest";

import { cn } from "../cn";

describe("cn", () => {
  it("merges classes cleanly", () => {
    const isHidden = false;
    expect(cn("px-2 py-1", "bg-raised", undefined, isHidden && "hidden")).toBe("px-2 py-1 bg-raised");
  });

  it("resolves conflicting tailwind classes", () => {
    expect(cn("p-2", "p-4")).toBe("p-4");
    expect(cn("text-foreground-secondary", "text-foreground")).toBe("text-foreground");
  });
});
