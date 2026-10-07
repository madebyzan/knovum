import { extname } from "pathe";

export type SpaceFilePresentation =
  | "pdf"
  | "document"
  | "spreadsheet"
  | "tabular-data"
  | "presentation"
  | "generic";

export function fileExtension(name: string) {
  return extname(name).replace(/^\./, "").toLowerCase();
}

export function classifyFilePresentation(name: string): SpaceFilePresentation {
  const extension = fileExtension(name);
  if (extension === "pdf") return "pdf";
  if (extension === "doc" || extension === "docx") return "document";
  if (extension === "xls" || extension === "xlsx") return "spreadsheet";
  if (extension === "csv" || extension === "tsv") return "tabular-data";
  if (extension === "ppt" || extension === "pptx") return "presentation";
  return "generic";
}

export function usesDocumentPreview(presentation: SpaceFilePresentation) {
  return (
    presentation === "pdf" ||
    presentation === "document" ||
    presentation === "spreadsheet" ||
    presentation === "presentation"
  );
}
