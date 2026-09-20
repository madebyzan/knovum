## Organization development verification

Source commit: `735f1e3f0b949d0929ceddde6cbeef5637c830d9`. Actual runner results; not a complete product acceptance.

| Check | Result |
| --- | --- |
| Types | FAIL (2) |
| Architecture | FAIL (1) |
| Frontend tests | FAIL (1) |
| Build | FAIL (1) |

<details><summary>Types failures</summary>

```text
$ vue-tsc --noEmit && tsc --noEmit -p tsconfig.node.json
src/features/spaces/__tests__/canvas-arrangement-operation.test.ts(29,54): error TS2339: Property 'nodes' does not exist on type '{ version: 1; structure: { version: 1; items: ({ kind: "content"; nodeId: string; id: string; parentId: string | null; order: number; locked: boolean; } | { kind: "group"; title: string; id: string; parentId: string | null; order: number; locked: boolean; } | { ...; })[]; }; layout: { ...; }; viewport: { ...; }; rel...'.
src/features/spaces/__tests__/canvas-arrangement-operation.test.ts(29,67): error TS7031: Binding element 'nodeId' implicitly has an 'any' type.
src/features/spaces/__tests__/canvas-arrangement-operation.test.ts(29,75): error TS7031: Binding element 'frame' implicitly has an 'any' type.
src/features/spaces/__tests__/canvas-arrangement-operation.test.ts(48,35): error TS2339: Property 'nodes' does not exist on type '{ version: 1; structure: { version: 1; items: ({ kind: "content"; nodeId: string; id: string; parentId: string | null; order: number; locked: boolean; } | { kind: "group"; title: string; id: string; parentId: string | null; order: number; locked: boolean; } | { ...; })[]; }; layout: { ...; }; viewport: { ...; }; rel...'.
src/features/spaces/__tests__/canvas-arrangement-operation.test.ts(48,48): error TS7031: Binding element 'frame' implicitly has an 'any' type.
src/features/spaces/__tests__/canvas-clipboard-paste.test.ts(110,95): error TS2493: Tuple type '[]' of length '0' has no element at index '0'.
src/features/spaces/__tests__/canvas-document-rebase.test.ts(9,3): error TS2353: Object literal may only specify known properties, and 'nodes' does not exist in type '{ version: 1; structure: { version: 1; items: ({ kind: "content"; nodeId: string; id: string; parentId: string | null; order: number; locked: boolean; } | { kind: "group"; title: string; id: string; parentId: string | null; order: number; locked: boolean; } | { ...; })[]; }; layout: { ...; }; viewport: { ...; }; rel...'.
src/features/spaces/__tests__/canvas-document-workspace.test.ts(12,5): error TS2353: Object literal may only specify known properties, and 'nodes' does not exist in type '{ version: 1; structure: { version: 1; items: ({ kind: "content"; nodeId: string; id: string; parentId: string | null; order: number; locked: boolean; } | { kind: "group"; title: string; id: string; parentId: string | null; order: number; locked: boolean; } | { ...; })[]; }; layout: { ...; }; viewport: { ...; }; rel...'.
src/features/spaces/__tests__/canvas-document-workspace.test.ts(46,25): error TS2353: Object literal may only specify known properties, and 'nodes' does not exist in type '{ version: 1; structure: { version: 1; items: ({ kind: "content"; nodeId: string; id: string; parentId: string | null; order: number; locked: boolean; } | { kind: "group"; title: string; id: string; parentId: string | null; order: number; locked: boolean; } | { ...; })[]; }; layout: { ...; }; viewport: { ...; }; rel...'.
src/features/spaces/__tests__/canvas-document-workspace.test.ts(46,41): error TS2339: Property 'nodes' does not exist on type '{ version: 1; structure: { version: 1; items: ({ kind: "content"; nodeId: string; id: string; parentId: string | null; order: number; locked: boolean; } | { kind: "group"; title: string; id: string; parentId: string | null; order: number; locked: boolean; } | { ...; })[]; }; layout: { ...; }; viewport: { ...; }; rel...'.
src/features/spaces/__tests__/canvas-document-workspace.test.ts(46,52): error TS7006: Parameter 'node' implicitly has an 'any' type.
src/features/spaces/__tests__/canvas-document-workspace.test.ts(60,31): error TS2339: Property 'nodes' does not exist on type '{ version: 1; structure: { version: 1; items: ({ kind: "content"; nodeId: string; id: string; parentId: string | null; order: number; locked: boolean; } | { kind: "group"; title: string; id: string; parentId: string | null; order: number; locked: boolean; } | { ...; })[]; }; layout: { ...; }; viewport: { ...; }; rel...'.
src/features/spaces/__tests__/canvas-document-workspace.test.ts(114,45): error TS2339: Property 'nodes' does not exist on type '{ version: 1; structure: { version: 1; items: ({ kind: "content"; nodeId: string; id: string; parentId: string | null; order: number; locked: boolean; } | { kind: "group"; title: string; id: string; parentId: string | null; order: number; locked: boolean; } | { ...; })[]; }; layout: { ...; }; viewport: { ...; }; rel...'.
src/features/spaces/__tests__/canvas-document.test.ts(58,10): error TS2339: Property 'nodes' does not exist on type '{ version: 1; structure: { version: 1; items: ({ kind: "content"; nodeId: string; id: string; parentId: string | null; order: number; locked: boolean; } | { kind: "group"; title: string; id: string; parentId: string | null; order: number; locked: boolean; } | { ...; })[]; }; layout: { ...; }; viewport: { ...; }; rel...'.
src/features/spaces/__tests__/canvas-document.test.ts(93,10): error TS2339: Property 'nodes' does not exist on type '{ version: 1; structure: { version: 1; items: ({ kind: "content"; nodeId: string; id: string; parentId: string | null; order: number; locked: boolean; } | { kind: "group"; title: string; id: string; parentId: string | null; order: number; locked: boolean; } | { ...; })[]; }; layout: { ...; }; viewport: { ...; }; rel...'.
src/features/spaces/__tests__/canvas-editor-navigation.test.ts(8,25): error TS2345: Argument of type '{ layouts: () => { nodeId: string; frame: { x: number; y: number; width: number; height: number; rotation: number; }; sizeAuthority: string; zIndex: number; }[]; selectedNodeIds: () => Set<string>; setSelection: Mock<...>; setViewport: (viewport: { x: number; y: number; zoom: number; }) => void; size: () => { ...; }...' is not assignable to parameter of type 'CanvasNavigationContext'.
  The types returned by 'layouts()' are incompatible between these types.
    Type '{ nodeId: string; frame: { x: number; y: number; width: number; height: number; rotation: number; }; sizeAuthority: string; zIndex: number; }[]' is not assignable to type 'readonly CanvasNodeLayoutV1[]'.
      Type '{ nodeId: string; frame: { x: number; y: number; width: number; height: number; rotation: number; }; sizeAuthority: string; zIndex: number; }' is not assignable to type 'CanvasNodeLayoutV1'.
        Types of property 'sizeAuthority' are incompatible.
          Type 'string' is not assignable to type 'CanvasNodeSizeAuthority'.
src/features/spaces/__tests__/canvas-editor-navigation.test.ts(15,25): error TS2345: Argument of type '{ layouts: () => { nodeId: string; frame: { x: number; y: number; width: number; height: number; rotation: number; }; sizeAuthority: string; zIndex: number; }[]; selectedNodeIds: () => Set<string>; setSelection: Mock<...>; setViewport: (viewport: { x: number; y: number; zoom: number; }) => void; size: () => { ...; }...' is not assignable to parameter of type 'CanvasNavigationContext'.
  The types returned by 'layouts()' are incompatible between these types.
    Type '{ nodeId: string; frame: { x: number; y: number; width: number; height: number; rotation: number; }; sizeAuthority: string; zIndex: number; }[]' is not assignable to type 'readonly CanvasNodeLayoutV1[]'.
      Type '{ nodeId: string; frame: { x: number; y: number; width: number; height: number; rotation: number; }; sizeAuthority: string; zIndex: number; }' is not assignable to type 'CanvasNodeLayoutV1'.
        Types of property 'sizeAuthority' are incompatible.
          Type 'string' is not assignable to type 'CanvasNodeSizeAuthority'.
src/features/spaces/__tests__/canvas-gesture-camera.test.ts(41,37): error TS2339: Property 'nodes' does not exist on type '{ version: 1; structure: { version: 1; items: ({ kind: "content"; nodeId: string; id: string; parentId: string | null; order: number; locked: boolean; } | { kind: "group"; title: string; id: string; parentId: string | null; order: number; locked: boolean; } | { ...; })[]; }; layout: { ...; }; viewport: { ...; }; rel...'.
src/features/spaces/__tests__/canvas-gesture-camera.test.ts(41,50): error TS7031: Binding element 'frame' implicitly has an 'any' type.
src/features/spaces/__tests__/canvas-gesture-camera.test.ts(72,35): error TS2339: Property 'nodes' does not exist on type '{ version: 1; structure: { version: 1; items: ({ kind: "content"; nodeId: string; id: string; parentId: string | null; order: number; locked: boolean; } | { kind: "group"; title: string; id: string; parentId: string | null; order: number; locked: boolean; } | { ...; })[]; }; layout: { ...; }; viewport: { ...; }; rel...'.
src/features/spaces/__tests__/canvas-gesture-camera.test.ts(72,48): error TS7031: Binding element 'frame' implicitly has an 'any' type.
src/features/spaces/__tests__/canvas-gesture-repository.test.ts(15,3): error TS2353: Object literal may only specify known properties, and 'nodes' does not exist in type '{ version: 1; structure: { version: 1; items: ({ kind: "content"; nodeId: string; id: string; parentId: string | null; order: number; locked: boolean; } | { kind: "group"; title: string; id: string; parentId: string | null; order: number; locked: boolean; } | { ...; })[]; }; layout: { ...; }; viewport: { ...; }; rel...'.
src/features/spaces/__tests__/canvas-gesture-repository.test.ts(25,50): error TS2554: Expected 2 arguments, but got 3.
src/features/spaces/__tests__/canvas-gesture-session.test.ts(74,27): error TS2339: Property 'nodes' does not exist on type '{ version: 1; structure: { version: 1; items: ({ kind: "content"; nodeId: string; id: string; parentId: string | null; order: number; locked: boolean; } | { kind: "group"; title: string; id: string; parentId: string | null; order: number; locked: boolean; } | { ...; })[]; }; layout: { ...; }; viewport: { ...; }; rel...'.
src/features/spaces/__tests__/canvas-gesture-session.test.ts(86,27): error TS2339: Property 'nodes' does not exist on type '{ version: 1; structure: { version: 1; items: ({ kind: "content"; nodeId: string; id: string; parentId: string | null; order: number; locked: boolean; } | { kind: "group"; title: string; id: string; parentId: string | null; order: number; locked: boolean; } | { ...; })[]; }; layout: { ...; }; viewport: { ...; }; rel...'.
src/features/spaces/__tests__/canvas-gesture-session.test.ts(106,27): error TS2339: Property 'nodes' does not exist on type '{ version: 1; structure: { version: 1; items: ({ kind: "content"; nodeId: string; id: string; parentId: string | null; order: number; locked: boolean; } | { kind: "group"; title: string; id: string; parentId: string | null; order: number; locked: boolean; } | { ...; })[]; }; layout: { ...; }; viewport: { ...; }; rel...'.
src/features/spaces/__tests__/canvas-gesture-session.test.ts(131,27): error TS2339: Property 'nodes' does not exist on type '{ version: 1; structure: { version: 1; items: ({ kind: "content"; nodeId: string; id: string; parentId: string | null; order: number; locked: boolean; } | { kind: "group"; title: string; id: string; parentId: string | null; order: number; locked: boolean; } | { ...; })[]; }; layout: { ...; }; viewport: { ...; }; rel...'.
src/features/spaces/__tests__/canvas-gesture-session.test.ts(131,40): error TS7031: Binding element 'frame' implicitly has an 'any' type.
src/features/spaces/__tests__/canvas-gesture-session.test.ts(136,33): error TS2339: Property 'nodes' does not exist on type '{ version: 1; structure: { version: 1; items: ({ kind: "content"; nodeId: string; id: string; parentId: string | null; order: number; locked: boolean; } | { kind: "group"; title: string; id: string; parentId: string | null; order: number; locked: boolean; } | { ...; })[]; }; layout: { ...; }; viewport: { ...; }; rel...'.
src/features/spaces/__tests__/canvas-gesture-session.test.ts(142,5): error TS2353: Object literal may only specify known properties, and 'documentChanged' does not exist in type 'CanvasDocumentModelEvents'.
src/features/spaces/__tests__/canvas-gesture-session.test.ts(179,35): error TS2339: Property 'nodes' does not exist on type '{ version: 1; structure: { version: 1; items: ({ kind: "content"; nodeId: string; id: string; parentId: string | null; order: number; locked: boolean; } | { kind: "group"; title: string; id: string; parentId: string | null; order: number; locked: boolean; } | { ...; })[]; }; layout: { ...; }; viewport: { ...; }; rel...'.
src/features/spaces/__tests__/canvas-gesture-update.test.ts(12,3): error TS2353: Object literal may only specify known properties, and 'nodes' does not exist in type '{ version: 1; structure: { version: 1; items: ({ kind: "content"; nodeId: string; id: string; parentId: string | null; order: number; locked: boolean; } | { kind: "group"; title: string; id: string; parentId: string | null; order: number; locked: boolean; } | { ...; })[]; }; layout: { ...; }; viewport: { ...; }; rel...'.
src/features/spaces/__tests__/canvas-gesture-update.test.ts(31,17): error TS2339: Property 'nodes' does not exist on type '{ layout: { version: 1; items: ({ kind: "group"; itemId: string; transform: { x: number; y: number; rotation: number; scale: number; }; } | { kind: "content"; itemId: string; transform: { x: number; y: number; rotation: number; scale: number; }; size: { ...; }; sizeAuthority: "content" | "frame"; fileSubpath?: strin...'.
src/features/spaces/__tests__/canvas-gesture-update.test.ts(39,46): error TS2339: Property 'nodes' does not exist on type '{ version: 1; structure: { version: 1; items: ({ kind: "content"; nodeId: string; id: string; parentId: string | null; order: number; locked: boolean; } | { kind: "group"; title: string; id: string; parentId: string | null; order: number; locked: boolean; } | { ...; })[]; }; layout: { ...; }; viewport: { ...; }; rel...'.
src/features/spaces/__tests__/canvas-gesture-update.test.ts(44,56): error TS2339: Property 'nodes' does not exist on type '{ layout: { version: 1; items: ({ kind: "group"; itemId: string; transform: { x: number; y: number; rotation: number; scale: number; }; } | { kind: "content"; itemId: string; transform: { x: number; y: number; rotation: number; scale: number; }; size: { ...; }; sizeAuthority: "content" | "frame"; fileSubpath?: strin...'.
src/features/spaces/__tests__/canvas-gesture-update.test.ts(49,46): error TS2339: Property 'nodes' does not exist on type '{ version: 1; structure: { version: 1; items: ({ kind: "content"; nodeId: string; id: string; parentId: string | null; order: number; locked: boolean; } | { kind: "group"; title: string; id: string; parentId: string | null; order: number; locked: boolean; } | { ...; })[]; }; layout: { ...; }; viewport: { ...; }; rel...'.
src/features/spaces/__tests__/canvas-gesture-update.test.ts(52,17): error TS2339: Property 'nodes' does not exist on type '{ layout: { version: 1; items: ({ kind: "group"; itemId: string; transform: { x: number; y: number; rotation: number; scale: number; }; } | { kind: "content"; itemId: string; transform: { x: number; y: number; rotation: number; scale: number; }; size: { ...; }; sizeAuthority: "content" | "frame"; fileSubpath?: strin...'.
src/features/spaces/__tests__/canvas-gesture-update.test.ts(52,41): error TS2339: Property 'nodes' does not exist on type '{ version: 1; structure: { version: 1; items: ({ kind: "content"; nodeId: string; id: string; parentId: string | null; order: number; locked: boolean; } | { kind: "group"; title: string; id: string; parentId: string | null; order: number; locked: boolean; } | { ...; })[]; }; layout: { ...; }; viewport: { ...; }; rel...'.
src/features/spaces/__tests__/canvas-gesture-update.test.ts(53,17): error TS2339: Property 'nodes' does not exist on type '{ layout: { version: 1; items: ({ kind: "group"; itemId: string; transform: { x: number; y: number; rotation: number; scale: number; }; } | { kind: "content"; itemId: string; transform: { x: number; y: number; rotation: number; scale: number; }; size: { ...; }; sizeAuthority: "content" | "frame"; fileSubpath?: strin...'.
src/features/spaces/__tests__/canvas-gesture-update.test.ts(58,46): error TS2339: Property 'nodes' does not exist on type '{ version: 1; structure: { version: 1; items: ({ kind: "content"; nodeId: string; id: string; parentId: string | null; order: number; locked: boolean; } | { kind: "group"; title: string; id: string; parentId: string | null; order: number; locked: boolean; } | { ...; })[]; }; layout: { ...; }; viewport: { ...; }; rel...'.
src/features/spaces/__tests__/canvas-gesture-update.test.ts(62,17): error TS2339: Property 'nodes' does not exist on type '{ layout: { version: 1; items: ({ kind: "group"; itemId: string; transform: { x: number; y: number; rotation: number; scale: number; }; } | { kind: "content"; itemId: string; transform: { x: number; y: number; rotation: number; scale: number; }; size: { ...; }; sizeAuthority: "content" | "frame"; fileSubpath?: strin...'.
src/features/spaces/__tests__/canvas-gesture-update.test.ts(63,21): error TS2339: Property 'nodes' does not exist on type '{ version: 1; structure: { version: 1; items: ({ kind: "content"; nodeId: string; id: string; parentId: string | null; order: number; locked: boolean; } | { kind: "group"; title: string; id: string; parentId: string | null; order: number; locked: boolean; } | { ...; })[]; }; layout: { ...; }; viewport: { ...; }; rel...'.
src/features/spaces/__tests__/canvas-movement-integration.test.ts(8,33): error TS2339: Property 'nodes' does not exist on type '{ version: 1; structure: { version: 1; items: ({ kind: "content"; nodeId: string; id: string; parentId: string | null; order: number; locked: boolean; } | { kind: "group"; title: string; id: string; parentId: string | null; order: number; locked: boolean; } | { ...; })[]; }; layout: { ...; }; viewport: { ...; }; rel...'.
src/features/spaces/__tests__/canvas-movement-integration.test.ts(8,46): error TS7031: Binding element 'nodeId' implicitly has an 'any' type.
src/features/spaces/__tests__/can
```
</details>

