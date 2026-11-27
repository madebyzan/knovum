import {
  parseImageCardCaption,
  resolveImageGroupColumns,
  type ImageGroupLayout,
  type LightboxImageItem,
} from "./image-group-layout";

function escapeHtml(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll('"', "&quot;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;");
}

function renderCompareHtml(before: LightboxImageItem, after: LightboxImageItem): string {
  const beforeLabel = escapeHtml(before.caption.trim() || "Before");
  const afterLabel = escapeHtml(after.caption.trim() || "After");
  return (
    `<div class="ui-image-compare" data-image-compare="true" tabindex="0" role="slider" aria-valuemin="2" aria-valuemax="98" aria-valuenow="50">` +
    `<img src="${escapeHtml(after.src)}" alt="${escapeHtml(after.caption)}" class="ui-image-compare-base" draggable="false" />` +
    `<div class="ui-image-compare-overlay"><img src="${escapeHtml(before.src)}" alt="${escapeHtml(before.caption)}" draggable="false" /></div>` +
    `<div class="ui-image-compare-divider"></div>` +
    `<div class="ui-image-compare-knob" aria-hidden="true">&#8596;</div>` +
    `<span class="ui-image-compare-label" data-side="before">${beforeLabel}</span>` +
    `<span class="ui-image-compare-label" data-side="after">${afterLabel}</span>` +
    `</div>`
  );
}

function renderCardBodyHtml(caption: string): string {
  const { title, description } = parseImageCardCaption(caption);
  if (!title && !description) return "";
  const titleHtml = title ? `<div class="ui-image-group-card-title">${escapeHtml(title)}</div>` : "";
  const descHtml = description
    ? `<div class="ui-image-group-card-desc">${escapeHtml(description)}</div>`
    : "";
  return `<figcaption class="ui-image-group-card-body">${titleHtml}${descHtml}</figcaption>`;
}

function renderItemHtml(item: LightboxImageItem, layout: ImageGroupLayout, index: number): string {
  const media = `<div class="ui-image-group-media" data-gallery-index="${index}"><img src="${escapeHtml(item.src)}" alt="${escapeHtml(item.caption)}" loading="lazy" /></div>`;
  if (layout === "cards") {
    return `<figure class="ui-image-group-item">${media}${renderCardBodyHtml(item.caption)}</figure>`;
  }
  const trimmed = item.caption.trim();
  const caption = trimmed
    ? `<figcaption class="ui-image-group-caption">${escapeHtml(trimmed)}</figcaption>`
    : "";
  return `<figure class="ui-image-group-item">${media}${caption}</figure>`;
}

export function renderImageGroupHtml(layout: ImageGroupLayout, images: readonly LightboxImageItem[]): string {
  const cols = resolveImageGroupColumns(layout, images.length);
  const useCompare = layout === "compare" && images.length >= 2;
  const compareHtml = useCompare ? renderCompareHtml(images[0]!, images[1]!) : "";
  const rest = useCompare ? images.slice(2) : images;
  const offset = useCompare ? 2 : 0;
  const gridStyle = useCompare ? ' style="margin-top:0.75rem"' : "";
  const gridHtml = rest.length
    ? `<div class="ui-image-group-grid"${gridStyle}>${rest.map((item, idx) => renderItemHtml(item, layout, idx + offset)).join("")}</div>`
    : "";
  return `<div class="ui-image-group" data-type="image-group" data-layout="${layout}" data-count="${images.length}" style="--image-group-cols:${cols}">${compareHtml}${gridHtml}</div>\n`;
}
