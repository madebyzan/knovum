import { documentCoverageFieldsSchema } from "./document-coverage-fields";
import { validateDocumentCoverage } from "./document-coverage-validation";

export const documentCoverageSchema = documentCoverageFieldsSchema.superRefine(validateDocumentCoverage);
