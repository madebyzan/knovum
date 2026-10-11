import { i18n } from "../../../../foundation/i18n/instance";
import type { DocumentFormat } from "../catalog/creation-intent";
import { startDerivationAction } from "../start-action";
import type { DocumentConversionSource } from "./conversion-source";
const RUN_TITLE_KEY_BY_FORMAT: Record<DocumentFormat, string> = {
  docx: "spaces.creation.document.conversion.tasks.docx",
  pdf: "spaces.creation.document.conversion.tasks.pdf",
  txt: "spaces.creation.document.conversion.tasks.txt",
  html: "spaces.creation.document.conversion.tasks.html",
  pptx: "spaces.creation.presentation.create",
};

interface DocumentConversionOptions {
  source: DocumentConversionSource;
  format: DocumentFormat;
}

export function convertDocument({ source, format }: DocumentConversionOptions) {
  return startDerivationAction({
    sources: [
      {
        nodeId: source.id,
        title: source.data.title,
        role: "primary",
        sourceRevision: source.data.knowledgeRevision,
      },
    ],
    actionId: format === "pptx" ? "render-presentation" : `convert-document-to-${format}`,
    parameters: {},
    title: i18n.global.t(RUN_TITLE_KEY_BY_FORMAT[format]),
  });
}
