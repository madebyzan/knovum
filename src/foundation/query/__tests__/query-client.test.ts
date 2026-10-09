import { onlineManager } from "@tanstack/vue-query";
import { afterEach, describe, expect, it, vi } from "vitest";

import { queryClient } from "../query-client";

afterEach(() => {
  onlineManager.setOnline(true);
  queryClient.clear();
});

describe("local query execution while the network is offline", () => {
  it("runs a query while the browser reports offline", async () => {
    onlineManager.setOnline(false);
    const load = vi.fn<() => Promise<string>>(async () => "local value");

    const value = await queryClient.fetchQuery({ queryKey: ["offline-local-query"], queryFn: load });

    expect(value).toBe("local value");
    expect(load).toHaveBeenCalledOnce();
  });

  it("runs a mutation while the browser reports offline", async () => {
    onlineManager.setOnline(false);
    const save = vi.fn<(value: string) => Promise<string>>(async (value) => value);
    const mutation = queryClient.getMutationCache().build(queryClient, { mutationFn: save });

    const value = await mutation.execute("saved locally");

    expect(value).toBe("saved locally");
    expect(save).toHaveBeenCalledOnce();
  });
});
