import { clsx, type ClassValue } from "clsx";
import { extendTailwindMerge } from "tailwind-merge";

// tailwind-merge treats every non-builtin `text-*` as a colour, so the tokens.css sizes must be
// declared or they lose every conflict against a real `text-<colour>` utility.
const TEXT_SIZES = [
  "caption-sm",
  "caption-md",
  "label-md",
  "body-sm",
  "body-md",
  "title-sm",
  "title-md",
  "display-sm",
];
const RADII = ["panel", "section-inner"];
const SHADOWS = ["control", "panel", "workspace", "popover", "dialog"];

const twMerge = extendTailwindMerge({
  extend: {
    classGroups: {
      "font-size": [{ text: TEXT_SIZES }],
      rounded: [{ rounded: RADII }],
      shadow: [{ shadow: SHADOWS }],
    },
  },
});

export function cn(...inputs: ClassValue[]): string {
  return twMerge(clsx(inputs));
}
