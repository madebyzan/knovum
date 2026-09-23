import { invoke } from "@tauri-apps/api/core";

import type { AutomationOccurrence } from "../contracts/generated/AutomationOccurrence";
import type { AutomationTrigger } from "../contracts/generated/AutomationTrigger";
import type { SaveAutomationTriggerRequest } from "../contracts/generated/SaveAutomationTriggerRequest";

export type { AutomationOccurrence, AutomationTrigger };
export type SaveAutomationTriggerInput = SaveAutomationTriggerRequest;

export function listAutomationTriggers(spaceId: string): Promise<AutomationTrigger[]> {
  return invoke("plugin:spaces|list_automation_triggers", { spaceId });
}

export function saveAutomationTrigger(request: SaveAutomationTriggerInput): Promise<AutomationTrigger> {
  return invoke("plugin:spaces|save_automation_trigger", { request });
}

export function setAutomationTriggerEnabled(triggerId: string, enabled: boolean): Promise<void> {
  return invoke("plugin:spaces|set_automation_trigger_enabled", { triggerId, enabled });
}

export function renewAutomationTrigger(triggerId: string, validUntil: number): Promise<void> {
  return invoke("plugin:spaces|renew_automation_trigger", { triggerId, validUntil });
}

export function deleteAutomationTrigger(triggerId: string): Promise<void> {
  return invoke("plugin:spaces|delete_automation_trigger", { triggerId });
}

export function testAutomationTrigger(triggerId: string): Promise<string> {
  return invoke("plugin:spaces|test_automation_trigger", { triggerId });
}

export function listAutomationOccurrences(spaceId: string): Promise<AutomationOccurrence[]> {
  return invoke("plugin:spaces|list_automation_occurrences", { spaceId });
}

export function previewAutomationSchedule(schedule: string, timezone: string, count = 5): Promise<number[]> {
  return invoke("plugin:spaces|preview_automation_schedule", { schedule, timezone, count });
}
