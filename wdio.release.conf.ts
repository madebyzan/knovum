import { config as base } from "./wdio.conf";

export const config: WebdriverIO.Config = {
  ...base,
  specs: ["./tests/release/**/*.release.e2e.ts"],
  mochaOpts: { ui: "bdd", timeout: 180_000 },
};
