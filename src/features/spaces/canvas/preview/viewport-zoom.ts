import type { InjectionKey, Ref } from "vue";

export const canvasPreviewZoomKey: InjectionKey<Readonly<Ref<number>>> = Symbol("canvas-preview-zoom");
