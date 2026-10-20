import type { AgentSkillFile } from "../../shared/contracts/generated/AgentSkillFile";
import type { ExtensionChange } from "../../shared/contracts/generated/ExtensionChange";
import type { SkillContent } from "../../shared/contracts/generated/SkillContent";
import type { SkillRevision } from "../../shared/contracts/generated/SkillRevision";
import { invokeCommand } from "../ipc/invoke-command";

export function listSkills() {
  return invokeCommand<SkillRevision[]>("plugin:extensions|list_skills");
}
export function readSkillFile(skillId: string, path: string) {
  return invokeCommand<AgentSkillFile>("plugin:extensions|read_skill_file", { skillId, path });
}
export function readSkill(skillId: string) {
  return invokeCommand<SkillContent>("plugin:extensions|read_skill", { skillId });
}
export function createSkill(name: string, markdown: string) {
  return invokeCommand<ExtensionChange<SkillRevision>>("plugin:extensions|create_skill", { name, markdown });
}
export function saveSkillRevision(skillId: string, expectedDigest: string, markdown: string) {
  return invokeCommand<ExtensionChange<SkillRevision>>("plugin:extensions|save_skill_revision", {
    skillId,
    expectedDigest,
    markdown,
  });
}
export function importSkill(path: string) {
  return invokeCommand<ExtensionChange<SkillRevision>>("plugin:extensions|import_skill", { path });
}
export function duplicateSkill(skillId: string) {
  return invokeCommand<ExtensionChange<SkillRevision>>("plugin:extensions|duplicate_skill", { skillId });
}
export function exportSkill(skillId: string, path: string) {
  return invokeCommand<void>("plugin:extensions|export_skill", { skillId, path });
}
export function enableSkill(skillId: string) {
  return invokeCommand<void>("plugin:extensions|enable_skill", { skillId });
}
export function disableSkill(skillId: string) {
  return invokeCommand<void>("plugin:extensions|disable_skill", { skillId });
}
export function deleteSkill(skillId: string) {
  return invokeCommand<ExtensionChange<null>>("plugin:extensions|delete_skill", { skillId });
}
