# Architecture

Source: `a9c6bb781622093bf6415df2bbbb5eecd1a5a013`. Exit code: 1.

```text
$ node tools/check-frontend-source-structure.mjs && node tools/check-ai-provider-catalog.mjs && depcruise --config .dependency-cruiser.cjs src
Frontend source structure gate failed (7):
- src/features/spaces/__tests__/canvas-surface-host.test.ts: 184 effective lines exceeds 180
- src/features/spaces/__tests__/konva-scene-projection.test.ts: 210 effective lines exceeds 180
- src/features/spaces/canvas/konva/use-konva-canvas-runtime.ts: 238 effective lines exceeds 180
- src/features/spaces/presentation/SpaceOutline.vue: 185 effective lines exceeds 180
- src/features/spaces/presentation/canvas/SpaceCanvas.vue: 194 effective lines exceeds 180
- src/features/spaces/presentation/knowledge/SpaceKnowledgeTypeManager.vue: 211 effective lines exceeds 180
- src/features/spaces/__tests__/capture-url-fixture.ts: files under __tests__ must use the exact .test.ts suffix
[ELIFECYCLE] Command failed with exit code 1.

```
