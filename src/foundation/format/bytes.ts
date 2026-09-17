import { clamp } from "es-toolkit";

const UNITS = ["B", "KiB", "MiB", "GiB", "TiB"] as const;

export function formatBytes(value: number) {
  if (!Number.isFinite(value) || value < 0) throw new Error("Byte size must be a finite non-negative number");
  if (value === 0) return "0 B";
  const index = clamp(Math.floor(Math.log(value) / Math.log(1024)), 0, UNITS.length - 1);
  const scaled = value / 1024 ** index;
  return `${scaled >= 10 || index === 0 ? scaled.toFixed(0) : scaled.toFixed(1)} ${UNITS[index]}`;
}
