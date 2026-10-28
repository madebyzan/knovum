import { vi } from "vitest";

import { lifecycleScenarios } from "./lifecycle";
import { submissionScenarios } from "./submission";

// Voice IPC, run observation and translation are replaced once for the whole module graph, so both
// scenarios registered below share a single mock wiring.
const host = vi.hoisted(() => () => import("../support/voice-candidate-host"));
vi.mock("../../voice-candidate-ipc", async () => (await host()).voiceHost);
vi.mock("../../ipc", async () => (await host()).voiceHost);
vi.mock("../../../runs/resource-run-observation", async () => (await host()).voiceHost);
vi.mock("vue-i18n", async (original) => (await host()).keyedI18n(original));

lifecycleScenarios();
submissionScenarios();
