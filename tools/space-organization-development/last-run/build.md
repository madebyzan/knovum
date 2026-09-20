# Build

Source: `a5062d5bb521fe954dee8b5869f664bd088c77fe`. Exit code: 1.

```text
$ vite build
vite v8.2.2 building client environment for production...
transforming...
✓ 4064 modules transformed.
✗ Build failed in 3.04s
error during build:
Build failed with 1 error:

[plugin vite:vue] /home/runner/work/knovum/knovum/src/features/spaces/presentation/canvas/SpaceCanvasSceneControls.vue:119:9
RolldownError: Duplicate attribute.
SyntaxError: Duplicate attribute.
    at createCompilerError (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+compiler-core@3.5.42/node_modules/@vue/compiler-core/dist/compiler-core.cjs.prod.js:1374:17)
    at emitError (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+compiler-core@3.5.42/node_modules/@vue/compiler-core/dist/compiler-core.cjs.prod.js:3001:5)
    at Object.onattribnameend (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+compiler-core@3.5.42/node_modules/@vue/compiler-core/dist/compiler-core.cjs.prod.js:2516:7)
    at Tokenizer.handleAttrNameEnd (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+compiler-core@3.5.42/node_modules/@vue/compiler-core/dist/compiler-core.cjs.prod.js:926:14)
    at Tokenizer.stateInDirName (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+compiler-core@3.5.42/node_modules/@vue/compiler-core/dist/compiler-core.cjs.prod.js:877:12)
    at Tokenizer.parse (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+compiler-core@3.5.42/node_modules/@vue/compiler-core/dist/compiler-core.cjs.prod.js:1121:16)
    at Object.baseParse (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+compiler-core@3.5.42/node_modules/@vue/compiler-core/dist/compiler-core.cjs.prod.js:3033:13)
    at Object.parse (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+compiler-dom@3.5.42/node_modules/@vue/compiler-dom/dist/compiler-dom.cjs.prod.js:668:23)
    at Object.parse$1 [as parse] (/home/runner/work/knovum/knovum/node_modules/.pnpm/@vue+compiler-sfc@3.5.42/node_modules/@vue/compiler-sfc/dist/compiler-sfc.cjs.js:1849:24)
    at createDescriptor (file:///home/runner/work/knovum/knovum/node_modules/.pnpm/@vitejs+plugin-vue@6.0.8_vite@8.2.2_@types+node@26.4.1_esbuild@0.28.2_jiti@2.7.0_tsx@4._e46765f1afb4f763aed643588ff41fe4/node_modules/@vitejs/plugin-vue/dist/index.mjs:50:42)
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
