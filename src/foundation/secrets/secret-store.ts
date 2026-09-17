import { invoke } from "@tauri-apps/api/core";

export function getSecret(secretKey: string): Promise<string | null> {
  return invoke<string | null>("plugin:secrets|get", { secretKey });
}

export function setSecret(secretKey: string, secret: string): Promise<void> {
  return invoke("plugin:secrets|set", { secretKey, secret });
}

export function deleteSecret(secretKey: string): Promise<void> {
  return invoke("plugin:secrets|delete", { secretKey });
}

export async function persistOptionalSecret(secretKey: string, value: string): Promise<void> {
  const secret = value.trim();
  if (secret) {
    await setSecret(secretKey, secret);
    return;
  }
  await deleteSecret(secretKey);
}
