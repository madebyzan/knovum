import { defineStore } from "pinia";

import { normalizeError } from "../errors/app-error";

export type BootstrapStatus = "starting" | "ready" | "failed";

export const useBootstrapStore = defineStore("bootstrap", {
  state: () => ({
    status: "starting" as BootstrapStatus,
    failure: undefined as Error | undefined,
  }),
  actions: {
    // The startup watchdog can fail the bootstrap while startup work is still in flight, so the
    // first terminal outcome wins and a late result cannot resurrect the revealed error state.
    ready() {
      if (this.status !== "starting") return;
      this.status = "ready";
    },
    fail(reason: unknown) {
      if (this.status !== "starting") return;
      this.status = "failed";
      this.failure = normalizeError(reason);
    },
  },
});
