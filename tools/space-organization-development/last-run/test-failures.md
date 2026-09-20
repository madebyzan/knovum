# Complete failing test diagnostics

```text
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-arrangement-operation.test.ts :: whole-operation arrangement through the canvas port distributes three Section roots and carries every member by its parent's delta
TypeError: Cannot destructure property 'width' of '(0 , __vite_ssr_import_1__.spaceNodeSizePolicy)(...)' as it is undefined.
    at /home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
    at Array.map (<anonymous>)
    at appendEditorContent (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:35:18)
    at createCanvasDocument (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:50:10)
    at Module.movementHarness (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/testing/canvas-movement-harness.ts:14:41)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-arrangement-operation.test.ts:27:21
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-arrangement-operation.test.ts :: whole-operation arrangement through the canvas port gives ELK only operation roots and expands its resulting movement to members
TypeError: Cannot destructure property 'width' of '(0 , __vite_ssr_import_1__.spaceNodeSizePolicy)(...)' as it is undefined.
    at /home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
    at Array.map (<anonymous>)
    at appendEditorContent (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:35:18)
    at createCanvasDocument (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:50:10)
    at Module.movementHarness (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/testing/canvas-movement-harness.ts:14:41)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-arrangement-operation.test.ts:39:21
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-arrangement-operation.test.ts :: whole-operation arrangement through the canvas port does not persist a partial alignment when any selected root is locked
TypeError: Cannot destructure property 'width' of '(0 , __vite_ssr_import_1__.spaceNodeSizePolicy)(...)' as it is undefined.
    at /home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
    at Array.map (<anonymous>)
    at appendEditorContent (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:35:18)
    at createCanvasDocument (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:50:10)
    at Module.movementHarness (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/testing/canvas-movement-harness.ts:14:41)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-arrangement-operation.test.ts:60:21
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-arrangement-operation.test.ts :: whole-operation arrangement through the canvas port leaves the document unchanged if automatic layout computation fails
TypeError: Cannot destructure property 'width' of '(0 , __vite_ssr_import_1__.spaceNodeSizePolicy)(...)' as it is undefined.
    at /home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
    at Array.map (<anonymous>)
    at appendEditorContent (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:35:18)
    at createCanvasDocument (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:50:10)
    at Module.movementHarness (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/testing/canvas-movement-harness.ts:14:41)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-arrangement-operation.test.ts:68:21
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
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
    at parseSpaceEditorDocument (/home/runner/work/knovum/knovum/src/features/spaces/canvas/contract/canvas-document.ts:79:36)
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
    at parseSpaceEditorDocument (/home/runner/work/knovum/knovum/src/features/spaces/canvas/contract/canvas-document.ts:79:36)
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
    at parseSpaceEditorDocument (/home/runner/work/knovum/knovum/src/features/spaces/canvas/contract/canvas-document.ts:79:36)
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
    at parseSpaceEditorDocument (/home/runner/work/knovum/knovum/src/features/spaces/canvas/contract/canvas-document.ts:79:36)
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
TypeError: Cannot destructure property 'width' of '(0 , __vite_ssr_import_1__.spaceNodeSizePolicy)(...)' as it is undefined.
    at /home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
    at Array.map (<anonymous>)
    at appendEditorContent (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:35:18)
    at createCanvasDocument (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:50:10)
    at movementHarness (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/testing/canvas-movement-harness.ts:14:41)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-gesture-camera.test.ts:11:23
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2027:60
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-gesture-camera.test.ts :: committing geometry while the camera remains live keeps camera autosave free of uncommitted geometry (rejected=true)
TypeError: Cannot destructure property 'width' of '(0 , __vite_ssr_import_1__.spaceNodeSizePolicy)(...)' as it is undefined.
    at /home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
    at Array.map (<anonymous>)
    at appendEditorContent (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:35:18)
    at createCanvasDocument (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:50:10)
    at movementHarness (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/testing/canvas-movement-harness.ts:14:41)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-gesture-camera.test.ts:11:23
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2027:60
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-gesture-camera.test.ts :: committing geometry while the camera remains live preserves camera changes while undo is committing
TypeError: Cannot destructure property 'width' of '(0 , __vite_ssr_import_1__.spaceNodeSizePolicy)(...)' as it is undefined.
    at /home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
    at Array.map (<anonymous>)
    at appendEditorContent (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:35:18)
    at createCanvasDocument (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:50:10)
    at movementHarness (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/testing/canvas-movement-harness.ts:14:41)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-gesture-camera.test.ts:51:21
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2027:60
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-gesture-camera.test.ts :: committing geometry while the camera remains live preserves camera changes while redo is committing
TypeError: Cannot destructure property 'width' of '(0 , __vite_ssr_import_1__.spaceNodeSizePolicy)(...)' as it is undefined.
    at /home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
    at Array.map (<anonymous>)
    at appendEditorContent (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:35:18)
    at createCanvasDocument (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:50:10)
    at movementHarness (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/testing/canvas-movement-harness.ts:14:41)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-gesture-camera.test.ts:51:21
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2027:60
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
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
TypeError: Cannot destructure property 'width' of '(0 , __vite_ssr_import_1__.spaceNodeSizePolicy)(...)' as it is undefined.
    at /home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
    at Array.map (<anonymous>)
    at appendEditorContent (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:35:18)
    at createCanvasDocument (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:50:10)
    at harness (/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-gesture-session.test.ts:141:41)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-gesture-session.test.ts:29:31
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-gesture-session.test.ts :: canvas gesture session serializes rapid keyboard nudges from the latest committed document
AssertionError: expected "vi.fn()" to be called 1 times, but got 0 times
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-gesture-session.test.ts:62:36
    at processTicksAndRejections (node:internal/process/task_queues:103:5)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-gesture-session.test.ts :: canvas gesture session calculates queued frame mutations from the latest committed document
TypeError: Cannot read properties of undefined (reading '0')
    at movedFrame (/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-gesture-session.test.ts:136:33)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-gesture-session.test.ts:82:51
    at Object.runFrameMutation (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/interaction/canvas-gesture-session.ts:107:27)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-gesture-session.test.ts:82:27
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2305:20
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-gesture-session.test.ts :: canvas gesture session keeps geometry interaction busy while an asynchronous frame calculation is pending
AssertionError: expected false to be true // Object.is equality
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-gesture-session.test.ts:100:32
    at processTicksAndRejections (node:internal/process/task_queues:103:5)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-gesture-session.test.ts :: canvas gesture session keeps a queued nudge bound to the selection that received the command
TypeError: this.events.commitDocument is not a function
    at /home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/document/canvas-document-model.ts:78:40
    at processTicksAndRejections (node:internal/process/task_queues:103:5)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-gesture-update.test.ts :: canvas gesture layout updates commits frame and typography scale as one layout mutation
TypeError: Cannot read properties of undefined (reading 'items')
    at indexStructure (/home/runner/work/knovum/knovum/src/features/spaces/model/structure/structure-index.ts:11:32)
    at indexStructureSnapshot (/home/runner/work/knovum/knovum/src/features/spaces/organization/structure/structure-context.ts:11:16)
    at applyEditorFrames (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/editor-frame-edit.ts:23:19)
    at applyCanvasGestureUpdates (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/document/canvas-document-mutations.ts:16:18)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-gesture-update.test.ts:24:18
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-gesture-update.test.ts :: canvas gesture layout updates captures absence explicitly so undo removes gesture-created appearance
TypeError: Cannot read properties of undefined (reading 'items')
    at indexStructure (/home/runner/work/knovum/knovum/src/features/spaces/model/structure/structure-index.ts:11:32)
    at indexStructureSnapshot (/home/runner/work/knovum/knovum/src/features/spaces/organization/structure/structure-context.ts:11:16)
    at projectEditorLayouts (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/editor-layout-projection.ts:9:19)
    at captureCanvasGestureLayouts (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/document/canvas-document-mutations.ts:36:10)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-gesture-update.test.ts:38:22
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-gesture-update.test.ts :: canvas gesture layout updates preserves unchanged records without inventing absent appearance fields
TypeError: Cannot read properties of undefined (reading 'items')
    at indexStructure (/home/runner/work/knovum/knovum/src/features/spaces/model/structure/structure-index.ts:11:32)
    at indexStructureSnapshot (/home/runner/work/knovum/knovum/src/features/spaces/organization/structure/structure-context.ts:11:16)
    at applyEditorFrames (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/editor-frame-edit.ts:23:19)
    at applyCanvasGestureUpdates (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/document/canvas-document-mutations.ts:16:18)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-gesture-update.test.ts:48:18
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-gesture-update.test.ts :: canvas gesture layout updates restores absence as the exact original layout shape without mutating the source
TypeError: Cannot read properties of undefined (reading 'items')
    at indexStructure (/home/runner/work/knovum/knovum/src/features/spaces/model/structure/structure-index.ts:11:32)
    at indexStructureSnapshot (/home/runner/work/knovum/knovum/src/features/spaces/organization/structure/structure-context.ts:11:16)
    at projectEditorLayouts (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/editor-layout-projection.ts:9:19)
    at captureCanvasGestureLayouts (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/document/canvas-document-mutations.ts:36:10)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-gesture-update.test.ts:57:22
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-movement-integration.test.ts :: wired movement and persistence semantics aligns two Sections and both member sets through the real editor port
TypeError: Cannot destructure property 'width' of '(0 , __vite_ssr_import_1__.spaceNodeSizePolicy)(...)' as it is undefined.
    at /home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
    at Array.map (<anonymous>)
    at appendEditorContent (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:35:18)
    at createCanvasDocument (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:50:10)
    at movementHarness (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/testing/canvas-movement-harness.ts:14:41)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-movement-integration.test.ts:13:21
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-movement-integration.test.ts :: wired movement and persistence semantics does not realign a selected member separately from its Section
TypeError: Cannot destructure property 'width' of '(0 , __vite_ssr_import_1__.spaceNodeSizePolicy)(...)' as it is undefined.
    at /home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
    at Array.map (<anonymous>)
    at appendEditorContent (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:35:18)
    at createCanvasDocument (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:50:10)
    at movementHarness (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/testing/canvas-movement-harness.ts:14:41)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-movement-integration.test.ts:25:21
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-movement-integration.test.ts :: wired movement and persistence semantics moves multiple Sections with keyboard input and does not rewrite memberships
TypeError: Cannot destructure property 'width' of '(0 , __vite_ssr_import_1__.spaceNodeSizePolicy)(...)' as it is undefined.
    at /home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
    at Array.map (<anonymous>)
    at appendEditorContent (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:35:18)
    at createCanvasDocument (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:50:10)
    at movementHarness (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/testing/canvas-movement-harness.ts:14:41)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-movement-integration.test.ts:35:21
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-movement-integration.test.ts :: wired movement and persistence semantics keeps locked descendants attached to an unlocked Section for keyboard input
TypeError: Cannot destructure property 'width' of '(0 , __vite_ssr_import_1__.spaceNodeSizePolicy)(...)' as it is undefined.
    at /home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
    at Array.map (<anonymous>)
    at appendEditorContent (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:35:18)
    at createCanvasDocument (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:50:10)
    at movementHarness (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/testing/canvas-movement-harness.ts:14:41)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-movement-integration.test.ts:48:21
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-movement-integration.test.ts :: wired movement and persistence semantics rejects an entire nudge against a locked operation root
TypeError: Cannot destructure property 'width' of '(0 , __vite_ssr_import_1__.spaceNodeSizePolicy)(...)' as it is undefined.
    at /home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
    at Array.map (<anonymous>)
    at appendEditorContent (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:35:18)
    at createCanvasDocument (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:50:10)
    at movementHarness (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/testing/canvas-movement-harness.ts:14:41)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-movement-integration.test.ts:60:21
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-movement-integration.test.ts :: wired movement and persistence semantics keeps fit separate from moving a Section and does not adopt overlapping content
TypeError: Cannot destructure property 'width' of '(0 , __vite_ssr_import_1__.spaceNodeSizePolicy)(...)' as it is undefined.
    at /home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
    at Array.map (<anonymous>)
    at appendEditorContent (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:35:18)
    at createCanvasDocument (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:50:10)
    at movementHarness (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/testing/canvas-movement-harness.ts:14:41)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-movement-integration.test.ts:68:21
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-movement-integration.test.ts :: wired movement and persistence semantics never sends an optimistic gesture or a rejected gesture to the independent autosave path
TypeError: Cannot destructure property 'width' of '(0 , __vite_ssr_import_1__.spaceNodeSizePolicy)(...)' as it is undefined.
    at /home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
    at Array.map (<anonymous>)
    at appendEditorContent (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:35:18)
    at createCanvasDocument (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:50:10)
    at movementHarness (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/testing/canvas-movement-harness.ts:14:41)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-movement-integration.test.ts:75:21
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-movement-integration.test.ts :: wired movement and persistence semantics records and undoes the entire aligned operation while preserving a newer camera
TypeError: Cannot destructure property 'width' of '(0 , __vite_ssr_import_1__.spaceNodeSizePolicy)(...)' as it is undefined.
    at /home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
    at Array.map (<anonymous>)
    at appendEditorContent (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:35:18)
    at createCanvasDocument (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:50:10)
    at movementHarness (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/testing/canvas-movement-harness.ts:14:41)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-movement-integration.test.ts:85:21
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-movement-lifecycle.test.ts :: movement gesture completion commits multiple dragged Sections without separately reparenting their members
TypeError: Cannot destructure property 'width' of '(0 , __vite_ssr_import_1__.spaceNodeSizePolicy)(...)' as it is undefined.
    at /home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
    at Array.map (<anonymous>)
    at appendEditorContent (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:35:18)
    at createCanvasDocument (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:50:10)
    at movementHarness (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/testing/canvas-movement-harness.ts:14:41)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-movement-lifecycle.test.ts:15:21
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-movement-lifecycle.test.ts :: movement gesture completion still permits an explicit content drag to leave its Section
TypeError: Cannot destructure property 'width' of '(0 , __vite_ssr_import_1__.spaceNodeSizePolicy)(...)' as it is undefined.
    at /home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
    at Array.map (<anonymous>)
    at appendEditorContent (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:35:18)
    at createCanvasDocument (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:50:10)
    at movementHarness (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/testing/canvas-movement-harness.ts:14:41)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-movement-lifecycle.test.ts:27:21
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-movement-lifecycle.test.ts :: movement gesture completion does not change membership when a resize moves a content center outside its Section
TypeError: Cannot destructure property 'width' of '(0 , __vite_ssr_import_1__.spaceNodeSizePolicy)(...)' as it is undefined.
    at /home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
    at Array.map (<anonymous>)
    at appendEditorContent (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:35:18)
    at createCanvasDocument (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:50:10)
    at movementHarness (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/testing/canvas-movement-harness.ts:14:41)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-movement-lifecycle.test.ts:40:21
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-movement-lifecycle.test.ts :: movement gesture completion does not record an empty selection or a zero-offset movement
TypeError: Cannot destructure property 'width' of '(0 , __vite_ssr_import_1__.spaceNodeSizePolicy)(...)' as it is undefined.
    at /home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
    at Array.map (<anonymous>)
    at appendEditorContent (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:35:18)
    at createCanvasDocument (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:50:10)
    at movementHarness (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/testing/canvas-movement-harness.ts:14:41)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-movement-lifecycle.test.ts:50:21
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-movement-lifecycle.test.ts :: movement gesture completion rejects a missing descendant layout before any persisted keyboard movement
TypeError: Cannot destructure property 'width' of '(0 , __vite_ssr_import_1__.spaceNodeSizePolicy)(...)' as it is undefined.
    at /home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
    at Array.map (<anonymous>)
    at appendEditorContent (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:35:18)
    at createCanvasDocument (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:50:10)
    at movementHarness (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/testing/canvas-movement-harness.ts:14:41)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-movement-lifecycle.test.ts:60:21
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-movement.test.ts :: container movement plans captures all selected Section members when s1 starts the drag
TypeError: Cannot destructure property 'width' of '(0 , __vite_ssr_import_1__.spaceNodeSizePolicy)(...)' as it is undefined.
    at /home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
    at Array.map (<anonymous>)
    at appendEditorContent (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:35:18)
    at createCanvasDocument (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:50:10)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-movement.test.ts:11:21
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2027:60
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-movement.test.ts :: container movement plans captures all selected Section members when s2 starts the drag
TypeError: Cannot destructure property 'width' of '(0 , __vite_ssr_import_1__.spaceNodeSizePolicy)(...)' as it is undefined.
    at /home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
    at Array.map (<anonymous>)
    at appendEditorContent (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:35:18)
    at createCanvasDocument (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:50:10)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-movement.test.ts:11:21
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2027:60
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-movement.test.ts :: container movement plans copies every captured frame and retains locked descendants
TypeError: Cannot destructure property 'width' of '(0 , __vite_ssr_import_1__.spaceNodeSizePolicy)(...)' as it is undefined.
    at /home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
    at Array.map (<anonymous>)
    at appendEditorContent (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:35:18)
    at createCanvasDocument (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:50:10)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-movement.test.ts:25:21
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-movement.test.ts :: container movement plans rejects capture rather than skipping a locked selected root
TypeError: Cannot destructure property 'width' of '(0 , __vite_ssr_import_1__.spaceNodeSizePolicy)(...)' as it is undefined.
    at /home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
    at Array.map (<anonymous>)
    at appendEditorContent (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:35:18)
    at createCanvasDocument (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:50:10)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-movement.test.ts:42:18
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-movement.test.ts :: container movement plans applies a different offset to each whole root for alignment/distribution
TypeError: Cannot destructure property 'width' of '(0 , __vite_ssr_import_1__.spaceNodeSizePolicy)(...)' as it is undefined.
    at /home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
    at Array.map (<anonymous>)
    at appendEditorContent (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:35:18)
    at createCanvasDocument (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:50:10)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-movement.test.ts:51:21
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-movement.test.ts :: container movement plans moves a selected child only once when its Section is also selected
TypeError: Cannot destructure property 'width' of '(0 , __vite_ssr_import_1__.spaceNodeSizePolicy)(...)' as it is undefined.
    at /home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
    at Array.map (<anonymous>)
    at appendEditorContent (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:35:18)
    at createCanvasDocument (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:50:10)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-movement.test.ts:62:21
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-movement.test.ts :: container movement plans does not interpret resizing as a movement
TypeError: Cannot destructure property 'width' of '(0 , __vite_ssr_import_1__.spaceNodeSizePolicy)(...)' as it is undefined.
    at /home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
    at Array.map (<anonymous>)
    at appendEditorContent (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:35:18)
    at createCanvasDocument (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:50:10)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-movement.test.ts:71:21
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-movement.test.ts :: container movement plans rejects missing descendant geometry instead of silently leaving it behind
TypeError: Cannot destructure property 'width' of '(0 , __vite_ssr_import_1__.spaceNodeSizePolicy)(...)' as it is undefined.
    at /home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
    at Array.map (<anonymous>)
    at appendEditorContent (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:35:18)
    at createCanvasDocument (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:50:10)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-movement.test.ts:81:21
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-overview-geometry.test.ts :: canvas overview geometry fits the whole document independently of editor viewport
TypeError: Cannot read properties of undefined (reading 'items')
    at indexStructure (/home/runner/work/knovum/knovum/src/features/spaces/model/structure/structure-index.ts:11:32)
    at indexStructureSnapshot (/home/runner/work/knovum/knovum/src/features/spaces/organization/structure/structure-context.ts:11:16)
    at projectCanvasScene (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/scene/scene-projection.ts:36:19)
    at sceneAt (/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-overview-geometry.test.ts:21:10)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-overview-geometry.test.ts:37:19
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-overview-geometry.test.ts :: canvas overview geometry includes rotated node corners
TypeError: Cannot read properties of undefined (reading 'items')
    at indexStructure (/home/runner/work/knovum/knovum/src/features/spaces/model/structure/structure-index.ts:11:32)
    at indexStructureSnapshot (/home/runner/work/knovum/knovum/src/features/spaces/organization/structure/structure-context.ts:11:16)
    at projectCanvasScene (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/scene/scene-projection.ts:36:19)
    at sceneAt (/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-overview-geometry.test.ts:21:10)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-overview-geometry.test.ts:42:45
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-overview-geometry.test.ts :: canvas overview geometry does not depend on a mounted viewport when empty
TypeError: Cannot read properties of undefined (reading 'items')
    at indexStructure (/home/runner/work/knovum/knovum/src/features/spaces/model/structure/structure-index.ts:11:32)
    at indexStructureSnapshot (/home/runner/work/knovum/knovum/src/features/spaces/organization/structure/structure-context.ts:11:16)
    at projectCanvasScene (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/scene/scene-projection.ts:36:19)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-overview-geometry.test.ts:49:19
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2305:20
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-package-codec.test.ts :: Canvas package codec remaps node references in the neutral document
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
    at parseSpaceEditorDocument (/home/runner/work/knovum/knovum/src/features/spaces/canvas/contract/canvas-document.ts:79:36)
    at Object.remapCanvasState [as remap] (/home/runner/work/knovum/knovum/src/features/spaces/canvas/canvas-package-codec.ts:14:20)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-package-codec.test.ts:7:38
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2305:20
    at new Promise (<anonymous>)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-section-gesture.test.ts :: canvas section gestures moves section members and preserves a newer viewport on undo
TypeError: Cannot destructure property 'width' of '(0 , __vite_ssr_import_1__.spaceNodeSizePolicy)(...)' as it is undefined.
    at /home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
    at Array.map (<anonymous>)
    at appendEditorContent (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:35:18)
    at createCanvasDocument (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:50:10)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-section-gesture.test.ts:19:43
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
Failed to resolve import "../canvas/konva/scene/section-membership" from "src/features/spaces/__tests__/canvas-section-membership.test.ts". Does the file exist?
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-surface-host.test.ts :: CanvasSurfaceHost does not install a stale load after the active space changes
TypeError: Cannot read properties of null (reading 'shapeFlag')
    at getRootNodes (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:7497:15)
    at getRootNodes (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:7502:16)
    at getRootNodes (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:7502:16)
    at VueWrapper.getRootNodes (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:7732:16)
    at VueWrapper.findAllDOMElements (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:7209:39)
    at VueWrapper.findAll (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:7744:21)
    at VueWrapper.find (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:7238:31)
    at VueWrapper.get (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:7320:29)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-surface-host.test.ts:62:20
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:20
TypeError: Cannot destructure property 'type' of 'vnode' as it is null.
    at unmount (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6703:7)
    at unmountComponent (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6845:7)
    at unmount (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6736:7)
    at unmountComponent (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6845:7)
    at unmount (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6736:7)
    at unmountComponent (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6845:7)
    at unmount (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6736:7)
    at render (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6878:9)
    at Object.unmount (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:4367:11)
    at VueWrapper.unmount (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:7857:20)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-surface-host.test.ts :: CanvasSurfaceHost rejects an in-flight commit after the active space changes
TypeError: Cannot read properties of null (reading 'shapeFlag')
    at getRootNodes (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:7497:15)
    at getRootNodes (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:7502:16)
    at VueWrapper.getRootNodes (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:7732:16)
    at VueWrapper.attachNativeEventListener (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:7759:35)
    at new VueWrapper (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:7703:14)
    at Object.<anonymous> (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:7860:64)
    at createVueWrapper (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:1553:82)
    at /home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:7278:15
    at Array.map (<anonymous>)
    at VueWrapper.findAllComponents (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:7277:24)
TypeError: Cannot destructure property 'type' of 'vnode' as it is null.
    at unmount (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6703:7)
    at unmountComponent (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6845:7)
    at unmount (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6736:7)
    at unmountComponent (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6845:7)
    at unmount (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6736:7)
    at unmountComponent (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6845:7)
    at unmount (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6736:7)
    at render (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6878:9)
    at Object.unmount (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:4367:11)
    at VueWrapper.unmount (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:7857:20)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-surface-host.test.ts :: CanvasSurfaceHost rejects an in-flight commit after unmount
Error: Cannot call props on an empty VueWrapper.
    at Object.get (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:1479:27)
    at deleteEmptySelection (/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-surface-host.test.ts:198:51)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-surface-host.test.ts:96:25
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:20
TypeError: Cannot destructure property 'type' of 'vnode' as it is null.
    at unmount (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6703:7)
    at unmountComponent (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6845:7)
    at unmount (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6736:7)
    at unmountComponent (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6845:7)
    at unmount (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6736:7)
    at unmountComponent (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6845:7)
    at unmount (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6736:7)
    at render (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6878:9)
    at Object.unmount (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:4367:11)
    at VueWrapper.unmount (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:7857:20)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-surface-host.test.ts :: CanvasSurfaceHost preserves an in-flight viewport change without replacing committed node frames
Error: Cannot call props on an empty VueWrapper.
    at Object.get (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:1479:27)
    at deleteEmptySelection (/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-surface-host.test.ts:198:51)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-surface-host.test.ts:115:25
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:20
TypeError: Cannot destructure property 'type' of 'vnode' as it is null.
    at unmount (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6703:7)
    at unmountComponent (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6845:7)
    at unmount (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6736:7)
    at unmountComponent (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6845:7)
    at unmount (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6736:7)
    at unmountComponent (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6845:7)
    at unmount (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6736:7)
    at render (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6878:9)
    at Object.unmount (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:4367:11)
    at VueWrapper.unmount (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:7857:20)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-surface-remount.test.ts :: CanvasSurfaceHost remount persistence persists reconciliation instead of leaving it as an in-memory repair
AssertionError: expected "vi.fn()" to be called with arguments: [ { spaceId: 'space', …(2) } ]

Number of calls: 0

    at Proxy.<anonymous> (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+expect@4.1.10/node_modules/@vitest/expect/dist/index.js:1487:10)
    at Proxy.<anonymous> (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+expect@4.1.10/node_modules/@vitest/expect/dist/index.js:1156:15)
    at Proxy.methodWrapper (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/chai@6.2.2/node_modules/chai/index.js:1700:25)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-surface-remount.test.ts:46:29
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:20
TypeError: Cannot destructure property 'type' of 'vnode' as it is null.
    at unmount (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6703:7)
    at unmountComponent (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6845:7)
    at unmount (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6736:7)
    at unmountComponent (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6845:7)
    at unmount (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6736:7)
    at unmountComponent (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6845:7)
    at unmount (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6736:7)
    at render (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6878:9)
    at Object.unmount (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:4367:11)
    at VueWrapper.unmount (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:7857:20)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-surface-remount.test.ts :: CanvasSurfaceHost remount persistence waits for the previous A flush across a keyed A to B to A remount
TypeError: Cannot destructure property 'type' of 'vnode' as it is null.
    at unmount (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6703:7)
    at unmountComponent (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6845:7)
    at unmount (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6736:7)
    at unmountComponent (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6845:7)
    at unmount (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6736:7)
    at unmountComponent (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6845:7)
    at unmount (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6736:7)
    at render (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6878:9)
    at Object.unmount (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:4367:11)
    at VueWrapper.unmount (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:7857:20)
TypeError: Cannot destructure property 'type' of 'vnode' as it is null.
    at unmount (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6703:7)
    at unmountComponent (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6845:7)
    at unmount (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6736:7)
    at unmountComponent (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6845:7)
    at unmount (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6736:7)
    at unmountComponent (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6845:7)
    at unmount (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6736:7)
    at render (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6878:9)
    at Object.unmount (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:4367:11)
    at VueWrapper.unmount (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:7857:20)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-surface-remount.test.ts :: CanvasSurfaceHost remount persistence waits for an in-flight formal commit before remounting the same space
TypeError: Cannot read properties of null (reading 'shapeFlag')
    at getRootNodes (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:7497:15)
    at getRootNodes (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:7502:16)
    at VueWrapper.getRootNodes (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:7732:16)
    at VueWrapper.attachNativeEventListener (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:7759:35)
    at new VueWrapper (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:7703:14)
    at Object.<anonymous> (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:7860:64)
    at createVueWrapper (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:1553:82)
    at /home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:7278:15
    at Array.map (<anonymous>)
    at VueWrapper.findAllComponents (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:7277:24)
TypeError: Cannot destructure property 'type' of 'vnode' as it is null.
    at unmount (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6703:7)
    at unmountComponent (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6845:7)
    at unmount (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6736:7)
    at unmountComponent (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6845:7)
    at unmount (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6736:7)
    at unmountComponent (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6845:7)
    at unmount (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6736:7)
    at render (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6878:9)
    at Object.unmount (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:4367:11)
    at VueWrapper.unmount (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:7857:20)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/clipboard-transfer.test.ts :: spaces clipboard round-trips the structured HTML marker while keeping useful plain text
TypeError: (0 , __vite_ssr_import_2__.isSpaceSectionPayload) is not a function
    at /home/runner/work/knovum/knovum/src/features/spaces/clipboard/transfer.ts:80:26
    at Array.filter (<anonymous>)
    at clipboardText (/home/runner/work/knovum/knovum/src/features/spaces/clipboard/transfer.ts:80:8)
    at writeSpaceClipboard (/home/runner/work/knovum/knovum/src/features/spaces/clipboard/transfer.ts:15:45)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/clipboard-transfer.test.ts:27:12
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/clipboard-transfer.test.ts :: spaces clipboard copies reactive node data without structured-clone failures
TypeError: (0 , __vite_ssr_import_2__.isSpaceSectionPayload) is not a function
    at /home/runner/work/knovum/knovum/src/features/spaces/clipboard/transfer.ts:80:26
    at Array.filter (<anonymous>)
    at clipboardText (/home/runner/work/knovum/knovum/src/features/spaces/clipboard/transfer.ts:80:8)
    at writeSpaceClipboard (/home/runner/work/knovum/knovum/src/features/spaces/clipboard/transfer.ts:15:45)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/clipboard-transfer.test.ts:44:12
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
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
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/konva-canvas-document.test.ts :: Konva canvas document synchronization creates canonical frames from domain nodes
TypeError: Cannot destructure property 'width' of '(0 , __vite_ssr_import_1__.spaceNodeSizePolicy)(...)' as it is undefined.
    at /home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
    at Array.map (<anonymous>)
    at appendEditorContent (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:35:18)
    at createCanvasDocument (/home/runner/work/knovum/knovum/src/features/spaces/canvas/document/canvas-document-sync.ts:50:10)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/konva-canvas-document.test.ts:32:22
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/konva-canvas-document.test.ts :: Konva canvas document synchronization adopts intrinsic media geometry until the persisted frame becomes authoritative
TypeError: (0 , __vite_ssr_import_1__.reconcileCanvasDocument) is not a function
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/konva-canvas-document.test.ts:50:23
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2305:20
    at new Promise (<anonymous>)
    at runWithTimeout (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2272:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2955:64
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/konva-canvas-document.test.ts :: Konva canvas document synchronization preserves authored frames while adding and removing domain nodes
TypeError: Cannot read properties of undefined (reading '0')
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/konva-canvas-document.test.ts:71:14
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2305:20
    at new Promise (<anonymous>)
    at runWithTimeout (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2272:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2955:64
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/konva-canvas-document.test.ts :: Konva canvas document synchronization rejects content authority on nodes without content-sized geometry
TypeError: Cannot read properties of undefined (reading '0')
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/konva-canvas-document.test.ts:83:14
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2305:20
    at new Promise (<anonymous>)
    at runWithTimeout (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2272:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2955:64
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/konva-canvas-document.test.ts :: Konva canvas document synchronization preserves document and layout identities when domain geometry is unchanged
TypeError: (0 , __vite_ssr_import_1__.reconcileCanvasDocument) is not a function
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/konva-canvas-document.test.ts:94:12
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2305:20
    at new Promise (<anonymous>)
    at runWithTimeout (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2272:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2955:64
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/konva-canvas-document.test.ts :: Konva canvas document synchronization keeps viewport changes made after an import when undoing imported content
TypeError: model.document.nodes is not iterable
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/konva-canvas-document.test.ts:107:27
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2305:20
    at new Promise (<anonymous>)
    at runWithTimeout (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2272:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2955:64
Failed to resolve import "../canvas/konva/document/canvas-layer-order" from "src/features/spaces/__tests__/konva-canvas-presentation-document.test.ts". Does the file exist?
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/konva-editor-port.test.ts :: Konva editor port routes clipboard operations through product ports and uses the last canvas pointer
TypeError: Cannot read properties of undefined (reading 'items')
    at indexStructure (/home/runner/work/knovum/knovum/src/features/spaces/model/structure/structure-index.ts:11:32)
    at /home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/selection/canvas-selection-deletion.ts:10:41
    at Object.cutSelection (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/create-canvas-editor-port.ts:49:25)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/konva-editor-port.test.ts:80:16
    at processTicksAndRejections (node:internal/process/task_queues:103:5)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/konva-editor-port.test.ts :: Konva editor port commits node and semantic relation deletion through one host transaction
TypeError: options.scopeGroupId is not a function
    at Object.snapshot [as getSelectionSnapshot] (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/create-canvas-editor-port.ts:37:27)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/konva-editor-port.test.ts:135:17
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2305:20
    at new Promise (<anonymous>)
    at runWithTimeout (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2272:10)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/konva-node-gesture-callbacks.test.ts :: Konva node gesture callbacks keeps event callbacks bound to their gesture controller
TypeError: this.options.document is not a function
    at beginDrag (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/interaction/node-gesture-controller.ts:41:20)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/konva-node-gesture-callbacks.test.ts:37:5
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2305:20
    at new Promise (<anonymous>)
    at runWithTimeout (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2272:10)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/konva-node-gesture.test.ts :: Konva node gestures moves a section and its domain members in one frame commit
TypeError: this.options.document is not a function
    at NodeGestureController.beginDrag (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/interaction/node-gesture-controller.ts:41:20)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/konva-node-gesture.test.ts:42:14
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2305:20
    at new Promise (<anonymous>)
    at runWithTimeout (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2272:10)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/konva-node-gesture.test.ts :: Konva node gestures rejects the complete drag when a selected operation root is locked
TypeError: this.options.document is not a function
    at NodeGestureController.beginDrag (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/interaction/node-gesture-controller.ts:41:20)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/konva-node-gesture.test.ts:75:21
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2305:20
    at new Promise (<anonymous>)
    at runWithTimeout (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2272:10)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/konva-node-gesture.test.ts :: Konva node gestures normalizes the native Transformer scale when the gesture ends
AssertionError: expected 1.5 to be 1 // Object.is equality
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/konva-node-gesture.test.ts:101:34
    at processTicksAndRejections (node:internal/process/task_queues:103:5)
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
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/konva-scene-projection.test.ts :: Konva scene projection always paints section backgrounds behind their members
TypeError: Cannot read properties of undefined (reading 'items')
    at indexStructure (/home/runner/work/knovum/knovum/src/features/spaces/model/structure/structure-index.ts:11:32)
    at indexStructureSnapshot (/home/runner/work/knovum/knovum/src/features/spaces/organization/structure/structure-context.ts:11:16)
    at projectCanvasScene (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/scene/scene-projection.ts:36:19)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/konva-scene-projection.test.ts:39:19
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2305:20
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/konva-scene-projection.test.ts :: Konva scene projection projects collapsed sections without mutating stored geometry or rendering members
TypeError: Cannot read properties of undefined (reading 'items')
    at indexStructure (/home/runner/work/knovum/knovum/src/features/spaces/model/structure/structure-index.ts:11:32)
    at indexStructureSnapshot (/home/runner/work/knovum/knovum/src/features/spaces/organization/structure/structure-context.ts:11:16)
    at projectCanvasScene (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/scene/scene-projection.ts:36:19)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/konva-scene-projection.test.ts:72:19
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2305:20
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/konva-scene-projection.test.ts :: Konva scene projection projects one identity for each semantic relation
TypeError: Cannot read properties of undefined (reading 'items')
    at indexStructure (/home/runner/work/knovum/knovum/src/features/spaces/model/structure/structure-index.ts:11:32)
    at indexStructureSnapshot (/home/runner/work/knovum/knovum/src/features/spaces/organization/structure/structure-context.ts:11:16)
    at projectCanvasScene (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/scene/scene-projection.ts:36:19)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/konva-scene-projection.test.ts:114:12
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2305:20
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/konva-scene-projection.test.ts :: Konva scene projection projects parallel relations onto stable distinct curve lanes
TypeError: Cannot read properties of undefined (reading 'items')
    at indexStructure (/home/runner/work/knovum/knovum/src/features/spaces/model/structure/structure-index.ts:11:32)
    at indexStructureSnapshot (/home/runner/work/knovum/knovum/src/features/spaces/organization/structure/structure-context.ts:11:16)
    at projectCanvasScene (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/scene/scene-projection.ts:36:19)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/konva-scene-projection.test.ts:150:23
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2305:20
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/konva-scene-projection.test.ts :: Konva scene projection projects external relations from collapsed members onto their section boundary
TypeError: Cannot read properties of undefined (reading 'items')
    at indexStructure (/home/runner/work/knovum/knovum/src/features/spaces/model/structure/structure-index.ts:11:32)
    at indexStructureSnapshot (/home/runner/work/knovum/knovum/src/features/spaces/organization/structure/structure-context.ts:11:16)
    at projectCanvasScene (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/scene/scene-projection.ts:36:19)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/konva-scene-projection.test.ts:205:25
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2305:20
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/konva-scene-projection.test.ts :: Konva scene projection keeps hidden layout data out of the rendered scene
TypeError: Cannot read properties of undefined (reading 'items')
    at indexStructure (/home/runner/work/knovum/knovum/src/features/spaces/model/structure/structure-index.ts:11:32)
    at indexStructureSnapshot (/home/runner/work/knovum/knovum/src/features/spaces/organization/structure/structure-context.ts:11:16)
    at projectCanvasScene (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/scene/scene-projection.ts:36:19)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/konva-scene-projection.test.ts:223:12
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2305:20
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/konva-selection.test.ts :: Konva canvas selection preserves a multi-selection when an already-selected node begins a drag
TypeError: options.document is not a function
    at setNodeIds (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/selection/use-canvas-selection.ts:90:41)
    at Object.selectNode (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/selection/use-canvas-selection.ts:40:5)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/konva-selection.test.ts:32:15
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2305:20
    at new Promise (<anonymous>)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/konva-selection.test.ts :: Konva canvas selection removes selected nodes that leave the projected scene
TypeError: options.document is not a function
    at Object.setNodeIds (/home/runner/work/knovum/knovum/src/features/spaces/canvas/konva/selection/use-canvas-selection.ts:90:41)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/konva-selection.test.ts:47:15
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2305:20
    at new Promise (<anonymous>)
    at runWithTimeout (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2272:10)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/manifest-parser.test.ts :: Space package manifest parser accepts regenerated canonical citations and detached historical snapshots
ZodError: [
  {
    "expected": "object",
    "code": "invalid_type",
    "path": [
      "canvasState",
      "document",
      "structure"
    ],
    "message": "Invalid input: expected object, received undefined"
  },
  {
    "expected": "object",
    "code": "invalid_type",
    "path": [
      "canvasState",
      "document",
      "layout"
    ],
    "message": "Invalid input: expected object, received undefined"
  },
  {
    "code": "unrecognized_keys",
    "keys": [
      "nodes"
    ],
    "path": [
      "canvasState",
      "document"
    ],
    "message": "Unrecognized key: \"nodes\""
  }
]
    at parseSpacePackageManifest (/home/runner/work/knovum/knovum/src/features/spaces/package/manifest-parser.ts:25:35)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/manifest-parser.test.ts:89:20
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2305:20
    at new Promise (<anonymous>)
    at runWithTimeout (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2272:10)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/manifest-parser.test.ts :: Space package manifest parser accepts All-Spaces threads with detached external citation snapshots
AssertionError: expected [Function] to not throw an error but '[\n  {\n    "expected": "object",\n  …' was thrown
    at Proxy.<anonymous> (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+expect@4.1.10/node_modules/@vitest/expect/dist/index.js:1552:16)
    at Proxy.<anonymous> (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+expect@4.1.10/node_modules/@vitest/expect/dist/index.js:1156:15)
    at Proxy.methodWrapper (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/chai@6.2.2/node_modules/chai/index.js:1700:25)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/manifest-parser.test.ts:100:92
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2305:20
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/manifest-parser.test.ts :: Space package manifest parser rejects a canonical citation whose snapshot revision is stale
AssertionError: expected [Function] to throw error matching /canonical citation revision/u but got '[\n  {\n    "expected": "object",\n  …'
    at Proxy.<anonymous> (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+expect@4.1.10/node_modules/@vitest/expect/dist/index.js:1552:16)
    at Proxy.<anonymous> (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+expect@4.1.10/node_modules/@vitest/expect/dist/index.js:1156:15)
    at Proxy.methodWrapper (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/chai@6.2.2/node_modules/chai/index.js:1700:25)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/manifest-parser.test.ts:107:88
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2305:20
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/manifest-parser.test.ts :: Space package manifest parser rejects derivations that reference unavailable nodes
AssertionError: expected [Function] to throw error matching /unavailable node/u but got '[\n  {\n    "expected": "object",\n  …'
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
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/node-content-height-batch.test.ts :: node content height batch fits text height in both directions without creating history
AssertionError: expected 220 to be 80 // Object.is equality
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/node-content-height-batch.test.ts:31:51
    at processTicksAndRejections (node:internal/process/task_queues:103:5)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/node-content-height-batch.test.ts :: node content height batch defers content layout while a direct resize owns the frame
AssertionError: expected 220 to be 96 // Object.is equality
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/node-content-height-batch.test.ts:79:51
    at processTicksAndRejections (node:internal/process/task_queues:103:5)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:20
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/node-size.test.ts :: spaces node size policy gives section a semantic desktop default
TypeError: Cannot read properties of undefined (reading 'width')
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/node-size.test.ts:44:28
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2025:60
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2305:20
    at new Promise (<anonymous>)
    at runWithTimeout (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2272:10)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/node-size.test.ts :: spaces node size policy uses the rendered compact geometry for collapsed sections
TypeError: Cannot read properties of undefined (reading 'resizeMode')
    at spaceNodeDimensions (/home/runner/work/knovum/knovum/src/features/spaces/canvas/node-size.ts:53:12)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/node-size.test.ts:77:7
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2305:20
    at new Promise (<anonymous>)
    at runWithTimeout (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2272:10)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/node-size.test.ts :: spaces node size policy keeps document and section geometry independently resizable
TypeError: Cannot convert undefined or null to object
    at Function.hasOwn (<anonymous>)
    at pick (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/es-toolkit@1.52.0/node_modules/es-toolkit/dist/object/pick.mjs:23:14)
    at spaceNodeResizePolicy (/home/runner/work/knovum/knovum/src/features/spaces/canvas/node-size.ts:71:10)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/node-size.test.ts:131:14
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2305:20
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/node.test.ts :: createSpaceNode rejects current section geometry in domain data
AssertionError: expected [Function] to throw error including 'Unrecognized keys' but got '[\n  {\n    "code": "invalid_union",\…'
    at Proxy.<anonymous> (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+expect@4.1.10/node_modules/@vitest/expect/dist/index.js:1552:16)
    at Proxy.<anonymous> (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+expect@4.1.10/node_modules/@vitest/expect/dist/index.js:1156:15)
    at Proxy.methodWrapper (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/chai@6.2.2/node_modules/chai/index.js:1700:25)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/node.test.ts:23:7
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2305:20
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/selection-actions.test.ts :: spaces organization allows a Section itself to be locked
Error: The Space editor is not ready for an organizational command
    at editor (/home/runner/work/knovum/knovum/src/features/spaces/organization/selection-actions.ts:8:23)
    at Object.setLocked (/home/runner/work/knovum/knovum/src/features/spaces/organization/selection-actions.ts:26:7)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/selection-actions.test.ts:52:24
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2305:20
    at new Promise (<anonymous>)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/selection-actions.test.ts :: deleteSelection removes selected relations before nodes and undoes both in one step
Error: The Space editor is not ready for an organizational command
    at editor (/home/runner/work/knovum/knovum/src/features/spaces/organization/selection-actions.ts:8:23)
    at Object.deleteSelection (/home/runner/work/knovum/knovum/src/features/spaces/organization/selection-actions.ts:28:7)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/selection-actions.test.ts:112:24
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2305:20
    at new Promise (<anonymous>)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/selection-actions.test.ts :: deleteSelection deletes a selected relation on its own
Error: The Space editor is not ready for an organizational command
    at editor (/home/runner/work/knovum/knovum/src/features/spaces/organization/selection-actions.ts:8:23)
    at Object.deleteSelection (/home/runner/work/knovum/knovum/src/features/spaces/organization/selection-actions.ts:28:7)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/selection-actions.test.ts:123:24
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2305:20
    at new Promise (<anonymous>)
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
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/space-session-actions.test.ts:85:3
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1889:40
    at runWithSuite (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2258:8)
    at Object.collect (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1889:10)
    at Object.collect (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1893:54)
    at processTicksAndRejections (node:internal/process/task_queues:103:5)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/space-session-actions.test.ts :: openSpace cancels a pending switch when the user returns to the active Space
AssertionError: expected false to be true // Object.is equality
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/space-session-actions.test.ts:122:33
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2305:20
    at new Promise (<anonymous>)
    at runWithTimeout (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2272:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2955:64
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/space-session-actions.test.ts :: draft Space creation reuses one session draft without persisting repeated new-Space actions
Error: STACK_TRACE_ERROR
    at task (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1784:27)
    at Object.<anonymous> (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1817:16)
    at Object.<anonymous> (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1563:28)
    at chain (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:599:14)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/space-session-actions.test.ts:136:3
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1889:40
    at runWithSuite (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2258:8)
    at Object.collect (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1889:10)
    at Object.collect (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1893:54)
    at processTicksAndRejections (node:internal/process/task_queues:103:5)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/space-session-actions.test.ts :: draft Space creation persists the draft exactly once before its first content write
Error: STACK_TRACE_ERROR
    at task (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1784:27)
    at Object.<anonymous> (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1817:16)
    at Object.<anonymous> (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1563:28)
    at chain (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:599:14)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/space-session-actions.test.ts:150:3
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1889:40
    at runWithSuite (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2258:8)
    at Object.collect (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1889:10)
    at Object.collect (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1893:54)
    at processTicksAndRejections (node:internal/process/task_queues:103:5)
/home/runner/work/knovum/knovum/src/features/spaces/__tests__/structure-contract.test.ts :: Space structure contract rejects parent cycles without recursive traversal
AssertionError: expected [Function] to throw error matching /cycle/u but got 'Sibling order must be unique and cont…'
    at Proxy.<anonymous> (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+expect@4.1.10/node_modules/@vitest/expect/dist/index.js:1552:16)
    at Proxy.<anonymous> (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+expect@4.1.10/node_modules/@vitest/expect/dist/index.js:1156:15)
    at Proxy.methodWrapper (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/chai@6.2.2/node_modules/chai/index.js:1700:25)
    at /home/runner/work/knovum/knovum/src/features/spaces/__tests__/structure-contract.test.ts:81:33
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:302:11
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:1903:26
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2326:20
    at new Promise (<anonymous>)
    at runWithCancel (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2323:10)
    at file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitest+runner@4.1.10/node_modules/@vitest/runner/dist/chunk-artifact.js:2305:20
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
```
