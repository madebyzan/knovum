import { invokeCommand } from "../ipc/invoke-command";

export function getSecret(secretKey: string): Promise<string | null> {
  return invokeCommand<string | null>("plugin:secrets|get", { secretKey });
}

export function isSecretConfigured(secretKey: string): Promise<boolean> {
  return invokeCommand<boolean>("plugin:secrets|inspect", { secretKey });
}

export function setSecret(secretKey: string, secret: string): Promise<void> {
  return invokeCommand("plugin:secrets|set", { secretKey, secret });
}

export function deleteSecret(secretKey: string): Promise<void> {
  return invokeCommand("plugin:secrets|delete", { secretKey });
}

export async function persistOptionalSecret(secretKey: string, value: string): Promise<void> {
  const secret = value.trim();
  if (secret) {
    await setSecret(secretKey, secret);
    return;
  }
  await deleteSecret(secretKey);
}
