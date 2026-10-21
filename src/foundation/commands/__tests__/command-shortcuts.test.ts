import { vi } from "vitest";

import { applicationShortcutScenarios } from "./command-shortcuts/application";
import { textEditingScenarios } from "./command-shortcuts/text-editing";

// The platform probe and logging are replaced once for the whole module graph, so both scenarios
// registered below share a single mock wiring.
const harness = vi.hoisted(() => () => import("./support/shortcut-harness"));
vi.mock("@tauri-apps/plugin-os", async () => (await harness()).osPlugin);
vi.mock("@tauri-apps/plugin-log", async () => (await harness()).logPlugin);

applicationShortcutScenarios();
textEditingScenarios();
