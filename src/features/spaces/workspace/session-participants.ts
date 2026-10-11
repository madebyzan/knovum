export interface SpaceSessionParticipant {
  commit(): Promise<void>;
  finalize?(): void;
  phase?: "content" | "layout" | "persistence";
  resume?(): void;
  suspend?(): void;
  editorOwner?: { spaceId: string; tabId: string; commitOnSwitch: boolean };
}

export interface SessionCommitScope {
  purpose: "switch" | "close";
  spaceIds?: readonly string[];
  tabIds?: readonly string[];
}

const participants = new Set<SpaceSessionParticipant>();

export function registerSpaceSessionParticipant(participant: SpaceSessionParticipant) {
  participants.add(participant);
  return () => {
    participants.delete(participant);
  };
}

export function snapshotSpaceSessionParticipants(): readonly SpaceSessionParticipant[] {
  return [...participants];
}

export async function flushSpaceSessionContent(scope?: SessionCommitScope) {
  const selected = participantsInPhase(snapshotSpaceSessionParticipants(), "content").filter(
    (participant) => !scope || includesParticipant(participant, scope),
  );
  await prepareParticipants(selected);
  finalizeParticipants(selected);
}

export async function prepareEditorSessionClose(scope: Omit<SessionCommitScope, "purpose">) {
  const selected = snapshotSpaceSessionParticipants().filter(
    (participant) =>
      participant.editorOwner && includesParticipant(participant, { ...scope, purpose: "close" }),
  );
  await prepareParticipants(selected);
  return () => finalizeParticipants(selected);
}

export async function flushSpaceSessionCommits(
  afterContent?: () => Promise<void>,
  scope?: SessionCommitScope,
) {
  const selected = snapshotSpaceSessionParticipants().filter(
    (participant) => !scope || includesParticipant(participant, scope),
  );
  await prepareParticipants(participantsInPhase(selected, "content"));
  await afterContent?.();
  await prepareParticipants(participantsInPhase(selected, "layout"));
  await prepareParticipants(participantsInPhase(selected, "persistence"));
  finalizeParticipants(selected);
}

function includesParticipant(participant: SpaceSessionParticipant, scope: SessionCommitScope) {
  const owner = participant.editorOwner;
  if (!owner) return !scope.tabIds;
  if (scope.spaceIds && !scope.spaceIds.includes(owner.spaceId)) return false;
  if (scope.tabIds && !scope.tabIds.includes(owner.tabId)) return false;
  return scope.purpose === "close" || owner.commitOnSwitch;
}

async function prepareParticipants(selected: readonly SpaceSessionParticipant[]) {
  for (const participant of selected) {
    // oxlint-disable-next-line eslint/no-await-in-loop -- Save decisions must be presented in a stable order.
    await participant.commit();
  }
}

function finalizeParticipants(selected: readonly SpaceSessionParticipant[]) {
  for (const participant of selected) participant.finalize?.();
}

function participantsInPhase(
  selected: readonly SpaceSessionParticipant[],
  phase: NonNullable<SpaceSessionParticipant["phase"]>,
) {
  return selected.filter(({ phase: owner }) => owner === phase || (phase === "content" && !owner));
}
