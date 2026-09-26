import { i18n } from "../../../../foundation/i18n/i18n";
import type { SpaceRelationKind } from "../../model/relations/relation";
import { popupSpaceContextMenu, spaceCheckItem, spaceMenuItem } from "../../workspace/space-menu";

const relationKinds: readonly SpaceRelationKind[] = [
  "reference",
  "related",
  "supports",
  "contradicts",
  "depends-on",
  "supersedes",
  "answers",
  "uses",
  "derives-from",
  "caused-by",
];

export async function showSpaceRelationMenu(
  current: SpaceRelationKind,
  update: (kind: SpaceRelationKind) => void | Promise<void>,
  remove: () => void | Promise<void>,
) {
  const t = i18n.global.t;
  await popupSpaceContextMenu([
    relationKinds.map((kind) =>
      spaceCheckItem(t(`spaces.knowledge.relations.kinds.${kind}`), current === kind, () => update(kind)),
    ),
    [spaceMenuItem(t("spaces.knowledge.relations.remove"), remove)],
  ]);
}
