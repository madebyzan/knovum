import { execFileSync } from "node:child_process";
import { basename } from "node:path";

export function isMicrosoftRuntime(path) {
  return /^(?:concrt|msvcp|vcruntime)140(?:_[a-z0-9_]+)?\.dll$/u.test(basename(path).toLowerCase());
}

export function verifyMicrosoftRuntime(path) {
  if (!isMicrosoftRuntime(path)) throw new Error("Unexpected Microsoft runtime filename");
  const script =
    "$ErrorActionPreference='Stop'; $s=Get-AuthenticodeSignature -LiteralPath $env:KNOVUM_SIGNATURE_VERIFY_PATH; if($s.Status -ne 'Valid' -or $null -eq $s.SignerCertificate -or $s.SignerCertificate.Subject -notmatch '(^|,\\s*)O=Microsoft Corporation(,|$)'){throw 'Invalid Microsoft runtime publisher'}";
  execFileSync("powershell.exe", ["-NoProfile", "-NonInteractive", "-Command", script], {
    env: { ...process.env, KNOVUM_SIGNATURE_VERIFY_PATH: path },
    windowsHide: true,
  });
}
