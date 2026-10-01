import type { CaptureIssueReason } from "../../contracts/generated/CaptureIssueReason";

export const captureIssueLabels: Record<CaptureIssueReason, string> = {
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
const captureActionLabelKeys = new Map([
  ["draft-document", "draftDocument"],
  ["draft-script", "draftScript"],
  ["summarize-meeting", "summarizeMeeting"],
  ["draft-social-post", "draftSocialPost"],
  ["generate-visualization", "generateVisualization"],
  ["generate-image", "generateImage"],
  ["generate-video", "generateVideo"],
  ["add-video-subtitles", "addVideoSubtitles"],
  ["render-presentation", "renderPresentation"],
  ["export-document-as-docx", "exportDocx"],
  ["export-document-as-pdf", "exportPdf"],
  ["export-document-as-txt", "exportTxt"],
]);

export function captureActionLabel(actionId: string, translate: (key: string) => string) {
  const labelKey = captureActionLabelKeys.get(actionId);
  return labelKey ? translate(`spaces.creation.workflow.captureActions.${labelKey}`) : actionId;
}
