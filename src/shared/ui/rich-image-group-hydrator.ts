import { renderImageGroupHtml } from "./image-group-html";
import { normalizeImageGroupLayout, type LightboxImageItem } from "./image-group-layout";

function bindCompareSlider(stage: HTMLElement) {
  if (stage.dataset.compareBound === "true") return;
  stage.dataset.compareBound = "true";
  let split = 50;
  let pointerId: number | null = null;
  let stageRect: { left: number; width: number } | null = null;
  let latestX = 0;
  let rafId: number | null = null;

  const applySplit = (next: number) => {
    split = Math.max(2, Math.min(98, next));
    stage.style.setProperty("--compare-split", `${split.toFixed(2)}%`);
    stage.setAttribute("aria-valuenow", String(Math.round(split)));
  };

  const paint = () => {
    rafId = null;
    if (!stageRect || stageRect.width <= 0) return;
    applySplit(((latestX - stageRect.left) / stageRect.width) * 100);
  };

  const end = (event?: PointerEvent) => {
    if (event && pointerId !== null && event.pointerId !== pointerId) return;
    if (rafId !== null) cancelAnimationFrame(rafId);
    rafId = null;
    if (pointerId !== null && stage.hasPointerCapture?.(pointerId)) stage.releasePointerCapture(pointerId);
    pointerId = null;
    stageRect = null;
  };

  stage.addEventListener("pointerdown", (event) => {
    if (event.button !== 0) return;
    event.preventDefault();
    const rect = stage.getBoundingClientRect();
    stageRect = { left: rect.left, width: rect.width };
    pointerId = event.pointerId;
    latestX = event.clientX;
    stage.setPointerCapture?.(event.pointerId);
    paint();
  });
  stage.addEventListener("pointermove", (event) => {
    if (pointerId === null || event.pointerId !== pointerId) return;
    latestX = event.clientX;
    if (rafId === null) rafId = requestAnimationFrame(paint);
  });
  stage.addEventListener("pointerup", end);
  stage.addEventListener("pointercancel", end);
  stage.addEventListener("dblclick", () => applySplit(50));
  stage.addEventListener("keydown", (event) => {
    if (event.key === "ArrowLeft") {
      event.preventDefault();
      applySplit(split - 5);
    } else if (event.key === "ArrowRight") {
      event.preventDefault();
      applySplit(split + 5);
    }
  });
}

function upgradeStaticNoteImageGroups(container: HTMLElement) {
  for (const group of container.querySelectorAll<HTMLElement>('[data-type="image-group"]')) {
    if (group.dataset.imageGroupUpgraded === "true") continue;
    const grid = group.querySelector<HTMLElement>(".ui-image-group-grid");
    const rawImages = grid
      ? [...grid.children].filter((el): el is HTMLImageElement => el.tagName === "IMG")
      : [];
    if (!rawImages.length) {
      group.dataset.imageGroupUpgraded = "true";
      continue;
    }
    const items: LightboxImageItem[] = rawImages.map((img) => ({
      src: img.getAttribute("src") ?? "",
      caption: img.getAttribute("alt") ?? "",
    }));
    const layout = normalizeImageGroupLayout(group.dataset.layout);
    const tpl = document.createElement("template");
    tpl.innerHTML = renderImageGroupHtml(layout, items);
    const next = tpl.content.firstElementChild;
    if (next instanceof HTMLElement) {
      next.dataset.imageGroupUpgraded = "true";
      group.replaceWith(next);
    }
  }
}

/** Upgrades Note `imageGroup` DOM structure and binds interactive 120fps Compare sliders. */
export function hydrateContainerImageGroups(container: HTMLElement): void {
  upgradeStaticNoteImageGroups(container);
  for (const stage of container.querySelectorAll<HTMLElement>('[data-image-compare="true"]')) {
    bindCompareSlider(stage);
  }
}
