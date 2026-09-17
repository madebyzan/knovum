declare global {
  interface Window {
    __knovumE2eBridge?: {
      runCommand(id: string): Promise<void>;
    };
  }
}

// oxlint-disable-next-line unicorn/require-module-specifiers -- mark file as module for declare global
export {};
