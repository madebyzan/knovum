import type { StructureScene } from "../canvas/document/structure-scene";
import type { SpaceRelation } from "../model/relations/relation";
import type { SpaceEditSnapshot } from "../model/structure/schema/edit-snapshot";

export function assertJsonCanvasExportable(snapshot: SpaceEditSnapshot, relations: readonly SpaceRelation[]) {
  if (snapshot.structure.some(({ kind }) => kind === "group"))
    throw new Error("JSON Canvas cannot represent Knovum Groups without flattening");
  if (snapshot.structure.some(({ locked }) => locked))
    throw new Error("JSON Canvas cannot preserve locked structure nodes");
  if (snapshot.relationStyles.length) throw new Error("JSON Canvas cannot preserve relation style overrides");
  for (const layout of snapshot.layouts) assertLayoutExportable(layout);
  if (relations.some(({ status, kind }) => status !== "active" || kind !== "related"))
    throw new Error("JSON Canvas cannot preserve non-active or typed relation semantics");
}

function assertLayoutExportable(layout: SpaceEditSnapshot["layouts"][number]) {
  if (layout.kind === "section") {
    if (layout.collapsed) throw new Error("JSON Canvas cannot preserve collapsed Sections");
    if (layout.appearance?.borderStyle) throw new Error("JSON Canvas cannot preserve Section border styles");
    return;
  }
  if (layout.kind === "group" || layout.transform.rotation !== 0 || layout.transform.scale !== 1)
    throw new Error("JSON Canvas cannot preserve rotated or scaled Knovum structure");
  const { fill: _fill, customFill: _customFill, ...unsupported } = layout.appearance ?? {};
  if (Object.keys(unsupported).length) throw new Error("JSON Canvas cannot preserve this content appearance");
}

export function assertSectionMembershipIsGeometric(
  contents: StructureScene["content"],
  sections: StructureScene["sections"],
) {
  for (const content of contents) {
    const containingSections = sections.filter(({ bounds }) => contains(bounds, content.bounds));
    if (containingSections.length > 1)
      throw new Error("JSON Canvas cannot preserve content contained by overlapping Sections");
    const inferred = containingSections[0]?.section.id;
    if ((inferred ?? null) !== content.sectionId)
      throw new Error("JSON Canvas geometry cannot preserve Section membership without loss");
  }
}

type Bounds = { left: number; top: number; right: number; bottom: number };

function contains(parent: Bounds, child: Bounds) {
  return (
    child.left >= parent.left &&
    child.top >= parent.top &&
    child.right <= parent.right &&
    child.bottom <= parent.bottom
  );
}
