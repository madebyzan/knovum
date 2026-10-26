import { afterEach, expect, it } from "vitest";

import {
  CANVAS_TEXT_SURFACE_ATTRIBUTE,
  isWithinCanvasTextSurface,
} from "../presentation/nodes/canvas-text-surface";

afterEach(() => document.body.replaceChildren());

function surface(nodeId: string, triggerId: string) {
  const root = document.createElement("div");
  root.setAttribute(CANVAS_TEXT_SURFACE_ATTRIBUTE, nodeId);
  root.appendChild(document.createElement("button")).id = triggerId;
  return document.body.appendChild(root);
}

function menu(labelledBy: string) {
  const content = document.createElement("div");
  content.setAttribute("role", "menu");
  content.setAttribute("aria-labelledby", labelledBy);
  return document.body.appendChild(content).appendChild(document.createElement("div"));
}

it("counts a menu as part of the surface whose trigger labels it, and only that one", () => {
  const editing = surface("editing", "editing-trigger");
  surface("other", "other-trigger");

  expect(isWithinCanvasTextSurface("editing", editing.firstElementChild)).toBe(true);
  expect(isWithinCanvasTextSurface("editing", menu("editing-trigger"))).toBe(true);
  expect(isWithinCanvasTextSurface("editing", menu("other-trigger"))).toBe(false);
  expect(isWithinCanvasTextSurface("editing", menu("missing-trigger"))).toBe(false);
  expect(isWithinCanvasTextSurface("editing", document.body)).toBe(false);
  expect(isWithinCanvasTextSurface("editing", null)).toBe(false);
});
