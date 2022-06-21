import mkdirp from "mkdirp";
import path from "path";
import { generateAll } from "@lgn/api-codegen";

export type ApiConfig = {
  /** Root path of the apis definitions files (.yaml) */
  path: string;
  /** An array containing the api to generate the client(s) for */
  names: string[];
  /** Filename without prefix nor extension */
  filename: string;
};

export type Config = {
  /** Maps external references to TS namespaces */
  aliasMappings?: Record<string, string>;
  /** Path to a Prettier config file */
  prettierConfigPath?: string;
  /** Skips code formatting */
  skipFormat?: boolean;
  /** Api configurations */
  apiOptions: [ApiConfig, ...ApiConfig[]];
};

/**
 * Automatically generates TypeScript code from OpenAPI files before Vite build
 */
export default function vitePluginApiCodegen({
  prettierConfigPath,
  skipFormat,
  aliasMappings,
  apiOptions,
}: Config) {
  return {
    name: "@lgn/vite-plugin-api-codegen",
    async buildStart() {
      const outDir = path.resolve(process.cwd(), "node_modules", "@lgn/api");

      await mkdirp(outDir);

      generateAll({
        outDir,
        aliasMappings,
        prettierConfigPath,
        skipFormat,
        withPackageJson: true,
        apiOptions,
      });
    },
  };
}
