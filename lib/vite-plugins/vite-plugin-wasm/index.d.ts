export declare type CrateConfig = {
    path: string;
    packageName?: string;
};
export declare type BuildCrateConfig = {
    baseDir: string;
    wasmPackPath: string;
    crate: CrateConfig;
    outDir: string;
    outName: string;
    quiet: boolean;
};
export declare type Config = {
    crates: CrateConfig[];
    outDir?: string;
    outName?: string;
    quiet?: boolean;
};
/**
 * Automatically builds any crates before Vite build
 */
export default function vitePluginWasmPack({ outName, outDir, crates, quiet, }: Config): {
    name: string;
    buildStart(): Promise<void>;
};
