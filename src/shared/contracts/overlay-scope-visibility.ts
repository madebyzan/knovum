import type { InjectionKey, Ref } from "vue";

export const overlayScopeVisibleKey: InjectionKey<Readonly<Ref<boolean>>> =
  Symbol("overlay-scope-visible");
