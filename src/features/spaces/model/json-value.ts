export type SpaceJsonValue =
  | string
  | number
  | boolean
  | null
  | SpaceJsonValue[]
  | { [key: string]: SpaceJsonValue };
