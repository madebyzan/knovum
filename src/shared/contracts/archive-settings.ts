import type { InjectionKey, Ref } from "vue";

export interface ArchivedSpace {
  id: string;
  title: string;
}

export interface ArchiveSettingsContract {
  spaces: Readonly<Ref<readonly ArchivedSpace[]>>;
  initialize(inboxTitle: string): Promise<void>;
  restore(id: string): Promise<void>;
  permanentlyDelete(id: string): Promise<void>;
  permanentlyDeleteAll(): Promise<void>;
}

export const archiveSettingsKey: InjectionKey<ArchiveSettingsContract> = Symbol("archive-settings");
