import { i18n } from "../../../foundation/i18n/i18n";
import type { SpaceMediaAnalysisLabels } from "./actions";

export function spaceMediaAnalysisLabels(): SpaceMediaAnalysisLabels {
  const t = i18n.global.t;
  return {
    title: t("spaces.media.analysisTask"),
    preparing: t("spaces.media.analysisPreparing"),
    transcoding: t("spaces.media.audioPreparing"),
    transcribing: t("spaces.media.transcribing"),
    organizing: t("spaces.media.organizing"),
  };
}
