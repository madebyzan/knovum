export type ActionAvailability<Reason extends string = string> =
  | { readonly state: "ready" }
  | { readonly state: "checking" }
  | { readonly state: "busy" }
  | { readonly state: "blocked"; readonly reason: Reason }
  | { readonly state: "hidden" };

export function canRunAction(availability: ActionAvailability): boolean {
  return availability.state === "ready";
}

export function shouldShowAction(availability: ActionAvailability): boolean {
  return availability.state !== "hidden";
}

export function availableWhen<Reason extends string>(
  isAvailable: boolean,
  reason: Reason,
): ActionAvailability<Reason> {
  return isAvailable ? { state: "ready" } : { state: "blocked", reason };
}
