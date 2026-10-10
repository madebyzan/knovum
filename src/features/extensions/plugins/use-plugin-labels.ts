import { useI18n } from "vue-i18n";

import type { PluginSource } from "../../../shared/contracts/generated/PluginSource";

export function usePluginLabels() {
  const { t } = useI18n();
  function describePlugin(source: PluginSource, name: string, description?: string | null) {
    const key = `extensions.bundled.${name}`;
    if (source.type === "bundled") {
      return { name: t(`${key}.name`), description: t(`${key}.description`) };
    }
    return { name, description };
  }
  return { describePlugin };
}
