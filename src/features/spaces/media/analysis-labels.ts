import { i18n } from "../../../foundation/i18n/instance";
import type { SpaceMediaAnalysisLabels } from "./actions";

export function spaceMediaAnalysisLabels(): SpaceMediaAnalysisLabels {
  const t = i18n.global.t;
  return {
    title: t("spaces.media.analysis.task"),
    preparing: t("spaces.media.analysis.preparing"),
    transcoding: t("spaces.media.analysis.audioPreparing"),
    transcribing: t("spaces.media.analysis.transcribing"),
    organizing: t("spaces.media.analysis.organizing"),
  };
}
