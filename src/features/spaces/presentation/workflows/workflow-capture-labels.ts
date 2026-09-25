import type { WorkflowCaptureIssueReason } from "../../workflows/captured-workflow-ipc";

export const captureIssueLabels: Record<WorkflowCaptureIssueReason, string> = {
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

export const captureActionLabels: Record<string, string> = {
  "draft-document": "draftDocument",
  "draft-script": "draftScript",
  "summarize-meeting": "summarizeMeeting",
  "draft-social-post": "draftSocialPost",
  "generate-visualization": "generateVisualization",
  "generate-image": "generateImage",
  "generate-video": "generateVideo",
  "extract-subtitles": "extractSubtitles",
  "render-subtitled-video": "renderSubtitledVideo",
  "render-presentation": "renderPresentation",
};
