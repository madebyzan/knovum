## Organization development verification

Source commit: `a965ee8eedbe8f7eec1781cd08edaf3cc763ecfe`. Actual runner results; not a complete product acceptance.

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
src/features/spaces/__tests__/canvas-gesture-session.test.ts(74,27): error TS2339: Property 'nodes' does not exist on type '{ version: 1; structure: { version: 1; items: ({ kind: "content"; nodeId: string; id: string; parentId: string | null; order: number; locked: boolean; } | { kind: "group"; title: string; id: string; parentId: string | null; order: number; locked: boolean; } | { ...; })[]; }; layout: { ...; }; viewport: { ...; }; rel...'.
src/features/spaces/__tests__/canvas-gesture-session.test.ts(86,27): error TS2339: Property 'nodes' does not exist on type '{ version: 1; structure: { version: 1; items: ({ kind: "content"; nodeId: string; id: string; parentId: string | null; order: number; locked: boolean; } | { kind: "group"; title: string; id: string; parentId: string | null; order: number; locked: boolean; } | { ...; })[]; }; layout: { ...; }; viewport: { ...; }; rel...'.
src/features/spaces/__tests__/canvas-gesture-session.test.ts(106,27): error TS2339: Property 'nodes' does not exist on type '{ version: 1; structure: { version: 1; items: ({ kind: "content"; nodeId: string; id: string; parentId: string | null; order: number; locked: boolean; } | { kind: "group"; title: string; id: string; parentId: string | null; order: number; locked: boolean; } | { ...; })[]; }; layout: { ...; }; viewport: { ...; }; rel...'.
src/features/spaces/__tests__/canvas-gesture-session.test.ts(131,27): error TS2339: Property 'nodes' does not exist on type '{ version: 1; structure: { version: 1; items: ({ kind: "content"; nodeId: string; id: string; parentId: string | null; order: number; locked: boolean; } | { kind: "group"; title: string; id: string; parentId: string | null; order: number; locked: boolean; } | { ...; })[]; }; layout: { ...; }; viewport: { ...; }; rel...'.
src/features/spaces/__tests__/canvas-gesture-session.test.ts(131,40): error TS7031: Binding element 'frame' implicitly has an 'any' type.
src/features/spaces/__tests__/canvas-gesture-session.test.ts(136,33): error TS2339: Property 'nodes' does not exist on type '{ version: 1; structure: { version: 1; items: ({ kind: "content"; nodeId: string; id: string; parentId: string | null; order: number; locked: boolean; } | { kind: "group"; title: string; id: string; parentId: string | null; order: number; locked: boolean; } | { ...; })[]; }; layout: { ...; }; viewport: { ...; }; rel...'.
src/features/spaces/__tests__/canvas-gesture-session.test.ts(179,35): error TS2339: Property 'nodes' does not exist on type '{ version: 1; structure: { version: 1; items: ({ kind: "content"; nodeId: string; id: string; parentId: string | null; order: number; locked: boolean; } | { kind: "group"; title: string; id: string; parentId: string | null; order: number; locked: boolean; } | { ...; })[]; }; layout: { ...; }; viewport: { ...; }; rel...'.
src/features/spaces/__tests__/canvas-gesture-update.test.ts(12,3): error TS2353: Object literal may only specify known properties, and 'nodes' does not exist in type '{ version: 1; structure: { version: 1; items: ({ kind: "content"; nodeId: string; id: string; parentId: string | null; order: number; locked: boolean; } | { kind: "group"; title: string; id: string; parentId: string | null; order: number; locked: boolean; } | { ...; })[]; }; layout: { ...; }; viewport: { ...; }; rel...'.
src/features/spaces/__tests__/canvas-gesture-update.test.ts(39,46): error TS2339: Property 'nodes' does not exist on type '{ version: 1; structure: { version: 1; items: ({ kind: "content"; nodeId: string; id: string; parentId: string | null; order: number; locked: boolean; } | { kind: "group"; title: string; id: string; parentId: string | null; order: number; locked: boolean; } | { ...; })[]; }; layout: { ...; }; viewport: { ...; }; rel...'.
src/features/spaces/__tests__/canvas-gesture-update.test.ts(49,46): error TS2339: Property 'nodes' does not exist on type '{ version: 1; structure: { version: 1; items: ({ kind: "content"; nodeId: string; id: string; parentId: string | null; order: number; locked: boolean; } | { kind: "group"; title: string; id: string; parentId: string | null; order: number; locked: boolean; } | { ...; })[]; }; layout: { ...; }; viewport: { ...; }; rel...'.
src/features/spaces/__tests__/canvas-gesture-update.test.ts(52,41): error TS2339: Property 'nodes' does not exist on type '{ version: 1; structure: { version: 1; items: ({ kind: "content"; nodeId: string; id: string; parentId: string | null; order: number; locked: boolean; } | { kind: "group"; title: string; id: string; parentId: string | null; order: number; locked: boolean; } | { ...; })[]; }; layout: { ...; }; viewport: { ...; }; rel...'.
src/features/spaces/__tests__/canvas-gesture-update.test.ts(58,46): error TS2339: Property 'nodes' does not exist on type '{ version: 1; structure: { version: 1; items: ({ kind: "content"; nodeId: string; id: string; parentId: string | null; order: number; locked: boolean; } | { kind: "group"; title: string; id: string; parentId: string | null; order: number; locked: boolean; } | { ...; })[]; }; layout: { ...; }; viewport: { ...; }; rel...'.
src/features/spaces/__tests__/canvas-gesture-update.test.ts(63,21): error TS2339: Property 'nodes' does not exist on type '{ version: 1; structure: { version: 1; items: ({ kind: "content"; nodeId: string; id: string; parentId: string | null; order: number; locked: boolean; } | { kind: "group"; title: string; id: string; parentId: string | null; order: number; locked: boolean; } | { ...; })[]; }; layout: { ...; }; viewport: { ...; }; rel...'.
src/features/spaces/__tests__/canvas-movement-integration.test.ts(8,33): error TS2339: Property 'nodes' does not exist on type '{ version: 1; structure: { version: 1; items: ({ kind: "content"; nodeId: string; id: string; parentId: string | null; order: number; locked: boolean; } | { kind: "group"; title: string; id: string; parentId: string | null; order: number; locked: boolean; } | { ...; })[]; }; layout: { ...; }; viewport: { ...; }; rel...'.
src/features/spaces/__tests__/canvas-movement-integration.test.ts(8,46): error TS7031: Binding element 'nodeId' implicitly has an 'any' type.
src/features/spaces/__tests__/canvas-movement-integration.test.ts(8,54): error TS7031: Binding element 'frame' implicitly has an 'any' type.
src/features/spaces/__tests__/canvas-movement-integration.test.ts(69,59): error TS2339: Property 'nodes' does not exist on type '{ version: 1; structure: { version: 1; items: ({ kind: "content"; nodeId: string; id: string; parentId: string | null; order: number; locked: boolean; } | { kind: "group"; title: string; id: string; parentId: string | null; order: number; locked: boolean; } | { ...; })[]; }; layout: { ...; }; viewport: { ...; }; rel...'.
src/features/spaces/__tests__/canvas-movement-integration.test.ts(69,73): error TS7031: Binding element 'nodeId' implicitly has an 'any' type.
src/features/spaces/__tests__/canvas-movement-integration.test.ts(71,35): error TS2339: Property 'nodes' does not exist on type '{ version: 1; structure: { version: 1; items: ({ kind: "content"; nodeId: string; id: string; parentId: string | null; order: number; locked: boolean; } | { kind: "group"; title: string; id: string; parentId: string | null; order: number; locked: boolean; } | { ...; })[]; }; layout: { ...; }; viewport: { ...; }; rel...'.
src/features/spaces/__tests__/canvas-movement-integration.test.ts(71,49): error TS7031: Binding element 'nodeId' implicitly has an 'any' type.
src/features/spaces/__tests__/canvas-movement-lifecycle.test.ts(7,33): error TS2339: Property 'nodes' does not exist on type '{ version: 1; structure: { version: 1; items: ({ kind: "content"; nodeId: string; id: string; parentId: string | null; order: number; locked: boolean; } | { kind: "group"; title: string; id: string; parentId: string | null; order: number; locked: boolean; } | { ...; })[]; }; layout: { ...; }; viewport: { ...; }; rel...'.
src/features/spaces/__tests__/canvas-movement-lifecycle.test.ts(7,46): error TS7031: Binding element 'nodeId' implicitly has an 'any' type.
src/features/spaces/__tests__/canvas-movement-lifecycle.test.ts(7,54): error TS7031: Binding element 'frame' implicitly has an 'any' type.
src/features/spaces/__tests__/canvas-movement-lifecycle.test.ts(21,35): error TS2339: Property 'nodes' does not exist on type '{ version: 1; structure: { version: 1; items: ({ kind: "content"; nodeId: string; id: string; parentId: string | null; order: number; locked: boolean; } | { kind: "group"; title: string; id: string; parentId: string | null; order: number; locked: boolean; } | { ...; })[]; }; layout: { ...; }; viewport: { ...; }; rel...'.
src/features/spaces/__tests__/canvas-movement-lifecycle.test.ts(21,48): error TS
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
472 passed, 75 failed
/home/runner/work/knovum/knovum/src/foundation/i18n/__tests__/locale-completeness.test.ts :: locale completeness uses semantic lowerCamelCase message keys
AssertionError: expected [ …(9) ] to deeply equal []
    at /home/runner/work/knovum/knovum/src/foundation/i18n/__tests__/locale-completeness.test.ts:112:72
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2305:20
    at new Promise (<anonymous>)
    at runWithTimeout (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2272:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2955:64
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-arrangement-operation.test.ts :: whole-operation arrangement through the canvas port distributes three Section roots and carries every member by its parent's delta
TypeError: Cannot read properties of undefined (reading 'map')
    at CanvasDocumentModel.indexLayouts (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/document/canvas-document-model.ts:157:49)
    at new CanvasDocumentModel (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/document/canvas-document-model.ts:31:10)
    at Module.movementHarness (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/testing/canvas-movement-harness.ts:14:17)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-arrangement-operation.test.ts:27:21
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2305:20
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-arrangement-operation.test.ts :: whole-operation arrangement through the canvas port gives ELK only operation roots and expands its resulting movement to members
TypeError: Cannot read properties of undefined (reading 'map')
    at CanvasDocumentModel.indexLayouts (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/document/canvas-document-model.ts:157:49)
    at new CanvasDocumentModel (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/document/canvas-document-model.ts:31:10)
    at Module.movementHarness (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/testing/canvas-movement-harness.ts:14:17)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-arrangement-operation.test.ts:39:21
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2305:20
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-arrangement-operation.test.ts :: whole-operation arrangement through the canvas port does not persist a partial alignment when any selected root is locked
TypeError: Cannot read properties of undefined (reading 'map')
    at CanvasDocumentModel.indexLayouts (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/document/canvas-document-model.ts:157:49)
    at new CanvasDocumentModel (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/document/canvas-document-model.ts:31:10)
    at Module.movementHarness (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/testing/canvas-movement-harness.ts:14:17)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-arrangement-operation.test.ts:60:21
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2305:20
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-arrangement-operation.test.ts :: whole-operation arrangement through the canvas port leaves the document unchanged if automatic layout computation fails
TypeError: Cannot read properties of undefined (reading 'map')
    at CanvasDocumentModel.indexLayouts (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/document/canvas-document-model.ts:157:49)
    at new CanvasDocumentModel (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/document/canvas-document-model.ts:31:10)
    at Module.movementHarness (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/testing/canvas-movement-harness.ts:14:17)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-arrangement-operation.test.ts:68:21
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2305:20
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
TypeError: Cannot read properties of undefined (reading 'map')
    at CanvasDocumentModel.indexLayouts (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/document/canvas-document-model.ts:157:49)
    at new CanvasDocumentModel (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/document/canvas-document-model.ts:31:10)
    at movementHarness (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/testing/canvas-movement-harness.ts:14:17)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-gesture-camera.test.ts:11:23
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2027:60
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modu
```
</details>

<details><summary>Build failures</summary>

```text
$ vite build
vite v8.2.2 building client environment for production...
transforming...
✓ 4029 modules transformed.
rendering chunks...
✗ Build failed in 3.82s
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
