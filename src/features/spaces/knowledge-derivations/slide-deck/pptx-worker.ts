/// <reference lib="webworker" />
import MarkdownIt from "markdown-it";
import pptxgen from "pptxgenjs";

interface PresentationRequest {
  title: string;
  markdown: string;
  images: Array<{ title: string; mime: string; bytes: ArrayBuffer }>;
}
interface SlideSection {
  title: string;
  lines: string[];
}
type PreparedImage = { title: string; data: string; width: number; height: number };

globalThis.addEventListener("message", async (event: MessageEvent<PresentationRequest>) => {
  try {
    const bytes = await build(event.data);
    globalThis.postMessage({ bytes }, [bytes]);
  } catch (error) {
    // oxlint-disable-next-line unicorn/require-post-message-target-origin -- DedicatedWorkerGlobalScope has no targetOrigin parameter.
    globalThis.postMessage({ error: String(error) });
  }
});

async function build(request: PresentationRequest) {
  const presentation = new pptxgen();
  presentation.layout = "LAYOUT_WIDE";
  presentation.author = "Knovum";
  presentation.subject = "Knowledge derivation";
  presentation.title = request.title;
  presentation.theme = {
    headFontFace: "Aptos Display",
    bodyFontFace: "Aptos",
  };
  const cover = presentation.addSlide();
  cover.background = { color: "F7F7F8" };
  cover.addText(request.title, {
    x: 0.9,
    y: 2.3,
    w: 11.5,
    h: 1.2,
    fontSize: 30,
    bold: true,
    color: "171719",
    margin: 0,
  });
  for (const section of sectionsFromMarkdown(request.markdown)) addSection(presentation, section);
  const images = await Promise.all(request.images.map(prepareImage));
  for (const image of images) addImage(presentation, image);
  const output = await presentation.write({ outputType: "arraybuffer" });
  if (!(output instanceof ArrayBuffer)) throw new Error("PPTX generator returned an unexpected output type");
  return output;
}

function sectionsFromMarkdown(markdown: string): SlideSection[] {
  const tokens = new MarkdownIt().parse(markdown, {});
  const sections: SlideSection[] = [];
  let section: SlideSection = { title: "Overview", lines: [] };
  for (let index = 0; index < tokens.length; index += 1) {
    const token = tokens[index]!;
    if (token.type === "heading_open") section = beginSection(sections, section, tokens[index + 1]?.content);
    else appendInline(section, token.type, tokens[index - 1]?.type, token.content);
  }
  if (section.lines.length || section.title !== "Overview") sections.push(section);
  return sections;
}

function beginSection(sections: SlideSection[], current: SlideSection, title?: string): SlideSection {
  if (current.lines.length > 0 || current.title !== "Overview") sections.push(current);
  return { title: title || "Section", lines: [] };
}

function appendInline(
  section: SlideSection,
  type: string,
  previousType: string | undefined,
  content: string,
) {
  const text = content.trim();
  if (type === "inline" && previousType !== "heading_open" && text) section.lines.push(text);
}

function addSection(presentation: pptxgen, section: SlideSection) {
  const slide = presentation.addSlide();
  slide.background = { color: "FFFFFF" };
  slide.addText(section.title, {
    x: 0.75,
    y: 0.55,
    w: 11.8,
    h: 0.55,
    fontSize: 24,
    bold: true,
    color: "171719",
    margin: 0,
  });
  slide.addText(
    section.lines.map((text) => ({ text, options: { bullet: { indent: 16 } } })),
    {
      x: 0.9,
      y: 1.45,
      w: 11.2,
      h: 5.45,
      fontSize: 18,
      color: "343438",
      breakLine: true,
      fit: "shrink",
      margin: 0.08,
    },
  );
}

function addImage(presentation: pptxgen, image: PreparedImage) {
  const slide = presentation.addSlide();
  slide.background = { color: "F3F3F5" };
  slide.addText(image.title, {
    x: 0.75,
    y: 0.35,
    w: 11.8,
    h: 0.45,
    fontSize: 20,
    bold: true,
    color: "171719",
    margin: 0,
  });
  const box = contain(image.width, image.height, { x: 1.1, y: 1, width: 11.1, height: 6 });
  slide.addImage({ data: image.data, x: box.x, y: box.y, w: box.width, h: box.height, transparency: 0 });
}

async function prepareImage(image: PresentationRequest["images"][number]): Promise<PreparedImage> {
  const bitmap = await createImageBitmap(new Blob([image.bytes], { type: image.mime }));
  const prepared = {
    title: image.title,
    data: `data:${image.mime};base64,${encodeBase64(new Uint8Array(image.bytes))}`,
    width: bitmap.width,
    height: bitmap.height,
  };
  bitmap.close();
  return prepared;
}

function encodeBase64(bytes: Uint8Array): string {
  const chunks: string[] = [];
  for (let offset = 0; offset < bytes.length; offset += 0x8000) {
    chunks.push(String.fromCharCode(...bytes.subarray(offset, offset + 0x8000)));
  }
  return btoa(chunks.join(""));
}

function contain(
  width: number,
  height: number,
  box: { x: number; y: number; width: number; height: number },
) {
  const scale = Math.min(box.width / width, box.height / height);
  const fittedWidth = width * scale;
  const fittedHeight = height * scale;
  return {
    x: box.x + (box.width - fittedWidth) / 2,
    y: box.y + (box.height - fittedHeight) / 2,
    width: fittedWidth,
    height: fittedHeight,
  };
}
