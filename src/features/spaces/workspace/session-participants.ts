export interface SpaceSessionParticipant {
  commit(): Promise<void>;
  finalize?(): void;
  phase?: "content" | "layout" | "persistence";
  resume?(): void;
  suspend?(): void;
}

const activeParticipants = new Set<SpaceSessionParticipant>();

export function registerSpaceSessionParticipant(participant: SpaceSessionParticipant) {
  activeParticipants.add(participant);
  return () => {
    activeParticipants.delete(participant);
  };
}

export function snapshotSpaceSessionParticipants(): readonly SpaceSessionParticipant[] {
  return [...activeParticipants];
}

export function flushSpaceSessionContent() {
  return commitParticipants(participantsInPhase(snapshotSpaceSessionParticipants(), "content"));
}

export async function flushSpaceSessionCommits(afterContent?: () => Promise<void>) {
  const participants = snapshotSpaceSessionParticipants();
  await prepareParticipants(participantsInPhase(participants, "content"));
  await afterContent?.();
  await prepareParticipants(participantsInPhase(participants, "layout"));
  await prepareParticipants(participantsInPhase(participants, "persistence"));
  finalizeParticipants(participants);
}

async function commitParticipants(participants: readonly SpaceSessionParticipant[]) {
  await prepareParticipants(participants);
  finalizeParticipants(participants);
}

async function prepareParticipants(participants: readonly SpaceSessionParticipant[]) {
  for (const participant of participants) {
    // oxlint-disable-next-line eslint/no-await-in-loop -- Leave decisions must be presented one at a time.
    await participant.commit();
  }
}

function finalizeParticipants(participants: readonly SpaceSessionParticipant[]) {
  for (const participant of participants) participant.finalize?.();
}

function participantsInPhase(
  participants: readonly SpaceSessionParticipant[],
  phase: NonNullable<SpaceSessionParticipant["phase"]>,
) {
  return participants.filter(({ phase: owner }) => owner === phase || (phase === "content" && !owner));
}
