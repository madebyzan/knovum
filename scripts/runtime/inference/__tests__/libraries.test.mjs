import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";

import { listRuntimeFiles } from "../../files.mjs";
import { relocateLibraries } from "../libraries.mjs";

test(
  "Mach-O modules relocate their first dependency into their own supplier closure",
  {
    skip: process.platform !== "darwin",
  },
  () => {
    const root = mkdtempSync(join(tmpdir(), "knovum-inference-module-"));
    try {
      const supplier = join(root, "supplier"),
        runtime = join(root, "runtime");
      mkdirSync(supplier);
      writeFileSync(join(supplier, "base.c"), "int supplier_value(void) { return 1; }\n");
      writeFileSync(
        join(supplier, "module.c"),
        "extern int supplier_value(void); int module_value(void) { return supplier_value(); }\n",
      );
      for (const owner of ["bin", "compiled-libraries/whisper"]) {
        const directory = join(runtime, owner);
        mkdirSync(directory, { recursive: true });
        execFileSync("clang", [
          "-dynamiclib",
          join(supplier, "base.c"),
          "-o",
          join(directory, "libggml-base.0.dylib"),
          "-Wl,-install_name,@rpath/libggml-base.0.dylib",
        ]);
        execFileSync("clang", [
          "-bundle",
          join(supplier, "module.c"),
          join(directory, "libggml-base.0.dylib"),
          "-o",
          join(directory, "libggml-cpu-test.so"),
          `-Wl,-rpath,${supplier}`,
        ]);
      }
      relocateLibraries(runtime, listRuntimeFiles(runtime));
      for (const owner of ["bin", "compiled-libraries/whisper"]) {
        const module = join(runtime, owner, "libggml-cpu-test.so");
        const dependencies = execFileSync("otool", ["-L", module], { encoding: "utf8" });
        assert.match(dependencies, /@loader_path\/libggml-base\.0\.dylib/u);
        assert.doesNotMatch(
          dependencies.split("\n").slice(1).join("\n"),
          /@rpath|\.\.\/|knovum-inference-module-/u,
        );
        const commands = execFileSync("otool", ["-l", module], { encoding: "utf8" });
        assert(!commands.includes(supplier));
        execFileSync("codesign", ["--verify", "--strict", module]);
      }
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  },
);
