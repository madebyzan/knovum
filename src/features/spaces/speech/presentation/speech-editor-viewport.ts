import type { InjectionKey, Ref } from "vue";

/** The project editor owns one viewport; long scripts virtualize within it. */
export const SPEECH_EDITOR_VIEWPORT_KEY: InjectionKey<Readonly<Ref<HTMLElement | null>>> =
  Symbol("speech-editor-viewport");
