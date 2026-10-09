import { createPinia, setActivePinia } from "pinia";
import { beforeEach, expect, it } from "vitest";

import { useNodeOperationDrafts } from "../session/node-operation-drafts";

beforeEach(() => setActivePinia(createPinia()));

it("leases one node operation at a time without a draft taxonomy", () => {
  const drafts = useNodeOperationDrafts();
  const first = drafts.begin("node-a");

  expect(first).toBeTypeOf("number");
  expect(drafts.active).toEqual({ id: first, nodeId: "node-a" });
  expect(drafts.begin("node-b")).toBeUndefined();

  drafts.end(first!);
  expect(drafts.begin("node-b")).toBeTypeOf("number");
  expect(drafts.active?.nodeId).toBe("node-b");
});
