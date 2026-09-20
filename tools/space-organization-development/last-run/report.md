## Organization development verification

Source commit: `ac3b8f0c3d7219df6f27cd49d49da80a6390c140`. Actual runner results; not a complete product acceptance.

| Check | Result |
| --- | --- |
| Types | FAIL (2) |
| Architecture | FAIL (1) |
| Frontend tests | FAIL (1) |
| Build | PASS |

<details><summary>Types failures</summary>

```text
$ vue-tsc --noEmit && tsc --noEmit -p tsconfig.node.json
src/features/spaces/__tests__/canvas-clipboard-paste.test.ts(110,95): error TS2493: Tuple type '[]' of length '0' has no element at index '0'.
src/features/spaces/__tests__/canvas-editor-navigation.test.ts(8,25): error TS2345: Argument of type '{ layouts: () => { nodeId: string; frame: { x: number; y: number; width: number; height: number; rotation: number; }; sizeAuthority: string; zIndex: number; }[]; selectedNodeIds: () => Set<string>; setSelection: Mock<...>; setViewport: (viewport: { x: number; y: number; zoom: number; }) => void; size: () => { ...; }...' is not assignable to parameter of type 'CanvasNavigationContext'.
  The types returned by 'layouts()' are incompatible between these types.
    Type '{ nodeId: string; frame: { x: number; y: number; width: number; height: number; rotation: number; }; sizeAuthority: string; zIndex: number; }[]' is not assignable to type 'readonly { nodeId: string; frame: { x: number; y: number; width: number; height: number; rotation: number; }; sizeAuthority: "content" | "frame"; zIndex: number; appearance?: { fill?: "blue" | "green" | "yellow" | "pink" | "gray" | undefined; ... 10 more ...; backgroundStyle?: "repeat" | ... 2 more ... | undefined; ...'.
      Type '{ nodeId: string; frame: { x: number; y: number; width: number; height: number; rotation: number; }; sizeAuthority: string; zIndex: number; }' is not assignable to type '{ nodeId: string; frame: { x: number; y: number; width: number; height: number; rotation: number; }; sizeAuthority: "content" | "frame"; zIndex: number; appearance?: { fill?: "blue" | "green" | "yellow" | "pink" | "gray" | undefined; ... 10 more ...; backgroundStyle?: "repeat" | ... 2 more ... | undefined; } | undef...'.
        Types of property 'sizeAuthority' are incompatible.
          Type 'string' is not assignable to type '"content" | "frame"'.
src/features/spaces/__tests__/canvas-editor-navigation.test.ts(15,25): error TS2345: Argument of type '{ layouts: () => { nodeId: string; frame: { x: number; y: number; width: number; height: number; rotation: number; }; sizeAuthority: string; zIndex: number; }[]; selectedNodeIds: () => Set<string>; setSelection: Mock<...>; setViewport: (viewport: { x: number; y: number; zoom: number; }) => void; size: () => { ...; }...' is not assignable to parameter of type 'CanvasNavigationContext'.
  The types returned by 'layouts()' are incompatible between these types.
    Type '{ nodeId: string; frame: { x: number; y: number; width: number; height: number; rotation: number; }; sizeAuthority: string; zIndex: number; }[]' is not assignable to type 'readonly { nodeId: string; frame: { x: number; y: number; width: number; height: number; rotation: number; }; sizeAuthority: "content" | "frame"; zIndex: number; appearance?: { fill?: "blue" | "green" | "yellow" | "pink" | "gray" | undefined; ... 10 more ...; backgroundStyle?: "repeat" | ... 2 more ... | undefined; ...'.
      Type '{ nodeId: string; frame: { x: number; y: number; width: number; height: number; rotation: number; }; sizeAuthority: string; zIndex: number; }' is not assignable to type '{ nodeId: string; frame: { x: number; y: number; width: number; height: number; rotation: number; }; sizeAuthority: "content" | "frame"; zIndex: number; appearance?: { fill?: "blue" | "green" | "yellow" | "pink" | "gray" | undefined; ... 10 more ...; backgroundStyle?: "repeat" | ... 2 more ... | undefined; } | undef...'.
        Types of property 'sizeAuthority' are incompatible.
          Type 'string' is not assignable to type '"content" | "frame"'.
src/features/spaces/__tests__/canvas-node-interaction.test.ts(28,28): error TS2345: Argument of type '{ kind: "file"; title: string; entries: { path: string; kind: "file"; }[]; }' is not assignable to parameter of type '{ kind: "subtitle"; sourceVideoNodeId: string; sourceVideoRevision: string; language: string; durationMs: number; cues: { startMs: number; endMs: number; text: string; }[]; title: string; tags?: string[] | undefined; sectionId?: string | undefined; locked?: boolean | undefined; sourceRevision?: string | undefined; k...'.
  Type '{ kind: "file"; title: string; entries: { path: string; kind: "file"; }[]; }' is missing the following properties from type '{ kind: "file"; assetPath: string; fileName: string; title: string; documentCoverage?: { schemaVersion: 1; parser: "pdf-inspector" | "office-oxide"; sourceRevision: string; pageCount: number | null; ... 5 more ...; warnings: ("office_vector_drawings_unsupported" | ... 4 more ... | "pdf_visual_coverage_limited")[]; }...': assetPath, fileName
src/features/spaces/__tests__/canvas-node-interaction.test.ts(35,53): error TS2345: Argument of type '{ kind: "file"; title: string; }' is not assignable to parameter of type '{ kind: "subtitle"; sourceVideoNodeId: string; sourceVideoRevision: string; language: string; durationMs: number; cues: { startMs: number; endMs: number; text: string; }[]; title: string; tags?: string[] | undefined; sectionId?: string | undefined; locked?: boolean | undefined; sourceRevision?: string | undefined; k...'.
  Type '{ kind: "file"; title: string; }' is missing the following properties from type '{ kind: "file"; assetPath: string; fileName: string; title: string; documentCoverage?: { schemaVersion: 1; parser: "pdf-inspector" | "office-oxide"; sourceRevision: string; pageCount: number | null; ... 5 more ...; warnings: ("office_vector_drawings_unsupported" | ... 4 more ... | "pdf_visual_coverage_limited")[]; }...': assetPath, fileName
src/features/spaces/__tests__/canvas-node-interaction.test.ts(81,10): error TS2322: Type '{ nodeId: string; frame: { x: number; y: number; width: number; height: number; rotation: number; }; sizeAuthority: string; zIndex: number; }' is not assignable to type '{ nodeId: string; frame: { x: number; y: number; width: number; height: number; rotation: number; }; sizeAuthority: "content" | "frame"; zIndex: number; appearance?: { fill?: "blue" | "green" | "yellow" | "pink" | "gray" | undefined; ... 10 more ...; backgroundStyle?: "repeat" | ... 2 more ... | undefined; } | undef...'.
  Types of property 'sizeAuthority' are incompatible.
    Type 'string' is not assignable to type '"content" | "frame"'.
src/features/spaces/__tests__/canvas-node-interaction.test.ts(110,44): error TS2345: Argument of type '{ layout: { nodeId: string; frame: { x: number; y: number; width: number; height: number; rotation: number; }; sizeAuthority: string; zIndex: number; }; node: SpaceCanvasNode<{ kind: "subtitle"; sourceVideoNodeId: string; ... 9 more ...; knowledgeRevision?: string | undefined; } | ... 12 more ... | { ...; }>; intera...' is not assignable to parameter of type 'readonly CanvasSceneNode[]'.
  Type '{ layout: { nodeId: string; frame: { x: number; y: number; width: number; height: number; rotation: number; }; sizeAuthority: string; zIndex: number; }; node: SpaceCanvasNode<{ kind: "subtitle"; sourceVideoNodeId: string; ... 9 more ...; knowledgeRevision?: string | undefined; } | ... 12 more ... | { ...; }>; intera...' is not assignable to type 'CanvasSceneNode'.
    The types of 'layout.sizeAuthority' are incompatible between these types.
      Type 'string' is not assignable to type '"content" | "frame"'.
src/features/spaces/__tests__/canvas-surface-host.test.ts(184,5): error TS2322: Type '{ events: CanvasHostEvents; importJsonCanvas: Mock<(request: CanvasJsonImportRequest) => Promise<CanvasJsonImportCommitResult>>; focusNodeIds: never[]; tool: "select"; nodes: SpaceCanvasNode<...>[]; relations: never[]; spaceId: string; }' is not assignable to type 'VNodeProps & { __v_isVNode?: never; [Symbol.iterator]?: never; } & Record<string, any> & { readonly events: CanvasHostEvents; ... 7 more ...; readonly onExchangeReady?: (exchange: CanvasExchangePort) => any; } & AllowedComponentProps & ComponentCustomProps'.
  Property 'interactionEnabled' is missing in type '{ events: CanvasHostEvents; importJsonCanvas: Mock<(request: CanvasJsonImportRequest) => Promise<CanvasJsonImportCommitResult>>; focusNodeIds: never[]; tool: "select"; nodes: SpaceCanvasNode<...>[]; relations: never[]; spaceId: string; }' but required in type '{ readonly events: CanvasHostEvents; readonly importJsonCanvas: (request: CanvasJsonImportRequest) => Promise<CanvasJsonImportCommitResult>; ... 6 more ...; readonly onExchangeReady?: (exchange: CanvasExchangePort) => any; }'.
src/features/spaces/__tests__/canvas-surface-remount.test.ts(143,5): error TS2322: Type '{ events: CanvasHostEvents; importJsonCanvas: Mock<(request: CanvasJsonImportRequest) => Promise<CanvasJsonImportCommitResult>>; focusNodeIds: never[]; tool: "select"; nodes: SpaceCanvasNode<...>[]; relations: never[]; spaceId: string; }' is not assignable to type 'VNodeProps & { __v_isVNode?: never; [Symbol.iterator]?: never; } & Record<string, any> & { readonly events: CanvasHostEvents; ... 7 more ...; readonly onExchangeReady?: (exchange: CanvasExchangePort) => any; } & AllowedComponentProps & ComponentCustomProps'.
  Property 'interactionEnabled' is missing in type '{ events: CanvasHostEvents; importJsonCanvas: Mock<(request: CanvasJsonImportRequest) => Promise<CanvasJsonImportCommitResult>>; focusNodeIds: never[]; tool: "select"; nodes: SpaceCanvasNode<...>[]; relations: never[]; spaceId: string; }' but required in type '{ readonly events: CanvasHostEvents; readonly importJsonCanvas: (request: CanvasJsonImportRequest) => Promise<CanvasJsonImportCommitResult>; ... 6 more ...; readonly onExchangeReady?: (exchange: CanvasExchangePort) => any; }'.
src/features/spaces/__tests__/json-canvas-mapping.test.ts(77,12): error TS2741: Property 'sizeAuthority' is missing in type '{ nodeId: string; sourceId: string; frame: { x: number; y: number; width: number; height: number; rotation: 0; }; appearance?: CanvasNodeAppearance; fileSubpath?: string; parentSourceId?: string; zIndex: number; content: JsonCanvasNodeContentV1; }' but required in type '{ nodeId: string; frame: { x: number; y: number; width: number; height: number; rotation: number; }; sizeAuthority: "content" | "frame"; zIndex: number; appearance?: { fill?: "blue" | "green" | "yellow" | "pink" | "gray" | undefined; ... 10 more ...; backgroundStyle?: "repeat" | ... 2 more ... | undefined; } | undef...'.
src/features/spaces/__tests__/json-canvas-mapping.test.ts(102,12): error TS2741: Property 'sizeAuthority' is missing in type '{ nodeId: string; sourceId: string; frame: { x: number; y: number; width: number; height: number; rotation: 0; }; appearance?: CanvasNodeAppearance; fileSubpath?: string; parentSourceId?: string; zIndex: number; content: JsonCanvasNodeContentV1; }' but required in type '{ nodeId: string; frame: { x: number; y: number; width: number; height: number; rotation: number; }; sizeAuthority: "content" | "frame"; zIndex: number; appearance?: { fill?: "blue" | "green" | "yellow" | "pink" | "gray" | undefined; ... 10 more ...; backgroundStyle?: "repeat" | ... 2 more ... | undefined; } | undef...'.
src/features/spaces/__tests__/json-canvas-service.test.ts(51,25): error TS7006: Parameter 'observer' implicitly has an 'any' type.
src/features/spaces/__tests__/konva-canvas-document.test.ts(116,30): error TS2345: Argument of type '{ nodes: ({ nodeId: string; frame: { x: number; y: number; width: number; height: number; rotation: number; }; sizeAuthority: "content" | "frame"; zIndex: number; appearance?: { fill?: "blue" | "green" | "yellow" | "pink" | "gray" | undefined; ... 10 more ...; backgroundStyle?: "repeat" | ... 2 more ... | undefined;...' is not assignable to parameter of type '{ version: 1; viewport: { x: number; y: number; zoom: number; }; nodes: { nodeId: string; frame: { x: number; y: number; width: number; height: number; rotation: number; }; sizeAuthority: "content" | "frame"; zIndex: number; appearance?: { ...; } | undefined; fileSubpath?: string | undefined; }[]; relationStyles?: {...'.
  Types of property 'nodes' are incompatible.
    Type '({ nodeId: string; frame: { x: number; y: number; width: number; height: number; rotation: number; }; sizeAuthority: "content" | "frame"; zIndex: number; appearance?: { fill?: "blue" | "green" | "yellow" | "pink" | "gray" | undefined; ... 10 more ...; backgroundStyle?: "repeat" | ... 2 more ... | undefined; } | unde...' is not assignable to type '{ nodeId: string; frame: { x: number; y: number; width: number; height: number; rotation: number; }; sizeAuthority: "content" | "frame"; zIndex: number; appearance?: { fill?: "blue" | "green" | "yellow" | "pink" | "gray" | undefined; ... 10 more ...; backgroundStyle?: "repeat" | ... 2 more ... | undefined; } | undef...'.
      Type '{ nodeId: string; frame: { x: number; y: number; width: number; height: number; rotation: number; }; sizeAuthority: "content" | "frame"; zIndex: number; appearance?: { fill?: "blue" | "green" | "yellow" | "pink" | "gray" | undefined; ... 10 more ...; backgroundStyle?: "repeat" | ... 2 more ... | undefined; } | undef...' is not assignable to type '{ nodeId: string; frame: { x: number; y: number; width: number; height: number; rotation: number; }; sizeAuthority: "content" | "frame"; zIndex: number; appearance?: { fill?: "blue" | "green" | "yellow" | "pink" | "gray" | undefined; ... 10 more ...; backgroundStyle?: "repeat" | ... 2 more ... | undefined; } | undef...'. Two different types with this name exist, but they are unrelated.
        Type '{ nodeId: string; frame: { x: number; y: number; width: number; height: number; rotation: number; }; sizeAuthority: string; zIndex: number; }' is not assignable to type '{ nodeId: string; frame: { x: number; y: number; width: number; height: number; rotation: number; }; sizeAuthority: "content" | "frame"; zIndex: number; appearance?: { fill?: "blue" | "green" | "yellow" | "pink" | "gray" | undefined; ... 10 more ...; backgroundStyle?: "repeat" | ... 2 more ... | undefined; } | undef...'.
          Types of property 'sizeAuthority' are incompatible.
            Type 'string' is not assignable to type '"content" | "frame"'.
src/features/spaces/__tests__/konva-canvas-document.test.ts(119,7): error TS2322: Type '{ nodes: ({ nodeId: string; frame: { x: number; y: number; width: number; height: number; rotation: number; }; sizeAuthority: "content" | "frame"; zIndex: number; appearance?: { fill?: "blue" | "green" | "yellow" | "pink" | "gray" | undefined; ... 10 more ...; backgroundStyle?: "repeat" | ... 2 more ... | undefined;...' is not assignable to type '{ version: 1; viewport: { x: number; y: number; zoom: number; }; nodes: { nodeId: string; frame: { x: number; y: number; width: number; height: number; rotation: number; }; sizeAuthority: "content" | "frame"; zIndex: number; appearance?: { ...; } | undefined; fileSubpath?: string | undefined; }[]; relationStyles?: {...'.
  Types of property 'nodes' are incompatible.
    Type '({ nodeId: string; frame: { x: number; y: number; width: number; height: number; rotation: number; }; sizeAuthority: "content" | "frame"; zIndex: number; appearance?: { fill?: "blue" | "green" | "yellow" | "pink" | "gray" | undefined; ... 10 more ...; backgroundStyle?: "repeat" | ... 2 more ... | undefined; } | unde...' is not assignable to type '{ nodeId: string; frame: { x: number; y: number; width: number; height: number; rotation: number; }; sizeAuthority: "content" | "frame"; zIndex: number; appearance?: { fill?: "blue" | "green" | "yellow" | "pink" | "gray" | undefined; ... 10 more ...; backgroundStyle?: "repeat" | ... 2 more ... | undefined; } | undef...'.
      Type '{ nodeId: string; frame: { x: number; y: number; width: number; height: number; rotation: number; }; sizeAuthority: "content" | "frame"; zIndex: number; appearance?: { fill?: "blue" | "green" | "yellow" | "pink" | "gray" | undefined; ... 10 more ...; backgroundStyle?: "repeat" | ... 2 more ... | undefined; } | undef...' is not assignable to type '{ nodeId: string; frame: { x: number; y: number; width: number; height: number; rotation: number; }; sizeAuthority: "content" | "frame"; zIndex: number; appearance?: { fill?: "blue" | "green" | "yellow" | "pink" | "gray" | undefined; ... 10 more ...; backgroundStyle?: "repeat" | ... 2 more ... | undefined; } | undef...'. Two different types with this name exist, but they are unrelated.
        Type '{ nodeId: string; frame: { x: number; y: number; width: number; height: number; rotation: number; }; sizeAuthority: string; zIndex: number; }' is not assignable to type '{ nodeId: string; frame: { x: number; y: number; width: number; height: number; rotation: number; }; sizeAuthority: "content" | "frame"; zIndex: number; appearance?: { fill?: "blue" | "green" | "yellow" | "pink" | "gray" | undefined; ... 10 more ...; backgroundStyle?: "repeat" | ... 2 more ... | undefined; } | undef...'.
          Types of property 'sizeAuthority' are incompatible.
            Type 'string' is not assignable to type '"content" | "frame"'.
src/features/spaces/__tests__/konva-rich-node-interactions.test.ts(27,9): error TS2322: Type 'Mock<(id: string, data: { kind: "note"; tiptapContent: TiptapJsonNode; plainText: string; title: string; tags?: string[] | undefined; sectionId?: string | undefined; locked?: boolean | undefined; sourceRevision?: string | undefined; knowledgeRevision?: string | undefined; }) => Promise<...>>' is not assignable to type '(id: string, data: SpaceCanvasTextPayload) => Promise<void>'.
  Types of parameters 'data' and 'data' are incompatible.
    Type 'SpaceCanvasTextPayload' is not assignable to type '{ kind: "note"; tiptapContent: TiptapJsonNode; plainText: string; title: string; tags?: string[] | undefined; sectionId?: string | undefined; locked?: boolean | undefined; sourceRevision?: string | undefined; knowledgeRevision?: string | undefined; }'.
      Type '{ kind: "text"; tiptapContent: TiptapJsonNode; plainText: string; title: string; tags?: string[] | undefined; sectionId?: string | undefined; locked?: boolean | undefined; sourceRevision?: 
```
</details>

