import type { AgentSkillFile } from "../../shared/contracts/generated/AgentSkillFile";
import type { ExtensionChange } from "../../shared/contracts/generated/ExtensionChange";
import type { SkillContent } from "../../shared/contracts/generated/SkillContent";
import type { SkillRevision } from "../../shared/contracts/generated/SkillRevision";
import { invokeCommand } from "../ipc/invoke-command";

export const listSkills = () => invokeCommand<SkillRevision[]>("plugin:extensions|list_skills");
export const readSkillFile = (skillId: string, path: string) =>
  invokeCommand<AgentSkillFile>("plugin:extensions|read_skill_file", { id: skillId, path });
export const readSkill = (skillId: string) =>
  invokeCommand<SkillContent>("plugin:extensions|read_skill", { id: skillId });
export const createSkill = (name: string, markdown: string) =>
  invokeCommand<ExtensionChange<SkillRevision>>("plugin:extensions|create_skill", { name, markdown });
export const saveSkillRevision = (skillId: string, expectedDigest: string, markdown: string) =>
  invokeCommand<ExtensionChange<SkillRevision>>("plugin:extensions|save_skill_revision", {
    id: skillId,
    expectedDigest,
    markdown,
  });
export const importSkill = (path: string) =>
  invokeCommand<ExtensionChange<SkillRevision>>("plugin:extensions|import_skill", { path });
export const copySkill = (skillId: string) =>
  invokeCommand<ExtensionChange<SkillRevision>>("plugin:extensions|copy_skill", { id: skillId });
export const exportSkill = (skillId: string, path: string) =>
  invokeCommand<void>("plugin:extensions|export_skill", { id: skillId, path });
export const setSkillEnabled = (skillId: string, enabled: boolean) =>
  invokeCommand<void>("plugin:extensions|set_skill_enabled", { id: skillId, enabled });
export const deleteSkill = (skillId: string) =>
  invokeCommand<ExtensionChange<null>>("plugin:extensions|delete_skill", { id: skillId });
