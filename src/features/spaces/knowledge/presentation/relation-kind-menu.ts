import { i18n } from "../../../../foundation/i18n/instance";
import type { RelationKind } from "../../contracts/generated/RelationKind";
import { popupSpaceContextMenu, spaceCheckItem } from "../../workspace/space-menu";

const RELATION_KINDS: readonly RelationKind[] = [
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

export async function showSpaceRelationKindMenu(
  current: RelationKind,
  update: (kind: RelationKind) => void | Promise<void>,
) {
  const t = i18n.global.t;
  await popupSpaceContextMenu(
    RELATION_KINDS.map((kind) =>
      spaceCheckItem(t(`spaces.knowledge.relations.kinds.${kind}`), current === kind, () => update(kind)),
    ),
  );
}
