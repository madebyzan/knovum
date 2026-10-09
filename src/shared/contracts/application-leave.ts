export class ApplicationLeaveCancelled extends Error {
  constructor() {
    super("Application leave cancelled");
    this.name = "ApplicationLeaveCancelled";
  }
}

export function isApplicationLeaveCancelled(error: unknown) {
  return error instanceof ApplicationLeaveCancelled;
}
