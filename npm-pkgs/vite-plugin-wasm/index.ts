import { spawn } from "child_process";
import path from "path";
import { stat } from "fs";
import { promisify } from "util";
import which from "which";

const targetFolder = "target";

const rustTarget = "wasm32-unknown-unknown";

export type CompileConfig = {
  name: string;
  release: boolean;
  quiet: boolean;
  cargoPath: string;
};

/**
 * Compiles the crates to wasm
 */
function compile({
  name,
  release,
  quiet,
  cargoPath,
}: CompileConfig): Promise<void> {
  return new Promise((resolve, reject) => {
    const cmd = spawn(cargoPath, [
      "build",
      "--target",
      "wasm32-unknown-unknown",
      "--package",
      name,
      ...(release ? ["--release"] : []),
    ]);

    cmd.stderr.setEncoding("utf-8");
    cmd.stdout.setEncoding("utf-8");

    if (!quiet) {
      // eslint-disable-next-line no-console
      cmd.stderr.on("data", console.error);
      // eslint-disable-next-line no-console
      cmd.stdout.on("data", console.log);
    }

    cmd.on("exit", async (code) => {
      if (code !== 0) {
        return reject(`Something went wrong, received code ${code} from cargo`);
      }

      resolve();
    });

    cmd.on("error", reject);
  });
}

export type GenerateConfig = {
  baseDir: string;
  packageName: string;
  name: string;
  quiet: boolean;
  release: boolean;
  // TODO: The root can easily be cased by looking for the pnpm-workspace.toml file recursively
  root: string;
  wasmBindgenPath: string;
};

/**
 * Generates js/ts code from a crate's wasm file
 */
async function generate({
  baseDir,
  packageName,
  name,
  quiet,
  release,
  root,
  wasmBindgenPath,
}: GenerateConfig) {
  const outDir = path.join(baseDir, "node_modules", packageName, "dist");

  const wasmPath = path.join(
    root,
    targetFolder,
    rustTarget,
    release ? "release" : "debug",
    `${name.replaceAll("-", "_")}.wasm`
  );

  try {
    await promisify(stat)(wasmPath);
  } catch {
    // eslint-disable-next-line no-console
    console.error(
      `Couldn't find "${wasmPath}", have you build the "${name}" crate in "${
        release ? "release" : "debug"
      }" mode?`
    );

    process.exit(4);
  }

  if (!quiet) {
    // eslint-disable-next-line no-console
    console.log(`Generating bindings from "${wasmPath}"`);
  }

  console.log("running command", wasmPath, outDir);

  return new Promise((resolve, reject) => {
    const cmd = spawn(wasmBindgenPath, [
      wasmPath,
      "--out-dir",
      outDir,
      "--target",
      "web",
    ]);

    cmd.stderr.setEncoding("utf-8");
    cmd.stdout.setEncoding("utf-8");

    if (!quiet) {
      // eslint-disable-next-line no-console
      cmd.stderr.on("data", console.error);
      // eslint-disable-next-line no-console
      cmd.stdout.on("data", console.log);
    }

    cmd.on("exit", async (code) => {
      if (code !== 0) {
        return reject(
          `Something went wrong, received code ${code} from wasm-bindgen`
        );
      }

      resolve(null);
    });

    cmd.on("error", reject);
  });
}

export type Config = {
  crates: Record<string, string>;
  quiet?: boolean;
  release?: boolean;
  root: string;
};

/**
 * Automatically generates binding from any crates before Vite builds
 */
export default function vitePluginWasmBindgen({
  crates,
  quiet = false,
  release = false,
  root,
}: Config) {
  return {
    name: "@lgn/vite-plugin-wasm",
    async buildStart() {
      const baseDir = process.cwd();

      let cargoPath: string;

      let wasmBindgenPath: string;

      try {
        [cargoPath, wasmBindgenPath] = await Promise.all([
          which("cargo"),
          which("wasm-bindgen"),
        ]);
      } catch {
        // eslint-disable-next-line no-console
        console.error("`cargo` or `wasm-bindgen` binary not found");

        process.exit(2);
      }

      // Build all crates concurrently
      try {
        await Promise.all(
          Object.entries(crates).map(([packageName, name]) =>
            compile({
              name,
              release,
              quiet,
              cargoPath,
            }).then(() =>
              generate({
                baseDir,
                packageName,
                name,
                quiet,
                release,
                root,
                wasmBindgenPath,
              })
            )
          )
        );
      } catch (error) {
        // eslint-disable-next-line no-console
        console.error(error);

        process.exit(3);
      }
    },
  };
}
