import { open } from "@tauri-apps/plugin-dialog";

import { importSkill, exportSkill } from "../../../foundation/extensions/skill-operations";

export async function importSkillDirectory() {
  const path = await open({ directory: true, multiple: false });
  if (typeof path === "string") return importSkill(path);
}

export async function exportSkillDirectory(skillId: string) {
  const path = await open({ directory: true, multiple: false });
  if (typeof path === "string") await exportSkill(skillId, path);
}
