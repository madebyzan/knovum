import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

const root = new URL("../../src-tauri/", import.meta.url);
const readConfig = (name) => JSON.parse(readFileSync(new URL(name, root), "utf8"));
const base = readConfig("tauri.conf.json");
const macos = readConfig("tauri.macos.conf.json");
const windows = readConfig("tauri.windows.conf.json");
const manifest = readFileSync(new URL("Cargo.toml", root), "utf8");
// Fail closed if the checked-in inline dependency declaration changes shape.
// This deliberately is not a general TOML parser or Tauri configuration loader.
const declaration = manifest.match(/^tauri\s*=\s*\{[^\n]*features\s*=\s*(\[[^\]]*\])/m);
assert.ok(declaration, "Expected the shared inline tauri dependency and its explicit features");
const features = new Set(JSON.parse(declaration[1]));

for (const [platform, override] of [
  ["linux", {}],
  ["macos", macos],
  ["windows", windows],
]) {
  test(`${platform}: configuration matches shared Cargo-managed features`, () => {
    const privateApi = override.app?.macOSPrivateApi ?? base.app.macOSPrivateApi ?? false;
    const assetProtocol =
      override.app?.security?.assetProtocol?.enable ?? base.app.security.assetProtocol.enable;
    assert.equal(privateApi, features.has("macos-private-api"));
    assert.equal(assetProtocol, features.has("protocol-asset"));
  });
}

test("macOS keeps its native overlay and hidden startup", () => {
  const window = macos.app.windows.find((entry) => entry.label === "main");
  assert.ok(window);
  assert.equal(window.decorations, true);
  assert.equal(window.titleBarStyle, "Overlay");
  assert.equal(window.transparent, true);
  assert.equal(window.visible, false);
});

test("Windows keeps its native caption and supported configuration", () => {
  const window = windows.app.windows.find((entry) => entry.label === "main");
  assert.ok(window);
  assert.equal(window.decorations, true);
  assert.equal(window.visible, false);
  assert.equal(Object.hasOwn(window, "noRedirectionBitmap"), false);
});
