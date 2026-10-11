import { i18n } from "../../../../foundation/i18n/instance";
import type { CaptureIssueReason } from "../../contracts/generated/CaptureIssueReason";

export const CAPTURE_ISSUE_LABELS: Record<CaptureIssueReason, string> = {
  "resource-unavailable": "resourceUnavailable",
  "derivation-incomplete": "derivationIncomplete",
  "execution-spec-unavailable": "executionSpecUnavailable",
  "derivation-inputs-unavailable": "derivationInputsUnavailable",
  "source-unavailable": "sourceUnavailable",
  "source-mismatch": "sourceMismatch",
  "source-identity-unavailable": "sourceIdentityUnavailable",
  "parameter-instance-reference": "parameterInstanceReference",
  "candidate-binding-unavailable": "candidateBindingUnavailable",
  "no-derived-output-selected": "noDerivedOutputSelected",
};

// Action ids arrive from the capture IPC as open strings, so a Map keeps lookups off the prototype chain.
const labelKeyByActionId = new Map([
  ["draft-document", "draftDocument"],
  ["draft-script", "draftScript"],
  ["summarize-meeting", "summarizeMeeting"],
  ["draft-social-post", "draftSocialPost"],
  ["generate-visualization", "generateVisualization"],
  ["generate-image", "generateImage"],
  ["generate-video", "generateVideo"],
  ["add-video-subtitles", "addVideoSubtitles"],
  ["render-presentation", "renderPresentation"],
  ["convert-document-to-docx", "convertDocx"],
  ["convert-document-to-pdf", "convertPdf"],
  ["convert-document-to-txt", "convertTxt"],
  ["convert-document-to-html", "convertHtml"],
]);

export function captureActionLabel(actionId: string) {
  const translate = i18n.global.t;

  const labelKey = labelKeyByActionId.get(actionId);
  return labelKey ? translate(`spaces.creation.workflow.captureActions.${labelKey}`) : actionId;
}
