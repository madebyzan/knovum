// Generated from Rust contracts. Do not edit.
export const maxKnowledgePassageBytes = 1900 as const;
export const relationKinds = ["reference","related","supports","contradicts","depends-on","supersedes","answers","uses","derives-from","caused-by"] as const;
export const relationStatuses = ["active","contested","invalidated"] as const;
export const visualKnowledgeLimits = {"description":8000,"detectedText":16000,"tags":32,"tag":80} as const;
export const spaceActionImpacts = ["read_only","creates_resource","mutates_resource"] as const;
export const spaceActionExecutionModes = ["immediate","durable"] as const;
export const spaceActionInputRoles = ["image","document","knowledge","transcript","video"] as const;
export const spaceActionOutputContracts = ["answer","document","file","image","knowledge_change_receipt","knowledge_node","presentation","resource_receipt","search_results","space_edit_receipt","space_contents","video"] as const;
export const spaceActionOutputArities = ["single","discovered"] as const;
