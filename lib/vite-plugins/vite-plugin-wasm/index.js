import { spawn } from "child_process";
import path from "path";
import { existsSync, readFile, writeFile } from "fs";
import os from "os";
/**
 * Change the `name` attribute inside a `package.json` file.
 */
async function renamePackageJsonName({ baseDir, cratePath, cratePackageName, outDir, }) {
    const packagePath = path.join(baseDir, cratePath, outDir, "package.json");
    const packageIn = await new Promise((resolve, reject) => readFile(packagePath, { encoding: "utf-8" }, (error, data) => error ? reject(error) : resolve(data)));
    await new Promise((resolve, reject) => writeFile(packagePath, JSON.stringify({
        ...JSON.parse(packageIn),
        name: cratePackageName,
    }, null, 2), (error) => (error ? reject(error) : resolve(null))));
}
/**
 * Recursively searchs the wasm-pack binary path from a base directory
 */
function resolveWasmPackPath(binName, baseDir) {
    function search(pathParts) {
        if (!pathParts.length) {
            return null;
        }
        const [head, ...tail] = pathParts;
        const baseDir = [head, ...tail].reverse().join(path.sep);
        const wasmPackPath = path.join(baseDir, "node_modules", ".bin", binName);
        if (!existsSync(wasmPackPath)) {
            return search(tail);
        }
        return wasmPackPath;
    }
    return search(baseDir.split(path.sep).reverse());
}
function resolveWasmPackBinName() {
    switch (os.type().toLowerCase()) {
        case "linux":
        case "darwin": {
            return "wasm-pack";
        }
        case "windows_nt": {
            return "wasm-pack.CMD";
        }
        default: {
            return null;
        }
    }
}
/**
 * Builds a crate
 */
function buildCrate({ baseDir, wasmPackPath, crate, outDir, outName, quiet, }) {
    return new Promise((resolve, reject) => {
        const cmd = spawn(wasmPackPath, ["build", "--out-dir", outDir, "--out-name", outName, "--target", "web"], { cwd: path.join(baseDir, crate.path) });
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
                return reject(`Something went wrong, received code ${code} from wasm-pack`);
            }
            if (crate.packageName) {
                await renamePackageJsonName({
                    baseDir,
                    cratePath: crate.path,
                    cratePackageName: crate.packageName,
                    outDir,
                });
            }
            resolve(null);
        });
        cmd.on("error", reject);
    });
}
/**
 * Automatically builds any crates before Vite build
 */
export default function vitePluginWasmPack({ outName = "index", outDir = "pkg", crates, quiet = false, }) {
    const baseDir = process.cwd();
    return {
        name: "vite-plugin-wasm",
        async buildStart() {
            const wasmPackBinName = resolveWasmPackBinName();
            if (!wasmPackBinName) {
                // eslint-disable-next-line no-console
                console.error(`Unknown os type: ${os.type()}`);
                process.exit(1);
            }
            const wasmPackPath = resolveWasmPackPath(wasmPackBinName, baseDir);
            if (!wasmPackPath) {
                // eslint-disable-next-line no-console
                console.error("wasm-pack binary not found");
                process.exit(2);
            }
            // Build all crates concurrently
            try {
                await Promise.all(crates.map((crate) => buildCrate({ baseDir, crate, outDir, outName, quiet, wasmPackPath })));
            }
            catch (error) {
                // eslint-disable-next-line no-console
                console.error(error);
                process.exit(3);
            }
        },
    };
}
