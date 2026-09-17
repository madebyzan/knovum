import { open } from "@tauri-apps/plugin-dialog";
import { computed, shallowRef, watch, type Ref } from "vue";

export interface AgentAttachment {
  name: string;
  path: string;
}

export function useAgentAttachments(openState: Ref<boolean>) {
  const paths = shallowRef<string[]>([]);
  const attachments = computed<AgentAttachment[]>(() =>
    paths.value.map((path) => ({ path, name: path.split(/[\\/]/).at(-1) ?? path })),
  );

  async function choose() {
    const selected = await open({ directory: false, multiple: true });
    if (!selected) return;
    const additions = Array.isArray(selected) ? selected : [selected];
    paths.value = [...new Set([...paths.value, ...additions])];
  }

  function remove(path: string) {
    paths.value = paths.value.filter((candidate) => candidate !== path);
  }

  watch(openState, (isOpen) => {
    if (!isOpen) paths.value = [];
  });

  return { paths, attachments, choose, remove };
}