<details><summary>Architecture failures</summary>

```text
$ node tools/check-frontend-source-structure.mjs && node tools/check-ai-provider-catalog.mjs && depcruise --config .dependency-cruiser.cjs src
Frontend source structure gate failed (7):
- src/features/spaces/__tests__/canvas-surface-host.test.ts: 184 effective lines exceeds 180
- src/features/spaces/__tests__/konva-scene-projection.test.ts: 210 effective lines exceeds 180
- src/features/spaces/canvas/konva/use-konva-canvas-runtime.ts: 183 effective lines exceeds 180
- src/features/spaces/presentation/SpaceOutline.vue: 185 effective lines exceeds 180
- src/features/spaces/presentation/canvas/SpaceCanvas.vue: 188 effective lines exceeds 180
- src/features/spaces/presentation/knowledge/SpaceKnowledgeTypeManager.vue: 211 effective lines exceeds 180
- src/features/spaces/__tests__/capture-url-fixture.ts: files under __tests__ must use the exact .test.ts suffix
[ELIFECYCLE] Command failed with exit code 1.

```
</details>

<details><summary>Frontend tests failures</summary>

```text
529 passed, 18 failed
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-surface-host.test.ts :: CanvasSurfaceHost rejects an in-flight commit after the active space changes
Error: expected [Function] to throw error including 'Canvas transaction target changed' but got 'Canvas workspace changed during persi…'
    at _Assertion.__VITEST_REJECTS__ (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+expect@4.1.10/node_modules/@vitest/expect/dist/index.js:1747:17)
    at _Assertion.propertyGetter (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/chai@6.2.2/node_modules/chai/index.js:1570:27)
    at Reflect.get (<anonymous>)
    at Object.proxyGetter [as get] (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/chai@6.2.2/node_modules/chai/index.js:1659:22)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-surface-host.test.ts:84:30
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:20
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-surface-host.test.ts :: CanvasSurfaceHost rejects an in-flight commit after unmount
Error: expected [Function] to throw error including 'Canvas transaction target changed' but got 'Canvas workspace changed during persi…'
    at _Assertion.__VITEST_REJECTS__ (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+expect@4.1.10/node_modules/@vitest/expect/dist/index.js:1747:17)
    at _Assertion.propertyGetter (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/chai@6.2.2/node_modules/chai/index.js:1570:27)
    at Reflect.get (<anonymous>)
    at Object.proxyGetter [as get] (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/chai@6.2.2/node_modules/chai/index.js:1659:22)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-surface-host.test.ts:105:30
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:20
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-surface-remount.test.ts :: CanvasSurfaceHost remount persistence waits for an in-flight formal commit before remounting the same space
Error: expected undefined to deeply equal Error: Canvas transaction target changed
    at _Assertion.__VITEST_RESOLVES__ (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+expect@4.1.10/node_modules/@vitest/expect/dist/index.js:1710:17)
    at _Assertion.propertyGetter (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/chai@6.2.2/node_modules/chai/index.js:1570:27)
    at Reflect.get (<anonymous>)
    at Object.proxyGetter [as get] (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/chai@6.2.2/node_modules/chai/index.js:1659:22)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-surface-remount.test.ts:126:31
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:20
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/derivation-schema.test.ts :: knowledge video package metadata accepts complete generated video metadata
AssertionError: expected false to be true // Object.is equality
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/derivation-schema.test.ts:32:76
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2305:20
    at new Promise (<anonymous>)
    at runWithTimeout (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2272:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2955:64
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/derivation-schema.test.ts :: document export package metadata accepts a completed Markdown PDF export without provider metadata
AssertionError: expected false to be true // Object.is equality
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/derivation-schema.test.ts:64:7
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2305:20
    at new Promise (<anonymous>)
    at runWithTimeout (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2272:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2955:64
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/knowledge-location.test.ts :: Space knowledge locations checks source and editor revisions against the correct authority
AssertionError: expected false to be true // Object.is equality
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/knowledge-location.test.ts:32:78
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2305:20
    at new Promise (<anonymous>)
    at runWithTimeout (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2272:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2955:64
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/konva-canvas-document.test.ts :: Konva canvas document synchronization adopts intrinsic media geometry until the persisted frame becomes authoritative
AssertionError: expected { nodeId: 'image', …(3) } to match object { sizeAuthority: 'content', …(1) }
(5 matching properties omitted from actual)
    at Proxy.<anonymous> (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+expect@4.1.10/node_modules/@vitest/expect/dist/index.js:1283:10)
    at Proxy.<anonymous> (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+expect@4.1.10/node_modules/@vitest/expect/dist/index.js:1156:15)
    at Proxy.methodWrapper (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/chai@6.2.2/node_modules/chai/index.js:1700:25)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/konva-canvas-document.test.ts:51:32
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2305:20
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/konva-rich-node-interactions.test.ts :: Konva rich node interactions persists direct note editing and refuses editing for locked notes
TypeError: Cannot read properties of undefined (reading 'icon')
    at /home/runner/work/knovum/knovum/src/features/spaces/presentation/canvas/nodes/SpaceCanvasTextAlignmentMenu.vue:28:38
    at renderFnWithContext (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:704:13)
    at renderSlot (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:3327:55)
    at /home/runner/work/knovum/knovum/src/shared/ui/UiIconButton.vue:49:7
    at renderFnWithContext (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:704:13)
    at renderSlot (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:3327:55)
    at /home/runner/work/knovum/knovum/src/shared/ui/UiTooltip.vue:34:32
    at renderFnWithContext (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:704:13)
    at renderSlot (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:3327:55)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/reka-ui@2.10.4_vue@3.5.42_typescript@6.0.3_/node_modules/reka-ui/dist/Tooltip/TooltipTrigger.js:94:30
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/konva-scene-projection.test.ts :: Konva scene projection projects collapsed sections without mutating stored geometry or rendering members
AssertionError: expected { x: +0, y: +0, width: 288, …(2) } to match object { width: 360, height: 36 }
(3 matching properties omitted from actual)
    at Proxy.<anonymous> (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+expect@4.1.10/node_modules/@vitest/expect/dist/index.js:1283:10)
    at Proxy.<anonymous> (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+expect@4.1.10/node_modules/@vitest/expect/dist/index.js:1156:15)
    at Proxy.methodWrapper (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/chai@6.2.2/node_modules/chai/index.js:1700:25)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/konva-scene-projection.test.ts:74:45
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2305:20
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/konva-scene-projection.test.ts :: Konva scene projection projects external relations from collapsed members onto their section boundary
AssertionError: expected [ 288, 18 ] to deeply equal [ 360, 18 ]
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/konva-scene-projection.test.ts:206:43
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2305:20
    at new Promise (<anonymous>)
    at runWithTimeout (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2272:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2955:64
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/manifest-parser.test.ts :: Space package manifest parser rejects derivations that reference unavailable nodes
AssertionError: expected [Function] to throw error matching /unavailable node/u but got '[\n  {\n    "expected": "string",\n  …'
    at Proxy.<anonymous> (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+expect@4.1.10/node_modules/@vitest/expect/dist/index.js:1552:16)
    at Proxy.<anonymous> (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+expect@4.1.10/node_modules/@vitest/expect/dist/index.js:1156:15)
    at Proxy.methodWrapper (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/chai@6.2.2/node_modules/chai/index.js:1700:25)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/manifest-parser.test.ts:128:88
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2305:20
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/space-session-actions.test.ts :: openSpace coalesces repeated clicks while the same space is loading
AssertionError: expected "vi.fn()" to be called 1 times, but got 0 times
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/space-session-actions.test.ts:78:38
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2305:20
    at new Promise (<anonymous>)
    at runWithTimeout (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2272:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2955:64
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/space-session-actions.test.ts :: openSpace keeps the current canvas snapshot visible until the new Space is ready
Error: STACK_TRACE_ERROR
    at task (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1784:27)
    at Object.<anonymous> (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1817:16)
    at Object.<anonymous> (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1563:28)
    at chain (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:599:14)
    at /home/runner/work/knovum
```
</details>

Complete logs and test JSON are attached to this workflow run. No gates, tests, or lint rules were disabled.
