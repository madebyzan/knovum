import { createPinia, setActivePinia } from "pinia";
import { beforeEach, expect, it } from "vitest";

import { useNodeOperationDraftsStore } from "../node-operation-drafts-store";

beforeEach(() => setActivePinia(createPinia()));

it("leases one node operation at a time without a draft taxonomy", () => {
  const drafts = useNodeOperationDraftsStore();
  const first = drafts.begin("node-a");

  expect(first).toBeTypeOf("number");
  expect(drafts.active).toEqual({ id: first, nodeId: "node-a" });
  expect(drafts.begin("node-b")).toBeUndefined();

  drafts.end(first!);
  expect(drafts.begin("node-b")).toBeTypeOf("number");
  expect(drafts.active?.nodeId).toBe("node-b");
});
