import { beforeEach, describe, expect, it, vi } from "vitest";

import type { ExternalAgentInstallation } from "../../../shared/contracts/generated/ExternalAgentInstallation";
import type { LocalModelAvailability } from "../../../shared/contracts/generated/LocalModelAvailability";
import { queryClient } from "../../query/query-client";
import { admitAiRequirements } from "../ai-readiness-admission";
import { installAiSettingsNavigation } from "../ai-settings-navigation";
import { DEFAULT_AI_MODEL_SETTINGS } from "../schema/model-settings";

const mocks = vi.hoisted(() => ({
  catalog: vi.fn<() => Promise<LocalModelAvailability[]>>(),
  installations: vi.fn<() => Promise<ExternalAgentInstallation[]>>(),
  info: vi.fn<(message: string, options?: object) => void>(),
  warning: vi.fn<(message: string, options?: object) => void>(),
  settings: {
    settings: undefined as unknown,
    loadError: undefined as unknown,
    ensureLoaded: vi.fn<() => Promise<void>>(),
  },
}));
vi.mock("../../runtime/local-resources", () => ({ getLocalModelCatalog: mocks.catalog }));
vi.mock("../external-agents", () => ({
  loadExternalAgentDescriptors: () =>
    Promise.resolve([{ kind: "codex", displayName: "Codex", executableNames: ["codex"], help: {} }]),
  detectExternalAgentInstallations: mocks.installations,
}));
vi.mock("vue-sonner", () => ({ toast: { info: mocks.info, warning: mocks.warning } }));
vi.mock("../ai-settings-store", () => ({ useAiSettingsStore: () => mocks.settings }));

const whisper: LocalModelAvailability = {
  id: "whisperSmall",
  name: "Whisper Small",
  capability: "transcription",
  operations: [],
  referenceTranscript: "unused",
  group: "audio",
  essential: true,
  selected: true,
  selectable: true,
  fixed: false,
  totalBytes: 2 * 1024 * 1024,
  installedBytes: 0,
  installed: false,
  verified: false,
  memoryAdvisoryBytes: null,
};
const transcription = [{ kind: "localCapability", capability: "transcription" }] as const;

describe("admitAiRequirements", () => {
  beforeEach(() => {
    queryClient.clear();
    mocks.catalog.mockReset();
    mocks.installations.mockReset().mockResolvedValue([]);
    mocks.info.mockReset();
    mocks.warning.mockReset();
    mocks.settings.settings = structuredClone(DEFAULT_AI_MODEL_SETTINGS);
    mocks.settings.loadError = undefined;
    mocks.settings.ensureLoaded.mockReset().mockResolvedValue();
  });

  it("starts a feature whose local model must be downloaded and says so", async () => {
    mocks.catalog.mockResolvedValue([whisper]);

    await expect(admitAiRequirements(transcription)).resolves.toBe(true);

    expect(mocks.info).toHaveBeenCalledOnce();
    expect(mocks.info.mock.calls[0]![0]).toContain("Whisper Small");
    expect(mocks.info.mock.calls[0]![0]).toContain("2.0 MiB");
    expect(mocks.settings.ensureLoaded).not.toHaveBeenCalled();
  });

  it("starts silently when everything is in place", async () => {
    mocks.catalog.mockResolvedValue([{ ...whisper, verified: true, installed: true }]);

    await expect(admitAiRequirements(transcription)).resolves.toBe(true);

    expect(mocks.info).not.toHaveBeenCalled();
    expect(mocks.warning).not.toHaveBeenCalled();
  });

  it("reads the catalog afresh, because a download may have finished since it was drawn", async () => {
    mocks.catalog.mockResolvedValueOnce([whisper]).mockResolvedValueOnce([{ ...whisper, verified: true }]);

    await admitAiRequirements(transcription);
    await admitAiRequirements(transcription);

    expect(mocks.catalog).toHaveBeenCalledTimes(2);
    expect(mocks.info).toHaveBeenCalledOnce();
  });

  it("does not start, and says why, when the catalog cannot be read", async () => {
    mocks.catalog.mockRejectedValue(new Error("catalog unavailable"));

    await expect(admitAiRequirements(transcription)).resolves.toBe(false);

    expect(mocks.catalog).toHaveBeenCalledOnce();
    expect(mocks.warning).toHaveBeenCalledWith(
      "Local models couldn't be checked. Try again in a moment.",
      {},
    );
  });

  it("stops an unconfigured routed feature and offers the Settings pane that fixes it", async () => {
    const navigate = vi.fn<(pane: string) => Promise<void>>().mockResolvedValue();
    installAiSettingsNavigation(navigate);

    await expect(admitAiRequirements([{ kind: "routed", purpose: "translation" }])).resolves.toBe(false);

    expect(mocks.catalog).not.toHaveBeenCalled();
    const [message, options] = mocks.warning.mock.calls[0]!;
    expect(message).toContain("Translation");
    (options as { action: { onClick: () => void } }).action.onClick();
    expect(navigate).toHaveBeenCalledWith("aiProviders");
  });

  it("stops a conversation with a desktop Agent whose command line is not installed", async () => {
    const navigate = vi.fn<(pane: string) => Promise<void>>().mockResolvedValue();
    installAiSettingsNavigation(navigate);
    mocks.settings.settings = {
      ...DEFAULT_AI_MODEL_SETTINGS,
      desktopAgents: [{ kind: "codex", enabled: true }],
    };
    mocks.installations.mockResolvedValue([{ kind: "codex", detected: true, executablePath: null }]);

    await expect(
      admitAiRequirements([
        { kind: "routed", purpose: "knowledge", target: { type: "agent", runtimeKind: "codex" } },
      ]),
    ).resolves.toBe(false);

    const [message, options] = mocks.warning.mock.calls[0]!;
    expect(message).toContain("Codex");
    (options as { action: { onClick: () => void } }).action.onClick();
    expect(navigate).toHaveBeenCalledWith("desktopAgents");
  });

  it("stops a routed feature whose settings failed to load", async () => {
    mocks.settings.loadError = new Error("store unavailable");

    await expect(admitAiRequirements([{ kind: "routed", purpose: "writing" }])).resolves.toBe(false);

    expect(mocks.warning).toHaveBeenCalledOnce();
  });
});