<details><summary>Architecture failures</summary>

```text
$ node tools/check-frontend-source-structure.mjs && node tools/check-ai-provider-catalog.mjs && depcruise --config .dependency-cruiser.cjs src
Frontend source structure gate failed (7):
- src/features/spaces/__tests__/canvas-surface-host.test.ts: 184 effective lines exceeds 180
- src/features/spaces/__tests__/konva-scene-projection.test.ts: 210 effective lines exceeds 180
- src/features/spaces/canvas/konva/use-konva-canvas-runtime.ts: 185 effective lines exceeds 180
- src/features/spaces/presentation/SpaceOutline.vue: 185 effective lines exceeds 180
- src/features/spaces/presentation/canvas/SpaceCanvas.vue: 188 effective lines exceeds 180
- src/features/spaces/presentation/knowledge/SpaceKnowledgeTypeManager.vue: 211 effective lines exceeds 180
- src/features/spaces/__tests__/capture-url-fixture.ts: files under __tests__ must use the exact .test.ts suffix
[ELIFECYCLE] Command failed with exit code 1.

```
</details>

<details><summary>Frontend tests failures</summary>

```text
464 passed, 83 failed
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-arrangement-operation.test.ts :: whole-operation arrangement through the canvas port distributes three Section roots and carries every member by its parent's delta
TypeError: Cannot read properties of undefined (reading 'filter')
    at layoutsFor (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/create-canvas-editor-port.ts:172:29)
    at /home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/create-canvas-editor-port.ts:59:48
    at /home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/interaction/canvas-gesture-session.ts:75:45
    at processTicksAndRejections (node:internal/process/task_queues:103:5)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-arrangement-operation.test.ts :: whole-operation arrangement through the canvas port gives ELK only operation roots and expands its resulting movement to members
TypeError: Cannot read properties of undefined (reading 'filter')
    at layoutsFor (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/create-canvas-editor-port.ts:172:29)
    at /home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/create-canvas-editor-port.ts:47:31
    at /home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/interaction/canvas-gesture-session.ts:75:45
    at processTicksAndRejections (node:internal/process/task_queues:103:5)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-arrangement-operation.test.ts :: whole-operation arrangement through the canvas port leaves the document unchanged if automatic layout computation fails
Error: expected [Function] to throw error matching /worker unavailable/u but got 'Cannot read properties of undefined (…'
    at _Assertion.__VITEST_REJECTS__ (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+expect@4.1.10/node_modules/@vitest/expect/dist/index.js:1747:17)
    at _Assertion.propertyGetter (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/chai@6.2.2/node_modules/chai/index.js:1570:27)
    at Reflect.get (<anonymous>)
    at Object.proxyGetter [as get] (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/chai@6.2.2/node_modules/chai/index.js:1659:22)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-arrangement-operation.test.ts:71:59
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-document-rebase.test.ts :: canvas document rebase merges independent relation presentation changes by relation identity
TypeError: Cannot read properties of undefined (reading 'items')
    at rebaseCanvasDocument (/home/runner/work/knovum/knovum/src/features/spaces/canvas/host/canvas-document-rebase.ts:16:24)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-document-rebase.test.ts:33:12
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2305:20
    at new Promise (<anonymous>)
    at runWithTimeout (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2272:10)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-document-workspace.test.ts :: Space-owned document workspace persists the captured camera at the committed revision even after detachment
TypeError: Cannot read properties of undefined (reading 'items')
    at rebaseCanvasDocument (/home/runner/work/knovum/knovum/src/features/spaces/canvas/host/canvas-document-rebase.ts:16:24)
    at /home/runner/work/knovum/knovum/src/features/spaces/canvas/host/canvas-document-workspace.ts:61:13
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-document.test.ts :: CanvasDocument v1 accepts engine-neutral layout state without domain content
ZodError: [
  {
    "expected": "object",
    "code": "invalid_type",
    "path": [
      "structure"
    ],
    "message": "Invalid input: expected object, received undefined"
  },
  {
    "expected": "object",
    "code": "invalid_type",
    "path": [
      "layout"
    ],
    "message": "Invalid input: expected object, received undefined"
  },
  {
    "code": "unrecognized_keys",
    "keys": [
      "nodes"
    ],
    "path": [],
    "message": "Unrecognized key: \"nodes\""
  }
]
    at parseSpaceEditorDocument (/home/runner/work/knovum/knovum/src/features/spaces/canvas/contract/canvas-document.ts:80:36)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-document.test.ts:30:12
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2305:20
    at new Promise (<anonymous>)
    at runWithTimeout (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2272:10)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-document.test.ts :: CanvasDocument v1 rejects duplicate nodes
AssertionError: expected [Function] to throw error matching /unique/u but got '[\n  {\n    "expected": "object",\n  …'
    at Proxy.<anonymous> (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+expect@4.1.10/node_modules/@vitest/expect/dist/index.js:1552:16)
    at Proxy.<anonymous> (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+expect@4.1.10/node_modules/@vitest/expect/dist/index.js:1156:15)
    at Proxy.methodWrapper (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/chai@6.2.2/node_modules/chai/index.js:1700:25)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-document.test.ts:45:84
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2305:20
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-document.test.ts :: CanvasDocument v1 uses the same unsigned 32-bit layer range as the Rust contract
ZodError: [
  {
    "expected": "object",
    "code": "invalid_type",
    "path": [
      "structure"
    ],
    "message": "Invalid input: expected object, received undefined"
  },
  {
    "expected": "object",
    "code": "invalid_type",
    "path": [
      "layout"
    ],
    "message": "Invalid input: expected object, received undefined"
  },
  {
    "code": "unrecognized_keys",
    "keys": [
      "nodes"
    ],
    "path": [],
    "message": "Unrecognized key: \"nodes\""
  }
]
    at parseSpaceEditorDocument (/home/runner/work/knovum/knovum/src/features/spaces/canvas/contract/canvas-document.ts:80:36)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-document.test.ts:55:7
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2305:20
    at new Promise (<anonymous>)
    at runWithTimeout (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2272:10)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-document.test.ts :: CanvasDocument v1 accepts only the current semantic appearance contract
ZodError: [
  {
    "expected": "object",
    "code": "invalid_type",
    "path": [
      "structure"
    ],
    "message": "Invalid input: expected object, received undefined"
  },
  {
    "expected": "object",
    "code": "invalid_type",
    "path": [
      "layout"
    ],
    "message": "Invalid input: expected object, received undefined"
  },
  {
    "code": "unrecognized_keys",
    "keys": [
      "nodes"
    ],
    "path": [],
    "message": "Unrecognized key: \"nodes\""
  }
]
    at parseSpaceEditorDocument (/home/runner/work/knovum/knovum/src/features/spaces/canvas/contract/canvas-document.ts:80:36)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-document.test.ts:79:7
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2305:20
    at new Promise (<anonymous>)
    at runWithTimeout (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2272:10)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-document.test.ts :: CanvasDocument v1 stores only non-default relation presentation overrides
ZodError: [
  {
    "expected": "object",
    "code": "invalid_type",
    "path": [
      "structure"
    ],
    "message": "Invalid input: expected object, received undefined"
  },
  {
    "expected": "object",
    "code": "invalid_type",
    "path": [
      "layout"
    ],
    "message": "Invalid input: expected object, received undefined"
  },
  {
    "code": "unrecognized_keys",
    "keys": [
      "nodes"
    ],
    "path": [],
    "message": "Unrecognized key: \"nodes\""
  }
]
    at parseSpaceEditorDocument (/home/runner/work/knovum/knovum/src/features/spaces/canvas/contract/canvas-document.ts:80:36)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-document.test.ts:144:7
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2305:20
    at new Promise (<anonymous>)
    at runWithTimeout (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2272:10)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-gesture-camera.test.ts :: committing geometry while the camera remains live keeps camera autosave free of uncommitted geometry (rejected=false)
AssertionError: expected "vi.fn()" to be called once, but got 0 times
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-gesture-camera.test.ts:26:46
    at processTicksAndRejections (node:internal/process/task_queues:103:5)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-gesture-camera.test.ts :: committing geometry while the camera remains live keeps camera autosave free of uncommitted geometry (rejected=true)
AssertionError: expected "vi.fn()" to be called once, but got 0 times
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-gesture-camera.test.ts:26:46
    at processTicksAndRejections (node:internal/process/task_queues:103:5)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-gesture-camera.test.ts :: committing geometry while the camera remains live preserves camera changes while undo is committing
TypeError: Cannot read properties of undefined (reading 'map')
    at commitNudge (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/interaction/canvas-gesture-session.ts:81:34)
    at /home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/interaction/canvas-gesture-session.ts:69:26
    at processTicksAndRejections (node:internal/process/task_queues:103:5)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-gesture-camera.test.ts :: committing geometry while the camera remains live preserves camera changes while redo is committing
TypeError: Cannot read properties of undefined (reading 'map')
    at commitNudge (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/interaction/canvas-gesture-session.ts:81:34)
    at /home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/interaction/canvas-gesture-session.ts:69:26
    at processTicksAndRejections (node:internal/process/task_queues:103:5)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-gesture-repository.test.ts :: canvas gesture repository constructs the strict persistence DTO without leaking gesture-only fields
AssertionError: expected "vi.fn()" to be called with arguments: [ …(2) ]

Received:

  1st vi.fn() call:

@@ -1,11 +1,10 @@
  [
    "plugin:spaces|commit_canvas_gesture",
    {
      "request": {
        "baseRevision": 3,
-       "changes": [],
        "document": {
          "nodes": [],
          "version": 1,
          "viewport": {
            "x": 0,


Number of calls: 1

    at Proxy.<anonymous> (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+expect@4.1.10/node_modules/@vitest/expect/dist/index.js:1487:10)
    at Proxy.<anonymous> (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+expect@4.1.10/node_modules/@vitest/expect/dist/index.js:1156:15)
    at Proxy.methodWrapper (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/chai@6.2.2/node_modules/chai/index.js:1700:25)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-gesture-repository.test.ts:27:26
    at processTicksAndRejections (node:internal/process/task_queues:103:5)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-gesture-session.test.ts :: canvas gesture session commits keyboard movement without implicitly changing Section membership
TypeError: Cannot read properties of undefined (reading 'map')
    at commitNudge (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/interaction/canvas-gesture-session.ts:81:34)
    at /home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/interaction/canvas-gesture-session.ts:69:26
    at processTicksAndRejections (node:internal/process/task_queues:103:5)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-gesture-session.test.ts :: canvas gesture session serializes rapid keyboard nudges from
```
</details>

