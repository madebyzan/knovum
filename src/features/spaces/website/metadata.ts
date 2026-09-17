import type { SpaceWebsitePayload } from "../model/nodes/node-payload";

export type SpaceWebsiteSection = NonNullable<SpaceWebsitePayload["websiteSections"]>[number];

export interface SpaceWebsiteMetadata {
  title: string;
  finalUrl: string;
  canonicalUrl?: string;
  siteName?: string;
  description?: string;
  faviconUrl?: string;
  previewImageUrl?: string;
  contentPreview?: string;
  knowledgeSections?: SpaceWebsiteSection[];
}
