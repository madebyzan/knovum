import { i18n } from "../../../../foundation/i18n/instance";
import type { DocumentFormat } from "../catalog/creation-intent";
import { startDerivationAction, type DerivationActionStore } from "../start-action";
import type { DocumentConversionSource } from "./conversion-source";
const RUN_TITLE_KEY_BY_FORMAT: Record<DocumentFormat, string> = {
  docx: "spaces.creation.document.conversion.docxRunTitle",
  pdf: "spaces.creation.document.conversion.pdfRunTitle",
  txt: "spaces.creation.document.conversion.txtRunTitle",
  html: "spaces.creation.document.conversion.htmlRunTitle",
  pptx: "spaces.creation.presentation.create",
};

interface DocumentConversionOptions {
  store: DerivationActionStore;
  source: DocumentConversionSource;
  format: DocumentFormat;
}

export function convertDocument({ store, source, format }: DocumentConversionOptions) {
  return startDerivationAction({
    store,
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
