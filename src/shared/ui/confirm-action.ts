import { readonly, shallowRef } from "vue";

export interface ConfirmActionOptions {
  title: string;
  description: string;
  confirmLabel: string;
  danger?: boolean;
}

interface Confirmation extends ConfirmActionOptions {
  resolve: (confirmed: boolean) => void;
}

const current = shallowRef<Confirmation>();
const queue: Confirmation[] = [];

export const activeConfirmation = readonly(current);

export function confirmAction(options: ConfirmActionOptions): Promise<boolean> {
  return new Promise((resolve) => {
    queue.push({ ...options, resolve });
    current.value ??= queue.shift();
  });
}

export function resolveConfirmation(confirmed: boolean): void {
  const confirmation = current.value;
  if (!confirmation) return;
  current.value = queue.shift();
  confirmation.resolve(confirmed);
}
