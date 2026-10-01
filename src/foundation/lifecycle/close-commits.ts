type CloseCommit = () => Promise<void>;

/** Close and update-install share the same save barrier, not separate save implementations. */
export class CloseCommitCoordinator {
  private readonly commits = new Set<CloseCommit>();
  private flushing: Promise<void> | undefined;
  restarting = false;

  register(commit: CloseCommit) {
    this.commits.add(commit);
    return () => this.commits.delete(commit);
  }

  flush(): Promise<void> {
    this.flushing ??= this.flushRegistered().finally(() => {
      this.flushing = undefined;
    });
    return this.flushing;
  }

  async restart(install: () => Promise<void>): Promise<void> {
    if (this.restarting) throw new Error("Application restart is already in progress");
    this.restarting = true;
    try {
      await this.flush();
      await install();
    } finally {
      this.restarting = false;
    }
  }

  private async flushRegistered(): Promise<void> {
    const outcomes = await Promise.allSettled(
      [...this.commits].map((commit) => Promise.resolve().then(commit)),
    );
    const failure = outcomes.find((outcome) => outcome.status === "rejected");
    if (failure?.status === "rejected") throw failure.reason;
  }
}
