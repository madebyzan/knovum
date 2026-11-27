import { Extension, type Editor, type Range } from "@tiptap/core";
import { Plugin, PluginKey } from "@tiptap/pm/state";
import { Suggestion, type SuggestionKeyDownProps, type SuggestionProps } from "@tiptap/suggestion";
import { shallowRef, type ShallowRef } from "vue";

import type { SlashCommandCallbacks, SlashCommandItem } from "./slash-command-definition";
import { filterSlashCommands } from "./slash-commands";

const SLASH_PLUGIN_KEY = new PluginKey("spaceSlashCommand");

interface SlashMenuRect {
  readonly left: number;
  readonly top: number;
  readonly bottom: number;
}

interface SlashMenuState {
  readonly open: boolean;
  readonly query: string;
  readonly items: readonly SlashCommandItem[];
  readonly selectedIndex: number;
  readonly rect: SlashMenuRect | null;
}

export interface SlashCommandController {
  readonly state: Readonly<ShallowRef<SlashMenuState>>;
  selectIndex: (index: number) => void;
  executeSelected: () => void;
  executeItem: (item: SlashCommandItem) => void;
  refreshPosition: () => void;
  close: () => void;
  attachCallbacks: (callbacks: SlashCommandCallbacks) => void;
  createExtension: () => Extension;
}

const CLOSED_STATE: SlashMenuState = {
  open: false,
  query: "",
  items: [],
  selectedIndex: 0,
  rect: null,
};

function readRect(clientRect?: (() => DOMRect | null) | null): SlashMenuRect | null {
  const rect = clientRect?.();
  if (!rect) return null;
  return { left: rect.left, top: rect.top, bottom: rect.bottom };
}

export function createSlashCommandController(): SlashCommandController {
  const state = shallowRef<SlashMenuState>(CLOSED_STATE);
  let activeProps: SuggestionProps<SlashCommandItem> | null = null;
  let callbacks: SlashCommandCallbacks = {
    openAiPrompt: () => undefined,
    continueWithAi: () => undefined,
    summarizeWithAi: () => undefined,
    openMathPrompt: () => undefined,
  };

  function close() {
    activeProps = null;
    state.value = CLOSED_STATE;
  }

  function updateFromSuggestion(props: SuggestionProps<SlashCommandItem>) {
    activeProps = props;
    const items = props.items;
    const nextIndex = items.length ? Math.min(state.value.selectedIndex, items.length - 1) : 0;
    state.value = {
      open: true,
      query: props.query,
      items,
      selectedIndex: nextIndex,
      rect: readRect(props.clientRect),
    };
  }

  function refreshPosition() {
    if (!activeProps || !state.value.open) return;
    const rect = readRect(activeProps.clientRect);
    if (rect) state.value = { ...state.value, rect };
  }

  function executeItem(item: SlashCommandItem) {
    const current = activeProps;
    if (!current) return;
    const { editor, range } = current;
    close();
    item.run(editor, range, callbacks);
  }

  function executeSelected() {
    const item = state.value.items[state.value.selectedIndex];
    if (item) executeItem(item);
  }

  function selectIndex(index: number) {
    const length = state.value.items.length;
    if (!length) return;
    const wrapped = ((index % length) + length) % length;
    state.value = { ...state.value, selectedIndex: wrapped };
  }

  function handleKeyDown({ event }: SuggestionKeyDownProps): boolean {
    if (!state.value.open || event.isComposing) return false;
    if (event.key === "Escape") {
      close();
      return true;
    }
    if (!state.value.items.length) return false;
    if (event.key === "ArrowDown") {
      selectIndex(state.value.selectedIndex + 1);
      return true;
    }
    if (event.key === "ArrowUp") {
      selectIndex(state.value.selectedIndex - 1);
      return true;
    }
    if (event.key === "Enter") {
      executeSelected();
      return true;
    }
    return false;
  }

  function createExtension() {
    return Extension.create({
      name: "spaceSlashCommand",
      addProseMirrorPlugins() {
        return [
          new Plugin({
            props: {
              handleTextInput: (view, from, to, text) => {
                if (text !== "／" || !this.editor.isEditable || this.editor.isActive("codeBlock"))
                  return false;
                const before = view.state.doc.textBetween(Math.max(0, from - 1), from, "\n", "\n");
                if (before !== "" && before !== "\n" && !/\s/u.test(before)) return false;
                view.dispatch(view.state.tr.insertText("/", from, to));
                return true;
              },
            },
          }),
          Suggestion<SlashCommandItem>({
            pluginKey: SLASH_PLUGIN_KEY,
            editor: this.editor,
            char: "/",
            allowSpaces: false,
            allow: ({ editor: instance, state: editorState, range }) => {
              if (!instance.isEditable || instance.isActive("codeBlock")) return false;
              const before = editorState.doc.textBetween(Math.max(0, range.from - 1), range.from, "\n", "\n");
              return before === "" || before === "\n" || /\s/u.test(before);
            },
            items: ({ query }) => [...filterSlashCommands(query)],
            command: ({
              editor: instance,
              range,
              props,
            }: {
              editor: Editor;
              range: Range;
              props: SlashCommandItem;
            }) => {
              close();
              props.run(instance, range, callbacks);
            },
            render: () => ({
              onStart: updateFromSuggestion,
              onUpdate: updateFromSuggestion,
              onKeyDown: handleKeyDown,
              onExit: close,
            }),
          }),
        ];
      },
    });
  }

  return {
    state,
    selectIndex,
    executeSelected,
    executeItem,
    refreshPosition,
    close,
    attachCallbacks(next) {
      callbacks = next;
    },
    createExtension,
  };
}
