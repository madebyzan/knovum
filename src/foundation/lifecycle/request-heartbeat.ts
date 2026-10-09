const HEARTBEAT_INTERVAL_MS = 2_000;

export function startRequestHeartbeat(acknowledge: () => Promise<boolean>) {
  let timer: ReturnType<typeof setTimeout> | undefined;
  let stopped = false;
  const schedule = () => {
    timer = setTimeout(async () => {
      if (stopped) return;
      const acknowledged = await acknowledge();
      if (stopped || !acknowledged) return;
      schedule();
    }, HEARTBEAT_INTERVAL_MS);
  };
  schedule();
  return () => {
    stopped = true;
    if (timer) clearTimeout(timer);
  };
}
