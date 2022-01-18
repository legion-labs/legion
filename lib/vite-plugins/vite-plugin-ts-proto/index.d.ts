export declare type ModuleConfig = {
    /**
     * Name (similar to the `"name"` attribute in the `package.json` file)
     * of the node module to generate proto files from and into.
     */
    name: string;
    /** A glob that matches all the proto files that must be included/excluded */
    glob: string;
};
export declare type Config = {
    /** An array containing the proto module configurations */
    modules: ModuleConfig[];
    /** No output if `quiet` is `true`, `false` by default */
    quiet?: boolean;
};
/**
 * Automatically generates TypeScript code from proto before Vite build
 */
export default function vitePluginProtoPack({ modules, quiet, }: Config): {
    name: string;
    buildStart(): Promise<void>;
};
