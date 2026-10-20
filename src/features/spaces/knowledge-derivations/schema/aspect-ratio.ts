import { z } from "zod";

export const imageAspectRatioSchema = z.enum(["1:1", "3:2", "2:3", "16:9", "9:16"]);
export const videoAspectRatioSchema = z.enum(["16:9", "9:16", "1:1"]);
