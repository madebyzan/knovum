import type { AppCommand } from "../../shared/contracts/app-command";
import type { EditingCommand } from "../../shared/contracts/generated/EditingCommand";
import { isRichTextTarget, isTextFieldTarget } from "../dom/editable-target";
import { hasActionableTextSelection } from "../dom/text-selection";
import { appCommands, runAppCommand } from "./run-app-command";
import { performApplicationMenuEditing } from "./text-editing";

// An editing intent goes to the most specific thing the user is editing, the way AppKit walks its
// responder chain from the first responder outwards: the focused text, then the selected text, then
// the active surface. The macOS Application Menu and the keyboard elsewhere ask the same chain.

export interface EditorHistory {
  undo(): void;
  redo(): void;
}

export type EditingResponder =
  | { kind: "platform" }
  | { kind: "editor-history"; history: EditorHistory }
  | { kind: "command"; command: AppCommand }
  | { kind: "none" };

const PLATFORM: EditingResponder = { kind: "platform" };
const NONE: EditingResponder = { kind: "none" };
const historyByElement = new WeakMap<Element, EditorHistory>();

/** A rich-text editor declares the history behind its editable element, since the DOM cannot. */
export function registerEditorHistory(element: Element, history: EditorHistory): () => void {
  historyByElement.set(element, history);
  return () => {
    if (historyByElement.get(element) === history) historyByElement.delete(element);
  };
}

function editorHistoryAt(target: Element): EditorHistory | undefined {
  for (let element: Element | null = target; element; element = element.parentElement) {
    const history = historyByElement.get(element);
    if (history) return history;
  }
  return undefined;
}

/** The responder when the user is typing: focused text answers every intent it can perform. */
function focusedTextResponder(intent: EditingCommand, focused: Element | null): EditingResponder | undefined {
  if (isTextFieldTarget(focused)) return PLATFORM;
  if (!focused || !isRichTextTarget(focused)) return undefined;
  if (intent !== "undo" && intent !== "redo") return PLATFORM;
  const history = editorHistoryAt(focused);
  return history ? { kind: "editor-history", history } : PLATFORM;
}

export function resolveEditingResponder(
  intent: EditingCommand,
  commands: readonly AppCommand[],
  focused: Element | null = document.activeElement,
): EditingResponder {
  const typing = focusedTextResponder(intent, focused);
  if (typing) return typing;
  if ((intent === "copy" || intent === "cut") && hasActionableTextSelection(focused)) return PLATFORM;
  // Nothing on the page is being edited, so the active surface acts on its own content. Without
  // one the intent has no target: the page itself is never undone, pasted into or selected.
  const command = commands.find((candidate) => candidate.editingIntent === intent);
  return command ? { kind: "command", command } : NONE;
}

/** The intent as the macOS Application Menu delivers it: the WebView has not seen the key press. */
export function runEditingCommand(intent: EditingCommand): void | Promise<void> {
  const responder = resolveEditingResponder(intent, appCommands());
  if (responder.kind === "platform") return performApplicationMenuEditing(intent);
  if (responder.kind === "editor-history") return responder.history[intent === "undo" ? "undo" : "redo"]();
  if (responder.kind === "command") runAppCommand(responder.command);
}

/**
 * The intent as a key press delivers it, on the platforms where the WebView receives editing
 * chords. The platform and a rich-text editor already handle the press themselves, so they keep it.
 */
export function handleEditingChord(
  intent: EditingCommand,
  commands: readonly AppCommand[],
  event: KeyboardEvent,
): void {
  const focused = event.target instanceof Element ? event.target : document.activeElement;
  const responder = resolveEditingResponder(intent, commands, focused);
  if (responder.kind === "platform" || responder.kind === "editor-history") return;
  event.preventDefault();
  if (responder.kind === "command" && !event.repeat) runAppCommand(responder.command);
}
