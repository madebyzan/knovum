import { clamp } from "es-toolkit";

export const IMAGE_ANGLE_YAW = { min: -90, max: 90 } as const;
export const IMAGE_ANGLE_PITCH = { min: -30, max: 60 } as const;

export function clampImageAngle(value: number, axis: "yaw" | "pitch") {
  const range = axis === "yaw" ? IMAGE_ANGLE_YAW : IMAGE_ANGLE_PITCH;
  return clamp(Math.round(value), range.min, range.max);
}