<details><summary>Build failures</summary>

```text
$ vite build
vite v8.2.2 building client environment for production...
transforming...
✓ 4048 modules transformed.
rendering chunks...
✗ Build failed in 2.56s
error during build:
Build failed with 1 error:

[MISSING_EXPORT] "reconcileCanvasDocument" is not exported by "src/features/spaces/canvas/document/canvas-document-sync.ts".
   ╭─[ src/features/spaces/canvas/host/use-canvas-document-session.ts:2:32 ]
   │
 2 │ import { createCanvasDocument, reconcileCanvasDocument } from "../document/canvas-document-sync";
   │                                ───────────┬───────────  
   │                                           ╰───────────── Missing export
───╯

    at aggregateBindingErrorsIntoJsError (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/rolldown@1.2.7/node_modules/rolldown/dist/shared/error-BP8lJHle.mjs:48:18)
    at unwrapBindingResult (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/rolldown@1.2.7/node_modules/rolldown/dist/shared/error-BP8lJHle.mjs:18:128)
    at #build (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/rolldown@1.2.7/node_modules/rolldown/dist/shared/rolldown-DfXO5K1m.mjs:133:34)
    at async buildEnvironment (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/vite@8.2.2_@types+node@26.4.1_esbuild@0.28.2_jiti@2.7.0_tsx@4.23.12_yaml@2.9.0/node_modules/vite/dist/node/chunks/node.js:33821:66)
    at async Object.build (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/vite@8.2.2_@types+node@26.4.1_esbuild@0.28.2_jiti@2.7.0_tsx@4.23.12_yaml@2.9.0/node_modules/vite/dist/node/chunks/node.js:34242:19)
    at async Object.buildApp (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/vite@8.2.2_@types+node@26.4.1_esbuild@0.28.2_jiti@2.7.0_tsx@4.23.12_yaml@2.9.0/node_modules/vite/dist/node/chunks/node.js:34239:153)
    at async CAC.<anonymous> (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/vite@8.2.2_@types+node@26.4.1_esbuild@0.28.2_jiti@2.7.0_tsx@4.23.12_yaml@2.9.0/node_modules/vite/dist/node/cli.js:776:3) {
  errors: [Getter/Setter]
}
[ELIFECYCLE] Command failed with exit code 1.

```
</details>

Complete logs and test JSON are attached to this workflow run. No gates, tests, or lint rules were disabled.
