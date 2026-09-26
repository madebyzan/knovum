import { open } from "@tauri-apps/plugin-dialog";
import { union, without } from "es-toolkit";
import { basename } from "pathe";
import { computed, shallowRef } from "vue";

export interface AgentAttachment {
  name: string;
  path: string;
}

export function useAgentAttachments() {
  const paths = shallowRef<string[]>([]);
  const attachments = computed<AgentAttachment[]>(() =>
    paths.value.map((path) => ({ path, name: basename(path) })),
  );

  async function choose() {
    const selected = await open({ directory: false, multiple: true });
    if (!selected) return;
    const additions = Array.isArray(selected) ? selected : [selected];
    paths.value = union(paths.value, additions);
  }

  function remove(path: string) {
    paths.value = without(paths.value, path);
  }

  function clear() {
    paths.value = [];
  }

  return { paths, attachments, choose, remove, clear };
}
