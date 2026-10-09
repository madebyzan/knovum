import { z } from "zod";
const templateMember = z.union([z.string(), z.array(z.string()), z.record(z.string(), z.string()), z.null()]);
export function resourceParameters(text: string) {
  return z.record(z.string(), templateMember).parse(JSON.parse(text));
}
export function promptArguments(text: string) {
  return z.record(z.string(), z.string()).parse(JSON.parse(text));
}
