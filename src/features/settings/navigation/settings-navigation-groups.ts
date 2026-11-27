import { compact } from "es-toolkit";
import { search } from "fast-fuzzy";
import type { Component } from "vue";

import { fuzzySearchThreshold } from "../../../shared/contracts/fuzzy-search-threshold";
import type { SettingsPaneId } from "./settings-pane";
import { SETTINGS_REGISTRY, TITLE_KEY_BY_SETTINGS_GROUP } from "./settings-registry";

type SettingsLocation = NonNullable<(typeof SETTINGS_REGISTRY)[number]["locations"]>[number];

interface SettingsNavItem {
  readonly id: SettingsPaneId;
  readonly icon: Component;
  readonly advanced: boolean;
  readonly locations?: readonly SettingsLocation[] | undefined;
  readonly label: string;
  readonly description: string;
  readonly location: string | undefined;
  readonly searchText: string;
}

export interface SettingsNavGroup {
  readonly id: string;
  readonly label: string;
  readonly settings: readonly SettingsNavItem[];
}

export function buildSettingsNavGroups(translate: (key: string) => string): SettingsNavGroup[] {
  return Object.entries(TITLE_KEY_BY_SETTINGS_GROUP).map(([group, titleKey]) => ({
    id: group,
    label: translate(titleKey),
    settings: SETTINGS_REGISTRY.filter((definition) => definition.group === group).map((definition) => ({
      id: definition.id,
      icon: definition.icon,
      advanced: Boolean(definition.advanced),
      locations: definition.locations,
      label: translate(definition.titleKey),
      description: translate(definition.descriptionKey),
      location: undefined,
      searchText: [definition.titleKey, ...definition.searchKeys].map((key) => translate(key)).join(" "),
    })),
  }));
}

export function searchSettingsNavGroups(
  groups: readonly SettingsNavGroup[],
  rawQuery: string,
  translate: (key: string) => string,
): SettingsNavGroup[] {
  const value = rawQuery.trim();
  if (!value) return [...groups];
  return compact(
    groups.map((group) => {
      const candidates = group.settings.flatMap((setting) =>
        setting.locations?.length
          ? setting.locations.map((location) => ({
              ...setting,
              location: location.id,
              label: translate(location.labelKey),
              description: translate(location.descriptionKey),
              searchText: `${translate(location.labelKey)} ${translate(location.descriptionKey)} ${setting.searchText}`,
            }))
          : [setting],
      );
      const matched = search(value, candidates, {
        keySelector: (setting) => setting.searchText,
        threshold: fuzzySearchThreshold(value, 0.7),
      });
      return matched.length > 0 ? { id: group.id, label: group.label, settings: matched } : null;
    }),
  );
}
