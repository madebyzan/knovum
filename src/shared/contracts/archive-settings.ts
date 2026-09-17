import type { InjectionKey, Ref } from "vue";

export interface ArchivedSpaceSummary {
  id: string;
  title: string;
}

export interface ArchiveSettingsContract {
  spaces: Readonly<Ref<readonly ArchivedSpaceSummary[]>>;
  initialize(inboxTitle: string): Promise<void>;
  restore(id: string): Promise<void>;
  permanentlyDelete(id: string): Promise<void>;
  permanentlyDeleteAll(): Promise<void>;
}

export const archiveSettingsKey: InjectionKey<ArchiveSettingsContract> = Symbol("archive-settings");
