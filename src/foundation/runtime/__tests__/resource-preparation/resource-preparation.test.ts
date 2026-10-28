import { vi } from "vitest";

import { observationScenarios } from "./observation";
import { producerScenarios } from "./producer";
import { retryScenarios } from "./retry";

// The resource engine, its query layer, logging and i18n are replaced once for the whole module
// graph, so every scenario registered below shares a single mock wiring.
const host = vi.hoisted(() => () => import("../support/resource-preparation-host"));
vi.mock("@tauri-apps/plugin-log", async () => (await host()).logPlugin);
vi.mock("../../../i18n/instance", async () => (await host()).keyedI18n);
vi.mock("../../local-resources", async () => (await host()).resourceEngine);
vi.mock("../../local-resources-query", async () => (await host()).resourceQuery);

producerScenarios();
observationScenarios();
retryScenarios();
