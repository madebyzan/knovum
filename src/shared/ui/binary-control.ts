import type { HTMLAttributes } from "vue";

/** Props shared by every labelled two-state control (checkbox, switch). */
export interface BinaryControlProps {
  id?: string | undefined;
  label: string;
  disabled?: boolean | undefined;
  class?: HTMLAttributes["class"];
}
