import { z } from "zod";
const templateMember = z.union([z.string(), z.array(z.string()), z.record(z.string(), z.string()), z.null()]);
export const resourceParameters = (text: string) =>
  z.record(z.string(), templateMember).parse(JSON.parse(text));
export const promptArguments = (text: string) => z.record(z.string(), z.string()).parse(JSON.parse(text));
