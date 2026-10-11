import { describe, expect, it } from "vitest";

import { knowledgeObjectPreview } from "../model/object-presentation";
import { SYSTEM_OBJECT_TYPE_IDS } from "../model/object-types";

describe("knowledgeObjectPreview", () => {
  it("uses the semantic primary field before a generic summary", () => {
    expect(
      knowledgeObjectPreview(SYSTEM_OBJECT_TYPE_IDS.claim, "Generic summary", {
        statement: "The durable claim",
      }),
    ).toBe("The durable claim");
  });

  it("flattens markdown into a compact canvas preview", () => {
    expect(
      knowledgeObjectPreview(
        SYSTEM_OBJECT_TYPE_IDS.evidence,
        "## Finding\n- [Source](https://example.com) `confirmed`",
        {},
      ),
    ).toBe("Finding Source confirmed");
  });
});
