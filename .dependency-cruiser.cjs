const fs = require("node:fs");
const path = require("node:path");

const featuresDirectory = path.join(__dirname, "src/features");
const featureNames = fs
  .readdirSync(featuresDirectory, { withFileTypes: true })
  .filter((entry) => entry.isDirectory())
  .map((entry) => entry.name);

const featureIsolationRules = featureNames.map((featureName) => ({
  name: `feature-${featureName}-is-isolated`,
  severity: "error",
  comment: "A feature may depend on foundation/shared code, but never on another feature.",
  from: { path: `^src/features/${featureName}(?:/|$)` },
  to: { path: `^src/features/(?!${featureName}(?:/|$))` },
}));

module.exports = {
  forbidden: [
    {
      name: "no-circular",
      severity: "error",
      from: {},
      to: { circular: true },
    },
    {
      name: "renderer-has-no-sql-client",
      severity: "error",
      comment: "SQLite authority is Rust-only; WebView code may not import the Tauri SQL client.",
      from: { path: "^src(?:/|$)" },
      to: { path: "^@tauri-apps/plugin-sql$" },
    },
    {
      name: "shared-is-leaf",
      severity: "error",
      comment: "Shared contracts and UI cannot depend on application, shell, foundation, or feature code.",
      from: { path: "^src/shared(?:/|$)" },
      to: { path: "^src/(?:app|shell|foundation|features)(?:/|$)" },
    },
    {
      name: "foundation-is-independent",
      severity: "error",
      comment: "Foundation may use shared/vendor APIs, never application composition, shell, or features.",
      from: { path: "^src/foundation(?:/|$)" },
      to: { path: "^src/(?:app|shell|features)(?:/|$)" },
    },
    {
      name: "features-do-not-own-shell",
      severity: "error",
      comment: "Features consume foundation/shared contracts and cannot depend on app composition or shell.",
      from: { path: "^src/features(?:/|$)" },
      to: { path: "^src/(?:app|shell)(?:/|$)" },
    },
    {
      name: "shell-does-not-own-features",
      severity: "error",
      comment:
        "The shell renders module contracts supplied by the composition root; it never imports feature implementation.",
      from: { path: "^src/shell(?:/|$)" },
      to: { path: "^src/features(?:/|$)" },
    },
    {
      name: "shell-does-not-own-app",
      severity: "error",
      comment: "The shell is reusable workbench UI and cannot depend on the app composition root.",
      from: { path: "^src/shell(?:/|$)" },
      to: { path: "^src/app(?:/|$)" },
    },
    {
      name: "spaces-presentation-does-not-own-persistence",
      severity: "error",
      comment: "Space presentation consumes capability and query contracts, never persistence adapters.",
      from: { path: "^src/features/spaces/presentation(?:/|$)" },
      to: {
        path: "^src/features/spaces/(?:canvas/persistence|credentials/credential-repository|knowledge/(?:persistence|relations/relation-repository)|nodes/node-repository|scenes/scene-repository|search/global-search-repository|workspace/space-repository)(?:/|$|\\.ts$)",
      },
    },
    {
      name: "spaces-core-does-not-depend-on-presentation",
      severity: "error",
      comment: "Only the Space composition surfaces may import presentation components.",
      from: {
        path: "^src/features/spaces/(?!presentation(?:/|$)|__tests__(?:/|$)|SpaceView\\.vue$|workspace\\.ts$)",
      },
      to: { path: "^src/features/spaces/presentation(?:/|$)" },
    },
    {
      name: "spaces-model-is-independent",
      severity: "error",
      comment: "Space domain models cannot depend on session, infrastructure adapters, or presentation code.",
      from: { path: "^src/features/spaces/(?:model|knowledge/model)(?:/|$)" },
      to: {
        path: "^src/features/spaces/(?:session|presentation|canvas/persistence|knowledge/persistence)(?:/|$)",
      },
    },
    {
      name: "spaces-infrastructure-does-not-depend-on-presentation",
      severity: "error",
      comment: "Capability-owned repositories and persistence adapters cannot depend on Space UI.",
      from: {
        path: "^src/features/spaces/(?:canvas/persistence|credentials/credential-repository|knowledge/(?:persistence|relations/relation-repository)|nodes/node-repository|scenes/scene-repository|search/global-search-repository|workspace/space-repository)(?:/|$|\\.ts$)",
      },
      to: { path: "^src/features/spaces/presentation(?:/|$)" },
    },
    {
      name: "konva-is-confined-to-the-spaces-adapter",
      severity: "error",
      comment: "Only the Spaces Konva adapter may depend on Konva implementation packages.",
      from: { path: "^src/(?!features/spaces/canvas/konva(?:/|$))" },
      to: { path: "^(?:konva(?:/|$)|vue-konva(?:/|$))" },
    },
    {
      name: "spaces-business-does-not-depend-on-konva-implementation",
      severity: "error",
      comment: "Only the neutral canvas host may assemble the Konva implementation.",
      from: { path: "^src/features/spaces/(?!(?:canvas/host|canvas/konva|__tests__)(?:/|$))" },
      to: { path: "^src/features/spaces/canvas/konva(?:/|$)" },
    },
    {
      name: "canvas-host-only-assembles-the-konva-surface",
      severity: "error",
      comment: "The neutral host may assemble the adapter entry component, never its internals.",
      from: { path: "^src/features/spaces/canvas/host(?:/|$)" },
      to: {
        path: "^src/features/spaces/canvas/konva/(?!KonvaCanvasSurface[.]vue$)",
      },
    },
    {
      name: "konva-adapter-does-not-own-canvas-exchange",
      severity: "error",
      comment:
        "The Konva adapter implements neutral canvas editing and never parses or serializes exchange formats.",
      from: { path: "^src/features/spaces/canvas/konva(?:/|$)" },
      to: { path: "^src/features/spaces/canvas-exchange(?:/|$)" },
    },
    ...featureIsolationRules,
  ],
  options: {
    doNotFollow: { path: "node_modules" },
    exclude: "node_modules",
    tsConfig: { fileName: "tsconfig.json" },
    tsPreCompilationDeps: true,
    enhancedResolveOptions: {
      exportsFields: ["exports"],
      conditionNames: ["import", "module", "browser", "default"],
    },
  },
};
