import { Layer } from "konva/lib/Layer";
import { getCurrentInstance, resolveComponent } from "vue";
import { install } from "vue-konva/core";

/** A layer has one scheduler: Konva while editing, the camera while navigating. */
export class CanvasRenderLayer extends Layer {
  private cameraOwned = false;

  setCameraOwned(value: boolean) {
    this.cameraOwned = value;
  }

  override batchDraw() {
    // Transformer explicitly calls batchDraw even when Konva.autoDrawEnabled is false.
    // During camera projection the owner draws this layer synchronously, including the transformer.
    return this.cameraOwned ? this : super.batchDraw();
  }
}

/** Register through vue-konva's public custom-node API, without replacing vendor prototypes. */
export function useCanvasRenderLayer() {
  const app = getCurrentInstance()!.appContext.app;
  if (!app.component("CanvasLayer")) {
    install(app, { prefix: "Canvas", customNodes: { Layer: CanvasRenderLayer } });
  }
  return resolveComponent("CanvasLayer");
}
