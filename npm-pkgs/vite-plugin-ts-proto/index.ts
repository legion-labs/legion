import { spawn } from "child_process";
import path from "path";
import glob from "glob";
import which from "which";
import mkdirp from "mkdirp";
import os from "os";
import { existsSync } from "fs";

/**
 * Recursively searchs the ts-proto binary path from a base directory
 */
function resolveTsProtoPath(binName: string, baseDir: string) {
  function search(pathParts: string[]): string | null {
    if (!pathParts.length) {
      return null;
    }

    const [head, ...tail] = pathParts;

    const baseDir = [head, ...tail].reverse().join(path.sep);

    const tsProtoPath = path.join(baseDir, "node_modules", ".bin", binName);

    if (!existsSync(tsProtoPath)) {
      return search(tail);
    }

    return tsProtoPath;
  }

  return search(baseDir.split(path.sep).reverse());
}

function resolveTsProtoBinName(): string | null {
  switch (os.type().toLowerCase()) {
    case "linux":
    case "darwin": {
      return "protoc-gen-ts_proto";
    }

    case "windows_nt": {
      return "protoc-gen-ts_proto.CMD";
    }

    default: {
      return null;
    }
  }
}

export type ModuleConfig = {
  /**
   * Name (similar to the `"name"` attribute in the `package.json` file)
   * of the node module to generate proto files from and into.
   */
  name: string;
  // TODO: Support array of strings
  /** A glob that matches all the proto files that must be included/excluded */
  glob: string;
};

export type Config = {
  /** An array containing the proto module configurations */
  modules: ModuleConfig[];
  /** No output if `quiet` is `true`, `false` by default */
  quiet?: boolean;
};

async function generateProtoForModule(
  module: { name: string; glob: string },
  {
    baseDir,
    protocPath,
    tsProtoPath,
    quiet,
  }: {
    baseDir: string;
    protocPath: string;
    tsProtoPath: string;
    quiet: boolean;
  }
) {
  const cwd = path.resolve(baseDir, "node_modules", module.name);
  const protos = path.resolve(cwd, "protos");

  const globMatches = await new Promise<string[]>((resolve, reject) =>
    glob(module.glob, { cwd: protos }, (error, matches) =>
      error ? reject(error) : resolve(matches)
    )
  );

  if (!globMatches) {
    console.warn(
      `No proto files matching glob ${module.glob} for ${module.name}`
    );

    return;
  }

  const tsProtoOut = path.resolve(cwd, "dist");

  await mkdirp(tsProtoOut);
  const args = [
    `--plugin=${tsProtoPath}`,
    `--ts_proto_out=${tsProtoOut}`,
    //`--proto_path=${cwd}`,
    `--proto_path=${protos}`,
    // TODO: All the options are hardcoded for now but it's easy to expose them via the `ModuleConfig` type
    "--ts_proto_opt=esModuleInterop=true",
    "--ts_proto_opt=outputClientImpl=grpc-web",
    "--ts_proto_opt=env=browser",
    "--ts_proto_opt=lowerCaseServiceMethods=true",
    ...globMatches,
  ];

  return new Promise((resolve, reject) => {
    const cmd = spawn(protocPath, args, { cwd });

    cmd.stderr.setEncoding("utf-8");
    cmd.stdout.setEncoding("utf-8");

    if (!quiet) {
      // eslint-disable-next-line no-console
      cmd.stderr.on("data", console.error);
      // eslint-disable-next-line no-console
      cmd.stdout.on("data", console.log);
    }

    cmd.on("exit", async () => {
      resolve(null);
    });

    cmd.on("error", reject);
  });
}

/**
 * Automatically generates TypeScript code from proto before Vite build
 */
export default function vitePluginProtoPack({
  modules,
  quiet = false,
}: Config) {
  return {
    name: "@lgn/vite-plugin-ts-proto",
    async buildStart() {
      const baseDir = process.cwd();

      const protocPath = await new Promise<string | undefined>(
        (resolve, reject) =>
          which("protoc", (error, path) =>
            error ? reject(error) : resolve(path)
          )
      );

      if (!protocPath) {
        // eslint-disable-next-line no-console
        console.error("`protoc` binary not found");

        process.exit(1);
      }

      const tsProtoBinName = resolveTsProtoBinName();

      if (!tsProtoBinName) {
        // eslint-disable-next-line no-console
        console.error(`Unknown os type: ${os.type()}`);

        process.exit(2);
      }

      const tsProtoPath = resolveTsProtoPath(tsProtoBinName, baseDir);

      if (!tsProtoPath) {
        // eslint-disable-next-line no-console
        console.error("ts-proto binary not found in node_modules");

        process.exit(3);
      }

      await Promise.all(
        modules.map((module) =>
          generateProtoForModule(module, {
            baseDir,
            protocPath,
            tsProtoPath,
            quiet,
          })
        )
      );
    },
  };
}
