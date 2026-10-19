import { Layer } from "konva/lib/Layer";
import type { Component } from "vue";
import { install, type KonvaNodeConstructor, type Layer as KonvaLayer } from "vue-konva/core";

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

function createKonvaNodeComponent<T extends Component>(
  name: string,
  nodeConstructor: KonvaNodeConstructor,
): T {
  let nodeComponent: T | undefined;
  install(
    {
      component(registeredName: string, component: Component) {
        if (registeredName === `V${name}`) {
          nodeComponent = component as T;
        }
      },
    },
    { customNodes: { [name]: nodeConstructor } },
  );
  if (!nodeComponent) {
    throw new Error(`Failed to create vue-konva component for ${name}`);
  }
  return nodeComponent;
}

/** Vue Konva component backed by `CanvasRenderLayer`. */
export const KonvaCanvasLayer = createKonvaNodeComponent<typeof KonvaLayer>("Layer", CanvasRenderLayer);
