import { toast } from "vue-sonner";

import { formatBytes } from "../format/bytes";
import { i18n } from "../i18n/instance";
import { routingDefaultsKey } from "./ai-execution";
import type { AiReadiness } from "./ai-readiness";
import { openAiSettings, type AiSettingsPane } from "./ai-settings-navigation";

const UNAVAILABLE_KEY_BY_SOURCE = {
  settings: "ai.configuration.unavailable",
  models: "ai.readiness.modelsUnavailable",
  agents: "ai.readiness.agentsUnavailable",
  platform: "ai.readiness.platformUnavailable",
} as const satisfies Record<Extract<AiReadiness, { state: "unavailable" }>["source"], string>;

export interface AiReadinessPrompt {
  /** Information never stops the feature; attention means it cannot run as things stand. */
  readonly tone: "information" | "attention";
  readonly message: string;
  readonly action?: { readonly label: string; readonly pane: AiSettingsPane };
}

/**
 * The one wording of every readiness state. `before` explains it beside a control that has not run
 * yet; `starting` reports it at the moment the person started the feature.
 */
export function describeAiReadiness(
  readiness: AiReadiness,
  moment: "before" | "starting",
): AiReadinessPrompt | undefined {
  const t = i18n.global.t;
  switch (readiness.state) {
    case "checking":
    case "ready":
      return undefined;
    case "download":
      return {
        tone: "information",
        message: t(moment === "before" ? "ai.readiness.download" : "ai.readiness.downloading", {
          model: new Intl.ListFormat(i18n.global.locale.value, { type: "conjunction" }).format(
            readiness.models.map(({ name }) => name),
          ),
          size: formatBytes(readiness.bytes),
        }),
        action: { label: t("ai.readiness.manageModels"), pane: "localModels" },
      };
    case "configure":
      return {
        tone: "attention",
        message: t("ai.readiness.configure", {
          purpose: t(`settings.routing.routes.${routingDefaultsKey(readiness.purpose)}.label`),
        }),
        action: { label: t("ai.configuration.open"), pane: readiness.pane },
      };
    case "install":
      return {
        tone: "attention",
        message: t("ai.readiness.installAgent", { agent: readiness.agent.name }),
        action: { label: t("ai.readiness.manageAgents"), pane: "desktopAgents" },
      };
    case "unavailable":
      return {
        tone: "attention",
        message: t(UNAVAILABLE_KEY_BY_SOURCE[readiness.source]),
      };
  }
}

/** Reports a started feature's readiness where it was started, without blocking anything itself. */
export function announceAiReadiness(readiness: AiReadiness) {
  const prompt = describeAiReadiness(readiness, "starting");
  if (!prompt) return;
  const { action } = prompt;
  const options = action
    ? { action: { label: action.label, onClick: () => void openAiSettings(action.pane) } }
    : {};
  if (prompt.tone === "information") toast.info(prompt.message, options);
  else toast.warning(prompt.message, options);
}
