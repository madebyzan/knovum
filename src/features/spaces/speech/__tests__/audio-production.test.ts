import { describe, expect, it } from "vitest";

import { audiobookDocumentScript } from "../audiobook-document-script";
import { parseDubbingSubtitles } from "../dubbing-subtitles";
import { useAudiobookOutput } from "../use-audiobook-output";

const entry = (path: string, text: string) => ({ path, text });

describe("audio production scripts", () => {
  it("imports EPUB chapters in declared spine order and excludes navigation", () => {
    const source = {
      kind: "epub" as const,
      entries: [
        entry(
          "META-INF/container.xml",
          '<container xmlns="urn:oasis:names:tc:opendocument:xmlns:container"><rootfiles><rootfile full-path="OEBPS/book.opf" /></rootfiles></container>',
        ),
        entry(
          "OEBPS/book.opf",
          '<package xmlns="http://www.idpf.org/2007/opf"><manifest><item id="last" href="last.xhtml" media-type="application/xhtml+xml"/><item id="first" href="first.xhtml" media-type="application/xhtml+xml"/></manifest><spine><itemref idref="first"/><itemref idref="last"/></spine></package>',
        ),
        entry("OEBPS/last.xhtml", "<h1>Second</h1><p>Second paragraph.</p>"),
        entry(
          "OEBPS/first.xhtml",
          "<nav>Do not narrate navigation</nav><h1>First</h1><p>First<br/>paragraph.</p>",
        ),
      ],
    };
    const script = audiobookDocumentScript(source, "Book", "Narrator");
    expect(script.chapters.map((chapter) => chapter.title)).toEqual(["First", "Second"]);
    expect(script.chapters[0]?.paragraphs).toEqual([{ role: "Narrator", text: "First\nparagraph." }]);
  });

  it("rejects external EPUB spine resources instead of downloading content", () => {
    const source = {
      kind: "epub" as const,
      entries: [
        entry(
          "META-INF/container.xml",
          '<container xmlns="urn:oasis:names:tc:opendocument:xmlns:container"><rootfile full-path="book.opf" /></container>',
        ),
        entry(
          "book.opf",
          '<package xmlns="http://www.idpf.org/2007/opf"><item id="a" href="https://example.org/chapter.xhtml" media-type="application/xhtml+xml"/><itemref idref="a"/></package>',
        ),
      ],
    };
    expect(() => audiobookDocumentScript(source, "Book", "Narrator")).toThrow("external resource");
  });

  it("keeps Markdown headings and paragraph boundaries", () => {
    const script = audiobookDocumentScript(
      {
        kind: "text",
        text: "# Chapter one\n\nFirst paragraph.\n\nSecond paragraph.\n\n# Chapter two\n\nThird paragraph.",
      },
      "Book",
      "Narrator",
    );
    expect(script.chapters.map((chapter) => chapter.paragraphs.length)).toEqual([2, 1]);
    expect(script.chapters[1]?.title).toBe("Chapter two");
  });

  it("imports millisecond subtitles and WebVTT speaker bindings", () => {
    expect(
      parseDubbingSubtitles("1\n00:00:01,250 --> 00:00:03,500\nHello &amp; goodbye.\n", "Narrator"),
    ).toEqual([{ startMs: 1250, endMs: 3500, text: "Hello & goodbye.", speakerKey: "Narrator" }]);
    expect(
      parseDubbingSubtitles(
        "WEBVTT\n\ncue-1\n00:01.000 --> 00:02.100 align:start\n<v Alice><b>Hello</b></v>",
        "Narrator",
      )[0],
    ).toEqual({ startMs: 1000, endMs: 2100, text: "Hello", speakerKey: "Alice" });
  });

  it("rejects malformed subtitle timings and unsupported duration", () => {
    expect(() => parseDubbingSubtitles("1\n00:60:00,000 --> 00:60:01,000\nHello", "Narrator")).toThrow(
      "timestamp",
    );
    expect(() => parseDubbingSubtitles("1\n00:00:02,000 --> 00:00:01,000\nHello", "Narrator")).toThrow(
      "limits",
    );
    expect(() => parseDubbingSubtitles("1\n00:00:00,000 --> 12:00:00,001\nHello", "Narrator")).toThrow(
      "limits",
    );
  });

  it("publishes configured pauses and precision and rejects WAV with an embedded cover", () => {
    const output = useAudiobookOutput();
    output.paragraphPause.value = "400";
    output.chapterPause.value = "1800";
    output.wavPrecision.value = "pcm24";
    expect(output.value()).toMatchObject({
      paragraphPauseMs: 400,
      chapterPauseMs: 1800,
      wavPrecision: "pcm24",
    });
    output.metadata.value = {
      title: "Book",
      author: "Author",
      narrator: "Narrator",
      cover: { assetPath: "/cover.png", sourceRevision: `sha256:${"a".repeat(64)}` },
    };
    output.format.value = "wav";
    expect(output.valid()).toBe(false);
    output.format.value = "m4b";
    expect(output.valid()).toBe(true);
    output.paragraphPause.value = "NaN";
    expect(output.valid()).toBe(false);
  });
});
