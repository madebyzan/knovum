import { queryOptions } from "@tanstack/vue-query";
import { z } from "zod";

import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import type { InstalledFont } from "../contracts/generated/InstalledFont";

const installedFontSchema: z.ZodType<InstalledFont> = z
  .object({
    family: z.string().min(1),
    weights: z.array(z.number().int().min(100).max(900)).min(1),
  })
  .strict();
const fontCatalogSchema = z.array(installedFontSchema);

export const videoFontCatalogQuery = queryOptions({
  queryKey: ["spaces", "video-editor", "installed-fonts"],
  queryFn: async (): Promise<InstalledFont[]> =>
    fontCatalogSchema.parse(await invokeCommand<unknown>("plugin:spaces|list_video_fonts")),
  staleTime: Infinity,
});
