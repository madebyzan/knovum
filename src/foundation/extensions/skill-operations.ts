import type { AgentSkillFile } from "../../shared/contracts/generated/AgentSkillFile";
import type { ExtensionChange } from "../../shared/contracts/generated/ExtensionChange";
import type { SkillContent } from "../../shared/contracts/generated/SkillContent";
import type { SkillRevision } from "../../shared/contracts/generated/SkillRevision";
import { invokeCommand } from "../ipc/invoke-command";

export const listSkills = () => invokeCommand<SkillRevision[]>("plugin:extensions|list_skills");
export const readSkillFile = (id: string, path: string) =>
  invokeCommand<AgentSkillFile>("plugin:extensions|read_skill_file", { id, path });
export const readSkill = (id: string) => invokeCommand<SkillContent>("plugin:extensions|read_skill", { id });
export const createSkill = (name: string, markdown: string) =>
  invokeCommand<ExtensionChange<SkillRevision>>("plugin:extensions|create_skill", { name, markdown });
export const saveSkillRevision = (id: string, expectedDigest: string, markdown: string) =>
  invokeCommand<ExtensionChange<SkillRevision>>("plugin:extensions|save_skill_revision", {
    id,
    expectedDigest,
    markdown,
  });
export const importSkill = (path: string) =>
  invokeCommand<ExtensionChange<SkillRevision>>("plugin:extensions|import_skill", { path });
export const copySkill = (id: string) =>
  invokeCommand<ExtensionChange<SkillRevision>>("plugin:extensions|copy_skill", { id });
export const exportSkill = (id: string, path: string) =>
  invokeCommand<void>("plugin:extensions|export_skill", { id, path });
export const setSkillEnabled = (id: string, enabled: boolean) =>
  invokeCommand<void>("plugin:extensions|set_skill_enabled", { id, enabled });
export const deleteSkill = (id: string) =>
  invokeCommand<ExtensionChange<null>>("plugin:extensions|delete_skill", { id });
