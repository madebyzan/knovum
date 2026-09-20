# Build

Source: `5677b1f49ad542d76ff63cf81b259e54de4ec23b`. Exit code: 1.

```text
$ vite build
vite v8.2.2 building client environment for production...
transforming...
✓ 4050 modules transformed.
rendering chunks...
✗ Build failed in 3.62s
error during build:
Build failed with 3 errors:

[MISSING_EXPORT] "structureContentNodeIds" is not exported by "src/features/spaces/organization/structure/structure-selection.ts".
   ╭─[ src/features/spaces/canvas/konva/scene/scene-projection.ts:3:10 ]
   │
 3 │ import { structureContentNodeIds, structureSelectionTarget } from "../../../organization/structure/structure-selection";
   │          ───────────┬───────────  
   │                     ╰───────────── Missing export
───╯

[MISSING_EXPORT] "structureContentNodeIds" is not exported by "src/features/spaces/organization/structure/structure-selection.ts".
   ╭─[ src/features/spaces/canvas/konva/create-canvas-editor-port.ts:2:35 ]
   │
 2 │ import { structureOperationRoots, structureContentNodeIds } from "../../organization/structure/structure-selection";
   │                                   ───────────┬───────────  
   │                                              ╰───────────── Missing export
───╯

[MISSING_EXPORT] "structureContentNodeIds" is not exported by "src/features/spaces/organization/structure/structure-selection.ts".
   ╭─[ src/features/spaces/canvas/konva/selection/canvas-selection-deletion.ts:2:10 ]
   │
 2 │ import { structureContentNodeIds } from "../../../organization/structure/structure-selection";
   │          ───────────┬───────────  
   │                     ╰───────────── Missing export
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
